# Research: Safely Close a Clean Project GDScript

**Date:** 2026-10-01

**Spec:** [spec.md](spec.md)

**Disposition:** Phase 0 decisions resolved. The user accepted **guarded native revalidation** during planning, now recorded in the [specification clarification](spec.md#clarifications) and FR-005. Select the stock exact-path close with target/protection guards, bounded private source-context validation and an actual native continuation-completion ledger. This is design readiness, not implemented capability or new support/acceptance.

## 1. Research scope and current decisions

Three parallel read-only investigations covered native close behavior, existing core/worker/bridge contracts and the owned-editor validation harness. The integration owner inspected the pinned stock sources and exercised seven actual `ScriptEditor.close_file` calls in four owned visible editor processes. Six were positive stock controls; one deliberately demonstrated an unsafe unrelated-source transition. These are native research observations, not seven passing product-close acceptance cases.

| Question | Finding / disposition |
|---|---|
| Existing stack and ownership | Reuse the existing Rust package, locked dependencies, GDScript addon, public-ABI native integration and owned-editor harness. |
| Public exact-path close | `EditorInterface.get_script_editor().close_file(path)` exists and can close selected or non-selected script tabs without a preliminary target-selection call. |
| Dirty target | The API is discard-capable, not a safe-close primitive. Exact target clean/revision/identity checks are mandatory before entry. |
| Unrelated current script | Native close can apply pending B to R and schedules validation. Dirty R != B is an observed unsafe dispatch profile; dirty R == B preservation and actual Undo/Redo passed a stock control. |
| Post-close evidence | Actual open-document absence, unchanged D and independently retained/released R remain separate facts. The API return is not independent verification. |
| Current caller/bridge reuse | Reuse checked observation identities, expected-state evidence, project confinement, shared owner and bounded supervision. Private bridge v4 has a fixed authenticated capability vector; a new capability requires a deliberate coordinated successor. |
| Accepted decision G1 | Permit ordinary native revalidation of unchanged already-loaded source only under effect/preservation checks and independent verification. Explicit reload/reparse/rescan/repair, differing pending source application, Save/discard and project execution remain prohibited. |
| Selected private compatibility | Coordinated bridge v5 and native revision 3, retaining existing public observation/edit/open/discovery v1 behavior. |
| Phase 1 design | The [data model](data-model.md), caller/bridge/native contracts and quickstart define the bounded proposed implementation and its required evidence. |

## 2. Existing technology and current-need reuse

**Decision:** Retain Rust 1.98.1, edition 2021; thin GDScript editor integration; the existing C++17 public GDExtension boundary where native checks are required; and Python 3.10+ real-editor fixtures. Current dependencies are locked `serde =1.0.229`, `serde_json =1.0.151`, `cap-std =4.0.3` and `ring =0.17.14` from [Cargo.toml](../../mcp-server/Cargo.toml) and [rust-toolchain.toml](../../mcp-server/rust-toolchain.toml).

**Rationale:** Current targeting, independent D/R/B observations, source-free authentication, confined reads, process deadlines and operation ownership are real reusable capabilities. Closing needs its own intent/effect interpretation, not a new daemon, generic document manager, alternate undo system, package or safety policy.

**Alternatives considered:** A new crate/SDK, engine fork, process service, lifecycle framework, replay database or additional CI/approval requirement adds cost without an established need. A direct path-close call without guards is insufficient because its discard and unrelated-source effects are source-established and observed below. Requiring every tab to be clean would unnecessarily exclude the demonstrated dirty-but-current preservation case; it is not selected as a replacement for effect-aware protection.

## 3. Exact candidate and observed provenance

The candidate executable actually returned `4.7.2.stable.official.ed1daf0bf`; its SHA-256 was `c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf`. The fixture's live editor witness identified full commit `ed1daf0bf001b61586d9930840f2f1394092c079`. The actual host reported macOS **26.6.2**, build **25G83**, **arm64**. The repository implementation baseline was `d75861fd83eca7a7470b53f6f4f4ce1f185dd8f1` before these planning-only changes.

The existing [observation harness](../../godot-addon/tests/run_observation.py) created mode-private synthetic projects/registry/control directories and launched real `--editor --single-window --display-driver macos --rendering-method gl_compatibility --rendering-driver opengl3` processes. A temporary fixture-only subclass added fixed close research actions and non-mutating signal observations. No operation was run against a human project, no product close caller/native family was implemented, and no permanent test was added.

Independent witnesses used the harness's disk byte/stat acquisition, actual Script source, CodeEdit text, document-attributed unsaved paths, current/saved buffer versions, Script/editor/buffer IDs, loaded Resource edited state and actual selected/open documents. Existing source was not substituted for another authority. Three owned-window images were captured and visually reviewed. The fixture's pre-existing deliberately invalid script produced its expected editor diagnostic; this is not a product failure or a new runtime correctness claim.

Retained private evidence:

- [Summary](file:///Users/petertam/.godot-close-planning-xzzf5fx2/evidence/summary.json), SHA-256 `b7392ba3dd79ee03e3e4a36911d73521afad922ac1e853a6a0d13aa7c689bb77`.
- [Clean non-selected close](file:///Users/petertam/.godot-close-planning-xzzf5fx2/evidence/clean-nonselected.json).
- [Dirty-current preservation and Undo/Redo](file:///Users/petertam/.godot-close-planning-xzzf5fx2/evidence/dirty-current-preserved.json).
- [Dirty-current divergence negative](file:///Users/petertam/.godot-close-planning-xzzf5fx2/evidence/dirty-current-negative.json).
- [Selected-target close](file:///Users/petertam/.godot-close-planning-xzzf5fx2/evidence/clean-selected.json), [last-tab close](file:///Users/petertam/.godot-close-planning-xzzf5fx2/evidence/clean-last-tab.json), [syntax-invalid control](file:///Users/petertam/.godot-close-planning-xzzf5fx2/evidence/syntax-invalid-last-tab.json) and [read-only control](file:///Users/petertam/.godot-close-planning-xzzf5fx2/evidence/read-only-last-tab.json).
- [Before](file:///Users/petertam/.godot-close-planning-xzzf5fx2/evidence/stock-before-close.png), [after](file:///Users/petertam/.godot-close-planning-xzzf5fx2/evidence/stock-after-close.png) and [last-tab closed](file:///Users/petertam/.godot-close-planning-xzzf5fx2/evidence/stock-last-tab-closed.png) owned-window images. SHA-256 respectively: `25ae8c1977367cbc28d29ecf07618c6003589663e0d9a057be2eb73ccf72b678`, `f16c17fc115853e7a974413c0bb50e2ad6ce5c541e5b6379b9c7b12df58d546f`, `42de09f82ae9def784542ed10f7d75030acbf6cab7483bc1b7bec9be4578883a`.

All **four owned editors exited 0 and were reaped**. Temporary projects, fixture code, control/registry files and the compiled window helper were removed. Raw editor logs were discarded rather than published as incidental source-bearing evidence. These local evidence links record provenance, not contributor prerequisites. The durable control sequences below describe what was exercised.

## 4. Public close route and reachable effects

### 4.1 Exact path and dirty-target discard

**Source-established:** The public [Godot 4.7 ScriptEditor API](https://docs.godotengine.org/en/4.7/classes/class_scripteditor.html#class-scripteditor-method-close-file) documents `close_file(path) -> Error` and explicitly says it discards unsaved changes. [Pinned `ScriptEditor::close_file`](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp#L2649-L2665) finds a tab by Resource path, calls `reload_from_file()` if that tab is unsaved, and calls `_close_tab(i, false, current == target)`. It has no expected-revision/identity parameter and no safe-discard flag.

**Decision:** A product request must never use the API as a dirty-state test or as already-closed recognition. Any effectful route requires independently clean target state and fresh same-target/source/version checks at native entry. Existing confirmed-closed observation is the no-effect recognition route. Path alone is not authority over a replacement buffer.

**Alternatives considered:** Selecting a tab and invoking a shortcut/menu violates the non-selected requirement and adds focus dependence. Calling private `_close_tab`, removing a child or freeing CodeEdit bypasses the public lifecycle boundary and does not eliminate or reproduce all required editor bookkeeping. No evidence justifies an engine patch at this stage.

### 4.2 Native navigation, source application and validation

[Native `_close_tab`](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp#L540-L600) emits the close notification, removes target navigation-history entries, saves target editor UI state and deletes that editor control. It may navigate backward when closing the selected target, and normally calls `_go_to_tab` when a tab remains. Source-local history disposal follows ordinary target CodeEdit destruction; this is not an undoable close or a source-history edit.

[Native `_go_to_tab`](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp#L360-L431) calls `apply_code()` on an unsaved current editor, selects the target tab, emits selection notification and calls `validate_script()`. [Script-list sorting](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp#L2021-L2055) can visit previous/current tabs; [history navigation](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp#L3495-L3547) also validates the visited script. Guarding only the closing target is therefore insufficient.

[TextEditorBase dispatch](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_base.h#L219-L223) reaches `CodeTextEditor::validate_script`; [its implementation](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/gui/code_editor.cpp#L1696-L1703) starts the idle timer, whose timeout emits the validation signal. [ScriptTextEditor validation](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_text_editor.cpp#L841-L900) invokes language validation, may set loaded source/update exports and processes pending dragged-export state. Its final base validation emits `edited_script_changed` ([source](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_base.cpp#L522-L525)). This is a deferred native continuation, not something proved finished by `close_file` returning.

**Observed negative:** In a synthetic project with a clean non-selected target and a selected unrelated dirty script with R != B, one native close changed the unrelated R to its pending B. B and D remained as independently observed, but preservation of the unrelated loaded authority failed. This actor is ordinary human editing, not malicious software. A product route must refuse this context before closing, not report the later source application as acceptable reconciliation.

**Observed positives:** A clean non-selected target closed while the other clean script remained selected, preserving that script's IDs, R/B, dirty/current/saved versions, Resource edited state and history availability. The same held with a dirty current script after ordinary fixture-preparation validation had already made R == B. Actual Undo restored its earlier source and Redo restored the unsaved change after target closure. The product must not perform that preparation or wait for convergence to manufacture eligibility.

**Observed revalidation:** After clearing the event ledger immediately before the close call, the remaining script emitted a fresh `edited_script_changed` validation event after return in clean non-selected, dirty-current-equal and selected-target controls. Separate snapshots and independent disk witnesses preserved the admitted sources in those positives. Stock native revalidation is thus real even when source does not change, which is the unresolved FR-005 conflict—not a hypothetical risk.

### 4.3 Selected, last-tab, syntax-invalid and read-only controls

Closing a clean selected target left the other script selected and preserved its source, flags, versions and native history witnesses; subsequent native revalidation was observed. Closing the final remaining tab yielded an independently empty open-script list and no current script. A clean syntax-invalid single script (`extends RefCounted` with an incomplete variable declaration) and a clean read-only single script also closed successfully without source repair, write, reparse-as-a-validity prerequisite or disk byte/stat changes. These last-tab controls do not establish that closing with other tabs avoids their validation.

A same-path reopen during fixture preparation reused the retained Script ID but produced different editor and buffer IDs. Existing expected-basis association checks must include all actual document identities, not only the cached Script or source hash. This observation does not claim universal lifecycle race protection or require an invented durable incarnation service; exact identity protection remains a native-boundary design obligation.

### 4.4 Retained R and history applicability

The selected stock controls retained the closed subject's Script in cache while its editor/buffer was absent. Source was independently read from that actual retained Resource. No unload was forced, no B was fabricated and no automatic reopen served as verification. The plan must also retain the explicit unloaded-versus-unobservable distinction rather than require a Resource to outlive native ownership.

The preserved unrelated history was exercised through real CodeEdit Undo and Redo, not just `has_undo`. Target-local disposal is the documented native close consequence, not a guarantee of persistent undo history across reopening. Product late-cancellation/reopened-buffer protection and cumulative A–E/durability/export tests were not performed by this study.

## 5. Core, caller and compatibility findings

**Existing facts:** [Observation evidence](../../mcp-server/src/observation.rs) includes checked request/project/session/path, Script/editor/buffer instance IDs, file identity, collection stamps, source witnesses and current buffer version. Each independent source is limited to **512 KiB**. [ExpectedRevisionBasis](../../mcp-server/src/script_edit/request.rs) derives a clean, stable, agreeing open-document basis with SHA-256/UTF-8 length and independently attributed witnesses; ordinary observation does not invent a saved version. Native preparation can independently acquire current/saved versions under the same existing public editor getters.

**Decision:** Reuse the same evidence semantics, exact target resolver, project confinement and supervised attempt clock. Do not use a digest alone as mutation authorization or move close behavior into read-only observation. A distinct close request/outcome should distinguish a basis-bearing attempt to close an open document from no-effect recognition; missing basis cannot authorize an open close.

**Compatibility decision:** [Current bridge v4](../004-discover-project-gdscript/contracts/bridge-protocol.md) authenticates an exact eight-boolean capability vector; current [native integration](../../godot-addon/native/README.md) is revision 2. Select **bridge v5**, appending authenticated `close_gdscript`, and **native revision 3** for the extended callable family, shared owner and close-continuation lifetime contract. The revision change identifies the actual new native family, rather than claiming revision 2 has the new semantics. Retain `editor_integration` artifact/entrypoint names and deliberate exact-build matching. Migrate all current peers, fixtures, capability/native checks and installation notices together, with no dual-version fallback; public observation/edit/open/discovery remain v1.

**Bounded operation:** Reuse the existing 9.5-second work budget and 0.5-second result-delivery reserve, beginning before parsing/selection. Preserve source-free stderr and one bounded structured stdout result. The parent records possible application before one-shot authorization; a timed-out worker cannot imply an entered editor call was cancelled or rolled back. Existing single operation ownership must extend through entered work and any required close verification/continuation, without queuing, replay or taking ownership of a newly reopened buffer.

**Alternatives considered:** Importing the complete open/edit state machine would carry irrelevant construction, compilation or persistence effects. A new lifecycle framework, slot or replay journal is unnecessary. The stable existing checked revision-basis conversion is reused internally without a duplicate close algorithm, new public alias or public edit API migration. Close-specific wire/stage names and source-summary output are defined in the [data model](data-model.md), not inferred from edit outcomes.

## 6. Verification decisions

Reuse [the existing owned harness](../../godot-addon/tests/run_script_edit.py), [opening runner](../../godot-addon/tests/run_script_open.py), [serial campaign utility](../../godot-addon/tests/run_editor_campaign.py) and [current validation policy](../../.github/README.md). No close runner, public close binary or campaign close suite currently exists; proposed names must be labeled future implementation interfaces in the final quickstart, not presented as commands already available.

Required close-specific coverage is positive selected/non-selected/last-tab/read-only/invalid-source behavior; already-closed cached/unloaded recognition; dirty/equal-dirty/divergent/missing/unsupported/routing refusals; namespace/session/document and same-text reopen races; overlap and terminal-discard safety; interruption before/after entry and during verification; actual unrelated native history; at least 20 requests with the specification's positive/no-change/dirty-refusal minima; composed discovery/open/observe/edit/close/reopen; source privacy and enabled/disabled/hook-only production exports.

The demonstrated unsafe unrelated-source application becomes a refusal regression. The permitted automatic revalidation continuation requires exact state/effect completion evidence. A close return or short fixed sleep cannot replace independent postconditions. A request-local ledger of actual ScriptEditor selection and protected ScriptEditorBase validation signals is selected; no universal callback registry or fictional pending-export-empty getter.

During implementation, run the task-owned affected case first. A shared authenticated-version/native-family/operation-owner cutover requires complete affected existing observation/edit/open/discovery suites, and the feature cumulative gate requires all close groups plus applicable existing A–E/history/durability and export evidence on unchanged runtime inputs. This is an obligation to demonstrate behavior, not to add a provider-specific GUI runner. Existing optional hosted GUI scope is not silently expanded or treated as evidence for unexecuted suites.

## 7. G1 — Native revalidation decision

**Resolved 2026-10-01:** The user selected “Allow guarded native revalidation.” The spec's clarification, US1.2, US2.5, edge cases, FR-005 and assumptions now record the permitted boundary; the feature was not reduced to last-tab-only closing.

The initial hold met the repository's four-part provenance rule:

1. **Original requirement:** FR-005 prohibited `reload/reparse/rescan` without distinguishing ordinary native close bookkeeping; US1.2/FR-008 required useful closing while another script remained selected, and FR-017 excluded refusal-everywhere behavior.
2. **Concrete in-scope mismatch:** The only identified public native exact-path close route schedules language validation of a remaining script, even for an unchanged clean source. Treating this native continuation as absent would violate the literal no-reparse rule and make diagnostics untruthful.
3. **Supported actor/environment:** Ordinary local Godot 4.7.2 editing, including clean tabs and human dirty work, in the existing supported local-user model. No hostile plugin, speculative attacker or new isolation requirement is involved.
4. **Reachable evidence:** Pinned close/navigation/validation source above and the actual visible-editor close-triggered validation events in the retained clean/non-selected/selected controls. A separate negative established why source-preservation guards remain necessary.

**Accepted decision:** Permit ordinary native close-triggered validation of unchanged already-loaded source only when its effects are admitted, independently verified and preserve D/R/B, unsaved state and unrelated history. Continue to prohibit explicit source repair/reload/reparse/rescan operations, applying differing pending buffer text, new project-code execution, Save/discard and unsafe/unobservable contexts. The target's own safe syntax error is not a parse-validity requirement. Native assignment of an identical source value does not authorize synchronizing differing authorities.

**Rejected alternative:** Keep the blanket no-reparse rule and leave the public route unable to meet remaining-tab cases. No last-tab-only scope cut, validation suppression, private engine call or engine fork was selected.

This is an approved specification clarification, not a constitutional amendment. Human-work protection, confinement, truthful evidence and all applicable gates remain unchanged.

## 8. Selected native protection and complexity decisions

**Decision:** Capture and guard the complete bounded remaining-document set, at most eight open script documents including the target and at most 512 KiB aggregate private remaining source. Native history-back and list sorting can visit more than the visibly selected document, while no supported public getter exposes the complete native history stack. Every remaining document must already have independently equal R/B and attributable flags/versions; dirty R==B is permitted. Apply the existing bounded opening lexical/compiled/effective-context profile and private one-shot validation to each remaining source, sequentially under the original request deadline. Keep the target separate and do not demand target parse validity.

**Rationale:** Guarding only the current tab misses reachable history/sort destinations; checking that every unrelated tab is clean is stricter than the observed risk and would discard the useful dirty-but-current positive. Inspecting the bounded possible set through existing getters is simpler than reading private history, predicting navigation or introducing an event-history service. The aggregate bound controls source storage/encoding and child work. Unsupported contexts refuse without changing configuration or asking the human to approve unsafe effects.

**Decision:** Observe actual visited protected editor IDs during the synchronous close and their post-entry `edited_script_changed` completion events. The native attempt owns this finite ledger and verifies unchanged source/identity/flags; a single addon wait observes completion within the existing lease. Equal source or a fixed delay is not a completion fence. Invalidated/missing events lead to known applied-unverified/unknown outcomes as warranted; no timer flushing, forced parse, reclose or compensating reopen occurs.

**Rationale:** The call returns before the observed native validation event. A return-only design can report success before source-changing pending work runs. The ledger uses existing public signals with current consumers and explicit cleanup, not a generic event bus or permanent monitor. Missing signals are an explicit supported-profile refusal/unverified condition, not fabricated proof.

**Shared code ownership:** Reuse the existing native `open_context` source/effective-context machinery through a small private responsibility-based extraction into `editor_context`; keep opening's current-document orchestration distinct from close's bounded roster/continuation handling. No second lexical scanner, new parser, public backend trait or unrelated refactor. Native close and callback/descriptor lifetime remain in a close-owned module under the existing Session owner variant.

| Current requirement/failure | Selected mechanism | Why simpler existing behavior is insufficient | Cost justified now |
|---|---|---|---|
| Agent must close one clean document | One close domain/caller/owner and stock public close | Observation cannot close; a raw close may discard work | One present user capability, not generic setup. |
| Native route can apply unrelated pending text | Bounded complete candidate-set R/B/identity/version/effect guards | Target-only/current-only checks miss native history/sorting | Up to seven private contexts; no source returned or new service. |
| Native validation can process effectful source/exports | Reuse current source-only validator and lexical/compiled profile as `close_context` | R/B equality alone does not certify native continuation effects | Existing child/process machinery, one child at a time, original deadline. |
| Close returns before native validation completes | Attempt-owned visited/completed editor ledger and independent final acquisition | Return code, equal text and fixed sleep cannot establish completion | Bounded signal hooks with current cleanup ownership, no persistent monitor. |
| New fixed authenticated/native family | Bridge v5 and native revision 3 coordinated cutover | v4/revision-2 contracts do not identify the new capability/lifetime | Required current consumers migrate once; no compatibility shim. |
| Native context now has two real consumers | Private `editor_context` extraction | Copying opening's scanner/compiled checks creates competing safety logic | Behavior-preserving current-task split; no generic framework. |

The [plan](plan.md) and design contracts own final fields, transitions, integration paths and acceptance commands. The selected combined guards/ledger are an **implementation design**, not a claim that this research exercised a complete guarded product close. No unresolved user decision or new infrastructure prerequisite remains.
