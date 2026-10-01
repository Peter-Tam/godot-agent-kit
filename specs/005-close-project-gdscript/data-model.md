# Data Model: Safe Clean-Document Closing

**Status:** Proposed Phase 1 design for the clarified [specification](spec.md), not implemented behavior. Rust owns checked intent and outcome reduction; the addon/native boundary supplies facts. JSON, sockets and Godot object handles are not core-domain types.

## 1. Reused values and ownership

Reuse existing `RequestId`, `ProjectRoot`, optional `SessionId`, `ResourcePath`, `ResolvedTarget`, `DocumentIdentity`, `Witness`, `CollectionStamp`, `ObservationInterval`, source/dirty availability and invalidation semantics from [observation.rs](../../mcp-server/src/observation.rs). Decimal IDs/counters remain canonical unsigned decimal strings at adapters; they are never lossy JSON numbers. Exact source comparisons precede retained SHA-256/UTF-8-length summaries.

Reuse the existing checked `ExpectedRevisionBasis::from_observation` implementation in [script_edit/request.rs](../../mcp-server/src/script_edit/request.rs) internally. Its conditions are a complete open standalone observation, performed consistency checks, no detected change, independently equal D/R/B, attributed clean dirty state, actual file/Script/editor/buffer identities, no known stale R/B and a current buffer version. It supplies no prior saved version. Do not clone the algorithm, re-export an alias, expose edit application policy through close, or change existing public edit contracts merely to move a stable type. New close callers use `CloseRequest::new(..., Option<&ObservationOutcome>)`; conversion happens once and retained intent contains no source bodies. No `EditAttempt` is reused.

New public core surface: checked `CloseRequest`, `ClosingOutcome` and the real close runner entry. Attempt state, native receipts, source-context records, codecs and protection witnesses remain private or crate-visible to actual consumers. Use the existing meaningful evidence errors; no public speculative variants or generic lifecycle framework.

## 2. Fixed supported profile and bounds

| Item | Bound / rule |
|---|---|
| Operation | One existing project-local external `.gd` in one authenticated editor lifetime; no batches, Save/discard/force/retry/reopen option. |
| Candidate | Exact official Godot 4.7.2 commit/executable and macOS arm64 recorded in [research.md](research.md#3-exact-candidate-and-observed-provenance); public-ABI native integration revision **3**, private bridge **5**. |
| Target locators | Existing project root ≤1024 UTF-8 bytes, resource path ≤2048, request ID ≤64 checked ASCII characters, session ID existing 32 lowercase hex format. |
| Target source | Each D/R/B ≤512 KiB UTF-8; effectful close requires exact LF UTF-8, no BOM/CR/NUL or unsupported control characters. Empty source remains a value. No write permission or Save-format eligibility required. |
| Open-document effect context | At most **8** open standalone GDScript documents including the target; exact unique paired Script/editor/CodeEdit associations. A no-source/empty editor is valid for already-closed recognition. Unsupported mixed/custom/external-editor contexts refuse effectful closing rather than guessing hidden navigation state. |
| Private protection source | Each remaining document ≤512 KiB; sum of retained single R==B source copies across remaining documents ≤512 KiB. The target source is separate. Bound before concatenation/allocation; never silently omit a candidate. |
| Effect metadata | Reuse opening's current source/compiled-property/method/ClassDB/global/autoload/warning bounds; cap the aggregate close non-source context projection at **256 KiB**. Set-like collections are sorted and unique. |
| Caller stdin | Exactly one UTF-8 JSON value plus EOF, ≤12 MiB; strict schema and duplicate/unknown-key checks. `basis` is an observation-v1 result or null, not a new revision-token format. |
| Wire / output | Existing depth 32; 4 KiB handshake/control frames, 4 MiB selected source-bearing requests, 12 MiB response/worker/final-result ceilings. Contexts are separate messages from full target snapshots; the source aggregate above does not increase these bounds. |
| Timing | 9.5-second operation budget from before input/selection plus 0.5-second delivery reserve; addon remaining lease is an integer 1–9000 ms and never extends the caller cutoff. All validator children share this one budget. |
| Diagnostics | At most 64 bounded source-free stage/reason/code records; no unselected paths, source, hashes, credentials or native pointers in public diagnostics. |

The eight-document/source-aggregate limits bound the set the native close may visit through history and list sorting; no public API exposes the complete native navigation stack. This is a conservative supported effect profile, not a requirement to close other tabs or change editor settings. A stable two-document ordinary project is a required useful positive; excessive/unsupported context refuses before closing. Limits are not claims that every maximum-sized input must succeed rather than return a bounded explicit non-success.

Already-closed recognition does not run effect-profile admission, enumerate protection candidates for validation, require target parse validity or apply the multi-document source aggregate to ordinary observation. Its source-local availability behavior remains that of observation v1. Missing/invalid disk targets and unknown open state are not already closed.

## 3. CloseRequest and prior basis

| Field | Meaning / validation |
|---|---|
| `request_id` | Fresh checked correlation, different from a supplied prior observation ID; no replay-cache meaning. |
| `project_root`, `session_id`, `script_path` | Existing explicit selectors; project is mandatory even if a prior result contains it. An omitted session still requires unique authenticated selection; the basis never chooses between candidates. |
| `expected` | Optional checked existing clean `ExpectedRevisionBasis`. Obtained by borrowing/validating the complete prior observation, then retaining only its checked identity/provenance/digest/version. Null grants no effectful authority. |

The CLI payload uses `schema_version: 1`, `request_id`, `basis`. No replacement source, expected-source override, arbitrary source filename, force or validation receipt is accepted.

A non-null malformed/ineligible basis produces an input/basis refusal; it is never silently treated as null. A well-formed basis must match requested and freshly resolved project/session/path and original file identity. Wrong-session/replacement-file evidence refuses even if a different file at that path happens to be closed. For the same valid file/session already confirmed closed, an old open-buffer revision is not applied or certified: return fresh `already_closed_unchanged` with expected evidence marked `not_applied_to_closed_state`. Null is the straightforward no-effect recognition input. If the document is open, null refuses `missing_basis` and any mismatched source/current version/Script/editor/buffer identity refuses before closure.

A prior closed or limited observation cannot become an effectful basis. The caller may instead submit null to recognize a still-closed target; a race that makes it open then refuses. Source equality never authorizes a newer editor/buffer. The native boundary freshly acquires saved version and Resource-edited state; neither is invented in the prior observation schema.

## 4. TargetPreparation

A request-local, immutable preparation contains:

- Authenticated resolved project/session/path, request ID and native build/revision.
- Caller-clock independently read D and existing file identity/namespace witnesses.
- Fresh ordinary target observation, exact paired Script/editor/CodeEdit identity, D/R/B comparison and buffer dirty evidence.
- Separately acquired target Resource-edited flag; both buffer dirty and Resource edited must be observed false for effectful closing.
- Native read-only project/parent/leaf descriptor binding, independently compared D bytes, current/saved versions and target source commitment.
- Selection relation and exact private identity before dispatch, effective settings/profile and the complete bounded protection set below.
- An aggregate immutable preparation guard, original expiry and causal invalidation state.

No target is opened, loaded, selected, saved, compiled, retagged or repaired to construct this evidence. A safe syntax-error target can prepare: closing does not require its parse result to be valid. The same source/effective-context exclusion rules as native opening prevent unsupported tool/export/static/dependency/dynamic-property effects, but no target validation child runs solely to prove close eligibility.

## 5. ProtectionSet and validation receipts

### Capture and purpose

Native closing may visit remaining tabs through history and script-list sorting, and may apply current pending text before scheduling validation. Therefore capture **every remaining supported open script document**, not only the currently selected one or an inferred history-back target. Keep the target separate so its permitted syntax error does not become a remaining-document parse-validity gate.

Each `ProtectedDocument` contains the existing opening-style source-context record: exact project-local path/file association; Script/editor/buffer IDs; independently equal R/B source; source hash/length; current/saved versions; attributed dirty and Resource-edited flags; native undo/redo availability; tool/base/compiled property/method facts; effective source-identifier ClassDB bindings and global/autoload/warning context. Capture selection separately. Dirty R==B is permitted when every guard passes; D may legitimately differ. Dirty R!=B, unknown attribution, effectful/unsupported source or compiled metadata refuses. Do not wait for idle parsing, set R, Save or tag B to make it eligible.

Reuse the existing bounded lexical/compiled effect profile and source-only stock validator described in the [opening native contract](../003-open-project-gdscript/contracts/native-integration.md#2-supported-new-open-profile). Extract only the actually shared context acquisition/profile helpers to a durable private `editor_context` responsibility; opening retains its existing current-document wrapper and public behavior. Closing applies those helpers to its bounded explicit records. No second parser, callback registry or new service.

### Per-document preflight

The existing supervisor owns one one-shot stock validation child at a time for each remaining document, all within the original attempt deadline. Purpose is **`close_context`**, separate from `open_context` and every edit purpose. Require exact captured source, applicable per-URI diagnostic/parser-symbol fences, effective context and owned-child cleanup. Target syntax validity is not requested. No helper is needed when there is no remaining document.

A `CloseContextValidation` receipt binds:

| Field | Meaning |
|---|---|
| `request_id`, `session_id`, `target_path` | The owning close request, not a free-standing parser request. |
| `document_path`, `script_id`, `editor_id`, `buffer_id` | Exact private protected document. Never exposed through public close output. |
| `source_sha256`, `utf8_bytes` | Source actually validated, equal to the native preparation source. |
| `guard_sha256` | The immutable protected-document projection, including identities/versions/profile facts. |
| `context_sha256` | Existing validator effective context commitment, distinct from the native guard. |
| `state`, `collection`, `cleanup` | Actual completed-valid/invalid/unavailable, caller-domain interval/fences and owned-child cleanup state. Only completed-valid, fully attributed, cleaned-up receipts authorize closing. |

Reuse the validator's existing source-only capture/admission/private HOME/XDG/owned-child handling and inherited endpoint limitation. This is a private safety consumer, not a new public validation or execution tool. Another session/document/purpose's result cannot authorize this attempt. Do not serialize a caller-supplied `valid` Boolean as proof.

### Guards and preservation

Before native entry, independently recheck the complete document roster, target identity/source/clean flags, protected identities/R/B/versions/dirty/Resource flags/history availability, effective context and relevant ClassDB/property facts. Compare every receipt and guard. A same-text edit/version change, newly opened tab, closed/reopened document, changed selected identity or configuration change invalidates preparation; do not recapture/revalidate/retry automatically under the old request.

After the authorized close, compare against the prepared roster minus the exact original target editor/buffer; that sole removal and a separately observed permitted native fallback selection are expected effects, not invalidation. Every protected identity remains required. A new/reopened/replacement document or another missing editor invalidates the result. Post-close checking recomputes protected guards and observes target absence/retained state; it does not demand equality with the pre-close aggregate hash whose roster still contained the target.

Metadata/file access remains confined to the selected project. Remaining-document D is not required to equal unsaved R/B and is never used to overwrite them; existing context path/file guards and the stock route's no-write behavior apply. Acceptance independently checks unrelated D as well as actual R/B/history. Source/code references held for preparation are real authorities, not copies used to fabricate observations.

## 6. NativeCloseFacts and continuation ledger

### One native effect

The native entry marks `closing: entered` before invoking the generated-bound public `ScriptEditor.close_file` on the exact freshly rechecked path. There is one authorized call, no method-name input and no event pumping inside it. The engine method hash comes from the pinned executable's generated API. Record actual `Error` return or call failure; `OK` alone does not establish verified closure or preservation.

Native facts preserve request/session/path/build binding, phase, target pre-close identities, entry/return stamps, actual close error, independently observed removal of the old document if known, native selection/target-history effects, sticky invalidation, and irrevocable pre-entry discard. Node IDs are not ownership: validate live Script/editor/buffer association at each access and retain actual Resource references only while required. After closure, release the attempt's target-only Resource references before the ordinary post-close R observation so retention/unloading is not manufactured; keep only attributed IDs/digests/facts, never dereference freed editor/buffer pointers.

Native `phase` is the closed enum `inspected|prepared|entered|returned|settling|settled|terminal` defined in the native contract; core selection/authorization states remain separate. Stage facts may survive terminalization and do not grant authority. On the newly closed branch, a present post-close R must have the original Script identity and unchanged source; a same-path replacement is invalidated evidence, not retained R. Recognition of a document already closed remains a fresh no-effect observation, not certification of old open state.

### Actual native validation completion

Before the effect, the native attempt installs bounded, attempt-owned, non-mutating signal witnesses on the ScriptEditor and the exact protected ScriptEditorBase instances:

- `editor_script_changed` during the synchronous close records which protected documents the native path actually visits. Those visits schedule native validation according to the pinned source. Visits to the soon-disposed target are not surviving validation obligations; its actual destruction must be confirmed.
- Each protected editor's `edited_script_changed` records completion of its real native validation, bound to that editor instance and current attempt. Pre-entry events are baseline only; no prior event or event from a reopened/replacement editor discharges a post-close obligation.
- A visit outside the admitted set, a changed roster/source/version/flag/configuration or unavailable required signal/lifetime attribution is invalidation, not a reason to widen the set after entry.

`CloseContinuation` has `required_editor_ids`, `completed_editor_ids`, callback collection stamps, current state (`not_applicable`, `pending`, `completed`, `unavailable`, `invalidated`) and a reason. IDs are bounded by the protection set. `required_editor_ids` is exact and monotonic for the attempt. `completed_editor_ids` represents only currently satisfied obligations for each editor's latest attributable required visit; it is not historical and is not monotonic. A later attributable visit keeps the editor in `required_editor_ids` but removes it from `completed_editor_ids` until a new matching attributable completion occurs. Historical earlier completion must not satisfy a renewed obligation. After the close has returned, the continuation is complete only when every required editor's latest outstanding visit has a matching attributable completion, with all frozen preservation checks intact. Repeated visits coalesce only for the same unchanged editor/version and actual native idle validation, without reusing an earlier completion for a later visit. A lost/missing callback is unavailable/pending until the deadline, never assumed completed by elapsed time or equal source.

No timer is stopped, forced or triggered; no validation method is called to obtain a witness. Public signal observation does not provide arbitrary callback invocation. Synchronous close/callback entry retains the current native owner until return. Waiting for native completion releases the editor thread, not the attempt's ownership; human editing stays possible and invalidates the old preparation. After an irreversible close, timeout or user interference produces known applied-unverified state rather than undoing or repeating closure. Ordinary editor activity can continue after a timed-out caller; it does not revive product closing authority.

Private `close_status(request_id)` lets the addon inspect this request-local ledger during its existing frame loop. The addon answers one pending `close_wait` request when the ledger reaches completed/not-applicable, invalidates, becomes unavailable or expires; this is event-driven observation of already-scheduled native work, not a retry, convergence loop, new scheduler or public wait tool.

## 7. State transitions, deadline and terminal precedence

| Attempt state | Legal next work / invariant |
|---|---|
| `accepted` | Checked input, original clock; no target source/effects. |
| `selected` | Unique authenticated target/capability; preserve ended-session and denial precedence. |
| `inspected` | Fresh valid-file/open association. Confirmed closed may enter no-effect recognition; open requires matching expected basis. |
| `prepared` | Immutable clean target and bounded complete protection set; no lifecycle effect. |
| `validated` | All required `close_context` receipts match; fresh final guards pass. |
| `authorized` | Parent records possible effects before issuing one-shot authorization; missing reply can no longer imply no effect. |
| `closing` | Native exact-path call entered once; keep entered references/owner, track observed events and effects. |
| `settling` | Native call returned; observe required automatic validation completions and preservation, without issuing another mutation. |
| `verifying` | Fresh ordinary target observation plus independent D and native protection/identity recheck. No copied preparation sample. |
| `terminal` | Immutable result, no new authority or late upgrade. Cleanup only owned handles/signals/processes. |

The recognition path is `inspected -> verifying -> terminal`; it performs no source-context validation, close or continuation wait. Every failure can terminalize with actual application knowledge. Pre-authorization failures are proven not-applied only when no effectful control was issued. After authorization, an attributable irrevocable pre-entry discard can establish no effect; absence of acknowledgment cannot. Known selection/removal effects survive later protocol/timeout errors. Terminal cleanup never changes source, selection or history.

Shared ownership remains one addon slot and one native `Session` owner variant (edit/open/close). Busy requests are irrevocably non-applied and never queued. Cancellation/disable/loss prevents new entry/stages; an entered native call or callback retains ownership/references until it returns. At terminal timeout after return, detach the observation ledger and release owned state without cancelling Godot's ordinary validation or pretending it rolled back. No stale product close can execute afterward; pending ordinary native work is explicit unverified evidence.

| Outcome | Application / condition |
|---|---|
| `verified_newly_closed` | `applied`; original open document removed, fresh current target absent, admitted D/file identity unchanged, retained R independently unchanged or observed unloaded, required native continuation complete, protection and all rechecks intact. |
| `already_closed_unchanged` | `not_applied`; valid exact current file/session is freshly confirmed closed, no effect authorization, no source/selection/history change. Source-local limitations remain explicit. |
| `refused` | `not_applied`; correct safe refusal or interruption with proven no effects/no late product entry. |
| `applied_unverified` | `applied` or `partly_applied`; at least one attributable lifecycle effect occurred but a required condition is missing/invalidated. |
| `effects_unknown` | `unknown`; effects were possible without sufficient attributable application or terminal-discard evidence. |

First causal failure and sticky invalidation are retained. Denied/ambiguous/invalid target binding suppresses unauthorized source-derived fields, but never erases independently known source-free effect knowledge. Complete closure evidence cannot override a later required-evidence failure. A newly reopened target makes final closure unverified even if the old buffer was successfully removed; never close the new one.

## 8. Public ClosingOutcome

All fields below are required; nullable records use explicit null. The caller contract defines exit behavior and examples.

| Field | Shape / interpretation |
|---|---|
| `schema_version`, `operation`, `request_id` | `1`, `close_gdscript`, checked correlation. |
| `requested_target`, `resolved_target` | Existing checked selectors / resolved identity, or null with the appropriate refusal. |
| `interval`, `outcome`, `reason`, `stage`, `application` | Existing caller-clock interval, the closed outcome vocabulary above, a bounded reason, furthest entered stage and independent effect knowledge. |
| `expected` | Null or prior target/identity/source digest/current-version/provenance summary with `use: matched\|mismatched\|not_applied_to_closed_state\|unavailable`; never full prior source or a permission token. |
| `progress` | `closing`, `native_revalidation`, `verification`, each `{state, reason, collection}` with state `not_started\|not_applicable\|entered\|completed\|failed\|unknown`. |
| `before` | Nullable attributable target summary: open state/document IDs, independent source summaries, dirty state, Resource-edited/current/saved-version facts and validity. |
| `observation` | Null or `{purpose: preparation\|recognition\|verification\|survivor, interval, snapshot}`; the snapshot uses existing observation semantics but replaces observed source text with `{sha256, utf8_bytes}`. It retains actual per-authority witnesses/collections, comparisons and invalidation. It is not a reusable observation-v1 edit basis. |
| `resource_state` | `{state: retained\|unloaded\|unavailable\|invalidated\|not_collected, resource_edited, reason, collection}`; the actual latest post-close cache/Resource observation, never inferred from missing B. |
| `protection` | `{status: not_applicable\|preserved\|unavailable\|invalidated, revalidation: not_applicable\|pending\|completed\|unavailable\|invalidated, required_count, completed_count, reason}`. Counts are nullable when not established; no protected paths, source, hashes or receipt bodies. |
| `selection` | Nullable `{before, after, request_effect}`; relations `target\|other\|no_source_editor\|unknown`, effect `none\|native_fallback\|unknown`. Private exact IDs establish preservation; public output does not name another document. |
| `history` | `{participation: not_participated, target_buffer: retained\|disposed\|unavailable\|not_applicable, unrelated: observed\|unavailable\|invalidated\|not_applicable, reason}`. Disposal means actual native target-buffer lifetime, not enumeration of an opaque history stack. |
| `diagnostics`, `safe_next_action` | Bounded source-free records and fixed guidance. Possibly applied results require fresh observation of the explicit original target, not retry/reopen/rollback. |

B and buffer-dirty state are `not_applicable` only for independently confirmed closed state. A present but unreadable B stays unavailable. A genuinely unloaded R keeps observation v1's unavailable/unloaded meaning; `resource_state` explains it without loading. An earlier sample remains earlier/invalidated evidence after possible effects, not fresh post-state. The closing target's parse success is not a required field or success condition; available errors remain source-attributed diagnostics rather than a fabricated clean parse.

## 9. Private interface contract shared by design artifacts

The [bridge](contracts/bridge-protocol.md) and [native contract](contracts/native-integration.md) must preserve these exact semantic interfaces:

- Bridge prefix: `[5, opcode, request_id, session_id, advertised_project_root, script_path, ...]`.
- `close_begin`: remaining-budget-ms; fresh target sample and passive inspect, one shared slot.
- `close_prepare`: captured target source, compact checked native basis, captured file/project identity; immutable target/protection context and guards.
- `close_recheck`: purpose `pre_close|recognition|post_close`; fresh immutable-context/target/file identity checks.
- `close_advance`: aggregate guard plus exact per-document completed-valid receipt bindings; only one close call, no arbitrary stage string.
- `close_wait`: no extra arguments; one pending response for the existing continuation ledger, bounded by the original lease.
- `close_verify`: purpose `recognition|post_close|survivor`; separate ordinary collector acquisition and native postconditions, with independently reacquired Rust D.
- `close_finish` / `close_abort`: no extra arguments; release only this attempt and report actual irreversible discard/effect facts.
- Native revision-3 family: `close_inspect`, `close_prepare`, `close_advance`, `close_status`, `close_verify`, `close_recheck`, `close_finish`, `close_abort`, `close_expire`. Existing session `close()` remains session teardown, not script closing.
- Native prepare takes `(request_id, captured_source, capture_and_basis)`; advance takes `(request_id, guard_sha256, receipt_bindings)`; verify/recheck take `(request_id, purpose)`; other request-local entries take request ID, while inspect takes `(path, correlation)` and expire takes no caller input.

Closed typed context hashing reuses the existing length-prefixed `F` and typed `E` encoding from the [opening bridge contract](../003-open-project-gdscript/contracts/bridge-protocol.md#prepare-a-closed-target), not arbitrary JSON canonicalization. Individual shared source contexts retain the existing guard projection/encoding semantics. The close aggregate uses domain `godot-agent-kit/close-context/v1`, target/request/session/project/file/selection identity, target source/version/clean facts, effective configuration (including actual idle-parse delay), sorted complete document-roster identities and sorted protected-document guard commitments. Exclude source bodies, clocks, receive stamps, helper progress and callback counters. Recompute all hashes independently in Rust; native comparisons use fresh values, never only echoed expected hashes.

The close result, worker stage facts, native receipts, callback ledger and validator receipt are separate records with separate clock domains. No one receipt can substitute for another or assert the public success enum.
