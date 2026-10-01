# Implementation Plan: Safely Close a Clean Project GDScript

**Branch**: `spec/005-close-project-gdscript` | **Feature identifier**: `005-close-project-gdscript` | **Date**: 2026-10-01 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/005-close-project-gdscript/spec.md`, including the accepted guarded-native-revalidation clarification.

**Status:** Phase 0 research and Phase 1 design complete; integrated artifact review and the post-design Constitution Check passed. The accepted native-revalidation clarification is recorded in the specification. The [task decomposition and granularity review](tasks.md#granularity-review) are complete; `/speckit.analyze` remains separate. No implementation task is selected, no product close is implemented, and no feature/support acceptance is claimed. Planning stays on the existing specification branch.

## Summary

Add one `close-gdscript` caller and a close-owned protocol-independent request/outcome model. Authenticate the exact local project/editor, inspect a valid target, and either freshly recognize an already-closed document without effects or close an observed clean open document under a matching prior revision/identity basis. Native `ScriptEditor.close_file` is the canonical route, but its dirty-discard branch, unrelated pending-source application and deferred validation make a raw call unsafe.

The selected design freezes the target and a bounded complete set of potentially visited remaining script documents; reuses existing exact-source/effect validation and shared ownership; invokes one guarded native close; observes actual native revalidation completion; then independently verifies closed-buffer absence, unchanged D, retained/released R and preserved unrelated work. The [research](research.md) contains seven stock close observations in four visible owned editors, including the unsafe R!=B control and preserved dirty R==B/native-history control. These inform the route and regressions, not completed product acceptance.

The user explicitly resolved [G1](research.md#7-g1--native-revalidation-decision): normal native close-triggered revalidation of unchanged already-loaded source is permitted under guards and independent verification. Explicit reload/reparse/rescan/repair, applying differing pending source, Save/discard, new project-code execution, rollback and automatic retries remain prohibited. No last-tab-only scope reduction or patched engine is selected.

## Technical Context

**Language/Version:** Existing Rust 1.98.1, edition 2021; thin GDScript editor integration; C++17 public GDExtension ABI; Python 3.10+ owned real-editor harness.

**Primary Dependencies:** Existing locked `serde =1.0.229`, `serde_json =1.0.151`, `cap-std =4.0.3`, `ring =0.17.14`, official stock Godot and the existing one-shot source-only validator. No new crate, SDK, parser, runtime, service or engine patch.

**Storage:** Existing source-free owner-private session registry; bounded request-local identities, source commitments, protected source captures, descriptor handles and continuation evidence. Existing disposable validator staging/owned acceptance projects only. No persistent history, callback registry, database, lease/replay service or source writes by closing.

**Testing:** Deterministic core eligibility/state/precedence and bounded input/wire tests; actual process/bridge/filesystem boundary cases; new owned close groups using existing independent D/R/B/dirty/history witnesses; current full suites when the shared version/native/owner boundary changes; full close/composed acceptance at the cumulative gate. [Quickstart](quickstart.md) separates currently runnable baseline, proposed implementation commands and planning evidence.

**Target Platform:** Exact official `4.7.2.stable.official.ed1daf0bf`, commit `ed1daf0bf001b61586d9930840f2f1394092c079`, executable SHA-256 `c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf`; actual planning host macOS 26.6.2 build 25G83 arm64. Retain the existing candidate only; close support requires its own CI/real-editor evidence.

**Project Type:** Existing local Rust library/CLI and Godot editor addon with a shared native bundle. No MCP adapter, editor launcher or general document manager.

**Performance Goals:** Ten-second controlled terminal result: 9.5 seconds for bounded input, selection, private validation, native effects/continuations and verification, plus 0.5 seconds for delivery with stdout drained. Addon lease ≤9000 ms and all child work share the original cutoff. A blocked editor/file/helper cannot prevent a truthful terminal result; no claim of cancelling synchronous editor work.

**Constraints:** One explicit existing standalone target; independent target D/R/B and clean/Resource-edited evidence; current caller basis and native same-turn entry guards; no forced source convergence, source write, Save/discard, unpermitted execution or UI selection to manufacture eligibility. Preserve unrelated dirty R==B work; refuse R!=B/unknown/unsupported effect contexts. Already-closed recognition remains non-interfering and does not require a fabricated open basis.

**Scale/Scope:** Existing target source bound 512 KiB per authority; at most eight open script documents including target for effectful close, aggregate retained remaining source ≤512 KiB and non-source context ≤256 KiB. Existing checked path/counter limits, 4 MiB selected requests and 12 MiB response/output bounds remain. Exact admission and evidence semantics are normative in [data-model.md](data-model.md#2-fixed-supported-profile-and-bounds). These are explicit refusal bounds, not arbitrary-project-size or concurrent-throughput claims.

## Constitution Check

*Gate: evaluated before Phase 0 research; re-evaluated after Phase 1 design.*

**Initial disposition:** PASS for bounded source research and owned synthetic-editor experiments under the existing architecture, with no constitutional exception or speculative operational prerequisite. Research found a real specification conflict, not a reason to ignore safety. The user approved the narrow native-revalidation clarification before its dependent design was selected.

**Post-design disposition:** PASS for planning, re-evaluated 2026-10-01 against the clarified specification, constitution and existing architecture. The selected safeguards and complexity decisions below remain required implementation and acceptance obligations; this review does not claim they have passed product runtime acceptance.

| Obligation | Selected design / required proof |
|---|---|
| I / IV — independent authorities and verification | Before effects, independently agreeing D/R/B and attributed clean state; afterward actual old-buffer removal/current absence, independent unchanged D, observed retained/unloaded R, completed native continuation and preserved protection set. Receipts or missing B reads are not success. |
| II — preserve human work | Dirty/equal-dirty target, stale basis, same-text reopen and unknown safety refuse. Newer/reopened and unrelated work remain intact. Dirty R!=B native application is a required refusal regression; dirty R==B preservation gets real Undo/Redo proof. |
| III — native lifecycle | One generated-bound public `close_file` call; ordinary target-local history disposal and permitted native fallback only. No private tab freeing, source-edit substitute, history reset or undoable-close claim. |
| V — least privilege | Existing authenticated local project/session route, no-follow confined read descriptors and private context capture. No arbitrary file/method, root load, evaluation, remote access, telemetry, force or approval bypass. |
| VI / XII — real regression evidence | Close-specific positive/refusal/identity/continuation/interruption/history cases plus affected full existing suites and cumulative A–E/durability/export on the actual delivery head. Native planning controls are not these gates. |
| VII — ownership | Rust owns checked intent and outcome; CLI/wire/process concerns stay at adapters; Godot state/effects stay at addon/native integration. No second safety model or generic transaction framework. |
| VIII — tooling/export boundary | Extend existing actual enabled, disabled and hook-only export checks to new close owner/native/fixture pieces; no active tooling or gameplay dependency ships. |
| IX — small composable surface | One close caller; fresh observation/open/edit remain separate. No Save-and-close, discard, compound reopen, history command, batch or tab catalog. |
| X — truthful diagnostics | Fixed causal reasons, stage/application certainty, native-continuation limits and source availability. Known effects survive loss; no implicit rollback/retry or late success upgrade. Private unrelated source/paths/hashes do not leak. |
| XI — independent implementation | Pinned public source/API and owned stock behavior select the route. Existing locked dependency provenance reused; no unrelated implementation copied. |
| XIII — current-risk complexity | Bounded full candidate protection and attempt-local validation ledger address demonstrated/reachable close effects. Reuse validator, owner, framing and harness; narrow shared context extraction only for two current consumers. No new service/registry/CI topology. |
| Compatibility / workflow | Public observation/edit/open/discovery v1 unchanged; new close v1. Coordinated private bridge v5/native revision 3 with all current consumers migrated and no shim. Later task granularity review/analysis and one-task-one-PR sequencing remain mandatory. |

The inherited threat model excludes hostile same-UID software and malicious code already running inside Godot. Known unsafe integration and detected interference still invalidate eligibility or success. No atomic exclusion against arbitrary external writers or universal callback isolation is claimed. No additional hosted provider, GUI runner topology or approval process is made a product prerequisite.

**Integrated artifact review (2026-10-01):** Checked the specification, model, caller, bridge, native contract and validation guide for consistent basis eligibility, bounds, private field/stage contracts, callback ownership, expected roster removal, retained Script identity and terminal precedence. All 21 story scenarios, 18 functional requirements and seven success criteria remain covered; the requirements-quality checklist remains 16/16. Nine feature documents passed local-link/anchor and retained-evidence-link checks; the JSON example and 12 shell blocks passed syntax checks, and malformed Markdown table separators were corrected. The installed prerequisite helper discovers the selected feature's research, model, contracts and quickstart. These are planning checks, not executed product close commands.

**Design-shape review:** Closing has one domain responsibility and reuses the existing checked basis, operation owner, source validator, transport and harness. The only shared extraction has two current consumers; native callback/state details remain private. The protection set and finite continuation ledger address source-established and observed native effects, with explicit bounds and cleanup rather than new infrastructure. No constitutional exception, unresolved material decision, speculative public API or additional operational prerequisite is selected.

## Execution Design

### Checked caller intent and no-effect recognition

The new public operation uses existing selectors and one bounded stdin object: schema 1, fresh request ID and full prior observation-v1 `basis` or null. Reuse `ExpectedRevisionBasis::from_observation` internally; LSP reference inspection found its real current edit/codec/consumer uses, so do not move or break that public contract merely to create a second name. `CloseRequest::new` borrows the optional observation and retains the checked basis rather than duplicating source text or eligibility code.

Unique authenticated selection remains independent of the prior result. If a valid exact target is freshly confirmed closed, recognize it without calling `close_file`, validating source, changing selection or loading R. A supplied basis cannot transfer across project/session/file identity; an old open version is explicitly not applied to a still-closed same file. If the target is open, null/ineligible/stale basis refuses. A new same-path buffer with the old source is still a new document identity.

### Immutable target and complete bounded effect context

Ordinary collection plus independent Rust D acquisition establishes the target; native preparation independently pins read-only descriptors, compares source/identity and observes clean buffer, Resource edited=false, current/saved versions and exact associations. No source writes, Save profile, timestamps, compilation or tags belong to closing.

Protect all remaining supported open scripts because native history-back and sorting can visit documents not inferable from the current tab. Reuse the current bounded lexical/compiled/effective-context checks, requiring already-equal R/B with actual dirty/Resource/history facts and stable identities/versions. Dirty R==B remains dirty; do not wait for or apply human convergence. Reject unsupported/mixed/custom/external-editor or oversize contexts before effects without silently shrinking the set.

This source/effect profile is conservative but must support ordinary selected/non-selected two-document cases, empty source, a safe syntax-invalid closing target and read-only target files. Parse validity is required only for remaining source whose native validation can run, not as a new requirement to close invalid target text. Keep remaining source private; D may legitimately differ from unsaved R/B.

### Existing source-only validator as a current consumer

Use purpose `close_context` in the existing supervisor-owned one-shot stock validator, one child per remaining document sequentially within the original deadline. Require exact-source/per-URI fences, effective context, same-request/document guard binding and owned-child cleanup. No helper for the closing target merely to make it parse-valid, and none for already-closed recognition or an empty remaining set. Preserve existing `open_context`/edit purpose separation and the inherited local-endpoint limitation.

A fresh pre-close recheck must match the complete preparation/receipt set; changed source/version/roster/selection/settings/file identity refuses. Do not restart validation against changed state under the old request. Inspect actual idle-parse timing as part of the effect context and refuse predictably unfinishable continuations before entry, without changing editor preferences.

### One native close and actual continuation completion

The supervisor records possible application before sending one authorization. Native guards the original target and every protected document immediately before entering bound `ScriptEditor.close_file(path)` on the main thread, without event pumping. It records the returned Error and actual effects separately; no method/phase chosen by untrusted input, no second call or fallback tab action.

A bounded request-owned ledger observes actual ScriptEditor selection visits during native close and actual `edited_script_changed` completion on the corresponding retained protected editor instances. Pre-entry, stale and replacement events never discharge new obligations. No fixed sleep, source equality, forced validation or acknowledgment serves as a fence. One pending bridge `close_wait` watches this existing work via the addon frame loop under the same lease; no retry scheduler or permanent monitor is introduced.

The native owner and registered callback data survive every entered synchronous call/callback. Cancellation/disable prevents new work and releases only safe owned state after return. The caller may time out before ordinary Godot work finishes; retain known applied/unknown effects and detach observation safely, never claim rollback or leave a queued product close over newer work.

### Independent postconditions and immutable terminal outcome

After actual required continuation completion, acquire a fresh ordinary target snapshot, Rust D and native identity/namespace/protection/selection rechecks. Release target-only retained Resource references before the ordinary R read so the outcome can truthfully distinguish naturally retained and unloaded state. Confirm old-buffer disposal and current target absence; a reopened buffer is newer work, not a close destination. Post-close B and buffer dirty state are not applicable only after observed absence.

Success requires unchanged admitted D/file identity, independently unchanged retained R or observed unloading, complete required native continuation and intact preservation evidence. No source is copied from input or another authority. Known/partial effects and invalidation are sticky; results follow [the outcome model](data-model.md#7-state-transitions-deadline-and-terminal-precedence), not inferred process success. Close results are source summaries and are not reusable observation/edit bases.

### Compatibility cutover

Private bridge v5 adds the ninth authenticated Boolean `close_gdscript` using the existing role-separated HMAC transcript and framing. Native revision 3 identifies the actual close family/continuation lifetime contract. Retain shared artifact names and exact generated-ABI/provenance checks. Update all current Rust/addon descriptors, decoders, tuple versions, capabilities, native checks, private fixture consumers and cross-language vectors together. No v4 listener, revision-2 close fallback or deprecated alias remains.

Current installation/reproduction notices in Features 001–004, native and shared workflow guides must point to the new matched version/session after implementation; historical acceptance stays historical. Restart/re-enable updated addon and use fresh session-bound bases. Observation/discovery still work without native mutation capability; closing is advertised only with its complete matched family. These migrations belong to the future implementing increment, not documentation-only changes to pretend v5 exists now.

## Project Structure

### Documentation (this feature)

```text
specs/005-close-project-gdscript/
├── spec.md
├── checklists/requirements.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── tasks.md
└── contracts/
    ├── close-api.md
    ├── bridge-protocol.md
    └── native-integration.md
```

[tasks.md](tasks.md) derives three meaningful PR-sized increments with tests/docs bundled: the native boundary and real fixture/validator consumer, the complete public caller with all inseparable safety, and cumulative repeated/composed acceptance. Its granularity review passed. Run `/speckit.analyze` and resolve actual blockers before selecting any approved implementation task; generation does not select a task or authorize batching PRs.

### Source Code (repository root)

The tree names **planned implementation locations**, not files created by this planning invocation. Existing files are modified only in their applicable later task; sensible responsibility-based subdivision is allowed without a generic framework.

```text
mcp-server/
├── Cargo.toml / Cargo.lock                 # add binary, retain locked dependencies
├── src/
│   ├── script_close.rs                     # new checked request + public outcome boundary
│   ├── script_close/{attempt,outcome}.rs   # private reduction and immutable evidence
│   ├── runner.rs / runner/close.rs         # current dispatcher + close supervisor/worker seam
│   ├── runner/close/                      # close process/bridge acquisition, if cohesion warrants
│   ├── runner/stock_validation.rs          # add private close_context consumer
│   ├── runner/stock_validation/            # matching admission/protocol/ownership extensions
│   ├── bridge.rs / bridge/wire.rs          # coordinated v5 capability/dispatch
│   ├── bridge/wire/close.rs                # close input/output/native tuple codec
│   ├── bridge/wire/close/                  # bounded input/output/editor separation as consumed
│   ├── bin/close-gdscript.rs               # one explicit CLI
│   └── lib.rs                             # real current-consumer exports only
└── tests/
    ├── script_close_contract.rs            # new semantic transition/boundary behavior
    ├── script_close_caller.rs              # real stdin/process/deadline/result cases
    └── bridge_boundary.rs + submodules     # shared version and new operation boundaries

godot-addon/
├── addons/godot_agent_kit/
│   ├── script_close.gd                     # new single-slot lifetime/native owner
│   ├── script_close_transport.gd           # typed private transport seam
│   ├── bridge.gd / plugin.gd               # v5/revision-3 capability and dispatch
│   └── export_guard.gd                     # existing export boundary retained/extended
├── native/
│   ├── script_close.cpp                   # one close attempt + finite native continuation ledger
│   ├── editor_context.cpp/.hpp            # shared current source/effect acquisition extracted
│   ├── open_context.cpp/.hpp              # retain opening-specific current-context orchestration
│   ├── document_guard.cpp/.hpp            # existing shared confined file/document guards
│   ├── native.hpp / session.cpp           # close owner, lifetime and API declarations
│   └── extension.cpp / build.py           # new bindings/family/provenance, no engine patch
└── tests/
    ├── run_script_close.py                 # new real close runner, no unimplemented all shortcut
    ├── fixtures/script_close/             # fixture-only human actions/fault barriers
    ├── run_editor_campaign.py             # close only when real runner/caller are consumed
    └── existing observation/edit/open/discovery and native tests
```

**Structure decision:** Keep two durable components and one Cargo package. Do not add independent crates, generic traits, public native/receipt APIs, lifecycle hooks for future features or source-copying wrappers. Large shared observation/runner/bridge files receive only necessary dispatch and version changes. New close policy, DTO/process handling and native integration own separate coherent responsibilities. Review files approaching 800–1000 lines for cohesion rather than enforcing a mechanical size limit.

## Verification and Requirement Coverage

The [quickstart](quickstart.md) is the runnable validation guide after the named implementation exists; the [research](research.md#3-exact-candidate-and-observed-provenance) records what actually ran now. Planning did not run a nonexistent close caller or the product cumulative suite.

| Requirement cluster | Required focused proof |
|---|---|
| US1.1–US1.5; FR-001/003/006–010/014; SC-001 | Selected/non-selected/last-tab native close; exact basis and postconditions; cached/unloaded already-closed recognition; safe invalid/read-only/empty sources; separate observe/reopen. |
| US2.1–US2.6; FR-002–005/008/012/014; SC-002 | Dirty/equal-dirty/stale/same-text reopen/missing evidence; full candidate-source/compiled/configuration limits; R!=B refusal and dirty R==B actual history; confined routing and unsupported contexts. |
| US3.1–US3.5; FR-004/009–013; SC-003 | Before/after-entry cancellation/loss/disable/timeout, delayed/premature/replacement native events, overlap, immutable effects and no late product close. |
| US4.1–US4.5; FR-015–018; SC-004–007 | At least 20 requests with five real closes/five closed recognitions/five dirty refusals; composed existing workflow; A–E/history/durability; privacy and all three export modes; result-only interpretation and exact supported provenance. |

All 21 story scenarios, 18 functional requirements and seven success criteria remain required. The native/source-application negative and the deferred validation event are new regression obligations, not reasons to weaken expected outcomes. Real native Undo/Redo of unrelated work must be exercised, not inferred from flags; target-local close disposal does not create a source-history action or erase others' stacks.

During development, first run the affected close group and directly affected regression. The v5 authenticated protocol, revision-3/native family, shared context helper and operation owner can invalidate complete existing suites, so that cutover requires their unfiltered observation/edit/open/discovery evidence. The feature cumulative gate requires all close groups and composed applicable A–E/Save/reopen/reparse/rescan/runtime and privacy/export on unchanged final-head inputs. Existing serial campaigns/resume fingerprints may be extended only with real runner consumers; optional GUI CI remains optional reproduction infrastructure, not a new approval/provider gate.

For Rust changes, existing locked fmt/Clippy/tests including doctests/rustdoc checks remain applicable. Native/fault builds, workflow/campaign checks and actual CLI/library smoke accompany their changed boundary. No source-text, mock-echo, copied-field or no-op tests; no generic all-features claim. No test/build/live-editor acceptance is waived by a template or a draft PR.

## Complexity Tracking

No constitutional violation or exception is selected. [Research §8](research.md#8-selected-native-protection-and-complexity-decisions) records the concrete requirement/failure, simpler alternative, existing mechanism's limit, ongoing cost and present justification for each material mechanism.

| Mechanism | Present justification / rejected simpler alternative | Cost and boundary |
|---|---|---|
| Close-owned request/runner/native attempt | Existing operations intentionally do not close; raw close discards dirty work | One current capability, same routing/slot/outcome principles; no framework. |
| Bounded complete remaining-document protection | Native history/sorting can apply/validate beyond current tab; target/current-only protection is insufficient | Eight-document and aggregate limits; reuse public getters/profile, refuse larger/unknown contexts. |
| Existing validator `close_context` purpose | Native validation processes source/export effects; equality alone is insufficient | Existing one-shot children sequentially under original cutoff; no target parse gate or new service. |
| Attempt-local continuation ledger | Actual close returns before native validation | Bounded public signal witnesses and cleanup; no timer forcing, persistent monitor or generic registry. |
| `editor_context` extraction | Opening and closing now both need the same source/effective/compiled safety logic | Private behavior-preserving split with two current consumers; preserve public APIs. |
| Coordinated v5/revision-3 cutover | Fixed authenticated capabilities and native callable/lifetime contract change | Migrate all peers/build checks/docs once; same artifact names, no shim or platform expansion. |

New public visibility must have a current caller. Retain the existing checked basis implementation without an unnecessary public breaking move; keep close attempt/native/codec internals private. Use typed recoverable failures for unavailable evidence, not `unwrap`/`expect` on practical failure paths. Review accidental complexity introduced by each later task before marking it complete; do not defer necessary cleanup to a new task or expand into unrelated refactoring.
