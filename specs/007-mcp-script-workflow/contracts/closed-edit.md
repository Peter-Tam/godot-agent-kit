# Closed-Script Native Transaction Contract

**Status:** Selected implementation design supported by [research](../research.md#2-closed-editing-selected-route-and-limitations), not a completed product capability. Feature 002's open-only path remains unchanged. Only the new supervised closed-source request may invoke this private native owner.

## 1. Admission and authorities

Require the exact supported native/editor family, authenticated session/project lifetime, one eligible existing standalone external GDScript path and a checked [closed expected basis](../data-model.md#5-closed-state-and-expected-basis). The integration runs on Godot's main thread under the existing shared operation slot.

R means loaded Script **source**, as in the specification. The transaction never promises a live bytecode/static-variable refresh or running-game hot reload. It neither calls reload/update_exports nor replaces existing live instances. Source validation uses the existing isolated exact-source stock validator; private compiled/effective-profile inspection is admission evidence, not a claim that desired class code is already running.

Two admitted Resource states exist:

1. **Present:** agreeing passive cache getters return the same supported GDScript at the exact resource path; actual source equals independently captured D; actual Resource edited state is false; current source/effective/compiled context is supported. Retain this object strongly for the attempt.
2. **Absent:** both passive getters successfully agree that the canonical target Resource is absent. No target Resource is loaded, allocated, published into cache or retained merely to manufacture evidence.

Unavailable, inconsistent, wrong-type/path, dirty/divergent or unsupported R refuses. Missing B is not enough: positively observe the target absent from the complete supported ScriptEditor roster, with no attributed unsaved target. Unknown or duplicate associations refuse. A target open at any entry guard requires a fresh read, not implicit close or a switch to Feature 002.

## 2. Read witness and lifecycle revision

Expose a read-only native `closed_inspect` that returns exact request/session/collection attribution, target absence, cache state and actual R identity/source/edited/profile observations, plus the configured-session close epoch. It acquires no mutation slot persistently and performs no write/load/open/validation. Busy or incomplete evidence means unavailable basis, not an invented clean target.

Attach one owned callback to public `ScriptEditor.script_close`. Advance the configured-session epoch on every such notification, even for an unrelated script. A closed→open→closed cycle therefore invalidates a previous closed basis without maintaining a generic per-target history store. Current roster checks independently catch a target that remains open. Native reconfiguration must advance/invalidate this epoch or require a fresh session, never silently reuse its old zero value; overflow makes closed basis unavailable. Disconnect the callback on disable/exit after entered calls safely unwind.

Rust independently acquires D and file revision, then cross-checks native observation/rechecks in one read interval. The epoch is a stale-state witness, not authority to mutate and not a client-written version counter. Complete pre/post observations are still non-atomic with respect to arbitrary external actors.

## 3. Preparation and validation

`closed_prepare` consumes trusted checked intent: original expected target/revision, observed Resource branch, exact intended source and the original deadline. It must:

- Claim the existing editor operation slot for this request and retained owner; reject overlap rather than queue.
- Pin existing project/parent/leaf descriptors with current no-follow, namespace, owner and regular-file rules. Open the existing target with write access only after profile/target validation; do not create a file. Retain exact inode, original bytes, mtime/ctime and requested source within existing bounds.
- Check current session, project/file identity/attachment, exact D and revision, close epoch, actual target absence and any retained R/path/source/edited state. Cache absent/present must match the read basis, not a newly convenient branch.
- Admit only the existing safe standalone LF UTF-8 source/effect profile for both original and desired source, adapted to a closed retained Script without inventing a CodeEdit. Read applicable editor Save-format settings directly and require exact intended text to survive later ordinary opening/Save; do not format it or change preferences.
- Capture the relevant effective/compiled context for the current source-only validator through existing confined mechanisms. No target load, scene execution, broad project copy, native parser fallback or live reload.

Rust's supervised stock validator performs exact intended-source preflight with request/session/target/context binding. Invalid source, unsupported effects, unavailable context or expired budget refuses before effects. Before authorization, recheck original captured state and validation context; changed state is not silently revalidated as a new request.

Already-equal intent performs current guards, validation and independent readback with zero setter/write/timestamp/history effects. A prior success or equal source hash alone cannot establish `verified_unchanged`.

## 4. One native source mutation

The core marks possible application before one authorized `closed_apply`. Native repeats all immediate guards on the main thread and holds the entered-call owner across the sequence. No event pumping, UI navigation or deferred product mutation is inserted inside it.

1. For present R only, mark potential Resource effect and call **explicit `Script.set_source_code`** on the exact retained Script. Re-read source/identity, cache association and edited state. The `_script_source` property/reload path is forbidden.
2. Recheck namespace, original D/revision, target absence/epoch and expected R state before persistence. If a newer change is visible, stop; never overwrite it merely because the setter already ran.
3. Mark potential disk effect. Use the retained target fd for bounded `pwrite` loop, `ftruncate`, `fsync` and independent `pread`; verify exact intended bytes and current namespace attachment. No pathname writer/ResourceSaver fallback.
4. Restore original mtime on **that same fd** with atime preserved, then independently fstat/readback and recheck attachment/source. Failed metadata restoration is applied-unverified, not a clean no-effect failure. Keep new ctime as revision evidence; do not forge original ctime.
5. Recheck present R/source/path/identity/edited=false or continued confirmed absence, close epoch and no target document. Do not clear an unexpected edited flag, tag a nonexistent buffer, emit a synthetic saved signal or repair changed state.

The absent-R branch skips only the setter. It remains a Godot-authoritative transaction with current native absence/epoch/namespace guards and verification, not a disk-only fallback. If a Resource appears, stop with truthful effects; do not load, overwrite, evict or select it as the original authority.

No CodeEdit operation/history entry exists while closed. No Save, ResourceSaver, editor-wide apply, reparse/rescan, reload, opening or closing is part of the mutation. Those ordinary actions may later witness durability, not manufacture success.

## 5. Independent verification

A returned native receipt states attempted/completed steps and observed failures; it is not a success verdict. The Rust worker independently reacquires D through confined descriptors, obtains a fresh editor/native target/cache/epoch/namespace sample and validates actual resulting source/context through the existing isolated helper under the original cutoff.

Success requires all applicable conditions together:

- Exact intended persisted D, same admitted file/project/session identity, successful persistence/mtime proof and fresh namespace attachment.
- Continued target document absence and unchanged close epoch; B and buffer history/dirty state are explicitly not applicable.
- For present R: same original retained Script/cache association, actual intended R source, edited=false and no unsupported profile/context change. For absent R: successful fresh absence evidence, never a failed getter interpreted as absent.
- Successful source validation tied to actual independently observed resulting source and context, not only the proposed request bytes.
- No detected newer-work, lifecycle, source, identity, permission or disclosure invalidation. Independent acquisition intervals/limitations remain visible; no atomicity claim.

Release attempt-only retained R references at completion; do not use their artificial retention to infer future cache residency. The result describes actually observed source/applicability at verification. Subsequent reads observe current reality afresh.

## 6. Cancellation, partial effects and refusal

Before authorization or a proven native entry refusal, discard authority so late messages cannot execute the old request. An entered setter/write, native channel loss, cancellation or deadline after possible entry is not a rollback. Track known Resource changes, written bytes, truncate/flush/readback and metadata failure separately; final reduction preserves actual/partial/unknown application.

Expiry/cancel/disable prevents new stages but retains an entered synchronous owner's memory until return. Never kill the user's editor. On changed/newer source or opened document after effects, stop without reasserting old source, closing the new document or switching branches. A terminal failed request cannot be resumed or replayed; a new intentional edit requires a fresh read/basis.

Causal failure precedence does not relax disclosure. Later denied/scope-invalid evidence suppresses prohibited before/after source summaries and private diagnostics while retaining source-free effect facts. Unexpected host/native failures are typed unavailable/unverified outcomes, not panics or fabricated success.

## 7. Required verification boundary

Implementation must independently prove cached/absent positives, dirty/equal-text-dirty or divergent loaded R refusals, missing getters, same-text file revisions, namespace replacement, lifecycle/epoch ABA and boundary races, cache-branch changes, lost/partial writes, failed mtime restoration, newer human work, interruption and no late effects after proven refusal. Include unchanged intent, Unicode/empty and bound cases, unrelated dirty document/history preservation, actual later open/Save/reparse/rescan/fresh runtime durability, privacy and exports.

Source-established direct-setter behavior and the scoped mechanics probe do not replace these guards or new positive product acceptance. The loaded class implementation is not hot-reloaded: report that limitation truthfully rather than use stale compiled metadata as new-source proof. If the supported route cannot satisfy the specification, stop dependent implementation and return to the product decision; do not weaken postconditions or ship universal refusal.
