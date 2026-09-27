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
SOURCE_SENTINEL = b"T002_SYNTHETIC_SOURCE_ONLY"
CAP_SENTINELS = (b"# " + b"r" * 64, b"# " + b"b" * 64, b"# " + b"d" * 64)
SOURCE_SENTINELS = (
    SOURCE_SENTINEL, *CAP_SENTINELS, b"SYNTHETIC_OTHER_DOCUMENT",
    b"FIXTURE_UNSAVED_CHANGE", b"TEMPORARY_DIRTY_EDIT", b"RESOURCE_DIFFERENT",
    b"DISTINCT_DISK_SOURCE", b"DISTINCT_UNIQUE_OBJECT", b"TRANSITION_BUFFER",
    b"TRANSITION_RESOURCE", b"TRANSITION_DISK", b"Synthetic plain-text editor tab.",
    b"FIXTURE_BUILTIN_NATIVE_SOURCE", b"FIXTURE_COLD_UNLOADED_SOURCE",
)
SEQUENCE_MARKERS = (b"FIXTURE_HISTORY_FIRST", b"FIXTURE_HISTORY_SECOND")
QUICKSTART_GROUPS = ("clean-open", "dirty-divergent", "dirty-unavailable",
                     "changing-document", "routing", "session-loss", "deadline",
                     "confinement", "closed-and-invalid", "surface-limits",
                     "sequential-readonly", "redaction", "export-boundary")
REDACTION_REPLAY = QUICKSTART_GROUPS[:11]
BOUNDARY_GROUPS = ("session-boundary", "executor-boundary")
ROUTE_MARKERS = (b"ROUTE_PROJECT_A", b"ROUTE_PROJECT_B", b"ROUTE_RESOURCE_A",
                 b"ROUTE_RESOURCE_B", b"ROUTE_RESOURCE_C", b"ROUTE_BUFFER_A",
                 b"ROUTE_BUFFER_B", b"ROUTE_BUFFER_C")


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
import AppKit
let pid = Int(CommandLine.arguments[1])!
// Present only the owned fixture process; Godot focus alone may leave an
// earlier editor hidden when several owned sessions share the application.
guard let application = NSRunningApplication(processIdentifier: pid_t(pid)) else { exit(2) }
application.unhide()
application.activate(options: [.activateAllWindows, .activateIgnoringOtherApps])
let windows = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID) as? [[String: Any]] ?? []
let owned = windows.filter {
    guard ($0[kCGWindowOwnerPID as String] as? Int) == pid,
          ($0[kCGWindowLayer as String] as? Int) == 0,
          let bounds = $0[kCGWindowBounds as String] as? [String: Any],
          let width = bounds["Width"] as? Double,
          let height = bounds["Height"] as? Double else { return false }
    return width > 100 && height > 100
}
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
        self.source_markers = set(ROUTE_MARKERS)
        self.cases = []
        self.counter = 0
        self.probe = work / "route-probe"
        self.window_probe = work / "owned-window"
        template = (Path.home() / "Library" / "Application Support" / "Godot" /
                    "export_templates" / "4.7.2.stable" / "macos.zip")
        self.summary = {"scenario": args.scenario, "cases": self.cases,
                        "groups": {}, "source_observation": args.scenario not in
                        ("export-boundary", "session-boundary"),
                        "support_claim": False, "coverage_scope": "complete_groups" if args.scenario == "all"
                        else "selected_group",
                        "host": {"os": platform.mac_ver()[0], "arch": platform.machine()},
                        "godot_sha256": digest(args.godot),
                        "observer_sha256": digest(args.observer),
                        "driver_sha256": digest(Path(__file__)),
                        "fixture_driver_sha256": digest(FIXTURE / "fixture_driver.gd"),
                        "fixture_files": {str(path.relative_to(FIXTURE)): digest(path) for path in
                                          sorted(FIXTURE.rglob("*")) if path.is_file()},
                        "lockfile_sha256": digest(REPO / "mcp-server" / "Cargo.lock"),
                        "macos_template_sha256": digest(template) if template.is_file() else None}

    def safe_log(self, name, payload):
        require(all(marker not in payload for marker in
                    (*SOURCE_SENTINELS, *SEQUENCE_MARKERS, *self.source_markers)),
                "redaction_source_" + name)
        for value in self.secrets:
            require(value not in payload, "redaction_authentication_" + name)
        (self.artifacts / name).write_bytes(payload)

    def case(self, name, **evidence):
        self.cases.append({"case": name, "status": "passed", "source_surfaces": "not_acquired", **evidence})

    def group(self, name, operation):
        start = len(self.cases)
        first_editor = len(self.editors)
        parent_work, parent_artifacts = self.work, self.artifacts
        self.work, self.artifacts = parent_work / name, parent_artifacts / name
        self.work.mkdir(mode=0o700)
        self.artifacts.mkdir(mode=0o700)
        artifact_directory = str(self.artifacts.relative_to(self.args.artifacts))
        try:
            operation()
        finally:
            try:
                for editor in reversed(self.editors[first_editor:]):
                    if not editor["stream"].closed:
                        self.close_editor(editor)
                del self.editors[first_editor:]
                # All descriptors belong to this private registry; no group
                # editor remains live. Stale metadata was tested before teardown.
                for descriptor in self.registry.glob("*.json"):
                    descriptor.unlink()
            finally:
                for case in self.cases[start:]:
                    case.setdefault("artifact_directory", artifact_directory)
                self.work, self.artifacts = parent_work, parent_artifacts
        cases = self.cases[start:]
        require(bool(cases) and all(case["status"] == "passed" for case in cases),
                "group_without_proven_cases_" + name)
        self.summary["groups"][name] = {
            "artifact_directory": artifact_directory,
            "case_count": len(cases), "case_names": [case["case"] for case in cases]}

    def verify_incidental_redaction(self):
        for path in self.artifacts.rglob("*"):
            if path.suffix not in (".json", ".stderr", ".log"):
                continue
            payload = path.read_bytes()
            require(all(secret not in payload for secret in self.secrets),
                    "final_artifact_secret_redaction_" + path.name)
            if path.name == "bootstrap.json" or path.suffix in (".stderr", ".log"):
                require(all(marker not in payload for marker in
                            (*SOURCE_SENTINELS, *SEQUENCE_MARKERS, *self.source_markers)),
                        "final_incidental_source_redaction_" + path.name)

    def initialize(self):
        version = run([self.args.godot, "--version"])
        require(version.returncode == 0 and version.stdout.decode().strip() == VERSION, "exact_godot_version")
        require(platform.system() == "Darwin" and platform.machine() == "arm64", "candidate_platform")
        self.summary["godot_version"] = VERSION
        self.summary["engine_hash"] = ENGINE_HASH
        if self.args.scenario in ("all", "export-boundary"):
            require(self.summary["macos_template_sha256"] is not None,
                    "exact_macos_export_template_missing")
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
        if self.probe.is_file():
            self.compile_window_probe()
            return
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
        if self.window_probe.is_file():
            return
        swift = self.work / "owned-window.swift"
        swift.write_text(WINDOW_SOURCE)
        result = run(["swiftc", swift, "-o", self.window_probe], timeout=120)
        self.safe_log("window-probe-compile.stderr", result.stderr)
        require(result.returncode == 0, "window_probe_compile")

    def fixture(self, name, *, controlled=False):
        project = self.work / name
        project.parent.mkdir(parents=True, exist_ok=True)
        shutil.copytree(FIXTURE, project)
        shutil.copytree(REPO / "godot-addon" / "addons" / "godot_agent_kit",
                        project / "addons" / "godot_agent_kit")
        driver = project / "addons" / "fixture_driver"
        driver.mkdir(parents=True)
        (driver / "plugin.cfg").write_text('[plugin]\nname="Observation Fixture Driver"\ndescription="Owned test preparation"\nauthor="godot-agent-kit"\nversion="1"\nscript="plugin.gd"\n')
        (driver / "plugin.gd").write_text('@tool\nextends "res://fixture_driver.gd"\n')
        # Keep every test-only bridge/collector under the already excluded driver
        # tree, including in export fixtures where neither helper is enabled.
        for script in ("fixture_bridge.gd", "fixture_collector.gd"):
            shutil.copy2(Path(__file__).parent / script, driver / script)
        config = project / "project.godot"
        text = config.read_text()
        require("[editor_plugins]" not in text, "fixture_plugin_configuration_owned_by_harness")
        config.write_text(text + '\n[editor_plugins]\nenabled=PackedStringArray("res://addons/godot_agent_kit/plugin.cfg", "res://addons/fixture_driver/plugin.cfg")\n')
        if controlled:
            plugin = project / "addons" / "godot_agent_kit" / "plugin.gd"
            original = plugin.read_text()
            old = 'preload("res://addons/godot_agent_kit/bridge.gd")'
            require(original.count(old) == 1, "fixture_private_bridge_seam")
            plugin.write_text(original.replace(old, 'preload("res://addons/fixture_driver/fixture_bridge.gd")'))
        return project

    def descriptors(self, project=None):
        descriptors = []
        for path in sorted(self.registry.glob("*.json")):
            descriptor = json.loads(path.read_text())
            encoded = json.dumps(descriptor).encode()
            require(all(marker not in encoded for marker in (*SOURCE_SENTINELS, *self.source_markers)),
                    "descriptor_has_no_synthetic_source")
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


    def observe(self, project, expected, exit_code, *, session=None, name=None, script="res://scripts/subject.gd"):
        started = time.monotonic()
        response = run(self.observation_command(project, session, script), timeout=5.2)
        elapsed = time.monotonic() - started
        return self.observer_result(response.returncode, response.stdout, response.stderr,
                                    elapsed, expected, exit_code, name or expected)

    def observation_command(self, project, session=None, script="res://scripts/subject.gd"):
        command = [self.args.observer, "--registry", self.registry, "--project", project,
                   "--script", script]
        if session is not None:
            command += ["--session", session]
        return [str(value) for value in command]

    def observer_result(self, code, stdout, stderr, elapsed, expected, exit_code, name):
        require(stdout.endswith(b"\n") and stdout.count(b"\n") == 1, "caller_single_json_" + name)
        result = json.loads(stdout)
        require(set(result) == {"schema_version", "request_id", "outcome", "interval",
                               "resolved_target", "snapshot", "diagnostics", "selection"},
                "caller_result_fields_" + name)
        diagnostics = json.dumps(result["diagnostics"], ensure_ascii=False).encode()
        require(all(marker not in diagnostics for marker in
                    (*SOURCE_SENTINELS, *SEQUENCE_MARKERS, *self.source_markers)) and
                all(secret not in diagnostics for secret in self.secrets),
                "source_free_diagnostics_" + name)
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
                snapshot["agreement"] == (
                    "agree" if disk["text"] == doc["R"] == doc["B"] else "divergent") and
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

    def controlled_editor(self, name):
        project = self.fixture(name, controlled=True)
        editor = self.start_editor(project)
        descriptor = wait_for(lambda: self.descriptors(project), "controlled_advertisement_" + name)[0]
        self.action(editor, "prepare_subject")
        wait_for(lambda: (lambda w: w if w["subject"]["associated"] else None)(
                 self.action(editor, "witness")), "controlled_subject_ready_" + name)
        return project, editor, descriptor, project / "scripts" / "subject.gd"

    def controlled_observation(self, project, editor, descriptor, path, name, expected,
                               *, changing=False, dirty=None):
        shot = self.screenshot(editor, name + ".png")
        disk_before = disk_witness(path)
        before = self.action(editor, "witness")
        doc = before["subject"]
        require(doc["associated"] and before["open_paths"].count("res://scripts/subject.gd") == 1
                and doc["script_id"] == before["script_ids"][doc["index"]]
                and doc["editor_id"] == before["editor_ids"][doc["index"]]
                and doc["dirty"] == ("res://scripts/subject.gd" in before["unsaved_paths"]),
                "independent_controlled_association_" + name)
        result = self.observe(project, expected, 0 if expected == "complete_observation" else 2,
                              session=descriptor["session_id"], name=name)
        disk_after = disk_witness(path) if path.exists() else None
        after = self.action(editor, "witness")
        transition = self.action(editor, "transition_witness") if changing else None
        json_file(self.artifacts / (name + "-witness.json"),
                  {"disk_before": disk_before, "before": before, "result": result,
                   "disk_after": disk_after, "after": after, "transition": transition})
        snapshot = result["snapshot"]
        require(snapshot["target"] == result["resolved_target"] and
                snapshot["target"]["session_id"] == descriptor["session_id"] and
                snapshot["target"]["script_path"] == "res://scripts/subject.gd",
                "controlled_target_" + name)
        if not changing:
            require(before == after and disk_before == disk_after,
                    "controlled_observer_noninterference_" + name)
            require(snapshot["document"]["identity"]["script_instance_id"] == doc["script_id"] and
                    snapshot["document"]["identity"]["editor_instance_id"] == doc["editor_id"] and
                    snapshot["document"]["identity"]["buffer_instance_id"] == doc["buffer_id"] and
                    snapshot["document"]["open_state"]["value"] == "open",
                    "controlled_live_identity_" + name)
            for authority, text in (("D", disk_before["text"]), ("R", doc["R"]), ("B", doc["B"])):
                source = snapshot["sources"][authority]
                require(source["availability"] == "observed" and source["text"] == text and
                        source["invalidated_evidence"] is None and source["staleness"]["state"] == "unknown",
                        "controlled_independent_" + authority + "_" + name)
            require(snapshot["comparisons"] == {
                "disk_resource": "equal" if disk_before["text"] == doc["R"] else "different",
                "disk_buffer": "equal" if disk_before["text"] == doc["B"] else "different",
                "resource_buffer": "equal" if doc["R"] == doc["B"] else "different"} and
                    snapshot["agreement"] == ("agree" if disk_before["text"] == doc["R"] == doc["B"]
                                              else "divergent") and
                    snapshot["consistency"]["detected_changes"] == [],
                    "controlled_exact_pairwise_" + name)
            if dirty is not None:
                require(doc["dirty"] == dirty and
                        snapshot["dirty"]["availability"] == "observed" and
                        snapshot["dirty"]["state"] == ("dirty" if dirty else "clean") and
                        snapshot["dirty"]["witness"]["source_version"] == str(doc["version"]),
                        "controlled_independent_dirty_" + name)
        self.cases[-1].update(screenshot=shot, evidence=name + "-witness.json",
                              disk_sha256=disk_before["sha256"])
        return result, disk_before, before, disk_after, after, transition

    def sequential_readonly(self):
        self.compile_window_probe()
        clean, clean_editor, clean_id, _ = self.controlled_editor("sequence-clean")
        dirty, dirty_editor, dirty_id, _ = self.controlled_editor("sequence-dirty")
        self.action(dirty_editor, "dirty_subject")
        history = self.action(dirty_editor, "seed_sequence_history")["history"]
        require(history["initial"] != history["first"] != history["second"],
                "sequence_history_distinct_prior_edits")
        closed = self.fixture("sequence-closed")
        closed_editor = self.start_editor(closed)
        closed_id = wait_for(lambda: self.descriptors(closed), "sequence_closed_advertisement")[0]
        self.action(closed_editor, "cache_subject")
        (closed / "scripts" / "cold.gd").write_text("extends RefCounted\n# FIXTURE_COLD_UNLOADED_SOURCE\n")
        shots = {phase: self.screenshot(editor, "sequence-" + phase + ".png")
                 for phase, editor in (("clean", clean_editor), ("dirty", dirty_editor),
                                       ("closed", closed_editor))}
        previous = None
        last_collection = {}
        requests = []

        def attempt(phase, project, editor, descriptor, target, selected, dirty_state):
            nonlocal previous
            name = "sequence_%02d_%s" % (len(requests) + 1, phase)
            script = "res://scripts/" + target
            files = sorted(path for path in (project / "scripts").iterdir() if path.is_file())
            files.append(project / "container.tscn")
            before_disk = {str(path.relative_to(project)): disk_witness(path) for path in files}
            disk = before_disk["scripts/" + target]
            before = self.action(editor, "witness")
            doc = before["subject"] if target == "subject.gd" else before["other"]
            require((before["current_script"] == script) == selected and
                    (doc["associated"] and doc["dirty"] == dirty_state if phase != "closed"
                     else script not in before["open_paths"] and not doc["associated"] and
                     script not in before["unsaved_paths"]),
                    "sequence_native_preparation_" + name)
            result = self.observe(project, "not_open" if phase == "closed" else
                                  "complete_observation", 0, session=descriptor["session_id"],
                                  script=script, name=name)
            after = self.action(editor, "witness")
            after_disk = {str(path.relative_to(project)): disk_witness(path) for path in files}
            evidence = name + "-witness.json"
            json_file(self.artifacts / evidence, {"phase": phase, "before_disk": before_disk,
                "before": before, "result": result, "after": after, "after_disk": after_disk})
            require(before == after and before_disk == after_disk,
                    "sequence_observer_noninterference_" + name)
            snapshot = result["snapshot"]
            source = snapshot["sources"]
            require(snapshot["target"] == result["resolved_target"] and
                    snapshot["target"]["project_root"] == str(project.resolve()) and
                    snapshot["target"]["session_id"] == descriptor["session_id"] and
                    snapshot["target"]["script_path"] == script and
                    snapshot["document"]["validity"]["value"] == "valid" and
                    source["D"]["availability"] == "observed" and
                    source["D"]["text"] == disk["text"] and
                    source["D"]["witness"]["disk_file_id"] ==
                    {"device": disk["device"], "inode": disk["inode"]} and
                    snapshot["consistency"]["checks"] == "performed" and
                    snapshot["consistency"]["detected_changes"] == [] and
                    result["selection"] is None and
                    result["interval"]["started_unix_ms"] <= result["interval"]["finished_unix_ms"],
                    "sequence_independent_disk_and_identity_" + name)
            if phase != "closed":
                if target == "subject.gd":
                    self.compare_snapshot(result, descriptor, before, disk, project / "scripts" / target)
                require(snapshot["document"]["open_state"]["value"] == "open" and
                        snapshot["document"]["identity"]["resource_path"] == script and
                        snapshot["document"]["identity"]["script_instance_id"] == doc["script_id"] and
                        snapshot["document"]["identity"]["editor_instance_id"] == doc["editor_id"] and
                        snapshot["document"]["identity"]["buffer_instance_id"] == doc["buffer_id"] and
                        all(source[key]["availability"] == "observed" and
                            source[key]["text"] == doc[key] and
                            source[key]["invalidated_evidence"] is None for key in ("R", "B")) and
                        source["R"]["witness"]["script_instance_id"] == doc["script_id"] and
                        source["B"]["witness"]["editor_instance_id"] == doc["editor_id"] and
                        source["B"]["witness"]["buffer_instance_id"] == doc["buffer_id"] and
                        source["B"]["witness"]["source_version"] == str(doc["version"]) and
                        snapshot["dirty"]["availability"] == "observed" and
                        snapshot["dirty"]["state"] == ("dirty" if dirty_state else "clean") and
                        snapshot["dirty"]["witness"]["source_version"] == str(doc["version"]) and
                        snapshot["dirty"]["witness"]["buffer_instance_id"] == doc["buffer_id"] and
                        snapshot["comparisons"] == {
                            "disk_resource": "equal" if disk["text"] == doc["R"] else "different",
                            "disk_buffer": "equal" if disk["text"] == doc["B"] else "different",
                            "resource_buffer": "equal" if doc["R"] == doc["B"] else "different"} and
                        snapshot["agreement"] == ("agree" if disk["text"] == doc["R"] == doc["B"]
                                                  else "divergent"),
                        "sequence_independent_native_open_authorities_" + name)
            else:
                # Native background import may cache a closed script between
                # requests. Compare actual pre-read cache evidence, not timing.
                prefix = "subject" if target == "subject.gd" else "cold"
                resource_id = before[prefix + "_cached_id"]
                cached = bool(resource_id)
                resource = before[prefix + "_cached_R"]
                require(snapshot["document"]["open_state"]["value"] == "not_open" and
                        source["B"]["availability"] == "not_applicable" and
                        snapshot["dirty"]["availability"] == "not_applicable" and
                        snapshot["dirty"]["state"] == "not_applicable" and
                        (source["R"]["availability"] == "observed") == cached and
                        (source["R"]["witness"]["script_instance_id"] == resource_id
                         if cached else True) and
                        (source["R"]["text"] == resource if cached else
                         source["R"]["availability"] == "unavailable" and
                         source["R"]["reason"]["code"] == "resource_not_loaded") and
                        (snapshot["document"]["identity"]["script_instance_id"] == resource_id
                         if cached else True) and
                        snapshot["comparisons"]["disk_resource"] ==
                        (("equal" if disk["text"] == resource else "different") if cached else "unknown"),
                        "sequence_independent_closed_native_authorities_" + name)
            clocks = {"D": source["D"]["collection"]}
            clocks.update({key: source[key]["collection"] for key in ("R", "B")
                           if source[key]["availability"] == "observed"})
            if snapshot["dirty"]["availability"] == "observed":
                clocks["dirty"] = snapshot["dirty"]["collection"]
            for key, stamp in clocks.items():
                require(stamp is not None and stamp["clock_id"] ==
                        ("caller" if key == "D" else "editor:" + descriptor["session_id"]) and
                        0 <= int(stamp["started_tick_us"]) <= int(stamp["finished_tick_us"]) and
                        0 <= stamp["received_elapsed_us"] <= result["interval"]["elapsed_us"],
                        "sequence_actual_collection_" + key + "_" + name)
                if key != "D":
                    identity = descriptor["session_id"], key
                    if identity in last_collection:
                        require(int(stamp["started_tick_us"]) >
                                int(last_collection[identity]["finished_tick_us"]),
                                "sequence_new_native_collection_" + key + "_" + name)
                    last_collection[identity] = stamp
            if previous is not None:
                require(result["request_id"] != previous["request_id"] and
                        result["interval"]["started_unix_ms"] >=
                        previous["interval"]["finished_unix_ms"],
                        "sequence_distinct_request_and_interval_" + name)
            previous = {"request_id": result["request_id"], "interval": result["interval"]}
            requests.append({"phase": phase, "request_id": result["request_id"],
                             "interval": result["interval"], "target": script,
                             "selected": selected, "dirty": dirty_state, "evidence": evidence})
            self.cases[-1].update(evidence=evidence, screenshot=shots[phase], phase=phase,
                                  interval=result["interval"], selected=selected,
                                  disk_sha256=disk["sha256"])

        for _ in range(3):
            attempt("clean", clean, clean_editor, clean_id, "subject.gd", True, False)
        self.action(clean_editor, "prepare_other")
        for _ in range(3):
            attempt("clean", clean, clean_editor, clean_id, "subject.gd", False, False)
        for _ in range(2):
            attempt("dirty", dirty, dirty_editor, dirty_id, "subject.gd", True, True)
        self.action(dirty_editor, "prepare_other")
        self.action(dirty_editor, "dirty_other")
        for _ in range(2):
            attempt("dirty", dirty, dirty_editor, dirty_id, "subject.gd", False, True)
        for _ in range(2):
            attempt("dirty", dirty, dirty_editor, dirty_id, "other.gd", True, True)
        self.action(dirty_editor, "prepare_subject")
        for _ in range(2):
            attempt("dirty", dirty, dirty_editor, dirty_id, "other.gd", False, True)
        for target in ("subject.gd", "cold.gd") * 3:
            attempt("closed", closed, closed_editor, closed_id, target, False, False)
        require(len(requests) == 20 and len({row["request_id"] for row in requests}) == 20 and
                [sum(row["phase"] == phase for row in requests) for phase in
                 ("clean", "dirty", "closed")] == [6, 8, 6],
                "sequence_exact_twenty_actual_observations")
        before_replay = self.action(dirty_editor, "witness")
        require(before_replay["subject"]["B"] == history["first"] and
                before_replay["subject"]["has_redo"], "sequence_original_redo_preserved")
        replay = self.action(dirty_editor, "replay_sequence_history")
        require(replay["before"] == before_replay["subject"] and
                [step["text"] for step in replay["steps"]] ==
                [history["second"], history["first"], history["initial"], history["first"]] and
                replay["after"]["B"] == history["first"] and replay["steps"][-1]["has_redo"],
                "sequence_fixture_native_history_reproduced")
        json_file(self.artifacts / "sequence-history-replay.json", {
            "known_prior_history": history, "before": before_replay,
            "native_replay": replay, "requests": requests})
        self.case("sequence_native_prior_history_replayed", source_surfaces="fixture_only",
                  evidence="sequence-history-replay.json", observations=len(requests))

    def dirty_divergent(self):
        self.compile_window_probe()
        for name, steps, disk_change, selected, pattern in (
                ("dirty_selected", ("dirty_subject",), None, True, "dirty_buffer"),
                ("dirty_nonselected", ("dirty_subject", "prepare_other"), None, False, "dirty_buffer"),
                ("dirty_distinct_resource", ("dirty_subject", "resource_subject"), None, True, "all_different"),
                ("dirty_equal_text", ("equal_dirty_subject",), None, True, "all_equal"),
                ("partial_disk_buffer", ("equal_dirty_subject", "resource_subject"), None, True, "equal_d_b"),
                ("partial_resource_buffer", (), "different", True, "equal_r_b"),
                ("exact_whitespace", (), "whitespace", True, "equal_r_b"),
                ("exact_line_endings", (), "line_endings", True, "equal_r_b")):
            project, editor, descriptor, path = self.controlled_editor(name)
            for step in steps:
                self.action(editor, step)
            if disk_change is not None:
                original = path.read_bytes()
                changed = {
                    "different": b"extends RefCounted\n# DISTINCT_DISK_SOURCE\n",
                    "whitespace": original.replace(b"\n", b" \n"),
                    "line_endings": original.replace(b"\n", b"\r\n"),
                }[disk_change]
                path.write_bytes(changed)
            doc = self.action(editor, "witness")["subject"]
            disk = disk_witness(path)
            require((doc["R"] == doc["B"] if pattern == "equal_r_b" else True) and
                    (disk["text"] != doc["B"] if pattern == "dirty_buffer" else True) and
                    (disk["text"] == doc["B"] if pattern == "equal_d_b" else True) and
                    (disk["text"] == doc["R"] == doc["B"] if pattern == "all_equal" else True) and
                    (len({disk["text"], doc["R"], doc["B"]}) == 3 if pattern == "all_different" else True) and
                    doc["dirty"] == bool(steps and steps[0] in
                                         ("dirty_subject", "equal_dirty_subject")) and
                    (self.action(editor, "witness")["current_script"] == "res://scripts/subject.gd") == selected,
                    "independent_prepared_divergence_" + name)
            self.controlled_observation(project, editor, descriptor, path, name,
                                        "complete_observation", dirty=doc["dirty"])

    def dirty_unavailable(self):
        self.compile_window_probe()
        for name, restriction, other in (
                ("dirty_indication_withheld", "restrict_dirty", False),
                ("unattributable_global_indication", "restrict_global", True)):
            project, editor, descriptor, path = self.controlled_editor(name)
            if other:
                self.action(editor, "prepare_other")
                self.action(editor, "dirty_other")
                self.action(editor, "prepare_subject")
            witness = self.action(editor, "witness")
            require(("res://scripts/other.gd" in witness["unsaved_paths"]) == other and
                    not witness["subject"]["dirty"] and
                    witness["subject"]["R"] == witness["subject"]["B"] == disk_witness(path)["text"],
                    "independent_global_unsaved_" + name)
            self.action(editor, restriction)
            result, _, _, _, _, _ = self.controlled_observation(
                project, editor, descriptor, path, name, "limited_observation")
            snapshot = result["snapshot"]
            require(snapshot["dirty"]["availability"] == "unavailable" and
                    snapshot["dirty"]["state"] == "unknown" and
                    snapshot["dirty"]["reason"]["code"] == "dirty_attribution_unavailable" and
                    snapshot["document"]["open_state"]["value"] == "open" and
                    snapshot["comparisons"] == {"disk_resource": "equal", "disk_buffer": "equal",
                                                "resource_buffer": "equal"},
                    "real_dirty_indication_not_inferred_" + name)
        for name, action in (("unsupported_association", "restrict_association"),
                             ("empty_path_tab", "unpathed_script"),
                             ("nonunique_path_tabs", "duplicate_script"),
                             ("mixed_text_script_documentation", "mixed_tabs")):
            project, editor, descriptor, path = self.controlled_editor(name)
            prepared = self.action(editor, action)
            before = self.action(editor, "witness")
            disk = disk_witness(path)
            require(not before["subject"]["associated"] or
                    (before["subject"]["R"] == disk["text"] and
                     before["subject"]["B"] == disk["text"]),
                    "real_association_input_sources_" + name)
            shot = self.screenshot(editor, name + ".png")
            result = self.observe(project, "limited_observation", 2,
                                  session=descriptor["session_id"], name=name)
            after = self.action(editor, "witness")
            require(before == after and disk == disk_witness(path),
                    "association_observer_does_not_change_tabs_" + name)
            snapshot = result["snapshot"]
            require(snapshot["target"]["script_path"] == "res://scripts/subject.gd" and
                    snapshot["sources"]["D"]["text"] == disk["text"] and
                    snapshot["sources"]["B"]["availability"] == "unavailable" and
                    snapshot["dirty"]["availability"] == "unavailable" and
                    snapshot["dirty"]["state"] == "unknown" and
                    snapshot["document"]["open_state"].get("value") != "not_open",
                    "unknown_association_not_wrong_buffer_" + name)
            if action == "restrict_association":
                require(snapshot["sources"]["R"]["availability"] == "observed" and
                        snapshot["sources"]["R"]["text"] == before["subject"]["R"] and
                        snapshot["document"]["open_state"]["value"] == "open",
                        "restricted_association_retains_known_resource")
            if action == "unpathed_script":
                require("" in before["open_paths"], "real_empty_resource_path")
            elif action == "duplicate_script":
                require(before["open_paths"].count("res://scripts/subject.gd") > 1,
                        "real_nonunique_resource_path")
                require(snapshot["sources"]["R"]["availability"] == "unavailable" and
                        snapshot["sources"]["R"]["reason"]["code"] == "resource_unreadable",
                        "nonunique_loaded_resources_are_not_reported_unloaded")
            elif action == "mixed_tabs":
                require(prepared["documentation_selected"] and prepared["text_selected"] and
                        before["editor_count"] > before["script_count"] and
                        before["script_types"].count("GDScript") >= 1,
                        "real_text_tab_and_documentation")
            evidence = name + "-witness.json"
            json_file(self.artifacts / evidence,
                      {"disk": disk, "before": before, "result": result, "after": after,
                       "prepared": prepared})
            self.cases[-1].update(screenshot=shot, evidence=evidence)


    def changing_document(self):
        self.compile_window_probe()
        for mode in ("buffer", "dirty", "resource", "disk", "rename", "remove", "close", "close_resource", "replace"):
            name = "change_" + mode
            project, editor, descriptor, path = self.controlled_editor(name)
            if mode in ("buffer", "resource", "close", "close_resource", "replace"):
                self.action(editor, "dirty_subject")
            self.action(editor, "transition_" + mode)
            result, disk, before, disk_after, after, barrier = self.controlled_observation(
                project, editor, descriptor, path, name, "limited_observation", changing=True)
            require(barrier["record"]["ok"] and barrier["record"]["mode"] == mode and
                    barrier["record"]["before"]["script_id"] == before["subject"]["script_id"] and
                    barrier["transition"] == "",
                    "real_recheck_transition_barrier_" + name)
            require(after == barrier["record"]["witness_after"],
                    "observer_preserves_post_transition_selection_history_sources_" + name)
            snapshot = result["snapshot"]
            if snapshot["sources"]["R"]["availability"] == "observed":
                require(snapshot["sources"]["R"]["text"] == barrier["record"]["original_resource"],
                        "retained_resource_is_still_independently_observed_" + name)
            changes = {(entry["surface"], entry["code"]) for entry in
                       snapshot["consistency"]["detected_changes"]}
            live = {authority: source.get("text") if source["availability"] == "observed"
                    else None for authority, source in snapshot["sources"].items()}
            pairs = {key: ("unknown" if live[left] is None or live[right] is None else
                           "equal" if live[left] == live[right] else "different")
                     for key, left, right in (("disk_resource", "D", "R"),
                                              ("disk_buffer", "D", "B"),
                                              ("resource_buffer", "R", "B"))}
            require(snapshot["comparisons"] == pairs and
                    snapshot["agreement"] == ("divergent" if "different" in pairs.values() else
                                              "agree" if all(pair == "equal" for pair in pairs.values())
                                              else "unknown"),
                    "changed_facts_never_enter_current_comparisons_" + name)
            if mode in ("buffer", "dirty", "resource", "disk", "remove"):
                surface = {"buffer": "B", "dirty": "B", "resource": "R",
                           "disk": "D", "remove": "D"}[mode]
                # Godot's safe-save may replace D's inode. Both forms must
                # invalidate only D; do not assume in-place filesystem writes.
                expected_change = (
                    "identity_changed" if surface == "D" and disk_after is not None
                    and (disk["device"], disk["inode"]) !=
                    (disk_after["device"], disk_after["inode"]) else "source_changed")
                require((surface, expected_change) in changes and
                        snapshot["sources"][surface]["availability"] == "unavailable" and
                        snapshot["sources"][surface]["invalidated_evidence"]["text"] ==
                        (disk["text"] if surface == "D" else before["subject"][surface]) and
                        (disk != disk_after if surface == "D" else before != after),
                        "changed_surface_invalidated_" + name)
                if mode == "dirty":
                    require(not before["subject"]["dirty"] and after["subject"]["dirty"] and
                            ("dirty", "source_changed") in changes and
                            snapshot["dirty"]["availability"] == "unavailable" and
                            snapshot["dirty"]["invalidated_evidence"]["state"] == "clean",
                            "actual_unsaved_indication_change_invalidated")
                for untouched in {"D", "R", "B"} - {surface}:
                    old_text = disk["text"] if untouched == "D" else before["subject"][untouched]
                    require(snapshot["sources"][untouched]["availability"] == "observed" and
                            snapshot["sources"][untouched]["text"] == old_text and
                            snapshot["sources"][untouched]["invalidated_evidence"] is None,
                            "changed_surface_preserves_independent_" + untouched + "_" + name)
                if surface == "D":
                    require(after["subject"]["R"] == before["subject"]["R"] and
                            after["subject"]["B"] == before["subject"]["B"],
                            "disk_change_does_not_repair_editor_" + name)
                else:
                    other = "R" if surface == "B" else "B"
                    require(disk == disk_after and
                            after["subject"][other] == before["subject"][other],
                            "editor_change_does_not_repair_other_authority_" + name)
            elif mode in ("close", "close_resource"):
                require(("document", "document_closed") in changes and
                        snapshot["document"]["open_state"].get("value") is None and
                        snapshot["document"]["open_state"]["invalidated_evidence"]["value"] == "open" and
                        snapshot["sources"]["B"]["availability"] == "unavailable" and
                        snapshot["sources"]["B"]["invalidated_evidence"]["text"] ==
                        before["subject"]["B"] and
                        snapshot["dirty"]["invalidated_evidence"] is not None and
                        snapshot["sources"]["D"]["text"] == disk["text"] and
                        snapshot["comparisons"]["disk_buffer"] == "unknown" and
                        snapshot["comparisons"]["resource_buffer"] == "unknown",
                        "closure_invalidates_open_buffer_not_known_closed")
                resource = snapshot["sources"]["R"]
                if mode == "close_resource":
                    require(barrier["record"]["original_resource"] != before["subject"]["R"] and
                            ("R", "source_changed") in changes and
                            resource["availability"] == "unavailable" and
                            resource["invalidated_evidence"]["text"] == before["subject"]["R"] and
                            snapshot["comparisons"]["disk_resource"] == "unknown",
                            "closure_does_not_hide_independent_resource_change")
                else:
                    require(resource["availability"] == "observed" and
                            resource["text"] == before["subject"]["R"],
                            "closure_preserves_unchanged_resource")
            else:
                require(("document", "identity_changed") in changes and
                        snapshot["document"]["open_state"].get("value") is None and
                        all(snapshot["sources"][source]["availability"] == "unavailable" and
                            snapshot["sources"][source]["invalidated_evidence"]["text"] ==
                            (disk["text"] if source == "D" else before["subject"][source])
                            for source in ("D", "R", "B")) and
                        snapshot["dirty"]["availability"] == "unavailable" and
                        snapshot["dirty"]["invalidated_evidence"] is not None and
                        snapshot["comparisons"] == {"disk_resource": "unknown",
                                                     "disk_buffer": "unknown",
                                                     "resource_buffer": "unknown"},
                        "replacement_excludes_original_document_facts_" + name)
                if mode == "replace":
                    require(barrier["record"]["replacement_id"] != before["subject"]["script_id"],
                            "genuine_new_document_instance")

    def story_read(self, project, editor, descriptor, name, expected, code, *,
                   script="res://scripts/subject.gd", disk_path=None, shot=None):
        shot = shot or self.screenshot(editor, name + ".png")
        before = self.action(editor, "witness")
        disk = disk_witness(disk_path) if disk_path is not None else None
        result = self.observe(project, expected, code, session=descriptor["session_id"],
                              script=script, name=name)
        after = self.action(editor, "witness")
        require(before == after and
                (disk_path is None or disk == disk_witness(disk_path)),
                "story_observer_read_only_" + name)
        require((expected in ("denied_access", "invalid_request") and
                 result["resolved_target"] is None) or
                (result["resolved_target"] is not None and
                 result["resolved_target"]["project_root"] == descriptor["project_root"] and
                 result["resolved_target"]["session_id"] == descriptor["session_id"] and
                 result["resolved_target"]["script_path"] == script),
                "story_original_target_" + name)
        evidence = name + "-witness.json"
        # Intentional source evidence; incidental stderr/editor logs stay redacted.
        json_file(self.artifacts / evidence, {"before": before, "after": after,
                                               "disk": disk, "result": result})
        self.cases[-1].update(evidence=evidence, screenshot=shot)
        return result, before, disk

    def closed_and_invalid(self):
        self.compile_window_probe()
        project = self.fixture("closed-invalid")
        editor = self.start_editor(project)
        descriptor = wait_for(lambda: self.descriptors(project), "closed_invalid_advertisement")[0]
        self.action(editor, "cache_subject")
        cached = self.action(editor, "witness")
        require(cached["subject_cached_id"] and
                "res://scripts/subject.gd" not in cached["open_paths"],
                "actual_cached_closed_native_script")
        path = project / "scripts" / "subject.gd"
        result, before, disk = self.story_read(
            project, editor, descriptor, "closed_cached", "not_open", 0, disk_path=path)
        snapshot = result["snapshot"]
        require(snapshot["document"]["validity"]["value"] == "valid" and
                snapshot["document"]["open_state"]["value"] == "not_open" and
                snapshot["document"]["identity"]["script_instance_id"] ==
                before["subject_cached_id"] and
                snapshot["sources"]["D"]["text"] == disk["text"] and
                snapshot["sources"]["R"]["availability"] == "observed" and
                snapshot["sources"]["R"]["text"] == before["subject_cached_R"] and
                snapshot["sources"]["R"]["witness"]["script_instance_id"] ==
                before["subject_cached_id"] and
                snapshot["sources"]["B"]["availability"] == "not_applicable" and
                snapshot["dirty"]["availability"] == "not_applicable" and
                snapshot["dirty"]["state"] == "not_applicable" and
                snapshot["comparisons"]["disk_resource"] ==
                ("equal" if disk["text"] == before["subject_cached_R"] else "different"),
                "closed_cached_independent_authorities")
        cold_shot = self.screenshot(editor, "closed_unloaded.png")
        cold = project / "scripts" / "cold.gd"
        cold.write_text("extends RefCounted\n# FIXTURE_COLD_UNLOADED_SOURCE\n")
        require(not self.action(editor, "witness")["cold_cached_id"],
                "cold_script_really_unloaded")
        result, before, disk = self.story_read(
            project, editor, descriptor, "closed_unloaded", "not_open", 0,
            script="res://scripts/cold.gd", disk_path=cold, shot=cold_shot)
        snapshot = result["snapshot"]
        require(not before["cold_cached_id"] and
                snapshot["document"]["open_state"]["value"] == "not_open" and
                snapshot["sources"]["D"]["text"] == disk["text"] and
                snapshot["sources"]["R"]["availability"] == "unavailable" and
                snapshot["sources"]["R"]["reason"]["code"] == "resource_not_loaded" and
                snapshot["sources"]["R"].get("text") is None and
                snapshot["sources"]["B"]["availability"] == "not_applicable" and
                snapshot["dirty"]["availability"] == "not_applicable" and
                snapshot["comparisons"]["disk_resource"] == "unknown",
                "closed_unloaded_never_force_loaded")
        require(not (project / "scripts" / "absent.gd").exists(),
                "missing_disk_precondition")
        result, _, _ = self.story_read(
            project, editor, descriptor, "closed_missing", "missing_target", 3,
            script="res://scripts/absent.gd")
        require(not (project / "scripts" / "absent.gd").exists(),
                "observer_did_not_create_missing_script")
        if result["snapshot"] is not None:
            snapshot = result["snapshot"]
            require(snapshot["document"]["validity"]["value"] == "missing" and
                    snapshot["sources"]["D"]["reason"]["code"] == "disk_missing" and
                    all(snapshot["sources"][key]["availability"] != "observed"
                        for key in ("R", "B")), "missing_not_closed_or_synthetic")
        result, _, note_disk = self.story_read(
            project, editor, descriptor, "closed_non_gdscript", "invalid_target", 3,
            script="res://scripts/note.txt", disk_path=project / "scripts" / "note.txt")
        if result["snapshot"] is not None:
            require(result["snapshot"]["document"]["validity"]["value"] == "invalid" and
                    all(item["availability"] != "observed"
                        for item in result["snapshot"]["sources"].values()),
                    "non_gdscript_invalid_without_source")
        self.action(editor, "prepare_invalid")
        invalid = project / "scripts" / "invalid.gd"
        result, before, disk = self.story_read(
            project, editor, descriptor, "open_syntax_invalid", "complete_observation", 0,
            script="res://scripts/invalid.gd", disk_path=invalid)
        snapshot = result["snapshot"]
        doc = before["invalid"]
        require(doc["associated"] and "var =" in disk["text"] and
                snapshot["document"]["validity"]["value"] == "valid" and
                snapshot["document"]["open_state"]["value"] == "open" and
                snapshot["document"]["identity"]["script_instance_id"] == doc["script_id"] and
                all(snapshot["sources"][key]["availability"] == "observed" and
                    snapshot["sources"][key]["text"] == text for key, text in
                    (("D", disk["text"]), ("R", doc["R"]), ("B", doc["B"]))) and
                snapshot["dirty"]["state"] == ("dirty" if doc["dirty"] else "clean"),
                "invalid_syntax_still_actual_gdscript")
        self.action(editor, "prepare_empty")
        result, before, disk = self.story_read(
            project, editor, descriptor, "open_empty", "complete_observation", 0,
            script="res://scripts/empty.gd", disk_path=project / "scripts" / "empty.gd")
        require(disk["size"] == 0 and before["empty"]["R"] == before["empty"]["B"] == "" and
                all(result["snapshot"]["sources"][key]["availability"] == "observed" and
                    result["snapshot"]["sources"][key]["text"] == ""
                    for key in ("D", "R", "B")), "empty_is_observed_not_unavailable")
        self.action(editor, "cache_builtin")
        result, before, _ = self.story_read(
            project, editor, descriptor, "closed_native_builtin", "not_open", 0,
            script="res://container.tscn::GDScript_x", disk_path=project / "container.tscn")
        snapshot = result["snapshot"]
        require(not before["builtin"]["associated"] and before["builtin_cached_id"] and
                snapshot["document"]["identity"]["kind"] == "builtin_gdscript" and
                snapshot["document"]["identity"]["script_instance_id"] == before["builtin_cached_id"] and
                snapshot["sources"]["D"]["reason"]["code"] == "no_standalone_disk_source" and
                snapshot["sources"]["R"]["text"] == before["builtin_R"] and
                snapshot["sources"]["B"]["availability"] == "not_applicable" and
                snapshot["dirty"]["availability"] == "not_applicable" and
                snapshot["agreement"] == "unknown",
                "closed_builtin_uses_existing_cached_native_identity")
        self.action(editor, "prepare_builtin")
        result, before, container_disk = self.story_read(
            project, editor, descriptor, "open_native_builtin", "limited_observation", 2,
            script="res://container.tscn::GDScript_x",
            disk_path=project / "container.tscn")
        snapshot = result["snapshot"]
        builtin = before["builtin"]
        container = container_disk["text"].encode()
        require(builtin["associated"] and builtin["script_type"] == "GDScript" and
                before["builtin_R"] == builtin["R"] and
                snapshot["document"]["identity"]["kind"] == "builtin_gdscript" and
                snapshot["document"]["identity"]["script_instance_id"] == builtin["script_id"] and
                snapshot["document"]["open_state"]["value"] == "open" and
                snapshot["sources"]["D"]["availability"] == "unavailable" and
                snapshot["sources"]["D"]["reason"]["code"] == "no_standalone_disk_source" and
                snapshot["sources"]["D"].get("text") is None and
                all(snapshot["sources"][key]["availability"] == "observed" and
                    snapshot["sources"][key]["text"] == builtin[key] for key in ("R", "B")) and
                snapshot["dirty"]["state"] == ("dirty" if builtin["dirty"] else "clean") and
                snapshot["comparisons"]["disk_resource"] == "unknown" and
                container != builtin["R"].encode(),
                "builtin_uses_existing_native_script_never_scene_bytes")
        result, before, _ = self.story_read(
            project, editor, descriptor, "unresolved_builtin", "unsupported_observation", 2,
            script="res://container.tscn::GDScript_absent",
            disk_path=project / "container.tscn")
        require(before["builtin"]["associated"] and
                (result["snapshot"] is None or all(
                    source["availability"] != "observed"
                    for source in result["snapshot"]["sources"].values())),
                "unresolved_builtin_never_force_loaded_or_guessed")
        container_before = disk_witness(project / "container.tscn")
        outside = self.work / "outside-builtin-scene.tscn"
        outside.write_bytes((project / "container.tscn").read_bytes())
        (project / "escaped.tscn").symlink_to(outside)
        for name, locator, outcome in (
                ("builtin_container_escape", "res://escaped.tscn::GDScript_x", "denied_access"),
                ("builtin_parent_escape", "res://../outside-builtin-scene.tscn::GDScript_x",
                 "invalid_request")):
            result, _, _ = self.story_read(
                project, editor, descriptor, name, outcome, 3, script=locator)
            require(result["snapshot"] is None and
                    disk_witness(project / "container.tscn") == container_before,
                    "unsafe_builtin_container_denied_without_source_" + name)

    def surface_limits(self):
        self.compile_window_probe()
        project, editor, descriptor, path = self.controlled_editor("surface-partial")
        self.action(editor, "prepare_other")
        for label, action, authority, reason in (
                ("resource_withheld", "restrict_resource", "R", "resource_unreadable"),
                ("buffer_withheld", "restrict_buffer", "B", "buffer_unreadable"),
                ("association_withheld", "restrict_association", "B",
                 "buffer_attribution_unavailable")):
            self.action(editor, action)
            result, before, disk = self.story_read(
                project, editor, descriptor, label, "limited_observation", 2, disk_path=path)
            snapshot = result["snapshot"]
            other = "B" if authority == "R" else "R"
            require(before["subject"]["associated"] and before["current_script"] !=
                    "res://scripts/subject.gd" and
                    snapshot["document"]["open_state"]["value"] == "open" and
                    snapshot["sources"]["D"]["text"] == disk["text"] and
                    snapshot["sources"][authority]["availability"] == "unavailable" and
                    snapshot["sources"][authority]["reason"]["code"] == reason and
                    snapshot["sources"][authority].get("text") is None and
                    snapshot["sources"][other]["availability"] == "observed" and
                    snapshot["sources"][other]["text"] == before["subject"][other] and
                    snapshot["comparisons"]["disk_resource" if authority == "R"
                                                else "disk_buffer"] == "unknown",
                    "negative_only_withholding_preserves_other_authorities_" + label)
        self.action(editor, "restrict_open")
        result, _, disk = self.story_read(
            project, editor, descriptor, "unknown_open", "limited_observation", 2, disk_path=path)
        snapshot = result["snapshot"]
        require(snapshot["document"]["open_state"].get("value") is None and
                snapshot["sources"]["D"]["text"] == disk["text"] and
                snapshot["sources"]["B"]["availability"] == "unavailable" and
                snapshot["dirty"]["availability"] == "unavailable" and
                snapshot["dirty"]["state"] == "unknown",
                "withheld_open_does_not_infer_closed")
        self.action(editor, "restore_dirty")
        self.action(editor, "mixed_tabs")
        result, before, disk = self.story_read(
            project, editor, descriptor, "mixed_text_script_documentation",
            "limited_observation", 2, disk_path=path)
        snapshot = result["snapshot"]
        require(before["editor_count"] > before["script_count"] and
                before["current_script"] != "res://scripts/subject.gd" and
                snapshot["document"]["open_state"]["value"] == "open" and
                snapshot["sources"]["D"]["text"] == disk["text"] and
                snapshot["sources"]["R"]["text"] == before["subject_cached_R"] and
                snapshot["sources"]["B"]["availability"] == "unavailable" and
                snapshot["sources"]["B"]["reason"]["code"] == "buffer_attribution_unavailable" and
                snapshot["dirty"]["state"] == "unknown",
                "mixed_tabs_never_guess_buffer_association")
        missing_project, missing_editor, missing_descriptor, missing_path = self.controlled_editor(
            "surface-open-missing")
        prior_disk = disk_witness(missing_path)
        self.action(missing_editor, "remove_subject_disk")
        require(not missing_path.exists(), "fixture_only_removal_of_open_disk")
        result, before, _ = self.story_read(
            missing_project, missing_editor, missing_descriptor,
            "open_disk_missing", "limited_observation", 2)
        snapshot = result["snapshot"]
        require(before["subject"]["associated"] and
                snapshot["document"]["open_state"]["value"] == "open" and
                snapshot["sources"]["D"]["availability"] == "unavailable" and
                snapshot["sources"]["D"]["reason"]["code"] == "disk_missing" and
                all(snapshot["sources"][key]["availability"] == "observed" and
                    snapshot["sources"][key]["text"] == before["subject"][key]
                    for key in ("R", "B")) and not missing_path.exists(),
                "open_missing_D_retains_native_R_B")
        self.cases[-1]["removed_disk_sha256"] = prior_disk["sha256"]
        unreadable_project, unreadable_editor, unreadable_descriptor, unreadable_path = \
            self.controlled_editor("surface-open-unreadable")
        original_disk = disk_witness(unreadable_path)
        original_mode = stat.S_IMODE(unreadable_path.stat().st_mode)
        unreadable_path.chmod(0)
        try:
            unreadable_before = unreadable_path.stat()
            require(stat.S_IMODE(unreadable_before.st_mode) == 0,
                    "fixture_disk_unreadable_mode")
            result, before, _ = self.story_read(
                unreadable_project, unreadable_editor, unreadable_descriptor,
                "open_disk_unreadable", "limited_observation", 2)
            unreadable_after = unreadable_path.stat()
            metadata_fields = ("st_dev", "st_ino", "st_size", "st_mode",
                               "st_mtime_ns", "st_ctime_ns")
            require(all(getattr(unreadable_before, key) ==
                        getattr(unreadable_after, key) for key in metadata_fields),
                    "unreadable_D_disk_metadata_unchanged")
            json_file(self.artifacts / "open_disk_unreadable-metadata.json", {
                "original_sha256": original_disk["sha256"],
                "before": {key: getattr(unreadable_before, key) for key in metadata_fields},
                "after": {key: getattr(unreadable_after, key) for key in metadata_fields}})
            self.cases[-1]["disk_metadata_evidence"] = "open_disk_unreadable-metadata.json"
            snapshot = result["snapshot"]
            require(snapshot["document"]["open_state"]["value"] == "open" and
                    snapshot["sources"]["D"]["availability"] == "unavailable" and
                    snapshot["sources"]["D"]["reason"]["code"] == "disk_unreadable" and
                    all(snapshot["sources"][key]["availability"] == "observed" and
                        snapshot["sources"][key]["text"] == before["subject"][key]
                        for key in ("R", "B")) and
                    stat.S_IMODE(unreadable_path.stat().st_mode) == 0,
                    "unreadable_D_preserves_other_authorities")
        finally:
            unreadable_path.chmod(original_mode)
        require(disk_witness(unreadable_path)["sha256"] == original_disk["sha256"],
                "unreadable_D_fixture_restored")
        for boundary, size, expected in (
                ("exact", 512 * 1024, "complete_observation"),
                ("over", 512 * 1024 + 1, "limited_observation")):
            name = "open_cap_d_" + boundary
            cap_project, cap_editor, cap_descriptor, cap_path = self.controlled_editor(name)
            self.action(cap_editor, "dirty_subject")
            cap_path.write_bytes(b"# " + b"d" * (size - 2))
            result, before, disk = self.story_read(
                cap_project, cap_editor, cap_descriptor, name, expected,
                0 if boundary == "exact" else 2, disk_path=cap_path)
            sources = result["snapshot"]["sources"]
            doc = before["subject"]
            require(doc["associated"] and disk["size"] == size and
                    len(doc["R"].encode()) < 512 * 1024 and
                    len(doc["B"].encode()) < 512 * 1024 and
                    result["snapshot"]["document"]["open_state"]["value"] == "open" and
                    all(sources[key]["availability"] == "observed" and
                        sources[key]["text"] == doc[key] for key in ("R", "B")) and
                    result["snapshot"]["dirty"]["state"] ==
                    ("dirty" if doc["dirty"] else "clean"),
                    "open_D_limit_preserves_preceding_native_R_B_" + name)
            require((sources["D"]["availability"] == "observed" and
                     sources["D"]["text"] == disk["text"]) if boundary == "exact" else
                    (sources["D"]["availability"] == "unavailable" and
                     sources["D"]["reason"]["code"] == "too_large" and
                     sources["D"].get("text") is None and
                     result["snapshot"]["comparisons"]["resource_buffer"] ==
                     ("equal" if doc["R"] == doc["B"] else "different")),
                    "open_D_exact_byte_boundary_" + name)
        self.collector_boundaries()
        for boundary, d_size, r_boundary in (
                ("both_exact", 512 * 1024, "exact"),
                ("d_over_r_exact", 512 * 1024 + 1, "exact"),
                ("d_exact_r_over", 512 * 1024, "over"),
                ("both_over", 512 * 1024 + 1, "over")):
            name = "closed_cap_" + boundary
            cap_project = self.fixture(name, controlled=True)
            cap_editor = self.start_editor(cap_project)
            cap_descriptor = wait_for(lambda: self.descriptors(cap_project),
                                      "closed_cap_advertisement_" + boundary)[0]
            cap_path = cap_project / "scripts" / "subject.gd"
            cap_path.write_bytes(b"# " + b"d" * (d_size - 2))
            self.action(cap_editor, "cap_closed_resource_" + r_boundary)
            result, before, disk = self.story_read(
                cap_project, cap_editor, cap_descriptor, name, "not_open", 0,
                disk_path=cap_path)
            sources = result["snapshot"]["sources"]
            expected_r = 512 * 1024 + (r_boundary == "over")
            require(not before["subject"]["associated"] and
                    len(before["subject_cached_R"].encode()) == expected_r and
                    disk["size"] == d_size and
                    result["snapshot"]["document"]["open_state"]["value"] == "not_open" and
                    sources["B"]["availability"] == "not_applicable" and
                    result["snapshot"]["dirty"]["availability"] == "not_applicable",
                    "closed_cap_actual_prepared_authorities_" + name)
            for authority, size, text in (("D", d_size, disk["text"]),
                                           ("R", expected_r, before["subject_cached_R"])):
                value = sources[authority]
                require((value["availability"] == "observed" and value["text"] == text)
                        if size == 512 * 1024 else
                        (value["availability"] == "unavailable" and
                         value["reason"]["code"] == "too_large" and value.get("text") is None),
                        "closed_independent_exact_byte_limit_" + authority + "_" + name)
            require(result["snapshot"]["comparisons"]["disk_resource"] ==
                    ("different" if d_size == expected_r == 512 * 1024 else "unknown"),
                    "closed_limit_retains_only_valid_comparison_" + name)
        unloaded_project = self.fixture("closed_cap_unloaded", controlled=True)
        unloaded_editor = self.start_editor(unloaded_project)
        unloaded_descriptor = wait_for(lambda: self.descriptors(unloaded_project),
                                       "closed_unloaded_cap_advertisement")[0]
        unloaded_shot = self.screenshot(unloaded_editor, "closed_uncached_d_over.png")
        cold_path = unloaded_project / "scripts" / "cold.gd"
        cold_path.write_bytes(b"# " + b"d" * (512 * 1024 - 1))
        require(not self.action(unloaded_editor, "witness")["cold_cached_id"],
                "closed_overlimit_native_resource_unloaded")
        result, before, disk = self.story_read(
            unloaded_project, unloaded_editor, unloaded_descriptor,
            "closed_uncached_d_over", "not_open", 0,
            script="res://scripts/cold.gd", disk_path=cold_path, shot=unloaded_shot)
        snapshot = result["snapshot"]
        require(not before["cold_cached_id"] and disk["size"] == 512 * 1024 + 1 and
                snapshot["document"]["open_state"]["value"] == "not_open" and
                snapshot["sources"]["D"]["availability"] == "unavailable" and
                snapshot["sources"]["D"]["reason"]["code"] == "too_large" and
                snapshot["sources"]["R"]["availability"] == "unavailable" and
                snapshot["sources"]["R"]["reason"]["code"] == "resource_not_loaded" and
                snapshot["sources"]["B"]["availability"] == "not_applicable" and
                snapshot["dirty"]["availability"] == "not_applicable",
                "closed_uncached_D_limit_remains_not_open")
        unknown_project, unknown_editor, unknown_descriptor, unknown_path = \
            self.controlled_editor("surface-unknown-limit")
        self.action(unknown_editor, "restrict_open")
        # Fixture-only disk preparation precedes the observation. No observer writes.
        unknown_path.write_bytes(b"# " + b"d" * (512 * 1024 - 1))
        result, _, disk = self.story_read(
            unknown_project, unknown_editor, unknown_descriptor, "unknown_open_d_over",
            "limited_observation", 2, disk_path=unknown_path)
        snapshot = result["snapshot"]
        require(disk["size"] == 512 * 1024 + 1 and
                snapshot["document"]["open_state"].get("value") is None and
                snapshot["sources"]["D"]["availability"] == "unavailable" and
                snapshot["sources"]["D"]["reason"]["code"] == "too_large" and
                snapshot["sources"]["B"]["availability"] != "not_applicable" and
                snapshot["dirty"]["availability"] != "not_applicable",
                "unknown_open_limit_is_not_closed_or_refusal")
        # Separate whole-request failure and known loss/silence outrank limits.
        outside = self.work / "source-beyond-project.gd"
        outside.write_text("extends RefCounted\n# OUTSIDE_SURFACE_LIMIT_PROJECT\n")
        self.source_markers.add(b"OUTSIDE_SURFACE_LIMIT_PROJECT")
        (project / "scripts" / "unsafe.gd").symlink_to(outside)
        denial, _, _ = self.story_read(
            project, editor, descriptor, "separate_scope_denial", "denied_access", 3,
            script="res://scripts/unsafe.gd")
        require(denial["snapshot"] is None and denial["resolved_target"] is None,
                "scope_denial_source_free")
        for name, action, expected in (
                ("separate_known_disconnection", "terminate", "disconnected_editor"),
                ("separate_silent_timeout", "suspend", "timeout")):
            interrupted_project, interrupted_editor, interrupted_descriptor, _ = \
                self.controlled_editor(name)
            result = self.interrupted_observation(
                interrupted_project, interrupted_editor, interrupted_descriptor,
                "observe", action, name, expected)
            require(result["outcome"] == expected and
                    (result["snapshot"] is None or
                     result["snapshot"]["consistency"]["checks"] != "performed"),
                    "stronger_terminal_not_masked_by_limits_" + name)

    def executor_boundary(self):
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

    def rebound_port(self, project, descriptor, *, source_bearing=False):
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
                    self.secrets.add(hello[5].encode())
                    nonce = secrets.token_hex(32)
                    self.secrets.add(nonce.encode())
                    fake = {"v": 1, "kind": "challenge", "request_id": hello[2], "session_id": hello[3],
                            "project_root": hello[4], "godot_version": VERSION, "engine_hash": ENGINE_HASH,
                            "capabilities": {name: False for name in CAPABILITIES}, "client_nonce": hello[5],
                            "server_nonce": nonce, "server_proof": "0" * 64}
                    self.secrets.add(fake["server_proof"].encode())
                    stream.sendall(packet(fake))
                    evidence["closed_before_client_auth"] = not stream.recv(1)
            except (OSError, EOFError, Failure):
                evidence["peer_failed"] = True
        thread = threading.Thread(target=impostor, daemon=True)
        thread.start()
        try:
            if source_bearing:
                result = self.observe(project, "denied_access", 3, session=descriptor["session_id"],
                                      name="source_bearing_rebound_impostor")
                require(result["diagnostics"][0]["code"] == "authentication_failed" and
                        result["resolved_target"] is None and result["snapshot"] is None,
                        "source_bearing_impostor_authentication_failure")
            else:
                result = self.route(project, "DeniedAccess", descriptor["session_id"],
                                    "stale_descriptor_rebound_impostor")
                require(result["code"] == "AuthenticationFailed", "impostor_specific_authentication_failure")
            thread.join(timeout=5)
            require(not thread.is_alive() and evidence == {"secret_absent": True, "closed_before_client_auth": True},
                    "impostor_never_receives_secret_or_client_auth")
        finally:
            listener.close()
            path.unlink(missing_ok=True)

    def distinct_project(self, parent, tag, *, controlled=False):
        project = self.fixture(parent + "/same-name", controlled=controlled)
        path = project / "scripts" / "subject.gd"
        original = path.read_bytes()
        require(original.count(SOURCE_SENTINEL) == 1, "fixture_route_sentinel")
        path.write_bytes(original.replace(SOURCE_SENTINEL, ("ROUTE_PROJECT_" + tag).encode()))
        return project

    def prepared_route_editor(self, project, tag):
        existing = {value["session_id"] for value in self.descriptors(project)}
        editor = self.start_editor(project)
        def new_descriptor():
            candidates = [value for value in self.descriptors(project)
                          if value["session_id"] not in existing]
            require(len(candidates) <= 1, "one_owned_new_lifetime_" + tag)
            return candidates[0] if candidates else None
        descriptor = wait_for(new_descriptor, "route_advertisement_" + tag)
        self.action(editor, "prepare_subject")
        wait_for(lambda: (lambda w: w if w["subject"]["associated"] else None)(
                 self.action(editor, "witness")), "route_document_ready_" + tag)
        self.screenshot(editor, "route-" + tag + "-prepared-" + str(self.counter) + ".png")
        prepared = self.action(editor, "route_session_" + tag.lower())
        require(prepared["associated"] and prepared["R"] ==
                "extends RefCounted\n# ROUTE_RESOURCE_" + tag + "\n" and
                prepared["B"] == "var =\n# ROUTE_BUFFER_" + tag + "\n",
                "route_native_distinct_authorities_" + tag)
        shot = self.screenshot(editor, "route-" + tag + "-" + str(self.counter) + ".png")
        witness = self.action(editor, "witness")
        require(witness["subject"]["R"] == prepared["R"] and
                witness["subject"]["B"] == prepared["B"] and
                witness["current_script"] == "res://scripts/subject.gd",
                "route_native_ready_" + tag)
        return editor, descriptor, shot

    def assert_route(self, result, project, editor, descriptor, *, name):
        path = project / "scripts" / "subject.gd"
        disk = disk_witness(path)
        witness = self.action(editor, "witness")
        self.compare_snapshot(result, descriptor, witness, disk, path)
        require(result["selection"] is None and result["resolved_target"] ==
                result["snapshot"]["target"] and result["resolved_target"]["project_root"] ==
                str(project.resolve()) and result["snapshot"]["dirty"]["state"] ==
                ("dirty" if witness["subject"]["dirty"] else "clean"),
                "route_exact_attributed_target_" + name)
        evidence = name + "-witness.json"
        json_file(self.artifacts / evidence, {"disk": disk, "witness": witness, "result": result})
        self.cases[-1].update(evidence=evidence, session_id=descriptor["session_id"],
                              project_root=str(project.resolve()), disk_sha256=disk["sha256"])

    def routing(self):
        self.compile_window_probe()
        first_project = self.distinct_project("routing-a", "A")
        second_project = self.distinct_project("routing-b", "B")
        require(first_project.name == second_project.name and first_project != second_project,
                "distinct_real_namesake_projects")
        first, one, first_shot = self.prepared_route_editor(first_project, "A")
        second, two, second_shot = self.prepared_route_editor(second_project, "B")
        require(one["session_id"] != two["session_id"] and
                first_project.joinpath("scripts/subject.gd").read_bytes() !=
                second_project.joinpath("scripts/subject.gd").read_bytes(), "distinct_namesake_lifetimes")
        for project, editor, descriptor, shot, name in (
                (first_project, first, one, first_shot, "namesake_project_a_omitted"),
                (second_project, second, two, second_shot, "namesake_project_b_exact")):
            before = self.action(editor, "witness")
            disk_before = disk_witness(project / "scripts" / "subject.gd")
            result = self.observe(project, "complete_observation", 0, name=name,
                                  session=None if name.endswith("omitted") else descriptor["session_id"])
            self.assert_route(result, project, editor, descriptor, name=name)
            require(before == self.action(editor, "witness") and
                    disk_before == disk_witness(project / "scripts" / "subject.gd"),
                    "routing_read_only_" + name)
            self.cases[-1]["screenshot"] = shot
        third, three, third_shot = self.prepared_route_editor(first_project, "C")
        require(len(self.descriptors(first_project)) == 2 and
                one["session_id"] != three["session_id"], "two_real_same_project_sessions")
        before_a, before_c = self.action(first, "witness"), self.action(third, "witness")
        disk_before = disk_witness(first_project / "scripts" / "subject.gd")
        ambiguous = self.observe(first_project, "ambiguous_target", 3, name="same_project_ambiguity")
        require(ambiguous["resolved_target"] is None and ambiguous["snapshot"] is None and
                ambiguous["selection"] is not None and
                ambiguous["selection"]["missing_selector"] == "session_id" and
                set(ambiguous["selection"]["candidate_sessions"]) ==
                {one["session_id"], three["session_id"]},
                "ambiguous_requires_session_and_discloses_no_source")
        for editor, descriptor, shot, name in ((first, one, first_shot, "same_project_exact_a"),
                                               (third, three, third_shot, "same_project_exact_c")):
            result = self.observe(first_project, "complete_observation", 0,
                                  session=descriptor["session_id"], name=name)
            self.assert_route(result, first_project, editor, descriptor, name=name)
            self.cases[-1]["screenshot"] = shot
        require(before_a == self.action(first, "witness") and
                before_c == self.action(third, "witness") and
                disk_before == disk_witness(first_project / "scripts" / "subject.gd"),
                "no_focus_or_source_mixing")
        for result, forbidden in (
                (self.artifacts.joinpath("namesake_project_a_omitted.json").read_bytes(),
                 (b"ROUTE_PROJECT_B", b"ROUTE_RESOURCE_B", b"ROUTE_BUFFER_B")),
                (self.artifacts.joinpath("namesake_project_b_exact.json").read_bytes(),
                 (b"ROUTE_PROJECT_A", b"ROUTE_RESOURCE_A", b"ROUTE_BUFFER_A")),
                (self.artifacts.joinpath("same_project_exact_a.json").read_bytes(),
                 (b"ROUTE_RESOURCE_C", b"ROUTE_BUFFER_C")),
                (self.artifacts.joinpath("same_project_exact_c.json").read_bytes(),
                 (b"ROUTE_RESOURCE_A", b"ROUTE_BUFFER_A"))):
            require(all(marker not in result for marker in forbidden),
                    "unselected_project_or_session_absent_from_result")

    def wait_barrier(self, editor, stage, process):
        def reached():
            require(process.poll() is None, "caller_exited_before_" + stage)
            path = editor["control"] / "event.json"
            if path.is_file():
                result = json.loads(path.read_text())
                return result if result == {"stage": stage} else None
            require(editor["process"].poll() is None, "editor_exited_before_" + stage)
            return None
        return wait_for(reached, "fixture_observation_barrier_" + stage, timeout=3.5)

    def interrupted_observation(self, project, editor, descriptor, stage, action, name, expected):
        self.action(editor, "hold_" + stage)
        event = editor["control"] / "event.json"
        event.unlink(missing_ok=True)
        path = project / "scripts" / "subject.gd"
        disk = disk_witness(path)
        before = self.action(editor, "witness")
        shot = self.screenshot(editor, name + ".png")
        started = time.monotonic()
        process = subprocess.Popen(self.observation_command(project, descriptor["session_id"]),
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        try:
            barrier = self.wait_barrier(editor, stage, process)
            if action == "terminate":
                editor["process"].terminate()
                editor["process"].wait(timeout=2)
            elif action == "disable":
                self.action(editor, "disable")
            elif action == "suspend":
                editor["process"].send_signal(signal.SIGSTOP)
                editor["suspended"] = True
            else:
                raise Failure("unsupported_fixture_interruption")
            stdout, stderr = process.communicate(timeout=max(0.1, 5.1 - (time.monotonic() - started)))
            result = self.observer_result(process.returncode, stdout, stderr, time.monotonic() - started,
                                          expected, 4, name)
        finally:
            if process.poll() is None:
                process.kill()
                process.communicate(timeout=2)
            if editor["suspended"]:
                editor["process"].send_signal(signal.SIGCONT)
                editor["suspended"] = False
        require(result["resolved_target"] is not None and
                result["resolved_target"]["session_id"] == descriptor["session_id"] and
                result["resolved_target"]["project_root"] == str(project.resolve()) and
                result["resolved_target"]["script_path"] == "res://scripts/subject.gd" and
                result["selection"] is None and disk == disk_witness(path),
                "interrupted_original_authenticated_target_" + name)
        if stage == "observe":
            require(result["snapshot"] is None or
                    all(value["availability"] != "observed" for value in
                        result["snapshot"]["sources"].values()),
                    "interruption_before_sample_cannot_claim_source_" + name)
        else:
            snapshot = result["snapshot"]
            require(snapshot is not None and snapshot["target"] == result["resolved_target"] and
                    snapshot["sources"]["D"]["availability"] == "observed" and
                    snapshot["sources"]["D"]["text"] == disk["text"] and
                    snapshot["consistency"]["checks"] == "unavailable" and
                    snapshot["consistency"]["recheck_reason"] == (
                        "session_ended" if expected == "disconnected_editor" else "deadline_exceeded"),
                    "validated_partial_keeps_original_attribution_without_complete_" + name)
            require(snapshot["sources"]["D"]["witness"]["disk_file_id"] ==
                    {"device": disk["device"], "inode": disk["inode"]} and
                    snapshot["document"]["identity"]["script_instance_id"] ==
                    before["subject"]["script_id"] and
                    snapshot["document"]["identity"]["buffer_instance_id"] ==
                    before["subject"]["buffer_id"],
                    "partial_document_and_disk_witnesses_" + name)
            if expected == "disconnected_editor":
                require(all(snapshot["sources"][surface]["availability"] == "unavailable" and
                            snapshot["sources"][surface]["reason"]["code"] == "session_ended" and
                            snapshot["sources"][surface]["invalidated_evidence"]["text"] ==
                            before["subject"][surface] for surface in ("R", "B")) and
                        snapshot["dirty"]["invalidated_evidence"]["state"] ==
                        ("dirty" if before["subject"]["dirty"] else "clean") and
                        snapshot["document"]["open_state"]["invalidated_evidence"]["value"] ==
                        "open" and snapshot["comparisons"]["disk_resource"] == "unknown" and
                        snapshot["comparisons"]["disk_buffer"] == "unknown",
                        "known_loss_invalidates_original_live_facts_" + name)
                for surface, identity in (("R", "script_instance_id"),
                                          ("B", "buffer_instance_id")):
                    prior = snapshot["sources"][surface]["invalidated_evidence"]
                    require(prior["witness"][identity] == before["subject"][
                        "script_id" if surface == "R" else "buffer_id"] and
                        prior["collection"]["clock_id"] == "editor:" + descriptor["session_id"],
                        "invalidated_source_keeps_original_witness_" + surface + "_" + name)
            else:
                require(all(snapshot["sources"][surface]["text"] == before["subject"][surface]
                            for surface in ("R", "B")) and
                        snapshot["document"]["open_state"]["value"] == "open",
                        "silent_interruption_retains_original_sample_" + name)
            evidence = name + "-witness.json"
            json_file(self.artifacts / evidence,
                      {"disk": disk, "before": before, "result": result, "barrier": barrier})
            self.cases[-1]["evidence"] = evidence
        self.cases[-1].update(barrier=stage, action=action, screenshot=shot,
                              session_id=descriptor["session_id"])
        return result

    def session_loss(self):
        self.compile_window_probe()
        absent = self.fixture("session-loss-absent")
        self.observe(absent, "editor_unavailable", 3, name="absent_project_no_autostart")
        require(not self.descriptors(absent), "absence_does_not_launch_editor")
        project, editor, descriptor, path = self.controlled_editor("session-loss")
        self.observe(project, "editor_unavailable", 3, session="f" * 32,
                     name="absent_exact_before_auth")
        self.interrupted_observation(project, editor, descriptor, "observe", "disable",
                                     "disconnect_after_auth_before_sample", "disconnected_editor")
        wait_for(lambda: not self.descriptors(project), "disabled_descriptor_removed")
        self.action(editor, "enable")
        replacement = wait_for(lambda: next(iter(self.descriptors(project)), None),
                               "reenabled_new_session")
        require(replacement["session_id"] != descriptor["session_id"] and
                replacement["token"] != descriptor["token"], "reenable_new_lifetime_and_secret")
        self.observe(project, "editor_unavailable", 3, session=descriptor["session_id"],
                     name="disabled_id_never_substituted")
        self.action(editor, "release_hold")
        self.interrupted_observation(project, editor, replacement, "recheck", "terminate",
                                     "disconnect_after_sample_and_disk", "disconnected_editor")
        self.close_editor(editor)
        # An unclean process termination can leave a descriptor; it is not
        # liveness evidence and must not authorize a replacement session.
        self.observe(project, "editor_unavailable", 3, session=replacement["session_id"],
                     name="ended_process_not_metadata_is_authority")
        self.cases[-1]["stale_descriptor_retained"] = (
            self.registry / (replacement["session_id"] + ".json")).exists()
        restarted, fresh, shot = self.prepared_route_editor(project, "B")
        require(fresh["session_id"] != replacement["session_id"] and
                fresh["token"] != replacement["token"], "restart_new_session_lifetime")
        self.observe(project, "editor_unavailable", 3, session=replacement["session_id"],
                     name="ended_id_never_substituted")
        result = self.observe(project, "complete_observation", 0,
                              session=fresh["session_id"], name="new_session_only_new_request")
        self.assert_route(result, project, restarted, fresh, name="new_session_only_new_request")
        self.cases[-1]["screenshot"] = shot

    def deadline(self):
        self.compile_window_probe()
        project, editor, descriptor, path = self.controlled_editor("deadline")
        self.interrupted_observation(project, editor, descriptor, "recheck", "suspend",
                                     "connected_silent_editor_after_sample_disk", "timeout")
        require(editor["process"].poll() is None, "timeout_does_not_kill_owned_editor")
        self.action(editor, "release_hold")
        disk = disk_witness(path)
        before = self.action(editor, "witness")
        shot = self.screenshot(editor, "isolated-worker.png")
        self.action(editor, "hold_recheck")
        event = editor["control"] / "event.json"
        event.unlink(missing_ok=True)
        started = time.monotonic()
        process = subprocess.Popen(self.observation_command(project, descriptor["session_id"]),
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        owned_worker = None
        try:
            barrier = self.wait_barrier(editor, "recheck", process)
            def child_pid():
                children = run(["/usr/bin/pgrep", "-P", process.pid])
                values = children.stdout.split()
                require(len(values) <= 1, "one_owned_worker_for_live_recheck")
                return int(values[0]) if values else None
            owned_worker = wait_for(child_pid, "live_worker_running_at_recheck", timeout=0.5)
            os.kill(owned_worker, signal.SIGSTOP)
            stdout, stderr = process.communicate(timeout=max(0.1, 5.1 - (time.monotonic() - started)))
            result = self.observer_result(process.returncode, stdout, stderr, time.monotonic() - started,
                                          "timeout", 4, "isolated_worker_after_sample_disk")
            snapshot = result["snapshot"]
            require(result["resolved_target"]["session_id"] == descriptor["session_id"] and
                    snapshot is not None and snapshot["target"] == result["resolved_target"] and
                    snapshot["sources"]["D"]["text"] == disk["text"] and
                    snapshot["sources"]["R"]["text"] == before["subject"]["R"] and
                    snapshot["sources"]["B"]["text"] == before["subject"]["B"] and
                    snapshot["consistency"]["checks"] != "performed" and
                    editor["process"].poll() is None and disk == disk_witness(path),
                    "stalled_worker_preserves_real_editor_and_partial_evidence")
            def gone():
                try:
                    os.kill(owned_worker, 0)
                    return False
                except ProcessLookupError:
                    return True
            wait_for(gone, "bounded_supervisor_reaps_owned_worker", timeout=2)
            evidence = "isolated-worker-witness.json"
            json_file(self.artifacts / evidence,
                      {"before": before, "disk": disk, "result": result, "barrier": barrier})
            self.cases[-1].update(evidence=evidence, screenshot=shot,
                                  barrier="recheck", worker_reaped=True)
        finally:
            if process.poll() is None:
                process.kill()
                process.communicate(timeout=2)
            if owned_worker is not None:
                try:
                    os.kill(owned_worker, signal.SIGKILL)
                except ProcessLookupError:
                    pass
            self.action(editor, "release_hold")
        # A caller-initiated attempt has fresh identity/evidence after timeout.
        fresh = self.observe(project, "complete_observation", 0,
                             session=descriptor["session_id"], name="fresh_after_deadline")
        self.compare_snapshot(fresh, descriptor, self.action(editor, "witness"), disk, path)
        require(fresh["request_id"] != result["request_id"],
                "explicit_new_request_after_timeout")

    def confinement(self):
        self.compile_window_probe()
        self.registry_refusal_cases()
        project, editor, descriptor, path = self.controlled_editor("confinement")
        shot = self.screenshot(editor, "confinement.png")
        original = disk_witness(path)
        before = self.action(editor, "witness")
        self.authentication_cases(descriptor, editor)
        for mode in ("reject_sample_identity", "reject_recheck_identity",
                     "reject_recheck_malformed", "reject_recheck_oversized"):
            self.action(editor, mode)
            result = self.observe(project, "protocol_error", 4, session=descriptor["session_id"],
                                  name="protocol_" + mode)
            require(result["resolved_target"]["session_id"] == descriptor["session_id"] and
                    result["selection"] is None, "protocol_error_retains_selected_identity_" + mode)
            if mode == "reject_sample_identity":
                require(result["snapshot"] is None,
                        "invalid_sample_identity_never_supplies_source")
            else:
                partial = result["snapshot"]
                require(partial is not None and partial["sources"]["D"]["text"] == original["text"] and
                        partial["sources"]["R"]["text"] == before["subject"]["R"] and
                        partial["sources"]["B"]["text"] == before["subject"]["B"] and
                        partial["consistency"]["checks"] == "unavailable",
                        "rejected_recheck_preserves_only_earlier_validated_evidence_" + mode)
            self.action(editor, "restore_dirty")
        for name, locator in (("parent_traversal", "res://../outside.gd"),
                              ("absolute_path", str(self.work / "outside.gd")),
                              ("user_scheme", "user://outside.gd"),
                              ("remote_url", "https://example.invalid/script.gd"),
                              ("decorated_path", "res://scripts/subject.gd?outside")):
            result = self.observe(project, "invalid_request", 3, session=descriptor["session_id"],
                                  name="refuse_" + name, script=locator)
            require(result["resolved_target"] is None and result["snapshot"] is None,
                    "invalid_locator_never_selects_source_" + name)
        outside = self.work / "outside.gd"
        outside.write_text("extends RefCounted\n# OUTSIDE_CONFIDENTIAL\n")
        self.source_markers.add(b"OUTSIDE_CONFIDENTIAL")
        alias = project / "scripts" / "escaped.gd"
        alias.symlink_to(outside)
        try:
            result = self.observe(project, "denied_access", 3, session=descriptor["session_id"],
                                  name="refuse_live_symlink_escape", script="res://scripts/escaped.gd")
            require(result["snapshot"] is None and result["resolved_target"] is None and
                    any(item["code"] == "out_of_project" for item in result["diagnostics"]),
                    "symlink_refusal_before_authenticated_source")
        finally:
            alias.unlink(missing_ok=True)
        stream, request_id = self.authenticated_peer(descriptor)
        with stream:
            for name, field, value in (("wrong_request", 2, "other-request"),
                                       ("wrong_session", 3, "f" * 32),
                                       ("wrong_project", 4, str(outside))):
                if name != "wrong_request":
                    stream.close()
                    stream, request_id = self.authenticated_peer(descriptor)
                message = [1, "observe", request_id, descriptor["session_id"],
                           descriptor["project_root"], "res://scripts/subject.gd"]
                message[field] = value
                self.refused(stream, packet(message))
                self.case("authenticated_refuse_" + name, screenshot=shot)
        for name, payload in (("malformed", b"\x00\x00\x00\x03!!!"),
                              ("oversized", struct.pack(">I", 4097))):
            with self.authenticated_peer(descriptor)[0] as peer:
                self.refused(peer, payload)
            self.case("authenticated_refuse_" + name, screenshot=shot)
        fake = dict(descriptor)
        fake["session_id"] = "e" * 32 if descriptor["session_id"] != "e" * 32 else "d" * 32
        fake_path = self.registry / (fake["session_id"] + ".json")
        for mode, expected, code in (("unsafe_mode", "denied_access", "unsafe_registry"),
                                      ("malformed_secret", "protocol_error", "invalid_frame")):
            if mode == "malformed_secret":
                fake["token"] = "not-a-secret"
            json_file(fake_path, fake)
            if mode == "unsafe_mode":
                fake_path.chmod(0o644)
            try:
                result = self.observe(project, expected, 3 if expected == "denied_access" else 4,
                                      session=descriptor["session_id"], name="refuse_" + mode)
                require(result["snapshot"] is None and result["resolved_target"] is None and
                        any(item["code"] == code for item in result["diagnostics"]),
                        "unsafe_metadata_blocks_source_" + mode)
            finally:
                fake_path.unlink(missing_ok=True)
        extras = []
        try:
            for index in range(32):
                fake["session_id"] = format(index, "032x")
                if fake["session_id"] == descriptor["session_id"]:
                    fake["session_id"] = "f" * 32
                fake["token"] = descriptor["token"]
                candidate = self.registry / (fake["session_id"] + ".json")
                require(not candidate.exists(), "capacity_unique_descriptor")
                json_file(candidate, fake)
                extras.append(candidate)
            result = self.observe(project, "unsupported_observation", 2, session=descriptor["session_id"],
                                  name="refuse_capacity_whole_request")
            require(result["snapshot"] is None and result["resolved_target"] is None,
                    "capacity_never_selects_convenient_candidate")
        finally:
            for candidate in extras:
                candidate.unlink(missing_ok=True)
        second = self.start_editor(project)
        wait_for(lambda: len(self.descriptors(project)) == 2, "confinement_two_live_descriptors")
        second["process"].send_signal(signal.SIGSTOP)
        second["suspended"] = True
        try:
            unresolved = self.observe(project, "timeout", 4, name="unresolved_liveness_no_guess")
            require(unresolved["snapshot"] is None and unresolved["resolved_target"] is None and
                    unresolved["selection"] is None, "unresolved_liveness_source_free")
        finally:
            second["process"].send_signal(signal.SIGCONT)
            second["suspended"] = False
        self.close_editor(second)
        wait_for(lambda: len(self.descriptors(project)) == 1, "confinement_second_session_ended")
        self.action(editor, "hold_recheck")
        event = editor["control"] / "event.json"
        event.unlink(missing_ok=True)
        started = time.monotonic()
        process = subprocess.Popen(self.observation_command(project, descriptor["session_id"]),
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        backup = project / "scripts" / "subject-original.gd"
        try:
            barrier = self.wait_barrier(editor, "recheck", process)
            path.rename(backup)
            path.symlink_to(outside)
            displaced = disk_witness(backup)
            self.action(editor, "release_hold")
            stdout, stderr = process.communicate(timeout=max(0.1, 5.1 - (time.monotonic() - started)))
            refused = self.observer_result(process.returncode, stdout, stderr,
                                           time.monotonic() - started, "denied_access", 3,
                                           "refuse_symlink_after_validated_sample_disk")
            require(refused["snapshot"] is None and refused["resolved_target"] is None and
                    any(item["code"] == "out_of_project" for item in refused["diagnostics"]),
                    "post_sample_symlink_race_suppresses_all_source")
            self.cases[-1].update(barrier=barrier["stage"], screenshot=shot)
            require(displaced == disk_witness(backup),
                    "confined_observer_preserves_displaced_source")
        finally:
            if process.poll() is None:
                process.kill()
                process.communicate(timeout=2)
            if path.is_symlink():
                path.unlink()
            if backup.exists():
                backup.rename(path)
            self.action(editor, "release_hold")
        restored = disk_witness(path)
        after = self.action(editor, "witness")
        evidence = "confinement-race-witness.json"
        json_file(self.artifacts / evidence,
                  {"original_disk": original, "restored_disk": restored,
                   "before": before, "after": after, "result": refused})
        self.cases[-1]["evidence"] = evidence
        # Fixture renames change ctime. Source, identity and write metadata must
        # survive, and the observer must not change any native editor witness.
        require(all(original[key] == restored[key] for key in
                    ("text", "sha256", "size", "device", "inode", "mtime_ns", "mode")),
                "confinement_denials_preserve_original_disk")
        require(before == after, "confinement_denials_preserve_original_editor")
        self.close_editor(editor)
        wait_for(lambda: not self.descriptors(project), "confinement_ended_session_cleanup")
        self.rebound_port(project, descriptor, source_bearing=True)

    def redaction(self, *, replay=True):
        if replay:
            for name in REDACTION_REPLAY:
                self.group(name, getattr(self, name.replace("-", "_")))
        require(all(name in self.summary["groups"] and
                    self.summary["groups"][name]["case_count"] > 0
                    for name in REDACTION_REPLAY),
                "redaction_full_success_and_failure_replay")
        self.summary["redaction_replay"] = {
            "groups": {name: self.summary["groups"][name]["case_count"]
                       for name in REDACTION_REPLAY},
            "reused_in_all": not replay}
        boundary_start = len(self.cases)
        self.session_boundary()
        self.executor_boundary()
        self.summary["redaction_boundary_cases"] = [
            case["case"] for case in self.cases[boundary_start:]]
        require(bool(self.summary["redaction_boundary_cases"]),
                "redaction_executed_session_and_executor_boundaries")
        require(any(case["case"] == "source_bearing_rebound_impostor" and
                    case["outcome"] == "denied_access" for case in self.cases) and
                any(case["case"] == "same_project_ambiguity" and
                    case["outcome"] == "ambiguous_target" for case in self.cases) and
                any(case["case"] == "stale_descriptor_rebound_impostor" for case in self.cases) and
                any(case["case"] == "live_reject_wrong_secret" for case in self.cases) and
                any(case["case"] == "stalled_worker" and
                    case["outcome"] == "timeout" for case in self.cases) and
                any(case["case"] == "sequence_native_prior_history_replayed"
                    for case in self.cases),
                "redaction_real_auth_stale_endpoint_executor_and_sequence_evidence")

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
            path = name.removeprefix("res://")
            if ("godot_agent_kit" in path or "fixture_driver" in path or
                    path.startswith(("container.tscn", "scripts/invalid.gd"))):
                raise Failure(stage + "_tooling_material:" + path)
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
    parser.add_argument("--scenario", required=True,
                        choices=("all", *QUICKSTART_GROUPS, *BOUNDARY_GROUPS))
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
    # Home is deliberately used instead of /tmp's symlink/writable ancestry.
    with tempfile.TemporaryDirectory(prefix=".godot-agent-kit-acceptance-", dir=Path.home()) as temporary:
        harness = Harness(args, Path(temporary))
        status = 0
        try:
            harness.initialize()
            if args.scenario == "all":
                for name in REDACTION_REPLAY:
                    harness.group(name, getattr(harness, name.replace("-", "_")))
                harness.group("redaction", lambda: harness.redaction(replay=False))
                harness.group("export-boundary", harness.export_boundary)
                require(set(harness.summary["groups"]) == set(QUICKSTART_GROUPS) and
                        all(harness.summary["groups"][name]["case_count"] > 0
                            for name in QUICKSTART_GROUPS),
                        "all_thirteen_quickstart_groups_exercised")
            elif args.scenario == "redaction":
                harness.group("redaction", harness.redaction)
            else:
                harness.group(args.scenario,
                              getattr(harness, args.scenario.replace("-", "_")))
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
            if args.scenario in ("all", "redaction"):
                try:
                    harness.verify_incidental_redaction()
                except (Failure, OSError) as error:
                    status = 1
                    harness.summary["status"] = "failed"
                    harness.summary["redaction_stage"] = (str(error) if isinstance(error, Failure)
                                                          else type(error).__name__)
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
