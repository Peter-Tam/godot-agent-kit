# Data Model: Trusted MCP Script Workflow

**Status:** Planning contract, not implemented. [Specification](spec.md), [research decisions](research.md) and [interface](contracts/mcp-interface.md) govern interpretation.

## 1. Ownership and existing records

MCP owns protocol messages, DTOs, connection state and delivery. Rust operation code owns checked intent, deadlines, evidence reduction and outcomes. The addon/native boundary owns live editor observations, source effects and immediate guards. No MCP identifier, JSON type or SDK cancellation token enters core/native semantics.

Reuse these existing definitions rather than redefine their fields:

- [Observation v1](../001-observe-gdscript-state/contracts/observation-api.md#3-result-and-process-behavior) and [observation entities](../001-observe-gdscript-state/data-model.md): target/document identity, D/R/B source observations, collections, dirty attribution, comparisons, consistency, diagnostics, interval and selection feedback.
- [Discovery v1](../004-discover-project-gdscript/contracts/discovery-api.md): scoped entries, complete/limited/interrupted/refused inventory, exact coverage and currentness.
- [Open-edit v1](../002-edit-open-gdscript/contracts/edit-api.md) and its checked `ExpectedRevisionBasis`, validation, application/progress, evidence summaries, persistence, native history and safe next action.

Existing local schemas/contracts remain unchanged. Checked internal captures/expected-state records and public read projections are distinct: the agent receives useful source/state and an opaque revision, not those internal records for later resubmission.

## 2. Supported profile and bounds

| Item | Rule |
| --- | --- |
| Environment | Exact supported official Godot 4.7.2/macOS 26.6.2 arm64 profile; no wider support implied. |
| Selectors | Existing project root ≤1024 UTF-8 bytes; exact resource locator ≤2048 bytes; explicit session, when supplied, is 32 lowercase hex. No focus/cwd/root-list targeting or normalization of resource names. |
| Domain correlation | New generated non-secret checked request ID for each intentional call; existing maximum 64 safe ASCII characters. MCP request IDs remain separate. |
| Source | Existing 512 KiB per independent authority and replacement, exact UTF-8; edit replacement admits LF and rejects NUL/CR/BOM without normalization. Read keeps existing empty/unavailable/invalidated semantics. |
| Discovery | Existing bounds: 1024 entries/directories, work 16384, depth 64, batch 64, 6 MiB result. Limits remain explicit coverage, not false absence. |
| Closed edit | Existing external standalone `.gd` source profile; no scene/built-in mutation, tool/custom Script, global registration, load/preload or unsafe effective/compiled context. Both original and desired sources must pass admission. |
| Closed metadata | Bounded scalar file/Resource/epoch evidence; no source duplicate, document-history list or persistent journal. Existing bounded roster getter failure refuses safety admission. |
| MCP input | Complete newline-delimited frame ≤16 MiB; JSON depth ≤64. Inner legacy open-edit payload still ≤12 MiB. Exact JSON representation bounds are checked in addition to decoded source bounds. |
| MCP result | Tool object ≤16 MiB; complete serialized MCP response ≤64 MiB. These retained caps cover any evidenced compatibility fallback, including escaped JSON text; they do not require a duplicate carrier or eager allocation. Public read projection does not duplicate agreeing source texts. |
| Control traffic | At most eight outstanding JSON-RPC IDs per connection and one admitted tool execution; duplicate live IDs are rejected without replacing the original cancellation owner. No operation queue. |
| Time | Read/discover: 5 s end-to-end, with the existing 4.5 s work cutoff; edit: 10 s with 9.5 s work cutoff. Initial handshake and partial-frame completion: 10 s. No idle timeout between complete requests. |

These are existing product or explicit framing/resource bounds, not description/token budgets. Exhaustion before dispatch is a source-free refusal. After possible effects, overflow/serialization/delivery failure must retain actual effect knowledge or report it unavailable—never fabricate not-applied.

## 3. Connection and call ownership

`ConnectionState = AwaitingInitialize | Ready | Closing | Closed`.

A connection stores the configured registry path, selected protocol revision, bounded request-ID ownership and active call control only. It does not store project source history, issued revisions, outcomes for replay or implicit selected projects. Client/server metadata is protocol information, not routing authority.

`CallControl` binds one MCP ID to a new domain request ID, original `AttemptClock`, checked selector intent, an operation-local atomic cancellation flag and its owned supervision handle. One call cannot cancel another. The SDK token is translated at the adapter; cancellation does not destroy the core owner. EOF, delivery loss or process shutdown cancels admission/new stages, drains bounded owned supervision and preserves effect uncertainty. Nothing claims rollback or cancels the user's editor.

## 4. Trusted script read

`ScriptReadResult` is a caller-facing projection of independently acquired Feature 001 observation, supplemented by checked closed-state evidence where applicable. Its fields are:

| Field | Type / meaning |
| --- | --- |
| `source` | Exact currently observed text or null; empty text remains an observed value. Prefer the observed open buffer, then disk, then loaded Script source, with provenance explicit below. This presentation choice does not select mutation authority. |
| `revision` | Opaque `sr1:` plus 64 lowercase hex digits, or null when a safe read precondition cannot be produced. |
| `state` | Target/document identity, lifecycle, authority/source availability and provenance, attributed dirty state, comparisons, stability, invalidation, observation interval, limitations and relevant next action as defined below. |

`state` preserves required observation meaning without exporting the observation-v1 envelope or native expected-state layout:

- `status`: the existing observation outcome classification, including complete, limited, not-open, refused and unavailable distinctions; not-open or dirty does not itself make a successful observation an edit failure.
- `target`: permitted requested/resolved project, script and selected session, plus target kind. No candidate content or authentication/routing internals.
- `document`: observed `open`, `closed` or `unknown` and nullable opaque document identity derived from the existing session/document identity tuple. Do not expose native object IDs merely to encode that identity.
- `source_origin`: `editor_buffer`, `disk`, `loaded_resource` or null. `sources` has those three named authorities; each distinguishes observed/unavailable/not-applicable, current versus invalidated evidence, and whether its text equals `source`. Return a distinct permitted text only when it differs; invalidated text remains explicitly historical, never a current `source` or revision input.
- `dirty`: existing document-attributed buffer and loaded-Resource distinctions; unknown, dirty and not-applicable are not interchangeable. `consistency` preserves pairwise comparisons, performed/unavailable checks and changed/unchanged/unknown stability, with no atomic-snapshot claim.
- `interval`, `diagnostics` and `limitations`: actual acquisition interval and permitted outcome-specific facts, including partial availability and loaded-class-not-reloaded where applicable. `revision_unavailable_reason` is null with a revision, otherwise the relevant eligibility failure. `next_action` retains an actionable safe response rather than a static failure catalog.

Internal `TrustedScriptCapture` retains the complete checked observation and either open eligibility, closed evidence or an unavailable reason. It exists only for the current call. A read issues a revision only when the existing open basis constructor succeeds, or ordinary observation and the private closed supplement pass the selected closed eligibility/rechecks. Dirty/divergent/partial/unsupported reads remain useful with `revision: null`; do not load/open, repair or hide state to issue one. The full interval includes the supplement/rechecks under the original read bound.

### Revision precondition

The protocol-independent read owner computes the revision with existing `ring` SHA-256 over a fixed versioned, domain-separated commitment to the stable safety-relevant state. Use explicit variant/field tags, fixed field order and length-delimited canonical checked values; reuse independently acquired source digests/byte lengths rather than copying source into a second serialization. This is one operation-specific encoding, not a generic token/canonical-JSON framework.

The commitment binds:

- Authenticated project identity, exact canonical script locator/kind, selected editor-session lifetime, file identity, observed lifecycle, independently acquired source witnesses and applicable dirty/availability distinctions.
- Open: every stable expected-state fact used by the existing `ExpectedRevisionBasis`, including actual Script/editor/buffer identity and current buffer version. Same-text editor-version changes and close/reopen identity changes must change the revision.
- Closed: the selected file length/hash/mtime/ctime, confirmed document absence, integration close epoch, and actual Resource absent/present branch with its identity/path/source/edited/profile evidence. Same-text writes, replacement, cache changes and closed→open→closed therefore invalidate it under the existing closed design.

Exclude request IDs, collection stamps, observation timestamps and incidental JSON ordering: separate complete captures of unchanged state must compare equal. Those excluded correlation/interval facts are still validated internally. Unavailable or invalidated required evidence cannot be hashed as if it were a valid state.

The revision is an unkeyed stale-intent precondition, **not authorization, authentication, an idempotency/replay token, a stored basis ID or permission to skip fresh checks**. No signing key, issuer registry, per-client token state or persistent store is needed. It is not consumed; that does not authorize replay or automatic retry. Safety continues to depend on authenticated fresh acquisition and native/core guards even if a caller fabricates a syntactically valid or matching digest.

## 5. Closed state and expected basis

`ClosedStateEvidence` is internal core-validated, request/session/collection-attributed evidence, not an MCP argument or read-output field:

| Field | Definition |
| --- | --- |
| `collection` | Current native collection stamp and exact selected session lifetime; same target as `observation`. |
| `lifecycle` | Literal `closed`, successful complete target-absence observation; unknown is ineligible. |
| `close_epoch` | Canonical unsigned decimal counter owned by the active editor integration session. Every public script-close notification advances it. Reconfiguration, missing registration or overflow invalidates old bases. |
| `file_revision` | Device/inode, length, SHA-256 and nanosecond mtime/ctime from independently acquired confined file metadata. Times are `{seconds: signed decimal string, nanoseconds: integer 0..999999999}`; identifiers/lengths use existing canonical decimal conventions. |
| `resource` | `absent` after successful agreeing cache getters, or `present` with actual stable Script instance/path, source witness, edited=false and supported effective/compiled profile. Getter failure is `unavailable`, never absent. |
| `consistency` | Actual before/after checks, invalidations and observation interval; never `atomic: true`. |

For mutation, `ClosedExpectedBasis` is constructed only from the frozen fresh internal observation and matching closed supplement whose recomputed revision matched the caller's precondition. It binds the expected target/session, disk revision, closed lifecycle/epoch and R applicability. Present R must equal D and be independently clean; known absent R needs no invented source/instance/dirty fact. B has no version, saved version or history record.

The native attempt retains descriptors and any existing Script strongly for its own lifetime. Serialized private object IDs are not retained references. At mutation entry, recapture actual facts and compare them to the frozen expected facts; a public revision is never authority. The private close epoch catches closed→open→closed ABA; currently open state catches a target left open. Unrelated close events conservatively require a fresh closed read rather than a per-target lifecycle registry.

Same-path replacement, same-text file writes with changed ctime, changed cache applicability/identity, Resource-edited state, changed source, session/epoch reset and unavailable evidence all make the old basis unusable. No claim of atomic exclusion of arbitrary filesystem writers, malicious same-UID software or in-editor hostile code is introduced.

## 6. Edit request and branch selection

The public request has only checked project/session/script selectors, `revision` and exact `replacement_source`. It contains no `basis`, prior read object, observation envelope or internal expected-state fields.

Execution freshly authenticates/resolves the requested target, reacquires relevant editor/disk/Resource/lifecycle state and recomputes the revision. Missing/unavailable required evidence refuses; a mismatch yields `revision_mismatch` with `next_action: {kind: "fresh_read"}` and no application. Never reinterpret an old open revision as permission for a newly closed target or vice versa.

Only after equality and current eligibility checks may execution construct the internal `ScriptEditRequest` with fresh correlation and `ExpectedScriptState = Open(existing basis) | Closed(closed basis)`. Freeze the exact matching capture; derive both the expected basis and any private worker observation-shaped input from it, not another later capture. The internal observation has its own request ID, distinct from the edit ID as required by the existing constructor.

- Open mode uses `ExpectedRevisionBasis::from_observation` on that capture and the existing open-only runner/worker.
- Closed mode derives `ClosedExpectedBasis` from the same capture/supplement and uses the selected closed runner/native owner.
- All prepare, pre-effect, native immediate and independent postcondition checks remain mandatory. A race after equality refuses or reports actual effects under the existing semantics; do not silently refresh the expected state or switch branches.
- Fresh acquisition, comparison, validation and mutation share the original edit clock/cancellation owner. No second read/edit budget is started.
- Already-satisfied replacement is `verified_unchanged` only after the same target/lifecycle/revision, safety, validation and applicable-postcondition checks, with no setter/write/mtime/history effect.
- No create/rename/delete/Save/history/force/batch option is added.

## 7. Closed attempt state and outcomes

`Admitted → Prepared → Validated → Authorized → ResourceApplied (present R only) → Persisted → MetadataVerified → IndependentlyVerified → Terminal`.

Pre-effect failure goes to terminal refusal only when non-application is established. Every entered setter/write is marked potentially applied before invocation. Failed or interrupted later stages preserve sticky known/partial/unknown effects; there is no transition back to clean refusal or automatic rollback. Guard failure after an effect stops further mutation and preserves newer work.

The [native contract](contracts/closed-edit.md) defines the exact ordering. Independent verification acquires actual D and any applicable R, namespace/identity/mtime and current document absence. Native receipts are evidence of steps, not the success predicate. Closed results report `history: not_applicable_closed`; open native-history participation remains unchanged.

Public edit result `mode` is `open`, `closed` or `undetermined` before usable basis selection. Its outcome preserves `verified_changed`, `verified_unchanged`, `refused`, `applied_unverified` and `application_unknown`; application preserves `not_applied`, `applied`, `partly_applied`, `unknown`. Existing causal ordering, validation versus availability, disclosure denial and immutable terminal outcomes remain authoritative. `lifecycle` contains admitted and independently observed final state; final unknown is not preserved-state proof.

Closed evidence explicitly distinguishes Resource present/absent/unavailable/invalidated and B not-applicable/unavailable. Its source/identity/validation/persistence facts use the existing source-free edit evidence conventions. Loaded class/runtime state is **not reloaded or verified** by source editing; the result identifies that limitation for a loaded target without pretending old compiled metadata is new-source runtime evidence.

## 8. Tool envelope and failure distinctions

Each public tool object has exactly `schema_version: 1`, `operation`, `request_id`, `result`, `error`. Exactly one of `result` or `error` is non-null. Tool-specific result records are defined in [the MCP contract](contracts/mcp-interface.md#structured-results).

An adapter `error` has `category` (`input`, `admission`, `host`), fixed `code`, fixed `stage`, `application`, nullable permitted requested/resolved target and typed `next_action`. It has no arbitrary exception/request text. A malformed request before dispatch reports not-applied; host failure after possible effects retains actual certainty or unknown. Editor-operation refusals remain actual operation results, not malformed-input errors.

Protocol/JSON-RPC errors remain outside this tool object when the request is not a valid tool invocation. The external request ID is bounded correlation, never target authority or a replay token. `isError` is an MCP summary, not a replacement for outcome/application/evidence. Dirty or divergent successful observations are not edit failures.

## 9. Compatibility and visibility

New MCP schema 1 is independently versioned from legacy local v1, private bridge v6 and native family revision 4. Breaking MCP changes require a deliberate schema/version migration; no alias or dual private implementation is planned. Matched Rust/addon/native peers are installed together, restarted and supplied fresh bases.

Only request/result types and supervised entrypoints needed by the new binary are public Rust APIs. Native RPCs, descriptor handles, authenticated sockets, raw source captures, mutation authorization and concrete state machines remain private/crate-private. Do not expose generic operation traits, registry hooks or speculative variants. Existing open-only public API and caller meanings remain intact.
