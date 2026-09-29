# Private Editor Bridge — Version 2

**Status:** T004 completed the coordinated v2 caller/worker/addon cutover on the stock §18 T1 / §19 L1 native/helper boundary with [real-editor caller acceptance](../quickstart.md#13-t004-caller-acceptance-2026-09-29). Observation's public schema stays v1, while all private peers use v2. There is no patched-editor requirement, private-mtime fabrication, v1 mutation fallback or editor-clock parser evidence. T005's cumulative feature gates remain pending.

## 1. Preserve bootstrap and source-free authentication

Retain owner-private registry bootstrap, random per-editor/plugin-lifetime session and 256-bit secret, private descriptor ownership/mode/type/ACL checks, canonical project directory identity, IPv4 loopback transport, bounded peer/discovery counts, source-free mutual HMAC authentication, and existing denial/ambiguity/liveness semantics. No new credential store, approval system, listener, port convention, daemon or external service.

Descriptor `v` becomes `2`; its other existing fields retain their meanings. A new addon lifetime replaces the old descriptor under the same guarded lifecycle rules. Both caller/worker and addon reject v1 rather than negotiate a mutation fallback. Observation's public outcome stays v1; its private traffic uses v2 after cutover.

Handshake tuple shapes/order remain hello → challenge → authenticate → hello/finish, using version `2`. Challenge/finish retain existing exact identity/nonces/proof fields, with these deliberate capability changes:

- `capabilities` contains the existing five booleans in the same order, then `edit_open_gdscript`.
- Add `native_api_revision`, integer `0` or `1`, and `native_build_id`, empty when unavailable or a 64-lowercase-hex identifier from the matched engine/native build manifest.
- Edit is advertised only when the exact matched stock native build supplies the required API family, independent saved-state inspection and guarded writer/finalizer. The caller separately obtains stock validation and fresh context/dependency evidence per attempt; unavailable validation cannot authorize success. Missing or unmatched native tooling advertises edit false.

Keep `F(x) = u32_be(byte_length(x)) || x` and role-separated `HMAC-SHA256(K, F(role) || T)` from v1. For v2, T contains exactly: UTF-8 `godot-agent-kit/editor-bridge/v2`; request ID; decoded session ID; advertised project root, Godot version and engine hash; the six one-byte booleans in the order above; four-byte big-endian `native_api_revision`; UTF-8 `native_build_id`; decoded client nonce; decoded server nonce. Apply F to **each** field, including each boolean and the revision bytes. Do not authenticate JSON serialization or reuse v1 proofs/domain. Implement fresh cross-language vectors and changed-capability/build/version/reflection/replay cases.

Native build identity is produced by implementation build provenance and matched by the integration; it is not a project-source read, endpoint echo, dynamic native-library download or claim of already-tested compatibility. The exact official engine and extension binaries must also be recorded in acceptance evidence.

No source or source hash is sent/read before successful unique selection. Source-free capability/authentication failure stays distinct from dirty/source failure. An unresolved second candidate is not discarded merely because another answered; an ended exact session never falls back to another editor.

## 2. Framing, bounds and strict shapes

Retain four-byte big-endian length + UTF-8 JSON, scalar fixed-position control tuples, exact arity/type validation and strict Rust response decoding. No Variant/resource/object deserialization or executable strings.

| Boundary | v2 rule |
|---|---|
| Descriptor | Existing 4 KiB; minimal operational metadata only. |
| Before authentication/selection | Existing 4 KiB frame limit; source-bearing preparation is impossible. |
| Selected peer request | 4 MiB ceiling to carry escaped ≤512 KiB replacement source; each opcode has its smaller structural bound where applicable. Read the prefix and reject oversized length before allocation. |
| Responses / private worker frames | Existing 12 MiB ceiling; typed bounded fields/collections, maximum nesting 32. |
| Fields | Existing request ID ≤64, project root ≤1024 UTF-8 bytes, resource path ≤2048; IDs/counters as canonical decimal strings; hashes 64 lowercase hex. |
| Active work | Existing one active collection slot admits one read collection or one edit attempt for the selected editor/session. Other edits receive terminal source-free/busy refusal before mutation, never a queued position. |
| Network/main-thread work | Preserve incremental nonblocking framing and existing 64 KiB/1 ms network processing cap per frame. Native parser/I/O calls are separate bounded-stage work and may outlive the caller deadline; never call them network throughput guarantees. |

Observation's per-source availability rules remain unchanged. Oversized mutation source/basis refuses the edit before application, not a truncated change. Root/dependency/diagnostic limits are in Primitive B. Malformed/extra/out-of-order/duplicate control steps terminate the attempt; they never execute a fallback operation. Godot JSON syntax tolerance does not relax semantic, identity or authentication validation.

Control numbers are checked for finite, integral values in their prescribed ranges before integer conversion; Godot's JSON parser represents JSON numbers as floating point. IDs, versions, byte counts and expiry ticks that are specified as decimal strings never pass through that numeric conversion. Every frame has one receiver timestamp shared by its nested evidence. Editor collection intervals describe the actual acquisition, including independent before/after inspections where carried; parsing separate fields must not manufacture separate frame receipt times.

Source-bearing responses use JSON-compliant escaping for every admitted control
character. The pinned engine's nonstandard vertical-tab escape and raw C0 bytes
are repaired at the shared serialization boundary without changing source text,
literal backslash sequences, representation limits or framing bounds.

## 3. Observation behavior after cutover

`observe` and `recheck` use their existing tuples with version `2`; the same collector, getters, attribution, source limits, invalidation rules and independent Rust D reader remain authoritative. No save/open/validation side effects are added to observation. An old observation's public schema remains usable as an expected edit basis only after fresh target/session/evidence checks.

An internal getter collection belonging to the active edit is permitted within that same attempt's slot; another request cannot reuse it. Keep preparation and verification samples separately attributed rather than overwriting old evidence with new-document values.

## 4. Edit exchange

After unique source-free discovery, compare the selected lifetime/project/engine
binding with the basis before any source acquisition or preparation. The basis
does not act as an implicit session selector. Publish matching selection facts
before capability/preparation can refuse. Existing confined read/access acquisition
precedes editor-state refusal, so denied access cannot be hidden by dirty/stale
state; its first D witness is reused. This edit-only denial policy does not change
observation v1's independent partial-D behavior.

All tuples begin `[2, opcode, request_id, session_id, advertised_project_root, script_path, ...]`. Every identity agrees with the authenticated selected connection and expected basis. There is no operation ID distinct from the existing request ID; `purpose`/phase fields distinguish evidence within the attempt.

### Prepare, without mutation authority

`edit_prepare` appends these scalar arguments in exact order:

1. `remaining_budget_ms` (integer 1–9000);
2. `basis_request_id`;
3. expected canonical project directory `device`, `inode`;
4. expected disk file `device`, `inode`;
5. expected Script, ScriptEditorBase, CodeEdit instance IDs;
6. expected CodeEdit current version from observation v1 (no caller-supplied saved version);
7. expected agreeing source SHA-256 and UTF-8 byte length;
8. exact `replacement_source` string.

Admit `edit_prepare` only through that existing slot. If occupied, permanently reject this attempt as `busy` without storing preparation/apply work or authorizing mutation; late frames on the refused attempt cannot apply after slot release. Bind the rejection to this request/session so the core can prove not-applied, not infer it from silence. An admitted attempt retains the slot through verification/terminal cleanup. A later new request must establish a fresh valid basis; changed revision defeats the old basis even when desired text matches current text, before any unchanged-intent branch.

The addon checks operation capability, source/path representation and all bounds, obtains actual R/B/dirty/open/association and public saved-state facts, and pins/confirms native project/file identity without writing. Changed intent uses a writable existing descriptor only after access checks; unchanged intent skips native preparation and never enters a mutating guard. The Rust supervisor owns stock preflight validation after receiving the preparation evidence. Preparation cannot mutate source, saved state or history, even if the worker dies.

Return `edit_prepared` with common envelope (`v`, `kind`, request/session/project/script, editor collection stamp), `status`, `reason`, `intent: changed|unchanged|null`, `native`, `sample`, `saved_state`, `context` and `expiry_tick_us`. Inapplicable/unavailable records are null. Context contains the official executable path, warning settings/provenance inputs and global-class names, not a parser verdict. Native facts include actual `prepared_current`/`prepared_saved` versions and retained-descriptor identity/T0. Guard the observed saved version until A legitimately tags the new version. The worker independently reads/rechecks D through Rust, and the core validates preparation against expected intent; native D claims never substitute for that witness.

Store one immutable attempt and one local expiry; do not accept a second prepare/replacement source. Refused/expired/disconnected preparation becomes terminal and releases resources. Source-free failures need not contain a sample. An unchanged attempt proceeds only to validation/verification; it cannot accept apply.

### Apply, only after supervisor authorization

The same-binary worker delivers the ordinary bound observation events plus typed saved-state/disk details to the supervisor. The core checks preparation; the supervisor owns the disposable stock validator and cleanup, records qualifying preflight evidence, then requires a fresh application guard and `ready_to_apply()`. Only then does it call `authorize()` and release exactly one `authorize_apply` control. A failed guard is known not dispatched; **possible application is recorded before the control write**, not before the guard. The worker cannot emit `edit_apply` earlier. Helper validation shares the caller's deadline and cancellation; a killable edit worker cannot own an orphanable validator group. This extends existing supervised IPC, not a new service or approval handshake.

`edit_apply` has no trailing source, path override, flags or receipt; it consumes the prepared changed intent on this exact connection, which must still own the integration slot. The addon rejects missing/expired/consumed/wrong-session preparation and rechecks native live target, dirty/revision/source/identity guards and current D immediately before the source/history boundary. Never use the earlier preparation alone to authorize a changed buffer.

Mark boundary entry before the first native operation that could affect source/history. Execute the native steps and local guards from the data model: complex CodeEdit edit → explicit guarded R setter → descriptor-bound persistence → A → fresh-source B. Do not yield/pump events to pretend cancellation is atomic. Reentrant/automatic editor paths must obey the document guard; unknown/replaced state stops further stages.

Each completed native stage emits one bounded `edit_progress` event: `buffer_applied`, `resource_applied`, `content_persisted`, `mtime_restored`, `edited_cleared`, then `saved_tagged`. Events carry native facts and applicable independent before/after getter evidence once. `edit_applied` carries final native facts and an empty `events` list after completion, or at most one terminal partial/refused event. It does not replay accumulated source snapshots or contain a parser verdict. Potential entry is not proven application; synchronous work can delay delivery past the caller deadline. Only the core classifies the outcome after independent verification.

A duplicate apply, regardless of request ID, never re-executes. Terminal prepared state is consumed; a closed connection has no resumption path or durable deduplication service.

### Independent verification and finish

`edit_verify` accepts purpose `preflight`, `post_change` or `unchanged` at its applicable stage. Preparation supplies the initial collection; a fresh preflight verify/recheck supplies the application guard. After `edit_applied`, use `post_change`; unchanged intent uses `unchanged` without application, Save or tagging. Collect actual R/B/dirty/open/identity through the read-only collector and public saved-state facts through the separate inspection API. `edit_sample` contains `sample`, `saved_state` and `context`, never a finalizer snapshot substituted for independent observation.

Rust reads D independently, then sends `edit_recheck` for the same purpose. The integration rechecks the retained original collector and freshly inspects saved state/context; Rust independently rechecks D. The supervisor obtains one stock parse of the captured proposed source for preflight, or the actual resulting source for post-change/unchanged validation. Fresh context/dependency checks qualify that result, followed by the final source/saved-state verification. Unavailable checks and every detected change remain explicit; a replacement-session/source sample cannot clear earlier invalidation. Only the core determines the terminal outcome.

`edit_finish` consumes/releases the attempt after result reduction. An `edit_abort` may be issued best-effort on cancellation. Either has no trailing arguments; neither undoes source or bookkeeping. A pre-boundary abort acknowledgment proves not applied only if it says the context is irrevocably terminal and apply cannot subsequently run. After boundary entry, preserve actual/unknown stages. Do not wait beyond the caller deadline for abort/finish delivery; disconnect cleanup still releases owned handles when the editor regains control.

Caller timeout, disconnect or an abort/finish request alone cannot release the active slot while an already-entered native stage can still mutate. Stop subsequent stages, retain live handles until entered work returns, and prevent deferred effects before admitting another attempt. An overlapping request's proven busy refusal does not downgrade the active request's `may_apply`/known effects or resolve its uncertainty. No new lock, queue or native concurrency policy is introduced.

## 5. Failure, late frames and privacy

The supervisor never infers not applied from an empty progress history after it released authorization. Timeout/EOF/worker failure then yields application unknown, or applied-unverified if valid earlier events prove effects. A complete attributable pre-boundary rejection is different: the consumed attempt can no longer apply, so refusal is truthful. Cancelling/killing the worker cannot revoke a mutating call already accepted by Godot.

Native/local expiry, request/session/object mismatch, reentrancy, protocol failure or known new work forbids starting the next effectful stage. Do not close the user's editor, automatically Save, restore a file, retarget, retry, or reconnect to finish an uncertain edit. Caller outcomes are immutable; late frames are discarded rather than promoted to success. Retain earlier validated evidence with original attribution/interval and explicit invalidation/unknowns.

The raw secret, nonces/proofs, native handles, replacement source, observed source, diagnostic source snippets and unrelated dirty paths stay out of descriptors, logs and routine progress messages. Requested results may carry only the selected target and confined dependency diagnostics needed by the contract. Native receipt-shaped input, arbitrary operation names/object methods, absolute write destinations and client callbacks are never accepted.

## 6. Required implementation migration and checks

Update every Rust/addon/worker codec, capability transcript and fixture peer together; replace v1 private fixtures rather than retain unsafe compatibility aliases. Keep all observation behavior regression cases. Add behavioral boundary tests for prepare without authorization, worker failure before/after authorization, duplicate/late apply, expiry/disconnect during each stage, changed evidence, malformed limits and cross-session/target frames. These tests must assert consumer-visible state/outcome, not merely echo copied fields.

Re-run existing bootstrap/routing/privacy/export cases with the installed official-stock native artifact, not a patched editor. Export presets and addon export guard must exclude the `.gdextension`, native binaries and all tooling/fixture resources from actual production exports; runtime launch of the exported fixture must expose no listener, native tool registration or credentials. No new CI trust boundary/workflow or distribution service is required.
