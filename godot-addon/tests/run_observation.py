#!/usr/bin/env python3
"""Owned real-editor observation, boundary and production-export acceptance.

This is test infrastructure, not a product editor-control/source API.
The Rust probes are temporary library consumers, not additional product operations.
"""
from __future__ import annotations

import argparse
import hashlib
import hmac
import json
import os
from pathlib import Path
import platform
import secrets
import shutil
import signal
import socket
import stat
import struct
import subprocess
import tempfile
import threading
import time
import zipfile

REPO = Path(__file__).resolve().parents[2]
FIXTURE = Path(__file__).resolve().parent / "fixtures" / "observation"
VERSION = "4.7.2.stable.official.ed1daf0bf"
ENGINE_HASH = "ed1daf0bf001b61586d9930840f2f1394092c079"
CAPABILITIES = ("observe_gdscript", "open_enumeration", "buffer_attribution",
                "unsaved_paths", "cached_resource_lookup")
MISSING_GROUPS = ("dirty-divergent", "dirty-unavailable", "changing-document",
                  "routing", "session-loss", "deadline", "confinement", "closed-and-invalid",
                  "surface-limits", "sequential-readonly", "redaction")
SOURCE_SENTINEL = b"T002_SYNTHETIC_SOURCE_ONLY"
CAP_SENTINELS = (b"# " + b"r" * 64, b"# " + b"b" * 64)


class Failure(Exception):
    """Safe stage-only failure; never includes frames, credentials, or source."""


def require(condition, stage):
    if not condition:
        raise Failure(stage)


def require_outcome(result, expected, stage):
    # Do not let a malformed source-bearing payload become failure/log text.
    known = {"complete_observation", "limited_observation", "not_open", "editor_unavailable",
             "ambiguous_target", "disconnected_editor", "timeout", "cancelled",
             "unsupported_observation", "denied_access", "protocol_error", "invalid_target",
             "missing_target", "invalid_request"}
    actual = result.get("outcome")
    require(actual == expected,
            stage + "_expected_" + expected + "_observed_" +
            (actual if isinstance(actual, str) and actual in known else "invalid"))


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest() if hasattr(hashlib, "file_digest") else _digest(stream)


def _digest(stream):
    value = hashlib.sha256()
    for block in iter(lambda: stream.read(1024 * 1024), b""):
        value.update(block)
    return value.hexdigest()



def disk_witness(path):
    before = path.stat()
    require(stat.S_ISREG(before.st_mode), "independent_disk_regular_file")
    raw = path.read_bytes()
    after = path.stat()
    require((before.st_dev, before.st_ino, before.st_size, before.st_mtime_ns, before.st_ctime_ns) ==
            (after.st_dev, after.st_ino, after.st_size, after.st_mtime_ns, after.st_ctime_ns) and
            len(raw) == before.st_size, "independent_disk_stable_read")
    try:
        text = raw.decode("utf-8", "strict")
    except UnicodeDecodeError as error:
        raise Failure("independent_disk_invalid_utf8") from error
    return {"text": text, "sha256": hashlib.sha256(raw).hexdigest(), "size": len(raw),
            "device": str(before.st_dev), "inode": str(before.st_ino),
            "mtime_ns": before.st_mtime_ns, "ctime_ns": before.st_ctime_ns,
            "mode": stat.S_IMODE(before.st_mode)}

def run(command, *, env=None, timeout=60):
    try:
        return subprocess.run([str(value) for value in command], env=env, capture_output=True,
                              timeout=timeout, check=False)
    except (OSError, subprocess.TimeoutExpired) as error:
        raise Failure("host_process_unavailable_or_deadline") from error


def wait_for(check, stage, timeout=20):
    deadline = time.monotonic() + timeout
    event = threading.Event()
    while True:
        result = check()
        if result:
            return result
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            raise Failure(stage)
        event.wait(min(0.025, remaining))


def json_file(path, value):
    temporary = path.with_suffix(".tmp")
    with temporary.open("x", encoding="utf-8") as stream:
        os.chmod(temporary, 0o600)
        json.dump(value, stream, ensure_ascii=False, sort_keys=True)
        stream.write("\n")
    temporary.replace(path)


def packet(value):
    body = json.dumps(value, separators=(",", ":"), ensure_ascii=False).encode()
    require(len(body) <= 4096, "fixture_control_bound")
    return struct.pack(">I", len(body)) + body


def peer_closed_without_data(stream):
    try:
        return stream.recv(1) == b""
    except ConnectionResetError:
        # Both EOF and reset are observable disconnection in the bridge contract.
        return True


def exact(stream, length):
    result = bytearray()
    while len(result) < length:
        part = stream.recv(length - len(result))
        if not part:
            raise EOFError
        result.extend(part)
    return bytes(result)


def receive(stream, limit=4096):
    length = struct.unpack(">I", exact(stream, 4))[0]
    require(0 < length <= limit, "control_frame_size")
    body = exact(stream, length)
    return json.loads(body), body


def proof(descriptor, response, role):
    fields = [b"godot-agent-kit/observation-bridge/v1", response["request_id"].encode(),
              bytes.fromhex(descriptor["session_id"]), descriptor["project_root"].encode(),
              descriptor["godot_version"].encode(), descriptor["engine_hash"].encode()]
    fields += [bytes([response["capabilities"][name]]) for name in CAPABILITIES]
    fields += [bytes.fromhex(response["client_nonce"]), bytes.fromhex(response["server_nonce"])]
    transcript = b"".join(struct.pack(">I", len(field)) + field for field in [role.encode(), *fields])
    return hmac.new(bytes.fromhex(descriptor["token"]), transcript, hashlib.sha256).hexdigest()


PROBE_SOURCE = r'''
use godot_agent_kit::{observation::*, target};
use std::{path::Path, time::{Duration, Instant}};
fn main() {
    let started = Instant::now();
    let a: Vec<String> = std::env::args().collect();
    let request = ObservationRequest::new(RequestId::new(a[4].clone()).unwrap(),
        ProjectRoot::new(a[2].clone()).unwrap(),
        if a[3] == "-" { None } else { Some(SessionId::new(a[3].clone()).unwrap()) },
        ResourcePath::new("res://scripts/subject.gd").unwrap());
    let result = match target::resolve(&request, Path::new(&a[1]), started + Duration::from_millis(4500)) {
        Ok(selected) => {
            let target = selected.target();
            serde_json::json!({"outcome":"selected", "session_id":target.session_id().as_str(),
                "project_root":target.project_root().as_str(), "godot_version":target.godot_version().version(),
                "script_path":target.script_path().as_str(),
                "project_file_id":{"device":target.project_file_id().device().as_str(),
                                   "inode":target.project_file_id().inode().as_str()},
                "observe_gdscript":selected.capabilities().observe_gdscript})
        }
        Err(failure) => serde_json::json!({"outcome":format!("{:?}", failure.outcome),
            "stage":format!("{:?}", failure.diagnostic.stage()),
            "code":format!("{:?}", failure.diagnostic.code()),
            "selection":failure.selection.as_ref().map(|selection| selection.candidate_sessions()
                .iter().map(|session| session.as_str()).collect::<Vec<_>>())}),
    };
    println!("{}", result);
}
'''

DISK_PROBE_SOURCE = r'''
use godot_agent_kit::{observation::*, project_fs, target};
use std::{path::Path, time::{Duration, Instant}};
fn main() {
    let started = Instant::now();
    let args: Vec<String> = std::env::args().collect();
    let request = ObservationRequest::new(RequestId::new("t003-disk-consumer").unwrap(),
        ProjectRoot::new(args[2].clone()).unwrap(), None,
        ResourcePath::new("res://scripts/subject.gd").unwrap());
    let selected = target::resolve(&request, Path::new(&args[1]),
        started + Duration::from_millis(4500)).expect("owned authenticated target");
    let disk = project_fs::read_disk(&selected, started).expect("confined D");
    let changes = project_fs::recheck_disk(&selected, &disk, started).expect("confined D recheck");
    assert!(changes.is_empty(), "unchanged owned source");
    let id = disk.witness().unwrap().disk_file_id().unwrap();
    println!("{}", serde_json::json!({
        "session_id":selected.target().session_id().as_str(),
        "project_root":selected.target().project_root().as_str(),
        "authority":"D", "availability":"observed", "text":disk.text().unwrap(),
        "file_id":{"device":id.device().as_str(), "inode":id.inode().as_str()},
        "checks":"performed", "detected_changes":[], "elapsed_us":started.elapsed().as_micros()
    }));
}
'''

WINDOW_SOURCE = r'''
import Foundation
import CoreGraphics
let pid = Int(CommandLine.arguments[1])!
let windows = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID) as? [[String: Any]] ?? []
let owned = windows.filter { ($0[kCGWindowOwnerPID as String] as? Int) == pid && ($0[kCGWindowLayer as String] as? Int) == 0 }
if let window = owned.first, let number = window[kCGWindowNumber as String] as? Int { print(number) } else { exit(2) }
'''


class Harness:
    def __init__(self, args, work):
        self.args = args
        self.work = work
        self.artifacts = args.artifacts
        self.registry = work / "registry"
        self.editors = []
        self.secrets = set()
        self.cases = []
        self.counter = 0
        self.probe = work / "route-probe"
        self.window_probe = work / "owned-window"
        self.summary = {"task": "T004" if args.scenario == "clean-open" else "T002",
                        "scenario": args.scenario, "cases": self.cases,
                        "source_observation": args.scenario in ("clean-open", "executor-boundary"),
                        "support_claim": False,
                        "host": {"os": platform.mac_ver()[0], "arch": platform.machine()},
                        "godot_sha256": digest(args.godot),
                        "observer_sha256": digest(args.observer),
                        "driver_sha256": digest(Path(__file__)),
                        "lockfile_sha256": digest(REPO / "mcp-server" / "Cargo.lock")}

    def safe_log(self, name, payload):
        require(SOURCE_SENTINEL not in payload and
                all(marker not in payload for marker in CAP_SENTINELS),
                "redaction_source_" + name)
        for value in self.secrets:
            require(value not in payload, "redaction_authentication_" + name)
        (self.artifacts / name).write_bytes(payload)

    def case(self, name, **evidence):
        self.cases.append({"case": name, "status": "passed", "source_surfaces": "not_acquired", **evidence})

    def initialize(self):
        version = run([self.args.godot, "--version"])
        require(version.returncode == 0 and version.stdout.decode().strip() == VERSION, "exact_godot_version")
        require(platform.system() == "Darwin" and platform.machine() == "arm64", "candidate_platform")
        self.summary["godot_version"] = VERSION
        self.summary["engine_hash"] = ENGINE_HASH
        result = run([self.args.observer, "init-registry", "--registry", self.registry])
        require(result.returncode == 0, "init_registry")
        require(stat.S_IMODE(self.registry.stat().st_mode) == 0o700, "private_registry")
        json.loads(result.stdout)
        self.safe_log("bootstrap.json", result.stdout)
        self.safe_log("bootstrap.stderr", result.stderr)
        for flag in ("--help", "--version"):
            response = run([self.args.observer, flag])
            require(response.returncode == 0, "caller_" + flag)
        self.case("bootstrap_help_version", registry_mode="0700")

    def compile_probes(self):
        source = self.work / "route-probe.rs"
        source.write_text(PROBE_SOURCE)
        debug = self.args.observer.parent
        libraries = list((debug / "deps").glob("libserde_json-*.rlib"))
        require(len(libraries) == 1 and (debug / "libgodot_agent_kit.rlib").is_file(), "built_library_prerequisite")
        result = run(["rustc", "+1.98.1", "--edition=2021", source, "--extern",
                      "godot_agent_kit=" + str(debug / "libgodot_agent_kit.rlib"), "--extern",
                      "serde_json=" + str(libraries[0]), "-L", "dependency=" + str(debug / "deps"),
                      "-o", self.probe])
        self.safe_log("library-consumer-compile.stderr", result.stderr)
        require(result.returncode == 0, "library_consumer_compile")
        self.compile_window_probe()

    def compile_window_probe(self):
        swift = self.work / "owned-window.swift"
        swift.write_text(WINDOW_SOURCE)
        result = run(["swiftc", swift, "-o", self.window_probe], timeout=120)
        self.safe_log("window-probe-compile.stderr", result.stderr)
        require(result.returncode == 0, "window_probe_compile")

    def fixture(self, name):
        project = self.work / name
        shutil.copytree(FIXTURE, project)
        shutil.copytree(REPO / "godot-addon" / "addons" / "godot_agent_kit",
                        project / "addons" / "godot_agent_kit")
        driver = project / "addons" / "fixture_driver"
        driver.mkdir(parents=True)
        (driver / "plugin.cfg").write_text('[plugin]\nname="Observation Fixture Driver"\ndescription="Owned test preparation"\nauthor="godot-agent-kit"\nversion="1"\nscript="plugin.gd"\n')
        (driver / "plugin.gd").write_text('@tool\nextends "res://fixture_driver.gd"\n')
        config = project / "project.godot"
        text = config.read_text()
        require("[editor_plugins]" not in text, "fixture_plugin_configuration_owned_by_harness")
        config.write_text(text + '\n[editor_plugins]\nenabled=PackedStringArray("res://addons/godot_agent_kit/plugin.cfg", "res://addons/fixture_driver/plugin.cfg")\n')
        return project

    def descriptors(self, project=None):
        descriptors = []
        for path in sorted(self.registry.glob("*.json")):
            descriptor = json.loads(path.read_text())
            self.secrets.add(descriptor["token"].encode())
            require(path.stat().st_uid == os.geteuid() and stat.S_IMODE(path.stat().st_mode) == 0o600,
                    "descriptor_owner_mode")
            require(descriptor["godot_version"] == VERSION and descriptor["engine_hash"] == ENGINE_HASH,
                    "descriptor_exact_engine")
            if project is None or Path(descriptor["project_root"]).resolve() == project.resolve():
                descriptors.append(descriptor)
        return descriptors

    def start_editor(self, project, *, configured=True, registry=None):
        self.counter += 1
        control = self.work / ("control-" + str(self.counter))
        control.mkdir(mode=0o700)
        log = self.work / ("editor-" + str(self.counter) + ".log")
        env = os.environ.copy()
        env.pop("GODOT_AGENT_KIT_REGISTRY", None)
        if configured:
            env["GODOT_AGENT_KIT_REGISTRY"] = str(registry or self.registry)
        env["GODOT_AGENT_KIT_FIXTURE_CONTROL"] = str(control)
        stream = log.open("wb")
        process = subprocess.Popen([str(self.args.godot), "--editor", "--path", str(project),
                                    "--single-window", "--display-driver", "macos", "--rendering-method",
                                    "gl_compatibility", "--rendering-driver", "opengl3"],
                                   env=env, stdout=stream, stderr=subprocess.STDOUT)
        editor = {"process": process, "control": control, "log": log, "stream": stream,
                  "project": project, "suspended": False}
        self.editors.append(editor)
        witness = self.action(editor, "witness", timeout=60)
        require(witness["editor_hint"] and witness["version"] == VERSION and witness["engine_hash"] == ENGINE_HASH,
                "live_editor_identity")
        return editor

    def action(self, editor, action, timeout=15):
        identity = secrets.token_hex(8)
        json_file(editor["control"] / "request.json", {"id": identity, "action": action})
        def response():
            path = editor["control"] / "response.json"
            if path.exists():
                result = json.loads(path.read_text())
                if result.get("id") == identity:
                    return result
            require(editor["process"].poll() is None, "editor_exited_during_" + action)
            return None
        result = wait_for(response, "fixture_action_" + action, timeout)
        require(result.get("ok") is True, "fixture_action_refused_" + action)
        return {key: value for key, value in result.items() if key not in ("id", "action", "ok")}

    def close_editor(self, editor):
        process = editor["process"]
        if process.poll() is None:
            if editor["suspended"]:
                process.send_signal(signal.SIGCONT)
                editor["suspended"] = False
            try:
                self.action(editor, "quit")
                process.wait(timeout=10)
            except (Failure, subprocess.TimeoutExpired):
                process.terminate()
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait(timeout=5)
        editor["stream"].close()
        self.safe_log(editor["log"].name, editor["log"].read_bytes())

    def screenshot(self, editor, name):
        self.action(editor, "present")
        def visible_window():
            require(editor["process"].poll() is None, "owned_editor_exited_before_capture")
            window = run([self.window_probe, editor["process"].pid], timeout=2)
            return window.stdout.decode().strip() if window.returncode == 0 and window.stdout.strip().isdigit() else None
        window = wait_for(visible_window, "owned_visible_window", timeout=20)
        path = self.artifacts / name
        result = run(["/usr/sbin/screencapture", "-x", "-l", window, path])
        require(result.returncode == 0 and path.is_file(), "owned_window_capture")
        return name

    def route(self, project, expected, session="-", case=None):
        start = time.monotonic()
        result = run([self.probe, self.registry, project, session, secrets.token_hex(16)], timeout=5.2)
        elapsed = time.monotonic() - start
        require(result.returncode == 0, "routing_consumer_exit")
        response = json.loads(result.stdout)
        require(response["outcome"] == expected, "routing_expected_" + expected)
        require(elapsed <= 5, "routing_five_second_deadline")
        if expected == "selected":
            require(response["project_root"] == str(project.resolve()), "canonical_project_identity")
            require(response["observe_gdscript"] is True, "installed_observation_capability")
            if session != "-":
                require(response["session_id"] == session, "exact_session_identity")
        if case:
            self.case(case, result=response, elapsed_seconds=elapsed)
        return response

    def connect(self, descriptor):
        stream = socket.create_connection(("127.0.0.1", descriptor["port"]), timeout=2)
        stream.settimeout(2)
        return stream

    def challenge(self, descriptor):
        stream = self.connect(descriptor)
        nonce = secrets.token_hex(32)
        self.secrets.add(nonce.encode())
        request = "python-proof-" + secrets.token_hex(4)
        hello = [1, "hello", request, descriptor["session_id"], descriptor["project_root"], nonce]
        wire = packet(hello)
        require(descriptor["token"].encode() not in wire, "secret_free_hello")
        stream.sendall(wire)
        response, raw = receive(stream)
        require(response.get("kind") == "challenge" and response.get("request_id") == request,
                "real_server_challenge")
        require(response.get("client_nonce") == nonce and response.get("session_id") == descriptor["session_id"],
                "real_server_transcript")
        require(response.get("project_root") == descriptor["project_root"] and
                response.get("godot_version") == VERSION and response.get("engine_hash") == ENGINE_HASH,
                "real_server_identity")
        require(hmac.compare_digest(response["server_proof"], proof(descriptor, response, "server")),
                "independent_server_proof")
        require(descriptor["token"].encode() not in raw, "secret_free_challenge")
        for key in ("server_nonce", "server_proof"):
            self.secrets.add(response[key].encode())
        return stream, response

    def authenticate(self, descriptor, response, client_proof):
        return [1, "authenticate", response["request_id"], descriptor["session_id"],
                descriptor["project_root"], response["client_nonce"], response["server_nonce"], client_proof]

    def refused(self, stream, payload):
        with stream:
            stream.sendall(payload)
            try:
                response, raw = receive(stream)
                require(response.get("kind") not in ("challenge", "hello", "sample", "recheck"), "unauthenticated_no_success")
                require(SOURCE_SENTINEL not in raw and b"source_code" not in raw, "unauthenticated_no_source")
                for secret in self.secrets:
                    require(secret not in raw, "refusal_redaction")
            except (EOFError, ConnectionResetError, BrokenPipeError):
                pass

    def authentication_cases(self, descriptor, editor):
        vectors = self.action(editor, "proof_vectors")
        require(vectors == {"transcript_bytes": 270, "server_matches": True,
                            "client_matches": True, "finish_matches": True}, "godot_contract_proof_vectors")
        self.case("cross_language_proof_vectors", **vectors)
        stream, challenge = self.challenge(descriptor)
        client = proof(descriptor, challenge, "client")
        self.secrets.add(client.encode())
        with stream:
            data = packet(self.authenticate(descriptor, challenge, client))
            require(descriptor["token"].encode() not in data, "secret_free_client_proof")
            stream.sendall(data)
            finish, raw = receive(stream)
            require(finish.get("kind") == "hello" and finish.get("finish_proof") == proof(descriptor, challenge, "finish"),
                    "independent_finish_proof")
            require({key: value for key, value in finish.items() if key not in ("kind", "finish_proof")} ==
                    {key: value for key, value in challenge.items() if key not in ("kind", "server_proof")},
                    "unchanged_finish_transcript")
            self.secrets.add(finish["finish_proof"].encode())
            require(descriptor["token"].encode() not in raw, "secret_free_finish")
        self.case("independent_mutual_proofs_and_secret_free_wire")
        for mode in ("wrong_secret", "replay", "reflection", "changed_request", "malformed_proof"):
            stream, response = self.challenge(descriptor)
            value = proof(descriptor, response, "client")
            if mode == "wrong_secret":
                value = "0" * 64
            elif mode == "replay":
                value = client
            elif mode == "reflection":
                value = response["server_proof"]
            elif mode == "malformed_proof":
                value = "not-a-proof"
            message = self.authenticate(descriptor, response, value)
            if mode == "changed_request":
                message[2] = "changed-request"
            self.refused(stream, packet(message))
            self.case("live_reject_" + mode)
        for operation in ("observe", "recheck", "authenticate"):
            message = [1, operation, "premature", descriptor["session_id"], descriptor["project_root"],
                       "res://scripts/subject.gd"]
            self.refused(self.connect(descriptor), packet(message))
            self.case("live_reject_premature_" + operation)
        for name, index, value in (("boolean_version", 0, True), ("fractional_version", 0, 1.5),
                                   ("nonscalar_session", 3, {}), ("nonscalar_project", 4, [])):
            message = [1, "hello", "invalid-type", descriptor["session_id"],
                       descriptor["project_root"], "a" * 64]
            message[index] = value
            self.refused(self.connect(descriptor), packet(message))
            self.case("live_reject_" + name)
        self.refused(self.connect(descriptor), struct.pack(">I", 4097))
        self.case("live_reject_oversized_control_before_body")
        stalled = []
        start = time.monotonic()
        try:
            for _ in range(32):
                # A verified challenge is the acceptance barrier, not TCP's backlog.
                # Do not finish authentication: every retained peer is still unauthenticated.
                try:
                    stream, _ = self.challenge(descriptor)
                except (OSError, EOFError) as error:
                    raise Failure("unauthenticated_accept_" + str(len(stalled) + 1) + "_" + type(error).__name__) from error
                stalled.append((stream, time.monotonic()))
            try:
                with self.connect(descriptor) as excess:
                    require(peer_closed_without_data(excess), "live_peer_capacity_rejects_excess")
            except ConnectionResetError:
                # Excess capacity may be reset before connect() itself returns.
                pass
            longest_idle = 0.0
            for stream, accepted_before in stalled:
                stream.settimeout(max(0.001, accepted_before + 4.5 - time.monotonic()))
                require(peer_closed_without_data(stream), "unauthenticated_peer_expires_without_source")
                longest_idle = max(longest_idle, time.monotonic() - accepted_before)
            self.case("live_peer_capacity_and_unauthenticated_expiry", incomplete_peers=32,
                      maximum_observed_idle_seconds=longest_idle, elapsed_seconds=time.monotonic() - start)
        except TimeoutError as error:
            raise Failure("unauthenticated_peer_capacity_or_expiry_deadline") from error
        finally:
            for stream, _ in stalled:
                stream.close()

    def registry_refusal_cases(self):
        project = self.fixture("unsafe-registries")
        inside = project / "registry"
        inside.mkdir(mode=0o700)
        acl_registry = self.work / "acl-registry"
        acl_registry.mkdir(mode=0o700)
        grant = run(["/bin/chmod", "+a", "everyone allow read,search,readattr,readextattr,readsecurity", acl_registry])
        require(grant.returncode == 0, "fixture_acl_preparation")
        require(stat.S_IMODE(acl_registry.stat().st_mode) == 0o700, "acl_does_not_change_mode_bits")
        for name, registry in (("in_project", inside), ("access_grant_acl", acl_registry)):
            editor = self.start_editor(project, registry=registry)
            witness = self.action(editor, "witness")
            require(witness["plugin_enabled"] and not witness["product_nodes"], "unsafe_registry_no_bridge_node_" + name)
            require(not list(registry.iterdir()), "unsafe_registry_no_metadata_" + name)
            listeners = run(["/usr/sbin/lsof", "-nP", "-a", "-p", editor["process"].pid,
                             "-iTCP", "-sTCP:LISTEN", "-Fn"])
            require(listeners.returncode in (0, 1), "unsafe_registry_listener_inspection")
            for line in listeners.stdout.decode().splitlines():
                if line.startswith("n") and ":" in line:
                    require(int(line.rsplit(":", 1)[1]) < 49152, "unsafe_registry_no_listener_" + name)
            self.close_editor(editor)
            self.case("live_refuse_registry_" + name, descriptor_count=0, bridge_nodes=0)
        acl_after = run(["/bin/ls", "-lde", acl_registry])
        require(acl_after.returncode == 0 and b"everyone allow" in acl_after.stdout,
                "unsafe_registry_acl_not_repaired")


    def observe(self, project, expected, exit_code, *, session=None, name=None):
        command = [self.args.observer, "--registry", self.registry, "--project", project,
                   "--script", "res://scripts/subject.gd"]
        if session is not None:
            command += ["--session", session]
        started = time.monotonic()
        response = run(command, timeout=5.2)
        elapsed = time.monotonic() - started
        return self.observer_result(response.returncode, response.stdout, response.stderr,
                                    elapsed, expected, exit_code, name or expected)

    def observer_result(self, code, stdout, stderr, elapsed, expected, exit_code, name):
        require(stdout.endswith(b"\n") and stdout.count(b"\n") == 1, "caller_single_json_" + name)
        result = json.loads(stdout)
        require(set(result) == {"schema_version", "request_id", "outcome", "interval",
                               "resolved_target", "snapshot", "diagnostics", "selection"},
                "caller_result_fields_" + name)
        if result["snapshot"] is None:
            self.safe_log(name + ".json", stdout)
        else:
            for secret in self.secrets:
                require(secret not in stdout, "source_result_authentication_redaction_" + name)
            (self.artifacts / (name + ".json")).write_bytes(stdout)  # Intentional source evidence.
        self.safe_log(name + ".stderr", stderr)
        require_outcome(result, expected, "caller_" + name + "_res_scripts_subject_gd")
        require(code == exit_code, "caller_exit_" + name)
        require(result["schema_version"] == 1, "caller_schema_" + name)
        if expected in ("editor_unavailable", "ambiguous_target", "denied_access"):
            require(result["snapshot"] is None, "caller_source_free_" + name)
        elif expected in ("not_open", "complete_observation", "limited_observation"):
            require(result["snapshot"] is not None, "caller_snapshot_" + name)
        require(elapsed < 5 and result["interval"]["elapsed_us"] < 5_000_000, "caller_deadline_" + name)
        self.case(name, source_surfaces="/".join(
            key for key, source in result["snapshot"]["sources"].items()
            if source["availability"] == "observed") if result["snapshot"] else "not_acquired",
                  outcome=expected, exit_code=code, elapsed_seconds=elapsed,
                  request_id=result["request_id"], target=result["resolved_target"])
        return result

    def live_witness(self, editor, path, disk):
        witness = self.action(editor, "witness")
        key = "subject" if path.name == "subject.gd" else "empty"
        doc = witness[key]
        resource_path = "res://scripts/" + path.name
        require(witness["current_script"] == resource_path and doc["associated"] and
                doc["path"] == resource_path and doc["matches"] == 1 and
                witness["script_count"] == witness["editor_count"] and
                witness["script_count"] == len(witness["open_paths"]) ==
                len(witness["script_ids"]) == len(witness["script_types"]) ==
                len(witness["editor_ids"]) == len(witness["editor_types"]) and
                len(set(witness["open_paths"])) == witness["script_count"] and
                len(set(witness["script_ids"])) == witness["script_count"] and
                len(set(witness["editor_ids"])) == witness["editor_count"] and
                all(value and value.startswith("res://") for value in witness["open_paths"]) and
                all(kind == "GDScript" for kind in witness["script_types"]) and
                witness["open_paths"].count(resource_path) == 1 and
                witness["script_ids"][doc["index"]] == doc["script_id"] and
                witness["editor_ids"][doc["index"]] == doc["editor_id"] and
                witness["editor_types"][doc["index"]] == doc["editor_type"] and
                doc["script_type"] == "GDScript" and doc["buffer_type"] == "CodeEdit" and
                doc["script_id"] != doc["editor_id"] != doc["buffer_id"],
                "independent_document_association_" + path.stem)
        require(doc["R"] == disk["text"] and doc["B"] == disk["text"] and
                not doc["dirty"] and resource_path not in witness["unsaved_paths"],
                "independent_clean_authorities_" + path.stem)
        return witness

    def authenticated_peer(self, descriptor):
        stream, challenge = self.challenge(descriptor)
        try:
            client = proof(descriptor, challenge, "client")
            self.secrets.add(client.encode())
            stream.sendall(packet(self.authenticate(descriptor, challenge, client)))
            finish, raw = receive(stream)
            require(finish.get("kind") == "hello" and
                    finish.get("finish_proof") == proof(descriptor, challenge, "finish"),
                    "observation_peer_mutual_proof")
            self.secrets.add(finish["finish_proof"].encode())
            require(descriptor["token"].encode() not in raw,
                    "observation_peer_secret_free_finish")
            require(all(challenge["capabilities"].get(name) is True for name in CAPABILITIES),
                    "live_collector_capabilities")
            return stream, challenge["request_id"]
        except (Failure, OSError, EOFError):
            stream.close()
            raise

    def peer_operation(self, stream, descriptor, request_id, operation, path):
        stream.sendall(packet([1, operation, request_id, descriptor["session_id"],
                               descriptor["project_root"], "res://scripts/" + path.name]))
        result, raw = receive(stream, 12 * 1024 * 1024)
        require(result.get("v") == 1 and result.get("kind") ==
                ("sample" if operation == "observe" else "recheck") and
                result.get("request_id") == request_id and
                result.get("session_id") == descriptor["session_id"] and
                result.get("project_root") == descriptor["project_root"] and
                result.get("script_path") == "res://scripts/" + path.name,
                "authenticated_" + operation + "_correlation")
        require(all(secret not in raw for secret in self.secrets),
                "authenticated_" + operation + "_secret_free")
        return result

    def compare_snapshot(self, result, descriptor, witness, disk, path):
        snapshot = result["snapshot"]
        doc = witness["subject" if path.name == "subject.gd" else "empty"]
        identity = snapshot["document"]["identity"]
        sources = snapshot["sources"]
        require(snapshot["target"] == result["resolved_target"] and
                snapshot["target"]["project_root"] == descriptor["project_root"] and
                snapshot["target"]["session_id"] == descriptor["session_id"] and
                snapshot["target"]["script_path"] == "res://scripts/" + path.name,
                "actual_resolved_project_session_resource")
        require(identity["kind"] == "external_gdscript" and
                identity["resource_path"] == "res://scripts/" + path.name and
                identity["script_instance_id"] == doc["script_id"] and
                identity["editor_instance_id"] == doc["editor_id"] and
                identity["buffer_instance_id"] == doc["buffer_id"] and
                identity["disk_file_id"] == {"device": disk["device"], "inode": disk["inode"]} and
                snapshot["document"]["validity"]["value"] == "valid" and
                snapshot["document"]["open_state"]["value"] == "open",
                "actual_document_identity_and_open_state")
        for authority, text in (("D", disk["text"]), ("R", doc["R"]), ("B", doc["B"])):
            source = sources[authority]
            require(source["authority"] == authority and source["availability"] == "observed" and
                    source["text"] == text and source["witness"]["resource_path"] ==
                    "res://scripts/" + path.name and
                    source["collection"] is not None and
                    source["staleness"]["state"] == "unknown" and
                    source["invalidated_evidence"] is None, "actual_source_" + authority)
            stamp = source["collection"]
            require(stamp["clock_id"] ==
                    ("caller" if authority == "D" else "editor:" + descriptor["session_id"]) and
                    0 <= int(stamp["started_tick_us"]) <= int(stamp["finished_tick_us"]) and
                    0 <= stamp["received_elapsed_us"] <= result["interval"]["elapsed_us"],
                    "actual_source_stamp_" + authority)
        require(sources["D"]["witness"]["disk_file_id"] ==
                {"device": disk["device"], "inode": disk["inode"]} and
                sources["R"]["witness"]["script_instance_id"] == doc["script_id"] and
                sources["B"]["witness"]["editor_instance_id"] == doc["editor_id"] and
                sources["B"]["witness"]["buffer_instance_id"] == doc["buffer_id"] and
                sources["B"]["witness"]["source_version"] == str(doc["version"]) and
                snapshot["dirty"]["witness"]["source_version"] == str(doc["version"]) and
                snapshot["dirty"]["availability"] == "observed" and
                snapshot["dirty"]["state"] == ("dirty" if doc["dirty"] else "clean") and
                snapshot["dirty"]["witness"]["buffer_instance_id"] == doc["buffer_id"],
                "independent_disk_resource_buffer_dirty_witnesses")
        comparisons = snapshot["comparisons"]
        require(comparisons == {
            "disk_resource": "equal" if disk["text"] == doc["R"] else "different",
            "disk_buffer": "equal" if disk["text"] == doc["B"] else "different",
            "resource_buffer": "equal" if doc["R"] == doc["B"] else "different"} and
                snapshot["agreement"] == "agree" and
                snapshot["consistency"]["checks"] == "performed" and
                snapshot["consistency"]["detected_changes"] == [] and
                snapshot["consistency"]["atomic"] is False,
                "actual_independent_comparisons_and_rechecks")

    def clean_read(self, editor, descriptor, project, path, name):
        shot = self.screenshot(editor, name + ".png")
        disk = disk_witness(path)
        before = self.live_witness(editor, path, disk)
        command = [self.args.observer, "--registry", self.registry, "--project", project,
                   "--script", "res://scripts/" + path.name, "--session", descriptor["session_id"]]
        started = time.monotonic()
        process = run(command, timeout=5.2)
        elapsed = time.monotonic() - started
        self.safe_log(name + ".stderr", process.stderr)
        require(all(secret not in process.stdout for secret in self.secrets),
                "caller_secret_free_" + name)
        result = json.loads(process.stdout)
        # Retain intentional synthetic evidence even when a real boundary fails.
        json_file(self.artifacts / (name + ".json"), {
            "disk": disk, "before": before, "result": result})
        require_outcome(result, "complete_observation", "D_R_B_res_scripts_" + path.name + "_" + name)
        require(process.returncode == 0 and process.stdout.endswith(b"\n") and
                process.stdout.count(b"\n") == 1 and elapsed < 5,
                "actual_complete_caller_" + name)
        require(result["schema_version"] == 1 and result["interval"]["elapsed_us"] < 5_000_000 and
                result["interval"]["started_unix_ms"] <= result["interval"]["finished_unix_ms"] and
                result["selection"] is None, "actual_complete_interval_" + name)
        self.compare_snapshot(result, descriptor, before, disk, path)
        after_disk = disk_witness(path)
        after = self.live_witness(editor, path, after_disk)
        require(before == after and disk == after_disk,
                "observer_editor_and_disk_noninterference_" + name)
        json_file(self.artifacts / (name + ".json"), {
            "disk": disk, "before": before, "after": after, "result": result})
        self.case(name, source_surfaces="D/R/B", request_id=result["request_id"],
                  session_id=descriptor["session_id"], project_root=descriptor["project_root"],
                  resource_path="res://scripts/" + path.name, screenshot=shot,
                  elapsed_seconds=elapsed, disk_sha256=disk["sha256"],
                  disk_size=disk["size"], disk_mtime_ns=disk["mtime_ns"],
                  interval=result["interval"], evidence=name + ".json")
        return result

    def clean_open(self):
        self.compile_window_probe()
        project = self.fixture("clean-open")
        editor = self.start_editor(project)
        descriptor = wait_for(lambda: self.descriptors(project), "clean_open_advertisement")[0]
        previous = None
        for path, action, cases in (
                (project / "scripts" / "subject.gd", "prepare_subject",
                 ("us1_1_clean_open", "us1_2_fresh_repeat")),
                (project / "scripts" / "empty.gd", "prepare_empty",
                 ("us1_3_empty_observed", "us1_3_empty_fresh_repeat"))):
            prepared = self.action(editor, action)
            require(prepared["ready"], "owned_fixture_preparation_" + path.stem)
            disk = disk_witness(path)
            require((path.name != "empty.gd" or disk["text"] == "") and
                    disk["size"] <= 512 * 1024, "prepared_strict_disk_" + path.stem)
            resource_path = "res://scripts/" + path.name
            wait_for(lambda: (lambda w: w if w["current_script"] == resource_path and
                     w["subject" if path.name == "subject.gd" else "empty"]["associated"]
                     else None)(self.action(editor, "witness")),
                     "independent_editor_ready_" + path.stem)
            self.live_witness(editor, path, disk)
            if path.name == "subject.gd":
                history = self.action(editor, "seed_history")
                require(history["has_redo"] and history["B"] == disk["text"] and
                        history["R"] == disk["text"] and not history["dirty"],
                        "fixture_prior_undo_redo_history_and_clean_source")
            for name in cases:
                result = self.clean_read(editor, descriptor, project, path, name)
                if previous is not None:
                    require(result["request_id"] != previous["request_id"] and
                            result["interval"] != previous["interval"],
                            "fresh_request_identity_and_interval_" + name)
                    for authority in ("R", "B"):
                        current_stamp = result["snapshot"]["sources"][authority]["collection"]
                        prior_stamp = previous["snapshot"]["sources"][authority]["collection"]
                        require(int(current_stamp["started_tick_us"]) > int(prior_stamp["finished_tick_us"]),
                                "fresh_editor_collection_" + authority + "_" + name)
                previous = result
        self.collector_boundaries()

    def collector_boundaries(self):
        for authority in ("R", "B"):
            project = self.fixture("cap-" + authority.lower())
            editor = self.start_editor(project)
            descriptor = wait_for(lambda: self.descriptors(project), "cap_advertisement_" + authority)[0]
            self.action(editor, "prepare_subject")
            wait_for(lambda: (lambda w: w if w["subject"]["associated"] else None)(
                     self.action(editor, "witness")), "cap_editor_ready_" + authority)
            path = project / "scripts" / "subject.gd"
            mode = "resource" if authority == "R" else "buffer"
            other = "B" if authority == "R" else "R"
            for boundary, size, outcome, code in (
                    ("exact", 512 * 1024, "complete_observation", 0),
                    ("over", 512 * 1024 + 1, "limited_observation", 2)):
                label = "cap-" + authority.lower() + "-" + boundary
                # Window activation can make Godot apply editor text to R. Finish
                # that preparation BEFORE seeding the distinct source authority.
                self.screenshot(editor, label + "-prepared.png")
                doc = self.action(editor, "cap_" + mode + "_" + boundary)
                disk = disk_witness(path)
                require(len(doc[authority].encode()) == size and
                        len(doc[other].encode()) <= 512 * 1024 and disk["size"] < 512 * 1024,
                        "independent_source_preparation_" + label)
                before = self.action(editor, "witness")
                result = self.observe(project, outcome, code, name=label)
                after = self.action(editor, "witness")
                evidence = label + "-witness.json"
                json_file(self.artifacts / evidence, {
                    "disk": disk, "before": before, "after": after, "result": result})
                snapshot = result["snapshot"]
                sources = snapshot["sources"]
                require(sources["D"]["text"] == disk["text"] and
                        sources[other]["availability"] == "observed" and
                        sources[other]["text"] == doc[other] and
                        snapshot["document"]["identity"]["script_instance_id"] == doc["script_id"] and
                        snapshot["document"]["identity"]["buffer_instance_id"] == doc["buffer_id"] and
                        snapshot["dirty"]["state"] == ("dirty" if doc["dirty"] else "clean") and
                        snapshot["consistency"]["checks"] == "performed" and
                        snapshot["consistency"]["detected_changes"] == [],
                        "independent_sources_and_rechecks_" + label)
                if boundary == "exact":
                    require(sources[authority]["availability"] == "observed" and
                            sources[authority]["text"] == doc[authority] and
                            snapshot["agreement"] == "divergent",
                            "exact_inclusive_source_limit_" + label)
                else:
                    require(sources[authority]["availability"] == "unavailable" and
                            sources[authority]["reason"]["code"] == "too_large" and
                            sources[authority].get("text") is None,
                            "independent_source_limit_" + label)
                require(before == after and disk == disk_witness(path),
                        "source_cap_noninterference_" + label)
                shot = self.screenshot(editor, label + ".png")
                self.cases[-1].update(screenshot=shot, evidence=evidence)
        self.recheck_boundaries()

    def recheck_boundaries(self):
        project = self.fixture("recheck-boundaries")
        editor = self.start_editor(project)
        descriptor = wait_for(lambda: self.descriptors(project), "recheck_advertisement")[0]
        self.action(editor, "prepare_subject")
        wait_for(lambda: (lambda w: w if w["subject"]["associated"] else None)(
                 self.action(editor, "witness")), "recheck_editor_ready")
        path = project / "scripts" / "subject.gd"
        disk = disk_witness(path)
        original = self.live_witness(editor, path, disk)
        refused, refusal_id = self.authenticated_peer(descriptor)
        with refused:
            refused.sendall(packet([1, "observe", refusal_id, descriptor["session_id"],
                                   descriptor["project_root"], "res://../outside.gd"]))
            failure, raw = receive(refused)
            require(failure == {
                "v": 1, "kind": "failure", "request_id": refusal_id,
                "session_id": descriptor["session_id"], "project_root": descriptor["project_root"],
                "script_path": "res://../outside.gd", "code": "out_of_project", "stage": "read_editor"},
                "authenticated_locator_refusal_has_no_source")
            require(all(secret not in raw for secret in self.secrets),
                    "authenticated_locator_refusal_has_no_credentials")
        self.case("authenticated_locator_refusal", outcome="denied_access", source_surfaces="not_acquired")
        stream, request_id = self.authenticated_peer(descriptor)
        with stream:
            sample = self.peer_operation(stream, descriptor, request_id, "observe", path)
            require(sample["R"]["text"] == original["subject"]["R"] and
                    sample["B"]["text"] == original["subject"]["B"] and
                    sample["dirty"]["state"] == "clean", "recheck_actual_initial_sources")
            busy = self.observe(project, "unsupported_observation", 2, name="active_collection_bound")
            require(busy["snapshot"] is None and
                    any(item["code"] == "unsupported_observation" for item in busy["diagnostics"]),
                    "busy_collector_is_source_free_not_disconnected")
            self.action(editor, "append_subject")  # Owned preparation between real bridge frames.
            changed = wait_for(lambda: (lambda w: w if
                               w["subject"]["B"] != original["subject"]["B"] and
                               w["subject"]["dirty"] else None)(
                               self.action(editor, "witness")), "recheck_real_buffer_dirty_barrier")
            require(changed["subject"]["B"] != original["subject"]["B"] and
                    changed["subject"]["dirty"] and
                    path == project / "scripts" / "subject.gd" and disk == disk_witness(path),
                    "recheck_real_buffer_change")
            rechecked = self.peer_operation(stream, descriptor, request_id, "recheck", path)
            changes = {(item["surface"], item["code"]) for item in rechecked["detected_changes"]}
            require(("B", "source_changed") in changes and
                    ("dirty", "source_changed") in changes and
                    rechecked["checks"] in ("performed", "unavailable"),
                    "recheck_changed_real_document_detected")
        shot = self.screenshot(editor, "recheck-boundaries.png")
        json_file(self.artifacts / "recheck-changed-evidence.json", {
            "before": original, "after": changed, "sample": sample, "recheck": rechecked})
        self.case("actual_changed_editor_recheck", source_surfaces="R/B",
                  screenshot=shot, changes=rechecked["detected_changes"],
                  session_id=descriptor["session_id"], evidence="recheck-changed-evidence.json")
        stream, request_id = self.authenticated_peer(descriptor)
        with stream:
            self.peer_operation(stream, descriptor, request_id, "observe", path)
            self.action(editor, "disable")
            try:
                self.peer_operation(stream, descriptor, request_id, "recheck", path)
            except (EOFError, ConnectionResetError, BrokenPipeError):
                pass
            else:
                raise Failure("failed_recheck_cannot_claim_performed")
        require(disk == disk_witness(path) and
                not (self.registry / (descriptor["session_id"] + ".json")).exists(),
                "failed_recheck_no_disk_write_or_stale_session")
        self.case("actual_failed_editor_recheck", source_surfaces="R/B",
                  session_id=descriptor["session_id"], reason="known_session_ended")

    def executor_boundary(self):
        self.summary["task"] = "T003"
        self.compile_probes()
        project = self.fixture("executor")
        subject = project / "scripts" / "subject.gd"
        before = (digest(subject), subject.stat().st_mtime_ns)
        self.observe(project, "editor_unavailable", 3, name="zero_sessions")
        first = self.start_editor(project)
        descriptor = wait_for(lambda: self.descriptors(project), "executor_advertisement")[0]
        self.action(first, "cache_subject")
        witness = self.action(first, "witness")
        require(witness["subject_cached_id"] and not witness["subject"]["associated"],
                "explicit_cached_closed_fixture")
        result = self.observe(project, "not_open", 0, name="closed_live_collector")
        require(result["resolved_target"]["session_id"] == descriptor["session_id"],
                "closed_retains_authenticated_target")
        require(result["snapshot"]["document"]["open_state"]["value"] == "not_open" and
                result["snapshot"]["sources"]["B"]["availability"] == "not_applicable" and
                result["snapshot"]["dirty"]["availability"] == "not_applicable" and
                result["snapshot"]["sources"]["D"]["text"] == subject.read_text() and
                result["snapshot"]["document"]["identity"]["script_instance_id"] ==
                witness["subject_cached_id"],
                "closed_document_not_manufactured_buffer")
        self.observe(project, "editor_unavailable", 3, session="f" * 32, name="ended_session_no_fallback")
        source = self.work / "disk-probe.rs"
        source.write_text(DISK_PROBE_SOURCE)
        debug = self.args.observer.parent
        libraries = list((debug / "deps").glob("libserde_json-*.rlib"))
        disk_probe = self.work / "disk-probe"
        compilation = run(["rustc", "+1.98.1", "--edition=2021", source, "--extern",
                           "godot_agent_kit=" + str(debug / "libgodot_agent_kit.rlib"), "--extern",
                           "serde_json=" + str(libraries[0]), "-L", "dependency=" + str(debug / "deps"),
                           "-o", disk_probe])
        self.safe_log("disk-consumer-compile.stderr", compilation.stderr)
        require(compilation.returncode == 0, "disk_consumer_compile")
        sampled = run([disk_probe, self.registry, project], timeout=5)
        require(sampled.returncode == 0, "authenticated_disk_consumer")
        disk = json.loads(sampled.stdout)
        require(disk["session_id"] == descriptor["session_id"] and
                disk["text"].encode() == subject.read_bytes() and
                int(disk["file_id"]["inode"]) == subject.stat().st_ino and
                disk["checks"] == "performed" and disk["detected_changes"] == [],
                "independent_disk_read_recheck")
        # Intentional synthetic D evidence, not an incidental source log.
        json_file(self.artifacts / "disk-consumer-evidence.json", disk)
        self.safe_log("disk-consumer.stderr", sampled.stderr)
        self.case("authenticated_confined_disk_read_recheck", source_surfaces="D",
                  session_id=descriptor["session_id"], file_id=disk["file_id"])
        shot = self.screenshot(first, "executor-boundary.png")
        second = self.start_editor(project)
        wait_for(lambda: len(self.descriptors(project)) == 2, "executor_ambiguous_advertisements")
        ambiguous = self.observe(project, "ambiguous_target", 3, name="two_sessions_no_source")
        require(ambiguous["resolved_target"] is None and ambiguous["selection"] is not None,
                "ambiguous_selector_feedback")
        self.observe(project, "not_open", 0, session=descriptor["session_id"],
                     name="exact_session_closed_document")
        first["process"].send_signal(signal.SIGSTOP)
        first["suspended"] = True
        try:
            self.observe(project, "timeout", 4, session=descriptor["session_id"], name="silent_editor")
            for cancel in (False, True):
                command = [self.args.observer, "--registry", self.registry, "--project", project,
                           "--session", descriptor["session_id"], "--script", "res://scripts/subject.gd"]
                started = time.monotonic()
                process = subprocess.Popen([str(arg) for arg in command], stdout=subprocess.PIPE,
                                           stderr=subprocess.PIPE)
                owned_worker = None
                try:
                    def child_pid():
                        children = run(["/usr/bin/pgrep", "-P", process.pid])
                        values = children.stdout.split()
                        require(len(values) <= 1, "exactly_one_owned_worker")
                        return int(values[0]) if values else None
                    owned_worker = wait_for(child_pid, "owned_worker_started", timeout=2)
                    os.kill(owned_worker, signal.SIGSTOP)
                    if cancel:
                        process.send_signal(signal.SIGINT)
                    stdout, stderr = process.communicate(timeout=5.1)
                    self.observer_result(process.returncode, stdout, stderr, time.monotonic() - started,
                                         "cancelled" if cancel else "timeout", 4,
                                         "cancelled_stalled_worker" if cancel else "stalled_worker")
                    def worker_gone():
                        try:
                            os.kill(owned_worker, 0)
                            return False
                        except ProcessLookupError:
                            return True
                    wait_for(worker_gone, "owned_worker_reaped", timeout=2)
                    require(first["process"].poll() is None and second["process"].poll() is None,
                            "supervisor_did_not_terminate_editors")
                finally:
                    if process.poll() is None:
                        process.kill()
                        process.communicate(timeout=2)
                    if owned_worker is not None:
                        try:
                            os.kill(owned_worker, signal.SIGKILL)
                        except ProcessLookupError:
                            pass
        finally:
            first["process"].send_signal(signal.SIGCONT)
            first["suspended"] = False
        final = self.action(first, "witness")
        require(witness.get("open_paths") == final.get("open_paths") and
                witness.get("current_script") == final.get("current_script") and
                before == (digest(subject), subject.stat().st_mtime_ns),
                "executor_source_selection_noninterference")
        self.case("executor_noninterference", screenshot=shot,
                  source_sha256=before[0], source_mtime_ns=before[1])

    def session_boundary(self):
        self.compile_probes()
        self.registry_refusal_cases()
        project = self.fixture("sessions")
        subject = project / "scripts" / "subject.gd"
        before = (digest(subject), subject.stat().st_mtime_ns)
        self.route(project, "EditorUnavailable", case="zero_candidates")
        unconfigured = self.start_editor(project, configured=False)
        require(not self.descriptors(), "missing_configuration_no_advertisement")
        listeners = run(["/usr/sbin/lsof", "-nP", "-a", "-p", unconfigured["process"].pid,
                         "-iTCP", "-sTCP:LISTEN", "-Fn"])
        require(listeners.returncode in (0, 1), "listener_inspection")
        for line in listeners.stdout.decode().splitlines():
            if line.startswith("n") and ":" in line:
                require(int(line.rsplit(":", 1)[1]) < 49152, "missing_configuration_no_bridge_listener")
        self.case("missing_configuration_no_listener")
        self.close_editor(unconfigured)
        first = self.start_editor(project)
        one = wait_for(lambda: self.descriptors(project), "first_advertisement")[0]
        initial_witness = self.action(first, "witness")
        self.route(project, "selected", case="one_authenticated_candidate")
        self.authentication_cases(one, first)
        shot = self.screenshot(first, "session-boundary.png")
        second = self.start_editor(project)
        pair = wait_for(lambda: self.descriptors(project) if len(self.descriptors(project)) == 2 else None,
                        "two_editor_advertisements")
        two = next(value for value in pair if value["session_id"] != one["session_id"])
        response = self.route(project, "AmbiguousTarget", case="two_authenticated_candidates")
        require(set(response["selection"]) == {one["session_id"], two["session_id"]}, "ambiguity_candidate_identity")
        self.route(project, "selected", one["session_id"], "exact_first_session")
        self.route(project, "selected", two["session_id"], "exact_second_session")
        self.route(project, "EditorUnavailable", "f" * 32, "absent_exact_session_no_fallback")
        second["process"].send_signal(signal.SIGSTOP)
        second["suspended"] = True
        try:
            self.route(project, "Timeout", case="unresolved_candidate_prevents_selection")
        finally:
            second["process"].send_signal(signal.SIGCONT)
            second["suspended"] = False
        self.action(first, "disable")
        wait_for(lambda: not (self.registry / (one["session_id"] + ".json")).exists(), "disable_removes_owned_descriptor")
        require((self.registry / (two["session_id"] + ".json")).exists(), "disable_preserves_other_session")
        self.route(project, "EditorUnavailable", one["session_id"], "disabled_session_no_replacement")
        self.action(first, "enable")
        renewed = wait_for(lambda: [value for value in self.descriptors(project)
                                   if value["session_id"] not in (one["session_id"], two["session_id"])],
                           "reenable_new_identity")[0]
        require(renewed["token"] != one["token"], "reenable_fresh_secret")
        self.route(project, "selected", renewed["session_id"], "reenable_fresh_authenticated_lifetime")
        self.close_editor(second)
        wait_for(lambda: not (self.registry / (two["session_id"] + ".json")).exists(), "second_exit_cleanup")
        final_witness = self.action(first, "witness")
        require(initial_witness.get("open_paths") == final_witness.get("open_paths") and
                initial_witness.get("current_script") == final_witness.get("current_script"), "source_free_selection_noninterference")
        self.close_editor(first)
        wait_for(lambda: not self.descriptors(), "editor_exit_metadata_cleanup")
        restarted = self.start_editor(project)
        replacement = wait_for(lambda: self.descriptors(project), "restart_advertisement")[0]
        require(replacement["session_id"] != renewed["session_id"] and replacement["token"] != renewed["token"],
                "restart_fresh_identity_and_secret")
        self.route(project, "EditorUnavailable", renewed["session_id"], "ended_session_never_substituted")
        self.route(project, "selected", replacement["session_id"], "restarted_session_authenticated")
        self.close_editor(restarted)
        wait_for(lambda: not self.descriptors(), "restart_exit_cleanup")
        require(before == (digest(subject), subject.stat().st_mtime_ns), "source_file_unchanged")
        self.case("source_free_lifecycle_noninterference", screenshot=shot,
                  before=initial_witness, after=final_witness, disk_sha256=before[0])
        self.rebound_port(project, replacement)

    def rebound_port(self, project, descriptor):
        listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        listener.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        listener.bind(("127.0.0.1", descriptor["port"]))
        listener.listen(1)
        listener.settimeout(5)
        path = self.registry / (descriptor["session_id"] + ".json")
        json_file(path, descriptor)  # Explicitly synthetic stale-metadata preparation.
        evidence = {}
        def impostor():
            try:
                with listener, listener.accept()[0] as stream:
                    stream.settimeout(2)
                    hello, raw = receive(stream)
                    evidence["secret_absent"] = descriptor["token"].encode() not in raw
                    nonce = secrets.token_hex(32)
                    self.secrets.add(nonce.encode())
                    fake = {"v": 1, "kind": "challenge", "request_id": hello[2], "session_id": hello[3],
                            "project_root": hello[4], "godot_version": VERSION, "engine_hash": ENGINE_HASH,
                            "capabilities": {name: False for name in CAPABILITIES}, "client_nonce": hello[5],
                            "server_nonce": nonce, "server_proof": "0" * 64}
                    stream.sendall(packet(fake))
                    evidence["closed_before_client_auth"] = not stream.recv(1)
            except (OSError, EOFError, Failure):
                evidence["peer_failed"] = True
        thread = threading.Thread(target=impostor, daemon=True)
        thread.start()
        try:
            result = self.route(project, "DeniedAccess", descriptor["session_id"], "stale_descriptor_rebound_impostor")
            require(result["code"] == "AuthenticationFailed", "impostor_specific_authentication_failure")
            thread.join(timeout=5)
            require(not thread.is_alive() and evidence == {"secret_absent": True, "closed_before_client_auth": True},
                    "impostor_never_receives_secret_or_client_auth")
        finally:
            listener.close()
            path.unlink(missing_ok=True)

    def export_boundary(self):
        template = Path.home() / "Library" / "Application Support" / "Godot" / "export_templates" / "4.7.2.stable" / "macos.zip"
        require(template.is_file(), "exact_macos_export_template_missing")
        self.summary["macos_template_sha256"] = digest(template)
        for label, enabled, hook_only in (("enabled", True, False), ("disabled", False, False),
                                          ("hook-only", True, True)):
            project = self.fixture("export-" + label)
            config = project / "project.godot"
            text = config.read_text()
            text = text[:text.index("[editor_plugins]")]
            text += '[editor_plugins]\nenabled=PackedStringArray(' + ('"res://addons/godot_agent_kit/plugin.cfg"' if enabled else '') + ')\n'
            config.write_text(text)
            if hook_only:
                # Deliberately remove only the product preset exclusion in this owned copy.
                # This exercises the hook before GDScript compilation/remapping independently.
                preset = project / "export_presets.cfg"
                preset.write_text(preset.read_text()
                                  .replace('"res://addons/godot_agent_kit/", ', "")
                                  .replace("addons/godot_agent_kit/*,addons/godot_agent_kit/**/*,", ""))
            out = self.work / ("export-output-" + label)
            out.mkdir(mode=0o700)
            archive, app = out / "fixture.zip", out / "fixture.app"
            env = os.environ.copy()
            env["GODOT_AGENT_KIT_REGISTRY"] = str(self.registry)
            env.pop("GODOT_AGENT_KIT_FIXTURE_CONTROL", None)
            for operation, destination in (("--export-pack", archive), ("--export-release", app)):
                result = run([self.args.godot, "--headless", "--path", project, operation, "macOS", destination],
                             env=env, timeout=180)
                self.safe_log("export-" + label + "-" + operation[2:] + ".log", result.stdout + result.stderr)
                require(result.returncode == 0 and destination.exists(), "export_generation_" + label)
            with zipfile.ZipFile(archive) as package:
                zip_entries = {name: package.read(name) for name in package.namelist() if not name.endswith("/")}
            (self.artifacts / ("export-" + label + ".zip")).write_bytes(archive.read_bytes())
            self.inspect_pack(zip_entries, "zip_" + label)
            packs = list(app.rglob("*.pck"))
            require(len(packs) == 1, "actual_app_pack_presence")
            entries = pck_entries(packs[0])
            (self.artifacts / ("export-" + label + ".pck")).write_bytes(packs[0].read_bytes())
            self.inspect_pack(entries, "app_pack_" + label)
            self.descriptors()
            require(not list(self.registry.iterdir()), "export_editor_metadata_cleanup")
            executables = list((app / "Contents" / "MacOS").iterdir())
            require(len(executables) == 1, "export_executable_identity")
            runtime_log = out / "runtime.log"
            with runtime_log.open("wb") as log:
                process = subprocess.Popen([str(executables[0]), "--quit-after", "240", "--max-fps", "60"],
                                           env=env, stdout=log, stderr=subprocess.STDOUT)
                try:
                    wait_for(lambda: b"T002_FIXTURE_RUNTIME_READY" in runtime_log.read_bytes(),
                             "actual_export_gameplay_readiness", timeout=15)
                    require(process.poll() is None, "export_alive_for_listener_inspection")
                    listeners = run(["/usr/sbin/lsof", "-nP", "-a", "-p", process.pid,
                                     "-iTCP", "-sTCP:LISTEN", "-Fn"])
                    require(listeners.returncode == 1 and not listeners.stdout, "export_runtime_no_listener")
                    process.wait(timeout=20)
                    require(process.returncode == 0, "actual_export_gameplay_exit")
                finally:
                    if process.poll() is None:
                        process.terminate()
                        try:
                            process.wait(timeout=5)
                        except subprocess.TimeoutExpired:
                            process.kill()
                            process.wait(timeout=5)
                    self.safe_log("export-" + label + "-runtime.log", runtime_log.read_bytes())
            require(b"ERROR:" not in runtime_log.read_bytes(), "export_runtime_no_missing_dependency")
            require(not list(self.registry.iterdir()), "export_runtime_no_advertisement")
            self.case("export_boundary_" + label, zip_files=sorted(zip_entries), app_pack_files=sorted(entries),
                      zip_sha256=digest(archive), pack_sha256=digest(packs[0]),
                      preset_sha256=digest(project / "export_presets.cfg"), runtime_exit=process.returncode,
                      registry_environment_present=True, observed_listeners=0, runtime_scene_nodes=1,
                      editor_plugin_setting_retained=any(
                          name.removeprefix("res://") == "project.binary" and b"godot_agent_kit" in value
                          for name, value in entries.items()))

    def inspect_pack(self, entries, stage):
        require("project.binary" in {path.removeprefix("res://") for path in entries}, stage + "_actual_project_pack")
        # Verify executable/resource entries, including compiler outputs and remaps.
        # Godot retains inert editor_plugins settings in project.binary; searching all
        # bytes for a name confuses those settings with shipped addon code. The actual
        # app separately proves the one-node scene, no missing dependency or listener.
        for name in entries:
            if "godot_agent_kit" in name or "fixture_driver" in name:
                raise Failure(stage + "_tooling_material:" + name)
        # Compiled/remapped files are inspected as actual entries, not just .gd names.
        require(any(name.endswith((".gdc", ".gd")) for name in entries), stage + "_gameplay_script_present")

    def cleanup(self):
        failures = []
        for editor in reversed(self.editors):
            try:
                self.close_editor(editor)
            except (Failure, OSError, subprocess.SubprocessError) as error:
                failures.append(str(error) if isinstance(error, Failure) else type(error).__name__)
        if failures:
            raise Failure("owned_cleanup:" + ",".join(failures))


def pck_entries(path):
    """Read unencrypted standalone PCK v2/v3/v4 directory and actual file bytes."""
    with path.open("rb") as stream:
        def integer(size):
            value = stream.read(size)
            require(len(value) == size, "pack_truncated")
            return int.from_bytes(value, "little")
        require(integer(4) == 0x43504447, "pack_magic")
        version = integer(4)
        require(version in (2, 3, 4), "pack_supported_format")
        engine = (integer(4), integer(4), integer(4))
        require(engine == (4, 7, 2), "pack_exact_engine")
        flags = integer(4)
        require(not flags & 1, "pack_directory_not_encrypted")
        file_base = integer(8)
        if version in (3, 4):
            stream.seek(integer(8))
        else:
            stream.seek(64, 1)
        count = integer(4)
        require(count <= 4096, "fixture_pack_file_count")
        directory = []
        for _ in range(count):
            size = integer(4)
            require(size <= 4096, "pack_path_bound")
            name = stream.read(size).rstrip(b"\x00").decode("utf-8")
            offset, length = integer(8), integer(8)
            expected_hash = stream.read(16)
            file_flags = integer(4)
            require(file_flags == 0 and length <= 16 * 1024 * 1024, "fixture_pack_plain_file")
            directory.append((name, file_base + offset, length, expected_hash))
        entries = {}
        for name, offset, length, expected_hash in directory:
            stream.seek(offset)
            value = stream.read(length)
            require(len(value) == length and hashlib.md5(value).digest() == expected_hash, "pack_file_integrity")
            require(name not in entries, "pack_duplicate_path")
            entries[name] = value
        return entries


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--godot", required=True, type=Path)
    parser.add_argument("--observer", required=True, type=Path)
    parser.add_argument("--scenario", required=True, choices=("all", "clean-open", "session-boundary", "executor-boundary", "export-boundary", *MISSING_GROUPS))
    parser.add_argument("--artifacts", required=True, type=Path)
    args = parser.parse_args()
    os.umask(0o077)
    for path in (args.godot, args.observer):
        require(path.is_absolute() and path.is_file() and os.access(path, os.X_OK), "absolute_executable_required")
    require(args.artifacts.is_absolute() and args.artifacts.is_dir() and not args.artifacts.is_symlink(),
            "absolute_existing_artifact_directory")
    metadata = args.artifacts.stat()
    require(metadata.st_uid == os.geteuid() and stat.S_IMODE(metadata.st_mode) == 0o700 and
            not list(args.artifacts.iterdir()), "empty_private_artifact_directory")
    if args.scenario == "all" or args.scenario in MISSING_GROUPS:
        missing = list(MISSING_GROUPS) if args.scenario == "all" else [args.scenario]
        json_file(args.artifacts / "summary.json", {"task": "T004", "status": "failed",
                  "stage": "coverage", "missing_groups": missing, "support_claim": False})
        print("Missing coverage: " + ", ".join(missing))
        return 1
    # Home is deliberately used instead of /tmp's symlink/writable ancestry.
    with tempfile.TemporaryDirectory(prefix=".godot-agent-kit-acceptance-", dir=Path.home()) as temporary:
        harness = Harness(args, Path(temporary))
        status = 0
        try:
            harness.initialize()
            if args.scenario == "session-boundary":
                harness.session_boundary()
            elif args.scenario == "executor-boundary":
                harness.executor_boundary()
            elif args.scenario == "clean-open":
                harness.clean_open()
            else:
                harness.export_boundary()
            harness.summary["status"] = "passed"
        except (Failure, OSError, ValueError, KeyError, EOFError, subprocess.SubprocessError) as error:
            status = 1
            harness.summary["status"] = "failed"
            harness.summary["stage"] = str(error) if isinstance(error, Failure) else type(error).__name__
        finally:
            try:
                harness.cleanup()
            except (Failure, OSError) as error:
                status = 1
                harness.summary["status"] = "failed"
                harness.summary["cleanup_stage"] = str(error) if isinstance(error, Failure) else type(error).__name__
            json_file(args.artifacts / "summary.json", harness.summary)
        print(json.dumps({"status": harness.summary["status"], "stage": harness.summary.get("stage"),
                          "passed_cases": len(harness.cases), "summary": str(args.artifacts / "summary.json")}))
        return status


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Failure as error:
        print("Acceptance prerequisite failed: " + str(error))
        raise SystemExit(1)
