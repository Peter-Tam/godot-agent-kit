# Native Script Editing Integration Contract

**Status:** Selected planning contract; not implemented or runtime-verified. Names below identify required operations, not existing Godot bindings. Applies to the exact candidate in [plan.md](../plan.md). The [data model](../data-model.md) owns common field meanings; the Rust/core alone classifies terminal outcomes.

## 1. Ownership and minimum exposure

Use the existing editor plugin/private bridge, a small C++17 GDExtension using Godot's generated public C ABI, and narrowly added engine APIs. The extension never casts opaque objects into private engine layouts or links private C++ symbols. No custom module, second parser, external writer/validator, or transaction service is selected.

| Responsibility | Owner / selected mechanism |
|---|---|
| Intent, stale/conflict policy, authorization, deadlines, uncertainty, final classification | Existing Rust library and supervised caller/worker. |
| Authentication, routing, connection/attempt lifetime, marshaling | Existing private bridge and Godot integration. |
| Native complex text operation, explicit Script source setter, descriptor-bound I/O | GDExtension through supported object methods and current-platform OS APIs. |
| Guarded document save bookkeeping and read-only saved-state inspection | Narrow engine API: actual ScriptEditorBase/Resource state is unbound today. |
| Exact-source validation and restricted dependency/effect handling | Narrow GDScript-owned engine API reusing the existing parser/analyzer. |
| Target-local guarding of automatic validation caused by the agent edit | Small editor-side companion to those APIs; not a second transaction policy. |

Engine API family revision **1** comprises `inspect_script_document`, a paired `begin_script_edit_guard` / `end_script_edit_guard`, Primitive A `finalize_script_document`, and Primitive B `validate_gdscript_source`. The inspection/guard operations exist only to make A/B independently observable and prevent the normal editor validation path from bypassing their effect boundary. They expose no generic object-call, filesystem, evaluator, or history-control API to the bridge/caller.

`inspect_script_document` independently reads the exact Script/document/CodeEdit association, document path/mtime baseline, Resource edited/mtime fields, and CodeEdit current/saved versions. It never tags, saves, synchronizes, loads, opens or selects. Existing R/B/dirty observation remains separate. Missing this readback capability means edit capability is unavailable, not an inferred successful finalization.

Preparation also inspects the target's native save-format settings and checks both the baseline and desired source against the same engine formatting rules in a non-mutating/check-only path. `save_current_script` runs `_auto_format_text` (trailing whitespace, final newlines and indentation); a later Save must not silently change the promised revision or destroy the Undo → Save → Redo case. Return `unsupported_representation/save_would_reformat` if either source would change, or if the relevant behavior cannot be established. Retain/recheck this fixed save-profile witness at application and verification. Never alter editor preferences, run formatting on the live document, or add a general formatter API. Representation limits are explicit; later independent human source/settings changes remain outside the successful interval.

The guard is a request-local engine object held only by the native attempt, not a new transaction/session UUID. It binds existing request/session identity and actual document objects. It is entered only after fresh checks and core authorization, immediately before native text mutation, and released on every terminal path. It does not lock out human work or serialize external filesystem writers. Identity/source changes invalidate it rather than trigger repair.

## 2. Common bound state and persistence receipt

Every native operation belongs to one authenticated connection, existing `request_id`, selected `session_id`, canonical project directory identity, and exact `res://` standalone `.gd` path. Document identity comprises actual Script, ScriptEditorBase and CodeEdit instance IDs plus their current association. Retain the Script reference; revalidate node IDs/open-document association before dereferencing editor objects. A retained Resource is not proof the document remains open.

The native attempt retains the expected baseline and immutable desired UTF-8 source, independent source hashes, post-application current buffer version, independently observed preparation-time saved version, expected R source, the engine guard, and one target file descriptor opened relative to the validated project directory. Observation v1 supplies no prior saved-version witness; preparation obtains it directly and unexpected retagging before A invalidates the attempt. Root/leaf identities must match the Rust-observed identities; no caller-supplied absolute destination is used. Component traversal uses existing confinement/owner/permission/ACL rules and no-follow existing-file opens. No create/truncate happens during preparation.

**Native-only PersistenceReceipt:** the descriptor writer creates this attempt-local record after the actual write, never from a client-supplied `saved` flag. It records:

- request/session/document binding and desired source hash/UTF-8 length;
- project and opened-file device/inode identities, retained descriptor ownership;
- `write_started`, bytes written, truncate completion and flush completion;
- descriptor readback identity/content result and current project-path attachment result;
- descriptor-observed modification time, including native precision and Godot's seconds representation;
- known interference, elapsed/interval evidence and explicit error/unknown states.

Only a complete successful write/truncate/flush/readback with unchanged binding and current namespace attachment is eligible for A. Receipt facts are **application evidence**, not the independent Rust D observation. A proof for another attempt/document, a closed descriptor, incomplete write/flush, unknown result, or detected revision/namespace change is ineligible. The receipt is used once, released with the attempt, and never persisted or accepted on the wire as authority.

The writer rechecks expected D bytes/identity and the current B/R/identity guards before I/O; it writes only the retained file object. Short writes are completed only while the same attempt remains eligible; any later error preserves actual byte/application facts. Empty replacement still performs the required truncate/flush. Recheck path attachment and source afterward. Namespace loss is non-success even if the original renamed/unlinked object received bytes; never reinstall it. This is not crash-atomic storage or arbitrary same-inode compare-and-write. Known interference always prevents success.

## 3. Primitive A: guarded target-document finalization

### Inputs and preconditions

`finalize_script_document(guard, expected_after, persistence_facts)` is one synchronous main-thread operation. `persistence_facts` is passed only by the trusted native receipt owner after its fresh descriptor/path checks; it is not exposed as an agent API. `expected_after` includes exact Script/document/CodeEdit IDs and association, resource path, expected current and saved buffer versions, exact desired source/hash, and expected R source/hash.

Before any bookkeeping write, the integration and engine hook establish:

1. The original session/connection/guard is live, not expired/cancelled/consumed; the document is still open and uniquely associated with the exact objects.
2. The buffer's current version and exact text match the recorded post-application state; its saved version still matches the independently read preparation-time value. Equality of text alone cannot authorize tagging a different version, and unexpected earlier retagging is invalidation. R still matches the expected resulting source and Resource identity.
3. The native receipt belongs to the same intent and object, reports complete persistence, and its fresh descriptor/content/path checks remain usable. A known later D change cannot be ignored.
4. Required observations and the captured save profile remain valid; there is no invalidation or nested finalization in progress.

Run the bookkeeping span without `await`, event-loop pumping, signals or arbitrary callouts. Non-yielding execution alone is not a reentrancy guarantee: reject nested invocation and avoid callouts between guards and updates. This protects editor-local state, not arbitrary external writers.

### Selected effects and ordering

R synchronization is a distinct preceding application step: after the native CodeEdit complex operation, compare B/version and expected original R, then call the **bound `Script.set_source_code` method** with the intended text and read R back. Do not use property assignment through `Object.set`/GDScript's `_set`, which can call `reload(true)`. Do not call `ScriptTextEditor::apply_code`, `update_exports`, reload, or pending dragged-export assignment. Pending dragged exports make the document unsupported before application rather than silently changing a scene.

Once A's complete guards pass, perform these exact target-only bookkeeping effects:

1. Set target Resource last-modified time and document `edited_file_data.last_modified_time` to the successful descriptor receipt's Godot-compatible time. Do not resolve a new pathname to obtain a replacement timestamp, and do not rewrite document paths/identity.
2. Set target Resource edited state false only for the guarded R source just persisted. This is the source Resource's save bookkeeping, not a claim that arbitrary scene/Inspector/export state was saved.
3. Tag the exact current CodeEdit version saved **last**, equivalent to the saved-version part of the native document tag. Do not set text again, change undo versions, clear history, select a tab or create another undo entry.
4. Refresh only the target's native saved/dirty display through internal non-callout UI bookkeeping. Re-read all changed fields and target/source/version guards before returning.

The engine hook deliberately splits the current document tag's effects so its timestamp comes from the descriptor receipt rather than `FileAccess::get_modified_time(path)`. Record each completed effect; do not claim all completed from an early acknowledgment. If a guard fails after a bookkeeping effect, stop and return partial evidence. Never tag a newer buffer or restore earlier fields/source over newer activity.

### Callbacks and exclusions

No `ResourceSaver.save`, `EditorNode.save_resource`, `_res_saved_callback`, synthetic `resource_saved`, EditorData save notification, scene-saved/POST_SAVE event, built-in-script marking or live-debugger reload is invoked. Those routines include pathname persistence, unrelated callback behavior or broader scene/runtime effects. Their absence is an explicit capability boundary: this operation finalizes one script's source/document saved state, not a general save-notification or hot-reload API.

Normal native Save/Undo/Redo/reopen remain the later developer interactions to prove durability. Native source history and current selection must be preserved. Target parser diagnostics may be refreshed from B's controlled validator result; export-placeholder propagation, arbitrary project callbacks, scene property assignments and broad documentation/scan work are not incidental finalization effects. No compiled-class, Inspector-export refresh or running-game hot-reload guarantee is inferred from R source synchronization.

### Results and failure semantics

Return `complete`, `rejected`, or `partial_or_unknown`, with reason, native interval, binding, observed before/after versions/hashes and completion flags for Resource mtime, document mtime, Resource edited state, saved-version tag and target display refresh. `rejected` means **A** made no bookkeeping changes; it does not mean the overall edit made no changes.

| Condition | Required result |
|---|---|
| Session/document closed/replaced; Script/editor/CodeEdit identity mismatch | Reject before bookkeeping, or stop with partial flags if detected later. Never retarget. |
| Current/saved version changed, including same-text new version; newer human B or unexpected R | Reject without tagging; retain newer state. Persisted D may already have changed. |
| Missing, failed, mismatched or uncertain persistence receipt | Reject; never manufacture saved state. |
| Namespace lost, D revision changed, descriptor evidence unavailable | Reject; preserve actual persistence facts, not a false successful save. |
| Reentrant call, cancellation or deadline before A | Reject with reason; do not start bookkeeping. |
| Later failure/invalidation after any bookkeeping step | `partial_or_unknown`, per-step facts and current observations; no rollback promise or cleanup write. |
| All guards/effects/readbacks pass | `complete` for A only; final edit success still belongs to Rust's independent reducer. |

After A, the collector separately calls `inspect_script_document` and reads actual R/B/dirty state; Rust separately reads current D through its own project capability. A's return object cannot stand in for any of those observations.

## 4. Primitive B: exact-source native GDScript validation

### Inputs and output

`validate_gdscript_source(source, source_path, read_context, correlation)` runs synchronously on the selected editor main thread. Inputs are an immutable exact source string, exact project-confined `res://` root path, a native-only selected-project read context, and existing request/session/document identity plus purpose `preflight`, `post_change` or `unchanged`. No mutable global buffer is the implicit input. Compute SHA-256 from the actual supplied UTF-8 bytes; compare the integration's expected hash before invocation. The root target is already open/loaded; validation does not open or load it.

The engine exposure delegates to **`GDScriptLanguage::validate` → `GDScriptParser::parse(source, path, false)` → `GDScriptAnalyzer::analyze`**, with the scoped effect/dependency policy below. It marshals the actual return and root/dependency `ScriptError` records; it never uses reload, `can_instantiate`, silence in a log, or empty diagnostics as a validity witness.

Return exactly one status:

- `valid`: the requested parser/analyzer stages completed under the selected policy for this exact source/context, without errors.
- `invalid`: a completed validation rejected this source/context; retain root/dependency diagnostics and the failed parser/analyzer stage when supplied.
- `unavailable`: invocation incomplete, context/attribution invalid, disallowed effect, access failure, resource bound, wrong thread or reentrant invocation. Partial diagnostics do not turn it into valid/invalid.

Every result includes request/session/document binding, purpose, actual input path/hash/byte length, editor-clock start/end ticks, result/reason, dependency/context witnesses and `diagnostics_complete`. Diagnostics carry `origin: root|dependency`, confined path (or null with reason), source hash when actually read, line/column when supplied, category and message. Unknown dependency source is not assigned the root hash. Missing dependency proven by a confined read can yield a completed dependency error; an unreadable or unconfined dependency yields unavailable. Redact denied/outside-project paths; never disclose their source or raw absolute paths. Errors/messages appear only in the requested result, not incidental logs.

Limits: root and each dependency ≤512 KiB UTF-8; at most 32 distinct source dependencies / 4 MiB dependency bytes; at most 64 error records with messages ≤2048 UTF-8 bytes and paths ≤2048 bytes. Exceeding a needed bound yields `unavailable` with limit reason and safely retained partial diagnostics, not truncated source or false valid. Warnings/functions/safe-line UI data are not a new caller diagnostics API.

### Selected effect policy: source analysis, not arbitrary evaluation

Allow the existing parser/analyzer, engine-native static type metadata, current project/global-class/autoload mapping lookup, confined read-only GDScript dependency text, normal parser/dependency-cache lookup/population and dependency-edge bookkeeping. Cache mutation is expected engine bookkeeping, not a purity failure. Do not clear global caches to simulate isolation.

Select a **validation-call-scoped read/effect context**, passed through the implicated GDScript parser/cache/analyzer paths:

1. All dependency bytes are obtained through the trusted native integration's project-directory capability reader, with no-follow traversal and before/after identity/content checks. Resolve each relative dependency against the directory of the source containing that reference: the supplied root path for root references, and Godot's per-dependency source paths for transitive references. Resolve permitted project mappings to a confined `res://` path before reading. No absolute/user/network/embedded/remapped-outside-project fallback. Unsupported binary/remapped dependency representations are unavailable, not silently loaded.
2. A cache hit is usable only when attributable to the same project/path and current guarded input digest. A current `Script.source_code` getter does not prove cached parser/analyzer/shallow metadata was derived from that source generation. Stale/unattributed metadata cannot certify the proposed source. Reuse the engine's parser types; use call-local parser references where needed to analyze the root overlay/current dependency bytes without replacing a loaded target Script or globally clearing caches. Fresh per-call read witnesses remain distinct from a cache lookup.
3. Permit shallow GDScript metadata construction only without full reload, script instantiation or initialization. Intercept non-GDScript `ResourceLoader` resolution **before** invoking loaders (including exists/type hooks), full-script loading, and dynamic object-property `Variant::get/get_named` paths. This profile returns `unavailable/unsupported_effect` rather than invoke project code. Plain built-in value operations and engine-native static metadata remain allowed. Do not substitute guessed types or skip analysis and return valid.
4. No user method, tool/static initializer, scripted constructor/getter, arbitrary loader/callback, process/network action, source write, root R/B assignment, document save or project scan is permitted as an effect of this call. `@tool` syntax alone is not execution or an automatic rejection; a resolution path requiring an excluded effect is. Unsupported effectful validation is a truthful capability refusal, not a new evaluation permission prompt.
5. Capture and recheck dependency identities/hashes and relevant project/class/autoload/remap context used by the invocation. Changed/missing witnesses make the result unavailable/invalidated. The result describes an observed context, not an atomic snapshot or project-wide compilation.

This is more than adding a naked binding to an effectful call: the small engine adaptation owns the concrete interception points. It is still the existing GDScript language implementation, not a second compiler, global execution sandbox or service. The bridge cannot supply executable read callbacks; its native-only reader/context is installed by the trusted integration. Existing exclusion of hostile code already executing inside the editor/same-UID malware remains unchanged.

### Guarding the ordinary editor path

A's companion document guard is necessary because successful ordinary non-tool `_validate_script` can copy B into R and call `update_exports`; `apply_code` is broader still. During the agent edit, route target validation/function-discovery callbacks caused by that edit through B's restricted policy, and suppress their duplicate R/export/dragged-property application. Explicit supported `Script.set_source_code` is the only planned R synchronization step.

Bind queued automatic work to the actual target and agent-produced CodeEdit version. Closing the guard must consume/cancel redundant work already satisfied by the explicit result or leave that queued work restricted until drained; it must not release an unrestricted delayed reload/export callback caused by this attempt. A subsequent independent human source change invalidates the guard and retains normal Godot handling for that newer edit. Do not globally disable validation, block human typing, rewrite newer text or clear unrelated editor state. No callback may mark a buffer saved.

Any inability to establish guard coverage is a non-success/unsupported implementation result, not permission to proceed. These hook changes and their real-editor regression evidence are implementation obligations; the plan does not assert they already exist.

### Placement in the edit flow

For changed intent, run B on proposed source during preparation to refuse invalid/unsupported-effect inputs before changing B or history. Then freshly recheck all mutation guards; preflight is not permission for a delayed edit. After source application, persistence and A, read the actual resulting B/R and invoke B again with that exact source as `post_change`. This second result is the required post-change parse witness. Dependency/context changes can still produce invalid/unavailable after an earlier valid result; report applied-unverified and do not force convergence.

For unchanged requests, skip proposed-source preflight and run one `unchanged` validation with independent verification, without native text mutation, writer, finalizer, saved tagging or history entries.

## 5. Implementation proof obligations

The [quickstart](../quickstart.md) supplies the acceptance matrix. Required native cases include wrong receipt/identity/version, same-text newer version, failed/unknown persistence, each partial bookkeeping step, actual Undo → Save → Redo, non-selected target and unrelated dirty work, and read-only saved-state inspection distinct from A's return.

B must prove valid/invalid/unavailable attribution, relative source dependencies and changed dependency/context, confinement on actual reads and cache hits, allowed cache activity, refusal before excluded loaders/getters/initializers, no root D/R/B writes, and no delayed automatic-validator/export bypass. Test explicit `Script.set_source_code` versus property/reload paths, nested callback rejection, and guard cleanup without changing ordinary unrelated/human behavior.

The patched editor must pass the current source-coherence and observation regression requirements, ordinary Save/reopen/reparse/rescan/runtime durability and export isolation before mutation support is claimed. These are future implementation/verification requirements, not completed tests or a prerequisite to writing this contract.

## 6. Pinned implementation basis

All engine references use `ed1daf0bf001b61586d9930840f2f1394092c079`:

- [Document tag/inspection state](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_base.cpp) and [state declaration](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_base.h): CodeEdit tag alone omits document mtime.
- [ScriptEditor save/check/callback paths](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp), [ResourceSaver bookkeeping](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/core/io/resource_saver.cpp), and [EditorNode notifications](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/editor_node.cpp): ordinary save includes broader behavior; ResourceSaver itself does not emit EditorNode's `resource_saved` signal.
- [Script source setter/property handler](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript.cpp): `set_source_code` changes source/source-changed bookkeeping; `_set` for script source additionally reloads.
- [Native editor application/validation](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_text_editor.cpp): R assignment/export update is in the successful non-tool validation branch, not the failed-validation branch; guard the actual call paths.
- [Validator](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript_editor.cpp), [analyzer effect sites](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript_analyzer.cpp), and [dependency cache/shallow loading](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript_cache.cpp): source-specific validation is reusable, but effect control and confined source resolution must be added rather than presumed.
