# Research: Trusted GDScript Creation and Deletion

**Date:** 2026-10-12

**Source baseline:** `b42b66801ccc5470505fa6a9848790a27b083a18`

**Disposition:** R1–R3 have selected designs. Repository/API inspection and an owned mechanics experiment support planning, not implementation acceptance. The four product answers in [the specification](spec.md#clarifications) are unchanged.

## Research decisions

| Question | Selected design | Evidence boundary |
| --- | --- | --- |
| R1 — Confined existence changes | Pin the authenticated project and existing parent chain; create the leaf exclusively, or recheck and unlink the admitted existing leaf relative to its retained parent. Independently reacquire namespace evidence afterward. | Existing descriptor guards plus the stock-guest filesystem experiment; no arbitrary-writer inode-CAS claim. |
| R2 — Editor and Resource transitions | Reuse guarded native close mechanics inside one delete owner, detach the retained Script's target path through public Resource APIs, then remove D. Create keeps B absent and observes R passively. | Existing close acceptance, pinned public engine implementation, and all three deletion-profile mechanics positives. Not a tested guarded create/delete product. |
| R3 — Validation and public execution | Add an absent-root complete-source validation purpose to the existing stock validator, two operation-owned supervisors, and two additive Schema 2 projections. Delete consumes the existing fresh read revision without changing its algorithm. | Existing source/consumer inspection, including Rust LSP navigation; new request/reducer/client behavior still requires implementation evidence. |

## R1: Filesystem authority and existence

The current Rust `project_fs` and `project_fs/validation` code already separates routing, confined source capture and missing-parent witnesses. Native `document_guard` retains descriptors, checks namespace attachment, regular-file identity, ownership and permissions/ACLs. These are the reuse points, not a new filesystem service.

Creation needs a retained **existing parent**, not an invented file revision. Walk each component with existing no-follow directory/permission guards. Confirm leaf absence with a no-follow lookup and native B/R absence. At the authorized native boundary use `openat(parent_fd, leaf, O_CREAT | O_EXCL | O_RDWR | O_NOFOLLOW | O_CLOEXEC, 0600)`. `EEXIST` never becomes an edit, truncation, rename-over or equal-source success. Persist complete bytes through that returned descriptor, flush and independently read back. Zero source bytes still require a newly created regular file. Do not create parent directories, make a live placeholder for validation, or remove a partial file on failure.

Deletion retains the existing target read-only and its source/identity/revision. Immediately before `unlinkat(parent_fd, leaf, 0)`, independently recheck original file identity/source and parent attachment, plus native phase-appropriate safety. Verify absence using the checked parent and fresh project namespace; a failed getter, missing parent or permission failure is not an absent leaf. A recreated leaf invalidates success and is never removed as cleanup.

### Concurrency boundary

`O_EXCL` supplies no-overwrite leaf acquisition. `unlinkat` is **not** an inode-conditional delete; checking a leaf then unlinking its name does not serialize an arbitrary non-cooperating external writer between those steps. Retained parent descriptors prevent redirection through a replacement parent but do not freeze the namespace. Keep the established [guarded single-editor concurrency boundary](../002-edit-open-gdscript/research.md#10-phase-1-concurrency-boundary-audit-and-corrected-planning-decision) and [read-basis limitations](../007-mcp-script-workflow/data-model.md#5-closed-state-and-expected-basis): fresh optimistic checks, no event pumping within a synchronous native step, sticky observed invalidation and independent postconditions. No stronger global atomicity guarantee is selected or inferred.

Detected leaf/parent/source/lifecycle replacement must prevent the next effect. A race discovered after an effect must preserve new work and report partial/unknown state. Advisory locks would not exclude non-participants; a rename-to-quarantine/trash design adds another destructive namespace effect and recovery promise without solving that stronger problem. Neither is selected.

### UID and engine bookkeeping

The [pinned filesystem implementation](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/file_system/editor_file_system.cpp) shows that `update_file`/scan can update UID/global-class bookkeeping and queue Script documentation loading. They are not passive absence observations and are not independent mutation verification. The experiment confirmed that ordinary scanning can load a newly created Script and create its `.gd.uid` sidecar without opening B.

Select **no explicit scan, `update_file`, UID write/delete or dependency cleanup inside create/delete**. Existing `.gd.uid` files may remain after deletion and may be reused by ordinary engine bookkeeping on same-path recreation; this does not constitute a surviving loaded Resource association or repair references. Inspect an existing target `.gd.uid` companion through the same no-follow/owner/regular-file boundary before effects; an unsafe link or non-regular companion refuses rather than setting up ordinary engine bookkeeping to follow it. Do not introduce a companion-file mutation API or claim UID identity isolation across recreation. File/Resource/document lifetime checks, not UID reuse, determine whether an old read revision is stale.

## R2: Public editor mechanics

The existing close owner already covers public `ScriptEditor.close_file`, exact document association, independently dirty state, same-text version changes, a complete bounded set of potentially visited remaining documents, protected-source validation and real close-triggered validation completions. Its API return is not closure proof. Its ownership rules retain entered calls/callbacks during cancellation or teardown. Reuse those mechanisms, not a public `close_script` call, a second operation slot, or a preliminary close/read/delete workflow.

For an open target, the selected delete owner removes only the admitted clean document as the **first explicit deletion effect**, waits for the existing bounded attributable continuation, and independently verifies the phase-authorized roster subtraction and preserved remaining work. It retains the original Script while doing so. Closed targets skip this effect; no target is opened. The attempt never recomputes a closed read revision to replace the admitted open revision.

For present R, call public `Resource.set_path("")` on the exact retained, unchanged, clean Script, with no takeover. The [pinned Resource implementation](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/core/io/resource.cpp) removes its old cache association when the path is cleared. Independently require the retained object's path to be empty and successful passive cache getters to agree there is no target-path Resource. The object can remain alive with unchanged source through another reference. Do not destroy it, clear its source, reload it, or claim scene/reference repair. Only after these guards may the original D be unlinked. A failure between these effects is a partially applied deletion, not a clean refusal.

For absent R, use the established agreement between `ResourceLoader.has_cached` and `get_cached_ref`; no forced load/unload. Creation likewise begins with positively absent B and R. After file creation, R may still be absent or may have appeared through necessary ordinary engine handling. In the latter case, independently check its actual identity, path, source, edited state and supported context. Do not adopt a pre-existing R, force-load for verification, or equate unavailable evidence with absence.

The existing source/effect profile remains decisive: standalone inert GDScript, no tool/custom Script, global registration, load/preload or unsafe compiled/effective contexts. Existing Save-format compatibility checks remain relevant to created source surviving a later ordinary Save. No native parser, ResourceSaver path, `set_source_code` on an existing Resource, or forced reload is needed for creation.

## Owned stock-editor mechanics experiment

The disposable experiment ran in the existing dedicated Tart guest, through its existing local control transport, on official Godot `4.7.2.stable.official.ed1daf0bf` (`ed1daf0bf001b61586d9930840f2f1394092c079`), macOS `26.6.2` / `25G83`, arm64, four vCPUs and 6 GiB. Engine SHA-256 was `c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf`; export-template SHA-256 was `88df5e2e6fee99088699be66e6d42e4da4fb0c5619d054297d755a49558a4792`. It used synthetic private files, a throwaway EditorPlugin and a narrow POSIX helper, not production create/delete code.

The completed run recorded 23 observations and four occupied-leaf controls. Its editor exited `0` with no failed assertions. Independent editor getters and filesystem reads established:

- Clean open deletion: actual document closure, unchanged retained Script source with empty Resource path, no target cache association, and D absent.
- Closed cached-R deletion: no document opening; retained R detached without destruction; D and target cache association absent.
- Closed absent-R deletion: D removed without allocating/loading a target Script or opening B.
- Exact Unicode and empty-file creation: D matched the intended complete bytes; B stayed absent and R initially stayed absent. Occupied regular-file/directory/link controls did not overwrite the occupying object or outside sentinel.
- A later ordinary scan left deleted paths absent, could load exact newly created R while B stayed absent, and generated ordinary `.gd.uid` files. Same-path recreation followed by ordinary opening produced a different Script identity and exact D/R/B, not reuse of the detached old Script.
- A separately launched fresh runtime loaded the created and recreated scripts with the expected synthetic constant value `41`, loaded the empty script with empty source, and observed both unrecreated deleted paths absent; it exited `0`.

Whole-study editor launch-to-exit time was about 9.699 seconds. **This is not a create/delete latency result**: the experiment did not include authenticated MCP execution, trusted pre/post validation, supervision, all adverse guards or consumed client output. It also did not establish real delete Undo, a full unrelated-history campaign, exports or both-client compatibility.

The first attempt is retained as failed evidence: startup scanning had already loaded the supposed absent-R fixture, and screenshot finalization waited past the owned deadline. The corrected run created that synthetic absent-R fixture after startup and used guest window capture. The failed attempt was not promoted to a pass. The completed run's before/after host-process sample was unchanged; a separate unowned host process observed during the first attempt was inspected and left untouched. Sampling is not a global process-absence proof. Research launched its owned editors/runtime only in the guest.

Private records retain the completed/failed runs and probe-source provenance outside Git. Completed `observations.json` SHA-256 is `51ee4eb56ab9b5dc11119f74313e5b64a6d6faf6070a3de94358f3030a0bf53d`; summary SHA-256 is `7447a42b0921733d0530a7503645c66c15e2408b9f11c2b53a43458c8e6390dd`. Owned executable probes, projects and sentinels were removed; the guest was stopped. No production file, permanent test or dependency was added by the experiment.

## R3: Trusted execution and complete-source validation

The current MCP handler recognizes discover/read/edit; its strict decoder, static catalog, transport clock/cancellation path and output projection are the existing extension points. `runner/read` owns fresh captures; `ScriptReadResult::revision` and its current constructors already bind open or closed eligibility to target/session/source/identity/lifecycle. Delete reuses that capture and commitment unchanged, then constructs a deletion-specific checked basis. It cannot use discovery or a caller-supplied native basis instead.

Creation cannot call existing-file read/preflight with an invented empty file or zero inode. The current validator closure requires a disk root; add an explicit **proposed absent root** capture branch and a create-preflight purpose. Bind supplied source to authenticated project/parent absence, selected effective warning/ClassDB/source-only context, request and deadline. Stage only in the existing private validator project; the selected live target must remain absent through preflight. Existing contexts, dependency restrictions, per-URI validation fences, resource/time limits and owned-child cleanup remain mandatory. Post-create validation uses independently captured actual D and fresh applicable native context, not only the supplied source or preflight verdict.

Use the existing absent-R closed-context acquisition primitives to obtain effective/source-profile facts without constructing a live Script or CodeEdit. Add a create-specific context record rather than fabricating existing `OpeningContext` document/file facts. Extract only shared private acquisition helpers; preserve existing encodings/purposes and public read semantics. Delete validates the admitted current/protected context as required by its inherited eligibility and close-preservation path; it does not parse an invented empty replacement or promise project-wide validity after removal.

Both new operations use the existing one-call clock, shared editor slot, one-shot native authorization, source-free error mapping and sticky effect reduction. A timeout or rejected output after dispatch must not use the adapter's pre-dispatch `not_applied` default. Extend every mutating-operation classification, including framing/EOF/cancel/host-failure paths, not just handler dispatch.

Select private bridge **v7** and native API revision **5**, with two additional authenticated capability fields, for the incompatible internal families. Migrate all current private producers/consumers together; refuse mismatched peers with no v6/revision-4 fallback. Public Schema **2**, `sr1`, MCP **2025-11-25**, existing local contracts and package version remain independent and unchanged. Catalog refresh/reconnection discovers the two new tools; it is not a public schema migration.

## Alternatives, costs and remaining evidence

The [plan's complexity review](plan.md#complexity-tracking) records current need, rejected simpler alternatives and ongoing cost. No generic CRUD/transaction framework, approval mechanism, lock service, new dependency, hosted runner or endpoint-isolation campaign is selected. The established [capability-delta readiness decision](../002-edit-open-gdscript/research.md#199-capability-delta-readiness-review-2026-09-29) remains the validator threat boundary.

No unresolved material product or mechanism question prevents this bounded plan. Guarded integration, hostile/interrupted transitions, both actual clients, exact delivered timing and privacy/export durability remain required **implementation acceptance**, not passed research results. If implementation demonstrates an inability to satisfy a required positive profile, correct the design explicitly; do not narrow the feature or ship universal refusal.

Primary API references: [Resource](https://docs.godotengine.org/en/4.7/classes/class_resource.html), [ResourceLoader](https://docs.godotengine.org/en/4.7/classes/class_resourceloader.html), [ScriptEditor](https://docs.godotengine.org/en/4.7/classes/class_scripteditor.html), [EditorFileSystem](https://docs.godotengine.org/en/4.7/classes/class_editorfilesystem.html), Apple's [open](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/open.2.html) and [unlink](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/unlink.2.html) semantics. Exact generated API/runtime evidence and existing accepted guards, not documentation alone, govern implementation.
