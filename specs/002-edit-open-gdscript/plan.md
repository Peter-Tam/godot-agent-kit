# Implementation Plan: Safely Edit Open GDScript

**Branch**: `spec/002-edit-open-gdscript` | **Feature identifier**: `002-edit-open-gdscript` | **Date**: 2026-09-27 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/002-edit-open-gdscript/spec.md`

**Status:** T001–T004 are complete under [§18 T1](research.md#18-preserved-mtime-behavioral-finalization-research-2026-09-28) and [§19.9 R1 / L1](research.md#199-capability-delta-readiness-review-2026-09-29). [T003 stock acceptance](quickstart.md#11-t003-native-boundary-acceptance-2026-09-29) records the private boundary; [T004 caller acceptance](quickstart.md#13-t004-caller-acceptance-2026-09-29) records the guarded public caller and affected regressions. T002's patched implementation is archived in [PR #29](https://github.com/Peter-Tam/godot-agent-kit/pull/29), not retained in HEAD. T005 and cumulative feature A–E remain pending; the stock endpoint limitation is non-blocking. Older design/review records describe their time, not the current lifecycle.

## Summary

Deliver one native, revision-guarded whole-source edit to an existing open standalone GDScript in one explicitly selected local editor. A caller supplies a checked prior-observation basis and intended source; Rust/core freshly checks independent D/R/B, dirty state and identities, authorizes application, and independently verifies resulting source, clean/saved state and source-attributed parse evidence before success. Refused/unchanged requests add no source/history changes; partial/uncertain changes never imply rollback or retry safety.

The private T003 boundary implements [§18 T1](research.md#18-preserved-mtime-behavioral-finalization-research-2026-09-28): retained-fd content persistence, same-fd T0 mtime restoration/readback, public Resource edited=false, CodeEdit saved-version tag and STOP. It pairs with a Rust-owned one-shot official-stock LSP validator using per-source diagnostics/symbol fences. §14–§17 and PR #31/#33 remain historical route evidence; T002's removed patched validator is archived in [PR #29](https://github.com/Peter-Tam/godot-agent-kit/pull/29). T004 owns the public caller/core/bridge migration, not another private T003 implementation; endpoint limitation remains non-blocking.

Shared boundary (T003 private stock helper/native stages implemented; T004 caller path planned):

```text
edit-gdscript / typed caller
  -> existing Rust core and supervised worker / authenticated bridge (planned)
  -> confined exact proposed-source/closure capture and prelaunch admission
  -> one-shot matching official-stock LSP helper; exact per-source completion fences
  -> fresh mutation guards -> addon + standard C-ABI GDExtension on stock editor
  -> exact CodeEdit native edit -> explicit Script R synchronization
  -> retained-fd D content persistence -> same-fd T0 metadata restore/readback
  -> fresh guards -> public edited=false -> fresh guards -> CodeEdit tag -> STOP
  -> independent actual-source/closure capture -> separate fresh helper;
       independently verify D/R/B/dirty/saved/context and validation
```

The previous [§11 patched-engine decision](research.md#11-concrete-native-design-and-resumed-planning), [§13 ordinary Save research](research.md#13-stock-target-save-finalization-research-2026-09-28), and [Phase 1 concurrency boundary](spec.md#phase-1-concurrency-boundary) remain historical evidence/requirements. §13 disqualifies custom ResourceFormatSaver fallback, all-script target Save and direct `ResourceSaver.save`. §14 handler/§16 H3 callback admission are superseded for production by §18's handler-free sequence; §17 remains the negative for tagging without restoring mtime. T002's patched validator remains a semantic oracle/hazard inventory, not a stock binding. The §15 broad `--check-only --script` negative remains valid; §19's isolated LSP design does not repeat that route.

## Technical Context

**Language/Version**: Existing Rust **1.98.1**, edition 2021; GDScript for the thin editor integration; **C++17** for the small standard-ABI extension. Python **3.10+** remains the owned-fixture driver environment.

**Primary Dependencies**: Reuse existing locked `serde =1.0.229`, `serde_json =1.0.151`, `cap-std =4.0.3`, `ring =0.17.14` and existing system toolchain. No new Rust crate, transport library, godot-cpp layer, parser, process service or public transport adapter is selected. The selected one-shot helper uses stock Godot LSP over loopback TCP with Content-Length JSON-RPC inside the existing supervised worker. No new module/API or OS profile/proxy is selected. Generate/use the pinned engine's public GDExtension C interface and opaque object/Variant dispatch for T1, preserving upstream SDK notices. Native OS I/O uses the demonstrated macOS descriptor APIs.

**Engine and version scope:** Earlier [§18 T1](research.md#18-preserved-mtime-behavioral-finalization-research-2026-09-28) research on official stock Godot 4.7.2 recorded 16 accepted case observations (15 final-campaign plus one separate settled history), not a 16/16 campaign or product A–E proof. §19 research recorded 65 source/closure study invocations, one separate privacy control and three additional-client probes (67–69), not product passes. T003 subsequently accepted the private stock boundary on macOS 26.6.2 arm64 with executable SHA-256 `c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf`; see [quickstart §11](quickstart.md#11-t003-native-boundary-acceptance-2026-09-29). Other platforms and public mutation support remain unclaimed. T002's patched-editor provenance remains in [PR #29](https://github.com/Peter-Tam/godot-agent-kit/pull/29).

**Storage:** The T003 worker uses the existing owner-private source-free session registry and one selected-project script, with request-local descriptors/evidence and an owned per-invocation source-only scratch project for the bounded captured closure. Its no-log, discarded-output, isolated HOME/XDG context is not an OS sandbox. T004 still owns authenticated caller binding and independent product outcome integration; [§19.9](research.md#199-capability-delta-readiness-review-2026-09-29) documents the non-blocking inherited endpoint limit.

**Testing:** T003 passed private stock primitive/helper, visible-Godot history/durability, privacy/export and affected observation regression evidence in [quickstart §11](quickstart.md#11-t003-native-boundary-acceptance-2026-09-29). Public A–E caller and cumulative feature gates remain T004/T005 work; the quickstart records both past results and remaining acceptance.

**Project Type**: Existing local Rust library/CLI plus editor addon/native integration. No MCP exposure.

**Performance Goals**: Controlled edit result ≤10 seconds (9.5-second acquisition/operation cutoff + 0.5 delivery reserve), including an unresponsive editor. Observation remains ≤5 seconds with its existing 4.5-second supervision. No sustained-throughput or arbitrary-project-size claim.

**Constraints**: One open standalone script; explicit target/session and expected basis; exact independently agreeing D/R/B and attributable clean state; native single-step history; target-bound non-redirectable persistence; no arbitrary evaluation/execution/outside-project access; truthful partial outcomes; no automatic retry/rollback. Success is an observed interval, not a frozen editor or arbitrary same-inode serialization.

**Scale/Scope**: Root/replacement and each source ≤512 KiB UTF-8; bounded 32 dependency sources / 4 MiB dependency bytes; one active editor collection/edit slot; inherited 32-peer/descriptor bounds. Full-source LF representation, explicit unsupported NUL/CR/BOM refusal, and native Save-profile eligibility preserve exact source rather than normalize it. Caller/bridge/diagnostic bounds are in the contracts.

## Selected Stock Saved-State Composition (T003 Private Implementation)

**T1 is implemented for the private T003 native attempt, not a public product
edit.** Prepare independently clean D/R/B and the same bound open
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
implementation is accepted for T003 private use; T004 still owns product integration.

## Selected Stock LSP Exact-Source Validator — L1 (T003 Private Implementation)

The [§19 source-only design](research.md#19-one-shot-stock-lsp-validation-research-2026-09-28) is implemented by T003's Rust-owned one-shot helper: capture exact proposed source and recursively supported literal `.gd` inheritance/preload dependencies, resolving relative paths against each referring source **before launching** a matching official Godot binary. Stage the immutable closure in a fresh private source-only project with isolated HOME/XDG and admitted effective warning context. Every staged URI requires its own diagnostics and parser-symbol completion fences; root-only completion is insufficient.

A small conservative prelaunch lexical admission and no-follow confined reader, **not** a second GDScript parser or generic effect system, recognizes literal `.gd` inheritance/preload and recursively bounded transitive paths. Permit native bases/ClassDB and same-build inert source-only `@tool`/static/init/new declarations. Return `unavailable` for unsupported/unresolved/computed/escaping dependencies, non-`.gd` resources, UID/remap/binary/embedded dependencies, out-of-project or document-link-probed literal paths, or global-class/autoload/GDExtension/custom-loader/object-value context the clone cannot reproduce. Restrict **all** link-probed string literals (`FileAccess::file_exists`), not just dependency expressions, before startup. No selected addons, plugins, scenes, resources, old UID/.godot/cache state or startup scripts enter the clone. Clone-only Godot `.gd.uid`/`.godot` bookkeeping writes are permitted; selected target D/R/B and project code are not. A dependency warning promoted to ERROR changes validity: include effective warning/directory policy in the bounded context witness, or refuse. Do not reject `@tool` or static syntax merely by spelling; §19's isolated pure `.gd` scans showed no project-code sentinel, whereas non-`.gd` scripted-resource loading and rejected addon loader hooks had effects before/around initialize.

The T003 worker launches the pinned official binary `--headless --editor --path <owned-project> --lsp-port <unique-loopback-port>` without `--log-file`, discards stdout/stderr, verifies child-owned loopback attribution and requires exact-URI diagnostics followed by shaped parser symbols for every admitted root/dependency URI. Earlier §19 synthetic studies logged only for provenance; [stock logger setup](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/main/main.cpp#L2312-L2339) shows why explicit logging is excluded. Root-only fencing was empirically false-valid for an invalid unused preload. Complete owner fences with severity-1 errors yield `invalid`, complete severity-1-free fences without missing witnesses yield `valid`, and incomplete/mismatched/unsafe evidence is `unavailable`; another connection alone does not invalidate owner evidence. The worker terminates/kills/reaps its owned child within budget; stock LSP `shutdown`/`exit` do not stop it.

Invocation 67 exercised two separate same-UID synthetic client connections in one owned process. After the owner initialized but **before its `didOpen`/fence**, the second client read source-derived symbols from the staged root and a separate owned sibling `.gd` **outside** the helper project, without supplying source text. It then opened the same root URI at version 37 with different inert typed-invalid text. The owner subsequently opened its captured version-1 source; its raw parser outcome was valid, while the second client's was invalid on its own connection. No managed-text overwrite, mixed diagnostic, false owner-valid verdict or cache-poisoning result was observed. In [inverse dependency invocations 68/69](file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-one-shot-lsp-39y7q6v3/evidence/peer-dependency-verified.json), the second client's version-37 overlay for the same helper URI opposed disk: disk helper `int`/peer overlay `String` yielded owner `valid`, then disk helper `String`/peer overlay `int` yielded owner `invalid` with the expected root severity-1 return-type error. Owner helper/root version-1 per-URI diagnostic-plus-symbol fences and peer helper version-37 fences retained their distinct source. Source review attributes dependency-parser production to staged disk, separately from peer-managed parser text; the tested owner outcome followed disk in **both** directions, not the peer overlay. Stock LSP accepts multiple peers (up to eight) and maintains per-peer managed files/parse results. A second trusted client or independent document parse is not a semantic failure. Separately, [count-only peer admission](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/language_server/gdscript_language_protocol.cpp#L142-L150), [outside-project absolute URI retention](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/language_server/gdscript_workspace.cpp#L502-L578), [unmanaged `.gd` reads under helper authority](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/language_server/gdscript_language_protocol.cpp#L407-L440) and [source-derived `documentSymbol` values](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/language_server/gdscript_text_document.cpp#L145-L156) establish the **inherited stock-LSP read capability**, also present in normal editor use. No cross-UID reachability/disclosure was run or observed, and no new authority for an in-scope actor was established. [§19.9](research.md#199-capability-delta-readiness-review-2026-09-29) applies FR-020’s capability-delta gate and records R1, not a privacy acceptance pass or a global sandbox claim. No additional experiment, privileged recipe, proxy or OS profile is prescribed.

The T003 private result binds request/session/target/purpose, exact root/source paths and hashes, bounded diagnostics, closure/context witness, completion/incomplete reason and supervisor-clock interval; it is not T002's engine-derived SHA/editor-clock/parser-stage/cache layout or a public wire schema. Changed intent validates captured proposed source before native effects and independently captured actual source with a separate fresh helper after T1 STOP; unchanged intent validates without mutation. Known application with invalid/unavailable post-change evidence remains partial, without rollback or retry. Earlier §19's 65 study and separate 66–69 invocations were research, not A–E or implementation proof: first 44 normal lifecycle 1.845480–2.347037s (median 1.9041955), final 10 1.993938–2.582493s (median 2.1155485), one focused followup 4.654501s. T003 later provided separate accepted product-boundary evidence in [§11](quickstart.md#11-t003-native-boundary-acceptance-2026-09-29); no arbitrary-project performance guarantee follows.

**T003 implementation and acceptance complete.** The existing supervised Rust worker owns the one-shot helper alongside the T1 native mutator; [quickstart §11](quickstart.md#11-t003-native-boundary-acceptance-2026-09-29) records exercised private behavior. T004 owns coherent typed caller/bridge/core evidence integration and historical T001 field migration; no new crate, service, scheduler, parser port, engine patch, sixth task or public schema came from T003. The developer's existing LSP remains independent. [§19.9](research.md#199-capability-delta-readiness-review-2026-09-29) records the non-blocking inherited endpoint limitation.

## Implementation Boundaries and Historical Patched Primitive A

### 1. Core intent, revision and compatibility

Add a focused script-edit module to the existing Rust library, using existing checked selectors, identity/collection/source evidence and classification patterns. The core consumes protocol-independent evidence; no JSON, TCP, Godot object or native ABI dependency enters its policy. Do not build a general Phase 2 transaction/revision framework.

The planned `edit-gdscript` caller will accept the selectors and bounded stdin JSON in [edit-api.md](contracts/edit-api.md). Whole-source intent avoids patch-coordinate ambiguity; source cannot enter argv or an arbitrary file-reading option. Expected basis extracts project/session/document/file identity, agreed source digest/length, exact CodeEdit current version and attributable clean state from prior observation. Observation v1 has no saved-version field; T003 native preparation obtains it directly. Pre-edit guards compare baseline current; after the complex edit the attempt freezes new current and retains preparation saved version for the [implemented private stock transition](#selected-stock-saved-state-composition-t003-private-implementation). T004 still owns authentic caller binding; a prior snapshot cannot itself authorize a write.

Keep observation's public v1 meaning, timing and per-surface availability unchanged. Private bridge **v2** is a coordinated clean cutover with authenticated edit/native capability fields, changed HMAC domain and strict tuples. Migrate all Rust/addon/worker/fixture peers and docs; no dual-protocol shim or v1 mutation fallback. Restart/re-enable the addon and observe again after upgrade.

### 2. Historical native application and Primitive A (not executable)

Use one native CodeEdit complex operation on the exact bound buffer; preserve previous undo history and unrelated tabs, with normal native redo-branch invalidation permitted. Do not use `set_text`/clear-history as an edit shortcut or create an alternate EditorUndoRedoManager history for TextEdit's existing text history. Never choose the destination by focus.

Immediately recheck current B/version and original R, then use the explicit supported **`Script.set_source_code` method**, not property `_set`, `apply_code`, export propagation or reload. Native source synchronization is not a compiled-class/Inspector/live-game update claim. Pending dragged exports or an unsafe Save representation refuse before application; do not alter editor preferences or scene properties.

The C++ integration pins the selected project and existing file, independently confirms its identity against the Rust basis, and writes/truncates/flushes only that retained object after fresh D/R/B/identity guards. Short-write/error/namespace-loss facts remain explicit. Replacement/parent redirection never changes the write destination. Arbitrary same-inode exclusion remains unclaimed; known changed source/identity still stops obsolete work.

The old A proposal prescribed private Resource/document mtime setters and §14 P1's saved-handler callback carried unadmitted tails under §16 H3. **Both are superseded:** T003's [stock native composition](#selected-stock-saved-state-composition-t003-private-implementation) restores retained-fd T0 before public edited=false and direct CodeEdit tagging, without private field manipulation or a handler.

Do not emit `resource_saved`, call Save All/current-tab Save, clear unrelated docs/history or claim normal scene-save effects. The [native Primitive A §3 guarded target-document saved transition](contracts/native-integration.md#3-primitive-a-guarded-target-document-saved-transition) now specifies T1; historical P1 is retained only in research. All behavioral and effect postconditions remain.

### 3. Primitive B and automatic editor effects

The completed T002 patched engine exposed `GDScriptLanguage::validate` → parser → analyzer on immutable exact source/path and bound actual input hash, target/session/request, invocation interval and root/dependency diagnostics. It returned completed valid, invalid or unavailable, not reload/log/silence proxies. This remains a **historical** semantic oracle/reference and hazard inventory, not a production patched dependency or selected stock editor binding.

Its call-scoped read/effect context permitted parser/dependency-cache activity and confined source-backed analysis with attributed dependency bytes and metadata while refusing excluded loader, initialization and object-property paths before dispatch. On official stock, `reload(false)` still compiles and can initialize, and broad `--headless --check-only --script` executed tool/non-tool static initializers while reporting exit 0 for invalid cases. §19 researched a bounded source-only LSP candidate with prelaunch capture/admission and exact per-source parser/diagnostic fences; its ordinary `initialize` analyzes staged source, so ad hoc root-only diagnostics do not suffice. Owner result attribution is established on the tested root/dependency source-only profile, independently of another client's managed text; the inherited endpoint limitation is non-blocking under [§19.9](research.md#199-capability-delta-readiness-review-2026-09-29).

The historical engine's target-local guard does not exist on stock. §16 saved-handler function discovery/live-reload/deferred-debugger hazards explain why that handler is not selected, not a production blocker for T1. Ordinary editor validation/export may still synchronize R or produce effects: confine/prevent forbidden effects and independently check actual source/context. Diagnostic source belongs in the requested bounded result, not incidental logs.

Changed intent uses fresh captured/admitted proposed-source preflight before mutation and a **different fresh** helper on independently read actual source for `post_change` after T1 STOP; invalid original scripts remain repairable if admission and resulting-source validation qualify. Unchanged intent uses one non-mutating helper. T003 implements these private primitives; T004 still owns authenticated caller orchestration and product outcomes. The [Primitive B contract](contracts/native-integration.md#4-primitive-b-exact-source-native-gdscript-validation) records the current stock boundary and T002’s separate historical API, not an instruction to port its editor binding.

### 4. Stage ordering, deadlines and independent verification

The former [ordered state machine](data-model.md#3-ordered-attempt-state-machine) retains stage/certainty semantics, but its patched finalization mechanism is superseded. **T003 implemented the private T1 + stock-validator stages below; T004 still owns caller orchestration and product outcome:**

```text
accept/select
  -> fresh confined proposed-source/closure capture and prelaunch admission;
       one-shot matching official-stock helper with exact per-source fences;
       require complete owner-source preflight valid; recheck context
  -> supervisor records may_apply, then authorizes its existing worker
  -> exact clean bound target/Save-profile/version/namespace guards; retain fd + fstat T0
  -> one native CodeEdit B complex edit / explicit held Script R synchronization
  -> fresh guards; same-retained-fd pwrite/ftruncate/fsync/pread D content receipt
  -> fresh receipt/source/version/namespace guards
  -> same-fd futimens(T0, atime UTIME_OMIT); fstat/content/namespace readback
  -> fresh complete content+metadata receipt and bound target/version guards
  -> public Resource edited=false; fresh guards; CodeEdit tag_saved_version; STOP
  -> independently read actual source/closure/context;
       separate fresh helper for post_change
  -> independent D/R/B, dirty/saved and context verification
  -> one Rust-classified terminal outcome
```

The first native call that can change source/history is the local potentially-applied boundary. The supervisor becomes conservative earlier, **before** releasing its worker's one-shot authorization, so a lost worker/acknowledgment cannot become a false not-applied result. Before that handoff, preparation cannot apply; afterward only an attributable irrevocable pre-boundary discard can prove refusal. A completed B/R/D change remains known application despite later loss. Reuse the existing supervised worker rather than create a helper service.

Reuse the existing one-active-collection/edit slot for the selected editor/session: claim it during preparation, require its ownership at native mutation entry, and retain it through verification/terminal cleanup. Another edit receives a source-free terminal busy refusal before the potentially-applied boundary, with no source/history/finalization change and no queued or replayable work. Releasing the slot cannot auto-apply rejected intent. A later request needs a freshly observed valid basis; old-basis text compatibility cannot override a changed revision. Human typing remains unblocked. Caller termination does not release this slot while entered native work can still mutate; its uncertainty/deadline semantics remain unchanged. T004 owns the real same-session overlap case under US2.4.

An editor-local remaining-time expiry stops new stages when control returns. Native parser/I/O work can block; the caller's supervised deadline still returns a truthful bounded outcome. Timeout/abort cannot revoke an entered stage or imply rollback; late replies cannot upgrade the result. No retry, reconnect/resume or repair write.

Neither the content/metadata receipt nor the public tag is independent evidence. Rust reads/rechecks D through its own project capability; the existing collector freshly reads actual R and B and document-attributed dirty state, plus separate native saved-state facts. Recheck source/identity/version, namespace and validation dependency/context/save-profile witnesses. Missing, changed, known-stale or divergent evidence prevents success even if an earlier/later sample agrees. Report explicit changed/unchanged/refused/applied-unverified/application-unknown and actionable content-write versus metadata-restore failures; no false not-applied result after a known write.

### 5. Historical implementation and evidence boundaries (not T003 authorization)

The PR #24 patched A candidate and §14 P1 handler were earlier saved-state proposals; [§16 H3](research.md#16-stock-saved-handler-admission-research-2026-09-28) and [§17 M3](research.md#17-minimal-behavioral-finalization-research-2026-09-28) record route-specific failures. T003 implemented [§18 T1](research.md#18-preserved-mtime-behavioral-finalization-research-2026-09-28) and the [§19 stock validator](research.md#19-one-shot-stock-lsp-validation-research-2026-09-28) for its private boundary. T002's patched oracle implementation was removed from HEAD and remains in [PR #29](https://github.com/Peter-Tam/godot-agent-kit/pull/29); it is not a fallback.

Historical native lifecycle/export evidence remains relevant, but no patched saved-state hook, private Resource mtime getter/setter or engine patch is a current implementation instruction.

## Constitution Check

**Current constitutional disposition: PASS for design readiness, not product acceptance.** [§18 T1](research.md#18-preserved-mtime-behavioral-finalization-research-2026-09-28) is unchanged. [§19.9 R1](research.md#199-capability-delta-readiness-review-2026-09-29) selects L1 exact-source validation and corrects the prior L2 privacy classification: FR-020 provenance exists, but no concrete new authority for an in-scope actor or reachable evidence of that delta was established. No constitutional waiver or weakened bridge/project boundary is requested. Principles I–IV’s independent D/R/B, human-work, native-history and verification obligations remain; V/X retain confined caller operations, authentication and source-free incidental output; VI/VIII/XII retain actual privacy/export and live-editor acceptance; XIII rejects an unevidenced isolation prerequisite. The matrix records continuing implementation obligations, not their discharge.

| Governing obligation | Design and future evidence |
|---|---|
| I / IV — independent D/R/B and observed postconditions | Separate actual D and editor R/B/dirty/current/saved reads; attributed post-change B; immutable partial evidence. Neither content/metadata receipt nor clean tag certifies success. Final intended, clean D/R/B, ordinary Save/reopen and native history behavior plus A–E/durability remain mandatory. Intermediate divergence during a guarded attempt is permitted, not an external atomic-every-step promise. |
| II — preserve human work and stale intent | Exact target/Script/editor/CodeEdit/session/namespace, prepared baseline current and saved versions, frozen post-complex-edit current and preparation saved versions for persistence/restoration/finalization guards, fresh source and separate receipt checks; never restore/tag newer work, including same-text newer version. Namespace loss forbids metadata restoration/tag, with no path reopen/retarget. No arbitrary-writer serialization promise. |
| III — native editor semantics | One CodeEdit complex edit, explicit `Script.set_source_code`, one retained-fd logical content-persistence operation (`pwrite` calls as needed for eligible short writes, then truncate/fsync/pread) followed by same-fd T0 mtime restoration/readback and guarded public edited=false/CodeEdit tag/STOP. §17's no-restore tag failed later Save/history; §18's observed stock GUI behavior supports this narrower remedy without handler callback effects. No private numeric mtime/cache parity, Save replay, engine A patch, signal broadcast or ResourceSaver is required. |
| V / X — confinement and truthful outcomes | Existing authentication/private metadata and project-bound no-follow descriptor/source reads remain. Selected prelaunch capture/admission, a fresh source-only clone, effective-warning witnesses and owner root/per-URI diagnostics-and-symbol fences establish scoped owner parse attribution, with dependency outcomes tracking staged disk on both inverse probes. Complete attributable owner source/context/fences with severity-1 diagnostics support `invalid`, without severity-1 or missing witnesses support `valid`; silence, incomplete/mismatched evidence, unsafe admission or deadline yields `unavailable`. An additional client alone does **not** make owner validation unavailable. [FR-020](spec.md) and constitutional V retain product confinement, authentication and privacy; [§19.9](research.md#199-capability-delta-readiness-review-2026-09-29) finds no new in-scope capability beyond stock LSP. No cross-UID disclosure was exercised; the inherited endpoint limitation is not a blocker or a claim of OS sandboxing. Ordinary-editor no-forbidden-effects/pre-effect checks and truthful partial outcomes remain gates; do not import §16 handler machinery. Separate content-write, metadata-restore/error/unavailable, namespace/version, parse/dependency/permission/timeout facts; a known write with failed restore/readback remains `AppliedUnverified`, never application unknown/refusal/not-applied. |
| VI / XII — layered verification and explicit support | Pure transition/boundary tests plus real visible-editor A–E, history, durability, timing, privacy/export and complete observation regression on exact recorded builds. No stock/other-version mutation support extrapolation. |
| VII / IX — protocol-independent, small composable surface | One typed/core edit and one CLI, existing private bridge, no MCP; core owns policy, native owns engine-local primitives. No duplicate transaction model. |
| VIII — tooling/gameplay isolation | Editor-only native integration; real exports and runtime checked with enabled/disabled addon and native artifacts, not reliance on `addons/` or `@tool`. |
| XI — independent implementation and dependency review | Public engine source/ABI, own narrow integration, existing locked Rust dependencies. Preserve SDK notices, review exact new native build inputs/advisories/licenses before their implementation delivery. No unreviewed copied framework. |
| XIII — proportional complexity | **Concrete requirement/failure:** independently prove target-bound clean/intended D/R/B, later ordinary Save without a false external-change prompt, native Undo → Save → Redo → Save and earlier history, reopen, human-work/namespace safety and truthful partial outcomes. §17's **simplest earlier** public edited=false + direct CodeEdit tag, after a descriptor write with a newer mtime, looked clean but a later human Save falsely prompted and failed to persist B29. **Simplest credible remedy selected by §18:** preserve T0 from `fstat` and, after one logical content-persistence operation (`pwrite` calls as needed for eligible short writes, then truncate/fsync/pread), restore T0 by `futimens` on the same retained fd, verify fstat/content/namespace and use public edited=false/CodeEdit tag. §18 observed one 51-byte `pwrite` in its positive stock run; this is evidence, not an exactly-one-syscall requirement. Observed later Save/history passes; no private field parity is justified. **Cost:** one metadata syscall, ownership/permission/error/descriptor-lifetime and platform-specific mtime/ctime/readback/race maintenance and GUI regression, limited to demonstrated macOS arm64; no portability abstraction. This is less than §14 P1's Callable topology, admission, callback generations/reverse consumers, function-discovery/live-reload/deferred routing and version maintenance. Generic effect safety and the separate validator remain; no new service, watcher, cache manager, scheduler or patched fallback. |
| Architecture/compatibility/workflow | The T003 Rust worker and stock native attempt implement §19 L1 / §18 T1 private boundaries with accepted GUI evidence, but no public caller/edit wire. T004 owns typed integration and migration of historical numeric `SavedStateEvidence` and patched-validator fields. T004/T005 dependencies and feature A–E/cumulative gates remain unchanged. |

**Principle XIII — capability-delta proportionality:** Exact-source validation is not supplied by mutable live-editor LSP/cache, root-only diagnostics (observed false-valid unused preload), broad check-only (static code ran and invalid cases exited 0), or log silence. A fresh admitted source-only helper and every-source owner fences are the simplest studied stock route; invocations 67–69 preserved owner/peer attribution and staged-disk dependency semantics. Normal stock editor already exposes the same loopback LSP implementation under the same local-user authority. The [four-field review](research.md#199-capability-delta-readiness-review-2026-09-29) establishes no new in-scope access delta, so global sandboxing, single-client exclusivity, cross-UID hardening, an authentication framework, proxy or firewall subsystem is not justified. Private staging is context isolation, not a proof of endpoint isolation. Costs remain bounded capture/admission, binary/protocol pinning, effective-warning context, per-source fences, startup/owned cleanup and truthful deadline/error handling. T1’s independently justified same-fd T0 costs are unchanged.

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
+-- quickstart.md                   current stock reproduction, historical evidence and remaining feature gates
+-- tasks.md                        five-task tracker and archived granularity review
```

The [task list](tasks.md) retains five PR-sized increments and unchanged dependencies; T001–T003 are complete. Earlier design reviews below are historical and their evidence is not product A–E acceptance. The [T003 implementation-shape review](#t003-implementation-shape-and-constitutional-review) and [stock acceptance](quickstart.md#11-t003-native-boundary-acceptance-2026-09-29) record completed private boundaries.

### Source Code (repository root)

Current implementation and remaining ownership; `planned` entries belong to T004, not the completed private T003 boundary:

```text
mcp-server/
+-- Cargo.toml / Cargo.lock         existing single package, locked dependencies
+-- src/
|   +-- lib.rs                     exports reusable core
|   +-- observation.rs             existing read-only semantics
|   +-- script_edit.rs             public edit-core facade
|   +-- script_edit/               private request/evidence/validation/outcome/attempt modules
|   +-- runner.rs / runner/         existing supervised worker and T003 stock validator
|   +-- project_fs.rs              existing independent D/confinement helpers
|   +-- bridge.rs / bridge/        existing session/authentication/framing boundary
|   +-- bridge/wire.rs             existing caller/worker/bridge JSON codecs
|   +-- bin/edit-gdscript.rs        planned thin caller entrypoint
+-- tests/                         required core/boundary behavioral regressions

godot-addon/
+-- addons/godot_agent_kit/
|   +-- plugin.gd / bridge.gd       existing lifecycle/routing; v2 cutover
|   +-- observation.gd             existing getter-only observations
|   +-- script_edit.gd             existing private native-attempt integration; T004 extends caller
|   +-- export_guard.gd            existing tooling exclusion
|   +-- native/                    installed editor-only stock extension/artifact
+-- native/                        T003 stock session/native attempt/build; no engine patch
+-- tests/
    +-- run_observation.py         retain all existing behavior acceptance
    +-- run_script_edit.py         stock native-primitives runner; T004/T005 extend coverage
```

**Structure Decision:** No new Rust package, top-level product component or generic LSP layer. T003's standard stock native mutator/integration lives in `godot-addon/native/`; the existing supervised Rust worker owns the implemented one-shot helper. T004 integrates typed caller/bridge/core evidence after T003 acceptance. T002's removed engine patch/oracle implementation remains only in [PR #29](https://github.com/Peter-Tam/godot-agent-kit/pull/29). Reuse existing owned fixture, window, authentication and independent witness facilities for later caller coverage.

**Ownership review:** Editor source/edit primitives stay in addon/native integration; confined capture, staging, one-shot helper supervision and owner source fences belong to the existing Rust worker. T004 owns typed/core/bridge/caller integration. Bridge, supervision, confinement and fixture lifecycle keep responsibility-based names. No extra infrastructure, generic LSP layer or generalized script-edit domain is selected.

## Complexity Tracking

No constitutional violation/waiver is requested. §17's direct no-handler tag failed later ordinary Save/history after content persistence changed mtime. §18 demonstrates narrower same-fd T0 restoration before tag on stock macOS arm64 GUI; its permission/error, fd lifetime, namespace, version and regression costs remain. §19's stock LSP candidate demonstrated need for all-source fences; invocations 67–69 established scoped owner parser/source/dependency attribution under the tested two-client source-only profile, while invocation 67 plus pinned source exposed the inherited stock read path outside the clone. [§19.9 R1](research.md#199-capability-delta-readiness-review-2026-09-29) finds no demonstrated new in-scope authority; that limitation is non-blocking. No parser port, framework, privileged OS prerequisite, patched fallback or other-platform abstraction is selected.

| Present mechanism/gap | Simplest alternative and insufficiency | Ongoing cost and present justification |
|---|---|---|
| Standard-ABI native library / bound persistence | Public Save paths can follow replacement paths; external writer adds a separate synchronization boundary. | C++ build, ABI/OS/lifetime tests; justified by demonstrated in-editor object/descriptor behavior. No godot-cpp/crate framework. |
| Selected T1 same-retained-fd mtime restore after bounded D content persistence; §14 P1 handler and patched A historical | §17 direct edited=false + CodeEdit tag produced immediate clean D/R/B but a later human Save falsely prompted external-change reconciliation and did not persist human B29; Undo/Save/Redo/Save also failed. Broad Save has fallback/unrelated effects and P1 carries unadmitted callback tails. | Capture T0 by `fstat` before mutation; after the **single logical** content operation (`pwrite` calls as needed for eligible short writes, then truncate/fsync/pread), `futimens` only the same fd's mtime (atime omitted), fstat/read back content and namespace, then guarded edited=false + direct CodeEdit tag/STOP. One metadata syscall, permission/ownership, fd/error/ctime/platform behavior and independent GUI regression remain costs. |
| Selected one-shot stock LSP validator; T002 patched oracle historical | Mutable user-editor LSP is not an immutable exact-source validator; root-only diagnostics missed an invalid unused preload; broad check-only executed static code and exited 0 on invalid scripts. Per-source fences and staged dependencies establish the studied owner semantics. The stock read endpoint is inherited behavior, with no demonstrated in-scope capability delta. | Reuse the existing worker for bounded source-only capture/admission, effective warnings, every-source fences, matching binary/protocol, owned lifecycle and deadline/error paths. No fixed helper module, new crate/service, privileged OS profile/proxy, authentication framework, global sandbox or patched fallback. Actual product privacy/export verification remains required. |
| Supervisor authorization/stage extension | Read-only worker termination cannot prove a sent mutation never applies. | One control handoff and certainty state in existing supervision; required by FR-012, not a recovery service. |
| Two kit callers based on X enter the same mutation path and overwrite/order changes | Reuse the existing collection/edit slot plus fresh revision checks; that mechanism is sufficient, so no new coordination layer is needed. | Make existing admission/cleanup ownership explicit and maintain T004's barrier regression. This closes a present stale-intent failure with no lock manager, lease service, queue, registry, replay system, merge algorithm or Phase 9 infrastructure. |
| Bridge v2 / edit stdin contract | Strict v1 shape/authentication cannot silently grow; source in argv or arbitrary file options adds exposure. | Coordinated codecs/fixtures/docs migration; one small operation, no compatibility shims or new protocol service. |
| Real stock capability/build proof and edit acceptance runner | Earlier T002/§18/§19 research alone did not establish product mutation; T003 subsequently passed private stock helper/native/export acceptance, not public A–E. | Keep pinned official executable/ABI, isolated fixture fault build and stock native-primitives runner; T004/T005 still own complete caller/cumulative A–E, with no custom distribution or new GUI-CI gate. |
| Historical T002 dynamic-getter validation sentinel | The former patched parser needed a fixed tests-enabled sentinel to expose real getter effects rather than a constant-folded expression. | That engine-patch fixture and its evidence remain archived in [PR #29](https://github.com/Peter-Tam/godot-agent-kit/pull/29), not a current stock test dependency. |
| Automatic native discovery leaves a skipped tooling manifest in Godot's export registry | Skipping the manifest alone left an unusable runtime registry entry in a real hook-only export. | T003 stock integration uses `.gdignore` and explicit editor `GDExtensionManager` loading; accepted enabled/disabled/hook-only exports exclude tooling and dangling dependencies. |

## Verification and Planning Completion

[quickstart.md](quickstart.md) maps all 22 requirements, 26 scenarios and eight success criteria to implementation verification, including all A–E, ≥20 successful sequential edits and ≥3 interleaved unsafe refusals. Existing Rust baseline checks and complete observation acceptance remain required where affected. Real GUI evidence may be maintainer-operated; existing hosted/protected workflow boundaries suffice. No CI provider/runner topology is introduced as a new prerequisite.

The original plan-generation pass performed only feature-path/workflow resolution, pinned-source/API review, specification/design coverage and local-link/anchor/Markdown/whitespace/full-diff checks. No Cargo/product suites, engine build, runtime probe, filter retry, A–E acceptance, task generation or analysis command was run by that pass. The installed `setup-plan.sh --json` was executed with the verified Feature 002 directory and retained the existing plan; the installed template was resolved before completing these artifacts. Before/after-plan hook checks found no `.specify/extensions.yml`, so no hook was registered or bypassed. That pass's read-only native/flow contract review findings were corrected without changing the then-current specification.

**Current disposition:** [§19.9 R1](research.md#199-capability-delta-readiness-review-2026-09-29) established L1 validator design with a non-blocking inherited endpoint limitation; T003 subsequently implemented it with §18 T1 in the accepted private stock boundary. T004/T005, public A–E and cumulative gates remain pending with unchanged five-task dependencies. The analyses below retain their historical design-stage statuses; [T003 acceptance](quickstart.md#11-t003-native-boundary-acceptance-2026-09-29) records the later implementation outcome.

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

**Historical PR #36 analysis:** the recorded result below is superseded by the [capability-delta readiness correction](#capability-delta-readiness-correction-review); its measurements and coverage remain unchanged.

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

### Capability-delta readiness correction review

The [§19.9 review](research.md#199-capability-delta-readiness-review-2026-09-29)
corrects the one PR #36 architectural L2 hold to **R1 / L1 validator design
established**. It compares the normal stock editor with the same-binary helper
and applies all four blocker fields to FR-020 and the inherited threat model.
No new in-scope access capability is established; the endpoint concern is an
inherited non-blocking limitation. T003 is ready but unstarted, T002 remains
complete, §18 T1 and all five task/dependency/acceptance boundaries are unchanged.
This is a documentation-only design correction; no production module, API,
visibility, lifecycle, schema or dependency changed. No new runtime probes or
retries of the declined experiment were run.

**Installed `/speckit.analyze` rerun: PASS for implementation readiness.** The
verified `SPECIFY_FEATURE_DIRECTORY=specs/002-edit-open-gdscript` prerequisite
ran once with `--json --require-spec --require-tasks --include-tasks`; the
resolved directory was Feature 002. No before/after analysis hooks were
registered. Read-only duplication, ambiguity, underspecification, constitutional
alignment, coverage and inconsistency passes found **0 CRITICAL conflicts,
0 HIGH architectural blockers**, no material ambiguity/harmful duplication and
no unmapped task. **30/30 buildable requirements** (22 FRs, eight SCs),
**26/26 numbered scenarios** and **8/8 edge cases** retain task ownership.
The inherited endpoint limitation is non-blocking, not a privacy acceptance pass.
The unchanged five-task decomposition still passes granularity review.

Static verification rendered all six changed Markdown files, resolved 220 local
relative links/anchors and all reference-style citations, and found all 165
retained `file://` link occurrences available. Pinned defaults/startup and
endpoint source anchors were inspected; `git diff --check` passed.
Preservation checks confirmed unchanged §18 research, native saved-state
contract §§2–3, T001/T002 acceptance, five-task dependency graph, coverage matrix,
retained §19.8 observations and historical PR #36 analysis. No Cargo/product
suite, Godot runtime/network/client/privacy probe or declined experiment ran.
T003 implementation did not start; publication is this design-correction PR only.

### T003 implementation-shape and constitutional review

The implementation retains the approved core → existing supervision → addon/native
boundary. T004's public caller/core/bridge evidence migration is not implemented.
Rust admission/capture, LSP fences and process ownership use private responsibility
modules; the reusable reaper has responsibility-based naming and narrow visibility.
The public fixture-facing helper types have a current executable test consumer.
The earlier capability review removed the unsupported stock-native `validate`
stub and unused fixture-build flag alias. The final stock-only cutover retains
common lifecycle/build discovery and stock editing under the sole private
`godot_agent_kit_native` metadata key; the historical oracle implementation
and its build mode are archived in [PR #29](https://github.com/Peter-Tam/godot-agent-kit/pull/29),
not present as a second current build or CI gate.
The retained `session.cpp` owns bounded project/session setup, engine identity
and the small native support functions used by the guarded attempt. Registration
stays in `extension.cpp`; `script_document.cpp` and Rust transaction semantics are
unchanged by this cleanup. No new API, visibility, state model, dependency or
infrastructure layer was introduced. Stock-only build/GUI/export and observation
verification is recorded in [quickstart §11](quickstart.md#stock-only-head-verification).
Native `script_document.cpp` owns one guarded attempt, its descriptors and partial
receipt; its size prompted review, but splitting the same stage/guard invariants
into another framework would not clarify ownership. Receipt booleans describe
independent partial effects; the addon lifecycle itself is a state enum.

Concrete review defects were repaired: source aliases/relative paths and effective
context, malformed or reversed parser fences, missing-reference attribution,
owned-group teardown after worker loss, callback interference before insertion,
reentrant slot release, exact representation and native expiry. The actual
synchronous shutdown regression failed before the slot fix and passed afterward.

The real worker-loss test also showed that leaving `Command`'s configured socket
descriptors alive hid EOF, and an exiting one-shot caller could terminate its
detached cleanup thread before private source removal. Closing the duplicates
and receiving a bounded cleanup acknowledgment from the existing reaper correct
those concrete failures without filesystem work on the delivery thread, a new
cleanup service or a journal. Normal worker-confirmed cleanup is not repeated.
Unknown child lifecycle facts remain nullable; missing cleanup confirmation
cannot be reported as a completed valid validation.

**Principle XIII — ordinary editor effects:** isolated source-only validation
cannot certify the live editor's later export update. A controlled cold tool
inheritance case actually ran its initializer through ordinary editor continuation.
The simpler unguarded CodeEdit path therefore fails the existing no-forbidden-
effects requirement. A narrow native source admission check now refuses tool,
script/global inheritance, load/preload, class registration and export contexts
before history entry. Its cost is explicit capability limitation, a bounded local
recognizer and real regression cases; it avoids an engine patch, global validation
disable, generic effect framework or saved-handler callback machinery.

The unlocked GUI run corrected fixture assumptions without changing mutation
semantics: rescan durability now explicitly requests a public filesystem scan
and observes its completion; fresh-editor setup explicitly opens both witness
documents; the excessive-expiry case supplies its actual boundary value.
Close/reopen refusal compares unrelated work immediately before and after the
native call, rather than treating the human's preceding tab reorder as a native
effect. All I/O failures after B/R application must report partial application.
Existing harness groups now close completed case editors instead of retaining
every earlier editor through the whole group; no new runner or process layer
was added. Bounded requested helper receipts remain available on failed asserts.

Principles I–IV/V/X retain independent D/R/B, human-work/namespace/version guards,
native history and truthful partial results. Applicable VI/VIII/XII gates for
T003 passed in the [recorded real-editor acceptance](quickstart.md#11-t003-native-boundary-acceptance-2026-09-29):
149 stock primitive/export cases, 194 observation regressions and 54 oracle/export
cases. Warning-policy invalidation and a post-change helper deadline preserve
known native application; an initially invalid script is repaired and survives
Save/reopen/runtime use. T003 is complete. T004/T005 and the full feature's
caller/cumulative A–E gates remain pending; no broader support is claimed.

### T004 selected implementation boundary

T004 alone is selected after T003's PR #38 merged. The integration reuses the
existing reducer, same-binary supervisor, authenticated bridge, collection/edit
slot, stock validator and native attempt; it introduces no dependency or service.
The private bridge makes a coordinated v2 cutover while public observation remains
v1. Caller supervision records possible application before releasing one worker
authorization and keeps blocking input, selection and both helper invocations
inside the existing edit deadline.

The historical core evidence migrates to public Resource edited state, actual
CodeEdit current/saved versions, independent disk metadata and the native
same-descriptor T0 restoration receipt. Validation records use the worker's
caller-clock interval, each captured source's diagnostics/symbol fences and
confirmed cleanup; they do not invent private mtimes or patched-parser fields.
The addon owns live editor/native facts, Rust acquisition owns independent disk
and helper evidence, and the protocol-independent reducer owns terminal policy.

Constitutional I–IV require fresh target/revision/dirty guards, native history and
independent intended D/R/B verification. V/VII/X retain authenticated source-free
selection, confinement, redaction and explicit partial/unknown outcomes.
VI/VIII/XII require the task's actual GUI/history/durability, observation and
export regression evidence before completion. XIII's existing authorization and
v2 justifications apply: read-only worker termination cannot establish non-application
after mutation dispatch, and strict v1 tuples cannot silently grow. No additional
approval, recovery, queue, replay or isolation mechanism is introduced. T005's
cumulative whole-feature acceptance remains separate and pending.

#### T004 review corrections

The implementation review identified a concrete ownership failure: calling the
stock validator supervisor from the killable edit worker could leave its separate
child process group and source-bearing scratch without a live cleanup owner after
edit-worker loss. The simpler alternative of killing only the edit-worker PID is
insufficient because the stock worker deliberately owns a separate process group.
The existing surviving edit supervisor therefore owns each stock validation call;
its worker sends a bounded, typed context/purpose request and waits for the result.
The supervisor supplies its checked target/intent and existing cancellation/deadline
to the existing stock validator. Two private IPC messages and their ordering checks
are the additional maintenance cost, justified by the current cleanup/cancellation
contract; no process service, recovery store or new child-lifecycle framework is added.

Other review corrections retain one outstanding sample/recheck cycle and consume it
before collecting another, inspect the actual immutable proposal, preserve factless
native refusals without invented receipts, and send each progress snapshot once.
Replaying every source snapshot in the terminal apply message could exceed the
existing response bound for otherwise admitted input. The terminal message instead
carries only its actual partial event when one exists, plus native receipt facts.
Public edit serialization follows the existing borrowed evidence convention rather
than constructing a second JSON tree. These changes remain within T004's original
safety, boundedness and implementation-shape obligations.

Real caller diagnostics exposed two additional evidence-boundary defects.
Separately parsing facts in one frame had assigned distinct receipt timestamps,
making valid collection containment fail. The receiver now samples one timestamp
per frame, and producer intervals cover their actual getter/native acquisition.
An editor-only R/B sample was also compared against a composite identity requiring
D's inode. The native-state DTO now carries only its observed editor identity and
one set of source/version/saved facts; disk identity remains independently checked
by the receipt and Rust D reader. This removes duplicate fields rather than
inventing or copying a disk witness into editor evidence.

A real mode-000 target exposed a privacy classification defect: native file
admission failure was folded into `unclean_or_unsupported`, and the caller retained
the prior source summary under `dirty_conflict`. Native file admission now returns
specific denial, namespace, observation or evidence-limit reasons without changing
its confinement predicates. Denial reaches the existing source-suppression policy.
The caller routing regression requires `denied_access` and absent source-derived
evidence. This is a correction to the existing privacy contract, not a new control
or an expanded threat model.

The final read-only reviews found missing lifetime binding before preparation,
late publication of known selection, conflated refusal/availability reasons, lost
post-application and partial-finalization evidence, and premature `may_apply`
classification on a failed fresh guard. Corrections retain unique discovery,
compare the actual selected lifetime before source, publish matching selection,
and use the existing confined acquisition before editor-state refusal. Its private
permission-denial fact is edit-only: observation v1 keeps per-surface
`DiskUnreadable` and independent R/B, as its existing contract requires.
The core sequence is now fresh guard → eligibility → authorization → one release;
no fictitious native discard witness or authorization bypass is added. Latest
survivor observations and confirmed edited-flag clearance survive later loss but
cannot establish final verification.

The exact pinned engine also emits nonstandard JSON for admitted source controls.
The existing bridge serialization boundary repairs only malformed escape bytes,
preserving literal backslashes; ordinary frames use native searches and keep the
original byte buffer. Fixture responses reuse this encoding, while independent
disk reads still prove exact source. This necessary wire correction adds no source
restriction, dependency, service, protocol fallback or wider framing allowance.

Implementation-shape review found coherent ownership despite large boundary files:
the CLI owns bounded input/result delivery; worker owns acquisition; supervisor
owns process/authorization lifetime and native-progress reduction; codecs own wire
facts; the core owns outcome policy. Godot bridge owns authentication, routing and
the shared slot; script_edit owns synchronous native lifetime; script_document
owns the single guarded descriptor/object attempt and independent inspection.
Private DTOs and effective module visibility stay scoped to current consumers.
Duplicated state fields were removed; stage facts remain independent observations,
not speculative lifecycle variants. A size-only split or new framework would not
reduce the next task's caller-level acceptance work. No unrelated refactor or new
approval/security mechanism is introduced. The required real-editor acceptance
has now passed as recorded in [quickstart §13](quickstart.md#13-t004-caller-acceptance-2026-09-29).

Caller fault injection exposed a terminal receipt-retention bug: after content
persisted, a failed metadata attempt remained buffered for a later restoration
event that could never arrive. Subsequent read-only verification could then finish
without publishing the known partial persistence. The supervisor now consumes the
terminal metadata receipt immediately. Actual write failure, `futimens` failure,
wrong-T0 readback and retained-fd `EBADF` caller cases preserve known application,
partial receipts and dirty state without a saved tag or success. The earlier
state-only diagnostics remain distinct from the subsequent passing standard GUI
campaign. T004 is complete; T005 retains the cumulative feature gates.
