# Phase 0 Research: Safe Open-GDScript Editing

**Date**: 2026-09-27 | **Specification**: [spec.md](spec.md) | **Gate record**: [plan.md](plan.md)

**Current status: Planning complete — concrete native integration selected (State A).** Section 11 resolves the finalization and validation design responsibilities and the normal design artifacts are generated; implementation and acceptance remain pending. Completed runtime results and pinned-source/API findings remain evidence. The later native probe did not complete because the coding environment refused it on cybersecurity-policy grounds; that tooling limitation is not technical evidence against native integration. Section 10's guarded single-editor concurrency boundary is retained. No implementation, tasks, engine patch or mutation-support claim follows from this planning decision.

**Continuation boundary:** Define the two narrow native capability contracts and complete the installed `/speckit.plan` workflow, including its ordinary design artifacts and constitutional review. This is source/API inspection and design, not a new runtime experiment. No filter bypass, equivalent rephrased retry, broad API probing, task generation or product implementation is performed. Plans specify how future implementation will establish guarantees; a finished binary and A–E proof are not prerequisites for writing the plan.

**Evidence history:** Sections 1–7 preserve the initial investigation published as `5748899`; §8 records A/B research published as `c0e3f66`. Section 9 records completed native observations from `9e5095e`; its overbroad concurrency conclusions are corrected in §10. Section 10 records the tooling-policy interruption, authoritative boundary audit and scoped specification clarification published as `9368ba7`. Section 11 supersedes the earlier planning-blocked decision, not the experimental evidence. Outcome 3D remains withdrawn; failures remain failures and no historical observation becomes feature acceptance.

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

### 9.2 Descriptor-bound persistence and its atomicity limitation — C2

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
| Same inode changed after the final revision check | A controller wrote a newer revision through the existing inode. The native `pwrite` then overwrote it and reached its candidate saved-tag stage. Subsequent D/R/B equality did not reveal the lost intermediate revision. This remains a negative result demonstrating absence of atomic external-writer exclusion; §10 corrects the claim that such exclusion is a Phase 1 MUST. The known interference prevents treating this experiment as verified success. |

These outcomes distinguish two properties: **descriptor binding prevents pathname substitution from redirecting the write; it does not make read/check/write a conditional compare-and-write on file contents.** The counterexample concerns the same validated object, not a replacement object. Retaining the FD, running on the editor's main thread, hashing again, or verifying afterward does not exclude a non-cooperating external in-place writer between the check and `pwrite`. This limitation remains documented. The earlier inference that FR-004/FR-013 therefore require arbitrary-writer atomic exclusion is superseded by §10's requirements audit. Phase 1 still requires fresh stale/conflict checks, preservation of unsaved editor work, bound writes, and non-success on known/detected invalidation; it cannot assume external activity away.

A small ordinary-user Darwin write-lease attempt on an owned file, using the current SDK's `F_SETLEASE`/`F_SETLEASE_ARG(F_WRLCK, 0)`, returned `EPERM` (`1`). No privileges or entitlements were sought. This does not establish that every Darwin coordination primitive is unavailable; it provides no usable exclusion guarantee for this candidate. Advisory locks or a custom engine build must not be asserted to solve non-cooperating writes without evidence.

**Disposition:** retain descriptor-bound writing as a promising Phase 1 persistence mechanism with demonstrated object binding and bounded refusal behavior. The prototype is not a completed transaction: native document finalization, source-attributed validation, cancellation/deadline behavior and independent verification still need an integrated design. It is not crash-atomic; errors after writing require truthful partial-application evidence, not assumed rollback. Neither a generic crash-atomic filesystem transaction nor arbitrary same-inode serialization is added as a planning prerequisite.

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

Standard GDExtension cannot write the unbound `edited_file_data` field or invoke its unbound document-level tag through the inspected supported ABI. Linking/casting against that private C++ layout is not a supported workaround. A narrow engine-side exposure remains a candidate for this specific integration gap, alongside validator exposure. No such patch was implemented or proven. The prototype's close/reopen, rescan, reparse and runtime durability are **not established**; its observed ordinary-Save failure demonstrates incomplete finalization, not that a guarded native finalizer cannot work.

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

**[INFERENCE] Upstream/local-patch assessment:** such a source-validation result is plausibly upstreamable because it exposes an existing engine capability without duplicating a parser. Upstream acceptance is unconfirmed. A local patch could be proportionate for the current exact candidate after the required semantics and a credible verification approach are specified; implementation/support claims would still require regression evidence. Maintaining/distributing a patched editor until a suitable released API exists remains a real cost. No patch is authorized or implemented here. Dependency/effect semantics and guarded finalization remain design questions, not failed experiments or a reason to require arbitrary-writer atomicity.

### 9.5 Integration-level comparison — C5

| Level | Persistence, validation and editor participation | Present cost/coupling | Sufficiency |
|---|---|---|---|
| Standard GDExtension only | Demonstrated existing-object access, native text/history, descriptor-bound writes and immediate D/R/B convergence. The inspected ABI lacks document-level saved-state finalization and a callable built-in source-validator result. | One current-platform native library and plugin lifecycle/export boundary; compiler/SDK and extension ABI compatibility checks; binary packaging for the tested candidate. No engine-private C++ linkage is needed for the demonstrated operations. | **Demonstrated useful boundary; incomplete current prototype/API coverage.** This finding does not disprove a native solution. |
| GDExtension plus narrow engine API exposure | Candidate exposure of the existing source-validator result and guarded document saved-state finalization, with persistence remaining inside the editor. A validator binding alone does not supply the identified finalization semantics. Arbitrary-writer compare-and-write is not required by the audited Phase 1 scope. | Purpose-specific engine API changes, with a patched-editor build/distribution and compatibility burden until released upstream; maintenance of validation effects and finalization guards. Reuses native history/parser rather than adding parallel systems. | **Promising candidate; minimum sufficient design unresolved.** The interruption is not evidence that these capabilities fail. The remaining questions are listed in §10. |
| Small custom module / narrowly patched editor build | Could invoke the internal validator and document finalization directly and host the same descriptor operation. | Direct editor/GDScript coupling, engine-source compilation, patch/module maintenance, exact-build distribution and real-editor compatibility evidence. More responsibility than narrow API exposure. | **Not selected or demonstrated necessary.** No evidence justifies jumping to a general engine fork; internal access alone would not settle the outstanding finalization/validation semantics. |

None of these levels needs a second automation safety model: any future native component belongs at the existing Godot integration boundary, with the protocol-independent core retaining common outcomes/coherence semantics and Godot retaining native history. No language-binding library, permanent native dependency, general plugin framework, or custom-engine distribution project is selected.

### 9.6 Principle XIII, decision, and verification boundary

**Current need:** prevent wrong-object saves, stale intent applied despite current checks, loss of unsaved editor work, misleading saved-state transitions and unattributed parse results. **Simplest credible alternative:** retain public native editing/history and add only missing in-editor primitives. The disposable C ABI library demonstrates object-bound persistence inside the editor without an external writer. Avoiding C++ is not a valid reason to reject that result; arbitrary external-writer serialization is not added to the current requirement.

**Remaining gaps:** ordinary path savers retain their demonstrated redirection failures; the descriptor candidate supplies useful protection against that risk. Direct CodeEdit tagging omits document timestamp bookkeeping, and the genuine validator lacks a supported result binding with settled invocation/effect semantics. These specific gaps remain open. Neither the uncompleted policy-blocked probe nor the absence of a Phase 9-style coordination guarantee establishes failure of the native direction.

**Ongoing cost and justification:** a scoped native library requires descriptor/object lifetime and partial-write handling, native saved-state integration, ABI/build/export verification and exact-version regression coverage. Narrow engine changes also require editor-build and source/API maintenance. Those costs may be proportionate when they close the actual current MUSTs with fewer moving parts than external composition. No present requirement justifies a general lock manager, journal, daemon or concurrency framework; the same-inode experiment does not authorize pulling such machinery into Phase 1.

**Corrected decision: State B — design still unresolved; native integration remains promising.** The earlier Outcome 3D classification overstated the scope of the evidence. The policy-refused later probe did not produce a technical result, and the atomicity limitation is not an independently established Phase 1 failure. The ordinary-Save counterexample remains a real failure of the tested finalization sequence. A small in-editor native boundary plus narrow finalization and validation capabilities remains the candidate; no minimum sufficient design is yet selected.

The current planning prerequisites are the concrete finalization and validation questions in [§10](#10-phase-1-concurrency-boundary-audit-and-corrected-planning-decision), with all existing Phase 1 stale/conflict, confinement, verification and uncertainty protections retained. They are not a requirement to solve arbitrary external-writer serialization or to complete the full release-acceptance suite before writing a plan.

The specification receives only the scoped concurrency clarification described in §10; no functional requirement or acceptance scenario is removed. `/speckit.plan` remains **not resumed** because design questions remain, not because the tooling filter proved a native mechanism unsafe or impossible. No downstream design artifacts, tasks, product implementation, engine patch, new PR, automatic merge, acceptance-suite run, or A–E/support claim follows from this correction.

**Verification and cleanup:** actual native compilation/loading/object calls and the descriptor/identity/failure cases above ran in owned GUI editors; both editor processes exited normally with code `0`. The first editor log contained a debugger-plugin-not-attached error. Its broad save action is not accepted as target-only evidence; the fresh process isolated the script-only Save result. C4's additional validator conclusions are source/API inspection, not a new runtime validation test. Disposable projects, probes, generated API files, binaries and sentinels were removed after recording the findings; no permanent dependency or product file changed. Existing observation acceptance was not rerun or repurposed as mutation evidence.

Primary sources for this continuation:

- [Pinned extension ABI implementation](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/core/extension/gdextension_interface.cpp) and [interface schema](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/core/extension/gdextension_interface.json). The exact runtime dumps above, not an assumed binding-generator surface, determined the probe.
- [Pinned document base and saved-state methods](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_base.cpp), [document state declarations](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_base.h), [script editor Save/timestamp/callback/shortcut paths](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp), and [ResourceSaver bookkeeping](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/core/io/resource_saver.cpp).
- [Pinned native validator](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript_editor.cpp), [parser](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript_parser.cpp), [analyzer](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript_analyzer.cpp), [cache](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript_cache.cpp), and [extension-language hook binding](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/core/object/script_language_extension.cpp).
- [Apple's descriptor-write contract](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/pwrite.2.html). Current-host semantics and the lease refusal above were actually exercised; an archived manual or newer SDK definition is not itself a compatibility/safety proof.

## 10. Phase 1 concurrency boundary audit and corrected planning decision

**Historical audit state:** This section records the boundary correction and remaining questions at `9368ba7`. Section 11 now resolves those planning questions; it does not revise the concurrency conclusion or relabel any experiment.

### 10.1 Evidence categories and interruption

This is a requirements/documentation audit, not a new filesystem or editor experiment. The maintainer reported that a later native probe was refused by the available coding environment's cybersecurity-policy filter and did not complete. No bypass, evasion or equivalent rephrased retry was attempted. The interruption is a tooling limitation, **not** a native-runtime failure, an unsupported Godot capability result, or proof that the remaining design cannot work. No blocked experiment is claimed as successful.

| Evidence category | What is established / what is not |
|---|---|
| Demonstrated positive runtime behavior | Exact-version GDExtension loading and existing-object access; native complex editing and exercised history; descriptor-bound writes that did not follow replacement paths; safe namespace/identity-loss outcomes; preservation of unrelated dirty work in target-specific cases. These remain bounded observations, not A–E acceptance. |
| Demonstrated negative runtime behavior | Earlier broad/path-based saves retained their documented failures. The descriptor prototype overwrote an intermediate same-inode write; it has no arbitrary-writer atomicity guarantee. Direct CodeEdit saved tagging left document bookkeeping stale and ordinary script Save after Undo required reconciliation. No result is relabeled as a successful experiment. |
| Pinned-source/API finding | The genuine `GDScriptLanguage::validate` entrypoint and its parser/analyzer path are identified; the inspected supported ABI does not expose that result or the document-level saved-state transition. Dependency/effect caveats and the missing timestamp bookkeeping have the source evidence in §9. |
| Unresolved technical question | The exact guarded finalization capability and the source-validation exposure/context/effect semantics needed to satisfy the current feature. Identifying a missing capability is not evidence that a narrow exposure is infeasible. |
| Tooling-policy research interruption | A later probe produced no completed runtime result because the coding environment refused it. This explains an evidence limit, not a technical design verdict. Any still-needed unavailable experiment remains unresolved. |

### 10.2 Authoritative evidence and interpretation

**Audit conclusion: Interpretation A — guarded single-editor mutation.** Phase 1 does not currently promise linearizable exclusion against an arbitrary non-cooperating process writing the same inode between the final valid checks and the physical write. The conclusion comes from reading the following requirements together, not from implementation difficulty or the tool-policy interruption:

| Authority | Exact governing language / relevance |
|---|---|
| [Constitution II](../../.specify/memory/constitution.md#ii-preserve-human-work-and-refuse-conflicts-safely), lines 30–38 | “Human unsaved editor state MUST take priority”; operations “MUST detect dirty or conflicting state wherever Godot exposes it”; read-modify-write operations “SHOULD use revision/hash-based optimistic concurrency to detect changes since observation,” with equivalent stale-write protection for alternatives. These require real current checks and editor-work protection; they do not specify exclusion of all non-participating filesystem writers. |
| [Constitution I](../../.specify/memory/constitution.md#i-editor-coherence-is-the-correctness-boundary) and [IV](../../.specify/memory/constitution.md#iv-verify-mutations-before-reporting-success), lines 16–25 and 56–65 | Success requires observed D/R/B coherence, defined mutation/synchronization/verification stages, explicit unavailable state and partial failures. They forbid false success but do not define a globally serialized filesystem history. |
| [Roadmap Phase 1](../../ROADMAP.md#phase-1--live-editor-script-coherence), lines 73–109 | Purpose: “one reliable vertical slice” with “human unsaved work protected.” Required capabilities include revision/stale-write protection and independent verification; A–E cover clean/dirty edits, native history, persistence and sequential stress. Arbitrary external-writer atomic exclusion is not an exit criterion. |
| [Roadmap Phase 9](../../ROADMAP.md#phase-9--multi-editor-multi-project-and-multi-agent-operation), lines 251–267 | “Expand beyond proven single-editor transactions”; capability areas add “leases/locks where appropriate,” “revision conflicts,” and “concurrent mutation handling”; exit proof includes coherence “under competing edits and session changes.” This is where stronger coordination across actors belongs if required, not an instruction to implement it in Phase 1. Phase 9 itself does not promise universal exclusion of arbitrary non-cooperating processes. |
| [Roadmap sequencing](../../ROADMAP.md#how-to-read-the-sequence), lines 22–29, and Phase 9 lines 258–263 | Later phases do not defer baseline safety. Ambiguous target refusal already applies; locks cannot replace revision checks or human-buffer protection. Therefore the boundary cannot waive current target, dirty-state, confinement or verification requirements. |
| [Working agreement](../../AGENTS.md#transaction-and-editor-safety), lines 252–271 | Requires attributable dirty/conflict observations, guarded stale revisions, independent D/R/B, native semantics and truthful partial outcomes. It does not add arbitrary-writer atomicity to the constitution. Its complexity gate rejects mechanisms without a current requirement. |
| [Feature 002](spec.md), FR-001–FR-008, FR-013, Scope and Assumptions | One already-open script in one selected editor; separate application/persistence/verification stages; fresh required evidence and non-success on invalidation; concurrent-agent orchestration excluded. FR-004's “at the mutation boundary” and FR-013's “preserve newer human work” are broad enough to be over-read without an explicit concurrency scope. That ambiguity warrants the small clarification below, not a new global atomicity requirement. |
| [Feature 001 observation semantics](../001-observe-gdscript-state/spec.md#outcome-interpretation) and [caller contract §4](../001-observe-gdscript-state/contracts/observation-api.md#4-completeness-and-failures) | A snapshot describes an interval, “not a promise that all authorities were frozen at one instant”; `consistency.atomic` is always false, and absence of detected changes is not proof of stability. The caller contract supplies no mutation revision token/authority. This defines inherited evidence limits; it does **not** waive Feature 002's fresh mutation-boundary checks. |

**Why the stronger reading is not selected:** optimistic checks can reject stale observations without promising atomic exclusion of unrelated writers that do not participate in the transaction. Neither the constitution nor the Phase 1 roadmap specifies that stronger promise. Feature 002's broad protection clauses must be read in its one-editor scope, alongside explicit uncertainty/non-success semantics; the word “newer” alone does not select a global serialization model. Elevating the controlled interleaving into a mandatory CAS/lease design was an unsupported expansion of the planning requirement.

**What remains required:**

- Before source application, fresh checks must reject stale revisions, dirty/conflicting editor state, wrong/changed target or session, and missing required observability. A stale preflight result cannot authorize a later edit.
- During persistence, the actual write must stay bound to the validated object despite path/parent replacement. Known changed revisions or document/Resource/buffer identities must prevent continuing an obsolete mutation. Where B already changed, report the actual applied/unknown state rather than claiming an unchanged refusal.
- After application, fresh independent D/R/B, dirty/sync and parse evidence remains mandatory. Known/detected invalidation or divergence requires non-success; do not reassert old intent, force reconciliation, silently retry, or infer rollback.
- The absence of arbitrary-writer serialization is an explicit limitation, not permission to ignore external changes, suppress evidence or claim all intermediate writes were preserved.

The same-inode counterexample stays a negative result: it demonstrates what the descriptor primitive does not guarantee. In that instrumented case the controller knew a competing write occurred, so the candidate's saved-tag acknowledgment and later equality cannot turn the experiment into verified success. For ordinary feature outcomes, verification attests the independently observed interval—not an unobserved globally atomic history. This audit does not invent detection of every transient external write.

### 10.3 Minimal specification clarification

The [Phase 1 concurrency boundary](spec.md#phase-1-concurrency-boundary) clarifies the scope of FR-004/FR-013 and their acceptance scenarios in one place. All 22 functional requirements, five stories, 26 scenarios and eight success criteria remain. It distinguishes required fresh stale/conflict checks, unsaved-buffer protection, target/session identity, non-redirectable persistence and invalidation-aware verification from unclaimed arbitrary same-inode atomic serialization.

No dirty-state requirement, preflight-to-application editor guard, target/path protection, safe non-success outcome, independent D/R/B/parse requirement, native history or durability gate is removed. Human activity and external changes remain realistic interference. No constitutional amendment, blanket quiet-filesystem assumption or generic concurrency model is introduced. The existing requirements-quality checklist is intentionally unchanged; its prior quality review is not evidence that an implementation or interrupted probe passed.

### 10.4 Remaining design work and planning decision

**State B — design still unresolved.** The scope blocker is narrowed: arbitrary external-writer CAS is not a prerequisite. The promising candidate remains **standard GDExtension + narrow guarded target-document finalization + narrow native GDScript validation exposure**, within the existing Godot integration boundary. It is not yet a sufficiently defined implementation design.

| Remaining capability | Known basis | Question still needed for a truthful plan |
|---|---|---|
| Guarded target-document finalization | Descriptor persistence and native history worked in bounded cases; the precise document timestamp omission is identified. | Specify the exact supported entrypoint, guarded target/source/version and persistence-result handoff, required native timestamp/saved-version/Resource bookkeeping, callback ordering, and failure/identity-change behavior. It must neither tag newer human text clean nor save unrelated scripts. A list of fields or a naked “mark saved” call is not the design. |
| Source-attributed validation | `GDScriptLanguage::validate` already provides source/path parser-plus-analyzer semantics and structured target/dependency diagnostics internally. | Specify the narrow callable exposure, selected-editor/project/thread context, source/revision/interval attribution and treatment of dependencies/effects under FR-009/FR-010/FR-020. No proposed binding or source inspection substitutes for an observed post-change result. Do not turn normal cache activity into a new blanket prohibition or silently permit unrelated-work mutation/arbitrary evaluation. |

These are design questions, **not** failed implementations of the proposed capabilities. Their needed runtime evidence remains unperformed where the environment cannot permit it; no blocked probe is retried here. Once the semantics are concrete enough to satisfy the current MUSTs, a plan can define implementation and verification work. The full A–E release suite is not being imposed as a prerequisite to drafting that plan.

The existing deadline/cancellation/application-uncertainty requirements must be designed around the chosen boundary; they do not call for a new service. `/speckit.plan` is **not resumed** in this audit. No tasks or implementation are generated.

**Principle XIII review:** the current failure modes are wrong-target persistence, stale/dirty editor intent, incomplete native saved state and unavailable attributed parse evidence. Reusing native history, the demonstrated in-editor writer and the engine's own validator is the simplest current candidate. Public CodeEdit tagging and the inspected ABI do not yet provide the two needed capabilities. A small native library/API exposure adds build, ABI/editor-version, lifetime and bookkeeping maintenance costs, potentially justified by removing external writer/validator synchronization. C++ alone is not a reason to reject it. General filesystem transactions, daemons, lock managers, journals, multi-agent machinery, new approval gates and new CI infrastructure do not close a newly established Phase 1 requirement here and are not introduced. No stronger guarantee is silently assigned to Phase 9 either.

## 11. Concrete native design and resumed planning

### 11.1 Decision, evidence and planning threshold

**Decision: State A — the two native gaps are concrete enough to plan.** Select the existing Rust/core → existing private bridge → Godot integration → standard GDExtension → narrow engine API boundary. The [native contract](contracts/native-integration.md) specifies guards, effects, outputs and failures; the [data model](data-model.md), [caller contract](contracts/edit-api.md) and [bridge contract](contracts/bridge-protocol.md) specify orchestration and compatibility. No material architectural question remains for planning.

This conclusion is a **planning decision based on demonstrated components and pinned source**, not a completed runtime implementation. The extension/object access, native history and descriptor-bound-write observations in §9 remain positive evidence. The same-inode overwrite, incomplete document timestamp/save bookkeeping, broad-save failures and reload/parse limitations remain negative evidence. The policy-interrupted probe supplies no new runtime result. None is rerun here.

The proposed engine hooks are implementable responsibilities within existing engine-owned code, not claims that stock bindings already exist. Exact implementation symbols, compiled patch/binary identities, fault behavior and acceptance results must be supplied by implementation. Those are construction and verification work; they are not unanswered choices about authority, effects, mutation ordering or failure policy. Full A–E acceptance remains a completion/release gate, not a circular prerequisite to planning.

### 11.2 Guarded document finalization selected

**Decision:** The native writer hands one private request/session/document-bound persistence receipt to a synchronous main-thread finalizer. Fresh guards bind the still-open Script/ScriptEditorBase/CodeEdit, exact post-application current/saved buffer versions and B/R source, current namespace/file identity and successful write/truncate/flush/readback. Missing, failed, mismatched, superseded or uncertain evidence refuses saved tagging.

The engine hook updates only the target's Resource/document mtime from descriptor evidence, Resource edited bookkeeping for the persisted source, and exact CodeEdit saved version, tagging last. A separate read-only inspection exposes actual saved metadata for later independent verification. Report each completed step, including partial bookkeeping after a later failure. Do not rewrite newer state or promise rollback.

**Rationale:** Public CodeEdit tagging omits `ScriptEditorBase::edited_file_data.last_modified_time`, whose stale value caused the observed Undo → Save reconciliation problem. Public extension ABI cannot access that field or its internal tag method. The engine already owns this state; a narrow exposure is smaller than an external synchronization system.

**Alternatives rejected:** Save All, current-tab/path-based save, direct private-layout access, synthetic `resource_saved`, or treating text equality/fsync as complete finalization. Native save callbacks also perform scene/runtime/broader notification work. The selected operation does not claim those effects or live debugger reload.

Further pinned-source inspection distinguishes direct `GDScript::set_source_code` (source plus source-changed bookkeeping) from its property `_set` path (which additionally reloads). Select the supported explicit setter method for guarded R synchronization, not `apply_code`/`update_exports` or property-driven reload. The actual `_validate_script` R assignment/export update is in the **successful non-tool** branch; failed validation is not evidence that R was updated. These are source findings, not new runtime tests. References and exact A inputs/results are in the [native contract](contracts/native-integration.md#6-pinned-implementation-basis).

The pinned ScriptEditor Save path also calls `_auto_format_text` before saving (trailing whitespace, final newlines, indentation). The selected preparation checks both original and intended source against that native save profile without formatting live state, and refuses an affected representation. Recheck the profile; never change preferences or promise exact Save/Undo durability for text the editor would reformat. This is a source-based eligibility decision required by existing exact-source/history durability, not a new formatter capability or runtime result.

### 11.3 Source-attributed validation selected

**Decision:** Expose one exact-source/path call to existing `GDScriptLanguage::validate`, parser and analyzer, on the selected editor main thread. Return `valid`, `invalid` or `unavailable`, exact native-computed input digest, existing request/session/document binding, invocation interval and root/dependency diagnostics. Preflight changed intent before mutation and obtain a fresh post-change result for actual resulting source; unchanged intent needs only its own non-mutating validation/verification pass.

**Rationale:** This is Godot's language semantics and current project context, not a second parser or reload/log proxy. Root/dependency diagnostics can be attributed from existing results. A naked binding is insufficient for FR-020 because cache/dependency/ResourceLoader/Variant paths have distinct effects.

**Selected effect boundary:** Permit ordinary parser/dependency-cache bookkeeping, confined source-backed GDScript dependencies and engine-native static metadata. Add a call-scoped read/effect context at the actual parser/cache/analyzer resolution sites. Supply dependency bytes through the native integration's existing-project directory capability, bind cache reuse to current digests, and refuse before non-GDScript loaders, full reload/instantiation or dynamic project-object property execution. Do not clear caches, invent validation success by skipping a dependency, or call the operation universally pure.

The selected-document guard also covers automatic validation/function-discovery/application callbacks caused by the agent edit, including queued work: no delayed unrestricted R/export update may escape after the explicit call. Preserve normal unrelated and subsequent independent human editing; do not globally disable validation. This small editor adaptation closes a real effect path, not a new generic evaluator/security framework. Source inspection identifies the paths; implementation must prove interception and no source/work mutation.

**Alternatives rejected:** `Script.reload`, a separate validator process/project, a general compiler/LSP API, broad language-extension machinery, universal cache purity, or trusting potentially effectful resource/property resolution. The selected unsupported-effect result is explicit and input-specific; positive source-backed editing remains required, not a refusal-only feature.

### 11.4 Orchestration and ownership selected

Rust retains target/revision intent, shared safety/outcome policy, deadline/cancellation interpretation, application uncertainty and independent verification. The bridge only authenticates/routes/binds bounded commands and stage evidence. GDExtension supplies native text/history operations, explicit source setter and retained-descriptor persistence; the engine owns unbound editor bookkeeping and controlled validation. No policy fork moves into native code.

Changed ordering is acceptance/selection → fresh preparation and proposed-source validation → core authorization → fresh mutation guards → native B application/R synchronization → guarded descriptor persistence → A → actual-source B → independent D/R/B/dirty/saved-state verification. Preflight prevents avoidable invalid-source changes; only the later B result is post-change evidence. Native boundary entry is potentially applied even before a response arrives.

Reuse the supervised worker, adding one parent-before-worker authorization handoff: before dispatch the supervisor records `may_apply`. Without authorization a dead/late worker cannot apply; afterward loss of acknowledgment is uncertain unless a terminal pre-boundary discard is proven. This resolves the already-identified timeout uncertainty without a service, journal or replay cache. Edits use 9.5 seconds plus 0.5 delivery reserve; observation's existing timing is unchanged. No retry or rollback is implied.

Expected revisions reuse prior target/object/file identities, agreeing source digest/length, current CodeEdit version and attributable clean state, freshly revalidated at each relevant boundary. Observation v1 does not contain a saved-version witness: acquire it directly in preparation and use that actual value for later native guards. No generalized Phase 2 revision framework is selected. Private bridge v2 deliberately migrates all strict v1 codecs/capability authentication while public observation-v1 semantics remain unchanged.

### 11.5 Principle XIII assessment of selected mechanisms

| Mechanism / current need | Simplest alternative and why insufficient | Ongoing cost / why justified now |
|---|---|---|
| Small C++17 GDExtension, retained descriptors and standard object ABI | Pure GDScript/public Save paths do not provide demonstrated non-redirectable persistence; an external writer adds synchronization and lifetime handoffs. | Native build/ABI/OS-error/lifetime maintenance; justified by already-demonstrated in-editor binding and one actual write target. No godot-cpp or new Rust dependency is selected. |
| Narrow finalizer, saved-state inspection and target guard | CodeEdit tag lacks document bookkeeping; general save callbacks exceed the one-document/no-unrelated-effects boundary. | Small editor API patch, exact-version review, callback/metadata regression coverage; directly closes the observed ordinary-Save failure and enables independent inspection. |
| Native source validator plus scoped dependency/effect context | Naked validator binding cannot guarantee confined, non-evaluating resolution; subprocess validation loses live context and adds machinery. | Parser/cache/analyzer hook maintenance and effect tests; directly required by attributed parse evidence plus FR-020. Keep normal cache activity; no second language implementation. |
| Paired worker authorization and stage knowledge | Existing read-only worker death is not proof a sent mutation cannot apply later. | A bounded control event/state transition in the existing worker; directly prevents false not-applied claims without another process class. |
| Private bridge v2 and separate edit CLI/schema | Strict v1 capabilities/tuples cannot silently change, and source-in-argv/path input widens exposure. | Coordinated codecs/fixtures/docs migration and bounded stdin parsing; no dual protocol, MCP or generic transport abstraction. |
| Native build/patch provenance and focused live edit fixtures | Stock engine lacks two APIs; observation-only tests cannot establish mutation. | Maintain a small exact-base API patch/build recipe and extend/reuse owned editor fixture facilities. Actual patched-editor behavior and artifact identity are required; no custom distribution/release service or new CI topology. |

Broader transactions, locks/leases, daemons, journals, rollback stores, multi-agent coordination, new approval gates and platform/distribution expansion remain excluded. The API/worker/bridge names describe their durable responsibilities, not feature/task IDs. The helper library is editor integration, not gameplay authority.

### 11.6 Planning completion versus future evidence

The installed planning workflow may now complete its normal Phase 1 artifacts and post-design constitution check. The [plan](plan.md) and [quickstart](quickstart.md) enumerate required implementation/verification work and exact candidate limitations. No approved safety requirement is waived; no known architectural MUST is left unsupported by the proposed contract.

Still **unimplemented/unverified**: engine APIs/effect guards, the completed native writer/finalizer integration, Rust/bridge mutation path, real A–E and all durability/negative cases, privacy/export regression on the installed native artifact, exact patched-build compatibility and timing. A future failure must be fixed or truthfully block implementation acceptance; design completion cannot override it. No test count, binary identity, runtime result, supported-version claim or implementation-task completion is invented here.

This continuation changes no behavioral specification: §10's existing concurrency clarification is sufficient. No new runtime probe, blocked-experiment retry, source implementation, engine patch, task list, analysis command, additional PR or merge is performed. The source/API inspection and documentation/workflow checks are the only new evidence category.

**Contract review corrections:** Read-only native and flow reviews identified two concrete issues before publication: observation v1 supplies only current buffer version, so saved-version evidence now comes from fresh preparation; transitive relative dependencies must resolve against each referring script's directory, not always the root target. Both contracts and their affected descriptions are corrected. Current cached source text is also distinguished from proof of the generation used to produce cached analysis metadata. These are documentation/source-review results, not runtime acceptance.
