---
description: "PR-sized implementation tasks for observation-only Feature 001"
---

# Tasks: Observe Live GDScript Editor State Safely

**Input**: Reviewed [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md), [data-model.md](data-model.md), [caller contract](contracts/observation-api.md), [bridge contract](contracts/bridge-protocol.md), and [quickstart.md](quickstart.md).
**Requirements-quality gates**: [Observation checklist](checklists/observation.md) 38/38; [requirements checklist](checklists/requirements.md) 16/16. These are requirements-quality results, not implementation, acceptance, or support claims.
**Granularity**: **8 tasks**, replacing 35. Each checkbox is one coherent implementation/evidence increment and one PR, including its directly necessary tests and documentation. Task IDs below replace the previous decomposition; the migration table identifies the old IDs explicitly.
**Scope**: Planning/task refinement only, including the user-authorized mutual-authentication correction to T002/T006's existing safe-session boundary. All eight implementation tasks remain unchecked and their dependencies are unchanged. No mutation, MCP, runtime tooling, debugger/LSP, distribution, or later roadmap work is authorized.

## Working Agreement and Normative Contracts

- Follow [AGENTS.md](../../AGENTS.md) and the [constitution](../../.specify/memory/constitution.md). Select exactly one dependency-ready ID; fetch updated `main`, preserve unrelated changes, create its `task/<task-id>-<description>` branch, and verify `SPECIFY_FEATURE_DIRECTORY` before invoking installed Spec Kit helpers. Implement only that task and its necessary tests/docs; inspect intended/staged changes, commit explicit paths, push, open its PR, report evidence, then stop for review/merge. No automatic merge or next-task start. After confirmed merge, safely delete remote/local source branches. Stacking requires explicit authorization and still means one task per PR.
- Task markers are implementation-owned; requirements-checklist markers remain reviewer-owned. No checklist changes or implicit reapproval are part of implementation bookkeeping.
- Every task has an objective, explicit dependencies, a capability boundary, evidence, and completion criteria. Paths name the approved future layout, not existing code. Setup, test preparation, contract cases, and routine docs are work **inside** those PRs, not extra task PRs.
- The data model's field/enum/availability/timing/precedence rules, both version-1 contracts, and the quickstart's evidence contract remain normative in full. The task summaries do not replace them with abbreviated schemas. Keep one implementation of the common semantics; do not duplicate the contracts here or create story-specific reducers.
- Safety is required when a boundary first exists. Later story tasks extend live behavior and scenario coverage; they are not permission to defer authentication, dirty-state uncertainty, invalidation, limits, privacy, or truthful outcomes on an already-exposed path. No stub collector, fabricated positive evidence, unconditional unsupported fallback, or knowingly failing prerequisite PR.
- No `[P]` markers: these capability slices share integration/harness files. The dependency graph identifies semantic independence, not authorization for concurrent delivery. Subagent work within a selected task remains under that task's single integration owner.

## Evidence and Completion Rules for Every PR

Write meaningful boundary/transition regressions with the behavior they protect; reproduce discovered bugs before fixing them. Do not create separate test-first PRs against already-shipped behavior or merge failing/ignored mandatory tests. Tests must exercise observable semantics, not copied fields, mock echoes, source-text patterns, or incidental wording.

Each PR must smoke its actual changed surface: a library consumer for pure semantics, actual local connections/processes for routing/supervision, and the real GUI editor for R/B/dirty/session claims. Boundary simulations can prove malformed-input and interruption handling, never visible-buffer observability. Use isolated synthetic fixtures, explicit deadlines and events/conditions, independent authority witnesses, and cleanup of only owned processes/state. Fixture preparation may edit/open/close/save/suspend; none becomes a product capability.

Record stage, target, authority, timing, limitations, and observed before/after state in directly relevant API docs and `specs/001-observe-gdscript-state/quickstart.md`; dependency/support evidence belongs in `specs/001-observe-gdscript-state/research.md`. These directly necessary documentation updates ship with each implementation PR. A future genuine contradiction with the approved artifacts blocks the affected task and requires review, not an undocumented contract change.

Applicable native baseline, from `mcp-server/` after its reviewed lockfile exists:

```sh
cargo +1.98.1 fmt --all -- --check
cargo +1.98.1 clippy --all-targets --locked -- -D warnings
cargo +1.98.1 test --locked
cargo +1.98.1 doc --no-deps --locked
```

Default `cargo test` includes doctests. This is one package, not a workspace; no blanket `--all-features` or MSRV promise. Build the real caller with `cargo +1.98.1 build --locked --bin observe-gdscript` once introduced. Run applicable checks after integration, not unrelated broad suites after every edit.

The exact candidate editor is `4.7.2.stable.official.ed1daf0bf`, full hash `ed1daf0bf001b61586d9930840f2f1394092c079`. Native CI targets macOS 15 arm64; GUI acceptance targets the planned macOS 26.6.2 arm64 trusted GUI environment. Record actual version/hash/OS/architecture and binary/template provenance/checksums. No untested version/platform support; native compilation and planning feasibility do not establish GUI acceptance.

For implemented quickstart groups, run from the repository root with absolute executable paths and a fresh private evidence directory:

```sh
python3 godot-addon/tests/run_observation.py \
  --godot "$GODOT" --observer "$OBSERVER" \
  --scenario "$GROUP" --artifacts "$ARTIFACTS"
```

The driver must honor Quickstart §3's flags, independent oracle, `summary.json`, scoped screenshots, intentional synthetic results, safe logs, and owned cleanup. Until all groups exist, `--scenario all` must fail with explicit missing coverage; never silently pass a subset. Missing GUI/templates/required CI evidence is an unmet gate, not a passing skip.

### Bounds and Invariants Carried into Implementation

These are existing contract limits, not new scope:

| Boundary | Required behavior | Initial owner |
|---|---|---|
| Private metadata | Owner-only 0700 registry; owner-only 0600 regular descriptors; no source; descriptor maximum 4 KiB; at most 32 descriptors, never choose a convenient subset | T002 |
| Session/authentication | Fresh independent 128-bit session ID and 256-bit secret per plugin lifetime; fresh 256-bit client/server nonces and role-separated HMAC-SHA256 proofs with constant-time verification; secret never transmitted; IPv4 `127.0.0.1`, ports 49152–65535, at most 16 bind attempts | T002 |
| Frames/control | Four-byte big-endian length plus UTF-8 JSON; control/handshake maximum 4 KiB, observation response/worker result maximum 12 MiB; exact scalar tuple arity/types; reject oversized announced lengths before allocation | T002 framing; T003 observation/worker validation |
| Fields/decoder | Request ID maximum 64 ASCII characters; project root 1024 UTF-8 bytes; resource locator 2048 UTF-8 bytes; response nesting at most 32; reject duplicate required response fields and wrong versions/identities | T001 semantic constraints; T002/T003 adapters |
| Editor work | At most 32 peers including incomplete/unauthenticated handshakes, one active collection; expire unauthenticated peers within 4.5 seconds; partial network I/O at most 64 KiB or 1 ms per frame, with a separate bounded getter pass | T002 networking; T004 collector |
| Each source | D, R, B independently capped at 512 KiB UTF-8; only that source becomes unavailable/`too_large`, still applicable; retain unrelated and earlier valid evidence, never truncate or substitute empty | T001 semantics; T003 D; T004 R/B; T007 full matrix |
| Attempt deadline | Start monotonic deadline before filesystem/selection work; stop collection at 4.5 seconds; controlled consuming caller receives terminal result within five seconds; never wait on blocked worker I/O | T003; every live scenario thereafter |

D/R/B remain independent, including equal and empty text. Dirty state requires document-specific editor evidence. Denial suppresses source; ambiguity never starts source reads. Known disconnect, protocol failure, cancellation, or timeout prevents complete/not-open success. Invalidated evidence cannot enter current comparisons. A per-source limit alone is never operation-wide refusal. Every exposed path must retain these invariants from its first PR.

## Phase 1: Reviewable Architectural Prerequisites

Three architectural capabilities replace ten setup/foundation fragments. They deliberately stop short of claiming a useful live-script observer; the first complete US1 slice is T004.

- [ ] T001 Deliver the reusable observation evidence and classification engine.
  **Objective:** Give any local adapter one executable, protocol-independent interpretation of identity, D/R/B, dirty state, comparisons, invalidation, and terminal outcomes.
  **Depends on:** None.
  **Implementation boundary:** `mcp-server/Cargo.toml`, `mcp-server/Cargo.lock`, `mcp-server/rust-toolchain.toml`, `mcp-server/src/lib.rs`, `mcp-server/src/observation.rs`, `mcp-server/tests/observation_contract.rs`, and `.github/workflows/observation-native.yml`. Bootstrap the package together with real domain behavior, not an empty library. Use Rust 1.98.1/edition 2021 and the planned `serde =1.0.229` with derive/defaults, `serde_json =1.0.151` with default std, and `cap-std =4.0.3` without default features; add no alternative framework, package topology, or license/MSRV claim. Domain types and reducers contain no Serde DTO, socket, MCP, or Godot-object dependency.
  **Tests/evidence:** Implement Data Model deterministic invariants and Caller Contract §5's semantic vectors in full evidence records. Cover independently observed empty/exact text, every D/R/B pair, partial divergence, dirty-equal text, unknown dirty/open state, closed-cached/unloaded distinctions, invalidated versus current text, source-limit preservation, identity replacement, denial suppression, and terminal precedence. Smoke a public library consumer. Run native baselines and the actual-lockfile license/provenance/advisory review, recording audit tool/database revisions. Native CI verifies OS/architecture, uses locked resolution, full-SHA-pinned actions and least token permissions, and never gives untrusted PR code secrets or privileged GUI execution.
  **Accept:** A consumer can construct validated evidence and obtain the specified comparisons, completeness, diagnostics, and outcome without transport/editor dependencies. IDs/counters preserve precision; collection clocks are not mixed; nullable/required fields and reason constraints follow the data model. Unknown staleness is not invalidation; divergence alone is neither staleness nor failure. CI/native/dependency evidence and API semantics are documented in this PR. No executable stub or live-editor/support claim.

- [ ] T002 Deliver authenticated, source-free local editor session routing with a safe addon lifecycle.
  **Objective:** Resolve exactly one intended local project/editor session, or refuse without acquiring source, through a real enabled Godot addon.
  **Depends on:** T001.
  **Implementation boundary:** `mcp-server/src/target.rs`, the registry portion of `mcp-server/src/project_fs.rs`, framing/mutual-authentication DTOs and proof handling in `mcp-server/src/bridge.rs`, library wiring, and the real `init-registry`/help/version command in `mcp-server/src/bin/observe-gdscript.rs` and `mcp-server/Cargo.toml`; `godot-addon/addons/godot_agent_kit/plugin.cfg`, `plugin.gd`, `bridge.gd`, and `export_guard.gd`. Introduce `ring =0.17.14` with default features disabled and `std` enabled here, with the directly necessary manifest/`Cargo.lock` changes and dependency review; keep cryptography out of the core. Include the directly needed synthetic `godot-addon/tests/fixtures/observation/` project, `main.tscn`, scripts, production `macOS` export preset, and lifecycle/export portions of `godot-addon/tests/run_observation.py` and `fixture_driver.gd`. These are supporting parts of the session boundary, not separate setup PRs. The observation command/worker belongs to T003, actual source handlers to T004; advertise only capabilities actually installed.
  **Tests/evidence:** In `mcp-server/tests/bridge_boundary.rs` and `mcp-server/tests/confinement.rs`, exercise actual framing/selection and unsafe metadata/locator/authentication paths. Verify Rust/Godot agreement on the bridge's three synthetic proof vectors; reject wrong secrets, stale-descriptor/rebound-port impersonation by an owned listener without the secret, replay/reflection, changed transcript fields, malformed proofs, and authentication before the required finish. Establish that wire traffic contains no raw secret and that an unauthenticated peer cannot obtain source. Review ring's actual-lockfile licenses/provenance/advisories and native build. In owned real editors, prove source-free zero/one/multiple candidate handling, exact-session selection, unresolved-candidate timeout, and disable/re-enable/restart identity/cleanup. Smoke real bootstrap and mutual hello, not just DTO construction. Run `export-boundary` now: exact-template addon-enabled and addon-disabled exports, ZIP and actual app-pack inspection including compiled/remapped files, and owned exported launch with registry configuration present as a negative control. No addon/fixture-driver material, advertisement/listener/tooling node, or gameplay dependency may survive. This launch is test evidence, not runtime tooling.
  **Accept:** Explicit canonical project identity and exact project-relative locator are validated; neither a descriptor nor an echoed hello proves listener identity/liveness. Verify fresh, session-bound mutual proofs before authenticating a candidate; failed proofs are source-free `denied_access`/`authentication_failed`, not a candidate to silently discard. No focused/first/newest candidate, unresolved-candidate discard, old-session substitution, source-bearing handshake, or source read before authenticated unique selection. Registry access is restricted to minimal owner-private routing metadata; unsafe state is refused, never repaired or given an insecure fallback. Missing addon configuration opens no listener. Disable/exit removes only owned metadata/references/peers/hooks/signals/nodes. The secret never enters wire frames; private tokens/endpoints and authentication nonces/proofs never enter public results, arguments, or logs. Record source-free lifecycle/security and export evidence with their verified limits; no script-observation claim.

- [ ] T003 Deliver bounded, project-confined observation execution through the local caller.
  **Objective:** Make the real caller enforce scope, deadline, strict evidence validation, and deterministic terminal delivery independently of blocked editor/filesystem work.
  **Depends on:** T002.
  **Implementation boundary:** `mcp-server/src/runner.rs`, `mcp-server/src/bin/observe-gdscript.rs`, confined D acquisition/recheck in `mcp-server/src/project_fs.rs`, observation/worker/result DTOs in `mcp-server/src/bridge.rs`, and necessary library wiring. Implement the planned same-binary read-only worker and complete caller interface, not a general process runner or a second D-only product operation. Consume T002's authenticated target gate and T001's reducer. The addon collector is not part of this PR; an actually absent advertised observer is explicitly unsupported, never a placeholder success or an unconditional unsupported implementation.
  **Tests/evidence:** Ship caller, protocol/worker, and filesystem regressions in `mcp-server/tests/observation_contract.rs`, `bridge_boundary.rs`, and `confinement.rs`. Exercise real CLI bootstrap/refusals against the T002 addon; smoke bounded D/read-recheck through a library consumer on an authenticated, uniquely selected owned project. Cover strict UTF-8/BOM/whitespace/line endings/empty text, missing/unreadable/nonregular source, path/symlink/race confinement, independent size bounds, fragmented/malformed/duplicate-field/wrong-identity responses, EOF versus silence, blocked owned worker, cancellation, and ignored late frames. Controlled boundary peers may establish validation/precedence, not positive R/B evidence. Run source/token sentinel checks across success-capable result serialization and failure diagnostics.
  **Accept:** The monotonic clock starts before resolution; collection stops at 4.5 seconds and the consuming caller gets one UTF-8 JSON result plus newline within five seconds for controlled stalls. Exit codes 0/2/3/4/1, correlation, nullable fields, safe stderr, and flag validation follow the caller contract. Only the owned worker can be terminated/reaped; no blocked join, editor startup/kill, automatic observation retry, or old snapshot/late-result promotion. D is capability-confined to the selected project, independently read/rechecked, and never supplied from a built-in container. Validate evidence before retention, preserve permitted partial facts with target/time, and suppress source on denied scope. Core outcomes already distinguish unavailable, disconnected, timeout, protocol failure, and cancellation. Document the usable executor boundary and the missing live-collector capability honestly; this is not yet US1 completion.

**Why three prerequisites, not one large foundation PR:** T001 owns reusable semantics and its native build gate; T002 owns a live authenticated session/security/export boundary; T003 owns process-isolated execution and disk confinement. Each has working behavioral evidence without a future collector. Merging all three with UI collection would combine independent architectural review concerns; splitting them by file, enum, fixture, or test would recreate the original problem.

## Phase 2: First Useful US1 Vertical Slice

- [ ] T004 [US1] Deliver clean-open D/R/B observation end to end, with independent real-editor acceptance.
  **Objective:** Let the real local caller inspect a safely selected clean open GDScript and report each actual authority and attributable clean state without changing the editor.
  **Depends on:** T003.
  **Implementation boundary:** `godot-addon/addons/godot_agent_kit/observation.gd`, real observe/recheck registration in `bridge.gd`/`plugin.gd`, and end-to-end sample → confined D → recheck → common reducer integration in `mcp-server/src/bridge.rs` and `runner.rs`. Complete the independent GUI witness/`clean-open` driver in `godot-addon/tests/run_observation.py` and `godot-addon/tests/fixtures/observation/fixture_driver.gd`. Add the protected `.github/workflows/observation-live.yml` path alongside the existing native workflow as the GUI harness becomes runnable; it can run implemented named groups on explicitly trusted reviewed revisions, not silently treat partial coverage as `all`.
  **Tests/evidence:** Ship public/bridge regressions for source attribution, fields, intervals, rechecks, and source-free failure with the collector/integration. Run `clean-open` for US1.1, US1.2, US1.3 using independently sampled disk, existing GDScript, actual CodeEdit, and document-attributed unsaved evidence; include actual empty source and fresh repeat requests. Capture native owned-window screenshots and before/after source, document selection/caret/version/dirty/history witnesses without using the product collector as the oracle. Exercise failed/changed recheck and per-source limits on the new collector boundary, plus redaction and export regressions for newly introduced source/driver paths. Validate the protected CI configuration and smoke the same driver entrypoint locally; actual trusted CI is required before a support claim.
  **Accept:** Existing open-script/editor arrays associate R/B only when counts, unique paths, types, order, and before/after identities justify it. R is actual `source_code`, B actual matched CodeEdit text, dirty actual attributable editor evidence; no selection/open/load/save/reload/reparse/rescan/history action manufactures facts. Each request has fresh identity/time/witnesses; no authority substitutes for another. Complete requires actual D/R/B/open/dirty/identity observation and successful relevant rechecks, never acknowledgment. Detected changes already invalidate affected evidence and withhold completeness; per-source limits retain other facts. Unavailable attribution is explicit, not a guessed document or clean state. All three US1 scenarios pass within five seconds with no interference. CI actions are full-SHA pinned, least-privilege, and cannot execute untrusted PR code with secrets or on a persistent privileged GUI runner. Document this exact development slice, not full Feature 001 or version support.

**First useful US1 checkpoint: four PRs — T001 → T002 → T003 → T004.** No US2/US3/US4 scenario group or final release claim is a prerequisite for demonstrating US1. Necessary safe routing, deadlines, uncertainty, invalidation and export isolation are prerequisites, not postponed hardening. The former fifteen-PR path was an artifact of file/layer/test decomposition, not fifteen independently useful capabilities.

## Phase 3: Remaining Independently Testable Stories

- [ ] T005 [US2] Deliver document-attributed unsaved/divergent and changing-document observations.
  **Objective:** Expose real human edits and interval changes without concealing divergence, inventing dirty attribution, or combining evidence from different document identities.
  **Depends on:** T004.
  **Implementation boundary:** Extend live attribution and request-local witnesses in `godot-addon/addons/godot_agent_kit/observation.gd` and propagation through `mcp-server/src/bridge.rs`/`runner.rs`; implement corresponding synthetic preparation, barriers, independent witnesses, and assertions in `godot-addon/tests/fixtures/observation/fixture_driver.gd` and `godot-addon/tests/run_observation.py`. Reuse the common model and the recheck/invalidation safety already required by T004; no new conflict-resolution or mutation mechanism.
  **Tests/evidence:** Include the affected Rust contract/event-sequence regressions and run `dirty-divergent`, `dirty-unavailable`, and `changing-document` in this PR. Prove US2.1, US2.2, US2.3, US2.4, US2.5 with selected/non-selected real buffers, mixed text/script/documentation tabs, nonunique/empty paths or unsupported association, equal-text-but-editor-dirty state, unavailable and unattributable dirty evidence, actual differing R, and controlled source/rename/remove/close/replacement transitions. Use barriers rather than timing races. Test-only restrictions may withhold observability but never inject fake positive D/R/B or a public bypass flag.
  **Accept:** Independently observable dirty/divergent snapshots remain complete; every observed pair is correct, including partial divergence and whitespace/line-ending differences. Global unsaved indications without document attribution yield dirty unknown and limited observation while retaining known target/open/source facts, not ambiguous/disconnected outcomes. Freshly observed stale R remains R; only specific evidence establishes staleness. Same-document changes invalidate affected facts; replacement invalidates every identity-dependent fact, with former text excluded from current comparisons. Detected closure during an open read is invalidation, not a manufactured closed snapshot. No automatic reread hides instability. All five scenarios preserve human text, selection and history; privacy/export/non-interference assertions cover the added paths. Document exactly observed attribution/consistency limits.

- [ ] T006 [US3] Deliver source-attributed multi-session routing and live interruption outcomes.
  **Objective:** Keep every returned fact bound to the intended editor lifetime, including ambiguous selection, absent/replaced sessions, and loss during a source-bearing observation.
  **Depends on:** T004. No dependency on T005.
  **Implementation boundary:** Integrate live source-provenance and lifecycle transitions across `mcp-server/src/target.rs`, `bridge.rs`, `runner.rs`, `bin/observe-gdscript.rs`, `godot-addon/addons/godot_agent_kit/plugin.gd`/`bridge.gd`, and the shared GUI driver/fixtures. T002/T003 already own working source-free selection, authentication, confinement and deadline primitives; this slice owns their source-bearing multi-editor and partial-stage behavior, not a second implementation or a deferred security layer.
  **Tests/evidence:** Ship the affected caller/bridge/confinement transition regressions and complete `routing`, `session-loss`, `deadline`, and `confinement` with the integration. Use distinguishable real projects/editors, identical names/basenames, and same-project sessions where supported. Exercise omitted/exact selectors, absent and ended IDs, disable/re-enable/restart, disconnection after different validated partial stages, an owned suspended connected editor, and an independently stalled owned worker. Check wrong proofs/identities, stale/unsafe descriptors, an owned impostor rebinding an ended session's port without its secret, unresolved candidate liveness, capacity bounds, malformed/oversized frames and traversal/symlink/race denial. Reuse T002's authenticated boundary and replay its no-secret-on-wire and replay/reflection regressions; no forged listener may supply accepted R/B/dirty evidence or trigger D acquisition. Restore/stop only owned processes; no passing skip when a required fixture cannot be established.
  **Accept:** US3.1, US3.2, US3.3, US3.4, US3.5, US3.6, US3.7 pass. Ambiguity returns missing-selector metadata and zero candidate source; exact selection returns only that project's/session's/document's facts. No focus/first/newest fallback, automatic editor startup, issued-observation retry, or replacement-session substitution. Before-authentication absence is unavailable; failed proofs are denied access; known post-authentication loss is disconnected; silence without known loss is timeout; malformed/mismatched frames are protocol errors. Partial facts retain original attribution and explicit invalidation/currency limits; no interruption or failed recheck yields complete/not-open success, and late evidence cannot upgrade a terminal result. Every controlled request meets five seconds. Source/token sentinels prove privacy and no outside-project access beyond minimal private session metadata. Extend `redaction` with these paths and document the exact lifecycle/deadline evidence.

- [ ] T007 [US4] Deliver closed, invalid, and partly observable document inspection without forced observability.
  **Objective:** Explain what exists and what can actually be read when the target is not a fully observable open document, preserving every independent fact.
  **Depends on:** T004. No dependency on T005 or T006.
  **Implementation boundary:** Extend document classification and cached-resource/surface handling in `godot-addon/addons/godot_agent_kit/observation.gd` and `mcp-server/src/project_fs.rs`, `bridge.rs`, and `runner.rs`; add independent fixture preparation and the two story groups to the shared Python/Godot harness. Keep existing source caps and common availability/precedence semantics rather than inventing new outcomes or limits.
  **Tests/evidence:** Ship caller/bridge/filesystem regressions and run `closed-and-invalid` plus `surface-limits` in this PR. Cover confirmed closed with actual cached R versus unloaded R; missing and non-GDScript targets; observable syntax-invalid/empty GDScript; open missing/unreadable D; unavailable R or known-open unreadable B; unknown open state; mixed tabs; already-identifiable built-in and unsupported built-in identity. Independently establish D, R and B limit boundaries, including closed D/R and earlier valid evidence, and a separate stronger denial/disconnection/timeout. No fake positive surface evidence.
  **Accept:** US4.1, US4.2, US4.3, US4.4, US4.5, US4.6 and CHK032's per-source-limit cases pass. Closed B/dirty alone are not applicable; cached R uses only `ResourceLoader.get_cached_ref`/an existing GDScript, unloaded R is unavailable/`resource_not_loaded`. Missing without an open document differs from an open document with missing D; syntax errors do not invalidate GDScript. Unknown open state never implies closed. Built-in D is unavailable/no-standalone-source, never container bytes; unresolved built-in identity is unsupported without loading. Every over-limit source remains applicable but unavailable/`too_large`, with no truncation/empty substitution or loss of other and earlier facts. Open/unknown-open is limited, confirmed closed remains not-open, and separate refusal/interruption precedence survives. No missing authority yields false three-way agreement; no observer opens/selects/loads/repairs anything. Extend privacy/export/non-interference evidence for these paths and document closed/partial limitations.

**Story checkpoints:** US2, US3 and US4 each ship their implementation, contract regressions, fixture preparation, real-editor groups and docs together. Each is runnable using T004's caller/harness without another later story's groups. Default delivery remains US2 → US3 → US4 by priority/ID, with shared-file integration serialized; this is a delivery convention, not a hidden dependency.

## Phase 4: Cumulative Feature Acceptance

- [ ] T008 Establish feature-wide non-interference and the verified compatibility/evidence baseline.
  **Objective:** Demonstrate that all completed capabilities remain read-only and truthful across the complete acceptance matrix, and record only support claims backed by that evidence.
  **Depends on:** T005, T006, T007 (and their prerequisites).
  **Implementation boundary:** Add the cumulative `sequential-readonly` orchestration/assertions to `godot-addon/tests/run_observation.py` and `godot-addon/tests/fixtures/observation/fixture_driver.gd`; switch the existing protected live workflow to the complete `--scenario all` gate and reconcile `.github/workflows/observation-native.yml` / `observation-live.yml` matrix evidence. Consolidate guarantees, limitations and evidence references in `specs/001-observe-gdscript-state/quickstart.md` and `research.md`. This is a whole-feature invariance/support gate, not a place to supply missing routine tests, implement unfinished story behavior, or waive failed prerequisites.
  **Tests/evidence:** Run the twenty-observation sequence: six clean, eight independently prepared human-dirty, six closed, including non-selected targets and known prior undo history. Observe actual D/R/B, disk metadata/content, open list, tab/caret, buffer versions, dirty state, and prior Undo/Redo availability before/after; fixture-only Undo/Redo must still reproduce the known prior sequence. Run all 13 named quickstart groups, all 21 scenarios and seven edge cases, full redaction and enabled/disabled export checks, every controlled deadline, applicable native baselines including doctests/docs, and actual-lockfile dependency review. Obtain actual trusted GUI CI/release-runner results on an explicitly trusted reviewed revision, not merely workflow configuration or a planning/local smoke.
  **Accept:** Every sequential request has fresh evidence or explicit limitations; zero observer-caused source writes, saves, open/close/selection/history changes. Result-only review can identify the selected target/refusal reason, D/R/B/dirty knowledge and every limitation for all scenarios. All required groups execute without hidden skips; every controlled attempt is within five seconds. Version/hash/OS/architecture/binary/template/driver/lockfile identities and redacted evidence are recorded. CI uses protected reviewed revisions, full-SHA pins and least permissions. Support is stated only for exact combinations with applicable native, real-editor and export evidence; macOS 15 compilation alone is not GUI support. If a required runner or other gate is unavailable, the task and feature remain incomplete. No product UndoRedo, applied-edit durability, mutation/MCP/runtime/debugger capability or roadmap Phase 1 exit is claimed. Deliver this task's own evidence/implementation PR and stop.

## Dependencies and PR Count

Each `Depends on` line is authoritative; no blanket phase gate adds unrelated dependencies.

```text
T001 Evidence/classification + native setup/CI
  └─ T002 Authenticated session routing + addon/export boundary
       └─ T003 Bounded caller/worker + confined D
            └─ T004 US1 real clean-open vertical slice
                 ├─ T005 US2 dirty/divergent/changing document ─┐
                 ├─ T006 US3 source routing/session loss ─────┼─ T008 Whole-feature acceptance
                 └─ T007 US4 closed/unavailable/limits ───────┘
```

- **First independently useful US1 slice: 4 completed task PRs**, not the previous 15. T001–T003 are independently reviewable architectural increments, not user-facing observation completion.
- **Full Feature 001: 8 completed task PRs.** Under the default workflow, each is delivered and reviewed/merged before starting the next. No branch/PR is created by this task-list refinement.
- Keep US2/US3/US4 separate rather than mixing all live behavior into one large US1 PR. Shared safety primitives are already mandatory, but each story's expanded live behavior and independent acceptance remain in its own capability PR.
- Keep one cumulative gate because mixed-state twenty-read invariance and the complete support matrix require all stories. Redaction, export isolation, dependency review, CI safety, and routine real-editor assertions are **not** deferred to that gate.

## Consolidation Map — Previous IDs to Current Tasks

The left column refers only to the previous 35-task list, not the current IDs. Some old tasks mixed responsibilities that now follow the boundary they protect; these are explicitly distributed below. No requirement is dropped by renumbering.

| Previous task(s) | Current owner(s) | Reason |
|---|---|---|
| Old T001, T003, T004, T005 | T001 | Package/toolchain/native CI, types and reducer form one executable reusable semantic capability, with intrinsic/contract tests and dependency evidence. No empty setup or enum-only PR. |
| Old T002 | T002 initially; T004–T007 extend fixtures with their capabilities | Synthetic project/preset and later scenario preparation ship where they first provide independent evidence, not as a standalone fixture PR. |
| Old T006 | T002 framing/session DTOs; T003 observation/worker/public-result DTOs | Boundary validation is reviewed with the working boundary and its receiving-side tests, not an isolated codec PR. |
| Old T007 | T002 private registry; T003 confined D | The old filesystem task spanned two security responsibilities; each now ships with its actual caller and tests. |
| Old T008, T009 | T002 | Addon lifecycle, authenticated source-free routing and selection are one tested cross-language capability. |
| Old T010 | T002 real bootstrap; T003 bounded observation caller/worker | Bootstrap belongs to session setup; observation supervision is independently reviewable execution behavior, not placeholder CLI wiring. |
| Old T011, T012 | T001 core cases; T002/T003 boundary cases; T004 live integration/rechecks | Remove test-only prerequisite PRs; put behavioral tests beside the implementation that establishes the contract. |
| Old T013, T014, T015 | T004 | Collector, caller integration and independent clean-open GUI acceptance must be reviewed together to establish US1. |
| Old T016, T017, T018, T019, T020 | T005, with shared semantic invariants in T001 and initial recheck safety in T004 | Dirty/change contracts, fixtures, attribution, invalidation propagation and live proof form one US2 capability PR, not five fragments. |
| Old T021, T022, T023, T024, T025 | T006; foundational boundary regressions ship in T002/T003 | Source-bearing routing/lifecycle/interruption integration and its complete US3 evidence ship together. Earlier security/executor boundaries must already pass their own tests. |
| Old T026, T027, T028, T029, T030 | T007; initial availability/limit enforcement in T001/T003/T004 | Closed/partial collection, fixtures and per-source-limit evidence form one US4 capability, using already-safe shared semantics. |
| Old T031 | T008 | Retain one genuinely cumulative mixed-state non-interference gate after all stories, not a routine delayed unit-test PR. |
| Old T032 | T002–T007 as paths appear; complete replay T008 | Authentication/privacy/redaction belongs with each exposed metadata/source/failure path, not a late hardening PR. |
| Old T033 | T002 export implementation/proof; T004–T007 regressions; T008 replay | Tooling isolation must be established when the addon/fixtures enter, then maintained as their contents grow. |
| Old T034 | T004 protected live CI; T008 complete matrix | Introduce the runner path with the usable GUI harness; enable complete-matrix acceptance when all groups exist. Native CI is already T001. |
| Old T035 | Per-task docs/evidence plus T008 cumulative reconciliation | Routine documentation and guarantees accompany capabilities; only full-feature/support reconciliation remains cumulative. |

## Complete Coverage and Acceptance Ownership

### All Functional Requirements

| Requirement | Implementation and evidence owners |
|---|---|
| FR-001 | T001 identity model; T002 explicit project/session selection; T004 real document identity; T006 namesake/exact-selection source proof |
| FR-002 | T002 source-free unique-target gate; T003 no source before authorization; T006 ambiguous/exact routing evidence |
| FR-003 | T001 lifetime/invalidation semantics; T002 fresh sessions; T004 rechecks; T005 document replacement; T006 ended/replaced editors |
| FR-004 | T001 validity/open-state distinctions; T004 known-open attribution; T007 closed/missing/invalid/unknown and syntax-invalid cases |
| FR-005 | T003 independent bounded D; T004 independent R/B and empty/exact text; T005 divergence; T007 built-in/partial/per-source limits |
| FR-006 | T001 observation/reason/invalidation records; T003/T004 validated evidence; T005 invalidated facts; T007 unavailable/not-applicable distinctions |
| FR-007 | T001 dirty orthogonality; T004 actual clean evidence; T005 unsaved/equal-text/unavailable/unattributed dirty; T007 closed dirty not-applicable |
| FR-008 | T001 exact pairwise/overall comparisons; T004 clean/empty; T005 complete/partial divergence; T007 missing-source agreement unknown |
| FR-009 | T001 complete/limited rules; T004 positive clean path and failed-recheck refusal; T005 complete dirty/divergent and limited unknown-dirty |
| FR-010 | T007 confirmed closed with cached/unloaded R, no B/dirty fabrication and no force-load |
| FR-011 | T002–T007 passive product paths and per-PR non-interference; T008 twenty-read source/selection/history proof |
| FR-012 | T001 clocks/authority records; T003/T004 fresh request-local evidence and safe diagnostics; T006 interrupted attribution; T008 repeated observations |
| FR-013 | T001 consistency/staleness rules; T003 D recheck; T004 editor recheck; T005 source/identity changes and evidence-backed stale R |
| FR-014 | T001 reducer/precedence; T003 structured caller/worker outcomes; T005 attribution limitations; T006 refusal/interruption; T007 document/surface distinctions |
| FR-015 | T003 independent deadline/blocked-worker proof; T004–T007 timed live cases; T006 suspended-editor proof; T008 all-case timing gate |
| FR-016 | T002 private metadata/authentication/local-only gate; T003 capability-confined D; T004–T007 source/token privacy and denied-target evidence; T008 full redaction replay |
| FR-017 | T001 reusable semantics; T002/T003 private DTO boundaries; T004–T007 reuse one reducer, no MCP or gameplay authority |
| FR-018 | T001 exact native/dependency evidence; T002 exact-editor gate/export evidence; T004–T007 real GUI acceptance; T004 protected CI; T008 actual matrix/support evidence |

### All 21 Acceptance Scenarios

Each row names the task that must deliver the complete scenario evidence, not merely a model field or a test stub. Shared boundary evidence remains required in its earlier owning task.

| Scenario | Required observable result | Task / quickstart group |
|---|---|---|
| US1.1 | Correct project/session/script; independently clean, open D/R/B; complete agreement | T004 / `clean-open` |
| US1.2 | Fresh repeated observation; no source/dirty/selection/history change | T004 / `clean-open` |
| US1.3 | Actual observed empty source, not unavailable | T004 / `clean-open` |
| US2.1 | Human unsaved B intact; actual independent D/R; attributable dirty | T005 / `dirty-divergent` |
| US2.2 | Complete divergence with every known pair classified correctly | T005 / `dirty-divergent` |
| US2.3 | D=R=B cannot imply clean when dirty evidence is unavailable | T005 / `dirty-unavailable` |
| US2.4 | Detected source/identity change invalidates affected facts, no current-complete claim or reused snapshot | T005 / `changing-document` |
| US2.5 | Unattributable unsaved indication gives dirty unknown/limited, not ambiguity/disconnect | T005 / `dirty-unavailable` |
| US3.1 | Ambiguous target, actionable selector feedback, zero candidate source | T006 / `routing` |
| US3.2 | Exact selection returns only intended project/session/script despite namesakes | T006 / `routing` |
| US3.3 | Absent editor yields structured unavailable, no cached state/startup | T006 / `session-loss` |
| US3.4 | Known disconnect preserves labeled partial evidence without complete claim | T006 / `session-loss` |
| US3.5 | Ended session never silently replaced by same-project restart | T006 / `session-loss` |
| US3.6 | Connected unresponsive editor yields bounded timeout, distinct from known loss | T006 / `deadline` |
| US3.7 | Outside-project/nonlocal target refused without source disclosure | T006 / `confinement` |
| US4.1 | Confirmed closed, actual cached/unloaded R, B/dirty not-applicable, no open/load | T007 / `closed-and-invalid` |
| US4.2 | Missing versus non-GDScript invalid versus valid closed; syntax error alone not invalid | T007 / `closed-and-invalid` |
| US4.3 | Known-open unreadable B stays open/unavailable with other facts retained | T007 / `surface-limits` |
| US4.4 | Open missing/unreadable D retains independent R/B and no full agreement | T007 / `surface-limits` |
| US4.5 | Unobservable R is unavailable with reason, never copied from D/B | T007 / `surface-limits` |
| US4.6 | Unknown open state stays unknown/limited, never inferred closed | T007 / `surface-limits` |

### Success Criteria, Edge Cases and Cross-Cutting Gates

| Criterion | Completion evidence |
|---|---|
| SC-001 | T004/T005 independently exact clean/unsaved D/R/B and actual dirty, with human edit preserved |
| SC-002 | T001 invariants plus T005 real complete divergence; T007 missing/limited sources never imply equality/cleanliness |
| SC-003 | T004–T007 all seven minimum classes and additional target/open-state/timeout/replacement outcomes; T008 full replay |
| SC-004 | T003 bounded executor, T006 stalled editor/worker, T004–T008 every controlled request within five seconds |
| SC-005 | T008 twenty observations, six clean/eight dirty/six closed, fresh evidence and zero observer interference |
| SC-006 | T001/T003 structured facts/reasons, each story's result-only interpretation, T008 complete reviewer reconciliation |
| Edge: non-selected script | T005 actual non-selected B/dirty or explicit open-but-unobservable; selection remains unchanged |
| Edge: duplicate names/basenames/same project | T002 identity/selection safety; T006 distinguishable real source attribution |
| Edge: rename/remove/close/replace during inspection | T005 barrier-controlled invalidation without mixed documents |
| Edge: whitespace/line-ending-only differences | T001 exact comparisons, T003 exact disk text, T005 real divergence |
| Edge: no attributable standalone D | T007 built-in D limitation, never unrelated container content |
| Edge: buffer equal to D but still dirty | T005 independently observed editor dirty, not equality inference |
| Edge: known-lagging R/B | T005 actual source with specific stale evidence, otherwise unknown staleness; no cached observation |
| CHK032 per-source limits | T001/T003/T004 initial semantics/enforcement; T007 independent D/R/B and closed D/R limits, preserved earlier evidence, stronger outcome precedence |
| CHK037 metadata scope | T002/T003 private operational metadata only, no arbitrary filesystem exception; T006 confinement/redaction; T008 replay |
| Constitution I/II/III/IV | T001–T007 independent authorities, human-work preservation, public native getters, truthful observed/limited/invalidated outcomes; no mutation-success claim |
| Constitution V/X | T002/T003 authentication/confinement/bounds and safe structured diagnostics; every source path's sentinel checks; T008 full `redaction` |
| Constitution VI/XII | Each live capability's real-editor regression evidence; T008 complete exact-version acceptance and retained discovered regressions |
| Constitution VII/IX | T001 reusable core; private Godot/caller adapters; one observation operation and bootstrap, no second safety model/tool catalog |
| Constitution VIII | T002 enabled/disabled export artifacts and actual owned exported launch; regressions when addon/driver paths change; T008 replay |
| Constitution XI / dependency gates | T001 actual tracked-lockfile license/provenance/advisory review; maintain with changes and revalidate for T008 |
| Compatibility / CI / review | Version-1 fields/outcomes and deliberate migration policy preserved; T001 native CI, T004 protected GUI CI, T008 actual supported-matrix evidence; constitutional compliance in affected PR planning/review |

Mutation A/B/D/E, applied-edit Save/reparse/rescan/runtime durability, and gate C's product UndoRedo transitions are inapplicable because the feature performs no mutation and claims no UndoRedo support. Fixture-only human edits/history actions and exported-fixture launch do not add these product capabilities. This scope justification does not waive live-editor observation, privacy, export isolation, or later mutation gates.

All **13 quickstart groups** remain owned: `clean-open` T004; `dirty-divergent`, `dirty-unavailable`, `changing-document` T005; `routing`, `session-loss`, `deadline`, `confinement` T006; `closed-and-invalid`, `surface-limits` T007; `sequential-readonly` T008; `redaction` incrementally T002–T007 with full replay T008; `export-boundary` T002 with affected-path regressions and full replay T008. T008 requires `--scenario all`, not a passing subset.

**Completion boundary:** All 18 functional requirements, 21 scenarios across four stories, six success criteria, seven edge cases and applicable constitutional gates remain required. Task consolidation preserves that coverage; the associated planning authentication correction is documented in research and the bridge contract without changing task IDs or dependencies. Completing a prerequisite or US1 does not complete Feature 001; completing Feature 001 does not complete roadmap Phase 1. No implementation task has been started or completed here.
