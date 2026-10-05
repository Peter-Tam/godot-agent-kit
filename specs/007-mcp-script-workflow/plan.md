# Implementation Plan: Use the Trusted GDScript Workflow Through MCP

**Branch:** `spec/007-mcp-script-workflow` | **Feature identifier:** `007-mcp-script-workflow` | **Date:** 2026-10-03 | **Spec:** [spec.md](spec.md)

**Input:** Maintainer-approved discover/read/edit requirements, including constitution v1.2.0 alignment in FR-022/SC-009.

**Status:** The four-task design passed [granularity/constitutional review](tasks.md#granularity-and-constitutional-review) and [consistency analysis](analysis.md); design PR #63 is merged. Feature 007 and T001–T004 are complete. T001–T003 were delivered in PRs #64–#66; T004's [actual-agent composed/cumulative acceptance](quickstart.md#t004-execution-evidence-2026-10-05) and implementation-shape review are complete on `task/T004-mcp-composed-acceptance`. Phase 3 exit and release are not claimed.

## Summary

Add one local stdio MCP executable to the existing Rust package, exposing exactly `discover_scripts`, `read_script` and `edit_script`. Use official rmcp 3.5.0 with the explicitly selected published MCP 2025-11-25 revision and a minimal Tokio runtime. Keep a concrete static catalog and bounded protocol transport above the existing authenticated, confined, supervised operation paths; public prose is short and sufficient, not the safety mechanism.

Read preserves Feature 001 observation semantics while presenting source, an opaque revision and useful current state. Edit accepts that revision, not a complete prior read object; execution freshly acquires and compares state before constructing its internal open/closed request. Open editing remains Feature 002 execution with the same open document and native history. New closed editing uses the unchanged narrow native main-thread source transaction: passive actual absence/cache checks, an explicit source setter only for existing clean R, confined retained-descriptor persistence and independent applicable-source/lifecycle verification. It never opens a target, force-loads R, relies on hoped-for reload or changes the old local edit caller's closed refusal.

[Research](research.md) resolves R1–R5 through current code/LSP, released primary sources and a scoped stock-editor mechanics probe. It does not claim a complete guarded closed-edit implementation. The probe also establishes the explicit limitation that source editing does not hot-reload an existing class implementation; fresh-runtime persistence and live-editor source coherence are distinct facts.

## Technical Context

**Language/Version:** Existing Rust 1.98.1 / edition 2021, thin GDScript editor integration, C++17 public GDExtension ABI and Python 3.10+ evidence tooling. No new application edition/MSRV claim.

**Primary Dependencies:** Retain locked serde 1.0.229, serde_json 1.0.151, cap-std 4.0.3 and ring 0.17.14. Later implementation adds exact rmcp 3.5.0 with default features off and `server`; Tokio 1.53.1 with default features off and `rt,time,sync,io-util,io-std`. No tool macros, HTTP/TLS/OAuth, client SDK, description framework or engine patch. [Provenance/license/advisory review](research.md#dependencies-and-provenance) records the SDK's mixed MIT/Apache transition and the still-required eventual locked-graph review.

**Storage:** Existing source-free owner-private registry; bounded request-local source/evidence and native retained descriptors/Script references. One integration-session close epoch for stale closed bases. No database, token/basis store, persistent outcome cache, replay service, callback history or generic registry.

**Testing:** Existing Rust deterministic/boundary conventions and owned real-Godot VM witnesses, plus required real **Codex CLI 0.153.4 / OMP 18.5.1** interoperability. Both installed versions were re-observed; [OMP suitability/carrier source evidence](research.md#8-codex-and-omp-client-correction-evidence) is not runtime acceptance. Task-owned checks and affected regressions follow [TEST_POLICY.md](../../TEST_POLICY.md); [quickstart](quickstart.md) records concrete scoped client setup, future commands and evidence limits. No automatic historical whole-suite campaign.

**Target Platform:** Official Godot `4.7.2.stable.official.ed1daf0bf`, commit `ed1daf0bf001b61586d9930840f2f1394092c079`, executable SHA-256 `c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf`; documented macOS 26.6.2 (25G83), arm64. The planning experiment used that guest profile. No broader engine/platform or running-game hot-reload support.

**Project Type:** Existing Rust library/CLI and editor addon/native bundle, adding its first external protocol adapter and a scoped closed-source capability below it. One new product binary, not a new workspace/crate/service.

**Performance Goals:** Original accepted-frame clock through validation, routing, operation, serialization and consumed-output delivery: five seconds for read/discover, ten for edit; existing work cutoffs 4.5/9.5 seconds. Closed timing and the new adapter overhead require new evidence. Handshake/partial-frame completion are bounded separately; no step renews an operation budget.

**Constraints:** Explicit authenticated project/session selection, confinement, current state-bound basis, dirty/newer-work preservation, admitted lifecycle, independent applicable authorities, truthful effects and disclosure. Closed B/history are not applicable, not invented. SDK cancellation/shutdown and tool-description prose cannot replace core/native guarantees.

**Scale/Scope:** Exactly three tools, one admitted tool execution per connection, no mutation queue. Existing source and operation profiles/limits; [model bounds](data-model.md#2-supported-profile-and-bounds) add finite framing/ID/output capacity, not numerical concision targets. Actual editor admission remains shared across all callers.

## Constitution Check

*Gate: evaluated before Phase 0 research; re-evaluated after Phase 1 design.*

**Initial disposition:** PASS for bounded primary-source/repository research and owned synthetic-editor experiments under the existing architecture. The maintainer approved v1.2.0 before this invocation. No constitutional exception or closed-edit feasibility assumption was authorized; R1–R5 were recorded before research.

**Post-design disposition:** PASS for planning against approved requirements and constitution v1.2.0. The native composition has source-established boundaries and observed positive mechanics; its complete guards, races, effects, timing and real-agent acceptance remain mandatory implementation proof. No unresolved product choice or constitutional MUST violation is being excused by future work. If construction cannot satisfy the selected source/lifecycle contract, return to the product/spec decision before dependent implementation continues.

| Obligation | Selected design and required evidence |
| --- | --- |
| I / IV — coherence and verified success | Independent D/R/B for open; independent D/applicable R and actual continuing B absence for closed. Native receipts, disk writes, SDK delivery and model claims are not success. |
| II — human work and stale intent | Existing open basis/dirty/history guards; new file mtime/ctime, Resource branch/identity/edited state, target absence and close epoch. No dirty-flag clearing, reassertion, branch switching or lifecycle compensation. |
| III — native authority | Existing open native transaction retained. Closed source setter and confined writer execute under Godot main-thread ownership, with passive absence admission and independent verification; no external production writer or loading a Resource to create evidence. |
| V — least privilege | Local stdio and existing authenticated registry/routing/confinement. No network transport, remote product mode, arbitrary command/evaluation, force, telemetry or new approval layer. |
| VI / XII — actual behavioral proof | New closed positive/race/refusal/interruption and MCP A–E/agent evidence, with applicable Save/reparse/rescan/runtime durability. Reuse valid unchanged evidence; rerun changed boundaries rather than every historical suite. |
| VII — protocol independence | MCP DTO/framing at adapter; checked read/closed intent, safety and outcomes below it; Godot APIs at addon/native integration. Existing supervised operations are reused, not reimplemented in MCP. |
| VIII — tooling separation | Matched addon/native/fixture additions stay out of production exports; enabled/disabled/hook-only evidence applies to affected artifacts. No gameplay authority. |
| IX — lean effective interface | Three familiar static operations; read returns source/revision/state and edit takes revision/replacement. Required observation meaning survives a deliberate projection, not an internal evidence envelope for echoing. Structured outcomes and targeted actions; no architecture lecture, defensive prose copies or quantitative description budget. |
| X — actionable truthful diagnostics | Preserve outcome/stage/application, independent availability/invalidation and safe next action. Post-effect private protocol/host failure is not malformed input; disclosure denial is sticky. |
| XI — independent compatible implementation | Public pinned engine source/behavior, released official SDK and concrete clients, license/provenance/advisory findings. No unrelated implementation copied or dependency/support pass invented. |
| XIII — justified complexity | Narrow closed owner/epoch, bounded transport and test-only stdio relay address specified failures; rationale/cost below. No transaction/workflow/retry/cancellation/redaction/tracing/metadata framework. |
| Compatibility / workflow | Existing local v1 APIs/refusals stay valid; private v6/native revision 4 and MCP schema 1 are unchanged. The native-documentation correction passed fresh analysis; later completion evidence changes no obligation. T001–T004 are complete, delivered one task/PR. |

**Threat/evidence boundary:** Retain existing normal-local-user threat model; hostile same-UID software and malicious code already running inside Godot are not newly claimed isolated. No atomic filesystem exclusion, universal callback purity or live bytecode refresh is claimed. Unknown safety facts refuse. The observed source setter's old compiled constants are explicitly not used as new-source runtime evidence.

## Execution design

### Connection and public operations

The [MCP contract](contracts/mcp-interface.md) selects initialization/version negotiation, static tool schemas, authoritative `structuredContent`, protocol-versus-operation error mapping and bounded transport. Full text duplication is not the default; a required fallback must be justified by selected-client evidence and recorded as a compatibility cost. Registry is required deployment configuration; each tool explicitly names its intended project/session/target. Server catalog support is distinct from actual session/script eligibility.

The custom transport is only bounded framing/parsing/delivery around SDK types/service. It prevents source-bearing SDK tracing, unbounded input and silent parse failures identified in released source. The protocol executor remains responsive while the existing bounded synchronous supervisor runs off-executor; the active work handle is retained through cancellation. EOF/output loss means delivery unavailable, not no effects or permission to replay.

### Trusted read and fixed lifecycle branch

The [model](data-model.md#4-trusted-script-read) presents exact source, meaningful editor-aware state and a bounded opaque revision. Existing `ring` computes a stateless commitment to the checked project/session/target, source, lifecycle and relevant open-version or closed-revision/Resource/epoch facts. It is a stale-intent precondition, not authorization, replay/idempotency or a persistent basis ID. Closed metadata remains internal and is obtained without opening/loading, under the same capture clock and cross-checks.

Edit freshly authenticates/resolves the target and reacquires all required current state. Only a matching recomputed revision and current eligibility allow construction of the internal request from that exact frozen capture; stale/changed state requires `fresh_read`. Open requests retain Feature 002's checked basis and runner, with private observation-shaped worker input assembled from the fresh server capture. Closed requests retain the selected checked basis/request/runner. All later native/core guards and independent postconditions remain mandatory under the original edit clock; no changed lifecycle selects the other branch.

### Closed source transaction

The [native contract](contracts/closed-edit.md) defines admission, source/effect validation, current state/epoch/namespace rechecks, one irreversible authorization, explicit R setter for present clean R, retained-fd persistence/T0 restoration and independent post-validation/verification. An absent R remains absent, not force-loaded. Dirty/newer state after entry stops later writes with truthful partial/unknown effects, not rollback.

The counter on public close notifications closes the closed→open→closed basis gap without a per-target history registry. Read/prepare/verify still independently inspect actual target absence and Resource state. The source-only helper and old confinement utilities are reused; no live reload/parser/Save fallback is selected. Closed history is not applicable; open native history remains required.

### Compatibility and completion evidence

[Execution ownership](contracts/execution.md) specifies native revision 4/private bridge v6, one complete `edit_closed_gdscript` capability and all-consumer cutover. Do not make old edit callers accept closed scripts or alter their v1 outputs. Install peers together and obtain a new session/basis; no private fallback remains.

The [acceptance matrix](quickstart.md#acceptance-coverage) covers all 26 approved scenarios, 22 FRs and nine SCs. Real clients must demonstrate actual protocol/tool use; at least one real-agent conversation supplies composed MCP A–E with independent witnesses. The test-only relay keeps the production server/editor local to the owned VM while client credentials stay on the host. Its setup/relay are fixed commands in existing test tooling, not general remote execution.

## Project Structure

### Documentation (this feature)

```text
specs/007-mcp-script-workflow/
├── spec.md
├── checklists/requirements.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── tasks.md
└── contracts/
    ├── mcp-interface.md
    ├── execution.md
    └── closed-edit.md
```

[tasks.md](tasks.md) derives four coherent implementation increments: protocol-independent trusted read/revision/closed editing; complete external MCP MVP with both clients' positive workflows; combined adversarial preservation/interrupted-effect behavior; composed real-agent A–E and cumulative acceptance. Direct tests/docs remain bundled with their implementation. All four tasks are complete.

### Source code (repository root)

These are planned implementation locations, **not scaffolds created by planning**:

```text
mcp-server/
├── Cargo.toml / Cargo.lock                    # selected SDK/runtime and one binary
├── src/
│   ├── bin/godot-agent-kit-mcp.rs             # executable and internal worker wiring
│   ├── mcp/                                  # concrete handler, DTO/catalog, bounded transport
│   ├── script_read.rs                        # trusted capture, revision and public read projection
│   ├── script_closed_edit/                   # checked intent, evidence and outcome reduction
│   ├── runner/read/                          # reused acquisition plus private closed supplement
│   ├── runner/closed_edit/                   # bounded supervisor and acquisition/authorization
│   ├── bridge/wire/                          # v6 closed capture/operation codecs
│   └── lib.rs                                # current-consumer semantic/runner exports
└── tests/                                    # focused adapter/read/closed semantic boundaries

godot-addon/
├── addons/godot_agent_kit/                    # thin read/closed owner + private transport wiring
├── native/
│   ├── script_closed_edit.cpp                # closed native state/effects/verification
│   ├── document_guard.cpp/.hpp               # reuse exact descriptor/document primitives
│   ├── editor_context.cpp/.hpp               # applicable source/effective-context reuse
│   └── native.hpp / session.cpp / extension.cpp / build.py
└── tests/
    ├── run_mcp.py                            # focused protocol/closed/composed evidence
    ├── fixtures/                             # owned synthetic human/race/independent witnesses
    └── run_in_vm.py / vm_guest.py             # fixed mcp suite + prepare/stdio relay
```

**Structure decision:** Retain one Cargo package and existing addon/native/test boundaries. Adapter transport, DTO projection, call ownership and closed-domain evidence have distinct responsibilities; do not append them to a large unrelated reducer/wire/native file just because they share a feature. Reuse or narrowly extract a helper only for concrete current consumers, without changing existing public local contracts or creating generic dispatch traits. New types use the narrowest required visibility.

## Complexity Tracking

No constitutional exception is requested. The template's violation table is inapplicable; the current-cost review required by Principle XIII is recorded instead.

| Addition / concrete need | Simplest alternative and why insufficient | Ongoing cost / why justified now |
| --- | --- | --- |
| Official SDK/minimal runtime | Existing CLI has no external MCP protocol; handwritten MCP duplicates versioned protocol maintenance. | One pinned SDK/runtime feature graph, locked license/advisory/compatibility review; required by actual client interoperability. |
| Small bounded transport | SDK convenience stdio buffers unbounded lines, can log payloads and ignores syntax errors. | One bounded framing/delivery owner and focused boundary tests; required by FR-003/FR-014/FR-017, not generic hardening. |
| Closed domain/native owner and read supplement | Open edit deliberately refuses closed targets; disk-only writing demonstrably leaves cached R stale. | One explicit source/lifecycle state machine with primitive reuse and new evidence; directly required by FR-007–FR-010. |
| Session close epoch | Current absence alone misses closed→open→closed between read and effect. | One owned signal/counter with lifecycle cleanup and ABA tests; avoids a per-target history registry. |
| Stateless revision and fresh expected-state acquisition | Echoing the whole read exposes internal evidence and moves redundant context; a persistent token store adds ownership/replay costs without enforcing safety. | One fixed state-commitment encoding using existing hashing and acquisition, with stale/race checks; no keys, cache or generic token layer. |
| Fixed test-only stdio relay | Host client plus guest-only editor cannot share ordinary local registry/path execution; moving personal client credentials into guest violates its setup boundary. | Small reuse of existing Tart control/source/run ownership; enables FR-019 without host GUI or a product remote service. |
| Static lean schemas/descriptions | A small catalog alone permits redundant instructions; a description DSL adds no safety. | Deliberate current text and ordinary review/real-agent evidence only; no extra process gate or measurement framework. |

**Design-shape review:** Each new production responsibility has a current caller and bounded state. One attempt enum represents lifecycle; effects/receipts retain factual partial progress rather than speculative state variants. Existing large modules receive only their necessary integration seam, with narrow responsibility-based files for the new behavior. No permanent research scaffold, generic framework, unguarded public mutation API or additional approval/CI topology is selected.

**Planning correction (2026-10-04):** The maintainer requested the public source/revision/state contract in place of the original whole-read echo. [Research](research.md#7-public-contract-correction-evidence) records exact-client carrier source findings and incomplete runtime probes. Constitution IX/XIII review preserves the approved specification, native closed route, selected protocol/dependencies and all enforced safety boundaries. Actual two-client result visibility remains an implementation acceptance obligation, not an interoperability claim at this design head.

**Task/client correction (2026-10-04):** The maintainer requested consolidation from six pending tasks to four and selected Codex + OMP instead of the earlier client pair. Installed version/help and source inspection establish OMP's stdio/2025-11-25/tool/model-result path; actual client use remains required. The preferred single structured-first carrier is unchanged. OMP's project config is temporary and additive, normal non-bypass approval is retained, and client-local cancellation, reconnect/resend and output-spill limits are explicit in research/acceptance. Temporary config ownership addresses the present need to attach the fixed relay without overwriting human project settings; a global config mutation or a new isolation/client framework is unnecessary. The existing LSP setup, specification, closed transaction, revisions, protocol/dependencies and Features 001–005 remain unchanged. This correction does not run analysis or implementation.

### T002 fixture ownership refinement

The fixed relay keeps prepared projects alive across connection EOF: tearing them
down at EOF would invalidate client reconnects and prevent independent later
durability checks. Existing artifact retrieval is deliberately non-destructive;
stopping the entire VM is not a substitute for those checks.

`finalize-mcp --run-id ID` is the smallest explicit terminal action in the existing
wrapper. It verifies the owned workflow and later durability, cleans up its
fixtures, and retrieves evidence, including on failure. Its cost is one fixed
allowlisted action using the existing receipt/control/cleanup boundary; no arbitrary
command/path, product remote mode, new dependency or generic orchestration service
is introduced. This implements T002's existing bounded-cleanup/independent-witness
obligations without changing product scope or later task acceptance.

The first actual MCP fixture run kept twelve owned editors alive concurrently.
Its 512 KiB changed edit persisted all bytes but exhausted verification time
(`applied_unverified`, 9.108222 s); other source/lifecycle positives completed.
Those independent profiles do not require twelve simultaneous editors.
The fixture now runs fixed `workflow`, `known`, `sources`, `bound` and
`observations` groups, with the bound case alone. Each selected client owes all
five groups in separate conversations. Separating `known` removes the shared
exact locator from discovery-first context (SC-002); its cost is one fixed
selector/conversation, not a new runner. `observations` includes one actual
barrier-invalidated read using T001's closed-state barrier and same-text disk
transition. Historical D and null revision are checked against independent
pre/post external-change witnesses; read no-effect checks retain the post-change
baseline. Stable unavailable/dirty cases cannot establish invalidation (US2.3,
FR-005). Reusing the existing fixture costs one fixed profile and barrier
handoff, not native production changes or a synthetic MCP result. The bound
prompt describes the requested comment instead of preloading half a MiB.

The first Codex run exited without tool calls but left an idle guest relay alive.
Sequential control handling then prevented `finalize-mcp` from reaching the owner;
the 180-second control wait failed. The existing relay's `select` now also watches
the fixed control listener, so finalization drains the owned MCP process before
validation/fixture cleanup even when the client never supplies EOF. This reuses one
event loop rather than adding a thread/service or treating client exit as success.
Its cost is one explicit control handoff and a live-child/socket regression.
No product deadline, transaction, source authority or mutation behavior changes.

**Acceptance correction:** US2.5 requires the agent, not a helper-only MCP
client, to read after ordinary fixture opening. `prepare-durability --run-id ID`
first validates all primary workflow results while cached/absent targets remain
closed, then opens only those verified targets and emits a read-only prompt.
The owner persists for a second invocation of the same selected real client.
Finalization requires recorded post-opening reads before independent
Save/reparse/rescan/runtime checks and terminal cleanup. Opening cannot manufacture
primary success; helper-only reads no longer substitute for model consumption.
This costs one fixed allowlisted fixture action and one read-only invocation per
client's workflow profile; it reuses receipts/control and does not introduce a
public lifecycle tool or take T004's composed-conversation scope.

The host's single existing operation lock is shared narrowly for long-lived
relay/control traffic and exclusive for setup/source-changing operations.
Exclusive locking of the relay prevented even the fixed finalizer from reaching
the guest listener; dropping locking entirely would allow source/setup races.
Shared mode preserves that exclusion at the cost of one mode argument and a
wrapper-level overlap regression, not new lock infrastructure. Guest immutable
worker/source/build checks and active-owner exclusion remain intact.

OMP's first model-visible workflow exposed the inherited `scripts/.gdignore`:
discovery correctly excluded the target, so that fixture could not prove discovery
of the edited script. Exposing the scripts before startup then correctly failed the
independent absent-R admission check: normal indexing had loaded the target.
MCP preparation now retains cold setup, focuses the selected fixture and waits for
its initial scan with the marker still present, then removes that owned marker
before forwarding its first discovery. The actual inventory must contain the
target, and every cold-profile call independently requires absent R before and
after. No Resource is unloaded and no LSP or product guard is disabled. Positive
prompts supply a filename hint, not an exact script path, as SC-002 requires.
Repeat affected profiles; earlier carrier/edit observations do not substitute for
this corrected discover-to-edit acceptance.

### T002 adapter ownership corrections

Review exposed three reachable mismatches with rmcp 3.5.0: `serve_directly`
left peer state at the requested rather than negotiated revision; SDK cancellation
suppressed completions needed to retire transport IDs; and its select loop could
drop receive-side replies while output was backpressured. The adapter now records
the negotiated revision, translates cancellation into the existing request-owned
flag while retaining completion/supervisor ownership, and retains one bounded
receive-side reply through consumed-output acknowledgment. The original deadline
and `Arc` owner identity survive polling and delayed completion. This costs one
pending-reply record, not another scheduler, queue or cancellation framework.

Unavailable/invalidated open-edit surfaces legitimately carry null private
identities. Projection removes those private fields too, while retaining the
separate public document identity; otherwise valid refusals and post-effect
evidence became generic `invalid_output`. No public schema, operation, authority,
mutation route or product deadline changes.

### T002 bounded worker-channel correction

The isolated 512 KiB MCP edit exposed avoidable local IPC delay: source-free
diagnostics measured 610 ms and 64 polling sleeps sending its 525 KiB startup
frame, plus 632/638 ms returning pre/post-validation source frames. Serialization
took only 13–14 ms; the editor's bounded request transfer was about 127 ms.
The resulting verification timeout is a reachable T002 supported-bound failure,
not justification to increase deadlines, frame budgets or source limits.

Reuse stock validation's existing bounded blocking Unix-socket pattern for the
closed worker channel: each read/write waits at most 50 ms or the remaining
original deadline, whichever is smaller, then checks cancellation/deadline again.
Both endpoints remain owned off the protocol executor. One shared private
frame decoder replaces the closed supervisor's sleep-based reader; no dependency,
unsafe binding, new worker, queue, cache or protocol change is needed. This
costs a small bounded-read loop and stalled-peer regressions, while removing the
separate polling path. Cancellation, effect retention, independent verification,
native mutation, confinement and original monotonic cutoffs remain unchanged
(Principles I–V, VII, X and XIII).

The final adapter review also found response-delivery races: a client could send
`initialized` after reading the initialize response but before the SDK processed
its write acknowledgment, and a blocked receive-side reply prevented cancellation
or EOF from reaching an active supervisor. Successful writer delivery now owns
the initialization transition. Receive-side reply retention is bounded by the
existing eight-ID capacity while cancellation/EOF remain observable; queued
replies are not queued tool operations. Keeping one awaited reply is simpler but
cannot meet both normal pipelining and timely cancellation. The additional bounded
reply state and deterministic channel-ordering regressions are justified by those
reachable failures; no larger capacity, protocol, operation queue or scheduler is
introduced.

The output review corrected two contract mismatches without changing the public
schema decision: wall-clock endpoints may move backwards while elapsed time remains
monotonic, and confirmed closed B must use `not_applicable_closed`, not the core's
internal `not_applicable` spelling. Projection now translates that value and keeps
unconfirmed B unavailable. Strict numeric interval types and post-effect certainty
remain checked. These are adapter corrections, not new core authority or mutation
semantics.

### T003 adversarial acceptance execution plan

Select only T003 after T002's merged PR #65. Recomputed fingerprints recover the
original analyzed baseline `c8dc7683409ca72ac0d7feb215cd3045dd6f7ed9`; cumulative
changes record T001/T002 implementation, delivery and evidence, without changing
remaining requirements. The original analysis remains current.

The concrete gap is public-path evidence at conflicting-state, effect and delivery
boundaries, including each selected real client's interpretation. Reusing the
legacy supervised fixture consumer would not exercise MCP correlation, projection
or delivery. The smallest additional apparatus is one bounded actual-stdio peer,
separately runnable preservation/interruption drivers using existing independent
editor witnesses, and fixed failure/reconnect profiles in the existing prepared
relay. A reply-loss profile must retain the actual undelivered server result and
survivor evidence without claiming the client received it.

These additions cost maintenance of task-specific scenarios and one shared
process/framing helper, not a second product protocol, generic orchestration
framework, replay service, new dependency, native family or public API. Existing
fixture-only barriers/callbacks and fault artifacts provide negative events;
malformed framing corrupts a genuine private reply rather than fabricating source
or successful effects. Production and fixture artifact identities remain separate.

Newly execute the task-owned preservation/interruption interactions and both
clients' limited/refusal/partial/unknown/reconnect interpretations. Run affected
transport/output and fixture/VM-selection tests and applicable Rust checks. Reuse
unchanged native/ABI, positive lifecycle, history and export evidence after
relevant-input review; an actual production fix must identify its reachable
invalidations. T004's composed A–E and cumulative feature decision stay pending.

Constitutional planning preserves independent authorities and live witnesses
(I/IV/VI), newer human work and lifecycle (II/III), local explicit routing and
confinement (V), core/native safety ownership (VII), fixture export separation
(VIII), structured targeted failure guidance (IX/X), and the existing independent
implementation/dependency provenance (XI). No weaker guarantee or expanded support
is selected; concrete behavior, not additional process ceremony, earns acceptance
(XII/XIII).

### T003 native dirty-Resource race correction

The actual MCP `apply`-barrier case changed an open Script's Resource to
equal-text-but-dirty after preparation. The native guard checked that flag only
after finalization; its separate removal callback guard detected it only after
buffer removal. The delivered result was truthfully `applied_unverified` /
`partly_applied`, but a known dirty Resource requires refusal before the first
effect (T003 US3.1/US3.4, FR-007/FR-008, Principles II–IV).

The correction belongs in the existing native open-edit guard, not MCP policy,
an alternate writer or a new observer service. Check the actual Resource dirty
state at the guarded mutation boundaries; retain the same target/version/source,
descriptor, context and synchronous callback protections. A focused real-editor
regression records independent surviving D/R/B, dirty state and history before
asserting the outcome, including later-stage dirty transitions and sticky effects.
The change adds no API, native family, dependency, replay or permission surface.

Rebuild/load both production and separate fixture native artifacts. Newly prove
the failing guard, affected clean open edits/native history, pre-effect races and
open interruption stages, plus a representative existing local open-edit caller.
Closed native mutation, transport, authentication, ABI and export mechanisms are
unchanged; review relevant equivalence and retain their valid coverage rather than
automatically replaying historical suites. Subsequent T003 closed cases also
exercise matched loading against the rebuilt native bundle.

The first corrected run established pre-effect refusal with unchanged buffer,
disk, Resource and history. At the later `buffer_applied` boundary, ordinary
ScriptEditor validation independently copied already-visible B into R while
the result retained native Resource synchronization and persistence as
`not_started`. The witness must not mislabel that editor propagation as another
native write. The shared comparison permits only R becoming that exact existing
B, retaining strict D/B, dirty-state, history, roster and identity equality;
raw observations/results remain unchanged. Waiting for or forcing reconciliation
would hide the transition instead of witnessing it. This is one bounded comparison
used by the dirty-Resource and interruption cases, not a new safety policy.

### T003 implementation-shape and constitutional review

The production change remains in the native open transaction's existing guard:
repeat the already-required per-Resource dirty check before every stage instead
of adding an adapter refusal after damage. It introduces no declaration, public
visibility, lifecycle boolean, speculative variant, dependency or allocation
layer. The native module's coherent responsibility remains the open document
transaction; this small change does not justify splitting unrelated existing code.

Test ownership is explicit: `mcp_peer.py` owns the real stdio peer, correlation,
public result review and independent comparison; preservation and interruption
modules own their separate adversarial behaviors; `mcp_failure_acceptance.py`
owns the two prepared real-client fault profiles through the existing relay.
Fixture-only callbacks perturb/witness real effects and remain separate from
production native artifacts. No generic cancellation, retry, recovery, outcome
storage or new workflow service was introduced. Helpers have current consumers;
no production API is exposed for T004. The longer interruption module contains
one related scenario family, not a new production transport/state/persistence layer.

Runtime review corrected fixture assumptions instead of changing valid product
contracts: cache disappearance must be real; an occupied native owner cannot be
reconfigured; acquisition/legacy deadline names retain their existing meaning;
normal editor B-to-R propagation is not a subsequent native stage; and output
failure is held at the genuine partial apply receipt. Raw outcomes/witnesses
remain intact. A focused witness regression rejects unrelated/third-source
changes rather than masking arbitrary differences.

New evidence applies Principles I/IV/VI to independent D/R/B or closed authorities;
II/III to dirty/newer work, lifecycle and usable native history; V/VIII to local
authority, confinement and unchanged export separation; VII to the unchanged
adapter/core/integration boundary; IX/X to meaningful structured uncertainty and
source-free diagnostics; XI to unchanged public-API/dependency provenance; and
XII/XIII to a concrete fault-driven guard repair and reused test boundaries.
Observed client reconnect limitations are documented, not hidden by a product
retry or prompt-based safety mechanism.

All task-owned preservation/interruption groups and both clients' failure/reconnect
profiles passed with [exact evidence and relevant-input review](quickstart.md#t003-execution-evidence-2026-10-05).
The public contract, approved remaining requirements and task granularity are
unchanged. These implementation/evidence updates do not invalidate the original
analysis attestation or authorize T004, broader support, release or Phase 3 exit.

### T004 composed acceptance execution plan

Select only T004 after T003's merged PR #66. The six analysis fingerprints were
recomputed and their exact original bytes recovered from
`c8dc7683409ca72ac0d7feb215cd3045dd6f7ed9`. Cumulative feature-directory changes
record completed implementation, client/dependency evidence and delivery; T004's
scope, acceptance and dependencies are unchanged. The original analysis remains
current, and the 16-item requirements-quality checklist remains read-only and passed.

The remaining requirement is a real coding-agent composed A–E interaction plus
cumulative privacy/export coverage, not another product capability. Extend the
existing owned MCP relay and independent editor witnesses with a composed profile
and a separate privacy/export driver. Keep runner/VM dispatch under one integration
owner; independent drivers use separate owned fixtures. Existing fixture history,
Save, lifecycle and runtime actions witness already-observed outcomes and never
repair a product result or make a closed edit eligible.

A protocol-only loop would be simpler but cannot satisfy US5/SC-005's actual-agent
requirement. The selected additions cost two capability-specific acceptance
drivers and narrow fixed-profile integration, reusing existing ownership, source
transfer, client configuration and artifact retrieval. No new service, dependency,
product tool, generic workflow framework or approval bypass is justified.

New evidence must establish twenty fresh-revision successes across open,
cached-Resource closed and absent-Resource closed profiles, with at least three
dirty and three stale refusals interleaved; actual prior-history-preserving
Undo/Save/Redo/Save, ordinary later lifecycle/durability, and privacy/export.
Run affected driver/selection regressions and actual VM scenarios. Reuse accepted
T001–T003 and historical evidence after relevant-input review; add Rust/native
checks only if an actual change invalidates their execution or provenance.

Constitutional review retains independent applicable authorities (I/IV/VI),
human/history/lifecycle preservation and native effects (II/III), existing local
permissions/confinement and export isolation (V/VIII), protocol-independent safety
(VII), lean descriptions and truthful outcomes (IX/X), existing provenance (XI),
and focused behavioral evidence without speculative infrastructure (XII/XIII).
Feature completion requires every remaining acceptance obligation; Phase 3 exit,
release, broader support and another task are not selected.

#### T004 fixture scheduling and failed-witness corrections

Initial runs retained all three composed editors after their individual workflows
had finished. Closed edits sometimes exhausted post-source-validation time; one
separate completed-native trace ended with disconnection. These failed attempts
remain non-acceptance evidence, not proof of a single scheduling cause.

Each composed profile owns an ordinary nested fixture context and closes its
editor only after its final actual MCP read and independent lifecycle/durability
witness. The next profile is created only after that shutdown; its session does
not exist or get advertised in advance. The finalizer checks completed reads and
persisted source after owned shutdown. This removes unrelated editor workload
without reducing the twenty
edits, three dirty/three stale refusals, authority/history checks or one-conversation
requirement. Keeping completed editors alive adds no required interaction;
changing product deadlines, retrying edits or adding a scheduler is not justified.

An experiment suspended future prepared profiles while the current profile ran.
An open edit still reached post-validation unavailability. That experiment did
not establish a remedy and was removed; no suspended-editor scheduling remains.

The ambiguity witness also exposed `loader_calls: 3 → 4` during initial editor
setup while every recorded source/identity/history surface was unchanged. Both
owned windows and their existing index/idle barriers now settle before the
preservation baseline. The loader-count assertion remains strict. Failed
no-effect comparisons now retain their before/after witness before asserting,
instead of losing the diagnostic evidence. Individually runnable privacy groups
reuse the existing scenario-selector pattern so failures do not replay completed
export or interruption work. Temporary source-free diagnostics are removed;
no production API, policy, timeout, schema or native transaction changed.

#### T004 serial-fixture correction

The unchanged four-vCPU guest returned bounded, truthful post-validation
non-successes during the composed sequence. Source-free measurements attributed
3.7–4.8 seconds to concurrent preflight engine startup, about 40–52 milliseconds
to listener ownership, and about two seconds to the post-change engine startup.
The compatibility-renderer experiment did not improve those measurements and was
removed with the temporary diagnostics.

The maintainer directed that the VM remain at four vCPUs. The attempted resource
comparison still observed four CPUs; no six-vCPU execution evidence exists, and
the experimental default is reverted. CPU allocation remains part of the actual
recorded acceptance environment identity.

First remove unnecessary test-level concurrency. Independent T004 fixtures and
cases run serially; retain simultaneous editors only where the scenario explicitly
requires overlap, such as ambiguous-session routing. Preserve the product's
concurrent original/desired preflight validation and every 5/10-second bound.
Remeasure on the existing settled four-vCPU guest. A resource-profile change may
only be proposed if an isolated real operation remains unable to reliably meet
the existing timing contract; it is not selected or authorized now.

The composed owner now prepares one profile at a time and verifies that its
owned editor roster contains exactly that editor. It records the live PID and
session. Prompt/receipt intents declare project paths and requested values, not
nonexistent future sessions; normal public project routing selects the sole
current session. Only after the final independently verified client read and
owned shutdown does the next fixture start. No editor is paused or reconstituted
mid-profile. Privacy's two-editor selection/ambiguity cases deliberately retain
their live competing owners; independent privacy groups and export modes are
already serial.

Implementation review found two acceptance-oracle gaps, reproduced with focused
failing regressions. Save must preserve pre-Save B, not adopt a coherently reverted
post-Save source as the expectation. Denied/ambiguous/interrupted privacy checks
must inspect the complete retained MCP frame, including text content and source
hashes, not only structuredContent. The corrected assertions add no product
behavior, retry, timeout, service or authority. Rerun the composed interaction and
affected privacy selection/interruption checks; unchanged export evidence remains
reusable.

#### T004 product-validation optimization direction

The maintainer now requires the acceptance VM to remain at four vCPUs; a
six-vCPU allocation is not the fix. The serial single-editor failure locates the
remaining bottleneck in required product behavior, not fixture orchestration.
Profile startup, original/desired validation, mutation, fresh actual-state
acquisition, final validation and completion before selecting an optimization.

Use temporary source-free phase timestamps and, if needed, an owned stock-child
startup sample. These diagnostics are not product telemetry or acceptance passes
and will be removed after the measurements. Retain the same official compiler,
per-source completion fences, current revision/context/state checks, independent
post-mutation acquisition/validation, real original/desired preflight concurrency,
all cleanup and disclosure guarantees, and the existing ten-second response bound.
No retries, deadline extensions, weaker acceptance or support expansion is allowed.
Any selected mechanism must record its measured requirement and Principle XIII
tradeoff here before implementation; run only its affected verification and the
required T004 composed interaction afterward.

#### Timing boundary and readiness requirements

The edit response contract remains **ten seconds from the original accepted
complete request frame through delivery of the complete trustworthy response**.
That clock includes argument/admission checks, fresh state/revision acquisition,
all source-specific preflight, mutation, independent actual-source verification,
cleanup and output. Preflight is not an untimed preparation phase. Work started
after request receipt is never subtracted, and a pending request is not reaccepted
under a new clock.

Validator readiness is a distinct lifecycle, not a different name for preflight.
If cold initialization is removed from the edit critical path, it must finish
before edit admission, under the existing **ten-second initialization/control
bound**. Readiness includes official-binary verification, isolated process and
namespace creation, compiler initialization and an observed usable channel.
Failure or expiry means unavailable readiness, not permission to mutate and
initialize later. Requests arriving before readiness may not be hidden in an
unbounded queue or silently retried. A one-shot caller that initializes only
after invocation must continue counting that initialization in its total bound.

Record two intervals separately: **readiness start → observed ready/failure**
and **accepted steady-state edit → complete trustworthy response**. The latter
is meaningful only when the required validators were already ready before the
request arrived. Retain full cold end-to-end timings for any cold caller; do not
label a duration with startup subtracted as measured steady-state latency.

The current implementation has no such readiness lifecycle. Its fresh processes
are launched inside each request and all their time is correctly still counted.
Source `4c61e16`, diagnostic run
`validation-profile-1/20261005T110624Z-2ec94b1f9888`, measured serial isolated open
and closed edits on the settled four-vCPU VM:

| Phase (seconds) | Open | Closed |
| --- | ---: | ---: |
| Proposed-source preflight / concurrent original+desired preflight span | 2.280 | 3.629 |
| Cold Godot spawn → owned connection inside that preflight | 1.918 | 3.068 (both concurrent children) |
| Native mutation | 0.542 | 0.028 |
| Independent actual-state acquisition | 0.203 | 0.056 |
| Actual-source validation | 1.770 | 2.168 |
| Cold Godot spawn → owned connection inside actual validation | 1.470 | 1.795 |
| Final independent acquisition plus finish | 0.191 | 0.055 |
| Complete public edit response | 5.783 | 6.478 |

Both operations verified in this diagnostic run; it does not erase the retained
serial composed failure or prove stable latency. The original/desired closed
validators started together. The profiling record contains no project source,
source hashes, project paths or credentials. Temporary instrumentation is not a
product telemetry interface.

The measured repeated initialization justifies examining bounded ready isolated
compilers instead of a larger VM or a stopwatch adjustment. The simplest existing
alternative is still one-shot validation with all cold work counted. A replacement
must prove fresh root **and dependency** parser/resource generations, exact current
warning policy, per-request isolation, source disposal, cancellation and worker
loss behavior, and independent actual-source post-verification. A fresh LSP root
parser or a matching preflight digest alone is not that proof. No persistent
compiler architecture is selected solely because its startup cost would be lower;
its concrete reset/configuration mechanism and current-consumer integration must
be justified before implementation.

#### Selected optimization: prepared native documentation, fresh validators

Keep fresh official-stock Godot processes for every source-validation pass. Cold
process launch after receipt remains inside the original edit deadline. Do not
introduce persistent parser/dependency/resource state or subtract startup from
the reported request interval.

The smaller reusable artifact is stock Godot's **native documentation cache**.
The pinned [`EditorHelp::_gen_doc_thread`](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/doc/editor_help.cpp#L3056-L3086)
excludes extension classes and writes native documentation before loading script
documentation. A source-free, empty private project can therefore prepare that
artifact without observing a target project. The artifact's binary encoding is
not byte-reproducible across generations; bind each prepared snapshot by its own
observed SHA-256 rather than inventing a universal cache hash.

Add optional host setup `--validator-engine ABSOLUTE_PATH`, accepting only the
already-supported exact official binary after the existing provenance check.
This is not a tool argument, arbitrary executable permission, editor selector
or target-editor launch. Existing `--registry`-only and one-shot CLI callers keep
their fully timed cold path. The documented long-lived MCP configuration and
actual-client acceptance use preparation explicitly.

Preparation runs once, in an owned same-binary worker and isolated stock child,
before protocol input is consumed. Its **ten-second total bound** includes binary
verification, empty-project creation, observed owned LSP initialization, child
reaping and receipt delivery. Failure prevents startup for explicitly configured
preparation; there is no silent retry or success fallback. Record a source-free
readiness duration separately from operation results. No target script, project
settings, addon, extension, global class, scene, dependency or verdict enters this
prepared snapshot.

The host retains one process-lifetime, private native-documentation snapshot,
with bounded size and SHA-256 binding, and removes it on normal shutdown. A weak
process-local registration lets existing validator supervisors borrow its owned
lifetime without coupling domain requests to MCP or adding configuration to
every operation API. Worker requests carry only the private snapshot descriptor.
Each fresh validator verifies and copies that snapshot into its own private HOME
before launch. A missing/changed prepared artifact is unavailable, not an
unverified cache hit. The validator never publishes data back into the snapshot.
All source admission, current warning policy, root/dependency completion fences,
fresh revision/state guards, post-change actual-source validation, process-group
ownership and clone disposal remain unchanged. Original/desired validation stays
concurrent.

**Measured basis:** owned four-vCPU startup probes recorded 2.116 seconds to
initialize a fresh unseeded helper and 1.311 seconds with only native documentation
copied in; concurrent seeded helpers initialized in 1.812/1.834 seconds. These are
mechanism measurements, not composed acceptance. The snapshot was approximately
3.27 MB. Disabling rendering did not establish a useful improvement. The faster
Dummy text driver is rejected: pinned
[`GDScriptParser`](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript_parser.cpp#L2827-L2831)
uses TextServer Unicode-security support for `confusable_identifier`; removing
that support would weaken authoritative warning validation.

**Principle XIII:** the current failure is verification exhaustion in the
retained 9.465-second composed edit. The simplest alternative regenerates the
same native documentation in each isolated process. A mutable warm-compiler
service would require an unproved cache reset and a larger lifecycle surface.
One bounded preparation worker plus a read-only, owned snapshot avoids those
costs. Its present costs are startup configuration, approximately 3.27 MB of
private temporary storage, bounded copy/hash I/O per validation, and focused
startup/cleanup/privacy and compiler-equivalence evidence. No service, dependency,
queue, background replenishment, new tool or longer deadline is added.

**Verification:** measure readiness independently, then full request latency with
all fresh-child launch time still included; do not call the helpers prewarmed.
Exercise valid/invalid source, fresh dependency bytes, changed warning policy
(including Unicode-security warnings), prepared-artifact loss/tampering,
startup failure/cancellation and owned cleanup. Rerun affected public edit paths
and required composed interaction, not historical campaigns whose relevant
behavior is unchanged. This preserves Principles I–V, VII–X and XII–XIII; no
mutation, support, acceptance or public tool-schema guarantee changes.

#### T004 implementation-shape and constitutional review

Static review of `3067fc6` found no current-task implementation defect or new
constitutional conflict. The two review slices covered preparation/validator
ownership and composed/privacy witnesses; neither review is runtime acceptance.

`native_docs.rs` owns the bounded source-free snapshot lifecycle: preparation,
strict receipt, private artifact binding, borrowing and disposal. Registration
and reply enums distinguish unavailable/preparing/ready/lost states without
speculative public variants. Existing `ownership.rs` retains process/namespace
mechanics, `stock_validation.rs` retains fresh source/context/fence validation,
and the MCP entry/transport owns only deployment configuration and startup.
Interfaces use current-consumer visibility; filesystem access borrows paths
rather than manufacturing a disposable owner. No parser pool, cached verdict,
request-clock renewal, configured-preparation fallback or new dependency was
found. The existing stock-loopback limitation is unchanged, not a newly imposed
sandbox requirement.

The composed, privacy and preparation-fault mixins each own one acceptance
responsibility and reuse the existing editor/history/export/relay machinery.
The composed state machine requires 8/6/6 changed edits, current read bases,
three dirty and three stale refusals, independent intended source/saved-state
witnesses, real history transitions, serial fixture ownership and later
durability. Ordinary fixture actions do not repair a reported product success.
Source-bearing transcripts remain private. Full-frame disclosure checks and
actual exported-artifact/runtime witnesses remain distinct from client evidence.

This shape is proportionate to the measured failure and current acceptance
requirements under Principle XIII. It preserves independent authorities and
verification (I/IV/VI), human work/native history (II/III), local confinement and
export separation (V/VIII), protocol-independent core ownership (VII), truthful
lean results (IX/X), source provenance (XI) and focused quality gates (XII).
The actual-agent SC-005/SC-009 interaction and cumulative evidence disposition
were subsequently [completed with independent runtime and client evidence](quickstart.md#t004-execution-evidence-2026-10-05);
static review alone was not treated as acceptance.
