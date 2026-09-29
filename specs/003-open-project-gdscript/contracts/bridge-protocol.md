# Private Editor Bridge — Version 3

**Status:** Planned coordinated cutover, not implemented protocol. Extends the completed [v2 boundary](../../002-edit-open-gdscript/contracts/bridge-protocol.md) for one opening operation. Public observation/edit v1 stay unchanged; opening has its own [caller v1](open-api.md).

## 1. Bootstrap, authentication and migration

Keep the same owner-private registry, descriptor ownership/mode/type/ACL checks, per-editor/plugin-lifetime session and 256-bit secret, canonical project identity, IPv4 loopback listener, discovery bounds and source-free mutual HMAC. No second listener, credentials, daemon, permissions or approval mechanism.

Descriptor and handshake `v` become `3`. Reject older/newer private versions; no negotiation or fallback. The capability record contains exactly the five existing observation booleans, `edit_open_gdscript`, then `open_gdscript`. Retain `native_api_revision` and `native_build_id`, now revision `0` (unavailable) or `2` (matched native family). An unmatched/incomplete family advertises the affected operation false; never infer opening from observation or editing capability. The installed shared artifact is `editor_integration`, not a second native library beside `script_edit`.

With `F(x) = u32_be(byte_length(x)) || x`, retain role-separated `HMAC-SHA256(K, F(role) || T)`. V3's exact transcript is F applied separately to: UTF-8 `godot-agent-kit/editor-bridge/v3`; request ID; decoded session ID; advertised project root; Godot version; engine hash; each of the seven one-byte booleans in order (`observe_gdscript`, `open_enumeration`, `buffer_attribution`, `unsaved_paths`, `cached_resource_lookup`, `edit_open_gdscript`, `open_gdscript`); four-byte big-endian native revision; UTF-8 native build ID; decoded client nonce; decoded server nonce. Hello/challenge/authenticate/finish order and role separation remain unchanged. Do not authenticate JSON serialization or reuse v2 proofs.

Update every Rust/addon/worker/fixture transcript and descriptor consumer together. Rebuild/install the matched revision-2 bundle, remove obsolete kit-owned artifact paths, restart the editor with the updated addon, and collect a fresh session descriptor. Public observation schema does not change, but earlier session-bound edit bases are not portable to this lifetime. Current native/operator docs and export checks must migrate; archived historical acceptance remains historical.

No source/hash/context capture precedes unique authenticated selection. An unresolved second candidate remains ambiguity, and an ended explicit session never falls back to another.

## 2. Framing and ownership

Preserve four-byte big-endian length-prefixed UTF-8 JSON, exact tuple arity/types, strict typed response decoding, finite/integral numeric checks, canonical decimal-string IDs/counters and depth 32. Reject oversized lengths before allocating bodies. Preserve standards-compliant JSON escaping; do not normalize source bytes.

- Descriptor and pre-selection/control frames: 4 KiB.
- Selected source-bearing requests: 4 MiB ceiling; each captured source ≤512 KiB.
- Responses and private worker frames: 12 MiB ceiling, with per-field/collection bounds.
- Existing network processing budget: incremental 64 KiB/1 ms per editor frame. This does not bound synchronous native compilation/opening duration.
- Existing operation slot: exactly one collection, edit or opening owner. Internal collector requests for that owner are permitted; overlapping external work receives a terminal `busy` refusal with no queued/resumable effect.

The connection/request/session owns the attempt; no additional transaction UUID, queue or replay service. Resource/document references and native entered-state ownership survive caller cancellation until the entered call returns. Disabling/disconnecting cannot free handles or admit new effectful work while an entered stage can still act. Every later stage rechecks expiry/ownership and cannot revive a discarded request.

All opening tuples begin `[3, opcode, request_id, session_id, advertised_project_root, script_path, ...]`. Every field must agree with the authenticated connection and immutable request. Trailing arguments below are exact and scalar. Remaining budget is an integer 1–9000 milliseconds, bounded by the caller's actual remaining operation time; local native expiry never extends parent time. IDs/hashes/byte lengths use existing checked formats.

## 3. Opening exchange

### Inspect and recognize without lifecycle authority

`open_begin` appends `remaining_budget_ms`. It reserves the common slot and uses the ordinary read-only collector plus passive native inspection to obtain unique target association, open/cache mode and Resource-edited evidence. Hold the exact cached Script reference if present; do not load, instantiate, select, assign source/path or validate current source.

`open_state` returns the common envelope (`v`, `kind`, request/session/project/script and actual editor collection stamp), `status`, `reason`, `sample`, `cache`, `resource_edited`, `expiry_tick_us`. Status is `inspected` or `refused`; cache is `absent`, `present` with actual GDScript identity/hash/length, or `unavailable` with reason. An inability to inspect is never cache absence. Wrong type or duplicate association refuses. Native instance IDs and evidence are not an application receipt.

Rust reads D independently and applies the existing per-source semantics. If the target is already open, `open_verify`/`open_recheck` with purpose `recognition` obtain/recheck its actual state without any authorization, current-source helper, focus change or new-open source-profile requirement. The core can return already-open even with limited/dirty/divergent source observations, but never with unknown exact open identity. Finish releases only the held references/slot.

### Prepare a closed target

`open_prepare` appends, in order:

1. expected project directory device and inode;
2. expected disk file device and inode;
3. expected cache mode `absent` or `present`;
4. expected cached Script ID, empty only for `absent`;
5. captured D SHA-256 and decimal UTF-8 byte length;
6. exact captured D text.

This is internal worker-to-addon captured disk data, not source accepted by the public caller. The integration independently pins a read-only file, checks exact bytes/identity/namespace, confirms the same still-closed/cache state and performs source/current-context admission. It freezes actual current source/version/dirty/compiled/context witnesses without changing them. Cache loss/replacement or an intervening open refuses rather than silently switching branches. A fresh request may subsequently recognize the human-opened document.

`open_prepared` returns the common envelope, `status: prepared|refused`, `reason`, `mode: cold|cached|null`, target/file/native guard summaries, latest `sample`, `resource_edited`, `context`, and local expiry. No effectful stage can start from this response alone.

`context` is either a proven no-source current editor, an admitted current GDScript record, or unavailable with reason. A current record carries exact private path/Script/editor/buffer IDs, independently obtained R/B hashes/lengths/witnesses, one copy of their equal source, current/saved versions and dirty/Resource-edited state, passive compiled metadata, effective configuration/warning/global-name evidence and its canonical SHA-256 fingerprint. Source equality is checked on actual bytes natively; one private source body avoids duplicate transport copies, not an invented R read. Nothing from this private context is serialized as requested target source.

The fingerprint covers a **guard projection**, not collection clocks, receive times, helper status, progress or the copied source body. The projection contains request/session/project/current-document identities; source hash/length; current/saved versions, dirty/Resource-edited/history availability; compiled tool/base/property/method facts; effective external-editor/warning settings; global/autoload names and relevant ClassDB bindings. Set-like lists are sorted, unique and bounded. Current identity/source/version/configuration changes invalidate it; a fresh acquisition timestamp alone does not. Rust recomputes the received projection's fingerprint before using it to bind helper evidence, and native code compares freshly acquired guard values before effects.

Use `SHA256(F("godot-agent-kit/open-context/v1") || E(projection))`, reusing the existing length-prefix primitive rather than adding a canonical-JSON dependency. E encodes null as byte `n`, boolean as `b` plus byte 0/1, string as `s || F(UTF-8)`, and a bounded nonnegative integer as `u || u64_be(value)`. Arrays encode `a || u32_be(count)` followed by E of each value; closed typed objects encode `o || u32_be(count)` followed by `F(key) || E(value)` in UTF-8 byte-sorted key order. No floats, negative numbers, duplicate keys or unknown fields enter this record. IDs/version counters remain their declared decimal strings. This private encoding only binds the current guard and needs cross-language vectors alongside v3 authentication; it is not a general serialization API. Missing required facts refuse, rather than being encoded as an apparently valid empty context.

### Supervisor authorization and native stages

The parent supervisor, not its killable worker, owns the existing disposable stock validator. For a current source it validates the prepared private bytes/context as purpose `open_context`, retaining request/session/current-path/hash/context and completion interval. Valid attributable completion is mandatory. For proven no-source context no helper runs. Invalid/unavailable validation cannot authorize effects. All children share the original deadline and owned process-group cleanup.

After fresh guards and validated evidence, the parent records **possible application before writing** exactly one private `authorize_open` control. It carries only the current-source/context fingerprints (both empty for proven no-source), not arbitrary source or a supplied final outcome. The worker cannot send an effectful stage before this control.

`open_advance` appends `stage`, `validated_current_sha256`, `validated_context_sha256`. Stage is exactly `bind`, `compile` or `open` and must equal the native attempt's next stage. Fingerprints must match the immutable admitted context and rechecked current state; the native boundary does not treat an arbitrary digest as an editor observation. The existing trusted supervisor owns the actual validation decision. No helper verdict is exposed as an agent-facing permission token.

- `bind`: retain the matching cached R, or create/initialize/path-bind a new native GDScript through the exact methods in the native contract. Cache publication is a known effect; cached reuse alone is not.
- `compile`: only for the newly created Resource; retained R skips it. A completed target parse error can proceed. Unknown or non-parse native failure stops and retains partial facts.
- `open`: invoke native opening on that exact Script after fresh guards. Capture actual resulting association/selection; do not confuse void return with success.

Each call returns one `open_progress` envelope with `status`, `reason`, reached/next stage, monotonic binding/compilation/document facts, applicable actual identities, Resource-edited flag and selection relation. It never repeats accumulated source snapshots. Unexpected, duplicate, expired or out-of-order stages terminate without another lifecycle call. No read of a receipt can certify the final D/R/B postcondition.

### Independent verification and termination

`open_verify` appends purpose `recognition` or `post_open`; the former is legal only on the no-effect branch and the latter after opening effects. `open_sample` carries a fresh ordinary collector `sample`, separate fresh `resource_edited`, protection recheck status and selection relation. Current/unrelated source is not returned again merely for verification. Rust independently reads D.

`open_recheck` appends the same purpose and uses the retained original collection identity, fresh current/reference/context/saved-state inspection and native file guards. Its `open_rechecked` response carries actual detected changes/unavailable reasons. Rust separately rechecks D/namespace. Preserve every known invalidation; later equality cannot erase it. The core alone reduces the terminal outcome.

`open_finish` and best-effort `open_abort` have no trailing arguments. The terminal response identifies whether this owned attempt is irrevocably discarded and includes earlier known effects; it does not close a document, clear a flag, Save or undo cache/source state. An abort sent before the deadline is not itself proof it was processed. Do not delay the terminal caller result waiting for cleanup acknowledgment.

## 4. Observation/edit preservation and privacy

Existing `observe`/`recheck` and edit tuples move to version 3 without changing their public contracts, operations, native save/history semantics or independent evidence requirements. The new private validator purpose is not accepted as edit validation. An opening snapshot is never silently converted into an edit basis. Existing calls still do not open a closed target.

Per-frame receive time is shared by all nested evidence; do not create artificial receive intervals per field. Native/editor/caller clock domains stay explicit. Wrong-target/session/connection facts are discarded as invalid; valid earlier effect facts survive later malformed frames. A killed worker after authorization may leave application unknown or known partial, never inferred not-applied.

Secrets, nonces/proofs, native pointers, captured target/current source, context hashes, raw compiler output and unrelated paths never enter descriptors or incidental logs. Only the public requested-target result can disclose its authorized source. Raw helper output is discarded under the inherited validator policy; temporary files/owned children are cleaned up. No arbitrary method dispatch, source evaluation or production fixture control is added.

## 5. Required boundary evidence

Implement cross-language v3 transcript vectors and behavioral version/capability/build/reflection/replay rejection; source suppression before unique selection; strict size/type/arity and stale/out-of-order stage rejection; no effect before authorization; no late application after terminal busy/discard; helper/worker failure before and after authorization; and exact retained-effect reduction under delayed/lost progress. Exercise actual editor integration for cache publication, compilation, opening, cancellation/disable, human/source/namespace changes and independent verification.

The cutover includes all existing observation/edit fixtures, native build/manifest consumers, installed bundle references and enabled/disabled/hook-only export checks. No compatibility shim, new CI trust boundary or additional service is selected. See [quickstart.md](../quickstart.md) for acceptance rather than treating this protocol document as evidence.
