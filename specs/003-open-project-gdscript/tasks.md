---
description: "PR-sized implementation tasks for guarded known-path GDScript opening"
---

# Tasks: Safely Open a Known Project GDScript

**Input:** [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md), [data-model.md](data-model.md), [caller contract](contracts/open-api.md), [bridge contract](contracts/bridge-protocol.md), [native contract](contracts/native-integration.md), and [quickstart.md](quickstart.md).

**State:** **1 / 3 implementation tasks complete. T001 is complete** under the maintainer's `orchestrate T001 only` instruction; its [native/cutover acceptance and implementation-shape review](quickstart.md#8-t001-native-boundary-and-cutover-acceptance-2026-09-30) passed. The required post-granularity consistency analysis passed on 2026-09-30 with all 18 FRs, seven SCs, 21 scenarios and eight edge cases mapped. T002 and T003 remain pending; no stacked workflow is authorized. Feature 003 and roadmap Phase 1 are not complete.

**Organization:** Three coherent PR-sized increments: a guarded native/current-validation capability with its coordinated compatibility cutover; a complete public opening caller and its consumed core; and cumulative repeated-use/composed-workflow acceptance. All four stories remain P1. US2, US3 and US4.1–US4.3 are inseparable safety requirements of the first caller and are implemented and proved inside T002, not postponed as separate safety/test-only PRs. Each story retains its own phase and independent acceptance below. Shared references are not extra task checkboxes.

## Working Agreement and Format

- Each `- [ ] Tnnn [P?] [USn?] Description with exact file paths` row is one implementation task and one PR. Indented scope, quoted constraints, tests and acceptance are part of that task's executable description. Common rules in this section apply to all three tasks. Internal work steps, commands, tests, files and subagents are not additional Spec Kit tasks.
- Follow [AGENTS.md](../../AGENTS.md), the [constitution](../../.specify/memory/constitution.md), approved spec/plan and this task list. Finish granularity review before `/speckit.analyze`; resolve actual blockers under the repository's four-part evidence rule. Do not silently redesign the plan, weaken a safety condition, broaden the threat model or use this task-generation command as implementation authorization.
- Select exactly one dependency-ready ID. Preserve existing work; fetch updated `main`, branch `task/<task-id>-<description>` from the authorized base, and verify `SPECIFY_FEATURE_DIRECTORY` points to `specs/003-open-project-gdscript` before helper use. Branch names alone do not choose the feature. Inspect complete intended/staged changes, stage explicit paths, commit, push and open that task's dedicated PR with requirement/evidence/constitutional context; report it and STOP. No automatic merge or next task. Stacking requires explicit authorization and still uses one task/PR. After confirmed merge, safely remove its remote/local source branches under repository policy.
- Mark only the selected task `[X]` when its acceptance, required evidence and implementation-shape review pass on the delivery head, independently of PR review/merge. Update this file and `PROJECT_STATUS.md` for material lifecycle transitions. Dependent implementation still waits for prerequisite PR merge unless stacking is explicitly authorized. Completion is not a PR state.
- **Tests are required by FR-018 and constitutional I–VI/XII.** Write meaningful failure/transition/contract regressions before the behavior they protect, observe failing-before where executable, and deliver passing tests and actual smoke evidence in the same task. Do not create a separate tests-first PR. Do not test source text, copied fields, mock echoes or incidental wording. A refused-everywhere implementation, mocks, headless runtime or copied mutation receipt cannot establish positive editor acceptance.
- No new crate, parser/framework, daemon, callback registry/sandbox, approval class, replay store, queue/lease, MCP, runtime product command, CI provider or platform is introduced. Reuse the locked dependencies, native public ABI, existing source-only validator, project confinement, authenticating bridge, collector and single operation owner. The inherited malicious-already-running-plugin/same-UID and local-validator-endpoint limits do not waive in-scope human-work or effect guards.
- Keep protocol/DTOs at the adapter, semantic reduction in Rust and Godot APIs at native/addon boundaries. Extract only helpers with current consumers. Use the narrowest visibility; do not expose reducer/context/receipt APIs to avoid dead-code warnings. Do not implement an unused opening core ahead of its caller. No compatibility aliases or dual private versions survive the cutover.
- Every materially changed implementation receives the repository's cohesion/ownership/visibility/state review before completion. Review large modules, multiple independent responsibilities, lifecycle booleans, speculative declarations and recoverable `unwrap`/`expect` paths; perform proportionate current-task cleanup without generic frameworks or unrelated refactoring. Tests/lint alone do not satisfy this gate.
- Each task owns directly affected API/operator docs and actual evidence in `specs/003-open-project-gdscript/quickstart.md`; record new dependency/support findings in `specs/003-open-project-gdscript/research.md` when applicable. Update its contract status truthfully. Historical Feature 001/002 evidence remains historical; only current reproduction/migration instructions change. No generated binaries, manifests, editor state, credentials or research source are committed.

### Path Conventions and Shared Validation

All implementation paths below are repository-relative. Names identified as new are created by their owning task; existing directories/packages are not setup deliverables. A private responsibility-based split beneath the planned module is allowed when cohesion warrants it, with current consumers migrated and documentation updated; the path lists do not prescribe one type per file or permit a second safety architecture.

Use existing Rust 1.98.1/edition 2021, locked Cargo dependencies, C++17/public generated GDExtension ABI and Python 3.10+. The candidate is official Godot `4.7.2.stable.official.ed1daf0bf`, full commit `ed1daf0bf001b61586d9930840f2f1394092c079`, executable SHA-256 `c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf`, macOS 26.6.2 arm64. Exact native/library/ABI/toolchain provenance and the [quickstart prerequisites](quickstart.md#1-exact-candidate-and-prerequisites) apply; no other version/platform is claimed.

For affected Rust work, from `mcp-server/`:

```sh
cargo +1.98.1 fmt --all -- --check
cargo +1.98.1 clippy --all-targets --locked -- -D warnings
cargo +1.98.1 test --locked
cargo +1.98.1 doc --no-deps --locked
```

Default tests include doctests. Build actual consumers with locked resolution. Run applicable existing native-build/workflow checks when those surfaces change. Exercise real child/connection paths for supervision changes and actual owned visible Godot windows for native/caller changes; use independent D/R/B/dirty/Resource-edited/history witnesses, bounded event-based waits and cleanup. GUI campaigns run serially after integrated code is ready. The planning probes and locked desktop did not pass these implementation gates; that historical environment limitation does not establish today's availability or justify a new CI prerequisite.

Use the commands/scenarios in [quickstart.md](quickstart.md#3-owned-opening-runner-and-future-caller-groups). T001 implements `native-boundary`; T002 implements the first public-caller groups; T003 adds the cumulative groups and complete `all`. Do not expose a no-op scenario, silently filter missing groups or label a partial runner `all`.

## Phase 1: Setup — Reuse the Completed Foundation

**Purpose:** Reuse completed Features 001/002: Cargo package, native build, addon, source-only validator, authenticated local routing, collector, fixture harness and shared workflows. No initialization, directory, dependency, formatter or standalone documentation checkbox is justified. Required setup travels with its first real consumer.

**Entry criteria:** Approved feature artifacts and the required subsequent analysis; the selected task's actual permitted smoke/validation environment. Generating tasks does not select a task, execute analysis or start implementation.

## Phase 2: Foundational — Guarded Native Opening and Compatible Integration

**Goal:** Establish the independently verifiable native opening boundary and real current-source validation before adding the public caller. The existing observation/edit product stays functional across the private-version/artifact migration.

- [X] T001 Deliver guarded native opening and current-source validation with the coordinated bridge-v3/native-revision-2 cutover in `godot-addon/native/script_open.cpp`, `godot-addon/addons/godot_agent_kit/script_open.gd`, `mcp-server/src/runner/stock_validation.rs`, `mcp-server/src/bridge.rs`, and bundled real-boundary coverage in `godot-addon/tests/run_script_open.py` plus `specs/003-open-project-gdscript/quickstart.md`.
  **Depends on:** No other Feature 003 task; completed Features 001/002 and normal reviewed-artifact/analysis entry criteria. This is one coherent private capability/cutover PR, not separate build, enum, helper, rename or test PRs.
  **Deliverable:** Real native `open_inspect`, `open_prepare`, `open_advance`, `open_verify`, `open_recheck`, `open_finish`/`open_abort` and a thin addon owner, exercised through the real shared operation slot by an owned private fixture. Real `open_context` validation has a current fixture consumer. No public `open-gdscript` command or unused Rust opening reducer is introduced in this task.
  **Native implementation and ownership:** Create `godot-addon/native/script_open.cpp` and `godot-addon/addons/godot_agent_kit/script_open.gd`. Evolve `godot-addon/native/native.hpp`, `godot-addon/native/session.cpp`, `godot-addon/native/extension.cpp`, `godot-addon/native/script_document.cpp`, `godot-addon/addons/godot_agent_kit/script_edit.gd`, `godot-addon/addons/godot_agent_kit/bridge.gd` and `godot-addon/addons/godot_agent_kit/plugin.gd`. Reuse/generalize the existing `_active` claim/release owner (currently edit-named) and private-fixture admission route; do not add an independent opening scheduler or simultaneous effectful attempt pointers. Use the selected single owner/lifecycle state, retained references and deferred entered-call cleanup. Extract only actually shared guards into the planned `godot-addon/native/document_guard.hpp` / `godot-addon/native/document_guard.cpp`; keep editing's writes/T0 restoration/flag clearing/tagging separate. New evidence paths must distinguish valid empty strings from unavailable/oversized getters rather than relying on a silent empty fallback.
  **Guarded behavior:** Follow every precondition and stage in the native contract. Inspect real open/cache identity without loading; retain existing R only when independently matching D and unedited. Pin/read-check the existing file read-only, preserving namespace/identity/content at each stage. Admit the bounded source/current/compiled/effective-context profile; refuse conflicting R/B and unknown unsafe state, but support independently dirty current R == B. Obtain valid current-source evidence before authorization; no waiting for human convergence. Cold acquisition uses only bound `Script.set_source_code` on the new unbound object → bound non-takeover `Resource.set_path` → initial `Script.reload(false)` on that same new still-closed Resource → `EditorInterface.edit_script(script, -1, 0, true)`. Cached R is never reassigned/reloaded/compiled. Target `ERR_PARSE_ERROR` may proceed; invalid/unavailable current-context validation cannot. No project-source write, Save, flag-clearing repair, tag, buffer rewrite, history reset, root loader, target instantiation or rollback.
  **Real validator reuse:** Migrate `mcp-server/src/runner/stock_validation.rs`, `mcp-server/src/runner/stock_validation/admission.rs`, affected `mcp-server/src/runner/stock_validation/protocol.rs` / `ownership.rs`, and `mcp-server/examples/stock_validation_fixture.rs` with a constrained current-context fixture producer. `capture_closure` currently permits supplied source only for `Preflight`: implement distinct `open_context` semantics for exact private observed R == B, not an edit proposal and not a requirement that current D equal unsaved R/B. Bind request/session/current document/path/hash/effective context and completed validity; wrong/missing/replaced evidence never authorizes native effects. Use the existing parent-owned `validate` deadline/reap/private-staging cleanup path, not a helper owned by the killable edit/open worker. Preserve all edit preflight/post-change/unchanged semantics and explicitly reject opening purpose where edit evidence is expected. Use LSP references before changing exported validator symbols; migrate actual consumers in `mcp-server/src/runner/edit/ipc.rs`, `worker.rs`, `supervisor.rs` and `mcp-server/src/bridge/wire/edit/editor.rs` / `output.rs` as required by their types and purpose validation. Do not add a second validator or public arbitrary-source operation.
  **Atomic compatibility migration:** Publishing revision 2 and renaming the product bundle happen in this same task as **all** affected v3 consumers, not as an incompatible intermediate v2/revision-2 head. Update `godot-addon/native/build.py`, extension descriptor generation/installation, plugin loader, native metadata, build-input hashes/manifest and current fixture/workflow consumers. Replace kit-owned `script_edit.gdextension`, `libscript_edit.macos.arm64.dylib` and `script_edit_library_init` with the planned `editor_integration` names; retain only `godot_agent_kit_native` as the shared metadata key. Update `mcp-server/src/bridge.rs`, `mcp-server/src/bridge/wire.rs`, existing edit wire modules, `mcp-server/src/target.rs`, `mcp-server/src/runner/edit/worker.rs`, `godot-addon/addons/godot_agent_kit/bridge.gd` and every descriptor/transcript/revision consumer. V3 authenticates all seven capability bits and revision 0-or-2; old peers reject without fallback. Public observation/edit v1 behavior remains unchanged. The private fixture can invoke the completed native owner; authenticated product `open_gdscript` remains **false until T002 wires the complete opening exchange**. This is truthful capability availability, not a fake opening handler or silent disablement of existing editing.
  **Wire/guard integrity:** Implement the v3 transcript and current-context fingerprint algorithm from the bridge contract with cross-language vectors. Hash only the typed guard projection, excluding fresh clocks/receive times/helper status/progress and the copied source body; recompute on the Rust fixture side and freshly compare native guard values. Keep ordered/bounded fields, same-attempt attribution and actual clock domains. Do not expose a generic serializer framework.
  **Tests and fixture integration:** Add `godot-addon/tests/fixtures/script_open/fixture_driver.gd` and inert fixture sources plus the `native-boundary` group in `godot-addon/tests/run_script_open.py`, reusing the existing owned-project/window/witness harness. Use the actual new addon owner and bridge slot, following the existing private edit fixture route in `godot-addon/tests/fixtures/script_edit/fixture_driver.gd`; no synthetic owner. Extend the existing stock-validator fixture/`godot-addon/tests/stock_acceptance.py` for real context-valid/invalid/unavailable/purpose/source/identity/warning-change and cleanup cases. Migrate existing vectors and capability/build/version/reflection/replay refusals in `mcp-server/tests/bridge_boundary.rs`, `godot-addon/tests/run_observation.py`, `godot-addon/tests/caller_edit_acceptance.py` and native/fixture consumers. Keep fault controls compiled only into the separate `GAK_FIXTURE` artifact. Update `godot-addon/tests/test_native_build.py`, `godot-addon/tests/native_finalization_acceptance.py`, `godot-addon/tests/run_script_edit.py`, `.github/tests/test_live_editor_execution.py` and existing `.github/workflows/ci.yml` / `live-editor.yml` only where current artifact/build commands require migration; no new workflow topology.
  **Independent acceptance:** The real native group must show cold/cached/empty/read-only/syntax-invalid positive opening with unchanged D, actual agreeing R/B, clean B and independently unedited R; property-assignment negative versus exact-method positive followed by a fresh existing observation/edit; dirty-current R == B and background human preservation with actual earlier Undo/Redo; dirty R != B, pending-drag/Undo/stale compiled tool/base/property/method, loader/dependency/global/autoload/extension/profile/limit refusals; and file/cache/session/document/context changes. Prove no entry before bound valid context, no stale/duplicate stage, cache publication as known partial effect, no later stage after terminal expiry/discard, and retained owner/references during entered calls. A between-stage hold must allow a competing actual kit request to receive terminal busy refusal and never run after release; entered-call stalls must not be misrepresented as a responsive editor. Native receipts are facts, not product success. Run the complete existing edit and observation suites on the cutover head, including actual native history/durability and all enabled/disabled/hook-only export variants; preserve their public contracts and deadlines. All relevant Rust/native/workflow checks and actual visible-editor smoke must pass before T001 is complete.
  **Documentation:** Update `godot-addon/native/README.md`, `.github/README.md` where current commands change, relevant current Feature 001/002 reproduction/migration notices, and this feature's native/bridge contracts and quickstart with exact implemented boundaries and evidence. Do not rewrite old acceptance as if it tested v3. Mark only T001 complete; the new product opening capability and feature remain incomplete.

  **Shared values and ownership constraints (verbatim from data-model.md):**

  > Reuse the existing checked `RequestId`, `ProjectRoot`, optional `SessionId`, external `ResourcePath`, `ResolvedTarget`, file/document identities, source/dirty availability, collection stamps, invalidation and observation interval semantics. Their formats are defined by the existing [observation data model](../001-observe-gdscript-state/data-model.md) and [caller contract](../001-observe-gdscript-state/contracts/observation-api.md).
  >
  > - Request IDs: existing safe ASCII format, ≤64 characters; CLI generates a fresh 32-hex ID. IDs are correlation, not idempotency keys.
  > - Session IDs: 32 lowercase hexadecimal characters, bound to the actual authenticated editor lifetime.
  > - Canonical absolute project root ≤1024 UTF-8 bytes; exact `res://` path ≤2048 bytes. No basename/focus selection, URI decoding, path normalization into a different target or outside-project escape.
  > - Source hashes: SHA-256 of exact UTF-8 bytes, with independently checked byte length. Empty source is observed empty, not unavailable.
  > - File/device/inode/object/version counters retain existing checked decimal-string formats at JSON boundaries. Native/editor and supervisor clocks are distinct; no cross-clock numerical equality is assumed.
  > - Source availability remains observed/unavailable/not-applicable with a reason; invalidated evidence is retained only as invalidated, not current. Equality does not imply dirty-state knowledge, freshness or parse validity.
  >
  > Public Rust consumers need the checked opening request, immutable outcome and runner entrypoint. Attempt state, context source, guards, receipts and codec models remain internal/crate-visible unless an actual current consumer requires more. Move or borrow existing evidence; do not copy full source into every progress event or final before/after record.

  **OpeningBinding constraints (verbatim from data-model.md):**

  > The resolved target plus authenticated bridge/native identities, existing connection/request slot ownership, request-local deadline and file identity. It binds the actual project device/inode, session, engine/native build, exact resource path and attempt. A new session/cache/document cannot inherit it. This is private transaction correlation, not a public mutation token or additional transaction/session identifier.

  **CapturedSource constraints (verbatim from data-model.md):**

  > Selected-path D text, SHA-256/length, file identity/stat witness, read interval, confinement/namespace recheck evidence and availability. The worker acquires it independently through the selected project capability. The native boundary independently pins/read-checks the same file before consuming the capture. No caller can provide these bytes as an arbitrary source payload.
  >
  > A cold new-open requires an in-limit supported capture. A cached target requires separately observed R == D plus attributable unedited Resource state. Read-only files are eligible; no write descriptor or writable-mode check belongs to opening. Missing/denied/changed source before effects refuses rather than using cached text as D.

  **TargetState constraints (verbatim from data-model.md):**

  > | State | Required facts |
  > |---|---|
  > | `open` | Unique actual Script/ScriptEditorBase/CodeEdit association, exact path and object IDs; independently observed open state. |
  > | `closed_cached` | Confirmed no target buffer; exact cached GDScript reference, source and Resource edited-state witness. |
  > | `closed_uncached` | Confirmed no target buffer and passive cache lookup establishes absence. Absence is not inferred from a failed lookup. |
  > | `unknown` | Missing/ambiguous open or cache attribution; reason retained. No new opening permitted. |
  >
  > Wrong-type cache occupants, duplicate document paths, changed objects, embedded/non-GDScript paths and unsupported associations have distinguishable refusal reasons. Already-open recognition may retain unavailable D/R/B; target/access-denial precedence and source-suppression rules still apply. A confirmed closed missing file is a missing target, not a valid closed source.

  **OpeningContext — private constraints (verbatim from data-model.md):**

  > The minimum editor state needed because native new-open navigation can apply or validate a departing document:
  >
  > - `kind`: `no_source_editor`, `current_gdscript` or `unavailable`. `get_current_editor`/`get_current_script` and actual association establish it; a missing source getter does not mean no current editor. Unknown/mixed unsupported source-editor context refuses.
  > - For a current GDScript: exact path/project affiliation, Script/editor/buffer IDs, independently acquired R and B, hashes/lengths, current/saved versions, attributable dirty state and history availability. Require R == B without inferring clean state.
  > - Effective internal-editor configuration, source profile, project global-class/autoload names relevant to admission, built-in/extension class bindings and warning/context evidence needed by existing stock validation. All are bounded and rechecked, not assumed from `project.godot` alone.
  > - Passive compiled witnesses: tool flag, script-base reference, property/method metadata. Reject unsupported exported/Object-valued/dynamic-property/stale-base context rather than assert an unobservable pending-export flag. Exact rules are in the native contract.
  > - `validation`: not-applicable only for a proven no-source context; otherwise complete valid source-only validator evidence bound to this exact current source/path/session/request/context. Invalid/unavailable current validation refuses before authorization.
  >
  > Current/unrelated source and paths stay private to safety processing and owned temporary validation. Public results report only a bounded context status/reason; they never disclose another document's source as target evidence. A dirty current document may be supported; no product operation waits for its R/B to converge or changes it to make it eligible.

  **PreparedOpening constraints (verbatim from data-model.md):**

  > Immutable binding, target mode, captured D, retained cached R if any, native file/reference guards, context witness, optional private current-validation receipt and deadline. It owns no source write or published new Resource. Preparation must be incapable of applying after a terminal cancellation unless a separately recorded authorization was already released; in that case uncertainty is retained until irrevocable discard is proved.

  **NativeLifecycleFacts constraints (verbatim from data-model.md):**

  > Monotonic facts, independently attributed to the existing request/session/connection owner:
  >
  > | Fact | Meaning |
  > |---|---|
  > | `cache_binding` | Not started, reused existing R without effect, new Resource published, failed before publication, or unknown. New publication is a lifecycle effect. |
  > | `initial_compilation` | Not applicable for retained R; for a new object: not started, completed valid, completed invalid with an attributable engine error, unavailable/failed, or unknown. Never reload existing R. |
  > | `document_open` | Not started, entered, actual new document association obtained, failed/unverified, or unknown. Entry/acknowledgment is not independent final verification. |
  > | `selection` | Native selection effect known/unknown and requested-target/other/no-source relation; no unselected source or project path is exposed. |
  > | `terminal_discard` | Attempt cannot accept any future lifecycle stage; identifies the owned request and whether any effect was already known. |
  >
  > An unbound temporary native object is not the target R. Native code may prove no target effect if it fails before cache publication/opening and irreversibly discards all work. Once publication/document/selection effect is established, later failure cannot reduce application to not-applied. Releasing references is ordinary ownership cleanup, not a claimed rollback of editor state.

  **Native metadata bounds (verbatim from contracts/native-integration.md):**

  > Metadata limits are explicit refusal limits, not throughput promises: ≤64 effective global-class names and ≤64 autoload names, names ≤256 UTF-8 bytes; ≤256 distinct source-identifier ClassDB lookups per source; ≤64 compiled property records and ≤64 compiled method names per inspected Script, property/method names ≤256 and property hint strings ≤2048 UTF-8 bytes. Bound their combined non-source admission record to 256 KiB. Warning/context/diagnostic collection limits remain those of the existing stock validator. Oversize/unknown context refuses before authorization rather than truncating away a name or property. No global registry monitor or reusable class index is introduced.

  **Constraint ownership:** These quotes define the native/private current consumer in T001 and remain mandatory for T002's Rust domain/adapter. CLI-generation and public-outcome clauses are realized with the real caller in T002, not as unused declarations in T001. Complete the boundary, validation and cutover behavior before checking T001; quoted constraints are not a substitute for implementation or evidence.

## Phase 3: User Story 1 — Open a Known Closed Script and Continue Safely (Priority: P1)

**Goal:** One usable public known-path opening caller with no manual tab preparation, no separate mutation policy and all inseparable preservation/refusal/interruption behavior.

**Independent test:** Exercise US1.1–US1.5 through the actual binary in the exact selected visible editor: cold and retained-cache targets, same basenames/sessions, independent unchanged D/equal R/B/clean B/unedited R, a syntax-invalid target, and separate fresh observation followed by a real existing edit. No fixture direct-open action may substitute for the caller transition.

- [ ] T002 [US1] Deliver the complete guarded opening caller and its consumed core in `mcp-server/src/script_open.rs`, `mcp-server/src/runner/open.rs`, `mcp-server/src/bin/open-gdscript.rs`, `mcp-server/src/bridge/wire/open.rs` and `godot-addon/addons/godot_agent_kit/bridge.gd`, bundling contract/process/live-editor tests in `mcp-server/tests/script_open_contract.rs`, `mcp-server/tests/script_open_caller.rs` and `godot-addon/tests/run_script_open.py` with `specs/003-open-project-gdscript/quickstart.md`.
  **Depends on:** Completed T001 and its merged PR unless a stacked-PR workflow is explicitly authorized. Do not redo its private cutover or silently bypass a failed native/current-validation prerequisite.
  **Scope and ownership:** Implement the planned protocol-independent opening domain, ordered state/terminal reduction and exact caller v1 in the existing Rust package; expose only the checked request, immutable outcome and runner entry actually consumed by the new CLI/library consumer. Keep attempt state, current source, guards, receipts and codecs private/crate-visible. Keep semantic policy separate from JSON/framing/supervision/native effects; no edit-derived opening state machine. Add the binary declaration in `mcp-server/Cargo.toml` and narrow module wiring in `mcp-server/src/lib.rs`, `runner.rs` and `bridge/wire.rs`. Reuse `observation.rs`, `target.rs` and `project_fs.rs` evidence and confined reads without changing existing operation semantics. Use cohesive private submodules for worker/IPC/supervision if needed rather than append another lifecycle to the large edit runner.
  **Actual opening exchange:** Integrate every opening tuple/response from the bridge contract (`open_begin`, `open_prepare`, `open_advance`, `open_verify`, `open_recheck`, `open_finish`/`open_abort`) with T001's `script_open.gd` owner, ordinary collector and native family. Update `godot-addon/addons/godot_agent_kit/plugin.gd`, `bridge.gd` and `script_open.gd` for authenticated product admission; advertise `open_gdscript` true only when the complete matched native/integration family is usable. Acquire no source/hash/current context before unique authenticated selection. Retain source-free ambiguity/denial and exact session/target binding; no editor launch, focus-based selection or retargeting.
  **Two branches:** Already-open uses fresh ordinary observation plus exact identity/open-state recheck with no lifecycle authorization, profile admission, current-source helper, selection or history action; preserve partial per-source observations and dirty/divergent/equal-but-dirty state. Closed uses independent Rust D capture, passive exact cache facts, immutable preparation, T001's current-context/source/effect guards and real `open_context` helper evidence. Parent supervisor owns helper cleanup/deadline and records may-apply before releasing exactly one authorization; killable worker cannot authorize early or orphan a helper. Cached/cold stage differences and target-invalid versus current-invalid semantics remain distinct. Fresh separate R/B/dirty/Resource-edited/protection acquisition plus independent D/recheck is required for success; copied initialization or stage receipts never fill a missing authority.
  **Field/codec constraints:** Implement all public fields, nullability, exact enums/reasons, stage facts, before summaries, one latest snapshot, separate Resource-edited and parse evidence, privacy projection, exit codes and fixed next-action guidance in the caller contract. Treat omitted/unknown/wrong-type input, unsafe numeric conversion, attribution/sequence mismatch, oversized lengths, source-local limits and malformed evidence as defined failures, never defaults. Use the exact v3 control shapes, fingerprint rules and frame bounds already selected; before/after samples keep acquisition purposes and invalidation. Source hashes/lengths come from actual bytes; empty source is not missing. Current/unrelated source/path/hash/diagnostics must not escape through serialization or Debug/log paths.
  **CLI and time:** Require `--registry`, `--project`, `--script`; omission of `--session` requires one authenticated session. Fresh 32-hex request ID, stdin unused, one bounded normal JSON stdout result plus newline, source-free stderr and the existing help convention. Reject duplicate/unknown/positional/force/reload/focus/retry/source options. Schema `1`, operation `open_gdscript`; successful new/recognition exits 0, refusal 3, applied-unverified/unknown 4, invalid arguments 2. Preserve ordinary output-delivery failure handling and make cancellation effect-sensitive. Start 9.5-second operation time before parsing/resolution, retain 0.5-second delivery reserve with stdout drained, and leave observation/edit deadlines unchanged. No automatic retry/reconnect/queue/rollback/close or response upgrade after terminal delivery.
  **Required tests before completion:** Bundle pure reducer/constraint/regression cases in `mcp-server/tests/script_open_contract.rs` (or private same-module tests for private state), process cases in `mcp-server/tests/script_open_caller.rs`, and actual bridge/confinement cases in `mcp-server/tests/bridge_boundary.rs` / `confinement.rs`. These must distinguish independent success from acknowledgment, dirty-equal recognition from clean inference, absent from unavailable cache, incomplete/current-invalid validation from target parse error, fresh/changed identity and same-text newer versions, source suppression, known effects surviving later faults, unknown post-authorization loss, irrevocable discard, late results and real ten-second bounds. No public internals solely to support an external test. Exercise a real compiled caller/runner, real child/connection failures and owned helper cleanup, not only fabricated frames.
  **Live-editor groups:** Implement and pass `new-open`, `already-open`, `preservation`, `routing`, `interruption` and `privacy-export` in `godot-addon/tests/run_script_open.py`, with private preparations/witnesses in `godot-addon/tests/fixtures/script_open/fixture_driver.gd`. US2.1–US2.5, US3.1–US3.5 and US4.1–US4.3 are mandatory **T002 acceptance**, not future-task work. Include actual dirty current R == B/background positives, native prior-history Undo/Redo, current R != B and pending-drag/stale-context refusals, boundary/human reopen/typing races, private shared-slot busy with no late entry, and stalls/loss before authorization, after authorization, after cache publication, during compilation/opening and verification. Independently observe survivor state; known partial effects cannot collapse to not-applied or unknown. Observe new selection without restoring it after human navigation. Time every controlled caller case externally and perform result-only interpretation before comparison with fixtures.
  **Preservation and packaging gates:** Prove public open → fresh observe → eligible edit without tab preparation, dirty refusal, actual edit Undo → Save → Redo → Save and ordinary applicable persistence/reparse/rescan/runtime smoke; retain existing A–E coverage. Run the full existing edit/observation suites on the integrated head. Verify selected/current/unrelated/other-project sentinels in normal/denied/ambiguous/interrupted paths and cleanup; all enabled/disabled/hook-only production exports must exclude the new opening node, native/tooling files and active behavior, then run the export. Do not defer a known broken safeguard or export path to T003. T003 supplies the complete new cumulative stress/composed matrix and feature-wide evidence, not missing operation safety.
  **Independent acceptance and docs:** All public branches and required groups above pass on the same head with unchanged old public contracts and applicable Rust/native/workflow checks. Update the caller/bridge/native contract status and exact runnable commands/evidence in `specs/003-open-project-gdscript/quickstart.md`, current native/operator docs and `PROJECT_STATUS.md`. No blanket support or feature-completion claim yet. Mark only T002 complete after shape review; publish its dedicated PR and stop.

  **OpenRequest constraints (verbatim from data-model.md):**

  > | Field | Meaning / validation |
  > |---|---|
  > | `request_id` | Fresh checked request correlation. |
  > | `project_root` | Explicit selected project; same checked type as observation. |
  > | `session_id` | Exact lifetime if supplied; omission requires unique authenticated selection. |
  > | `script_path` | Standalone external `.gd` locator, exact spelling. |
  >
  > No replacement source, prior edit basis, force/reload, focus control, permission override, retries or arbitrary command. The adapter may privately wrap/borrow an `ObservationRequest` to reuse target resolution; it must not turn observation into opening.

  **VerificationEvidence constraints (verbatim from data-model.md):**

  > Fresh target observation from the existing collector, independent fresh D, same-identity/namespace/source/context rechecks, actual open/buffer identity and dirty state, observation interval and detected invalidation. Neither the captured source nor native initialization receipts may stand in for these reads.
  >
  > For new opening, require all applicable D/R/B observed and exactly equal to the admitted source; target open with the exact expected Script and actual buffer; clean attributable buffer dirty state and separately observed Resource edited=false; stable required file/session/document/protection witnesses. Source/profile/context changes that invalidate authorization prevent success even if later text happens to match. Resource edited state is opening-specific supplementary evidence, not a change to observation v1.
  >
  > Already-open recognition requires exact currently established open identity, no lifecycle authorization/effects, and the necessary identity/open-state recheck. Its source observations may be dirty, divergent, limited or explicitly invalidated; they do not become complete/coherent merely because the document was recognized. If open identity itself becomes unknown/changed, return a no-effect refusal/interruption instead.

  **OpeningOutcome constraints (verbatim from data-model.md):**

  > Immutable public domain result containing:
  >
  > - Schema/operation version at the adapter, request and requested/resolved target identity.
  > - Overall outcome, reason, reached stage and application certainty.
  > - Bounded per-stage effect/progress facts and actual observation interval.
  > - Before summaries (availability/hash/length/identities, not duplicate source bodies) and at most one latest authorized target source snapshot.
  > - Target parse evidence if actually obtained, separate from opening success. Cached R is not recompiled to fabricate a parse result. Missing parse evidence is explicit, not a blocker to otherwise verified opening.
  > - Private-context disposition without current/unrelated source; detected changes and safe source-free diagnostics.
  > - History participation `not_participated` and relevant observed preservation evidence/limitations; selection effect and safe next action.
  >
  > A parse-invalid newly opened target can be a successful open. An invalid current context is an admission refusal, not a target parse error. Only the existing edit operation promises valid post-edit parse evidence.

  **Ordered attempt state constraints (verbatim from data-model.md):**

  > ```text
  > Accepted -> Selected -> Prepared
  >                          |
  >                          +-- already open -> Observe/recheck identity -> Terminal
  >                          |
  >                          +-- closed -> Validate current context (or proven N/A)
  >                                       -> Authorized
  >                                       -> Bind/reuse Resource
  >                                       -> Compile new Resource only
  >                                       -> Open document
  >                                       -> Independently observe/recheck
  >                                       -> Terminal
  > ```
  >
  > Stages exposed by the caller are `accepted`, `selected`, `prepared`, `context_validating`, `authorized`, `binding`, `compiling`, `opening`, `verifying`, `verified`. Omitted stages are not fabricated: cached R skips compilation, no-source current context skips validation, and already-open skips every authorization/effect stage.
  >
  > Use one attempt-state enum carrying the data valid for that state, not several booleans that allow impossible lifecycle combinations. Native session ownership likewise admits exactly one read/edit/open owner and prevents multiple live effectful attempt pointers. No generalized transaction framework is introduced.
  >
  > Preparation remains non-applying. Parent authorization changes knowledge to **may apply** before the control write. Child/bridge/native progress must be monotonically ordered and bound to the same attempt. A queued, stale, expired or mismatched stage is not a new prepare and cannot resurrect a discarded attempt. Native `entered` ownership survives cancellation/disable until the synchronous call returns.

  **Terminal outcome, enum and precedence constraints (verbatim from data-model.md):**

  > | Outcome | Required meaning |
  > |---|---|
  > | `verified_newly_opened` | Request caused a new open; fresh independent target and protection postconditions passed. Application is applied. |
  > | `already_open_unchanged` | Exact existing document recognized; no lifecycle authorization/effect; observations retain their own limitations. Application is not-applied. |
  > | `refused` | No lifecycle effect occurred and none can occur later from this attempt. Specific input/target/context/evidence/capability reason. |
  > | `applied_unverified` | Some lifecycle effect is known but verification is incomplete/invalidated or the operation only partly applied. Preserve known effects and missing facts. |
  > | `effects_unknown` | Authorization/dispatch may have led to effects, but their occurrence/extent is not established. No false no-effect or rollback claim. |
  >
  > Reduction rules:
  >
  > 1. Never disclose source before exact authorized target selection; ambiguous/denied attribution suppresses unauthorized source even if a malformed frame supplied it.
  > 2. Validate every fact's request/session/target/connection ownership/clock/sequence before use. Invalid later messages cannot erase earlier valid known effects or upgrade terminal success.
  > 3. If effects are known, any timeout, disconnection, protocol failure, missing observation, context change or divergence produces `applied_unverified`, not `refused`/`effects_unknown` merely because later evidence is missing.
  > 4. If authorization was released and no valid effect or irrevocable no-effect proof exists, use `effects_unknown`.
  > 5. Before authorization, or after an attributable irrevocable pre-effect discard, report `refused` for failures. A refused busy request is never retained/queued for later entry.
  > 6. Only evaluate positive outcomes after all of their distinct verification conditions. Already-open does not require clean/equal/full source; newly opened does.
  > 7. A known source/context invalidation cannot be erased by eventual equality. Preserve newer human work; do not reassert captured bytes, Save or close a buffer to repair the result.
  >
  > `application` is `not_applied`, `applied`, `partly_applied` or `unknown`. It describes actual lifecycle knowledge, not source bytes written. `partly_applied` includes a new cached Resource without a verified document. `stage` is the furthest entered stage, not proof that it completed.

  **Deadline, cancellation and retry constraints (verbatim from data-model.md):**

  > The existing supervisor clock starts before CLI parsing/resolution. Acquisition/effects/verification share 9.5 seconds; final delivery has a 0.5-second reserve with a draining stdout consumer. Native deadlines use editor-local remaining-time limits; the parent never numerically compares unrelated clocks.
  >
  > Timeout does not cancel a synchronous native call. Parent terminates/reaps only owned worker/helper processes; editor/native ownership remains safe until entered work returns. Each new stage must check expiry/identity/source/context before effects. Late evidence cannot amend a delivered outcome. There is no automatic reconnect, retry, queued resume or response replay cache.
  >
  > After any possibly applied result, freshly observe the original explicit target and inspect human work before another intentional operation. A new request for a still-open document may then be satisfied without effects. This is conditional no-change behavior, not unconditional idempotency across close/reopen, replacement or session restart.

  **Privacy, size and compatibility constraints (verbatim from data-model.md):**

  > Public output includes only the authorized requested target's source; private current-source validation never becomes a second source result. Preserve actual empty values and per-surface unavailable reasons. Never truncate text to fit a frame or infer cleanliness from equality. Use before summaries plus one final snapshot to avoid unnecessary source copies and result inflation.
  >
  > Retain 512 KiB/source, 4 MiB selected-peer request, 12 MiB response/worker frame, 4 KiB source-free control and JSON depth 32 limits. Bounded source/profile/compiled/context collections are specified in the contracts. A limit during new-open preparation refuses before effects where observable; a newly discovered post-effect limit produces an unverified result. Already-open recognition retains the existing independent source-limit semantics.
  >
  > Opening v1 is a new public operation. Observe/edit v1 semantics stay unchanged. Private v3/native revision 2 is a coordinated clean cutover, including the shared native bundle rename. No obsolete aliases, old mutation fallback or feature/task IDs in shared APIs. This model changes no current implementation until an approved task is selected.

  **Inherited constraints:** T001's verbatim shared-value and native entity constraints also apply to the corresponding T002 Rust models, producers and codecs. All required/optional/nullable fields and wire-only enum/diagnostic bounds in `contracts/open-api.md` remain mandatory at this actual adapter; no summary here replaces that full contract.

## Phase 4: User Story 2 — Keep Existing Buffers and Human Work Intact (Priority: P1)

**Goal:** Already-open requests are genuinely non-interfering; new opening preserves all applicable existing human source/dirty/history state and later human work.

**Implementation ownership:** T001 supplies the actual native/context guards; **T002 owns the complete caller behavior and US2.1–US2.5 acceptance**. No additional checkbox: a separate “add preservation” or tests-only PR after a working unsafe opener would violate the specification. T002 cannot be marked complete before this phase passes.

**Independent test criteria:** Through the public caller, request clean and non-selected targets, dirty-different, dirty-equal, divergent and independently source-limited targets. Require exact existing identity and unchanged selection/source/dirty/history with no duplicate buffer or lifecycle authorization. Open a different target with an independently dirty current R == B and with dirty background work, preserving actual earlier native Undo/Redo. Current R != B and unavailable safety facts refuse before native navigation. Human opens/edits before entry either receive fresh no-effect recognition or an attributable refusal; an identity change is never automatically rebased. Human text or close/reopen after effects survives with truthful applied-unverified evidence and no repair/closure. Observe actual D, R, B, Resource-edited and current/saved versions, not merely equal text or `has_undo`.

## Phase 5: User Story 3 — Refuse Unsafe or Unidentifiable Opening (Priority: P1)

**Goal:** A path never grants outside-project/source-execution authority or permits stale/ambiguous targeting.

**Implementation ownership:** T001 owns the native/profile/private-validator/v3 boundary; **T002 owns public routing/refusal and US3.1–US3.5 acceptance**. No separate safety checkbox or later authorization framework. T002 cannot be marked complete before this phase passes.

**Independent test criteria:** Distinguish ambiguous/ended/replaced/unavailable sessions and denied/outside/missing/embedded/non-GDScript/unsupported targets without candidate source. Confirm retained conflicting/dirty/unknown R is refused rather than treated as absent or repaired. Replace target file/parents/cache/document/session between checks and effects; preserve the original attribution and never open a replacement under stale intent. Exercise effect-profile and unavailable-current-context refusals, exact/over limits, wrong identities/frames and a real overlap with no queued/late effect after terminal refusal. Include a genuine safe syntax-error positive so parser invalidity does not become a blanket unsupported shortcut. Known unsafe context is not waived by the inherited malicious-plugin/endpoint limitations.

## Phase 6: User Story 4 — Understand Interruptions and Repeated Use (Priority: P1)

**Goal:** Effect certainty remains truthful through interruptions and repeated ordinary editor use, with complete composed durability, privacy/export and exact compatibility evidence.

**Ownership:** T002 implements and proves US4.1–US4.3 and first-caller privacy/export behavior before exposure. T003 owns US4.4–US4.6 cumulative proof and reruns the whole capability, including the earlier interruption matrix. No late “make cancellation safe” task is permitted.

**Independent test criteria:** Compare every interrupted result with actual survivor state and original target/interval: no false verified, no false not-applied, no implied rollback, no late application after proven refusal, no automatic retry/retarget, and every controlled caller ≤10 seconds. Then run at least twenty opening requests with five genuinely closed successes and five dirty already-open preservations, deliberate human close/reopen/typing, and the full composed A–E/durability/export matrix below.

- [ ] T003 [US4] Deliver cumulative repeated-opening and composed-workflow acceptance in `godot-addon/tests/run_script_open.py` and `godot-addon/tests/fixtures/script_open/fixture_driver.gd`, reusing `godot-addon/tests/run_script_edit.py` and `godot-addon/tests/run_observation.py`, and record verified compatibility/completion in `specs/003-open-project-gdscript/quickstart.md`, `specs/003-open-project-gdscript/tasks.md` and `PROJECT_STATUS.md`.
  **Depends on:** Completed T002 and its merged PR unless stacking is explicitly authorized. T001 native and T002 caller acceptance must already hold; no missing public behavior or known safety failure is moved into this task.
  **Implementation:** Add the meaningful `sequential` and `composed` workflows to the existing opening runner and a complete `all` path that executes every quickstart group exactly once. Reuse or narrowly extend the existing edit/observation fixture helpers, including `godot-addon/tests/cumulative_edit_acceptance.py` and `caller_privacy_acceptance.py` where their real behavior is consumed, instead of cloning suites or creating a feature-specific CI workflow. Support normal and separate native-fault artifacts, absolute configured paths, new private artifact directories, owned-window proof, source-free result review and process/private-staging cleanup. Correct any newly discovered in-scope regression in its owning production module within this task, with a meaningful regression test and affected-acceptance reevaluation; do not broaden architecture or quietly lower assertions.
  **Opening stress:** At least 20 real caller requests across supported closed, already-open clean/dirty/divergent and unsafe targets; at least five must start genuinely closed and return verified newly opened, and at least five must prove dirty already-open preservation. Include deliberate human close/reopen, text/dirty/version changes and prior-history transitions between requests. Verify current identities, no duplicate buffers, unchanged source on opens/refusals, no late reopens after a proven refusal, and native history preservation. These are distinct state transitions, not repeated same-input rows counted as new scenarios.
  **Composed A–E and durability:** The first closed-to-open step uses **the actual product opener**, followed by separate fresh observation and existing edit; no manual/direct fixture open substitute. Prove A clean edit coherence, B dirty-different and dirty-equal refusal, C real apply → Undo → ordinary Save → Redo → Save plus prior history, D ordinary Save and human close/reopen/cache-consumer persistence, and E twenty fresh-basis edits with the existing unsafe interleavings. Include reparse, rescan and a permitted owned runtime fixture whose behavior reflects the intended edit, not merely launch/exit. Preserve read-only observation, closed-target edit refusal, exact-source/current-state requirements and all existing supported edit behavior. Opening itself must add no source-history action; no reversible-tab-open claim or new Save/close/runtime product command.
  **Full acceptance:** Run unfiltered `run_script_open.py --scenario all` with all required inputs, then the existing complete `run_script_edit.py --scenario all` and `run_observation.py --scenario all` under the quickstart commands in separate private output directories. Run GUI groups serially. Reuse earlier evidence only where the delivery head has not invalidated it, but do not substitute native planning probes or old edit runs for this feature's complete caller/composed acceptance. Check every result alone for target/effect/evidence/next-action interpretability, then compare independent witnesses and external elapsed time. Distinguish invocation records from distinct scenarios; do not double-count groups or label filters `all`.
  **Privacy/export/support:** Re-run selected/current/unrelated/other-project sentinels through authorized, ambiguous, denied and interrupted paths, including real helper failure/cleanup. Prove no current/unselected source/path/hash, raw diagnostics or secrets in public output/logs/registry. Inspect actual enabled-addon, disabled-addon and hook-only production exports for all tooling/native/fixture artifacts and credentials, run them and observe no tool listener/native registration/gameplay dependency. Preserve exact engine/library/ABI/source/toolchain provenance and the scoped endpoint/callback limitations; claim no untested environment or stronger isolation. Existing hosted/native/workflow checks cover changed surfaces without a new provider or trust boundary.
  **Completion gate:** All 21 story scenarios, FR-001–FR-018, SC-001–SC-007, eight edge cases, applicable A–E and Save/reopen/reparse/rescan/runtime, unchanged observation and all export gates pass on the delivery head. Perform the proportional implementation-shape review and required Cargo/native/workflow checks, update quickstart/current operator and contract status to the verified guarantees, mark only T003 `[X]`, and record Feature 003 complete independently of PR merge. Roadmap Phase 1 remains in progress unless its separate remaining discovery/lifecycle exit gates are actually satisfied. Publish the T003 PR and stop; do not auto-merge or start another feature.
  **Constraint inheritance:** All verbatim model constraints attached to T001/T002 and all caller/native/bridge contracts remain requirements for each sequence, record, failure and any regression fix in T003. No new request/model field, transport, permission or broader compatibility profile is introduced by this cumulative task.

## Phase 7: Polish & Cross-Cutting Concerns — Bundled, Not Separate Tasks

Required docs, cleanup of owned temporary scaffolding, compatibility migration, privacy/export checks and implementation-shape review belong to the task introducing/changing the behavior. There is no standalone formatter, generic hardening, documentation or cleanup PR. T003 is a new cumulative state-transition/compatibility outcome, not permission to postpone known T001/T002 safeguards.

All three tasks retain local-only authenticated routing, protocol-independent semantics, human-work preservation, truthful evidence and exact support boundaries. This phase does not authorize discovery, other lifecycle controls, MCP, runtime/debugger tooling, new dependencies or unplanned security infrastructure.

## Dependencies & Execution Order

### Task graph

```text
Completed Features 001 and 002 + reviewed artifacts + required analysis
                              |
                              v
T001: native/current-source capability + atomic v3/revision-2 cutover
                              |
                              v
T002: complete public caller + consumed core + all inseparable safety
                              |
                              v
T003: repeated-use/composed acceptance + verified feature completion
```

Each edge means prerequisite acceptance **and** normal PR-merge sequencing, unless the maintainer explicitly authorizes stacking. `[P]` would describe technical independence, not permission to batch tasks, bypass review/merge or run GUI suites concurrently. No task receives `[P]`: this graph has real dependencies and shared integration paths. Setup and final polish create no extra prerequisites.

### Story completion order and owners

| Story | Primary checkbox | Shared implementation/evidence | Completion condition |
|---|---|---|---|
| US1 — known closed opening (P1) | T002 | T001 native/current-source boundary; T003 cumulative regression | US1.1–US1.5 through the real caller, including fresh observe/edit. |
| US2 — human-work preservation (P1) | No additional checkbox | T001 guards + T002 complete behavior/tests | US2.1–US2.5 pass before T002 completes. |
| US3 — safe refusal (P1) | No additional checkbox | T001 effect/confinement/version guards + T002 routing/refusal | US3.1–US3.5 pass before T002 completes. |
| US4 — interruption/repeated use (P1) | T003 | T002 already owns US4.1–US4.3 and first privacy/export proof | T003 completes US4.4–US4.6 and revalidates the whole story. |

US1–US3 and the first three US4 scenarios are independently testable outcomes of one safe caller, not independently shippable unsafe stages. Their acceptance is concurrent in T002's completion gate; the story-phase order does not defer safety. Full US4/feature completion follows T003.

## Parallel Execution Examples by Story

These are optional **within-one-selected-task** authoring slices, not extra tasks, parallel task delivery or permission to begin a dependent task. Fix the existing contract/interface before splitting disjoint ownership; one integration owner gathers all results before builds/tests, then runs actual GUI acceptance serially. Do not concurrently edit shared files. Rows naming the same files are alternative examples, not mutually independent batches.

| Story / selected task | Disjoint authoring example | Integration condition |
|---|---|---|
| US1 / T002 | One contributor owns `mcp-server/src/script_open.rs`, `mcp-server/src/runner/open.rs`, `mcp-server/src/bin/open-gdscript.rs`, `mcp-server/src/bridge/wire/open.rs` and Rust tests; another owns product wiring in `godot-addon/addons/godot_agent_kit/script_open.gd` / `bridge.gd` and `godot-addon/tests/fixtures/script_open/fixture_driver.gd`. | Use T001's accepted native boundary and fixed v3 contracts; one owner integrates, then exercises the real caller. |
| US2 / T002 | Rust outcome/dirty/identity cases in `mcp-server/tests/script_open_contract.rs` can be authored separately from actual dirty/human-history fixture preparations in `godot-addon/tests/fixtures/script_open/fixture_driver.gd`. | Both must preserve identical dirty-equal/changed-identity semantics; real preservation/Undo/Redo waits for integrated code. |
| US3 / T002 | Rust route/denial/framing cases in `mcp-server/tests/bridge_boundary.rs` and `mcp-server/tests/confinement.rs` can be authored separately from controlled file/cache/context refusal preparations in `godot-addon/tests/fixtures/script_open/fixture_driver.gd`. | No widened profile or source disclosure; one owner observes before-effect refusal and no late entry. |
| US4 / T003 | One contributor owns cumulative sequences in `godot-addon/tests/run_script_open.py`; another reviews/updates affected export/workflow regression handling in `.github/tests/test_live_editor_execution.py` and `.github/README.md` only where current commands/evidence require it. | No invented busywork or new workflow; combine results before complete serial GUI/export runs and result-only review. |

T001 may likewise split native/source/owner work from the existing Rust validator migration or disjoint fixture authoring after interface ownership is fixed, but its native revision, v3 peers, installed artifact and existing consumers ship atomically in one PR. Do not create separate worker-side validation ownership or duplicate source policy to make tasks appear independent.

## Requirement and Acceptance Coverage

| Required acceptance | Owning tasks | Required proof |
|---|---|---|
| US1.1–US1.5; SC-001 | T001 native; T002 caller; T003 cumulative | Cold/cached/exact target, actual source/buffer/clean/unedited evidence, syntax-invalid positive and fresh observe/edit. |
| US2.1–US2.5; SC-002 | T001 native guards; T002 caller | Already-open non-selection, dirty-equal/divergent/limited evidence, unrelated/current human history, before/after-boundary changes and no repair. |
| US3.1–US3.5; SC-003 | T001 guard/validator/cutover; T002 end-to-end | Ambiguity/access/type/profile/cache/unknown facts/identity/namespace/overlap refusals with no unauthorized source or prohibited effect. |
| US4.1–US4.3; SC-004 | T001 stage/owner facts; T002 reducer/real processes/editor | Known/partial/unknown precedence, bounded cancellation/loss, immutable terminal results, no late application after proved refusal and fresh-observation guidance. |
| US4.4; SC-005 | T003 | ≥20 actual opening requests, ≥5 genuinely closed successes and ≥5 dirty recognitions with deliberate human interleavings. |
| US4.5; SC-006 | T001/T002 affected regression gates; T003 complete composition | Product opener before fresh observe/edit, all applicable A–E, native history and ordinary Save/close-reopen/reparse/rescan/runtime; old public contracts unchanged. |
| US4.6; SC-007 | Every introducing task; T002 first caller, T003 cumulative | Result-only interpretation and actual privacy/export proof, including private current-source validation and all three export modes. |
| FR-001–FR-002 | T001 compatible private boundary; T002 public surface | One known target, source-free unique authenticated routing, no implicit open/guess/replacement. |
| FR-003–FR-007 | T001 actual guards/owner/native methods; T002 consumed policy | Cache absence versus unavailability, fresh guarded entry, native no-write lifecycle, unchanged recognition and no unpermitted source/dependency effects. |
| FR-008–FR-009 | T001 independent native facts; T002 final reduction | Separate D/R/B/clean/unedited/protection verification; limited recognition not edit authorization. |
| FR-010–FR-014 | T001 helper/entered lifetime; T002 complete caller/codec | Structured effects/intervals/reasons, ten-second result, newer human work, no false rollback/late apply or automatic retries. |
| FR-015 | T001 limits/profile/getters; T002 adapters/reducer | Exact source/availability/nullable handling, boundary and over-limit cases, no truncation or inferred empty/clean values. |
| FR-016–FR-017 | Every introducing task; T003 full regression | Unchanged observation/edit v1, native/reducer boundaries, local/private/export isolation and no new gameplay authority. |
| FR-018; SC-001–SC-007 | T001/T002 owned acceptance; T003 cumulative completion | Required pure, process/boundary and actual visible-editor proof, exact supported head, no borrowed planning/partial-suite claim. |

All eight spec edge cases have owners: empty/representation/size (T001/T002); retained cached R (T001/T002); non-selected dirty target (T002); rename/delete/replacement/path redirects (T001/T002); readable read-only source (T001/T002); syntax error independent of opening (T001/T002); native prompt/stall with no acceptance/Save/discard (T001/T002); and human tab/close/reopen changes during verification (T001/T002). T003 exercises them in the cumulative matrix where applicable. This traceability cannot narrow a normative scenario, bound, outcome or gate in the linked artifacts.

## Implementation Strategy

### First Useful Slice — Requested MVP Scope

Deliver T001's complete native/current-validation boundary, then **T002's complete public caller with US1, US2, US3 and US4.1–US4.3 safety together**. “US1 only” cannot mean an opener that later gains dirty protection, refusals or truthful uncertainty. The first useful caller is two PRs from the current implementation baseline, not a chain of file/model/schema/helper setup PRs. No feature/release-completion claim precedes T003.

### Incremental Delivery

1. T001 adds one real reusable native capability and migrates its genuinely coupled current consumers, with private native/context validation and old product regressions proved. Opening remains unadvertised at the unfinished product exchange; no no-op command is published.
2. T002 supplies the actual caller/runner and its now-consumed core, exact private exchange, all safe branches and actual first-caller acceptance. This is the first product opening capability, not an experimental unsafe path.
3. T003 supplies the cumulative transitions/composed compatibility evidence, full runner, source/privacy/export review and exact feature completion. It does not absorb known incomplete earlier acceptance.
4. Stop at every task PR. Completion follows accepted behavior/evidence/shape, not merge status; authorization to begin the next task still follows the repository's sequencing policy. No automatic task or feature selection.

## Granularity Review

**Review state: PASS (2026-09-30).** Parent review plus independent coverage/contract and granularity/dependency reviews found no actionable task-boundary, coverage or cutover defect. T001 has real shared-slot/native and validator fixture consumers; T002 consumes its core in the actual caller and bundles every inseparable safeguard; T003 adds cumulative transitions and compatibility evidence rather than deferred safety. This is the required artifact/granularity review, not `/speckit.analyze`, implementation approval, a separate Spec Kit task or a new CI/reviewer approval gate.

The decomposition deliberately merges a proposed standalone pure-core foundation into T002: the current repository has no opening product consumer, and an isolated private reducer would create dead code or unjustified public state merely to be callable. The native boundary has a real existing fixture integration pattern and stock-validator consumer, so T001 remains independently verifiable. Publishing native revision 2 cannot temporarily break v2/revision-1-only loader/caller gates; all version/artifact consumers therefore migrate in that same T001 PR.

| Review criterion | Decomposition |
|---|---|
| Meaningful PR-sized increments | Complete guarded native/current-source capability and compatible cutover; complete public caller/core with inseparable safety; cumulative state-transition/compatibility acceptance. |
| No trivial fragments | No directory, enum, method, fingerprint, helper-purpose, rename, core-only, formatting or tests-only PR. |
| Tests/docs bundled | Each task owns failing-before where executable, passing boundary/smoke/evidence and affected docs. US2/US3 and interruption behavior cannot be delayed beyond T002. |
| Reasonable serial path | One verifiable native prerequisite before the first useful caller; two PRs to T002, three total. The existing validator is reused rather than rebuilt as another foundation. |
| Preserved coverage | Four P1 stories, all 21 scenarios, 18 FRs, seven SCs, eight edge cases and applicable A–E/durability/privacy/export obligations have explicit owners. |
| Clear dependencies | T001 → T002 → T003, no hidden parallel task permission, no cyclic story prerequisite. |
| Current consumers and cost | Existing native/addon fixture owner and stock-validator example consume T001; actual CLI consumes T002 core. Shared names/visibility follow durable current responsibilities, not future feature/task IDs. |
| Constitutional scope | Guarded native no-source-write opening, preserved human work/history, independent verification, typed uncertainty, exact local confinement and actual GUI/export proof remain mandatory; no new service, sandbox, approvals or wider support. |

**Artifact validation:** Three unique sequential pending checklist rows have exact file paths and the correct foundation/story labels; no task is marked `[P]`. Fourteen complete data-model constraint blocks and the native metadata bounds are quoted verbatim. All 21 scenarios, 18 FRs and seven SCs have task references; the coverage review also confirmed all eight edge cases. Local links/anchors, fenced blocks, template-marker removal and whitespace passed. The installed prerequisite helper discovered `tasks.md` together with the design artifacts under the verified feature override. No product code, Cargo/native/GUI acceptance or analysis command was run by this task-generation workflow.
