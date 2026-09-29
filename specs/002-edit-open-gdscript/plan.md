# Implementation Plan: Safely Edit Open GDScript

**Branch**: `spec/002-edit-open-gdscript` | **Feature identifier**: `002-edit-open-gdscript` | **Date**: 2026-09-27 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/002-edit-open-gdscript/spec.md`

**Status**: [§18 T1](research.md#18-preserved-mtime-behavioral-finalization-research-2026-09-28) selects handler-free same-retained-fd T0 mtime restoration on official stock Godot 4.7.2/macOS arm64; T1 is unchanged/closed. [§19 stock LSP research](research.md#19-one-shot-stock-lsp-validation-research-2026-09-28) now has **L2 narrowed to one FR-020 privacy boundary gap** for the new helper-authority source-read endpoint under normal local-user access boundaries, not an owner-parser correctness or mandatory single-client hold. Owner root/per-URI source attribution is established on the studied source-only profile; stock product validation remains unselected and T003 **unstarted/not implementation-ready** until that one boundary is qualified. T002's completed patched validator remains an oracle, not stock product code. T001/T002 complete, T004–T005 pending; five tasks unchanged. The prior installed analysis and corrected I1 privacy control are historical; final installed analysis of this disposition is complete with one HIGH FR-020 endpoint-privacy hold. No product implementation or A–E acceptance follows.

## Summary

Deliver one native, revision-guarded whole-source edit to an existing open standalone GDScript in one explicitly selected local editor. A caller supplies a checked prior-observation basis and intended source; Rust/core freshly checks independent D/R/B, dirty state and identities, authorizes application, and independently verifies resulting source, clean/saved state and source-attributed parse evidence before success. Refused/unchanged requests add no source/history changes; partial/uncertain changes never imply rollback or retry safety.

The selected conceptual saved-state composition is [§18 T1](research.md#18-preserved-mtime-behavioral-finalization-research-2026-09-28): one descriptor-bound content write/truncate/fsync/readback, distinct same-retained-descriptor restoration of original fstat mtime T0 with `futimens` and independent metadata/content/namespace readback, then guarded public Resource edited=false and CodeEdit saved-version tag, **STOP**. No ScriptEditor saved-handler discovery/invocation, signal broadcast, full Save or private Godot mtime parity belongs to production. §14–§17 and PR #31/#33 retain measured historical conclusions; §17's un-restored direct tag failed. §19's stock LSP remains a **candidate**, not a selected validator: owner root/per-URI result attribution and staged-disk dependency behavior are established on the tested source-only path, while the new helper source-read endpoint still lacks a qualified FR-020 normal-local-user boundary. Both validation and T1 integration remain unimplemented; T003 cannot start until that one L2 gap is resolved.

Shared boundary (planned; no product mutator or production validator exists):

```text
edit-gdscript / typed caller
  -> existing Rust core and supervised worker / authenticated bridge (planned)
  -> validator selection BLOCKED (L2): qualify FR-020 normal-local-user
       access boundary for the new helper-authority source-read endpoint
  -> fresh mutation guards -> addon + standard C-ABI GDExtension on stock editor
  -> exact CodeEdit native edit -> explicit Script R synchronization
  -> retained-fd D content persistence -> same-fd T0 metadata restore/readback
  -> fresh guards -> public edited=false -> fresh guards -> CodeEdit tag -> STOP
  -> if qualified: independent actual-source/closure capture -> distinct fresh
       helper; independently verify D/R/B/dirty/saved/context and validation
```

The previous [§11 patched-engine decision](research.md#11-concrete-native-design-and-resumed-planning), [§13 ordinary Save research](research.md#13-stock-target-save-finalization-research-2026-09-28), and [Phase 1 concurrency boundary](spec.md#phase-1-concurrency-boundary) remain historical evidence/requirements. §13 disqualifies custom ResourceFormatSaver fallback, all-script target Save and direct `ResourceSaver.save`. §14 handler/§16 H3 callback admission are superseded for production by §18's handler-free sequence; §17 remains the negative for tagging without restoring mtime. T002's patched validator remains a semantic oracle/hazard inventory, not a stock binding. The §15 broad `--check-only --script` negative remains valid; §19's isolated LSP design does not repeat that route.

## Technical Context

**Language/Version**: Existing Rust **1.98.1**, edition 2021; GDScript for the thin editor integration; **C++17** for the small standard-ABI extension. Python **3.10+** remains the owned-fixture driver environment.

**Primary Dependencies**: Reuse existing locked `serde =1.0.229`, `serde_json =1.0.151`, `cap-std =4.0.3`, `ring =0.17.14` and existing system toolchain. No new Rust crate, transport library, godot-cpp layer, parser, process service or public transport adapter is selected. The candidate one-shot helper uses stock Godot LSP over loopback TCP with Content-Length JSON-RPC inside the existing supervised worker, **only if** the L2 FR-020 endpoint access boundary qualifies. No new module/API or OS profile/proxy is selected. Generate/use the pinned engine's public GDExtension C interface and opaque object/Variant dispatch for T1, preserving upstream SDK notices. Native OS I/O uses the demonstrated macOS descriptor APIs.

**Engine and version scope:** The historical PR #24 candidate selected patched Godot **4.7.2**, base commit `ed1daf0bf001b61586d9930840f2f1394092c079`; its A patch is superseded and no A binary exists. T002's patched validator acceptance remains complete, not stock mutation support. §18 exercised official stock Godot **4.7.2** (binary SHA-256 `c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf`) on maintainer-operated **macOS 26.6.2 arm64** GUI with a standard public-ABI native probe. Exactly **16 accepted case observations** comprise 15 final-campaign cases plus a separately settled current-build history case, not a 16/16 campaign or product A–E acceptance. §19's source/closure study exercised **65 fresh official-stock LSP invocations** (44 initial, 10 followup, 11 final closure including one root-only negative control); separate privacy-only invocation 66, two-client invocation 67 and inverse dependency invocations 68/69 do not enlarge the final closure's 10 cases/21 URI-source fences. Invocation 67 observed two distinct same-URI version-1/version-37 owner/peer parser results and source-derived reads of the staged root and owned outside-helper `.gd`; invocations 68/69 observed owner validity following staged-disk helper type in **both directions**, not the other client's version-37 overlay. No false owner verdict or cache poisoning was observed. These are bounded same-UID research observations, not a cross-UID runtime result, product validator or mutation proof. Other versions/platforms and product runtime guarantees remain unresolved.

**Storage (candidate only, pending L2):** Existing owner-private source-free session registry; one existing selected-project script; request-local memory/descriptors/evidence and an owned, per-invocation source-only scratch project containing staged copies of a bounded closure and disposable Godot bookkeeping. The privacy recipe omits `--log-file`, discards stdout/stderr and requests no diagnostic file; scratch project and isolated helper HOME/XDG data would be deleted after invocation. This is transient disk source storage, not a persistent source cache or project-snapshot service. **A private root and listener-PID verification do not establish a normal-local-user access boundary for the new LSP source-read endpoint**; neither does an ordinary second trusted client invalidate owner results. No new persistent mutation token store, replay cache, backup/journal or recovery service. A descriptor-bound write is not crash-atomic storage.

**Testing**: Before selecting a product validator, qualify the new helper-authority source-read endpoint against the existing FR-020 normal-local-user access boundary; no single-client mandate, privileged OS mechanism or retry of the declined experiment is prescribed. If qualified, implement pure core boundary/transition tests; real supervised-worker/private-bridge tests; native primitive and exact-source capture/admission/fence/lifecycle regressions; actual visible-Godot A–E plus Save/reopen/reparse/rescan/runtime and privacy/export/observation regression. Reuse existing owned-editor harness facilities, independent witnesses and deadlines. The [quickstart](quickstart.md) is a future execution/evidence contract, not a passing-results record.

**Project Type**: Existing local Rust library/CLI plus editor addon/native integration. No MCP exposure.

**Performance Goals**: Controlled edit result ≤10 seconds (9.5-second acquisition/operation cutoff + 0.5 delivery reserve), including an unresponsive editor. Observation remains ≤5 seconds with its existing 4.5-second supervision. No sustained-throughput or arbitrary-project-size claim.

**Constraints**: One open standalone script; explicit target/session and expected basis; exact independently agreeing D/R/B and attributable clean state; native single-step history; target-bound non-redirectable persistence; no arbitrary evaluation/execution/outside-project access; truthful partial outcomes; no automatic retry/rollback. Success is an observed interval, not a frozen editor or arbitrary same-inode serialization.

**Scale/Scope**: Root/replacement and each source ≤512 KiB UTF-8; bounded 32 dependency sources / 4 MiB dependency bytes; one active editor collection/edit slot; inherited 32-peer/descriptor bounds. Full-source LF representation, explicit unsupported NUL/CR/BOM refusal, and native Save-profile eligibility preserve exact source rather than normalize it. Caller/bridge/diagnostic bounds are in the contracts.

## Selected Stock Saved-State Composition (Design Only)

**T1 is the selected conceptual production sequence, not implemented T003 or
full acceptance.** Prepare independently clean D/R/B and the same bound open
standalone Script, ScriptEditorBase and CodeEdit, project/session/path, native
Save-profile and exact source/current-and-saved versions. Retain a project-bound
exact-file descriptor and record its `fstat` mtime **T0** before mutation; the
prepared current/saved versions are the pre-edit baseline. Recheck immediately
before one native CodeEdit complex edit; freeze the **post-edit current** and
**preparation saved** versions for every subsequent effect guard. Read B back,
explicitly set the held Script R using `Script.set_source_code`, then read R
back. Never select the target by focus, reload or broadcast `resource_saved`.

Guard the exact identity/source/versions and current no-follow namespace
attachment before one logical same-retained-fd content persistence:
`pwrite` (multiple calls only if eligible short writes require completion),
`ftruncate`, `fsync`, `pread`. The receipt records actual content write
calls/bytes, truncate/flush and descriptor content readback, fstat identity
and namespace attachment with actual errors/unavailable facts; it is not an
independent Rust D witness. After fresh guards, restore **only mtime T0** on
that **same retained fd** using macOS `futimens` (`atime=UTIME_OMIT`), not a
path reopen, replacement write or new Save. Recheck `fstat` mtime, identity,
descriptor content and current no-follow path attachment; record the
metadata syscall and its error/readback separately from content persistence.
`futimens` can change ctime, needs owner/superuser for explicit timestamp
under the demonstrated macOS API and may fail; no inode/ctime/size/private
Godot Resource or document mtime numerical parity is a requirement. These
guards do not claim arbitrary same-inode exclusion or crash atomicity.

Only complete same-attempt content **and** restored-metadata receipts plus
fresh same-document/Script/editor/buffer/session/path/source/namespace/
post-edit-current/preparation-saved checks permit finalization. Call
`EditorInterface.set_object_edited(held_script, false)`, repeat every guard,
then call the public `CodeEdit.tag_saved_version()` and **STOP**: no handler
discovery, Callable invocation, listener emission, full Save, reload or
follow-on handler callback tail. A same-text newer version fails the version
guard; a newer human B or detached/replaced leaf or parent namespace cannot
be restored/tagged, even when the displaced descriptor still holds intended
bytes. No rollback, compensating write or retargeting. Once B, R or D
application is known (`Applied` or `PartlyApplied`), failed or unavailable
metadata syscall, attachment or readback yields `AppliedUnverified`; only
those later observations can be unknown, not overall application certainty.
Edited-flag-only bookkeeping is partial. Preserve the exact stage and
observed errors, not a false refusal/not-applied or success. Later human
typing after a successful tag remains dirty and is never overwritten.

The [pinned stock external-change check](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp#L809-L854)
compares the document's remembered last-modified **seconds** with the path
mtime; on macOS the [file getter](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/drivers/unix/file_access_unix.cpp#L375-L394)
returns `st_mtime`. Direct CodeEdit tagging does not update the private
document field. §17's un-restored route crossed timestamp seconds and a later
ordinary Save falsely prompted for external changes. In [§18's observed
stock GUI sequence](research.md#18-preserved-mtime-behavioral-finalization-research-2026-09-28),
same-fd restoration to T0 before the public tag was followed by ordinary
human Save (no false dialog), Undo → Save → Redo → Save and earlier history,
reopen, scan and fresh runtime behavior, with bounded dirty-unrelated and
race/error evidence. This is behavioral evidence, not proof of private field
equality, generic timestamp portability, arbitrary editor continuation safety,
full Feature A–E or exact-source validation. Immediate success still requires
independent actual D/R/B, dirty/current/saved, identity/namespace/context and
eventual explicit validation; the receipt/tag is not success authority.

The completed T001 `SavedStateEvidence` still includes numeric
`resource_mtime`/`document_mtime` equality and finalization flags from the
historical patched design. **T004 owns migration** of that typed evidence,
reducer and affected contracts before consuming stock T1 facts; neither
fabricate private numerical metadata nor select a new API/wire schema here.
T1 replaces §14 P1's incoming Callable topology, callback admission,
function-discovery/live-reload tail and deferred-debugger routing obligations
for production. Generic forbidden-effect safeguards and exact-source validation
implementation remain; §19's stock candidate is held at L2, not selected production proof.

## Stock LSP Exact-Source Candidate — L2 FR-020 Endpoint Privacy Hold (Design Only)

The [§19 source-only candidate](research.md#19-one-shot-stock-lsp-validation-research-2026-09-28) would capture exact proposed source and recursively supported literal `.gd` inheritance/preload dependencies, resolving relative paths against each referring source **before launching** a matching official Godot binary. It would stage this immutable closure in a fresh mode-0700 private source-only project with isolated HOME/XDG config/data/cache, a minimal `project.godot`, and relevant effective selected-project GDScript warning enable/per-warning levels/directory rules (including overrides). `initialize` analyzes **all discoverable staged `.gd` sources** before its response or `didOpen`; a root-only capture cannot suffice. Recheck identity/hashes and context at mutation and verification boundaries if a qualifying helper is eventually selected; do not require physical absolute-path or live-editor cache parity. A bounded observed context is not an atomic arbitrary-writer snapshot. The clone confines **our staged source**, not the new helper endpoint's unmanaged source reads.

A small conservative prelaunch lexical admission and no-follow confined reader, **not** a second GDScript parser or generic effect system, recognizes literal `.gd` inheritance/preload and recursively bounded transitive paths. Permit native bases/ClassDB and same-build inert source-only `@tool`/static/init/new declarations. Return `unavailable` for unsupported/unresolved/computed/escaping dependencies, non-`.gd` resources, UID/remap/binary/embedded dependencies, out-of-project or document-link-probed literal paths, or global-class/autoload/GDExtension/custom-loader/object-value context the clone cannot reproduce. Restrict **all** link-probed string literals (`FileAccess::file_exists`), not just dependency expressions, before startup. No selected addons, plugins, scenes, resources, old UID/.godot/cache state or startup scripts enter the clone. Clone-only Godot `.gd.uid`/`.godot` bookkeeping writes are permitted; selected target D/R/B and project code are not. A dependency warning promoted to ERROR changes validity: include effective warning/directory policy in the bounded context witness, or refuse. Do not reject `@tool` or static syntax merely by spelling; §19's isolated pure `.gd` scans showed no project-code sentinel, whereas non-`.gd` scripted-resource loading and rejected addon loader hooks had effects before/around initialize.

The researched invocation used the pinned official binary `--headless --editor --path <owned-project> --lsp-port <unique-loopback-port>` **without `--log-file`**, with stdout/stderr discarded. Earlier §19 evidence used `--log-file` only for synthetic provenance; [pinned stock logger setup](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/main/main.cpp#L2312-L2339) shows that explicit flag forces editor file logging. A future privacy recipe must not retain source-bearing startup text or treat cleanup as privacy. Research verified owner-child listener attribution and `initialize`/`initialized` followed by exact-UTF-8 `didOpen` and unique `documentSymbol` fences on each admitted root/dependency URI. Exact-URI diagnostics plus a shaped symbol distinguish a completed owner parse; empty diagnostics alone do not. Root-only fencing was empirically false-valid for an invalid unused preload, so every existing admitted source would need fencing, including body errors and warning-as-error policy. With exact owner root and per-URI source/context attribution, complete fences with severity-1 diagnostics support owner `invalid`; complete fences without severity-1 or missing witnesses support owner `valid`. Missing, mismatched, unsafe or timed-out evidence remains `unavailable`; **a separate connection is not itself unavailability**. Reap the owned child with bounded terminate/kill; stock `shutdown` responds `-32601`, and `exit` does not terminate it.

Invocation 67 exercised two separate same-UID synthetic client connections in one owned process. After the owner initialized but **before its `didOpen`/fence**, the second client read source-derived symbols from the staged root and a separate owned sibling `.gd` **outside** the helper project, without supplying source text. It then opened the same root URI at version 37 with different inert typed-invalid text. The owner subsequently opened its captured version-1 source; its raw parser outcome was valid, while the second client's was invalid on its own connection. No managed-text overwrite, mixed diagnostic, false owner-valid verdict or cache-poisoning result was observed. In [inverse dependency invocations 68/69](file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-one-shot-lsp-39y7q6v3/evidence/peer-dependency-verified.json), the second client's version-37 overlay for the same helper URI opposed disk: disk helper `int`/peer overlay `String` yielded owner `valid`, then disk helper `String`/peer overlay `int` yielded owner `invalid` with the expected root severity-1 return-type error. Owner helper/root version-1 per-URI diagnostic-plus-symbol fences and peer helper version-37 fences retained their distinct source. Source review attributes dependency-parser production to staged disk, separately from peer-managed parser text; the tested owner outcome followed disk in **both** directions, not the peer overlay. Stock LSP accepts multiple peers (up to eight) and maintains per-peer managed files/parse results. A second trusted client or independent document parse is not a semantic failure. Separately, [count-only peer admission](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/language_server/gdscript_language_protocol.cpp#L142-L150), [outside-project absolute URI retention](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/language_server/gdscript_workspace.cpp#L502-L578), [unmanaged `.gd` reads under helper authority](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/language_server/gdscript_language_protocol.cpp#L407-L440) and [source-derived `documentSymbol` values](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/language_server/gdscript_text_document.cpp#L145-L156) expose a **new helper-authority source-read endpoint** without caller-authentication/user-identity check in the reviewed path. The selected launch recipe verifies listener PID and uses private staging; it does not establish an FR-020 normal-local-user access boundary for that endpoint. Keep the new endpoint within the existing local/project-confined privacy boundary; this does not demand single-client exclusivity, global Godot sandboxing or protection from hostile same-UID software. No cross-UID reachability/disclosure was run or observed. No qualifying public stock boundary was selected/proved, and no L3 impossibility follows. No additional experiment, privileged recipe, proxy or OS profile is prescribed.

The **candidate minimal result**, if L2 is resolved, binds existing request/session/target/purpose, exact logical `res://` root/source paths and hashes, actionable root/dependency diagnostics, closure/context witness, completion/incomplete reason and **supervisor-clock** interval. This is not a selected wire schema, engine-computed SHA/editor-clock parity/parser-stage/`diagnostics_complete` contract or T002's historical layout. Preserve redaction and bounded diagnostics; never leak denied physical paths, source or incidental clone logs. Conditional changed-intent design uses fresh proposed-source validation before mutation and a **separate fresh** helper on independently captured actual post-change source/dependencies/context after T1 STOP; unchanged intent uses one helper and no mutation. If post-change validation is unavailable after known application, report `AppliedUnverified`; no retry, pool or rollback. The 65-invocation study and separate research invocations 66–69 are not A–E, maximum-size performance or implementation proof. First 44 normal lifecycle 1.845480–2.347037s (median 1.9041955), final 10 1.993938–2.582493s (median 2.1155485); one focused followup took 4.654501s. Two ordinary small-closure invocations plausibly fit 10 seconds, not an arbitrary-project guarantee.

**T003 is unstarted and not implementation-ready for this one FR-020 endpoint boundary gap**, not for unproved owner parser attribution or mandatory dependency cache-generation parity. Only after resolving L2 could the existing supervised Rust worker host a focused one-shot helper alongside the T1 native mutator; `mcp-server/src/gdscript_validation.rs` is **not a mandated new module**. T004 owns coherent typed caller/bridge/core evidence integration and migration from historical T001 fields when a validator is selected. No new Rust crate, service, global scheduler, parser port, engine patch, sixth task, schema or changed T1 sequence is selected. Ordinary editor effect safety, source/effect confinement for our work and all implementation/acceptance gates remain pending.

## Implementation Boundaries and Historical Patched Primitive A

### 1. Core intent, revision and compatibility

Add a focused script-edit module to the existing Rust library, using existing checked selectors, identity/collection/source evidence and classification patterns. The core consumes protocol-independent evidence; no JSON, TCP, Godot object or native ABI dependency enters its policy. Do not build a general Phase 2 transaction/revision framework.

One new `edit-gdscript` caller accepts the selectors and bounded stdin JSON in [edit-api.md](contracts/edit-api.md). Whole-source intent avoids patch-coordinate ambiguity; source does not enter argv or an arbitrary file-reading option. Expected basis extracts project/session/document/file identity, agreed source digest/length, exact current CodeEdit version and attributable clean state from prior observation. Observation v1 has no saved-version field; acquire it during fresh preparation. Pre-edit guards compare the prepared current version; after the complex edit freeze its new current version while keeping the preparation saved version for the stock transition, as in the [current selected design](#selected-stock-saved-state-composition-design-only). Fresh checks, not that snapshot, authorize the write.

Keep observation's public v1 meaning, timing and per-surface availability unchanged. Private bridge **v2** is a coordinated clean cutover with authenticated edit/native capability fields, changed HMAC domain and strict tuples. Migrate all Rust/addon/worker/fixture peers and docs; no dual-protocol shim or v1 mutation fallback. Restart/re-enable the addon and observe again after upgrade.

### 2. Historical native application and Primitive A (not executable)

Use one native CodeEdit complex operation on the exact bound buffer; preserve previous undo history and unrelated tabs, with normal native redo-branch invalidation permitted. Do not use `set_text`/clear-history as an edit shortcut or create an alternate EditorUndoRedoManager history for TextEdit's existing text history. Never choose the destination by focus.

Immediately recheck current B/version and original R, then use the explicit supported **`Script.set_source_code` method**, not property `_set`, `apply_code`, export propagation or reload. Native source synchronization is not a compiled-class/Inspector/live-game update claim. Pending dragged exports or an unsafe Save representation refuse before application; do not alter editor preferences or scene properties.

The C++ integration pins the selected project and existing file, independently confirms its identity against the Rust basis, and writes/truncates/flushes only that retained object after fresh D/R/B/identity guards. Short-write/error/namespace-loss facts remain explicit. Replacement/parent redirection never changes the write destination. Arbitrary same-inode exclusion remains unclaimed; known changed source/identity still stops obsolete work.

The old A proposal prescribed setting private Resource/document mtimes and tagging saved last; §14's subsequent P1 handler candidate performed a broader native saved callback with unsafe tails under §16 H3. **Both are superseded for production:** the [selected §18 sequence](#selected-stock-saved-state-composition-design-only) restores the retained fd's T0 before public edited=false and direct CodeEdit tagging, without private field manipulation or a handler.

Do not emit `resource_saved`, call Save All/current-tab Save, clear unrelated docs/history or claim normal scene-save effects. The [native Primitive A §3 guarded target-document saved transition](contracts/native-integration.md#3-primitive-a-guarded-target-document-saved-transition) now specifies T1; historical P1 is retained only in research. All behavioral and effect postconditions remain.

### 3. Primitive B and automatic editor effects

The completed T002 patched engine exposed `GDScriptLanguage::validate` → parser → analyzer on immutable exact source/path and bound actual input hash, target/session/request, invocation interval and root/dependency diagnostics. It returned completed valid, invalid or unavailable, not reload/log/silence proxies. This remains a **historical** semantic oracle/reference and hazard inventory, not a production patched dependency or selected stock editor binding.

Its call-scoped read/effect context permitted parser/dependency-cache activity and confined source-backed analysis with attributed dependency bytes and metadata while refusing excluded loader, initialization and object-property paths before dispatch. On official stock, `reload(false)` still compiles and can initialize, and broad `--headless --check-only --script` executed tool/non-tool static initializers while reporting exit 0 for invalid cases. §19 researched a bounded source-only LSP candidate with prelaunch capture/admission and exact per-source parser/diagnostic fences; its ordinary `initialize` analyzes staged source, so ad hoc root-only diagnostics do not suffice. Owner result attribution is established on the tested root/dependency source-only profile, independently of another client's managed text; the one unresolved L2 gap concerns FR-020 access to the new helper-authority source-read endpoint.

The historical engine's target-local guard does not exist on stock. §16 saved-handler function discovery/live-reload/deferred-debugger hazards explain why that handler is not selected, not a production blocker for T1. Ordinary editor validation/export may still synchronize R or produce effects: confine/prevent forbidden effects and independently check actual source/context. Diagnostic source belongs in the requested bounded result, not incidental logs.

**If** a stock helper qualifies after L2, changed intent would run fresh captured/admitted proposed-source preflight before mutation and a **different fresh** helper on independently read actual source for `post_change` after T1 STOP; invalid original scripts remain repairable if admission and resulting-source validation qualify. Unchanged intent would run one non-mutating helper. Neither helper is selected/implemented now. The [Primitive B contract](contracts/native-integration.md#4-primitive-b-exact-source-native-gdscript-validation) records this conditional candidate and T002's separate historical API, not an instruction to port its editor binding.

### 4. Stage ordering, deadlines and independent verification

The former [ordered state machine](data-model.md#3-ordered-attempt-state-machine) retains stage/certainty semantics, but its patched finalization mechanism is superseded. **Conditional T1 + stock-validator flow only after resolving L2** (no implementation authorization):

```text
accept/select
  -> ONLY AFTER validator qualification: fresh confined proposed-source/closure
       capture and qualified FR-020 helper source-read access boundary;
       fresh helper requires complete owner-source preflight valid; recheck context
  -> supervisor records may_apply, then authorizes its existing worker
  -> exact clean bound target/Save-profile/version/namespace guards; retain fd + fstat T0
  -> one native CodeEdit B complex edit / explicit held Script R synchronization
  -> fresh guards; same-retained-fd pwrite/ftruncate/fsync/pread D content receipt
  -> fresh receipt/source/version/namespace guards
  -> same-fd futimens(T0, atime UTIME_OMIT); fstat/content/namespace readback
  -> fresh complete content+metadata receipt and bound target/version guards
  -> public Resource edited=false; fresh guards; CodeEdit tag_saved_version; STOP
  -> if qualified, independently read actual source/closure/context;
       separate fresh helper for post_change
  -> independent D/R/B, dirty/saved and context verification
  -> one Rust-classified terminal outcome
```

The first native call that can change source/history is the local potentially-applied boundary. The supervisor becomes conservative earlier, **before** releasing its worker's one-shot authorization, so a lost worker/acknowledgment cannot become a false not-applied result. Before that handoff, preparation cannot apply; afterward only an attributable irrevocable pre-boundary discard can prove refusal. A completed B/R/D change remains known application despite later loss. Reuse the existing supervised worker rather than create a helper service.

Reuse the existing one-active-collection/edit slot for the selected editor/session: claim it during preparation, require its ownership at native mutation entry, and retain it through verification/terminal cleanup. Another edit receives a source-free terminal busy refusal before the potentially-applied boundary, with no source/history/finalization change and no queued or replayable work. Releasing the slot cannot auto-apply rejected intent. A later request needs a freshly observed valid basis; old-basis text compatibility cannot override a changed revision. Human typing remains unblocked. Caller termination does not release this slot while entered native work can still mutate; its uncertainty/deadline semantics remain unchanged. T004 owns the real same-session overlap case under US2.4.

An editor-local remaining-time expiry stops new stages when control returns. Native parser/I/O work can block; the caller's supervised deadline still returns a truthful bounded outcome. Timeout/abort cannot revoke an entered stage or imply rollback; late replies cannot upgrade the result. No retry, reconnect/resume or repair write.

Neither the content/metadata receipt nor the public tag is independent evidence. Rust reads/rechecks D through its own project capability; the existing collector freshly reads actual R and B and document-attributed dirty state, plus separate native saved-state facts. Recheck source/identity/version, namespace and validation dependency/context/save-profile witnesses. Missing, changed, known-stale or divergent evidence prevents success even if an earlier/later sample agrees. Report explicit changed/unchanged/refused/applied-unverified/application-unknown and actionable content-write versus metadata-restore failures; no false not-applied result after a known write.

### 5. Historical implementation and evidence boundaries (not T003 authorization)

The PR #24 patched A candidate and §14 P1 post-persistence handler were earlier saved-state proposals; [§16 H3](research.md#16-stock-saved-handler-admission-research-2026-09-28) rejected safe admission of tested P1, while [§17 M3](research.md#17-minimal-behavioral-finalization-research-2026-09-28) measured a later false Save/history failure for **un-restored** direct tagging. [§18 T1](research.md#18-preserved-mtime-behavioral-finalization-research-2026-09-28) supersedes both saved-state proposals. T002's patched validation API remains historical, while [§19 research](research.md#19-one-shot-stock-lsp-validation-research-2026-09-28) leaves stock validation at L2, with no current helper selection. T003 has not begun; prior reviews are historical, not product acceptance.

Historical native lifecycle/export evidence remains relevant to the eventual standard extension, but no patched saved-state hook, private Resource mtime getter/setter or finalizer API is a current implementation instruction.

## Constitution Check

**Historical planning check:** Earlier bounded candidate reviews passed at their dates, not as implementation/acceptance. **Current disposition:** §13 Save routes, §14 P1 handler, §16 H3 admission and §17 un-restored direct tag remain [historical evidence](research.md#17-minimal-behavioral-finalization-research-2026-09-28). [§18 T1](research.md#18-preserved-mtime-behavioral-finalization-research-2026-09-28) selects handler-free same-retained-fd T0 restoration. [§19 stock LSP research](research.md#19-one-shot-stock-lsp-validation-research-2026-09-28) finds L2, **one HIGH intentional FR-020 readiness blocker**: the new helper-authority source-read endpoint lacks a qualified normal-local-user access boundary. Tested owner root/per-URI results and staged-disk dependency behavior remain attributable despite the other client. Earlier combined-design installed analysis and corrected I1 privacy recipe are historical, not a current readiness pass; installed analysis for this final disposition is complete with one HIGH FR-020 endpoint-privacy hold. No waiver, relaxed safety guarantee, T003 implementation or A–E pass follows. This matrix records continuing obligations, not their discharge.

| Governing obligation | Design and future evidence |
|---|---|
| I / IV — independent D/R/B and observed postconditions | Separate actual D and editor R/B/dirty/current/saved reads; attributed post-change B; immutable partial evidence. Neither content/metadata receipt nor clean tag certifies success. Final intended, clean D/R/B, ordinary Save/reopen and native history behavior plus A–E/durability remain mandatory. Intermediate divergence during a guarded attempt is permitted, not an external atomic-every-step promise. |
| II — preserve human work and stale intent | Exact target/Script/editor/CodeEdit/session/namespace, prepared baseline current and saved versions, frozen post-complex-edit current and preparation saved versions for persistence/restoration/finalization guards, fresh source and separate receipt checks; never restore/tag newer work, including same-text newer version. Namespace loss forbids metadata restoration/tag, with no path reopen/retarget. No arbitrary-writer serialization promise. |
| III — native editor semantics | One CodeEdit complex edit, explicit `Script.set_source_code`, one retained-fd logical content-persistence operation (`pwrite` calls as needed for eligible short writes, then truncate/fsync/pread) followed by same-fd T0 mtime restoration/readback and guarded public edited=false/CodeEdit tag/STOP. §17's no-restore tag failed later Save/history; §18's observed stock GUI behavior supports this narrower remedy without handler callback effects. No private numeric mtime/cache parity, Save replay, engine A patch, signal broadcast or ResourceSaver is required. |
| V / X — confinement and truthful outcomes | Existing authentication/private metadata and project-bound no-follow descriptor/source reads remain. Candidate prelaunch capture/admission, a fresh source-only clone, effective-warning witnesses and owner root/per-URI diagnostics-and-symbol fences establish scoped owner parse attribution, with dependency outcomes tracking staged disk on both inverse probes. Complete attributable owner source/context/fences with severity-1 diagnostics support `invalid`, without severity-1 or missing witnesses support `valid`; silence, incomplete/mismatched evidence, unsafe admission or deadline yields `unavailable`. An additional client alone does **not** make owner validation unavailable. Separately, [FR-020](spec.md) and constitutional V require the new helper-authority source-read endpoint to remain within the existing normal-local-user access boundary; private staging/listener PID/short life do not qualify it. No cross-UID disclosure was exercised. Ordinary-editor no-forbidden-effects/pre-effect checks and truthful partial outcomes remain gates; do not import §16 handler machinery. Separate content-write, metadata-restore/error/unavailable, namespace/version, parse/dependency/permission/timeout facts; a known write with failed restore/readback remains `AppliedUnverified`, never application unknown/refusal/not-applied. |
| VI / XII — layered verification and explicit support | Pure transition/boundary tests plus real visible-editor A–E, history, durability, timing, privacy/export and complete observation regression on exact recorded builds. No stock/other-version mutation support extrapolation. |
| VII / IX — protocol-independent, small composable surface | One typed/core edit and one CLI, existing private bridge, no MCP; core owns policy, native owns engine-local primitives. No duplicate transaction model. |
| VIII — tooling/gameplay isolation | Editor-only native integration; real exports and runtime checked with enabled/disabled addon and native artifacts, not reliance on `addons/` or `@tool`. |
| XI — independent implementation and dependency review | Public engine source/ABI, own narrow integration, existing locked Rust dependencies. Preserve SDK notices, review exact new native build inputs/advisories/licenses before their implementation delivery. No unreviewed copied framework. |
| XIII — proportional complexity | **Concrete requirement/failure:** independently prove target-bound clean/intended D/R/B, later ordinary Save without a false external-change prompt, native Undo → Save → Redo → Save and earlier history, reopen, human-work/namespace safety and truthful partial outcomes. §17's **simplest earlier** public edited=false + direct CodeEdit tag, after a descriptor write with a newer mtime, looked clean but a later human Save falsely prompted and failed to persist B29. **Simplest credible remedy selected by §18:** preserve T0 from `fstat` and, after one logical content-persistence operation (`pwrite` calls as needed for eligible short writes, then truncate/fsync/pread), restore T0 by `futimens` on the same retained fd, verify fstat/content/namespace and use public edited=false/CodeEdit tag. §18 observed one 51-byte `pwrite` in its positive stock run; this is evidence, not an exactly-one-syscall requirement. Observed later Save/history passes; no private field parity is justified. **Cost:** one metadata syscall, ownership/permission/error/descriptor-lifetime and platform-specific mtime/ctime/readback/race maintenance and GUI regression, limited to demonstrated macOS arm64; no portability abstraction. This is less than §14 P1's Callable topology, admission, callback generations/reverse consumers, function-discovery/live-reload/deferred routing and version maintenance. Generic effect safety and the separate validator remain; no new service, watcher, cache manager, scheduler or patched fallback. |
| Architecture/compatibility/workflow | Existing Rust core, supervised worker and eventual private bridge/caller cutover retain responsibility. §18 T1 alone is selected; §19's stock LSP is a candidate held at **L2** for the one FR-020 helper endpoint normal-local-user access gap, not a selected validator, fixed module, API or wire shape. T003 remains unstarted/not implementation-ready pending that one boundary; final installed analysis is complete with one HIGH FR-020 endpoint-privacy hold. Existing worker mechanisms may host a qualifying helper; T004 would own typed integration/migration of historical numeric `SavedStateEvidence` after selection. T004/T005 dependencies and five-task structure remain unchanged; all implementation and acceptance gates remain. |

**Principle XIII — L2 proportionality:** The exact-source requirement is not met by a live editor LSP (mutable editor source/cache), root-only diagnostics (observed false-valid for an invalid unused preload), broad check-only (static code ran and invalid cases exited 0), or ordinary validation/log silence. The clone and completed owner root/per-URI fences **do** provide scoped parser-result attribution on the tested source-only path; invocations 67–69 retained separate owner/peer version-1/version-37 text and results, with both inverse dependency outcomes tracking staged disk rather than peer overlay. They do **not** confine the **new source-read endpoint**: invocation 67 observed a same-UID second client reading a synthetic owned `.gd` outside the helper project before its own source submission. Pinned count-only admission, absolute URI mapping, unmanaged `FileAccess` reads and source-derived symbols establish that new helper-authority capability without a caller-authentication/user-identity check in the reviewed path. The existing FR-020/constitutional V normal-local-user privacy boundary, not process purity, calls for qualifying access to that endpoint. Private clone, listener PID, isolated HOME/XDG, no-log privacy correction from invocation 66 and short life do not do so. Cross-UID reachability/disclosure was **not** tested; this is a source-supported boundary gap, not observed cross-user exploitation or owner-result corruption. No hostile same-UID resistance, single-client rule, global Godot sandbox, OS privilege/profile, proxy, service or follow-up experiment is introduced. No L3 impossibility finding. Weigh any eventual qualifying boundary against this one requirement rather than introduce infrastructure by default. T1's independent same-fd T0 complexity record is unchanged.

All A–E are applicable, including C because native Undo/Redo is claimed. At least one permitted runtime fixture must prove changed behavior. No missing runtime/buffer/parse observation is an inapplicability justification. Feature and task completion depend on acceptance evidence, separately from GitHub review/merge metadata. Feature completion would not automatically complete Roadmap Phase 1.

## Project Structure

### Documentation (this feature)

```text
specs/002-edit-open-gdscript/
+-- spec.md                         existing behavioral specification; unchanged
+-- checklists/requirements.md      existing specification-quality review
+-- research.md                     retained evidence + current design decisions
+-- plan.md                         this implementation plan
+-- data-model.md                   identity, evidence, stages and outcomes
+-- contracts/
|   +-- edit-api.md                 typed/caller input, output, compatibility
|   +-- bridge-protocol.md          private v2 and supervised authorization
|   +-- native-integration.md       finalizer/validator/inspection/guard contracts
+-- quickstart.md                   future execution and evidence guide
+-- tasks.md                        subsequent task derivation and granularity review
```

The [task list](tasks.md) retains five PR-sized increments, completed granularity review and unchanged dependencies; T001–T002 are complete. The [§13 rerun](#stock-research-artifact-review), [post-persistence review](#post-persistence-design-review), [stock validation/effect design review](#stock-validation-and-effect-design-review), [saved-handler admission design review](#saved-handler-admission-design-review), [minimal behavioral finalization design review](#minimal-behavioral-finalization-design-review), [§18 review](#preserved-mtime-behavioral-finalization-design-review) and [earlier combined §19 review](#one-shot-stock-lsp-validation-design-review) are historical. Their coverage is not acceptance and their tentative L1 readiness was superseded by subsequent retained evidence; final installed analysis of the narrowed L2 is complete with one HIGH FR-020 endpoint-privacy hold.

### Source Code (repository root)

Existing files are retained; `planned` entries below are implementation ownership, not files created by this planning PR:

```text
mcp-server/
+-- Cargo.toml / Cargo.lock         existing single package, locked dependencies
+-- src/
|   +-- lib.rs                     exports reusable core
|   +-- observation.rs             existing read-only semantics
|   +-- script_edit.rs             public edit-core facade
|   +-- script_edit/               private request/evidence/validation/outcome/attempt modules
|   +-- runner.rs                  existing supervised execution boundary; candidate validator could reside here
|   +-- project_fs.rs              existing independent D/confinement helpers
|   +-- bridge.rs / bridge/        existing session/authentication/framing boundary
|   +-- bridge/wire.rs             existing caller/worker/bridge JSON codecs
|   +-- bin/edit-gdscript.rs        planned thin caller entrypoint
+-- tests/                         required core/boundary behavioral regressions

godot-addon/
+-- addons/godot_agent_kit/
|   +-- plugin.gd / bridge.gd       existing lifecycle/routing; v2 cutover
|   +-- observation.gd             existing getter-only observations
|   +-- script_edit.gd             planned thin native-attempt integration
|   +-- export_guard.gd            extend native tooling exclusion
|   +-- native/                    planned installed editor-only extension/artifact
+-- native/                        existing T002 patched-validator source/build; planned stock writer/integration, no A patch
+-- tests/
    +-- run_observation.py         retain all existing behavior acceptance
    +-- run_script_edit.py         planned focused mutation acceptance runner
    +-- fixtures/                  reuse owned-editor setup/witness facilities
```

**Structure Decision:** No new Rust package, top-level product component or mandated `gdscript_validation.rs` module. Standard native mutator/integration belongs to `godot-addon/` after T003 qualification; the **existing supervised worker** could host a future stock helper if L2 is resolved, while T004 would integrate typed caller/bridge/core evidence after validator selection. T002's engine patch remains a historical patched-validation oracle, not a production patch. Reuse existing `Harness`, owned fixture setup, window capture, auth and independent witness facilities if implementing; extract only helpers actually needed by both runners, not a generic plugin/test framework. Existing shared artifacts must not acquire feature/task IDs in public APIs, transport, logs or ownership.

**Ownership review:** Editor source/edit primitives remain with addon/native integration; confined capture, any qualifying validator supervision, owner source fences and qualification of the new helper endpoint's FR-020 normal-local-user access boundary could belong to the existing Rust worker, **conditional on L2 resolution**. Bridge, supervision, confinement and fixture lifecycle keep responsibility-based names. No extra infrastructure or generalized script-edit domain is selected.

## Complexity Tracking

No constitutional violation/waiver is requested. §17's direct no-handler tag failed later ordinary Save/history after content persistence changed mtime. §18 demonstrates narrower same-fd T0 restoration before tag on stock macOS arm64 GUI; its permission/error, fd lifetime, namespace, version and regression costs remain. §19's stock LSP candidate demonstrated need for all-source fences; invocations 67–69 established scoped owner parser/source/dependency attribution under the tested two-client source-only profile, while invocation 67 plus pinned source exposed a separate new helper-authority source-read capability outside the clone. **L2's one FR-020 normal-local-user endpoint boundary remains unqualified.** No parser port, framework, privileged OS prerequisite, patched fallback or other-platform abstraction is selected.

| Present mechanism/gap | Simplest alternative and insufficiency | Ongoing cost and present justification |
|---|---|---|
| Standard-ABI native library / bound persistence | Public Save paths can follow replacement paths; external writer adds a separate synchronization boundary. | C++ build, ABI/OS/lifetime tests; justified by demonstrated in-editor object/descriptor behavior. No godot-cpp/crate framework. |
| Selected T1 same-retained-fd mtime restore after bounded D content persistence; §14 P1 handler and patched A historical | §17 direct edited=false + CodeEdit tag produced immediate clean D/R/B but a later human Save falsely prompted external-change reconciliation and did not persist human B29; Undo/Save/Redo/Save also failed. Broad Save has fallback/unrelated effects and P1 carries unadmitted callback tails. | Capture T0 by `fstat` before mutation; after the **single logical** content operation (`pwrite` calls as needed for eligible short writes, then truncate/fsync/pread), `futimens` only the same fd's mtime (atime omitted), fstat/read back content and namespace, then guarded edited=false + direct CodeEdit tag/STOP. One metadata syscall, permission/ownership, fd/error/ctime/platform behavior and independent GUI regression remain costs. |
| Stock LSP candidate at L2; T002 patched oracle historical | Private clone/rootUri, listener PID verification and short lifetime do not provide a normal-local-user boundary for the new helper source-read endpoint: invocation 67's same-UID second client read an owned outside-helper script without sending source, and pinned count-only admission, absolute URI mapping, unmanaged `FileAccess` and source-derived symbols support this capability. This is not a cross-UID runtime observation. Same-URI owner/peer parses separated correctly and invocations 68/69 show both owner dependency outcomes follow staged disk, not the other client's overlay; no false owner verdict/cache poisoning was seen. Broad check-only executes static code and can exit 0 for invalid scripts; root-only diagnostics missed a bad unused preload. | Qualify the **one existing FR-020 normal-local-user access boundary** for this new helper endpoint before product selection, without demanding exclusive ownership of all helper input, a single LSP client, hostile same-UID resistance or global Godot confinement. No qualifying public stock boundary was selected/proved, and no impossibility finding follows. If qualified, existing worker could host a bounded source-only helper; pinned protocol/version, conservative admission, warning context, every-source owner fences, ordinary editor effects, deadline/cleanup and error paths remain implementation gates. Do not mandate `gdscript_validation.rs`, privileged OS profile/proxy, service, new gate or patched fallback. |
| Supervisor authorization/stage extension | Read-only worker termination cannot prove a sent mutation never applies. | One control handoff and certainty state in existing supervision; required by FR-012, not a recovery service. |
| Two kit callers based on X enter the same mutation path and overwrite/order changes | Reuse the existing collection/edit slot plus fresh revision checks; that mechanism is sufficient, so no new coordination layer is needed. | Make existing admission/cleanup ownership explicit and maintain T004's barrier regression. This closes a present stale-intent failure with no lock manager, lease service, queue, registry, replay system, merge algorithm or Phase 9 infrastructure. |
| Bridge v2 / edit stdin contract | Strict v1 shape/authentication cannot silently grow; source in argv or arbitrary file options adds exposure. | Coordinated codecs/fixtures/docs migration; one small operation, no compatibility shims or new protocol service. |
| Real stock capability/build proof and edit acceptance runner | T002's patched editor, §18 disposable stock probe and §19 LSP research show bounded evidence, **not** a selected product validator, production integration or A–E acceptance. | After resolving the one FR-020 endpoint gap and selecting a validator, verify official-binary/version matching, qualified normal-local-user access to the new source-read endpoint, exact clone source admission/owner fences/no selected-project code effects, native ABI/syscall/error behavior and owned visible-editor A–E/durability/observation gates; no custom distribution, provider-specific CI or new approval layer. |
| Historical T002 dynamic-getter validation sentinel | A scripted `Engine.get_meta(...)` expression is not constant-folded, and public autoload type resolution does not supply the analyzer with a live object constant; either misses the actual effect path. | The existing fixed-name `TESTS_ENABLED` fixture object supplied patched T002's real getter sentinel; its completed build/evidence remains valid, not stock mutation proof or a new product operation. |
| Automatic native discovery leaves a skipped tooling manifest in Godot's generated export extension registry | The existing export hook removes the manifest/library but not the engine-generated registry entry; the real hook-only export then fails at runtime. Rewriting the shared exporter or reproducing its preset filtering would add unnecessary scope. | Keep the installed native directory under `.gdignore` and load its existing manifest explicitly through `GDExtensionManager` in the editor plugin. This uses the existing lifecycle, avoids stale runtime registry entries and missing-binary startup errors, and requires only enabled/disabled/hook-only export plus missing-native regressions. Other extensions keep their normal discovery/export behavior. |

## Verification and Planning Completion

[quickstart.md](quickstart.md) maps all 22 requirements, 26 scenarios and eight success criteria to implementation verification, including all A–E, ≥20 successful sequential edits and ≥3 interleaved unsafe refusals. Existing Rust baseline checks and complete observation acceptance remain required where affected. Real GUI evidence may be maintainer-operated; existing hosted/protected workflow boundaries suffice. No CI provider/runner topology is introduced as a new prerequisite.

The original plan-generation pass performed only feature-path/workflow resolution, pinned-source/API review, specification/design coverage and local-link/anchor/Markdown/whitespace/full-diff checks. No Cargo/product suites, engine build, runtime probe, filter retry, A–E acceptance, task generation or analysis command was run by that pass. The installed `setup-plan.sh --json` was executed with the verified Feature 002 directory and retained the existing plan; the installed template was resolved before completing these artifacts. Before/after-plan hook checks found no `.specify/extensions.yml`, so no hook was registered or bypassed. That pass's read-only native/flow contract review findings were corrected without changing the then-current specification.

**Current design disposition — final installed analysis complete:** [§18 T1](research.md#18-preserved-mtime-behavioral-finalization-research-2026-09-28) alone selects restored-T0 same-fd handler-free saved state against §17's later Save/history failure. [§19 research](research.md#19-one-shot-stock-lsp-validation-research-2026-09-28) leaves stock product validation at **L2: one FR-020 normal-local-user helper source-read endpoint gap**. Invocations 1–65 remain source/closure evidence; 66 separately passed its bounded privacy-only no-log control; 67 retained owner version-1 valid and other version-37 invalid results while exposing same-UID outside-helper source reads; 68/69 showed owner valid with staged-disk `int`/peer `String`, then owner invalid with staged-disk `String`/peer `int`. No false owner verdict/cache contamination was observed, and no cross-UID runtime result was obtained. §14 P1/§16 H3 are historical handler research, not independent current blockers. Earlier installed analysis of tentative L1 plus I1 correction is historical, not a current no-blocker finding; final installed analysis is complete with one HIGH FR-020 endpoint-privacy hold. T003 is **unstarted/not implementation-ready**, five tasks/dependencies unchanged, and all implementation and A–E acceptance gates pending.

### Stock research artifact review

**Historical §13 result only; superseded for saved-state production by §18 T1 (after §14 P1).** The following analysis text is preserved as the actual earlier result, not rewritten as if it assessed the retained-mtime design.

The read-only `/speckit.analyze` workflow was rerun on 2026-09-28 with the verified
`SPECIFY_FEATURE_DIRECTORY=specs/002-edit-open-gdscript` override; its installed
prerequisite helper passed and no before/after analysis hooks were registered.
All 22 functional requirements and eight success criteria retain task owners
(100% ownership coverage), with five tasks, two complete, and unchanged scenario
and dependency coverage. No new ambiguity, duplication, unmapped task or
constitutional waiver was found.

**U1 — HIGH, acknowledged implementation-readiness blocker:** no supported stock
one-target saved-state mechanism is selected that closes saver fallback and
unrelated-buffer effects. The explicit T003 hold is therefore correct; this
analysis is **not** a passing implementation-readiness result. Resolve that
mechanism and review affected artifacts before `/speckit.implement T003`.
The independent stock validation/effect-confinement question was not reassessed.

### Post-persistence design review

**Historical §14 assessment, superseded for production by §18 T1.**

The installed read-only `/speckit.analyze` workflow was rerun on 2026-09-28
with `SPECIFY_FEATURE_DIRECTORY=specs/002-edit-open-gdscript`; its required
prerequisite helper resolved this feature and passed. No before/after hooks
were registered. Independent semantic and evidence/contract reviews covered
the changed design, constitution, spec, tasks and affected contracts.

All **30 buildable requirements** (22 FRs and eight SCs), **26 scenarios**
and **eight edge cases** retain task ownership: **100% coverage**, five tasks,
two complete and three pending. No unmapped task, harmful duplication,
new requirement ambiguity or critical constitutional conflict was found.

Review found and corrected a temporal-version inconsistency: pre-edit current
is the preparation baseline, but persistence/pre-tag guards freeze the
**post-complex-edit current** and **preparation-time saved** versions
(6/6 → 8/6 → native 8/8). Evidence review also corrected T002's implemented
patched-B wording, identified T001's historical numeric-mtime layout for
explicit T004 migration, and restored historical labels on superseded
caller/bridge/verification recipes. No product code or final API shape changed.

**U1 — HIGH, acknowledged implementation-readiness blocker:** official-stock
exact-source validation and confinement of automatic/edit-generated,
saved-handler and deferred debugger effects remain unresolved. P1 resolves
the saved-state question, not U1. The corrected research/design artifacts are
consistent; this is **not a T003 readiness pass**. T003 has not started.

### Stock validation and effect design review

**Historical §15 assessment, superseded for production by §18 T1; the exact-source validator obligation remains.**

The installed read-only `/speckit.analyze` workflow ran on 2026-09-28 with
`SPECIFY_FEATURE_DIRECTORY=specs/002-edit-open-gdscript`; the prerequisite
resolved the exact feature override and passed. No before/after analysis hooks
were registered. Semantic and evidence/contract review covered the §15 V2
decision and affected plan, native contract, tasks and status.

All **30 buildable requirements** (22 FRs and eight SCs), **26 scenarios**
and **eight edge cases** retain task ownership: **100% coverage**, five tasks,
two complete and three pending. No unmapped task, new requirement ambiguity,
harmful duplication or CRITICAL constitutional conflict was found. Ownership
is planned work, **not** completed acceptance or tested architecture.

**U1 — HIGH, intentional implementation-readiness hold:** P1 resolves saved
state, not stock validation/effect safety. The next focused design question is
useful source/context admission of unavoidable target-version validation,
export and deferred callbacks on attributed dependency bytes/generations before
forbidden dispatch. Independently, a qualifying explicit stock validator
remains unselected/unproved. Both obligations require proof; one successful
callback experiment cannot make T003 implementation-ready. V2 retains T003
unstarted, without a product edit or A–E/runtime acceptance claim.

Evidence review identified three documentation defects: the historical
command ledger, a pinned LSP citation extending past EOF, and unqualified
single-blocker framing. They are corrected within this research PR; no
product behavior, task count, dependency, P1 mechanism or specification
guarantee changed.

### Saved-handler admission design review

**Historical §16 P1 handler assessment, superseded for production by §18 T1; its handler-specific admission hold is not a current production blocker.**

The installed read-only `/speckit.analyze` workflow ran on 2026-09-28 with
`SPECIFY_FEATURE_DIRECTORY=specs/002-edit-open-gdscript`; the prerequisite
resolved the verified Feature 002 override and passed. No before/after
analysis hooks were registered. This review assessed the [§16 H3
research](research.md#16-stock-saved-handler-admission-research-2026-09-28)
and current task ownership, not a stock implementation or acceptance run.

All **30 buildable requirements** (22 FRs and eight SCs), **26 numbered
scenarios** and **eight edge cases** retain task owners: **30/30, 26/26 and
8/8** ownership. Five implementation tasks remain, T001–T002 complete and
T003–T005 pending; no unmapped task, new requirement ambiguity, harmful
duplication or CRITICAL constitutional conflict was found. Ownership is not
completed acceptance. Read-only evidence/source review identified five
material §16 research corrections, and semantic review identified a current
research-introduction inconsistency. All six were corrected and checked
against pinned source and retained runtime evidence; Markdown, local links
and source anchors passed verification. These documentation corrections
do not resolve U1 or establish implementation readiness.

**U1 — HIGH, intentional implementation-readiness hold:** H3 establishes no
useful safe admission profile for the tested stock saved-handler/public-observer
composition, not universal impossibility. P1 remains selected for saved state
only. Actual consumed source/dependency/parser/shallow/export generations and
reverse-consumer attribution, current-editor/function-discovery and ordinary
validation/export callback closure, and shared deferred runtime routing and
completion remain independent pre-dispatch safety obligations. An explicit
exact-source stock validator with attributable dependencies/context and
completed valid/invalid/unavailable results remains separately unselected and
unproved. This research/design assessment is complete, but T003 remains
unstarted and not implementation-ready; no accepted admission predicate,
product edit, A–E acceptance or passing architecture claim follows.

### Minimal behavioral finalization design review

**Historical §17 un-restored direct-tag assessment, superseded for production by §18's same-fd T0 restoration; the observed §17 failure remains valid.**

The installed read-only `/speckit.analyze` workflow ran on 2026-09-28
with `SPECIFY_FEATURE_DIRECTORY=specs/002-edit-open-gdscript`. Its prerequisite
helper ran once with `--require-spec --require-tasks --include-tasks`, passed
and resolved the verified Feature 002 override; checks found both BEFORE
and AFTER analysis hooks absent. The review read the spec, plan, tasks,
constitution and full intended diff, with independent semantic and evidence
reviews of the [§17 M3/F-blocking research](research.md#17-minimal-behavioral-finalization-research-2026-09-28).

All **30/30 buildable requirements** (22 FRs and eight SCs), **26/26 numbered
scenarios** and **8/8 edge cases** retain task ownership. Five tasks remain,
T001–T002 complete and T003–T005 pending. No unmapped task, new requirement
ambiguity, harmful duplication or critical constitutional issue was found.
Evidence review identified three §17 documentation corrections—relaunch
autosave attribution, the full excluded OS-delivery list and explicit
requirement classification—tracked in §17; they do not change the measured
candidate failure or establish a production route.

**U1 — HIGH, intentional implementation-readiness hold:** §17 M3 rejects the
tested exact minimal no-handler route on later ordinary Save/history behavior,
not every handler-free approach or proof that the whole P1 handler is necessary.
P1 remains held under §16 H3 for actual-effect safety; an independent explicit
exact-source stock validator is still unselected/unproved. The research/design
assessment is complete, **not an implementation-readiness pass**: T003 is
unstarted, and no accepted behavioral finalizer, safe effect admission,
product edit or passing mutation-acceptance gate follows.

The three publication corrections were verified against retained raw evidence.
Markdown checks passed for all five changed documents; 249 local paths/anchors,
the five-task/two-complete inventory and retained evidence hashes passed.
Owned throwaway projects, probe sources, binaries and generated SDK files were
removed; raw observations and provenance remain local. No production module,
API visibility, lifecycle representation or implementation shape changed.
These are research-publication checks, not T003 or Feature 002 acceptance.

### Preserved-mtime behavioral finalization design review

**Historical §18 assessment only, before a stock validator was selected:** the following U1 finding describes that earlier unselected validator, not T1's saved-state decision. The tentative unpublished L1 readiness assessment later recorded below has itself been superseded by L2; neither historical review is the final installed analysis.

The installed read-only `/speckit.analyze` workflow ran on 2026-09-28 with
the verified `SPECIFY_FEATURE_DIRECTORY=specs/002-edit-open-gdscript`.
Its prerequisite ran once with `--require-spec --require-tasks --include-tasks`,
resolved this feature and passed. Before/after hook inspection found no
registered analysis hooks. Parent and independent semantic/evidence reviews
covered the specification, constitution, complete intended nine-document diff,
task inventory and accepted raw §18 observations.

All **30/30 buildable requirements** (22 FRs and eight SCs), **26/26 numbered
scenarios** and **8/8 edge cases** retain task ownership. The five-task
granularity/dependency structure is unchanged: T001/T002 complete, T003–T005
pending. No unmapped task, new requirement ambiguity, harmful duplication or
CRITICAL constitutional conflict remains. Coverage is ownership, not acceptance.

Review found and corrected three publication inconsistencies: an extra
undefined effect-safety architecture hold in status/task prose; a surviving
saved-handler admission clause in the native contract; and wording permitting
overall application uncertainty after a known content write. The corrected
contract keeps known `Applied`/`PartlyApplied` as `AppliedUnverified`, limits
unknowns to actual missing observations, and separates logical persistence
from short-write syscall counts. Generic safety remains an invariant, not a
second handler-era prerequisite.

**U1 — HIGH, intentional implementation-readiness hold:** the remaining
architectural blocker is **exact-source valid / invalid / unavailable stock
validation**, including its existing source/context attribution and confinement
requirements. T1 establishes bounded behavioral finalization, not that validator
or product A–E acceptance. **T003 implementation did not start.**

Implementation-shape review is documentation-only: no production module,
visibility, lifecycle representation, schema or API changed. Principle XIII
justifies the same-fd metadata operation against the demonstrated false Save
and removes the obsolete handler machinery. Native research compiled with
`-Wall -Wextra -Werror`; accepted evidence is 15 final-campaign cases plus
separate current-build history, with fixture exclusions and the stock wrap
diagnostic retained. All owned editor processes ended; 134 owned disposable
projects/control paths, probe sources, binaries and SDK scaffolds were removed
after evidence hashing. Raw JSON, logs, screenshots and provenance remain local.
No Markdown language server is configured; no Cargo/product suite was rerun for
these documentation-only repository changes.

### One-shot stock LSP validation design review

**Intermediate review of tentative unpublished L1, superseded by invocation 67; not the current disposition.** The historical coverage and I1 findings below remain factual but do not certify client-input isolation or T003 readiness.


The installed read-only `/speckit.analyze` prerequisite ran once with the
verified Feature 002 override, `--json --require-spec --require-tasks
--include-tasks`; no `.specify/extensions.yml` hook was registered in the
before-check. The actual [analysis report](file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-one-shot-lsp-39y7q6v3/evidence/spec-analysis-before-privacy.json)
and [intended-diff review](file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-one-shot-lsp-39y7q6v3/evidence/intended-diff-review.json)
covered the combined §18 T1/§19 L1 design: **30/30 buildable requirements**
(22 FRs and eight SCs), **26/26 numbered scenarios**, **8/8 edge cases**,
five implementation tasks with T001/T002 complete and T003–T005 pending.
There are no unmapped tasks, new requirement ambiguity, harmful duplication,
constitutional waiver or CRITICAL conflict; coverage assigns work, not product
acceptance. The prior [publication review](file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-one-shot-lsp-39y7q6v3/evidence/publication-review.json)
found three factual publication errors now corrected: invalid out-of-range CLI
port is not an occupied port; filtering the `GODOT_` fixture-control variable
is not HOME/XDG isolation; unchanged validation is not proposed-source preflight.

The actual analysis identified **I1, HIGH bounded publication inconsistency**:
the proposed production `--log-file` recipe forced a file logger even where
editor logging is disabled. This was **not** a HIGH architectural selection
blocker in that intermediate review. The corrected **candidate** privacy recipe omits that flag, discards helper stdout and stderr without retaining/forwarding source-bearing startup text, and would return only requested bounded LSP diagnostics. Pinned
[stock logger setup](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/main/main.cpp#L2312-L2339)
supports the correction; earlier synthetic research commands with `--log-file`
remain actual historical evidence. A separate
[official-stock privacy control](file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-one-shot-lsp-39y7q6v3/evidence/privacy-verified.json)
launched without `--log-file` and with stdout/stderr discarded. In **2.494936 s**
spawn-to-reap, both root/helper source fences completed, a private-sentinel
helper error was returned in the requested bounded LSP diagnostic, and
inspection of owned project/HOME/XDG files found no `.log` or stdout/stderr
capture file and no sentinel outside the intentionally staged helper source.
This privacy-only control is the **66th helper invocation**, distinct from the
65-study invocations and their final 10-case/21-source-fence closure; it passed
no selected target editor/project path and adds no live GUI nonmutation witness
or full-feature privacy guarantee.

**Intermediate disposition, superseded:** At this point I1 and the three publication errors had been corrected, and the review identified no further HIGH architectural selection blocker **under its then-assumed exclusive source universe**. This was not a final readiness pass. Its tentative worker-owned LSP design had not considered the new helper-authority source-read endpoint exposed by invocation 67; its blanket exclusion inference is withdrawn after retained invocation 68/69 owner/dependency results. No public API, wire schema, production module, product support or T003 work followed. All implementation, A–E, durability, privacy/export and live product acceptance gates remain pending.

### Final stock LSP FR-020 endpoint disposition — completed analysis

The installed read-only `/speckit.analyze` workflow was rerun for the final
narrowed L2 disposition with the verified Feature 002 override. Its prerequisite
ran once; no before/after analysis hooks were registered. The actual
[final analysis](file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-one-shot-lsp-39y7q6v3/evidence/final-spec-analysis.json)
and independent
[complete intended-diff review](file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-one-shot-lsp-39y7q6v3/evidence/behavioral-intended-diff-review.json)
found **no merge-blocking publication defect or CRITICAL constitutional conflict**.
Coverage remains **30/30 buildable requirements** (22 FRs and eight SCs),
**26/26 numbered scenarios** and **8/8 edge cases**, with five tasks,
T001/T002 complete and T003–T005 pending. No unmapped task, zero-coverage
requirement, harmful duplication or new material requirement ambiguity was found.
Coverage is ownership, not product acceptance. The one nonblocking historical
“client-isolation” shorthand was corrected to the precise FR-020 endpoint boundary.

**U1 — HIGH, intentional implementation-readiness hold:** the new helper-authority
`.gd` source-read endpoint has no qualified normal-local-user access/privacy
boundary under existing FR-020. Pinned source and invocation 67 establish its
request/read/source-derived-symbol capability, not observed cross-UID disclosure.
The [final behavioral disposition](file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-one-shot-lsp-39y7q6v3/evidence/behavioral-final-disposition.json)
supersedes the exclusivity interpretation, not the retained measurements.
The [inverse dependency evidence](file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-one-shot-lsp-39y7q6v3/evidence/peer-dependency-verified.json)
confirms owner-valid staged-disk `int`/peer `String` and owner-invalid staged-disk
`String`/peer `int`; no false owner result was observed. Another connection or
independent document alone is not an owner-validity failure or `unavailable`.

The declined additional experiment remains a tooling/environment limitation;
no retry, reformulation, Godot failure or semantic downgrade is inferred.
No single-client requirement, hostile same-UID hardening, global Godot sandbox,
new control or L3 impossibility finding is selected. This documentation-only
implementation-shape review changes no production module, API visibility,
lifecycle representation or schema. Principle XIII retains only the existing
FR-020 obligation tied to the concrete new access path.

T1 remains selected/closed and unchanged; T002 remains a completed oracle.
**T003 is unstarted/not implementation-ready** while U1 remains unresolved.
T004 retains future coherent evidence/caller/bridge migration and integration;
T005 retains cumulative acceptance. Generic confinement/effect safety,
A–E, durability, privacy/export and full-edit timing gates remain mandatory.
No stock product validator, helper module, mutation implementation or feature
completion follows this analysis. Normal one-PR/no-auto-merge/STOP sequencing remains.
