"""Dedicated Tart 2.40.1 control plane; guest commands use local vsock only."""
from __future__ import annotations

import fcntl
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import stat
import struct
import subprocess
import time
import uuid

VM_NAME = "godot-agent-kit-live-editor"
BASE_NAME = VM_NAME + "-base"
GUEST_ROOT = "/Users/admin/.godot-agent-kit-vm"
GUEST_PYTHON = "/opt/homebrew/bin/python3"
DEFAULT_IMAGE = "ghcr.io/cirruslabs/macos-tahoe-base@sha256:1b093499716409d29e8b5336844528e1cae375db97d2ad8e5aeff78cf0da201e"
BACKEND_VERSION = "2.40.1"
DEFAULT_STATE = Path.home() / ".local/state/godot-agent-kit-vm"
READY_TIMEOUT = 180.0


class VMError(RuntimeError):
    """Infrastructure failure, never a request to execute on the host."""


class Tart:
    def __init__(self, state: Path = DEFAULT_STATE):
        self.state = Path(state).expanduser().absolute()
        if self.state == Path("/") or self.state == Path.home():
            raise VMError("Refusing non-dedicated VM state directory")
        self._safe_path(self.state)
        self.state.mkdir(mode=0o700, parents=True, exist_ok=True)
        if self.state.stat().st_uid != os.getuid():
            raise VMError("VM state must belong to the current user")
        self.home = self.state / "tart"
        self.backend = self.state / "backend/tart.app/Contents/MacOS/tart"
        self.receipt = self.state / "tart-profile.json"
        self.launcher = self.state / "tart-launcher.json"
        self.log = self.state / "tart-launcher.log"
        for path in (self.home, self.receipt, self.launcher, self.log):
            self._safe_path(path)
        self.home.mkdir(mode=0o700, exist_ok=True)
        # Dedicated state contains private control receipts and guest output.
        os.chmod(self.state, 0o700)
        os.chmod(self.home, 0o700)
        self.env = {k: v for k, v in os.environ.items()
                    if k not in ("TRACEPARENT", "TRACESTATE")
                    and not k.startswith(("OTEL", "CIRRUS", "TART_"))}
        self.env.update(TART_HOME=str(self.home), TART_NO_AUTO_PRUNE="1")
        self._version_checked = False

    @staticmethod
    def _safe_path(path: Path):
        for part in (path, *path.parents):
            if part.is_symlink():
                raise VMError(f"Refusing symlink in VM state: {part}")

    def _json(self, path):
        self._safe_path(path)
        try:
            value = json.loads(path.read_text())
            if not isinstance(value, dict):
                raise ValueError("expected receipt object")
            return value
        except (OSError, ValueError) as exc:
            raise VMError(f"Missing or invalid owned VM receipt: {path}") from exc

    def _write_json(self, path, value):
        self._safe_path(path)
        temporary = path.with_name(path.name + "." + uuid.uuid4().hex)
        fd = os.open(temporary, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
        with os.fdopen(fd, "w") as stream:
            json.dump(value, stream, sort_keys=True)
            stream.write("\n")
        os.replace(temporary, path)

    def _backend(self):
        if self._version_checked:
            return
        self._safe_path(self.backend)
        if not self.backend.is_file() or not os.access(self.backend, os.X_OK):
            raise VMError("Tart 2.40.1 is not installed in dedicated state; follow .github/LOCAL_VM.md manual backend installation")
        try:
            result = subprocess.run([str(self.backend), "--version"], env=self.env,
                                    capture_output=True, timeout=10, check=True)
        except (OSError, subprocess.SubprocessError) as exc:
            raise VMError(f"Cannot query dedicated Tart backend: {exc}") from exc
        if result.stdout.decode().strip() != BACKEND_VERSION:
            raise VMError(f"Expected Tart {BACKEND_VERSION}, got {result.stdout!r}")
        self._version_checked = True

    def _run(self, args, *, input=None, stdin=None, stdout=None, timeout=30):
        self._backend()
        try:
            return subprocess.run([str(self.backend), *args], env=self.env,
                                  input=input, stdin=stdin if stdin is not None or input is not None else subprocess.DEVNULL,
                                  stdout=subprocess.PIPE if stdout is None else stdout,
                                  stderr=subprocess.PIPE, timeout=timeout, check=False)
        except (OSError, subprocess.SubprocessError) as exc:
            raise VMError(f"Tart {args[0]} transport failed: {exc}") from exc

    @staticmethod
    def _require_success(result):
        if result.returncode:
            detail = (result.stderr or b"").decode(errors="replace").strip()
            raise VMError(f"Tart failed ({result.returncode}): {detail}")
        return result

    def _inventory(self):
        result = self._require_success(self._run(["list", "--source", "local", "--format", "json"]))
        try:
            rows = json.loads(result.stdout)
            if not isinstance(rows, list):
                raise ValueError("expected list")
            return {row["Name"]: row for row in rows}
        except (ValueError, TypeError, KeyError) as exc:
            raise VMError("Invalid Tart local inventory") from exc

    def _owned(self):
        receipt = self._json(self.receipt)
        if receipt.get("vm") != VM_NAME or receipt.get("home") != str(self.home) or receipt.get("image") != DEFAULT_IMAGE:
            raise VMError("VM profile ownership/provenance does not match dedicated state")
        self._safe_path(self.home / "vms" / VM_NAME)
        return receipt

    def _stopped(self, name, inventory):
        row = inventory.get(name)
        if row is None:
            raise VMError(f"VM {name} is missing; run setup first")
        if row.get("State") != "stopped" or row.get("Running") is not False:
            raise VMError(f"VM {name} must be stopped, not running or suspended")

    def setup(self, image: str = DEFAULT_IMAGE):
        if image != DEFAULT_IMAGE:
            raise VMError("Only the pinned DEFAULT_IMAGE is permitted")
        inventory = self._inventory()
        if self.receipt.exists():
            receipt = self._owned()
        else:
            if VM_NAME in inventory or BASE_NAME in inventory:
                raise VMError("Refusing unmarked existing VM profile")
            receipt = {"vm": VM_NAME, "home": str(self.home), "image": image,
                       "base": False, "configured": False}
            self._write_json(self.receipt, receipt)
        if VM_NAME in inventory:
            self._stopped(VM_NAME, inventory)
        else:
            self._require_success(self._run(["clone", image, VM_NAME], timeout=None))
            receipt["configured"] = False
        if not receipt.get("configured"):
            self._require_success(self._run(["set", VM_NAME, "--cpu", "2", "--memory", "6144", "--display", "1440x900", "--no-display-refit"]))
            receipt["configured"] = True
            self._write_json(self.receipt, receipt)

    def _launch_argv(self, bootstrap):
        return [str(self.backend), "run", VM_NAME, "--no-graphics", "--no-audio",
                "--no-clipboard", "--no-usb-accessories", *([] if bootstrap else ["--net-host"])]

    @staticmethod
    def _process_identity(pid):
        result = subprocess.run(["/bin/ps", "-ww", "-p", str(pid), "-o", "lstart=", "-o", "command="],
                                capture_output=True, check=False, timeout=5)
        return result.stdout.decode().strip() if result.returncode == 0 else ""

    def _lock_pid(self):
        # Darwin struct flock: off_t start/length, pid_t pid, short type/whence.
        path = self.home / "vms" / VM_NAME / "config.json"
        self._safe_path(path)
        fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW)
        try:
            lock = fcntl.fcntl(fd, fcntl.F_GETLK, struct.pack("qqihh", 0, 0, 0, fcntl.F_WRLCK, os.SEEK_SET))
            return struct.unpack("qqihh", lock)[2]
        finally:
            os.close(fd)

    def _verify_launcher(self):
        receipt = self._json(self.launcher)
        pid = receipt.get("pid")
        mode = receipt.get("network")
        if (type(pid) is not int or pid <= 1 or mode not in ("host-only", "bootstrap-nat")
                or receipt.get("argv") != self._launch_argv(mode == "bootstrap-nat")):
            raise VMError("Unsafe/unowned VM launcher receipt; refusing running VM")
        try:
            identity = self._process_identity(pid)
            lock_pid = self._lock_pid()
        except (OSError, subprocess.SubprocessError) as exc:
            raise VMError(f"Cannot verify owned VM launcher: {exc}") from exc
        if not identity or identity != receipt.get("identity") or lock_pid != pid:
            raise VMError("Unsafe/unowned running VM: launcher PID/identity does not match Tart VM lock")
        return receipt

    def status(self):
        inventory = self._inventory()
        row = inventory.get(VM_NAME)
        result = {"vm": VM_NAME, "exists": row is not None, "running": False,
                  "state": "missing" if row is None else row.get("State"), "backend": BACKEND_VERSION}
        if row:
            self._owned()
            result["running"] = row.get("Running") is True
            if result["running"]:
                result["network"] = self._verify_launcher()["network"]
        return result

    def start(self, bootstrap=False):
        if type(bootstrap) is not bool:
            raise VMError("bootstrap must explicitly be a boolean")
        self._backend()
        self._owned()
        inventory = self._inventory()
        if inventory.get(VM_NAME, {}).get("Running"):
            receipt = self._verify_launcher()
            if receipt["network"] != ("bootstrap-nat" if bootstrap else "host-only"):
                raise VMError("Running VM network differs; stop before changing bootstrap mode")
            self._require_success(self._run(["exec", VM_NAME, "/usr/bin/true"], timeout=10))
            return
        self._stopped(VM_NAME, inventory)
        fd = os.open(self.log, os.O_WRONLY | os.O_CREAT | os.O_TRUNC | os.O_NOFOLLOW, 0o600)
        try:
            os.fchmod(fd, 0o600)
            process = subprocess.Popen(self._launch_argv(bootstrap), env=self.env,
                                       stdin=subprocess.DEVNULL, stdout=fd, stderr=fd,
                                       start_new_session=True, close_fds=True)
        except OSError as exc:
            raise VMError(f"Cannot launch Tart VM: {exc}") from exc
        finally:
            os.close(fd)
        identity = self._process_identity(process.pid)
        self._write_json(self.launcher, {"pid": process.pid, "identity": identity,
                         "argv": self._launch_argv(bootstrap),
                         "network": "bootstrap-nat" if bootstrap else "host-only"})
        deadline = time.monotonic() + READY_TIMEOUT
        last_error = "guest agent did not respond"
        while time.monotonic() < deadline:
            if process.poll() is not None:
                raise VMError(f"Tart launcher exited ({process.returncode}); see private log {self.log}")
            if self._inventory().get(VM_NAME, {}).get("Running"):
                self._verify_launcher()
                result = self._run(["exec", VM_NAME, "/usr/bin/true"], timeout=min(10, max(0.1, deadline - time.monotonic())))
                if result.returncode == 0:
                    return
                last_error = (result.stderr or b"").decode(errors="replace").strip()
            time.sleep(1)
        raise VMError(f"VM readiness timed out: Tart Guest Agent/vsock unavailable ({last_error}); VM remains owned, use stop; log {self.log}")

    def stop(self):
        self._owned()
        inventory = self._inventory()
        if VM_NAME not in inventory:
            raise VMError("VM is missing; run setup first")
        if inventory[VM_NAME].get("State") == "stopped":
            return
        self._verify_launcher()
        self._require_success(self._run(["stop", VM_NAME, "--timeout", "30"], timeout=40))
        self._stopped(VM_NAME, self._inventory())
        if self.launcher.exists():
            self.launcher.unlink()

    def save_base(self):
        receipt = self._owned()
        inventory = self._inventory()
        self._stopped(VM_NAME, inventory)
        if BASE_NAME in inventory:
            if not receipt.get("base"):
                raise VMError("Refusing to replace unowned base VM")
            self._stopped(BASE_NAME, inventory)
            raise VMError("Owned base already exists; refusing unnecessary replacement")
        self._require_success(self._run(["clone", VM_NAME, BASE_NAME], timeout=None))
        receipt["base"] = True
        self._write_json(self.receipt, receipt)

    def reset(self):
        receipt = self._owned()
        inventory = self._inventory()
        if not receipt.get("base"):
            raise VMError("No owned base VM; stop and save_base first")
        self._safe_path(self.home / "vms" / BASE_NAME)
        self._stopped(BASE_NAME, inventory)
        if VM_NAME in inventory:
            self._stopped(VM_NAME, inventory)
            self._require_success(self._run(["delete", VM_NAME], timeout=None))
        self._require_success(self._run(["clone", BASE_NAME, VM_NAME], timeout=None))
        if self.launcher.exists():
            self.launcher.unlink()

    def exec(self, argv, *, input=None, stdin=None, stdout=None, check=True, timeout=None):
        if not isinstance(argv, (list, tuple)) or not argv or any(not isinstance(arg, str) or "\0" in arg for arg in argv) or not argv[0]:
            raise VMError("Guest command must be a nonempty direct argv sequence")
        if input is not None and stdin is not None:
            raise VMError("Provide input or stdin, not both")
        self._owned()
        row = self._inventory().get(VM_NAME)
        if row is None or not row.get("Running"):
            raise VMError("VM is missing or stopped; start it before guest exec (no host fallback)")
        self._verify_launcher()
        args = ["exec", *(["-i"] if input is not None or stdin is not None else []), VM_NAME, *argv]
        result = self._run(args, input=input, stdin=stdin, stdout=stdout, timeout=timeout)
        if check and result.returncode:
            raise subprocess.CalledProcessError(result.returncode, result.args, output=result.stdout, stderr=result.stderr)
        return result

    def upload(self, local: Path, guest_absolute: str):
        path = PurePosixPath(guest_absolute)
        if not path.is_absolute() or ".." in path.parts or "\0" in guest_absolute or str(path) == "/":
            raise VMError("Upload destination must be an absolute guest file path")
        # Fixed receiver: no shell and no data interpreted as code. Atomic install;
        # verify digest/length across vsock before replacing destination.
        receiver = """import hashlib,json,os,pathlib,sys,tempfile
p=pathlib.Path(sys.argv[1]); p.parent.mkdir(parents=True,exist_ok=True)
fd,t=tempfile.mkstemp(prefix='.vm-upload-',dir=p.parent)
h=hashlib.sha256(); n=0
try:
 with os.fdopen(fd,'wb') as f:
  while True:
   b=sys.stdin.buffer.read(1048576)
   if not b: break
   f.write(b); h.update(b); n+=len(b)
 if n!=int(sys.argv[2]) or h.hexdigest()!=sys.argv[3]: raise RuntimeError('upload digest/length mismatch')
 os.replace(t,p)
 print(json.dumps({'bytes':n,'sha256':h.hexdigest()}))
finally:
 if os.path.exists(t): os.unlink(t)
"""
        try:
            with Path(local).open("rb") as stream:
                if not stat.S_ISREG(os.fstat(stream.fileno()).st_mode):
                    raise VMError("Upload source must be a regular file")
                digest = hashlib.sha256()
                size = 0
                while chunk := stream.read(1024 * 1024):
                    digest.update(chunk)
                    size += len(chunk)
                stream.seek(0)
                result = self.exec([GUEST_PYTHON, "-c", receiver, guest_absolute, str(size), digest.hexdigest()], stdin=stream)
            receipt = json.loads(result.stdout)
            if receipt != {"bytes": size, "sha256": digest.hexdigest()}:
                raise VMError("Guest upload receipt did not match transferred bytes")
        except (OSError, ValueError, subprocess.SubprocessError) as exc:
            raise VMError(f"Guest upload failed: {exc}") from exc
