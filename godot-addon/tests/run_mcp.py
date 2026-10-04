#!/usr/bin/env python3
"""Focused T001 core/native acceptance; no MCP or real-client claim."""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import stat
import subprocess
import tempfile

import run_observation as observation
from run_script_edit import STOCK_SHA256
from run_script_close import CloseHarness
from closed_script_acceptance import ClosedScriptAcceptanceMixin, FIXTURE

SCENARIOS = ("closed-native", "closed-lifecycle")


class WorkflowHarness(ClosedScriptAcceptanceMixin, CloseHarness):
    def __init__(self, args, work):
        super().__init__(args, work)
        self.summary.update(
            coverage_scope="T001_protocol_independent_" + args.scenario,
            mcp_acceptance=False, real_client_acceptance=False,
            trusted_script_execution_acceptance=True,
            product_close_acceptance=False, public_close_gdscript=False,
            product_opening_caller_acceptance=False, public_open_gdscript=False,
            workflow_sha256=observation.digest(args.workflow),
            driver_sha256=observation.digest(Path(__file__)),
            closed_acceptance_sha256=observation.digest(Path(__file__).with_name("closed_script_acceptance.py")),
            closed_fixture_files={str(path.relative_to(FIXTURE)): observation.digest(path)
                                  for path in sorted(FIXTURE.rglob("*")) if path.is_file()},
            changed_boundary="private_v6_native_revision4_authenticated_closed_capability_shared_owner",
            support_claim=False,
        )
        self.summary["acceptance_coverage"] = {
            "closed-native": ["actual_supervised_Rust_read_edit", "cached_absent_positives",
                              "zero_effect_unchanged", "empty_Unicode_source_bounds",
                              "read_projection_revision_stability_invalidation", "dirty_divergent_equal_dirty_R",
                              "unsupported_sources_context_getters", "same_text_namespace_cache_epoch_ABA",
                              "prepare_apply_verify_recheck_races", "lost_partial_write_mtime_failure",
                              "supplement_D_R_lifecycle_invalidation", "safe_session_selection_core_refusal_details",
                              "authenticated_closed_malformed_duplicate_unowned_out_of_order_tuples",
                              "immediate_project_settings_guard", "prepare_busy_operation_reason"],
            "closed-lifecycle": ["pre_post_effect_cancel_disable_channel_loss", "newer_work_no_rollback",
                                 "unrelated_real_history", "later_real_open_Save_reparse_rescan_fresh_runtime",
                                 "representative_v6_native4_legacy_operations", "privacy_production_exports",
                                 "request_bound_native_callback_effect_prefix_no_late_effects",
                                 "partial_lost_write_mtime_failure_then_chmod_scope_sticky_privacy"],
        }
        self.source_markers.update((b"CLOSED_PRIVATE_TARGET", b"CLOSED_NEWER_WORK",
                                    b"CLOSED_RESOURCE_DIVERGENCE", b"CLOSED_RESOURCE_DIRTY"))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("godot", "observer", "editor", "stock-validator", "native-fault-addon", "artifacts",
                 "opener", "closer", "discoverer", "workflow"):
        parser.add_argument("--" + name, required=True, type=Path)
    parser.add_argument("--scenario", required=True, choices=SCENARIOS)
    args = parser.parse_args()
    os.umask(0o077)
    for name in ("godot", "observer", "editor", "stock_validator", "opener", "closer", "discoverer", "workflow"):
        path = getattr(args, name)
        observation.require(path.is_absolute() and path.is_file() and os.access(path, os.X_OK),
                            "absolute_actual_executable_" + name)
    observation.require(args.artifacts.is_absolute() and args.artifacts.is_dir() and
                        not args.artifacts.is_symlink() and not list(args.artifacts.iterdir()) and
                        args.artifacts.stat().st_uid == os.geteuid() and
                        stat.S_IMODE(args.artifacts.stat().st_mode) == 0o700,
                        "owned_empty_private_artifact_directory")
    observation.require(observation.digest(args.godot) == STOCK_SHA256, "pinned_official_stock_binary")
    args.candidate_version = observation.VERSION
    args.candidate_engine_hash = observation.ENGINE_HASH
    with tempfile.TemporaryDirectory(prefix=".godot-agent-kit-workflow-", dir=Path.home()) as directory:
        harness = WorkflowHarness(args, Path(directory))
        code = 0
        try:
            harness.initialize()
            harness.group(args.scenario, {"closed-native": harness.closed_native,
                                          "closed-lifecycle": harness.closed_lifecycle}[args.scenario])
            harness.summary["status"] = "passed"
        except (observation.Failure, OSError, ValueError, KeyError, TypeError, EOFError,
                subprocess.SubprocessError) as error:
            code = 1
            harness.summary.update(status="failed", stage=str(error) if isinstance(error, observation.Failure)
                                   else type(error).__name__)
        finally:
            try:
                harness.cleanup()
                harness.verify_incidental_redaction()
            except (observation.Failure, OSError) as error:
                code = 1
                harness.summary.update(status="failed", cleanup_stage=str(error))
            harness.summary["passed_case_count"] = len(harness.cases) if not code else 0
            observation.json_file(args.artifacts / "summary.json", harness.summary)
        print(json.dumps(dict(status=harness.summary["status"], stage=harness.summary.get("stage"),
                              passed_cases=harness.summary["passed_case_count"],
                              coverage_scope=harness.summary["coverage_scope"],
                              summary=str(args.artifacts / "summary.json"))))
        return code


if __name__ == "__main__":
    raise SystemExit(main())
