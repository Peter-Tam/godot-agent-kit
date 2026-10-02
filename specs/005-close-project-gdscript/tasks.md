---
description: "PR-sized implementation tasks for safe clean-document GDScript closing"
---

# Tasks: Safely Close a Clean Project GDScript

**Input:** [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md), [data-model.md](data-model.md), [caller v1](contracts/close-api.md), [private bridge v5](contracts/bridge-protocol.md), [native revision 3](contracts/native-integration.md), and [quickstart.md](quickstart.md).

**State:** **2 / 3 complete**. T001 and T002 are complete, with [native acceptance](quickstart.md#9-t001-native-boundary-acceptance-2026-10-02), [public-caller and full affected-suite acceptance](quickstart.md#10-t002-public-caller-acceptance-2026-10-02), and [T002 shape/constitutional review](plan.md#t002-implementation-shape-and-constitutional-review--2026-10-02). T003 remains pending and unstarted. Selection-time granularity and consistency review passed; later prerequisite currentness follows repository workflow policy. Completion is independent of PR state; dependent implementation still waits for merge or explicit stacked-PR authorization.

**Organization:** Three coherent increments: the guarded native close/continuation boundary and real context validation with coordinated v5/revision-3 compatibility; the complete public caller and consumed core; cumulative repeated/composed workflow acceptance. All four P1 stories retain their own phases and independent criteria. US2/US3 and first-use US4 safeguards are inseparable from US1 and belong to T002, not later safety/test PRs. Cross-references do not create duplicate task checkboxes.

## Working Agreement and Format

- Each checklist row is one Spec Kit implementation task and one PR, in the form `- [ ] Tnnn [P?] [USn?] Description with exact file paths`. Its indented scope, tests, verbatim constraints and acceptance are part of that task description. Commands, files, internal steps and subagent work are not separate tasks.
- Follow the [constitution](../../.specify/memory/constitution.md), [working agreement](../../AGENTS.md) and approved feature artifacts. Complete granularity review before `/speckit.analyze`; resolve actual analysis blockers before implementation. This document does not run either analysis or implementation.
- Select exactly one dependency-ready ID; preserve unrelated changes, fetch updated `main`, and branch `task/<task-id>-<description>` from the authorized base. Verify `SPECIFY_FEATURE_DIRECTORY=specs/005-close-project-gdscript` before installed helpers; task branches alone do not select the feature. Inspect intended/staged changes, stage explicit paths, commit, push, open the dedicated PR, report validation and STOP. No automatic merge or next task; stacking needs explicit authorization. After confirmed merge, safely delete remote/local source branches under repository policy.
- Mark only the selected task `[X]` after its acceptance, required validation and implementation-shape review pass on its delivery head, independently of PR review/merge. Update this file and `PROJECT_STATUS.md` for material lifecycle changes. Completion does not bypass normal prerequisite PR-merge sequencing.
- **Tests are required by FR-017 and the constitutional boundary/coherence gates.** Add meaningful failing-before regressions where executable, implement their behavior, then integrate and prove it. Tests, actual smoke and directly necessary docs ship with their implementation, not tests-only precursor PRs. No source-text, forwarding, mock-echo, copied-field or bare-not-throw tests; no universal-refusal substitute for required positives.
- Rust owns checked intent and terminal reduction; CLI/JSON/process/transport stay at adapters; Godot state and native effects stay at the addon/integration boundary. Reuse the existing checked basis, local authentication, confinement, one shared slot, source-only validator and owned harness. No MCP, Save/discard, batch/reopen compound command, automatic retry, rollback, new crate/service/framework, wider platform or CI-provider prerequisite.
- The accepted FR-005 clarification permits only guarded ordinary native revalidation of unchanged already-loaded source. It does not permit applying differing pending B to R, explicit reload/reparse/rescan, source writes, new project-code execution, manufactured eligibility or lost human history.
- Before completion, review materially changed module responsibilities, current-consumer visibility, lifecycle states, speculative APIs and recoverable evidence errors. Review cohesion around 800–1000 lines without an arbitrary size limit. Necessary task-introduced cleanup belongs in that task; do not create a later refactor checkbox or expand into unrelated refactoring. Run LSP references before exported-symbol changes and migrate actual consumers.
- Every task owns current contract/operator updates and reproducible evidence in `specs/005-close-project-gdscript/quickstart.md`, with constitutional/shape review in `plan.md`. Preserve historical acceptance; amend current reproduction notices only. Do not commit credentials, generated native/editor state, raw source-bearing evidence or temporary probes.

### Paths, Candidate and Shared Verification

Paths are repository-relative; new paths below are implementation locations, not scaffolds created by this command. Simple responsibility-based private subdivision is allowed when actually consumed and justified by cohesion. Existing packages, dependencies and harnesses need no setup task.

Use Rust 1.98.1/edition 2021, the current locked dependencies, thin GDScript, public-ABI C++17 and Python 3.10+. The exact candidate is official Godot `4.7.2.stable.official.ed1daf0bf`, commit `ed1daf0bf001b61586d9930840f2f1394092c079`, executable SHA-256 `c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf`, macOS 26.6.2 build 25G83 arm64. Use matching templates and separately built normal/fixture-fault native bundles. No broader support is inferred from earlier features or seven planning controls.

For affected Rust work, from `mcp-server/`:

```sh
cargo +1.98.1 fmt --all -- --check
cargo +1.98.1 clippy --all-targets --locked -- -D warnings
cargo +1.98.1 test --locked
cargo +1.98.1 doc --no-deps --locked
```

Default tests include doctests. Build actual consumers with locked resolution and the existing native builder; do not request `close-gdscript` before T002 creates it. Use [quickstart §1–§3](quickstart.md#1-exact-candidate-and-owned-prerequisites) for provenance/build/runner conventions. Apply existing native-build, campaign and workflow checks only where changed.

Exercise actual process/bridge/filesystem boundaries and owned visible Godot windows with independent D/R/B, document/file/selection identity, dirty/current/saved-version/Resource-edited and real-history witnesses. Run the failed/affected scenario first during development; collect integrated results before broader verification. Real-editor executions run serially with fresh private artifacts, explicit deadlines, owned-window proof and cleanup. Preserve T001's justified v5/native-family/shared-owner/context full observation/edit/open/discovery evidence and all T002 acceptance obligations/facts. T003 supplies new sequential/full composed execution and cumulative VALID coverage under [TEST_POLICY.md](../../TEST_POLICY.md), not unconditional historical reexecution at the literal final commit. Planning probes, primitive receipts or headless runs cannot replace required live evidence.

## Phase 1: Setup — Reuse the Completed Foundation

**Purpose:** Reuse completed Features 001–004, the existing Cargo package/lock/toolchain, native builder, addon, registry/authentication, project confinement, operation ownership, source-only validation and owned-editor harness. No directory, dependency, formatter, generic infrastructure or standalone documentation task is justified. Necessary fixture/build setup belongs with its first consumer.

**Entry criteria:** Approved artifacts, completed granularity review and subsequent consistency analysis, normal delivery sequencing, and the selected task's actual permitted validation environment. Generating this list does not select or start T001.

## Phase 2: Foundational — Guarded Native Closing and Compatible Integration

**Goal:** Establish an independently exercised native close/continuation boundary and exact-source validation, while current observation/edit/open/discovery continue to work across the coordinated private migration. No public close caller or unfinished advertised close exchange.

- [X] T001 Deliver guarded native closing, real protected-source validation and the coordinated bridge-v5/native-revision-3 migration in `godot-addon/native/script_close.cpp`, `godot-addon/addons/godot_agent_kit/script_close.gd`, `mcp-server/src/runner/stock_validation.rs` and `mcp-server/src/bridge.rs`, bundling native-boundary coverage in `godot-addon/tests/run_script_close.py`, `godot-addon/tests/fixtures/script_close/fixture_driver.gd` and `specs/005-close-project-gdscript/quickstart.md`.
  **Depends on:** Completed Features 001–004 and the reviewed-artifact/analysis entry criteria; no earlier Feature 005 task. This is one independently verifiable native capability/cutover PR, not separate method, version, context-extraction, signal or test PRs.

  **Deliverable and current consumer:** Implement every native revision-3 close entry and the thin internal addon owner from the native contract. The owned fixture drives the real owner through the existing shared-slot private-fixture route and consumes actual Rust `close_context` validation. No public close binary, unused Rust close reducer or product success verdict is introduced. Keep authenticated `close_gdscript` **false until T002 implements the complete product exchange**; existing edit/open capabilities remain truthful and usable. The ninth capability key still participates in the v5 transcript now. A private tested owner is not a fake product handler or a reason to advertise unfinished capability.

  **Tests first, native owner and exact guards:** Add native-boundary behavior cases before the corresponding implementation where executable. Create `godot-addon/native/script_close.cpp` with private declarations as needed; extend `godot-addon/native/native.hpp`, `session.cpp`, `extension.cpp` and `godot-addon/addons/godot_agent_kit/script_close.gd`, `bridge.gd`, `plugin.gd`. Reuse one native Session edit/open/close owner and the addon `_active` slot. Implement passive inspect/recognition; immutable independent clean target D/R/B and exact prior/file/Script/editor/buffer/version checks; read-only descriptor/namespace guards; and all remaining supported document protection. Dirty/equal-dirty targets refuse; dirty unrelated R==B can qualify but R!=B refuses. Do not require target parse validity or write permission, select the target to inspect it, or reuse edit/open application state machines.

  **Shared current-source safety:** Extract only the actual common lexical/compiled/effective-context acquisition from `godot-addon/native/open_context.cpp`, `open_context.hpp` into `godot-addon/native/editor_context.cpp`, `editor_context.hpp`; keep opening-specific orchestration and all existing behavior intact. Reuse `godot-addon/native/document_guard.cpp` and `document_guard.hpp` for file/document ownership. Preserve tool/base/export/Object/dynamic-property/static/load/preload/global/autoload/ClassDB/warning protections, actual history availability and idle-parse-delay capture. Guard the full bounded roster, including history-back/sort destinations; no inferred next tab, private history getter, pending-export-empty fiction, second scanner or general callback framework.

  **Real validation and bound authorization:** Extend `mcp-server/src/runner/stock_validation.rs` and the affected `mcp-server/src/runner/stock_validation/admission.rs`, `protocol.rs`, `ownership.rs` plus `mcp-server/examples/stock_validation_fixture.rs` for the distinct `close_context` purpose. The fixture is a real current consumer, not a public arbitrary-source operation. Validate actual private captured remaining R==B, one child at a time, bound to request/session/target/protected path/identities/source/guard/effective context, per-URI diagnostic/parser-symbol fences and cleanup. Recompute the typed F/E aggregate and receipt bindings independently on the Rust/fixture side. No target child, no child for recognition or an empty remaining set. Failed/missing/wrong-purpose/wrong-document receipts cannot authorize; no native `valid` Boolean substitutes for supervisor proof. Preserve current opening/edit validation and migrate all current purpose/type consumers found by LSP. The original native lease and owned fixture attempt deadline cannot renew per helper, stage or waiter; T002 later supplies the actual caller-clock supervisor.

  **One native effect and finite continuation:** Implement same-turn final target/roster/source/flags/settings/file guards, mark entered before one generated-bound `ScriptEditor.close_file(original_path)` without event pumping, and retain actual Error/selection/removal facts. Observe actual `editor_script_changed` visits and exact protected `edited_script_changed` completions using request-owned callbacks. Pre-entry/replacement events do not discharge obligations; later visits require new attributable completion. Never force/flush validation, stop timers, reclose or infer completion from equality/delay. Retain entered call/callback data through cancellation/disable until return; original expiry prevents new stages, and cleanup disconnects only owned safe state without cancelling ordinary Godot validation or claiming rollback.

  **Independent final facts:** Ordinary target collection plus separately reacquired Rust D and native checks must distinguish intended roster minus the original target from unexpected removals/new/reopened documents. Release target-only Resource holds before ordinary post-close R sampling; retained R must keep its original Script identity/source, or genuinely report unloading. Preserve every protected source/identity/flag/version/history and allowed selection relation. Native completed/OK is only a fact, never public verified closure. No source, selection, history or reopen compensation during finish/abort.

  **Coordinated current-consumer migration:** In this same task update `mcp-server/src/bridge.rs`, `mcp-server/src/bridge/wire.rs` and existing edit/open/discovery/observation wire consumers; `godot-addon/addons/godot_agent_kit/bridge.gd`, `plugin.gd`, `observation.gd`, `script_edit.gd`, `script_open.gd`, `script_open_transport.gd`, `script_discovery.gd`; native builder/registration/manifests/family checks in `godot-addon/native/build.py`, `native.hpp`, `extension.cpp`; and actual fixtures/operator consumers. Use domain `godot-agent-kit/editor-bridge/v5`, the exact ordered nine Boolean keys, typed length-prefix HMAC and revision 0-or-3 with actual matching build ID. Retain `editor_integration.gdextension`, `libeditor_integration.macos.arm64.dylib`, `editor_integration_library_init` and metadata key `godot_agent_kit_native`. Reject old peers/revision 2 without fallback/alias; native-unavailable observation/discovery still work. No library rename or incompatible intermediate head.

  **Fixture, boundary and build evidence:** Create the real `native-boundary` group in `godot-addon/tests/run_script_close.py` and fixed private controls/inert sources under `godot-addon/tests/fixtures/script_close/fixture_driver.gd`. Reuse `godot-addon/tests/fixture_bridge.gd`, `run_observation.py`, `run_script_edit.py`, `stock_acceptance.py` and existing ownership/window/history/export facilities. The T001 native group must run without a nonexistent `--closer` executable; document its actual command now, and reserve public-only flags/groups for T002. No fake/no-op `all`, product group or dummy closer. Extend `mcp-server/tests/bridge_boundary.rs`, `confinement.rs`, existing validator tests and actual cross-language fixtures for v5 proofs/F/E vectors, capabilities/build metadata/reflection/replay/old-peer rejection, exact bounds/known fields and purpose binding. Extend `godot-addon/tests/test_native_build.py` and current native/observation/edit/open/discovery fixture consumers for revision 3; separate `GAK_FIXTURE` controls must never enter the normal artifact. Change `.github/workflows/ci.yml`, `live-editor.yml` and `.github/tests/test_live_editor_execution.py` only where real current build/execution inputs require it; no new workflow/provider.

  **Independent acceptance:** Pass actual selected/non-selected/last-tab, stable two-document, empty, safe syntax-invalid and read-only positives; retained-original and naturally unloaded R; dirty/equal-dirty/stale/missing/unsupported/effect/limit refusals; dirty-current/background R==B with real earlier Undo/Redo; and the demonstrated R!=B refusal before close entry. Exercise multiple history/sort destinations, known configuration/idle-delay changes, exact/one-over limits, missing/wrong/premature/replacement/completion events, callback/owner lifetime, duplicate advance/wait, no lease renewal, file/roster/session/same-text identity races, lost/expired/disabled attempts, overlap and no late close. After native OK/completion, a deliberately invalidated independent witness must still prevent a verified claim. Independently observe preservation, not receipt echoes. Then pass complete existing observation/edit/open/discovery suites, including A–E/history/durability and enabled/disabled/hook-only export checks, on the integrated cutover head with the applicable Rust/native/workflow checks. Review actual owned windows, private source/credential sentinels and child/registry cleanup.

  **Docs and completion:** Update current migration/install/restart notices in `godot-addon/native/README.md`, `.github/README.md`, `specs/001-observe-gdscript-state/contracts/bridge-protocol.md`, `specs/001-observe-gdscript-state/quickstart.md`, `specs/002-edit-open-gdscript/contracts/bridge-protocol.md`, `specs/002-edit-open-gdscript/quickstart.md`, `specs/003-open-project-gdscript/contracts/bridge-protocol.md`, `specs/003-open-project-gdscript/quickstart.md`, `specs/004-discover-project-gdscript/contracts/bridge-protocol.md` and `specs/004-discover-project-gdscript/quickstart.md` where current reproduction changes. Record actual private/native scope, commands, provenance, limitations and shape/constitutional review in this feature's contracts, plan and quickstart; preserve old acceptance as historical. Mark only T001 complete and leave product close/Feature 005 incomplete.

  **Constraint ownership:** The following exact model constraints govern this boundary and are inherited by T002/T003. T001 implements native/addon producers, real validator receipts and private-fixture acquisition, not a duplicate public terminal reducer. Caller-only stdin/output fields in the shared bounds belong to T002. All exact tuple/response fields, nullability, enums, clocks and rejection rules in both private contracts remain normative; no missing future consumer justifies a speculative API.

  **Verbatim data-model constraints — §2 Fixed supported profile and bounds:**

  > | Item | Bound / rule |
  > |---|---|
  > | Operation | One existing project-local external `.gd` in one authenticated editor lifetime; no batches, Save/discard/force/retry/reopen option. |
  > | Candidate | Exact official Godot 4.7.2 commit/executable and macOS arm64 recorded in [research.md](research.md#3-exact-candidate-and-observed-provenance); public-ABI native integration revision **3**, private bridge **5**. |
  > | Target locators | Existing project root ≤1024 UTF-8 bytes, resource path ≤2048, request ID ≤64 checked ASCII characters, session ID existing 32 lowercase hex format. |
  > | Target source | Each D/R/B ≤512 KiB UTF-8; effectful close requires exact LF UTF-8, no BOM/CR/NUL or unsupported control characters. Empty source remains a value. No write permission or Save-format eligibility required. |
  > | Open-document effect context | At most **8** open standalone GDScript documents including the target; exact unique paired Script/editor/CodeEdit associations. A no-source/empty editor is valid for already-closed recognition. Unsupported mixed/custom/external-editor contexts refuse effectful closing rather than guessing hidden navigation state. |
  > | Private protection source | Each remaining document ≤512 KiB; sum of retained single R==B source copies across remaining documents ≤512 KiB. The target source is separate. Bound before concatenation/allocation; never silently omit a candidate. |
  > | Effect metadata | Reuse opening's current source/compiled-property/method/ClassDB/global/autoload/warning bounds; cap the aggregate close non-source context projection at **256 KiB**. Set-like collections are sorted and unique. |
  > | Caller stdin | Exactly one UTF-8 JSON value plus EOF, ≤12 MiB; strict schema and duplicate/unknown-key checks. `basis` is an observation-v1 result or null, not a new revision-token format. |
  > | Wire / output | Existing depth 32; 4 KiB handshake/control frames, 4 MiB selected source-bearing requests, 12 MiB response/worker/final-result ceilings. Contexts are separate messages from full target snapshots; the source aggregate above does not increase these bounds. |
  > | Timing | 9.5-second operation budget from before input/selection plus 0.5-second delivery reserve; addon remaining lease is an integer 1–9000 ms and never extends the caller cutoff. All validator children share this one budget. |
  > | Diagnostics | At most 64 bounded source-free stage/reason/code records; no unselected paths, source, hashes, credentials or native pointers in public diagnostics. |
  >
  > The eight-document/source-aggregate limits bound the set the native close may visit through history and list sorting; no public API exposes the complete native navigation stack. This is a conservative supported effect profile, not a requirement to close other tabs or change editor settings. A stable two-document ordinary project is a required useful positive; excessive/unsupported context refuses before closing. Limits are not claims that every maximum-sized input must succeed rather than return a bounded explicit non-success.
  >
  > Already-closed recognition does not run effect-profile admission, enumerate protection candidates for validation, require target parse validity or apply the multi-document source aggregate to ordinary observation. Its source-local availability behavior remains that of observation v1. Missing/invalid disk targets and unknown open state are not already closed.

  **Verbatim data-model constraints — §4 TargetPreparation:**

  > A request-local, immutable preparation contains:
  >
  > - Authenticated resolved project/session/path, request ID and native build/revision.
  > - Caller-clock independently read D and existing file identity/namespace witnesses.
  > - Fresh ordinary target observation, exact paired Script/editor/CodeEdit identity, D/R/B comparison and buffer dirty evidence.
  > - Separately acquired target Resource-edited flag; both buffer dirty and Resource edited must be observed false for effectful closing.
  > - Native read-only project/parent/leaf descriptor binding, independently compared D bytes, current/saved versions and target source commitment.
  > - Selection relation and exact private identity before dispatch, effective settings/profile and the complete bounded protection set below.
  > - An aggregate immutable preparation guard, original expiry and causal invalidation state.
  >
  > No target is opened, loaded, selected, saved, compiled, retagged or repaired to construct this evidence. A safe syntax-error target can prepare: closing does not require its parse result to be valid. The same source/effective-context exclusion rules as native opening prevent unsupported tool/export/static/dependency/dynamic-property effects, but no target validation child runs solely to prove close eligibility.

  **Verbatim data-model constraints — §5 ProtectionSet and validation receipts:**

  > ### Capture and purpose
  >
  > Native closing may visit remaining tabs through history and script-list sorting, and may apply current pending text before scheduling validation. Therefore capture **every remaining supported open script document**, not only the currently selected one or an inferred history-back target. Keep the target separate so its permitted syntax error does not become a remaining-document parse-validity gate.
  >
  > Each `ProtectedDocument` contains the existing opening-style source-context record: exact project-local path/file association; Script/editor/buffer IDs; independently equal R/B source; source hash/length; current/saved versions; attributed dirty and Resource-edited flags; native undo/redo availability; tool/base/compiled property/method facts; effective source-identifier ClassDB bindings and global/autoload/warning context. Capture selection separately. Dirty R==B is permitted when every guard passes; D may legitimately differ. Dirty R!=B, unknown attribution, effectful/unsupported source or compiled metadata refuses. Do not wait for idle parsing, set R, Save or tag B to make it eligible.
  >
  > Reuse the existing bounded lexical/compiled effect profile and source-only stock validator described in the [opening native contract](../003-open-project-gdscript/contracts/native-integration.md#2-supported-new-open-profile). Extract only the actually shared context acquisition/profile helpers to a durable private `editor_context` responsibility; opening retains its existing current-document wrapper and public behavior. Closing applies those helpers to its bounded explicit records. No second parser, callback registry or new service.
  >
  > ### Per-document preflight
  >
  > The existing supervisor owns one one-shot stock validation child at a time for each remaining document, all within the original attempt deadline. Purpose is **`close_context`**, separate from `open_context` and every edit purpose. Require exact captured source, applicable per-URI diagnostic/parser-symbol fences, effective context and owned-child cleanup. Target syntax validity is not requested. No helper is needed when there is no remaining document.
  >
  > A `CloseContextValidation` receipt binds:
  >
  > | Field | Meaning |
  > |---|---|
  > | `request_id`, `session_id`, `target_path` | The owning close request, not a free-standing parser request. |
  > | `document_path`, `script_id`, `editor_id`, `buffer_id` | Exact private protected document. Never exposed through public close output. |
  > | `source_sha256`, `utf8_bytes` | Source actually validated, equal to the native preparation source. |
  > | `guard_sha256` | The immutable protected-document projection, including identities/versions/profile facts. |
  > | `context_sha256` | Existing validator effective context commitment, distinct from the native guard. |
  > | `state`, `collection`, `cleanup` | Actual completed-valid/invalid/unavailable, caller-domain interval/fences and owned-child cleanup state. Only completed-valid, fully attributed, cleaned-up receipts authorize closing. |
  >
  > Reuse the validator's existing source-only capture/admission/private HOME/XDG/owned-child handling and inherited endpoint limitation. This is a private safety consumer, not a new public validation or execution tool. Another session/document/purpose's result cannot authorize this attempt. Do not serialize a caller-supplied `valid` Boolean as proof.
  >
  > ### Guards and preservation
  >
  > Before native entry, independently recheck the complete document roster, target identity/source/clean flags, protected identities/R/B/versions/dirty/Resource flags/history availability, effective context and relevant ClassDB/property facts. Compare every receipt and guard. A same-text edit/version change, newly opened tab, closed/reopened document, changed selected identity or configuration change invalidates preparation; do not recapture/revalidate/retry automatically under the old request.
  >
  > After the authorized close, compare against the prepared roster minus the exact original target editor/buffer; that sole removal and a separately observed permitted native fallback selection are expected effects, not invalidation. Every protected identity remains required. A new/reopened/replacement document or another missing editor invalidates the result. Post-close checking recomputes protected guards and observes target absence/retained state; it does not demand equality with the pre-close aggregate hash whose roster still contained the target.
  >
  > Metadata/file access remains confined to the selected project. Remaining-document D is not required to equal unsaved R/B and is never used to overwrite them; existing context path/file guards and the stock route's no-write behavior apply. Acceptance independently checks unrelated D as well as actual R/B/history. Source/code references held for preparation are real authorities, not copies used to fabricate observations.

  **Verbatim data-model constraints — §6 NativeCloseFacts and continuation ledger:**

  > ### One native effect
  >
  > The native entry marks `closing: entered` before invoking the generated-bound public `ScriptEditor.close_file` on the exact freshly rechecked path. There is one authorized call, no method-name input and no event pumping inside it. The engine method hash comes from the pinned executable's generated API. Record actual `Error` return or call failure; `OK` alone does not establish verified closure or preservation.
  >
  > Native facts preserve request/session/path/build binding, phase, target pre-close identities, entry/return stamps, actual close error, independently observed removal of the old document if known, native selection/target-history effects, sticky invalidation, and irrevocable pre-entry discard. Node IDs are not ownership: validate live Script/editor/buffer association at each access and retain actual Resource references only while required. After closure, release the attempt's target-only Resource references before the ordinary post-close R observation so retention/unloading is not manufactured; keep only attributed IDs/digests/facts, never dereference freed editor/buffer pointers.
  >
  > Native `phase` is the closed enum `inspected|prepared|entered|returned|settling|settled|terminal` defined in the native contract; core selection/authorization states remain separate. Stage facts may survive terminalization and do not grant authority. On the newly closed branch, a present post-close R must have the original Script identity and unchanged source; a same-path replacement is invalidated evidence, not retained R. Recognition of a document already closed remains a fresh no-effect observation, not certification of old open state.
  >
  > ### Actual native validation completion
  >
  > Before the effect, the native attempt installs bounded, attempt-owned, non-mutating signal witnesses on the ScriptEditor and the exact protected ScriptEditorBase instances:
  >
  > - `editor_script_changed` during the synchronous close records which protected documents the native path actually visits. Those visits schedule native validation according to the pinned source. Visits to the soon-disposed target are not surviving validation obligations; its actual destruction must be confirmed.
  > - Each protected editor's `edited_script_changed` records completion of its real native validation, bound to that editor instance and current attempt. Pre-entry events are baseline only; no prior event or event from a reopened/replacement editor discharges a post-close obligation.
  > - A visit outside the admitted set, a changed roster/source/version/flag/configuration or unavailable required signal/lifetime attribution is invalidation, not a reason to widen the set after entry.
  >
  > `CloseContinuation` has `required_editor_ids`, `completed_editor_ids`, callback collection stamps, current state (`not_applicable`, `pending`, `completed`, `unavailable`, `invalidated`) and a reason. IDs are bounded by the protection set. `required_editor_ids` is exact and monotonic for the attempt. `completed_editor_ids` represents only currently satisfied obligations for each editor's latest attributable required visit; it is not historical and is not monotonic. A later attributable visit keeps the editor in `required_editor_ids` but removes it from `completed_editor_ids` until a new matching attributable completion occurs. Historical earlier completion must not satisfy a renewed obligation. After the close has returned, the continuation is complete only when every required editor's latest outstanding visit has a matching attributable completion, with all frozen preservation checks intact. Repeated visits coalesce only for the same unchanged editor/version and actual native idle validation, without reusing an earlier completion for a later visit. A lost/missing callback is unavailable/pending until the deadline, never assumed completed by elapsed time or equal source.
  >
  > No timer is stopped, forced or triggered; no validation method is called to obtain a witness. Public signal observation does not provide arbitrary callback invocation. Synchronous close/callback entry retains the current native owner until return. Waiting for native completion releases the editor thread, not the attempt's ownership; human editing stays possible and invalidates the old preparation. After an irreversible close, timeout or user interference produces known applied-unverified state rather than undoing or repeating closure. Ordinary editor activity can continue after a timed-out caller; it does not revive product closing authority.
  >
  > Private `close_status(request_id)` lets the addon inspect this request-local ledger during its existing frame loop. The addon answers one pending `close_wait` request when the ledger reaches completed/not-applicable, invalidates, becomes unavailable or expires; this is event-driven observation of already-scheduled native work, not a retry, convergence loop, new scheduler or public wait tool.

  **Verbatim data-model constraints — §9 Private interface contract shared by design artifacts:**

  > The [bridge](contracts/bridge-protocol.md) and [native contract](contracts/native-integration.md) must preserve these exact semantic interfaces:
  >
  > - Bridge prefix: `[5, opcode, request_id, session_id, advertised_project_root, script_path, ...]`.
  > - `close_begin`: remaining-budget-ms; fresh target sample and passive inspect, one shared slot.
  > - `close_prepare`: captured target source, compact checked native basis, captured file/project identity; immutable target/protection context and guards.
  > - `close_recheck`: purpose `pre_close|recognition|post_close`; fresh immutable-context/target/file identity checks.
  > - `close_advance`: aggregate guard plus exact per-document completed-valid receipt bindings; only one close call, no arbitrary stage string.
  > - `close_wait`: no extra arguments; one pending response for the existing continuation ledger, bounded by the original lease.
  > - `close_verify`: purpose `recognition|post_close|survivor`; separate ordinary collector acquisition and native postconditions, with independently reacquired Rust D.
  > - `close_finish` / `close_abort`: no extra arguments; release only this attempt and report actual irreversible discard/effect facts.
  > - Native revision-3 family: `close_inspect`, `close_prepare`, `close_advance`, `close_status`, `close_verify`, `close_recheck`, `close_finish`, `close_abort`, `close_expire`. Existing session `close()` remains session teardown, not script closing.
  > - Native prepare takes `(request_id, captured_source, capture_and_basis)`; advance takes `(request_id, guard_sha256, receipt_bindings)`; verify/recheck take `(request_id, purpose)`; other request-local entries take request ID, while inspect takes `(path, correlation)` and expire takes no caller input.
  >
  > Closed typed context hashing reuses the existing length-prefixed `F` and typed `E` encoding from the [opening bridge contract](../003-open-project-gdscript/contracts/bridge-protocol.md#prepare-a-closed-target), not arbitrary JSON canonicalization. Individual shared source contexts retain the existing guard projection/encoding semantics. The close aggregate uses domain `godot-agent-kit/close-context/v1`, target/request/session/project/file/selection identity, target source/version/clean facts, effective configuration (including actual idle-parse delay), sorted complete document-roster identities and sorted protected-document guard commitments. Exclude source bodies, clocks, receive stamps, helper progress and callback counters. Recompute all hashes independently in Rust; native comparisons use fresh values, never only echoed expected hashes.
  >
  > The close result, worker stage facts, native receipts, callback ledger and validator receipt are separate records with separate clock domains. No one receipt can substitute for another or assert the public success enum.

## Phase 3: User Story 1 — Finish Work by Closing the Intended Clean Script (Priority: P1)

**Goal:** Deliver one complete caller-driven safe close, including independent already-closed recognition and every inseparable preservation/interruption behavior. A happy-path-only close is not a useful or permitted MVP.

**Independent test:** Use the actual public caller in an owned visible editor with exact clean D/R/B and prior basis. Prove selected/non-selected/last-tab closure, independent unchanged D, retained-original/unloaded R, actual B absence and completed native continuation; then separately observe/reopen with existing tools. Include safe syntax-invalid, read-only and empty source and zero-effect already-closed cases. US2/US3 phase criteria also gate this delivery.

- [X] T002 [US1] Deliver the complete safe-close caller and consumed core in `mcp-server/src/script_close.rs`, `mcp-server/src/runner/close.rs`, `mcp-server/src/bridge/wire/close.rs`, `mcp-server/src/bin/close-gdscript.rs` and `godot-addon/addons/godot_agent_kit/script_close_transport.gd`, bundling contract/process/live-editor coverage in `mcp-server/tests/script_close_contract.rs`, `mcp-server/tests/script_close_caller.rs`, `godot-addon/tests/run_script_close.py` and `specs/005-close-project-gdscript/quickstart.md`.
  **Depends on:** Completed T001 and its merged PR unless stacking is explicitly authorized. Do not bypass a failed native/v5 prerequisite or recreate its owner/validator. This task implements and proves all US1.1–US1.5, US2.1–US2.6 and US3.1–US3.5, plus first-use US4.1/US4.3 and the complete US4.4/US4.5 safeguards. T003 owns new cumulative sequences/matrix, not deferred first-use safety.

  **Tests → consumed domain → acquisition → caller → integration:** Add meaningful model/precedence, malformed-input, boundary and process regressions first where executable. Implement checked `CloseRequest` and immutable `ClosingOutcome` in `mcp-server/src/script_close.rs`, with private `script_close/attempt.rs` and `script_close/outcome.rs` as responsibility warrants. Reuse `ExpectedRevisionBasis::from_observation` internally once; no cloned algorithm/public alias, edit-policy exposure or `EditAttempt` reuse. Keep full prior source out of retained intent and result summaries. Add only currently consumed exports/dispatch to `mcp-server/src/lib.rs`, `runner.rs`, `bridge/wire.rs` and the actual binary entry to `mcp-server/Cargo.toml`; preserve locked dependencies and no new crate.

  **Input and exact target:** Implement the caller contract's required `--registry`, `--project`, `--script`, optional exact `--session`, existing help/signal conventions, and exactly one ≤12 MiB UTF-8 stdin JSON object plus EOF with only `schema_version`, `request_id`, `basis`. Reject unknown/duplicate/nested-invalid/type/depth/encoding/trailing/oversize inputs without reflection or unbounded growth. Start the original clock before parsing, including blocked stdin. Reuse `mcp-server/src/target.rs` and `mcp-server/src/project_fs.rs` selected-project confinement; never use the basis to select an ambiguous/ended session or focus-based replacement. No source/hash/context acquisition before unique authentication.

  **Basis and recognition:** A non-null malformed/ineligible basis refuses rather than becoming null. Check selector/session/file binding before either branch. Null may recognize a freshly confirmed valid closed document without close/context validation/load/selection/history changes, but refuses an open document. A valid old open basis on the same still-closed file/session is explicitly not applied; a replacement file/session cannot exploit recognition. For open state require independent exact current D/R/B, attributed clean/Resource-edited=false and matching file/Script/editor/buffer/source/current version plus fresh native saved version and all protection guards. Preserve valid empty/read-only/safe syntax-error positives; no target parse gate, writable requirement, repair or forced human convergence.

  **Owned runner and complete private exchange:** Implement the existing same-executable killable-worker/supervisor pattern under `mcp-server/src/runner/close.rs` and private `runner/close/` files when needed. The parent owns the original clock, actual sequential `close_context` child validation/cleanup, verified receipt bindings and immutable terminal outcome; the worker only acquires/forwards attributable facts. Implement strict complete codecs in `mcp-server/src/bridge/wire/close.rs` and private submodules and the addon transport `godot-addon/addons/godot_agent_kit/script_close_transport.gd`, consumed through T001's real owner in `script_close.gd`/`bridge.gd`/`plugin.gd`. Supply all begin/prepare/recheck/advance/wait/verify/finish/abort stages, exact purposes, fields, binding, arity, bounds and stage rejection. Recompute every typed source-context/aggregate guard before trusting it. No arbitrary native dispatch, copied preparation as verification or worker-provided success verdict.

  **Single authorization and truthful outcomes:** Record possible effects before sending one `authorize_close` control. Native final guards remain mandatory; at most one advance, one wait and one pending asynchronous response, no lease extension. Collect separate fresh target observation, independently reacquired Rust D and native postconditions after real required continuation completion. Preserve causal failure, sticky invalidation and source-free known effects when a later frame is malformed, late, lost or scope-invalid. Before authorization, no effectful control establishes not-applied; afterward only actual irrevocable pre-entry discard can do so. Known application without required evidence is `applied_unverified`; uncertain possible application is `effects_unknown`. A timeout/cancel/EOF is never rollback. Never close a human-reopened buffer, restore source, Save or reopen as compensation.

  **Results and process contract:** Emit one bounded schema-1 `close_gdscript` JSON result plus newline with every required nullable key and the exact caller-contract reason/stage/application vocabularies. Summaries preserve real authority witnesses/intervals/availability; no target body or protected path/source/hash/receipt escapes. B/dirty become not-applicable only after confirmed absence; unloaded R differs from unreadable retained R. A close summary is not an edit/close basis. Implement exits 0 for the two positive outcomes, 3 for proven refusal, 4 for applied-unverified/unknown, 2 for malformed input and 1 for host/delivery failure; source-free stderr. Safe guidance after possible effects requires fresh ordinary observation, not retry/rollback. Prove ≤10-second externally measured terminal delivery with stdout drained (9.5-second work and 0.5-second reserve), including blocked stdin/worker/editor and signal cancellation.

  **Actual caller test groups and capability exposure:** Extend T001's runner/fixtures with complete `clean-close`, `already-closed`, `preservation`, `routing`, `interruption` and `privacy-export` groups plus inherited `native-boundary`. Now consume a real `--closer` for public groups and enable authenticated close capability only when the complete matched exchange is installed. Add first-use public discovery → open → observe → edit → fresh observe → close → reopen and dirty refusal controls within the implemented groups, reusing existing harness witnesses. Do not advertise unfinished `sequential`, full `composed` or `all`; an unavailable full run must explicitly refuse, never skip missing cases. No direct fixture close replaces the product in public cases.

  **Required semantic and real-boundary coverage:** In `mcp-server/tests/script_close_contract.rs`, `script_close_caller.rs`, `bridge_boundary.rs`, `confinement.rs` and private module tests, cover exact/one-over every bound, correct nullable/enum contracts, wrong target/session/file/document/version/purpose/receipt/stamp, malformed/deep/oversized/late frames, clock-before-input and immutable terminal precedence. Test actual files/permissions/namespace changes, independently dirty/stale/missing surfaces, retained/unloaded/replaced R, already-closed source limitations, and missing/null/ineligible/valid-but-non-applied bases. Exercise process-group child loss/reap/staging cleanup, blocked real IPC/stdin/editor, SIGINT/SIGTERM, known EOF, pre/post-entry cancellation/disable/expiry, malformed or lost delivery after known effects, reopened newer work, and observation/edit/open/discovery/close overlap in both directions. No forbidden force/source/retry flag or direct shell entry to unowned worker mode may bypass checks. Private barriers make races deterministic; release them after terminal delivery to prove no late close or outcome upgrade.

  **First-caller live acceptance:** All criteria in the US2/US3 phases below are T002 gates. Run all seven implemented groups through actual product/native boundaries, actual source/version/dirty/selection/history witnesses and inspected owned windows. Include stable two-document clean and dirty-R==B positives, R!=B refusal before entry, history/sort destinations, unavailable/misbound continuation and invalidated postconditions despite native OK. First-use composition must preserve persisted source and applicable A–E/Save/reparse/rescan/runtime behavior; then complete existing observation/edit/open/discovery suites because the shared authenticated owner/caller integration changed. Every public result receives result-only interpretation, then comparison to independent evidence and elapsed time. Apply authorized/denied/ambiguous/interrupted target/protected/other-project/credential sentinels; inspect and execute enabled, disabled and hook-only exports. Known safety/acceptance failures cannot be deferred to T003.

  **Actual build/campaign consumers and documentation:** Add the real close binary to `.github/workflows/ci.yml`'s locked build list. Extend `godot-addon/tests/run_editor_campaign.py` with `--closer`, the real close suite/input fingerprints and truthful incomplete-suite refusal, and `godot-addon/tests/test_editor_campaign.py` plus affected `.github/tests/test_live_editor_execution.py` for actual execution/resume/invalidation behavior. Until T003 supplies all groups, neither `--suite close` nor campaign `--suite all` may claim complete close acceptance; reject unsupported full coverage explicitly. Update `.github/README.md`, `godot-addon/native/README.md`, caller/bridge/native implementation-status notes and this feature's quickstart/plan with actual commands, counts, support limits and shape/constitutional review. Mark only T002 complete; Feature 005 remains incomplete pending cumulative acceptance.

  **Constraint inheritance:** Every T001 quoted native/protection/limit/interface constraint applies here. The following exact model blocks add current-consumer intent, lifecycle/outcome and public-field constraints. Wire-only exact required keys, nullability, stage/purpose sets, numeric encodings and reason/exit semantics in the caller/bridge/native contracts are mandatory as well; quotes do not replace the contracts.

  **Verbatim data-model constraints — §1 Reused values and ownership:**

  > Reuse existing `RequestId`, `ProjectRoot`, optional `SessionId`, `ResourcePath`, `ResolvedTarget`, `DocumentIdentity`, `Witness`, `CollectionStamp`, `ObservationInterval`, source/dirty availability and invalidation semantics from [observation.rs](../../mcp-server/src/observation.rs). Decimal IDs/counters remain canonical unsigned decimal strings at adapters; they are never lossy JSON numbers. Exact source comparisons precede retained SHA-256/UTF-8-length summaries.
  >
  > Reuse the existing checked `ExpectedRevisionBasis::from_observation` implementation in [script_edit/request.rs](../../mcp-server/src/script_edit/request.rs) internally. Its conditions are a complete open standalone observation, performed consistency checks, no detected change, independently equal D/R/B, attributed clean dirty state, actual file/Script/editor/buffer identities, no known stale R/B and a current buffer version. It supplies no prior saved version. Do not clone the algorithm, re-export an alias, expose edit application policy through close, or change existing public edit contracts merely to move a stable type. New close callers use `CloseRequest::new(..., Option<&ObservationOutcome>)`; conversion happens once and retained intent contains no source bodies. No `EditAttempt` is reused.
  >
  > New public core surface: checked `CloseRequest`, `ClosingOutcome` and the real close runner entry. Attempt state, native receipts, source-context records, codecs and protection witnesses remain private or crate-visible to actual consumers. Use the existing meaningful evidence errors; no public speculative variants or generic lifecycle framework.

  **Verbatim data-model constraints — §3 CloseRequest and prior basis:**

  > | Field | Meaning / validation |
  > |---|---|
  > | `request_id` | Fresh checked correlation, different from a supplied prior observation ID; no replay-cache meaning. |
  > | `project_root`, `session_id`, `script_path` | Existing explicit selectors; project is mandatory even if a prior result contains it. An omitted session still requires unique authenticated selection; the basis never chooses between candidates. |
  > | `expected` | Optional checked existing clean `ExpectedRevisionBasis`. Obtained by borrowing/validating the complete prior observation, then retaining only its checked identity/provenance/digest/version. Null grants no effectful authority. |
  >
  > The CLI payload uses `schema_version: 1`, `request_id`, `basis`. No replacement source, expected-source override, arbitrary source filename, force or validation receipt is accepted.
  >
  > A non-null malformed/ineligible basis produces an input/basis refusal; it is never silently treated as null. A well-formed basis must match requested and freshly resolved project/session/path and original file identity. Wrong-session/replacement-file evidence refuses even if a different file at that path happens to be closed. For the same valid file/session already confirmed closed, an old open-buffer revision is not applied or certified: return fresh `already_closed_unchanged` with expected evidence marked `not_applied_to_closed_state`. Null is the straightforward no-effect recognition input. If the document is open, null refuses `missing_basis` and any mismatched source/current version/Script/editor/buffer identity refuses before closure.
  >
  > A prior closed or limited observation cannot become an effectful basis. The caller may instead submit null to recognize a still-closed target; a race that makes it open then refuses. Source equality never authorizes a newer editor/buffer. The native boundary freshly acquires saved version and Resource-edited state; neither is invented in the prior observation schema.

  **Verbatim data-model constraints — §7 State transitions, deadline and terminal precedence:**

  > | Attempt state | Legal next work / invariant |
  > |---|---|
  > | `accepted` | Checked input, original clock; no target source/effects. |
  > | `selected` | Unique authenticated target/capability; preserve ended-session and denial precedence. |
  > | `inspected` | Fresh valid-file/open association. Confirmed closed may enter no-effect recognition; open requires matching expected basis. |
  > | `prepared` | Immutable clean target and bounded complete protection set; no lifecycle effect. |
  > | `validated` | All required `close_context` receipts match; fresh final guards pass. |
  > | `authorized` | Parent records possible effects before issuing one-shot authorization; missing reply can no longer imply no effect. |
  > | `closing` | Native exact-path call entered once; keep entered references/owner, track observed events and effects. |
  > | `settling` | Native call returned; observe required automatic validation completions and preservation, without issuing another mutation. |
  > | `verifying` | Fresh ordinary target observation plus independent D and native protection/identity recheck. No copied preparation sample. |
  > | `terminal` | Immutable result, no new authority or late upgrade. Cleanup only owned handles/signals/processes. |
  >
  > The recognition path is `inspected -> verifying -> terminal`; it performs no source-context validation, close or continuation wait. Every failure can terminalize with actual application knowledge. Pre-authorization failures are proven not-applied only when no effectful control was issued. After authorization, an attributable irrevocable pre-entry discard can establish no effect; absence of acknowledgment cannot. Known selection/removal effects survive later protocol/timeout errors. Terminal cleanup never changes source, selection or history.
  >
  > Shared ownership remains one addon slot and one native `Session` owner variant (edit/open/close). Busy requests are irrevocably non-applied and never queued. Cancellation/disable/loss prevents new entry/stages; an entered native call or callback retains ownership/references until it returns. At terminal timeout after return, detach the observation ledger and release owned state without cancelling Godot's ordinary validation or pretending it rolled back. No stale product close can execute afterward; pending ordinary native work is explicit unverified evidence.
  >
  > | Outcome | Application / condition |
  > |---|---|
  > | `verified_newly_closed` | `applied`; original open document removed, fresh current target absent, admitted D/file identity unchanged, retained R independently unchanged or observed unloaded, required native continuation complete, protection and all rechecks intact. |
  > | `already_closed_unchanged` | `not_applied`; valid exact current file/session is freshly confirmed closed, no effect authorization, no source/selection/history change. Source-local limitations remain explicit. |
  > | `refused` | `not_applied`; correct safe refusal or interruption with proven no effects/no late product entry. |
  > | `applied_unverified` | `applied` or `partly_applied`; at least one attributable lifecycle effect occurred but a required condition is missing/invalidated. |
  > | `effects_unknown` | `unknown`; effects were possible without sufficient attributable application or terminal-discard evidence. |
  >
  > First causal failure and sticky invalidation are retained. Denied/ambiguous/invalid target binding suppresses unauthorized source-derived fields, but never erases independently known source-free effect knowledge. Complete closure evidence cannot override a later required-evidence failure. A newly reopened target makes final closure unverified even if the old buffer was successfully removed; never close the new one.

  **Verbatim data-model constraints — §8 Public ClosingOutcome:**

  > All fields below are required; nullable records use explicit null. The caller contract defines exit behavior and examples.
  >
  > | Field | Shape / interpretation |
  > |---|---|
  > | `schema_version`, `operation`, `request_id` | `1`, `close_gdscript`, checked correlation. |
  > | `requested_target`, `resolved_target` | Existing checked selectors / resolved identity, or null with the appropriate refusal. |
  > | `interval`, `outcome`, `reason`, `stage`, `application` | Existing caller-clock interval, the closed outcome vocabulary above, a bounded reason, furthest entered stage and independent effect knowledge. |
  > | `expected` | Null or prior target/identity/source digest/current-version/provenance summary with `use: matched\|mismatched\|not_applied_to_closed_state\|unavailable`; never full prior source or a permission token. |
  > | `progress` | `closing`, `native_revalidation`, `verification`, each `{state, reason, collection}` with state `not_started\|not_applicable\|entered\|completed\|failed\|unknown`. |
  > | `before` | Nullable attributable target summary: open state/document IDs, independent source summaries, dirty state, Resource-edited/current/saved-version facts and validity. |
  > | `observation` | Null or `{purpose: preparation\|recognition\|verification\|survivor, interval, snapshot}`; the snapshot uses existing observation semantics but replaces observed source text with `{sha256, utf8_bytes}`. It retains actual per-authority witnesses/collections, comparisons and invalidation. It is not a reusable observation-v1 edit basis. |
  > | `resource_state` | `{state: retained\|unloaded\|unavailable\|invalidated\|not_collected, resource_edited, reason, collection}`; the actual latest post-close cache/Resource observation, never inferred from missing B. |
  > | `protection` | `{status: not_applicable\|preserved\|unavailable\|invalidated, revalidation: not_applicable\|pending\|completed\|unavailable\|invalidated, required_count, completed_count, reason}`. Counts are nullable when not established; no protected paths, source, hashes or receipt bodies. |
  > | `selection` | Nullable `{before, after, request_effect}`; relations `target\|other\|no_source_editor\|unknown`, effect `none\|native_fallback\|unknown`. Private exact IDs establish preservation; public output does not name another document. |
  > | `history` | `{participation: not_participated, target_buffer: retained\|disposed\|unavailable\|not_applicable, unrelated: observed\|unavailable\|invalidated\|not_applicable, reason}`. Disposal means actual native target-buffer lifetime, not enumeration of an opaque history stack. |
  > | `diagnostics`, `safe_next_action` | Bounded source-free records and fixed guidance. Possibly applied results require fresh observation of the explicit original target, not retry/reopen/rollback. |
  >
  > B and buffer-dirty state are `not_applicable` only for independently confirmed closed state. A present but unreadable B stays unavailable. A genuinely unloaded R keeps observation v1's unavailable/unloaded meaning; `resource_state` explains it without loading. An earlier sample remains earlier/invalidated evidence after possible effects, not fresh post-state. The closing target's parse success is not a required field or success condition; available errors remain source-attributed diagnostics rather than a fabricated clean parse.

## Phase 4: User Story 2 — Preserve Human Work and Refuse Stale Closing Intent (Priority: P1)

**Goal:** Preserve target and unrelated human work/history and refuse unsafe or stale intent before native effects wherever observable.

**Implementation/tests owner:** T001 proves native guards and their actual source/continuation effects; **T002 implements and proves all US2.1–US2.6 end-to-end** before its completion. No separate checkbox or later authorization/conflict/history PR: US1 cannot ship safely without this phase.

**Independent test criteria:** Prepare dirty-different and equal-text-dirty targets with distinctive text/history; separately diverge D/R/B, stale R/B and lose required attribution. Exercise null/unusable/stale basis, same-text human close/reopen and version changes, wrong/replaced project/session/file identities, missing D with surviving B, and typing/Save/rename/parent/leaf/roster/settings changes between preparation and entry. Assert correct first causal refusal, no new close/Save/discard/source/selection/history effect, and intact newer buffers. Demonstrate unrelated selected/background dirty R==B positives with real prior Undo/Redo; reproduce the stock R!=B negative only as a primitive control and prove the product refuses before entry without applying its source. Use ambiguous/ended/replaced/denied/outside/missing/embedded/non-GDScript/custom/unsafe/oversize contexts and private source sentinels; unknown open state is never already closed. After actual entry, invalidation retains known effects and newer work under US3 rather than falsely claiming no application.

## Phase 5: User Story 3 — Know Whether an Interrupted Close Took Effect (Priority: P1)

**Goal:** Distinguish proven no effect, known applied/partial-unverified and unknown effects within the original deadline, without later stale closure or automatic retry.

**Implementation/tests owner:** T001 owns entered-native/callback lifetime and irreversible discard facts; **T002 owns all US3.1–US3.5 supervisor/caller behavior and acceptance**. T003 adds cumulative interleavings, not a missing timeout or cancellation safeguard. No additional checkbox is justified.

**Independent test criteria:** Interrupt the real caller before authorization, while native close is entered, after actual removal, during pending native validation and during independent verification. Include stalled stdin/worker/editor, SIGINT/SIGTERM, EOF/connection loss, disable/expiry, lost helper and malformed/delayed/out-of-order replies. Independently witness actual source/file/editor/buffer identity and selection/history effects; require correct stage versus completion, uncertainty and safe fresh-observation guidance within ten seconds with stdout drained. Human reopen/edit before verification must survive, with old closure unverified and no reclose/reopen/Save/source restoration. Exercise all same-slot ownership directions with observation/edit/open/discovery/close, retain entered owners/callback data until return, then release barriers and prove no refused attempt executes or terminal outcome upgrades later. Native OK, a returned method, an empty getter or an elapsed timer cannot supply missing verification.

## Phase 6: User Story 4 — Complete and Repeat the Safe Script Workflow (Priority: P1)

**Goal:** Prove cumulative VALID repeated-close/composed A–E/history/durability, unchanged existing-operation, privacy/export and exact supported-environment evidence, not reexecute historical suites at the literal final commit.

**Ownership:** T002 already supplies first-use US4.1/US4.3 and complete US4.4/US4.5 safeguards. T003 newly executes US4.2's stateful request sequence and the full US4.1/US4.3 composed matrix, and reviews cumulative evidence validity for all stories. It may fix newly discovered in-scope regressions but cannot inherit known incomplete T001/T002 acceptance.

**Independent test criteria:** Run actual discovery → open → observe → edit → fresh observe → close → existing reopen, without manual path lookup/tab closing/reconciliation. Interleave at least 20 real closing requests with deliberate human edits/reopenings, including at least five actual clean closes, five already-closed recognitions and five dirty refusals including equal-text-dirty. Independently prove unchanged admitted source, preserved newer/unrelated work and history, native continuation truth, and applicable A–E/Save/reparse/rescan/runtime persistence. Establish VALID coverage of current callers' unchanged contracts/deadlines and all three actual export modes by new execution or relevant-input-reviewed accepted evidence.

- [ ] T003 [US4] Deliver cumulative repeated-close and composed-workflow acceptance in `godot-addon/tests/run_script_close.py`, `godot-addon/tests/fixtures/script_close/fixture_driver.gd`, `godot-addon/tests/run_editor_campaign.py` and `godot-addon/tests/test_editor_campaign.py`, reusing existing observation/edit/open/discovery harnesses and recording final evidence in `specs/005-close-project-gdscript/quickstart.md`, `specs/005-close-project-gdscript/tasks.md` and `PROJECT_STATUS.md`.
  **Depends on:** Completed T002 and its merged PR unless stacking is explicitly authorized. All first-caller branches and safety groups must already pass. No new product operation, schema, permission, dependency or private version is introduced.

  **Cumulative harness capability:** Implement actual `sequential` and full `composed` groups, then complete `--scenario all` so all nine quickstart groups run exactly once, serially. Reuse existing independent history/durability/window/export helpers rather than copying suites or adding a framework. Finish `--suite close` and campaign `--suite all` inclusion with actual `--closer`/native/addon/fixture/contract execution fingerprints and correct resume invalidation in `godot-addon/tests/run_editor_campaign.py` and `test_editor_campaign.py`. No silent missing-group filtering, duplicate nested-scenario counts or reused failed/corrupt/changed-input evidence. Update `.github/README.md` and affected workflow execution tests for real current commands only.

  **Twenty-request stateful sequence:** Implement at least 20 public close requests with ≥5 actual clean closures, ≥5 no-effect already-closed recognitions and ≥5 dirty refusals including equal-text-dirty, with stale/unsafe requests and deliberate between-request human typing, Save and same-text/different-text reopenings. Every effectful request obtains fresh ordinary observation/basis; no earlier close summary becomes authority. Independently witness source, D identity, actual retained/unloaded R, old/new editor/buffer identity, current/saved versions, dirty/Resource flags, selection and real prior history; distinguish intended human/native lifecycle actions from prohibited source changes. Releasing late barriers must never close a newer document. Repeated mock rows or primitive-only calls do not meet the minima.

  **Full composed matrix:** Begin without a script locator; product discovery supplies it. Use product opening, observation and editing, obtain a new observation after the edit, close through the real caller, and reopen only as a separate existing opener action. Prove A clean D==R==B; B dirty-different/equal-text refusal with work intact; C actual apply → Undo → ordinary Save → Redo → Save while the buffer exists and unrelated history reachability through closing; D intended revision survives product close/existing reopen; E existing twenty fresh-basis edit stress plus the separate close-request minima. Reparse/rescan and a permitted owned runtime must preserve/execute the intended revision. Neither force-load/unload nor reopen may substitute for observing post-close R/B. Target-local history disposal is not persistent-history or undoable-close support.

  **Cumulative VALID evidence and execution decision:** Newly run `sequential` and full `composed`, directly affected regressions, and static/build/campaign checks applicable to actual changed surfaces. Reuse the seven accepted prerequisite close groups and accepted observation/edit/open/discovery evidence only after reviewing the exact diff since accepted runs: relevant production paths, addon/native/Rust binary behavior and provenance, protocol/ABI, fixture/witness semantics, supported environment and acceptance requirements must remain unchanged or demonstrably equivalent for rebuilt/relinked artifacts; version claims alone are insufficient. Changed relevant inputs invalidate affected evidence only. A production fix requires directly affected close groups/regressions; broader execution requires the exact changed shared behavior/boundary, exact suite(s) and concrete failure mode. Generic shared integration, final-head/cumulative confidence or an `all` interface is not justification. Fixture/sequential/composed code, campaign registration/fingerprint/resume tests or docs-only changes require no unchanged historical observation/edit/open/discovery reruns. Implement and orchestration-test all interfaces; full multi-hour `--scenario all`/campaign `--suite all` execution is convenience or independently justified release work, not automatic T003 acceptance. Record decisions in quickstart §6's evidence review; failed/incomplete evidence is never reused. Preserve checkpoint identity/invalidation rules: independent semantic evidence reuse does not make invalid checkpoints passed. New executions use fresh private directories, exact normal/fault native and executable/build/source provenance, drained stdout, measured deadlines and actual owned-window review. Fix in-scope regressions with meaningful tests and rerun affected cases; review remaining cumulative evidence validity on the corrected inputs. All nine groups and every requirement still need VALID coverage; planning controls cannot complete the feature.

  **Result-only/privacy/export acceptance:** For every public result, identify requested/resolved target or refusal; new/already-satisfied/refused/applied-unverified/unknown outcome; expected-basis use; reached versus completed stages; actual or unavailable D/R/B/dirty/Resource evidence; selection/history/preservation/native-continuation facts; evidence interval/validity and safe next action using the result alone, then compare independent witnesses. Require zero target/protected/other-project/credential sentinel disclosure outside authorized summaries and no raw source/receipt/helper data in incidental logs/registry/campaign manifests. Verify actual child/process-group/private staging/signal/slot cleanup. Inspect and execute enabled-addon, disabled-addon and export-hook-only games for absent active tooling, fixture artifacts, native registration, listeners and gameplay dependencies. No new provider/approval prerequisite.

  These result/privacy/export requirements define evidence coverage, not unconditional reruns. Review every newly executed result; retain accepted result reviews and actual export/cleanup evidence when relevant inputs remain valid under the execution decision above. New or changed fixture/export/witness semantics require their affected evidence to run.

  **Completion and delivery:** All 21 scenarios, FR-001–FR-018, SC-001–SC-007, nine edge cases and applicable A–E/history/durability/export gates must pass. Record exact supported environment, measured limits, justified inapplicabilities and constitutional/implementation-shape review in `specs/005-close-project-gdscript/plan.md`, `quickstart.md`, current contracts/operator docs and `PROJECT_STATUS.md`. Mark only T003 `[X]` and Feature 005 complete when acceptance holds, independently of PR state. Roadmap Phase 1 remains in progress: separate Save/history controls and phase exit assessment are not delivered here. Open the dedicated T003 PR and STOP; do not merge or select another task/feature.

  **Constraint inheritance:** Every verbatim field/state/limit/validation/retention constraint attached to T001/T002, and all three contracts, applies to this task's sequences/results and any regression fix. Cumulative proof does not introduce a second policy or a weaker profile.

## Phase 7: Polish & Cross-Cutting Concerns — Bundled With Their Owners

Each task includes its own current docs/migration notices, owned temporary-scaffold cleanup, redaction/export verification, regression fixes and implementation-shape/constitutional review. No independent formatting, generic hardening, docs-only, refactor or cleanup checkbox is justified. T003 adds a meaningful stateful/composed acceptance capability rather than postponed safety. No wider platform, lifecycle tool, MCP, gameplay or distribution scope is authorized.

## Dependencies & Execution Order

### Task graph

```text
Completed Features 001–004 + approved artifacts + granularity review + analysis
                                     |
                                     v
T001: guarded native boundary + real validator consumer + v5/revision-3 cutover
                                     |
                                     v
T002: complete public caller/core + all first-use US1/US2/US3/US4 safeguards
                                     |
                                     v
T003: repeated/composed transitions + complete cumulative VALID acceptance
```

Every edge requires prerequisite acceptance and normal PR-merge sequencing unless stacking is explicitly authorized. No task is marked `[P]`: each has a real incomplete prerequisite and shared integration paths. Setup/polish add no prerequisite PRs; within-task parallel authoring does not permit parallel task delivery or simultaneous GUI campaigns.

### Story completion order and counts

| Story | Primary checklist count | Owning tasks and completion |
|---|---:|---|
| US1 — intended clean closure (P1) | 1: T002 | T001 native boundary; US1.1–US1.5 caller acceptance in T002; cumulative regression in T003. |
| US2 — human work/stale-intent preservation (P1) | 0 additional | T001 guards; all US2.1–US2.6 implemented/proved in T002 before first caller completion. |
| US3 — truthful interrupted effects (P1) | 0 additional | T001 lifetime/facts; all US3.1–US3.5 implemented/proved in T002; T003 repeats interleavings. |
| US4 — repeated/composed workflow (P1) | 1: T003 | T002 first-use US4.1/US4.3 and US4.4/US4.5; T003 full US4.1–US4.5 cumulative gate. |
| Shared foundation | 1: T001 | Actual native/validator fixture consumer, compatible migration, tests/docs and affected old-suite evidence. |

**Total: 3 unique tasks.** US1–US3 complete together at T002 acceptance; US4/feature completion follows T003. Story criteria are independently testable but unsafe partial capability delivery is prohibited.

## Parallel Execution Examples by Story

Examples below are optional disjoint **authoring slices within one selected task**, not extra Spec Kit tasks or `[P]` permission. Fix the existing interfaces and file ownership first. One integration owner gathers all results before builds/tests; actual GUI acceptance runs serially. Examples sharing a file across rows are alternatives, not simultaneous work assignments.

| Story / selected task | Disjoint authoring example | Integration condition |
|---|---|---|
| US1 / T002 | Core/transition work in `mcp-server/src/script_close.rs` and `script_close/` alongside fixed fixture preparation/witnesses in `godot-addon/tests/fixtures/script_close/fixture_driver.gd`. | Caller/native contracts fixed first; use real integrated caller and independent witnesses before acceptance. |
| US2 / T002 | Actual filesystem/confinement regressions in `mcp-server/tests/confinement.rs` alongside dirty/identity/history fixture controls in `godot-addon/tests/fixtures/script_close/fixture_driver.gd`. | Same immutable identity/no-force semantics; no shared-file edits or mock substitute for live races. |
| US3 / T002 | Core precedence cases in `mcp-server/tests/script_close_contract.rs` alongside real stdin/process/cancel/timeout cases in `mcp-server/tests/script_close_caller.rs`. | Fixed public outcome/runner interfaces; integrate first, then exercise actual stalls/loss and original deadline. |
| US4 / T003 | Stateful sequence/composition in `godot-addon/tests/run_script_close.py` alongside campaign/fingerprint/resume work in `godot-addon/tests/run_editor_campaign.py` and `test_editor_campaign.py`. | Agree nine-group coverage contract; integrate before serial new sequential/full composed execution, affected regressions and relevant-input evidence review. |

T001 can similarly split close-native/context source work from Rust v5 transcript/validator work and disjoint private fixture controls after fixing the normative native/bridge contracts. Every required current consumer migrates in the same T001 PR; no independently shipped incompatible peer pair or speculative API.

## Requirement and Acceptance Coverage

| Required acceptance | Owning task(s) | Proof |
|---|---|---|
| US1.1–US1.2, US1.4; FR-001/FR-003/FR-005/FR-006/FR-008/FR-014; SC-001 | T001 native; T002 caller | Exact clean target, selected/non-selected/last-tab, two-document and empty/read-only/safe-invalid positives; real continuation and fresh unchanged-source/closed-state witnesses. |
| US1.3, US2.3; FR-007/FR-009/FR-010; SC-002 | T001 passive facts; T002 | Null/valid-old/non-applied basis, cached/unloaded/source-limited recognition; zero effects; replacement file/session and unknown/missing target refusal. |
| US1.5; FR-006/FR-015; SC-001/SC-005 | T002; T003 cumulative | Separate ordinary observation and existing reopening restore persisted coherent sources without treating close summaries as authority. |
| US2.1–US2.2, US2.4–US2.5; FR-003–FR-005/FR-008/FR-012/FR-014; SC-002/SC-006 | T001 boundary; T002 end-to-end | Dirty/equal-dirty/stale/divergent/missing evidence; exact/new identities and boundary races; protected R==B real history; R!=B refusal; full bounded effect context. |
| US2.6; FR-002/FR-014/FR-016; SC-002/SC-007 | T001 authentication; T002 | Real intended/ambiguous/ended/replaced/denied/outside/invalid/unsafe routing, no candidate source or destructive prompt, no file-disappearance permission. |
| US3.1–US3.4; FR-009–FR-013; SC-003 | T001 lifetime/facts; T002 | Actual before/after-entry/continuation/verification interruptions, original ≤10-second terminal bound, new-buffer preservation, truthful known/unknown effects and fresh-observation guidance. |
| US3.5; FR-004/FR-011/FR-013/FR-015; SC-003/SC-006 | T001 owner; T002; T003 interleavings | All shared-slot operation pairs in applicable directions, entered-owner retention, irrevocable refusal and no late replay/terminal upgrade. |
| US4.1, US4.3; FR-015/FR-017/FR-018; SC-005/SC-006 | T002 first use; T003 full matrix | Actual discovery/open/observe/edit/fresh-observe/close/reopen; A–E, real Undo/Redo and ordinary Save/reparse/rescan/runtime durability. |
| US4.2; FR-004/FR-007/FR-008/FR-011–FR-014/FR-017; SC-004 | T003 | ≥20 real requests, ≥5 real closes/≥5 no-change/≥5 dirty refusals, deliberate edits/reopenings, zero lost source/history or stale closure. |
| US4.4; FR-015/FR-018; SC-006 | T001/T002 affected suites; T003 cumulative | Accepted full observation/edit/open/discovery evidence on their changed shared boundaries, reviewed for relevant-input validity; rerun affected evidence when invalidated. Preserve public schemas/limits/deadlines and no implicit close. |
| US4.5; FR-002/FR-009/FR-010/FR-016–FR-018; SC-007 | Every introducing task; T003 cumulative | Result-only review, private source/credential sentinels, owned cleanup, all three inspected/executed export modes and exact tested support. |
| FR-017/FR-018; all SCs | T001/T002 task gates; T003 feature gate | Meaningful deterministic and actual process/bridge/filesystem/live-editor evidence; useful positives and complete cumulative VALID coverage, including newly executed sequential/full composed. |

All **nine edge cases** have explicit T001/T002 native/caller owners and T003 cumulative coverage where applicable: same-text reopen; ordinary target-local versus unrelated native history; retained/unloaded/unobservable R; deleted/renamed/replaced/redirected D with surviving B; empty/whitespace/line-ending/representation limits; selected/non-selected/last-tab selection; destructive prompts/unsupported effect contexts; permitted but actually observed native revalidation; and human/external post-close invalidation. The matrix assigns owners without narrowing normative requirements or gates.

### Entity and Contract Ownership

| Artifact / entity | First implementation owner and consumer |
|---|---|
| `TargetPreparation`, `ProtectedDocument`, `CloseContextValidation`, `NativeCloseFacts`, `CloseContinuation` | T001 native/addon/validator with actual owned fixture; T002 consumes the same facts/receipts, not a duplicate safety model. |
| `CloseRequest`, checked prior basis, core attempt state, `ClosingOutcome` | T002 actual CLI/library runner; existing checked basis reused without a new public alias. |
| Native contract and shared v5 compatibility | T001 complete private native family, shared current-peer migration and real fixture acceptance; product close capability remains false. |
| Complete close v5 product tuples/worker controls and caller v1 | T002 codecs, supervisor and real addon exchange; only then advertise close. |
| Nine-group runner and complete campaign | T001 native group; T002 six caller/safety groups and truthful full-suite unavailability; T003 actual sequential/composed/all integration. |

## Implementation Strategy

### First Useful Slice — Safe MVP

The first useful public close is **T001 + T002**, two coherent PRs from the current baseline. MVP means US1 **with all US2/US3 and first-use US4 safety**, never an unguarded close or last-tab-only shortcut. T001 independently establishes the exercised private native/validation/compatibility boundary; it is not a chain of method/schema stubs. Full feature/release completion requires T003.

### Incremental Delivery

1. T001 delivers the native guards/continuation and exact-source validation with all current peers on v5/revision 3; no public closing claim.
2. T002 delivers the actual caller, now-consumed domain/supervisor/transport and every safe/refused/unknown branch, with first-use and affected existing-suite evidence.
3. T003 adds cumulative repeated/composed transitions and complete VALID privacy/export/compatibility evidence through new execution and relevant-input-reviewed reuse; it cannot absorb known missing safeguards.
4. Stop after each dedicated task PR. Acceptance/shape/evidence determine completion; repository PR sequencing determines permission to begin the next task. Do not auto-merge or auto-select another task.

## Granularity Review

**Review state: PASS (2026-10-01).** Parent review and independent coverage/contract and granularity/dependency reviews found no actionable omission, scope, dependency or executability defect. This completes the required proportionate granularity review before `/speckit.analyze`; it is not that command, implementation approval or a new review/CI gate.

| Criterion | Reviewed decomposition |
|---|---|
| Meaningful PR-sized increments | Independently exercised native/validation/compatibility boundary; complete caller with all inseparable safety; stateful/composed cumulative acceptance. |
| No trivial fragments | No directory, method, enum, capability bit, codec, helper, test-only precursor or docs-only task. Core models ship with their actual caller. |
| Tests/docs bundled | Every task includes meaningful boundary/behavior tests, actual smoke, current docs and proportionate shape/constitutional review. |
| Serial path to useful capability | One real private boundary before the public caller: two PRs to a safe MVP, three total. Setup/polish add no prerequisite PRs. |
| Coverage preserved | Four P1 stories, 21 scenarios, 18 FRs, seven SCs and nine edge cases have owners; all nine model sections quote their field/enum/nullability/bound/transition constraints verbatim. |
| Clear dependencies | T001 → T002 → T003; no misleading `[P]` or duplicate story checkboxes. US2/US3 are first-caller gates, not deferred implementation. |
| Actual current consumers | T001 has a real shared-slot fixture and stock-validator consumer; T002 supplies the complete authenticated product exchange before advertising close. No speculative public API. |
| Honest cumulative boundary | T001 native group and T002 safety groups passed their own task-owned gates; T003 newly executes repeated/full composed transitions and establishes complete cumulative VALID evidence, not literal-final-commit historical reruns or known missing safeguards. |
| Principle XIII and constitutional compliance | Reuse the current owner, validator, framing, confinement and harness; bounded protection/continuation addresses observed effects. No source repair, new execution authority, generic framework, provider prerequisite or weakened evidence/human-work guarantee. |

**Artifact validation (2026-10-01):** All three unique sequential pending task rows have the required checkbox, ID, phase-appropriate story label and exact file paths; none is incorrectly marked `[P]`. Seven phases retain all four P1 stories. All nine complete data-model constraint quotations match their source verbatim. Coverage review accounts for 21 scenarios, 18 functional requirements, seven success criteria and nine edge cases.

Task links/anchors, Markdown tables/fences, template-marker removal, whitespace and the shell example's syntax passed; updated plan/status links also passed. The installed prerequisite helper discovered `tasks.md` and every design artifact under the verified Feature 005 override. No extension hooks were configured. No product code, Cargo/native/GUI acceptance or `/speckit.analyze` was executed by this task-generation workflow.
