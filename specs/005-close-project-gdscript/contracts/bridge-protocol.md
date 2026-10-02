# Private Editor Bridge — Version 5

**Status:** T001's coordinated v5 migration and revision-3 native/private-fixture boundary are complete, with [accepted task evidence](../quickstart.md#9-t001-native-boundary-acceptance-2026-10-02). T002's complete product close tuple/worker/caller exchange has [passed task acceptance](../quickstart.md#10-t002-public-caller-acceptance-2026-10-02); [T003 cumulative acceptance](../quickstart.md#12-t003-cumulative-acceptance-2026-10-02) completes Feature 005 without changing this protocol. The ninth authenticated `close_gdscript` bit requires the installed complete transport and matched native family. Public observation/edit/open/discovery v1 contracts remain unchanged. The [close caller](close-api.md), [model §9](../data-model.md#9-private-interface-contract-shared-by-design-artifacts) and [native contract](native-integration.md) remain normative.

## 1. Fixed authenticated migration

Reuse the owner-private registry/descriptor ownership, mode/type/ACL checks, canonical project identity, IPv4 loopback listener, per-plugin/editor-lifetime session and 256-bit secret, bounded source-free candidate selection and role-separated mutual HMAC. No second listener, credential, daemon, permission or approval mechanism.

Descriptor, hello, operation and response version become **5**. Reject all other private versions, no negotiation/fallback/dual domain. Exactly nine Boolean capability keys are authenticated in this order:

1. `observe_gdscript`
2. `open_enumeration`
3. `buffer_attribution`
4. `unsaved_paths`
5. `cached_resource_lookup`
6. `edit_open_gdscript`
7. `open_gdscript`
8. `discover_gdscripts`
9. `close_gdscript`

Enumeration is not discovery. Close is not inferred from edit/open/observation or discovery. `native_api_revision` is **0** for unavailable, or **3** for the complete matched native family and validated build ID. Revision 2 is not a compatible revision-3 family. Incomplete/mismatched native installation leaves affected native operations false, never advertises safe close from a method name alone; independent read-only discovery/observation availability retains its actual meaning. Rebuild the matched bundle at the existing `editor_integration` artifact names; no parallel `script_edit` or close-only native library.

Let `F(x) = u32_be(byte_length(x)) || x`. Retain `HMAC-SHA256(K, F(role) || T)` and existing server/client/finish role strings and hello/challenge/authenticate/finish order. **T is F applied separately to each item**, concatenated in order:

- UTF-8 `godot-agent-kit/editor-bridge/v5`;
- request ID; decoded session ID; advertised project root; Godot version; engine hash;
- each of the nine one-byte 0/1 capability values above;
- four-byte big-endian native revision; UTF-8 native build ID;
- decoded client nonce; decoded server nonce.

Authenticate typed fields, not JSON serialization. Reject missing/extra/duplicate/non-Boolean capability keys, wrong native metadata, roles, identity or transcript context. Do not reuse v4 proofs. Nonces/proofs/secret remain private. Unique authenticated selection precedes source/hash/context acquisition; explicit ended sessions never fall back and unresolved candidates remain ambiguous.

## 2. Framing, slot and attempt lifetime

Preserve four-byte big-endian length-prefixed UTF-8 JSON, depth 32, strict known-key/duplicate/type/arity decoding, finite integral numeric fields, canonical unsigned decimal-string IDs/counters/ticks and standards-compliant escaping. Bound lengths before allocation. Control/handshake frames are 4 KiB; selected source-bearing requests 4 MiB; response/worker frames 12 MiB. Target sources remain individually ≤512 KiB. Close admits at most eight exact open standalone Script/editor/CodeEdit documents including target, ≤512 KiB summed single R==B copies for remaining documents, and ≤256 KiB aggregate non-source context metadata. Context messages stay separate from full target snapshots; no framing increase or omitted candidate to fit. Existing incremental network processing (64 KiB/1 ms per editor frame) is retained, not a synchronous native duration guarantee.

The existing addon `_active` slot has one observation/edit/open/discovery/close owner. Native `Session` has one edit/open/close owner variant. Internal ordinary collector stages for the admitted owner are allowed, not overlapping external requests. Busy is terminal non-applied, no queue/resumable effect. Request/session/connection and immutable target own the attempt; no second transaction ID, replay service or generic scheduler.

One parent budget begins before input/selection: 9.5 seconds work plus 0.5 seconds bounded delivery. Begin accepts integer remaining budget 1–9000 ms, bounded by actual parent time. Native/editor expiry is a distinct clock, not interchangeable with caller timestamps, and never extends parent cutoff. No stage, duplicate tuple or waiter renews the lease.

Cancellation, channel/worker loss, disable and expiry prevent new entry/stages. An entered native call or synchronous callback retains exact ownership/references until return; neither cancellation nor Node ID alone permits freeing/dereferencing live state or admitting a conflicting effect. After return, terminal timeout detaches attempt-owned ledger/signals and releases references without cancelling Godot's ordinary pending validation, force-parsing or rolling back. Late ordinary events may continue but no late product close can execute or upgrade a terminal caller outcome. Source/reference acquisition never opens/loads/selects/saves/repairs the target.

## 3. Exact request tuples and stage order

Every close tuple begins `[5, opcode, request_id, session_id, advertised_project_root, script_path, ...]`. All fields agree with authenticated connection and immutable selected target. The suffixes below are **exact**, not an extensible argument bag. Unknown opcode/purpose/keys, wrong owner/binding, duplicate or out-of-order stage, expired/terminal request and incorrect arity stop the offending exchange without invoking another effect or releasing another owner.

| Opcode | Exact suffix | Legal action |
|---|---|---|
| `close_begin` | `remaining_budget_ms` | Claim common slot; ordinary fresh target sample plus passive native inspect. |
| `close_prepare` | `captured_source`, `capture_and_basis` | Freeze immutable target/protection context and source/effect guards; open branch only. |
| `close_recheck` | `purpose` | `pre_close`, `recognition`, `post_close`; fresh target/file/immutable-context checks for the corresponding branch. |
| `close_advance` | `guard_sha256`, `receipt_bindings` | Exactly one authorized native close call; no stage string or method input. |
| `close_wait` | None | One asynchronous response observing existing continuation ledger, bounded by original lease. |
| `close_verify` | `purpose` | `recognition`, `post_close`, `survivor`; separate fresh ordinary collector plus native postconditions. |
| `close_finish` | None | Release only this owned attempt, report discard/effect facts. |
| `close_abort` | None | Best-effort irreversible pre-entry retirement or release after return, preserving known effects. |

Illustrative valid begin tuple, **not an executed request or output**:

```json
[5,"close_begin","example-close-1","00112233445566778899aabbccddeeff","/fixture/project","res://scripts/subject.gd",8000]
```

`capture_and_basis` is a closed private object with exactly `project_device`, `project_inode`, `target_device`, `target_inode`, `source_sha256`, `utf8_bytes`, `basis`. Devices/inodes and basis instance IDs/current version are checked decimal strings; source length is a bounded nonnegative integer and digest lowercase SHA-256 hex. `basis` has exactly `script_instance_id`, `editor_instance_id`, `buffer_instance_id`, `current_version`. This is a compact projection of the checked expected basis, not the public prior observation or a caller-created authority. Request/session/project/path are bound by the prefix. Native preparation freshly acquires saved version and Resource-edited state. Captured source is independently read Rust D, exact bytes equal to admitted D/R/B; native pins read-only project/file descriptors and checks namespace, identity and exact bytes rather than trusting digest equality. No source body is accepted from public close input as replacement text.

The open branch is begin -> prepare -> sequential supervisor validation -> pre-close recheck -> advance -> wait when applicable -> post-close verify/recheck -> finish. Guards are checked again inside native advance. Recognition is begin -> recognition verify/recheck -> finish, with no prepare/validation/authorization/advance/wait. `survivor` collects a still-open/newly reopened target after failure without trying to close it, and cannot recover successful closure authority. All branches may abort/expire with actual effect knowledge. Every stage preserves first causal failure and sticky invalidation; equal later bytes do not reset them.

Native revision-3 fixed family is `close_inspect`, `close_prepare`, `close_advance`, `close_status`, `close_verify`, `close_recheck`, `close_finish`, `close_abort`, `close_expire`. Inspect takes `(path, correlation)`; prepare `(request_id, captured_source, capture_and_basis)`; advance `(request_id, guard_sha256, receipt_bindings)`; verify/recheck `(request_id, purpose)`; remaining request-local entries take request ID, expire no caller input. Session `close()` remains session teardown, not script closing. There is no arbitrary native method dispatch. Exact engine binding/lifetime details belong to the [native contract](native-integration.md).

## 4. Preparation, typed guards and private receipts

Prepare captures the exact clean target, file/project identity, selection, effective configuration and **every remaining supported open script**, not an inferred next tab/current-only set. Target syntax validity is not required and no target validation child is run solely for close eligibility. Unknown roster/association or unsupported mixed/custom/external context refuses. Remaining documents require exact R==B obtained from separate authorities; one retained source copy avoids transport duplication, not an invented R read. Attributable dirty R==B may qualify with unchanged dirty/Resource flags/current/saved versions and usable history. Dirty R!=B cannot qualify by waiting, tagging, assigning R or Save.

Each private source-context record reuses opening's exact closed guard projection and `context` record semantics: `kind`, `reason`, `projection`, `source`, `sha256`, all required with null when unavailable. The projection includes project-local document/file association, request/session identities, Script/editor/buffer IDs, source hash/length, current/saved versions, dirty/Resource-edited flags, undo/redo availability, compiled tool/base/property/method facts, effective external-editor/warning settings, source-identifier ClassDB bindings, global/autoload facts. Effective configuration includes actual idle-parse delay. Per-record profile bounds and source-only validator capture/admission are reused through private shared `editor_context` responsibility, not a second scanner/parser or changed public opening behavior. Set-like lists are sorted, unique and bounded.

Retain opening's closed typed encoding, **not canonical JSON**:

- null: byte `n`;
- Boolean: `b` plus byte 0/1;
- string: `s || F(UTF-8)`;
- bounded nonnegative integer: `u || u64_be(value)`;
- array: `a || u32_be(count)` then E for each item;
- closed object: `o || u32_be(count)` then `F(key) || E(value)` in UTF-8 byte-sorted key order.

No float, negative integer, unknown/duplicate key enters E; declared decimal-string counters stay strings. Shared source contexts retain existing opening projection/domain semantics, not a newly invented hash algorithm. Close aggregate is `SHA256(F("godot-agent-kit/close-context/v1") || E(projection))`. Its closed projection binds target/request/session/project/file/selection identity, target source/current/saved-version/clean facts, effective configuration including idle-parse delay, sorted complete document-roster identities and sorted protected-document guard commitments. Exclude source bodies, clocks/receive stamps, helper status/progress and callback counters. Rust independently recomputes every received projection/aggregate hash; native guards compare fresh actual values and bytes, never only echoed expected digests. A roster/source/version/selection/configuration/identity change invalidates, never silently recaptures a wider set.

The native/fixture aggregate projection has exactly `request_id`, `session_id`,
`project_root`, `project_device`, `project_inode`, `target_path`, `target`,
`selected_script_id`, `selected_editor_id`, `selected_buffer_id`,
`idle_parse_delay_us`, `idle_parse_error_delay_us`, `effective_sha256`, `roster`
and `protected`. The two delay strings commit the actual normal/error settings
under `text_editor/completion`; their maximum governs admission, and both
remain guarded. A smaller error delay is not permission to ignore the normal
timer. `effective_sha256` retains the existing shared effective-context meaning.

`target`/`target_guard` contains exactly `target_device`, `target_inode`,
`script_id`, `editor_id`, `buffer_id`, `source_sha256`, `source_length`,
`version`, `saved_version`, `dirty`, `resource_edited`, `guard_sha256`.
`roster` records contain `path`, `script_id`, `editor_id`, `buffer_id`;
`protected` records add `guard_sha256`. Both lists and the corresponding
source-context `documents` are path-sorted and identity-unique. IDs, lengths,
versions and delay microseconds in these projections are canonical decimal
strings; the two flags are Booleans. The target's guard is its admitted
source/compiled context, not a target parser receipt. Source bodies are held
only in the separate private per-document records.

The internal native fixture uses this projection and strict native facts
without implementing the product close response envelope. Its actual Rust
example consumer performs confined target capture/recheck and whole-set
`close_context` validation under one remaining deadline. Direct closed
decoding applies shared source/metadata budgets before retaining records;
an internally tagged enum must not buffer the entire context first.


The supervisor owns existing one-shot stock source-only validator children, **one at a time** per remaining document, all sharing original deadline and owned cleanup. Purpose is exactly `close_context`, separate from `open_context` and edit purposes. No remaining document means no child. Private completed-valid receipt binds exactly:

`request_id`, `session_id`, `target_path`, `document_path`, `script_id`, `editor_id`, `buffer_id`, `source_sha256`, `utf8_bytes`, `guard_sha256`, `context_sha256`, `state`, `collection`, `cleanup`.

State/collection/cleanup carry actual completed-valid/invalid/unavailable result, caller-domain interval and per-URI diagnostics/parser-symbol fences, plus owned-child cleanup. `guard_sha256` is native immutable protection projection; `context_sha256` is validator effective-context commitment, not interchangeable. Existing source-only private HOME/XDG, inherited endpoint limitation and raw-output disposal apply. Only completed-valid, fully attributable, cleaned-up receipts authorize. A result for another purpose/session/document/source cannot authorize; caller Boolean `valid` is never proof.

`receipt_bindings` is a bounded sorted array (maximum seven, exact admitted protected set, no duplicates) of closed objects containing `request_id`, `session_id`, `target_path`, `document_path`, `script_id`, `editor_id`, `buffer_id`, `source_sha256`, `utf8_bytes`, `guard_sha256`, `context_sha256`. It is the supervisor's compact binding of receipts it actually verified, not their source bodies, public input or a free-form verdict. Native advance compares the immutable admitted identities/commitments and final fresh guards. A missing/extra binding refuses. The full completion/fence/cleanup receipt remains private supervisor evidence; native binding cannot manufacture it.

Before issuing the single private `authorize_close` worker control, the parent records **possible effects**. That closed control carries the aggregate guard and exact verified receipt bindings under immutable request identity. The killable worker cannot issue advance before it. Supervisor/worker/native facts remain separate; no worker event supplies a public success enum. Once authorization could be delivered, missing reply cannot establish no effect. No helper progress or digest is an externally usable permission token.

## 5. Strict response projection

All responses are closed typed objects with exact known keys; nullable fields required, unknown/duplicate/wrong-type fields rejected. Common envelope:

`v`, `kind`, `request_id`, `session_id`, `project_root`, `script_path`, `collection`, `status`, `reason`, `native`, `expiry_tick_us`.

`v` is 5; binding matches prefix/authentication. `collection` is actual editor-domain collection; addon receive stamp is zero and Rust assigns one actual caller receipt to the whole frame. `expiry_tick_us` is nullable canonical decimal editor tick. Busy/unclaimed refusal has no owned expiry/native state and cannot retire another owner. Status is a private acquisition/stage fact, **never a public closing outcome**. Native/editor/caller clocks remain explicit.

`native` is null before actual inspection. Otherwise its exact required keys are `request_id`, `session_id`, `script_path`, `native_build_id`, `native_api_revision`, `phase`, `script_instance_id`, `editor_instance_id`, `buffer_instance_id`, `entry_collection`, `return_collection`, `entered`, `close_error`, `old_document_removed`, `selection`, `target_buffer`, `protection`, `continuation`, `invalidated`, `reason`, `terminal_discard`. Binding/revision/phase and sticky Booleans are actual facts; acquisition-dependent fields are explicitly null when unknown. IDs are canonical decimal strings, not pointers; `close_error` is the actual bounded integral Godot Error code or null, never a substituted success. `old_document_removed` is observed Boolean/null, `target_buffer` is `retained|disposed|unavailable|not_applicable`, and phase follows the native attempt transitions in the [native contract](native-integration.md). Selection/protection/continuation use the closed source-free records below. Entry/return collections are separate actual native-domain stamps. Collection/removal/disposal remains unknown unless observed; `OK` is not closure. This projection reports native facts, not `verified_newly_closed`.

| Kind | Status / additional required fields |
|---|---|
| `close_state` | `inspected\|refused`; nullable fresh `sample`, `resource_edited`, `resource_state`, `selection`. |
| `close_prepared` | `prepared\|refused`; nullable `sample`, `target_guard`, `context`, `guard_sha256`, `validation`. |
| `close_rechecked` | `rechecked\|refused`; `purpose`, ordinary `recheck`, nullable native `protection`, `selection`, `resource_state`. |
| `close_progress` | `returned\|refused\|unavailable`; no repeated source/context/sample. |
| `close_waited` | `completed\|invalidated\|unavailable\|expired`; `continuation`, nullable `protection`; no source/sample. |
| `close_sample` | `observed\|unavailable\|refused`; `purpose`, nullable fresh `sample`, `resource_edited`, `resource_state`, `protection`, `selection`. |
| `close_finished`, `close_aborted` | `released\|refused`; Boolean `terminal_discard`, retaining native effect projection when attributable. |

`sample` uses ordinary target observation fields, never another document's source. `resource_edited` is separately obtained Boolean/null; `resource_state` has exactly `state`, `resource_edited`, `reason`, `collection`, distinguishing actual retained/unloaded/unavailable/invalidated/not-collected state. `selection` has exactly `before`, `after`, `request_effect`, with caller-contract source-free relation/effect values. `target_guard` is the typed admitted target/file projection, not a public basis. `context` is a closed aggregate object with exactly `projection`, `documents`; projection is §4's aggregate and documents are sorted private source-context records. `validation` is a bounded private list of captured per-document existing validator executable/warnings/global-class inputs, each bound to `document_path`, `script_id`, `editor_id`, `buffer_id`, never caller-selected paths/executables. `protection` has exactly `status`, `revalidation`, `required_count`, `completed_count`, `reason`, using the caller-contract vocabularies/count semantics; no repeated private source in verification. `continuation` has exactly `state`, `reason`, `required_editor_ids`, `completed_editor_ids`, `collections`; sets and stamps are bounded by admitted protected editors. States are `not_applicable|pending|completed|unavailable|invalidated`; no arbitrary callback data. Native continuation IDs stay private and are projected to counts in public protection.

Every response must agree with legal stage order and monotonic facts. Native failure must not erase ordinary collector-detected changes. Wrong binding cannot add target/source evidence; earlier valid source-free application facts survive malformed later frames. A response's hash is not trusted merely because the message is authenticated.

## 6. Native continuation versus independent verification

Before effect, native installs bounded attempt-owned non-mutating witnesses on ScriptEditor and exact protected ScriptEditorBase instances. Actual `editor_script_changed` during synchronous close records visited admitted editors. Their actual post-entry `edited_script_changed` events record real native validation completion on exact editor/version and attempt; pre-entry events are baseline only. Visits to the disposed target are not surviving obligations but actual target disposal must be established. An out-of-set visit/replacement/lost signal attribution invalidates, never expands admission. A later visit invalidates earlier completion until new attributable completion; repeated visits coalesce only for unchanged editor/version and actual native idle validation.

Advance marks native closing entered **before** one generated-bound exact-path `ScriptEditor.close_file` call, without pumping events. Record actual Error/call failure, known selection/removal and return facts. Do not force validation, stop/start timers, call a validation method, explicitly reload/reparse/rescan, or infer completion from delay/equal source. Required completion set must cover every surviving visited protected editor with frozen preservation intact.

There is **at most one `close_wait` per attempt and one pending asynchronous response**. It has no timeout/lease extension arguments. Addon observes native `close_status(request_id)` in its existing frame loop and answers when ledger is completed/not-applicable, invalidated, unavailable or original lease expires. If already terminal, respond once immediately with actual terminal facts. Duplicate wait/stage requests are protocol failures, do not attach more waiters, replay advance or renew ownership. Pending state is internal; no intermediate response polling, queue, fixed sleep or new scheduler. Parent cutoff can terminalize first; cleanup acknowledgment cannot delay delivery. Waiting releases the editor thread, not the owner's authority/lifetime; human input can invalidate. An entered callback retains native owner until return even when cancellation occurs.

Completion is only **one required native fact**, not verified closing. `close_verify` separately obtains a fresh ordinary target snapshot and native postconditions; Rust independently reacquires D/namespace/file identity. Check old-document removal, fresh target absence, unchanged D, independent retained R or observed unloading, full protected roster/source/versions/dirty/Resource/history/effective context and selection. Release attempt target-only Resource references before post-close R observation so retained/unloaded state is not manufactured; never access freed target editor/buffer. Retain only attributable IDs/digests/facts. Already-closed recognition also uses fresh ordinary collection without effect-context validation. Unknown open state cannot make B not applicable.

Post-close roster comparison allows exactly the original target editor/buffer removal and the separately observed native fallback for a selected target. All protected identities must remain; a new/reopened/replacement document invalidates the attempt. Do not compare a post-close aggregate against the untransformed pre-close roster hash. Recheck protected commitments and actual target absence independently. A present retained R on the newly closed branch must retain the original Script identity and source; a replacement R with equal bytes is still invalidation. Already-closed recognition does not certify an older open identity as applied.

## 7. Termination, effect truth and privacy

Finish/abort release only owned descriptors/references/signals/processes, never source, selection/history, Save/discard, reopen/undo or editor shutdown. Irrevocable pre-entry discard must be attributable to native state or the exact addon claimed-but-unentered slot; merely sending abort does not prove processing. Known effects remain after release. Before authorization, no effectful control gives proven not-applied; afterward missing acknowledgment means possible effects unless actual discard is established. Known removal/selection/disposal yields applied-unverified on missing required evidence; without application/discard proof use effects-unknown. Core alone reduces [public outcomes/exits](close-api.md#4-outcomes-and-exact-process-exits), preserving first causal failure and sticky invalidation. Complete closure evidence cannot defeat later required-evidence failure. Reopened/newer state is preserved, never reclosed.

Expiry/loss/disable prevent future product stage entry; ordinary Godot validation already scheduled may still run and is reported unverified when not witnessed. Late events cannot revive a request, clear invalidation or upgrade final output. No automatic reconnect, revalidation/recapture, retry, force, rollback or queued closing. New request IDs are correlation, not idempotency keys across reopening/session replacement.

No source/hash/context access before unique authentication. Target summaries are authorized only for selected target; private protected path/source/hash/receipts and raw compiler output never escape public results, descriptors or incidental logs. Native pointers, secret/proof/nonce, helper paths and worker frames remain private. Reuse existing local project confinement, rooted read-only descriptors, owned-child cleanup and export isolation. No new dependency/service, permission/approval boundary, telemetry, arbitrary code evaluation or CI topology.

## 8. Coordinated implementation consumers and required evidence

The future cutover must migrate all affected **current** consumers together; these are boundary obligations, not an implementation task list:

- Rust `bridge.rs`, strict descriptors/handshake/capability/transcript codecs and every existing observation/edit/open/discovery tuple/envelope, selected-capability checks, worker IPC and same-executable supervisor controls; add close's typed private worker facts and authorization handling using existing framing/deadline conventions.
- Addon `bridge.gd`, plugin wiring, all existing edit/open/discovery/observation transports/owners and the one shared slot; install fixed close transport and native revision-3 family/build/export checks. Existing operations move private version only, not public schema/behavior.
- Native manifest/build/install/ABI consumers for `editor_integration`, complete revision-3 reflection/family checks and lifetime ownership, maintaining existing artifact names and removing obsolete kit-owned paths. Revision-2 bundles cannot advertise revision 3.
- Rust routing/confinement/caller/bridge fixtures, observation harness descriptor/HMAC/premature-operation assertions, fixture-driver cross-language vectors, existing edit/open/discovery callers and private fixture controls that encode/check v4/revision 2; all shared owner/worker-loss/delayed-response evidence.
- Current installation/reproduction notices in existing feature contracts/quickstarts, native operator guidance, README/status/build binary lists and shared CI fixture/campaign inputs where the new caller/version/family actually affects them. Preserve historical v2/v3/v4 acceptance as historical. No new provider/runner topology.

Rebuild callers/library and matched native bundle, install addon/bundle together, deliberately restart editors and collect fresh v5 session descriptors. Prior session-bound edit/close bases do not transfer. Stale private peers refuse, never select another session silently or fall back to older ABI. No compatibility aliases/listener/transcript domains remain. Public existing operations retain their deadlines, source semantics, save/history behavior and no implicit close.

Future boundary evidence includes cross-language v5 HMAC and F/E guard vectors; rejection of v4/wrong capabilities/native metadata/replay/reflection; strict bounds/arity/types/stage/purpose/receipt binding; no private capture before selection; all same-slot busy interactions; duplicate advance/wait and no lease renewal; source aggregate/metadata limits before allocation; sequential helper completion/cleanup and purpose separation; pre/post-authorization worker/channel loss and actual discard/effect reduction; cancellation/disable/expiry during entered call and pending completion; late/reopened-editor events and immutable terminal results; independent verification failure despite native OK/completion; source sentinels and enabled/disabled/hook-only production exports. Actual native visits/completion and protected source/history/selection must be exercised in the real editor, including useful stable two-document positives, not mock echoes or a return-only test. Final implementation requires full affected existing observe/edit/open/discovery suites and [closing cumulative acceptance](../quickstart.md) on unchanged runtime inputs. This document is not that evidence.
