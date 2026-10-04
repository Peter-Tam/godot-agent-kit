#!/usr/bin/env python3
"""Run existing real-editor acceptance in the owned local Tart macOS VM only."""
from __future__ import annotations

import argparse
from contextlib import contextmanager
from datetime import datetime, timezone
import fcntl
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
import threading
import uuid

from vm_tart import DEFAULT_IMAGE, GUEST_PYTHON, GUEST_ROOT, Tart, VMError

_REPO = Path(__file__).resolve().parents[2]
_STATE = Path.home() / ".local/state/godot-agent-kit-vm"
_SUITES = ("observation", "edit", "open", "discovery", "close")
_GODOT_SHA = "c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf"
_TEMPLATE_SHA = "88df5e2e6fee99088699be66e6d42e4da4fb0c5619d054297d755a49558a4792"
_RUN_ID = re.compile(r"[A-Za-z0-9][A-Za-z0-9_-]{0,79}\Z")


def _digest(path):
    value = hashlib.sha256()
    with Path(path).open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            value.update(block)
    return value.hexdigest()


def _git(repo, *args):
    result = subprocess.run(["git", "-C", str(repo), *args], capture_output=True)
    if result.returncode:
        raise VMError("Source selection failed: " + result.stderr.decode(errors="replace").strip())
    return result.stdout


def _revision(repo, requested):
    if requested is None and _git(repo, "status", "--porcelain", "--untracked-files=normal"):
        raise VMError("Uncommitted source: commit it first, or explicitly select --revision <commit>. "
                      "Only the selected committed tree is copied; never the host working tree.")
    revision = _git(repo, "rev-parse", "--verify", "--end-of-options",
                    (requested or "HEAD") + "^{commit}").decode().strip()
    if not re.fullmatch(r"[0-9a-f]{40}", revision):
        raise VMError("Source must resolve to one full Git commit.")
    return revision


def _run_id(value):
    if not _RUN_ID.fullmatch(value):
        raise argparse.ArgumentTypeError("run identity must be 1–80 letters, digits, underscores or hyphens")
    return value


def _identity():
    return datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ-") + uuid.uuid4().hex[:12]


@contextmanager
def _locked(state):
    state.mkdir(parents=True, exist_ok=True, mode=0o700)
    if state.is_symlink() or state.stat().st_uid != os.getuid() or state.stat().st_mode & 0o077:
        raise VMError("VM state must be an owned, nonsymlink mode-0700 directory.")
    with (state / ".operation.lock").open("a") as lock:
        try:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError as error:
            raise VMError("Another VM control operation is active; do not race setup, runs or reset.") from error
        yield


def _install_worker(tart):
    # A restored base may carry an old helper. Pin each operation to immutable
    # bytes so upgrades/interruptions never rewrite another active worker.
    payload = Path(__file__).with_name("vm_guest.py").read_bytes()
    identity = hashlib.sha256(payload).hexdigest()
    destination = GUEST_ROOT + "/workers/" + identity + ".py"
    with tempfile.NamedTemporaryFile(prefix="gak-vm-worker-") as snapshot:
        snapshot.write(payload)
        snapshot.flush()
        tart.upload(Path(snapshot.name), destination)
    return destination, identity


def _worker(tart, worker, *args, **kwargs):
    return tart.exec([GUEST_PYTHON, worker, *args], **kwargs)


def _sync(tart, repo, revision, worker):
    # Archive uses committed tracked files only, excluding host binaries, secrets and projects.
    with tempfile.TemporaryDirectory(prefix="gak-vm-source-") as temporary:
        archive = Path(temporary) / "source.tar"
        _git(repo, "archive", "--format=tar", "--output=" + str(archive), revision)
        with archive.open("rb") as stream:
            result = _worker(tart, worker, "sync", "--revision", revision,
                             "--archive-sha256", _digest(archive), stdin=stream, timeout=180)
        if result.stdout:
            print(result.stdout.decode(errors="replace").strip())


def _extract(archive, destination):
    """Accept regular evidence files only; never restore links/devices/permissions."""
    destination.mkdir(mode=0o700, parents=True, exist_ok=False)
    with tarfile.open(archive, "r:*") as source:
        seen = set()
        for member in source:
            name = PurePosixPath(member.name)
            if (name.is_absolute() or ".." in name.parts or not name.parts
                    or member.name in seen or not (member.isfile() or member.isdir())):
                raise VMError("Unsafe guest evidence archive entry: " + member.name)
            seen.add(member.name)
            target = destination.joinpath(*name.parts)
            if member.isdir():
                target.mkdir(mode=0o700, parents=True, exist_ok=True)
                continue
            target.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
            with source.extractfile(member) as incoming, target.open("xb") as outgoing:
                shutil.copyfileobj(incoming, outgoing)
            target.chmod(0o600)


def _fetch(tart, state, run_id, captures, worker):
    # Every retrieval is immutable; do not overwrite a previous failure or partial copy.
    root = state / "artifacts" / run_id
    root.mkdir(mode=0o700, parents=True, exist_ok=True)
    destination = root / _identity()
    with tempfile.TemporaryDirectory(prefix=".fetch-", dir=root) as temporary:
        archive = Path(temporary) / "evidence.tar"
        with archive.open("wb") as stream:
            _worker(tart, worker, "export", "--run-id", run_id,
                    *(["--captures"] if captures else []), stdout=stream, timeout=180)
        _extract(archive, destination)
    print("VM evidence: " + str(destination))
    return destination


def _host_godot_processes():
    # Read process executable names, not unrelated host command arguments or window contents.
    result = subprocess.run(["/bin/ps", "-axo", "pid=,ucomm="], capture_output=True, check=True, text=True)
    matches = []
    for line in result.stdout.splitlines():
        fields = line.strip().split(None, 1)
        if len(fields) != 2:
            continue
        executable = fields[1]
        if executable.lower().startswith("godot"):
            matches.append(line.strip())
    return matches


class _HostObservation:
    """Sample process absence; the no-viewer VM boundary, not sampling, isolates focus."""
    def __init__(self):
        self.stop = threading.Event()
        self.matches = set()
        self.samples = 0
        self.error = None
        self.thread = threading.Thread(target=self._watch, daemon=True)

    def _watch(self):
        try:
            while True:
                self.matches.update(_host_godot_processes())
                self.samples += 1
                if self.stop.wait(0.25):
                    return
        except (OSError, subprocess.SubprocessError) as error:
            self.error = str(error)

    def __enter__(self):
        self.baseline = _host_godot_processes()
        self.thread.start()
        return self

    def __exit__(self, *_):
        self.stop.set()
        self.thread.join()

    def evidence(self):
        return {"host_godot_processes": sorted(self.matches), "baseline_host_godot_processes": self.baseline,
                "new_host_godot_processes": sorted(self.matches.difference(self.baseline)),
                "process_samples": self.samples,
                "sampling_error": self.error, "sampling_is_not_focus_proof": True,
                "execution_boundary": "Tart macOS guest, no viewer or shared directories",
                "host_focus_activation_requested": False}


def _execute(tart, state, args, repo=_REPO):
    vm_status = tart.status()
    if not vm_status["running"]:
        raise VMError("Owned VM is missing or stopped; use setup/start before running acceptance.")
    if not vm_status.get("startup_readiness"):
        raise VMError("Owned VM has not completed startup readiness; use start before running acceptance.")
    revision = _revision(repo, args.revision)
    run_id = args.run_id or _identity()
    wrapper_hash = _digest(__file__)
    worker, worker_hash = _install_worker(tart)
    print(f"VM run {run_id}; exact source {revision}", flush=True)
    _sync(tart, repo, revision, worker)
    command = [args.operation, "--revision", revision, "--run-id", run_id, "--suite", args.suite]
    if args.operation == "run":
        command += ["--scenario", args.scenario]
    else:
        if args.resume:
            command.append("--resume")
        if args.keep_going:
            command.append("--keep-going")
    host_run = state / "artifacts" / run_id
    host_run.mkdir(mode=0o700, parents=True, exist_ok=True)
    invocation = _identity()
    status = 1
    with _HostObservation() as observation:
        with (host_run / (invocation + ".stdout")).open("wb") as stream:
            result = _worker(tart, worker, *command, stdout=stream, check=False)
        status = result.returncode
        if result.stderr:
            print(result.stderr.decode(errors="replace"), file=sys.stderr, end="")
    evidence = {"revision": revision, "run_id": run_id, "runner_exit_code": status,
                "host_wrapper_sha256": wrapper_hash,
                "guest_worker_sha256": worker_hash,
                "vm": tart.status(), **observation.evidence()}
    (host_run / (invocation + ".host.json")).write_text(json.dumps(evidence, indent=2) + "\n")
    try:
        _fetch(tart, state, run_id, args.captures, worker)
    except (VMError, OSError, tarfile.TarError, subprocess.SubprocessError) as error:
        print(f"Evidence retrieval failed: {error}; retained in guest as {run_id}. "
              "Use fetch-artifacts after restoring control.", file=sys.stderr)
        if status == 0:
            status = 1
    if observation.matches.difference(observation.baseline) or observation.error:
        print("Host process observation needs attribution; inspect the isolation record. "
              "Independent host applications were not controlled or terminated.", file=sys.stderr)
    return status if status >= 0 else 128 - status


def _setup(tart, state, args):
    revision = _revision(_REPO, args.revision)
    app = args.godot_app.resolve(strict=True)
    template = args.export_template.resolve(strict=True)
    if _digest(app / "Contents/MacOS/Godot") != _GODOT_SHA or _digest(template) != _TEMPLATE_SHA:
        raise VMError("Exact official Godot executable/template SHA-256 mismatch; no host Godot was executed.")
    if not tart.status()["running"]:
        tart.setup(DEFAULT_IMAGE)
    tart.start(bootstrap=True)
    # The base image contains Homebrew/CLT and the GUI guest agent, but not all images have Python.
    probe = tart.exec([GUEST_PYTHON, "-c", "import sys; sys.exit(sys.version_info < (3, 10))"],
                      check=False, timeout=30)
    if probe.returncode:
        tart.exec(["/usr/bin/env", "HOMEBREW_NO_ANALYTICS=1", "HOMEBREW_NO_AUTO_UPDATE=1",
                   "/opt/homebrew/bin/brew", "install", "python"], timeout=1200)
    tart.exec(["/bin/mkdir", "-p", GUEST_ROOT + "/inputs"], timeout=30)
    tart.exec(["/bin/chmod", "700", GUEST_ROOT], timeout=30)
    with tempfile.TemporaryDirectory(prefix="gak-vm-engine-") as temporary:
        archive = Path(temporary) / "engine.tar"
        with tarfile.open(archive, "w") as stream:
            stream.add(app, arcname="Godot.app")
        with archive.open("rb") as stream:
            tart.exec(["/usr/bin/tar", "-xf", "-", "-C", GUEST_ROOT + "/inputs"], stdin=stream, timeout=180)
    template_dir = "/Users/admin/Library/Application Support/Godot/export_templates/4.7.2.stable"
    tart.exec(["/bin/mkdir", "-p", template_dir], timeout=30)
    tart.upload(template, template_dir + "/macos.zip")
    worker, _ = _install_worker(tart)
    result = _worker(tart, worker, "provision", "--generation", uuid.uuid4().hex, timeout=1800)
    print(result.stdout.decode(errors="replace"))
    _sync(tart, _REPO, revision, worker)
    tart.exec(["/usr/bin/env", "-i", "HOME=/Users/admin", "RUSTUP_AUTO_INSTALL=0",
               "PATH=/Users/admin/.cargo/bin:/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin",
               "/Users/admin/.cargo/bin/cargo", "+1.98.1", "fetch", "--locked",
               "--manifest-path", GUEST_ROOT + "/workspace/repo/mcp-server/Cargo.toml"], timeout=600)
    tart.stop()
    tart.save_base()
    print("Provisioned base saved. Use start for normal host-only execution.")


def _mcp_config(repo, run_id):
    return {"mcpServers": {"godot_agent_kit": {
        "type": "stdio", "command": "python3",
        "args": [str(repo / "godot-addon/tests/run_in_vm.py"), "mcp-stdio", "--run-id", run_id],
        "cwd": str(repo), "timeout": 15000, "instructions": False}}}


def _prepare_mcp(tart, state, args, repo=_REPO):
    status = tart.status()
    if not status["running"] or not status.get("startup_readiness"):
        raise VMError("Owned VM must be running and ready before MCP preparation.")
    revision = _revision(repo, args.revision)
    destination = args.artifacts.expanduser().absolute()
    if (not destination.is_dir() or destination.is_symlink() or list(destination.iterdir())
            or destination.stat().st_uid != os.getuid() or destination.stat().st_mode & 0o077):
        raise VMError("MCP artifacts must be an empty owned private directory.")
    worker, worker_hash = _install_worker(tart)
    _sync(tart, repo, revision, worker)
    result = _worker(tart, worker, "prepare-mcp", "--revision", revision,
                     "--run-id", args.run_id, "--profile", args.profile, timeout=1200)
    prepared = json.loads(result.stdout)
    receipt = prepared["receipt"]
    if receipt["revision"] != revision:
        raise VMError("MCP preparation revision mismatch.")
    config = _mcp_config(repo.resolve(), args.run_id)
    if state != _STATE:
        config["mcpServers"]["godot_agent_kit"]["args"][1:1] = ["--state", str(state)]
    content = json.dumps(config, indent=2) + "\n"
    local = dict(run_id=args.run_id, revision=revision, guest=receipt, vm=status,
                 worker=worker, worker_sha256=worker_hash, host_checkout=str(repo.resolve()),
                 host_config_sha256=hashlib.sha256(content.encode()).hexdigest())
    owned = state / "prepared" / args.run_id
    owned.mkdir(mode=0o700, parents=True, exist_ok=False)
    for path, text in ((owned / "receipt.json", json.dumps(local, indent=2) + "\n"),
                       (destination / "receipt.json", json.dumps(local, indent=2) + "\n"),
                       (destination / "mcp.json", content),
                       (destination / "prompt.txt", prepared["prompt"])):
        with path.open("x") as stream:
            stream.write(text)
        path.chmod(0o600)
    print("Prepared owned MCP run " + args.run_id + "; private artifacts: " + str(destination))
    return 0


def _mcp_control(tart, state, args):
    owned = state / "prepared" / args.run_id
    if owned.is_symlink():
        raise VMError("Unsafe prepared run receipt.")
    receipt = json.loads((owned / "receipt.json").read_text())
    worker = receipt["worker"]
    if worker != GUEST_ROOT + "/workers/" + receipt["worker_sha256"] + ".py":
        raise VMError("Prepared worker identity mismatch.")
    if args.operation == "mcp-stdio":
        result = _worker(tart, worker, "mcp-stdio", "--run-id", args.run_id,
                         stdin=sys.stdin.buffer, stdout=sys.stdout.buffer, check=False)
    else:
        result = _worker(tart, worker, "finalize-mcp", "--run-id", args.run_id, check=False, timeout=200)
        _fetch(tart, state, args.run_id, True, worker)
    if result.stderr:
        print("Owned MCP relay/control failed." if result.returncode else "Owned MCP relay diagnostic.",
              file=sys.stderr)
    return result.returncode


def _parser():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--state", type=Path, default=_STATE, help="private local state directory")
    commands = parser.add_subparsers(dest="operation", required=True)
    setup = commands.add_parser("setup", help="clone/provision the owned VM; never execute host Godot")
    setup.add_argument("--godot-app", type=Path, required=True)
    setup.add_argument("--export-template", type=Path, required=True)
    setup.add_argument("--revision", help="commit whose locked build dependencies are cached in the base")
    start = commands.add_parser("start", help="start without a host viewer, default host-only network")
    start.add_argument("--bootstrap-network", action="store_true", help="explicit temporary NAT for dependency downloads")
    for name in ("status", "stop", "base"):
        commands.add_parser(name)
    reset = commands.add_parser("reset", help="destroy only this stopped test VM and restore its saved base")
    reset.add_argument("--discard-guest-runs", action="store_true", required=True,
                       help="explicitly acknowledge deletion of guest workspaces and evidence")
    for operation in ("run", "campaign"):
        run = commands.add_parser(operation)
        run.add_argument("suite", choices=(*_SUITES, "all") if operation == "campaign" else (*_SUITES, "mcp"))
        run.add_argument("--revision", help="committed revision; default HEAD requires a clean worktree")
        run.add_argument("--run-id", type=_run_id)
        run.add_argument("--captures", action="store_true", help="retrieve private source-bearing guest screenshots too")
        if operation == "run":
            run.add_argument("--scenario", required=True, help="unchanged existing runner scenario name")
        else:
            run.add_argument("--resume", action="store_true")
            run.add_argument("--keep-going", action="store_true")
    prepare = commands.add_parser("prepare-mcp")
    prepare.add_argument("--revision", required=True)
    prepare.add_argument("--run-id", required=True, type=_run_id)
    prepare.add_argument("--artifacts", required=True, type=Path)
    prepare.add_argument("--profile", choices=("workflow", "sources", "bound", "observations"), default="workflow")
    for name in ("mcp-stdio", "finalize-mcp"):
        control = commands.add_parser(name)
        control.add_argument("--run-id", required=True, type=_run_id)
    fetch = commands.add_parser("fetch-artifacts")
    fetch.add_argument("run_id", type=_run_id)
    fetch.add_argument("--captures", action="store_true")
    return parser


def main(argv=None):
    parser = _parser()
    args = parser.parse_args(argv)
    if args.operation == "campaign" and args.resume and not args.run_id:
        parser.error("--resume requires the original --run-id")
    state = args.state.expanduser().absolute()
    try:
        with _locked(state):
            tart = Tart(state)
            if args.operation == "prepare-mcp":
                return _prepare_mcp(tart, state, args)
            if args.operation in ("mcp-stdio", "finalize-mcp"):
                return _mcp_control(tart, state, args)
            if args.operation == "setup":
                _setup(tart, state, args)
            elif args.operation == "start":
                tart.start(bootstrap=args.bootstrap_network)
            elif args.operation == "status":
                status = tart.status()
                print(json.dumps(status, indent=2))
            elif args.operation == "stop":
                tart.stop()
            elif args.operation == "base":
                tart.save_base()
            elif args.operation == "reset":
                tart.reset()
            elif args.operation == "fetch-artifacts":
                worker, _ = _install_worker(tart)
                _fetch(tart, state, args.run_id, args.captures, worker)
            else:
                return _execute(tart, state, args)
        return 0
    except (VMError, OSError, ValueError, tarfile.TarError, subprocess.SubprocessError) as error:
        print(f"VM execution refused: {error}\nNo host Godot fallback. See .github/LOCAL_VM.md.", file=sys.stderr)
        if isinstance(error, subprocess.CalledProcessError) and error.stderr:
            detail = error.stderr.decode(errors="replace") if isinstance(error.stderr, bytes) else error.stderr
            print(detail.strip(), file=sys.stderr)
        return 1
    except KeyboardInterrupt:
        print("VM control interrupted; guest work may still be active. Inspect status and fetch-artifacts; "
              "do not assume cancellation or a passing result. No host fallback.", file=sys.stderr)
        return 130


if __name__ == "__main__":
    raise SystemExit(main())
