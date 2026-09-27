# Phase 0 Research: Safe Open-GDScript Editing

**Date**: 2026-09-27 | **Specification**: [spec.md](spec.md) | **Gate record**: [plan.md](plan.md)

**Result: Outcome 3D — no sufficient safe native route established; planning remains BLOCKED.** The native continuation in §9 demonstrated standard GDExtension access to live editor objects and descriptor-bound writes that do not follow replacement pathnames. It also exposed a same-object competing-revision overwrite and incomplete editor saved-state integration. The native source validator is not available through the inspected extension ABI; exposing it requires an engine-side API change with explicit dependency/effect semantics. No minimum sufficient integration level or safe scope reduction was established. This is not a claim that native integration is impossible. The specification remains unchanged; no implementation, tasks, or mutation support is authorized.

**Continuation boundary:** Resolve **A — target-bound single-script persistence**, including revision/newer-work protection and native editor saved state, and **B — source-attributed parse evidence** for the selected target/revision/interval. The maintainer authorized C1–C5 research into standard GDExtension first, then narrow engine API exposure, then a module/editor build only if necessary. Both original guarantees must be established before planning resumes. No engine patch, product implementation, or task generation is authorized by this research.

**Evidence history:** Sections 1–7 preserve the initial blocked investigation, published in existing PR #24 as `5748899`. Section 8 records the focused A/B continuation published as `c0e3f66`. Section 9 adds native-integration research on the same branch and PR; earlier outcomes remain historical evidence, not current completion or mutation-acceptance claims.

## 1. Existing foundation and inherited decisions

**Decision:** Retain the existing component ownership and observation behavior. No new package, dependency, transport, service, approval gate, or CI topology is selected.

**Rationale:** The current requirement is one guarded open-document mutation, not generalized transaction infrastructure. Existing authenticated routing and independent observation are reusable; an observation is not a mutation authorization or a safe write primitive.

Inspected current boundaries:

- `mcp-server/Cargo.toml`: one Rust package/library and the `observe-gdscript` caller; Rust 1.98.1/edition 2021. Existing direct dependencies are `serde =1.0.229`, `serde_json =1.0.151`, `cap-std =4.0.3`, and `ring =0.17.14`; no lockfile or feature changes were made.
- `mcp-server/src/project_fs.rs`: capability-rooted source reads, directory/file identity checks, symlink refusal, metadata/ACL checks, and `O_NOFOLLOW`/`O_NONBLOCK` reads. Its `validate_locator` contract checks a locator without opening source. The inspected source-read path, including opened/named identity rechecks, is not a confined mutation/persistence API.
- `mcp-server/src/runner.rs`, `target.rs`, `bridge.rs`, and `bridge/wire.rs`: authenticated selected-session lifetime, bounded framed observation, read-only worker supervision, and partial evidence. Killing an observation worker does not establish that a previously sent future editor mutation could not apply.
- `godot-addon/addons/godot_agent_kit/observation.gd`: attributable Script/ScriptEditorBase/CodeEdit identity, separately read R/B, document-specific `get_unsaved_files()` evidence, and request-local rechecks. None is a disk write capability.
- The [existing caller contract](../001-observe-gdscript-state/contracts/observation-api.md), [bridge contract](../001-observe-gdscript-state/contracts/bridge-protocol.md), and [data model](../001-observe-gdscript-state/data-model.md) remain unchanged. Version-1 strict decoding/capability fields cannot silently acquire new required mutation fields or new meanings for complete dirty/divergent observations.

**Alternatives considered:** Replacing the observation core, introducing MCP, widening platform support, or creating a generic transaction service would not solve the demonstrated native persistence problem and would add unrelated scope.

## 2. Research method and exact environment

Two independent read-only research assignments examined native Godot APIs/history/persistence and the Rust caller/bridge/confinement boundary. Their source-based recommendations were not treated as executed verification. Parent investigation then exercised the actual installed editor in a newly created, owner-private disposable project, never a user project or the product addon.

Observed tools/platform:

- `godot --version`: `4.7.2.stable.official.ed1daf0bf`.
- `Engine.get_version_info()`: full hash `ed1daf0bf001b61586d9930840f2f1394092c079`.
- `sw_vers`: macOS **26.6.2**, build **25G83**; `uname -m`: **arm64**.
- `rustc +1.98.1 --version`: `rustc 1.98.1 (48a229cea 2026-09-01)`.
- Actual GUI editor startup: macOS display driver, GL Compatibility/OpenGL 4.1 renderer, Apple M2.

A disposable ClassDB inventory was run with `--headless --editor --script` to inspect actual public method signatures. This was API inventory only, not visible-buffer evidence. Its immediate editor shutdown emitted scan-abort/resource-cleanup diagnostics; it is not recorded as a clean editor lifecycle acceptance run.

A separate actual GUI editor used a disposable EditorPlugin with fixed, fixture-only actions and synthetic scripts. The controller read D separately from the filesystem; the plugin read the real loaded Script, real CodeEdit, versions, dirty paths and native history. There were **27 recorded fixture actions**, not 27 product acceptance cases. A controller file-reader selector mistake was corrected before the first mutation; it supplied no successful behavioral evidence. No product code, executable, test suite, or public mutation endpoint was added.

The owned editor exited normally with code 0. A scoped native-window screenshot was inspected; it showed the owned editor but an earlier rendered tab state, so it is **not** proof of the final edited visual state. The positive statements below are limited to actual live editor-object/history and separate disk observations; full visible-surface A–E acceptance remains required.

## 3. Native text replacement and history

**Decision:** `CodeEdit`/`TextEdit` native history is the supported research direction for script text, rather than a second `EditorUndoRedoManager` action history. This is not approval of an end-to-end mutation route.

**Rationale:** The installed editor exposes public `begin_complex_operation`, `remove_text`, `insert_text`, `end_complex_operation`, `undo`, `redo`, `get_version`, `get_saved_version`, and `tag_saved_version`. The disposable probe performed a whole-source replacement as one complex text operation. No assignment to `CodeEdit.text`, history clearing, or alternate undo stack was used.

Observed sequence, using synthetic source revisions `S0` (original), `S1` (saved prior native edit), and `S2` (new intended edit):

| Step | Independently observed state | Native text version / saved version |
|---|---|---|
| Initial open | D = R = B = S0; target clean | 2 / 2 |
| First grouped edit | B = S1; D and initial R still S0; target dirty | 4 / 2 |
| Target resource save after native R synchronization | D = R = B = S1; target still dirty | 4 / 2 |
| Explicit saved-version tagging | Same source on D/R/B; target clean | 4 / 4 |
| Second grouped edit | B = S2; target dirty; unrelated buffer retains its unsaved sentinel | 6 / 4 |
| Target resource save then saved-version tagging | D = R = B = S2; only unrelated document remains dirty | 6 / 6 |
| Native Undo | B = S1; D initially remains S2; target dirty | 4 / 6 |
| Save S1 then tag | D = R = B = S1; unrelated unsaved work unchanged | 4 / 4 |
| Native Redo | B = S2; target dirty; Redo survives intervening save | 6 / 4 |
| Save S2 then tag | D = R = B = S2; unrelated unsaved work unchanged | 6 / 6 |

An additional Undo → Undo reached S0, and Redo → Redo reached S2, demonstrating that the earlier native undo entry remained reachable. Closing/reopening the saved target preserved S2 on D/R/B; the reopened buffer had new buffer/history state, as expected for a normal close/reopen. These are bounded fixture observations, not a claim about every source representation or native UI shortcut route.

The target was non-selected during the primary edit/history sequence. Its existing object association was used, not editor focus. The unrelated document's original disk bytes and its distinct unsaved buffer were both checked before the negative Save All experiment.

**Alternatives considered:** Direct `set_text`, clearing history, resource-only editing, or registering a separate editor action do not by themselves establish ordinary text Undo/Redo and prior-history preservation. The positive native text path still requires a safe persistence integration.

Sources: [TextEdit](https://docs.godotengine.org/en/4.7/classes/class_textedit.html), [CodeEdit](https://docs.godotengine.org/en/4.7/classes/class_codeedit.html), [EditorUndoRedoManager](https://docs.godotengine.org/en/4.7/classes/class_editorundoredomanager.html), and the existing native-history fixture actions in `godot-addon/tests/fixtures/observation/fixture_driver.gd`.

## 4. Persistence: positive mechanics, failed safety boundary

### 4.1 Public native Save All is unsuitable

**Decision:** Reject `ScriptEditor.save_all_scripts()` for this capability.

**Rationale:** The actual runtime method inventory and public documentation expose Save All, not a target-script save method. In the disposable editor, invoking Save All persisted the unrelated document's `HUMAN_UNSAVED_OTHER` sentinel and removed its dirty indication. This is a real side effect, not just a concern based on the method name. It violates the selected feature's requirement not to save unrelated human work.

**Alternatives considered:** Save-all-scenes is neither a script-targeted persistence API nor a safe way to preserve other dirty artifacts. Temporarily concealing another document's dirty state or manipulating its saved version to evade Save All would itself interfere with human state and is rejected.

Source: [ScriptEditor.save_all_scripts](https://docs.godotengine.org/en/4.7/classes/class_scripteditor.html#class-scripteditor-method-save-all-scripts).

### 4.2 ResourceSaver requires separate editor bookkeeping

**Decision:** Treat `ResourceSaver.save(target_script, target_path)` plus carefully guarded saved-version bookkeeping as a **candidate with limited positive mechanics**, not a selected safe transaction implementation.

**Rationale:** After the native editor had independently propagated B to the same loaded R, ResourceSaver returned `OK` and the external disk witness matched R/B. It did **not** clear the target's dirty indication or update CodeEdit's saved version. Explicit `tag_saved_version()` did so in the probe. The target-only candidate preserved the unrelated dirty buffer and native text history through the sequence above.

The candidate must not tag whatever version happens to be current after an asynchronous save: that could falsely mark a later human edit saved. Any future route needs an exact identity/version/source guard for that bookkeeping and independent postconditions. The probe's separate actions are not such a production transaction.

**Alternatives considered:** Treating ResourceSaver's `OK`, a saved-version setter, or a disk read alone as success would conflate persistence acknowledgment with independently observed editor coherence. Waiting for R/B equality is not a general parse witness or a filesystem confinement mechanism.

Sources: [ResourceSaver.save](https://docs.godotengine.org/en/4.7/classes/class_resourcesaver.html#class-resourcesaver-method-save), [TextEdit.tag_saved_version](https://docs.godotengine.org/en/4.7/classes/class_textedit.html#class-textedit-method-tag-saved-version).

### 4.3 Path redirection counterexample

**Decision:** Reject a previously validated pathname followed by ResourceSaver as a sufficient confined persistence boundary. **This is the blocking gate finding.**

The controlled counterexample used only owned synthetic data:

1. Observe the clean target, its original project-local disk source, R and B.
2. Preserve the original `scripts/` directory under another name in the same disposable project.
3. Replace the `scripts/` directory entry with a symlink to a separately created owner-private directory outside that project. Put an `OUTSIDE_MUST_NOT_CHANGE` sentinel in its `subject.gd`.
4. Call the path-based ResourceSaver candidate with the original `res://scripts/subject.gd` locator and still-loaded target Script.
5. Independently read the outside sentinel. ResourceSaver returned `OK`; the outside file now contained the loaded target's source rather than its original sentinel.
6. Restore the owned project's original directory entry.

The test deliberately splits observation and save so the replacement is deterministic. It is **not** a completed implementation of the required final guard, nor proof that every possible alternative is impossible. It proves that the investigated save primitive re-resolves its path and does not inherit the Rust reader's directory/file capability. A fresh path check rejects an already-present symlink, but a check followed by a separate path-resolving save still has a replacement window. The signature accepts a path, not an already validated directory/file handle or an expected file revision. Post-write detection cannot undo an out-of-scope write or count as prevention.

The same research boundary applies to stale external disk writes: a revision digest detects comparison differences; it does not itself make a later write conditional. No claim that arbitrary concurrent filesystem writers are safely handled was established. Existing no-follow reads are useful evidence collection, not a write guarantee.

**Alternatives considered:**

- Additional before/after path reads: useful for diagnosis and refusal where they detect a change, but insufficient by themselves to bind the actual write.
- Direct file write followed by a hoped-for reload: does not establish native history or D/R/B coherence and is not an acceptable cutover.
- Native buffer editing combined with a capability-scoped writer: a credible further research direction, but its target/revision binding, interaction with native Save/dirty state, and preservation under competing writes need a concrete design and proof. Merely naming a new `save` method or assuming advisory locks coordinate all writers would leave the central safety requirement unresolved.
- A new engine/native integration: not selected. It needs a concrete public interface, independently developed feasibility evidence, and Principle XIII cost justification rather than being introduced as speculative infrastructure.

The current research therefore has **no verified, complete persistence design** satisfying both target confinement and stale/human-work protection. This is a design blocker, not a missing confirmation from the user and not a waiver for implementation.

## 5. Parse and mutation-boundary evidence

**Decision:** Do not infer a valid parse from matching text, a save acknowledgment, or old compiled state. Do not publish a complete parse/diagnostic contract while its source/revision attribution and side effects remain unresolved.

**Rationale:** The installed `Script.reload(keep_state)` returns an error code and reloads class implementation; setting `source_code` alone does not reload it. The disposable valid script returned `OK` from `reload(true)` after its R/B sources matched. This proves only that particular reload result for that source. It does not establish independent, source-attributed invalid-script diagnostics, dependency-relative parsing, no unwanted execution/state effects, or arbitrary supported-script behavior. No syntax-error or tool-script acceptance matrix was run.

The native text-edit candidate can recheck document identity, exact source, CodeEdit version and attributable dirty state on the main thread immediately before applying a complex operation without an `await`. That avoids voluntary editor-frame interleaving; it does not lock the filesystem or prove absence of reentrant callbacks. The final combined persistence and verification boundary remains unresolved.

**Alternatives considered:** `can_instantiate()` may describe an older compiled class or fail for reasons other than parse validity. A new temporary GDScript parser object must first demonstrate correct path/dependency context and no unintended execution; a second GDScript parser implementation is not justified. Private editor methods or widget-tree scraping are not selected public APIs.

Sources: [Script.source_code and reload](https://docs.godotengine.org/en/4.7/classes/class_script.html), [ScriptEditorBase](https://docs.godotengine.org/en/4.7/classes/class_scripteditorbase.html), and [pinned script editor implementation](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_text_editor.cpp).

## 6. Caller, revision and interruption constraints retained for a resumed design

**Decision:** Preserve the observation v1 contract and its five-second deadline; a future edit result must be distinct and retain the specification's ten-second terminal bound. Do not finalize a mutation wire format before resolving its actual application/persistence boundary.

**Rationale:** Existing evidence/request IDs and CodeEdit versions are not complete, cross-authority mutation revisions. A proposed edit basis must bind the intended project/session/document lifetime and exact source state, then be freshly checked at the mutation boundary. Existing `ring` can provide a digest without a dependency, but a hash is not a lock, authority grant, or conditional write.

The eventual edit outcome must distinguish not-applied with proof of permanent discard, applied/partly applied, applied-but-unverified, and application-unknown. Once an edit can have reached Godot, socket loss or worker termination cannot prove it did not apply or cannot apply later. No automatic retry or implicit rollback is allowed. A durable replay service is not selected merely to name these states.

A single bounded full resulting source is the simplest candidate edit representation; alternative patch formats have no demonstrated requirement here. Source/identifier/frame bounds must be chosen against the actual protocol design, while preserving the existing 512 KiB-per-source observation behavior and strict v1 compatibility. No edit CLI, new schema version, public type, or capability bit has been implemented or finalized by this research.

**Alternatives considered:** Reinterpreting `complete_observation` as permission, using an observation request ID as a revision, killing the current read-only worker and claiming rollback, or adding optional fields to strict old decoders without a deliberate compatibility path are unsafe/incomplete.

## 7. Gate disposition and required next evidence

The initial constitutional gate passed **for research**, not for an implementation design. The attempted post-research evaluation is **FAIL** for the investigated persistence routes under Principles II–V and XII and feature requirements FR-002–FR-005, FR-008, FR-013 and FR-020. FR-009's general parse-evidence path also remains unresolved. The specification itself is not amended or declared invalid by these findings.

Before Phase 1 can proceed, research must establish a concrete target-bound persistence design that prevents out-of-scope writes and protects the expected revision/newer work, integrate it with native dirty/history semantics, and establish source-attributed parse evidence. Demonstrate the relevant replacement/stale/dirty cases and a positive native edit/Undo/Redo/persistence path without using post-hoc diagnosis as prevention. A new mechanism must answer Principle XIII's requirement/alternative/insufficiency/cost/justification questions in review.

Not established by this run: the product mutation path; all 26 acceptance scenarios; full A–E; twenty-edit stress; real UI-shortcut history routing; Save/reparse/rescan/runtime durability as a complete matrix; mutation deadlines/cancellation; mutation privacy/export isolation; or mutation version/platform support. Existing observation acceptance remains unchanged and is not rerun or repurposed as proof of mutation.

**Phase 1 prerequisite is not met.** No data model, mutation contracts, quickstart acceptance command, task list, or product implementation is fabricated to paper over the missing design. The owned research editor was stopped normally; disposable scripts, project data, and outside-project sentinels are removed after recording this evidence.

## 8. Focused A/B continuation and decision

### 8.1 Scope, method, and new evidence

The maintainer authorized continuation on `spec/002-edit-open-gdscript` in existing PR #24. The blocked evidence was published first; no new PR was created. Two independent read-only source/API investigations were collected before integrating their conclusions. Parent investigation then combined actual runtime bindings, pinned engine source, and small new disposable real-editor experiments.

The new editor again identified itself as Godot `4.7.2.stable.official.ed1daf0bf`, full hash `ed1daf0bf001b61586d9930840f2f1394092c079`, using the macOS display driver and Apple M2 Compatibility renderer. Its **14 recorded fixture actions** included **eight top-level reload attempts and one nested reload**. These are research actions, not product tests or an A–E acceptance count. Separate controller disk reads and native Script/CodeEdit observations were retained. An owned-window screenshot visibly showed the selected `subject.gd` with its original `return 17`; this corroborated the actual GUI fixture, not parse success or a completed mutation.

The accepted initial Save All and explicit-path/parent-symlink failures were **not rerun**. The new persistence experiment tested the different claim that an already-loaded Resource with an established path/UID becomes target-bound when the path argument is omitted.

### 8.2 Question A: no qualifying public persistence route found

**Decision:** No examined public Godot 4.7.2 editor/addon API provides the required target-bound single-script persistence and expected-revision guarantee. This is a conclusion about the inspected public bindings and native routes, not a claim that every future engine change or alternative implementation is impossible.

| Existing surface | Actual behavior / boundary | Disposition |
|---|---|---|
| `ScriptEditor.save_all_scripts()` | Public; saves open scripts broadly. Its unrelated-human-work failure is accepted prior evidence. | Rejected; not repeated. |
| Native `ScriptEditor::save_current_script()` | An internal C++ method, absent from public ClassDB/GDScript bindings. Uses the current editor, a prior on-disk timestamp check, and then pathname-based saving. | Not a public target-taking operation; current selection and a timestamp check do not bind the eventual write. |
| `EditorInterface` / `EditorPlugin` | Scene-save APIs, lifecycle hooks, and `resource_saved(Resource)` notification exist; no validated-target/expected-revision script-save operation was found. | Scene saving and after-save notifications do not provide the missing script write guarantee. |
| `ResourceSaver.save(script)` | Public; empty/default path is replaced with the Resource's stored `resource_path`, then a format saver receives that pathname. | Experimentally rejected as target-bound persistence. |
| Resource path, instance identity, UID, save notification | Identify engine Resource associations or report a completed action; they do not supply the actual writer with a validated filesystem object and expected revision. | Useful attribution facts, not write authority. |
| CodeEdit / ScriptEditorBase | Native text/history and editor association are available; no target-bound persistence primitive was found in their actual bindings. | Retain the observed text/history mechanics without inventing a save capability. |

**New omitted-path experiment:** Keep the exact already-open Script Resource and its stored path. Independently observe its original disk file, then replace only that file with a different owned sentinel document at the same path, retaining the original modification time. The probe did not alter UID sidecars. Invoke `ResourceSaver.save(script)` without passing a path.

Observed: the loaded Resource ID and source remained unchanged; the call reported `OK`; the replacement document was overwritten with the old loaded source. The unrelated dirty buffer retained its unsaved sentinel and its disk bytes remained unchanged. Independent filesystem witnesses distinguished original, replacement, and written files: inode values were `118864818`, `118865653`, and `118865654` on the same fixture device. Before saving, the probe's `ResourceLoader.get_resource_uid(path)` query returned `8446319580652804428`; this was a separate query, not a ResourceSaver return value or a before/after UID-equality assertion. These are experiment witnesses, not proposed product identifiers or an inode framework.

Ordinary safe-save inode replacement is not itself the failure. The failure is that the established Resource and stored path did not condition the write on the previously validated file/revision: the substituted document was overwritten while the save returned success. Preserving mtime also demonstrates why a timestamp is not object/revision binding. The original owned fixture leaf was restored after the experiment.

**Source trace:** `ResourceSaver::save` substitutes `p_resource->get_path()` when its path argument is empty and passes the resulting path to the selected format saver. Success callbacks and timestamp bookkeeping happen after that save. Native current-document saving selects the current editor and ultimately uses `EditorNode::save_resource`/ResourceSaver for scripts or `FileAccess::open(path, WRITE)` for text files. Its preliminary modification-time check is not a conditional writer. Resource equality in the editor's saved callback associates editor bookkeeping with an in-memory Resource; it does not prove which disk object received the write.

Repeated path checks, normalization, focus selection, unchanged Resource identity, or post-write mismatch detection do not cure this gap. In particular, downgrading an outcome after writing outside scope does not prevent the already-performed unsafe write.

### 8.3 Question B: public result surfaces and binding limits

**Decision:** A particular `Script.reload(true)` attempt can be attributed to a Resource/path, exact R source hash, and call interval. Its `Error` return is not, by itself, a general fresh parse-success result. No public, directly callable general source-validator/result surface was established for the existing GDScript addon.

Actual runtime inspection established:

- `Engine.get_script_language_count()` and `Engine.get_script_language()` **are callable**. The returned built-in language object exposes `ScriptLanguage`/Object bindings, but neither `validate` nor `_validate`. Enumeration availability must not be confused with validator availability.
- `ScriptLanguageExtension._validate(...)` is a virtual implementation hook for an extension language, not a proxy to the existing GDScript validator. Instantiating or registering another language would not expose that missing route.
- The actual open script editor, `ScriptEditorBase`, and CodeEdit expose no public source/revision-correlated diagnostic result or parse-validity getter. `edited_script_changed` and similar notifications carry no completed parse-result/source-revision payload.
- `Script.reload(keep_state)` returns an `Error`, not a structured per-call error list or source revision. The native C++ `is_valid()` discussed in engine source is not a callable public Script/GDScript method in this runtime; the probe used the public, weaker `can_instantiate()` predicate explicitly as such.

Pinned `GDScriptLanguage::validate(source, path, ...)` does parse and analysis and can populate structured target/dependency errors with their paths and positions. That engine-internal C++ interface shows a genuine source-specific result, but it is not bound for callers through the inspected GDScript/ClassDB surface. Its parser/analyzer scope would be relevant to FR-009 if a supported callable route existed; this research does **not** invent a requirement for full compilation merely to reject a parse-only result.

The editor's internal validation consumes CodeEdit text, stores its diagnostics internally, and can update loaded source/exports. An argumentless validation/change signal is not a fresh success result, and triggering validation is not necessarily passive observation.

`Logger._log_error` is public and can deliver function, file, line, message and backtraces. It is a process-wide, potentially multithreaded error stream without a source-revision/completed-validation result. It may supplement negative diagnostics with careful attribution, but cannot prove positive validation by silence or repair the reload short-circuit below. No logger collection infrastructure was introduced.

### 8.4 New exact-source reload experiments

Every top-level call recorded the selected Script ID/path, exact source and SHA-256 before/after reload, native start/end microseconds, `Error`, CodeEdit source/version, and separate disk source. The Script identity/path remained bound to the owned target and before/after R hashes matched the submitted fixture source. **The probe intentionally changed R, not B or D:** the results therefore cannot be passed off as validation of the original visible buffer. This was an API-attribution experiment, not a proposed editing route.

| Source/context | Actual return | Additional observation |
|---|---|---|
| Target syntax error | `43` / `ERR_PARSE_ERROR` | Source-correlated failure, not a result for the unchanged B. |
| Valid relative `extends "base.gd"` | `0` / `OK` | Relative path context worked in this fixture; `can_instantiate()` still returned false in the editor's non-tool context. |
| Undeclared identifier / analysis error | `43` / `ERR_PARSE_ERROR` | Same public error code as syntax failure. |
| Missing relative superclass | `43` / `ERR_PARSE_ERROR` | Same code again; a target-text-only syntax classification would be misleading. |
| Syntax-valid tool source after the missing-dependency attempt | `36` / `ERR_COMPILATION_FAILED` | Native diagnostic: “Failed to compile depended scripts,” referring to the earlier missing dependency. `can_instantiate()` was true afterward despite the failed reload. This unexpected result was retained, not repinned as a pass. |
| Tool syntax error immediately afterward | `43` / `ERR_PARSE_ERROR` | Before reload, `can_instantiate()` was still true for the newly assigned invalid source; afterward it was false. |
| Tool static initializer, `keep_state=true` | `0` / `OK` | Wrote the owned sentinel `STATIC_INIT_EXECUTED_WITH_KEEP_STATE_TRUE`. Keeping state does not make reload passive or suppress execution. |
| Tool initializer invoking a nested reload | Outer and nested calls returned `0` / `OK` | The nested call received unchanged invalid source and did not parse it; details below. |

The missing dependency was supplied as a valid owned fixture file before the static-initializer cases, changing the relevant environment. No previous failed result was rerun merely to relabel it. Error interpretation used the returned code and controlled inputs; native diagnostics were inspected for the unexpected compilation failure, not promoted into a general logging-based validation contract.

**Reentrant counterexample:** During the outer tool initializer, retain the same cached target Script, temporarily assign the two lines `extends Node` and `func broken(`, each followed by a newline, and invoke `reload(true)` on that already-reloading Script. The nested call returned `OK` while its before/after source hash remained `0e791cce4d813744b89d1518f40caaff1867a16b41b60b722cccbd5904cb0dac`; its recorded interval was `667197390`–`667197391` microseconds. The same Script ID/path was retained. The initializer restored the outer source afterward. Pinned `GDScript::reload` confirms the `if (reloading) return OK` short-circuit. Thus even identity/hash/time correlation around an arbitrary reload call does not by itself prove a fresh parse occurred.

The outer static-initializer source hash was `c008aced726300b6c0bcbdd0a8546af53d72c7cc9df7d89fcaf7e6ae5ee976d6`; its before/after hash was also unchanged. These synthetic hashes/intervals identify the exercised calls, not an engine-generated revision protocol.

**Narrowest truthful current evidence:** report a reload attempt for the observed R snapshot, its actual error code, interval, before/after source/identity/buffer facts, and known effects/limitations. Do not label it unconditional fresh parse success, passive validation, validation of another source authority, or absence of dependency/initialization effects. A general qualifying FR-009/FR-010 path still needs a supported result surface and proof of its invocation semantics; these experiments do not establish one.

### 8.5 Principle XIII and feature-boundary decision

**Outcome 2 — partial mechanisms only.** The initial native text/history mechanics remain useful evidence. A is unsupported by the examined public save routes; B has bounded observable reload-attempt evidence but no established general qualifying parse-result route. Neither fact authorizes an unsafe mutation feature or resuming Phase 1 design.

| Candidate | Concrete failure addressed / why it is insufficient | Decision |
|---|---|---|
| Established Resource path/UID; current-document selection | Intends to prevent wrong-target writes; the actual saver still resolves a mutable path, and the new omitted-path test overwrote a replacement. | Reject as target binding. |
| More path/mtime/hash checks | Detect some changes but do not bind the subsequent write; post-hoc detection cannot prevent clobbering. | Do not substitute for the missing persistence property. |
| Reload plus request-local source/interval witnesses | Prevents confusing one observed invocation/source with another; does not prevent execution, dependency effects, or guarded `OK` without a parse. | Retain as limited research evidence, not a new approved validation mechanism. |
| Change signals, editor error UI, or Logger | Can indicate activity/failure, not a source-revision-correlated positive validation result. A logger adds concurrency/privacy handling without closing the positive-evidence gap. | Do not build a diagnostic collector to claim certainty it cannot supply. |
| Custom native binding, writer, locks, daemon, or transaction framework | Might be proposed against the current gaps, but no smallest practical complete mechanism was demonstrated in this existing-API research. | None introduced or selected merely to unblock planning. |

**Requirements not supported by these routes:** the combined FR-002/FR-004/FR-013/FR-020 target/revision/confinement and newer-work guarantees for persistence; consequently the intended-target verified success required by FR-008. The general FR-009 post-change parse result and FR-010 source-attributed diagnostics are not established by the available signals, predicates, or arbitrary reload return alone. The missing capabilities are specific; native editing/history itself is not declared impossible.

**Narrowing assessment:** a buffer-only edit would deliberately leave D/R/B divergent and remove FR-008 plus constitutional Principle I rather than deliver a smaller safe edit. Requiring manual reconciliation/persistence, assuming paths never change, dropping parse verification, or calling every applied edit “partial” does not satisfy the specified positive capability and applicable A–E gates. Restricting claims to the few tested synthetic scripts is not an independently useful support boundary. A read-only proposal/inspection capability would be different scope, not a way to mark this mutation feature complete; the observation foundation already exists. No constitutionally consistent, independently useful mutation narrowing was established, and the specification was not changed.

**Resume condition:** a concrete qualifying persistence and parse-evidence route must become available and be justified/proven against the current MUSTs. This record does not invent such a route, require new infrastructure, or demand a new approval mechanism. No data model, contracts, quickstart, tasks, or implementation are generated; planning remains blocked.

### 8.6 Primary sources and verification boundary

- [ScriptEditor public methods](https://docs.godotengine.org/en/4.7/classes/class_scripteditor.html), [ScriptEditorBase](https://docs.godotengine.org/en/4.7/classes/class_scripteditorbase.html), [EditorInterface](https://docs.godotengine.org/en/4.7/classes/class_editorinterface.html), and [EditorPlugin resource-saved signal](https://docs.godotengine.org/en/4.7/classes/class_editorplugin.html#class-editorplugin-signal-resource-saved).
- [ResourceSaver contract](https://docs.godotengine.org/en/4.7/classes/class_resourcesaver.html#class-resourcesaver-method-save) and [pinned `ResourceSaver::save`](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/core/io/resource_saver.cpp).
- [Pinned native current-script save and bindings](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp).
- [ScriptLanguage](https://docs.godotengine.org/en/4.7/classes/class_scriptlanguage.html), [ScriptLanguageExtension validation hook](https://docs.godotengine.org/en/4.7/classes/class_scriptlanguageextension.html#class-scriptlanguageextension-private-method-validate), and [pinned source-specific parser/analyzer validation](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript_editor.cpp).
- [Script reload contract](https://docs.godotengine.org/en/4.7/classes/class_script.html#class-script-method-reload), [pinned reload/cache/static-initialization implementation](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript.cpp), and [Logger's actual callback contract](https://docs.godotengine.org/en/4.7/classes/class_logger.html).

No public API safety conclusion above is inferred merely from a method name. Native current-document and validator internals were source-inspected; unavailable public bindings were checked against the actual runtime. The omitted-path replacement and reload/initialization/reentrancy conclusions have the specific new experimental evidence above. The owned editor exited normally with code 0; disposable probes are cleaned up after recording these results. No product source, permanent tests, dependencies, CI topology, or observation guarantees changed. No mutation support or complete A–E result is claimed.

## 9. Native integration continuation: C1–C5

### 9.1 Method, provenance, and supported extension access — C1

Research followed the requested order: inspect and exercise the **standard extension ABI first**, assess narrowly missing engine APIs, then compare a source-integrated module. No custom engine was built or patched. The installed Godot binary was not modified.

**Runtime-observed environment:** Godot `4.7.2.stable.official.ed1daf0bf`, full hash `ed1daf0bf001b61586d9930840f2f1394092c079`; macOS `26.6.2` (`25G83`), Darwin `25.6.0`, arm64, Apple M2, macOS display driver and Compatibility renderer. A disposable C++17 dynamic library compiled with Apple clang `21.0.0` (`clang-2100.3.34.2`) and macOS SDK `27.0`. The SDK version is build provenance, not a claim of testing a different host OS.

The exact installed engine generated its interfaces:

```sh
godot --headless --dump-gdextension-interface --dump-gdextension-interface-json --dump-extension-api
clang++ -std=c++17 -Wall -Wextra -Werror -dynamiclib -I api native_probe.cpp -o project/native_probe.dylib
```

Both commands completed successfully. The generated API identified a single-precision 64-bit build; the probe used its declared opaque value sizes and C function signatures. No `godot-cpp`, engine-internal headers, private object layouts, or direct internal-symbol linking was used. Generated-file SHA-256 witnesses:

| Generated artifact | SHA-256 |
|---|---|
| `gdextension_interface.h` | `640b48188708ba0016f8d7ace9e0e1d3279a41fa1226c59ff3193b15538bd254` |
| `gdextension_interface.json` | `7d8c0a039d9743eb8ebf88681ae0c641d8d3aa5ffca11081745a84da803e09a1` |
| `extension_api.json` | `d0e4c08c03b165156dabe6bfb6a906baf0069189f62035341230a246c86d6986` |

**Demonstrated through the supported ABI in a real editor:**

- Create a native custom `Callable` using `callable_custom_create2` and invoke it from the owned fixture plugin.
- Obtain the existing `EditorInterface` singleton, its `ScriptEditor`, and the already-open script/editor arrays through generic Variant/Object calls.
- Read the existing target `Script.source_code`, CodeEdit text, current/saved versions, and instance IDs. Native instance IDs matched the independently captured GDScript-side identities; the probe did not create substitute Script/CodeEdit objects.
- Perform a CodeEdit complex operation, then observe native R synchronization separately; tag the exact buffer's saved version after the bounded file-write checks. Native Undo/Redo and earlier history remained reachable in the exercised sequence.

**Public-contract boundary:** the C ABI provides opaque object/Variant handles, instance lookup, and dispatch to exposed ClassDB methods. `object_method_bind_call`/`ptrcall` do not expose arbitrary C++ members. The editor-specific ABI helpers register/remove plugins, load help XML, and register a classes-used callback; they are not persistence or GDScript-validation hooks.

**Exact unexposed surfaces:** the generated API has no built-in GDScript source-validator method. `ScriptLanguageExtension._validate` is an implementation hook for an extension-provided language, not a call-through to GDScript's existing validator. `ScriptEditorBase` exposes `get_base_editor`, but not its C++ `get_edited_resource`, `apply_code`, `validate_script`, or `tag_saved_version`. Resource edited-state and last-modified-time setters are not present in this exact public API either. A method's presence in engine C++ does not make it supported GDExtension access.

### 9.2 Descriptor-bound persistence and its remaining revision gap — C2

**Investigated primitive, not a product implementation:** retain a project-directory descriptor; open each existing relative component with `openat` and `O_NOFOLLOW` (directories also require `O_DIRECTORY`); retain an `O_RDWR` leaf descriptor without `O_TRUNC`/`O_CREAT`; inspect the opened file's identity and expected bytes; write through that same descriptor with `pwrite`, then `ftruncate` and `fsync`. The prototype handles short writes and checks its own errors. Source, live document identity, CodeEdit version, and current pathname attachment are observations around the write, not substitutes for the descriptor binding.

The write executes **inside the editor's native extension**, not in an external writer process. A separate Python controller owns the disposable files and independently witnesses disk contents/identity. For controlled interleavings, the native call pauses after its last pre-write checks; the controller changes an owned fixture and releases it. The pause is experimental scheduling, not a proposed product mechanism. No lock service, staging journal, rollback storage, filesystem framework, or multi-file transaction was introduced.

| Exercised case | Observed behavior |
|---|---|
| Ordinary existing `.gd` | Native buffer edit, independently observed R synchronization, descriptor write/flush, and guarded CodeEdit saved tag produced matching D/R/B and a clean target. The inode was retained; the unrelated buffer remained dirty and its disk source unchanged. This was immediate candidate convergence, not verified feature success. |
| Leaf replaced after final checks | Only the retained original object received the intended source. Replacement inode `118877157` retained its sentinel; original inode `118876572` received the write. Current-path identity loss produced `namespace_lost_after_write`; no saved tag. |
| Parent replaced with a link to an owned outside directory after final checks | The original file in the retained directory received the write; the outside sentinel was unchanged. Current-path attachment failed and the result was non-success, without tagging the buffer saved. |
| Target renamed after final checks | The renamed original received the write. The old project path was absent; the result was non-success, not a claim of persistence at that path. |
| Target unlinked after final checks | An independent retained reader observed the intended bytes on the unlinked inode, with link count zero. No file was forced back into the namespace; the result was non-success. |
| Expected disk bytes changed before the write guard | `disk_revision_changed`; newer bytes remained unchanged by the native writer. |
| Resource identity changed / CodeEdit identity changed | Each was refused as `document_identity_changed`, before a disk write. |
| CodeEdit version changed after pinning | `buffer_revision_changed`; the newer synthetic human buffer was retained and disk was not written. |
| R had not yet synchronized to B | `resource_buffer_not_synchronized`; no disk write. |
| Descriptor opened read-only | Real `pwrite` failure: `EBADF` (`9`), zero bytes written, disk unchanged and saved tag withheld. |
| Controlled synchronization-failure status | After a real write and successful `fsync`, an explicitly injected `EIO` status exercised the failure branch: applied bytes remained visible, but no saved tag. This is **not** evidence of an actual hardware/`fsync` failure. |
| Same inode changed after the final revision check | A controller wrote a newer revision through the existing inode. The native `pwrite` then overwrote it and reached its candidate saved-tag stage. Subsequent D/R/B equality did not reveal the lost intermediate revision. This **fails** the required stale-write/newer-work guarantee. |

These outcomes distinguish two properties: **descriptor binding prevents pathname substitution from redirecting the write; it does not make read/check/write a conditional compare-and-write on file contents.** The latter counterexample concerns the same validated object, not a replacement object. Retaining the FD, running on the editor's main thread, hashing again, or verifying afterward does not exclude a non-cooperating external in-place writer between the check and `pwrite`. Editor-main-thread serialization protects against ordinary interleaved editor input, not every filesystem writer. This is a concrete FR-004/FR-013 gap, not a demand for speculative multi-agent infrastructure.

A small ordinary-user Darwin write-lease attempt on an owned file, using the current SDK's `F_SETLEASE`/`F_SETLEASE_ARG(F_WRLCK, 0)`, returned `EPERM` (`1`). No privileges or entitlements were sought. This does not establish that every Darwin coordination primitive is unavailable; it provides no usable exclusion guarantee for this candidate. Advisory locks or a custom engine build must not be asserted to solve non-cooperating writes without evidence.

**Disposition:** retain descriptor-bound writing as demonstrated useful native capability. Reject this prototype as the complete expected-revision persistence primitive. It also lacks a product cancellation/deadline boundary and is not crash-atomic; errors after writing require truthful partial-application evidence, not assumed rollback.

### 9.3 Native editor saved state is more than a CodeEdit tag — C3

The experiment deliberately used existing CodeEdit history, not a second history stack. One native Undo restored the prior buffer; Redo restored the candidate source, and earlier native history remained reachable. At successful primitive writes, unrelated dirty-buffer text and its separate disk source were preserved. Identity/version/R-synchronization refusals and the write/failure cases above did not manufacture a saved buffer state.

However, **direct `CodeEdit.tag_saved_version()` is not the editor document's complete native saved-state transition**. Pinned source establishes the missing distinction:

- `TextEditorBase::tag_saved_version()` tags CodeEdit **and** calls `ScriptEditorBase::tag_saved_version()`.
- The base method updates `edited_file_data.last_modified_time` for the document.
- `ScriptEditor::_test_script_times_on_disk()` compares that field with the current file time; it is not simply comparing `Resource` source or CodeEdit's saved version.
- `ScriptEditor::_res_saved_callback()` invokes the document-level tag and updates script names/live-reload bookkeeping after the engine's resource-save notification. ResourceSaver additionally maintains Resource save state and timestamps.

**New real-editor counterexample:** in a fresh owned editor, perform the descriptor-backed edit to source returning `207`, observe matching D/R/B and a clean target, then Undo to source returning `17`. Send the native **script-only Save shortcut, Cmd+Alt+S**; the fixture recorded both modifiers. Godot displayed **“Files have been modified outside Godot”**, emitted no resource-saved events, and left D at `207` while R/B remained at the undone `17`. The unrelated dirty document and its disk were unchanged. Redo remained available and restored `207`. Thus the candidate did not satisfy ordinary Undo → Save synchronization without human reconciliation.

Evidence qualification: an earlier Cmd+S dispatched the broader editor save action and persisted both synthetic documents. That is not target-only Save evidence. Later controls edited on disk were not loaded into that first running tool script; their responses are also excluded from the target-only claim. The fresh process identified fixture revision 2, recorded the actual Cmd+Alt+S event, started without the old dialog, and supplied the isolated result above. No failed result was relabeled as a passing target-only save.

**Minimum missing editor capability:** a guarded target-document saved-state finalization path, covering the native document bookkeeping as well as the exact buffer version after persistence. Its target/source/version and still-valid write evidence must be checked; it must not mark newer human text saved after a failure or identity loss. The exact necessary Resource callbacks/export/doc effects must be accounted for, not imitated by emitting a convenient signal. A naked public “mark clean” method would not establish those guarantees.

Standard GDExtension cannot write the unbound `edited_file_data` field or invoke its unbound document-level tag. Linking/casting against that private C++ layout is not a supported workaround. A narrow engine-side exposure could address this specific integration gap; it is additional to the validator exposure, and does not fix the disk revision race. No such patch was implemented or proven. The new candidate's close/reopen, rescan, reparse and runtime durability are **not established**; the observed ordinary-Save failure already prevents a complete route.

### 9.4 Native source validator, effects, and minimum exposure — C4

**Pinned-source entrypoint:** `GDScriptLanguage::validate(source, path, functions, errors, warnings, safe_lines)` creates a local `GDScriptParser` and `GDScriptAnalyzer`, calls `parser.parse(source, path, false)`, then `analyzer.analyze()` if parsing succeeded. It returns a boolean and can produce `ScriptLanguage::ScriptError` records containing path, line, column and message. Root errors use the supplied path; depended-parser errors retain their dependency paths. This is the actual parser-plus-analysis route used by the GDScript editor, not a substitute parser or an arbitrary `Script.reload()` result.

The root entrypoint consumes an explicit source string and path, not a target Script or CodeEdit. It therefore does not require directly assigning the proposed source to B/R or saving D. A returned path is a context/diagnostic label, not proof of selected document identity. Callers still need the exact immutable source input, target/session identity, revision and invocation interval, plus independent post-change D/R/B witnesses. An earlier proposed-source result cannot silently stand in for required fresh post-change evidence. Full bytecode compilation, gameplay execution and project-wide diagnostics are not added to FR-009.

**Effects cannot be inferred away:**

| Source path | Pinned behavior / evidence limit |
|---|---|
| Root parse and analysis | No direct assignment to the target Script/CodeEdit or source-save call in the root validator. It is a fresh local parser/analyzer invocation, not the reentrant reload short-circuit established in §8. No direct native-validator runtime invocation was performed in this research. |
| Relative GDScript inheritance and dependencies | Analyzer/cache resolution uses the supplied script path and current project context; parser/dependency caches and dependency edges may be used or changed. Dependency source can be read on cache misses. A source/path pair is not a snapshot of the entire dependency environment. |
| Globals, classes and autoloads | Live ScriptServer/project settings and registered language/class context participate. Some non-GDScript resolution paths call ResourceLoader. A separate validator process would not automatically reproduce this context. |
| `preload()` | `reduce_preload` resolves the path, checks resource availability/type, obtains a shallow GDScript through the GDScript cache or calls `ResourceLoader::load(..., CACHE_MODE_REUSE)` for other resources. The source explicitly notes “Don't load if validating: use completion cache.” That improvement is not implemented by a new binding. |
| Reduced values / property access | Analyzer paths also use `Variant::get` on reduced values. **[INFERENCE]** resource loaders, scripted-resource construction or property behavior can therefore have effects beyond examining the root text; this call tree does not establish a universal no-script-execution or no-initializer guarantee. No new dependency-constructor/static-initializer probe was run, and effects from §8's reload experiments are not attributed to this validator. |

Cache activity alone is not being invented as a new prohibition. The required distinction is between documented native validation effects and arbitrary execution, unrelated-work mutation, or confinement violations prohibited by the existing specification. A resumed design must establish that boundary rather than claim that the word “validate” guarantees purity. Target-independent dependency diagnostics must remain distinct, not be mislabeled as target parse failures.

**Availability:** no matching source-validator ClassDB method or C ABI entrypoint appears in the exact generated extension surface. Implementing `ScriptLanguageExtension._validate` does not supply access to the built-in language's override. Calling `GDScriptLanguage::validate` by including/linking engine internals requires an engine-side module/build or new API exposure, not stock GDExtension.

**Smallest missing result exposure, conceptual only:** a GDScript-owned source/path validation call that delegates to this existing parser/analyzer and returns its validity and structured root/dependency errors. A small static binding on an exposed GDScript-owned class is one possible placement; this is not a finalized API/schema. It need not expose a compiler framework, general scripting-language protocol, LSP, subprocess service, or internal parser pointers. The binding must document project/thread context and actual effects; freshness/identity witnesses remain part of the existing automation semantics.

**[INFERENCE] Upstream/local-patch assessment:** such a source-validation result is plausibly upstreamable because it exposes an existing engine capability without duplicating a parser. Upstream acceptance is unconfirmed. A local patch could be proportionate for the current exact candidate only after its effects and required safety behavior are concrete and regression-proven. It would still require maintaining/distributing a patched editor until an appropriate released API exists; a few binding lines do not remove that cost. No patch is authorized or implemented here. A bare wrapper does not by itself solve dependency-effect concerns, editor finalization, or the same-inode revision race.

### 9.5 Integration-level comparison — C5

| Level | Persistence, validation and editor participation | Present cost/coupling | Sufficiency |
|---|---|---|---|
| Standard GDExtension only | Demonstrated existing-object access, native text/history, descriptor-bound writes and immediate D/R/B convergence. Missing conditional disk revision protection, native document saved-state finalization and callable built-in source-validator result. | One current-platform native library and plugin lifecycle/export boundary; compiler/SDK and extension ABI compatibility checks; binary packaging for the tested candidate. No engine-private C++ linkage is needed for the demonstrated operations. | **Not sufficient.** Failure is specific missing semantics/access, not use of C++. |
| GDExtension plus narrow engine API exposure | Could expose the existing source-validator result and guarded document saved-state finalization while keeping the writer inside the editor. A validator binding alone would miss the independently demonstrated saved-state gap. Neither exposure supplies filesystem compare-and-write. | Small, purpose-specific engine API changes, but still an exact patched-editor build/distribution and compatibility burden until released upstream; maintenance of validation effects and finalization guards. Reuses native history/parser rather than adding parallel systems. | **Closest next candidate, not an established minimum sufficient level.** No complete design closes the revision gap or proves these added API semantics. Outcome 3B is not claimed. |
| Small custom module / narrowly patched editor build | Can call the internal validator and document finalization directly, avoiding a missing ClassDB binding. It can host the same descriptor operation. Internal access alone does not repair its revision race or make dependency loading harmless. | Direct coupling to GDScript parser/cache/analyzer and editor internals; engine-source compilation, patch/module maintenance, exact-build distribution and repeated real-editor compatibility evidence. More responsibility than a narrow exposure, without demonstrated additional protection for the remaining disk race. | **Not shown necessary or sufficient.** No reason was established to skip straight to a module or general engine fork; Outcome 3C is not claimed. |

None of these levels needs a second automation safety model: any future native component belongs at the existing Godot integration boundary, with the protocol-independent core retaining common outcomes/coherence semantics and Godot retaining native history. Moving code into a module is not itself an atomicity guarantee. No language-binding library, permanent native dependency, general plugin framework, or custom-engine distribution project is selected.

### 9.6 Principle XIII, decision, and verification boundary

**Current need:** prevent the already-demonstrated wrong-object save, stale/newer-work overwrite, misleading saved-state transition and unattributed parse result. **Simplest credible alternative:** retain public native editing/history and add only missing in-editor primitives. The disposable C ABI library proves that native integration can eliminate an external writer and its cross-process synchronization costs for object-bound persistence. Avoiding C++ would not be a valid reason to reject that result.

**Why existing mechanisms remain insufficient:** ordinary savers follow paths; the native descriptor candidate closes that particular redirection risk but not the competing-revision race; a CodeEdit saved tag omits document timestamp bookkeeping; the genuine validator lacks a supported result binding and has dependency effects requiring truthful treatment. These are observed or pinned-source-specific gaps, not speculative future requirements.

**Ongoing cost and justification:** a scoped native library would require descriptor/object lifetime and partial-write handling, native saved-state integration, ABI/build/export verification and exact-version regression coverage. Narrow engine changes additionally require a maintained editor build and source/API compatibility review. Those costs can be justified if they actually close the current MUSTs and remove more complex external composition; they are not yet justified as a selected complete design by the partial prototype. A module adds private coupling without demonstrated extra protection for the outstanding filesystem race.

**Decision: Outcome 3D — no sufficient proportionate native route established in this research.** This is a bounded research result, not a proof that all native mechanisms are impossible. The descriptor primitive is useful positive evidence, but the same-object revision counterexample and ordinary-Save failure prevent claiming a safe complete edit. No minimum sufficient integration level is selected. The next candidate is still a small in-editor native boundary with narrowly exposed engine capabilities, not an automatic fallback to an external writer/validator composition.

Before planning can resume, evidence must establish:

1. Target-bound persistence with actual expected-revision/newer-work protection at its mutation boundary, including the observed same-inode interleaving or a demonstrated equivalent protection—not an assumed cooperative-writer restriction.
2. Guarded exact-document saved-state integration that survives ordinary native Undo/Save/Redo and applicable durability without reconciliation, false clean state, unrelated saves, or history loss.
3. A supported fresh source-attributed native validation result with explicit project/dependency/effect semantics consistent with the existing safety requirements.

The behavioral specification was not disproved or amended; the examined implementation routes are insufficient. `/speckit.plan` was **not resumed**. No downstream design artifacts, tasks, product implementation, engine patch, new PR, automatic merge, acceptance-suite run, or A–E/support claim follows from this research.

**Verification and cleanup:** actual native compilation/loading/object calls and the descriptor/identity/failure cases above ran in owned GUI editors; both editor processes exited normally with code `0`. The first editor log contained a debugger-plugin-not-attached error. Its broad save action is not accepted as target-only evidence; the fresh process isolated the script-only Save result. C4's additional validator conclusions are source/API inspection, not a new runtime validation test. Disposable projects, probes, generated API files, binaries and sentinels were removed after recording the findings; no permanent dependency or product file changed. Existing observation acceptance was not rerun or repurposed as mutation evidence.

Primary sources for this continuation:

- [Pinned extension ABI implementation](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/core/extension/gdextension_interface.cpp) and [interface schema](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/core/extension/gdextension_interface.json). The exact runtime dumps above, not an assumed binding-generator surface, determined the probe.
- [Pinned document base and saved-state methods](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_base.cpp), [document state declarations](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_base.h), [script editor Save/timestamp/callback/shortcut paths](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp), and [ResourceSaver bookkeeping](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/core/io/resource_saver.cpp).
- [Pinned native validator](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript_editor.cpp), [parser](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript_parser.cpp), [analyzer](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript_analyzer.cpp), [cache](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript_cache.cpp), and [extension-language hook binding](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/core/object/script_language_extension.cpp).
- [Apple's descriptor-write contract](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/pwrite.2.html). Current-host semantics and the lease refusal above were actually exercised; an archived manual or newer SDK definition is not itself a compatibility/safety proof.
