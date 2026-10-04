# Tasks: Use the Trusted GDScript Workflow Through MCP

**Input:** Design documents in `specs/007-mcp-script-workflow/`: [specification](spec.md), [plan](plan.md), [research](research.md), [data model](data-model.md), [MCP contract](contracts/mcp-interface.md), [execution contract](contracts/execution.md), [closed-edit contract](contracts/closed-edit.md) and [validation guide](quickstart.md).

**Status:** Four approved implementation tasks following the renewed granularity review and [passed consistency analysis](analysis.md). The implementation-prerequisite check found exact matching fingerprints and no semantic design drift. Only T001 is selected and in progress on `task/T001-trusted-script-execution`; integrated verification must finish before its checkbox is marked complete. T002–T004 remain pending.

**Prerequisites:** Review the selected technical plan, complete `/speckit.analyze` after granularity review, and satisfy the repository's implementation prerequisites before selecting exactly one dependency-ready task. Verify the active directory as `specs/007-mcp-script-workflow`; task branch names do not select it.

**Tests:** Explicitly required by FR-019–FR-020, SC-001–SC-009 and constitutional mutation gates. Direct deterministic, boundary, native, real-editor and positive real-agent evidence belongs to the capability introducing it. T003 has one adversarial/effect-safety purpose across preservation and interruption; T004 owns the new composed A–E interaction and cumulative evidence. Separately runnable scenario groups are not separate Spec Kit tasks.

**Organization:** All five stories remain P1, but tasks follow coherent implementation boundaries rather than one-checkbox-per-story symmetry: T001 is protocol-independent; T002 covers US1 and US2; T003 covers US3 and US4; T004 covers US5 and cumulative obligations. Setup and final cross-cutting headings contain no artificial tasks. Exactly one task per PR; finish, publish and stop. No automatic merge or dependent work without merge or explicitly authorized stacking.

## Format and path conventions

Each implementation item uses `- [ ] Tnnn [P?] [USn? ...] Description with exact paths`; multiple story labels identify shared ownership. Indented paragraphs, quoted constraints and acceptance clauses are part of that task's binding description. There are no task-level `[P]` markers: these increments depend on shared integration and valid prior evidence. Intra-task parallel examples do not authorize batching task PRs.

Paths are repository-root-relative. New paths below are planned files, not assertions that they exist. Use the existing one-package Rust layout, thin addon/native boundary, Python acceptance conventions and dedicated VM. No root Cargo workspace, new crate, generic framework, CI topology or additional approval gate is selected. Read existing implementations and applicable guidance before editing; use LSP references before exported-symbol changes.

## Phase 1: Setup (existing project; bundled initialization)

**Goal:** Reuse the completed package, authenticated routing, confined execution, native integration, fixtures and VM instead of another foundation.

No separate setup task. T001 includes only the necessary native/bridge/worker integration and T002 includes dependency resolution, binary wiring and fixed client-test setup. Existing Features 001–005 and Phases 1/2 remain complete. Native/SDK setup without a working consumer is not an independently useful PR here.

**Checkpoint:** Existing project and selected design provide setup; implementation still requires the prerequisites above. No scaffold, dependency-only or documentation-only implementation task is created.

## Phase 2: Foundational (complete trusted read/closed-edit capability)

**Goal:** Deliver the reusable capability the three-tool adapter needs, with enforced safety and positive real-editor proof before any MCP mutation is exposed.

**Independent test:** A bounded supervised Rust caller in the owned VM reads eligible open/closed state, compares a fresh revision and edits either lifecycle without changing it. Cached and absent R closed positives, meaningful refusals, effect-sensitive faults and later durability are independently witnessed. This is a complete core/integration outcome, not an unused schema or native-only layer.

- [ ] T001 Implement trusted revision-based read/edit execution and the complete closed-source transaction in `mcp-server/src/script_read.rs`, `mcp-server/src/runner/read.rs`, `mcp-server/src/runner/read/`, `mcp-server/src/script_closed_edit.rs`, `mcp-server/src/script_closed_edit/`, `mcp-server/src/runner/closed_edit.rs`, `mcp-server/src/runner/closed_edit/`, `mcp-server/src/bridge/wire/`, `godot-addon/addons/godot_agent_kit/script_closed_edit.gd`, `godot-addon/addons/godot_agent_kit/script_closed_edit_transport.gd` and `godot-addon/native/script_closed_edit.cpp`; bundle deterministic/boundary coverage in `mcp-server/tests/script_read.rs` and `mcp-server/tests/script_closed_edit.rs`, native/closed VM evidence in `godot-addon/tests/run_mcp.py` and `godot-addon/tests/closed_script_acceptance.py`, and the implementation/evidence updates in `specs/007-mcp-script-workflow/quickstart.md` and `godot-addon/native/README.md`.

  **Dependencies:** Existing approved design and implementation prerequisites only. No MCP SDK or transport dependency. This task supplies the complete supervised read/revision/closed-edit API that T002 consumes; retain the existing open-only edit caller and public local v1 meanings.

  **Implementation, in responsibility order:**

  1. Add checked capture/revision/request/outcome models, then reuse ordinary observation acquisition and its original clock for the private closed supplement. Project exact source, revision and useful state without exposing an observation envelope or duplicating agreeing source. Use existing `ring` SHA-256, fixed domain/variant/field tags, ordered length-delimited checked values and existing independent source digests/lengths. Bind all stable open expected-state facts and closed file/Resource/epoch facts, including file mtime/ctime. Apply the model's exact exclusion: “Exclude request IDs, collection stamps, observation timestamps and incidental JSON ordering: separate complete captures of unchanged state must compare equal.” No token store, signing key, per-client state or generic canonicalization layer.
  2. Freshly authenticate, resolve and acquire for every edit, compare the opaque revision, freeze that exact matching capture and only then construct the open or closed internal request. Open uses `ExpectedRevisionBasis::from_observation`, existing `EditRequest`/`runner::edit::run`, and bounded private worker input from the fresh capture with its own observation ID distinct from the edit ID. Never accept a client observation, reacquire into a refreshed basis, switch lifecycle branches or reset the edit clock after comparison.
  3. Implement the closed native family under the existing main-thread admission/owner. Own and clean up the public `script_close` callback and session epoch; passive complete roster/cache getters establish actual absence, not failed getters. Retain existing clean R strongly only for an attempt; absent R stays absent. Reuse exact-source/effect/compiled-context admission, Save-format checks, the stock validator, confined retained descriptors and the parent-before-worker irreversible authorization boundary.
  4. Follow `contracts/closed-edit.md` exactly: prepare and prevalidate original/desired sources; repeat immediate guards; mark effects before entry; explicit `Script.set_source_code` only for present R; recheck; same-fd `pwrite`/`ftruncate`/`fsync`/`pread`; same-fd original-mtime restoration preserving atime and new ctime; recheck identity/namespace/R/absence/epoch. No `_script_source`, forced load, ResourceSaver, reload, dirty clearing, saved signal, buffer/history operation, deferred mutation or external production writer. Already-equal intent validates and independently verifies without setter/write/timestamp/history effects.
  5. Independently reacquire actual D, applicable R and continued B absence plus source/context validation before success. Receipts and desired text are not witnesses. Implement sticky known/partial/unknown effects, disclosure denial after earlier failure, expiry/cancel/disable ownership, no late execution after proven refusal and no rollback/reassertion/compensating lifecycle action. Preserve the original cutoff and maximum nine-second effect lease.
  6. Cut over all current Rust/addon/native/fixture consumers together to private bridge v6/native family revision 4, including `mcp-server/src/bridge.rs`, `mcp-server/src/bridge/wire.rs`, `mcp-server/src/runner.rs`, existing `mcp-server/src/bin/` worker dispatch, `mcp-server/src/lib.rs`, `godot-addon/addons/godot_agent_kit/bridge.gd`, `godot-addon/addons/godot_agent_kit/plugin.gd`, `godot-addon/native/native.hpp`, `godot-addon/native/session.cpp`, `godot-addon/native/extension.cpp` and `godot-addon/native/build.py`. Reuse/narrowly extract only actually shared primitives in `godot-addon/native/document_guard.cpp`, `godot-addon/native/document_guard.hpp`, `godot-addon/native/editor_context.cpp` and `godot-addon/native/editor_context.hpp`. Update cross-language vectors, native manifest/build provenance and matched-install guidance. Advertise `edit_closed_gdscript` only when the complete family is present; no v5/revision-3 fallback or alias. Ordinary read/discovery remains available without native mutation support. Do not commit transient binaries merely because a build produced them.
  7. Add the fixed `mcp` suite and individually runnable `closed-native`/`closed-lifecycle` groups through `godot-addon/tests/run_in_vm.py` and `godot-addon/tests/vm_guest.py`, using a fixture-only supervised consumer before the product MCP binary exists. Put owned witnesses in `godot-addon/tests/fixtures/mcp/`; identify pre-adapter evidence as core/native, not MCP or real-client acceptance. Extend the groups through actual MCP in T002. Test changed registration/ownership semantics in `godot-addon/tests/test_run_in_vm.py` and `godot-addon/tests/test_vm_guest.py` without historical GUI replay.

  **Binding model bounds (verbatim from data-model.md §2):**

  | Item | Required rule |
  | --- | --- |
  | Environment | Exact supported official Godot 4.7.2/macOS 26.6.2 arm64 profile; no wider support implied. |
  | Selectors | Existing project root ≤1024 UTF-8 bytes; exact resource locator ≤2048 bytes; explicit session, when supplied, is 32 lowercase hex. No focus/cwd/root-list targeting or normalization of resource names. |
  | Domain correlation | New generated non-secret checked request ID for each intentional call; existing maximum 64 safe ASCII characters. MCP request IDs remain separate. |
  | Source | Existing 512 KiB per independent authority and replacement, exact UTF-8; edit replacement admits LF and rejects NUL/CR/BOM without normalization. Read keeps existing empty/unavailable/invalidated semantics. |
  | Discovery | Existing bounds: 1024 entries/directories, work 16384, depth 64, batch 64, 6 MiB result. Limits remain explicit coverage, not false absence. |
  | Closed edit | Existing external standalone `.gd` source profile; no scene/built-in mutation, tool/custom Script, global registration, load/preload or unsafe effective/compiled context. Both original and desired sources must pass admission. |
  | Closed metadata | Bounded scalar file/Resource/epoch evidence; no source duplicate, document-history list or persistent journal. Existing bounded roster getter failure refuses safety admission. |
  | Time | Read/discover: 5 s end-to-end, with the existing 4.5 s work cutoff; edit: 10 s with 9.5 s work cutoff. Initial handshake and partial-frame completion: 10 s. No idle timeout between complete requests. |

  The selector/source/discovery/time rules also bind T002 and every subsequent MCP case; connection-only timing begins in T002, not a new T001 protocol layer.

  **Read fields and constrained state (verbatim from data-model.md §4):**

  | Field | Required rule |
  | --- | --- |
  | `source` | Exact currently observed text or null; empty text remains an observed value. Prefer the observed open buffer, then disk, then loaded Script source, with provenance explicit below. This presentation choice does not select mutation authority. |
  | `revision` | Opaque `sr1:` plus 64 lowercase hex digits, or null when a safe read precondition cannot be produced. |
  | `state` | Target/document identity, lifecycle, authority/source availability and provenance, attributed dirty state, comparisons, stability, invalidation, observation interval, limitations and relevant next action as defined below. |

  > `status`: the existing observation outcome classification, including complete, limited, not-open, refused and unavailable distinctions; not-open or dirty does not itself make a successful observation an edit failure.
  >
  > `target`: permitted requested/resolved project, script and selected session, plus target kind. No candidate content or authentication/routing internals.
  >
  > `document`: observed `open`, `closed` or `unknown` and nullable opaque document identity derived from the existing session/document identity tuple. Do not expose native object IDs merely to encode that identity.
  >
  > `source_origin`: `editor_buffer`, `disk`, `loaded_resource` or null. `sources` has those three named authorities; each distinguishes observed/unavailable/not-applicable, current versus invalidated evidence, and whether its text equals `source`. Return a distinct permitted text only when it differs; invalidated text remains explicitly historical, never a current `source` or revision input.
  >
  > `dirty`: existing document-attributed buffer and loaded-Resource distinctions; unknown, dirty and not-applicable are not interchangeable. `consistency` preserves pairwise comparisons, performed/unavailable checks and changed/unchanged/unknown stability, with no atomic-snapshot claim.
  >
  > `interval`, `diagnostics` and `limitations`: actual acquisition interval and permitted outcome-specific facts, including partial availability and loaded-class-not-reloaded where applicable. `revision_unavailable_reason` is null with a revision, otherwise the relevant eligibility failure. `next_action` retains an actionable safe response rather than a static failure catalog.

  **Closed evidence fields (verbatim from data-model.md §5; internal, not public arguments):**

  | Field | Required rule |
  | --- | --- |
  | `collection` | Current native collection stamp and exact selected session lifetime; same target as `observation`. |
  | `lifecycle` | Literal `closed`, successful complete target-absence observation; unknown is ineligible. |
  | `close_epoch` | Canonical unsigned decimal counter owned by the active editor integration session. Every public script-close notification advances it. Reconfiguration, missing registration or overflow invalidates old bases. |
  | `file_revision` | Device/inode, length, SHA-256 and nanosecond mtime/ctime from independently acquired confined file metadata. Times are `{seconds: signed decimal string, nanoseconds: integer 0..999999999}`; identifiers/lengths use existing canonical decimal conventions. |
  | `resource` | `absent` after successful agreeing cache getters, or `present` with actual stable Script instance/path, source witness, edited=false and supported effective/compiled profile. Getter failure is `unavailable`, never absent. |
  | `consistency` | Actual before/after checks, invalidations and observation interval; never `atomic: true`. |

  **Request/state/effect constraints (verbatim from data-model.md §§6–7):**

  > The public request has only checked project/session/script selectors, `revision` and exact `replacement_source`. It contains no `basis`, prior read object, observation envelope or internal expected-state fields.
  >
  > Missing/unavailable required evidence refuses; a mismatch yields `revision_mismatch` with `next_action: {kind: "fresh_read"}` and no application.
  >
  > `ExpectedScriptState = Open(existing basis) | Closed(closed basis)`.
  >
  > `Admitted → Prepared → Validated → Authorized → ResourceApplied (present R only) → Persisted → MetadataVerified → IndependentlyVerified → Terminal`.
  >
  > Public edit result `mode` is `open`, `closed` or `undetermined` before usable basis selection. Its outcome preserves `verified_changed`, `verified_unchanged`, `refused`, `applied_unverified` and `application_unknown`; application preserves `not_applied`, `applied`, `partly_applied`, `unknown`. Existing causal ordering, validation versus availability, disclosure denial and immutable terminal outcomes remain authoritative. `lifecycle` contains admitted and independently observed final state; final unknown is not preserved-state proof.
  >
  > Closed evidence explicitly distinguishes Resource present/absent/unavailable/invalidated and B not-applicable/unavailable. Its source/identity/validation/persistence facts use the existing source-free edit evidence conventions. Loaded class/runtime state is **not reloaded or verified** by source editing; the result identifies that limitation for a loaded target without pretending old compiled metadata is new-source runtime evidence.

  > Closed results report `history: not_applicable_closed`; open native-history participation remains unchanged.

  Present R must equal D and be independently clean. Absent R has no invented instance/source/dirty facts; B has no version, saved version or history record. Narrow visibility to actual current supervised consumers; raw captures, RPCs, authorization and attempt state remain private/crate-private.

  **Tests and acceptance bundled here:** Write meaningful deterministic/core/boundary cases before their implementation. Exercise revision stability across incidental IDs/intervals and invalidation for every stable field; null revisions on unsafe/partial reads; exact matching-capture handoff; cached/absent closed positives; unchanged/empty/Unicode/bounds; dirty/equal-text-dirty/divergent R, unsupported source/context, missing getters, same-text file writes, namespace replacement, cache transitions, epoch reset/overflow/ABA, lifecycle races, unrelated human history, lost/partial write and mtime failure, newer work and pre/post-effect cancellation. Real VM witnesses must independently observe D/R/B applicability and no manufactured Resource/document, later open/Save/reparse/rescan/fresh-runtime durability, source privacy and affected exports. A refused-everywhere implementation fails this task.

  Prove affected v6/revision-4 authentication/strict tuples, capability negotiation, load, shared-slot exclusion, cancel/disable and representative existing observation/edit/open/discovery/close behavior. Keep the old edit caller's closed refusal. Record the exact changed boundary, affected cases and reachable regression; demonstrate native equivalence or rerun affected evidence, not every historical campaign. Apply the Rust/native checks and shape review below. Record actual closed execution timings and provenance; MCP accepted-frame timing belongs to T002 and its adversarial extensions in T003.

**Checkpoint:** A complete independently verified protocol-independent read/revision/closed-edit family exists; all required safety mechanisms and direct tests are present. No MCP support or real-client pass is claimed yet.

## Phase 3: Complete external MCP MVP and positive real-agent workflow (US1 + US2, P1)

**Goal:** Deliver the complete useful external workflow, not transport plus representative smoke: both selected coding-agent clients discover, read, edit and freshly read eligible open, cached-R closed and absent-R closed targets while preserving lifecycle.

**Independent test:** Both selected clients initialize, discover exactly three tools and perform `discover → read → edit → fresh read` on the three supported target profiles with independently witnessed source/identity/lifecycle. Direct known-target edits, unchanged intent and informative dirty/limited/non-editable reads also work. Negotiation, malformed/unknown requests and unavailable editors remain truthful and source-free. Later opening witnesses already-verified closed persistence; it never enables or verifies the original mutation.

- [ ] T002 [US1] [US2] Deliver the complete local stdio MCP MVP and positive real-agent workflow in `mcp-server/src/bin/godot-agent-kit-mcp.rs`, `mcp-server/src/mcp/mod.rs`, `mcp-server/src/mcp/transport.rs`, `mcp-server/src/mcp/schema.rs`, `mcp-server/src/mcp/handler.rs`, `mcp-server/Cargo.toml` and `mcp-server/Cargo.lock`; bundle protocol/operation tests in `mcp-server/tests/mcp_transport.rs` and `mcp-server/tests/mcp_tools.rs`, real-agent lifecycle evidence in `godot-addon/tests/mcp_lifecycle_acceptance.py`, `godot-addon/tests/fixtures/mcp/` and `godot-addon/tests/run_mcp.py`, fixed setup/stdio relay in `godot-addon/tests/run_in_vm.py` and `godot-addon/tests/vm_guest.py`, and connection/migration/evidence guidance in `specs/007-mcp-script-workflow/quickstart.md` and `.github/LOCAL_VM.md`.

  **Dependencies:** T001 complete and delivered under the sequencing rule. T002 exposes real discover/read/open-edit/closed-edit execution, not tool stubs or an advertised future edit capability.

  **Tests → implementation → integration:** Establish protocol/DTO/dispatch regression cases, then add exact `rmcp =3.5.0` (`default-features=false`, `server`) and Tokio `=1.53.1` (`default-features=false`, `rt,time,sync,io-util,io-std`). Preserve Rust 1.98.1/edition 2021 and existing pins. Review the actual locked transitive graph's provenance, licenses/notices and advisories; record findings in `specs/007-mcp-script-workflow/research.md`. No macros, HTTP/OAuth/TLS, extra client SDK or dependency-only PR.

  Implement MCP 2025-11-25 explicitly rather than SDK defaults: initialize/initialized gating, source-free ping/version discovery, sole-revision fallback, fixed ordered `discover_scripts`/`read_script`/`edit_script` catalog, `capabilities: {tools: {}}`, no pagination/listChanged/other advertised features, and explicit newer/unsupported-operation failure. A source-free modern `server/discover` probe may identify the selected revision without Godot dispatch. Use the exact schemas, annotations and concise catalog text in `contracts/mcp-interface.md`; review sufficiency qualitatively, not by a length threshold.

  Reuse T001/existing supervised runners. Preserve authoritative discovery coverage and read/edit projections; no safety decisions, filesystem writer, automatic retry or lifecycle orchestration at the adapter. Validate output as well as input. Return the exact typed object in `structuredContent` plus a terse outcome/action summary; no default full-object/source/revision text duplicate. Failures/limited reads retain their intended `isError` meanings. Both selected clients must actually use model-visible source/revision/state and refusal/action facts. If a required client demonstrably needs full JSON text, first record the failed smaller carrier and sufficient identical-object fallback plus context/escaping cost, and update the existing contract/research; do not invent client-name heuristics or silently drop a client.

  Implement a bounded SDK-type transport, not a second MCP protocol: strict duplicate-key/depth/UTF-8/ID checks, finite line collection/writes, source-free standard parse/request/method/params errors, and no payload excerpts. Capture `AttemptClock` at complete-frame acceptance before tool validation/routing; retain it through consumed-output serialization/flush. Run one synchronous supervisor off the current-thread executor with retained ownership and per-call cancellation; never queue an edit or drop its owner because an SDK response is suppressed. Implement EOF, SIGINT/SIGTERM, blocked output and bounded shutdown without killing Godot or claiming joined blocked stdin. Do not enable SDK message tracing or honor `RUST_LOG` for it.

  Add required `--registry` configuration and no-effect `--help`/`--version`; reject unknown/duplicate flags before dispatch. Dispatch existing internal workers before MCP startup and keep their private socket/stdout separation. Preserve all five existing local binaries. Implement fixed `prepare-mcp --revision SHA --run-id ID --artifacts DIR` and `mcp-stdio --run-id ID` test commands from quickstart, with committed guest inputs, source-free run receipt, owned project/session, fixed executable/registry, clean protocol relay and bounded cleanup. No arbitrary guest command/path endpoint, host GUI fallback, credential copy or product remote mode. Extend affected VM wrapper tests in `godot-addon/tests/test_run_in_vm.py`, `godot-addon/tests/test_vm_guest.py` and `godot-addon/tests/test_vm_tart.py` only where the corresponding behavior changes.

  **Binding connection/framing constraints (verbatim from data-model.md §§2–3):**

  | Item | Required rule |
  | --- | --- |
  | MCP input | Complete newline-delimited frame ≤16 MiB; JSON depth ≤64. Inner legacy open-edit payload still ≤12 MiB. Exact JSON representation bounds are checked in addition to decoded source bounds. |
  | MCP result | Tool object ≤16 MiB; complete serialized MCP response ≤64 MiB. These retained caps cover any evidenced compatibility fallback, including escaped JSON text; they do not require a duplicate carrier or eager allocation. Public read projection does not duplicate agreeing source texts. |
  | Control traffic | At most eight outstanding JSON-RPC IDs per connection and one admitted tool execution; duplicate live IDs are rejected without replacing the original cancellation owner. No operation queue. |

  > `ConnectionState = AwaitingInitialize | Ready | Closing | Closed`.
  >
  > A connection stores the configured registry path, selected protocol revision, bounded request-ID ownership and active call control only. It does not store project source history, issued revisions, outcomes for replay or implicit selected projects. Client/server metadata is protocol information, not routing authority.
  >
  > `CallControl` binds one MCP ID to a new domain request ID, original `AttemptClock`, checked selector intent, an operation-local atomic cancellation flag and its owned supervision handle. One call cannot cancel another. The SDK token is translated at the adapter; cancellation does not destroy the core owner. EOF, delivery loss or process shutdown cancels admission/new stages, drains bounded owned supervision and preserves effect uncertainty. Nothing claims rollback or cancels the user's editor.

  The quoted selector/source/discovery/time constraints in T001 apply without alteration. From the MCP contract: string JSON-RPC IDs are ≤128 UTF-8 bytes; numeric IDs are exact SDK-representable signed integers, not rounded floats. Unknown cancellation IDs are ignored. Bound initial handshake/partial frames to 10 seconds, retain no between-request idle timeout, and bound runtime teardown to one second after original-cutoff owned cleanup. Overflow/serialization/delivery failure after possible effects retains known facts or unknown, never fabricated not-applied.

  **Binding public field constraints:** Use JSON Schema 2020-12 objects with `additionalProperties: false`; reject unknown/duplicate keys. Exact fields from `contracts/mcp-interface.md`:

  | Field | Required rule |
  | --- | --- |
  | `project_root` | Nonempty string; required on all tools |
  | `session_id` | Optional string matching `^[0-9a-f]{32}$` |
  | `script_path` | Nonempty string; required on read/edit |
  | `revision` | String matching `^sr1:[0-9a-f]{64}$`; required on edit |
  | `replacement_source` | String, including empty; required on edit |

  Omitted session means unique authenticated selection; explicit null is not a session. Discover accepts only project/session; read adds script; edit adds revision/replacement. No public basis, force, lifecycle, timeout, retry, output-format or raw command fields. Apply byte bounds semantically, not JSON Schema character counts. Public read/state fields and enums are exactly the quoted T001 model constraints; no private native IDs/expected-state payload.

  **Envelope/error constraints (verbatim from data-model.md §8):**

  > Each public tool object has exactly `schema_version: 1`, `operation`, `request_id`, `result`, `error`. Exactly one of `result` or `error` is non-null.
  >
  > An adapter `error` has `category` (`input`, `admission`, `host`), fixed `code`, fixed `stage`, `application`, nullable permitted requested/resolved target and typed `next_action`. It has no arbitrary exception/request text. A malformed request before dispatch reports not-applied; host failure after possible effects retains actual certainty or unknown. Editor-operation refusals remain actual operation results, not malformed-input errors.
  >
  > Protocol/JSON-RPC errors remain outside this tool object when the request is not a valid tool invocation. The external request ID is bounded correlation, never target authority or a replay token. `isError` is an MCP summary, not a replacement for outcome/application/evidence. Dirty or divergent successful observations are not edit failures.

  Public read has exactly `source`, `revision`, `state`; edit has exactly `mode`, `outcome`. Adapter errors have exactly `category`, `code`, `stage`, `application`, `requested_target`, `resolved_target`, `next_action`. Next-action variants are `{kind: "correct_request"}`, `{kind: "select_session", candidate_sessions: [...]}`, `{kind: "fresh_read"}`, `{kind: "check_setup"}`, `{kind: "unsupported"}` and `{kind: "none"}` with only permitted relevant details. Adapter capacity is `server_busy`, not observed `editor_busy`. Closed lifecycle is `{admitted: "closed", observed_final: "closed" | "open" | "unknown"}`; absent B/history is `not_applicable_closed` only when independently confirmed.

  **Acceptance bundled here:** Actual process smoke and deterministic tests cover negotiation/initialization, full schemas, all three real dispatch paths, malformed/out-of-bound/duplicate input, no unknown-tool effects, limits including escaped output, exact IDs/ownership, output validation after possible effects, operation versus host/request errors, limited/dirty reads, busy/no queue, direct cancel/shutdown/output-loss ownership and source-free stdout/stderr with sentinels. **Codex CLI 0.153.4 and OMP 18.5.1** must perform the complete positive public workflow below using the source-backed scoped setup in `quickstart.md`, not only connect/list and representative smoke. Keep credentials host-owned, preserve normal approval and use scoped configuration without permission bypass. Version/config output or an SDK-only harness is insufficient. Record accepted-frame-to-consumed-output bounds and apply Rust/changed-tooling checks. Composed A–E and full feature completion remain T004 obligations.

  **Positive lifecycle workflow bundled with implementation:** Cover US2.1–US2.5 and the MCP positive portion of US5.5 in `godot-addon/tests/mcp_lifecycle_acceptance.py`, extending `closed-native`/`closed-lifecycle` through the actual MCP executable and the fixed prepared prompts/receipts. For each selected real client, prove `discover → read → edit → fresh read` for an eligible open script, a closed script with cached clean R and a closed script with confirmed absent R. Also prove directly known-target edit without mandatory discovery; already-satisfied zero setter/write/mtime/history intent; stable unchanged revision and meaningful changed revision; partial discovery and dirty/divergent/closed/partly observable informative reads with null unsafe revision; source provenance and distinct invalidated/unavailable/empty facts. Include multiline/Unicode/empty source and supported bound cases without truncation or duplicated agreeing source. Closed reads/edits never load/open/create a target Resource or document; open edits retain the same open document.

  Real-agent calls/results and independently acquired D/R/B/roster/history witnesses must agree. Demonstrate model-visible source/revision/relevant state, informative limited reads and actionable refusal under the selected carrier; a raw protocol event or model claim alone fails. Use selectors plus current revision/replacement only, with no shell/file/private-bridge substitution. After verified closed success, ordinary fixture opening plus fresh MCP read/Save/reparse/rescan/fresh-runtime checks witnesses durability without manufacturing initial success or claiming live class reload. Reuse T001 native witnesses; no separate lifecycle task, second agent orchestrator, token service or lifecycle tool. Fix exposed lifecycle/projection defects in their actual production owner within T002.

**Checkpoint / MVP:** T001–T002 provide the first truthful external MCP MVP: the complete US1/US2 public workflow through both real clients, including both positive closed profiles and lifecycle preservation. T003 adversarial acceptance and T004 composed/cumulative acceptance remain required for full feature completion; basic safety is already enforced and directly tested in T001/T002.

## Phase 4: Adversarial safety, preservation and interrupted-effect behavior (US3 + US4, P1)

**Goal:** The new public MCP path must not weaken trusted mutation semantics under stale, conflicting, racing, interrupted or partially delivered conditions. Preservation and interrupted effects are one behavior boundary, not separate test-matrix PRs.

**Independent test:** Barrier-controlled fixtures perturb target/human/lifecycle state and interrupt open, cached-R closed and absent-R closed edits before/after authorization. Compare actual surviving source/history/identity with delivered effect facts or explicit delivery unavailability and fresh-read recovery; prove original-clock bounds and no late effects after proven refusal.

- [ ] T003 [US3] [US4] Establish adversarial public-path preservation and effect-sensitive termination across `mcp-server/src/script_read.rs`, `mcp-server/src/script_closed_edit/`, `mcp-server/src/runner/closed_edit/`, `mcp-server/src/mcp/handler.rs`, `mcp-server/src/mcp/transport.rs` and `godot-addon/native/script_closed_edit.cpp`; bundle fault-revealed fixes with regression coverage in `mcp-server/tests/mcp_transport.rs`, `mcp-server/tests/mcp_tools.rs`, `godot-addon/tests/mcp_preservation_acceptance.py`, `godot-addon/tests/mcp_interruption_acceptance.py`, `godot-addon/tests/fixtures/mcp/` and `godot-addon/tests/run_mcp.py`, and record independent safety, survivor-state, privacy and timing evidence in `specs/007-mcp-script-workflow/quickstart.md`.

  **Dependencies:** T002. T001/T002 already own all baseline guards and direct regression evidence; this task establishes their adversarial public-path interactions and corrects any demonstrated enforcement/mapping defect in its actual owner. Keep `preservation`, `interruption` and affected `closed-lifecycle` cases separately named and runnable, with shared event barriers and witnesses. They are not separate Spec Kit tasks, one monolithic test file or an obligation to execute one aggregate command. No new generic cancellation/recovery/retry framework, blanket hardening layer or persistent outcome service.

  **Preservation and stale-state acceptance:** Cover all US3.1–US3.7. Include open dirty/equal-text-dirty and prior native history; missing/malformed/wrong-target/stale revisions; same-text buffer versions, closed/reopened identities, closed→open, open→closed and closed→open→closed epoch; same-text ctime, leaf/parent namespace and session replacement; R present/absent/identity/edited/profile transitions; unknown roster/getters/absence; invalid source/effects/context. A fabricated matching digest never bypasses current safety or permission checks. Introduce races after capture equality, before authorization, at native entry and after a real effect; preserve newer human edits and any newly opened document without reassertion or compensation.

  Exercise ambiguous sessions, ended explicit sessions, escaping/outside-project paths and denied access; no focus/fallback target, unauthorized inventory/source/hash/diagnostic disclosure or editor launch. Preserve unrelated dirty documents/history. Overlap independent connections and existing local callers to exercise the actual shared editor slot, not only adapter capacity: rejection is not queued/replayed and cannot cancel or retarget another owner. Independently verify zero lost human work, unsafe overwrite, guessed target or lifecycle workaround, including human changes arriving before/after effects and their interruption combinations.

  **Interruption and delivery acceptance:** Cover all US4.1–US4.4 with the same preservation invariants. Exercise cancellation, original-deadline expiry, unresponsive editor, EOF/disconnect, SIGINT/SIGTERM, addon disable, worker/native channel loss, malformed private reply, output serialization/size failure and blocked/lost output before/after authorization and after known R/disk/partial/metadata effects. Fault artifacts remain separately identified and cannot ship in exports.

  Deliverable structured results preserve requested/resolved target, distinct MCP/domain correlation, reason/stage, availability/invalidation, effect certainty and safe next action; private editor-protocol failure after effects is never invalid-client-input/not-applied. If cancellation suppresses the response or output is lost, record delivery unavailable, not rollback/successful delivery/safe replay. Independently inspect survivor state and freshly read the original explicit target before another intentional edit. Confirm bounded owned cleanup, no killed editor, no late effect after proven refusal, no queued/replayed/compensating edit and no cross-call cancellation for duplicate/unknown IDs. Preserve sticky disclosure denial even after an earlier causal failure.

  Measure from accepted complete bounded frame before validation/routing through consumed-output serialization/flush: every read/discover ≤5 s and open/cached-closed/absent-closed edit ≤10 s, including blocked-editor cases; preserve 4.5/9.5 s work cutoffs and 0.5 s output reserve. Additional acquisition/validation never renews the budget. Separately prove finite output-backpressure/handshake/partial-frame/shutdown behavior; blocked-client tests are not consumed-output performance passes. With each selected real client, demonstrate model-visible limited/refusal and partial/unknown outcome/action interpretation under the selected carrier, with no blind replay or incidental source-bearing error/log leakage. Distinguish actual client-local abandonment from wire cancellation; use the focused protocol driver for server cancellation-notification cases rather than mislabeling a client action. Record relevant client reconnect/retry and output-spill behavior without changing product semantics.

  Newly execute the task-owned adversarial interactions and failure-driven regressions; reuse valid earlier direct guards and positives after relevant-input review. A fix that changes a shared boundary needs only the concretely affected additional evidence under TEST_POLICY.md, not an automatic historical campaign.

**Checkpoint:** US3 and US4 are established together across conflicting state, effect boundaries and delivery failure. Results are grounded in surviving independently observed state, not SDK acknowledgments, a generic error flag or process exit.

## Phase 5: Composed real-agent A–E and cumulative feature acceptance (US5, P1)

**Goal:** Complete the real-agent MCP A–E interaction and cumulative feature evidence without replaying unchanged historical campaigns or inventing closed-buffer history.

**Independent test:** At least one actual coding-agent conversation drives the full MCP discover/read/edit workflow; owned ordinary editor/fixture history and lifecycle actions independently establish A–E and durability. Both selected clients retain representative and carrier evidence from earlier tasks.

- [ ] T004 [US5] Deliver the composed real-agent A–E and cumulative acceptance workflow in `godot-addon/tests/mcp_composed_acceptance.py`, `godot-addon/tests/mcp_privacy_acceptance.py`, `godot-addon/tests/fixtures/mcp/` and `godot-addon/tests/run_mcp.py` as the `composed` and `privacy-export` groups; bundle directly required scenario/runner regression checks and any proven defect fix, then record complete coverage, implementation-shape/constitutional review and verified limitations in `specs/007-mcp-script-workflow/quickstart.md`, `specs/007-mcp-script-workflow/plan.md`, `specs/007-mcp-script-workflow/tasks.md` and `PROJECT_STATUS.md`.

  **Dependencies:** T003; all prior task acceptance remains valid after relevant-input review. This task owns the new composed interaction and final evidence coverage, not unfinished implementation, delayed shape cleanup, a historical full-suite replay or a release/Phase 3 exit decision.

  **Scope and acceptance:** Cover US5.1–US5.5 and SC-005 exactly: A clean-open independent intended D/R/B and saved state; B dirty/equal-text-dirty safe refusal and human/history preservation; C actual native Undo → Save → Redo → Save with prior history intact, MCP reads and independent authority witnesses (unsaved history remains truthful, not falsely persisted or editable); D ordinary close/reopen for open edits and later opening for already-verified closed edits, plus applicable Save/reparse/rescan/fresh-runtime persistence; E **twenty successful fresh-read/revision MCP edits covering supported open and closed profiles, with at least three dirty and three stale refusals interleaved**. Use both cached and absent R closed coverage, preserve each edit's admitted lifecycle, and apply history only where a real open buffer exists. No second edit substitutes for Undo/Redo, and no ordinary witness repairs divergence or enables a closed edit.

  At least one real-agent conversation supplies the full composed MCP interaction, not an internal harness relabeled as agent use. Inspect actual tool transcripts, source/revision/state use, safe refusal/uncertainty actions, timings, live D/R/B/history/lifecycle records and owned captures. Keep prompt guidance limited to target/human intent; qualitative FR-022/SC-009 review must establish sufficient lean descriptions with no internal architecture coaching or numerical prose budget.

  Complete privacy/export evidence across authorized, denied, ambiguous, interrupted, startup, help, worker and error paths: no incidental source/inventory/credential leakage, including sticky disclosure denial after earlier causal failure. Verify affected enabled/disabled/hook-only production exports exclude addon/native/fixture/probe tooling. Retain local-only/no-telemetry/exact support and all existing local v1 contracts. Review every scenario, FR and SC as newly executed, validly reused, rerun after concrete invalidation or substantively inapplicable with reasons; missing observation is not inapplicability. Reuse T001–T003 and accepted historical evidence where relevant inputs match, and newly run composed/privacy-export plus affected regressions only. Do not turn this task head into a release candidate or automatically declare Phase 3 exit.

**Checkpoint:** Feature 007 may be recorded complete only with all four task acceptances, complete valid evidence coverage and applicable shape/constitutional gates satisfied. PR review/merge is delivery metadata, not feature correctness. Phase 3 exit and release remain separate decisions.

## Phase 6: Polish and cross-cutting concerns (owned by capability tasks)

No standalone polish, security-hardening, formatting or documentation task. Each task owns its directly required docs, discovered defects, applicable static/build checks and proportionate shape cleanup before completion. T004 owns cumulative coverage and the current feature/task lifecycle update, not unfinished implementation deferred from T001–T003.

### Verification and completion rules for every task

- Follow [TEST_POLICY.md](../../TEST_POLICY.md). Start with affected tests/scenarios; required permanent tests prove consumer-visible behavior, boundaries, state transitions, precedence or preservation, not source wording, wiring echoes or incidental defaults. Pair any discovered regression with failing-before/passing-after evidence where practical.
- Rust changes: run applicable formatting, Clippy `--all-targets --locked -- -D warnings`, `cargo test --locked` including doctests, rustdoc and actual affected consumer builds from `mcp-server/` under the pinned toolchain. No blanket `--all-features` or invented workspace. Native changes require affected build/load/behavior and provenance; wrapper/runner changes require affected selection/identity/cleanup checks. An unchanged native build or historical group does not require repetition solely because HEAD changed.
- Real Godot uses `godot-addon/tests/run_in_vm.py` on the dedicated macOS arm64 VM with committed source, independent witnesses, event barriers/deadlines and owned captures. No arbitrary sleeps, host GUI fallback or headless replacement for visible-editor evidence. Retain exact source/engine/platform/native/ABI/SDK/client/fixture identity and actual timings; retrieve artifacts and stop owned runs. Raw source-bearing transcripts and credentials stay outside Git.
- For wider regression execution, name the exact changed boundary, exact affected suite/group and concrete reachable invalidation. The v6/revision-4 owner/negotiation change warrants affected cross-language/loading/admission/cancel/export and representative existing-operation preservation, not automatic historical `all` campaigns. Rebuilt artifacts require demonstrated relevant equivalence or affected reruns. Maintain cumulative valid coverage, not a literal-final-commit replay.
- Complete each task's implementation-shape review under [AGENTS.md](../../AGENTS.md#implementation-shape-completion-gate): coherent ownership, narrow current-consumer visibility, no speculative APIs/frameworks, lifecycle enum rather than impossible boolean combinations, typed safety/evidence failures, and proportionate responsibility-based splits of newly enlarged modules. Review large wire/runner/native modules explicitly; do not mechanically split by line count or broaden unrelated refactors.
- Record applicable constitutional compliance in the existing plan/review/evidence: independent authority verification; human work/stale/lifecycle protection; native mutation; effect-sensitive results; real-editor A–E where applicable; protocol separation; export/privacy; lean interfaces; source provenance; justified present complexity. The plan's Principle XIII rationale covers the selected mechanisms; any additional mechanism needs its own concrete requirement, simpler alternative, insufficiency and ongoing-cost justification before introduction.
- Mark only the selected accepted task `[X]`, update material `PROJECT_STATUS.md` lifecycle truth and publish one focused task PR; stop. A completed task/feature is complete even with an open PR, but dependent work waits for merge unless stacking is explicitly authorized. No automatic merge, support expansion or next task.

## Requirement, entity and contract coverage

| Approved scenario | Implementation owner | Acceptance owner |
| --- | --- | --- |
| US1.1–US1.5 | T002 complete connection/catalog/schema/handler/carrier | T002 both-client workflow, transport and qualitative interface proof; T003 uncertain-result interpretation; T004 composed qualitative review |
| US2.1–US2.5 | T001 trusted capture/revision/open reuse/closed capability; T002 public dispatch/projections | T001 direct native/core proof; T002 complete real-agent lifecycle workflow, known-target/no-op/informative reads and later-opening durability |
| US3.1–US3.7 | T001 core/native guards and effects; T002 enforced public admission; T003 adversarial interaction corrections | T001/T002 direct regressions; T003 combined preservation/race/interruption evidence |
| US4.1–US4.4 | T001 effect reducer/supervisor; T002 call/error/delivery ownership; T003 adversarial termination corrections | T002 direct boundary tests; T003 complete interruption/carrier/timing/survivor-state evidence |
| US5.1–US5.4 | T001 native coherence/history/durability; T002 real MCP workflow; T004 composed interaction | T004 actual-agent A–E, with valid T001–T003 supporting evidence |
| US5.5 | T001 complete closed family; T002 both positive closed public profiles; T003 adversarial closed behavior | T001 deterministic/boundary/native positives and faults; T002 real-agent positives; T003 races/interruption; T004 cumulative coverage |

| Functional requirement | Implementation owner | Acceptance owner |
| --- | --- | --- |
| FR-001 | T002 external MCP executable/client configuration | T002 both real clients and positive workflows; T004 composed agent use |
| FR-002 | T002 exact three-tool catalog/schemas | T002 discovery/schema/lean-use proof; T004 qualitative review |
| FR-003 | T001 checked semantics; T002 strict input/output/transport | T001/T002 direct boundaries; T003 post-effect serialization/failure cases |
| FR-004 | T001 existing authenticated routing/confinement; T002 dispatch | T001/T002 direct guards; T003 ambiguity/denial/replacement/escape cases |
| FR-005 | T001 trusted read/revision; T002 projection | T001 read semantics; T002 real-client informative reads and carrier |
| FR-006 | T002 existing discovery mapping | T002 exact complete/limited/interrupted/refused scope; T003 denied/racing scope |
| FR-007 | T001 fresh matching capture/lifecycle guards; T002 mapping | T001 direct guards; T002 positive lifecycle/no-op; T003 stale/lifecycle/ABA races |
| FR-008 | T001 reuse of Feature 002; T002 open dispatch | T001 affected legacy preservation; T002 open workflow; T003 dirty/stale; T004 A–E/history |
| FR-009 | T001 full guarded closed family | T001 cached/absent native positives/refusals; T002 positive MCP profiles; T003 faults; T004 durability/coverage |
| FR-010 | T001 core/native ownership; T002 adapter boundary | T001/T002 shape and boundary review; T003 no bypass; T004 final constitutional review |
| FR-011 | T001 truthful domain outcomes; T002 projection/carrier | T002 useful results; T003 partial/unknown/invalidated results; T004 cumulative coverage |
| FR-012 | T002 request/tool/host error mapping | T002 direct error distinctions; T003 failures after possible effects |
| FR-013 | T001 bounded ownership/cancel; T002 connection/call lifecycle | T001/T002 direct tests; T003 cancel/EOF/disable/shutdown and survivor state |
| FR-014 | T001 clocks/supervisors; T002 accepted-frame delivery clock | T001 closed timing; T002 positive consumed-output timing; T003 blocked/deadline/delivery cases |
| FR-015 | T001 immutable terminal/authorization state; T002 no replay/queue | T002 direct busy/error behavior; T003 no late effects/retry/compensation and fresh-read recovery |
| FR-016 | T001 complete capability negotiation; T002 static catalog | T001 peer compatibility; T002 unavailable-versus-connected support; T003 failures |
| FR-017 | T001 domain disclosure; T002 transport/log/carrier separation | T001/T002 sentinels; T003 sticky denial/failure privacy; T004 cumulative privacy/export |
| FR-018 | T001 coordinated private cutover/unchanged local v1; T002 MCP schema 1 | T001 affected legacy/native evidence; T002 contract tests; T004 currentness review |
| FR-019 | T002 both real-client positive workflow; T004 composed interaction | T002 open/cached-closed/absent-closed workflows; T004 actual-agent A–E |
| FR-020 | T001 new closed semantics; T002 MCP path; T003 adversarial behavior; T004 coverage | Every task's focused evidence/reuse review; T004 cumulative valid coverage |
| FR-021 | T001 tooling boundary; T002 local setup/no extra authority | T001 affected exports; T002 client/setup/privacy; T003 failures; T004 privacy/export |
| FR-022 | T002 lean static descriptions and structured guidance | T002 actual-agent choice/source/revision/state use; T003 failure actions; T004 qualitative review |

| Success criterion | Implementation owner | Acceptance owner |
| --- | --- | --- |
| SC-001 | T002 complete server and both real-client paths | T002 exact catalog/representative calls and full positive workflows; T004 composed use |
| SC-002 | T001 trusted lifecycle behavior; T002 useful public workflow | T002 discover/read/edit/fresh-read open and both closed profiles with independent witnesses |
| SC-003 | T001 core/native enforcement; T002 public admission; T003 adversarial interactions | T001/T002 direct safety proof; T003 full preservation/race/refusal matrix |
| SC-004 | T001 effect semantics; T002 carrier/errors; T003 interruption behavior | T002 direct mapping; T003 model-visible certainty/actions and delivery loss |
| SC-005 | T001 native guarantees; T002 MCP path; T004 composed interaction | T004 actual-agent A–E, twenty fresh-basis successes, three dirty and three stale refusals, both closed profiles |
| SC-006 | T001 work clocks; T002 accepted-frame/output owner | T002 positive timings; T003 blocked open/closed and cancellation/deadline timings |
| SC-007 | T001 disclosure; T002 catalog/stdio/privacy | T002 traffic/advertisement; T003 denied/interrupted errors; T004 cumulative privacy |
| SC-008 | T001 compatibility/new closed proof; T002 MCP path; T004 coverage ownership | T001 affected old contracts/new positives; T002/T003 public behavior; T004 evidence/export/support review |
| SC-009 | T002 lean descriptions/projections/carrier | T002 both clients' model-visible use; T003 targeted failure guidance; T004 qualitative review |

All **26 acceptance scenarios, FR-001–FR-022 and SC-001–SC-009** have implementation and acceptance owners. Cross-cutting safety/privacy/timing binds each applicable task, not only a table row. The unchanged model's bounds and constrained fields remain quoted in T001/T002 task descriptions. Existing observation/discovery/open-edit definitions remain authoritative for reused fields.

`contracts/closed-edit.md` is implemented end-to-end by T001; `contracts/execution.md` by T001 core/private cutover and T002 adapter ownership; `contracts/mcp-interface.md` by T002. T003 verifies and corrects adversarial interactions in those existing owners, not another safety architecture. T004 owns composed/cumulative acceptance and final shape/constitutional review. No requirement is discharged merely by assigning it a test group.

## Dependencies and execution order

```text
Existing setup + approved design/current analysis + implementation authorization
  → T001 complete trusted read/revision/closed-edit capability
  → T002 US1 + US2 complete external MCP MVP and positive real-agent workflows
  → T003 US3 + US4 adversarial preservation and interrupted-effect behavior
  → T004 US5 composed real-agent A–E and cumulative feature acceptance
```

T001 is independently useful without MCP. T002 delivers complete external value using that capability, including the lifecycle evidence that naturally proves its implementation. T003 combines the shared human-work/effect-safety boundary rather than serializing separate preservation and interruption matrices. T004 consumes valid prior evidence and owns the genuinely new history/sequential composition. The order is T001 → T002 → T003 → T004, one task/PR at a time; shared story labels do not create extra tasks or permission to advance automatically.

## Parallel execution examples per story

No independent task-level `[P]` opportunity is claimed. Within the **one selected task**, after shared interfaces are fixed, independent file work can be divided as follows; one integration owner owns shared runner/native seams and final verification. Parallel contributors skip mid-flight broad builds/checks; collect all changes before running the task's integrated checks.

| Story/task | Safe intra-task example | Shared boundary / finish condition |
| --- | --- | --- |
| US1 / T002 | Adapter schema/catalog work in `mcp-server/src/mcp/schema.rs` alongside fixed relay command/testing work in `godot-addon/tests/run_in_vm.py` and `godot-addon/tests/test_run_in_vm.py` | Agree public contract and prepared-run receipt first; handler/VM guest integration has one owner; both clients then run against the same complete executable. |
| US2 / T002 | Lifecycle transcript driver in `godot-addon/tests/mcp_lifecycle_acceptance.py` alongside independent D/R/roster witness work in `godot-addon/tests/fixtures/mcp/` | Fix the event/witness contract first; one owner integrates the same complete MCP slice and `run_mcp.py`; no separate lifecycle PR. |
| US3 / T003 | Python target/preservation cases in `godot-addon/tests/mcp_preservation_acceptance.py` alongside fixture-side human/lifecycle barriers in `godot-addon/tests/fixtures/mcp/` | Share effect-stage expectations with interruption cases; integrate before acceptance and fix actual production defects within T003. |
| US4 / T003 | Process/transport fault driver in `godot-addon/tests/mcp_interruption_acceptance.py` alongside independent survivor-state witnesses | Same task and safety purpose as preservation; no competing edits to the cancellation owner; verify actual effects and delivery separately. |
| US5 / T004 | Composed history/sequential driver in `godot-addon/tests/mcp_composed_acceptance.py` alongside privacy/export driver in `godot-addon/tests/mcp_privacy_acceptance.py` | One owner integrates runner/coverage; use isolated owned fixtures, not concurrent access to one editor. |

T001 additionally permits closed-domain deterministic tests in `mcp-server/tests/script_closed_edit.rs` alongside native witness implementation after basis/receipt semantics are fixed; shared protocol/native integration remains one owner's responsibility. These are contributors to one task, not new Spec Kit task IDs or additional PR units.

## Implementation strategy

### MVP first

Complete T001 and T002 separately with their own directly related safety/behavior evidence and one PR each. T002 is the first external MVP: both real coding-agent clients perform the full discover/read/edit/fresh-read workflow for open, cached-R closed and absent-R closed targets, direct known-target and unchanged intent, and representative informative non-editable reads. A catalog, representative transport smoke or universally refusing closed path is insufficient. T003 and T004 remain required for full feature acceptance.

### Incremental delivery

Deliver the protocol-independent capability, then its complete external workflow, then adversarial preservation/interrupted-effect correctness, then composed real-agent A–E and cumulative acceptance. Keep direct tests/docs with the implementation they prove; scenario driver names do not determine PR boundaries. Reuse accepted evidence after relevant-input review and rerun only affected portions. If a selected mechanism cannot satisfy acceptance, report the concrete product/design blocker rather than weakening scope or adding an open-first fallback.

### Granularity and constitutional review

**Review status:** Passed on 2026-10-04 after a fresh review of the actual revised task/acceptance boundaries against AGENTS.md, TEST_POLICY.md and the maintainer's correction. This supersedes the earlier six-task granularity disposition. The following answers record that review; they are not `/speckit.analyze`, task completion, technical-plan approval or implementation acceptance.

**Why the previous split is superseded:** The earlier lifecycle task proved the useful public behavior introduced by the MCP-server task; separating it let that task's acceptance stop at representative calls rather than the complete first external workflow. The separate preservation and interruption tasks exercised the same target/human-work/effect boundary and depended on the same fixtures and owners. Different matrices/files did not justify separate Spec Kit PRs under AGENTS.md. Their cases/drivers remain; their artificial task boundary does not.

T001 stays cross-layer because its closed capability is not complete or safe to advertise with only Rust models, addon transport or native effects implemented. T002 now bundles the full positive client workflow. T003 has one adversarial/effect-safety purpose across separately runnable groups. T004 remains new composed A–E plus cumulative evidence, not unfinished implementation cleanup. No specification, architecture, safety rule, support profile, selected dependency or constitutional gate is weakened; no historical whole-suite/release obligation is added.

| Required review question | Disposition and concrete basis |
| --- | --- |
| Does every task deliver a meaningful capability, behavior or verifiable architectural outcome? | Yes: T001 complete reusable read/revision/closed mutation; T002 complete external real-agent workflow; T003 adversarial preservation/effect correctness of that public path; T004 the new composed native-history/sequential interaction and complete valid feature evidence. No file or symbol is a delivery unit. |
| Is each realistically reviewable as one focused PR? | Yes, with the task's single outcome as the boundary. T001 reviews one coordinated capability across existing owners; T002 reviews one adapter and its useful positive workflows; T003 reviews one adversarial mutation contract; T004 reviews composed acceptance/evidence. Distinct production/test responsibilities stay in separate files where appropriate, not separate feature tasks. |
| Are tests/docs bundled with directly related implementation? | Yes. T001 owns direct native/core safety and positives. T002 now owns all positive lifecycle/client proof needed for the MCP implementation. T003 includes any demonstrated safety/mapping fixes with their regression evidence. Every task owns its setup, focused docs and shape cleanup; T004 does not collect unfinished work. |
| Have artificial test-only task IDs been removed? | Yes. The standalone positive lifecycle task is folded into T002. Preservation and interruption no longer have separate IDs merely because they use different drivers/groups; together they establish T003's adversarial behavior. T004 is retained for genuinely composed A–E/cumulative acceptance, as requested. |
| How many serial PRs precede useful external value? | One protocol-independent prerequisite, T001; the second PR, T002, delivers the full useful external MVP. No third lifecycle-only PR is needed before real-agent discover/read/edit/fresh-read works. |
| Is all requirement/scenario coverage retained? | Yes. The explicit ownership tables cover 26/26 scenarios, 22/22 FRs and 9/9 SCs with both implementation and acceptance owners. Cached/absent-R positives, human work, effect certainty, A–E, twenty successes/three dirty/three stale refusals, both clients, privacy/export and existing local contracts remain required. |
| Is dependency ordering clear? | Yes: T001 → T002 → T003 → T004, all pending, one task/PR and the existing post-publication stop. Multiple story labels do not add task IDs. Intra-task examples have one integration owner; no false task-level parallelism is claimed. |
| Is test scope justified under TEST_POLICY.md? | Yes. Task-owned and directly invalidated groups plus applicable static/build checks; individually runnable preservation/interruption groups remain separate. The private cutover names affected admission/loading/cancel/export/legacy behavior. T004 adds the actual composed interaction and reviews reusable evidence, not historical full-suite or release replay. |
| Does T001 remain large only for a coherent cross-layer boundary? | Yes. Checked capture, native ownership/effects, supervised verification and matched private peers jointly constitute the usable closed capability. A Rust-only, addon-only or native-only PR would not deliver that boundary; file-count reduction does not justify unsafe/incomplete intermediate capability advertisement. No unrelated architecture or speculative framework is included. |
| Does T002 now represent the complete first external MVP? | Yes. Both selected real agents must perform the full workflow for open, cached-R closed and absent-R closed scripts, known-target and unchanged intent, and informative non-editable reads with model-visible source/revision/state and independent witnesses. Transport smoke alone cannot complete T002. |
| Does T003 have one coherent adversarial/effect-safety purpose? | Yes. Conflict/race/identity, newer human work, overlap, interruption and delivery are evaluated against the same admitted attempt and truthful effects. Separate drivers improve execution/review without creating separate PR obligations. Baseline enforcement remains inseparable from T001/T002. |
| Does T004 remain cumulative acceptance rather than deferred cleanup? | Yes. It newly proves genuine history, lifecycle durability and sequential composition, then accounts for all still-valid evidence and final shape/constitutional compliance. Earlier tasks cannot defer required implementation, direct tests or accidental complexity to it. Neither a release nor Phase 3 exit is implied. |

**Constitutional disposition:** The correction preserves independent D/R/B or applicable closed authorities, human-work/stale/lifecycle protection, native mutation/real open history, independently verified success and sticky effects/disclosure. Protocol ownership, local authority/export isolation and lean structured interfaces remain unchanged. Principle XIII favors removing per-matrix PR ceremony and reusing the selected relay rather than new client/config/replay machinery. Codex + OMP suitability is source-backed only; their actual runtime acceptance is still mandatory. No approved product specification, clarification, native closed route or dependency selection was changed.
