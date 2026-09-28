# Phase 0 Research: Safe Open-GDScript Editing

**Date**: 2026-09-27 | **Specification**: [spec.md](spec.md) | **Gate record**: [plan.md](plan.md)

**Historical §11 planning status: State A — concrete patched native integration selected at that stage.** Section 11 records that design; later T001 core and T002 native-validation implementation evidence is recorded in [quickstart §§9–10](quickstart.md#9-t001-core-acceptance-2026-09-27). Native mutation/caller acceptance remains pending. The historical native probe interruption was a tooling-policy limitation, not technical evidence against native integration; it remains an incomplete probe. Section 10's guarded single-editor concurrency boundary is retained. No mutation-support claim follows from this research.

**Current decision (2026-09-28):** [§17](#17-minimal-behavioral-finalization-research-2026-09-28) records **M3 / F-blocking** for the tested minimal **no-handler** route: despite immediate guarded D/R/B equality and a clean native CodeEdit, a later ordinary human Save encounters a false external-change dialog and does not persist newer human text; a corrected Undo → Save → Redo → Save sequence also fails at its first Save. No supported bounded repair was established. This is a required **behavioral** Save/history failure, not a demand for private mtime parity, proof that all handler-free routes are impossible or proof that the whole handler is necessary. [§14](#14-stock-post-persistence-saved-transition-research-2026-09-28)'s P1 is retained **only as a held saved-state candidate**, not an admitted production route; [§16](#16-stock-saved-handler-admission-research-2026-09-28)'s H3 admission hold remains. §13–§16 are historical observations and prior design stages, not overturned experiments. A qualifying **stock explicit exact-source validator** and its dependency/effect confinement remain independently unresolved. T001/T002 remain complete; T003–T005 pending, T003 unstarted/on hold. Neither candidate failures nor checked research facts are Feature 002 A–E acceptance.

**Prior decision labels (historical):** §13's F3 ordinary-Save fallback/unrelated-editor failures remain; §14's bounded P1 native-handler saved-state mechanics superseded F3 **only for that saved-state candidate**, while §15's V2 design question and §16's H3 investigation retain their evidence and limitations. §17 adds a different no-handler experiment and changes the current decision, not those recorded outcomes.

**Planning continuation boundary (historical):** Define the two narrow native capability contracts and complete the installed `/speckit.plan` workflow, including its ordinary design artifacts and constitutional review. That continuation was source/API inspection and design, not a new runtime experiment. No filter bypass, equivalent rephrased retry, broad API probing, task generation or product implementation was performed in that pass. Plans specify how later implementation will establish guarantees; a finished binary and A–E proof are not prerequisites for writing the plan.

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

The “complete native saved-state transition” formulation here describes the **then-tested CodeEdit-only failure**, not a standing requirement to reproduce native callback history or numerically equalize private bookkeeping. §17 independently finds a real later-Save failure for a guarded direct tag across a naturally changed timestamp second; require the observable ordinary Save/history behavior, not a particular private-field mechanism.

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

## 12. T002 implementation evidence (2026-09-28)

The separately authorized T002 task implements only Primitive B: the existing
Godot validator, a call-local dependency/effect context, generated-public-ABI
C++17 dispatch, and a confined native reader. No private-layout cast, private
binary symbol, external validator, writer or finalizer was introduced.

The exact `4.7.2.stable.custom_build.ed1daf0bf` acceptance build passed 54
native GUI/lifecycle/export cases. The existing stock editor passed all 194
observation cases with the native artifact present; an unbuilt-checkout smoke
proved complete D/R/B observation with no installed native API.
[Quickstart §10](quickstart.md#10-t002-native-validation-acceptance-2026-09-28)
records actual commands, build/ABI/artifact hashes, source/effect boundary
witnesses, checks and the implementation-shape/constitutional review.
[Native build instructions](../../godot-addon/native/README.md) record the
reproduction path and dependency/license review.

Runtime review reproduced and corrected scripted diagnostic stringification,
false-valid failed-autoload analysis and cyclic parser-reference retention.
Real hook-only export exposed a stale generated extension-registry entry;
`.gdignore` and explicit plugin loading fixed it without changing the exporter
or other extensions. The plan's complexity record covers the actual effect
fixture and loading decisions.

These new implementation results do not amend historical failed/incomplete
probes or certify the full editing family. Native mutation/finalization,
edit-generated callback guards, caller integration and A–E/durability remain
T003–T005 work. Only the recorded tests-enabled macOS arm64 candidate is verified.

## 13. Stock target-save finalization research (2026-09-28)

### 13.1 Decision, scope, provenance and evidence classes

**F3 — no qualifying route selected among the audited stock finalization routes.** The
positive result is important: a normal *single-script* Save in official stock Godot
performs native Resource **and** script-document saved-state transitions after a
successful exact-object descriptor-backed custom saver. The negative result is
different: a truthful saver error does **not** veto later path-based built-in savers,
and the normal Save applies changes to *all* script editors before dispatch.
Replacement and unrelated-buffer cases below actually violate the feature's
confinement/no-unrelated-work guarantees. This is not proof that every conceivable
stock architecture is impossible, nor approval of a patched/custom editor.
Retain the product goal of official stock Godot + addon + bundled standard
GDExtension; no engine patches, product source, bridge, caller, packages, parser,
functional-specification or task-count/dependency/acceptance changes result
from this research. Task wording/status is updated to record the design hold.

Evidence classes are **demonstrated positive stock runtime**, **demonstrated
negative stock runtime**, **stock public API/source finding**, **inference needing
runtime proof**, **unresolved technical question**, and **tooling/environment
limitation**. A deliberate non-success in a finished GUI case is a negative
behavioral result, not a failed run. These **19** final disposable GUI cases
(plus one visual replay) are *not* the A–E release tests, implementation
acceptance, or a new support claim.

Pinned official runtime: `4.7.2.stable.official.ed1daf0bf` / source commit
`ed1daf0bf001b61586d9930840f2f1394092c079`; editor SHA-256
`c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf`.
Owned research branch base `671ee50b00a81b6be24372308f4a5dd1925db495`;
macOS 26.6.2 build 25G83, arm64, Apple clang 21.0.0, SDK 27.0,
standard GUI display and Apple M2 Compatibility renderer. The generated
official `gdextension_interface.h`, interface JSON and extension API hashes
were respectively `640b48188708ba0016f8d7ace9e0e1d3279a41fa1226c59ff3193b15538bd254`,
`7d8c0a039d9743eb8ebf88681ae0c641d8d3aa5ffca11081745a84da803e09a1`,
and `d0e4c08c03b165156dabe6bfb6a906baf0069189f62035341230a246c86d6986`.
The C++17 public-ABI probe library hash was
`c748b4ea3100832cb2e2050c641a2525b784f5c5e28d190f96133d4d760e1ca8`;
it used opaque Variant/Object dispatch and platform
`openat`/`O_NOFOLLOW`/`pwrite`/`ftruncate`/`fsync`/readback, not internal
engine headers, private layouts or symbols. Its scripted `@tool`
`ResourceFormatSaver` called that writer and returned **real** status, never
fake `OK`. The experiment compiled the native library using official
generated headers with `-std=c++17 -Wall -Wextra -Werror -fPIC -dynamiclib
-pthread`. It did not install a product dependency.

**Local, uncommitted research artifacts**, not repo acceptance fixtures:
`/var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-target-save-research-feoq538g/`.
`evidence/final-cases.json` SHA-256
`a66e07ddbce458574c99f71ad178547061384f8eba1111b34bd1dc066ec72f31`
contains complete final case records; `evidence/verified-claims.json` records
50 checked independent facts, `evidence/public-inventory.json` the bound
method inventory, and `evidence/provenance.json` the hashes. The separate
`evidence/visual-replay.json` SHA-256
`bf3ee093cb60ea0a8818e39e99bac0a7f08b72aa4e1b16fcffa25629b24dad0e`
and inspected `evidence/stock-saved-final.png` SHA-256
`0e91564abda55b3d63cb6f2544cedefbc13b6e7004266fd097830006c9c24bea`
show the subject's `return 23` clean tab and the other script still dirty.
These **local** evidence paths are not stable publication links; pinned public
source links below are reproducible independently. During the experiment,
the disposable `probe_native.cpp`, `run_probe.py`, `probe-notes.txt` and
owned fixture were used to build against pinned generated official headers
and execute
`python3 run_probe.py --godot /Users/petertam/.local/bin/godot --case all`;
the separate `--case happy_custom` visual replay was also executed.
These temporary probe sources, library and source checkout are removed
after recording the documentation; **retained** raw evidence/provenance/
screenshot and the API sequence, case recipes and hashes here describe
what another independent reproduction would have to rebuild and rerun.
Each case created a new owner-private project, exited its actual GUI editor
and cleaned only its sentinel-checked owned workspace; the outside-project
sentinel for the symlink case was another *owned* sibling. All 19
completed with process exit 0; the separate visual replay also exited 0.
The expected built-in
`Cannot save GDScript` error occurred only in the deliberately unwritable
case. Independent Python component-wise no-follow descriptor reads/stat
supplied D, not a Godot report of its own write. Real loaded Script getters
supplied R; real CodeEdit getters, saved/current versions, native undo/redo,
dirty paths and editor signals supplied B/editor witnesses. No synthetic
`resource_saved`/menu signal, Save All, manual saved tagging, private
shortcut invocation, preferences override, or global typing block was used.

**Excluded evidence:** Early fixture instrumentation missed Dictionary
overwrite on merge, a human injection, scripted-saver lifetime and a stale
attached-state flag; those intermediate apparent passes are **not** counted.
After corrections the entire final matrix was rerun. The prior tag-only
ordinary-Save failure in §9 was *not* rerun. The earlier policy-interrupted
experiment remains a tooling limitation, not a runtime negative or a
workaround/bypass; it contributes no result to these 19 cases.

### 13.2 A — actual ordered Save path; D/E — complete saved state

**Stock public API/source finding:** On macOS the File Save shortcut is
`script_editor/save`, Cmd+Alt+S (not Cmd+S / Save All). The fixture delivered
real pressed/released `InputEventKey`s through public `Viewport.push_input`;
the synchronous return followed actual native receipts and saved callbacks.
For an ordinary standalone, nonimported GDScript resource:

1. [`ScriptEditor::save_current_script`](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp#L2420-L2447)
   captures the *current* document, calls
   [`_test_script_times_on_disk`](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp#L803-L850)
   (an all-open-document mtime reconciliation, **not** a conditional
   object/revision write), autoformats the target, clears its script docs
   and invokes `EditorNode::save_resource(resource)`. It does **not**
   directly call GDScript `apply_code` here. Built-in/imported paths can
   instead save a scene or ask for Save As and are outside this qualifying
   route. [Shortcut binding](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp#L3954-L3965).
2. [`EditorNode::save_resource_in_path`](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/editor_node.cpp#L1736-L1750)
   first calls `editor_data.apply_changes_in_editors()`. The
   [`ScriptEditorPlugin::apply_changes`](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp#L4381-L4383)
   bridge reaches [`ScriptEditor::apply_scripts`](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp#L2512-L2523):
   every open script editor receives `insert_final_newline` when it is a
   `TextEditorBase`, then `apply_code`. GDScript's
   [`ScriptTextEditor::apply_code`](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_text_editor.cpp#L227-L238)
   transfers B into Script R and updates exports/cache; this step is not
   isolated to the selected target.
3. [`ResourceSaver::save`](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/core/io/resource_saver.cpp#L102-L155)
   tries each recognizing saver/path until one returns `OK`; its
   [built-in GDScript saver](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript_resource_format.cpp#L149-L182)
   opens the path for writing R. **On OK**, ResourceSaver clears the
   inherited Object edited flag, takes a Resource mtime from the *path*
   and runs the global save callback. That callback
   [`EditorNode::_resource_saved`](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/editor_node.cpp#L7833-L7844)
   concerns filesystem/folding; within save recursion it is suppressed/
   deferred, not an emitted `resource_saved` document transition.
4. After OK,
   [`EditorNode`](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/editor_node.cpp#L1752-L1775)
   handles Resource path/cache and fs/folding, emits actual `resource_saved`
   and notifies plugins. Error returns at lines 1752–1760 *after*
   all-editor application, without this signal/tag. The
   [`ScriptEditor` handler](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp#L723-L742)
   matches the actual Resource pointer; its
   [`TextEditorBase::tag_saved_version`](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_base.cpp#L566-L573)
   tags CodeEdit saved version **and** calls
   [`ScriptEditorBase::tag_saved_version`](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_base.cpp#L91-L93)
   for document mtime; the handler also updates names/built-ins and
   schedules live-script reload. The target's docs update on return to
   `save_current_script`. Tagging does not clear undo history. These
   scene/export/runtime branches are *source findings*, not proof of
   gameplay/export behavior.

**Metadata correction:** Stock public
[`EditorInterface::is_object_edited` and `set_object_edited`](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/editor_interface.cpp#L714-L721)
are actually [ClassDB-bound](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/editor_interface.cpp#L917-L924);
they read/write the Resource's inherited Object edited flag. The §9
blanket suggestion that it is unobservable from stock is superseded.
The experiment **read** its false value after successful saves, not
simulated success by writing the flag.

| State / transition | Stock supported access in audited surface | Observed/source distinction |
|---|---|---|
| D contents / identity | Independent no-follow file descriptor reads/stat; native writer retains validated descriptors | Actual separate witness and actual receipt; a path read alone is not preventive. |
| R source and Resource edited flag | `Script.source_code`; public `EditorInterface.is_object_edited`/`set_object_edited` | Read-only false after normal Saves. Direct ResourceSaver also updates the Resource's edited flag on OK. |
| Resource last-modified time | No public direct read/write established for this internal Resource field | `ResourceSaver::save` updates it from path mtime on OK; **no private numeric value claimed**. |
| B, current/saved versions, dirty/history | Public CodeEdit/TextEdit getters, undo/redo and `tag_saved_version`; `ScriptEditor.get_unsaved_files()` | Native Save's resource callback tags the specific document; direct ResourceSaver alone leaves it dirty. A manual CodeEdit tag is not full saved-state evidence. |
| ScriptEditorBase document last-modified time | No supported direct read/write established on stock | Native handler calls the internal timestamp tag; behavioral witness below, **not** a direct numeric read. |
| Native save event and UI | Real `EditorNode.resource_saved`, normal callbacks, native tab/dirty status and close/reopen | Direct ResourceSaver has no new native document event/tag; spoofing the signal is not a substitute. |

**Demonstrated positive stock runtime:** Both `happy_builtin` and
`happy_custom` saved a prior native 17→18 edit (version 2→4), then an
agent 18→23 edit in one native complex operation (version 6). Ordinary
Save produced D=R=B=23, current=saved=6 and a clean target; Undo to 18
was version 4/6 dirty, then ordinary Save made D=R=B=18, version 4/4.
Redo 23 remained available and saved clean at 6/6; earlier Undo→Undo
still reached 17 and Redo→Redo returned to 23. Close/reopen returned
clean D=R=B=23 with new editor/buffer identities. Each custom successful
Save had one actual 44-byte descriptor write, flush/readback receipt
and retained inode. The seed and agent saves crossed an mtime second
(disk seconds 1790588946→1790588947); later Undo→Save needed **no**
external-modification reconciliation. This is a strong behavioral
saved-state witness across timestamp change, **not** an observation of
private Resource/document mtime values. The other initially dirty
newline-terminated document kept D, its dirty B and exercised native
Undo/Redo; its R **may** have been synchronized by the selection or
all-editor apply, so this happy case alone is not a no-side-effects
claim. The inspected screenshot corroborates the final visible dirty
markers, not all intermediate state.

**Demonstrated negative stock runtime:** `direct` used the *same* real
descriptor saver with `Script.set_source_code(B)` then direct
`ResourceSaver.save(target,path)`; D=R=B=23, but current/saved=6/4,
dirty and **no new** native `resource_saved` signal after the seed.
Resource bookkeeping is not the missing document transition. §11's
proposed special finalizer would be unnecessary for *successful normal
Save* bookkeeping; its independent guard/inspection aims do not
magically become stock public APIs.

### 13.3 B/C — public candidates and exact-document selection

The seven questions for each public candidate are **exact target,
focus dependence, unrelated saves/work, native history, full saved
bookkeeping, fresh guards, and attribution** (including what the API
does *not* establish):

| Candidate | Target | Focus | Unrelated saves/work | History | Full bookkeeping | Fresh guards | Attribution / disposition |
|---|---|---|---|---|---|---|---|
| `ScriptEditor.save_all_scripts()` | All scripts | No exact target | Actually saved unrelated dirty D (§4) | Broad native path | Broad callbacks | No target-only veto | Reject; earlier runtime finding, **not** repeated among 19 cases. |
| Public keyboard/menu Save | Current script only | Must select target | All-editor apply can change other B/history; fallthrough may write wrong D | Native history preserved on happy path | Yes **on OK** | Can check before/after selection, not veto dispatch | Saved event matches Resource, but pathname fallback need not match validated object; unsafe. |
| `EditorInterface.edit_script(held_script)` then Save | Held Script requested; postcheck identity | Changes current tab/focus | Same all-editor/fallback defects | Native on successful Save | Yes **on OK** | Open-membership/ID/epoch and postselection checks | `edit_script` can open/fallback by path; not proof of eligibility or safe write. |
| Direct `ResourceSaver.save(held_resource,path)` | Exact Resource argument | None | Other tabs not automatically saved; path fallback remains | Does not clear undo | **No** document tag/event; Resource-only | Can guard D/R/B before call; cannot stop later saver | Real receipt binds descriptor write, not the missing editor transition. |
| Front custom `ResourceFormatSaver` inside normal Save | Recognize object and path | Normal Save needs current tab | Other B may change; non-OK falls through | Native on OK | Yes **on OK** | Guard inside saver, but no error veto | Descriptor receipt on success; built-in path write after failure defeats exact-object attribution. |
| CodeEdit tag / Object edited setter | Existing buffer/Object | None | No disk save | Tag does not clear undo | **No** document mtime/native save | Can check current version | Cannot turn indicators into a real attributed save. |
| Internal EditorNode `save_resource(resource)` / ScriptEditor `save_current_script()` | Object argument / current tab respectively | Current tab for latter | Internal all-editor apply still occurs | Internal native behavior | Internal success path | Not supported stock extension entrypoints | `ScriptEditorBase.get_edited_resource`, `apply_code`, document tag likewise unbound on **stock** (T002 patched getter does not count). |
| EditorCommandPalette / TabContainer | Palette no target command; tab child possible | Tab selection | Selection does not save | None itself | None itself | Exact-child tab route is source-only | No public `execute_command`; untested TabContainer composition cannot cancel fallback/all-editor work. |

The stock ClassDB inventory includes ScriptEditor, ScriptEditorBase,
EditorInterface, ResourceSaver/ResourceFormatSaver and CodeEdit, but
**not** EditorNode as an exported ClassDB class. ResourceSaver exposes
removal by a *known* `ResourceFormatSaver` reference, not enumeration
of the built-in saver instance or an exclusive/stop-dispatch API.
Adding more front savers merely adds more candidates before the
built-in fallback. Its
[`remove_custom_savers`](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/core/io/resource_saver.cpp#L272-L282)
removes instances with a ScriptInstance during
[`EditorFileSystem` class refresh](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/file_system/editor_file_system.cpp#L2204-L2205).
The corrected fixture registered its **scripted** saver immediately
before synchronous Save, checked presence by a unique public recognized
extension marker and removed the exact instance only if still present.
A native ResourceFormatSaver has no ScriptInstance: do **not** infer
that native extension registration inevitably suffers that fixture
lifetime issue. Neither variant changes the source-level fallback rule.

**Demonstrated positive bounded selection:** Stable equal-length
`get_open_scripts` / `get_open_script_editors` arrays in the all-GDScript
fixture identified one unique exact path and held the Script,
ScriptEditorBase, CodeEdit and instance IDs. With the target initially
nonselected, `EditorInterface.edit_script(held_script)` selected it;
checking current Script/editor identities immediately afterward
prevented assuming focus itself proves identity. After Save, a guarded
selection epoch restored the previously selected *other* Script/focus.
In `callback_retarget`, a real `resource_saved` callback selected other,
changed the epoch, and the fixture **skipped** its restoration rather
than overriding that newer selection. Only subject D was saved. The
[`edit_script`/tab-switch source](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp#L2200-L2285)
has Resource-pointer **or path** fallback and may open a missing
document; [`_go_to_tab`](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp#L360-L431)
applies departing unsaved script, changes navigation/focus and notifies/
validates. A future candidate must refuse a missing/duplicate/replaced
open association before calling `edit_script`, then recheck after it;
must not open a document to manufacture eligibility. Actual target
Save was synchronous through `push_input`, with no ordinary OS typing
interleaving in that nonyielding main-thread span; reentrant callbacks
and alternate UI layouts still require proof. Selection does **not**
repair the persistence/all-document problems.

### 13.4 F/G — full final case matrix and safety boundaries

All rows refer to the **final** 19 independent owned GUI cases. `D`
is external disk text, `R` the held Script, `B` the real CodeEdit; numeric
17/18/23/333/444/555 denote the synthetic `return` value, not test
counts. Initial scripted source and independent D differed from user
files: only disposable synthetic scripts were touched.

| Case(s) | Demonstrated stock runtime and classification |
|---|---|
| `happy_builtin`, `happy_custom` | **Positive:** normal target Save completes the 17→18→23, Undo/Save/Redo/Save/close/reopen sequence above with real saved signals. Descriptor receipt on **every** custom Save; other dirty document with final newline remains dirty, its D and history preserved. |
| `direct` | **Negative:** descriptor ResourceSaver succeeds, D=R=B=23, but target remains version 6/4 dirty and lacks a new editor save event. |
| `read_only` | **Negative:** native pin is deliberately `O_RDONLY`; real `pwrite` EBADF 9, zero bytes and truthful `ERR_CANT_CREATE`; built-in fallback writes 23, tags 6 clean and emits `resource_saved`. Saver failure is not a veto. |
| `unwritable` | **Positive failure semantics:** owned target mode 0400/parent 0500 plus read-only native pin; EBADF followed by built-in `Cannot save GDScript`, D=18, R/B=23, version 6/4 dirty, no new saved event. Application already occurred: **applied/unverified**, not not-applied/rollback. |
| `identity_before`, `barrier_leaf`, `barrier_parent` | **Negative:** before-call replacement can be caught by a fresh outer guard, but the former stock save accepted same-second mtime; latter two replace owned leaf/parent *after* native final prewrite checks. Native reports detached/0 bytes/non-OK; built-in follows pathname, overwrites replacement 444→23, tags target clean. The after-check/built-in-open gap remains. |
| `barrier_parent_symlink` | **Negative, decisive confinement witness:** after native barrier, owned project `scripts` parent becomes a link to an **owned outside-project sibling** with D=444. Native descriptor writer refuses detached/0 bytes; independent project no-follow D read refuses (errno 62), yet the independent outside sentinel is 23 after built-in fallback. No unowned data touched; postcheck cannot retract the wrong-object write. |
| `human_during_saver` | **Negative:** external same-inode D changes to 444 at barrier; native detects `content_changed_or_unreadable` (errno 35), writes zero, returns failure; built-in still overwrites 444→23. This proves failure dispatch ignores detected invalidation, **not** an obligation to exclude every arbitrary transient same-inode writer. |
| `preapply_stale`, `unchanged_no_history` | **Positive bounded guards:** preapply human B=444/version 6 after observing 18 causes refusal before AI source/history; unchanged source stays version 4 and Undo reaches 17 / Redo 18, with no extra save/history operation. These are *fixture guards*, not T003. |
| `human_before_selection`, `human_after_selection`, `human_before_save` | **Positive bounded refusal:** after AI B application, a human B=444/version 8 at each named point makes guards refuse further selection/save; D remains 18 and old AI source is not reasserted. Since application already happened, outcome is **applied/unverified**, never safe whole-attempt not-applied. |
| `human_in_saver` | **Negative bounded reentrancy:** scripted saver injects human CodeEdit B=555/version 8 after all-editor apply set R=23, before descriptor save. Writer really persists R=23; native callback tags *current* version 8 clean although B=555 ≠ D/R=23. No human text overwritten, but false clean marker. This fixture has no last-B check; a no-callout native saver could avoid this particular injection point. Error-return from a guard still meets the fallback defect. |
| `human_on_saved` | **Positive bounded aftermath:** callback inserts human B=333 after native tag; current/saved=8/6 dirty, D=23, later R/B=333 through normal selection sync. Newer text retained; only **applied/unverified** may be reported. |
| `other_no_final_newline` | **Negative unrelated work:** explicitly dirty *after seed*, unrelated B sentinel has no final newline, current/saved=5/2. Saving target inserts newline into unrelated B and changes version 5→6; unrelated D unchanged and dirty persists. Its Undo removes *only* that new newline; Redo restores it. “Not saved to disk” is insufficient: unrelated B and history changed. |
| `callback_retarget` | **Positive bounded restoration:** Save target 23, callback selects other, epoch changes, restore=false and current=other; other D unsaved. Not proof against arbitrary callback ordering. |

**Conditional selection safety:** a production candidate must establish
one preexisting open target, uniquely match resource/path/document/buffer
IDs and session/project, retain those references, refuse duplicates or
replacement, and compare epoch/old focus before any restoration. Recheck
the current Script/editor pair *after* selection and at subsequent
boundaries. The successful fixture used a stable all-GDScript array; it
does not establish arbitrary mixed-tab mapping or make `edit_script`
safe simply because it accepts a Script.

**Descriptor requirement and failed dispatch:** successful guarded
`openat`/no-follow attached descriptor write, truncation, flush and
readback binds actual D to the validated target; a descriptor held only
for *observation* does not bind a later built-in pathname write. On
detached/permission/content guard failure the extension must return
truthful non-OK, yet
[`ResourceSaver` continues to later savers](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/core/io/resource_saver.cpp#L110-L154);
the built-in may write a different file (even outside the project)
and trigger a misleading clean tag. Returning fake `OK` to suppress
fallback would falsely complete saved state without successful D
persistence, and post-write D/R/B verification cannot undo a wrong
write. Preflight identity/digest guards reject already-observed stale
states, not a replacement between final guard and fallback open.
Adding a later check, saver, edited-flag/tag tweak, or callback observer
does not supply a fail-closed exclusive dispatch. This is the precise
known mandatory gap; arbitrary same-inode atomic serialization was
not demanded by the scoped §10 concurrency clarification.

### 13.5 H — architecture comparison, Principle XIII, disposition

| Architecture | Positive finding / cost | Mandatory gap in the audited route |
|---|---|---|
| Direct `ResourceSaver` + public CodeEdit/metadata composition | Exact held Resource and descriptor writer can save D without focus; public edited flag and CodeEdit versions exist; lower editor coupling. | Real direct Save has no native document event/timestamp/tag. Manual tag cannot claim complete ordinary saved state, and normal direct dispatch still has path fallback. |
| Stock current-tab Save + selection + front custom descriptor saver | **Actually** completes native bookkeeping/history after saver OK; can select/conditionally restore noncurrent target. Avoids custom editor distribution, patched-build CI and engine API maintenance. | Real non-OK falls through to built-in path saver, including an outside-project write. Earlier all-script apply changes unrelated B/history. No supported exclusive-veto or target-only apply primitive established in this route. |
| §11 narrow patched/editor API finalizer + existing native writer | Historical planning proposal might expose precise guard/inspection/validation control. | Outside official-stock product goal. No automatic authorization, claimed implementation, upstream acceptance or evidence it solves the observed full route; do not silently switch T003 to it. |
| Different supported stock composition | May retain stock-distribution advantage if someone identifies concrete fail-closed one-target primitives and exercises them. | **Unresolved technical question**, not a proved impossibility theorem. Requires its own complete runtime/safety evidence before selection. |

**Principle XIII total product cost:** a standard bundled GDExtension
adds C++17 build/ABI and platform compatibility, saver-instance lifetime,
retained-descriptor error/identity management, selection/reentrant-state
machine and regression burden. These are potentially proportionate
because successful stock normal Save reuses real native history and
bookkeeping, avoids editor replacement, patched binary installation/
trust, exact-version custom builds and distribution, team setup,
custom-editor CI and update burden. Conversely the simpler/cheaper
stock path cannot be selected by accepting an observed outside-project
write or unrelated buffer/history change. More extension-side checks
improve refusal/attribution but cannot cancel a builtin write after a
truthful error or undo the all-script apply before the saver runs.
Private ABI/layout/symbol access or a fake-OK callback is not a
cost-saving workaround. Do not add a generic journal, daemon, global
typing block, arbitrary-writer lease or new infrastructure to mask this
specific missing composition. The constraints of Constitution I–IV,
V/VI/X/XII and the same unchanged feature specification still govern;
withholding success preserves them, rather than weakening requirements.

**Inference needing runtime proof:** a TabContainer identity-selection
route, a native-only saver removing scripted class-refresh/reentrancy
points, and arbitrary callback/UI ordering. None by itself resolves
the **demonstrated** dispatch fallback or all-editor apply. **Unresolved
technical question:** whether *another* supported stock, fail-closed,
one-target save composition exists. Source validation, parser/dependency
effect confinement and post-change attribution remain separate
unresolved work and were not probed here. **Tooling/environment
limitations:** only pinned official 4.7.2 on macOS arm64 was exercised;
no other version/platform support, live game/export behavior, private
mtime numeric values, complete callback-topology proof or A–E/full
mutation acceptance is claimed. T001/T002 stay complete as recorded
(T002's patched-editor evidence remains truthful historical evidence),
T003 stays pending; task count, dependencies and acceptance remain
unchanged while task wording/status records the design hold. The
design decision is explicitly reopened, and current F3 withholds
implementation authorization until a safe stock design is established
and reviewed.

## 14. Stock post-persistence saved transition research (2026-09-28)

### 14.1 A — decision, question and evidence provenance

**P1 — stock post-persistence saved transition established, conditionally.**
The question is whether an already-confined, exact-object descriptor write
can be followed by the *real* stock script-document saved transition,
without normal Save's all-editor apply or ResourceSaver fallback. For the
tested standalone GDScript lifecycle it can: invoke only the **existing
native ScriptEditor receiver Callable** obtained by public signal-connection
introspection, after persistence and fresh guards. This is not a solution
to independent stock validation/dependency/effect confinement, T003
authorization, or A–E acceptance. §13 F3 remains the negative result
for its *normal Save/custom saver* routes; prior experiments remain
historical. No patched editor, custom saver, private ABI, new public
finalizer API or product implementation is selected here.

Evidence classes are **stock public API/source finding** (pinned sources),
**demonstrated positive stock runtime** (actual owned GUI/independent
witnesses), **demonstrated negative stock runtime** (completed adversarial
cases), **inference needing further proof** (other versions/topologies)
and **unresolved technical question** (independent validator/effects).
The existing public ingredients below are a design, not a final wire
schema or product API.

Official stock `4.7.2.stable.official.ed1daf0bf`, full source hash
`ed1daf0bf001b61586d9930840f2f1394092c079`, binary SHA-256
`c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf`;
macOS 26.6.2 (25G83), arm64, Apple clang 21.0.0, SDK 27.0.
Generated official header/API hashes are recorded in §13.1; C++17
standard-public-C-ABI probe library SHA-256
`28e0a60a3f18656f12c2cfeed78f2f85a120b15b7e750d5706073daf9259a9c6`.
Local **uncommitted, non-publication-stable** workspace:
`/var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-post-persist-research-vds7zbtv/`.
Its `evidence/gui-20260928-120720-b1b1f7/` final matrix has **18
completed GUI cases, zero harness errors**. `evidence/final-cases.json`
SHA-256 `e31053a99ef48a37c3446f7116ad8cb278555e18d2f7b9422cf63549e30921f1`
aggregates full raw records; `evidence/verified-claims.json` SHA-256
`2727ba56fbe284918dcfc3ae29d8c1c08403e6241469c4a462925a602f7bee69`
holds **95 parent-verified independent facts**. Retained aggregate
`evidence/visual-replay.json` SHA-256
`54c015546fd50399309a22cdb8eb89a6ef6d45071a4993b8523e572c2f5207c1`;
inspected `evidence/gui-20260928-120855-be6e72/happy_saved_handler_callable/stock-post-persist.png`
SHA-256 `c6ad421b9b1911923ec13b04ca6c93580bec53f7f6406ab4c1dbba71d1ab3963`.
`evidence/provenance.json` records these hashes.
Research reproduction command (historical, **not currently executable from retained artifacts**): the owned local probe once built with
`clang++ -std=c++17 -Wall -Wextra -Werror -fPIC -dynamiclib -pthread
-Igenerated probe_native.cpp -o libprobe.dylib`, then ran
`python3 run_probe.py --godot /Users/petertam/.local/bin/godot --case all`;
separately `--case happy_saved_handler_callable --visual-pause 1`.
Cleanup removed owned probe sources, binaries, generated headers and disposable
projects; only raw evidence and its provenance were retained. Do not mistake
the recorded command for source availability or a fresh reproducible run.
Earlier campaigns with a Dictionary.merge bug, fixture sequencing,
parse-variable collision, GUI focus readiness and missing Python constant
are excluded; the **entire corrected matrix** was rerun before cleanup.
No §13 normal-Save result was silently reclassified.

### 14.2 B — source/API mechanism and exact guarded order

Obtain canonical ScriptEditor through public
[`EditorInterface.get_script_editor()`](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/editor_interface.cpp#L418-L420)
([binding](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/editor_interface.cpp#L868)).
Public [`Object.get_incoming_connections()`](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/core/object/object.cpp#L1411-L1421)
returns signal/Callable edges. Find the **unique existing** incoming
`resource_saved` Signal whose Callable receiver is that canonical
ScriptEditor. Verify `Signal.get_object()/get_name()`,
`Callable.get_object()/is_valid()`, receiver instance IDs, the incoming
edge and emitter's `get_signal_connection_list`; retain actual Signal
and Callable. Observed emitter class `EditorNode`, flags `0` and
reflected method metadata `ScriptEditor::_res_saved_callback` are
*recorded only*: never look up/invoke by private method name. Call the
actual public returned
[`Callable.call`](https://docs.godotengine.org/en/4.7/classes/class_callable.html#class-callable-method-call)
with the **held exact Script**. No emission, private ClassDB binding,
editor-tree lookup, binary symbol/layout access or claim of a stable
dedicated finalizer API. Refuse absent/ambiguous/changed topology.

Selected sequence:

1. During preparation, capture baseline clean CodeEdit current **6** and
   saved **6** (fixture example), source, still-open unique Script/editor/
   CodeEdit mapping and IDs, session/project and path. Recheck the
   **baseline current** version and saved version, source and attachment
   before the one native CodeEdit complex operation preserving history.
   After that operation, freeze its **actual post-edit current 8** together
   with the **original preparation-time saved 6**; guard this 8/6 pair
   through R synchronization, persistence and every pre-tag check. Explicitly
   `Script.set_source_code` on the same held Script and read back R. Do
   **not** use preparation's 6/6 as the post-edit guard; the agent's own
   native edit changes current version.
2. Write through the retained validated no-follow descriptor with
   `pwrite`/`ftruncate`/`fsync`/readback and current no-follow namespace
   identity check. Private attempt-local receipt binds successful bytes,
   inode/attachment and expected source. No ResourceSaver dispatch,
   registered `ResourceFormatSaver`, built-in fallback, focus selection
   or second source write in finalization.
3. After receipt, freshly guard session/path, still-open IDs/mapping,
   namespace, frozen post-edit B current **8** and preparation-time saved
   **6**, B/R source and exact Script. A later human current **10** or
   equal-text current **12** invalidates this guard. Use public
   [`EditorInterface.set_object_edited(Script, false)`](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/editor_interface.cpp#L714-L721)
   ([bound here](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/editor_interface.cpp#L917-L924)).
   Record partial bookkeeping if the following fresh guard fails;
   then call the retained actual native Callable on held Script.
   **No yield/pump/user callout between final guard and native tag.**
   Setter has no callout; in pinned native handler the CodeEdit current
   version is saved first and document mtime is then read from actual
   path, with no reentrant user callout before/between these tags.
   This is stock ordering, not historical patched tag-last/descriptor-
   fed bookkeeping.
4. Independently recheck D/R/B, dirty, current/saved versions,
   identity/namespace after callback; the clean fixture expects observed
   current **8** and saved **8**. Eventual exact-source validation
   and final independent verification remain separate design work.
   Neither receipt nor response masquerades as independent observation.

[`ScriptEditor`'s handler](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp#L723-L742)
matches actual Resource pointer.
[`TextEditorBase::tag_saved_version`](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_base.cpp#L566-L573)
combines [`TextEdit` version-only tag](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/scene/gui/text_edit.cpp#L4890-L4900)
with [`ScriptEditorBase` path-mtime tag](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_base.cpp#L91-L93).
The handler's
[`built-in scan/name refresh/live reload`](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp#L749-L803)
can tag/reload other documents sharing a built-in parent path:
require standalone target and refuse colliding built-in association.
The name refresh is **not** UI-only: its member-overview tail
transitively calls current-editor `get_functions()` and validates current
B/path; §16 corrects the earlier bounded-bookkeeping interpretation with
source and runtime distinctions. The conditional sort/navigation branch
can also apply an unsaved tab, though that branch was not observed in the
normal settled case. Require exact cached held Script context; no
ResourceLoader replacement or running-game safety is established.

### 14.3 C/D — effect decomposition and alternatives

| Effect after external target write | Descriptor already does it? | Necessity, public access and behavioral witness | Unrelated effects/bound |
|---|---|---|---|
| Target D bytes/size/flush/attachment/readback | Yes: one retained-fd write/receipt; independent component-relative `O_NOFOLLOW` D/stat witness | **Required** exact source/namespace; successful pinned attempt had one 35-byte write. | Finalizer makes no second write; no atomic exclusion of arbitrary concurrent external writers claimed. |
| Held Script R source/path/cache identity | Explicit `set_source_code`/getter before D; descriptor does not edit R | **Required** held source and unchanged path/cache, no `take_over_path` or reload. | No all-editor apply. |
| Resource inherited Object edited flag | No: ResourceSaver bypassed | **Required** public `set_object_edited(false)`/`is_object_edited`; fixture seeded true only if setter left it false, to witness true→false. | Target-only setter, no callback; later refusal is partial. |
| Private Resource path mtime | No | No supported numeric stock getter/setter established; **not independently required** by observed standalone Script behavior. Future stock Save/reopen across natural mtime-second change behaved coherently; no numerical synchronization asserted. | Never inspect/write guessed private metadata. |
| CodeEdit saved version, dirty marker, native history | No | **For P1:** matching native receiver tags current; public versions/dirty/Undo/Redo witness; bare CodeEdit tag did not establish later Save in §17. | Tag writes no source/history step; frozen versions protect newer human B. |
| ScriptEditorBase document path mtime | No | **P1 mechanism:** matching native receiver reads path mtime; later ordinary Save/reopen behavior, not private numeric equality, is required. §17 independently shows a natural timestamp change + bare tag triggers a false later Save prompt. | [`_test_script_times_on_disk`](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp#L803-L850) compares document, not Resource, mtime; this does not require whole handler. |
| Tab/list UI, callbacks | No | Native receiver refreshes names/list and invokes live-script reload; visually observed clean target/dirty other. | Native validation and conditional deferred debugger reload remain independent effect blockers. |

**Historical P1 mechanism, not a general requirement:** §17's no-handler experiment independently demonstrates that document timestamp freshness matters for a later Save; it does **not** make this private value, the whole handler, or its callback tail a mandatory production implementation. The row above describes §14's bounded P1 route and its then-observed behavior, not the failing §17 route.

This requires a **genuine external write**: guarded mutation sets B/R,
retained descriptor durably writes/reads back D, *then* truthful stock
saved bookkeeping. Emission/marker without D success fabricates clean.
[`ResourceSaver` success bookkeeping](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/core/io/resource_saver.cpp#L102-L155)
and [`EditorNode` own subsequent signal/plugin notification](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/editor_node.cpp#L1752-L1775)
do not accompany a bypassed save automatically.
The [`global ResourceSaver callback`](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/editor_node.cpp#L7833-L7844)
handles other filesystem/folding work; separate
[`EditorData plugin fanout`](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/editor_data.cpp#L373-L377)
is not required for this document tag.

| Candidate/public source finding | Runtime/API disposition |
|---|---|
| `EditorPlugin.resource_saved.emit` | **Negative:** different emitter, no ScriptEditor tag; subject 8/6 dirty. |
| Actual EditorNode `resource_saved.emit` | **Bounded positive mechanics**, but [`Object` synchronous signal dispatch](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/core/object/object.cpp#L1179-L1260) invokes arbitrary peer listeners. Earlier listeners edit target or retarget focus **before** native tag, without an interstitial guard. Broadcast is disqualified; direct existing Callable bypasses these peers. |
| Direct public CodeEdit tag/edited setter alone | Version/flag only, no document-mtime/native event; earlier negative not re-probed here. |
| Direct `ResourceSaver.save`, front `ResourceFormatSaver`, normal Save, Save All | §13 fallback/all-editor work still disqualifies. Selected route **removes/non-uses** saver registration and never dispatches these for agent attempt. Later human Saves are separate ordinary operations. |
| `EditorFileSystem.update_file(s)`/scan, `Resource.emit_changed`, reload/replace/reconcile | No document tag, or altered source/history/broader files; [`EditorFileSystem` source](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/file_system/editor_file_system.cpp#L2395-L2564). No preserve-old-timestamp workaround. |
| Generic Resource reload | [`Script::editor_can_reload_from_file` false](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/core/object/script_language.h#L118-L123); [`EditorNode` respects it](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/editor_node.cpp#L1291-L1327). ScriptEditor owns script-document reconciliation. |

§17 directly tests this previously un-reprobed alternative with fresh guards and a natural timestamp-second crossing; its clean immediate state is insufficient because later ordinary Save falsely requests reconciliation.

### 14.4 E/F/G — final GUI matrix, races and later lifecycle

Preparation: subject 16, native 17→ordinary Save, native 18→ordinary
Save (clean **6/6** with prior history). Other clean D/R/B=70 with
newline; select OTHER then make one native dirty 71 edit **without**
final newline (B **4/2**, R/D=70). Target remains nonselected for one
18→23 complex edit (B **8/6**), explicit R=23 and pinned expected
D=18. Actual 35-byte descriptor write makes D=23 after a natural
mtime-second crossing. Python component-relative no-follow fd
reads/stat witness D; actual Script/CodeEdit getters/history witness
R/B. Positive direct callback: Resource edited false, subject **8/8
clean**, D=R=B=23, OTHER selected and its D/R/B/versions/dirty/history
unchanged immediately and next frame. Native write_calls stays **1**;
subject D bytes/size/dev/inode/mtime_ns/ctime_ns unchanged *across
finalization*.

| # / final case | Demonstrated stock runtime |
|---|---|
| 1 `happy_plugin_signal` | **Negative:** wrong emitter leaves subject 8/6 dirty despite D/R/B=23. |
| 2 `happy_editor_node_signal` | **Bounded positive mechanics:** actual emitter tags and survives later history/reopen; broadcast unsafe by cases 16–17. |
| 3 `happy_saved_handler_callable` | **Positive selected route:** direct actual receiver tags 8/8; target-only immediate/next-frame and later lifecycle witnesses. |
| 4 `guard_after_persist` | Human B=444/current 10 after D write; refuse before Resource cleanup/tag; saved 6 dirty. |
| 5 `guard_before_resource` | Human 444/current 10/saved 6 refused before Resource cleanup. |
| 6 `guard_before_tag` | Resource flag already cleared, human 444/current 10/saved 6: refuse tag, mark partial bookkeeping. |
| 7 `guard_same_text_newer_version` | Human 444 then restores visible 23: current **12**, saved **6**. Refuse despite source equality. |
| 8 `guard_after_tag` | Native 8/8 then human 444/current **10**, saved **8** dirty; preserve B, applied/unverified, never rollback. |
| 9 `wrong_resource` | Actual OTHER Script ID supplied: refused before bookkeeping. |
| 10 `wrong_mapping` | Actual OTHER CodeEdit ID supplied: refused, no document substitution. |
| 11 `closed_target` | Public `close_file` removes target after persistence; held Script alone does not authorize tag. |
| 12 `same_path_reopen` | New editor/buffer IDs at same path, clean new **2/2** from D; old attempt refused, not rebound. |
| 13 `readonly_receipt` | Actual O_RDONLY `pwrite`: **EBADF 9**, **0 bytes**, D=18, no fallback/tag, B/R=23 and 8/6 dirty. Whole attempt applied/unverified despite finalizer-local refusal. |
| 14 `unknown_receipt` | Deliberately discard receipt **after successful D=23 write**: unknown is not I/O failure, but no tag, 8/6 dirty. |
| 15 `namespace_replacement` | New owned inode with **same bytes** installed after successful write; retained fd detached, no tag/no replacement write. |
| 16 `reentrant_edit` | **Negative broadcast:** earlier owned listener edits B=444/current 10; native signal receiver falsely tags **10/10 clean** while D/R=23. |
| 17 `callback_retarget` | **Negative broadcast:** earlier listener changes OTHER→subject; tab departure synchronizes unrelated R **70→71**, though B/history/D unchanged. That R change alone violates confinement. Fixture `expected_guard=false` is an observed invariant failure, not an incomplete case. |
| 18 `direct_callable_listener_isolation` | **Positive distinction:** real external retarget listener armed; direct receiver calls **zero peer listeners**, keeps OTHER selected and its D/R/B/history unchanged, subject 8/8 clean. |

Failed, missing, unknown, mismatched receipt or changed identity/source/
version means **no native tag**. B/R/D may already have changed: a
refusal can be partial/applied-unverified, not safely "not applied".
Never replay old intent, overwrite newer human B or compensate with
rollback. Before-tag guard-to-native-tag has no await/user callout;
after callback perform fresh independent checks. Namespace attachment
checks bound success but do not claim atomic exclusion of arbitrary
external writers.

In cases 2–3 OTHER native Undo/Redo reaches clean 70 then dirty 71,
without newline or extra history step. Subject Undo reaches 18 at
**6/8 dirty**; later *human ordinary* stock Cmd+Alt+S saves 18
clean, Redo reaches 23 and ordinary Save cleans 23. Undo→Undo still
reaches 17; native Redo twice returns to 23. Close editor/reopen
returns clean D/R/B=23 with new document/buffer IDs and no external-
change/reconciliation dialog. Later **human Saves** can apply to
OTHER, add its newline and change R/history: OTHER-unchanged pertains
only to the **agent finalization**, never the entire human-Save
lifecycle.

Separate `evidence/gui-20260928-120855-be6e72/` visual replay of
`happy_saved_handler_callable` exited 0 with full lifecycle. Parent
inspected stock screenshot: OTHER 71 with no terminal newline,
`other.gd(*)` dirty, `subject.gd` clean and OTHER selected. Capture
used the owned-PID NSRunningApplication/CGWindow helper, no other
windows. Reopen logs in both happy final-matrix cases and replay
contain stock `get_line_wrap_count` (`p_line=-2`) diagnostics despite
observed clean coherent reopen: **not error-free UI or absence of
every warning**. No running-game/export/other-version support or A–E
acceptance follows.

### 14.5 H/I — callbacks, cost and disposition

After tag the handler refreshes script names and
[`trigger_live_script_reload`](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp#L749-L803).
It immediately obtains cached Script or ResourceLoader fallback and
calls language validation if path not already queued, **even when
runtime auto-reload is disabled**; it records path and may schedule
deferred debugger reload. Exact cached held Script and built-in-path
exclusion matter. Simple RefCounted variable fixtures with no running
game showed no unrelated D/R/B/history mutation during direct call,
but do **not** prove general validation, dependency, runtime or
deferred-effect confinement. That independent stock validation/effect
issue remains T003 blocker; no parser research repeated. Source
ordering is audited for the pinned build only: bounded method/topology
capability checks plus actual GUI proofs are required on supported
versions, not private-method ABI guarantees.

**Principle XIII total cost:** standard bundled GDExtension
orchestration adds C++17 builds, version/source/topology auditing,
Callable lifetime/exact mapping, descriptor receipt/namespace guards,
partial/reentrant outcome accounting and regression maintenance.
It also avoids the selected route's scripted/native
`ResourceFormatSaver` lifetime, error fallback and all-editor apply.
This bounded extension cost is preferable to a patched/custom editor's
build, installation, trust, exact-version distribution, team setup
and custom-editor CI **only if** separate validation/effect guarantees
can be met. No private Resource/document mtime numeric inspector is
selected; future Save/reopen is behavioral sufficiency only within
tested standalone lifecycle, not private field equality. §11 patch-A
selection, tag-last/no-callback and private numeric saved-state field
requirements are superseded as *current design*, while historical
evidence and patched T002 validator record stay truthful. No new
workflow, service, bridge/caller schema or task. T001/T002 remain
complete; T003–T005 pending and **T003 on hold** for independent stock
validation/effect control and affected design review.

**Current qualification:** §17 retains P1 under H3 as a held candidate, not a selected safe production path; the new minimal alternative failed mandatory later Save/history behavior. Neither negative establishes that calling the entire saved handler is necessary.

The completed T001 core was accepted against its then-approved numeric
`SavedStateEvidence` layout; that historical implementation still requires
private Resource/document mtime values and equality checks absent from P1
stock evidence (see [data-model migration boundary](data-model.md#savedstateevidence)).
Later T004 integration must migrate affected typed evidence, reducer and
contracts before stock P1 can be consumed. This is neither T001 incompletion
nor a new core implementation or replacement Rust/API field selection here.

## 15. Stock validation and effect-confinement research (2026-09-28)

### 15.1 Scope, evidence classes and retained provenance

This section assesses **explicit exact-source validation** and **independent safety of the stock editor/saved-handler continuations**, not persistence or the already selected P1 tag order. The installation constraint is official unpatched Godot 4.7.2 plus the addon and a standard public-ABI GDExtension. No private symbol/layout/vtable/detour, patched editor, custom editor, production validator, new product API or T003 implementation is selected. T001/T002 remain complete; T003–T005 remain pending, T003 on design hold. The T002 patch is an historical semantic oracle and effect-site inventory only ([contract §4](contracts/native-integration.md#4-primitive-b-exact-source-native-gdscript-validation), [quickstart §10](quickstart.md#10-t002-native-validation-acceptance-2026-09-28)).

The seven **exact evidence classes** used below are **demonstrated positive stock runtime** (P), **demonstrated negative stock runtime** (N), **stock public API/source finding** (S), **differential oracle finding** (O), **inference needing runtime proof** (I), **unresolved technical question** (U), and **tooling/environment limitation** (L). Bracketed letters mean only these classes, never stronger proof. Tables identify mixed claims by separate columns or statements. Source links at the end of §15 point to the checked *unpatched* pinned commit `ed1daf0bf001b61586d9930840f2f1394092c079`; source inspection is S, not a performed runtime case.

**[P]** Parent-verified, retained **local** evidence lives outside git under `/var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-stock-validation-effects-a3vtbtw0/`; official executable `4.7.2.stable.official.ed1daf0bf` SHA-256 `c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf`, macOS 26.6.2 (25G83), arm64, clang 21/SDK 27. Official generated public headers/API and extension were used, not patched T002 ABI. `evidence/helper-verified.json` and `evidence/reload-verified.json` retain independent checks. The real EditorPlugin **headless** EditorNode reload corpus has 19 usable observations in `reload-probe/evidence/20260928T145448-cbc00d93/` plus **one corrected loader-only** observation in `20260928T150947-04478d3e/`: 20 usable cases, not 21. Main summary/controller SHA-256 `be1d1a20d17bc6882cc27fbd20a1b4b1cfec20b166f417055d22d7fb76c8b4f7` / `116170ebb58533c8f803081e72e685522b9be626a3ba1d236d7ef376109a9787`; corrected loader summary/controller `93ff869a76b93c00cec6f32ff0f004fd0da8c33b7cef1d7ed8d815e80627e0a0` / `0f71b74af9cdcdb5c9fab0540d1c23eda48de6a6d1214f1597f629b4cff08bc2`; public ABI native library `2e258f3686b6ffa4c718d9131176ce1d3c8cfb2d413b6e1fe629c966addedd6b`. The earlier `20260928T144014-590042b0` controller failed GDScript type inference before candidate calls; the main corpus's original loader row lacked its staged loader and is excluded, **not** a stock refusal. The corrected run reused the successful native library. No dynamic getter or during-call dependency-reader barrier ran on stock [L].

**[P]** The separate helper's `helper-probe/evidence/summary.json` SHA-256 is `88ead3c4899e08b9293177e0a766a8d7b606f9531114bbab92c3cb214308f4a4`; its public-ABI library SHA-256 `69f62d4b9d7f54d746773e40ff1e6c9bf447aff4e1e228b0d6680d4998dc7f4d`. It completed **22** isolated `--headless --check-only --script res://scripts/subject.gd` cases (11 synthetic fixtures with and without `--editor`), native metadata initialized, no timeouts, unchanged root inputs and no recorded input writes, elapsed 0.3035375–0.6743918 seconds per process. This is startup `--script`, **not** a post-EditorNode EditorPlugin call or selected live editor context. The main headless real-editor corpus took 2.595 seconds as a whole; neither figure proves the Feature 002 controlled ten-second caller bound. Historical patched oracle `/var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-agent-kit-native-ex09m3nz/native-acceptance/summary.json` SHA-256 `f397774e96779c99fdb2d8fa783250182cc789e06773d72a1c3a5c5b65d23a49` was **not rerun** [O]. Local raw evidence, not deleted probe source, is the witness. The historical commands below are provenance, **not runnable reproduction instructions after cleanup**. The first reload invocation reached the failed controller; resuming that same work directory produced the usable main corpus. Only the corrected loader invocation is relative, with its working directory explicitly shown. No `--godot` argument was used:

```sh
python3 /var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-stock-validation-effects-a3vtbtw0/helper-probe/run_probe.py
python3 /var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-stock-validation-effects-a3vtbtw0/reload-probe/run_probe.py
python3 /var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-stock-validation-effects-a3vtbtw0/reload-probe/run_probe.py --resume /var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-stock-validation-effects-a3vtbtw0/reload-probe/work/20260928T144014-590042b0
# Working directory: /var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-stock-validation-effects-a3vtbtw0/reload-probe/
python3 run_probe.py --resume work/20260928T144014-590042b0 --only custom_resource_loader
```

Probe sources, binaries, generated headers and disposable projects are not repository assets.

### 15.2 A1–A4: what each stock engine actually does

**A1 — detached `GDScript`/`reload`. [P]** The standard-ABI native call constructed a *fresh* GDScript, set cached `res://scripts/subject.gd` path and exact supplied source, then invoked `reload(false)` from a genuine headless EditorPlugin callback after EditorNode initialization. Source/path getters round-tripped exactly in all 20 usable cases; root disk sentinel SHA-256 `4785ddadee2316444d062477e769d406dcecff853d59a6030516f76c8ed65d5d` remained unchanged. This is useful **input marshalling**, and several source-only relative/transitive arrangements returned `Error=0` [P], not a completed, attributed, effect-confined `valid`. A fresh Script per call did not certify a fresh shared dependency parser generation. **[S]** Both keep-state modes parse → analyze → compile → conditional `_static_init`; `can_run` derives from scripting-enabled or parsed tool status. `reload(false)` skips the `keep_state` static restoration and *later* `update_exports`, not analyzer/compile/init; it also can reject an instance-in-use before parsing. `reload(true)` can update exports, hence is no safer. Compile errors may return before init, but an induced compile failure is not a supported successful-after-analyzer stop and compiler resolution itself can load resources. A syntax/token-only preflight does not establish analyzer completion or purity. Setter `set_source_code` changes Script source bookkeeping; assigning the script-source *property* instead invokes `reload(true)` and is not interchangeable ([reload entry][s15-reload-entry], [reload][s15-reload], [source setter][s15-setter], [source property][s15-property]). **[N]** With `Error=0`, Logger 0 and unchanged root disk, a tool preload executed a static initializer, and a tool root's static `new()` executed a constructor during the measured real-editor native-call intervals. The non-tool static fixture created no new non-tool marker during its measured call; an earlier tool marker was isolated/removed, so neither false attribution nor general non-tool safety follows. Root-only `@tool` filtering is insufficient for dependency/tool/statics. No public variant has a proved post-analysis/pre-execution stop [U].

**A2 — ordinary editor validation. [S]** `CodeTextEditor` idle timeout calls `_validate_script()` then emits `validate_script`; `ScriptTextEditor::_validate_script` reads **current B**, calls the native language's `validate(text, script path)`, partitions root/dependency errors for UI, and on **successful non-tool** validation calls `script->set_source_code(text)` and `update_exports`; unsuccessful validation skips that success branch. It still performs connected-method lookup, warnings/error/background updates, pending dragged-export assignment and base editor signals. `apply_code()` separately source-sets, updates exports and applies pending dragged exports. `get_functions()` independently validates current B/path and updates its UI function list only on success ([editor methods][s15-editor-methods], [timer][s15-timer], [text editor][s15-text]). No detached arbitrary source argument, exact attempt/version completion token or effect-confined read context is exposed by this editor path. **[S]** Internal `GDScriptLanguage::validate(source,path)` really constructs a local parser/analyzer and collects root/dependency `ScriptError`s, but a C++ virtual in the engine is not an exposed standard-GDExtension callable with a scoped reader/effect interceptor ([validator][s15-validator]). Calling the normal UI validation solely to get a result would also authorize its B→R/export/callback work.

**A3 — GDScript LSP. [S]** `didOpen`/full-text `didChange` keep supplied text by URI-resolved path in a per-client document, then `ExtendGDScriptParser` parses/analyzes it and publishes diagnostics. These particular message handlers do not themselves write target D/R/B; `didSave`, in contrast, invokes ResourceLoader, reload/export/editor-live-reload work. The LSP document/parser result and diagnostic notification are URI/client based, not a selected target's source hash/current and saved version, dependency-generation certificate or call-complete valid/invalid/unavailable envelope. Null/no managed parser, an incomplete analysis and successful empty diagnostics must not collapse to the same result. Parser/analyzer still reach ordinary cache/load/value effects; link generation and scene-cache lookups introduce additional filesystem/ResourceLoader work. Using the selected editor's own LSP server shares that same process's project/class/autoload context without a second Godot process [S]; it still does not establish the selected target/version's confined dependency generation, prevention of forbidden effects or an attributed completed validity decision [U]. A separate LSP project/peer does not inherit a selected editor session by assertion ([protocol][s15-lsp], [LSP parser][s15-lsp-parser], [LSP save][s15-lsp-save]). Whether a restricted composition can prove no forbidden dispatch and exact selected context remains open [U]; neither diagnostic transport nor URI equality proves it.

**A4 — second official-stock process. [P]** The helper staged an owned source/config under the requested synthetic `res://` path without modifying the *real* target and exercised native/global/autoload metadata and relative/transitive source arrangements. **[N]** Non-tool *and* tool static initializers ran in both `--editor` and game-mode helper cases (four source-defined marker writes; constructors absent). All **eight** invalid cases (syntax, type, missing direct inheritance, transitive missing inheritance in each mode) exited **OS code 0** despite script errors. Valid helper output also includes cleanup `ERROR:` diagnostics; silence, nonempty stderr or process exit cannot certify validity/completion. **[S]** Main startup full-loads `--script` before the check-only return and before actual EditorNode constructor disables non-tool scripting; `--editor` hint alone does not suppress initialization. On macOS headless the `Main::start` invalid return is not carried into OS exit status ([startup][s15-startup], [full load][s15-full], [editor disable][s15-disable], [headless exit][s15-exit]). A helper is still a legitimate architectural candidate [I]: cold owned namespace avoids real target writes/selected-editor cache contamination and may simplify some confinement, but boot/staging/context mapping, incomplete diagnostics, initialization refusal and OS sandbox scope remain costs. OS denial of writes/network does **not** prove absence of project-code execution or safe permitted reads/in-process callbacks; immutable staging does not make live project settings/global classes/autoloads equal. The observed startup times establish only these synthetic process times [P], not caller deadline acceptance.

**[S] Startup/exit qualification:** EditorNode construction is later than the check-only branch ([EditorNode creation][s15-editor-create]); the observed editor hint does not make this a normal EditorPlugin entry. `OS_MacOS_Headless::run` does not transfer `Main::start` failure into the OS exit-code state ([headless exit][s15-exit]); OS exit defaults to success and platform main returns that state ([OS default][s15-os-default], [platform main][s15-platform-exit]). These pinned paths explain the measured eight invalid-but-exit-0 cases [N], rather than reinterpreting script errors as acceptance.

**Independent requested validator matrix.** Each cell uses §15.1 classes; “open” is not a selected production implementation. Exact source/path means actual supplied root bytes and `res://` attribution, not merely a staged file name.

| Candidate | Exact source/path | Real semantics | Confined dependencies | No forbidden effects | Attributed diagnostics | Completion state | Disposition |
|---|---|---|---|---|---|---|---|
| Fresh detached reload | [P] Exact fresh root/path getters; R belongs to scratch | [S] Parse/analyze **and compile**, not validation-only; [P] pertinent errors/relative successes | [S] Engine remap/open/path cache, not injected witnessed bytes | [N] Tool init/constructor, loader hooks; false keep-state does not prevent | [P] Some root/dependency Logger text; no consumed hashes | [S] `Error` describes reload; Logger not complete validation | [U] Restricted profile open; measured broad route not qualifying |
| Editor validation | [S] Current B and Script path only | [S] Genuine language validator, plus B→R/export on success | [S] Native cache/loader, no scoped native reader | [S] Extra editor/export/dragged-property callbacks unbounded for proposed input | [S] UI root/dependency errors, no attempt hash/version | [S] Idle timer/UI update, no target-version completion token | [U] Must independently confine normal continuation; not detached preflight |
| LSP | [S] Full text/URI in per-client store | [S] Genuine parser/analyzer | [S] Analyzer caches/loaders, scene/link lookups | [S] No effect interceptor; didSave explicitly effectful | [S] URI diagnostics not supplied/dependency generation | [S] No source-hash/attempt completed-result witness | [U] Restricted context/safety proof absent |
| Separate stock process | [P] Staged exact synthetic root/path; not selected live editor | [S] Startup full-load, not parser-only | [I] Cold owned namespace possible; live mapping unproved | [N] Four static executions; OS sandbox ≠ no execution | [N] All eight invalid cases exit 0; logs not structured certificate | [N] Exit/silence ambiguous despite observed script errors | [U] Narrow allowlist/helper still open, broad measured route unsafe |
| Extension-owned parser adaptation | [I] Immutable supplied source/path possible | [S] Upstream algorithms are reference; [I] useful adapted subset not built | [I] Call-local descriptor-fed graph possible | [I] Before-dispatch refusal possible, not proven | [I] Upstream error locations can be retained | [I] Explicit three-state envelope possible, not built | [U] Credible official-ABI candidate, **not selected**; callback boundary remains |

### 15.3 B/C — execution inventory and exact dependency ownership

**[S]** This is the requested **eleven-effect** inventory. “Required” distinguishes *semantic analysis* from extra reload/editor effects; “avoidable” means an alternative route in principle, not a demonstrated public stock control. A conservative token scanner may **refuse** forms, never assert semantic validity or that a hidden dispatch is absent. Engine parser/dependency cache lookup, population and edges are permitted **when correlated**, unlike arbitrary project-code execution ([cache][s15-cache], [analyzer][s15-analyzer], [exports][s15-exports]).

| Effect | Required for correct analysis? | Avoidable with different public route? | Predetect before dispatch? | Confine externally? | Refuse when unbounded? |
|---|---|---|---|---|---|
| Static/tool initializer | No; extra reload/full-load stage; inner classes too ([reload][s15-reload]) | Parser/analyzer-only internal route avoids it, not a stock public scoped API | `@tool`, static declarations can be conservatively refused across **all** dependencies; tool annotation alone is not execution | Helper OS restrictions limit some outward effects, not execution | Yes before reload/loader on any hazardous graph |
| Constructor / `_init` | No instance required for analysis; static `new()` can call it | Analysis without instantiate, not guaranteed by reload | `new`/call tokens only partial; recursive source closure needed | Isolation cannot make a constructor nonexecuting | Yes where reachable or unknown |
| Scripted getter/property access | Not for static engine-native metadata; analyzer constant member reduction may reach `get_named`→`Object::get`→script-instance/extension getter ([getter][s15-getter], [Variant dispatch][s15-variant-getter], [Object dispatch][s15-object-getter]) | Static descriptors/local safe values in adaptation; no stock general interceptor | Dot/bracket tokens cannot establish receiver type; Object values need semantic guard | Sandbox cannot undo getter execution | Yes before Object/extension getter dispatch |
| Custom ResourceLoader callback | No arbitrary user loader required for source-only `.gd`; stock preloads may dispatch | Confined text-reader/local resolver proposed, not injected into stock cache | Literal preload partial; global class/autoload can load without it | Staging/OS read sandbox does not stop callback | Yes before any unsupported loader callback |
| Loader recognized-extensions/exists/resource-type hooks | No for confined source bytes; stock loader may query *before* `load` | Same local resolver only if all paths redirected before hook | Extension/type/path plus registered loader context, not substring safety | Post-hook sentinel is too late | Yes before *recognized_extensions*, `exists`, `get_resource_type` and load |
| Object/string conversion and Variant reduction | Built-in constant folding yes; project Object conversion/stringification no ([constant evaluation][s15-constant], [getter][s15-getter]) | Safe built-in values via public Variant ABI; no purity for arbitrary values | Concrete nested value/provenance needed, tokens insufficient | Outside-process effects still execute inside it | Yes for Object/Callable/Signal/unknown values before evaluate/format |
| Arbitrary Callable/user method | No for analysis of method bodies | Parser/analyzer-only and descriptors can avoid invocation | Calls visible syntactically, indirect dispatch not fully lexical | OS policy cannot prohibit in-process user call | Yes before dispatch |
| Process/network/file effects | No; reachable through initializers, loaders, getters or user callbacks | Eliminate the dispatch; not by checking output files | API-name blacklist misses aliases/indirection | Sandbox can cap OS surface but not no-execution, in-project reads/effects | Yes if upstream execution path cannot be ruled out |
| Resource/Script/D/R/B mutation | No for pure B; reload mutates scratch Script, ordinary editor may mutate target R | Exact-source parser/analyzer route proposed; saved-state B/R/D changes separately authorized | Source tokens do not predict editor callback timing | Descriptor reader/write guard covers only owned target operation | Yes for validator-originated target writes/replacement |
| Parser cache/global-class/autoload context activity | Cache/edge bookkeeping and metadata **reads** can be required; arbitrary config mutation not | Local graph/context snapshot possible; global cache need not be cleared | Cache status/generation cannot be lexed | Hash/context witnesses possible, but stock cache use not thereby certified | Refuse stale/unattributed metadata or changing mappings, **not** normal correlated bookkeeping |
| Export update/application | Not required for syntax/type proof; reload(true), ordinary non-tool validation and `apply_code` can do it | `reload(false)` removes only its own later export tail; editor callback remains | Exports/dragged-property state not decided by root tokens | Post-hoc Inspector observation not prevention | Refuse if this target/version callback can apply outside admitted closure |

**C — per-candidate opener, injected bytes, reopen, generation, attribution and early refusal. [S]** The contract asks whether **the selected validator consumed the descriptor-read bytes**, not whether an external reader once hashed a file. Native parser refs resolve remaps, read source/binary and advance statuses; `get_parser` keys by path, records owner edges and may reuse a prior ref ([cache][s15-cache], [cache graph][s15-cache-graph]). Relative references must resolve against *each containing source path*, including transitive children, not always the root. A later path open, `.gd.remap`/UID, binary `.gdc`, outside alias or arbitrary ResourceLoader hook can bypass a preflight-only check. A missing file established by confined read may produce attributed `invalid`; unreadable/unconfined/changed source or unsupported loader must yield `unavailable`, without leaking denied paths ([native contract][s15-contract]).

| Candidate | Who opens dependency / can exact bytes be supplied? | Later pathname reopen and generation | Relative attribution / earliest safe refusal |
|---|---|---|---|
| Fresh reload true/false | Stock parser/cache, compiler and ResourceLoader open; extension supplies only root, not each parsed dependency | Path-keyed parser/shallow/full cache can reuse status; fresh root and disk hash do not certify consumed generation; compiler may load again | Stock resolver understands relative/transitive syntax, but no per-edge confined read witness; refuse risky graph **before** invoking reload or result unavailable |
| Editor `_validate_script`/`get_functions` | Native language validator consumes B and opens own dependencies | Shared cache and normal export/loader continuation; no public per-read digest | UI errors include dependency paths, not source-hash generation; no public intercept-before-loader or target/version-specific refusal |
| LSP full text/URI | Root comes from managed text; analyzer/scene/link/cache paths open dependencies | Document reparse does not invalidate all engine path-keyed refs; didSave adds load/reload | URI alone not per-containing-file source witness; client/scene lookups must be excluded or safely pre-refused |
| Helper `--script` | Staged overlay supplies files, then new process ResourceLoader/cache opens them; not an injected no-follow descriptor stream | Cold startup avoids some old host cache, but loader can reopen staged path; selected editor context/generation not transferred automatically | Owned relative files work in synthetic cases [P]; require confined staging and identical mappings plus early lexical/metadata admission, not OS exit/log silence |
| Adapted upstream parser/analyzer | Proposed call-local resolver feeds immutable exact source and confined descriptor-read `.gd` children | Proposed no global cache; hash/identity/context recheck each graph node, reject mid-call change | Per-containing-path relative resolution and source-derived class interfaces are possible [I]; refuse remap/UID/binary/non-GD/outside/unknown metadata **before** loader or Object dispatch; compilation not demonstrated |

**[P]** The stock between-call `same_path_before_change` → `same_path_changed_dependency` → `same_path_restored_dependency` produced reload Errors **0 → 43 → 0**, with independent changed/restored file hashes. **[U]** This does not show which cached parser/shallow generation was consumed, same-bytes replacement, selected target cache state or *during-call* invalidation. The `autoload_initial` interval logged errors for earlier `absent.gd` and `cycle_a.gd` paths before the root failure despite oracle-valid root/context [N/O]; `get_parser` owner dependency accumulation and `finish_compiling` later full-load provide a plausible cross-case mechanism [S/I] ([cache graph][s15-cache-graph], [compile dependencies][s15-finish]), not a proven language-semantic regression. Process-global Logger temporal windows are not attempt-local diagnostic ownership.

### 15.4 D — diagnostic floor, not a weakened spec

**[S]** The behavioral floor from [spec FR-009/FR-020](spec.md), [native contract §4](contracts/native-integration.md#4-primitive-b-exact-source-native-gdscript-validation) and [data model](data-model.md) remains exactly `valid` (completed parser **and** analyzer on actual supplied bytes/context), `invalid` (completed semantic rejection with attributable root/dependency evidence), or `unavailable` (incomplete, unsupported effect, access/limit, reentrancy or invalidated context); bind identity, exact root `res://` path/UTF-8 hash, purpose (`preflight`/`post_change`/`unchanged`), dependency/context witnesses and `diagnostics_complete`. Distinguish root error, dependency error, unsupported effect, invalidated dependency/context and incomplete validation; do not assign an unread dependency the root hash. Needed bounds remain root/dependency ≤512 KiB, ≤32 dependencies/4 MiB, ≤64 errors and bounded messages/paths. The contract also *currently* requires actual byte length, editor-clock start/end ticks, reason, origin/path/source hash when read, line/column **when supplied**, category/message, path redaction and no incidental error-log disclosure. No change to those obligations is made here. **[I]** Design review may consider whether exact line/column, category/message formatting, timing fields and internal limit numbers are implementation-level presentation rather than distinct spec-MUST consumer behavior; only a separately approved contract migration could adjust them. This research **does not** drop fields or treat a bare bool, `reload` Error, exit status or silence as the existing completion/attribution contract. Stock native `validate` can return bool and ScriptErrors including root/dependency path/line/column/message [S]; stock reload/Logger and LSP notifications do not thereby supply the native-contract receipt. T002's historical completed diagnostic/control behavior is O, not qualifying stock B.

**[S] Actual spec versus review alternative:** [FR-009](spec.md) mandates a target post-change parse result without parse error and explicit unavailable/parse-error non-success; [FR-010](spec.md) mandates *relevant source-attributed diagnostics* with target/revision/stage/application/coherence evidence; [FR-020](spec.md) forbids arbitrary evaluation, unconfined access and incidental content logs; [FR-022](spec.md) disallows universal-refusal “completion.” Neither exact every-line/column/category/message formatting nor a general LSP compiler-diagnostics service is separately demanded by those FRs. The existing native contract **does** bind the concrete fields and limits specified immediately above, including line/column when supplied: that contract stands until a reviewed design decision changes it. A possible smaller user-visible presentation is a review question [I], **not** authorization to omit fields today or a claim that stock Logger satisfies FR-010.

### 15.5 E/F/G — native saved handler, ordinary callbacks and public observers

**[S]** P1 still obtains the **existing incoming** `EditorNode.resource_saved` → canonical `ScriptEditor` Callable through public signal introspection, checks the unique emitter/receiver/edge and directly invokes it with the held exact Script after descriptor receipt, edited-flag clearing and renewed identity/path/source/current-and-saved-version/namespace/cached-Script guards (§14; [contract §3](contracts/native-integration.md#3-primitive-a-guarded-target-document-saved-transition)). No private method-name lookup, signal broadcast, Save/Save All, ResourceSaver, replacement finalizer or patched engine. The native receiver tags matching ScriptEditorBase/CodeEdit (saved version **then** document path mtime), scans built-in siblings, refreshes names/**current member overview**, then calls `trigger_live_script_reload(script path)` ([handler][s15-handler], [tag][s15-tag], [§16](#16-stock-saved-handler-admission-research-2026-09-28)). Tag precedes validation; an error or unbounded effect *after* tag cannot be reclassified as “not applied.” Refuse colliding built-in siblings before mutation: they can be tagged, reloaded and have docs updated. Name refresh is not limited to UI bookkeeping: it can validate the current editor's B/path, which need not be the selected target.

**[S]** On a path not already in the internal queue, `trigger_live_script_reload` looks for cached Script, otherwise calls `ResourceLoader::load`, then invokes `script->get_language()->validate(script->get_source_code(), path)` and appends the path only if validation permits; this is a **synchronous** source/cache/dependency/effect concern even when auto-reload is off ([trigger][s15-trigger]). The direct handler does **not** unconditionally call `update_exports`, but its name refresh **does transitively call current-editor `get_functions()` and validate current B/path** ([§16](#16-stock-saved-handler-admission-research-2026-09-28)); the former “no get_functions” claim was false. Equal B/R at guarded entry is not proof of *dependency parser* generation. The internal path queue may already contain this path, in which case the live-reload validation branch is skipped; no public queue-membership witness is established.

**[S]** Ordinary edit-generated idle validation independently consumes current B and may source-set R/update exports; `get_functions()` independently validates B/path; `apply_code()` always source-sets/updates exports and can apply dragged properties. `GDScript::update_exports` may parse/analyze, full-load a script base, propagate placeholders and recurse into cached inheritors ([timer][s15-timer], [editor methods][s15-editor-methods], [text editor][s15-text], [exports][s15-exports]). Timer validation emits downstream editor signals and performs connected-method/UI work. Neither explicit preflight nor handler result intercepts these paths. The supported policy must bind the proposed and post-change B/current-and-saved versions, R, exact cached Script, dependencies/context and queued callbacks to **the same target attempt**; newer independent human input must keep normal handling and may not be retagged by an old agent attempt. No public per-target cancellation/version-correlated timer completion mechanism was established [U].

**[S]** When `!pending_auto_reload && auto_reload_running_scripts`, the handler schedules `_live_auto_reload_running_scripts` deferred; that callback clears pending, hands a **shared path list**, not an attempt hash or CodeEdit version, to `EditorDebuggerNode::reload_scripts`, then clears it. Node fans out to debugger tabs; `ScriptEditorDebugger` sends its `reload_scripts` packet only when its remote peer is connected ([trigger][s15-trigger], [debugger fanout][s15-debugger], [remote send][s15-debugger-send]). Public `EditorDebuggerPlugin.get_sessions` and `EditorDebuggerSession.is_active` plus `started`/`stopped` can observe session topology ([public debugger plugin][s15-debugger-plugin], [public debugger session][s15-public-debugger]); they are **not** a private queue peek, target/version acknowledgement, drain fence or proof a later connection cannot change the deferred effect [U]. Zero active sessions can prevent a remote send **at the checked instant**, not eliminate immediate validation, pending work or future sessions. Turning off the live-reload setting affects optional scheduling only; temporarily changing UI settings can persist metadata or race a human preference, and no safe restoration/target-only bypass is established ([debug option][s15-debug-option]). Do not assume a private queue API, a next idle frame, no currently running game or post-effect D/R/B observations prevent forbidden earlier execution.

**Independent requested saved-handler matrix.** “Required” means required for the **selected truthful saved-state transition** or independently required editor behavior, not a license for unbounded effects; row observations distinguish source from §14's bounded positive saved-state runtime.

| Effect | Happens? | Required? | Safe on stock? | Can avoid/observe/refuse? |
|---|---|---|---|---|
| Target saved tag | [P/S] Matching held Script tab tags current CodeEdit version in P1 | Yes, after successful D receipt and fresh guards | [P] Bounded standalone P1 mechanics only; callback tail still open | Direct retained Callable avoids peer signal broadcast; refuse wrong target/versions/collision before tag |
| Document mtime update | [S] Same tag reads actual path mtime after CodeEdit tag | Yes for ordinary ScriptEditor document bookkeeping | [P] Later Save/reopen behavior observed in §14; no numeric private mtime assertion | Keep native whole handler; path/namespace guards and later behavioral witnesses, no bare CodeEdit tag |
| Validation | [S] Immediate cached-Script source validation unless path already queued; fallback loader if absent | Not necessary to tag, but unavoidable on that normal handler branch | [U] Confined dependency generation/effect boundary not established | Require exact cached Script; source/context admission **before** call; queue membership not publicly controllable |
| Export update | [S] **No unconditional direct-handler export call**; yes via independent ordinary validation/`apply_code` and base/export propagation | Normal editor behavior for some sources, not standalone tag | [U] No target/version closure proof | Account for timer/function/export work and refuse unbounded graph; `reload(false)` alone does not stop this |
| Debugger/live reload | [S] Conditional deferred schedule; eventual remote send only to connected peer | Not needed for saved tag; native editor option may enable it | [U] Path-only shared payload may affect running project | Public session observer exists; no supported target/version queue completion; refuse if cannot prove harmless before dispatch |
| Peer callbacks | [P] Direct Callable bypassed other `resource_saved` signal listeners; [S] handler still does global name/built-in and ordinary validation signals separately | No peer signal broadcast needed; normal editor callbacks remain | [P] Isolation of save-signal peers in §14, not all editor callbacks | Never broadcast; refuse built-in collision; account for ordinary editor signals separately |
| Deferred work | [S] Path queue/`pending_auto_reload` and edit idle timer can outlive Callable | Not required to assert synchronous tag; ordinary editor work can be pending | [U] No public exact attempt/version drain fence | Observe public timer/session/state and independent D/R/B; observation *after* forbidden execution insufficient, refuse if closure unprovable |

### 15.6 H and A5 — supported refusal, useful positive scope and upstream adaptation

**[S]** Existing Feature 002 policy permits H2: effectful/unsupported source/context can return `unavailable/unsupported_effect` **before** execution, without relaxing FR-020 or replacing missing diagnostics with success ([contract §4](contracts/native-integration.md#4-primitive-b-exact-source-native-gdscript-validation)). H1's blanket “every source must be validated without any project code executing” is not a reason to accept an unsafe operation or universally refuse. **[I]** A conservative pinned lexer/token allowlist could admit ordinary non-tool, native-base declarations/functions and **some** confined, non-remapped source-only relative/transitive `.gd` dependencies; safe local/native built-in constants and captured global-class/autoload `.gd` mappings are possible only with matched metadata and recursive graph policy. Reject tool/static initializer expressions, executable preload chains, unknown Object/Callable/Signal values, custom/foreign loaders, UID/remap/binary/outside representations and unapproved extension classes *before* stock dispatch. Do not use substring matching as a semantic checker: `extends Global`, autoload names, nested class static variables and transitive preloads can load without a visible root `preload`; member/index syntax does not identify a receiver type. The accepted subset still needs Godot's real parser/analyzer and complete diagnostic/context proof, and both explicit and automatic callback chains must remain inside the **same** admission boundary [U]. A no-dependency root is a discriminating first fixture, not useful-scope completion on its own. Do not silently rewrite current contract's “`@tool` syntax alone is not execution” into unconditional policy rejection; any narrower supported profile requires design review.

**A5 — minimum candidate adaptation, nine input families. [I]** An adapted *upstream* tokenizer/parser/analyzer inside a standard-ABI extension could offer a local parser graph and descriptor-fed source resolver without relying on host private symbols. This is a source-maintained adaptation, not “copy three `.cpp` files,” not a fresh generic language parser and not selected or compiled. Source says `GDScriptLanguage::validate` uses parse→analyze and collects dependent parser errors ([validator][s15-validator]); analyzer phases require inheritance/interface/body/dependency ordering ([analyzer][s15-analyzer]). Minimum upstream families: tokenizer/tokens/source spans; grammar/AST/annotations/parser diagnostics/`DataType`; analyzer phases, supported builtin type and constant reduction; local parser-ref status/edge graph and source-backed shallow class/interface representation. A public ABI metadata/value adapter and native confined descriptor-reader/result witness join them. **No compiler, bytecode generator, VM, full engine cache or editor framework** belongs in this proposed minimum; no source proof yet establishes its smallest linkable translation units [U].

| Upstream dependency category | Supply source for proposed adaptation / safety boundary |
|---|---|
| Pure/tokenizer/parser logic | Imported pinned upstream tokenizer, token/source-location logic, grammar and AST in **extension-owned** local code; not callable host private parser |
| Godot core value/types | Public generated GDExtension ABI opaque `String`/`Variant`/containers where sufficient; own local parser `DataType`/AST records where host-private layouts are unavailable; verify exact precision/version |
| Native class metadata | Public `ClassDB` class/method/property/enum/constant/signal descriptors and provenance; **no** instance getters, refuse unknown extension-defined type |
| Project/global class metadata | Public `ProjectSettings.get_global_class_list()` plus captured class path/base/language/tool/abstract context; construct source-derived shallow interfaces from confined `.gd` graph, not stale Script objects |
| Autoload information | Public `ProjectSettings.get_setting("autoload/<name>")`/property-list settings and applicable overrides, immutable name/path/singleton context; refuse scene/foreign/effectful autoload |
| Dependency loading | Exact root immutable bytes and per-containing-file relative `.gd` bytes from native no-follow descriptor reader, identity/hash rechecks; refuse UID/remap/binary/outside/non-GD before dispatch |
| Parser cache/shallow script metadata | Call-local owner/dependency edges and status transitions plus parsed source class interfaces; never trust host path-keyed cache or clear global cache |
| Constant/value evaluation | Public Variant operations **only** on proven inert builtin values under imported analyzer semantics; refuse Object/Callable/Signal, scripted getter and unknown nested values before operation |
| Diagnostic formatting | Imported parser/analyzer source spans/error stage; extension-owned root/dependency/complete result envelope, redacted paths, actual read hashes and bounded messages |

**[P]** The helper's native public-ABI sample successfully read `ProjectSettings.get_global_class_list()` (`GlobalBase`, Node base and `.gd` path), `get_setting("autoload/ProbeSingleton")` and its property-list key; public ClassDB sampled `Node` API type, `get_node` method and `process_mode` property. `ClassDB.class_exists("GlobalBase")` was correctly false for a **script** global class. Thus “no public global/native metadata route” is false for these sampled fields; neither comprehensive analyzer metadata parity nor live selected-registry generation is proved. [S] The supported metadata methods are [ProjectSettings binding][s15-settings] and [ClassDB docs][s15-classdb], distinct from private ScriptServer parser state. [U] Need exact enum/default/builtin/overload/warning/native-extension provenance and settings/registry drift parity for accepted syntax.

Six adaptation threshold answers, without substituting a generic Principle XIII checklist:

1. **Exact stock limitation [S]:** upstream parse/analyze accepts exact source/path internally, but public stock extension interfaces do not expose its parser-ref read/effect context; reload/full load compiles and can initialize. Adaptation could expose immutable bytes and a local resolver; it **cannot** intercept ordinary host saved-handler/function/export callbacks by itself.
2. **Minimum upstream subset [I/U]:** tokenizer/AST/parser diagnostics, analyzer phases and safe builtin operations, local parser-reference/shallow source metadata and public ABI adapter/descriptor reader, as mapped above; avoid compiler/VM/private host symbols. Actual compiling/linking dependency set unproved.
3. **Semantic/context reconstruction [S/I]:** pinned grammar, scoping/type/inheritance, conversion, callable signatures/defaults, enums/constant rules, native CORE/EDITOR ClassDB, global classes, autoload settings, warning-as-error configuration, exact root `res://` identity, per-file relative paths, immutable graph bytes and witnessed context/dependency generations. Never infer source classes by loading project objects, guess cached types or accept scene/UID/remap/foreign paths by pathname alone.
4. **Drift [I]:** pin official source commit, extension ABI/schema and precision; separately track imported source and local deltas, review tokenizer/parser/analyzer/type/metadata/effect-site changes and accepted/refused corpus on every newly supported Godot version. An ABI match is not semantic parity; unreviewed versions unsupported.
5. **Official/T002 differential confidence [I]:** compare *the accepted exact roots/context* to official stock language behavior and historical T002 oracle, plus source-level dispatch audit and pre-effect refusal/confinement witnesses. Agreements, safe refusal, forbidden execution, ambiguous completion and stale generations stay distinct. Neither 54 historical patched passes nor the 22 synthetic helper cases certify an adapted stock product. Differential alone cannot prove callbacks safe.
6. **Total cost against replacing every user editor binary [I/U]:** an addon-bundled standard-ABI extension keeps users on the official editor instead of building/distributing/updating an exact-version custom editor for each user; that installation benefit is real. Maintainers still own upstream semantic drift, local metadata/context adaptation and differential/effect regression, *plus the independent host callback boundary*. Helper avoids some source upkeep but adds cold boot/staging/live-context/completion reconciliation. No LOC count, probe timings or inability to call private host parser selects or dismisses adaptation; choose it only after bounded feasibility **and** callback closure, not merely because other routes failed.

### 15.7 I — actual oracle results versus observed stock corpus

**[O]** Oracle statuses below are from actual historical T002 `validation.status`/`result.status` in [retained patched summary][s15-oracle], **never** reload probe `expected_oracle` labels. **[P/N]** Stock E/L means actual native `reload(false)` Error integer / process-global Logger error count in [main controller][s15-rmain] or [corrected loader controller][s15-rloader]. “No marker delta” means only five monitored sentinel files did not change during the measured native-call interval; it is **not** absence of arbitrary effects or a qualifying `valid`. All stock calls used a fresh synthetic Script at `res://scripts/subject.gd`, not the selected live CodeEdit/Script/receipt. The root disk sentinel was unchanged in all 20 usable observations; no selected R/B/version or consumed dependency-generation witness was returned. The oracle selected subject has *different* D SHA-256 `7e62bd065cc1b608bb08e9052ec88d5e3b635e24618ca15bee9166ed7c39fb6e`, selected R/B SHA-256 `6b8579a9413331f70f780638183541de16b983306a8e4b9866658675c4b482da`, versions 3/2, and reader/context state. Even where root bytes and staged dependency files match, **context parity does not**.

| Requested category / actual stock case | Exact root SHA-256; actual T002 oracle result [O] | Official stock runtime [P/N] and defensible differential limit |
|---|---|---|
| Simple valid / `valid_unicode` | `3c81edb924ed46f8d8fb75c4320e17988d7036708e3a7b582359725ddcef5459`; `native_valid_unicode_root`: **valid**, 45 UTF-8 bytes | E/L 0/0, no marker delta [P]; exact root bytes/path, not a complete validity certificate |
| Empty / `empty_source` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`; `native_empty_source`: **valid**, zero bytes | 0/0, no marker delta [P]; useful input control, not useful positive feature scope alone |
| Syntax / `invalid_root_parser` | `9ef2231356e04ec3f72b51e7bdce2cc90333f64e30005dbec54784442f1ba390`; **invalid/parse_error**, root 2:1 | 43/1, “Expected variable name after var,” root line 2 [P]; pertinent rejection, Logger lacks oracle completion/context receipt |
| Type / `invalid_root_analyzer` | `f8cba6857ffc3f1752f51d0e4ec2e6e986b33fba8b5f4345e463d68c9245aa69`; **invalid/analysis_error**, two root diagnostics at 2:18 | 43/2, String→int/typed-constant errors [P]; Logger's “Parse Error” prefix does not mean analyzer was skipped |
| Relative inheritance / `direct_relative` | `60705900d06a5b7db36e073a6ff2df6669cfd1a1c38ee77bc326f4c767f33b49`; **valid**, `level_one.gd` consumed | 0/0, no marker delta [P]; staged `level_one.gd` matches oracle dependency bytes, not stock consumed-generation proof |
| Transitive nested / `transitive_nested` | `e83d35534a5ecc7c5fc4633af02d9f0115396f5010b07f5878740524ecd786ef`; **valid**, `level_one.gd`, `deep/level_two.gd`, `sibling.gd` consumed | 0/0 [P]; source graph includes dependency-relative preload, but stock gives no per-edge read witness |
| Missing dependency / `missing_dependency` | `62562a6f728edd7b27d78684d1d650b1017a36407bd0817c4a13dd7c62dcb8c1`; **invalid/dependency_error**, missing dependency source hash null | 43/1, absent.gd unresolved at root line 1 [P]; no separate confined missing-read provenance |
| Cyclic root overlay / `cycle_fixture_root_source` | `3f6e9ec9af5e474556dfbc6c41101d4e54dcbb4d32bd48ac9b7a20eb55f8c0ab` is **stock-only recorded hash**. Oracle `native_cyclic_parser_lifetime`: 32 returned invalid calls required by harness; stable object count, but no serialized per-call statuses/input hash | 43/1, cyclic inheritance root line 1 [P]; same literal as historical harness, **not hash/context-certified parity**; stock root disk D differs from oracle overlay |
| Additional `cycle_a`↔`cycle_b` / `cyclic_dependencies` | `092b4dc95972be491cc5b810e425afeba14f75fdb6c7a76a608ea01dab761bab`; **no exact oracle root** | 43/1 unresolved superclass [P]; stock-only extra cycle control |
| Initial autoload / `autoload_initial` | `016ff06e8444616e866ae55f620ea9c6c49e134d42e4e1ae0dbd7e02bb4c46a5`; **valid**, `context_node.gd`, oracle context `b4437789c5453d23d6853cf590aced63f7f03ad0de8a83d5fa3c7b5d017931e3` | 43/3 including **prior** absent.gd/cycle_a.gd paths and root compile-dependent failure [N]; unattributed cache/Logger interval, **not** proved semantic mismatch |
| Changed autoload / `autoload_changed` | Same root hash `016ff06e8444616e866ae55f620ea9c6c49e134d42e4e1ae0dbd7e02bb4c46a5`; **valid**, `context_other.gd`, context `e3dd914017bcc0b2909e46c5e273247aa32f5b1af61e067a7e18ef0e51724b29` | 0/0 [P]; setting staged, no stock current-context certificate |
| Invalid autoload / `autoload_invalid` | `6a64b5350dc6d8718be6b7265cfba7802e1182708856344facc349f25e03aea9`; **invalid/analysis_error**, `context_invalid.gd:2` parse + root error | 43/2, pertinent dependency/root errors [P]; no stock dependency hash attached to logs |
| Ordinary GDScript preload / `gdscript_preload` | `317d481622dad7703ed768f4815da81df366de7e240f446699b21aa141afe0bd` stock-only root; **no exact oracle root** (oracle transitive row includes sibling preload) | 0/0 [P]; extra source-only control, not matched valid result |
| Tool dependency/static init / `tool_script_preload` | `5c5019e41f3b50dc5a78df6d9dfd4a6f667f6716e0b546f650bcfabf14d6a8b6`; **valid** oracle, tool dependency SHA-256 `e3bb3253ecaa2d176e3187d401bc181fc4a794d58fad1459c002f86831c55c0b`, no execution in historical fixture | **0/0 plus newly created static-initializer sentinel** during stock native call [N]; matched root/staged dependency, direct forbidden execution despite zero errors |
| Non-tool dependency static / `non_tool_static_preload` | `e150acbbea344430b6e79438266a0e5a2a3e02efdce8783c0e7b81a97c2f9bef` stock-only; **no exact oracle row** | 0/0, no newly created non-tool static/constructor marker [P]; earlier tool marker was preexisting and isolated, not execution by this row; no general safety conclusion |
| Tool root static `new()` / `tool_root_static` | `de7437132d56b72fd20fb5bd06d5abe103ee165ac1e35e5ef338d2b3d947e99b` stock-only; **no exact oracle row** | **0/0 plus newly created constructor sentinel** [N]; tool dependency was warmed by earlier case, not another cold initializer proof |
| Custom ResourceLoader / corrected `custom_resource_loader` | `c2181dc229519baf3b0ca75fc7b0b4a64c7a7b8c7536f8be9d80e6a880b46a61`; `native_excluded_loader_before_hook`: **unavailable/unsupported_effect**, incomplete, no hook in historical fixture | Corrected loader-only E/L **43/2**, newly created loader sentinel; `recognized_extensions`, `exists`, `resource_type` **executed before failure** [N]. No `load` callback observed: **do not claim it ran**. Exclude missing-loader original row [L] |
| Dynamic getter/object value / not exercised stock | Oracle-only root `0f835dc638ff02ef3fea76fe462c62b5c447ed15c9eddec8365edaf2f29569f4`; `native_dynamic_getter_refused_before_effect`: **unavailable/unsupported_effect** | No stock getter result [L]. Source `Variant::get_named` → scripted Object getter is an effect hazard [S], not a measured stock execution |
| Diagnostic Object stringification / not exercised stock | Oracle-only root `f9ed0d8bd55be9afa7efc5ab9ff110fe685d70230ece09de371229a58e8f1958`; `native_diagnostic_stringification_refused`: **unavailable/unsupported_effect** | No stock stringification fixture [L]; historical refusal not a stock result |
| Dependency changed **during** validation / not exercised stock | Root `60705900d06a5b7db36e073a6ff2df6669cfd1a1c38ee77bc326f4c767f33b49` as direct-relative; oracle `native_midcall_dependency_change_invalidates`: **unavailable/context_invalidated**, `dependencies_current=false`, four native reader calls | No controlled stock reader barrier [L]; between-call change below does not prove during-call invalidation |
| Same path baseline / `same_path_before_change` | `60705900d06a5b7db36e073a6ff2df6669cfd1a1c38ee77bc326f4c767f33b49`; oracle baseline **valid** | 0/0 [P]; fresh Script in shared probe process, no cache hit proof |
| Same path changed dependency / `same_path_changed_dependency` | Same root; `native_changed_dependency_parser`: **invalid/analysis_error**, changed dependency parse and root analysis errors | 43/1 [P], staged changed file hash matches oracle; **not** consumed-parser-hash proof |
| Same path restored dependency / `same_path_restored_dependency` | Same root; `native_stale_cache_revalidated`: **valid** with original dependency hash | 0/0 [P]; between-call recovery, not same-bytes inode replacement/selected-script or mid-call proof |

**[O/P]** Matched staged dependency SHA-256s, as distinct from stock **consumed** source hashes: original/restored `level_one.gd` `85512bf987d1331898543783114d896e7763e626d72b52e0741d5f80cba793c1`; changed `level_one.gd` `1bdd0be05611e937e3253a2b4eaaddfd1229cf763f2bddb772fade434c53aaad`; `deep/level_two.gd` `06ca9055ac00ebbb079e08453f05cf7769fe642f7a3f7629192f0844073199cc`; `sibling.gd` `44f2757a9db949d6553af6212254be258bcc00d1851d58317aeed5af3bb684c1`; `context_node.gd` `49573a3b7813f9037bf072db3c81faa25283385f6b8571e9823a42d848ceccc7`; `context_other.gd` `696df6ff63ba0a551d1ff4589aa68e9f48d75fa89b9719a19859c7ae7e61af18`; `context_invalid.gd` `0d6c1deca4f5409bdec722cf176b29790e44aec821dafac50c321cf63d05fb5a`. The oracle recorded these as dependency source witnesses in matched cases; stock staging and before/after disk hashing show file bytes, **not** the generation actually referenced inside engine cache. First autoload Logger interval contains previous-case paths; cached owner edges/`finish_compiling` explain why treating Error 43 as a standalone semantic oracle mismatch would be unsound ([cache][s15-cache]).

**[P/N]** The helper's 22 separate *non-matched* synthetic cases additionally establish public metadata sampling, valid-root/relative/transitive/global/autoload startup controls, four static executions and eight invalid-exit-0 counterexamples. Helper roots differ from T002: e.g. helper valid `81882e4dff96309b53f98da2e0ddfd054a6aa252eff54c0ae77984718eb3f166`, non-tool static `ade9ee96598ac10be1bf9445ab93e837347d915eb0a7794022aaacc2dbe5b4b4`, tool static `77304d97a09c019de3df5a5f6454d9d8f0fc3cac84f1c92f762930c6f7d734b8` ([helper raw summary][s15-helper]). They do **not** add matched oracle statuses or a qualifying stock `valid`. No stock getter/during-call barrier instrumentation and no visible GUI A–E surface proof occurred in this campaign [L]. The earlier P1 GUI evidence in §14 is saved-state evidence only.

### 15.8 Final V2 decision and next experiment

**V2 — validation/effect design hold. [U]** The **next focused technical question** is whether a useful, source-derived admission boundary can keep the *selected target/version's unavoidable stock validation, export and deferred callback chain* on **attributed dependency generations** and prevent forbidden loader/getter/initializer/remote-reload dispatch **before** it happens. This concerns stock handler/editor continuation safety; **independently**, explicit exact-source preflight and post-change B validation still needs a qualifying stock implementation with attributed consumed source/dependency/context generations, completed valid/invalid/unavailable result and effect-confinement proof. These are two proof obligations under the current design hold, not one parser gap relabeled as solved: a separate adapted validator cannot intercept host continuations, and even success on the next callback experiment would be necessary but **not sufficient** to release T003. The current probe contains useful mechanism positives [P] and concrete forbidden effects [N], but **zero** stock full-contract qualifying `valid` results; no production explicit validator is selected. Neither V1 implementation readiness nor V3 exhaustion of a tighter H2 profile/adaptation follows. P1's guarded saved-state order, T002's historical acceptance, the five-task ownership and Feature 002 requirements remain unchanged; do not reopen persistence, choose a patched product, or start T003.

**Single precise next callback-closure experiment [I]:** In one owned actual stock EditorPlugin project, admit an ordinary non-tool native-base root with **one** confined relative `.gd` dependency under a proposed *shared* root/dependency eligibility rule. Record exact preflight/proposed/post-change B/R/D hashes, CodeEdit current/saved versions, selected Script/cache association and dependency identity/hash; first warm hostile/stale metadata at the **same dependency path**, then restore candidate-clean bytes. Invoke the already selected **existing incoming saved-handler Callable**, allow ordinary edit-scheduled `_validate_script`/`get_functions`/export continuation via **public signal/timer topology**, and track public debugger sessions (initially no active remote, then a later connection) plus a newer independent human source change while work is queued. Determine whether supported public admission/refusal and observers can prove each callback's harmless consumption of the *candidate dependency generation*, bound completion and prevention before loader/getter/initializer or remote reload dispatch. If the admitted callback closure is unsafe or attribution/completion is incomplete, **refuse**; do not use after-effect observation, preference toggles, private queue access, a different saved tag, or an implementation of T003 as substitute.

Checked pinned source / raw evidence references for §15:

[s15-reload-entry]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript.cpp#L737-L802
[s15-reload]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript.cpp#L803-L901
[s15-setter]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript.cpp#L446-L457
[s15-property]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript.cpp#L1009-L1018
[s15-validator]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript_editor.cpp#L146-L216
[s15-cache]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript_cache.cpp#L68-L112
[s15-cache-graph]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript_cache.cpp#L217-L259
[s15-finish]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript_cache.cpp#L423-L446
[s15-analyzer]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript_analyzer.cpp#L6623-L6669
[s15-constant]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript_analyzer.cpp#L3141-L3152
[s15-getter]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript_analyzer.cpp#L4816-L4849
[s15-variant-getter]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/core/variant/variant_setget.cpp#L267-L289
[s15-object-getter]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/core/object/object.cpp#L316-L334
[s15-lsp]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/language_server/gdscript_language_protocol.cpp#L407-L545
[s15-lsp-parser]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/language_server/gdscript_extend_parser.cpp#L967-L980
[s15-lsp-save]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/language_server/gdscript_text_document.cpp#L85-L124
[s15-startup]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/main/main.cpp#L4360-L4389
[s15-editor-create]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/main/main.cpp#L4601-L4611
[s15-full]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript_cache.cpp#L347-L404
[s15-disable]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/editor_node.cpp#L8387-L8388
[s15-exit]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/platform/macos/os_macos.mm#L1226-L1272
[s15-os-default]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/core/os/os.h#L89-L91
[s15-platform-exit]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/platform/macos/godot_main_macos.mm#L106-L141
[s15-handler]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp#L723-L774
[s15-trigger]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp#L776-L800
[s15-tag]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_base.cpp#L566-L573
[s15-timer]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/gui/code_editor.cpp#L1696-L1703
[s15-text]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_text_editor.cpp#L841-L897
[s15-editor-methods]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_text_editor.cpp#L210-L238
[s15-exports]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript.cpp#L497-L655
[s15-debugger]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/debugger/editor_debugger_node.cpp#L684-L688
[s15-debugger-send]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/debugger/script_editor_debugger.cpp#L1774-L1780
[s15-debugger-plugin]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/doc/classes/EditorDebuggerPlugin.xml#L6-L10
[s15-public-debugger]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/doc/classes/EditorDebuggerSession.xml#L20-L24
[s15-debug-option]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/debugger/debugger_editor_plugin.cpp#L192-L196
[s15-settings]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/core/config/project_settings.cpp#L1620-L1626
[s15-classdb]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/doc/classes/ClassDB.xml#L35-L269
[s15-contract]: contracts/native-integration.md#4-primitive-b-exact-source-native-gdscript-validation
[s15-oracle]: file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-agent-kit-native-ex09m3nz/native-acceptance/summary.json
[s15-rmain]: file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-stock-validation-effects-a3vtbtw0/reload-probe/evidence/20260928T145448-cbc00d93/controller.json
[s15-rloader]: file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-stock-validation-effects-a3vtbtw0/reload-probe/evidence/20260928T150947-04478d3e/controller.json
[s15-helper]: file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-stock-validation-effects-a3vtbtw0/helper-probe/evidence/summary.json

## 16. Stock saved-handler admission research (2026-09-28)

### 16.1 Scope and evidence discipline

**[S]** This is the admission/continuation investigation promised by §15.8, on **official unpatched** Godot `4.7.2.stable.official.ed1daf0bf` (engine SHA-256 `c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf`), pinned source commit `ed1daf0bf001b61586d9930840f2f1394092c079`, macOS 26.6.2 arm64 and standard public-ABI GDExtension. The seven classes remain **P** demonstrated positive stock runtime, **N** demonstrated negative stock runtime, **S** stock public API/source finding, **O** differential oracle finding, **I** inference needing runtime proof, **U** unresolved technical question and **L** tooling/environment limitation. Every bracket below denotes exactly one class; source inspection never upgrades to observed runtime. No new oracle comparison or validator implementation was performed.

**[P]** The [local provenance][s16-provenance], [source review][s16-source-review], [29-case verified summary][s16-verified], [controller summary][s16-controller] and [four supplemental intervals][s16-targeted] retain checks and raw per-case `result.json`, `steps.json` and `editor.log` under those evidence directories. The main **29** cases completed with zero harness errors and editor exits 0; independent checks covered 58 grouped facts, **not** 58 feature acceptance tests. Two supplemental cases also completed with post-operation screenshots. **[L]** The other two supplemental cases recorded runtime/state/history and then failed only post-window screenshot capture (`visible_owned_window_unavailable`): these are **33 measured intervals, not 33 fully passing harness cases**. Retained evidence is local/uncommitted, not a public downloadable corpus. **[S]** [Cleanup][s16-cleanup] removed the owned probe source, binaries, generated headers, projects, window helper, extracted upstream tree and archive; no owned editor/game process remained. Raw evidence/provenance remains outside product source. The recorded commands are historical and are **not runnable from the retained artifacts** after cleanup.

**[P]** The selected ordinary geometry was one standalone open non-tool `subject.gd` and one in-project `helper.gd` extending stock `Node`, with prior native saved/Undo history; helper static `value()->int` and instance `inherited_value()->int` both returned 31. Alternative roots were `extends Node; const Helper = preload("helper.gd"); func value()->int: return Helper.value()` and `extends "helper.gd"; func evaluate()->int: return inherited_value()`. The prior root returned 17 and proposed source returned 31. The other open `other.gd` was dirty, lacked final newline and retained native Undo/Redo. Main 29 kept **other current**, subject non-current; supplemental current-target and active-session cases change those dimensions explicitly. Independent no-follow descriptor/extension and Python D identity/hash reads corroborated root/helper disk witnesses. No ordinary baseline `@tool`, export, Object-valued constant or non-GD dependency was present.

### 16.2 A — ordered handler and ordinary continuation

**[S]** Ordered pinned-source graph (B = CodeEdit buffer, R = held Script source, D = disk; a path/hash is not a consumed parser generation):

| Order, locality | Branch and source/observed surface |
|---|---|
| 1. Selected synchronous tag | **[S]** The retained **actual** incoming saved-handler Callable receives the held Script. Matching tab Resource pointers tag CodeEdit's current as saved, then record document path mtime; no Script setter, D write or Undo replacement occurs in this native tag ([handler][s16-handler], [tag][s16-tag]). The P1 descriptor write, R setter, edited=false and renewed target/namespace/version guards already precede this call (§14). |
| 2. Sibling scan | **[S]** The handler scans built-in parent-path siblings; a collision can tag another tab, trigger its live path, reload it `true` and update docs. A standalone exact target **with no colliding built-in association** remains a required refusal boundary, not proof of whole-handler purity ([handler][s16-handler]). |
| 3. Global name refresh | **[S]** `_update_script_names` enumerates all tabs and optionally scene-used Scripts, formats/sorts list data, then calls `_update_members_overview` unless the `waiting_update_names` branch suppresses it. This invokes **current** `ScriptTextEditor::get_functions()` → language `validate(current B, current Script path)`, which can read cache/D/dependencies and call loader hooks; current need not be subject ([names][s16-names], [overview][s16-overview], [functions][s16-functions]). |
| 4. Conditional sort/navigation | **[S]** `_sort_list_on_update` can reorder tabs and call `_go_to_tab` (apply unsaved **current** R/exports, navigate, request validation); inspected flag-set sites immediately run a names update. A latent flag in the normal settled case is **not** demonstrated ([names][s16-names], [tab][s16-tab]). |
| 5. Selected live-trigger, synchronous | **[S]** For a target path absent from the **shared** `script_paths_to_reload` list, use `ResourceCache::get_ref` else `ResourceLoader::load`; if Script valid, validate its **R** at path, potentially opening dependencies/cache/loader callbacks **inside the handler**, and return early on invalid. If already queued, skip that validation. Then append path. The auto-reload setting does **not** gate these synchronous steps ([trigger][s16-trigger]). |
| 6. Conditional deferred schedule | **[S]** If not already pending **and** native `auto_reload_running_scripts` is true, schedule `_live_auto_reload_running_scripts`; pending and paths are ScriptEditor-wide, not attempt/version keyed. Its later body clears pending, passes the **shared path list** to `EditorDebuggerNode` and clears the list ([trigger][s16-trigger]). |
| 7. Debugger/remote continuation | **[S]** Node fans out to debugger tabs; only an active peer sends `reload_scripts` with **paths**, not B/hash/version. Remote debugger records paths and on idle obtains cached or loaded Scripts and invokes language `reload_scripts(..., true)` ([node][s16-node], [peer][s16-peer], [peer-send][s16-peer-send], [remote-command][s16-remote-command], [remote][s16-remote]). **[S]** `GDScriptLanguage::reload_scripts` enumerates loaded root Scripts, sorts by inheritance, includes requested roots and inheritors of requested bases, **re-reads standalone D** via `load_source_code(scr->get_path())` (built-in scripts instead reload scenes), then invokes `scr->reload(p_soft_reload)` with `true` on this path. The standalone read uses `FileAccess::open(..., READ)`; `GDScript::reload` can parse, analyze, compile and run static initialization even with soft reload. Neither a prior R/B check nor the path-only remote message binds that later D or its inherited consumers ([language reload][s16-language-reload], [standalone read][s16-standalone-read], [script reload][s16-script-reload]). **[I]** A later changed generation could thus be consumed after the earlier guard. **[U]** No stock request/version-bound remote completion receipt was established; the owned runtime source/output witnesses below remain bounded observations. |
| 8. Independently edit-scheduled callbacks | **[S]** TextEdit B changes start the owning CodeTextEditor's idle timer; timeout calls its empty virtual `_validate_script` hook, then emits `validate_script`. Its connection to the **owning TextEditorBase** invokes that document's validation, including when it is non-current; only later `name_changed` → `_update_script_names` → members/functions follows whichever editor is current at that time. Successful non-tool document validation source-sets **that** Script R, updates exports, connected methods/warnings/UI and pending dragged-export assignments. `update_exports` reparses R, may full-load a base, update placeholders and recurse across historical reverse inherters, even without `@export` in the two current files. Completion timer is conditional on insertion/visibility, not an unconditional `set_source_code` tail ([idle][s16-idle], [idle-timeout][s16-idle-timeout], [idle-hook][s16-idle-hook], [idle-connection][s16-idle-connection], [validation][s16-validation], [signals][s16-signals], [name-connection][s16-name-connection], [exports][s16-exports], [completion][s16-completion]). |

**[S]** Explicit GDScript `set_source_code` only assigns R and marks source changed; it neither certifies parser refresh nor itself enqueues the TextEdit idle timer ([setter][s16-setter]). **[S]** The early tag, current-B function discovery, synchronous selected R validation, independent idle B validation/exports and later path-only remote routing have different targets and clocks; one successful source hash or one timer signal cannot stand in for their combined closure. The §15.5 statement that direct handler did not call `get_functions` and §14's overbroad UI-only interpretation are corrected above, without erasing their earlier observations.

### 16.3 B — minimum evidence before any admission

The following is an **evidence requirement audit**, not an implemented admission rule. “Required” is a contract/causal prerequisite; “restriction” is a conservative possible refusal, not proof of the complement; “unresolved” is missing supported proof. The “before” column asks whether the cited **public** witness can settle the failure **before native dispatch**, not whether a post-effect log can discover it.

| Necessary evidence and status | Concrete failure if missing | Supported observable | Stable across relevant work? | Detect before dispatch? | Needed for useful safe admission? |
|---|---|---|---|---|---|
| **[S] Required:** exact target/CodeEdit/Script, current+saved versions, D/R/B and namespace/retained-fd receipt, no built-in collision | A newer B, wrong receiver or sibling could be tagged after only a path match | **[P]** P1 public versions/identities/source plus independent descriptor bytes/identity and matching Callable (§14) | **[S]** Recheck at final guard and independently after; arbitrary external writer after guard remains outside in-process atomicity | **[P]** Yes for observed state at final guard, not indefinitely | **[S]** Yes; necessary, **insufficient** for parser/deferred effects |
| **[S] Required:** confined identity and exact content of root and every claimed dependency at sampling/final guard, plus proof of which generation is consumed | **[P]** A dependency changed after sampling and before handler; the observed mismatch refused while B6/saved4 stayed dirty | **[P]** No-follow descriptor content/SHA-256 and identity reads; size/mtime were additional **observed change hints**, not independently mandatory fields ([guard race][s16-verified]) | **[S]** Guard samples cannot exclude transient changes restored before observation, an in-handler change or arbitrary writer after the check | **[P]** Yes for the **observable mismatches at the final guard** in the exercised race, not all intervening writes or later consumption | **[S]** Exact content/identity and consumed-generation attribution are necessary; imposing each size/mtime field is not established as necessary |
| **[S] Required:** attributed *consumed* root/dependency parser, shallow/full and context generation at each handler/function/idle/export read | Same pathname, apparently matching B/R and D snapshot can still leave cache provenance unknown | Public Script getter/method metadata and on-disk hash; internal refs/cache/hash lack public consumed generation ([cache][s16-cache], [cache-map][s16-cache-map], [cache-script][s16-cache-script], [reload-hash][s16-reload-hash]) | No; path-keyed status and later callbacks differ | **[U]** No qualifying public per-consumption certificate established | Yes for the stated no-unattributed-consumption guarantee; **unresolved** on stock |
| **[S] Required:** pre-hook closure for ordinary `.gd` loader interactions | **[N]** Registered loader `get_resource_type` executed **inside** handler on source-only preload | **[P]** Controlled callback counter/marker exists only after execution; **[I]** inspecting registration/context could support conservative refusal | **[S]** Callback registry/context and dependencies can change; source scan alone is not a hook-exclusion witness | **[U]** No supported blanket pre-hook proof shown; refusing when registered hook/context is unknown is a **restriction [I]** | **[S]** Yes before any forbidden hook, not by post-hook marker |
| **[S] Required:** full current-editor/function-discovery/ordinary-success export and reverse-consumer closure | Non-target current B is validated; exports can full-load base/propagate to inherited Scripts/placeholders | Current editor/B identity, public idle timer/signal and Script state; no public reverse cache graph or target/version consumption receipt ([overview][s16-overview], [exports][s16-exports]) | Current editor, timer, cached inherters and callback timing may change | Only current snapshot; full closure **unresolved** | Yes; neither “no exports in root” nor subject non-current closes it |
| **[S] Required:** identify live-trigger cache/fallback, shared paths/pending state and actual **effective** native reload mode/session timing | Synchronous validation/load still occurs with off; enabled deferred path can reload active peer later | Public cache/Script identity, menu metadata and debugger sessions; no public internal pending/path queue or effective flag getter ([trigger][s16-trigger], [option][s16-option]) | No: shared queue, menu setter and sessions can change between observations and deferred dispatch | Not as a lifetime/version fence; no supported cancellation/queue receipt established | Yes for no uncontrolled remote effects; “no queue ever” is **not** a requirement if safe exact completion were provable |
| **[U] Required:** same-attempt completion and forbidden-effect exclusion for **all** work, rather than one signal/quiet sample | Handler has already tagged before validation, and later ordinary/newer-human or remote callbacks can run | Public `validate_script` count, stopped timers, sessions, D/R/B/history and bounded runtime marker observations | These are snapshots/event counts, not immutable source/dependency or remote ack | No universal pre-effect fence established | Yes; observe ordinary completion **when safe**, but post-effect observation cannot justify earlier dispatch |

### 16.4 C/D — same-path cache and dependency barrier

**[P]** Seven same-path controls warmed helper A (int) against disk B (String), reversed B/A, preserved mtime, synchronized helper R to D without reload in both directions, and repeated both R/D-aligned variants with literal `extends`. Old cached method metadata persisted as A/B. **[P]** Later target diagnostic **control text** followed **current disk type semantics** (String→int error when D=B, no corresponding label when D=A), including preserved mtime; those controls were not visibly displayed because **other** was current. Evidence: [case summaries][s16-verified] and raw [main cases][s16-main].

**[S]** `get_parser` path hit advances its existing ref without disk/mtime comparison; only a new/EMPTY parse remaps/reads D and records an internal 32-bit source hash. Shallow/full Script caches are separately path keyed; refs can also be destroyed, so an old compiled method list is **not** proof a live parser ref stayed stale. `GDScript::reload` can compare its own R with parser hash and remove it, but that is not a public, safe parser-only refresh or external consumed-hash witness ([cache][s16-cache], [cache-map][s16-cache-map], [cache-script][s16-cache-script], [cache-lifetime][s16-cache-lifetime], [reload-hash][s16-reload-hash]). **[U]** No stale parser consumption was demonstrated, and no public witness ties each completed validation/export to exact parser/shallow/full bytes and dependency/context generation. Avoid both claims “cache was safely fresh” and “stale parser was consumed.”

**[P]** Changing helper after sampling/persistence but before final guard caused SHA/size/mtime refusal (root already applied B6/saved4, dirty): a **partial applied-unverified** attempt, not rollback. **[N]** A controlled loader `get_resource_type` callback changed helper **inside** the saved handler after that guard. **[S]** Internal reads remain possible after the last guard ([preload][s16-preload], [trigger][s16-trigger]); **[U]** no no-unattributed-consumption prevention or generation receipt was established. **[P]** Another helper change after handler return/before deferred work left root unchanged while later ordinary validation produced the String→int diagnostic. **[S]** The in-process last check cannot atomically exclude an arbitrary external writer thereafter ([race cases][s16-verified]).

### 16.5 E/F — reachability inventory and closure

Admission classifications are **“not reachable for admitted profile”**, **“reachable but inert”**, **“reachable and preventable by pre-admission evidence”**, **“reachable and only conservatively rejectable”**, and **“reachable with no qualifying pre-dispatch guard”**. They concern a specified **candidate source/context/path**, never an intrinsic property of a language token. **[U]** There is **no accepted/actually admitted profile** here: the first label is only a *conditional* exclusion if a future complete profile proves that branch unreachable. “Preventable” means refuse a witnessed bad candidate **before** calling the handler, not accept the unwitnessed complement. A distinct timing/evidence column avoids treating a source branch as an executed runtime effect.

| Hazard class | Admission classification at identified candidate/path | Separate timing and evidence boundary |
|---|---|---|
| `@tool` | **[I] reachable and only conservatively rejectable:** a known tool annotation can be refused from inspected root/dependencies; that refusal does not prove transitive context or a non-tool complement safe. | **[P]** Tool marker fired at setup, not anew during that case's handler/deferred interval. **[S]** Full remote reload can initialize runnable scripts; `keep_state=true` is not a no-effect switch ([script reload][s16-script-reload]). |
| Static initializer | **[I] reachable and only conservatively rejectable:** inspect/refuse known initializers where future reload may run them, but a declaration is not an execution or a complete pre-dispatch effect graph. | **[P]** Declared non-tool static marker stayed zero in the bounded pass. **[S]** Remote soft reload can call `_static_init` after parse/analyze/compile ([script reload][s16-script-reload]); earlier §15 reload negatives belong to their own route. |
| `new()` / `_init` constructor or user call | **[I] reachable and only conservatively rejectable:** known construction/user-call paths can be refused; no source-token-only proof closes calls from other contexts or later runtime reload. | **[P]** Constructor remained uninstantiated in the bounded interval. **[S]** Ordinary `Helper.value()` occurs in a subject function body; its declaration alone does not call that body. **[U]** This pass did not establish an in-handler constructor invocation ([language reload][s16-language-reload]). |
| Autoload Object resolution | **[U] reachable with no qualifying pre-dispatch guard:** for a candidate referencing mapped autoload Object/scene, no complete pre-hook Object/loader/context witness was established. | **[S]** ProjectSettings autoload lookup queries resource type, may inspect singleton Object/scene script and raise depended parser status ([autoload][s16-autoload]); **[U]** no autoload runtime sentinel here. |
| Global class resolution | **[I] reachable and only conservatively rejectable:** a known mapped `class_name` reference can be screened out; changing registry/path/context defeats a static root/helper-only list, and accepting the remainder is unproved. | **[S]** ScriptServer global-class mapping produces an analyzer metatype, and typed global references may resolve a depended parser ([globals][s16-globals], [typed dependencies][s16-typed]); **[U]** no separate global-class runtime sentinel. |
| Preload/resource lookup, ordinary `.gd` | **[U] reachable with no qualifying pre-dispatch guard:** the inspected ordinary preload candidate with registered custom loader has no established public pre-hook fence. **[S]** For a *hypothetical completely closed* literal-extends-only profile, its **particular preload-type-query edge** would be “not reachable for admitted profile”; this is not an admitted profile or a guarantee about other branches ([inheritance][s16-inheritance], [preload][s16-preload]). | **[N]** Registered `get_resource_type` ran synchronously on ordinary `.gd` source-only preload. **[S]** `exists` and loader type query precede shallow GDScript dependency lookup ([preload][s16-preload]). |
| Custom loader hooks | **[U] reachable with no qualifying pre-dispatch guard:** a post-callback marker does not prevent an already-executed project hook; a general safe hook/registry/context exclusion has not been proven. | **[N]** Ordinary reentrant `.gd` cases ran **`get_resource_type`** and could change helper or newer human B after tag ([main cases][s16-main]). **[N]** The non-GD sentinel separately ran actual **`_load`** inside handler ([sentinel evidence][s16-sentinel]); these are different observed callbacks. |
| Scripted/dynamic getter | **[I] reachable and only conservatively rejectable:** excluding known non-GDScript Object-valued reductions avoids a known getter path, not all indirect Objects/callbacks. | **[S]** For a constant non-GDScript Object, `Variant::get_named` can reach Object access; GDScript metatype analysis takes a different route ([getter][s16-getter], [s15-variant-getter], [s15-object-getter]). **[U]** No distinct getter runtime sentinel was exercised. |
| Callable invocation (versus descriptor) | **[S] reachable but inert** for analyzer `make_callable_type` and method metadata **alone**: creating a Callable *type descriptor* does not invoke its target ([call/signal types][s16-call-signal-types], [call/signal members][s16-call-signal-members]). **[U] reachable with no qualifying pre-dispatch guard** for an actual unconfined user/loader Callable dispatch on an unresolved path; descriptor-only analysis cannot certify its absence. | **[S]** Utility/method/lambda reductions can create or describe callable values without showing a `.call()` ([callable values][s16-callable-values]); the selected native saved-handler Callable is the deliberately authorized receiver, not permission for arbitrary project calls. **[U]** No distinct dynamic Callable invocation sentinel in this pass. |
| Signal / Object value evaluation | **[S] reachable but inert** for `make_signal_type` and signal member metadata alone: those operations construct a **Signal descriptor**, not emit or invoke a project Signal. **[I] reachable and only conservatively rejectable** when a separately reduced Object value/attribute could dispatch a getter or associated code ([call/signal types][s16-call-signal-types], [call/signal members][s16-call-signal-members], [getter][s16-getter]). | **[U]** No separate project Signal-value evaluation sentinel; native editor `validate_script`/`name_changed` emissions and listeners are **other** queued-work paths (§16.6), not evidence that a project Signal value emitted. |
| Non-GDScript preload/resource | **[I] reachable and preventable by pre-admission evidence** for the **known literal sentinel dependency** if it is identified and refused before handler; this says nothing about undiscovered/indirect resources. **[U] reachable with no qualifying pre-dispatch guard** once that non-GD resource is allowed into the examined handler path. | **[S]** Non-GDScript branch calls `ResourceLoader::load` ([preload][s16-preload]). **[N]** Controlled non-GD sentinel ran `_load` inside handler (six synchronous loader events, 12 later), outside the ordinary two-`.gd` candidate; its `_load` execution is not inferred merely from ordinary `.gd` type-hook events ([sentinel evidence][s16-sentinel]). |
| Dynamic Object constant/member reduction | **[I] reachable and only conservatively rejectable:** reject known unresolved Object-valued constants and indexed/attribute reductions rather than deeming all unannotated non-tool scripts pure. | **[S]** `make_subscript_reduced_value` can call `get_named` or Variant `get` on reduced values, distinct from compiler/runtime user-function dispatch; Object access can invoke getters ([object reduction][s16-object-reduction], [getter][s16-getter], [s15-object-getter]). **[U]** No independent dynamic-Object/stringification runtime sentinel was exercised. |

**[S]** Literal `extends "helper.gd"` directly obtains a depended parser and avoids **only** the preload-specific loader-type query, not cache/context/exports/function discovery ([inheritance][s16-inheritance], [preload][s16-preload]). **[P]** The changed-helper literal-extends control yielded a current-disk-semantic diagnostic, not a consumed parser-generation receipt ([case summaries][s16-verified]). **[S]** No `@export` in the two present source files does not exclude cached reverse inherters, placeholders, full base loads or validation of the other current B ([exports][s16-exports], [overview][s16-overview]). **[N]** In the out-of-profile current-other sentinel, six hooks ran inside handler; the **fixture intentionally** changed other B to version5/saved2 ([other sentinel][s16-other-sentinel]). **[P]** 22 later callbacks and **other's own ordinary idle validation** synchronized R; D/history remained unchanged in that measured interval ([other sentinel][s16-other-sentinel]). Neither B edit nor R synchronization is a demonstrated selected-handler mutation.

**[S]** A finite closure would need to resolve the actual root and transitive helper paths **and** typed annotations/extends chains, `class_name` registry mappings, autoload/project mappings, UID-to-path/translation/import remaps and any non-GD resources before assuming those are the consumed dependencies ([typed dependencies][s16-typed], [inheritance][s16-inheritance], [globals][s16-globals], [autoload][s16-autoload], [preload][s16-preload], [cache][s16-cache], [resource remaps][s16-remaps]). **[S]** The handler's current-other function discovery and later ordinary export full-base/historical reverse-inherter/placeholder traversal are additional consumers, not implied by a forward root/helper hash list ([overview][s16-overview], [exports][s16-exports]). **[I]** Root+one-helper is therefore a useful bounded fixture, not a proven closed dependency/effect graph; literal extends avoids only the preload-specific loader-type query. **[U]** No public pre-dispatch inventory/generation fence for all those consumers is established; refusing unknown closure is necessary, but a static two-file allowlist is not an accepting admission rule.

### 16.6 G — queued-work origins and six admission questions

**[S]** The six questions for **each origin** are detect before dispatch; cancel; suppress effects without breaking normal editor behavior; wait for completion; attribute exact target B/version; and attest consumed dependency generation/effects. “Not established” does **not** demand cancelling *every* normal timer: safe, attributable normal completion could instead be observed, provided forbidden dispatch was prevented **before** it occurred.

| Origin | Detect before? | Cancel? | Suppress safely? | Wait for actual completion? | Exact target/version? | Consumed generation/effects? |
|---|---|---|---|---|---|---|
| **[S]** TextEdit edit → owning document's idle validation → later current-name refresh | Public owning timer state and `validate_script` event; other or newer B can restart | No target-version public cancellation established | Must not discard ordinary human validation | Timer stopped/event count observable, not attempt token | Event has no immutable owning B/hash version; later `name_changed` follows **current** editor | No per-dependency parser/effect receipt ([idle-timeout][s16-idle-timeout], [idle-hook][s16-idle-hook], [idle-connection][s16-idle-connection], [validation][s16-validation], [name-connection][s16-name-connection]) |
| **[S]** Saved-handler names → current `get_functions` | Current editor snapshot observable; conditional overview call may run synchronously | No per-call public skip on retained handler | Skipping current editor behavior not selected | Synchronous return is not consumed-generation proof | Current B may be **other**, not target | No hash/gen or loader pre-effect interceptor ([names][s16-names], [overview][s16-overview]) |
| **[S]** Saved-handler selected live-trigger/ScriptEditor shared path queue | Public target cache/Script snapshot; internal list membership opaque | No public path-entry cancellation established | Native off does not suppress sync validate/load | Callable return covers sync call only; deferred queue separate | Shared path entry has no attempt/version | No internal consumed hash/effect witness ([trigger][s16-trigger]) |
| **[S]** Successful editor validation → exports/base/reverse inherters/signals | Current B and timer known, private reverse graph unknown | No supported target-version cancellation established | Suppressing normal export propagation not selected | Public timer/signal and state sample, not reverse graph drain | Later editor may differ and callbacks can restart | No complete reverse consumer/dependency receipt ([validation][s16-validation], [exports][s16-exports]) |
| **[S]** Deferred ScriptEditor → debugger peer → remote idle | Active sessions observable **at sampling**; shared pending/path list not exposed | No public already-queued remote cancellation established | Effective off is not proven for arbitrary queued work; changing global settings is not product policy | Session started/stopped and bounded remote probes, no stock ack/drain | Protocol transmits path alone | Remote idle may load/reload later generation ([trigger][s16-trigger], [remote][s16-remote]) |
| **[S]** Conditional completion timer and name/signal listeners | Insertion/visibility conditional; listener set/context may vary | No per-attempt cancellation established | Blanket disabling completion/other consumers not selected | Timer state and later signals observable | No universal target B/hash envelope | No consumed dependency/effect receipt ([completion][s16-completion], [signals][s16-signals]) |

### 16.7 H/I/J — races, remote sessions, unrelated state

**[P]** A newer human B before handler (7/4) and an equal-text-but-newer B (9/4) were both **refused** by current/saved-version guards. Human edit after tag, post-return/pre-deferred, on public validation timeout and within reentrant loader left **newer B7/saved6 dirty**; old AI text was not restored in those measured windows. Reentrant edit occurred **after** native saved tag, not an invented pre-tag hook. The queued-callback case observed validation count **1→3** (old then human restart), while ordinary cases observed **1→2**: real public events, not exact-version completion certificates ([race cases][s16-verified], [main cases][s16-main]).

**[P]** Ordinary preload and literal extends yielded guard6/4→native tag6/6, D=R=B and preserved D identity; prior native saved history was reachable by Undo, then original2, and Redo restored current. Other normal B3/saved2, dirty/no final newline and Undo/Redo survived through handler, next frame, validation signal, stopped timer and stable following sample. **[L]** These checks end at explicit pre-quit/history windows; final controller D after QUIT may reflect Godot shutdown/ordinary saves, not new close/reopen persistence proof. **[P]** The earlier §14 P1 Save/reopen record is separate ([main cases][s16-main], [verified summary][s16-verified]).

**[P]** Main 29 included reload metadata false/true × session inactive/active, but active-session startup **auto-saved other before setup**; its within-call stability did not prove dirty/no-newline preservation. Four supplemental cases started game **before** preparing other dirty and preserved that state/history across operation. **[N]** Active owned session, target already current, literal extends, **project metadata=false and native menu item's public `is_item_checked` getter false** still produced actual runtime 17→31 and matching new remote Script SHA within 6.62 seconds ([metadata-false][s16-metadata-false]). The false checkbox is a getter observation, **not** a screenshot of an expanded menu. **[S]** Native flag starts true and startup menu initialization only toggles through setter when saved metadata is true, explaining why false preference/menu getter is not an effective-mode witness ([default][s16-default], [option-startup][s16-option-startup]).

**[P]** In disposable project setup only, actual public menu `id_pressed` actions toggled opposite→desired to exercise the native setter: effectively off retained old source/output 17 for observed 5.95 seconds; effectively on showed new source/output 31 in 6.14 seconds. An ordinary current-target/default case showed 31 in 7.05 seconds. Object IDs, independent SHA/text/output and history were checked in their recorded intervals ([targeted][s16-targeted]). **[L]** Off/on post-window screenshots were unavailable (`visible_owned_window_unavailable`); prior runtime/state receipts still exist, but no visual post-window screenshot is claimed. **[U]** No stock remote exact-version acknowledgement, internal queue-drain token, effective-flag getter or later-session-attach-at-deferred-barrier trial established a *lifetime* inactivity/off fence. Changing global/user settings or disabling normal editor behavior is not selected.

**[N]** Registered loader hooks ran on plain `.gd` preload synchronously inside handler; controlled non-GD sentinel also ran its loader's actual `_load`, not merely `get_resource_type` ([sentinel evidence][s16-sentinel]). **[N]** In the out-of-profile current-other sentinel, the fixture itself **intentionally edited other B** (version5/saved2) ([other sentinel][s16-other-sentinel]). **[P]** Its own ordinary later idle validation synchronized that document's R; neither B edit nor R synchronization is attributed to the selected saved handler. **[P]** Ordinary two-file runs kept unrelated dirty/no-newline source, versions and native history unchanged through the observed window ([verified summary][s16-verified]). **[I]** These bounded facts prove neither that every P1 save mutates unrelated documents nor that all unrelated continuations are pure.

### 16.8 K/L — required admission matrix and decision

| Condition/hazard | Reachable minimum? | Public predispatch witness? | Safe to admit? | Evidence |
|---|---|---|---|---|
| Tool/static init | **[P]** Absent from ordinary two-file fixture; tool marker only setup, non-tool marker zero | **[I]** Source refusal can flag declarations across inspected graph, not certify all indirect effects | **[U]** Not an accepted positive complement | [hazards][s16-verified]; [preload][s16-preload] |
| Constructor/user call | **[P]** Constructor declared, not instantiated; body invocation not handler execution | **[I]** Tokens/declarations alone do not establish call reachability | **[U]** Require proof before a reachable user dispatch | [hazards][s16-verified]; [remote][s16-remote] |
| Dependency generation mismatch | **[P]** Same-path A/B and pre-guard race exercised; disk diagnostics followed D | **[S]** D hash/identity and R/method list public, **not** parser consumption hash | **[U]** No attributed consumed-generation proof | [cache][s16-cache]; [cache cases][s16-verified] |
| Custom loader | **[N]** Yes: ordinary `.gd` preload invoked hook inside handler | **[S]** Callback registration/context can be conservatively screened, no proven per-dispatch pre-hook intercept | **[U]** Not admitted with unknown hook closure | [preload][s16-preload]; [loader cases][s16-verified] |
| Autoload/global Object | **[S]** Ambient resolution can extend graph; no runtime case here | **[U]** No complete public context/Object purity certificate established | **[U]** Unresolved, not presumed executed | [inheritance][s16-inheritance]; [getter][s16-getter] |
| Queued validation | **[P]** Ordinary 1→2, newer human restart 1→3 | **[S]** Signal/timer observable, not version/gen certificate | **[U]** Normal completion may be observed if safe; missing closure prevents acceptance | [idle][s16-idle]; [race cases][s16-verified] |
| Function discovery | **[S]** Direct names tail calls current `get_functions`, including other-current case | **[S]** Current identity/B public at instant, not future loader/dep closure | **[U]** Two-file target-only rule insufficient | [names][s16-names]; [overview][s16-overview]; [functions][s16-functions] |
| Export propagation | **[S]** Successful non-tool ordinary validation can reach base/reverse cached inherters | **[S]** No public full historical reverse graph/generation receipt established | **[U]** “No current @export” insufficient | [exports][s16-exports]; [validation][s16-validation] |
| Debugger/live | **[N]** Active metadata=false/menu **public getter unchecked** current-target still remote 17→31 | **[S]** Sessions/menu snapshots but no effective native flag/queued path version getter | **[U]** Off preference or entry-time inactive not enough | [metadata-false][s16-metadata-false]; [option][s16-option]; [trigger][s16-trigger] |
| Unrelated mutation | **[N]** Out-of-profile fixture **intentionally changed other B**; later ordinary idle validation synchronized R. **[P]** Ordinary other preserved in measured window | **[S]** Current identity/state visible; sort/reverse/listener closure not certified | **[U]** No handler-caused B edit shown by the sentinel; no blanket safety claim | [verified summary][s16-verified]; [names][s16-names]; [exports][s16-exports] |
| Newer human | **[P]** 7/4, equal-text 9/4 refused; after-tag B7/saved6 retained | **[P]** Public versions and B/source detect before final guard and after | **[U]** Local guard success does not certify future callback/version safety | [race cases][s16-verified]; [idle][s16-idle] |

**[P]** The ordinary preload and literal-extends positives demonstrate meaningful existing saved-state mechanics and controlled ordinary follow-on observations. **[U]** They do not establish an admitted safe source profile; no dependency-free positive profile was established by this campaign either. The current minimum example has a dependency, while direct handler/current editor/runtime paths remain to be closed even if a future profile omits one. **[S]** Literal extends avoids one preload loader-type dispatch ([inheritance][s16-inheritance], [preload][s16-preload]). **[U]** No supported tail separation/replacement or full safety proof follows.

**[I] Necessary-but-insufficient, reject-unknown screen only:** require fresh P1 target/identity/namespace/D-R-B/version/receipt/collision checks; bound root and every discovered dependency/context and its actual consumed generation, exclude or pre-confine loader/Object/user dispatch, close current-editor and historical export consumers, and account for each ordinary/deferred callback's exact target and generation before effects. This is a list of **necessary conditions**, **not** a callable `ADMIT` predicate. **[U]** Public consumed-generation/pre-hook/reverse-graph/shared-queue/lifetime witnesses remain unresolved. **[I]** Conservative pre-refusal of unknown/unsupported branches is a possible **restriction**, not an accepted nonempty complement; neither turning all operations `unavailable` nor accepting after a quiet interval meets the contract.

**H3 — no useful safe saved-handler admission profile established for the demonstrated stock/public-observer composition. [U]** Independent blockers remain dependency/parser/shallow/export generation and reverse-consumer attribution; current-editor/function-discovery/ordinary validation and export callback closure; and shared deferred runtime routing/completion. This is **not** H2's one-parser-problem diagnosis, nor proof that every future supported-API design is impossible. No accepted production admission rule, T003 implementation or T003 readiness follows. **[S]** P1 saved-state route stays selected **unchanged** (§14); do not substitute a patched editor, Save/Save All, ResourceSaver, signal broadcast or unrelated settings policy. **[U]** Future supported-API separation/replacement of the unsafe tail remains an open research question, not an implementation selection. Explicit exact-source `valid`/`invalid`/`unavailable` validation with attributable diagnostics/context/dependencies remains **separately unresolved** (§15); this section does not waive it. Under Principle XIII, do not add an effect system, compiler, dependency database, cache manager, scheduler, debugger or collaboration framework to manufacture a profile; keep T001/T002 complete, all five tasks unchanged, T003–T005 pending and **T003 on design hold**.
**[S]** The [plan's saved-handler admission design review](plan.md#saved-handler-admission-design-review) records the actual read-only `/speckit.analyze` result, complete requirement ownership and corrected documentation findings. Markdown/local/source-anchor checks and disposable-artifact cleanup passed; these are research publication checks, not a passing implementation/validation gate.

**Current §17 qualification:** “P1 stays selected” above records §16's then-current saved-state candidate, **not** a later admission decision. M3 holds that P1 candidate under H3 while the tested no-handler alternative separately fails ordinary Save/history; §17 does not declare the handler necessary.

Pinned stock source and retained local evidence for §16 (file URLs are local provenance, **not** durable public downloads or runnable reproduction instructions):

[s16-provenance]: file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-saved-handler-admission-vz8_wz7_/evidence/provenance.json
[s16-source-review]: file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-saved-handler-admission-vz8_wz7_/evidence/source-review.json
[s16-verified]: file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-saved-handler-admission-vz8_wz7_/evidence/verified-runtime.json
[s16-controller]: file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-saved-handler-admission-vz8_wz7_/evidence/gui-20260928-165950-80f5af/controller-summary.json
[s16-main]: file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-saved-handler-admission-vz8_wz7_/evidence/gui-20260928-165950-80f5af/
[s16-targeted]: file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-saved-handler-admission-vz8_wz7_/evidence/verified-targeted-runtime.json
[s16-metadata-false]: file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-saved-handler-admission-vz8_wz7_/evidence/gui-20260928-180921-b1c81b/
[s16-sentinel]: file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-saved-handler-admission-vz8_wz7_/evidence/gui-20260928-165950-80f5af/loader_sentinel/result.json
[s16-other-sentinel]: file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-saved-handler-admission-vz8_wz7_/evidence/gui-20260928-165950-80f5af/hazard_other_current/result.json
[s16-cleanup]: file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-saved-handler-admission-vz8_wz7_/evidence/cleanup.json
[s16-handler]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp#L723-L774
[s16-tag]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_base.cpp#L566-L573
[s16-names]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp#L1897-L2123
[s16-overview]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp#L1762-L1770
[s16-functions]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_text_editor.cpp#L210-L224
[s16-tab]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp#L360-L431
[s16-trigger]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp#L776-L800
[s16-node]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/debugger/editor_debugger_node.cpp#L684-L688
[s16-peer]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/debugger/script_editor_debugger.cpp#L79-L88
[s16-peer-send]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/debugger/script_editor_debugger.cpp#L1778-L1780
[s16-remote]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/core/debugger/remote_debugger.cpp#L690-L728
[s16-remote-command]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/core/debugger/remote_debugger.cpp#L530-L533
[s16-language-reload]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript.cpp#L2470-L2566
[s16-script-reload]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript.cpp#L775-L887
[s16-standalone-read]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript.cpp#L1124-L1162
[s16-idle]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/gui/code_editor.cpp#L1008-L1018
[s16-idle-timeout]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/gui/code_editor.cpp#L1696-L1703
[s16-idle-hook]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/gui/code_editor.h#L228-L230
[s16-idle-connection]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_base.cpp#L646-L653
[s16-name-connection]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp#L2309-L2315
[s16-validation]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_text_editor.cpp#L841-L897
[s16-exports]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript.cpp#L497-L655
[s16-signals]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_base.cpp#L522-L525
[s16-completion]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/gui/code_editor.cpp#L1021-L1055
[s16-setter]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript.cpp#L446-L454
[s16-cache]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript_cache.cpp#L68-L112
[s16-cache-map]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript_cache.cpp#L217-L242
[s16-cache-script]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript_cache.cpp#L303-L406
[s16-cache-lifetime]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript_cache.cpp#L141-L147
[s16-reload-hash]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript.cpp#L775-L803
[s16-preload]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript_analyzer.cpp#L4723-L4785
[s16-inheritance]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript_analyzer.cpp#L437-L449
[s16-getter]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript_analyzer.cpp#L4816-L4849
[s16-call-signal-types]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript_analyzer.cpp#L84-L103
[s16-call-signal-members]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript_analyzer.cpp#L4297-L4318
[s16-callable-values]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript_analyzer.cpp#L4654-L4713
[s16-globals]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript_analyzer.cpp#L4572-L4575
[s16-autoload]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript_analyzer.cpp#L4577-L4614
[s16-typed]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript_analyzer.cpp#L775-L833
[s16-object-reduction]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript_analyzer.cpp#L5369-L5404
[s16-remaps]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/core/io/resource_loader.cpp#L1442-L1539
[s16-option]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/debugger/debugger_editor_plugin.cpp#L189-L195
[s16-option-startup]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/debugger/debugger_editor_plugin.cpp#L232-L267
[s16-default]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp#L3818-L3819

## 17. Minimal behavioral finalization research (2026-09-28)

### 17.1 A/B — question, method, provenance and evidence classes

**Question:** Can a stock official-editor one-target edit, explicit Script source synchronization and descriptor persistence finish by setting the Resource edited flag false and calling public `CodeEdit.tag_saved_version()` **without invoking any ScriptEditor saved handler**, while preserving later ordinary Save/history, newer human work and unrelated documents? This is an alternative **experiment**, not a replacement production route, a private ABI patch, a new validator or T003 implementation. [P] means demonstrated positive stock runtime; [N] demonstrated negative stock runtime; [S] pinned official public API/source finding; [I] inference needing further runtime proof; [U] unresolved technical question; [L] environment/tooling or evidence limitation. Distinguish each from feature acceptance and from §15's historical oracle class.

On branch `research/002-minimal-behavioral-finalization` from `f521baa22788a39a80f7e7f9a6228bdb16ccfa32`, owned disposable GUI projects used official **4.7.2.stable.official.ed1daf0bf** (binary SHA-256 `c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf`, [pinned source](https://github.com/godotengine/godot/tree/ed1daf0bf001b61586d9930840f2f1394092c079)), macOS 26.6.2 (25G83) arm64, Apple clang 21.0.0/SDK 27.0. [Local provenance][s17-provenance] records official generated header/interface/extension-API SHA-256 `640b48188708ba0016f8d7ace9e0e1d3279a41fa1226c59ff3193b15538bd254` / `7d8c0a039d9743eb8ebf88681ae0c641d8d3aa5ffca11081745a84da803e09a1` / `d0e4c08c03b165156dabe6bfb6a906baf0069189f62035341230a246c86d6986`, probe C++/fixture/controller hashes, per-campaign library hashes and artifact hashes. The first [13-case summary][s17-initial] used library SHA `e8209a6e920f762226c3d96095e8dd9ff24602883f984ec168adfc066fece6e9`; [corrected lifecycle][s17-corrected] (four usable cases; one excluded failed setup) and [corrected valid unrelated][s17-valid] used `4f30ba0f32409aa71c59c9815f0cfa8d79b2fe0bd4720f6ac0e8887061bccdcf`. The [31 initial checked facts][s17-initial-facts] and [14 corrected checked facts][s17-final-facts] are **fact checks, not passing acceptance cases**. Per-case timelines, stage JSON, editor logs, runtime JSONL, screenshot and separate no-follow disk witnesses are retained under those local evidence directories; local `file://` references are not public durable downloads or executable reproduction after throwaway source/library/project cleanup. Parent inspected the actual GUI screenshots, including the [later-Save dialog][s17-dialog], and independently checked final facts.

**Exact tested order [P]:** prepare a clean target from original `17` by one previous native text edit to saved `18` (D=R=B=18, CodeEdit current/saved `4/4`, earlier Undo entry retained); bind the exact Script/ScriptEditorBase/CodeEdit IDs and association, current and saved versions, original D/R/B/source and namespace. For candidate `23`, use one native CodeEdit `begin_complex_operation` → `remove_text` → `insert_text` → `end_complex_operation` (B=23, `6/4` dirty), verify fresh B/version/old-R guard, invoke bound `Script.set_source_code(23)` and read R, then no-follow project/file descriptor `pwrite`/`ftruncate`/`fsync`/`pread` with readback, actual identity/attachment and attempt-local receipt (D=23). Recheck receipt, exact binding/association, B/R/source, post-edit current and prepared saved versions and namespace; invoke `EditorInterface.set_object_edited(script, false)`; recheck guards; call public `CodeEdit`/`TextEdit.tag_saved_version()` **directly**, then **STOP**. No handler Callable discovery/call, `resource_saved` emission, ResourceSaver, stock Save/Save All, private field or mtime write, source reload, signal broadcast or validation service inside the candidate. Later human Save/Undo/Redo/reopen steps are deliberately separate lifecycle probes, not candidate effects. The write is genuinely one retained-fd target persistence, not an event/marker-only clean simulation.

### 17.2 C/D/E — immediate positives versus actual later failures

| Boundary / evidence | Independently measured outcome, not a stronger guarantee |
|---|---|
| [P] Immediate current and noncurrent target ([initial][s17-initial], [owned current screenshot][s17-immediate-visual]) | D=R=B=23; CodeEdit `6/6` clean, public Resource edited=false; exact IDs, selected tab/focus and unrelated state retained. A native finish receipt reports **pre-tag** `6/4`, so independent post-finish getters, not that receipt, witness `6/6`. One 57-byte pwrite, stable file inode/mtime/ctime *during finalization*, target save-event delta 0 (setup target count 1), other/selection counters unchanged. Counters are scoped observations, not proof of engine-wide non-dispatch or general purity. Initial current GUI screenshot shows subject clean and other dirty. |
| [N] Later human text + real shortcut ([initial human][s17-human], [visual dialog][s17-dialog]) | Human B=29 at `8/6` stays newer/dirty; actual foreground OS Cmd+Alt+S produces **“Files have been modified outside Godot”** for `subject.gd`, not a target saved event. D remains 23, immediate R=23 and settled R=29. The dialog does not lose B, but blocks ordinary persistence of the newer human revision. Both immediate and settled human cases exhibit this outcome. |
| [N] Natural timestamp-second + stock control ([corrected natural][s17-natural], [control][s17-control]) | Before/after descriptor mtime `1790622053`→`1790622057` seconds **naturally**, without `utime` or timestamp preservation. Public `Input.parse_input_event` for Cmd+Alt+S is witnessed as native stock input (input delta 1/save delta 0); after human B29, D23/R23/B29, `8/6` dirty and same dialog, later R29. Control has the same ordinary preparation and human B29 but **no minimal descriptor write/tag**: shortcut input delta 1/save delta 1, D=R=B29, `6/6` clean, **no dialog**. This isolates a concrete behavioral failure rather than an asserted private numeric mismatch. |
| [N] Actual Undo → Save → Redo → Save ([corrected history][s17-history]) | Candidate23, native Undo reaches B18, `4/6` dirty with D/R23. First delivered ordinary Save raises the external-change dialog and **does not persist 18**. Explicit fixture Escape/Cancel dismisses it without repair. Redo restores23, `6/6`; next delivered Save completes D=R=B23, `8/8`. Two following Undo operations yield 23/version6 then 18/version4, **not** the expected prior17 in two steps: an extra history step appeared. Redo/Redo returns23. Do not claim 17 was destroyed: initial history campaign separately reached prior17 with Undo/Undo, but its intervening OS Save keys were **undelivered**, so it is not a passing Undo/Save/Redo/Save witness. [S] `_reload_scripts` calls `reload_text`; its `set_text` provides a source-supported explanation for the extra entry, not instrumented private-history attribution ([reload][s17-reload], [reload text][s17-reload-text]). |
| [P/L] Reopen ([initial reopen][s17-reopen]) | Normal `close_file` removed exact prior association; public `edit_script` reopened new ScriptEditorBase/CodeEdit from D, showing D=R=B23, `2/2` clean, no observed dialog/text loss. Its later OS Save was **undelivered**; do not credit it as later Save proof. |

**Classification: M3, F-blocking [N/S/U].** Mandatory later human ordinary Save of guarded persisted edits and the native Undo → Save → Redo → Save behavior fail in the tested no-handler route. A visually clean post-tag state and successful reopen do not rescue a false reconciliation prompt that prevents saving newer human work. [S] `_test_script_times_on_disk` compares remembered **document mtime** to path mtime, not D text to R; on mismatch it prompts if auto-reload is disabled **or** document unsaved, otherwise calls reload; `save_current_script` returns on that test **before** format/save ([check][s17-check], [save][s17-save]). [S] Public edited=false only sets the Object flag, and CodeEdit tag only sets saved version ([edited][s17-edited], [code tag][s17-code-tag]); native document tag additionally reads path mtime ([document tag][s17-document-tag]). This documents **concrete behavioral relevance of the document timestamp**, not an obligation to expose, set or numerically equalize any private mtime/Resource cache field. No bounded supported narrow repair was established below. M3 does **not** prove all handler-free designs impossible, mandate the whole handler, or turn measured negatives into product acceptance.

### 17.3 F/G/H/I — races, unrelated documents, runtime and limits

| Boundary | Demonstrated stock outcome and limitation |
|---|---|
| [P] Human races ([initial race cases][s17-initial]) | Before persistence, B29/current8/saved4 refuses with `current_version_changed`, D18/R23/B29 and **zero descriptor write calls**; partial R/B change is acknowledged. After persistence, newer B29 refuses tag, D23/R23/B29 `8/4` dirty. Even same visible text B23 at newer current8/saved4 refuses tag. Immediately **after** successful tag, human B29/current8/saved6 stays dirty. No rollback/replay or silently saved newer human text; not a blanket race-free/atomic writer claim. |
| [P] Other invalid no-final-newline ([initial][s17-initial]) | Separate other D/R70, invalid unsaved B71 containing `var =`, `4/2` dirty, retains exact text/IDs/version/history, selection/focus and disk across candidate. Other native Undo→70/version2 and Redo→71/version4 still work; candidate did not repair/reformat its error. |
| [P] Other **valid** no-final-newline ([corrected valid][s17-valid-case]) | Let *ordinary own idle validation* settle **before** target candidate: other D70/R71/B71, version `4/2` dirty, no terminal newline. Exact other D/R/B, Script/editor/CodeEdit IDs, versions/dirty/history flags, caret/selection/focus and target IDs remain unchanged across candidate and stable samples; its native Undo/Redo restores its own history, R71/B71. No forced unrelated setter and no candidate-caused final newline. Subsequent *human stock Save* can separately add a newline/update R/history (§14/corrected history); do not charge it to the candidate interval. |
| [P/L] Active runtime ([corrected active][s17-active], [initial runtime][s17-initial]) | Owned game launches **before** other dirty setup. Candidate editor D/R/B23 clean `6/6`; 24 samples keep same running PID 54104/Script identity/source/value18 over ~5.213–11.084 seconds; no automatic hot update observed and no editor-source corruption. Corrected [`/026`][s17-active-save] attempted a single-script human29 Save but the dialog blocked it (D23); [`/028`][s17-active-stop] normally stopped the game with D23 and did **not** Save. The normal [`/029`][s17-active-relaunch] relaunch **itself** performed stock run-time autosave: target D23→29 and save-event count 1→2, **other** D70→71 with count 0→1; newly launched PID 55345 then ran29. This is **not** a successful later single-script Save, nor evidence of candidate noninterference with the other document during relaunch. **Separate initial** clean relaunch without human29 runs23 at PID14105; never combine it with corrected-run relaunch29. Running-process immediate hot reload is **not** a spec requirement; fresh-launch durability and no forbidden callback effects remain requirements. Samples are not a debugger queue-drain, consumed-source-generation, all-session or hot-reload guarantee. |

**Excluded/limited evidence [L]:** the first campaign's disposable native-stage field serialized as a boolean (not a native-success gate); use outer action/stage labels and independent editor/disk surfaces, and the corrected campaign's proper stage results. One corrected valid-other *setup* failure is excluded, replaced by the separate completed valid fixture. An “operation completed”/fact count does not mean lifecycle acceptance. **All five initial undelivered OS Save receipts** (`returncode=3`, `posted=false`) are [`history/021-save18`][s17-undelivered-h21], [`history/023-save23`][s17-undelivered-h23], [`reopen/022`][s17-undelivered-reopen], [`natural-timestamp/023`][s17-undelivered-natural], and [`stock-control/014`][s17-undelivered-control]; none proves either Save success or Save failure. Initial later-human-immediate and [later-human-settled][s17-human-settled] instead have **delivered OS** shortcuts and dialog/input/visual/disk witnesses ([initial human][s17-human]); corrected Save/Escape uses public `Input.parse_input_event` to reach the **stock shortcut**, not an OS-event delivery campaign ([corrected history][s17-history], [corrected natural][s17-natural], [control][s17-control]). Both delivery paths can ground the particular witnessed outcome; never promote an undelivered receipt or a completed flag to a general pass. A captured receipt and editor getter are not independent disk proof; retained no-follow Python disk witnesses supply separate D/identity observations. Screenshot absence cannot establish invisible callback absence; save-event/input counters cover only registered observers. No final wire/API/schema, `Resource` numeric mtime, engine-wide callback parity, or execution-proof of a validator is inferred.

### 17.4 J/K/L — narrow repair audit and behavioral decision

**Supported narrow-remedy audit [S/U]:** `ScriptEditor.reload_open_files()` performs the time check and external-editor update, not a target document timestamp-only repair ([reload entry][s17-reload-entry]). `reload_scripts`/`_reload_scripts` traverses open documents; on differing mtime it loads source, sets R, calls `reload(true)`, `reload_text`, and updates names, while `refresh_only` still traverses all documents and has no target-tag-only public contract ([reload][s17-reload], [reload text][s17-reload-text]). It can alter B/history and broaden effects; it is not an allowed no-B-reload narrow fix. `ScriptEditorBase` public binding exposes `add_syntax_highlighter`/`get_base_editor`, not its internal document tag; the latter updates remembered mtime but lacks supported public ABI ([binding][s17-binding], [document tag][s17-document-tag]). `ScriptEditor.update_docs_from_script`/`clear_docs_from_script` update documentation, not document time ([docs][s17-docs]); `EditorFileSystem.update_file` updates filesystem metadata/scanning, not this open editor's remembered time ([file system][s17-file-system]). Broad Save/Save All, signal broadcast, ResourceSaver fallback, close/reopen that discards live identity/history, timestamp fakery, private ABI or disabling native checks violate earlier boundaries; the preserved §13–§16 record gives their own independent reasons. **No qualifying bounded supported repair was established**, not a claim none can ever exist. The historical direct saved-handler P1 avoids this *specific* immediate document-tag gap, but remains held by H3; this result does not license its unsafe tail or establish full-handler necessity.

**J — classification under the current saved-state/native-finalization requirement.** M3 is a **behavioral** blocker in the tested no-handler route: FR-015/FR-016/FR-017, US4.1–US4.3/US5.1 and SC-005/SC-006 demand the actual history, Save and fresh-launch outcomes below, not private bookkeeping parity. This does not establish that the earlier source17 history was destroyed or that the whole saved handler is necessary.

| Concrete invariant or proposed mechanism | One classification | Requirement and evidence relevance |
|---|---|---|
| Exact bound open target/namespace and fresh current **and saved** versions; protect newer human B before persistence/tag, refusing stale retarget/overwrite without false rollback. | **observable required behavior** | FR-001–FR-004, FR-013; US3.5, US5.3. [Race cases][s17-initial] demonstrate zero-write pre-persistence refusal and truthful partially applied D/R/B on later races; they do not prove arbitrary atomicity. |
| Intended D=R=B23 and genuinely clean target after candidate; one reversible native edit with earlier history intact, real Undo → Save → Redo → Save converging each D/R/B state. | **observable required behavior** | FR-006, FR-015–FR-016; US4.1–US4.3; SC-005. [Immediate][s17-initial] gets `6/6`, but [corrected history][s17-history] has blocked first Save and extra history entry; initial two Undos reached17 without delivered intervening Saves, not a passing lifecycle. |
| Later ordinary single-script Save without false reconciliation, close/reopen, reparse/rescan and permitted fresh launch retain the intended edit; **not** active-process hot reload. | **observable required behavior** | FR-017; US5.1–US5.2; SC-006. [Initial human][s17-human] and [corrected natural][s17-natural] fail later Save, [reopen][s17-reopen] proves only clean reopen, and [corrected active][s17-active] runs29 only after relaunch's own autosave, while the separate initial relaunch runs23. |
| No forbidden callback/execution or unintended unrelated edits, confinement and honest `rejected`/partly-applied/unknown reporting once effects occurred. | **observable required behavior** | FR-005, FR-010–FR-013, FR-020; US3.1/US3.3/US3.5, US5.5. [Valid-other][s17-valid-case] and [races][s17-initial] delimit candidate-interval noninterference/partial D/R/B, not engine-wide purity or a pass for relaunch's other-document autosave. |
| Fresh independently attributed final D/R/B, clean getter and separate no-follow disk/identity observation; an attempt receipt or pre-tag `6/4` finish receipt cannot stand in for post-tag `6/6` or independent D. | **independent-verification requirement** | FR-007–FR-008, FR-010, FR-022; US3.1, US4.1, SC-005. [Immediate][s17-initial] supplies post-finish getters and independent disk evidence, not a completed lifecycle; Save receipt needs actual delivered input and observed event/dialog/D. |
| Separately qualifying **explicit exact-source validator**: completed parser/analyzer on requested bytes/context for `valid`, attributable semantic rejection for `invalid`, and `unavailable` for incomplete/unsupported/confinement/context failure, with preflight/post-change and forbidden-effects boundary. | **independent-verification requirement** | FR-009, FR-022; US3.4, SC-003. Still **unresolved independently** of finalization; neither clean D/R/B nor this probe implements/proves it. Source-generation attribution matters only where actual validation or forbidden callback effects depend on it. |
| Invoke the old saved handler, discover incoming `Callable`, reproduce saved-handler callbacks/live-deferred generation and native Save choreography as a fixed recipe. | **historical implementation assumption** | §14–§16 P1/H3 concerns *one* held implementation of FR-006/FR-015–FR-017 and SC-005/SC-006. The alternative [corrected history][s17-history] fails those outcomes; that does **not** prove the full handler or its risky tail required or safe. |
| Numerically equalize private `Resource`/document mtime, cache generation or callback-history counters, expose writable private timestamp or demand matching internal Save transitions. | **internal parity requirement without behavioral justification** | FR-015–FR-017; US4.1–US4.3/US5.1; SC-005/SC-006 specify outcomes, not numbers. **Exception in relevance, not category:** actual document-mtime mismatch triggers the [false ordinary Save dialog][s17-human] via [stock check][s17-check], so preventing that *behavior* is required; equality of private fields, arbitrary cache/callback parity and a whole-handler call do not follow. |

**K — intermediate states.** During a guarded attempt B may precede R, R may precede D, and internal bookkeeping may transiently differ; do not demand global atomic equality at each step. Check stale/current-and-saved-version, binding/namespace, protected persistence and irreversible forbidden-effect boundaries **before** their effects, and verify final behavior independently. D23/R23/B29 after a human race is a truthful partially applied/non-success result, not “nothing happened” or permission to retag/overwrite human B. No false clean success, unauthorized retarget, unrelated mutation, stale overwrite or unbounded result follows from permissive intermediate divergence.

**L — contingent complexity cleanup.** Had a qualified no-handler route met these behavior/effect obligations, production could drop incoming Callable discovery/topology/admission, saved-handler function-discovery and live/deferred-generation safety machinery **associated only with that route**, while retaining guards, writer, validation, ordinary-editor effect coverage and Rust classification. The **tested** alternative did not pass; no unconditional cleanup, whole-handler tail reintroduction or new production design is selected. Existing T001 numeric `SavedStateEvidence` layout/migration stays future T004 work if a supported route is ultimately chosen; no task count/dependency/spec/schema change and no T003 implementation in this research PR.

[s17-provenance]: file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-minimal-finalization-fqifmq5i/evidence/provenance.json
[s17-initial]: file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-minimal-finalization-fqifmq5i/evidence/20260928-194444-4810068c/summary.json
[s17-corrected]: file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-minimal-finalization-fqifmq5i/evidence/20260928-200033-96f6b496/summary.json
[s17-valid]: file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-minimal-finalization-fqifmq5i/evidence/20260928-201223-548b7667/summary.json
[s17-initial-facts]: file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-minimal-finalization-fqifmq5i/evidence/verified-initial-facts.json
[s17-final-facts]: file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-minimal-finalization-fqifmq5i/evidence/verified-final-facts.json
[s17-human]: file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-minimal-finalization-fqifmq5i/evidence/20260928-194444-4810068c/later-human-immediate/timeline.json
[s17-human-settled]: file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-minimal-finalization-fqifmq5i/evidence/20260928-194444-4810068c/later-human-settled/timeline.json
[s17-undelivered-h21]: file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-minimal-finalization-fqifmq5i/evidence/20260928-194444-4810068c/history/021-attempt-save-18-stock-Cmd-Option-S.json
[s17-undelivered-h23]: file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-minimal-finalization-fqifmq5i/evidence/20260928-194444-4810068c/history/023-attempt-save-23-stock-Cmd-Option-S.json
[s17-undelivered-reopen]: file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-minimal-finalization-fqifmq5i/evidence/20260928-194444-4810068c/reopen/022-reopened-target-stock-Cmd-Option-S.json
[s17-undelivered-natural]: file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-minimal-finalization-fqifmq5i/evidence/20260928-194444-4810068c/natural-timestamp/023-natural-later-human-29-stock-Cmd-Option-S.json
[s17-undelivered-control]: file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-minimal-finalization-fqifmq5i/evidence/20260928-194444-4810068c/stock-control/014-control-human-29-stock-Cmd-Option-S.json
[s17-dialog]: file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-minimal-finalization-fqifmq5i/evidence/20260928-194444-4810068c/later-human-immediate/stock-save-22-0.png
[s17-immediate-visual]: file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-minimal-finalization-fqifmq5i/evidence/20260928-194444-4810068c/immediate-current/final-editor.png
[s17-natural]: file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-minimal-finalization-fqifmq5i/evidence/20260928-200033-96f6b496/natural-timestamp/timeline.json
[s17-control]: file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-minimal-finalization-fqifmq5i/evidence/20260928-200033-96f6b496/stock-control/timeline.json
[s17-history]: file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-minimal-finalization-fqifmq5i/evidence/20260928-200033-96f6b496/history/timeline.json
[s17-reopen]: file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-minimal-finalization-fqifmq5i/evidence/20260928-194444-4810068c/reopen/timeline.json
[s17-valid-case]: file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-minimal-finalization-fqifmq5i/evidence/20260928-201223-548b7667/other-valid-no-newline/timeline.json
[s17-active]: file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-minimal-finalization-fqifmq5i/evidence/20260928-200033-96f6b496/active-runtime/runtime.jsonl
[s17-active-save]: file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-minimal-finalization-fqifmq5i/evidence/20260928-200033-96f6b496/active-runtime/026-active-running-human-29-stock-Cmd-Option-S.json
[s17-active-stop]: file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-minimal-finalization-fqifmq5i/evidence/20260928-200033-96f6b496/active-runtime/028-stop-game-normal.json
[s17-active-relaunch]: file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-minimal-finalization-fqifmq5i/evidence/20260928-200033-96f6b496/active-runtime/029-relaunch-game-normal.json
[s17-check]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp#L803-L850
[s17-save]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp#L2420-L2447
[s17-edited]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/editor_interface.cpp#L714-L721
[s17-code-tag]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/scene/gui/text_edit.cpp#L4890-L4900
[s17-document-tag]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_base.cpp#L566-L573
[s17-binding]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_base.cpp#L43-L56
[s17-reload-entry]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp#L2373-L2394
[s17-reload]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp#L2526-L2593
[s17-reload-text]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_base.cpp#L575-L608
[s17-docs]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp#L3439-L3457
[s17-file-system]: https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/file_system/editor_file_system.cpp#L2395-L2455
