# Trusted Script Existence Integration

**Status:** Selected Phase 1 design, not implemented. [Research](../research.md) supports the mechanics, not guarded product acceptance. The [data model](../data-model.md) owns evidence and terminal meanings; the [MCP contract](mcp-interface.md) owns public DTOs. This private contract is not another public API.

## Shared boundary and compatibility

Rust owns checked intent, current target selection/basis comparison, stock validation, one-shot authorization, original deadline, independent reacquisition and terminal reduction. The addon owns authenticated fixed-stage forwarding, the existing `_active` slot and response lifetime. Native C++ owns actual Godot objects, descriptor/namespace guards, explicit effects and bounded continuation facts. No layer treats native `OK` as verified success.

Move the existing private bridge from v6 to **v7**, including its authenticated domain, descriptor/request/response checks and proof vectors. The capability record has all ten existing exact boolean fields in their existing order, followed by `create_gdscript` and `delete_gdscript`. All twelve participate in authentication; missing/extra/wrong-type/mismatched fields refuse. No optional-field downgrade or v6 fallback.

Select native API revision **5** with both new operation families. Keep existing bundle/entrypoint/metadata naming and exact pinned engine ABI, generated MethodBind hashes, source/build identity and library provenance. Update native registration, builder/manifest, plugin family validation, addon producers, Rust consumers and fixtures together. An unmatched/incomplete family cannot advertise these capabilities. Observation/discovery can remain available without native mutation support under the existing rules; missing native support cannot turn create/delete into filesystem fallbacks.

Keep existing proof/selected-request/response framing limits and strict closed message objects. No new product socket, port, credential, process service, approval gate or operational registry. Worker dispatch is through existing same-binary private entrypoints. Native expiry uses the remaining original lease, never a renewed ten seconds. Public Schema 2, `sr1`, MCP and existing local-operation contracts are independent of these private versions.

## Fixed operation exchange

Add operation-owned request/response types, not a client-selected generic action. Use the current authenticated bridge envelope/correlation and exact request/session matching. All native entries verify that the configured session and retained owner still belong to that request. New wire kinds are:

| Kind | Checked payload and behavior |
| --- | --- |
| `create_prepare` | Exact path, complete source and independently captured creation basis. Reserve the shared owner, pin parent/absence, acquire native context; no effects. |
| `create_apply` | Request-bound guard digest and completed create-preflight receipt. Consume once; guarded exclusive creation/persistence only. |
| `create_verify` | Request identity only. Fresh native B/R/context/namespace facts; no load, scan, mutation or cached success. |
| `create_finish` / `create_abort` | Request identity only. Terminal cleanup of owned handles/state, with actual entry/discard facts; never delete a partial file. |
| `delete_prepare` | Exact path and independently captured original deletion basis after fresh `sr1` comparison. Reserve owner; retain target and bounded protected set; no lifecycle effect. |
| `delete_advance` | Guard digest, completed applicable validation receipts and one expected phase: `document`, `resource` or `file`. Only the next applicable phase can consume it, once. Not arbitrary method dispatch. |
| `delete_wait` | Request identity only. One held response until the current close-continuation facts settle or deadline/invalidations win; no effect entry. |
| `delete_verify` | Request identity only. Fresh original-target/absence/retained-Resource/protection facts and actual ledger; no mutation. |
| `delete_finish` / `delete_abort` | Request identity only. Terminal cleanup and actual discard/effect facts; no reopen, reassociation, restore, repeated unlink or Save. |

Responses pair the request kind with bounded facts or a typed source-free refusal; they cannot contain a public success boolean. Wrong phase, duplicate authorization, wrong request/session, expired owner or unknown field cannot progress an effect. A terminal request cannot restart. A request-bound native status getter and existing per-frame expiry support the delete wait; neither schedules a future mutation. Finishing or aborting another operation is impossible through these entries.

Private preparation/fact records have these explicitly separated roles:

- **Common binding:** request/session identity, exact project/path, original expiry, canonical project/parent identity chain and guard digest. File/device/inode/epoch/version values use existing canonical decimal encodings. No zero ID substitutes for absence.
- **Create preparation:** checked absent leaf, complete target-document/cache absence and close epoch, requested source digest/length, Save-format and source/effective context projection. The actual source is sent once where needed, bounded independently from its digest.
- **Delete preparation:** original file revision/source commitment, original open/cached/absent branch, actual retained Script/editor/buffer identities and versions/dirty state, complete protected roster/selection and per-document context/guard commitments. Only preparation/validation-private messages may contain protected source.
- **Validation receipts:** exact purpose, request/session/path, root origin, source digest/length, parent/file and native guard/context commitment, completed-valid per-URI fence and owned-child cleanup evidence. A caller-provided `valid` flag or matching hash alone is insufficient. Empty protected sets are explicitly checked, not missing evidence.
- **Progress:** operation phase, applicable effect entries/completions, actual return/failure facts and preservation/continuation invalidations. Parent possible-entry state is separately retained, so loss of a native receipt cannot erase uncertainty.
- **Fresh verification:** new collection interval and actual authority/context/preservation facts; comparisons to original/new-file lifetime and permitted phase transitions. The Rust collector separately acquires D or positive namespace absence.

Use existing typed tagged/length-delimited commitment helpers with distinct `godot-agent-kit/create-context/v1` and `godot-agent-kit/delete-context/v1` domains. Bind purpose and expected phase; never let a close/edit receipt authorize delete. Keep unchanged existing operation encodings unchanged. Do not expose private basis/receipt fields as MCP arguments or mint another agent token.

## Creation

### Admission and preflight

1. Authenticate the explicitly selected project/session before source acquisition/disclosure. Admit only an exact supported standalone `.gd` path and complete bounded source. Parent directories must exist and satisfy current no-follow, ownership/permission/ACL and namespace rules. Inspect the exact `.gd.uid` companion under the same boundary if it exists; links/non-regular/denied companions refuse, and no companion write/delete is authorized.
2. Rust independently confirms absent D relative to the checked parent. Native confirms no target document/unsaved association and successful agreeing passive cache absence, plus current close epoch. A pre-existing R/B with missing D is conflicting occupied state, not a new empty target. Duplicate/wrong-type/unavailable associations refuse.
3. Acquire actual editor effective settings, supported source profile and Save-format compatibility without loading, allocating, opening or validating a live target. Share existing passive context acquisition; do not copy source-profile scanners or fabricate `OpeningContext` Script/CodeEdit/file evidence.
4. Run the existing isolated stock validator with a new create-preflight purpose and explicit `ProposedAbsentRoot`. Its private root is supplied intended source, bound to current authenticated project/parent absence and source-only context. Existing confined dependency/profile limits, warning settings, per-URI parser/diagnostic fences, child cleanup and source-free logging remain mandatory. The live target stays absent.
5. Recheck current project/parent/leaf absence, R/B absence/epoch, permissions and effective context after validation. Freeze a checked preparation. Changed state refuses; do not silently capture and validate a different target or renew the budget.

This branch must accept supported empty/Unicode/full-source cases; an inability to validate any absent root is not a completed creation capability. Invalid/unavailable/unsupported preflight leaves no live placeholder or parent directory.

### Exclusive effect and persistence

The Rust owner marks possible application before releasing `create_apply`. Native repeats immediate main-thread guards, without event pumping, then marks file acquisition entered before:

`openat(parent_fd, leaf, O_CREAT | O_EXCL | O_RDWR | O_NOFOLLOW | O_CLOEXEC, 0600)`.

An occupied leaf never becomes overwrite, adoption, truncation, rename-over or equal-source success. An attributable exclusive `openat` failure such as `EEXIST` records `file_creation: failed_no_effect`: the call entered but created nothing. With source persistence not entered and no other effect, the result is `refused/not_applied`. Missing return evidence remains unknown, not a guessed failed acquisition. Once a new file descriptor exists, file creation is an effect even if it contains zero/partial bytes.

Require the returned regular file's identity/permissions and namespace attachment, then perform bounded descriptor writes of the exact intended bytes, `fsync` and independent descriptor readback. No old mtime exists to restore; do not invent one. Check deadline/attachment and any observed invalidation before each subsequent effect step. Replaced leaf/parent, newer source, opened target or conflicting Resource stops further work with truthful effects. Never unlink a partial file, overwrite a replacement, force a reload, or normalize source to rescue the request.

No source setter, ResourceSaver, native source UndoRedo action, explicit scan/`update_file`, UID creation, open/close/Save or forced load/unload is part of create. Ordinary engine handling may establish a new R; that permission never admits a pre-existing conflicting R.

### Independent postconditions

After the native receipt, Rust freshly reacquires D through the current selected project and native obtains a separate actual target/cache/epoch sample. Require the same newly created file lifetime, exact intended bytes, successful persistence evidence and current parent attachment. B must remain absent throughout observed guards; a changed close epoch/observed target appearance invalidates even if later closed again.

Successful passive cache getters must agree on either absent R or one newly associated supported Script at the exact path with independently observed intended source and edited=false. Retain a newly observed object only for attributable verification; do not force-load it, silently replace it, or infer permanent residency after releasing the hold. Unavailable/wrong/divergent association cannot be ignored.

Run fresh actual-source stock validation against the independently reacquired D and applicable current context under the original cutoff. A preflight pass is not this postcondition. Recheck actual file/native/context state after the helper; any sticky invalidation or missing required evidence produces non-success. New R/ordinary UID bookkeeping is reported as applicable state/limitations, not a hot-reload or reference-repair guarantee.

## Deletion

### Original basis and preservation preparation

1. Authenticate and acquire the existing target through `runner/read`'s trusted capture. Compare the submitted non-null `sr1` with the freshly computed existing revision. Missing source/required observations, current revision unavailability or mismatch refuses. Do not synthesize a deletion basis for an otherwise ineligible read or ask discovery for authority.
2. Freeze the admitted profile: clean open D/R/B, clean closed cached R coherent with D, or closed with positively absent R. Retain original read-only file/parent descriptors and actual Script strongly when present. Require current source, source/effective profile, independent dirty state, identities/lifecycle and relevant epoch. No extra target parse-validity rule is invented beyond current read/safety eligibility; no intended replacement source exists.
3. For open deletion, reuse the existing [close protection and continuation contract](../../005-close-project-gdscript/contracts/native-integration.md#4-immutable-target-and-protection-preparation): complete bounded potentially visited remaining roster, actual selection, R/B equality, dirty/Resource flags, current/saved versions, history availability and effective context. Dirty unrelated R==B work may be protected without saving; unknown/divergent/unsupported preservation refuses. Validate captured protected sources in the existing isolated helper with delete-bound receipts; do not bypass actual per-URI completion.
4. Recheck all original/prepared facts and deadline immediately before first effect. A changed target never switches profile; a changed protected set is not expanded automatically. Closed targets require current B absence and unchanged applicable epoch, not a fabricated CodeEdit or a close call.

### Explicit staged effects

**Document phase — open only.** Mark entry before one bound `ScriptEditor.close_file(original_path)` on the admitted clean target. Retain the entered owner and original Script. No forced-discard prompt acceptance, Save or second close. The same-main-thread final guard is essential because this public API is discard-capable. Record actual return separately from observed removal.

Use the existing real `editor_script_changed` / `edited_script_changed` witnesses for all actually visited protected editors. Wait for latest attributable required completions under the original lease; a native return, sleep or fixed frame count cannot settle it. Before advancing, independently verify expected roster equals the prepared roster minus only the original target, no replacement B, preserved protected source/identity/dirty/version/history, phase-appropriate close epoch, and original D/R still unchanged. The selected target may cause ordinary native fallback selection; a non-selected deletion must preserve original selection. Do not repair selection or cancel ordinary human/editor activity.

This is part of the admitted deletion, not a preliminary close followed by a new closed read. The owner and original basis survive the permitted lifetime transition. Additional close/open/roster changes invalidate; do not demand equality to a pre-close aggregate that necessarily included the removed document.

**Resource phase — originally present R only.** With current original file, absent target B, unchanged source/clean original R and complete preservation evidence, mark entry and call public `Resource.set_path("")` on that retained Script, without takeover. Do not clear source, free a referenced object, reload, or operate on a newly cached replacement. Independently verify the retained object's path is empty, source unchanged and passive getters agree no Resource remains at the target path. A return alone is insufficient. An originally absent R must remain positively absent and skips this effect.

**File phase.** Recheck original D identity/source/revision and parent/project attachment, current target-document/cache absence, detached retained R when applicable, protection/continuation state and remaining deadline. Mark possible file removal, then issue one `unlinkat(parent_fd, leaf, 0)`. No unlink by freshly substituted path, retry on missing file, recursive removal, rename-to-trash or `.gd.uid` deletion. A missing/replaced original leaf before authorized unlink is a stale/missing result with whatever prior document/association effects already occurred.

The [research concurrency limit](../research.md#concurrency-boundary) is explicit: the native sequence does not pump events, but the namespace check and unlink are not a filesystem inode-CAS against arbitrary outside writers. Detected replacement always stops or defeats success. No lock/lease/quarantine service is introduced to promise stronger isolation than the supported model.

### Independent disappearance and preservation

Rust performs a fresh authenticated confined lookup: the same valid parent namespace must positively lack the leaf. Native independently reacquires the full target roster/cache association and retained original-object facts. Require no target B and no loaded Resource associated with the deleted path, with original retained R detached when applicable. A new same-path file/Script/document invalidates success, even if source matches. Do not remove it or relabel it inapplicable.

Recheck protected documents/selection/history and real close-continuation completion after all effects. No post-deletion empty-source parse or project-wide validation is claimed. Report ordinary UID sidecars or retained references as limitations; they do not authorize dependency repair or imply that dependents still parse/run. Physical destruction of retained R is not required.

The original script is absent only when all applicable independent authorities say so. File disappearance alone, a failed cache getter or missing inventory entry cannot yield `verified_deleted`.

## Cancellation, failure and lifetime

Keep the common one-shot/cancellation rules across every stage. An attributable pre-entry refusal/discard or completed primitive with proved no effect can establish non-application for that step. Without such proof, cancellation, lease expiry, native/worker disconnection, output rejection or lost delivery after possible entry retain known effects or application uncertainty. Closing a document or clearing its path before failed unlink is a real partial deletion; exclusive creation before failed write is a real partial creation. No exception/default DTO may erase these facts.

Cancel/disable/session loss stops new stages and invalidates unused authorizations. Retain entered synchronous call/callback storage and object references until return; do not kill the user's editor or claim an already-entered syscall/ordinary Godot continuation was rolled back. Terminal late messages cannot mutate or upgrade a delivered result. Cleanup releases only owned handles, signal registrations and private validator artifacts, never target files, replacement documents or human history.

Maintain strict disclosure throughout: private original/protected/requested source and validation staging stay out of incidental logs/errors. A later denied namespace suppresses prohibited evidence without suppressing known source-free effect facts. New native methods and test-only faults remain subject to the existing editor-only/export boundary and separate normal/fixture artifact provenance.

## Required proof

The [quickstart](../quickstart.md) owns concrete validation groups and coverage. Prove create empty/Unicode/source bounds and no-overwrite occupancy; all three delete positives; same-text/file/Resource/document/lifecycle ABA; dirty/equal-text dirty and missing evidence; every effect boundary/interruption and late-new-work preservation; real close continuation/unrelated history; independent R/B/D disappearance/exactness; later Save/rescan/reparse/reopen/runtime behavior; both actual clients, original delivery bounds, privacy/export and directly affected old callers. Mocks, receipts and this design's mechanics probe cannot replace that proof.
