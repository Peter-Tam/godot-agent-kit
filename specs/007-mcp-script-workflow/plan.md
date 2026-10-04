# Implementation Plan: Use the Trusted GDScript Workflow Through MCP

**Branch:** `spec/007-mcp-script-workflow` | **Feature identifier:** `007-mcp-script-workflow` | **Date:** 2026-10-03 | **Spec:** [spec.md](spec.md)

**Input:** Maintainer-approved discover/read/edit requirements, including constitution v1.2.0 alignment in FR-022/SC-009.

**Status:** The four-task design passed [granularity/constitutional review](tasks.md#granularity-and-constitutional-review) and [consistency analysis](analysis.md); design PR #63 is merged. T001 is complete and merged in PR #64 with [core/native acceptance and implementation review](quickstart.md#t001-execution-evidence-2026-10-04). T002 implementation/acceptance is in progress; T003–T004 remain pending. This delivery/status update does not change approved remaining obligations or claim real-client interoperability, feature completion or Phase 3 exit.

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
| Compatibility / workflow | Existing local v1 APIs/refusals stay valid; private v6/native revision 4 remains unchanged and MCP uses schema 1. Analysis remains current after cumulative T001 delivery-only drift. Only T002 is selected, following merged T001; it remains one task/PR. |

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

[tasks.md](tasks.md) derives four coherent implementation increments: protocol-independent trusted read/revision/closed editing; complete external MCP MVP with both clients' positive workflows; combined adversarial preservation/interrupted-effect behavior; composed real-agent A–E and cumulative acceptance. Direct tests/docs remain bundled with their implementation, and separately runnable groups do not create extra task IDs. T001 is complete; T002–T004 remain pending under the one-task-one-PR sequencing rule.

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
