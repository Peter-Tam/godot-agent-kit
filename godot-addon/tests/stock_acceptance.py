"""Actual official-stock one-shot validator acceptance in owned GUI projects."""
import json
import os
import re
from pathlib import Path
import secrets
import signal
import socket
import subprocess
import time

import run_observation as observation

ROOT = "res://scripts/subject.gd"
DIRECT = 'extends "./native/level_one.gd"\nconst NATIVE_DIRECT := 1\n'
NESTED = 'extends "./native/deep/level_two.gd"\nconst NATIVE_TRANSITIVE := 2\n'
INVALID = 'extends RefCounted\nvar = # STOCK_PRIVATE_PARSER_SOURCE\n'
WARNING = ('extends RefCounted\nfunc value() -> int:\n'
           '\treturn 3 / 2 # STOCK_PRIVATE_WARNING_SOURCE\n')


class StockAcceptanceMixin:
    def stock_controlled_run(self, request, peer=None, interruption=None):
        """Control only owned processes; never add hooks to the product worker."""
        observation.require(peer is None or interruption is None, "one_process_control")
        process = subprocess.Popen([str(self.args.stock_validator)], stdin=subprocess.PIPE,
                                   stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
        worker = None
        suspended = False
        connection = None
        started = time.monotonic()
        try:
            process.stdin.write(json.dumps(request, ensure_ascii=False).encode() + b"\n")
            process.stdin.close()
            process.stdin = None

            def children(parent):
                found = observation.run(["/usr/bin/pgrep", "-P", parent], timeout=1)
                observation.require(found.returncode in (0, 1), "owned_worker_child_inspection")
                return [int(value) for value in found.stdout.split()]

            def worker_pid():
                values = children(process.pid)
                observation.require(len(values) <= 1, "single_owned_validation_worker")
                return values[0] if values else None

            worker = observation.wait_for(worker_pid, "owned_validation_worker", timeout=2)

            def engine_pid():
                for pid in children(worker):
                    command = observation.run(["/bin/ps", "-p", pid, "-o", "comm="], timeout=1)
                    if Path(command.stdout.decode().strip()).name == Path(request["binary"]).name:
                        return pid
                return None

            engine = observation.wait_for(engine_pid, "owned_stock_engine_child", timeout=3)
            os.kill(worker, signal.SIGSTOP)
            suspended = True

            def endpoint():
                sockets = observation.run(
                    ["/usr/sbin/lsof", "-nP", "-a", "-p", engine, "-iTCP",
                     "-sTCP:LISTEN", "-Fn"], timeout=1)
                ports = [int(line.rsplit(":", 1)[1]) for line in sockets.stdout.decode().splitlines()
                         if line.startswith("n127.0.0.1:")]
                observation.require(len(ports) <= 1, "unique_owned_stock_listener")
                return ports[0] if ports else None

            port = observation.wait_for(endpoint, "owned_peer_listener", timeout=4)
            working = observation.run(
                ["/usr/sbin/lsof", "-a", "-p", engine, "-d", "cwd", "-Fn"], timeout=1)
            directories = [Path(line[1:]) for line in working.stdout.decode().splitlines()
                           if line.startswith("n")]
            observation.require(len(directories) == 1 and directories[0].name == "project" and
                                directories[0].parent.name.startswith("godot-validation-"),
                                "peer_only_owned_source_clone")
            project = directories[0]
            if interruption is not None:
                observation.require(interruption in ("worker_loss", "deadline"),
                                    "known_owned_interruption")
                if interruption == "worker_loss":
                    os.kill(worker, signal.SIGKILL)
                else:
                    os.kill(engine, signal.SIGSTOP)
                    os.kill(worker, signal.SIGCONT)
                suspended = False
                data, _ = process.communicate(
                    timeout=max(0.1, 10 - (time.monotonic() - started)))
                elapsed = time.monotonic() - started
                observation.require(elapsed <= 10, "interrupted_helper_terminal_deadline")
                observation.json_file(
                    self.artifacts / f"controlled-{request['request_id']}-{engine}.json",
                    {"result": json.loads(data), "owned_child_pid": engine,
                     "owned_private_root": str(project.parent), "caller_seconds": elapsed})

                def engine_gone():
                    try:
                        os.kill(engine, 0)
                        return False
                    except ProcessLookupError:
                        return True

                observation.wait_for(engine_gone, "interrupted_owned_engine_gone", timeout=2)
                observation.wait_for(lambda: not project.parent.exists(),
                                     "interrupted_owned_clone_removed", timeout=2)
                return subprocess.CompletedProcess(process.args, process.returncode, data), {
                    "interruption": interruption, "owned_child_pid": engine,
                    "child_gone": True, "clone_removed": True, "caller_seconds": elapsed}
            path = project / peer["path"].removeprefix("res://")
            observation.require(peer["path"].startswith("res://") and
                                path.is_file() and not path.is_symlink(),
                                "peer_only_existing_staged_uri")
            uri = path.as_uri()
            connection = socket.create_connection(("127.0.0.1", port), timeout=2)
            connection.settimeout(3)
            stream = connection.makefile("rb")

            def send(packet):
                body = json.dumps({"jsonrpc": "2.0", **packet}, ensure_ascii=False).encode()
                connection.sendall(f"Content-Length: {len(body)}\r\n\r\n".encode() + body)

            def receive():
                length = None
                for _ in range(16):
                    line = stream.readline(8193)
                    observation.require(0 < len(line) <= 8192, "bounded_peer_header")
                    if line == b"\r\n":
                        break
                    if line.lower().startswith(b"content-length:"):
                        length = int(line.split(b":", 1)[1])
                observation.require(length is not None and 0 < length <= 2 * 1024 * 1024,
                                    "bounded_peer_response")
                body = stream.read(length)
                observation.require(len(body) == length, "complete_peer_response")
                return json.loads(body)

            send({"id": 1, "method": "initialize", "params": {
                "rootUri": project.as_uri(), "rootPath": str(project),
                "workspaceFolders": [{"uri": project.as_uri(), "name": "Owned peer fixture"}],
                "capabilities": {"textDocument": {"documentSymbol": {
                    "hierarchicalDocumentSymbolSupport": True}}}}})
            for _ in range(128):
                packet = receive()
                if packet.get("id") == 1:
                    observation.require(isinstance(packet.get("result"), dict),
                                        "actual_peer_initialized")
                    break
            else:
                raise observation.Failure("missing_peer_initialize")
            send({"method": "initialized", "params": {}})
            send({"method": "textDocument/didOpen", "params": {"textDocument": {
                "uri": uri, "languageId": "gdscript", "version": 37, "text": peer["source"]}}})
            send({"id": 2, "method": "textDocument/documentSymbol",
                  "params": {"textDocument": {"uri": uri}}})
            diagnostics = None
            for _ in range(128):
                packet = receive()
                if packet.get("method") == "textDocument/publishDiagnostics":
                    if packet.get("params", {}).get("uri") == uri:
                        diagnostics = packet["params"].get("diagnostics")
                if packet.get("id") == 2:
                    symbols = packet.get("result")
                    observation.require(isinstance(diagnostics, list) and
                                        isinstance(symbols, list) and symbols and
                                        isinstance(symbols[0].get("range"), dict),
                                        "actual_peer_diagnostics_then_parser_fence")
                    observation.require(any(d.get("severity") == 1 for d in diagnostics)
                                        is peer["invalid"], "independent_peer_source_verdict")
                    break
            else:
                raise observation.Failure("missing_peer_source_fence")
            os.kill(worker, signal.SIGCONT)
            suspended = False
            data, _ = process.communicate(timeout=max(0.1, 10 - (time.monotonic() - started)))
            observation.require(time.monotonic() - started <= 10, "peer_case_terminal_deadline")
            return subprocess.CompletedProcess(process.args, process.returncode, data), {
                "path": peer["path"], "version": 37, "invalid": peer["invalid"],
                "source_sha256": self.stock_sha(peer["source"]), "owned_child_pid": engine}
        finally:
            if connection is not None:
                connection.close()
            if suspended:
                try:
                    os.kill(worker, signal.SIGCONT)
                except ProcessLookupError:
                    pass
            if worker is not None:
                try:
                    os.killpg(worker, signal.SIGKILL)
                except ProcessLookupError:
                    pass
            if process.poll() is None:
                process.kill()
                process.wait(timeout=2)

    def stock_validate(self, editor, project, name, source, expected, *, purpose="preflight",
                       warnings=None, dependency=None, origin=None, spawn=None, binary=None,
                       peer=None, interruption=None):
        """Invoke a separately built test-only Rust library consumer, not the public caller."""
        if editor["control"] not in self.stock_sessions:
            session = secrets.token_hex(16)
            observation.require(self.native_action(editor, "native_configure",
                                                   session_id=session).get("configured") is True,
                                "owned_editor_warning_session_configured_" + name)
            self.stock_sessions[editor["control"]] = session
        session = self.stock_sessions[editor["control"]]
        selected = self.native_action(editor, "native_warning_info")
        profile = selected["warning_profile"]
        observation.require(selected["session_id"] == session and
                            selected["project_root"] == str(project.resolve()) and
                            isinstance(profile["enable"], bool) and
                            isinstance(profile["levels"], dict) and
                            len(profile["levels"]) == 46 and
                            all(isinstance(value, int) and 0 <= value <= 2
                                for value in profile["levels"].values()) and
                            isinstance(profile["directory_rules"], dict) and
                            isinstance(selected.get("global_classes"), list) and
                            len(selected["global_classes"]) <= 256 and
                            all(isinstance(item, str) for item in selected["global_classes"]),
                            "independent_full_selected_editor_warning_and_global_context_" + name)
        if warnings is not None:
            observation.require(profile["enable"] is warnings["enable"] and
                                all(profile["levels"].get(key) == value for key, value in
                                    warnings.get("levels", {}).items()) and
                                (warnings.get("directory_rules") is None or
                                 profile["directory_rules"] == warnings["directory_rules"]),
                                "selected_effective_warnings_match_fixture_case_" + name)
        full_profile = {**profile, "provenance": {
            "source": "editor_project_settings", "project_root": selected["project_root"],
            "session_id": session}}
        before, disks_before = self.snapshot(editor, project)
        actual = source if source is not None else disks_before["subject"]["text"]
        request = {"binary": str(binary or self.args.godot), "project": str(project),
                   "script": ROOT, "source": source, "purpose": purpose,
                   "request_id": secrets.token_hex(16), "session_id": session,
                   "warnings": full_profile,
                   "global_classes": selected["global_classes"]}
        peer_evidence = None
        try:
            if peer is None and interruption is None:
                output = subprocess.run([str(self.args.stock_validator)], input=json.dumps(
                    request, ensure_ascii=False).encode("utf-8") + b"\n",
                    stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, timeout=10)
            else:
                output, peer_evidence = self.stock_controlled_run(request, peer, interruption)
        except subprocess.TimeoutExpired as error:
            raise observation.Failure("stock_one_shot_deadline_" + name) from error
        observation.require(output.returncode == 0 and len(output.stdout) <= 262144,
                            "bounded_stock_fixture_response_" + name)
        result = json.loads(output.stdout)
        # Retain the bounded requested receipt even when an assertion fails.
        # Raw engine stdout/stderr remain discarded, never captured here.
        observation.json_file(self.artifacts / (name + "-result.json"), result)
        if interruption is not None:
            observation.require(result.get("status") == "unavailable" and
                                result.get("child_spawned") is not False,
                                "lost_worker_cannot_claim_no_engine_was_spawned")
        after, disks_after = self.snapshot(editor, project)
        observation.require(self.source_free_state(before, disks_before) ==
                            self.source_free_state(after, disks_after) and
                            before["subject"]["R"] == after["subject"]["R"] and
                            before["subject"]["B"] == after["subject"]["B"] and
                            before["other"]["R"] == after["other"]["R"] and
                            before["other"]["B"] == after["other"]["B"] and
                            disks_before == disks_after,
                            "stock_helper_changed_independent_target_or_unrelated_work_" + name)
        observed_after = self.native_action(editor, "native_warning_info")
        observation.require(observed_after["warning_profile"] == profile and
                            observed_after["global_classes"] == selected["global_classes"] and
                            observed_after["session_id"] == session and
                            observed_after["project_root"] == selected["project_root"],
                            "stock_helper_did_not_change_selected_effective_context_" + name)
        observation.require(result.get("status") == expected and
                            result.get("request_id") == request["request_id"] and
                            result.get("session_id") == request["session_id"] and
                            result.get("purpose") == purpose and
                            result.get("root_path") == ROOT,
                            "stock_exact_owner_result_binding_" + name)
        spawned = result.get("child_spawned")
        observation.require((spawned is None and expected == "unavailable" and
                             result.get("child_pid") is None and
                             result.get("child_reaped") is None) or
                            (spawned is False and result.get("child_pid") is None) or
                            (spawned is True and result.get("child_reaped") is True and
                             isinstance(result.get("child_pid"), int) and
                             result["child_pid"] > 0),
                            "stock_owned_child_reaped_or_no_child_" + name)
        observation.require(result.get("cleanup_confirmed") is True,
                            "owned_scratch_cleanup_confirmed_before_caller_exit_" + name)
        observation.require(isinstance(result.get("elapsed_us"), int) and
                            0 <= result["elapsed_us"] <= 9_500_000 and
                            result.get("finished_unix_ms", 0) >=
                            result.get("started_unix_ms", 0),
                            "stock_bounded_supervisor_interval_" + name)
        if spawn is not None:
            observation.require(result.get("child_spawned") is spawn,
                                "stock_admission_before_spawn_" + name)
        clone = result.get("clone_path")
        observation.require(clone is None or
                            (isinstance(clone, str) and not Path(clone).exists()),
                            "stock_private_clone_cleaned_" + name)
        launch = result.get("launch_args", [])
        observation.require("--log-file" not in launch and
                            (not result.get("child_spawned") or
                             (launch.count("--headless") == 1 and
                              launch.count("--editor") == 1 and
                              launch.count("--lsp-port") == 1)),
                            "stock_private_no_log_one_shot_launch_" + name)
        if spawned:
            observation.require(result.get("clone_extra_files") is False and
                                result.get("clone_log_files") is False,
                                "stock_clone_has_no_runtime_effect_or_incidental_log_" + name)
        sources = result.get("sources", [])
        observation.require(isinstance(sources, list) and len(sources) <= 33 and
                            len({item["path"] for item in sources}) == len(sources) and
                            all(item["path"].startswith("res://") and
                                re.fullmatch(r"[0-9a-f]{64}", str(item["sha256"])) and
                                0 <= item["utf8_bytes"] <= 524288 for item in sources),
                            "stock_bounded_owner_sources_" + name)
        if expected != "unavailable":
            observation.require(result.get("child_spawned") is True and
                                re.fullmatch(r"[0-9a-f]{64}",
                                             str(result.get("context_sha256", ""))) and
                                all(item.get("diagnostics_completed") is True and
                                    item.get("symbols_completed") is True for item in sources),
                                "complete_per_uri_owned_fences_" + name)
            root = next((item for item in sources if item["path"] == ROOT), None)
            observation.require(root is not None and root["sha256"] == self.stock_sha(actual) and
                                root["utf8_bytes"] == len(actual.encode("utf-8")),
                                "stock_exact_proposed_or_independent_actual_root_" + name)
            for item in sources:
                if item["path"] != ROOT:
                    independent = self.stock_dependency(project, item["path"].removeprefix("res://"))
                    observation.require(item["sha256"] == independent["sha256"] and
                                        item["utf8_bytes"] == independent["size"] and
                                        str(item["device"]) == independent["device"] and
                                        str(item["inode"]) == independent["inode"],
                                        "stock_exact_confined_dependency_" + name)
            if dependency is not None:
                observation.require(any(item["path"] == dependency for item in sources),
                                    "stock_required_dependency_fenced_" + name)
        diagnostics = result.get("diagnostics", [])
        observation.require(isinstance(diagnostics, list) and len(diagnostics) <= 64 and
                            all(isinstance(item.get("path"), str) and
                                item["path"].startswith("res://") and
                                len(item["path"].encode("utf-8")) <= 2048 and
                                isinstance(item.get("message"), str) and
                                len(item["message"].encode("utf-8")) <= 2048
                                for item in diagnostics), "stock_bounded_confined_diagnostics_" + name)
        if expected == "valid":
            observation.require(not diagnostics, "stock_completed_valid_no_errors_" + name)
        if expected == "invalid":
            observation.require(any(item.get("path") == (origin or ROOT) and
                                    item.get("source_sha256") ==
                                    next((s["sha256"] for s in sources if
                                          s["path"] == item.get("path")), None)
                                    for item in diagnostics),
                                "stock_attributed_actual_error_" + name)
        if expected == "unavailable":
            observation.require(isinstance(result.get("reason"), str) and result["reason"],
                                "stock_unavailable_has_reason_" + name)
        self.case(name, mapping="T003/native-integration §4; research §19",
                  validation_status=expected, reason=result.get("reason"), purpose=purpose,
                  independent_peer=peer_evidence,
                  requested_root_sha256=self.stock_sha(actual),
                  context=result.get("context_sha256"),
                  source_fences=[{"path": item["path"], "sha256": item["sha256"],
                                  "diagnostics_completed": item.get("diagnostics_completed"),
                                  "symbols_completed": item.get("symbols_completed")}
                                 for item in sources],
                  diagnostics=[{"path": item["path"], "line": item.get("line"),
                                "column": item.get("column"),
                                "source_sha256": item.get("source_sha256"),
                                "message_sha256": self.stock_sha(item["message"])}
                               for item in diagnostics],
                  child_pid=result.get("child_pid"), child_reaped=result["child_reaped"],
                  before=self.source_free_state(before, disks_before),
                  after=self.source_free_state(after, disks_after),
                  source_surfaces="independent_d_r_b_dirty_versions_history")
        return result

    def stock_warning_case(self, name, *, enable, level, directory_rules=None, status,
                           base_enable=None, base_level=None):
        project = self.fixture("stock-warning-" + name, oracle=False)
        settings = (f"\n[debug]\ngdscript/warnings/enable="
                    f"{'true' if (enable if base_enable is None else base_enable) else 'false'}\n"
                    f"gdscript/warnings/integer_division={level if base_level is None else base_level}\n")
        if base_enable is not None:
            settings += f"gdscript/warnings/enable.editor={'true' if enable else 'false'}\n"
        if base_level is not None:
            settings += f"gdscript/warnings/integer_division.editor={level}\n"
        if directory_rules is not None:
            settings += ("gdscript/warnings/directory_rules=" +
                         json.dumps(directory_rules, separators=(", ", ": ")) + "\n")
        project_file = project / "project.godot"
        project_file.write_text(project_file.read_text() + settings)
        self.installed_native(project)
        editor = self.start_editor(project, configured=False)
        self.action(editor, "prepare_subject")
        self.action(editor, "prepare_other")
        self.action(editor, "dirty_other")
        self.action(editor, "seed_sequence_history")
        self.native_action(editor, "native_idle")
        selected = self.native_action(editor, "native_warning_info")["warning_profile"]
        observation.require(selected["enable"] is enable and
                            selected["levels"]["integer_division"] == level and
                            (directory_rules is None or
                             selected["directory_rules"] == directory_rules),
                            "selected_gui_effective_warning_configuration_" + name)
        required = {"enable": enable, "levels": {"integer_division": level},
                    "directory_rules": directory_rules}
        root = self.stock_validate(editor, project, "stock_warning_" + name,
                                   WARNING, status, warnings=required)
        dependency = project / "scripts/native/cold/warning.gd"
        dependency.write_text(WARNING)
        reference = 'extends RefCounted\nconst UNUSED = preload("./native/cold/warning.gd")\n'
        dependent = self.stock_validate(
            editor, project, "stock_warning_dependency_" + name, reference, status,
            warnings=required, dependency="res://scripts/native/cold/warning.gd",
            origin="res://scripts/native/cold/warning.gd" if status == "invalid" else None)
        observation.require(root["context_sha256"] == dependent["context_sha256"],
                            "same_effective_selected_context_root_and_dependency_" + name)
        return root["context_sha256"]

    def stock_global_context_case(self):
        project = self.fixture("stock-global-context", oracle=False)
        (project / "scripts/ExistingGlobal.gd").write_text(
            "class_name ExistingGlobal\nextends RefCounted\n")
        (project / "scripts/existing_global.gd").write_text(
            "class_name existing_global\nextends RefCounted\n")
        self.installed_native(project)
        editor = self.start_editor(project, configured=False)
        self.action(editor, "prepare_subject")
        self.action(editor, "prepare_other")
        self.action(editor, "dirty_other")
        self.native_action(editor, "native_idle")
        names = self.native_action(editor, "native_warning_info")["global_classes"]
        observation.require("ExistingGlobal" in names and "existing_global" in names,
                            "actual_selected_editor_global_class_registry")
        self.stock_validate(editor, project, "stock_global_shadow_collision_refused",
                            "extends RefCounted\nclass ExistingGlobal extends RefCounted:\n"
                            "\tvar value: int = 0\n", "unavailable", spawn=False)
        self.stock_validate(editor, project, "stock_lowercase_global_reference_refused",
                            "extends RefCounted\nvar item: existing_global\n",
                            "unavailable", spawn=False)
        self.stock_validate(editor, project, "stock_unrelated_global_context_supported",
                            "extends RefCounted\nvar item: int = 1\n", "valid")

    def stock_validation(self):
        self.compile_window_probe()
        project = self.fixture("stock-helper", oracle=False)
        self.installed_native(project)
        editor = self.start_editor(project, configured=False)
        self.action(editor, "prepare_subject")
        self.action(editor, "prepare_other")
        self.action(editor, "dirty_other")
        self.action(editor, "seed_sequence_history")
        self.native_action(editor, "native_idle")
        initial, _ = self.snapshot(editor, project)
        observation.require(initial["subject"]["has_redo"] and initial["other"]["dirty"] and
                            initial["current_script"] == "res://scripts/other.gd",
                            "stock_actual_unrelated_dirty_nonselected_history")
        before_capture = self.screenshot(editor, "stock-helper-before.png")
        self.stock_validate(editor, project, "stock_valid_unicode", 'extends RefCounted\n# café\n',
                            "valid")
        self.stock_validate(editor, project, "stock_empty", "", "valid")
        self.stock_validate(editor, project, "stock_root_parser_error", INVALID, "invalid")
        self.stock_validate(editor, project, "stock_root_analyzer_error",
                            'extends RefCounted\nvar broken: int = "not an int"\n', "invalid")
        self.stock_validate(editor, project, "stock_relative_dependency", DIRECT, "valid",
                            dependency="res://scripts/native/level_one.gd")
        self.stock_validate(editor, project, "stock_transitive_dependency", NESTED, "valid",
                            dependency="res://scripts/native/deep/level_two.gd")
        invalid_peer = {"path": ROOT, "source":
                        'extends RefCounted\nvar PEER_ONLY: int = "wrong"\n', "invalid": True}
        valid_peer = {"path": ROOT, "source":
                      'extends RefCounted\nvar PEER_ONLY: int = 37\n', "invalid": False}
        self.stock_validate(editor, project, "stock_independent_peer_invalid_owner_valid",
                            "extends RefCounted\nvar OWNER_ONLY: int = 1\n", "valid",
                            peer=invalid_peer)
        self.stock_validate(editor, project, "stock_independent_peer_valid_owner_invalid",
                            'extends RefCounted\nvar OWNER_ONLY: int = "wrong"\n', "invalid",
                            peer=valid_peer)
        self.stock_validate(editor, project, "stock_peer_dependency_overlay_owner_disk",
                            DIRECT, "valid", dependency="res://scripts/native/level_one.gd",
                            peer={**invalid_peer, "path": "res://scripts/native/level_one.gd"})
        self.stock_validate(editor, project, "stock_worker_loss_after_engine_spawn",
                            "extends RefCounted\n", "unavailable", interruption="worker_loss")
        self.stock_validate(editor, project, "stock_unresponsive_engine_deadline",
                            "extends RefCounted\n", "unavailable", interruption="deadline")
        sibling = project / "scripts/native/sibling.gd"
        original_sibling = sibling.read_bytes()
        try:
            sibling.write_text('extends RefCounted\nvar broken: int = "bad"\n')
            self.stock_validate(editor, project, "stock_transitive_invalid_dependency", NESTED,
                                "invalid", origin="res://scripts/native/sibling.gd",
                                dependency="res://scripts/native/sibling.gd")
        finally:
            sibling.write_bytes(original_sibling)
        self.stock_validate(editor, project, "stock_transitive_revalidated_after_restore",
                            NESTED, "valid", dependency="res://scripts/native/sibling.gd")
        dependency = project / "scripts/native/level_one.gd"
        previous = dependency.read_bytes()
        try:
            dependency.write_text('extends RefCounted\nvar broken: int = "bad"\n')
            self.stock_validate(editor, project, "stock_same_root_changed_dependency", DIRECT,
                                "invalid", origin="res://scripts/native/level_one.gd",
                                dependency="res://scripts/native/level_one.gd")
        finally:
            dependency.write_bytes(previous)
        self.stock_validate(editor, project, "stock_changed_dependency_fresh_child", DIRECT,
                            "valid", dependency="res://scripts/native/level_one.gd")
        unused = project / "scripts/native/unused.gd"
        unused.write_text('extends RefCounted\nvar broken: int = "bad"\n')
        self.stock_validate(editor, project, "stock_unused_preload_invalid_dependency",
                            'extends RefCounted\nconst UNUSED = preload("./native/unused.gd")\n',
                            "invalid", origin="res://scripts/native/unused.gd",
                            dependency="res://scripts/native/unused.gd")
        same_size_source = 'extends RefCounted\nconst HELPER = preload("./native/same_size.gd")\n'
        same_size = project / "scripts/native/same_size.gd"
        same_size.write_text("extends RefCounted\nvar VALUE: int = 100\n")
        self.stock_validate(editor, project, "stock_unused_same_size_helper_valid",
                            same_size_source, "valid",
                            dependency="res://scripts/native/same_size.gd")
        same_size.write_text('extends RefCounted\nvar VALUE: int = "x"\n')
        observation.require(len("extends RefCounted\nvar VALUE: int = 100\n") ==
                            len('extends RefCounted\nvar VALUE: int = "x"\n'),
                            "real_same_size_dependency_change")
        self.stock_validate(editor, project, "stock_unused_same_size_helper_invalid",
                            same_size_source, "invalid", origin="res://scripts/native/same_size.gd",
                            dependency="res://scripts/native/same_size.gd")
        tool = ('@tool\nextends RefCounted\nstatic var MARKER := _mark()\n'
                'static func _mark() -> int:\n'
                '\tvar output := FileAccess.open("res://runtime-marker.txt", FileAccess.WRITE)\n'
                '\tif output != null:\n\t\toutput.store_string("STATIC_INITIALIZER_EXECUTED")\n'
                '\t\toutput.close()\n\treturn 1\n')
        self.stock_validate(editor, project, "stock_inert_tool_static_initializer", tool, "valid")
        observation.require(not (project / "runtime-marker.txt").exists(),
                            "no_selected_project_tool_initializer_execution")
        stock_tool = project / "scripts/native/cold/stock_tool_initializer.gd"
        stock_tool.write_bytes(
            (Path(__file__).parent / "fixtures/script_edit/stock_tool_initializer.gd").read_bytes())
        self.stock_validate(editor, project, "stock_tool_dependency_source_only",
                            'extends RefCounted\nconst TOOL = preload("./native/cold/stock_tool_initializer.gd")\n',
                            "valid", dependency="res://scripts/native/cold/stock_tool_initializer.gd")
        observation.require(not (project / "runtime-marker-stock-initializer.txt").exists() and
                            not (project / "runtime-marker-stock-constructor.txt").exists() and
                            not (editor["control"] / "excluded_initializer_was_called").exists() and
                            not (editor["control"] / "excluded_constructor_was_called").exists(),
                            "stock_confined_source_only_tool_constructor_and_initializer_inert")
        self.stock_validate(editor, project, "stock_warning_default", WARNING, "valid")
        warning_dependency = project / "scripts/native/cold/warning.gd"
        warning_dependency.write_text(WARNING)
        warning_root = 'extends RefCounted\nconst UNUSED = preload("./native/cold/warning.gd")\n'
        self.stock_validate(editor, project, "stock_dependency_warning_default",
                            warning_root, "valid",
                            dependency="res://scripts/native/cold/warning.gd")
        self.stock_validate(editor, project, "stock_unchanged_fresh_actual_source", None, "valid",
                            purpose="unchanged")
        self.stock_validate(editor, project, "stock_post_change_fresh_actual_source", None, "valid",
                            purpose="post_change")
        self.stock_validate(editor, project, "stock_confined_missing_literal",
                            'extends "./native/absent.gd"\n', "invalid")
        a = project / "scripts/native/a"
        b = project / "scripts/native/b"
        a.mkdir()
        b.mkdir()
        self.stock_validate(editor, project, "stock_distinct_missing_same_basename_unavailable",
                            'extends "./native/a/missing.gd"\n'
                            'const B = preload("./native/b/missing.gd")\n',
                            "unavailable", spawn=True)
        alias = project / "scripts/native/hard_alias.gd"
        alias.hardlink_to(project / "scripts/subject.gd")
        self.stock_validate(editor, project, "stock_root_hardlink_alias_refused_before_spawn",
                            'extends RefCounted\nconst A = preload("./native/hard_alias.gd")\n',
                            "unavailable", spawn=False)
        case_alias = project / "scripts/Subject.gd"
        if case_alias.exists() and case_alias.samefile(project / "scripts/subject.gd"):
            self.stock_validate(editor, project, "stock_root_case_alias_refused_before_spawn",
                                'extends RefCounted\nconst A = preload("Subject.gd")\n',
                                "unavailable", spawn=False)
        self.stock_validate(editor, project, "stock_computed_load_refused_before_launch",
                            'extends RefCounted\nfunc get_file(path: String):\n\treturn load(path)\n',
                            "unavailable", spawn=False)
        self.stock_validate(editor, project, "stock_wrong_binary_hash_before_launch",
                            'extends RefCounted\n', "unavailable", spawn=False,
                            binary=Path("/usr/bin/true"))
        self.stock_validate(editor, project, "stock_outside_refusal_before_launch",
                            'extends "../../../outside.gd"\n', "unavailable", spawn=False)
        self.stock_validate(editor, project, "stock_non_gd_preload_refusal_before_launch",
                            'extends RefCounted\nconst BAD = preload("./native/excluded.tres")\n',
                            "unavailable", spawn=False)
        outside = project.parent / "stock-outside.gd"
        outside.write_text("extends RefCounted\n# OUTSIDE_STOCK_SECRET\n")
        (project / "scripts/native/link.gd").symlink_to(outside)
        self.source_markers.add(b"OUTSIDE_STOCK_SECRET")
        self.stock_validate(editor, project, "stock_symlink_refusal_before_launch",
                            'extends "./native/link.gd"\n', "unavailable", spawn=False)
        observation.require(outside.read_text() == "extends RefCounted\n# OUTSIDE_STOCK_SECRET\n",
                            "stock_outside_source_untouched")
        large = "extends RefCounted\n# " + "é" * 262133 + "x"
        observation.require(len(large.encode("utf-8")) == 524288, "stock_exact_unicode_root_size")
        self.stock_validate(editor, project, "stock_exact_unicode_limit", large, "valid")
        self.stock_validate(editor, project, "stock_root_over_limit", large + "x", "unavailable",
                            spawn=False)
        bounded = project / "scripts/native/cold/bounds"
        bounded.mkdir()
        for index in range(33):
            (bounded / f"source_{index:02d}.gd").write_text(
                f"extends RefCounted\nconst FIXTURE_INDEX := {index}\n")
        def dependency_source(count):
            return "extends RefCounted\n" + "".join(
                f'const SOURCE_{index:02d} = preload("./native/cold/bounds/source_{index:02d}.gd")\n'
                for index in range(count))
        at_count = self.stock_validate(editor, project, "stock_exact_dependency_count",
                                       dependency_source(32), "valid")
        observation.require(len(at_count["sources"]) == 33,
                            "all_thirty_two_distinct_dependency_uri_fences")
        self.stock_validate(editor, project, "stock_over_dependency_count",
                            dependency_source(33), "unavailable", spawn=False)
        prefix = "extends RefCounted\n# "
        one = bounded / "large.gd"
        one.write_text(prefix + "x" * (524288 - len(prefix.encode())))
        large_dep = 'extends RefCounted\nconst LARGE = preload("./native/cold/bounds/large.gd")\n'
        self.stock_validate(editor, project, "stock_exact_dependency_source_limit",
                            large_dep, "valid",
                            dependency="res://scripts/native/cold/bounds/large.gd")
        one.write_text(prefix + "x" * (524289 - len(prefix.encode())))
        self.stock_validate(editor, project, "stock_over_dependency_source_limit",
                            large_dep, "unavailable", spawn=False)
        for index in range(8):
            (bounded / f"byte_{index:02d}.gd").write_text(
                prefix + "x" * (524288 - len(prefix.encode())))
        (bounded / "byte_08.gd").write_text("x")
        def byte_source(count):
            return "extends RefCounted\n" + "".join(
                f'const BYTE_{index:02d} = preload("./native/cold/bounds/byte_{index:02d}.gd")\n'
                for index in range(count))
        at_bytes = self.stock_validate(editor, project, "stock_exact_total_dependency_bytes",
                                       byte_source(8), "valid")
        observation.require(sum(item["utf8_bytes"] for item in at_bytes["sources"]
                                if item["path"] != ROOT) == 4 * 1024 * 1024,
                            "real_four_megabyte_dependency_closure")
        self.stock_validate(editor, project, "stock_over_total_dependency_bytes",
                            byte_source(9), "unavailable", spawn=False)
        def analyzer_errors(count):
            return "extends RefCounted\n" + "".join(
                f'func check_{index:02d}():\n\tmissing_{index:02d}()\n'
                for index in range(count))
        at_diagnostics = self.stock_validate(
            editor, project, "stock_exact_diagnostic_record_limit",
            analyzer_errors(64), "invalid")
        observation.require(len(at_diagnostics["diagnostics"]) == 64,
                            "real_sixty_four_source_attributed_errors")
        self.stock_validate(editor, project, "stock_over_diagnostic_record_limit",
                            analyzer_errors(65), "unavailable")
        identifier = "missing_length_probe"
        def analyzer_message(name):
            return f"extends RefCounted\nfunc check():\n\t{name}()\n"
        sample = self.stock_validate(editor, project, "stock_message_length_profile",
                                     analyzer_message(identifier), "invalid")
        message = sample["diagnostics"][0]["message"]
        observation.require(message.count(identifier) == 1,
                            "actual_stock_error_message_contains_identifier_once")
        overhead = len(message.encode("utf-8")) - len(identifier)
        exact = self.stock_validate(
            editor, project, "stock_exact_message_byte_limit",
            analyzer_message("m" * (2048 - overhead)), "invalid")
        observation.require(len(exact["diagnostics"][0]["message"].encode("utf-8")) == 2048,
                            "real_stock_diagnostic_exact_byte_limit")
        self.stock_validate(editor, project, "stock_over_message_byte_limit",
                            analyzer_message("m" * (2049 - overhead)), "unavailable")
        after_capture = self.screenshot(editor, "stock-helper-after.png")
        replay = self.action(editor, "replay_sequence_history")
        observation.require(replay["after"]["has_redo"],
                            "stock_helper_preserved_actual_prior_history")
        self.case("stock_helper_real_history_and_window", before_capture=before_capture,
                  after_capture=after_capture, action_count=len(replay["steps"]),
                  source_surfaces="actual_native_undo_redo")
        error_context = self.stock_warning_case("as_error", enable=True, level=2,
                                                status="invalid")
        disabled_context = self.stock_warning_case("disabled", enable=False, level=2,
                                                   status="valid")
        excluded_context = self.stock_warning_case(
            "directory_excluded", enable=True, level=2,
            directory_rules={"res://addons": 0, "res://scripts": 0}, status="valid")
        observation.require(len({error_context, disabled_context, excluded_context}) == 3,
                            "changed_selected_effective_context_distinct_witnesses")
        overridden_context = self.stock_warning_case(
            "editor_overrides", enable=True, level=2, base_enable=False,
            base_level=1, status="invalid")
        observation.require(overridden_context not in
                            {error_context, disabled_context, excluded_context},
                            "effective_editor_override_and_base_configuration_witness")
        self.stock_global_context_case()
