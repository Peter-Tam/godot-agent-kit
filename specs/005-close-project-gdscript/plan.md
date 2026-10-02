# Implementation Plan: Safely Close a Clean Project GDScript

**Branch**: `spec/005-close-project-gdscript` | **Feature identifier**: `005-close-project-gdscript` | **Date**: 2026-10-01 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/005-close-project-gdscript/spec.md`, including the accepted guarded-native-revalidation clarification.

**Status:** Phase 0 research, Phase 1 design and selection-time granularity/consistency review passed. T001 and T002 are complete with [native acceptance](quickstart.md#9-t001-native-boundary-acceptance-2026-10-02), [public-caller and full affected-suite acceptance](quickstart.md#10-t002-public-caller-acceptance-2026-10-02), and the implementation-shape/constitutional reviews below. T003 remains pending and unstarted; Feature 005 is not complete.

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

## T001 selection and constitutional scope — 2026-10-01

The single-task implementation instruction selects T001 after completed
Features 001–004, the merged planning artifacts, passed granularity review and
read-only consistency analysis. The requirements-quality checklist remains
16/16, unchanged. Coverage and native-contract reviews found no actionable
constitutional, dependency or interface contradiction. The checked executable,
host and toolchain match the planned candidate; no runtime acceptance is claimed
by these prerequisite checks.

T001 retains the approved complexity decisions: one native close owner and
bounded continuation ledger address the observed dirty-source application and
deferred validation; shared source-context acquisition has two current
consumers; the existing validator and fixture consume exact protected source.
No public close caller/reducer/transport, new dependency, service, approval
mechanism or CI topology belongs to this increment. The ninth authenticated
capability remains false until T002's complete exchange. The private fixture
must prove native behavior without advertising a product success verdict.

### Implementation-discovered boundary details

The first real-editor preparation refused, rather than manufacturing native
eligibility. The private JSON handoff must convert only finite integral
bounded byte lengths to Godot integers; the native ABI retains its strict
integer checks. The pinned [CodeTextEditor settings](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/gui/code_editor.cpp#L1166-L1167)
have separate normal and error-state idle delays under `text_editor/completion`.
Both are captured and committed; admission uses their maximum and rechecks
both values. No editor preference, timer or source is changed to pass admission.

The strict context decoder needs a small private shared budget for retained
source and metadata. The concrete failure is that an internally tagged Serde
enum buffers its complete `Content` before nested source/array bounds run;
an otherwise bounded 12-MiB fixture frame can exceed the task's retained
512-KiB source/256-KiB context limits first. A direct closed struct plus the
existing per-record validation is the simplest alternative, but per-record
checks alone cannot enforce a remaining cross-document budget before copying.
Direct structs and schema-specific nested Serde seeds therefore share that
budget for the two current source-context consumers. The maintenance cost is
explicit field/visitor coverage in the private codec, justified by T001's
pre-allocation requirement and covered by malformed-tail boundary regressions.
There is no new dependency, public parser framework or service; Serde's
escaped-string scratch remains bounded by the existing outer frame ceiling.

Survivor acquisition is separate from a successful-close guard. It retains
scope/deadline confinement and sticky original effects/invalidation while
observing a still-open or newly reopened target; it retires rather than renews
authority. Terminal cleanup cannot turn an unprepared protection set into
`preserved`, or erase established original-document removal merely because a
different document now occupies the same path.

### Implementation-shape review

The new bounded decoder pushed `opening_context.rs` beyond a coherent
source-context model/validation surface. Its schema-specific budget, seed and
visitor code now lives in private `opening_context/decode.rs`; the parent
retains models, semantic checks, serialization, F/E encoding and existing
behavioral tests. This is a relocation of the required decoder, not another
parsing abstraction. The existing sibling-consumer paths remain private to
`stock_validation`; the child does not expand public visibility. The concrete
cost is one private module and explicit field coverage, offset by separating
decoding/allocation limits from semantic admission and receipt checks.

`script_close.cpp` owns one private close attempt: admission, one generated
native effect, callback attribution, independent verification and retirement
all share that attempt's identities and immutable guard. Its size triggers a
cohesion review, but another state-owner boundary would expose mutable attempt
internals without a current separate consumer. Common lexical/compiled/effective
acquisition instead lives in `editor_context`, with both opening and closing
as actual consumers. There is no generic callback registry or duplicated
scanner. Historical `entered`/discard facts remain separate from the current
phase; selection/removal availability is not converted into guessed Booleans.

The addon owns the existing shared slot, collection and waiter lifetime, not
a second reducer. Rust's close-context module owns aggregate binding and
sequential reuse of the existing validator; opaque public types/functions are
needed by the current external example consumer. No public close outcome,
future-only extension point, dependency, new unsafe Rust or operational gate
is introduced. Native ABI calls retain generated public signatures and checked
failure outcomes. The split passed formatting, Clippy, all 317 default Rust
tests, rustdoc and actual consumer builds. Both retained example tests prove
pre-allocation source-bound rejection; they passed separately after deleting
an incidental Serde-error-wording test. Live acceptance is recorded separately.

The Resource-lifetime fixture distinguishes compiler-cache retention from
document ownership. Both raw stock close and guarded close retained an ordinary
loaded target after all attempt references were released. Godot's
[reload path](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript.cpp#L780-L783)
registers named Scripts in a
[strong compilation cache](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript_cache.h#L89-L91).
The naturally-unloaded control therefore prepares a fresh replacement
document before admission while the compiler retains the older Resource.
It independently establishes the new clean D/R/B identity, performs the
same guarded close, and requires both ordinary cache absence and destruction
of that original captured Script instance. It never clears a cache or drops
an extra reference after close to obtain the expected result. This positive
passed independently; ordinary retained-original controls remain separate.

Dirty-history preparation now reuses the existing fixture's deadline-limited
R==B observation instead of assuming that 180 process frames suffice for idle
validation. This happens before any close attempt, without a Resource setter,
Save, timer change or product retry. The dirty-current positive leaves that
document selected while closing the separate target; the dirty-background
positive uses a different selected document. Both passed real earlier and
post-close Undo/Redo plus exact source/history preservation. A deliberate
R!=B case still refuses before entry. Owned-window captures are taken on the
actual Script screen; the selected close, last-tab removal and preserved
dirty-buffer/history surfaces have been visually inspected.

Upstream workflow governance was fast-forwarded from `origin/main` during
verification (`8744cf6`, PR #51); it changed only `AGENTS.md` and
`SPECKIT_WORKFLOW.md`, not tested runtime inputs. The selection-time analysis
above is historical, not a schema-1 currentness attestation for later edited
artifacts. Future implementation prerequisites must apply the
[current analysis-recognition policy](../../SPECKIT_WORKFLOW.md#g-recognize-current-analysis-before-implementation);
no pass is backfilled from remembered analysis or newly computed hashes.

## T002 implementation-shape and constitutional review — 2026-10-02

T002 was selected from updated `main` after T001's completed acceptance and
merged PR #52. The requirements checklist remained 16/16 and read-only.
No extension hooks were configured. A fresh read-only consistency analysis
covered all 18 functional requirements, seven success criteria and 21 story
scenarios without actionable findings; its exact clean input state is preserved
in [analysis.md](analysis.md). This review and execution evidence do not change
the approved remaining T003 obligations.

The implementation retains the approved separation. `script_close` owns checked
intent, source-free evidence summaries and immutable effect-sensitive reduction.
The CLI owns selectors, bounded stdin, signals and delivery. Close wire modules
own strict public/private representation, native-fact validation and result
encoding. The runner owns supervised acquisition and actual one-shot validation;
its private channel owns frame ordering, original expiry, admitted identities
and monotonic native visit evidence. The addon transport owns fixed stage
routing and response lifetime; the existing native/addon owner remains the
single editor-operation owner. No Godot or JSON types enter the close core.

The supervisor's initial combined channel/acquisition implementation exceeded
1,000 lines and combined independently nameable responsibilities. Moving frame
ordering and attributable failure evidence into its private `channel.rs` child
keeps the existing architecture without a new service, trait or public API.
Only current supervisor consumers receive parent-visible methods/fields.
The RPC helper derives its cutoff from the original start instead of accepting
a redundant deadline at every call; best-effort abort retains its shorter bound.

Reusing live-frame decoding for a prior public observation replaced historical
receipt times and rejected legitimate caller-clock validity. The private
`basis.rs` decoder now preserves actual public-v1 provenance while continuing to
use the existing checked `ExpectedRevisionBasis` eligibility conversion once.
This is representation decoding, not another closing or editing policy.
Closed typed fields, required nulls and ordinary observation validation remain
mandatory; an unusable valid observation is not relabeled malformed input.

Independent review and integration found concrete evidence-path defects:
required nullable keys could be omitted; callback stamps lacked enclosing
native-entry bounds; fresh survivor selection was conflated with historical
native selection; preparation omitted independently acquired D identity; and
repeated historical native facts replaced their first actual receipt times.
These paths now preserve exact authority, ordering and causal failure. The
preparation/survivor snapshot uses the existing ordinary collector's disk
identity binding. Authorization invalidates current-state summaries rather
than presenting pre-effect Resource/selection/history as a known post-state.
Failure-only survivor acquisition cannot authorize a second close or renew the
original lease. Repeated native events retain their first observed receipt;
new completion events remain distinct.

The lifecycle and application knowledge use separate small enums: accepted
through verification is not the same concept as possible, discarded, selection
or removal effects. Channel entry/removal/invalidation fields are independent
sticky facts, not interchangeable lifecycle flags. No future-only variant,
extension hook, arbitrary dispatcher, public native receipt, dependency,
compatibility shim, Save/discard/retry/rollback route or additional approval/
provider requirement was introduced.

Principles I–IV remain the correctness boundary: independent clean target
D/R/B before entry, preserved human work and exact identities, one native close,
and fresh postconditions plus actual continuation evidence before success.
V, VII–XI retain local authenticated confinement, protocol-independent reduction,
minimal public surface, private protected-source validation, source-free
diagnostics and production-export isolation. VI/XII still require the task-owned
real-editor groups and full affected existing suites; a positive smoke or
passing unit tests alone cannot complete T002. Principle XIII is satisfied by
reusing the existing owner, validator, confinement and harness; the narrow
decoding/channel splits address demonstrated current correctness and cohesion
problems rather than speculative reuse. Actual acceptance and limitations are
recorded separately in the quickstart.

The source-limited already-closed case exposed a shared file-pin assumption:
`pin_file` applied a source-size admission limit even when no source was needed.
Its explicit `bounded_source` argument now remains true for every existing
edit/open/context caller; close inspection disables only that size admission
when it has independently found no open document. Ownership, regular-file,
no-follow, namespace and retained-descriptor checks are unchanged. A closed
attempt cannot prepare or advance, and effectful preparation still enforces
all source limits. Duplicating metadata pinning would duplicate confinement
logic; one explicit current-use argument avoids that cost without allocation,
a new service, or a wider mutation capability. Both matched native artifacts
were rebuilt; the changed shared family requires the complete existing suites.

Recognition also uses the ordinary collector's independent D validity rather
than requiring a nonexistent loaded Script to establish file validity.
Historical public observations are decoded against their own target/session
before checking current request binding: valid old-session evidence is a
`session_changed` refusal, not malformed JSON. Close input reason mapping now
lives once at that adapter; the CLI only delivers it and drops the raw stdin
allocation before starting acquisition.

A failing-before/passing-after regression exposed late scope-denial disclosure
after a known removal. A private sticky scope-invalidation fact now removes
expected/preparation/survivor source-derived summaries at terminal reduction,
including after a different first causal error or later acquired envelope.
It does not erase the known removal or replace the original failure. This
fact is orthogonal to lifecycle/application knowledge, not another lifecycle
flag. The ordinary Save fixture was also corrected to accept an actual
dirty-to-saved transition when human edits left bytes equal to disk; dirty
does not imply different text. The first-use public scenario independently
asserts that distinction before refusal and ordinary Save.

The public-basis decoder now requires the nested nullable fields that ordinary
observation v1 actually emits. A CLI smoke previously accepted an omitted D
`reason` and reached routing (exit 3); the corrected caller refuses it as
`invalid_request` (exit 2). The existing bounded duplicate-key scan checks
presence without a second source-bearing DTO/tree or changes to shared live
decoders. Observation v1's intentional omission of unavailable source
`text`/`collection` and unknown-staleness evidence remains valid. The regression
removes each emitted nullable field independently, alongside real partial-
observation compatibility cases.

Acceptance reuse was corrected at actual boundaries: the mixed-document case
makes its text asset visible to FileSystemDock and uses the existing
`prepared_subject_buffer` witness; ambiguous-session checks compare source,
identity, history, selection and configuration, not ordinary startup visits
or unrelated loader activity. Native entered-call barriers publish entry
before deliberate connection loss can terminalize the caller.
Closing's validator controls now record the actual pre-interruption process
group and private-directory ownership/mode before using the existing cleanup
assertion. Those now-shared inspection/cleanup helpers and evidence fields use
operation-neutral names; both consumers migrated without aliases. Existing
harness inheritance remains, rather than introducing a second process harness.

The inherited duplicate-wait check now observes that the first waiter really
is pending, rejects a second waiter, excludes a completion claim without the
required event, and measures return against the original remaining lease.
It no longer pins an incidental subset of terminal status labels. The
accepted native witness returned in 6.994712 seconds with 6.995177 seconds
remaining and the original expiry unchanged. The full 123-record native group
passed.

The affected observation suite exposed another fixture synchronization
assumption: documentation/text-tab activation was checked after one frame.
The existing action now waits for the actual help selection, FileSystemDock
item and extra text editor under one five-second preparation deadline.
Three fresh visible mixed-document controls and the complete 273-record
observation suite passed. No product retry, source repair or new readiness
infrastructure was added.

Full edit validation stalled near independent cases while its conflict
and routing groups retained up to thirteen completed editors. The isolated
oversized-buffer case and focused groups passed. Those groups, and the same
retention pattern in interruption/post-change cases, now close each owned
editor after its final evidence assertion. Intentional overlapping/two-project
cases retain both editors until their comparisons finish. This reuses existing
teardown, changes no product deadline/assertion and introduces no lifetime
framework. Focused conflicts (38 records), focused routing (36) and the complete
408-record edit suite passed with bounded fixture lifetimes.
