"""Real public-caller scenarios; fixture control is separate from expected evidence."""
from __future__ import annotations

import errno
import json
import os
from pathlib import Path
import secrets
import shutil
import signal
import stat
import time

import run_observation as observation
from discovery_scope_acceptance import slot_free
from opening_fixture_witness import TARGET, TARGET_SOURCE, CURRENT, BACKGROUND


class DiscoveryLiveCases:
    def inventory(self):
        for visible in (False, True):
            def prepare(project):
                for folder in range(10):
                    for number in range(10):
                        self.add_script(project, f"catalog/folder-{folder:02}/script-{number:02}.gd")
                if visible:
                    shutil.rmtree(project / "addons/godot_agent_kit/native")
                for relative, source in (("special/empty.gd", ""), ("special/invalid.gd", "var =\n"),
                                         ("special/tool.gd", "@tool\nextends RefCounted\n"),
                                         ("special/large.gd", "# " + "x" * 600000 + "\n"),
                                         ("special/UPPER.GD", "extends RefCounted\n"),
                                         ("special/Mixed.gD", "extends RefCounted\n"),
                                         ("legal spaces/café.gd", "extends RefCounted\n"),
                                         ("something.gd/child.gd", "extends RefCounted\n"),
                                         ("addons/ordinary/visible.gd", "extends RefCounted\n"),
                                         ("vcs-only/visible.gd", "extends RefCounted\n"),
                                         ("export-only/visible.gd", "extends RefCounted\n")):
                    self.add_script(project, relative, source)
                readonly = self.add_script(project, "special/readonly.gd")
                readonly.chmod(0o444)
                (project / ".gitignore").write_text("vcs-only/\n")
                # Actual export preset exclusion must not affect membership.
                preset = project / "export_presets.cfg"
                preset.write_text(preset.read_text().replace('exclude_filter="', 'exclude_filter="export-only/*,'))
                self.add_script(project, ".hidden.gd", included=False)
                self.add_script(project, ".hidden/child.gd", included=False)
                for directory, marker in (("ignored", ".gdignore"), ("nested", "project.godot")):
                    self.add_script(project, directory + "/child/deep.gd", included=False)
                    (project / directory / marker).write_text("marker contents do not matter\n")
                for directory, marker, literal in (("literal-ignore", ".GDIGNORE", ".gdignore"),
                                                    ("literal-project", "PROJECT.GODOT", "project.godot")):
                    path = self.add_script(project, directory + "/child.gd", included=False)
                    (project / directory / marker).touch()
                    if not (project / directory / literal).exists():
                        self.expected[project].add("res://" + str(path.relative_to(project)))
                self.add_script(project, "directory-marker/child.gd")
                (project / "directory-marker/.gdignore").mkdir()
                # Root markers do not hide the selected root.
                (project / ".gdignore").touch()
                if not visible:
                    self.add_script(project, "godot/not-effective-data.gd")
                hidden = self.add_script(project, "finder-hidden/child.gd")
                hidden_file = self.add_script(project, "finder-hidden-file.gd")
                result = observation.run(["/usr/bin/chflags", "hidden", hidden.parent, hidden_file])
                observation.require(result.returncode == 0 and hidden_file.stat().st_flags & 32768,
                                    "real_finder_hidden_controls")
                # Distinct hard-link names are paths, not file-identity deduplication.
                os.link(readonly, project / "special/hardlink.gd")
                self.expected[project].add("res://special/hardlink.gd")
                lower = self.add_script(project, "special/case.gd")
                upper = project / "special/CASE.gd"
                try:
                    with upper.open("x") as stream:
                        stream.write("extends RefCounted\n")
                except FileExistsError:
                    self.case("case_distinct_names_inapplicable_" + str(visible),
                              applicability="case_insensitive_fixture_filesystem")
                else:
                    observation.require(lower.stat().st_ino != upper.stat().st_ino, "actual_case_distinct_files")
                    self.expected[project].add("res://special/CASE.gd")
            profile = "visible" if visible else "hidden"
            with self.live("inventory-" + profile, setup=prepare, visible=visible) as (project, editor, descriptor):
                oracle = self.settled(editor)
                if visible:
                    peer, challenge = self.challenge(descriptor)
                    peer.close()
                    observation.require(challenge["native_api_revision"] == 0 and
                                        challenge["capabilities"]["discover_gdscripts"] is True and
                                        challenge["capabilities"]["open_gdscript"] is False and
                                        challenge["capabilities"]["edit_open_gdscript"] is False,
                                        "discovery_available_without_native_mutation_bundle")
                effective_directory = project / ("godot" if visible else ".godot")
                effective_directory.mkdir(exist_ok=True)
                (effective_directory / "must-not-inventory.gd").write_text("# private editor data\n")
                expected = self.expected[project]
                observation.require(len(expected) >= 100 and len({path.rsplit("/", 1)[0] for path in expected}) >= 10,
                                    "independent_large_catalog")
                observation.require(set(oracle["paths"]) == expected, "native_name_path_visibility_oracle_" + profile)
                result = self.discover(project, descriptor, "exact_inventory_" + profile,
                                       outcome="complete_listing", expected=expected)
                effective = "res://godot" if visible else "res://.godot"
                observation.require(result["inventory"]["scope"]["project_data_directory"] == effective,
                                    "effective_data_directory_" + profile)
                action = "scope_unsaved_hidden" if visible else "scope_unsaved_visible"
                changed = self.action(editor, action)
                observation.require(changed["settings_directory"] == effective + "/editor" and
                                    changed["mutable_hidden_setting"] == visible, "live_setting_distinct_from_effective_getter")
                self.discover(project, descriptor, "unsaved_setting_same_effective_" + profile,
                              outcome="complete_listing", expected=expected)
                self.case("native_visibility_parity_" + profile, oracle_count=len(expected),
                          includes_visible_addon_and_fixture_sources=True,
                          screenshot=self.screenshot(editor, "inventory-" + profile + ".png"))
        for hidden_only in (False, True):
            def empty(project):
                # Relocate the root fixture script under the explicitly ignored
                # tooling tree. No visible scripts are silently subtracted.
                root = project / "fixture_driver.gd"
                root.rename(project / "addons/fixture_driver/base_fixture_driver.gd")
                native = project / "addons/fixture_driver/native_fixture_driver.gd"
                native.write_text(native.read_text().replace('extends "res://fixture_driver.gd"',
                                  'extends "res://addons/fixture_driver/base_fixture_driver.gd"'))
                (project / "addons/.gdignore").touch()
                (project / "scripts/.gdignore").touch()
                if hidden_only:
                    self.add_script(project, ".hidden/only.gd", included=False)
                    self.add_script(project, "ignored/only.gd", included=False)
                    (project / "ignored/.gdignore").touch()
                    self.add_script(project, "nested/only.gd", included=False)
                    (project / "nested/project.godot").touch()
                self.expected[project] = set()
            with self.live("hidden-only-empty" if hidden_only else "empty", setup=empty) as (project, editor, descriptor):
                observation.require(self.settled(editor)["paths"] == [], "independent_native_empty_scope")
                self.discover(project, descriptor, "hidden_only_complete_empty" if hidden_only else "complete_empty",
                              outcome="complete_listing", expected=set())

    def routing(self):
        with self.live("routing-selected", setup=lambda project: self.add_script(project, "SELECTED_PATH_SENTINEL.gd")) as (project, editor, descriptor):
            with self.live("routing-other", setup=lambda project: self.add_script(project, "OTHER_PROJECT_PATH_SENTINEL.gd")) as (other, other_editor, other_descriptor):
                exact = self.discover(project, descriptor, "exact_selected_project", outcome="complete_listing", expected=self.expected[project])
                observation.require(exact["resolved_target"]["session_id"] == descriptor["session_id"], "selected_exact_lifetime")
                self.discover(project, None, "unique_project_only", outcome="complete_listing", expected=self.expected[project])
                mismatch = self.discover(project, other_descriptor, "wrong_project_session", outcome="refused", reason="editor_unavailable")
                observation.require(mismatch["inventory"] is None, "no_other_project_candidate_inventory")
                second = self.start_editor(project)
                try:
                    identities = observation.wait_for(lambda: self.descriptors(project) if len(self.descriptors(project)) == 2 else None,
                                                      "two_same_project_editor_sessions")
                    ambiguous = self.discover(project, None, "ambiguous_same_project", outcome="refused", reason="ambiguous_target")
                    selection = ambiguous["selection"]
                    observation.require(selection is not None and
                                        all(identity["session_id"] in json.dumps(selection) for identity in identities) and
                                        "INVENTORY_SENTINEL" not in json.dumps(selection) and
                                        "SELECTED_PATH_SENTINEL" not in json.dumps(selection),
                                        "authenticated_session_only_selection_feedback")
                    self.discover(project, descriptor, "explicit_among_same_project", outcome="complete_listing", expected=self.expected[project])
                finally:
                    self.close_editor(second)
                    self.editors.remove(second)
                original = stat.S_IMODE(project.stat().st_mode)
                try:
                    project.chmod(0o777)
                    self.discover(project, descriptor, "unsafe_root_mode", outcome="refused", reason="denied_access")
                finally:
                    project.chmod(original)
                parent_mode = stat.S_IMODE(self.work.stat().st_mode)
                try:
                    self.work.chmod(0o777)
                    self.discover(project, descriptor, "unsafe_project_ancestor", outcome="refused", reason="denied_access")
                finally:
                    self.work.chmod(parent_mode)
                acl = observation.run(["/bin/chmod", "+a",
                                       "everyone allow read,search,readattr,readextattr,readsecurity", project])
                observation.require(acl.returncode == 0 and stat.S_IMODE(project.stat().st_mode) == original,
                                    "real_granted_acl_without_mode_change")
                try:
                    self.discover(project, descriptor, "unsafe_root_granted_acl", outcome="refused", reason="denied_access")
                finally:
                    observation.require(observation.run(["/bin/chmod", "-a#", "0", project]).returncode == 0,
                                        "owned_granted_acl_cleanup")
                descriptor_file = next(path for path in self.registry.glob("*.json")
                                       if json.loads(path.read_text())["session_id"] == descriptor["session_id"])
                original_descriptor = json.loads(descriptor_file.read_text())
                changed_descriptor = dict(original_descriptor, token=secrets.token_hex(32))
                observation.json_file(descriptor_file, changed_descriptor)
                try:
                    self.discover(project, descriptor, "exact_candidate_authentication_denial",
                                  outcome="refused", reason="authentication_failed")
                finally:
                    observation.json_file(descriptor_file, original_descriptor)
                for label, arguments in (("unknown_flag", ("--query", "PRIVATE_UNVALIDATED_INPUT")),
                                         ("duplicate_project", ("--project", "PRIVATE_UNVALIDATED_INPUT")),
                                         ("positional", ("PRIVATE_UNVALIDATED_INPUT",))):
                    refused = self.finish_discovery(*self.start_discovery(project, descriptor, extra=arguments),
                                                   "invalid_request_" + label, outcome="refused", reason="invalid_request")
                    observation.require(refused["requested_target"] is None and
                                        "PRIVATE_UNVALIDATED_INPUT" not in json.dumps(refused),
                                        "invalid_request_does_not_echo_unvalidated_fields")
                outside = self.work / "outside-owned"
                outside.mkdir()
                (outside / "OUTSIDE_INVENTORY_SENTINEL.gd").write_text("# OUTSIDE_SOURCE_SENTINEL\n")
                link = project / "outside-redirect"
                link.symlink_to(outside, target_is_directory=True)
                limited = self.discover(project, descriptor, "outside_symlink_local_gap", outcome="limited_listing", reason="unsafe_entry")
                observation.require(all("OUTSIDE" not in path for path in limited["inventory"]["entries"]) and
                                    str(outside) not in json.dumps(limited), "no_redirect_target_disclosure")
                link.unlink()
                # Hold the actual scope reply before the worker enumerates.
                self.gate(editor, "begin")
                process, started = self.start_discovery(project, descriptor)
                self.wait_gate(editor, "begin")
                replaced = project.with_name(project.name + "-original")
                project.rename(replaced)
                project.mkdir()
                (project / "project.godot").write_text("[application]\nconfig/name=\"Replacement\"\n")
                (project / "REPLACEMENT_PRIVATE_PATH.gd").write_text("# replacement\n")
                try:
                    self.release(editor)
                    result = self.finish_discovery(process, started, "root_replaced_before_enumeration", outcome="refused", reason="project_identity_changed")
                    observation.require(result["inventory"] is None, "root_replacement_global_suppression")
                finally:
                    shutil.rmtree(project)
                    replaced.rename(project)
            ended = dict(descriptor)
            self.close_editor(editor)
            self.editors.remove(editor)
            replacement = self.start_editor(project)
            try:
                new = observation.wait_for(lambda: next((item for item in self.descriptors(project) if item["session_id"] != ended["session_id"]), None), "replacement_editor_lifetime")
                self.discover(project, ended, "ended_session_no_substitution", outcome="refused", reason="editor_unavailable")
                self.discover(project, new, "explicit_new_lifetime", outcome="complete_listing", expected=self.expected[project])
            finally:
                self.close_editor(replacement)
                self.editors.remove(replacement)
            # live() normally owns teardown; put the already closed editor back
            # so that its context can perform idempotent log/handle cleanup.
            self.editors.append(editor)
        invalid = self.discover(self.work / "missing-project", None, "missing_project", outcome="refused", reason="invalid_project")
        observation.require(invalid["inventory"] is None, "no_offline_inventory")

    def coverage(self):
        with self.live("coverage") as (project, editor, descriptor):
            expected = self.expected[project]
            baseline = self.discover(project, descriptor, "fresh_initial", outcome="complete_listing", expected=expected)
            oracle = self.settled(editor)
            fresh = self.add_script(project, "scripts/fresh-cache-miss.gd")
            cached = self.open_action(editor, "discovery_oracle")
            observation.require(not cached["scanning"] and not cached["importing"] and
                                "res://scripts/fresh-cache-miss.gd" not in cached["paths"] and
                                cached["paths"] == oracle["paths"], "idle_editor_cache_misses_new_file_negative_control")
            self.discover(project, descriptor, "fresh_create_without_rescan", outcome="complete_listing", expected=expected)
            renamed = fresh.with_name("renamed-cache-miss.gd")
            fresh.rename(renamed)
            expected.remove("res://scripts/fresh-cache-miss.gd")
            expected.add("res://scripts/renamed-cache-miss.gd")
            self.discover(project, descriptor, "fresh_rename_without_rescan", outcome="complete_listing", expected=expected)
            renamed.unlink()
            expected.remove("res://scripts/renamed-cache-miss.gd")
            self.discover(project, descriptor, "fresh_remove_without_rescan", outcome="complete_listing", expected=expected)
            fifo = project / "scripts/nonregular.gd"
            os.mkfifo(fifo, 0o600)
            try:
                self.discover(project, descriptor, "nonregular_script_fifo_never_opened",
                              outcome="complete_listing", expected=expected)
            finally:
                fifo.unlink()
            self.gate(editor, "begin")
            process, started = self.start_discovery(project, descriptor)
            self.wait_gate(editor, "begin")
            source_path = project / "scripts/subject.gd"
            source_path.write_text(TARGET_SOURCE + "# source-only change is not membership\n")
            self.release(editor)
            self.finish_discovery(process, started, "source_only_change_not_inventory_revision",
                                  outcome="complete_listing", expected=expected)
            for relative, directory, reason in (("unreadable-directory", True, "directory_unreadable"),
                                                ("scripts/unreadable.gd", False, "entry_unreadable")):
                path = project / relative
                if directory:
                    path.mkdir()
                    (path / "hidden-from-access.gd").write_text("extends RefCounted\n")
                else:
                    path.write_text("extends RefCounted\n")
                path.chmod(0)
                try:
                    result = self.discover(project, descriptor, "actual_" + reason, outcome="limited_listing", reason=reason)
                    observation.require(set(result["inventory"]["entries"]) == expected, "local_access_gap_preserves_safe_inventory")
                finally:
                    path.chmod(0o700 if directory else 0o600)
                    shutil.rmtree(path) if directory else path.unlink()
            for name in ("bad?name.gd", "bad#name.gd", "bad:name.gd", "bad\\name.gd", "bad\x01name.gd", "bad\nname.gd"):
                path = project / "scripts" / name
                path.write_text("extends RefCounted\n")
                try:
                    result = self.discover(project, descriptor, "unsupported_name_" + str(len(self.cases)), outcome="limited_listing", reason="unsupported_path")
                    observation.require(set(result["inventory"]["entries"]) == expected and name not in json.dumps(result), "no_lossy_or_raw_unsupported_name")
                finally:
                    path.unlink()
            raw = os.fsencode(project / "scripts") + b"/bad-\xff.gd"
            try:
                fd = os.open(raw, os.O_CREAT | os.O_EXCL | os.O_WRONLY, 0o600)
            except OSError as error:
                if error.errno != errno.EILSEQ:
                    raise
                self.case("non_utf8_name_not_representable_by_filesystem",
                          applicability="unavailable: filesystem rejects ill-formed UTF-8 names")
            else:
                os.close(fd)
                try:
                    self.discover(project, descriptor, "non_utf8_name", outcome="limited_listing", reason="unsupported_path")
                finally:
                    os.unlink(raw)
            marker_dir = project / "unsafe-marker"
            marker_dir.mkdir()
            (marker_dir / "child.gd").write_text("extends RefCounted\n")
            marker = marker_dir / ".gdignore"
            marker.symlink_to(project / "scripts/subject.gd")
            try:
                result = self.discover(project, descriptor, "symlink_marker_is_not_exclusion", outcome="limited_listing", reason="unsupported_marker")
                observation.require("res://unsafe-marker/child.gd" not in result["inventory"]["entries"], "unsafe_marker_subtree_withheld")
            finally:
                marker.unlink()
                shutil.rmtree(marker_dir)
            for mode, outcome, reason in (("scanning", "limited_listing", "editor_inventory_busy"),
                                           ("importing", "limited_listing", "editor_inventory_busy"),
                                           ("unavailable", "refused", "unsupported_visibility_policy"),
                                           ("unsupported", "refused", "unsupported_visibility_policy"),
                                           ("disabled_capability", "refused", "unsupported_capability")):
                self.action(editor, "scope_" + mode)
                try:
                    self.discover(project, descriptor, "getter_fault_" + mode, outcome=outcome, reason=reason)
                finally:
                    self.action(editor, "scope_normal")
            self.gate(editor, "recheck")
            process, started = self.start_discovery(project, descriptor)
            self.wait_gate(editor, "recheck")
            # A queued positive recheck may be delayed without changing facts.
            self.release(editor)
            self.finish_discovery(process, started, "stable_recheck_delivery", outcome="complete_listing", expected=expected)
            for change, reason in (("epoch", "editor_inventory_changed"),
                                   ("scope", "scope_changed"), ("scan", "editor_inventory_busy"),
                                   ("unavailable", "recheck_unavailable")):
                self.gate(editor, "begin")
                process, started = self.start_discovery(project, descriptor)
                self.wait_gate(editor, "begin")
                if change == "epoch":
                    self.action(editor, "scope_epoch_change")
                elif change == "scope":
                    self.open_action(editor, "discovery_scope_change")
                elif change == "unavailable":
                    self.action(editor, "scope_unavailable")
                else:
                    self.action(editor, "scope_scanning")
                try:
                    self.release(editor)
                    result = self.finish_discovery(process, started, "detected_editor_" + change + "_change",
                                                   outcome="limited_listing", reason=reason)
                    if change == "scope":
                        observation.require(result["inventory"] is None, "lost_admitted_scope_clears_inventory")
                    else:
                        observation.require(result["inventory"] is not None and
                                            set(result["inventory"]["entries"]) == expected,
                                            "scope_unchanged_retains_independently_rechecked_entries")
                finally:
                    self.action(editor, "scope_normal")
            image = project / "scope_import.svg"
            image.write_text('<svg xmlns="http://www.w3.org/2000/svg" width="2" height="2"><rect width="2" height="2" fill="red"/></svg>')
            # Deliberate fixture import preparation; production discovery itself
            # must never scan/reimport to manufacture a complete list.
            self.action(editor, "scope_live_scan")
            self.discover(project, descriptor, "actual_public_scanning_fact", outcome="limited_listing", reason="editor_inventory_busy")
            self.action(editor, "scope_normal")
            self.settled(editor)
            self.action(editor, "scope_live_import")
            self.discover(project, descriptor, "actual_public_importing_fact", outcome="limited_listing", reason="editor_inventory_busy")
            self.action(editor, "scope_normal")
            self.settled(editor)
        self.boundaries()
        self.zero_and_overflow()

    def fixture_metadata_counts(self, project):
        # Independent fixture witness, not product traversal: these are the two
        # explicitly installed ignored subtrees in this baseline fixture. Count
        # every name (including hidden/non-script names) in admitted directories.
        ignored = {"addons/godot_agent_kit/native", "scripts/native/cold"}
        entries = directories = 0
        for root, child_directories, files in os.walk(project, followlinks=False):
            relative = Path(root).relative_to(project)
            directories += 1
            if str(relative) in ignored:
                child_directories.clear()
                continue
            entries += len(child_directories) + len(files)
            child_directories[:] = [name for name in child_directories if not name.startswith(".")]
        return entries, directories

    def boundaries(self):
        for kind, limit in (("entry", 1024), ("directory", 1024), ("work", 16384)):
            with self.live("bound-" + kind) as (project, editor, descriptor):
                baseline = self.discover(project, descriptor, "bound_baseline_" + kind, outcome="complete_listing", expected=self.expected[project])
                independent_entries, independent_directories = self.fixture_metadata_counts(project)
                inventory = baseline["inventory"]
                observation.require(inventory["visited_entries"] == independent_entries and
                                    inventory["visited_directories"] == independent_directories,
                                    "independent_acquisition_counters_" + kind)
                count = len(self.expected[project]) if kind == "entry" else independent_directories if kind == "directory" else independent_entries
                for number in range(limit - count):
                    if kind == "entry":
                        self.add_script(project, f"scripts/bound-{number:05}.gd")
                    elif kind == "directory":
                        path = project / "scripts" / f"bound-{number:05}"
                        path.mkdir()
                    else:
                        path = project / "scripts" / f"bound-{number:05}.txt"
                        path.touch()
                exact = self.discover(project, descriptor, "exact_" + kind + "_limit", outcome="complete_listing", expected=self.expected[project])
                actual = len(exact["inventory"]["entries"]) if kind == "entry" else exact["inventory"]["visited_directories"] if kind == "directory" else exact["inventory"]["visited_entries"]
                observation.require(actual == limit, "independent_exact_fixed_bound_" + kind)
                if kind == "entry":
                    self.add_script(project, "scripts/one-over.gd")
                elif kind == "directory":
                    (project / "scripts/one-over").mkdir()
                else:
                    (project / "scripts/one-over.txt").touch()
                over = self.discover(project, descriptor, "one_over_" + kind + "_limit", outcome="limited_listing", reason=kind + "_limit")
                observation.require(over["inventory"]["coverage"] == "partial", "overflow_not_exhaustive_" + kind)
        with self.live("depth-bound") as (project, editor, descriptor):
            path = project
            for _ in range(64):
                path = path / "d"
                path.mkdir()
            self.discover(project, descriptor, "exact_depth_64", outcome="complete_listing", expected=self.expected[project])
            (path / "d").mkdir()
            self.discover(project, descriptor, "one_over_depth_64", outcome="limited_listing", reason="depth_limit")
        with self.live("locator-bound") as (project, editor, descriptor):
            # Construct with dir_fd so the macOS ambient PATH_MAX is not the
            # limiting factor for a capability-relative locator.
            fd = os.open(project, os.O_RDONLY | os.O_DIRECTORY)
            components = []
            try:
                for _ in range(9):
                    component = "p" * 200
                    os.mkdir(component, dir_fd=fd)
                    child = os.open(component, os.O_RDONLY | os.O_DIRECTORY, dir_fd=fd)
                    os.close(fd)
                    fd = child
                    components.append(component)
                prefix = "res://" + "/".join(components) + "/"
                final = "s" * (2048 - len(prefix.encode()) - 3) + ".gd"
                file = os.open(final, os.O_CREAT | os.O_EXCL | os.O_WRONLY, 0o600, dir_fd=fd)
                os.close(file)
                expected = self.expected[project] | {prefix + final}
                self.discover(project, descriptor, "exact_locator_2048_bytes", outcome="complete_listing", expected=expected)
                over = "x" + final
                file = os.open(over, os.O_CREAT | os.O_EXCL | os.O_WRONLY, 0o600, dir_fd=fd)
                os.close(file)
                result = self.discover(project, descriptor, "one_over_locator_2048_bytes", outcome="limited_listing", reason="unsupported_path")
                observation.require(prefix + over not in result["inventory"]["entries"], "overlength_locator_not_truncated")
            finally:
                os.close(fd)

    def zero_and_overflow(self):
        with self.live("zero-limited") as (project, editor, descriptor):
            # Hide infrastructure using actual child markers, not result filters.
            (project / "addons/.gdignore").touch()
            (project / "scripts/.gdignore").touch()
            root = project / "fixture_driver.gd"
            root.rename(project / ".fixture_driver.gd")
            gap = project / "only-visible"
            gap.mkdir()
            (gap / "exists.gd").touch()
            gap.chmod(0)
            try:
                result = self.discover(project, descriptor, "zero_entries_not_empty_project", outcome="limited_listing", reason="directory_unreadable")
                observation.require(result["inventory"]["entries"] == [], "explicit_zero_entry_partial")
            finally:
                gap.chmod(0o700)
                (project / ".fixture_driver.gd").rename(root)
        with self.live("diagnostic-overflow") as (project, editor, descriptor):
            for number in range(63):
                (project / "scripts" / f"unsupported-{number:02}?.gd").touch()
            exact = self.discover(project, descriptor, "exact_63_detailed_gaps", outcome="limited_listing", reason="unsupported_path")
            observation.require(len(exact["diagnostics"]) == 63 and
                                all(item["code"] == "unsupported_path" and item["omitted_count"] is None
                                    for item in exact["diagnostics"]), "exact_diagnostic_detail_bound")
            (project / "scripts/unsupported-63?.gd").touch()
            over = self.discover(project, descriptor, "one_over_63_detailed_gaps", outcome="limited_listing", reason="additional_gaps")
            aggregate = [item for item in over["diagnostics"] if item["code"] == "additional_gaps"]
            observation.require(len(over["diagnostics"]) == 64 and len(aggregate) == 1 and
                                aggregate[0]["omitted_count"] == 1, "first_omitted_diagnostic_count")
            for number in range(64, 70):
                (project / "scripts" / f"unsupported-{number:02}?.gd").touch()
            result = self.discover(project, descriptor, "diagnostic_overflow_aggregate", outcome="limited_listing", reason="additional_gaps")
            diagnostics = result["diagnostics"]
            aggregate = [item for item in diagnostics if item["code"] == "additional_gaps"]
            observation.require(len(diagnostics) == 64 and len(aggregate) == 1 and aggregate[0]["omitted_count"] == 7,
                                "63_details_plus_actual_omitted_count")

    def interruption(self):
        for stage in ("begin", "recheck"):
            for operation in ("timeout", "sigint", "sigterm", "disable", "disconnect", "expiry", "worker_stop", "malformed", "oversized", "identity"):
                with self.live("interrupt-" + stage + "-" + operation) as (project, editor, descriptor):
                    self.gate(editor, stage, fault=operation if operation in ("malformed", "oversized", "identity") else "")
                    before = self.action(editor, "witness")
                    disks = {path: observation.disk_witness(project / "scripts" / (path + ".gd")) for path in ("subject", "other")}
                    process, started = self.start_discovery(project, descriptor)
                    self.wait_gate(editor, stage)
                    stopped = None
                    if operation in ("sigint", "sigterm"):
                        process.send_signal(signal.SIGINT if operation == "sigint" else signal.SIGTERM)
                    elif operation == "disable":
                        self.action(editor, "disable")
                    elif operation == "disconnect":
                        # Kill only the owned editor, producing real channel EOF.
                        editor["process"].terminate()
                    elif operation == "expiry":
                        expired = self.open_action(editor, "discovery_expire")
                        observation.require(int(expired["expiry_tick_us"]) > 0, "private_control_forces_actual_owner_expiry_path")
                    elif operation == "worker_stop":
                        children = observation.run(["/usr/bin/pgrep", "-P", str(process.pid)])
                        candidates = [int(value) for value in children.stdout.split()]
                        observation.require(len(candidates) == 1, "one_owned_same_executable_worker")
                        stopped = candidates[0]
                        command = observation.run(["/bin/ps", "-ww", "-p", str(stopped), "-o", "command="])
                        observation.require(str(self.args.discoverer).encode() in command.stdout and b"internal-discovery-worker" in command.stdout,
                                            "owned_worker_identity_before_sigstop")
                        os.kill(stopped, signal.SIGSTOP)
                    elif operation in ("malformed", "oversized", "identity"):
                        self.release(editor)
                    reason = "cancelled" if operation in ("sigint", "sigterm") else "protocol_error" if operation in ("malformed", "oversized", "identity") else "disconnected_editor" if operation in ("disable", "disconnect", "expiry") else "timeout"
                    result = self.finish_discovery(process, started, stage + "_" + operation, outcome="interrupted", reason=reason)
                    if stage == "recheck" and operation not in ("identity",):
                        observation.require(result["inventory"] is not None and result["inventory"]["entries"] and
                                            result["inventory"]["validity"] == "earlier_observation", "checked_batches_retained_only_as_earlier")
                    if stopped is not None:
                        # Parent must have killed/reaped its actual held child.
                        observation.require(observation.run(["/bin/ps", "-p", str(stopped), "-o", "pid="]).returncode != 0,
                                            "stalled_worker_owned_cleanup")
                    if operation != "disconnect":
                        if operation == "disable":
                            self.action(editor, "enable")
                        self.release(editor)
                        slot_free(self, editor)
                        time.sleep(0.2)
                        after = self.action(editor, "witness")
                        observation.require(after == before and all(disks[path] == observation.disk_witness(project / "scripts" / (path + ".gd")) for path in disks),
                                            "no_late_replay_or_editor_effect_" + stage + "_" + operation)
                        self.case("late_release_no_upgrade_" + stage + "_" + operation,
                                  terminal_request=result["request_id"], no_automatic_retry=True)
        self.overlap()
        with self.live("unresponsive-editor") as (project, editor, descriptor):
            editor["process"].send_signal(signal.SIGSTOP)
            editor["suspended"] = True
            try:
                self.discover(project, descriptor, "real_unresponsive_editor", outcome="interrupted", reason="timeout")
            finally:
                editor["process"].send_signal(signal.SIGCONT)
                editor["suspended"] = False

    def overlap(self):
        with self.live("shared-slot") as (project, editor, descriptor):
            self.action(editor, "prepare_other")
            self.action(editor, "prepare_subject")
            peer, request = self.authenticated_peer(descriptor)
            with peer:
                self.peer_operation(peer, descriptor, request, "observe", project / "scripts/subject.gd")
                self.discover(project, descriptor, "real_observation_owner_busy", outcome="refused", reason="busy")
                self.peer_operation(peer, descriptor, request, "recheck", project / "scripts/subject.gd")
            slot_free(self, editor)
            idle_basis = self.edit_basis(project, descriptor, "discovery_reverse_overlap_fresh_basis")
            self.gate(editor, "begin")
            process, started = self.start_discovery(project, descriptor)
            self.wait_gate(editor, "begin")
            self.discover(project, descriptor, "discovery_owner_blocks_discovery", outcome="refused", reason="busy")
            self.observe(project, "unsupported_observation", 2, session=descriptor["session_id"], name="discovery_owner_blocks_observation")
            self.public_open(project, descriptor, "discovery_owner_blocks_open", expected="refused", reason="busy")
            self.edit(project, idle_basis, self.revision(83), "discovery_owner_blocks_edit",
                      "refused", 3, session=descriptor["session_id"], reason="busy", application="not_applied")
            process.send_signal(signal.SIGINT)
            self.finish_discovery(process, started, "release_shared_slot_cancel", outcome="interrupted", reason="cancelled")
            self.release(editor)
            slot_free(self, editor)
            self.discover(project, descriptor, "slot_reusable_after_cancel", outcome="complete_listing", expected=self.expected[project])
            # Hold genuine product opening at its existing fixture begin seam.
            opened, open_started, _ = self.held_open(project, editor, descriptor, "begin", "discovery_busy_open_owner")
            try:
                observation.require(self.action(editor, "scope_slot")["busy"], "actual_open_owner_retains_shared_slot")
                self.discover(project, descriptor, "real_open_owner_busy", outcome="refused", reason="busy")
            finally:
                opened.send_signal(signal.SIGINT)
                self.complete_open(opened, open_started, "open_owner_cancel_after_discovery_busy", "refused", reason="cancelled")
                self.open_action(editor, "open_release")
            slot_free(self, editor)
            basis = self.edit_basis(project, descriptor, "discovery_overlap_fresh_edit_basis")
            edited, payload, edit_started = self.held_edit(
                project, editor, basis, self.revision(89), "prepare", "discovery_busy_edit_owner",
                session=descriptor["session_id"])
            try:
                observation.require(self.action(editor, "scope_slot")["busy"], "actual_edit_owner_retains_shared_slot")
                self.discover(project, descriptor, "real_edit_owner_busy", outcome="refused", reason="busy")
            finally:
                edited.send_signal(signal.SIGTERM)
                self.complete_edit(edited, payload, "edit_owner_cancel_after_discovery_busy", "refused", 4,
                                   started=edit_started, reason="cancellation", application="not_applied")
                self.native_action(editor, "native_edit_release")
            slot_free(self, editor)
            latest = self.edit_basis(project, descriptor, "discovery_entered_native_fresh_basis")
            prior, disks = self.state(editor, project)
            desired = self.revision(91)
            self.native_action(editor, "native_edit_probe_entered")
            edited, payload, edit_started = self.held_edit(
                project, editor, latest, desired, "buffer_applied", "discovery_entered_native_owner",
                session=descriptor["session_id"])
            entered = self.native_action(editor, "native_edit_probe_result")["callback"]
            observation.require(entered.get("called") is True and entered.get("slot_before") is True and
                                entered.get("slot_after") is True, "real_entered_native_owner_retention")
            middle, middle_disk = self.state(editor, project)
            observation.require(middle["subject"]["B"] == desired and
                                middle_disk["subject"]["text"] == disks["subject"]["text"],
                                "actual_native_buffer_entered_before_persistence")
            self.discover(project, descriptor, "public_discovery_cannot_disturb_entered_native_owner",
                          outcome="refused", reason="busy")
            observation.require(self.state(editor, project) == (middle, middle_disk),
                                "discovery_busy_has_no_native_owner_or_source_effect")
            self.release_edit(editor, edited, payload, edit_started,
                              "original_entered_native_owner_completes", "verified_changed", 0)
            self.changed_state(editor, project, prior, disks, desired, "entered_native_survives_discovery")

    def readonly(self):
        for mode in ("clean", "dirty", "equal_dirty", "divergent", "nonselected"):
            with self.live("readonly-" + mode) as (project, editor, descriptor):
                self.action(editor, "prepare_subject")
                if mode in ("dirty", "divergent", "nonselected"):
                    self.action(editor, "dirty_subject")
                # History preparation is explicit fixture activity.
                if mode == "equal_dirty":
                    self.open_action(editor, "open_mutate", mutation="dirty_disk_equal", path=TARGET)
                    history = None
                else:
                    history = self.action(editor, "seed_sequence_history")["history"]
                self.action(editor, "scope_selection")
                self.action(editor, "prepare_other")
                self.action(editor, "dirty_other")
                if mode != "nonselected":
                    self.action(editor, "prepare_subject")
                if mode in ("clean", "equal_dirty"):
                    self.native_action(editor, "native_idle")
                if mode == "clean":
                    self.open_action(editor, "open_mutate", mutation="undo", path=TARGET)
                    self.native_action(editor, "native_idle")
                elif mode == "divergent":
                    self.action(editor, "resource_subject")
                capture = self.screenshot(editor, "readonly-" + mode + "-before.png")
                before = self.action(editor, "witness")
                state, disk = self.open_state(editor, project)
                document = before["subject"]
                if mode == "clean":
                    observation.require(not document["dirty"] and document["R"] == document["B"] ==
                                        disk["target"]["text"] and document["has_redo"], "independently_clean_with_prior_redo")
                elif mode == "equal_dirty":
                    observation.require(document["dirty"] and document["R"] == document["B"] ==
                                        disk["target"]["text"], "independently_equal_text_but_dirty")
                elif mode == "divergent":
                    observation.require(document["dirty"] and len({document["R"], document["B"],
                                        disk["target"]["text"]}) == 3, "independently_divergent_three_authorities")
                else:
                    observation.require(document["dirty"] and document["B"] != disk["target"]["text"],
                                        "independently_dirty_human_buffer")
                observation.require((before["current_script"] == TARGET) == (mode != "nonselected") and
                                    before["other"]["dirty"], "independent_nonselected_dirty_work")
                # Install after the editor's initial scan, so the independent
                # cache witness establishes a genuinely never-loaded script.
                self.add_script(project, "scripts/never-loaded.gd")
                unloaded = self.open_action(editor, "discovery_unloaded", path="res://scripts/never-loaded.gd")
                observation.require(not unloaded["cached"] and "res://scripts/never-loaded.gd" not in unloaded["open_paths"], "actual_unloaded_control")
                self.discover(project, descriptor, "readonly_" + mode, outcome="complete_listing", expected=self.expected[project])
                slot_free(self, editor)
                after = self.action(editor, "witness")
                now_state, now_disk = self.open_state(editor, project)
                now_unloaded = self.open_action(editor, "discovery_unloaded", path="res://scripts/never-loaded.gd")
                observation.require(before == after and state == now_state and disk == now_disk and unloaded == now_unloaded,
                                    "independent_d_r_b_identity_dirty_selection_history_unloaded_" + mode)
                time.sleep(0.25)
                observation.require(self.action(editor, "witness") == after and self.open_state(editor, project) == (now_state, now_disk),
                                    "no_delayed_readonly_effect_" + mode)
                if mode == "equal_dirty":
                    steps = []
                    for operation in ("undo", "undo", "redo", "redo"):
                        self.open_action(editor, "open_mutate", mutation=operation, path=TARGET)
                        steps.append(self.action(editor, "witness")["subject"]["B"])
                    observation.require(steps[0] != before["subject"]["B"] and
                                        steps[0] == steps[2] and
                                        steps[1] == steps[3] == before["subject"]["B"],
                                        "equal_dirty_actual_prior_native_undo_redo")
                else:
                    if mode == "clean":
                        self.open_action(editor, "open_mutate", mutation="redo", path=TARGET)
                    replay = self.action(editor, "replay_sequence_history")
                    observation.require([step["text"] for step in replay["steps"]] ==
                                        [history["second"], history["first"], history["initial"], history["first"]],
                                        "actual_prior_native_undo_redo_reachability_" + mode)
                self.case("readonly_independent_witness_and_history_" + mode,
                          source_surfaces="independent_D_R_B_dirty_identity_selection_versions_native_history",
                          screenshot=capture, unloaded_script_retained=True, actual_undo_redo_transitions=4)
        self.first_use()
        self.known_document_controls()

    def known_document_controls(self):
        with self.live("known-ignored-and-deleted") as (project, editor, descriptor):
            self.action(editor, "prepare_subject")
            self.action(editor, "dirty_subject")
            history = self.action(editor, "seed_sequence_history")["history"]
            prior = self.action(editor, "witness")
            (project / "scripts/.gdignore").touch()
            ignored = {path for path in self.expected[project] if not path.startswith("res://scripts/")}
            self.discover(project, descriptor, "ignored_known_document_not_inventory", outcome="complete_listing", expected=ignored)
            observed = self.observe(project, "complete_observation", 0, session=descriptor["session_id"],
                                    name="ignored_known_document_still_observable")
            observation.require(observed["snapshot"]["sources"]["B"]["text"] == prior["subject"]["B"] and
                                observed["snapshot"]["dirty"]["state"] == "dirty",
                                "discovery_visibility_does_not_change_known_document_semantics")
            (project / "scripts/.gdignore").unlink()
            (project / "scripts/subject.gd").unlink()
            expected = self.expected[project] - {TARGET}
            self.discover(project, descriptor, "deleted_live_buffer_not_inventory", outcome="complete_listing", expected=expected)
            observed = self.observe(project, "limited_observation", 2, session=descriptor["session_id"],
                                    name="deleted_live_buffer_keeps_r_b")
            sources = observed["snapshot"]["sources"]
            observation.require(sources["D"]["availability"] == "unavailable" and
                                sources["D"]["reason"]["code"] == "disk_missing" and
                                sources["R"]["text"] == prior["subject"]["R"] and
                                sources["B"]["text"] == prior["subject"]["B"] and
                                self.action(editor, "witness") == prior,
                                "deleted_file_does_not_erase_unsaved_native_work")
            replay = self.action(editor, "replay_sequence_history")
            observation.require([step["text"] for step in replay["steps"]] ==
                                [history["second"], history["first"], history["initial"], history["first"]],
                                "deleted_live_buffer_prior_undo_redo_retained")
        with self.live("later-removed-path") as (project, editor, descriptor):
            listed = self.discover(project, descriptor, "later_use_listing", outcome="complete_listing", expected=self.expected[project])
            chosen = next(path for path in listed["inventory"]["entries"] if path.endswith("/subject.gd"))
            (project / chosen.removeprefix("res://")).unlink()
            self.public_open(project, descriptor, "removed_returned_path_requires_fresh_resolution",
                             expected="refused", reason="missing_script", script=chosen)

    def first_use(self):
        with self.live("first-use") as (project, editor, descriptor):
            self.installed_native(project)
            self.open_action(editor, "open_setup", paths=[BACKGROUND, CURRENT])
            listing = self.discover(project, descriptor, "first_use_without_script_locator", outcome="complete_listing", expected=self.expected[project])
            chosen = next(path for path in listing["inventory"]["entries"] if path.endswith("/subject.gd"))
            observation.require(chosen == TARGET, "returned_locator_selects_owned_eligible_subject")
            before, disk = self.open_state(editor, project)
            observation.require(not before["target"]["associated"], "discovery_did_not_open_chosen_target")
            self.observe(project, "not_open", 0, session=descriptor["session_id"], script=chosen, name="discovered_closed_fresh_observation")
            self.public_open(project, descriptor, "discovered_closed_explicit_open", script=chosen)
            after, now = self.open_state(editor, project)
            observation.require(after["target"]["R"] == after["target"]["B"] == now["target"]["text"] == TARGET_SOURCE and not after["target"]["dirty"],
                                "first_use_separate_open_clean_d_r_b")
            self.assert_preserved(before, after, disk, now, "first_use_clean_open")
            # Reuse existing focused real fresh-basis edit, dirty refusal,
            # Undo/Save/Redo/Save, old history, reopen/reparse/rescan/runtime smoke.
            self.focused_open_edit(editor, project, descriptor, TARGET_SOURCE)
            # The reused runtime smoke installs this visible ordinary consumer.
            self.expected[project].add("res://scripts/native/consumer.gd")
            dirty = self.discover(project, descriptor, "discovered_dirty_not_edit_authority", outcome="complete_listing", expected=self.expected[project])
            observation.require(chosen in dirty["inventory"]["entries"], "dirty_still_discoverable")
            dirty_choice = next(path for path in dirty["inventory"]["entries"] if path.endswith("/other.gd"))
            before, disks = self.open_state(editor, project)
            observation.require(before["current"]["dirty"], "separate_discovered_dirty_human_document")
            basis = self.observe(project, "complete_observation", 0, session=descriptor["session_id"],
                                 script=dirty_choice, name="discovered_dirty_separate_fresh_basis")
            self.edit(project, basis, disks["current"]["text"] + "# explicit attempted edit\n",
                      "discovered_dirty_edit_refuses", "refused", 3, session=descriptor["session_id"],
                      script=dirty_choice, application="not_applied")
            observation.require(self.open_state(editor, project) == (before, disks),
                                "discovered_dirty_refusal_preserves_text_identity_history")

    def privacy_export(self):
        with self.live("privacy-selected", setup=lambda project: self.add_script(project, "SELECTED_INVENTORY_SENTINEL.gd")) as (project, editor, descriptor):
            with self.live("privacy-other", setup=lambda project: self.add_script(project, "UNSELECTED_INVENTORY_SENTINEL.gd")) as (other, other_editor, other_descriptor):
                result = self.discover(project, descriptor, "privacy_authorized_inventory", outcome="complete_listing", expected=self.expected[project])
                observation.require("UNSELECTED_INVENTORY_SENTINEL" not in json.dumps(result), "selected_result_contains_no_other_inventory")
                second = self.start_editor(project)
                try:
                    observation.wait_for(lambda: len(self.descriptors(project)) == 2, "privacy_two_live_candidates")
                    ambiguous = self.discover(project, None, "privacy_ambiguous", outcome="refused", reason="ambiguous_target")
                    observation.require("INVENTORY_SENTINEL" not in json.dumps(ambiguous), "ambiguous_has_no_candidate_paths")
                finally:
                    self.close_editor(second)
                    self.editors.remove(second)
                original = stat.S_IMODE(project.stat().st_mode)
                try:
                    project.chmod(0o777)
                    denied = self.discover(project, descriptor, "privacy_denied", outcome="refused", reason="denied_access")
                    observation.require("INVENTORY_SENTINEL" not in json.dumps(denied), "denied_has_no_inventory_paths")
                finally:
                    project.chmod(original)
                self.gate(editor, "recheck")
                process, started = self.start_discovery(project, descriptor)
                self.wait_gate(editor, "recheck")
                process.send_signal(signal.SIGINT)
                self.finish_discovery(process, started, "privacy_interrupted", outcome="interrupted", reason="cancelled")
                self.release(editor)
                slot_free(self, editor)
                for path in self.registry.glob("*.json"):
                    payload = path.read_bytes()
                    observation.require(b"INVENTORY_SENTINEL" not in payload and b"DISCOVERY_PRIVATE_SOURCE_BODY" not in payload,
                                        "registry_has_no_new_inventory_or_source")
                for path in self.artifacts.glob("*.stderr"):
                    observation.require(b"INVENTORY_SENTINEL" not in path.read_bytes(), "stderr_has_no_inventory")
                self.case("selected_other_denied_ambiguous_interrupted_privacy", result_only_review=True)
        # Existing export harness really inspects and executes enabled, disabled
        # and hook-only packs and verifies there are no listeners at runtime.
        self.export_boundary()
