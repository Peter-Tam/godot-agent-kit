# Research: Safely Open a Known Project GDScript

**Date:** 2026-09-29

**Spec:** [spec.md](spec.md)

**Disposition:** Phase 0 complete. Select guarded, source-bound native opening under the inherited local-editor threat model. The initial B1 callback-isolation hold was withdrawn after checking its actor against that model; the negative findings remain evidence, not erased failures. This is design readiness, not implemented capability, GUI acceptance or a new support claim.

## 1. Questions and outcomes

Two independent read-only investigations examined native lifecycle behavior and existing Rust/core/worker/bridge contracts. The native investigation initially lacked its tool surface, then resumed and completed. The integration owner inspected pinned source and ran bounded probes in six disposable stock-editor processes, including two composed checks with the existing observation/edit callers. No product code, permanent tests, engine patch, dependency or implementation task was introduced.

| Question | Decision |
|---|---|
| Existing stack and ownership | Reuse Rust/GDScript/public-ABI C++17, locked dependencies and existing owned-editor harness. |
| New caller and evidence semantics | Distinct `open-gdscript`/`open_gdscript`; reuse checked selectors, observations and worker supervision, not edit-persistence outcomes. |
| Private compatibility | Coordinated authenticated bridge v3; public observation/edit schemas remain v1. |
| Cold target acquisition | Confined capture → new native GDScript → non-takeover cache path claim → initial compilation → native editor open. No ordinary root ResourceLoader dispatch. |
| Cached closed target | Retain and open its exact existing Script only when source/identity/edited-state guards pass; never overwrite or reload it. |
| Already-open target | Fresh no-effect observation/recheck only; no loading, opening, focusing or source-profile admission. |
| Native dirty-tab effects | Check the actual departing document. Do not dispatch with conflicting R/B, unknown state or unsafe source context. Equal R/B with an independently dirty human buffer is permitted; dirty is not inferred from equality. |
| Script/editor callback safety | Preserve the inherited exclusion of malicious code already running inside Godot. Known unsafe integration/context refuses; no generic registry isolation, callback shutdown or OS sandbox is introduced. |
| Native proof and support | Positive state/source/history probes inform the design. Production boundary/race/durability/export and unlocked-GUI acceptance remain implementation gates. |

No `NEEDS CLARIFICATION` decision remains. Exact interfaces, supported profile and refusal semantics are in the [native contract](contracts/native-integration.md), [caller contract](contracts/open-api.md), [bridge contract](contracts/bridge-protocol.md) and [data model](data-model.md).

## 2. Existing stack and minimal reuse

**Decision:** Retain Rust 1.98.1/edition 2021, thin GDScript integration, C++17 public GDExtension ABI and Python 3.10+ fixtures. Reuse the existing `serde =1.0.229`, `serde_json =1.0.151`, `cap-std =4.0.3` and `ring =0.17.14` lockfile selections. No new dependency or public transport.

**Evidence:** [Cargo.toml](../../mcp-server/Cargo.toml), [rust-toolchain.toml](../../mcp-server/rust-toolchain.toml), and the [current native guide](../../godot-addon/native/README.md). Existing provenance/license choices remain applicable. No new third-party implementation was copied.

**Rationale:** The existing checked identities, authenticated project/session route, confined D reads, independent R/B collector, one active operation and owned-worker deadline are the actual reusable mechanisms. Opening needs its own lifecycle policy, not another safety framework.

**Alternatives considered:** A new crate, godot-cpp layer, process service, general transaction framework, standalone language implementation, hosted GUI gate or engine fork adds maintenance without being necessary to this selected operation. No new OS isolation property is made a requirement.

## 3. Caller, supervision and compatibility

**Decision:** Add one separate local caller with the existing project/session/script selectors and a fresh request ID. No caller-supplied source, expected edit basis, force/reload, focus toggle, retry or arbitrary operation option. A later edit still takes a fresh observation and its existing basis.

Reuse [observation.rs](../../mcp-server/src/observation.rs) identities/availability/collection stamps, [target.rs](../../mcp-server/src/target.rs) exact selection, [project_fs.rs](../../mcp-server/src/project_fs.rs) independent confined source reads, and existing worker supervision. A 9.5-second operation budget plus 0.5-second delivery reserve implements the ten-second controlled target; the budget starts before request parsing/resolution. Existing observation/edit timing is unchanged.

The parent records possible effects before one-shot worker authorization. The shared editor slot spans prepare through verification/cleanup. Busy refusals are final and unqueued; a disconnected/expired attempt cannot acquire a new authorization. Entered native work retains the slot until it returns, even if the caller has already received an immutable partial/unknown result. Killing the owned worker never kills the editor or proves rollback.

**Decision:** Private bridge **v3** clean cutover. Current [bridge.rs](../../mcp-server/src/bridge.rs) rejects unknown capability fields and signs a fixed six-boolean v2 capability vector with native revision/build identity. Authenticate a seventh `open_gdscript` bit in a v3 domain and migrate all peers/fixtures/docs together. Keep public observation/edit schemas v1; new opening has its own v1 result schema. No v2 fallback or dual-protocol shim.

**Alternatives considered:** Opening under observation violates read-only semantics. Reusing edit outcomes conflates loading/cache publication/tab creation with source writing/persistence. Extending strict authenticated v2 under the same version gives incompatible semantics; a replay service is unnecessary.

## 4. Candidate and executed evidence

The executable returned `4.7.2.stable.official.ed1daf0bf`, full commit `ed1daf0bf001b61586d9930840f2f1394092c079`, SHA-256 `c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf`. Host: macOS 26.6.2 arm64.

The existing [observation harness](../../godot-addon/tests/run_observation.py) provided owned projects, real macOS editor processes and actual Script/CodeEdit getters. Temporary fixture control code exercised native API sequences; it did not implement production admission, descriptors, bridge v3 or timeouts. The test sources were inert, known files, with cold-source directories excluded from automatic indexing to establish cache absence.

Four local evidence sets remain outside the repository:

- [Initial source/callback study](file:///Users/petertam/.godot-open-planning-pvyzy_al/evidence/summary.json): **23 fixture requests, 13 opening invocations**, three owned editor processes. Summary SHA-256 `135f8645f54bf93322a3dfc938846a692391c2e5f5eb9537d99bc83f37fd3df3`.
- [Current-source/captured-resource study](file:///Users/petertam/.godot-open-current-research-yiw81nzq/evidence/summary.json): **23 fixture requests, five opening invocations**, one owned editor process. Summary SHA-256 `ca73e312e3064b42790592ea9ee324dc1dd9d71ec27123fc65e66142635aba39`.
- [Property-assignment composition control](file:///Users/petertam/.godot-open-compose-research-ogspsgfp/evidence/summary.json): **two fixture requests, one opening invocation**, one owned editor process; existing observation/edit callers also invoked. Summary SHA-256 `5a3d2340d172f73c52d2d369a815e9216ac61e0022f0598021fcfbb4d152d0b9`.
- [Explicit-method composition control](file:///Users/petertam/.godot-open-method-research-t5instqq/evidence/summary.json): **two fixture requests, one opening invocation**, one owned editor process; existing observation/edit callers also invoked. Summary SHA-256 `79e094d14e3bccac59e28c3b49d0e1276b69b2413e8af3427079f0f7bf47b19a`.

Total: **50 recorded fixture requests and 20 opening invocations**, not 20 passing product cases. Two additional invocations of each existing public observation/edit caller are separate from those fixture counts. Repeated getter requests used to establish natural editor state are counted as requests, not independent scenarios. Each native action recorded before/immediate/eight-process-frame-after state. Eight frames are bounded diagnostic observations, not proof that all continuations drained. All six owned editors exited 0. Temporary projects, probe sources, registries, control files and compiled window helper were removed; only private evidence remains. These local paths are provenance, not contributor prerequisites; the durable sequences and source references below describe the findings.

### 4.1 Ordinary load/open and syntax errors

**Observed:** Cache lookup followed by ordinary `ResourceLoader.load(path, "GDScript", CACHE_MODE_REUSE)` and `EditorInterface.edit_script` opened a cold inert source with independently matching D/R/B and a clean buffer. Retaining the same Script across a fixture close/reopen preserved its identity and source. A safe syntax-error source also opened clean with actual invalid text on D/R/B.

The [stock GDScript loader](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript_resource_format.cpp#L39-L59) deliberately returns a non-null Script despite parse failure. Syntax invalidity is not absence of a GDScript document.

### 4.2 Dirty current document: negative and positive controls

**Observed negative:** With a dirty current unrelated buffer whose R differed from B, both `grab_focus=false` and `grab_focus=true` opened the target but changed unrelated R to pending B. B and its CodeEdit version were unchanged. This is an in-scope native source effect, not a hypothetical malicious actor.

**Source:** [`_go_to_tab`](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp#L360-L431) calls `apply_code()` on the departing unsaved document; [`apply_code`](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_text_editor.cpp#L226-L241) sets R from B and updates exports. Native sorting also navigates previous/current tabs in [`_update_script_names`](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp#L2021-L2048). Focus suppression is not an effect fence.

**Observed positive:** A native human edit inserted a valid comment into the current unrelated document. Fixture preparation waited for ordinary editor validation to produce R == B while the independently observed buffer remained dirty. Opening a cold target then preserved its Script/editor/buffer IDs, R, B, dirty state, current/saved versions and Undo/Redo availability. Actual native Undo restored its prior buffer and Redo restored the human edit. The new target was independently clean and D/R/B-equal.

**Decision:** Use `grab_focus=true` for a new native open; it makes the selected target the current tab and the admitted departing document the previous tab before the native sort. Freshly guard that document's exact identity, R/B, dirty/current/saved state, source-effect profile and absence of pending source-changing state. R/B disagreement or unavailable context refuses before dispatch. Equality permits an already-dirty buffer only with independent dirty evidence; it does not tag it clean or Save it. Product requests do not wait for, apply or repair human convergence. The waiting above was research fixture preparation, not proposed product behavior. Already-open recognition never calls this path.

### 4.3 Ordinary loader interception

**Observed:** A controlled front ResourceFormatLoader recorded two callbacks while an inert `.gd` target was loaded; stock fallback still produced a clean matching target. A type hint did not exclude custom loader dispatch.

The [ResourceLoader contract](https://docs.godotengine.org/en/4.7/classes/class_resourceloader.html#class-resourceloader-method-load) and [pinned dispatch](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/core/io/resource_loader.cpp#L276-L300) establish ordered loader callbacks. Ordinary loading also reopens a pathname rather than consuming the exact confined source capture.

**Decision:** Do not use ordinary ResourceLoader loading for the cold product branch. This avoids an unnecessary root loader/dependency authority and binds initialization to independently captured bytes. Cache lookup remains passive and is retained.

### 4.4 Fresh captured Resource and cached reuse

**Observed:** Creating a fresh native GDScript from the known source, setting its normal `resource_path` without takeover, initially compiling it, then using native `edit_script` avoided the tested root loader callbacks. The initial study also examined uncompiled construction; that is not selected because it needlessly changes normal compiled-resource behavior.

The second study exercised the selected acquisition sequence for valid, syntax-invalid and read-only files. Every target obtained an actual associated buffer, independently equal D/R/B and clean state. Compilation returned OK for valid/read-only sources and **43 / ERR_PARSE_ERROR** for the invalid source, whose real buffer still opened without repairing text. Closing and reopening the captured valid source reused the same cached Script identity. These are getter/primitive observations, not full durability acceptance.

**Decision:** For a cache miss, consume exact confined D, create native GDScript, call the bound `Script.set_source_code` method only while it is a new unbound object, claim the original path using the bound `Resource.set_path` method, then `Script.reload(false)` only on this new object to perform initial compilation. Do not substitute property assignment or `Object.set`. Refuse on cache collision; never steal or replace a pre-existing Resource. Independently read cache identity/source and the Resource edited flag afterward. For a cached target, retain the existing R and never assign its source, path or reload it.

The [public resource_path contract](https://docs.godotengine.org/en/4.7/classes/class_resource.html#class-resource-property-resource-path) and [pinned setter](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/core/io/resource.cpp#L85-L123) establish non-takeover collision refusal. The bound method is `Resource.set_path`; `set_path_cache` and `take_over_path` have different semantics and are prohibited here. The [initial compilation source](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript.cpp#L737-L900) explains why source/effect admission is necessary before this call. It is not a reload/repair of an existing R and does not grant permission to instantiate or execute the target.

**Alternatives considered:** File writes/reload, cache replacement, synthetic result values and takeover violate the contract. Returning an uncompiled Resource when normal initialization is required is unnecessary. Source-bound initialization plus independent R/B getters is not fabricated observation: initialization creates a real authority, and separate reads/rechecks determine what that authority actually contains.

### 4.5 Exact methods preserve composed edit eligibility

**Observed negative:** The property-assignment prototype produced `complete_observation` with clean, agreeing D/R/B, but the existing real edit caller refused `dirty_conflict`, `not_applied`. Buffer cleanliness was insufficient for the separate native Resource-edited check. This was a prototype integration failure, not a regression in the unchanged edit contract.

**Observed positive:** Replacing both `script.source_code = ...` and `script.resource_path = ...` with explicit `script.set_source_code(...)` and `script.set_path(...)` calls, without clearing any edited flag or tagging a buffer, left `EditorInterface.is_object_edited(script)` false after opening. A fresh existing observation returned `complete_observation`; the existing edit caller returned `verified_changed`, `complete`, `applied` (exit 0). Separate disk/Script/CodeEdit witnesses matched the intended replacement source.

**Decision and regression:** The native contract requires the exact bound methods and independent Resource-edited inspection in addition to buffer dirty state. No `set_object_edited(false)` repair is added to opening. Preserve this negative/positive composition case during implementation. It establishes useful method-sequence compatibility, not the new product caller's full guards, GUI acceptance, history or durability.

## 5. Selected native profile and guard scope

The [native contract](contracts/native-integration.md) is normative. Initial new-open support is bounded LF UTF-8, no BOM/CR/NUL or other non-tab/newline control characters, ≤512 KiB, standalone `.gd`, internal GDScript editor, exact stock build, no source dependencies or effectful declarations. A conservative lexical admission, not a second parser, rejects annotations, `class_name`, `static`, `const`, `load`/`preload`, dynamic property hooks (`_get`, `_set`, `_get_property_list`), script/global-class inheritance and references to effective project global-class/autoload names or extension-defined classes. Only built-in native bases are admitted. Comments/quoted strings must be handled correctly; ambiguous unsupported lexical forms refuse rather than run them. Successful target parsing is not required: the supported `var =` syntax-error case remains positive. This deliberately narrower opening profile does not change existing observation/edit eligibility.

Apply the same relevant effect profile to native source inputs touched by the opening transition, including the departing current script. Bound and freshly recheck effective global-class/autoload/extension-name context, external-editor configuration, exact open-document association, retained cache identity and relevant source/version/dirty evidence. Unknown or changed state refuses before entry. No UI selection, preference change, source synchronization, plugin disablement or callback unregistration manufactures eligibility.

**Current-document validation decision:** When an existing current GDScript will be visited by the native transition, reuse the existing one-shot stock validator on its exact privately captured R == B source and effective context before authorization; require valid, attributable completion. This is not a parse-success requirement for the requested target. The current document's source stays private to admission and is never returned as requested target source. Its identity, versions, source and context are rechecked at native entry. A confirmed empty Script Editor needs no current-source validator. Invalid/unavailable current validation refuses without waiting for human convergence.

**Rationale and cost:** Native `apply_code`/validation also refreshes exports and can process pending dragged-property state, as shown by the [pinned validation/assignment paths](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_text_editor.cpp#L841-L896). Source equality alone does not establish safe source semantics. The source profile, valid current-source fence and passive compiled-property/base/method checks in the native contract avoid claiming an unobservable `pending_exports_empty` flag. This reuses one existing bounded, no-log validator invocation, rather than adding a parser or service; it adds no new public validation tool. Its inherited local-LSP endpoint limitation is unchanged. Existing pending-drag/Undo state is a required native regression, not a promise inferred from a quiet editor.

Use current shared project/session guards and a **read-only** exact-file descriptor for opening; do not reuse edit's write-access requirement. Source initialization uses captured bytes, never a later path read or caller-provided replacement. Keep object references and recheck cache ownership, original file namespace/content and affected-document state before each native stage. Native publication/open acknowledgments are facts, not success. A detected source/identity/context change after effects yields partial/unverified state and preserves newer human work.

**[INFERENCE — design expectation, not acceptance]:** Combining these guards with the exercised native sequence is expected to preserve the supported ordinary editor workflow. Production tests must prove the actual integration, race barriers, read-only behavior, native-history preservation and durability. No prototype result is substituted for those tests.

## 6. Scope correction and callback limitation

The initial investigation promoted B1 after registered EditorSyntaxHighlighter callbacks ran for ordinary cold opening, fresh-Resource opening and cached reopening (one `_create` each); passive already-open observation ran none. A deliberately source-writing callback then appended a marker to another dirty B with a clean departing current tab. Target R/B still agreed. These negative controls remain in the initial evidence, including [22-research_open.json](file:///Users/petertam/.godot-open-planning-pvyzy_al/evidence/22-research_open.json).

The [native new-document loop](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp#L2319-L2332) calls registered factories, and [the factory wrapper](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/syntax_highlighters.cpp#L50-L66) can dispatch a script override. This is an existing [editor extension mechanism](https://docs.godotengine.org/en/4.7/classes/class_editorsyntaxhighlighter.html), not a new agent evaluation endpoint.

**Corrected decision:** B1 is not a justified blanket implementation-readiness blocker. The inherited [security and compatibility boundary](../001-observe-gdscript-state/plan.md#security-and-compatibility-decisions) explicitly excludes “hostile same-UID processes or malicious project code already running in Godot.” The intentionally buffer-writing callback was already installed and executing in the editor with that authority. It demonstrates a limitation of native composition against such code, not an in-scope new execution capability that justifies a universal callback registry, clean-room editor requirement or sandbox. The original blocker assessment did not satisfy the required in-scope-actor element under Principle XIII.

This correction does **not** excuse ordinary native dirty-buffer application, project source newly executed because of loading, known unsafe extension behavior or detected interference. The former has an explicit admission rule and positive control (§4.2); source-bound acquisition and the source/context profile address newly introduced root/dependency effects. Known unsafe context must refuse; known unexpected changes prevent success and retain truthful effect knowledge. Existing editor tooling is not automatically granted permission to change unrelated source, and neither approval nor an extension allowlist replaces the documented guards. The kit does not promise protection against an already-running malicious plugin capable of changing buffers or falsifying editor state independently of a request.

Native editor display/diagnostic behavior and already-authorized tooling callbacks remain part of the existing Godot environment. The kit does not capture/forward raw editor logs, expose arbitrary callback invocation, or add an execution permission class. Project source stays out of kit incidental logs, bridge metadata and unselected results. A demonstrated additional in-scope failure during implementation must be fixed or held under the four-part blocker rule, not hidden behind this limitation.

## 7. Evidence limits and verification decisions

The visible-window probe failed. Separate CoreGraphics diagnostics established screen-capture permission but `CGSSessionScreenIsLocked=true`; the owned Godot window was not an on-screen witness. No screenshot was obtained and no retry attempted to bypass the lock. This is an environment limitation, not proof of GUI acceptance and not a requirement for a new CI provider or runner.

All runtime findings above came from actual stock editor processes and independent Script/CodeEdit/D getters. They do not prove full race safety, entire prior history, production outcome timing, export isolation or all Save/reparse/rescan/runtime durability. The selected source-bound initialization still requires complete ordinary Save, cache-consumer, reopen, reparse and fresh-runtime compatibility evidence during implementation. The [quickstart](quickstart.md) makes those gates explicit and separates future commands from already executable baseline checks.

No Rust/product suite was rerun in this planning invocation because production code did not change. The two focused composed checks did invoke the existing public observation/edit binaries; only the explicit-method variant succeeded at editing. Existing Feature 001/002 support and acceptance remain unchanged. Six clean process exits and removal of all owned research scaffolding were observed; retained private evidence is not shipped.

## 8. Principle XIII decisions

| Present requirement/failure | Simplest credible mechanism | Why existing behavior alone is insufficient | Cost and current justification |
|---|---|---|---|
| Open a known closed script | One explicit opening domain/runner and native lifecycle, reusing routing/observations/slot | Read-only observation cannot create B; edit intentionally refuses closed targets | Small current consumer, not a framework or speculative setup. |
| Native departing R can change | Fresh R/B/dirty/version/context admission and recheck of the affected document | Target-only equality and focus=false miss the measured effect | Bounded current-document evidence; justified by actual human-source preservation. |
| Ordinary root load dispatches loaders/reopens path | Confined capture and new native Resource with non-takeover path claim; retain cached R separately | A type hint does not bind root bytes or avoid loader callbacks | Small public-ABI lifecycle composition; no loader proxy or engine patch. |
| New authenticated capability | Private v3 and native API revision 2 clean cutover | v2 fields/transcript are exact, not extensible silently | One coordinated caller/addon/fixture/doc migration; no compatibility shim. |
| Native artifact now serves edit and open | Rename installed bundle/entry symbol to `editor_integration`; preserve domain-specific edit/open modules | `script_edit` bundle naming no longer describes its shared role | Mechanical current-consumer migration, not a new layer. |
| Adversarial already-running editor code | Document inherited limitation; preserve in-scope guards | A generic callback-isolation proof exceeds the inherited threat model | No registry monitor, sandbox, approval ceremony or fresh-editor prerequisite. |

All technical choices are resolved for the bounded design. The [plan](plan.md) and Phase 1 artifacts retain real-editor acceptance, source confinement, human-work protection, independent verification and exact support obligations without claiming they have already passed.
