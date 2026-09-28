# Native Script Editing Integration Contract

**Status (2026-09-28):** [§18 T1](../research.md#18-preserved-mtime-behavioral-finalization-research-2026-09-28) selects the **handler-free same-retained-fd T0 mtime restoration** sequence conceptually for official stock Godot 4.7.2/macOS arm64. The earlier [§17 M3](../research.md#17-minimal-behavioral-finalization-research-2026-09-28) negative applies to direct tagging **without** mtime restoration: later human Save falsely prompted and did not persist B29; history also failed. [§14 P1](../research.md#14-stock-post-persistence-saved-transition-research-2026-09-28) and [§16 H3](../research.md#16-stock-saved-handler-admission-research-2026-09-28) remain preserved historical handler research, superseded **for production**, not independent T003 holds. T002's patched read-only validator is an oracle/hazard inventory, **not** a stock validator. The remaining architectural T003 hold is completed exact-source stock valid/invalid/unavailable validation; all generic effect safety and behavioral/product A–E gates remain. T001/T002 complete, T003 unstarted, T004–T005 pending. No final wire/schema/API, non-macOS support or Feature 002 acceptance is claimed.

## 1. Ownership and minimum exposure

**Selected T1 conceptual design, not implemented product support:** Official stock Godot plus addon and bundled standard public-ABI C++17 GDExtension remain the target. Existing editor integration/private bridge own admission/routing; native work owns one CodeEdit complex edit, explicit bound `Script.set_source_code`, one descriptor-bound content persistence, a distinct same-fd T0 restoration, then public Resource edited=false and direct CodeEdit tag/STOP. No saved-handler Callable discovery/invocation, listener emission, ResourceSaver, full Save, patched engine or private Godot timestamp field is selected. Historical §14 P1 used a broad callback under §16 H3; §17's narrower **un-restored** direct tag failed later Save/history. §18 supplies observed behavioral remedy, not a validator, product implementation or blanket proof of all effects.

| Responsibility | Owner / selected mechanism / proof obligation |
|---|---|
| Intent, stale/conflict policy, authorization, deadlines, uncertainty, final classification | Existing Rust library and supervised caller/worker. |
| Authentication, routing, connection/attempt lifetime, marshaling | Existing private bridge and Godot integration. |
| Native B edit, explicit Script R setter, retained-fd D content and metadata | Standard GDExtension through supported object methods and demonstrated macOS descriptor APIs. Exactly one logical content-persistence operation (`pwrite` calls as needed for eligible short writes, then truncate/fsync/readback), then a separately checked same-fd `futimens` restoration of T0 (atime omitted). |
| Guarded saved transition and independent public saved-state evidence | Fresh target/source/version/receipt/namespace guards before the metadata syscall and before edited=false, then recheck and direct CodeEdit tag/STOP. Independent D/R/B, Resource edited flag, CodeEdit versions/dirty and exact association plus later ordinary Save/Undo/Redo/reopen must corroborate. No private numerical Resource/document mtime requirement. |
| Exact-source validation and confined effects | No qualifying explicit stock validator selected. T002's patched validator is a semantic reference; ordinary editor validation is not completed exact-source evidence. Preserve generic no-forbidden-effects and ordinary-editor-effect safety without P1 handler-specific callback admission machinery. |

The earlier proposed engine API family revision **1** (`inspect_script_document`, `begin_script_edit_guard` / `end_script_edit_guard`, `finalize_script_document`, `validate_gdscript_source`) and private numeric inspection/bookkeeping were a patched-engine design, **superseded as current stock saved-state requirements**, not stock bindings to implement on T003. T002 exposed the narrower `GDScript.gdscript_validation_api_revision()` and `validate_gdscript_source` ClassDB bindings plus `ScriptEditorBase.get_edited_resource()` association readback in its patched editor. Its editor-local metadata reports **validation** revision `1`, not this whole family or edit capability; see the [implemented native surface](../../../godot-addon/native/README.md#integration-surface-and-acceptance). No generic object-call, filesystem, evaluator or history-control API is granted to bridge/caller. Stock public observation instead reads the actual still-open association, `EditorInterface.is_object_edited(Script)`, CodeEdit current/saved versions and dirty state, and independent D/R/B; it does not numerically inspect or set private Resource mtime or document mtime. §17 demonstrates document timestamp freshness has a real later Save consequence without making private numeric parity a requirement.

Preparation also inspects the target's native save-format settings and checks both the baseline and desired source against the same engine formatting rules in a non-mutating/check-only path. `save_current_script` runs `_auto_format_text` (trailing whitespace, final newlines and indentation); a later Save must not silently change the promised revision or destroy the Undo → Save → Redo case. Return `unsupported_representation/save_would_reformat` if either source would change, or if the relevant behavior cannot be established. Retain/recheck this fixed save-profile witness at application and verification. Never alter editor preferences, run formatting on the live document, or add a general formatter API. Representation limits are explicit; later independent human source/settings changes remain outside the successful interval.

The attempt-local guard binds request/session, actual document objects, source/current-and-saved versions, retained descriptor, original T0 and the separate content/metadata receipt; it is **not** a new engine guard object or transaction/session UUID. Recheck binding, receipt, identity and versions immediately before each native effect, including after edited-flag clearing and before direct tagging. It neither locks out human work nor serializes external filesystem writers: identity/source/version/namespace changes invalidate it, without repair. Intermediate D/R/B divergence is allowed with accurate partial classification and independent final observations; no every-step external atomicity claim. Exact-source validation and generic effect confinement remain necessary before T003.

Kit admission remains the integration's existing single active collection/edit slot, not a second native scheduler or lock policy. Only its current attempt may invoke effectful native stages; a rejected overlap has no native work to resume after release. Caller termination does not release the slot/handles while entered native work can still mutate: stop new stages and complete safe cleanup under [NativeAttempt](../data-model.md#nativeattempt) before another attempt is admitted. Human typing remains possible; never merge or rebase competing intentions.

## 2. Common bound state and persistence receipt

Every native operation belongs to one authenticated connection, existing
`request_id`, selected `session_id`, canonical project directory identity,
and exact `res://` standalone `.gd` path. Document identity comprises actual
Script, ScriptEditorBase and CodeEdit instance IDs plus their current exact
still-open association. Retain the Script reference; revalidate editor/buffer
lifetime/association before dereferencing. A retained Resource or same-path
reopened tab alone does not authorize finalization.

The attempt holds immutable expected and desired UTF-8 sources and hashes,
prepared CodeEdit current **and saved** versions, frozen post-complex-edit
current version, exact bound Script/editor/CodeEdit IDs, project-bound
directory and one existing exact-file descriptor. Root/leaf identities must
match the independent Rust basis. Prepare with no create/truncate and `fstat`
the retained fd for **T0 mtime before mutation**. Observation v1 does not
provide a saved-version witness; preparation obtains it directly. Follow
existing owner/permission/ACL and no-follow traversal rules; no
caller-supplied absolute destination. No saved-handler edge/Callable is
captured.

**Native-only, attempt-local persistence receipt — distinct phases:**

- Binding: request/session/document/project/path, original and intended
  source digest/UTF-8 length, retained fd ownership and device/inode identity,
  prepared current/saved versions and T0 from pre-edit `fstat`.
- **Content phase:** one logical descriptor-bound content persistence:
  `pwrite` (short writes may require more than one syscall while still
  eligible), `ftruncate` (including empty replacement), `fsync` and `pread`
  readback. Record `write_started`, actual write calls/bytes, truncate and
  fsync completion, descriptor content hash/length/identity, fresh no-follow
  namespace attachment and actual short-write/error/unknown/interference
  facts. The attempt is known applied after a successful content change,
  even if a later stage fails. No second logical content persistence or fallback Save.
- **Metadata phase (only after complete content and fresh guards):**
  `futimens` on **the same retained descriptor**, times[0] `UTIME_OMIT` for
  atime and times[1] **T0** for mtime. Record whether called, actual errno or
  unavailable result, `fstat` before/after mtime and retained identity,
  descriptor content re-read/hash and fresh no-follow project-path
  attachment/content/identity. A successful return alone is insufficient:
  readback must match T0 and intended content with intact namespace.
  `futimens` may change ctime; ctime equality is not required. No pathname
  reopen, metadata change on a replacement, or numeric Godot Resource/
  document mtime read/set.
- Explicit stage/interval, changed-version/source, namespace-loss,
  permission/ownership, closed fd and other error/unknown facts. This is
  **application evidence only**, never an independent Rust D observation,
  client-provided `saved` authority or a persistent token.

Before content I/O, recheck D bytes/identity, still-open B/R binding and
current no-follow path attachment. Before mtime restore, recheck complete
content receipt plus frozen post-edit current/preparation saved versions,
exact B/R text and bound document/session/path/namespace; equal text at a
newer version cannot pass. Recheck `fstat`/content/namespace after restore,
then again before finalization. Changed/lost leaf or parent namespace
**refuses restoration and tagging** even when the displaced fd contains
intended bytes; never retarget, reinstall or rollback. A failed/unknown
metadata syscall, mismatched T0/identity/readback or lost fd cannot tag.
Content success followed by metadata failure is known application/partial:
`AppliedUnverified` for known `Applied` or `PartlyApplied`, **not**
`ApplicationUnknown`, `Refused`, `NotApplied` or success. Missing metadata
or later readback can make that observation unavailable, not erase known
application. The §18 fault probe's actual EBADF after
content-write success demonstrated this boundary; a separate attempted
invalid-nsec syscall returned success on that host but wrong normalized
mtime, and the readback guard refused tagging. Do not assume EINVAL.

The earlier patched receipt-fed private fields, §13 Save/saver paths,
§14 P1 handler Callable and §17 un-restored direct tag are **superseded for
production**; retain their research and pinned-source links as historical
evidence. Descriptor persistence/restoration is not crash-atomic storage,
an arbitrary same-inode compare-and-write, or cross-platform support.

## 3. Primitive A: guarded target-document saved transition

### Existing public ingredients and preconditions

"A" is the **semantic** saved-state obligation, not a new
`finalize_script_document` engine method or an established native/bridge
schema. T1 uses supported `Script.set_source_code`,
`EditorInterface.set_object_edited(Script, false)` and
`CodeEdit.tag_saved_version()` together with a retained-fd macOS `futimens`.
No saved-handler connection/Callable discovery or native handler call,
private Godot mtime access, `resource_saved` emission or path-based Save.

Before each effect, require all of:

1. Live authenticated request/session/attempt slot, same held Script,
   ScriptEditorBase and CodeEdit IDs, path and unique still-open association;
   expiry/cancellation/closed or same-path reopened tab cannot qualify.
2. Fresh exact B source and frozen **post-complex-edit current** version,
   unchanged **preparation saved** version, and exact R source on the same
   Script. A newer human version is ineligible even with the same text.
3. Exact retained fd identity and current no-follow leaf **and parent**
   namespace attachment, content readback and same-attempt complete content
   receipt; before finalization also require successful same-fd metadata
   restoration of T0 confirmed by `fstat` plus fresh content/namespace
   readback. Wrong, missing, failed or unknown phases cannot authorize tag.
4. Captured native Save-format profile remains eligible; no unrelated
   document/history or forbidden effect is admitted. No focus change to
   manufacture eligibility; unsupported topology refuses.

### Selected T1 order and effects

Prepare exact clean target/Save profile/source/versions and `fstat` the
retained fd mtime T0. One CodeEdit complex operation changes only B and
preserves earlier native history; compare B/current and original R, call
the bound `Script.set_source_code` method with desired source and read R
back. Do not assign via `Object.set`/GDScript `_set`, call
`ScriptTextEditor::apply_code`/`update_exports`, reload, or propagate pending
dragged-export properties. Fresh guards precede the single logical same-fd
content-persistence phase (`pwrite` as needed for eligible short writes,
`ftruncate`, `fsync`, `pread`). After its readback,
repeat all source/version/namespace/receipt checks; **only then** invoke
`futimens` on that retained fd, atime `UTIME_OMIT`, mtime T0. Verify `fstat`
T0, unchanged retained identity, exact fd content and no-follow namespace
attachment. No path reopen or content write during restoration.

Freshly recheck both receipt phases and all binding/source/version/namespace
guards before `EditorInterface.set_object_edited(held_script, false)`.
Repeat the same guards **after** that setter, then directly
`CodeEdit.tag_saved_version()` and **STOP**. No event pump, yield, focus
switch, second write, Save, signal broadcast, Callable or follow-on handler
callback between final guard and tag. Final independent observations follow;
they are not another finalization effect. A human edit after tagging is
dirty and must not be overwritten/restored. The documented stock
[external-change test](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp#L809-L854)
compares remembered document seconds with path mtime; macOS
[`FileAccessUnix::_get_modified_time`](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/drivers/unix/file_access_unix.cpp#L375-L394)
returns `st_mtime` seconds. The §17 un-restored tag left the later Save
behavior broken; §18's T0 restoration allowed the tested stock Save/history
paths without reading or setting a private Godot Resource/document mtime.
No arbitrary private parity, active hot reload or effect-free ordinary
editor continuation is inferred from that behavior.

### Results, independent observation and failure

Record attempt-local distinct `write_started`/content bytes/truncate/fsync/
pread results, `futimens` invocation/error, pre/post-restore `fstat` mtime,
fd content/identity/namespace readback, edited-flag clear, fresh pre-tag
guard, tag attempt, versions/dirty and actual errors/unavailable states.
`complete`, `rejected` and `partial_or_unknown` are **conceptual stage
descriptions**, not a new wire shape or success by receipt alone.

| Condition | Required result |
|---|---|
| Closed/replaced target, wrong Script/editor/CodeEdit/session, changed Save profile | Refuse the next effect; if B/R/D or edited flag already changed, report the exact applied/partial steps. Never retarget. |
| Changed current **or** saved version, even equal desired text, newer B or unexpected R | Refuse content/restore/tag at the next guard; leave newer human state unchanged. Earlier D content may be known applied. |
| Lost/replaced leaf or parent namespace, mismatched fd identity/content | No metadata restore or tag after detection, no path reopen, saver fallback, rollback or repair. Preserve actual displaced-object write and namespace facts. |
| Incomplete/failed content phase, missing/unknown receipt | Refuse restore/tag; preserve actual write/bytes/error and Rust application certainty. |
| Complete content application but failed/unknown `futimens` or fstat/content/namespace readback (including closed fd) | No tag; report `AppliedUnverified` for known `Applied`/`PartlyApplied`. Unknown later metadata, attachment or readback remains an observation-level unknown, not `ApplicationUnknown`. §18 observed actual EBADF after a 51-byte write; do not assume invalid nanoseconds return EINVAL on macOS. |
| Invalidation after public edited=false or after tag | Report known bookkeeping/tag as `AppliedUnverified` when later checks fail; keep specific unknown observations without downgrading known application. No rollback/cleanup write or implicit retry. Newer human B after tag remains dirty. |
| Complete content + metadata phases, fresh final guards and tag, independent immediate readbacks | Candidate saved-state witness **only**; independent Rust D/R/B/dirty/context and explicit post-change validation gate product success. §18 separately exercised later Save/history, reopen and runtime, but did not pass product A–E. |

Independently read actual still-open association, public Resource edited flag,
CodeEdit current/saved versions and dirty state, R/B, and Rust's fresh D and
namespace. A metadata receipt, syscall return, public tag or edited-flag
acknowledgment cannot replace those observations. Neither private numerical
Resource/document mtime equality nor full saved-handler callback replay is
a requirement. §14–§17 and PR #31/#33 evidence is **historical, superseded
for selected production**, not deleted or interpreted as T1 product proof.

## 4. Primitive B: exact-source native GDScript validation

This section specifies the independently required exact-source stock validator and records T002's completed patched-editor semantic oracle/reference and hazard inventory. The example native binding below is **historical**, not a selected stock method, implementation shape or wire schema. [§18 T1](../research.md#18-preserved-mtime-behavioral-finalization-research-2026-09-28) selects handler-free saved-state behavior only; it neither implements nor validates the explicit parser/analyzer/diagnostic obligation. [§15](../research.md#15-stock-validation-and-effect-confinement-research-2026-09-28) and [§16](../research.md#16-stock-saved-handler-admission-research-2026-09-28) remain historical handler/effect research, superseded as production handler-admission requirements. Generic forbidden-effect safety remains.

### Inputs and output

`validate_gdscript_source(source, source_path, read_context, correlation)` runs synchronously on the selected editor main thread. Inputs are an immutable exact source string, exact project-confined `res://` root path, a native-only selected-project read context, and existing request/session/document identity plus purpose `preflight`, `post_change` or `unchanged`. No mutable global buffer is the implicit input. Compute SHA-256 from the actual supplied UTF-8 bytes; compare the integration's expected hash before invocation. The root target is already open/loaded; validation does not open or load it.

The **historical T002** engine exposure delegated to `GDScriptLanguage::validate` → `GDScriptParser::parse(source, path, false)` → `GDScriptAnalyzer::analyze` with the scoped effect/dependency policy below and returned actual root/dependency `ScriptError` records. Stock must still produce real completed parser/analyzer semantics; no stock binding, helper or source port is selected, and reload, `can_instantiate`, log silence or empty diagnostics are not validity witnesses.

Return exactly one status:

- `valid`: the requested parser/analyzer stages completed under the selected policy for this exact source/context, without errors.
- `invalid`: a completed validation rejected this source/context; retain root/dependency diagnostics and the failed parser/analyzer stage when supplied.
- `unavailable`: invocation incomplete, context/attribution invalid, disallowed effect, access failure, resource bound, wrong thread or reentrant invocation. Partial diagnostics do not turn it into valid/invalid.

Every result includes request/session/document binding, purpose, actual input path/hash/byte length, editor-clock start/end ticks, result/reason, dependency/context witnesses and `diagnostics_complete`. Diagnostics carry `origin: root|dependency`, confined path (or null with reason), source hash when actually read, line/column when supplied, category and message. Unknown dependency source is not assigned the root hash. Missing dependency proven by a confined read can yield a completed dependency error; an unreadable or unconfined dependency yields unavailable. Redact denied/outside-project paths; never disclose their source or raw absolute paths. Errors/messages appear only in the requested result, not incidental logs.

Limits: root and each dependency ≤512 KiB UTF-8; at most 32 distinct source dependencies / 4 MiB dependency bytes; at most 64 error records with messages ≤2048 UTF-8 bytes and paths ≤2048 bytes. Exceeding a needed bound yields `unavailable` with limit reason and safely retained partial diagnostics, not truncated source or false valid. Warnings/functions/safe-line UI data are not a new caller diagnostics API.

### Required effect policy: source analysis, not arbitrary evaluation

Allow the existing parser/analyzer, engine-native static type metadata, current project/global-class/autoload mapping lookup, confined read-only GDScript dependency text, normal parser/dependency-cache lookup/population and dependency-edge bookkeeping. Cache mutation is expected engine bookkeeping, not a purity failure. Do not clear global caches to simulate isolation.

T002 implemented a **patched-editor validation-call-scoped read/effect context** through the implicated GDScript parser/cache/analyzer paths and proved its confined dependency/effect behavior in 54 native cases. It is a completed semantic oracle/reference and hazard inventory, not a product patch or qualifying equivalent in official stock Godot. The following policy remains an independent stock requirement; it does not prescribe porting the patch or introduce a stock public guard API:

1. All dependency bytes are obtained through the trusted native integration's project-directory capability reader, with no-follow traversal and before/after identity/content checks. Resolve each relative dependency against the directory of the source containing that reference: the supplied root path for root references, and Godot's per-dependency source paths for transitive references. Resolve permitted project mappings to a confined `res://` path before reading. No absolute/user/network/embedded/remapped-outside-project fallback. Unsupported binary/remapped dependency representations are unavailable, not silently loaded.
2. A cache hit is usable only when attributable to the same project/path and current guarded input digest. A current `Script.source_code` getter does not prove cached parser/analyzer/shallow metadata was derived from that source generation. Stale/unattributed metadata cannot certify the proposed source. T002 used engine parser types and call-local references to analyze the root overlay/current dependency bytes without replacing the loaded Script or clearing global caches; that is reference behavior, not a selected stock port. A stock solution must independently account for cache provenance and fresh per-call read witnesses.
3. Permit shallow GDScript metadata construction only without full reload, script instantiation or initialization. Prevent non-GDScript `ResourceLoader` resolution **before** invoking loaders (including exists/type hooks), full-script loading, and dynamic object-property `Variant::get/get_named` paths. An unadmitted path returns `unavailable/unsupported_effect` rather than invoking project code. Plain built-in value operations and engine-native static metadata remain allowed. Do not substitute guessed types or skip analysis and return valid.
4. No user method, tool/static initializer, scripted constructor/getter, arbitrary loader/callback, process/network action, source write, root R/B assignment, document save or project scan is permitted as an effect of this call. `@tool` syntax alone is not execution or an automatic rejection; a resolution path requiring an excluded effect is. Unsupported effectful validation is a truthful capability refusal, not a new evaluation permission prompt.
5. Capture and recheck dependency identities/hashes and relevant project/class/autoload/remap context used by the invocation. Changed/missing witnesses make the result unavailable/invalidated. The result describes an observed context, not an atomic snapshot or project-wide compilation.

T002's patched GDScript-owned validator reused Godot's parser/analyzer rather
than building a second compiler; its patched-editor acceptance remains
complete. On stock, public ClassDB/native class, global-class and autoload
mapping metadata is available, but demonstrated metadata samples do not prove
analyzer parity or current live-editor generation. `reload(false)` omits export
refresh, **not** parsing/analysis/compilation or conditional static
initialization. In the measured `--headless --check-only --script` helper,
source is fully loaded before EditorNode disables scripting even with
`--editor`: tool and non-tool static initializers executed, and all observed
invalid cases exited 0. Neither exit status/log silence nor broad reload
qualifies as an attributed valid/invalid/unavailable result. Restricted
helper/source eligibility and public-ABI parser/analyzer adaptation remain
unselected, unexhausted design-review alternatives; a conservative source
scanner can refuse hazardous syntax but cannot certify semantics, and does
not silently turn `@tool` alone into a rejection rule.

The remaining architectural hold is a stock explicit validator satisfying
the above source/context attribution, completed-status and confined read/
effect requirements. Ordinary edit-generated validation/export must still
preserve human work and avoid forbidden effects as implementation/acceptance
invariants, not as a separate saved-handler admission prerequisite. The
bridge cannot supply executable read callbacks. Existing exclusion of
hostile code already executing inside the editor/same-UID malware remains
unchanged.

### Guarding the ordinary editor path

Successful ordinary non-tool `_validate_script` can copy B into R and call `update_exports`; `apply_code` and export propagation can assign source/export state and pending dragged-export properties. Preserve the general obligation to contain actual ordinary editor effects and verify resulting source/identity independently; do not globally disable validation, typing or other human work. T002's historical engine guard does not exist on stock. T1 does **not** call a saved handler, so §16's additional incoming Callable topology, `get_functions` names tail, saved-handler immediate live reload, consumed handler callback-generation/reverse-consumer admission and shared deferred debugger routing/completion are **historical, not selected production machinery**. That cutover does not certify ordinary editor work is pure, nor turn natural editor validation into explicit completed exact-source validation.

If explicit stock validation cannot independently establish completed source-attributed `valid`/`invalid`/`unavailable` with confined dependency/effect handling, T003 remains blocked; conservative source refusal is not a substitute parser or a silent `@tool`-only policy. Existing generic no-forbidden-effects and truthfulness obligations stand, without importing a hypothetical handler path's extra admission blocker. Historical patched hooks, restricted helper and unproved saved-handler tail replacement are not selected production fallbacks.

### Placement in the edit flow

For changed intent, run B on proposed source during preparation to refuse invalid/unsupported-effect inputs before changing B or history. Then freshly recheck mutation guards; preflight does not authorize a delayed edit. After source application, same-fd content persistence/T0 restore and guarded public edited=false/CodeEdit tag/STOP, read actual resulting B/R and invoke B again with that exact source as `post_change`. This second result is the required post-change parse witness. Dependency/context changes can still produce invalid/unavailable after earlier valid; report applied-unverified and do not force convergence. T1 solves the §17 later Save/history failure in bounded stock evidence; it does not solve exact-source validation or generic effect safety.

For unchanged requests, skip proposed-source preflight and run one `unchanged` validation with independent verification, without native text mutation, writer, finalizer, saved tagging or history entries.

## 5. Implementation proof obligations

The [quickstart](../quickstart.md) supplies the acceptance matrix. For selected T1, prove same bound Script/editor/CodeEdit association, closed/same-path-reopened target refusal, changed current **or saved** version (including same-text newer version), save-format eligibility, project/file identity and no-follow leaf/parent namespace stability. Prove one logical content-persistence operation (`pwrite` calls as needed to complete eligible short writes, then truncate/fsync/pread) with explicit actual call count/errors/bytes/readback, **separate same-retained-fd T0 `futimens`** (atime omitted) with actual errno/unavailable and `fstat`/content/namespace readback, and no restoration/tag after wrong/failed/unknown receipt or namespace loss. A complete content write followed by EBADF or other failed restoration must remain known partial application, never false refusal/success; a normalized wrong mtime despite successful syscall also cannot tag. Prove edited-flag-only partial bookkeeping, post-tag human edit preservation, no retargeting/rollback/second logical content-persistence operation, preserved unrelated dirty work/history/focus during the candidate, and independent D/R/B, public edited flag, dirty/current/saved versions. Prove later **delivered** ordinary human Save, Undo → Save → Redo → Save plus earlier native history, reopen/fresh launch, scanning and fresh runtime without a false external-change prompt. No private Godot numeric mtime equality or globally atomic intermediate state is necessary. The §18 evidence comprises **15 final-campaign accepted cases and separate settled current-build history = 16 accepted observations**, not 16/16 campaign or product A–E acceptance. §14 P1, §16 H3 and §17 un-restored failures remain historical.

Independently prove a qualifying **stock explicit validator** with valid/invalid/unavailable attribution, relative source dependencies and changed dependency/context, confinement on actual reads/cache hits, allowed cache activity, refusal before excluded loaders/getters/initializers, and no root D/R/B writes. Keep generic confinement and ordinary-editor forbidden-effect safety for work actually caused by the selected edit; the eliminated saved-handler's callback generation/function-discovery/live-reload/deferred-debugger admission profile is not another production gate. Include source-setter versus property/reload paths, nested effects and cleanup without changing unrelated/human behavior. T002 patched B acceptance remains complete but cannot discharge stock obligations.

T003 remains **unstarted** on the [§18 T1 disposition](../research.md#18-preserved-mtime-behavioral-finalization-research-2026-09-28): saved-state finalization is selected conceptually, **explicit exact-source stock valid/invalid/unavailable validation is the remaining architectural blocker**. Generic effect safety, source/history/confinement/partial-result invariants and all A–E/durability acceptance remain mandatory implementation gates, not second handler-era design holds. T004 owns eventual migration of T001's historical numeric `SavedStateEvidence`; no new schema, task count/dependency, specification change, patched editor or custom saver is introduced. Other platforms are unresolved.

## 6. Pinned implementation basis

All engine references use `ed1daf0bf001b61586d9930840f2f1394092c079`:

- [ScriptEditor document tag](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_base.cpp#L91-L93), [CodeEdit saved-version tag](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/scene/gui/text_edit.cpp#L4890-L4900), [external-change check](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp#L809-L854), [Unix `st_mtime` getter](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/drivers/unix/file_access_unix.cpp#L375-L394) and [public Resource edited setter](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/editor_interface.cpp#L714-L721): T1 restores the descriptor's T0 before direct public tagging; no private numeric equality is prescribed.
- **Historical, superseded for production:** [ScriptEditor saved handler](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp#L723-L803), [names tail/current editor](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp#L1897-L2123), [function discovery](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_text_editor.cpp#L210-L224), [incoming connections](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/core/object/object.cpp#L1411-L1421) and [EditorNode Save notifications](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/editor_node.cpp#L1752-L1775). These explain §14–§16, not a T1 implementation dependency.
- [Script source setter/property handler](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript.cpp): `set_source_code` changes source/source-changed bookkeeping; `_set` for script source additionally reloads.
- [Native editor application/validation](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_text_editor.cpp): R assignment/export update is in the successful non-tool validation branch, not the failed-validation branch; guard the actual call paths.
- [Validator](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript_editor.cpp), [analyzer effect sites](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript_analyzer.cpp), and [dependency cache/shallow loading](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript_cache.cpp): source-specific validation is reusable, but effect control and confined source resolution must be added rather than presumed.
