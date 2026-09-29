# Edit Open GDScript — Caller Contract v1

**Status:** T004 completed this CLI/JSON adapter and coordinated private bridge-v2 cutover against the stock §18 T1 native boundary and §19 L1 helper. [Caller acceptance](../quickstart.md#13-t004-caller-acceptance-2026-09-29) records the real-editor evidence separately from [T003's private acceptance](../quickstart.md#11-t003-native-boundary-acceptance-2026-09-29). The core uses public saved state, retained-descriptor T0 evidence and caller-clock stock validation, not private Resource/document mtimes or patched-parser evidence. T005's cumulative feature gates remain pending.

## 1. One operation and invocation

The `edit-gdscript` binary performs one full-source replacement of one existing, already-open, standalone GDScript:

```sh
edit-gdscript --registry "$REGISTRY" --project "$PROJECT" \
  --session "$SESSION" --script res://scripts/subject.gd
```

It reads exactly one bounded UTF-8 JSON document from stdin, followed by EOF. Stdin avoids source in argv/process listings and avoids granting arbitrary outside-project source-file access. Payload shape is exactly:

- `schema_version`: integer `1`;
- `request_id`: new checked request correlation;
- `basis`: the complete prior observation-v1 outcome for this exact selected project/session/document;
- `replacement_source`: the exact desired whole source string.

This is not an observation replay. Extract [ExpectedRevisionBasis](../data-model.md#expectedrevisionbasis) from the prior result, independently reacquire current evidence, and compare it. A dirty/divergent/limited/stale/invalidated, wrong-target or missing basis refuses without source/history mutation. Do not invent a mutation token from the observation request ID. The public Rust input is typed `EditRequest` plus checked `ExpectedRevisionBasis`; a conversion from the existing observation outcome supplies the same basis fields. Protocol/JSON/process details stay outside the reusable reducer.

`--project`, `--registry`, optional exact `--session`, resource locator and request ID retain existing checked formats. Require the explicit project even when it appears in `basis`. Omitted session requires unique source-free authenticated selection; expected session must then match. Two live sessions are ambiguity, not permission to trust the basis as an implicit selector. Closed, built-in, unsupported or unavailable documents are not opened/loaded/repaired to make them editable. Native editor capability/compatibility must be established before dispatching source-bearing mutation preparation.

There are no path/range patch operations, force switches, retries, Save All, open/close, history commands, execution flags, broader permission grants or fallback writers. Full-source representation keeps one intent and one stale basis; preservation/safety checks remain independent of representation.

## 2. Input bounds and unchanged intent

Reuse lexical project/resource/registry confinement and privacy rules from [observation v1](../../001-observe-gdscript-state/contracts/observation-api.md). Project root ≤1024 UTF-8 bytes, resource path ≤2048; counters/instance IDs are checked decimal strings. Decode schema/field types and reject duplicate or unknown required-shape fields and trailing JSON values. Cap stdin at 12 MiB before unbounded allocation; source size is checked after decoding, ≤512 KiB UTF-8. Selection, blocking input and all worker work are inside the supervised deadline.

The selected initial edit representation is exact LF UTF-8 with no NUL/CR/BOM; explicitly refuse unsupported representations rather than silently normalize. Empty source, Unicode, tabs, whitespace-only changes and final-newline changes are representable inputs, subject to native Save-profile eligibility: both original and desired source must survive current Save formatting unchanged, or the request refuses with `save_would_reformat`. Never change editor preferences to make it pass. Exactly 512 KiB is in bounds; above it refuses the mutation, without changing observation's existing per-surface limit behavior. Backend representability is checked before application and exact equality afterward.

If desired bytes equal current independently agreeing source, take the unchanged branch: validate source and all required current safety/saved-state evidence, independently verify, and return `verified_unchanged`. No complex edit, writer, saved tagging, Resource assignment or history entry. Do not require a writable descriptor for that branch; no write is attempted. A parse error, dirty state or missing required evidence is still non-success, not a no-op loophole.

## 3. Structured result and process behavior

Emit exactly one JSON EditOutcome plus newline on stdout. Fields and enum semantics are in [EditOutcome](../data-model.md#attemptprogress-and-editoutcome). Logs on stderr contain only correlated safe stage/reason metadata, not source, payloads, credentials, endpoints, authentication proofs or unrelated project paths. Target/dependency diagnostics belong only in the requested bounded result. Surface records carry actual source hashes/lengths and availability; full before/after source is not repeated in the edit result.

Once the complete input shape and its checked request ID have been decoded,
semantic refusals retain that correlation. Rejection before decoding—such as
invalid CLI selectors, malformed shape or an input deadline—uses a fresh valid
request ID instead of extracting an unvalidated field. Only checked selectors
appear in `requested_target`; invalid selectors leave it null.

| Exit | Meaning |
|---|---|
| `0` | `verified_changed` or `verified_unchanged`; inspect the explicit outcome to distinguish them. |
| `2` | Known `applied_unverified`, including partial persistence/finalization or failed post-change validation. |
| `3` | Proven `refused` for busy admission, input, target, stale/dirty/conflict, permission or unsupported capability/representation/effect. |
| `4` | `application_unknown`, or a proven pre-application timeout/cancellation/disconnection/protocol failure. Inspect application knowledge and reason; exit alone cannot prove not applied. |
| `1` | Unexpected host/launcher failure preventing normal delivery; structured error if possible, never an invented mutation result. |

The controlled caller must drain stdout. The supervisor collects evidence for at most 9.5 seconds and reserves 0.5 seconds for result delivery; it does not kill the user's editor or wait indefinitely for a stuck native/filesystem call. Post-authorization timeout/loss may mean the edit applied or an entered stage may still finish. [Application-certainty rules](../data-model.md#4-deadline-cancellation-and-application-certainty) are mandatory.

Never retry automatically, even with the same `request_id`. The safe next action after any applied/unknown result is a fresh read-only observation of the original explicitly selected target and inspection of human work. A later intentional edit is a new request/basis, not rollback, resume, or delivery deduplication. This operation is not declared idempotent by source equality: history, persistence and partial failure matter.

An overlap rejected by the selected editor/session's active slot returns `refused` with reason `busy` and `application: not_applied`; it cannot enter mutation, change source/history/finalization, queue or auto-apply later. Observe again and submit a new request for a later edit. If the first edit changed the revision, the previous basis remains stale even when the requested text equals the new source; `verified_unchanged` cannot bypass basis validation. Busy refusal is not evidence that the active request has stopped or rolled back.

## 4. Verification and refusal rules

Core success requires fresh independent D/R/B equality to the intended exact source, target/document/session continuity, attributable clean state, verified same-retained-descriptor T0 restoration/readback for changed intent, actual public saved-state readback, source-attributed post-change exact-source parser/analyzer validity, fresh required dependency/context checks and no known invalidation. An application acknowledgment, successful flush, restored timestamp alone, saved-version tag, Resource getter, silent logger, preflight parse or native receipt cannot substitute for that conjunction.

Human dirty work always wins. Dirty-equal-text is still dirty; unknown dirty state is not clean. Changed same-text buffer version is stale relative to the basis. A closed/replaced document or ended/replaced session is never retargeted. Denied/ambiguous requests reveal no source-derived evidence. An invalid-source proposal can be refused in preflight; if source was already changed before validation becomes invalid/unavailable, retain `applied_unverified` instead.

The operation does not claim arbitrary non-cooperating same-inode writer exclusion, crash atomicity, automatic rollback, compile/runtime correctness, live-debugger reload, or a generally saved scene/Inspector state. Known interference cannot become success merely because later source samples agree. These limits do not waive A–E, native-history, normal Save/reopen/reparse/rescan/runtime-durability acceptance.

## 5. Consumer-visible contract cases

These are behavior cases for implementation tests and live fixtures, not implementation results:

| Scenario | Required result / invariant |
|---|---|
| Clean eligible different source, complete A/B and independent evidence | `verified_changed`; intended D/R/B and clean/saved state; one native undoable operation. |
| Clean eligible equal valid source | `verified_unchanged`; no write/history/bookkeeping transition. |
| Equal but invalid source, or dirty equal source | Proven refusal with parse/dirty reason; no unchanged success. |
| Stale basis, wrong identity/session, missing safety fact | Refusal before application; preserve source, saved state and history. |
| Preflight root syntax or dependency error | Refused with correct source attribution, no mutation. |
| Unsupported dependency/effect/representation | Refused before its disallowed effect; no guessed positive validation. |
| New human edit between prepare and apply | Refused at fresh boundary without losing the edit. |
| New human edit after B/R or D changed | Applied-unverified with invalidation; do not tag newer text or reassert old intent. |
| Descriptor namespace loss, short write/failure, failed flush or failed/unverified same-fd T0 restoration after content write | Truthful known/unknown persistence and application; no saved tag or false success. Known changed content remains `applied_unverified`, never a false `refused`/`not_applied` or rollback. |
| A partially completes, or post-change exact-source validation returns invalid/unavailable | Applied-unverified with per-stage facts; no rollback claim. |
| Authorization sent, response lost | Application unknown unless retained attributable facts already prove a change or irreversible pre-boundary discard. |
| Known competing write later happens to match desired bytes | Non-success; equality cannot erase known invalidation. |

## 6. Compatibility and migration

Observation's public JSON/library behavior stays v1 and read-only, including clean/dirty/divergent/limited outcomes. Edit is a new caller operation/schema v1, not an optional observation flag. New outcome enums or changed required fields/precedence require deliberate edit-schema versioning.

T004 cuts every private caller/worker/addon/fixture peer over to bridge v2, with authenticated edit capability and native build identity. V1 private peers are rejected rather than negotiated into mutation; observation's public schema remains unchanged. Restart/re-enable the updated addon to obtain a fresh descriptor/session, then collect a new observation basis. The typed saved-state and validation layouts migrate together to actual stock evidence; no private-mtime aliases, patched-editor requirement or cross-process editor-clock parity remains. The caller's real-editor acceptance and T005's cumulative feature evidence remain separate completion gates.

## 7. Implemented Rust core

`godot_agent_kit::script_edit` exports the checked `ReplacementSource`,
`ExpectedRevisionBasis`, `EditRequest`, evidence records and `EditAttempt` reducer.
The module has no JSON, transport, filesystem-acquisition or Godot dependency.
Evidence records are trusted integration inputs, **not caller-provided authorization**.
All bindings, clocks, ordering and required postconditions are checked again by the reducer.

Consumer sequence:

1. Extract `ExpectedRevisionBasis::from_observation(&prior)` and construct
   `ReplacementSource::new(String)` / `EditRequest::new(...)` with a new request ID.
   Ineligible observations can produce `EditOutcome::refuse_without_basis(...)`.
   The basis retains actual source witnesses/collection stamps, clean/open provenance
   and current B version; it contains no invented prior saved version.
2. Create `EditAttempt`, `select` a freshly authenticated target, then `prepare`
   with a fresh `ObservationOutcome`, independent `SavedStateEvidence` and caller-clock
   `DiskMetadata`. A missing inspection stays unavailable with an explicit reason.
3. Changed intent requires `validation(Preflight)`, then fresh
   `guard_application(...)` and a true `ready_to_apply()`. Only then call
   `authorize()` immediately before releasing the one-shot worker control.
   A failed guard has not dispatched and remains not-applied. Record
   `enter_application()` before source/history entry. `buffer_changed`,
   `resource_synced` and permanent `discard_before_boundary` require a
   request/session/document-bound `NativeWitness`.
   The `enter_resource_sync`, `enter_persistence` and `enter_finalization` methods
   check eligibility for starting their respective stages; `*_unknown` records lost replies.
4. Supply actual `PersistenceReceipt`, `FinalizationResult` and `validation(PostChange)`
   facts, then independent `verify(...)` source/dirty/saved/disk observations and
   `ContextRecheck`. Finalization's `before_*` fields describe entry to A, **after**
   source application and persistence, not a copy of the original revision.
   If later helper/verification work is lost, `retain_after(...)` can preserve an
   independently observed survivor snapshot. It does not complete verification
   or substitute for the final successful sample. Partial finalization retains
   confirmed bookkeeping and before/after facts even when a later tag is unknown.
5. Unchanged intent uses only `validation(Unchanged)` and `verify(...)` after preparation.
   It never authorizes application, writes, tags saved state or participates in history.
6. `fail(...)` retains terminal causes; `finish(interval)` consumes the attempt and
   emits one of the five outcomes. No late evidence can be supplied to that consumed
   attempt. `Err(EditError)` is sticky non-success; finish the attempt to report retained
   knowledge rather than replace it with a generic refusal.

Acknowledging an effect is distinct from authorizing the next effect. An attributable
R/write/bookkeeping result can retain known partial application even when an earlier
acknowledgment is missing; it never fills in missing earlier stage completion or
passes independent verification. A write-start alone establishes no known change.
Read-only survivor collection remains available after an unsuccessful applied stage.

The core compares acquired D/R/B text exactly before retaining hash/length summaries;
it does not duplicate full source in edit outcomes. It preserves invalidated source
summaries and their original attribution. Denial/ambiguity/busy suppress source-derived
fields even when another error occurs later, without erasing known application.
Incomplete validation/context or exceeded dependency/diagnostic bounds produces
unavailable bounded evidence, never a positive parse result.

Source and diagnostic text are redacted from their `Debug` implementations. Other
evidence contains requested paths and revision summaries, so whole records are still
not suitable for incidental logs. This API does not authenticate/acquire facts, perform
native edits, enforce a real-time deadline, or prove native history/durability.
[T001 verification](../quickstart.md#9-t001-core-acceptance-2026-09-27) records its boundary.
