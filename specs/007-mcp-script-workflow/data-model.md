# Data Model: Trusted MCP Script Workflow

**Status:** Planning contract, not implemented. [Specification](spec.md), [research decisions](research.md) and [interface](contracts/mcp-interface.md) govern interpretation.

## 1. Ownership and existing records

MCP owns protocol messages, DTOs, connection state and delivery. Rust operation code owns checked intent, deadlines, evidence reduction and outcomes. The addon/native boundary owns live editor observations, source effects and immediate guards. No MCP identifier, JSON type or SDK cancellation token enters core/native semantics.

Reuse these existing definitions rather than redefine their fields:

- [Observation v1](../001-observe-gdscript-state/contracts/observation-api.md#3-result-and-process-behavior) and [observation entities](../001-observe-gdscript-state/data-model.md): target/document identity, D/R/B source observations, collections, dirty attribution, comparisons, consistency, diagnostics, interval and selection feedback.
- [Discovery v1](../004-discover-project-gdscript/contracts/discovery-api.md): scoped entries, complete/limited/interrupted/refused inventory, exact coverage and currentness.
- [Open-edit v1](../002-edit-open-gdscript/contracts/edit-api.md) and its checked `ExpectedRevisionBasis`, validation, application/progress, evidence summaries, persistence, native history and safe next action.

Existing local schemas/contracts remain unchanged. New read/closed domain types and MCP projections are separate contracts with actual current consumers, not replacements for Feature 001/002 APIs.

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
| MCP result | Tool object ≤16 MiB, allowing the existing ≤12 MiB operation record plus its new bounded metadata/envelope; complete serialized MCP response ≤64 MiB. Structured content plus its escaped JSON text uses at most 3× the object plus protocol envelope space. These are caps, not eager allocations or a source-size expansion. |
| Control traffic | At most eight outstanding JSON-RPC IDs per connection and one admitted tool execution; duplicate live IDs are rejected without replacing the original cancellation owner. No operation queue. |
| Time | Read/discover: 5 s end-to-end, with the existing 4.5 s work cutoff; edit: 10 s with 9.5 s work cutoff. Initial handshake and partial-frame completion: 10 s. No idle timeout between complete requests. |

These are existing product or explicit framing/resource bounds, not description/token budgets. Exhaustion before dispatch is a source-free refusal. After possible effects, overflow/serialization/delivery failure must retain actual effect knowledge or report it unavailable—never fabricate not-applied.

## 3. Connection and call ownership

`ConnectionState = AwaitingInitialize | Ready | Closing | Closed`.

A connection stores the configured registry path, selected protocol revision, bounded request-ID ownership and active call control only. It does not store project source history, edit-basis tokens, outcomes for replay or implicit selected projects. Client/server metadata is protocol information, not routing authority.

`CallControl` binds one MCP ID to a new domain request ID, original `AttemptClock`, checked selector intent, an operation-local atomic cancellation flag and its owned supervision handle. One call cannot cancel another. The SDK token is translated at the adapter; cancellation does not destroy the core owner. EOF, delivery loss or process shutdown cancels admission/new stages, drains bounded owned supervision and preserves effect uncertainty. Nothing claims rollback or cancels the user's editor.

## 4. Trusted script read

`ScriptReadResult` contains:

| Field | Type / meaning |
| --- | --- |
| `observation` | Complete existing observation-v1 record; not a filesystem-only substitute. |
| `edit_basis` | Exactly one tagged eligibility record below; no second copy of the observation. |

`edit_basis` variants:

- `{kind: "open"}`: `ExpectedRevisionBasis::from_observation` succeeds on the actual prior open observation. This indicates usable expected facts, not continuing edit permission.
- `{kind: "closed", state: ClosedStateEvidence}`: ordinary observation and independently obtained native closed facts agree on the same target/source/lifetime and observation interval, with successful rechecks.
- `{kind: "unavailable", reason: code}`: no usable edit basis. Dirty/divergent/partial reads remain information; no forced load/open or state repair is attempted merely to produce a basis.

Every read may return source and limitations allowed by Feature 001 even when the native closed capability is missing. Closed B and document dirty state remain not applicable only on confirmed absence. Native-inspection mismatch invalidates the basis and relevant facts rather than silently replacing the earlier observation with newer authorization. The read's full interval includes native supplement/rechecks under the original five-second bound.

## 5. Closed state and expected basis

`ClosedStateEvidence` is core-validated, request/session/collection-attributed evidence:

| Field | Definition |
| --- | --- |
| `collection` | Current native collection stamp and exact selected session lifetime; same target as `observation`. |
| `lifecycle` | Literal `closed`, successful complete target-absence observation; unknown is ineligible. |
| `close_epoch` | Canonical unsigned decimal counter owned by the active editor integration session. Every public script-close notification advances it. Reconfiguration, missing registration or overflow invalidates old bases. |
| `file_revision` | Device/inode, length, SHA-256 and nanosecond mtime/ctime from independently acquired confined file metadata. Times are `{seconds: signed decimal string, nanoseconds: integer 0..999999999}`; identifiers/lengths use existing canonical decimal conventions. |
| `resource` | `absent` after successful agreeing cache getters, or `present` with actual stable Script instance/path, source witness, edited=false and supported effective/compiled profile. Getter failure is `unavailable`, never absent. |
| `consistency` | Actual before/after checks, invalidations and observation interval; never `atomic: true`. |

`ClosedExpectedBasis` is constructed only by revalidating this record together with the entire prior observation. It binds the expected target/session, disk revision, closed lifecycle/epoch and R applicability. Present R must equal D and be independently clean; known absent R needs no invented source/instance/dirty fact. B has no version, saved version or history record.

The native attempt retains descriptors and any existing Script strongly for its own lifetime. Serialized object IDs are not retained references. At mutation entry, recapture actual facts and compare them to expected facts; a syntactically valid caller record is never authority. The private close epoch catches closed→open→closed ABA; currently open state catches a target left open. Unrelated close events conservatively require a fresh closed read rather than a per-target lifecycle registry.

Same-path replacement, same-text file writes with changed ctime, changed cache applicability/identity, Resource-edited state, changed source, session/epoch reset and unavailable evidence all make the old basis unusable. No claim of atomic exclusion of arbitrary filesystem writers, malicious same-UID software or in-editor hostile code is introduced.

## 6. Edit request and branch selection

`ScriptEditRequest` has checked project/session/script selectors, fresh correlation, exact replacement and `ExpectedScriptState = Open(existing basis) | Closed(closed basis)`.

The adapter input `basis` is the **entire preceding successful read tool object**, passed unchanged; it is not an opaque token or a second operation. Envelope validation stays at MCP. Core reconstructs checked prior evidence from its `ScriptReadResult`, then validates the explicitly requested target against that evidence.

- Open mode dispatches the existing open-only runner/worker and its guarantees.
- Closed mode dispatches the new closed runner/native owner.
- Missing/ineligible/wrong-target bases refuse; an observed lifecycle change never selects the other branch automatically.
- Already-satisfied replacement is `verified_unchanged` only after the same target/lifecycle/basis, safety, source validation and applicable-postcondition checks, with no setter/write/mtime/history effect.
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
