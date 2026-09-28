# Phase 0 Research: Safe Open-GDScript Editing

**Date**: 2026-09-27 | **Specification**: [spec.md](spec.md) | **Gate record**: [plan.md](plan.md)

**Historical §11 planning status: State A — concrete patched native integration selected at that stage.** Section 11 records that design; later T001 core and T002 native-validation implementation evidence is recorded in [quickstart §§9–10](quickstart.md#9-t001-core-acceptance-2026-09-27). Native mutation/caller acceptance remains pending. The historical native probe interruption was a tooling-policy limitation, not technical evidence against native integration; it remains an incomplete probe. Section 10's guarded single-editor concurrency boundary is retained. No mutation-support claim follows from this research.

**Current decision (2026-09-28):** [§14](#14-stock-post-persistence-saved-transition-research-2026-09-28) establishes **P1**, a bounded stock post-persistence saved transition using the already-connected native ScriptEditor handler's actual public `Callable`, after an independently verified descriptor write. This supersedes §13's F3 **only for the saved-state design**: §13's normal-Save fallback and unrelated-editor effects remain demonstrated historical failures, not a selected route. Independent stock validation/effect confinement remains unresolved; T001/T002 are complete, T003–T005 pending, T003 on hold. No product mutation-support or A–E claim follows.

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
The global script-list refresh is bounded UI bookkeeping, not proof
the handler touches literally no other state. Require exact cached
held Script context; no ResourceLoader replacement or running-game
behavior is established.

### 14.3 C/D — effect decomposition and alternatives

| Effect after external target write | Descriptor already does it? | Necessity, public access and behavioral witness | Unrelated effects/bound |
|---|---|---|---|
| Target D bytes/size/flush/attachment/readback | Yes: one retained-fd write/receipt; independent component-relative `O_NOFOLLOW` D/stat witness | **Required** exact source/namespace; successful pinned attempt had one 35-byte write. | Finalizer makes no second write; no atomic exclusion of arbitrary concurrent external writers claimed. |
| Held Script R source/path/cache identity | Explicit `set_source_code`/getter before D; descriptor does not edit R | **Required** held source and unchanged path/cache, no `take_over_path` or reload. | No all-editor apply. |
| Resource inherited Object edited flag | No: ResourceSaver bypassed | **Required** public `set_object_edited(false)`/`is_object_edited`; fixture seeded true only if setter left it false, to witness true→false. | Target-only setter, no callback; later refusal is partial. |
| Private Resource path mtime | No | No supported numeric stock getter/setter established; **not independently required** by observed standalone Script behavior. Future stock Save/reopen across natural mtime-second change behaved coherently; no numerical synchronization asserted. | Never inspect/write guessed private metadata. |
| CodeEdit saved version, dirty marker, native history | No | **Required:** matching native receiver tags current; public versions/dirty/Undo/Redo witness; bare CodeEdit tag insufficient. | Tag writes no source/history step; frozen versions protect newer human B. |
| ScriptEditorBase document path mtime | No | **Required:** matching native receiver reads path mtime; subsequent ordinary Save/reopen without reconciliation dialog are behavioral, not numeric private-field witnesses. | [`_test_script_times_on_disk`](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp#L803-L850) compares document, not Resource, mtime. |
| Tab/list UI, callbacks | No | Native receiver refreshes names/list and invokes live-script reload; visually observed clean target/dirty other. | Native validation and conditional deferred debugger reload remain independent effect blockers. |

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

The completed T001 core was accepted against its then-approved numeric
`SavedStateEvidence` layout; that historical implementation still requires
private Resource/document mtime values and equality checks absent from P1
stock evidence (see [data-model migration boundary](data-model.md#savedstateevidence)).
Later T004 integration must migrate affected typed evidence, reducer and
contracts before stock P1 can be consumed. This is neither T001 incompletion
nor a new core implementation or replacement Rust/API field selection here.
