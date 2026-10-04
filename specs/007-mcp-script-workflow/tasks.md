# Tasks: Use the Trusted GDScript Workflow Through MCP

**Input:** Design documents in `specs/007-mcp-script-workflow/`: [specification](spec.md), [plan](plan.md), [research](research.md), [data model](data-model.md), [MCP contract](contracts/mcp-interface.md), [execution contract](contracts/execution.md), [closed-edit contract](contracts/closed-edit.md) and [validation guide](quickstart.md).

**Status:** Task derivation complete; all six implementation tasks are pending. Granularity review is recorded below. Consistency analysis and implementation have not run; no implementation task is selected or authorized by generating this document. Requirements/constitution approval is distinct from technical-plan review and implementation authorization.

**Prerequisites:** Review the selected technical plan, complete `/speckit.analyze` after granularity review, and satisfy the repository's implementation prerequisites before selecting exactly one dependency-ready task. Verify the active directory as `specs/007-mcp-script-workflow`; task branch names do not select it.

**Tests:** Explicitly required by FR-019–FR-020, SC-001–SC-009 and constitutional mutation gates. Direct deterministic, boundary, real-editor and documentation obligations are bundled with their implementation task, not split into file-, layer- or test-only prerequisite PRs. Later scenario tasks add independently runnable stateful/real-client acceptance capabilities, not permission to defer safety or elementary regression coverage.

**Organization:** All five stories are P1 and retain specification order. One shared protocol-independent capability precedes the first complete MCP slice. Setup and final cross-cutting headings intentionally contain no artificial standalone tasks. Exactly one Spec Kit task per PR; finish, publish and stop. No automatic merge or dependent work without merge or explicitly authorized stacking.

## Format and path conventions

Each implementation item uses `- [ ] Tnnn [P?] [USn?] Description with exact paths`. Indented paragraphs, quoted constraints and acceptance clauses are part of that task's binding description. `[USn]` is present only in story phases. There are no `[P]` task markers: these increments share integration/fixture or evidence paths and default delivery is serial. Intra-task parallel examples below do not authorize batching task PRs.

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
  7. Add the fixed `mcp` suite and individually runnable `closed-native`/`closed-lifecycle` groups through `godot-addon/tests/run_in_vm.py` and `godot-addon/tests/vm_guest.py`, using a fixture-only supervised consumer before the product MCP binary exists. Put owned witnesses in `godot-addon/tests/fixtures/mcp/`; identify pre-adapter evidence as core/native, not MCP or real-client acceptance. Extend the groups through actual MCP in T003. Test changed registration/ownership semantics in `godot-addon/tests/test_run_in_vm.py` and `godot-addon/tests/test_vm_guest.py` without historical GUI replay.

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

  Prove affected v6/revision-4 authentication/strict tuples, capability negotiation, load, shared-slot exclusion, cancel/disable and representative existing observation/edit/open/discovery/close behavior. Keep the old edit caller's closed refusal. Record the exact changed boundary, affected cases and reachable regression; demonstrate native equivalence or rerun affected evidence, not every historical campaign. Apply the Rust/native checks and shape review below. Record actual closed execution timings and provenance; MCP accepted-frame timing remains T002/T005 proof.

**Checkpoint:** A complete independently verified protocol-independent read/revision/closed-edit family exists; all required safety mechanisms and direct tests are present. No MCP support or real-client pass is claimed yet.

## Phase 3: User Story 1 — Connect and Discover the Supported Workflow (P1)

**Goal:** A real external client connects to a complete, bounded three-tool MCP server and can distinguish protocol support from editor eligibility using lean, sufficient contracts.

**Independent test:** Both selected external clients initialize, list exactly three tools and perform representative discover/read/edit calls on owned fixtures. Wrong negotiation, malformed/unknown requests and unavailable editors have explicit source-free outcomes and no unintended effects. A real agent can choose tools and use the source/revision/state contract without internal evidence knowledge.

- [ ] T002 [US1] Deliver the complete local stdio MCP server in `mcp-server/src/bin/godot-agent-kit-mcp.rs`, `mcp-server/src/mcp/mod.rs`, `mcp-server/src/mcp/transport.rs`, `mcp-server/src/mcp/schema.rs`, `mcp-server/src/mcp/handler.rs`, `mcp-server/Cargo.toml` and `mcp-server/Cargo.lock`; bundle protocol and operation-boundary coverage in `mcp-server/tests/mcp_transport.rs` and `mcp-server/tests/mcp_tools.rs`, the `transport` group in `godot-addon/tests/run_mcp.py`, fixed real-client setup/stdio relay in `godot-addon/tests/run_in_vm.py` and `godot-addon/tests/vm_guest.py`, and exact connection/migration/evidence guidance in `specs/007-mcp-script-workflow/quickstart.md` and `.github/LOCAL_VM.md`.

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

  **Acceptance bundled here:** Actual process smoke and deterministic tests cover negotiation/initialization, full schemas, all three real dispatch paths, malformed/out-of-bound/duplicate input, no unknown-tool effects, limits including escaped output, exact IDs/ownership, output validation after possible effects, operation versus host/request errors, limited/dirty reads, busy/no queue, direct cancel/shutdown/output-loss ownership and source-free stdout/stderr with sentinels. Exercise positive open/closed calls through the new executable in the VM. Codex 0.153.4 and Claude Code 2.1.222 each connect/list and complete representative discover/read/edit plus an interpretable refusal; version/config output or an SDK-only harness is insufficient. Keep credentials host-owned and use invocation-scoped configuration without permission bypass. The recorded authentication/tool-host limitations must be resolved for those runs or this task cannot claim its client acceptance. Record accepted-frame-to-consumed-output bounds and apply Rust/changed-tooling checks. Do not claim composed A–E, broad client support or Phase 3 completion.

**Checkpoint / MVP:** T001–T002 provide the smallest truthful connected three-tool MVP, including real closed editing. US1 alone cannot be reduced to a catalog advertising unavailable edit stubs. Broader story and feature acceptance remain pending.

## Phase 4: User Story 2 — Read and Edit Without Managing Editor Documents (P1)

**Goal:** Reproducible real-agent workflows find/read/edit both initial lifecycles and directly known targets without tab management, with independently reviewed public source/state/revision results.

**Independent test:** A real coding agent starts without an exact path, discovers the intended open and closed fixtures, reads, edits and freshly reads each. Independently observe the same open identity or continued closed absence and coherent applicable authorities; later fixture opening checks persistence only after closed success.

- [ ] T003 [US2] Implement the lifecycle-preserving real-agent acceptance workflow in `godot-addon/tests/mcp_lifecycle_acceptance.py`, `godot-addon/tests/fixtures/mcp/`, `godot-addon/tests/run_mcp.py` and the fixed prepared prompts/receipts in `godot-addon/tests/vm_guest.py`; extend `closed-native`/`closed-lifecycle` through actual MCP, bundle directly required runner checks and fix any demonstrated lifecycle/projection defect in `mcp-server/src/script_read.rs`, `mcp-server/src/mcp/handler.rs` or `mcp-server/src/runner/closed_edit/`, and record exact evidence in `specs/007-mcp-script-workflow/quickstart.md`.

  **Dependencies:** T002. Reuse T001's native witnesses and T002's fixed relay; no second agent orchestrator, token service or lifecycle tool. These are new composed public-path acceptance scenarios, not replacement for T001/T002 unit/boundary proof.

  **Scope and acceptance:** Cover US2.1–US2.5 and the MCP positive portion of US5.5: cached clean R and confirmed absent R; clean open same-document edit; already-satisfied zero setter/write/mtime/history intent; known-target direct edit without discovery; fresh revision stability and actual changed revision; partial discovery and dirty/divergent/closed/partly observable informative reads with null unsafe revision; source provenance and distinct invalidated/unavailable/empty facts. Include multiline/Unicode/empty source and supported bound cases without truncation or duplicated agreeing source. Closed reads/edits must never load/open/create a target Resource or document, and open edits must never close it.

  Real-agent transcript and independently acquired D/R/B/roster/history witnesses must agree; model claims alone fail. Demonstrate model-visible source/revision/relevant state and informative limited reads with each selected client, extending T002's carrier proof. Preserve exact selectors and current revision/replacement-only inputs; no shell/file/private-bridge substitution. After verified closed success, ordinary fixture opening plus read/Save/reparse/rescan/fresh-runtime checks witnesses durability without retroactively manufacturing success or claiming loaded-class hot reload. Record task-owned timings, provenance and relevant evidence reuse; run only these groups and affected checks.

**Checkpoint:** US2's five scenarios are reproducible through the real public path; baseline native closed proof is now supplemented by actual MCP/agent lifecycle evidence.

## Phase 5: User Story 3 — Keep Human Work and Target Boundaries Intact (P1)

**Goal:** Reproducible adversarial MCP scenarios establish preservation across stale/lifecycle/namespace/cache races and independent callers, including human work arriving after effects.

**Independent test:** Barrier-controlled fixtures introduce dirty/equal-text-dirty, stale, replaced, denied, unsupported and unavailable states before/after relevant boundaries; independent witnesses verify preserved source/history/identity and the exact refusal or effect-sensitive outcome, not just an error flag.

- [ ] T004 [US3] Implement the stateful preservation and boundary-race acceptance matrix in `godot-addon/tests/mcp_preservation_acceptance.py`, `godot-addon/tests/fixtures/mcp/` and `godot-addon/tests/run_mcp.py` as the `preservation` group plus affected `closed-lifecycle` cases; bundle any exposed enforcement regression with its fix in `mcp-server/src/script_read.rs`, `mcp-server/src/script_closed_edit/`, `mcp-server/src/mcp/handler.rs` or `godot-addon/native/script_closed_edit.cpp`, and record independent preservation/disclosure evidence in `specs/007-mcp-script-workflow/quickstart.md`.

  **Dependencies:** T003 in default execution order; shared fixtures/runner/evidence make this serial, not `[P]`. All product guards already belong to T001/T002. This task adds end-to-end hostile-state sequences and independent witnesses, not a deferred safety layer.

  **Scope and acceptance:** Cover all US3.1–US3.7, with US3.5–US3.6 transport-interruption combinations completed by T005. Include open dirty/equal-text-dirty and prior native history; missing/malformed/wrong-target/stale revisions; same-text buffer versions, closed/reopened identities, closed→open, open→closed and closed→open→closed epoch; same-text ctime, leaf/parent namespace and session replacement; R present/absent/identity/edited/profile transitions; unknown roster/getters/absence; invalid source/effects/context. A fabricated matching digest never bypasses current safety or permission checks. Introduce races after capture equality, before authorization, at native entry and after a real effect; preserve newer human edits and any newly opened document without reassertion or compensation.

  Exercise ambiguous sessions, ended explicit sessions, escaping paths and denied access; no focus/fallback target, unauthorized inventory/source/hash/diagnostic disclosure or editor launch. Preserve unrelated dirty documents/history. Overlap independent connections and existing local callers to exercise the actual shared editor slot, not only the adapter's single-call capacity: rejection is not queued/replayed and does not cancel another owner. Observe a proven refusal cannot later apply. Independently verify zero lost human work/unsafe overwrite/guessed target/lifecycle workaround and truthful known/partial/unknown results. Reuse prior direct guards when relevant inputs remain equivalent, while newly executing these MCP stateful races and any failure-driven regressions.

**Checkpoint:** US3 preservation and target-boundary behavior is independently established on the actual MCP path; T005 adds cancellation/delivery race combinations without weakening these checks.

## Phase 6: User Story 4 — Understand Failures and Interrupted Effects (P1)

**Goal:** Demonstrate effect-sensitive terminal results and recovery guidance through real process/protocol failures with original-clock bounds and surviving human/editor state.

**Independent test:** Interrupt open and both supported closed Resource profiles before and after authorization, while consuming or deliberately blocking/dropping output. Compare delivered facts or explicit delivery unavailability against independent surviving state and a fresh read of the original target where possible.

- [ ] T005 [US4] Implement the interruption, delivery-loss and deadline acceptance matrix in `godot-addon/tests/mcp_interruption_acceptance.py`, `godot-addon/tests/fixtures/mcp/` and `godot-addon/tests/run_mcp.py` as the `interruption` group; bundle fault-revealed fixes and regression tests in `mcp-server/src/mcp/transport.rs`, `mcp-server/src/mcp/handler.rs`, `mcp-server/tests/mcp_transport.rs`, `mcp-server/src/runner/closed_edit/` or `godot-addon/native/script_closed_edit.cpp`, and record result-carrier, survivor-state and timing evidence in `specs/007-mcp-script-workflow/quickstart.md`.

  **Dependencies:** T004; reuse its synchronization barriers/newer-work witnesses, existing owned fault-artifact conventions and T002's cancellation owner. No new generic cancellation/recovery/retry framework or persistent outcome service.

  **Scope and acceptance:** Cover US4.1–US4.4 and remaining US3.5–US3.6 combinations. Exercise cancellation, original-deadline expiry, unresponsive editor, EOF, SIGINT/SIGTERM, addon disable, worker/native channel loss, malformed private reply, output serialization/size failure and blocked/lost output before/after authorization and after known R/disk/partial/metadata effects. Include preserved newer human source/document state and stale lifecycle after entry. Fault artifacts remain separately identified and cannot ship in exports.

  Deliverable structured results preserve requested/resolved target, distinct MCP/domain correlation, reason/stage, availability/invalidation, effect certainty and safe next action; an editor protocol failure after effects is never invalid-client-input/not-applied. If SDK cancellation suppresses the response or output is lost, record delivery unavailable, not rollback/successful delivery/safe retry; independently inspect survivor state and freshly read the original explicit target before another deliberate action. Confirm bounded owned cleanup, no killed editor, no late effect after proven refusal, no queue/replay and no cross-call cancellation for duplicate/unknown IDs.

  Measure from accepted complete bounded frame before validation/routing through consumed-output serialization/flush: every read/discover ≤5 s and open/cached-closed/absent-closed edit ≤10 s, including blocked-editor cases; preserve 4.5/9.5 s work cutoffs and 0.5 s output reserve. Additional acquisition/validation never renews the budget. Separately prove finite output-backpressure/handshake/partial-frame/shutdown behavior; blocked-client tests are not consumed-output performance passes. With each selected real client, demonstrate model-visible limited/refusal and partial/unknown outcome/action interpretation under the selected carrier; no automatic retry or source-bearing error/log leakage.

**Checkpoint:** US4 and the interruption portions of US3 have effect-sensitive live evidence; cancellation/delivery behavior is not inferred from SDK acknowledgments or process exit alone.

## Phase 7: User Story 5 — Retain Native History and Durable Results in Real Agent Use (P1)

**Goal:** Complete the real-agent MCP A–E interaction and cumulative feature evidence without replaying unchanged historical campaigns or inventing closed-buffer history.

**Independent test:** At least one actual coding-agent conversation drives the full MCP discover/read/edit workflow; owned ordinary editor/fixture history and lifecycle actions independently establish A–E and durability. Both selected clients retain representative and carrier evidence from earlier tasks.

- [ ] T006 [US5] Deliver the composed real-agent A–E and cumulative acceptance workflow in `godot-addon/tests/mcp_composed_acceptance.py`, `godot-addon/tests/mcp_privacy_acceptance.py`, `godot-addon/tests/fixtures/mcp/` and `godot-addon/tests/run_mcp.py` as the `composed` and `privacy-export` groups; bundle directly required scenario/runner regression checks and any proven defect fix, then record complete coverage, implementation-shape/constitutional review and verified limitations in `specs/007-mcp-script-workflow/quickstart.md`, `specs/007-mcp-script-workflow/plan.md`, `specs/007-mcp-script-workflow/tasks.md` and `PROJECT_STATUS.md`.

  **Dependencies:** T005; all prior task acceptance remains valid after relevant-input review. This task owns the new composed interaction and final evidence coverage, not a generic full-suite rerun or delayed cleanup of earlier task complexity.

  **Scope and acceptance:** Cover US5.1–US5.5 and SC-005 exactly: A clean-open independent intended D/R/B and saved state; B dirty/equal-text-dirty safe refusal and human/history preservation; C actual native Undo → Save → Redo → Save with prior history intact, MCP reads and independent authority witnesses (unsaved history remains truthful, not falsely persisted or editable); D ordinary close/reopen for open edits and later opening for already-verified closed edits, plus applicable Save/reparse/rescan/fresh-runtime persistence; E **twenty successful fresh-read/revision MCP edits covering supported open and closed profiles, with at least three dirty and three stale refusals interleaved**. Use both cached and absent R closed coverage, preserve each edit's admitted lifecycle, and apply history only where a real open buffer exists. No second edit substitutes for Undo/Redo, and no ordinary witness repairs divergence or enables a closed edit.

  At least one real-agent conversation supplies the full composed MCP interaction, not an internal harness relabeled as agent use. Inspect actual tool transcripts, source/revision/state use, safe refusal/uncertainty actions, timings, live D/R/B/history/lifecycle records and owned captures. Keep prompt guidance limited to target/human intent; qualitative FR-022/SC-009 review must establish sufficient lean descriptions with no internal architecture coaching or numerical prose budget.

  Complete privacy/export evidence across authorized, denied, ambiguous, interrupted, startup, help, worker and error paths: no incidental source/inventory/credential leakage, including sticky disclosure denial after earlier causal failure. Verify affected enabled/disabled/hook-only production exports exclude addon/native/fixture/probe tooling. Retain local-only/no-telemetry/exact support and all existing local v1 contracts. Review every scenario, FR and SC as newly executed, validly reused, rerun after concrete invalidation or substantively inapplicable with reasons; missing observation is not inapplicability. Reuse T001–T005 and accepted historical evidence where relevant inputs match, and newly run composed/privacy-export plus affected regressions only. Do not turn this task head into a release candidate or automatically declare Phase 3 exit.

**Checkpoint:** Feature 007 may be recorded complete only with all six task acceptances, complete valid evidence coverage and applicable shape/constitutional gates satisfied. PR review/merge is delivery metadata, not feature correctness. Phase 3 exit and release remain separate decisions.

## Phase 8: Polish and cross-cutting concerns (owned by capability tasks)

No standalone polish, security-hardening, formatting or documentation task. Each task owns its directly required docs, discovered defects, applicable static/build checks and proportionate shape cleanup before it can complete. T006 owns cumulative coverage and current feature/task lifecycle updates, not unfinished implementation deferred from T001–T005.

### Verification and completion rules for every task

- Follow [TEST_POLICY.md](../../TEST_POLICY.md). Start with affected tests/scenarios; required permanent tests prove consumer-visible behavior, boundaries, state transitions, precedence or preservation, not source wording, wiring echoes or incidental defaults. Pair any discovered regression with failing-before/passing-after evidence where practical.
- Rust changes: run applicable formatting, Clippy `--all-targets --locked -- -D warnings`, `cargo test --locked` including doctests, rustdoc and actual affected consumer builds from `mcp-server/` under the pinned toolchain. No blanket `--all-features` or invented workspace. Native changes require affected build/load/behavior and provenance; wrapper/runner changes require affected selection/identity/cleanup checks. An unchanged native build or historical group does not require repetition solely because HEAD changed.
- Real Godot uses `godot-addon/tests/run_in_vm.py` on the dedicated macOS arm64 VM with committed source, independent witnesses, event barriers/deadlines and owned captures. No arbitrary sleeps, host GUI fallback or headless replacement for visible-editor evidence. Retain exact source/engine/platform/native/ABI/SDK/client/fixture identity and actual timings; retrieve artifacts and stop owned runs. Raw source-bearing transcripts and credentials stay outside Git.
- For wider regression execution, name the exact changed boundary, exact affected suite/group and concrete reachable invalidation. The v6/revision-4 owner/negotiation change warrants affected cross-language/loading/admission/cancel/export and representative existing-operation preservation, not automatic historical `all` campaigns. Rebuilt artifacts require demonstrated relevant equivalence or affected reruns. Maintain cumulative valid coverage, not a literal-final-commit replay.
- Complete each task's implementation-shape review under [AGENTS.md](../../AGENTS.md#implementation-shape-completion-gate): coherent ownership, narrow current-consumer visibility, no speculative APIs/frameworks, lifecycle enum rather than impossible boolean combinations, typed safety/evidence failures, and proportionate responsibility-based splits of newly enlarged modules. Review large wire/runner/native modules explicitly; do not mechanically split by line count or broaden unrelated refactors.
- Record applicable constitutional compliance in the existing plan/review/evidence: independent authority verification; human work/stale/lifecycle protection; native mutation; effect-sensitive results; real-editor A–E where applicable; protocol separation; export/privacy; lean interfaces; source provenance; justified present complexity. The plan's Principle XIII rationale covers the selected mechanisms; any additional mechanism needs its own concrete requirement, simpler alternative, insufficiency and ongoing-cost justification before introduction.
- Mark only the selected accepted task `[X]`, update material `PROJECT_STATUS.md` lifecycle truth and publish one focused task PR; stop. A completed task/feature is complete even with an open PR, but dependent work waits for merge unless stacking is explicitly authorized. No automatic merge, support expansion or next task.

## Requirement, entity and contract coverage

| Obligation | Implementation owner | Acceptance owner |
| --- | --- | --- |
| US1.1–US1.5; FR-001–FR-003, FR-016, FR-022; SC-001, SC-007, SC-009 | T002 connection/call, static tool descriptions, public schema/projections | T002 both-client/transport/lean-use proof; T003/T005 carrier edge cases; T006 composed qualitative/privacy review |
| US2.1–US2.5; FR-005–FR-010; SC-002 | T001 trusted capture/revision/checked expected state, native closed transaction; T002 all three tool mappings | T001 direct positive/safety coverage, T002 actual MCP positives, T003 real-agent lifecycle workflow |
| US3.1–US3.7; FR-004, FR-007–FR-010, FR-015; SC-003 | T001 routing/closed core and existing open guards; T002 enforced adapter admission | T004 preservation/target/race sequences, T005 effect/interruption combinations |
| US4.1–US4.4; FR-011–FR-015; SC-004, SC-006 | T001 effect reducer/supervisor, T002 call ownership/error/delivery | T002 direct boundary tests, T005 full interruption/carrier/timing matrix |
| US5.1–US5.5; FR-019–FR-020; SC-005, SC-008 | T001 native/runtime source durability; T002 MCP path; T003–T006 owned scenario capabilities | T001 new closed core evidence, T003 positive MCP closed, T004/T005 safety/faults, T006 actual-agent A–E and cumulative coverage |
| FR-017–FR-018, FR-021; SC-007–SC-008 | T001 matched cutover/export and unchanged local contracts; T002 local transport/disclosure/compatibility | T001 affected legacy/native/export evidence, T002 traffic/client evidence, T004/T005 denied/interrupted paths, T006 privacy-export/coverage |

All **26 acceptance scenarios, FR-001–FR-022 and SC-001–SC-009** have owners; cross-cutting safety/privacy/timing binds each applicable task, not only the table row. The data-model bounds and constrained fields are quoted in T001/T002 task descriptions rather than left to implementation choice. Existing observation/discovery/open-edit definitions remain authoritative for reused fields.

Contract ownership is explicit: `contracts/closed-edit.md` is implemented end-to-end by T001; `contracts/execution.md` by T001 core/private cutover and T002 adapter ownership; `contracts/mcp-interface.md` by T002. T003–T006 exercise those contracts through progressively composed scenarios, with any failure fixed in its actual owner. The fixed concrete clients and source profile come from research, not new choices during task execution.

## Dependencies and execution order

```text
Existing setup + approved design/current analysis + implementation authorization
  → T001 complete trusted read/revision/closed-edit capability
  → T002 US1 complete MCP connection/catalog/calls + both clients (MVP)
  → T003 US2 real-agent lifecycle workflows
  → T004 US3 preservation and boundary-race matrix
  → T005 US4 interruption/delivery/deadline matrix
  → T006 US5 composed real-agent A–E + cumulative coverage
```

US2–US5 share the actual MCP prerequisite and incrementally extend the same owned fixture/runner surface; they are independently runnable/assessable after prerequisites, not independent implementations that can safely expose partial safety. T004/T005 build on prior fixture barriers; T006 consumes valid earlier evidence and owns the new history/sequential composition. Story order follows the specification's equal P1 ordering. This dependency graph is not permission to implement the next node automatically.

## Parallel execution examples per story

No independent task-level `[P]` opportunity is claimed. Within the **one selected task**, after shared interfaces are fixed, independent file work can be divided as follows; one integration owner owns shared runner/native seams and final verification. Parallel contributors skip mid-flight broad builds/checks; collect all changes before running the task's integrated checks.

| Story/task | Safe intra-task example | Shared boundary / finish condition |
| --- | --- | --- |
| US1 / T002 | Adapter schema/catalog work in `mcp-server/src/mcp/schema.rs` alongside fixed relay command/testing work in `godot-addon/tests/run_in_vm.py` and `godot-addon/tests/test_run_in_vm.py` | Agree public contract and prepared-run receipt first; handler/VM guest integration has one owner; both clients then run against the same complete executable. |
| US2 / T003 | Lifecycle transcript driver in `godot-addon/tests/mcp_lifecycle_acceptance.py` alongside independent D/R/roster witness work in `godot-addon/tests/fixtures/mcp/` | Fix event/witness contract first; one owner integrates `run_mcp.py`; do not run racing campaigns against one editor. |
| US3 / T004 | Python target/preservation matrix in `godot-addon/tests/mcp_preservation_acceptance.py` alongside fixture-side human/lifecycle race barriers in `godot-addon/tests/fixtures/mcp/` | Specify barriers and independently observed outcomes first; integrate before acceptance and fix production failures in the same task. |
| US4 / T005 | Process/transport fault driver in `godot-addon/tests/mcp_interruption_acceptance.py` alongside independent survivor-state/fault witness cases in `godot-addon/tests/fixtures/mcp/` | Share original-clock/stage expectations; no competing edits to the cancellation owner; verify actual effect and delivery separately. |
| US5 / T006 | Composed history/sequential driver in `godot-addon/tests/mcp_composed_acceptance.py` alongside privacy/export driver in `godot-addon/tests/mcp_privacy_acceptance.py` | Reuse existing fixtures; one owner integrates runner/coverage; agent and export runs use isolated owned fixtures, not concurrent access to one editor. |

T001 additionally permits closed-domain deterministic tests in `mcp-server/tests/script_closed_edit.rs` alongside native witness implementation after basis/receipt semantics are fixed; shared protocol/native integration remains one owner's responsibility. These are contributors to one task, not new Spec Kit task IDs or additional PR units.

## Implementation strategy

### MVP first

Complete T001 and T002 separately with their own direct safety/behavior evidence and one PR each. T002's US1 checkpoint is the first external MCP MVP: exactly three working operations, including positive closed mutation, and two real-client representative passes. Do not call an advertised-only catalog or a universally refusing closed path an MVP. T003–T006 remain required for full feature acceptance.

### Incremental delivery

Each subsequent PR adds one independently runnable acceptance capability: real-agent lifecycle composition, adversarial preservation, interrupted-effect interpretation, then native-history/sequential A–E with cumulative privacy/export coverage. Elementary tests/docs and all safety enforcement remain with the production task that introduces them. Reuse accepted evidence after relevant-input review and rerun only affected portions. If a selected mechanism cannot satisfy its acceptance, report the concrete product/design blocker rather than silently weakening scope or adding an open-first fallback.

### Granularity and constitutional review

**Review status:** Passed on 2026-10-04 after independent story/coverage/granularity and model/native-contract reviews, plus parent integration. The review covered all 26 scenarios, 22 FRs, nine SCs, constrained model fields, meaningful PR contributions, dependencies and focused verification scope. Before publication, T001 was corrected to exclude only observation timestamps from revision hashing while retaining file mtime/ctime, and to quote the closed-history field constraint verbatim. No requirement, selected mechanism or acceptance obligation was weakened. This is task-granularity/contract review, not `/speckit.analyze` or implementation acceptance.

The reviewed decomposition has six meaningful increments, no file/symbol/dependency-only tasks, one prerequisite PR before the first connected slice, and tests/docs bundled with production owners. T001 spans Rust/addon/native because a partial closed family cannot safely advertise the capability; it is not split into serial layer PRs. T003–T006 add distinct stateful/real-agent validation capabilities mandated by the specification; they do not postpone the direct safety tests required for T001/T002 completion. Task-level parallelism is intentionally not asserted across shared fixture/integration files.

No new architecture, requirement, gate, support profile or dependency beyond the selected plan is introduced. Mandatory safety, exact field constraints, compatibility cutover, lean interface and evidence boundaries remain intact. Every task rejects unjustified historical whole-suite replay; the genuinely new composed A–E interaction belongs to T006. Formal `/speckit.analyze` is still a separate, not-yet-run stage.
