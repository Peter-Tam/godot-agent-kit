# Implementation Plan: Safely Edit Open GDScript

**Branch**: `spec/002-edit-open-gdscript` | **Feature identifier**: `002-edit-open-gdscript` | **Date**: 2026-09-27 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/002-edit-open-gdscript/spec.md`

**Status**: **Stock post-persistence saved transition selected (P1), not mutation implementation approval; T003 remains on hold for independent stock validation/effect-confinement design and review.** [Research §14](research.md#14-stock-post-persistence-saved-transition-research-2026-09-28) establishes a bounded public stock route for the saved-state question, superseding the historical patched Primitive A below. T001's reusable core and T002's patched read-only native validation are complete with [core](quickstart.md#9-t001-core-acceptance-2026-09-27) and [native](quickstart.md#10-t002-native-validation-acceptance-2026-09-28) acceptance evidence; T003–T005 and all mutation A–E/durability acceptance remain pending. The product target is official stock Godot, addon and bundled standard GDExtension.

## Summary

Deliver one native, revision-guarded whole-source edit to an existing open standalone GDScript in one explicitly selected local editor. A caller supplies a checked prior-observation basis and intended source; Rust/core freshly checks independent D/R/B, dirty state and identities, authorizes application, and independently verifies resulting source, clean/saved state and source-attributed parse evidence before success. Refused/unchanged requests add no source/history changes; partial/uncertain changes never imply rollback or retry safety.

The **current saved-state design** is the [stock post-persistence transition](research.md#14-stock-post-persistence-saved-transition-research-2026-09-28), not the earlier engine finalizer A. Its public stock ingredients and semantic responsibilities are described below; they do not prescribe final production wire/API shapes. The independent stock validation/effect-confinement problem, including native handler callbacks, remains a T003 blocker. No implementation is authorized by this research decision.

Selected boundary (planned; no product mutator exists):

```text
edit-gdscript / typed caller
  -> existing Rust core and supervised worker / authenticated bridge (planned)
  -> addon + standard C-ABI GDExtension on official stock Godot
  -> exact CodeEdit native edit -> explicit Script R synchronization
  -> retained-descriptor D persistence -> guarded stock exact-document saved transition
  -> eventual confined native validation -> independent D/R/B/dirty/saved/context verification
```

The previous [§11 patched-engine decision](research.md#11-concrete-native-design-and-resumed-planning), [§13 ordinary Save research](research.md#13-stock-target-save-finalization-research-2026-09-28), and [Phase 1 concurrency boundary](spec.md#phase-1-concurrency-boundary) remain evidence/requirements. §13 disqualifies Save routing through a custom ResourceFormatSaver (non-OK falls through to builtin path writes, including observed outside-project redirection), all-script target Save (changes unrelated dirty R/history), and direct `ResourceSaver.save` (no ScriptEditor saved transition). §14 instead selects **direct invocation of the actual existing native ScriptEditor saved-handler Callable**, discovered via the canonical ScriptEditor's public incoming `resource_saved` signal connections; neither emitting a signal nor looking up the private handler by name qualifies. No engine A patch or custom saver is selected. T002's patched validator completion is real and separate; it does not prove a stock validator/effect guard.

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

After fresh independent clean D/R/B, identity, session, open-document, source, CodeEdit current **and saved** version, Save-profile and capability checks, freeze the preparation-time current and saved versions. A pre-edit boundary guard compares current to that prepared current version. One CodeEdit complex operation then changes only the exact target B and advances its current version; freeze this **post-complex-edit current** version for every subsequent persistence and pre-tag guard, while retaining the **preparation-time saved** version. Explicit `Script.set_source_code` synchronizes the held target R with readback; the standard native writer then writes/truncates/flushes/reads back only its retained project-bound descriptor and checks current no-follow namespace attachment. This does not claim an arbitrary same-inode writer lock or crash atomicity. The persistence receipt is private, same-attempt and non-authorizing on its own.

After D persistence, recheck the receipt, exact still-open Script/ScriptEditorBase/CodeEdit associations, selected project/session/path/namespace and current source against the frozen **post-edit current** version and **preparation-time saved** version; equal text at a newer version cannot pass. Using `EditorInterface.get_script_editor()` and `Object.get_incoming_connections()` on that canonical ScriptEditor, verify a unique incoming `resource_saved` connection, its real emitter and actual native receiver Callable against the emitter's public connection list and live object identities. The reflected private method name is evidence only, never a lookup/call API. `EditorInterface.set_object_edited(held_script, false)` performs public Resource bookkeeping, then another fresh guard precedes direct invocation of the retained actual `Callable.call(held_script)`. Do not emit a signal: other listener order can mutate new B or unrelated R before the native receiver runs. No focus-dependent Save, synthetic event, `ResourceFormatSaver`, CodeEdit-only tag, explicit reload operation or patched finalizer.
The final guard after clearing Resource edited state repeats the receipt, namespace, still-open Script/CodeEdit/session/path/source and frozen post-edit-current/preparation-saved version checks; there is no signal broadcast or other callout between this guard and the native tag. For example, preparation at current/saved **6/6**, one complex edit to **8/6**, and both pre-tag guards at **8/6** lead to native saved tagging at **8/8**; a human edit to current **10** or same-text **12** invalidates the attempt even if source bytes later agree.

The native saved handler tags CodeEdit's saved version **before** document path-mtime tagging; its subsequent built-in path scan, UI name refresh and live-script-reload trigger mean this call is **not** a pure, no-callback, tag-last span. Refuse ambiguous built-in sibling path associations and loss of the exact cached Script; bounded editor UI list refresh is acceptable, unrelated source/history change is not. The trigger can execute native validation even when automatic runtime reload is disabled; it schedules deferred debugger reload only when enabled. No running-game or effect-confinement guarantee follows. Restrict/consume those callback effects and resolve independent stock exact-source validation before T003 implementation. Preflight and post-change validation remain required, and no stock replacement for patched T002 has been approved here.

Public Resource edited state is observable, but the private Resource mtime is **not** numerically read or set by this route; do not require equality to descriptor mtime. The handler natively derives the ScriptEditorBase document mtime from path after tagging the CodeEdit version. The tested standalone-script Save/reopen lifecycle across a natural timestamp-second change behaved coherently; that behavioral observation does not establish all metadata invariants. After any partial bookkeeping or later human edit, never restore/tag newer B or retry; known B/R/D changes are applied-unverified even if the finalizer-local refusal was correct. Success requires independent post-call D/R/B, identity/namespace, dirty/current/saved-version, context and eventually validation witnesses, not the writer or Callable return.

T001's completed core acceptance applies to its then-approved typed evidence contract. The implemented `SavedStateEvidence` currently has numeric `resource_mtime` and `document_mtime` fields, and the reducer requires both to agree with independent disk mtime and complete finalization mtime flags. That historical interface cannot be populated from unavailable private Resource mtime on stock P1. Within existing T004 caller/core integration, migrate the affected typed saved-state evidence, finalization/reducer checks and contracts before consuming P1 facts; never fabricate numeric metadata or retain the old mtime equality as a stock success gate. The exact replacement API/wire shape is not settled by this research, and T001's historical completion is not reopened.

This selected stock route carries conservative version/topology checks and standard-extension guard/readback maintenance instead of installed custom-editor trust, per-version patched binary distribution and replacement maintenance. It adds no new service, policy engine, approval layer or generalized transaction. [Research §14](research.md#14-stock-post-persistence-saved-transition-research-2026-09-28) records official source links and bounded GUI results; these are saved-state design evidence, **not** T003 readiness, mutation support or A–E proof.

Pinned upstream basis: [public incoming connections](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/core/object/object.cpp#L1411-L1421), [native saved handler and built-in-path scan](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp#L723-L803), [document tag/path mtime](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_base.cpp#L91-L93) and [public edited setter](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/editor_interface.cpp#L714-L721). The official stock 4.7.2/macOS arm64 research completed 18 GUI cases and 95 independently checked facts, including a separately inspected visible non-selected dirty tab; the bounded stock reopen logged a `get_line_wrap_count` diagnostic. These are mechanism/lifecycle research, not an error-free UI or full mutation acceptance.

## Historical Patched Primitive A Candidate (Superseded for Saved State)

### 1. Core intent, revision and compatibility

Add a focused script-edit module to the existing Rust library, using existing checked selectors, identity/collection/source evidence and classification patterns. The core consumes protocol-independent evidence; no JSON, TCP, Godot object or native ABI dependency enters its policy. Do not build a general Phase 2 transaction/revision framework.

One new `edit-gdscript` caller accepts the selectors and bounded stdin JSON in [edit-api.md](contracts/edit-api.md). Whole-source intent avoids patch-coordinate ambiguity; source does not enter argv or an arbitrary file-reading option. Expected basis extracts project/session/document/file identity, agreed source digest/length, exact current CodeEdit version and attributable clean state from prior observation. Observation v1 has no saved-version field; acquire it during fresh preparation. Pre-edit guards compare the prepared current version; after the complex edit freeze its new current version while keeping the preparation saved version for the stock transition, as in the [current selected design](#selected-stock-saved-state-composition-design-only). Fresh checks, not that snapshot, authorize the write.

Keep observation's public v1 meaning, timing and per-surface availability unchanged. Private bridge **v2** is a coordinated clean cutover with authenticated edit/native capability fields, changed HMAC domain and strict tuples. Migrate all Rust/addon/worker/fixture peers and docs; no dual-protocol shim or v1 mutation fallback. Restart/re-enable the addon and observe again after upgrade.

### 2. Historical native application and Primitive A (not executable)

Use one native CodeEdit complex operation on the exact bound buffer; preserve previous undo history and unrelated tabs, with normal native redo-branch invalidation permitted. Do not use `set_text`/clear-history as an edit shortcut or create an alternate EditorUndoRedoManager history for TextEdit's existing text history. Never choose the destination by focus.

Immediately recheck current B/version and original R, then use the explicit supported **`Script.set_source_code` method**, not property `_set`, `apply_code`, export propagation or reload. Native source synchronization is not a compiled-class/Inspector/live-game update claim. Pending dragged exports or an unsafe Save representation refuse before application; do not alter editor preferences or scene properties.

The C++ integration pins the selected project and existing file, independently confirms its identity against the Rust basis, and writes/truncates/flushes only that retained object after fresh D/R/B/identity guards. Short-write/error/namespace-loss facts remain explicit. Replacement/parent redirection never changes the write destination. Arbitrary same-inode exclusion remains unclaimed; known changed source/identity still stops obsolete work.

The old A proposal consumed a private persistence receipt after guards and prescribed setting private Resource/document mtimes and tagging saved last without callouts. **Superseded:** the selected public stock handler tags the CodeEdit saved version before reading document path mtime and can trigger native validation/deferred debugger reload; Resource private mtime is not numerically synchronized. See the [current stock composition](#selected-stock-saved-state-composition-design-only) rather than implementing the old A contract or its no-callback assumption.

Do not emit `resource_saved`, call Save All/current-tab Save, clear unrelated docs/history or claim normal scene-save effects. The current [Primitive A §3 guarded target-document saved transition](contracts/native-integration.md#3-primitive-a-guarded-target-document-saved-transition) specifies the selected stock Callable contract and identifies the rejected historical patched route; use its current contract, not that old proposal.

### 3. Primitive B and automatic editor effects

Expose existing `GDScriptLanguage::validate` → parser → analyzer on immutable exact source/path in the selected editor main-thread/project context. Bind actual input hash, target/session/request, invocation interval and root/dependency diagnostics. Return completed valid, completed invalid, or unavailable; no reload/log/silence proxy.

Permit normal parser/dependency-cache activity and confined source-backed GDScript analysis. A small call-scoped read/effect context gives the existing parser/cache/analyzer actual confined bytes/current hashes, including cache-hit attribution and live project/class/autoload mappings. It refuses before non-GDScript loader hooks, full loading/instantiation or dynamic project-object property access. Do not add a second parser, dependency-world snapshot/lock, cache-purity mandate or general compiler/evaluator service.

The historical engine's target-local guard was intended to cover ordinary validation/function-discovery/application work caused by the agent edit, including queued callbacks. Its patched T002 validation acceptance remains real, but does **not** establish a stock callback guard. The selected saved handler itself invokes native validation and may defer debugger reload; independently resolve confinement of all edit-generated effects without suppressing later human/unrelated behavior. Diagnostic source belongs only in the requested bounded result, not incidental native/Rust logs.

Preflight **changed** intent to avoid preventable invalid-source mutation; after the stock saved transition, read actual resulting source and invoke an independently justified native validator afresh. The post-change result, not preflight or saved handler callbacks, contributes to success. An invalid original script remains repairable if all original safety/representation checks hold and the resulting source validates. Unchanged intent runs one non-mutating validation/verification path. The [historical Primitive B contract](contracts/native-integration.md#4-primitive-b-exact-source-native-gdscript-validation) records T002's completed patched validation boundary; stock validation/effects still need separate design.

### 4. Stage ordering, deadlines and independent verification

The former [ordered state machine](data-model.md#3-ordered-attempt-state-machine) retains stage/certainty semantics, but its patched finalization mechanism is superseded. The selected saved-state sequence is:

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

The PR #24 candidate proposed a small pinned engine adaptation alongside addon native integration; its patched Primitive A instructions above remain only for comparison. [§14](research.md#14-stock-post-persistence-saved-transition-research-2026-09-28) now selects the public, exact-target post-persistence saved transition for official stock, not the patched editor or a Save shortcut. T003 still cannot begin: the stock saved handler's native validation/deferred effects need independently justified confinement and eventual validator integration. The [post-persistence analysis](#post-persistence-design-review) retains that blocker; no weakening of the specification or product mutation support follows.

Historical native lifecycle/export evidence remains relevant to the eventual standard extension, but no patched saved-state hook, private Resource mtime getter/setter or finalizer API is a current implementation instruction.

## Constitution Check

**Historical planning check:** The earlier bounded patched-candidate planning check passed then, not as implementation/acceptance. **Current disposition:** [§13](research.md#13-stock-target-save-finalization-research-2026-09-28) rejects the audited Save routes; [§14](research.md#14-stock-post-persistence-saved-transition-research-2026-09-28) establishes P1 for the **saved-state question only** through public stock incoming-edge Callable discovery, descriptor persistence and guarded exact-document tagging. No constitutional waiver, relaxed guarantee, implementation approval or A–E pass follows. The current matrix records design obligations, not their discharge; [the prior §13 analysis](#stock-research-artifact-review) is historical and the [post-persistence analysis](#post-persistence-design-review) records the corrected design review. Independent stock validation/effect confinement, including handler callbacks, is still unresolved.

| Governing obligation | Design and future evidence |
|---|---|
| I / IV — independent D/R/B and observed postconditions | Separate actual D and editor R/B/dirty/current/saved reads; attributed post-change B; immutable partial evidence. Receipt/Callable return cannot certify success. A–E and durability remain mandatory. |
| II — preserve human work and stale intent | Exact target/Script/CodeEdit/session/namespace, pre-edit prepared current version, then frozen post-complex-edit current and preparation-time saved versions for receipt and pre-Callable guards; fresh source checks before edited flag and again before native Callable; after-call independent checks; never restore/tag newer work. No arbitrary-writer serialization promise. |
| III — native editor semantics | One CodeEdit complex edit, explicit `Script.set_source_code`, retained-descriptor persistence then public Resource edited=false and actual existing native saved-handler Callable. Native saved version precedes document path-mtime tag; private Resource mtime is not prescribed/read. No engine finalizer patch, signal broadcast or ResourceSaver. |
| V / X — confinement and truthful outcomes | Existing authentication/private metadata; native project-bound descriptor/dependency reads; no evaluation path; separate stage/application/parse/dependency/permission/timeout failures. Controlled unknown after authorization/loss. |
| VI / XII — layered verification and explicit support | Pure transition/boundary tests plus real visible-editor A–E, history, durability, timing, privacy/export and complete observation regression on exact recorded builds. No stock/other-version mutation support extrapolation. |
| VII / IX — protocol-independent, small composable surface | One typed/core edit and one CLI, existing private bridge, no MCP; core owns policy, native owns engine-local primitives. No duplicate transaction model. |
| VIII — tooling/gameplay isolation | Editor-only native integration; real exports and runtime checked with enabled/disabled addon and native artifacts, not reliance on `addons/` or `@tool`. |
| XI — independent implementation and dependency review | Public engine source/ABI, own narrow integration, existing locked Rust dependencies. Preserve SDK notices, review exact new native build inputs/advisories/licenses before their implementation delivery. No unreviewed copied framework. |
| XIII — proportional complexity | [§14](research.md#14-stock-post-persistence-saved-transition-research-2026-09-28) establishes bounded standard-extension composition instead of a patched editor for saved-state work. Exact topology/version checks, descriptor/receipt guards and real GUI regression cost remain; they are less than custom editor installation/trust, per-version binary distribution and team replacement only while guarantees hold. No cheaper route excuses effects on unrelated work. |
| Architecture/compatibility/workflow | Existing Rust core and eventual private bridge/caller cutover retain responsibility. The selected saved-state ingredients are not a new final API/schema. T003 is held for independent stock validation/effect-confinement review; T004/T005 retain dependencies. |

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

The [task list](tasks.md) retains five PR-sized increments, completed granularity review and unchanged dependencies; T001–T002 are complete. The earlier read-only `/speckit.analyze` result applies to the earlier design after local-overlap clarification. The [§13 rerun](#stock-research-artifact-review) predates P1; the [current review](#post-persistence-design-review) preserves full requirement ownership and the independent stock validation/effect blocker. Neither result authorizes T003.

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

No constitutional violation/waiver is requested. P1 selects a bounded standard extension plus stock saved-handler composition **only for saved state**. Its costs are topology/version capability checks, request-local receipt/descriptor and current/saved-version guards, actual native Callable identity verification, independently inspected state and real GUI regressions. A custom editor instead imposes trusted binary installation, per-version patched builds/distribution, team replacement, CI and ongoing maintenance. The former patched A cost row below is retained only as historical comparison; neither cost profile resolves independent stock validation/effect confinement or licenses a redirected write/unrelated history change.

| Present mechanism/gap | Simplest alternative and insufficiency | Ongoing cost and present justification |
|---|---|---|
| Standard-ABI native library / bound persistence | Public Save paths can follow replacement paths; external writer adds a separate synchronization boundary. | C++ build, ABI/OS/lifetime tests; justified by demonstrated in-editor object/descriptor behavior. No godot-cpp/crate framework. |
| Exact stock saved handler after bound persistence (selected); patched engine finalizer (historical, superseded) | CodeEdit-only tag omits document mtime; broad Save uses fallback/all-script application; signal broadcast admits reentrant listeners. The old A patch set private mtimes and tagged last but is no longer selected. | Public incoming-edge discovery of actual native Callable, exact Script/topology/version/namespace and receipt rechecks, Resource edited flag and post-call independent evidence. Native callback validation/deferred reload require separate effect confinement; no saved-state engine patch distribution. |
| Patched T002 validator + scoped read/effect context (completed there); stock validation/effects unresolved | Reload is not attributed parse success; naked binding can traverse effectful/unconfined dependency paths; subprocess loses context. The T002 engine adaptation does not provide the selected stock callback guard. | Keep recorded patched parser/cache/analyzer acceptance as historical validation evidence; independently resolve stock exact-source parse and handler/deferred effects before T003. No second parser or generalized compiler. |
| Supervisor authorization/stage extension | Read-only worker termination cannot prove a sent mutation never applies. | One control handoff and certainty state in existing supervision; required by FR-012, not a recovery service. |
| Two kit callers based on X enter the same mutation path and overwrite/order changes | Reuse the existing collection/edit slot plus fresh revision checks; that mechanism is sufficient, so no new coordination layer is needed. | Make existing admission/cleanup ownership explicit and maintain T004's barrier regression. This closes a present stale-intent failure with no lock manager, lease service, queue, registry, replay system, merge algorithm or Phase 9 infrastructure. |
| Bridge v2 / edit stdin contract | Strict v1 shape/authentication cannot silently grow; source in argv or arbitrary file options adds exposure. | Coordinated codecs/fixtures/docs migration; one small operation, no compatibility shims or new protocol service. |
| Real stock capability/build proof and edit acceptance runner | The historical patched-editor build recipe and observation-only proof do not establish the selected stock behavior beyond the tested GUI probe. | Verify exact stock API/topology and standard extension build, use existing owned real-editor fixtures and bounded regressions; no custom distribution, provider-specific CI or new approval layer. |
| Historical T002 dynamic-getter validation sentinel | A scripted `Engine.get_meta(...)` expression is not constant-folded, and public autoload type resolution does not supply the analyzer with a live object constant; either misses the actual effect path. | The existing fixed-name `TESTS_ENABLED` fixture object supplied patched T002's real getter sentinel; its completed build/evidence remains valid, not stock mutation proof or a new product operation. |
| Automatic native discovery leaves a skipped tooling manifest in Godot's generated export extension registry | The existing export hook removes the manifest/library but not the engine-generated registry entry; the real hook-only export then fails at runtime. Rewriting the shared exporter or reproducing its preset filtering would add unnecessary scope. | Keep the installed native directory under `.gdignore` and load its existing manifest explicitly through `GDExtensionManager` in the editor plugin. This uses the existing lifecycle, avoids stale runtime registry entries and missing-binary startup errors, and requires only enabled/disabled/hook-only export plus missing-native regressions. Other extensions keep their normal discovery/export behavior. |

## Verification and Planning Completion

[quickstart.md](quickstart.md) maps all 22 requirements, 26 scenarios and eight success criteria to implementation verification, including all A–E, ≥20 successful sequential edits and ≥3 interleaved unsafe refusals. Existing Rust baseline checks and complete observation acceptance remain required where affected. Real GUI evidence may be maintainer-operated; existing hosted/protected workflow boundaries suffice. No CI provider/runner topology is introduced as a new prerequisite.

The original plan-generation pass performed only feature-path/workflow resolution, pinned-source/API review, specification/design coverage and local-link/anchor/Markdown/whitespace/full-diff checks. No Cargo/product suites, engine build, runtime probe, filter retry, A–E acceptance, task generation or analysis command was run by that pass. The installed `setup-plan.sh --json` was executed with the verified Feature 002 directory and retained the existing plan; the installed template was resolved before completing these artifacts. Before/after-plan hook checks found no `.specify/extensions.yml`, so no hook was registered or bypassed. That pass's read-only native/flow contract review findings were corrected without changing the then-current specification.

**Remaining material design question:** P1 [§14](research.md#14-stock-post-persistence-saved-transition-research-2026-09-28) selects an exact-document post-persistence saved transition on official stock with public incoming-edge Callable discovery. [§13](research.md#13-stock-target-save-finalization-research-2026-09-28) still rejects ordinary Save, custom-saver fallback and direct ResourceSaver as selected routes. The unresolved blocker is **independent stock exact-source validation and confinement of edit-generated native handler validation/deferred debugger effects**, not the availability of a saved-state mechanism. T003 remains unstarted/on hold; task scope/count/dependencies and A–E requirements are unchanged. The current analysis below does not discharge that independent blocker or approve implementation.

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
