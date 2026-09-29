# Native Script Editing Integration Contract

**Status (2026-09-29):** T001–T003 are complete under [§18 T1](../research.md#18-preserved-mtime-behavioral-finalization-research-2026-09-28) and [§19.9 R1 / L1](../research.md#199-capability-delta-readiness-review-2026-09-29). [T003 acceptance](../quickstart.md#11-t003-native-boundary-acceptance-2026-09-29) and the [current private implementation](#7-current-private-implementation-boundary) describe the official-stock path. T002's patched validator is accepted historical evidence in [PR #29](https://github.com/Peter-Tam/godot-agent-kit/pull/29), removed from HEAD and not a fallback. T004/T005 remain pending; the inherited endpoint limitation is non-blocking and no public edit capability exists.

## 1. Ownership and minimum exposure

**T003 private boundary implemented, public product edit not yet exposed:** Official stock Godot plus addon and standard public-ABI C++17 GDExtension implement one guarded CodeEdit complex edit, explicit bound `Script.set_source_code`, descriptor-bound content persistence, same-fd T0 restoration, then public Resource edited=false and direct CodeEdit tag/STOP. No saved-handler Callable, ResourceSaver, full Save, engine patch or private Godot timestamp setter is used. T004 owns caller/core/bridge integration and feature A–E behavior.

| Responsibility | Owner / selected mechanism / proof obligation |
|---|---|
| Intent, stale/conflict policy, authorization, deadlines, uncertainty, final classification | Existing Rust library and supervised caller/worker. |
| Authentication, routing, connection/attempt lifetime, marshaling | Existing private bridge and Godot integration. |
| Native B edit, explicit Script R setter, retained-fd D content and metadata | Standard GDExtension through supported object methods and demonstrated macOS descriptor APIs. Exactly one logical content-persistence operation (`pwrite` calls as needed for eligible short writes, then truncate/fsync/readback), then a separately checked same-fd `futimens` restoration of T0 (atime omitted). |
| Guarded saved transition and independent public saved-state evidence | Fresh target/source/version/receipt/namespace guards before the metadata syscall and before edited=false, then recheck and direct CodeEdit tag/STOP. Independent D/R/B, Resource edited flag, CodeEdit versions/dirty and exact association plus later ordinary Save/Undo/Redo/reopen must corroborate. No private numerical Resource/document mtime requirement. |
| Exact-source validation and confined effects | **Implemented T003 private boundary:** existing supervised Rust worker owns bounded capture/staging, one-shot stock LSP child/protocol, per-source diagnostics/symbol fences, deadline and reaping. Addon/native retains editor getters/mutator. T004 still integrates authenticated typed caller/bridge/core evidence; T002 patched bindings are not a fallback. The inherited endpoint limitation is non-blocking, not a new single-client or global sandbox requirement. |

The earlier proposed engine API family revision **1** and T002's patched `GDScript` validator/ClassDB bindings are historical design/acceptance, not stock bindings. The now-removed implementation is archived in [PR #29](https://github.com/Peter-Tam/godot-agent-kit/pull/29). T003's editor-local key is exactly `godot_agent_kit_native` with private revision `1`, not an advertised bridge editing capability; see the [stock native guide](../../../godot-addon/native/README.md#build-and-private-integration).

Preparation also inspects the target's native save-format settings and checks both the baseline and desired source against the same engine formatting rules in a non-mutating/check-only path. `save_current_script` runs `_auto_format_text` (trailing whitespace, final newlines and indentation); a later Save must not silently change the promised revision or destroy the Undo → Save → Redo case. Return `unsupported_representation/save_would_reformat` if either source would change, or if the relevant behavior cannot be established. Retain/recheck this fixed save-profile witness at application and verification. Never alter editor preferences, run formatting on the live document, or add a general formatter API. Representation limits are explicit; later independent human source/settings changes remain outside the successful interval.

The attempt-local T1 guard binds request/session, actual document objects, source/current-and-saved versions, retained descriptor, original T0 and the separate content/metadata receipt; it is **not** a new engine guard object or transaction/session UUID. Recheck binding, receipt, identity and versions immediately before each native effect, including after edited-flag clearing and before direct tagging. It neither locks out human work nor serializes external filesystem writers: identity/source/version/namespace changes invalidate it, without repair. Intermediate D/R/B divergence is allowed with accurate partial classification and independent final observations; no every-step external atomicity claim. The selected stock helper requires prelaunch admission and confined staged inputs; its inherited endpoint limitation does not establish a new in-scope authority. Ordinary-editor effect safety remains a separate implementation gate.

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

**Stock LSP validator at L1, implemented privately by T003:** [§19.9 R1](../research.md#199-capability-delta-readiness-review-2026-09-29) selected one fresh matching official executable in a disposable source-only project per captured proposed/actual closure. T003's Rust worker implements bounded no-follow capture, prelaunch admission, staging, exact child attribution, per-URI diagnostics/symbol fences and owned teardown; the [accepted GUI/worker evidence](../quickstart.md#11-t003-native-boundary-acceptance-2026-09-29) is distinct from earlier research. T004 binds these facts to the public authenticated caller. The inherited stock endpoint limitation stays non-blocking.

### Prelaunch capture, admission and effect boundary

The implemented helper captures exact proposed root bytes for preflight, or independently read actual resulting root for `post_change`/`unchanged`, and recursively captures supported literal `.gd` inheritance/preload dependencies against each **referring source**. It stages this bounded immutable closure and no other selected-project scripts (root and each dependency ≤512 KiB UTF-8, at most 32 distinct dependencies / 4 MiB dependency bytes). `initialize` analyzes discoverable staged `.gd` before `didOpen`, so root-only capture is insufficient. Staging plus exact owner `didOpen` and per-URI fences attribute the studied owner's root/source-only results; inverse two-client dependency probes establish that the owner parser's helper dependency follows **staged disk**, not the other client's managed overlay. This is source/result attribution, not an OS filesystem sandbox claim. Captured identity/hash/bytes through staging and JSON-RPC `text` plus rechecked selected context are attribution witnesses, not proof of editor-cache purity, atomic arbitrary-writer snapshot or selected-editor memory/physical-path parity. External parser cache-generation parity and process-wide input exclusivity are not mandatory.

A bounded lexical recognizer plus confined reads before process start conservatively admits supported literal relationships; it is not a second GDScript parser or generic effect system. Native bases/ClassDB and same-build inert source-only `@tool`/static/init/new declarations need not be rejected merely by spelling. Unsupported/unresolved/computed/escaping or unreadable dependencies, non-`.gd` resources, UID/remap/binary/embedded dependencies, out-of-project/link-probed paths and unreproducible global-class/autoload/GDExtension/custom-loader/object-value context remain `unavailable`, never guessed valid/invalid. Document-link existence probes use `FileAccess::file_exists`: admit **all** link-probed string literals, not only dependency expressions. Conservative admission controls the supervisor's **own staged input**; it is not a new isolation requirement for inherited stock-LSP access.

For a supported literal `.gd` relationship, a confined no-follow read proving absence records a missing witness bound to its referring source/logical path, not a synthetic staged URI/fence. Every existing admitted owner source needs its own completed fence; only a corresponding severity-1 semantic/root diagnostic for each missing literal after those fences establishes owner `invalid`. An absent diagnostic or incomplete fence is owner `unavailable`, not `valid` or partial `invalid`. Unreadable, unconfined or ambiguous resolution is `unavailable` before launch, not a proven-missing input. A separate client's own parse cannot supply an owner witness or turn otherwise complete owner evidence into unavailable.

The selected private root has fresh HOME/XDG config/data/cache and minimal `project.godot` (config_version=5 and project name) plus bounded selected-project effective GDScript warning enable/per-warning levels/applicable directory rules, including selected editor overrides. Unreproducible required context must refuse rather than copy addons, plugins, autoloads, scenes, resources, old `.godot` or UID/remap/extension state. Dependency warning-as-error can change validity. Godot `.gd.uid`/`.godot` bookkeeping inside the disposable clone is distinct from selected-target effects. The clone did not write the owned outside source in invocation 67; it also did **not** prevent a second client from reading that source. Private HOME/XDG and rootUri isolate configuration/context, not OS filesystem authority; the inherited endpoint limitation does not demand a global Godot sandbox.

Source audit: `initialize` synchronously invokes workspace `reload_all_workspace_scripts`, parsing/analyzing **all** staged scripts before response or `didOpen`; the headless first metadata scan parses without analyzer, while GUI documentation full-loading is skipped in headless cmdline mode. The LSP parser runs actual parse then analyze, diagnostics and symbols; it does not compile/initialize pure `.gd` declarations. Literal inheritance can traverse inherited bodies, but an unused `.gd` preload need not fully analyze its dependency body in the root stream. Direct `.gd` preload makes shallow scripts, not instances; non-`.gd` preload can load a scripted Resource and execute constructor/getter/formatting paths before `didOpen`. Document links may probe arbitrary paths, and reverse scene-cache loading is a risk with resource owners; the fresh `.gd` dependency producer returns an empty list, not a demonstrated `.gd`-owner execution path. These findings justify **prelaunch** admission of our staged source-only profile and ongoing ordinary-effect gates, not an unrestricted stock parser safety assertion or a new architectural blocker.

**Retained owner evidence and inherited endpoint limitation:** [Retained invocation 67](file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-one-shot-lsp-39y7q6v3/evidence/peer-verified.json) used two separate TCP connections in one same-UID synthetic harness process. After owner `initialize` but before its `didOpen`/fence, the second client received source-derived constants by `documentSymbol` for the staged root and an owned sibling `.gd` **outside the helper project**, without sending text; it then `didOpen`ed the **same root URI** at version 37 with distinct inert invalid source. The owner's subsequent version-1 `didOpen` retained its exact captured text and produced empty URI diagnostics plus the PRIVATE symbol; the second client's connection produced severity-1 typed-invalid diagnostics plus its SECOND_ONLY symbol. The [inverse dependency probes 68/69](file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-one-shot-lsp-39y7q6v3/evidence/peer-dependency-verified.json) used owner version-1 root/helper fences and peer version-37 opposite helper text: staged-disk `int`/peer `String` gave owner `valid`; staged-disk `String`/peer `int` gave owner `invalid` with the expected severity-1 root return-type diagnostic. The dependency-parser producer reads staged file source; peer-managed parser state is separate. Thus owner root/per-URI and staged-disk dependency outcomes were attributable in **both directions** for the tested source-only path; no managed-text overwrite, mixed diagnostic, false owner result or cache poisoning was observed. These are not every-method/global-cache proofs, selected-editor/GUI nonmutation runs, cross-UID tests or observed Resource execution. At pinned Godot `ed1daf0bf001b61586d9930840f2f1394092c079`, [count-only peer admission](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/language_server/gdscript_language_protocol.cpp#L142-L150), [absolute outside-project URI retention](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/language_server/gdscript_workspace.cpp#L502-L578), [unmanaged `.gd` FileAccess reads under helper authority](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/language_server/gdscript_language_protocol.cpp#L407-L440) and [source-derived symbol values/docs](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/language_server/gdscript_text_document.cpp#L145-L156) support the **inherited stock source-read capability** without a caller-authentication/user-identity check in the reviewed path. No new in-scope authority beyond normal stock editor use is established. Source review also identifies [stock `willSave`/`didSave` Resource paths](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/language_server/gdscript_text_document.cpp#L83-L118); those effects were **not** observed here and do not establish a new architectural blocker.

**Non-blocking FR-020 endpoint limitation:** The validator uses stock Godot’s short-lived loopback LSP listener. Stock LSP is not authenticated per OS user in the reviewed request path and can process helper-readable GDScript paths. Hostile same-UID software is outside the project threat model, and no cross-user disclosure or other in-scope capability increase was demonstrated. This is an inherited local-endpoint limitation, not an agent-facing arbitrary-filesystem capability. Hostile shared-host isolation is not a supported deployment guarantee. [§19.9](../research.md#199-capability-delta-readiness-review-2026-09-29) records exact requirement provenance, comparison and the missing blocker fields. Existing authenticated/project-confined product operations and source-free metadata remain mandatory. Untested cross-UID access and the declined experiment are limitations, not technical failures or prerequisites; no retry, sandbox, exclusivity, proxy/firewall/daemon or authentication framework is prescribed.

### One invocation and completed source fences

The researched no-log invocation launched a matching version/hash official binary as `--headless --editor --path <owned-project> --lsp-port <unique-loopback-port>` with **no `--log-file`**, helper stdout/stderr discarded, and child-PID loopback listener verification. These controls isolate the owned context and prevent wrong-child submission; they are not an OS sandbox. The selected helper uses TCP Content-Length JSON-RPC: `initialize` with cloned rootUri/rootPath/workspaceFolders and `hierarchicalDocumentSymbolSupport=true`, await response, send `initialized`, then once for **each exact admitted root/dependency URI** `textDocument/didOpen` (`languageId=gdscript`, `version=1`, captured UTF-8 text), followed by unique-id `textDocument/documentSymbol`. Exact-URI `publishDiagnostics` and shaped parsed-class symbol response localize each clone URI to its logical `res://` source. No `didSave`, completion, edit notification, reused child or changing source generations belongs in **our client recipe**; the owner fences remain valid if a separate client acts independently and their own messages are not mistaken for owner results.

The developer’s normal editor LSP remains independent and untouched: never reuse, stop, reconfigure or interfere with it. Helper correctness must not depend on its enabled/disabled state. Use the owned helper’s private configuration, disposable project, unique loopback port and lifecycle.


The [pinned stock logger setup](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/main/main.cpp#L2312-L2339) disables editor file logging by default, but explicit `--log-file` forces a file logger even when Project Settings disables it. §19 intentionally used `--log-file` for early synthetic provenance; that historical launch is **not** a future helper privacy recipe. The selected helper must neither request a diagnostic log nor retain/forward source-bearing startup output; cleanup cannot substitute for privacy. No global change to selected editor logging is proposed. The inherited endpoint limitation is separate from actual no-log privacy acceptance.

The separate [official-stock privacy control](file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-one-shot-lsp-39y7q6v3/evidence/privacy-verified.json) exercised this no-`--log-file`, discarded-stdout/stderr recipe with a private-sentinel diagnostic in a literal `.gd` preload. Both root/helper source fences completed, the helper error appeared only in the requested bounded LSP diagnostic, and inspection of all owned project/HOME/XDG files found no `.log` or stdout/stderr capture files and no sentinel outside the staged helper source. The child was reaped after **2.494936 s**. This was one privacy-only control after the 65 source/closure study invocations, not another final-closure case, selected-editor nonmutation witness or production privacy/acceptance pass.

Symbols alone also exist for invalid source; empty diagnostics alone can mean no completed parse. The candidate all-staged-source fence, not root-only diagnostics, addresses observed false-valid root-only handling of an invalid unused preloaded dependency: initialize can emit unmanaged errors, LSP publishes only opened URIs and every existing staged source must be independently fenced, including warning-as-error cases. With attributable owner root/per-URI source and context, complete diagnostics-plus-symbol fences with severity-1 errors support owner `invalid`; complete fences without severity-1 or missing witnesses support owner `valid`. Incomplete/mismatched fences, undiagnosed missing witnesses, source/context change, limits, unsafe admission or deadline mean owner `unavailable`, retaining partials. A second accepted client **alone** cannot downgrade valid/invalid owner evidence to unavailable; the inherited endpoint limitation is non-blocking, while actual product privacy acceptance remains required.

The **selected conceptual minimum** is `valid|invalid|unavailable` with request/session/target and `preflight|post_change|unchanged` correlation, root logical path/hash, relevant source path/origin/hash and actionable bounded error diagnostics, closure/context witness, completion/incomplete reason and supervisor-clock invocation interval. This is **not** a selected wire schema, T002 historical diagnostic layout, engine-returned SHA/clock or parser-stage/`diagnostics_complete` requirement. Bound messages/count/path and redact denied physical paths, source and incidental logs. Close socket and terminate/kill/reap the owned child on completion, loss or deadline under existing 9.5-second operation plus 0.5 delivery reserve; stock `shutdown` returns `-32601` and `exit` does not terminate it. Unavailable post-change validation cannot undo known application or become success.

### Historical T002 patched-editor reference — not stock instructions

T002's **patched** `GDScript.gdscript_validation_api_revision()` (validation revision `1`) and `validate_gdscript_source(source, source_path, read_context, correlation)` ClassDB binding ran synchronously on its editor main thread, using `GDScriptLanguage::validate` → `GDScriptParser::parse(source, path, false)` → `GDScriptAnalyzer::analyze` under a call-scoped confined read/effect context. That historical call bound request/session/document, purpose, actual supplied UTF-8 root hash/path/length and **editor-clock** interval; it returned `valid|invalid|unavailable`, dependency/context witnesses, `diagnostics_complete` and root/dependency `ScriptError` records with path/hash/position/category/message when supplied. It guarded cache attribution, dependency reads (including transitive relative paths), non-GDScript loader exists/type hooks, full loads, dynamic object-property getters and project-code execution before dispatch. Its 54 native acceptance cases establish a useful semantic oracle and hazard inventory, not a stock API, production admission scanner or new wire contract. Its separate proposed `inspect_script_document`/edit-guard/finalizer family, private numeric timestamps and saved handler are likewise historical.

The old broad `reload(false)` or `--headless --check-only --script` approach is not a qualifying helper: reload can compile/initialize, and check-only ran static code while exiting 0 even for invalid input. §19’s selected source-only profile and per-URI fences establish **scoped owner parser attribution** on the tested path, with a non-blocking inherited stock endpoint limitation; ordinary editor validation cannot substitute for explicit exact-source validation. Keep confinement, no-forbidden-effects, privacy and independent-result requirements as later implementation gates, not extra architectural blockers. Hostile code already executing in the editor/same-UID malware remains outside this authorized local design's threat model; normal-local-user isolation is not waived.

### Guarding the ordinary editor path

Successful ordinary non-tool `_validate_script` can copy B into R and call `update_exports`; `apply_code` and export propagation can assign source/export state and pending dragged-export properties. Preserve the general obligation to contain actual ordinary editor effects and verify resulting source/identity independently; do not globally disable validation, typing or other human work. T002's historical engine guard does not exist on stock. T1 does **not** call a saved handler, so §16's additional incoming Callable topology, `get_functions` names tail, saved-handler immediate live reload, consumed handler callback-generation/reverse-consumer admission and shared deferred debugger routing/completion are **historical, not selected production machinery**. That cutover does not certify ordinary editor work is pure, nor turn natural editor validation into explicit completed exact-source validation.

The selected helper's inherited endpoint limitation is non-blocking under §19.9. That does not waive ordinary-editor effect safety. The implemented native admission checks both baseline and desired source before history entry and refuses tool sources, script/global-class inheritance, preload/load, class registration and exported declarations. A controlled cold-tool inheritance fixture demonstrated why: ordinary queued editor export work can initialize a dependency that the isolated source-only parser does not execute. The helper and live-editor profiles therefore have distinct effect boundaries; no engine patch or saved-handler machinery is a fallback.

### Placement in the edit flow

**Implemented T003 private flow:** Changed intent validates freshly captured proposed source/closure before guarded native effects; after T1 application and STOP, another fresh helper validates independently captured actual source/dependencies/context. Invalid or unavailable post-change validation preserves known application as partial evidence; it never implies rollback or caller success. T004 owns complete authenticated caller/core outcome integration.

Unchanged intent uses one non-mutating validation and independent verification without native edit, writer, finalizer or history entry.

## 5. Implementation proof obligations

The [quickstart §11](../quickstart.md#11-t003-native-boundary-acceptance-2026-09-29)
records T003's completed private proof: bound Script/editor/CodeEdit association,
closed/reopened and changed-version refusal, Save-profile eligibility, retained
fd/namespace and exact content/mtime receipts, no metadata restoration or tag
after missing/failing receipt, and known-partial classification when content
changes but restoration/readback fails. Actual native GUI witnesses include
ordinary Save without false reconciliation, Undo → Save → Redo → Save, earlier
history, preservation of human and unrelated work, and export isolation.

Its Rust helper acceptance exercises confined source/dependency capture,
effective warning/context admission, all required URI diagnostics/symbol
fences, valid/invalid/unavailable outcomes, stopped/failed-child cleanup,
discarded-output/no-log privacy and independent user LSP behavior. The
then-observed additional-client/disk-dependency probes support scoped owner
attribution, not global endpoint exclusivity. T004/T005 still own authenticated
caller and cumulative A–E/durability gates; earlier T002 patched results
cannot discharge those later obligations.

**T003 implemented and accepted the private stock boundary:** [quickstart §11](../quickstart.md#11-t003-native-boundary-acceptance-2026-09-29) records real-editor, helper, history/durability, privacy/export and observation regression evidence. The earlier [capability-delta analysis](../plan.md#capability-delta-readiness-correction-review) established L1 readiness before implementation; it is not a current unstarted status. T004 still owns typed/core/caller/bridge integration and historical numeric-mtime/patched-validator evidence migration; public A–E and cumulative feature gates remain pending. No patched editor, custom saver, extra task or wider-platform support follows.

## 6. Pinned implementation basis

All engine references use `ed1daf0bf001b61586d9930840f2f1394092c079`:

- [ScriptEditor document tag](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_base.cpp#L91-L93), [CodeEdit saved-version tag](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/scene/gui/text_edit.cpp#L4890-L4900), [external-change check](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp#L809-L854), [Unix `st_mtime` getter](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/drivers/unix/file_access_unix.cpp#L375-L394) and [public Resource edited setter](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/editor_interface.cpp#L714-L721): T1 restores the descriptor's T0 before direct public tagging; no private numeric equality is prescribed.
- **Historical, superseded for production:** [ScriptEditor saved handler](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp#L723-L803), [names tail/current editor](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp#L1897-L2123), [function discovery](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_text_editor.cpp#L210-L224), [incoming connections](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/core/object/object.cpp#L1411-L1421) and [EditorNode Save notifications](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/editor_node.cpp#L1752-L1775). These explain §14–§16, not a T1 implementation dependency.
- [Script source setter/property handler](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript.cpp): `set_source_code` changes source/source-changed bookkeeping; `_set` for script source additionally reloads.
- [Native editor application/validation](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_text_editor.cpp): R assignment/export update is in the successful non-tool validation branch, not the failed-validation branch; guard the actual call paths.
- **Historical T002 validator sources:** [patched-engine source-specific validator](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript_editor.cpp), [analyzer effect sites](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript_analyzer.cpp) and [dependency cache/shallow loading](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript_cache.cpp). These do not create stock patched bindings.
- **Selected stock LSP validator source basis:** [LSP initialize and workspace scan](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/language_server/gdscript_language_protocol.cpp#L269-L272), [all-workspace script parsing](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/language_server/gdscript_workspace.cpp#L191-L229), [parser/analyzer, diagnostics and symbols](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/language_server/gdscript_extend_parser.cpp#L967-L980), [peer state/routing and managed parsing](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/language_server/gdscript_language_protocol.cpp#L299-L325), [dependency parser disk-source producer](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript_cache.cpp#L217-L242), [document-link path probes](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/language_server/gdscript_extend_parser.cpp#L119-L147), [reverse owner loading](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/language_server/scene_cache.cpp#L39-L89) and the endpoint admission/URI mapping/Resource-read source anchors above. Owner-result source attribution and endpoint access privacy are different obligations.

## 7. Current private implementation boundary

T003 adds `runner::stock_validation` and private admission/protocol/ownership
modules in the existing Rust package. Its current executable consumer is
`examples/stock_validation_fixture.rs`, not a product edit CLI. Trusted integration
supplies request/session/project/purpose, proposed source or actual-D capture,
effective selected-editor warning policy and bounded global-class names.
Public ProjectSettings values can be StringName values; the fixture converts
actual class names to strings rather than silently dropping that context.
Disk configuration is an identity/change witness, not authority to replace
effective editor overrides. T004 must bind and recheck this context through the
authenticated caller/bridge; historical typed-core evidence remains unmigrated.

Helper child-spawn/reap facts are nullable after worker loss; they do not default
to a false claim that no child existed. Scratch cleanup has separate confirmation.
The supervisor reserves cleanup time inside its operation budget and uses the
existing detached reaper with a bounded acknowledgment, not unbounded waits or
filesystem deletion on its result-delivery thread.

The standard native metadata exposes private
`edit_prepare(path, expected_source, desired_source, correlation)`,
`edit_advance(request_id, expected_stage)`, `edit_cancel(request_id)` and
`edit_expire()`. Correlation contains request/session, exact document instance IDs
and an editor-clock expiry no more than nine seconds ahead. Native descriptors,
held objects and receipts never become reusable caller handles.

Stages are `prepared`, `buffer_applied`, `resource_applied`, `content_persisted`,
`mtime_restored`, `edited_cleared` and `saved_tagged`. Local
`prepared|ready|complete|refused|busy|partial` responses report primitive state and
actual effects, **not** the core's terminal EditOutcome. `complete` requires
independent caller-side validation and observation before product success.
The private addon executes the six effect stages without yielding and holds the
existing bridge collection slot until entered native work returns, including
reentrant cancellation/shutdown. Between removal and insertion, a fresh
association/source/current/saved check protects synchronous newer human text.
Cleanup may retain a partial native history entry; it does not roll back.

Fixture-only `GAK_FIXTURE` builds expose fault injection only in a separately
identified library. The normal artifact has no fault callable. The
[stock native guide](../../../godot-addon/native/README.md) documents build,
exact-source/Save/effect limits and the sole current script-edit runner scenario
`native-primitives`. Required stock GUI/history/durability/privacy/export and
affected observation acceptance passed for T003. No public wire schema or edit
capability is introduced by this private surface.
