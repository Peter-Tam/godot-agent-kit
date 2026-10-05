#!/usr/bin/env python3
"""Focused core/native and actual stdio MCP acceptance; real-client proof is separate."""
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
from mcp_lifecycle_acceptance import McpLifecycleMixin, PROFILE_GROUPS, serve_prepared
from mcp_failure_acceptance import McpFailureMixin, FAILURE_GROUPS
from mcp_preservation_acceptance import McpPreservationAcceptanceMixin, PRESERVATION_GROUPS
from mcp_interruption_acceptance import McpInterruptionMixin, INTERRUPTION_GROUPS
from mcp_composed_acceptance import McpComposedMixin
from mcp_privacy_acceptance import McpPrivacyAcceptanceMixin, PRIVACY_GROUPS
from mcp_validation_acceptance import McpValidationMixin

CLIENT_GROUPS = {**PROFILE_GROUPS, **FAILURE_GROUPS}

SCENARIOS = ("closed-native", "closed-lifecycle", "closed-positives", "closed-refusals",
             "closed-revisions-and-boundary-races", "closed-effect-faults",
             "closed-acquisition-invalidation-and-selection", "closed-authenticated-wire-boundaries",
             "closed-save-profile-and-shared-slot", "closed-cancel-and-newer-work",
             "closed-later-durability-and-history", "matched-v6-native4-legacy-preservation",
             "closed-privacy-export", "transport", *("transport-" + name for name in CLIENT_GROUPS),
             "preservation", *PRESERVATION_GROUPS, "interruption", *INTERRUPTION_GROUPS,
             "composed", "privacy-export", *PRIVACY_GROUPS, "validation-preparation")


class WorkflowHarness(McpValidationMixin, McpComposedMixin, McpPrivacyAcceptanceMixin, McpFailureMixin,
                      McpPreservationAcceptanceMixin, McpInterruptionMixin,
                      McpLifecycleMixin, ClosedScriptAcceptanceMixin, CloseHarness):
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
        if args.scenario.startswith("transport"):
            self.summary.update(coverage_scope="T002_actual_MCP_transport_lifecycle",
                                mcp_acceptance=True, real_client_acceptance=False)
        if (args.scenario.startswith(("preservation", "interruption")) or
                args.scenario in ("transport-failures", "transport-reconnect") or args.profile in FAILURE_GROUPS):
            self.summary.update(coverage_scope="T003_adversarial_MCP_" + args.scenario,
                                changed_boundary="MCP_adversarial_fixtures_and_protocol_error_coverage",
                                mcp_acceptance=True, real_client_acceptance=False,
                                mcp_server_sha256=observation.digest(args.mcp_server),
                                adversarial_driver_sha256={name: observation.digest(Path(__file__).with_name(name))
                                    for name in ("mcp_peer.py", "mcp_preservation_acceptance.py",
                                                 "mcp_interruption_acceptance.py", "mcp_failure_acceptance.py")})
        if args.scenario in ("composed", "privacy-export", *PRIVACY_GROUPS) or args.profile == "composed":
            self.summary.update(coverage_scope="composed_MCP_acceptance_" + args.scenario,
                                changed_boundary="composed_and_privacy_acceptance_fixtures",
                                mcp_acceptance=True, real_client_acceptance=False,
                                mcp_server_sha256=observation.digest(args.mcp_server),
                                acceptance_driver_sha256={name: observation.digest(Path(__file__).with_name(name))
                                    for name in ("mcp_composed_acceptance.py", "mcp_privacy_acceptance.py",
                                                 "mcp_lifecycle_acceptance.py", "mcp_peer.py")})


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("godot", "observer", "editor", "stock-validator", "native-fault-addon", "artifacts",
                 "opener", "closer", "discoverer", "workflow"):
        parser.add_argument("--" + name, required=True, type=Path)
    parser.add_argument("--scenario", required=True, choices=SCENARIOS)
    parser.add_argument("--mcp-server", type=Path)
    parser.add_argument("--prepared-run", type=Path)
    parser.add_argument("--revision")
    parser.add_argument("--profile", choices=(*CLIENT_GROUPS, "composed"), default="workflow")
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
            if args.prepared_run is not None:
                observation.require(args.mcp_server is not None and args.mcp_server.is_absolute(),
                                    "fixed_actual_MCP_executable")
                serve_prepared(harness, args.prepared_run)
                return 0
            harness.initialize()
            methods = {
                "closed-native": harness.closed_native,
                "closed-lifecycle": harness.closed_lifecycle,
                "closed-positives": harness.closed_positives,
                "closed-refusals": harness.closed_source_refusals,
                "closed-revisions-and-boundary-races": harness.closed_revision_races,
                "closed-effect-faults": harness.closed_effect_faults,
                "closed-acquisition-invalidation-and-selection": harness.closed_acquisition_boundaries,
                "closed-authenticated-wire-boundaries": harness.closed_wire_boundaries,
                "closed-save-profile-and-shared-slot": harness.closed_save_and_slot_guards,
                "closed-cancel-and-newer-work": harness.closed_cancellation,
                "closed-later-durability-and-history": harness.closed_durability,
                "matched-v6-native4-legacy-preservation": harness.closed_legacy_preservation,
                "closed-privacy-export": harness.closed_privacy_export,
                "transport": harness.mcp_transport,
                "preservation": harness.mcp_preservation,
                "interruption": harness.mcp_interruption,
                "composed": harness.mcp_composed,
                "privacy-export": harness.mcp_privacy_export,
                "validation-preparation": harness.mcp_validation_preparation,
            }
            methods.update(("transport-" + name, harness.mcp_transport) for name in CLIENT_GROUPS)
            methods.update((name, getattr(harness, method)) for name, method in
                           (*PRESERVATION_GROUPS.items(), *INTERRUPTION_GROUPS.items(), *PRIVACY_GROUPS.items()))
            if args.scenario not in ("closed-native", "closed-lifecycle"):
                harness.compile_window_probe()
            harness.group(args.scenario, methods[args.scenario])
            harness.summary["status"] = "passed"
        except (observation.Failure, OSError, ValueError, KeyError, TypeError, EOFError,
                subprocess.SubprocessError) as error:
            code = 1
            harness.summary.update(status="failed", stage=str(error) if isinstance(error, observation.Failure)
                                   else type(error).__name__)
        finally:
            try:
                if not getattr(harness, "prepared_cleanup_complete", False):
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
