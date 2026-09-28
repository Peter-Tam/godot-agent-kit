# Implementation Plan: Safely Edit Open GDScript

**Branch**: `spec/002-edit-open-gdscript` | **Feature identifier**: `002-edit-open-gdscript` | **Date**: 2026-09-27 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/002-edit-open-gdscript/spec.md`

**Status**: **[§17 M3/F-blocking](research.md#17-minimal-behavioral-finalization-research-2026-09-28): the tested minimal no-handler finalization fails required later ordinary Save/history behavior; no bounded supported narrow repair is established.** [§14 P1](research.md#14-stock-post-persistence-saved-transition-research-2026-09-28) remains the held **saved-state-only** handler candidate, not an admitted production route: [§16 H3](research.md#16-stock-saved-handler-admission-research-2026-09-28) established no useful safe callback admission profile for its demonstrated composition. M3 neither proves every handler-free route impossible nor proves the full handler necessary. An explicit exact-source stock validator is independently unselected/unproved. T001's reusable core and T002's patched read-only validation remain complete with [core](quickstart.md#9-t001-core-acceptance-2026-09-27) and [native](quickstart.md#10-t002-native-validation-acceptance-2026-09-28) acceptance evidence; T003 is unstarted/on hold, T004–T005 and mutation A–E/durability acceptance pending. The product target remains official stock Godot, addon and bundled standard GDExtension.

## Summary

Deliver one native, revision-guarded whole-source edit to an existing open standalone GDScript in one explicitly selected local editor. A caller supplies a checked prior-observation basis and intended source; Rust/core freshly checks independent D/R/B, dirty state and identities, authorizes application, and independently verifies resulting source, clean/saved state and source-attributed parse evidence before success. Refused/unchanged requests add no source/history changes; partial/uncertain changes never imply rollback or retry safety.

The **held saved-state candidate** is the [§14 P1 stock post-persistence transition](research.md#14-stock-post-persistence-saved-transition-research-2026-09-28), not the earlier engine finalizer A. Its public stock ingredients below describe a candidate, not final production wire/API shapes or an implementation instruction. [§15 V2](research.md#15-stock-validation-and-effect-confinement-research-2026-09-28) and [§16 H3](research.md#16-stock-saved-handler-admission-research-2026-09-28) leave actual dependency/parser/shallow/export generations and reverse consumers, current-editor/function-discovery and ordinary validation/export callback closure, and shared deferred runtime routing/completion **independently** unresolved before dispatch of that handler. Observed root/helper bytes, public metadata, cache getters or post-effect state do not prove consumed generations or exclude forbidden dispatch. [§17 M3](research.md#17-minimal-behavioral-finalization-research-2026-09-28) separately rules out only the tested minimal no-handler sequence on behavioral grounds. The explicit exact-source validator still needs its valid/invalid/unavailable parser/analyzer/diagnostic contract; neither an adapted extension validator nor an isolated helper alone controls existing native editor continuations. T003 remains held, not implementation-ready.

Shared boundary (planned; saved-state mechanism unselected; no product mutator exists):

```text
edit-gdscript / typed caller
  -> existing Rust core and supervised worker / authenticated bridge (planned)
  -> addon + standard C-ABI GDExtension on official stock Godot
  -> exact CodeEdit native edit -> explicit Script R synchronization
  -> retained-descriptor D persistence -> supported behavioral finalization (unselected)
  -> eventual confined native validation -> independent D/R/B/dirty/saved/context verification
```

The previous [§11 patched-engine decision](research.md#11-concrete-native-design-and-resumed-planning), [§13 ordinary Save research](research.md#13-stock-target-save-finalization-research-2026-09-28), and [Phase 1 concurrency boundary](spec.md#phase-1-concurrency-boundary) remain evidence/requirements. §13 disqualifies Save routing through a custom ResourceFormatSaver (non-OK falls through to builtin path writes, including observed outside-project redirection), all-script target Save (changes unrelated dirty R/history), and direct `ResourceSaver.save` (no ScriptEditor saved transition). §14's **direct actual ScriptEditor saved-handler Callable**, discovered via canonical public incoming `resource_saved` connections, is retained only as a held saved-state candidate under §16 H3. [§17](research.md#17-minimal-behavioral-finalization-research-2026-09-28) tests a narrower route without any handler and finds a false later external-change Save prompt and disrupted Save/history sequence, not proof the handler is necessary. No engine A patch or custom saver is selected. T002's patched validator completion is real and separate; it does not prove a stock validator/effect guard.

## Technical Context

**Language/Version**: Existing Rust **1.98.1**, edition 2021; GDScript for the thin editor integration; **C++17** for the small standard-ABI extension. Python **3.10+** remains the owned-fixture driver environment.

**Primary Dependencies**: Reuse existing locked `serde =1.0.229`, `serde_json =1.0.151`, `cap-std =4.0.3`, `ring =0.17.14` and existing system toolchain. No new Rust crate, godot-cpp layer, parser, process service or transport. Generate/use the pinned engine's public GDExtension C interface and opaque object/Variant dispatch, preserving upstream SDK notices. Native OS I/O uses the demonstrated macOS descriptor APIs.

**Engine and version scope:** The historical PR #24 candidate selected patched Godot **4.7.2**, base commit `ed1daf0bf001b61586d9930840f2f1394092c079`, plus proposed A/guard and a matched extension. That A patch is superseded for saved-state finalization; no A binary exists. T002's patched validator acceptance remains complete, not stock mutation support. The selected saved-state research exercised official stock Godot **4.7.2** on maintainer-operated **macOS 26.6.2 arm64** GUI, with a standard C++17 public-ABI native probe; other platform/version support, runtime-game effects and product A–E acceptance are unproven. A future implementation must establish its exact build/topology capability and real GUI evidence.

**Storage**: Existing owner-private source-free session registry; one existing project script; request-local memory/descriptors/evidence. No new persistent mutation token store, source cache, replay cache, backup/journal or recovery service. A descriptor-bound write is not crash-atomic storage.

**Testing**: Pure core boundary/transition tests; real supervised-worker/private-bridge tests; native primitive regressions; actual visible-Godot A–E plus Save/reopen/reparse/rescan/runtime and privacy/export/observation regression. Reuse existing owned-editor harness facilities, independent witnesses and deadlines. The [quickstart](quickstart.md) is the future execution/evidence contract, not a passing-results record.

**Project Type**: Existing local Rust library/CLI plus editor addon/native integration. No MCP exposure.

**Performance Goals**: Controlled edit result ≤10 seconds (9.5-second acquisition/operation cutoff + 0.5 delivery reserve), including an unresponsive editor. Observation remains ≤5 seconds with its existing 4.5-second supervision. No sustained-throughput or arbitrary-project-size claim.

**Constraints**: One open standalone script; explicit target/session and expected basis; exact independently agreeing D/R/B and attributable clean state; native single-step history; target-bound non-redirectable persistence; no arbitrary evaluation/execution/outside-project access; truthful partial outcomes; no automatic retry/rollback. Success is an observed interval, not a frozen editor or arbitrary same-inode serialization.

**Scale/Scope**: Root/replacement and each source ≤512 KiB UTF-8; bounded 32 dependency sources / 4 MiB dependency bytes; one active editor collection/edit slot; inherited 32-peer/descriptor bounds. Full-source LF representation, explicit unsupported NUL/CR/BOM refusal, and native Save-profile eligibility preserve exact source rather than normalize it. Caller/bridge/diagnostic bounds are in the contracts.

## Selected Stock Saved-State Composition (Design Only)

The following describes **held P1**, not the tested §17 minimal route or an
approved production recipe. A supported finalizer must meet behavioral
postconditions rather than numerically reproduce private Godot state.

After fresh independent clean D/R/B, identity, session, open-document, source, CodeEdit current **and saved** version, Save-profile and capability checks, freeze the preparation-time current and saved versions. A pre-edit boundary guard compares current to that prepared current version. One CodeEdit complex operation then changes only the exact target B and advances its current version; freeze this **post-complex-edit current** version for every subsequent persistence and pre-tag guard, while retaining the **preparation-time saved** version. Explicit `Script.set_source_code` synchronizes the held target R with readback; the standard native writer then writes/truncates/flushes/reads back only its retained project-bound descriptor and checks current no-follow namespace attachment. This does not claim an arbitrary same-inode writer lock or crash atomicity. The persistence receipt is private, same-attempt and non-authorizing on its own.

After D persistence, recheck the receipt, exact still-open Script/ScriptEditorBase/CodeEdit associations, selected project/session/path/namespace and current source against the frozen **post-edit current** version and **preparation-time saved** version; equal text at a newer version cannot pass. Using `EditorInterface.get_script_editor()` and `Object.get_incoming_connections()` on that canonical ScriptEditor, verify a unique incoming `resource_saved` connection, its real emitter and actual native receiver Callable against the emitter's public connection list and live object identities. The reflected private method name is evidence only, never a lookup/call API. `EditorInterface.set_object_edited(held_script, false)` performs public Resource bookkeeping, then another fresh guard precedes direct invocation of the retained actual `Callable.call(held_script)`. Do not emit a signal: other listener order can mutate new B or unrelated R before the native receiver runs. No focus-dependent Save, synthetic event, `ResourceFormatSaver`, CodeEdit-only tag, explicit reload operation or patched finalizer.
The final guard after clearing Resource edited state repeats the receipt, namespace, still-open Script/CodeEdit/session/path/source and frozen post-edit-current/preparation-saved version checks; there is no signal broadcast or other callout between this guard and the native tag. For example, preparation at current/saved **6/6**, one complex edit to **8/6**, and both pre-tag guards at **8/6** lead to native saved tagging at **8/8**; a human edit to current **10** or same-text **12** invalidates the attempt even if source bytes later agree.

The native saved handler tags CodeEdit's saved version **before** document path-mtime tagging; its subsequent built-in path scan, name refresh and live-script-reload trigger mean this call is **not** a pure, no-callback, tag-last span. Refuse ambiguous built-in sibling path associations and loss of the exact cached Script; UI list refresh itself is not a purity proof: the names tail calls `_update_members_overview` → `ScriptTextEditor::get_functions`, which validates the **current** editor's B/path. When the target is current, it can validate the target; a later validation `name_changed` can repeat discovery for a different then-current editor. Never select/focus the target to change this at operation time. The trigger can execute native validation even when automatic runtime reload is disabled and may conditionally queue shared deferred debugger reload. Project metadata and an unchecked menu item do **not** prove the effective native scheduling flag is off: [§16](research.md#16-stock-saved-handler-admission-research-2026-09-28) observed actual runtime 17→31 with metadata/menu false; controlled native menu actions in disposable setup bounded effective off (17) and on (31), not a product safety maneuver or version-bound completion witness. No running-game or effect-confinement guarantee follows. No accepting admission profile exists; resolve the independent callback and explicit-validator obligations before T003.

Public Resource edited state is observable, but the private Resource mtime is **not** numerically read or set by P1; do not require equality to descriptor mtime. The handler natively derives the ScriptEditorBase document mtime from path after tagging the CodeEdit version. §14's tested Save/reopen behavior across a natural timestamp-second change was bounded candidate evidence, not all-metadata or safe production proof. [§17](research.md#17-minimal-behavioral-finalization-research-2026-09-28) makes document timestamp freshness behaviorally relevant: after the minimal no-handler tag, a later human Save falsely prompts “Files have been modified outside Godot” despite a successful-looking clean D/R/B and Resource flag. Neither writable private fields, numeric parity nor full Save callback history follows as a requirement. After partial bookkeeping or a later human edit, never restore/tag newer B or retry; known B/R/D changes remain applied-unverified. Success needs independent D/R/B, identity/namespace, dirty/current/saved-version, context, eventual validation and normal subsequent Save/history behavior, not a receipt, tag or Callable return.

T001's completed core acceptance applies to its then-approved typed evidence contract. The implemented `SavedStateEvidence` currently has numeric `resource_mtime` and `document_mtime` fields, and the reducer requires both to agree with independent disk mtime and complete finalization mtime flags. That historical interface cannot be populated from unavailable private Resource mtime on stock P1. Within existing T004 caller/core integration, migrate the affected typed saved-state evidence, finalization/reducer checks and contracts before consuming P1 facts; never fabricate numeric metadata or retain the old mtime equality as a stock success gate. The exact replacement API/wire shape is not settled by this research, and T001's historical completion is not reopened.

The held P1 candidate carries conservative version/topology checks and standard-extension guard/readback maintenance rather than installed custom-editor trust, per-version patched binary distribution and replacement maintenance. It adds no new service, policy engine, approval layer or generalized transaction. [§14](research.md#14-stock-post-persistence-saved-transition-research-2026-09-28) records bounded saved-state evidence, **not** T003 readiness or A–E proof; [§17](research.md#17-minimal-behavioral-finalization-research-2026-09-28) records why the less callback-heavy tested alternative fails its later Save/history behavioral gate.

Pinned upstream basis: [public incoming connections](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/core/object/object.cpp#L1411-L1421), [native saved handler and built-in-path scan](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp#L723-L803), [document tag/path mtime](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_base.cpp#L91-L93) and [public edited setter](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/editor_interface.cpp#L714-L721). The official stock 4.7.2/macOS arm64 research completed 18 GUI cases and 95 independently checked facts, including a separately inspected visible non-selected dirty tab; the bounded stock reopen logged a `get_line_wrap_count` diagnostic. These are mechanism/lifecycle research, not an error-free UI or full mutation acceptance.

**§17 behavioral disposition:** The tested guarded native B edit → explicit
Script R sync → retained-descriptor D persistence → Resource edited=false →
CodeEdit saved-version tag, **without any saved handler**, initially yielded
independently observed clean D=R=B and preserved target identity, focus and
unrelated dirty work. A later actual stock Save after newer human typing
instead displayed a false external-change dialog and did not persist that
typing; a separate Undo → ordinary Save → Redo → Save lifecycle hit that
dialog and exposed an extra history step. A valid unrelated dirty no-final-newline
fixture remained unchanged during the candidate interval. Reopening was clean;
active hot reload was not observed and is not a specification requirement.
The [source/remedy audit](research.md#17-minimal-behavioral-finalization-research-2026-09-28)
found no supported narrow behavioral reconciliation satisfying target/history/
effect constraints. This is a measured **negative for that route**, not green feature
acceptance, a universal handler-free impossibility result or permission to call
the held P1 handler. During one guarded attempt intermediate D/R/B/internal
divergence is permitted, but guards, no newer-human overwrite, target/namespace
confinement, no forbidden unrelated/irreversible effects, truthful partial
outcomes and independently verified final and subsequent Save/history behavior
remain mandatory; no external atomic-every-step promise is made.

## Historical Patched Primitive A Candidate (Superseded for Saved State)

### 1. Core intent, revision and compatibility

Add a focused script-edit module to the existing Rust library, using existing checked selectors, identity/collection/source evidence and classification patterns. The core consumes protocol-independent evidence; no JSON, TCP, Godot object or native ABI dependency enters its policy. Do not build a general Phase 2 transaction/revision framework.

One new `edit-gdscript` caller accepts the selectors and bounded stdin JSON in [edit-api.md](contracts/edit-api.md). Whole-source intent avoids patch-coordinate ambiguity; source does not enter argv or an arbitrary file-reading option. Expected basis extracts project/session/document/file identity, agreed source digest/length, exact current CodeEdit version and attributable clean state from prior observation. Observation v1 has no saved-version field; acquire it during fresh preparation. Pre-edit guards compare the prepared current version; after the complex edit freeze its new current version while keeping the preparation saved version for the stock transition, as in the [current selected design](#selected-stock-saved-state-composition-design-only). Fresh checks, not that snapshot, authorize the write.

Keep observation's public v1 meaning, timing and per-surface availability unchanged. Private bridge **v2** is a coordinated clean cutover with authenticated edit/native capability fields, changed HMAC domain and strict tuples. Migrate all Rust/addon/worker/fixture peers and docs; no dual-protocol shim or v1 mutation fallback. Restart/re-enable the addon and observe again after upgrade.

### 2. Historical native application and Primitive A (not executable)

Use one native CodeEdit complex operation on the exact bound buffer; preserve previous undo history and unrelated tabs, with normal native redo-branch invalidation permitted. Do not use `set_text`/clear-history as an edit shortcut or create an alternate EditorUndoRedoManager history for TextEdit's existing text history. Never choose the destination by focus.

Immediately recheck current B/version and original R, then use the explicit supported **`Script.set_source_code` method**, not property `_set`, `apply_code`, export propagation or reload. Native source synchronization is not a compiled-class/Inspector/live-game update claim. Pending dragged exports or an unsafe Save representation refuse before application; do not alter editor preferences or scene properties.

The C++ integration pins the selected project and existing file, independently confirms its identity against the Rust basis, and writes/truncates/flushes only that retained object after fresh D/R/B/identity guards. Short-write/error/namespace-loss facts remain explicit. Replacement/parent redirection never changes the write destination. Arbitrary same-inode exclusion remains unclaimed; known changed source/identity still stops obsolete work.

The old A proposal consumed a private persistence receipt after guards and prescribed setting private Resource/document mtimes and tagging saved last without callouts. **Superseded:** the held P1 stock-handler candidate tags CodeEdit saved version before reading document path mtime and can trigger native validation/deferred debugger reload; private Resource mtime is not numerically synchronized. The [held P1 composition](#selected-stock-saved-state-composition-design-only) is not approval to implement either that unsafe tail or the old A contract.

Do not emit `resource_saved`, call Save All/current-tab Save, clear unrelated docs/history or claim normal scene-save effects. The [native Primitive A §3 guarded target-document saved transition](contracts/native-integration.md#3-primitive-a-guarded-target-document-saved-transition) describes the **held P1 Callable candidate** and rejects historical patched A; it is not an authorized T003 production-finalizer requirement. A later supported route must satisfy the current behavioral postconditions and effect safety.

### 3. Primitive B and automatic editor effects

The completed T002 patched engine exposed `GDScriptLanguage::validate` → parser → analyzer on immutable exact source/path and bound actual input hash, target/session/request, invocation interval and root/dependency diagnostics. It returned completed valid, invalid or unavailable, not reload/log/silence proxies. This remains a semantic oracle/reference and hazard inventory, **not** a production patched dependency or a selected stock validator.

Its call-scoped read/effect context permitted parser/dependency-cache activity and confined source-backed analysis with attributed dependency bytes and metadata while refusing excluded loader, initialization and object-property paths before dispatch. On official stock, public ClassDB/global-class/autoload metadata is available; that access is not proof of full parser/analyzer parity or callback confinement. `reload(false)` omits export refresh but still parses, analyzes, compiles and can initialize. The measured `--headless --check-only --script` helper loads fully before EditorNode disables scripting even with `--editor`; non-tool and tool static initializers executed, and invalid cases exited 0. Neither broad reload nor exit/log silence qualifies as validation. A narrower restricted helper or public-ABI source adaptation remains an unselected research alternative; conservative source scanning/refusal is a design-review proposal, not a new semantic parser or a silent `@tool`-only policy.

The historical engine's target-local guard was intended to cover ordinary validation/function-discovery/application work caused by the agent edit, including queued callbacks. T002 does **not** establish a stock callback guard. [§16](research.md#16-stock-saved-handler-admission-research-2026-09-28) confirms the saved handler itself can reach **current-editor** `get_functions` validation via its names tail; ordinary validation can separately synchronize R and update exports, and conditional shared deferred debugger routing can later act on a changed session. Neither an inactive session at entry, a menu/preference false value, handler return, a validation signal count nor a later idle tick certifies version-bound consumption, deferred drain or absence of forbidden effects. Current disk-sensitive diagnostics in same-path stale-metadata cases do **not** prove stale parser consumption; no public consumed parser/dependency generation witness was obtained. A qualifying admission boundary must independently close actual parser/shallow/export generation and reverse-consumer attribution, function-discovery/ordinary callback context, and deferred runtime routing/completion **before** dispatch; the tested public observers and source restrictions did not establish it. An explicit exact-source stock validator remains a separate unproved prerequisite with semantic/diagnostic attribution. Diagnostic source belongs only in the requested bounded result, not incidental native/Rust logs.

Preflight **changed** intent to avoid preventable invalid-source mutation; after the stock saved transition, read actual resulting source and invoke an independently justified native validator afresh. The post-change result, not preflight or saved handler callbacks, contributes to success. An invalid original script remains repairable if all original safety/representation checks hold and the resulting source validates. Unchanged intent runs one non-mutating validation/verification path. The [historical Primitive B contract](contracts/native-integration.md#4-primitive-b-exact-source-native-gdscript-validation) records T002's completed patched validation boundary; stock validation/effects still need separate design.

### 4. Stage ordering, deadlines and independent verification

The former [ordered state machine](data-model.md#3-ordered-attempt-state-machine) retains stage/certainty semantics, but its patched finalization mechanism is superseded. The following is **held P1's conditional candidate sequence**, not the selected production finalizer; the §17 minimal no-handler sequence failed later Save/history:

```text
accept/select
  -> fresh safety + eventual confined proposed-source validation (changed intent)
  -> supervisor records may_apply, then authorizes its existing worker
  -> editor's fresh boundary guards
  -> one native CodeEdit B edit / explicit held Script R synchronization
  -> retained-descriptor D persistence + namespace/readback receipt
  -> fresh receipt, topology, post-edit-current/preparation-saved version, source and identity guards
  -> public Resource edited=false; fresh guard; direct existing native saved-handler Callable
  -> immediate independent post-call state/version/identity rechecks
  -> eventual independent actual-source native validation
  -> independent D/R/B + dirty/current/saved-state + context rechecks
  -> one Rust-classified terminal outcome
```

The first native call that can change source/history is the local potentially-applied boundary. The supervisor becomes conservative earlier, **before** releasing its worker's one-shot authorization, so a lost worker/acknowledgment cannot become a false not-applied result. Before that handoff, preparation cannot apply; afterward only an attributable irrevocable pre-boundary discard can prove refusal. A completed B/R/D change remains known application despite later loss. Reuse the existing supervised worker rather than create a helper service.

Reuse the existing one-active-collection/edit slot for the selected editor/session: claim it during preparation, require its ownership at native mutation entry, and retain it through verification/terminal cleanup. Another edit receives a source-free terminal busy refusal before the potentially-applied boundary, with no source/history/finalization change and no queued or replayable work. Releasing the slot cannot auto-apply rejected intent. A later request needs a freshly observed valid basis; old-basis text compatibility cannot override a changed revision. Human typing remains unblocked. Caller termination does not release this slot while entered native work can still mutate; its uncertainty/deadline semantics remain unchanged. T004 owns the real same-session overlap case under US2.4.

An editor-local remaining-time expiry stops new stages when control returns. Native parser/I/O work can block; the caller's supervised deadline still returns a truthful bounded outcome. Timeout/abort cannot revoke an entered stage or imply rollback; late replies cannot upgrade the result. No retry, reconnect/resume or repair write.

The saved-handler/receipt response is not independent evidence. Rust reads/rechecks D through its own project capability; the existing collector freshly reads actual R and B and document-attributed dirty state, plus the separate native saved-state getter. Recheck source/identity/version and validation dependency/context/save-profile witnesses. Missing, changed, known-stale or divergent evidence prevents success even if an earlier/later sample happens to agree. Return explicit changed/unchanged/refused/applied-unverified/application-unknown and actionable reasons.

### 5. Historical implementation and evidence boundaries (not T003 authorization)

The PR #24 candidate proposed a small pinned engine adaptation alongside addon native integration; its patched Primitive A instructions above remain only for comparison. [§14](research.md#14-stock-post-persistence-saved-transition-research-2026-09-28) selected the public, exact-target post-persistence handler for its **saved-state-only candidate**, not a Save shortcut or approved product route. [§16](research.md#16-stock-saved-handler-admission-research-2026-09-28) concluded H3 without a safe admission profile for the tested composition; [§17 M3](research.md#17-minimal-behavioral-finalization-research-2026-09-28) rejects the tested narrower no-handler route on subsequent Save/history behavior, without proving the full handler necessary or all narrower approaches impossible. The independent exact-source validator is unselected. T003 cannot begin; historical [post-persistence](#post-persistence-design-review), [§15](#stock-validation-and-effect-design-review) and [§16](#saved-handler-admission-design-review) assessments did not evaluate M3. No weakened specification or product mutation support follows.

Historical native lifecycle/export evidence remains relevant to the eventual standard extension, but no patched saved-state hook, private Resource mtime getter/setter or finalizer API is a current implementation instruction.

## Constitution Check

**Historical planning check:** The earlier bounded patched-candidate planning check passed then, not as implementation/acceptance. **Current disposition:** [§13](research.md#13-stock-target-save-finalization-research-2026-09-28) rejects audited Save routes; [§14](research.md#14-stock-post-persistence-saved-transition-research-2026-09-28) retains P1 as a held saved-state-only candidate; [§16](research.md#16-stock-saved-handler-admission-research-2026-09-28) concludes H3 without an accepted handler admission profile; [§17](research.md#17-minimal-behavioral-finalization-research-2026-09-28) establishes M3/F-blocking for the tested no-handler route. No constitutional waiver, relaxed safety guarantee, implementation approval or A–E pass follows. The matrix records obligations, not their discharge. The [§13 review](#stock-research-artifact-review), [post-persistence review](#post-persistence-design-review), [§15 review](#stock-validation-and-effect-design-review) and [§16 review](#saved-handler-admission-design-review) remain truthful historical assessments, not §17 approval.

| Governing obligation | Design and future evidence |
|---|---|
| I / IV — independent D/R/B and observed postconditions | Separate actual D and editor R/B/dirty/current/saved reads; attributed post-change B; immutable partial evidence. Neither receipt, clean tag nor Callable return certifies success. Final intended, clean D/R/B, ordinary Save/reopen and native history behavior plus A–E/durability remain mandatory. Intermediate divergence during a guarded attempt is permitted, not an external atomic-every-step promise. |
| II — preserve human work and stale intent | Exact target/Script/CodeEdit/session/namespace, prepared baseline current and saved versions, frozen post-complex-edit current and preparation saved versions for persistence/finalization guards, fresh source and receipt checks and after-effect independent checks; never restore/tag newer work. No arbitrary-writer serialization promise. |
| III — native editor semantics | One CodeEdit complex edit, explicit `Script.set_source_code`, retained-descriptor persistence and supported behavioral saved-state finalization remain the goal. The tested no-handler public edited=false + CodeEdit tag fails later Save/history; the held P1 handler would tag saved version before document path-mtime but has unconfined effects under H3. No private numeric mtime/cache parity, full Save callback replay, engine A patch, signal broadcast or ResourceSaver is a mandatory end state. |
| V / X — confinement and truthful outcomes | Existing authentication/private metadata; native project-bound descriptor/dependency reads; independently complete, source-attributed valid/invalid/unavailable explicit validation remains unresolved on stock. Separately, mandatory no-unattributed-consumption/no-uncontrolled-effects admission before stock handler/ordinary/deferred work needs actual dependency/parser/shallow/export generations and reverse-consumer attribution, current-editor/function-discovery and validation/export callback closure, plus shared deferred runtime routing/completion. §16 establishes no useful safe accepting profile for its tested composition; source restrictions, disk hashes, public flags, callback acknowledgment or post-effect observation do not discharge these obligations. Preserve separate stage/application/parse/dependency/permission/timeout failures and controlled unknown after authorization/loss. |
| VI / XII — layered verification and explicit support | Pure transition/boundary tests plus real visible-editor A–E, history, durability, timing, privacy/export and complete observation regression on exact recorded builds. No stock/other-version mutation support extrapolation. |
| VII / IX — protocol-independent, small composable surface | One typed/core edit and one CLI, existing private bridge, no MCP; core owns policy, native owns engine-local primitives. No duplicate transaction model. |
| VIII — tooling/gameplay isolation | Editor-only native integration; real exports and runtime checked with enabled/disabled addon and native artifacts, not reliance on `addons/` or `@tool`. |
| XI — independent implementation and dependency review | Public engine source/ABI, own narrow integration, existing locked Rust dependencies. Preserve SDK notices, review exact new native build inputs/advisories/licenses before their implementation delivery. No unreviewed copied framework. |
| XIII — proportional complexity | **Requirement:** independently prove target-bound clean/intended D/R/B, native Undo → ordinary Save → Redo → Save and earlier history, later human Save without false outside-change reconciliation, reopen/persistence, newer-human/namespace/receipt guards, no forbidden unrelated/effectful work and truthful partial outcomes; an explicit exact-source valid/invalid/unavailable stock validator remains separately required. Active runtime hot reload is not a spec requirement, but uncontrolled runtime effects are forbidden. **Simplest tested alternative:** a guarded no-handler Resource edited=false + CodeEdit tag reduced handler topology/callback costs but failed actual later Save/history after a natural timestamp change; the source audit found no supported narrow behavioral reconciliation preserving live document/history/effect constraints. **Principle XIII cost:** synchronizing private invariants adds code, coupling, version sensitivity, testing and architecture costs that require a concrete observable failure; the observed false Save justifies behavioral reconciliation, not private field parity or whole-handler replay. **Held P1 cost:** incoming Callable topology and generation/reverse-consumer, current-editor/ordinary-callback, deferred-runtime attribution and completion remain unresolved under H3, so P1 is not admitted. A helper/adaptation alone cannot guard existing stock continuations; a source scanner is not a compiler or effect fence. **Principle XIII decision:** no new effect system, cache manager, scheduler, debugger framework, custom editor or patched fallback is justified now. Any later candidate must earn ABI/version/drift, provenance, guarded lifetime and real GUI-differential cost against these invariants before selection; cheaper distribution never waives source, history, human-work or confinement safety. |
| Architecture/compatibility/workflow | Existing Rust core and eventual private bridge/caller cutover retain responsibility. No final API/schema or production finalizer selected. T003 is unstarted/on §17 M3/F-blocking and §16 H3 design hold pending a supported behavioral finalization architecture, confinement of any unavoidable callbacks and independently proven explicit validation; T004/T005 retain dependencies and acceptance. |

All A–E are applicable, including C because native Undo/Redo is claimed. At least one permitted runtime fixture must prove changed behavior. No missing runtime/buffer/parse observation is an inapplicability justification. Feature and task completion depend on acceptance evidence, separately from GitHub review/merge metadata. Feature completion would not automatically complete Roadmap Phase 1.

## Project Structure

### Documentation (this feature)

```text
specs/002-edit-open-gdscript/
+-- spec.md                         existing behavioral specification; unchanged
+-- checklists/requirements.md      existing specification-quality review
+-- research.md                     retained evidence + current design decisions
+-- plan.md                         this implementation plan
+-- data-model.md                   identity, evidence, stages and outcomes
+-- contracts/
|   +-- edit-api.md                 typed/caller input, output, compatibility
|   +-- bridge-protocol.md          private v2 and supervised authorization
|   +-- native-integration.md       finalizer/validator/inspection/guard contracts
+-- quickstart.md                   future execution and evidence guide
+-- tasks.md                        subsequent task derivation and granularity review
```

The [task list](tasks.md) retains five PR-sized increments, completed granularity review and unchanged dependencies; T001–T002 are complete. The [§13 rerun](#stock-research-artifact-review) predates P1; the [post-persistence review](#post-persistence-design-review) predates §15; the [stock validation/effect design review](#stock-validation-and-effect-design-review) assessed §15; the [saved-handler admission design review](#saved-handler-admission-design-review) assessed §16 H3. Those results remain historical. The [current minimal behavioral finalization design review](#minimal-behavioral-finalization-design-review) assesses §17 M3/F-blocking without authorizing T003.

### Source Code (repository root)

Existing files are retained; `planned` entries below are implementation ownership, not files created by this planning PR:

```text
mcp-server/
+-- Cargo.toml / Cargo.lock         existing single package, locked dependencies
+-- src/
|   +-- lib.rs                     exports reusable core
|   +-- observation.rs             existing read-only semantics
|   +-- script_edit.rs             public edit-core facade
|   +-- script_edit/               private request/evidence/validation/outcome/attempt modules
|   +-- runner.rs                  extend existing supervised execution boundary
|   +-- project_fs.rs              existing independent D/confinement helpers
|   +-- bridge.rs / bridge/        existing session/authentication/framing boundary
|   +-- bridge/wire.rs             existing caller/worker/bridge JSON codecs
|   +-- bin/edit-gdscript.rs        planned thin caller entrypoint
+-- tests/                         required core/boundary behavioral regressions

godot-addon/
+-- addons/godot_agent_kit/
|   +-- plugin.gd / bridge.gd       existing lifecycle/routing; v2 cutover
|   +-- observation.gd             existing getter-only observations
|   +-- script_edit.gd             planned thin native-attempt integration
|   +-- export_guard.gd            extend native tooling exclusion
|   +-- native/                    planned installed editor-only extension/artifact
+-- native/                        existing T002 patched-validator source/build; planned stock writer/integration, no A patch
+-- tests/
    +-- run_observation.py         retain all existing behavior acceptance
    +-- run_script_edit.py         planned focused mutation acceptance runner
    +-- fixtures/                  reuse owned-editor setup/witness facilities
```

**Structure Decision:** No new Rust package or top-level product component. Standard native integration belongs to `godot-addon/`; agent intent/core remains `mcp-server/`. T002's engine patch remains historical patched-validation evidence, not a planned saved-state patch or proof of stock validator control. Reuse the existing `Harness`, owned fixture setup, window capture, auth and independent witness facilities rather than clone the large observation harness. Extract only helpers actually needed by both runners under responsibility-based names if direct reuse is insufficient; no generic plugin/test framework. Existing shared artifacts must not acquire feature/task IDs in public APIs, transport, logs or ownership.

**Ownership review:** A/B are genuinely GDScript/editor capability-specific; bridge, supervision, confinement and fixture lifecycle are shared and keep responsibility-based names. No duplicate future infrastructure is built, and the script-edit domain is not generalized beyond present needs.

## Complexity Tracking

No constitutional violation/waiver is requested. The held P1 handler route has topology/version checks, request-local descriptor/receipt and current/saved-version guards, Callable identity verification, independent state inspection and real GUI regression costs. [§16 H3](research.md#16-stock-saved-handler-admission-research-2026-09-28) leaves consumed-generation/reverse-consumer, current-editor/ordinary-callback and deferred-runtime effects unresolved. [§17 M3](research.md#17-minimal-behavioral-finalization-research-2026-09-28) measures the lower-callback-cost minimal route and rejects it for false later Save/history behavior; no supported bounded repair was established. Neither route is approved. Exact-source stock validation remains independently unresolved. A custom editor imposes trusted binary installation, per-version patched builds/distribution, team replacement, CI and maintenance and is not a product fallback. No cheaper candidate licenses redirected writes, forbidden execution, unrelated history change or a false clean result.

| Present mechanism/gap | Simplest alternative and insufficiency | Ongoing cost and present justification |
|---|---|---|
| Standard-ABI native library / bound persistence | Public Save paths can follow replacement paths; external writer adds a separate synchronization boundary. | C++ build, ABI/OS/lifetime tests; justified by demonstrated in-editor object/descriptor behavior. No godot-cpp/crate framework. |
| Held P1 stock saved handler after bound persistence; tested §17 no-handler alternative; patched engine finalizer (historical) | The no-handler edited=false + CodeEdit tag produced clean observed D/R/B yet a later human Save falsely prompted outside-change reconciliation and the Undo/Save/Redo/Save lifecycle did not preserve expected behavior. No supported narrow repair was established; full handler necessity and private numeric mtime/cache parity are not proved. Broad Save has fallback/all-script effects; signal broadcast admits listeners. | Retain P1 only as a saved-state candidate: incoming-edge Callable discovery, target/topology/version/receipt guards and independent later Save/history witnesses carry cost, while H3 still blocks admission for callback/deferred effects. Any replacement must justify its own narrower effect/lifetime cost; no saved-state engine patch distribution or full handler replay requirement. |
| Patched T002 validator + scoped read/effect context (completed semantic oracle/reference/hazard inventory); stock explicit validation and callbacks unresolved | `reload(false)` still compiles and can initialize; the broad check-only helper executed static code and returned exit 0 for invalid scripts. Source-derived refusal and public flags/getters cannot establish consumed parser/shallow/export generation or reverse consumers, the current-editor/ordinary callback closure, and shared deferred runtime completion. Neither restricted helper nor public-ABI adaptation is selected; either could address explicit validation only if proven, not fence existing stock callbacks. | §16 supplies a bounded negative admission result, **not** an accepting predicate or an infrastructure proposal. T003 requires separate explicit valid/invalid/unavailable source attribution and useful safe pre-dispatch callback proof; future helper/context or adaptation/drift/differential costs need evidence. T002's patch remains an oracle, never a production dependency. No generic compiler/effect system/cache manager/scheduler, helper service or engine patch selected. |
| Supervisor authorization/stage extension | Read-only worker termination cannot prove a sent mutation never applies. | One control handoff and certainty state in existing supervision; required by FR-012, not a recovery service. |
| Two kit callers based on X enter the same mutation path and overwrite/order changes | Reuse the existing collection/edit slot plus fresh revision checks; that mechanism is sufficient, so no new coordination layer is needed. | Make existing admission/cleanup ownership explicit and maintain T004's barrier regression. This closes a present stale-intent failure with no lock manager, lease service, queue, registry, replay system, merge algorithm or Phase 9 infrastructure. |
| Bridge v2 / edit stdin contract | Strict v1 shape/authentication cannot silently grow; source in argv or arbitrary file options adds exposure. | Coordinated codecs/fixtures/docs migration; one small operation, no compatibility shims or new protocol service. |
| Real stock capability/build proof and edit acceptance runner | Historical patched-editor and observation-only evidence do not prove supported mutation; §17's minimal stock probe fails later Save/history even though immediate D/R/B is clean. | Verify a future safe stock route's exact ABI/topology and standard extension build and exercise owned visible-editor behavioral gates; no custom distribution, provider-specific CI or new approval layer. |
| Historical T002 dynamic-getter validation sentinel | A scripted `Engine.get_meta(...)` expression is not constant-folded, and public autoload type resolution does not supply the analyzer with a live object constant; either misses the actual effect path. | The existing fixed-name `TESTS_ENABLED` fixture object supplied patched T002's real getter sentinel; its completed build/evidence remains valid, not stock mutation proof or a new product operation. |
| Automatic native discovery leaves a skipped tooling manifest in Godot's generated export extension registry | The existing export hook removes the manifest/library but not the engine-generated registry entry; the real hook-only export then fails at runtime. Rewriting the shared exporter or reproducing its preset filtering would add unnecessary scope. | Keep the installed native directory under `.gdignore` and load its existing manifest explicitly through `GDExtensionManager` in the editor plugin. This uses the existing lifecycle, avoids stale runtime registry entries and missing-binary startup errors, and requires only enabled/disabled/hook-only export plus missing-native regressions. Other extensions keep their normal discovery/export behavior. |

## Verification and Planning Completion

[quickstart.md](quickstart.md) maps all 22 requirements, 26 scenarios and eight success criteria to implementation verification, including all A–E, ≥20 successful sequential edits and ≥3 interleaved unsafe refusals. Existing Rust baseline checks and complete observation acceptance remain required where affected. Real GUI evidence may be maintainer-operated; existing hosted/protected workflow boundaries suffice. No CI provider/runner topology is introduced as a new prerequisite.

The original plan-generation pass performed only feature-path/workflow resolution, pinned-source/API review, specification/design coverage and local-link/anchor/Markdown/whitespace/full-diff checks. No Cargo/product suites, engine build, runtime probe, filter retry, A–E acceptance, task generation or analysis command was run by that pass. The installed `setup-plan.sh --json` was executed with the verified Feature 002 directory and retained the existing plan; the installed template was resolved before completing these artifacts. Before/after-plan hook checks found no `.specify/extensions.yml`, so no hook was registered or bypassed. That pass's read-only native/flow contract review findings were corrected without changing the then-current specification.

**Remaining material design obligations — §17 M3/F-blocking and §16 H3:** [§17](research.md#17-minimal-behavioral-finalization-research-2026-09-28) finds the tested minimal no-handler finalization insufficient: independently clean immediate D/R/B does not prevent false external-change reconciliation on a later ordinary Save or guarantee the required native Undo/Save/Redo/Save history. No bounded supported narrow behavioral reconciliation was found; this does not prove all handler-free routes impossible or the full handler necessary. Held [§14 P1](research.md#14-stock-post-persistence-saved-transition-research-2026-09-28) addresses saved-state mechanics only, while [§16 H3](research.md#16-stock-saved-handler-admission-research-2026-09-28) leaves consumed dependency/parser/shallow/export generation and reverse-consumer attribution, current-editor/ordinary callback closure and shared deferred runtime routing/completion independently unresolved before handler dispatch. Separately, no qualifying explicit exact-source stock validator has been selected/proved. Behavioral clean/Save/history/durability, effect safety and validation are required before T003, not Godot-private numeric bookkeeping parity. T003 is unstarted/on hold; five-task scope/dependencies and A–E requirements are unchanged. The historical reviews below assess prior designs; the [current §17 review](#minimal-behavioral-finalization-design-review) assesses this disposition.

### Stock research artifact review

**Historical §13 result only; superseded for the saved-state question by P1 in §14.** The following analysis text is preserved as the actual earlier result, not rewritten as if it assessed the post-persistence design.

The read-only `/speckit.analyze` workflow was rerun on 2026-09-28 with the verified
`SPECIFY_FEATURE_DIRECTORY=specs/002-edit-open-gdscript` override; its installed
prerequisite helper passed and no before/after analysis hooks were registered.
All 22 functional requirements and eight success criteria retain task owners
(100% ownership coverage), with five tasks, two complete, and unchanged scenario
and dependency coverage. No new ambiguity, duplication, unmapped task or
constitutional waiver was found.

**U1 — HIGH, acknowledged implementation-readiness blocker:** no supported stock
one-target saved-state mechanism is selected that closes saver fallback and
unrelated-buffer effects. The explicit T003 hold is therefore correct; this
analysis is **not** a passing implementation-readiness result. Resolve that
mechanism and review affected artifacts before `/speckit.implement T003`.
The independent stock validation/effect-confinement question was not reassessed.

### Post-persistence design review

The installed read-only `/speckit.analyze` workflow was rerun on 2026-09-28
with `SPECIFY_FEATURE_DIRECTORY=specs/002-edit-open-gdscript`; its required
prerequisite helper resolved this feature and passed. No before/after hooks
were registered. Independent semantic and evidence/contract reviews covered
the changed design, constitution, spec, tasks and affected contracts.

All **30 buildable requirements** (22 FRs and eight SCs), **26 scenarios**
and **eight edge cases** retain task ownership: **100% coverage**, five tasks,
two complete and three pending. No unmapped task, harmful duplication,
new requirement ambiguity or critical constitutional conflict was found.

Review found and corrected a temporal-version inconsistency: pre-edit current
is the preparation baseline, but persistence/pre-tag guards freeze the
**post-complex-edit current** and **preparation-time saved** versions
(6/6 → 8/6 → native 8/8). Evidence review also corrected T002's implemented
patched-B wording, identified T001's historical numeric-mtime layout for
explicit T004 migration, and restored historical labels on superseded
caller/bridge/verification recipes. No product code or final API shape changed.

**U1 — HIGH, acknowledged implementation-readiness blocker:** official-stock
exact-source validation and confinement of automatic/edit-generated,
saved-handler and deferred debugger effects remain unresolved. P1 resolves
the saved-state question, not U1. The corrected research/design artifacts are
consistent; this is **not a T003 readiness pass**. T003 has not started.

### Stock validation and effect design review

The installed read-only `/speckit.analyze` workflow ran on 2026-09-28 with
`SPECIFY_FEATURE_DIRECTORY=specs/002-edit-open-gdscript`; the prerequisite
resolved the exact feature override and passed. No before/after analysis hooks
were registered. Semantic and evidence/contract review covered the §15 V2
decision and affected plan, native contract, tasks and status.

All **30 buildable requirements** (22 FRs and eight SCs), **26 scenarios**
and **eight edge cases** retain task ownership: **100% coverage**, five tasks,
two complete and three pending. No unmapped task, new requirement ambiguity,
harmful duplication or CRITICAL constitutional conflict was found. Ownership
is planned work, **not** completed acceptance or tested architecture.

**U1 — HIGH, intentional implementation-readiness hold:** P1 resolves saved
state, not stock validation/effect safety. The next focused design question is
useful source/context admission of unavoidable target-version validation,
export and deferred callbacks on attributed dependency bytes/generations before
forbidden dispatch. Independently, a qualifying explicit stock validator
remains unselected/unproved. Both obligations require proof; one successful
callback experiment cannot make T003 implementation-ready. V2 retains T003
unstarted, without a product edit or A–E/runtime acceptance claim.

Evidence review identified three documentation defects: the historical
command ledger, a pinned LSP citation extending past EOF, and unqualified
single-blocker framing. They are corrected within this research PR; no
product behavior, task count, dependency, P1 mechanism or specification
guarantee changed.

### Saved-handler admission design review

The installed read-only `/speckit.analyze` workflow ran on 2026-09-28 with
`SPECIFY_FEATURE_DIRECTORY=specs/002-edit-open-gdscript`; the prerequisite
resolved the verified Feature 002 override and passed. No before/after
analysis hooks were registered. This review assessed the [§16 H3
research](research.md#16-stock-saved-handler-admission-research-2026-09-28)
and current task ownership, not a stock implementation or acceptance run.

All **30 buildable requirements** (22 FRs and eight SCs), **26 numbered
scenarios** and **eight edge cases** retain task owners: **30/30, 26/26 and
8/8** ownership. Five implementation tasks remain, T001–T002 complete and
T003–T005 pending; no unmapped task, new requirement ambiguity, harmful
duplication or CRITICAL constitutional conflict was found. Ownership is not
completed acceptance. Read-only evidence/source review identified five
material §16 research corrections, and semantic review identified a current
research-introduction inconsistency. All six were corrected and checked
against pinned source and retained runtime evidence; Markdown, local links
and source anchors passed verification. These documentation corrections
do not resolve U1 or establish implementation readiness.

**U1 — HIGH, intentional implementation-readiness hold:** H3 establishes no
useful safe admission profile for the tested stock saved-handler/public-observer
composition, not universal impossibility. P1 remains selected for saved state
only. Actual consumed source/dependency/parser/shallow/export generations and
reverse-consumer attribution, current-editor/function-discovery and ordinary
validation/export callback closure, and shared deferred runtime routing and
completion remain independent pre-dispatch safety obligations. An explicit
exact-source stock validator with attributable dependencies/context and
completed valid/invalid/unavailable results remains separately unselected and
unproved. This research/design assessment is complete, but T003 remains
unstarted and not implementation-ready; no accepted admission predicate,
product edit, A–E acceptance or passing architecture claim follows.

### Minimal behavioral finalization design review

The installed read-only `/speckit.analyze` workflow ran on 2026-09-28
with `SPECIFY_FEATURE_DIRECTORY=specs/002-edit-open-gdscript`. Its prerequisite
helper ran once with `--require-spec --require-tasks --include-tasks`, passed
and resolved the verified Feature 002 override; checks found both BEFORE
and AFTER analysis hooks absent. The review read the spec, plan, tasks,
constitution and full intended diff, with independent semantic and evidence
reviews of the [§17 M3/F-blocking research](research.md#17-minimal-behavioral-finalization-research-2026-09-28).

All **30/30 buildable requirements** (22 FRs and eight SCs), **26/26 numbered
scenarios** and **8/8 edge cases** retain task ownership. Five tasks remain,
T001–T002 complete and T003–T005 pending. No unmapped task, new requirement
ambiguity, harmful duplication or critical constitutional issue was found.
Evidence review identified three §17 documentation corrections—relaunch
autosave attribution, the full excluded OS-delivery list and explicit
requirement classification—tracked in §17; they do not change the measured
candidate failure or establish a production route.

**U1 — HIGH, intentional implementation-readiness hold:** §17 M3 rejects the
tested exact minimal no-handler route on later ordinary Save/history behavior,
not every handler-free approach or proof that the whole P1 handler is necessary.
P1 remains held under §16 H3 for actual-effect safety; an independent explicit
exact-source stock validator is still unselected/unproved. The research/design
assessment is complete, **not an implementation-readiness pass**: T003 is
unstarted, and no accepted behavioral finalizer, safe effect admission,
product edit or passing mutation-acceptance gate follows.

The three publication corrections were verified against retained raw evidence.
Markdown checks passed for all five changed documents; 249 local paths/anchors,
the five-task/two-complete inventory and retained evidence hashes passed.
Owned throwaway projects, probe sources, binaries and generated SDK files were
removed; raw observations and provenance remain local. No production module,
API visibility, lifecycle representation or implementation shape changed.
These are research-publication checks, not T003 or Feature 002 acceptance.
