# Native Known-Path Opening Integration

**Status:** T001's native/addon integration, required acceptance and implementation-shape review are complete on the exact selected candidate. T002 connects the product caller to that unchanged native family and passed [public-caller acceptance](../quickstart.md#9-t002-public-caller-acceptance-2026-09-30). Primitive/composed planning research is in [research.md](../research.md); [native/cutover acceptance](../quickstart.md#8-t001-native-boundary-and-cutover-acceptance-2026-09-30) is separate. T003's cumulative feature acceptance remains pending. Standard public GDExtension ABI only; no engine patch/private editor field or alternate editor/history implementation.

## 1. Responsibilities and native API revision 2

Rust owns intent, admission policy, private current-source validation, authorization, caller deadline and terminal outcome. The addon owns authenticated connection/slot lifetime and collector integration. Native C++ owns exact object/file guards and narrowly staged native effects. No layer treats an acknowledgment as independently verified D/R/B.

Keep metadata key `godot_agent_kit_native`. Revision 2 retains the implemented edit API semantics and adds the private opening family below; it is authenticated by bridge v3. Names are responsibility-based, not feature/task IDs. Build-time method hashes and Variant layouts come from the exact official executable's public extension API; absent/wrong signatures disable capability rather than falling back to dynamic arbitrary calls.

| Private native entry | Contract |
|---|---|
| `open_inspect` | Bind request/path/deadline to the shared owner; passively inspect and retain target cache/document references. No load, selection or source mutation. |
| `open_prepare` | Consume the independently captured D/file identity for the inspected closed target; pin/read-check file and freeze target/current-context admission evidence. No lifecycle effect. |
| `open_advance` | Consume one authorized exact next stage: `bind`, optional `compile`, `open`; recheck before/after every stage. No arbitrary operation/method parameter. |
| `open_verify` | Fresh target Resource-edited/protection/selection inspection for recognition or post-open verification. Separate ordinary collector obtains R/B. |
| `open_recheck` | Recheck the retained same identities, source, versions, configuration and file namespace; retain detected invalidation. |
| `open_finish` / `open_abort` | Terminalize owned work without rollback; release handles only after any entered stage has returned. |
| `open_expire` | Owner's per-frame expiry cleanup; terminalize expired work without releasing an entered call's references early. |

These are private typed integration calls, not additional user tools. Use one shared session-owner state for collection/edit/open and one lifecycle-state enum for opening, not simultaneous effectful attempt pointers. Retain actual Resource references; validate ScriptEditorBase/CodeEdit object lifetime and association before dereferencing. A raw ID alone is not ownership. Generation of a new transaction UUID or generic backend framework is unnecessary.

Extract only helpers with current edit/open consumers (project/file identity, no-follow read checks, actual document association, bounded source inspection). Keep editing's writable fd, persistence/T0 restoration, edited-flag clearing and saved tagging in the edit responsibility. Do not make opening call the edit state machine.

## 2. Supported new-open profile

New opening admits one exact external `.gd`, regular readable project file, ≤512 KiB UTF-8, LF only, no BOM/CR/NUL or non-tab/newline control characters. Empty source is valid. No write permission is required. Reject embedded/UID/remapped/non-GDScript/external-editor targets, unsupported object associations or unavailable effective configuration. Existing observation and already-open recognition keep their broader independent per-source availability rules.

Before initial compilation or native opening, a bounded lexical admission checks the target and any applicable current GDScript. It is not a second parser and must not require target parse success:

- Exclude annotations (including tool/export forms), `class_name`, `static`, `const`, `load`/`preload`, dynamic property hooks `_get`, `_set`, `_get_property_list`, and script/global-class inheritance.
- Admit only built-in native inheritance. Reject identifiers resolving to effective project global classes, autoloads or extension-defined classes. Use actual project/editor/ClassDB context, not a guessed set from disk configuration.
- Correctly distinguish comments and quoted strings from executable tokens. Bound scanning and recognize supported quote/escape forms; ambiguous unsupported lexical forms refuse before effects. Do not implement this with raw substring matching or claim general GDScript parsing.
- Never instantiate the target, evaluate a function, dispatch a root ResourceFormatLoader or follow a source dependency. Ordinary function bodies are not executed by this capability. The admitted syntax-error control `extends RefCounted\nvar =\n` must reach actual native opening without repair.

Metadata limits are explicit refusal limits, not throughput promises: ≤64 effective global-class names and ≤64 autoload names, names ≤256 UTF-8 bytes; ≤256 distinct source-identifier ClassDB lookups per source; ≤64 compiled property records and ≤64 compiled method names per inspected Script, property/method names ≤256 and property hint strings ≤2048 UTF-8 bytes. Bound their combined non-source admission record to 256 KiB. Warning/context/diagnostic collection limits remain those of the existing stock validator. Oversize/unknown context refuses before authorization rather than truncating away a name or property. No global registry monitor or reusable class index is introduced.

For each source identifier that resolves through ClassDB, require the actual built-in API category; reject extension/editor-only unsupported bindings rather than trusting an apparent base name. Recheck relevant bindings and effective name/settings sets at each boundary. Collection/fingerprinting orders are canonical and typed; missing values are not empty sets. Installed known-incompatible editor configuration refuses; do not disable a plugin, change settings or select another tab to manufacture eligibility.

## 3. Passive target and current-context evidence

### Target

Independently establish unique open-document association with the existing collector and passive `ResourceLoader.get_cached_ref`, without `load` or a scan that acquires new scripts. Retain any actual cached GDScript. Cache absence must be observed, not inferred from an error. If present, source must independently match D and `EditorInterface.is_object_edited` must be observed false before new opening. A wrong-type occupant, different/stale/dirty R or unknown edited state refuses. Never overwrite/reload it even if the bytes could be made equal.

For an already-open target, skip new-open source/profile/current validation and every lifecycle stage. Its actual dirty/divergent/unavailable state is permissible evidence, not a reason to repair it. Recheck exact document identity and open state; preserve selection and native editing history. A denied or ambiguous target still cannot disclose source.

### Departing current document

`ScriptEditor.get_current_editor`, `get_current_script` and exact open associations identify either a proven no-source current editor (including an actual help/no-document context), an applicable current GDScript, or unsupported/unknown context. A failed getter is not proof of no current source.

For a current GDScript require:

1. Same selected project, supported standalone identity and valid Script/editor/CodeEdit association. Independently read actual R and B and require exact equality. Capture attributable buffer dirty state, Resource-edited flag, current/saved versions and native Undo/Redo availability. Dirty R == B may be eligible; its dirty flags and versions must remain unchanged. Dirty R != B refuses before native navigation can apply pending text.
2. The source profile above, actual tool=false and no script base, plus bounded passive compiled property/method metadata. Reject compiled tool/script-base contexts, script properties with editor/export usage, Object-valued properties, nonempty property hint strings and dynamic property hooks, including stale compiled metadata that no longer matches the current source's apparent profile. Category/group display records are not script variables. Typed/object property hints matter because native pending-drag assignment matches property name/class hint before setting it.
3. A valid attributable result from the existing one-shot **source-only** stock validator on this exact R == B source and effective warning/context. This is purpose `open_context`, not edit preflight or target validation. The parent owns helper lifetime/cleanup and binds request/session/current identity/path/hash/context/completion. No original-current D/R equality is required: genuinely unsaved current text remains unsaved. No current-source helper runs for proven no-source context.
4. Fresh rechecks of these identities, text, versions, flags, metadata, effective configuration and validation context before each effect. Invalid/unavailable current validation refuses; no waiting, synchronization, Save, edited clearing or saved tagging is permitted to obtain eligibility.

R/B equality alone is insufficient: native navigation applies/validates the departing document, and validation can process pending dragged-export state. There is no public `pending_exports_empty` claim. The bounded source/compiled/current-validity profile prevents the relevant ordinary source/property effects; [pinned paths and observed controls](../research.md#42-dirty-current-document-negative-and-positive-controls) justify it. Tests must include real pending-drag/Undo/stale-metadata histories, not merely a quiet current editor.

Current source stays private to admission/owned temporary validation. Return no current path, source, hash or diagnostic snippet through the public opening result. Preserve the stock helper's inherited local endpoint limitation; it is not a new arbitrary-evaluation surface or claimed OS sandbox.

## 4. File capture and immutable preparation

The worker independently reads D through the existing project capability and no-follow confinement. Native preparation independently opens the same existing file read-only, holds the project/parent/leaf descriptors and compares complete captured bytes, file identity, size and namespace attachment. No create/truncate/write access, mtime restoration or filesystem scan/reload.

Before every native stage, confirm the authenticated session/owner, deadline, original project and file/parent identity, exact content and cache/document state. Same bytes at a replacement identity do not authorize retargeting. A cached target must remain that exact held Script with matching source and unedited state. A cold target must remain uncached until this attempt's non-takeover publication. All comparison/acquisition failures are typed; unavailable evidence cannot be unwrapped or inferred clean.

A newly opened target or context change before publication causes a no-effect refusal if terminal discard is proved. Do not restart preparation or automatically turn it into an authorized new branch. After publication, preserve known partial effects and stop further stages on invalidation. The model does not claim atomic exclusion of arbitrary external writers between checks; it does require all observed changes to defeat stale authorization/verification.

## 5. Authorized native stages

The parent records possible application before authorization; the native owner accepts a stage only after the trusted supervisor's context validation is bound to the same preparation. Mark native entry before a potentially effectful call. Do not pump events to simulate cancellable native execution.

### Bind

For a cold target:

1. Construct a native GDScript through the supported public ABI and immediately hold a strong reference. It is a new unbound object, not an observation of target R.
2. Initialize its exact source with the bound **`Script.set_source_code`** method. Do not use `Object.set`, property assignment, a Script `_set` dispatch or caller-supplied replacement bytes.
3. After fresh file/cache/context checks, call bound **`Resource.set_path`** with the exact original `res://` path. This normal setter refuses collision; never use `take_over_path` or `set_path_cache`.
4. Independently inspect actual path/cache ownership/source and Resource-edited flag. If the new object is published, record that lifecycle effect even if a subsequent check fails. Confirm edited=false; never call `set_object_edited(false)` to repair it.

The exact-method [composed control](../research.md#45-exact-methods-preserve-composed-edit-eligibility) passed existing guarded editing; the property-assignment variant did not. Preserve this regression rather than treating a clean B as proof of clean R.

For a cached target, bind the retained actual Script without assignment, reload, compile, path changes or dirty-state changes. This reuse is a native fact, not a new cache-publication effect.

### Initial compilation — cold only

Recheck the published identity, still-closed target, source/file/context and owner. Call `Script.reload(false)` **only on the new Resource constructed by this attempt**, for its initial normal compilation. Never use it on an existing cached Script or after a human has opened/replaced the target. Record actual completed Error value and source/context attribution. `OK` is valid and a completed `ERR_PARSE_ERROR` is invalid target parsing; either may continue. A call failure, unknown result or other native failure stops with known/unknown effects, not an invented parser verdict.

Initial compilation is permitted native loading, not permission to instantiate, execute target methods, load dependencies or hot-reload a game. Recheck actual source/cache/edited state afterward. Do not parse by secretly editing the live buffer. No private parser/cache/timestamp manipulation is selected.

### Open

Under fresh target/source/current-context/file guards, invoke bound `EditorInterface.edit_script(script, -1, 0, true)` on the exact retained Script. Focus-enabled native navigation is chosen because focus=false did not avoid departing-source effects. This call may select the new target; it is never used to inspect/select an already-open document.

Record entry, returned actual association and selection separately. The void return alone does not prove a buffer exists. Keep the admitted departing document references and preserve its source/dirty/version/history witnesses. After opening, expected active selection can legitimately be the target; this does not permit changing the frozen preservation witness for the departing document. If a human selects another tab afterward, observe/report it without stealing focus back or substituting that tab's evidence.

Never implement the new buffer by assigning CodeEdit text, tag it saved, clear history or close/reopen it to force agreement. Ordinary native construction of the new document establishes its initial buffer; independent checks determine whether that result is clean/coherent.

## 6. Independent verification and interrupted ownership

Use the existing collector in a separate verification acquisition for actual target R/B/dirty/open identity and fresh Rust D. Independently inspect Resource edited=false and the retained protection witnesses. New success requires the exact requested Script and actual buffer, D/R/B equal to admitted D, clean attributable buffer state, unedited Resource, intact identities/namespace/configuration and no known required-evidence invalidation. Native set/readback/progress facts are not substitutes.

Preserve newer human text, flags, versions and selection. Detecting a post-effect change returns known applied/partial unverified state; no Save, reassertion of captured source, cache takeover, edited clearing, closing or rollback. A retained old reference is not authority over a same-path reopened document.

Caller timeout bounds caller response, not synchronous editor completion. Keep entered native references/slot ownership until return; forbid later stages once expired/cancelled/disconnected. Only a proved terminal pre-effect discard permits not-applied. Resource publication without a buffer is partial application. Lost response after authorization is unknown unless earlier valid facts establish effects. Clean up owned handles, not user editor state; late replies cannot amend an already-delivered outcome.

## 7. Boundaries and required proof

Native display callbacks and already-authorized editor tooling remain inside the inherited local-editor trust boundary. The [B1 scope correction](../research.md#6-scope-correction-and-callback-limitation) does not excuse known unsafe context, newly introduced project-source execution or detected human-work loss. No universal callback registry/isolation, plugin shutdown or clean-room-editor prerequisite is selected. The kit does not claim protection against malicious code already running inside Godot with authority to change/falsify buffers.

Require actual cold/cached/empty/read-only/syntax-invalid opening; dirty current R == B and background preservation; R != B refusal; loader/dependency/tool/static/global/extension/compiled-metadata refusals; file/cache/session/open-state races; loss/timeout/disable at each effect stage; native human history transitions; and the composed fresh-observe/edit A–E plus Save/reopen/reparse/rescan/runtime/export checks. The source-bound method sequence must retain ordinary cache consumers and save/reopen behavior, not merely create a visually clean tab.

The shared bundle rename must update build inputs, generated manifest/hash identities, plugin loading, fixtures, current documentation and export exclusions together. No old library/entrypoint aliases remain. This contract authorizes no implementation until normal approved-task selection; [quickstart.md](../quickstart.md) defines the required future evidence.

## 8. Implemented private fixture boundary

T001's actual addon node is `GodotAgentKitScriptOpen`; it claims the bridge's
single `_active.operation_owner` slot through `_claim_operation` and releases
through `_release_operation`. Native `Session` uses one edit/open owner variant.
Disconnect/disable retains the entered owner and detached bridge until return.
No authenticated product opening opcode is installed in T001.
T002's `script_open_transport.gd` connects authenticated tuples to that same
owner and the ordinary collector. `advance_bound` consumes the exact staged
peer authorization; private native fixture controls remain outside product input.

The native custom Callables take:

| Call | Arguments |
|---|---|
| `open_inspect` | `path`, `{request_id, session_id, expiry_tick_us}` |
| `open_prepare` | `request_id`, captured source, `{project_device, project_inode, target_device, target_inode, cache_mode, cached_script_id, sha256, utf8_bytes}` |
| `open_advance` | `request_id`, exact next stage, validated source hash, validated context hash |
| `open_verify` / `open_recheck` | `request_id`, `recognition` or `post_open` |
| `open_finish` / `open_abort` | `request_id` |
| `open_expire` | No arguments |

IDs, counters, lengths and native expiry are canonical decimal strings; the
request/session use their existing checked formats. The local remaining budget
must not exceed nine seconds. Cache mode is `absent`/`present`; the absent cached
ID is empty. Native replies retain monotonic cache/compilation/document facts,
separate Resource-edited and selection observations, invalidation and terminal
discard. They do not contain a product success verdict or copied verification R/B.

Preparation returns private `{kind, reason, projection, source, sha256}` context.
The typed guard and its exact encoding follow the [bridge contract](bridge-protocol.md).
Rust independently validates/recomputes it; the fixture supplies `source: null`,
purpose `open_context` and this real context to the existing stock validator.
Only an exact completed-valid receipt with owned-child reap and cleanup produces
`opening_binding`. Its `guard_sha256` is distinct from the validator's
`context_sha256`; edit evidence rejects this purpose and binding.

`document_guard` owns only shared descriptor/namespace/document/source checks.
Save-format eligibility remains edit-only in `script_document.cpp`; opening
preserves supported LF whitespace and does not apply future Save preferences.
Compiled category/group display records remain bounded, fingerprinted metadata,
not script variables: their display hints do not bypass checks on actual
exported, Object-valued or hinted variables.

The separate `GAK_FIXTURE` artifact adds controlled stage faults and an actual
extension-category refusal class. Neither belongs to the product native family
or exports. The normal opening runner requires actual owned-window captures;
state-only diagnostics cannot satisfy T001 acceptance.
