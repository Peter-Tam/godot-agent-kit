# Phase 0 Research: Safe Open-GDScript Editing

**Date**: 2026-09-27 | **Specification**: [spec.md](spec.md) | **Gate record**: [plan.md](plan.md)

**Result: Outcome 2 — partial mechanisms only; planning remains BLOCKED.** The focused A/B continuation in §8 found no qualifying target-bound single-script persistence route among the examined public Godot 4.7.2 APIs. Source-correlated reload-attempt evidence is observable, but is not a general fresh parse-success witness. No independently useful safe-mutation narrowing was established. This record does not weaken the specification, authorize implementation/task generation, or claim a mutation-supported environment.

**Continuation boundary:** Resolve only **A — target-bound single-script persistence**, including preservation of unrelated unsaved work, and **B — source-attributed parse evidence** for the selected target/revision/interval. Both questions must be answered before planning can resume, and their answers must preserve every applicable safety invariant. No implementation or task generation is authorized while the gate remains blocked.

**Evidence history:** Sections 1–7 preserve the initial blocked investigation, committed and pushed to existing PR #24 as `5748899` before the focused continuation. Section 8 records the additional questions, new experiments, and decision; it does not relabel the earlier probe as feature acceptance.

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
