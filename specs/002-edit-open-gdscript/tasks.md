---
description: "PR-sized implementation tasks for guarded open-GDScript editing"
---

# Tasks: Safely Edit Open GDScript

**Input:** [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md), [data-model.md](data-model.md), [caller contract](contracts/edit-api.md), [bridge contract](contracts/bridge-protocol.md), [native contract](contracts/native-integration.md), and [quickstart.md](quickstart.md).

**State:** Task derivation complete; **0 / 5 implementation tasks complete**. Granularity review is recorded below. Analysis and implementation have not run. Deriving tasks is not approval of the plan, a supported-version claim, or authorization to implement against an unreviewed delivery head.

**Organization:** Five coherent PR-sized increments: three verifiable foundations, one complete caller-edit slice, and one cumulative durability/compatibility increment. All five user stories remain P1. US2–US4 are inseparable safety/history requirements of US1 and are implemented and tested inside T004, not postponed to separate test-only or “add safety later” PRs. Their own phases below retain goals, ownership and independent acceptance. Each ID has exactly one checkbox; shared-story references are not duplicate tasks.

## Working Agreement and Format

- Checklist format is `- [ ] Tnnn [P?] [USn?] Description with exact file paths`. Every checkbox is **one implementation task and one PR**, not one file, symbol, layer, command or test. Indented objective/scope/constraints/evidence/acceptance text is part of that task's executable description.
- Follow [AGENTS.md](../../AGENTS.md), the [constitution](../../.specify/memory/constitution.md), reviewed spec/plan, then this task list. Before implementation: complete task-granularity review → `/speckit.analyze`, resolve blockers, obtain the normal reviewed/approved artifacts and select exactly one named dependency-ready task. Generating this list does not run those later steps.
- For each implementation task, preserve existing work, fetch updated `main`, branch `task/<task-id>-<description>`, verify `SPECIFY_FEATURE_DIRECTORY` points to this feature, implement only that task and its required tests/docs, inspect complete intended/staged changes, commit explicit paths, push and open its dedicated PR. Report it and STOP. No automatic merge/next task; stacked work requires explicit authorization. After confirmed merge, safely delete its source branch remotely and locally under repository policy.
- Mark only the selected task `[X]` when its acceptance and required evidence pass on its delivery head, independently of whether its PR is merged. `PROJECT_STATUS.md` records material lifecycle changes separately from PR metadata. Dependent implementation still waits for prerequisite PR merge unless stacking is explicitly authorized.
- **Tests are required**, not optional: spec FR-022 and constitutional mutation gates require pure decision, boundary and real-editor coverage. Write the task's meaningful regression/contract cases before its implementation, observe their initial failure where executable, then ship passing tests with the behavior. No separate test-first PR, skipped mandatory gate, source-text/wiring/mock-echo test, fabricated positive witness or refusal-only final feature.
- Existing Cargo/addon/session/observation infrastructure is already implemented. Do not reinitialize it or add services, locks/leases, journals, replay stores, generic transaction/compiler frameworks, MCP, telemetry, approval gates or new CI topology. Keep the audited Phase 1 concurrency boundary: guarded single-editor mutation, not arbitrary same-inode atomic serialization.
- Native APIs are proposed implementation work, not already-tested bindings. A research filter interruption is neither native failure nor a passing probe. No bypass/rephrased retry is authorized by task generation; unavailable mandatory future implementation evidence blocks that task's completion rather than being invented.
- Every implementation PR updates its directly affected API/run documentation plus `quickstart.md` with actual evidence and `research.md` with new dependency/support facts where applicable. In task descriptions these feature-local names mean `specs/002-edit-open-gdscript/quickstart.md` and `specs/002-edit-open-gdscript/research.md`. Implementation bookkeeping also updates this `tasks.md` and `PROJECT_STATUS.md` as required.

### Path Conventions and Shared Validation

Paths below are repository-relative. New native/test leaf paths are planned files under the approved existing component boundaries, not files created by this task-generation change. **The existing caller/worker/bridge codec is `mcp-server/src/bridge/wire.rs`; do not create a competing `mcp-server/src/wire.rs`.** Godot source paths mentioned inside `engine-api.patch` refer to the pinned external engine tree, not new top-level product components.

Use Rust 1.98.1/edition 2021 and the existing locked dependencies; C++17/public generated GDExtension C ABI; Python 3.10+; and the planned patched Godot 4.7.2 base `ed1daf0bf001b61586d9930840f2f1394092c079`. The real-editor candidate is macOS 26.6.2 arm64. Record exact patch/API/native-build IDs, compiler/SDK and exercised artifact hashes when they actually exist; do not claim stock or untested-build mutation support.

For affected Rust work, run from `mcp-server/`:

```sh
cargo +1.98.1 fmt --all -- --check
cargo +1.98.1 clippy --all-targets --locked -- -D warnings
cargo +1.98.1 test --locked
cargo +1.98.1 doc --no-deps --locked
```

Default `cargo test` includes doctests. Build actual affected callers after they exist; no root workspace, blanket `--all-features`, new dependency or MSRV promise. Pure-core tasks smoke a real library consumer; transport/process tasks exercise actual connections and child processes; native/editor changes exercise the owned visible GUI and separate D/R/B/dirty/saved-state witnesses. Headless/mocked/disk-only results cannot prove B or native history. Record ordinary applicable CI plus maintainer-operated live acceptance; protected GUI CI remains optional.

Use the planned `godot-addon/tests/run_script_edit.py` and existing `godot-addon/tests/run_observation.py`, reusing fixture lifecycle/window/witness facilities rather than copying the observation harness. At intermediate heads, `--scenario all` must explicitly fail for missing required coverage, never silently pass a subset. Native-only groups may run before a mutator exists; the runner must not require or stub an unavailable caller for those groups. Test-only preparation/fault barriers are not product operations. Run build/lint/tests after integrating a selected task's parallel work, not independently against incomplete sibling edits.

## Phase 1: Setup — Reuse the Completed Foundation

**Purpose:** Confirm the existing Rust package, observation addon, private session routing, owned-editor fixtures and shared workflows. Feature 001 is complete. There is **no separate setup checkbox**: directories, manifests, generated-ABI build handling and necessary fixture setup belong to the first capability that consumes them below. No empty crate, directory, CI, formatter or documentation-only PR.

**Entry criteria:** Reviewed Feature 002 artifacts and the later required analysis; sufficient permitted environment for the selected task's actual smoke/acceptance. No implementation is begun by this document.

## Phase 2: Foundational Capabilities

These three boundaries are independently verifiable without publishing a partial caller mutation. T001 is pure core; T002 is usable native validation; T003 is the guarded native editing/persistence boundary. Each includes its own tests, smoke and documentation. There is no product edit command until T004 has all its mandatory safeguards and evidence.

- [ ] T001 Deliver protocol-independent edit eligibility, evidence and terminal-outcome semantics in `mcp-server/src/script_edit.rs`, export through `mcp-server/src/lib.rs`, and bundle behavioral contracts in `mcp-server/tests/script_edit_contract.rs` plus evidence in `specs/002-edit-open-gdscript/quickstart.md`.
  **Objective:** Any adapter can check an observed revision basis and interpret a complete/partial/interrupted edit without JSON, sockets, Godot objects or a second safety policy.
  **Depends on:** None of this feature's implementation tasks; normal reviewed-artifact/analysis entry criteria apply.
  **Implementation:** Reuse checked selectors, identities, decimal counters, collection stamps and availability/invalidation semantics from `observation.rs`. Implement the typed request/basis, source/save/primitive evidence, stage knowledge and one reducer for all five outcomes. Extract only actually available prior observation fields; saved version is fresh preparation evidence, not invented observation-v1 data. Enforce dirty/known-stale/divergent/missing-basis refusal, exact intended-source agreement, invalidation and denial suppression. Keep schema/JSON enforcement at the later T004 adapter; quoted wire constants below remain normative but do not add a Serde/transport dependency to the core. Use owned evidence without needless copying and never manufacture observed hashes/timestamps from requests.
  **Data-model request constraints (verbatim):**

  > | Field | Rule |
  > |---|---|
  > | `schema_version`, `operation` | `1`, `edit_open_gdscript`; separate contract from observation v1. |
  > | `request_id` | Existing nonsecret RequestId format: 1–64 ASCII letters/digits/`-`/`_`, new for each attempt. No transaction UUID. |
  > | `project_root`, optional `session_id`, `script_path` | Existing explicit routing/selectors and confinement; only existing open standalone `.gd` is editable. An omitted session still requires unique source-free selection. |
  > | `expected` | Required ExpectedRevisionBasis derived from a prior complete, clean, agreeing open observation; never an authorization token. |
  > | `replacement_source` | One whole resulting source string, ≤512 KiB UTF-8. No patches/batches/force/retry options. |
  >
  > A prior complete dirty/divergent/limited observation is valid observation information but cannot supply an eligible edit basis. The CLI accepts that prior observation as input and rejects an ineligible basis; typed consumers can extract the same checked fields. Caller-supplied evidence can be forged or stale, so all meaningful facts must be freshly acquired and compared before application. No observation ID alone grants write authority.
  >
  > Supported text is exact UTF-8, including empty text, tabs, Unicode and final-newline differences, representable by the native CodeEdit operation. The initial implementation explicitly rejects NUL and carriage-return/BOM representations before mutation rather than normalize them; LF text is preserved byte-for-byte across D/R/B. Existing observation continues to preserve/read its broader encoding/line-ending evidence without adopting these mutation input restrictions. Native representation checks and post-read equality remain mandatory.
  >
  > Preparation additionally proves that the baseline and desired source are unchanged by the target's current native Save formatting profile. Otherwise refuse explicitly; do not change text or editor preferences to manufacture Save/history durability. Recheck that profile at application and verification.

  **Data-model expected-basis constraints (verbatim):**

  > | Field group | Meaning |
  > |---|---|
  > | Selected identity | Canonical project root/device/inode; exact session; script resource path; native Script, ScriptEditorBase and CodeEdit IDs; standalone disk device/inode. |
  > | Source basis | SHA-256 and UTF-8 byte length of the independently agreeing prior D/R/B source. The adapter computes it from actual prior text, not an advertised generic revision token. |
  > | Editor version | Exact prior CodeEdit current version (`B.witness.source_version`), a decimal string, plus attributable prior clean/open state. Observation v1 does not expose saved version; acquire it independently during fresh preparation and guard that prepared value afterward. Versions are local witnesses, not cross-authority clocks or a global revision. |
  > | Provenance | Prior observation request ID and interval, original identity/source witnesses. Correlation is not authority and supplies no freshness lifetime exemption. |
  >
  > Reuse Feature 001 identity/witness types and decimal-string counters. No Resource UID, mtime or source hash alone identifies the filesystem write target. Fresh project/session/document/Resource/buffer/leaf identity must match. Closure/reopening or replacement invalidates the basis even if source text matches. Dirty state is independently observed, never inferred from hashes. Known stale R/B or unavailable required observations refuse mutation. Phase 1's [concurrency boundary](spec.md#phase-1-concurrency-boundary) remains unchanged.

  **Data-model source-evidence constraints (verbatim):**

  > Reuses independent authority, availability, identity, collection stamp and invalidation meanings from observation. The mutation reducer obtains exact text independently and compares it internally. The public edit result carries digest/byte length and witnesses rather than duplicate full before/after source strings:
  >
  > `authority`, `availability`, `source_sha256`, `utf8_bytes`, `collection`, `identity`, `reason`, and optional invalidated evidence.
  >
  > Only actual observations get hashes. Unavailable is not an empty string or equality. D comes from Rust's confined reader; R/B come from separate actual editor getters. The native writer receipt is not D evidence. Denied/ambiguous targets suppress source-derived evidence and unauthorized paths. Previously valid evidence retains its original interval and invalidation reason after interruption; no replacement-document facts are mixed into it.

  **Data-model finalization evidence constraints (verbatim):**

  > `status: complete|rejected|partial_or_unknown`, reason, bound attempt/document, interval, before/after source/version/metadata observations and per-step bookkeeping completion. A rejected finalizer says nothing by itself about whether B/R/D already changed. Partial effects remain evidence; no rollback field is inferred.

  **Data-model validation evidence constraints (verbatim):**

  > `status: valid|invalid|unavailable`, reason, purpose, request/session/document binding, exact input path/hash/length, editor-clock invocation interval, dependency/context witnesses, diagnostics and completeness. Actual parser/analyzer completion determines valid/invalid; unsupported effects, incomplete or invalidated context make the result unavailable. Root and dependency diagnostics retain distinct origins and source attribution. Valid does not mean bytecode/gameplay success or immutable dependencies. See [Primitive B](contracts/native-integration.md#4-primitive-b-exact-source-native-gdscript-validation).

  **Data-model progress, required/null fields, enums and outcome constraints (verbatim):**

  > Progress is monotonic stage knowledge, not an assumption that every prior effect completed. Record separate `buffer_application`, `resource_sync`, `persistence`, `finalization`, `validation`, and `verification` states as `not_started`, `entered`, `completed`, `failed`, or `unknown`, with evidence/reason. Keep the greatest safely established application knowledge alongside any later failure.
  >
  > Public EditOutcome has these required top-level fields; nullable fields never invent unavailable facts:
  >
  > | Field | Value |
  > |---|---|
  > | `schema_version`, `operation`, `request_id` | `1`, `edit_open_gdscript`, current request. |
  > | `requested_target`, `resolved_target` | Explicit request selectors; verified resolved identity or null. |
  > | `interval`, `outcome`, `reason`, `stage` | Actual overall timing, terminal enum, cause and furthest reached stage. |
  > | `application`, `progress` | Application knowledge: `not_applied`, `applied`, `partly_applied` or `unknown`; per-stage AttemptProgress. |
  > | `expected` | Checked ExpectedRevisionBasis or null when missing/invalid/suppressed. |
  > | `before`, `after` | Nullable independently collected EditEvidence records described below. |
  > | `persistence`, `finalization`, `validation` | Nullable persistence summary/A result; validation records with their explicit purposes, empty when not invoked. |
  > | `history` | `not_participated`, `native_complex_edit` or `unknown`; participation is not proof of actual Undo/Redo acceptance. |
  > | `diagnostics`, `selection`, `safe_next_action` | Bounded cause/action records; nullable source-free disambiguation metadata; safe next action. |
  >
  > Each EditEvidence contains `document` (open/identity/validity evidence), `sources` with D/R/B SurfaceEvidence, independent `dirty`, `saved_state`, `comparisons`, `agreement`, and `consistency` (actual rechecks and detected invalidations). Unavailable records retain reasons; current and invalidated evidence stay separate. Thus dirty state and uncertainty are visible without inferring them from source equality. Denial/ambiguity suppress source-derived fields, including expected hashes, while preserving non-source application/stage knowledge if an earlier effect was already established.
  >
  > | Outcome | Required meaning |
  > |---|---|
  > | `verified_changed` | A real native source edit occurred; all fresh independent source, clean/saved-state and post-change validation conditions passed without invalidation. |
  > | `verified_unchanged` | Desired text was already current; all safety/validation/verification checks passed with no source, save, bookkeeping or history mutation. |
  > | `refused` | Proven not applied and irrevocably unable to apply later from this attempt; reason identifies input/target/conflict/capability/error/interruption. |
  > | `applied_unverified` | A source/history/persistence/bookkeeping change is known, but required later evidence failed, is missing or was invalidated. Includes partly applied cases. |
  > | `application_unknown` | A mutating command may have reached the editor but actual application cannot be determined. Missing acknowledgment is not not-applied proof. |
  >
  > Reason categories include dirty conflict, revision mismatch, identity/session change, divergence, known stale R/B, unavailable observation, closed/unsupported target, ambiguity/access denial, unsupported engine/effect/representation, persistence failure/unknown, partial finalization, parse/dependency error, validation unavailable, deadline, cancellation, disconnection and protocol failure. Keep stage and reason separate; a parse error after a write is not a clean pre-application refusal.

  **Tests and smoke:** Cover eligible/ineligible basis extraction, missing prior current version versus absent prior saved version, dirty-equal, empty/exact source and supported representation boundaries, same-text changed versions, wrong target/session/clock, each missing/invalidated authority, parse/dependency unavailable or invalid after known application, partial bookkeeping and permanent pre-boundary discard. Table-test valid evidence sequences that distinguish no authorization, authorization with lost acknowledgment, entered-but-unconfirmed and known-changed stages; denied source suppression must not erase known application. Exercise all nullable/reason/enum boundaries quoted above. Smoke a throwaway external Rust consumer invoking the public checked types/reducer; do not substitute JSON echoes or claim live-editor proof.
  **Accept:** The reusable API yields the specified five outcomes from validated evidence, never equates an acknowledgment/hash/mtime with independent verification, never downgrades known application to refusal, and never upgrades a terminal result with late evidence. Existing observation public behavior stays unchanged. Rust baselines, consumer smoke and API/evidence documentation pass in this PR. No CLI/native/source mutation or full-feature support is claimed.

- [ ] T002 Deliver confined, source-attributed native GDScript validation through `godot-addon/native/engine-api.patch`, `godot-addon/native/validation.cpp`, `godot-addon/native/extension.cpp`, `godot-addon/native/build.py` and `godot-addon/addons/godot_agent_kit/native/script_edit.gdextension`, with real-engine cases in `godot-addon/tests/run_script_edit.py` and `godot-addon/tests/fixtures/script_edit/fixture_driver.gd`.
  **Objective:** A trusted editor integration can obtain a real exact-source native parser/analyzer result, including safe dependency/effect refusals, through supported GDExtension dispatch. This is a useful non-mutating integration boundary before source application exists.
  **Depends on:** None of this feature's implementation tasks; fixed `native-integration.md`/data-model interfaces are the shared contract, not pending T001 code.
  **Implementation:** Implement Primitive B completely in a reviewable patch against the exact Godot base, exposing the existing GDScript validator rather than another parser. Patch only the necessary GDScript parser/cache/analyzer/editor binding sites; keep normal unguarded behavior intact. Add C++17 C-ABI marshaling, trusted project-directory dependency reader, actual source hash/interval/context witnesses and current-input cache attribution. Resolve nested relative dependencies at their referring script; do not trust current cached source as proof of analysis generation. Refuse before excluded ResourceLoader exists/type/load hooks, dynamic project-object access, full reload/instantiation or initialization; permit normal parser/cache bookkeeping. Do not load/open the root or assign D/R/B to validate. Implement call-local thread/context/reentrancy/invalidation guards, including any ordinary validation path directly caused by this primitive; edit-origin queued-callback lifetime integration belongs to T003 before any native edit is offered.
  **Build/lifecycle/export scope:** Introduce build setup with this first real consumer, not a separate bootstrap task. `godot-addon/native/README.md` records exact engine patch application/build instructions, generated public-ABI provenance/licenses, native build identity and actual tool/dependency review. Ignore generated headers/build output/binaries via the necessary `.gitignore` entries rather than commit transient artifacts. Keep editor-only registration and disable/exit cleanup, using `godot-addon/addons/godot_agent_kit/plugin.gd` and `export_guard.gd` where needed. No private engine layout/linking, godot-cpp/framework, native module, automatic engine download/distribution or new runtime capability. A partial native API family does not advertise edit capability. Extend existing `.github/workflows/ci.yml` path filters and an appropriate native build/patch/API check when native files are introduced; preserve full-SHA action pins, least permissions and existing trust boundaries. Compilation/headless API checks do not substitute for live acceptance.
  **Required source/result fields:** Produce the exact `ValidationResult` quoted under T001 directly from the real engine call; those fixed data-model rules do not create an implementation dependency on T001. Native contract field constraints are also carried verbatim:
  > Every result includes request/session/document binding, purpose, actual input path/hash/byte length, editor-clock start/end ticks, result/reason, dependency/context witnesses and `diagnostics_complete`. Diagnostics carry `origin: root|dependency`, confined path (or null with reason), source hash when actually read, line/column when supplied, category and message. Unknown dependency source is not assigned the root hash. Missing dependency proven by a confined read can yield a completed dependency error; an unreadable or unconfined dependency yields unavailable. Redact denied/outside-project paths; never disclose their source or raw absolute paths. Errors/messages appear only in the requested result, not incidental logs.
  >
  > Limits: root and each dependency ≤512 KiB UTF-8; at most 32 distinct source dependencies / 4 MiB dependency bytes; at most 64 error records with messages ≤2048 UTF-8 bytes and paths ≤2048 bytes. Exceeding a needed bound yields `unavailable` with limit reason and safely retained partial diagnostics, not truncated source or false valid. Warnings/functions/safe-line UI data are not a new caller diagnostics API.
  **Tests and smoke:** Build/load the real patched editor/extension and exercise valid, root-invalid, analyzer/dependency-invalid and unavailable results in an owned GUI project. Include relative/transitive dependencies, changed context/dependency, stale cache, wrong source/path/hash/session/thread, reentrancy, diagnostic/source bounds and excluded effects. Use independent sentinels and getters to prove no execution, root D/R/B/dirty/history mutation or incidental content logging; allow documented cache activity. Add a `native-validation` sub-group of the planned `native-primitives` coverage so this task runs without a mutator, never a fake caller. Run native-present enabled/disabled production export isolation and relevant existing observation non-interference checks on candidates actually supported at this head; no untested patched-editor observation compatibility claim. Record exact commands, artifacts and actual guarantees in the native README, `quickstart.md` and `research.md`.
  **Accept:** B is actually callable over supported C ABI/ClassDB, gives correctly attributed valid/invalid/unavailable results and refuses before disallowed effects or unconfined reads. Missing/mismatched APIs fail closed without breaking stock observation. Native build/CI, live primitive smoke, lifecycle/privacy/export checks and docs pass. No writer/finalizer stub, mutation advertisement, historical-probe retest claim, or caller edit success is included.

- [ ] T003 Deliver guarded native script application, bound persistence and exact-document saved-state finalization in `godot-addon/native/script_document.cpp`, extend `godot-addon/native/engine-api.patch` and `godot-addon/native/extension.cpp`, and bundle native-history/failure cases in `godot-addon/tests/run_script_edit.py` and `godot-addon/tests/fixtures/script_edit/fixture_driver.gd`.
  **Objective:** A trusted integration can apply one native text operation, persist only its validated file object, finalize the exact document and independently inspect actual saved state, without broader save/evaluation side effects.
  **Depends on:** T002. No code dependency on T001: this backend returns local guards/stage facts, never a second core `EditOutcome` classifier.
  **Implementation:** Complete the native API family with `inspect_script_document`, paired edit guard, A and its B integration. Read current/saved versions and Save profile directly; reject changed original/desired representations without changing preferences. Hold actual Script references and revalidate node identity/association, clean/open state, exact source and expected versions before relevant effects. Enter the potentially-applied boundary before any native source/history operation. Use one CodeEdit complex operation and the bound `Script.set_source_code` method, never property-driven reload, `set_text`/history clearing, `apply_code`, export/dragged-property propagation or a substitute history. Before persistence recheck D and R/B/identity, then write/truncate/flush/read back the retained project-relative existing descriptor. Bind the private receipt to this attempt and reject namespace/revision/identity loss. A performs descriptor-derived Resource/document mtime and edited bookkeeping, tags the exact saved version last, refreshes only target saved display and reports per-step partial facts. No current-tab/Save All/ResourceSaver route, synthetic save signal, scene/runtime callback, force convergence, restore or retry. Finish with actual-source B, not preflight reuse. Ensure every automatic/deferred target validation/function-discovery/application callback caused by the edit remains restricted or consumed through guard cleanup; normal independent human/unrelated behavior must remain intact.
  **Data-model attempt/lifetime constraints (verbatim; T004 later binds the actual authenticated peer to this backend):**

  > One authenticated connection/request-local in-memory record owned by the Godot integration. It retains actual object associations, the immutable intended source, fresh native guard witnesses, the pinned project/file descriptors, editor-clock expiry and current stage. It owns the engine document guard, one persistence receipt and bounded validation records. No durable operation registry, replay cache, journal or background service.
  >
  > Only one active read collection **or** edit attempt is admitted by the existing editor integration slot. Excess attempts are refused without mutation, not queued. This is capacity/lifetime management, not a concurrency lock guarantee. Human editing is not disabled; changed target evidence invalidates the attempt. Disconnect/expiry/disable releases references/descriptors and prevents new stages; already-entered I/O/mutation is not rolled back.

  **Data-model independently inspected saved-state constraints (verbatim):**

  > A separate read-only engine inspection returns document path/mtime baseline, Resource edited/mtime fields, CodeEdit current/saved versions, object association, current native save-format profile and collection interval. Compare with the independent current disk metadata and the intended source. The finalizer's own returned step flags are not this inspection. At verified success, the target is independently attributable as clean, its saved version equals its current intended buffer version, and Resource/document save metadata agrees with current target persistence evidence.

  **Data-model receipt constraints (verbatim):**

  > Native-only bounded application evidence defined in [Primitive A's contract](contracts/native-integration.md#2-common-bound-state-and-persistence-receipt). Public/bridge results contain a non-authorizing summary: stage/binding, write-start/byte/truncate/flush/readback facts, current attachment, descriptor identity/mtime, and errors/unknowns. Never serialize a reusable native handle or accept a result-shaped receipt from a caller.

  **Tests and smoke:** Extend native coverage with `native-finalization`; the aggregate `native-primitives` must now execute both sub-groups. Exercise actual non-current-tab edits with unrelated dirty work, current/saved-version changes including equal text, wrong/failed/unknown receipts, Resource/editor/buffer replacement/closure, lost namespace and actual write/truncate/flush failure boundaries. Label injected status separately from real I/O failure. Read saved metadata independently of A's return; capture actual owned-window buffer/dirty state. Prove native apply → one Undo → ordinary Save → Redo → Save, earlier history reachability and no entries for rejected/unchanged native attempts. Include no delayed effect after guard close/disable, unchanged human behavior, exact empty/Unicode/size/Save-profile cases, close/reopen and representative reparse/rescan/runtime source durability. Check native artifact export/privacy/lifecycle after these changes and affected CI. Do not claim arbitrary same-inode exclusion; known invalidation still prevents a positive local claim.
  **Accept:** The native boundary performs real target-bound writes and complete target-local bookkeeping without lost human work/history or redirected persistence. Failure leaves actual known effects observable and never falsely tags newer text saved. B's policy cannot be bypassed by edit-generated callbacks. Real primitive/history/durability regressions and independent inspection pass on recorded builds; native API/build documentation and feature evidence ship together. This is a native backend, not a public caller mutation or full feature acceptance.

**Foundation checkpoint:** T001 supplies real core semantics; T002 supplies real read-only native validation; T003 supplies real guarded native editing. No task exists merely to create a struct, file, build script or test. The native patch/build/library setup is bundled with its first consumer. Normal delivery order is T001 → T002 → T003; only T002 → T003 is a foundation code dependency.

## Phase 3: User Story 1 — Make One Coherent, Persisted Edit (P1)

**Goal:** A real local caller edits one clean open target, including a non-selected tab with unrelated dirty work, or reports independently verified unchanged state without mutation. **Mandatory shared scope:** all US2–US4 preservation, uncertainty and native-history behavior is part of this task, not future hardening.

**Independent test:** Run `clean-open` through the actual caller with separate D/R/B, dirty, saved-state and post-change parse witnesses. The result must distinguish verified changed/unchanged and preserve unrelated state, focus and history. Execute the shared US2–US4 groups in the same PR before completion.

- [ ] T004 [US1] Deliver the complete guarded caller edit and inseparable US2–US4 behavior in `mcp-server/src/bin/edit-gdscript.rs`, `mcp-server/src/runner.rs`, `mcp-server/src/bridge/wire.rs`, `godot-addon/addons/godot_agent_kit/script_edit.gd`, `godot-addon/addons/godot_agent_kit/bridge.gd` and `godot-addon/addons/godot_agent_kit/plugin.gd`, with caller/bridge tests and real-editor story acceptance in `godot-addon/tests/run_script_edit.py`.
  **Objective:** Connect the completed native boundary to the one reusable core, with exact authenticated routing, full revision/dirty protection, immutable intent, bounded execution, explicit partial/unknown outcomes and independent verification from the first exposed edit path.
  **Depends on:** T001 and T003 (therefore T002).
  **Implementation:** Add the thin binary/Cargo target, typed public edit-v1 adapter and all required JSON shapes, reusing core types from T001. Implement exactly the stdin payload, selectors, exit codes and no-source-in-argv/log behavior in `edit-api.md`. Extend existing `runner.rs` supervision/IPC: the parent records `may_apply` before one worker authorization; the worker cannot send apply earlier. Reuse `target.rs` and `project_fs.rs` for fresh source-free selected-session identity and independent D/read-recheck. Never reuse a prior bound observation as live mutation authority. Migrate all actual v1 codecs/peers/capabilities/transcripts together in `bridge.rs`, `bridge/wire.rs`, addon bridge/plugin and existing boundary fixtures; there is no dual protocol or downgrade fallback. Implement every prepare/apply/progress/verify/recheck/finish/abort state and malformed/order/duplicate/late/cancel/disconnect path, not just the successful route. Advertise edit only for the installed complete matched native family and handlers. Register/clean up the thin native-attempt integration, preserve observation's getter-only semantics, and separately inspect R/B/dirty/saved state rather than reuse application reports.
  **Detailed paths:** Reuse/update `mcp-server/Cargo.toml`, `mcp-server/src/lib.rs`, `mcp-server/src/bridge.rs`, `mcp-server/src/target.rs`, `mcp-server/src/project_fs.rs`, `mcp-server/tests/script_edit_contract.rs`, `mcp-server/tests/bridge_boundary.rs`, `mcp-server/tests/confinement.rs`; add `mcp-server/tests/script_edit_caller.rs`. Extend `godot-addon/tests/fixtures/script_edit/fixture_driver.gd`, the real runner, and existing `godot-addon/tests/run_observation.py`, `fixture_bridge.gd` and `fixture_collector.gd` as required by v2. Add no duplicate wire module, generic runner, policy fork or standalone history/save command. Extend existing `ci.yml` to build the new binary and exercise new boundary tests; changes to optional `live-editor.yml` must retain its existing trust gates and associated `.github/tests/test_live_editor_gate.py` / `test_live_editor_execution.py` behavior, not make a GUI runner mandatory.
  **Inherited field constraints:** Enforce every exact request/basis/result/source/enum/null/reason constraint quoted under T001, and consume T002/T003 evidence without inventing native receipts or saved-version history. The adapter must reject duplicate/unknown shape fields, wrong versions/types/counters/identities and unbounded payloads before treating them as evidence. Context/binding failures preserve safely acquired prior facts and application knowledge while suppressing denied source.
  **Data-model deadline constraints (verbatim):**
  > The edit supervisor uses **9.5 seconds** for acquisition/operation evidence and reserves **0.5 seconds** for bounded result delivery in the controlled consuming caller. Reuse the existing same-binary supervised worker and nonblocking reap strategy; do not kill the user's editor or wait indefinitely for blocked filesystem/editor work. Observation retains its existing 4.5/5-second behavior.
  >
  > The editor establishes a local monotonic expiry from the remaining budget supplied during preparation (capped at 9000 ms), checks it before every mutating stage and after blocking work, and refuses new stages after expiry. Clock domains are not compared directly. A transport delay or already-entered native call can outlive the caller; report uncertainty, not a false global cancellation guarantee.
  >
  > Cancellation/EOF/protocol failure/deadline is terminal for the caller and prevents success. After authorization, only an attributable irreversible pre-boundary discard can prove not applied. An `abort` attempt without that acknowledgment is not proof. Once an effectful stage has entered, cancellation cannot promise that the stage has not or will not finish; stop subsequent stages when control returns. Native guard cleanup does not undo effects.
  **Additional adapter limits (verbatim from the caller/bridge contracts):**
  > Reuse lexical project/resource/registry confinement and privacy rules from [observation v1](../001-observe-gdscript-state/contracts/observation-api.md). Project root ≤1024 UTF-8 bytes, resource path ≤2048; counters/instance IDs are checked decimal strings. Decode schema/field types and reject duplicate or unknown required-shape fields and trailing JSON values. Cap stdin at 12 MiB before unbounded allocation; source size is checked after decoding, ≤512 KiB UTF-8. Selection, blocking input and all worker work are inside the supervised deadline.
  >
  > `capabilities` contains the existing five booleans in the same order, then `edit_open_gdscript`.
  >
  > Add `native_api_revision`, integer `0` or `1`, and `native_build_id`, empty when unavailable or a 64-lowercase-hex identifier from the matched engine/native build manifest.
  **Bridge bounds:** Four-byte big-endian length; 4 KiB descriptor/unauthenticated-control bound, 4 MiB selected request bound, 12 MiB response/worker bound, nesting ≤32, inherited 32 peers/descriptors, one active collection/edit slot and 64 KiB/1 ms incremental network work. Follow the exact authenticated field order/domain/roles in `bridge-protocol.md`; request IDs stay ≤64 ASCII, roots ≤1024 and paths ≤2048 UTF-8 bytes, and hashes/decimal IDs must not pass through floating-point conversion. Preparation's `remaining_budget_ms` is integer 1–9000; no caller-supplied saved version. These constraints apply before allocation/acceptance, not after a source-bearing malformed frame has been trusted.
  **Ordered behavior:** Acceptance/selection → fresh independent preparation and proposed-source B for changed intent → parent authorization → fresh native guards → buffer application/R synchronization → bound persistence → A → fresh actual-source B → independent D/R/B/dirty/saved-state and context rechecks → one core terminal outcome. Unchanged intent skips preflight duplication and authorization/application/writer/finalizer/history; it still validates and independently verifies. Read-only verification may preserve evidence after an unsuccessful applied stage when available within the deadline; no failure authorizes another effect or repair.
  **Tests and smoke:** Implement and pass the full `clean-open`, `conflicts`, `routing`, `interruption`, `validation` and `history` groups, plus `native-primitives`, in this same PR. This covers all **21 US1–US4 scenarios**, not a clean-path-only subset. Include missing/dirty/stale/known-stale/divergent/oversized/unrepresentable basis, same-text changed versions, unsupported/closed/unknown/denied targets, wrong/multiple/ended sessions and unresponsive editor. Use real barriers for before/after authorization, application, persistence, each finalizer step and verification; preserve newer human text and distinguish actual changes from unknowns. Release controlled pending work after a proven refusal to prove no late apply. Verify one native Undo/Save/Redo/Save and earlier history with no entries for refused/unchanged. Repair coherent syntax-invalid original source, preserve exact supported empty/Unicode text and test meaningful post-change parse/context failure, not merely preflight rejection. Exercise actual process/stdin/network limits and HMAC replay/reflection/build-field changes; simulations cannot establish R/B. Smoke real changed and unchanged CLI calls on the patched editor, with independent screenshots/state witnesses and every measured controlled edit ≤10 seconds. Also pass affected Save/reopen/reparse/rescan/runtime, representative repeat-edit, observation/privacy/export regressions before exposing the path; a representative sequence does not claim full SC-007/E acceptance.
  **Accept:** All mandatory protections, stage/application certainty, history and independent-verification behavior work end to end from the first public edit. All US1–US4 groups and directly affected existing checks pass; stdout/exit codes and redacted stderr match the caller contract. No typed stub, native-response echo as oracle, unconditional unsupported implementation or knowingly failing safety case. Direct implementation tests/docs and exact evidence ship here. T005 owns cumulative whole-feature stress/compatibility evidence, not missing safety, routine tests or unfinished US2–US4 behavior; no full-feature/release support claim yet.

## Phase 4: User Story 2 — Preserve Human Work and Reject Stale Intent (P1)

**Goal:** Dirty/stale/divergent/unobservable, changed-target and unauthorized attempts preserve human source, dirty state and native history and never guess/open/repair a target.

**Implementation ownership:** **T004 [US1]**, using T001–T003's completed core/native boundaries. No additional checkbox: delaying these safeguards or splitting their natural integration tests into another PR would violate the spec's inseparable-safety rule and the repository's task-granularity policy.

**Independent acceptance:** Run `conflicts` and `routing` for **US2.1–US2.7**. Bracket dirty-different/equal, changed same-text version, stale clean basis, known stale R/B, missing observations, between-check/application edits, session ambiguity/restart and unsupported/denied targets with actual source/identity/dirty/history witnesses. Every unsafe attempt has zero operation-caused source/history change and a distinct reason; known post-boundary changes use the US3 outcome path instead of false refusal. These groups must pass before T004 is complete.

## Phase 5: User Story 3 — Understand Interrupted or Unverified Changes (P1)

**Goal:** Stage, application certainty and unavailable/invalidated facts remain truthful through cancellation, timeout, disconnection, persistence/finalization/parse failure and newer human work.

**Implementation ownership:** **T004 [US1]**, with T001's reducer and T002/T003 stage evidence. No second error-policy or future “add timeout safety” task.

**Independent acceptance:** Run `interruption` and `validation` for **US3.1–US3.6**. Verify no-authorize/no-late-apply, authorize/lost-ack uncertainty, known partial writes/bookkeeping, invalid/unavailable post-change parse, preserved newer human text and fresh-observe/new-basis guidance. Native application acknowledgment never passes success; an entered stage is not automatically a completed change. Every controlled edit including stalls returns within ten seconds. These groups must pass before T004 is complete.

## Phase 6: User Story 4 — Reverse and Reapply Through Native History (P1)

**Goal:** An agent change is one actual native reversible operation without erasing earlier valid history; refused/unchanged attempts add no entries.

**Implementation ownership:** **T003** implements/proves the native boundary; **T004 [US1]** integrates/proves caller behavior. No separate history stack or caller Undo/Redo operation and no test-only PR after exposing the edit.

**Independent acceptance:** Run `history` for **US4.1–US4.4** through the actual caller. One Undo restores complete prior B; observe actual R/D/dirty, then ordinary Save converges D/R/B. Redo must survive that Save and restore intended source; Save converges again. Earlier entries remain reachable in order; normal disposal of an existing redo branch for a new edit is permitted. Both refused and unchanged cases preserve history. This must pass before T004 is complete.

## Phase 7: User Story 5 — Keep Successful Changes Through Normal Editor Use (P1)

**Goal:** Prove cumulative durability, repeated safe use, unchanged observation semantics and privacy/export isolation across the completed caller/native integration.

**Independent test:** Run the full planned edit suite and existing observation suite on the recorded candidate; independently witness normal editor actions, ≥20 successful fresh-basis edits with Saves and ≥3 interleaved dirty/stale refusals, plus a runtime behavior marker and real production exports.

- [ ] T005 [US5] Establish cumulative script-edit durability and verified compatibility in `godot-addon/tests/run_script_edit.py`, `godot-addon/tests/fixtures/script_edit/fixture_driver.gd`, `godot-addon/tests/run_observation.py`, existing `.github/workflows/ci.yml` and evidence in `specs/002-edit-open-gdscript/quickstart.md` / `research.md`.
  **Objective:** Deliver the whole-feature interaction/support gate, not deferred routine story tests or missing implementation from T004.
  **Depends on:** T004 (and all its prerequisites).
  **Implementation:** Complete cumulative `durability`, `sequential` and whole-run `privacy-export` orchestration and artifact/result-only review. Reuse the existing fixture project/export preset facilities; add the scoped script-edit `project.godot`, `main.tscn`, `scripts/subject.gd`, `scripts/other.gd`, `scripts/main.gd` and `export_presets.cfg` under `godot-addon/tests/fixtures/script_edit/` only where the earlier tasks have not already supplied them. Use a real fixture runtime result that identifies the edited revision. Preserve all existing observation groups and count actual cases, not historical 194-case evidence as a new result. Maintain ordinary native/workflow CI for the exact exercised head/build and necessary path filters; optional `live-editor.yml` remains the same trusted GUI boundary, not a new completion prerequisite. Resolve actual discovered cross-boundary defects with their reproducing regressions in the owning existing code; a missing T004 prerequisite blocks this task rather than being silently rescheduled here.
  **Tests and smoke:** Pass **US5.1–US5.5**, every quickstart group and all 26 scenarios/22 requirements/eight success criteria on the current delivery head. Execute Save → close/reopen and completed reparse/rescan events; launch at least one permitted runtime fixture and observe changed behavior, not just exit zero. Perform at least 20 distinct successful fresh-basis edits with intervening Saves and at least three interleaved dirty/stale refusals, independently checking each transition, preserved human work and usable native history, then final close/reopen. Run the complete existing observation acceptance including twenty-read non-interference and its ≤5-second limit. Review all outcomes from their result fields alone, including safe next action and uncertainty; every controlled edit remains ≤10 seconds. Run full redaction and actual enabled/disabled/hook-only export variants with installed native artifact, inspect PCK and bundled files and execute the exported game to prove no active tooling/native registration/listener/credential behavior or patched-editor runtime dependency survives. No hidden skips, passing partial `all`, headless substitute for B, missing-runtime waiver or neighboring-version support claim.
  **Accept:** All A–E and applicable Save/reopen/reparse/rescan/runtime durability, observation, privacy/export, timing and appropriate CI gates pass with exact identities and independent evidence. Consolidate verified guarantees/limitations in the existing feature contracts/quickstart/research and record constitutional/ownership/dependency review. If this satisfies all feature criteria, mark Feature 002 complete even with its task PR still open; assess Roadmap Phase 1's separate exit gates without assuming completion. Leave missing acceptance pending, never “pending merge.” Deliver only this task's PR and stop.

## Phase 8: Final Polish and Cross-Cutting Checkpoint

**Owner:** T005; no extra checkbox or status-only/test-only/docs-only PR. Every earlier task already carries its required tests/docs, privacy/export and smoke obligations. This final checkpoint reconciles full-suite evidence, actual supported-build claims, field/compatibility documentation, constitutional compliance and removal of owned test artifacts. It is not a place to add missing guards, broaden scope, ignore failing prerequisites or turn an unavailable gate into a pass.

## Dependencies and Execution Order

```text
T001  Core eligibility/evidence/outcomes ----------------------+
                                                             |
T002  Native source validation -> T003 Native guarded edit ----+-> T004 Complete caller edit -> T005 Cumulative acceptance
```

- All tasks additionally require reviewed artifacts and the normal analysis gate before implementation. Feature 001 is satisfied; its tasks are not reopened.
- T001 and T002 have no mutual code dependency; T003 depends on T002; T004 depends on T001/T003; T005 depends on T004. Task IDs follow the normal serial delivery order.
- **First useful caller slice: four PRs**, T001–T004, with only three meaningful pre-caller boundaries. There is no independent setup PR and no per-file/enum/test packaging. The native validation and guarded-mutation boundaries are separate because one is a usable non-mutating language service inside the editor and the other introduces filesystem/history effects requiring its own review and real-editor evidence; they are not arbitrary source-file divisions.
- Story completion order is **US1 + US2 + US3 + US4 together at T004**, then cumulative **US5 at T005**. Each story's scenarios can be independently selected/run; that does not make its safety optional or justify a separate incomplete mutation path.
- Default delivery is serial, one task and one PR at a time with the post-PR STOP. Semantic independence is not stacked/concurrent implementation authorization.

### Task Count and Story Ownership

| Phase/story | Dedicated checkboxes | Required implementation ownership |
|---|---|---|
| Setup | 0 | Existing infrastructure; setup bundled with first consumer. |
| Foundation | 3 | T001, T002, T003. |
| US1 | 1 | T004; consumes T001–T003. |
| US2 | 0 additional | T004 includes all seven scenarios; T001–T003 supply its primitives. |
| US3 | 0 additional | T004 includes all six scenarios; T001–T003 supply its primitives. |
| US4 | 0 additional | T003 native history and T004's complete caller history scenarios. |
| US5 | 1 | T005 cumulative proof; T004 already implements durability-safe behavior. |
| Final cross-cutting | 0 additional | T005; routine tests/docs are bundled earlier. |
| **Total** | **5** | Five future implementation PRs; none completed here. |

## Parallel Execution Examples per Story

**No whole-task `[P]` markers.** T001/T002 are semantically independent, but the implementation tasks intentionally share `quickstart.md`, `research.md`, task/status bookkeeping and, later, native/harness integration paths. Strict different-file parallelism therefore is not claimed. Default one-task/one-PR delivery remains serial. The examples below are **sub-work within one selected task** after its interfaces/prerequisites are fixed, not extra Spec Kit tasks or permission to start another PR. One integration owner collects all results before integration/build/lint/tests; run shared GUI scenarios serially on owned fixtures to avoid focus/process interference.

| Story | Permitted within-task parallel example | Integration boundary |
|---|---|---|
| US1 / T004 | One worker implements Rust caller/worker/codec and its Rust tests; another implements the thin Godot bridge/native-attempt adapter and its GDScript fixture actions. | Freeze the existing v2/native contract first; one owner integrates and runs the real caller. |
| US2 / T004 | A Rust worker covers basis/refusal/selection boundaries in `script_edit_caller.rs`/`confinement.rs`; a Godot worker implements distinct human/identity/dirty fixture preparations in `fixture_driver.gd`. | Same production contract; no shared-file concurrent writes or live scenario execution before integration. |
| US3 / T004 | A Rust worker implements authorization/worker-loss cases in `runner.rs`/`script_edit_caller.rs`; a Godot worker supplies controlled native-stage stall/disconnect/partial-state fixture actions. | Parent owns final certainty reducer mapping and runs the combined interruption matrix after both finish. |
| US4 / T003, then T004 | Native implementation and independent fixture-history preparation may be authored separately once exact API calls are fixed. | Do not run T003 and T004 concurrently by this example; actual Undo/Save/Redo is verified only on integrated code. |
| US5 / T005 | One worker implements cumulative edit sequences in `run_script_edit.py`; another reviews/updates existing workflow checks or observation regression handling in different files. | One owner combines artifact/result review and runs the complete edit/observation/export suites once integration is ready. |

Examples for different stories that name the same files are alternatives within the selected task, not simultaneously independent work. Native/API investigation and pure-core consumer design can also be read-only parallel preparation without bypassing delivery gates. Do not invent `[P]` markers or independent tests-only PRs just to satisfy template examples.

## Requirement and Acceptance Coverage

| Acceptance | Owning task(s) | Required proof |
|---|---|---|
| US1.1–US1.4; SC-001 | T001–T003 primitives; T004 caller acceptance | Clean/non-selected/unrelated-dirty/unchanged; independent intended D/R/B/dirty/saved/parse. |
| US2.1–US2.7; SC-002 | T004, with primitive refusals already in T001–T003 | All dirty/stale/missing/target/session/access/boundary-change cases preserve source/history. |
| US3.1–US3.6; SC-003–SC-004 | T001 semantics, T002/T003 local facts, T004 actual process/editor integration | No false success/not-applied/rollback, accurate parse/partial evidence, no late apply, ≤10 seconds. |
| US4.1–US4.4; SC-005 | T003 native; T004 real caller | One Undo, Save, Redo, Save and earlier history; refused/unchanged add no entries. |
| US5.1–US5.5; SC-006–SC-008 | T004 affected-boundary regressions; T005 cumulative completion | Full durability/runtime, ≥20 successes/≥3 refusals, unchanged observation and privacy/export. |
| FR-001–FR-004 | T001 model/basis, T003 native guards, T004 end-to-end | Exact target, immutable intent, independent clean source and fresh boundary checks. |
| FR-005–FR-009 | T002 validation, T003 native edit/finalizer, T004 end-to-end | Unchanged refusal, native history, distinct stages, independent success and actual post-change parse. |
| FR-010–FR-014 | T001 all outcome/evidence rules; T004 actual supervision/transport | Required fields/reasons/partial state, fixed deadlines, preserved new work and no retries. |
| FR-015–FR-018 | T003/T004 native/caller behavior; T005 cumulative durability | Real history, exact Save/reopen persistence, repeated safety and truly unchanged requests. |
| FR-019–FR-021 | Every introducing boundary; T004 cutover, T005 cumulative regression | Observation v1 preserved, local confinement/privacy, core/native ownership and actual export isolation. |
| FR-022 and A–E | T002/T003 primitive evidence, T004 affected real-editor gates, T005 complete acceptance | Pure/boundary/real GUI evidence on exact builds; no release/full-feature claim before all pass. |

All eight spec edge cases are covered: representation/empty/limits (T001–T004); rename/delete/replace/close/reopen and missing authorities (T003/T004); unwritable/failed persistence (T003/T004); invalid-source repair (T002/T004); restart after interruption and changing actor/history (T003/T004); cumulative persistence and ordinary usage (T005). The full quickstart remains normative; this table cannot narrow a required scenario or gate.

## Implementation Strategy

1. Deliver T001's reusable core and T002's real native validator separately with their own smoke/regressions/docs; do not add a public edit flag or empty infrastructure.
2. Deliver T003's complete native mutation/finalization boundary, including local refusal/partial/history/durability proof, before any caller can use it.
3. Deliver T004 as the **first useful caller slice**: US1 plus all inseparable US2–US4 protections and tests. This is the requested first-slice/MVP recommendation, not permission to ship “US1 only” without dirty/stale/uncertainty/history safety. Full feature/release acceptance remains pending.
4. Deliver T005's cumulative US5 gate and actual compatibility/evidence baseline. A missing prerequisite is a blocker, not a reason to move unfinished safety into this task.
5. Stop after every task PR as required; do not merge or auto-advance. Feature completion follows acceptance, not GitHub state. Roadmap Phase 1 remains separately assessed.

## Granularity Review

**Review state: PASS (2026-09-27).** Parent review and independent native-boundary/granularity and core/coverage reviews found no unresolved task-boundary or acceptance blocker. T002's non-mutating validation and T003's effectful editing boundary are independently useful/verifiable; T004 bundles every inseparable US1–US4 obligation; T005 is cumulative acceptance, not deferred safety. The four-PR path to the first caller is justified by three actual architectural outcomes, not file/test fragments. This is an artifact/granularity review, not `/speckit.analyze`, implementation approval or runtime acceptance.

| Review criterion | Decomposition decision |
|---|---|
| Meaningful PR-sized increments | Core safety semantics; native validation; native guarded editing; complete caller edit; cumulative feature acceptance. |
| No trivial file/symbol/setup tasks | No standalone crate/directory/schema/enum/build-script/logging/formatting tasks; setup travels with its actual consumer. |
| Tests/docs bundled | Every task owns its required tests, smoke and docs. US2–US4 are not delayed tests-only/safety-only PRs. |
| Reasonable serial path | Three independently useful/verifiable boundaries precede the first caller slice; four PRs to T004, five total. |
| Preserved coverage | All 22 FRs, 26 scenarios, eight SCs, eight edge cases, A–E and durability/observation/privacy/export gates have explicit owners. |
| Clear dependencies | T002 → T003; T001 + T003 → T004 → T005. No cyclic or hidden story dependency; shared safety is explicit. |
| Complexity and ownership | No architecture change/new framework; existing policy stays in Rust, native APIs stay in the addon, and current build/ABI costs remain those justified by the plan. |

The generic template's separate tests/models/endpoints/setup examples were not copied into separate PR tasks: repository granularity and the approved mutation safety boundary require coherent increments. Story phases with shared ownership are deliberate, not missing work. A materially different native integration or weakened acceptance discovered during implementation requires review of the affected approved artifacts rather than silently changing this list.
