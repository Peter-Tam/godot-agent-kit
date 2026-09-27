# Phase 0 Research: Safe Open-GDScript Editing

**Date**: 2026-09-27 | **Specification**: [spec.md](spec.md) | **Gate record**: [plan.md](plan.md)

**Result: BLOCKED — not a completed technical design.** The native text-edit/history candidate has useful positive feasibility evidence, but neither investigated persistence route provides the required complete safety boundary. Phase 1 design/contracts must not assume that missing boundary exists. This record does not change the specification, weaken the constitution, authorize implementation, or claim a mutation-supported environment.

**Continuation boundary:** Resolve only **A — target-bound single-script persistence**, including preservation of unrelated unsaved work, and **B — source-attributed parse evidence** for the selected target/revision/interval. Both questions must be answered before planning can resume, and their answers must preserve every applicable safety invariant. No implementation or task generation is authorized while the gate remains blocked.

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
