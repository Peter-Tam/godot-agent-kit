# Close Project GDScript — Caller Contract v1

**Status:** Implemented and [accepted for T002](../quickstart.md#10-t002-public-caller-acceptance-2026-10-02) by the `close-gdscript` caller and protocol-independent core, with [T003 cumulative acceptance](../quickstart.md#12-t003-cumulative-acceptance-2026-10-02) completing Feature 005. T003 changes no caller semantics. Governed by the [specification](../spec.md), [data model](../data-model.md), [private bridge](bridge-protocol.md) and [native integration](native-integration.md). Earlier [research](../research.md) remains native planning evidence, not public caller acceptance. The tested environment is the exact recorded official Godot 4.7.2 executable/commit on macOS arm64, not a general version range.

## 1. Invocation and strict input

Invocation:

```sh
printf '%s\n' '{"schema_version":1,"request_id":"close-recognition-1","basis":null}' |
  close-gdscript --registry "$REGISTRY" --project "$PROJECT" \
    --session "$SESSION" --script res://scripts/subject.gd
```

`--registry`, `--project` and `--script` are required. `--session` may be omitted only when the existing authenticated resolver selects exactly one live session. Project selection remains explicit even when the basis names a project. The basis cannot resolve ambiguity, choose focus, replace an ended explicit session or launch an editor. Existing help, signal and argument conventions apply; unknown/duplicate flags, positional arguments and force/reload/Save/discard/retry/reopen/batch options are invalid.

Stdin must contain exactly one UTF-8 JSON object followed by EOF; JSON whitespace around it is allowed. Its **only** keys are `schema_version`, `request_id`, `basis`, all required:

| Key | Accepted value |
|---|---|
| `schema_version` | Integer `1`, not a string, Boolean or fractional number. |
| `request_id` | Checked ASCII correlation, at most 64 bytes, fresh and different from the prior observation request ID. |
| `basis` | A complete existing public observation-v1 result, or explicit `null`. |

The example is a valid illustrative recognition input, not an example success output. Missing, unknown or duplicate keys, duplicate nested observation keys, wrong types, invalid UTF-8, trailing non-whitespace/second values, premature EOF and more than 12 MiB refuse as malformed input. Depth is at most 32. A blocked stdin reader shares the original operation deadline; it cannot postpone starting that deadline. Reject excess before unbounded allocation, never truncate or normalize. No hash-token shortcut, arbitrary source filename, replacement text, expected-source override, permission token or caller-supplied validation receipt is accepted. A prior observation may itself contain its observed source; that is evidence in the existing schema, not source to apply. There is no new source-file input flag.

Normal delivery is exactly one bounded UTF-8 JSON result plus newline on stdout. Stderr is source-free; raw OS/editor/compiler messages are not an alternative result. The library surface is checked `CloseRequest::new(..., Option<&ObservationOutcome>)`, immutable `ClosingOutcome` and the real close runner. The borrowed prior outcome is converted once through existing `ExpectedRevisionBasis::from_observation` internally; there is no new public alias, copied eligibility algorithm, `EditAttempt` reuse or edit-policy parameter. Checked retained intent contains identity/revision/provenance summaries, not retained prior source bodies.

## 2. Basis, recognition and effect eligibility

A non-null basis must be a complete open standalone observation-v1 with performed consistency checks, no detected change, independent exact D/R/B equality, attributed clean buffer state, actual file/Script/editor/buffer identities, no known stale R/B and current buffer version. Prior saved version or atomic stability is not invented. A limited/closed/dirty/divergent/ineligible prior result is a basis refusal, never silently null. Malformed serialization uses exit 2; a correctly decoded but unusable basis uses exit 3 with the actual basis reason.

The basis must match explicit selectors and freshly resolved project/session/path and original file identity. A wrong session or replacement file refuses even if the replacement happens to be closed. For the same valid file/session freshly confirmed closed, an otherwise eligible old open-buffer basis is **not** applied to the closed state: return no-effect recognition and `expected.use: not_applied_to_closed_state`. Null is the direct recognition input. Null against an open target refuses `missing_basis`; a race from closed to open never authorizes closing.

An open target additionally requires fresh exact matching Script/editor/buffer identity, source and current version; independent D==R==B; attributed clean buffer and Resource-edited=false; pinned file/namespace and all effect-context guards. Same-text edit/version changes and close/reopen identities refuse. Native saved version and Resource-edited state are independently acquired now, not asserted by observation v1. No Save, discard prompt, reload, source repair, selection or validation is used to manufacture eligibility. Read-only source and a safe syntax-error target alone do not prohibit closing; target parse success is not a gate.

Every remaining supported open script is protected, because native history/sorting can visit more than the selected tab. At most eight documents including target, one private R==B copy per remaining document, at most 512 KiB aggregate remaining source, and at most 256 KiB aggregate non-source effect metadata are admitted. Unsupported/mixed/custom/external-editor contexts refuse effectful closing. Dirty remaining R==B is permitted with attributable flags, versions, history/effect guards and completed-valid private source-only validation for each remaining document; dirty R!=B refuses. Validation is purpose `close_context`, sequential one child at a time under the original cutoff, not a public validator or an instruction to wait for source convergence.

Already-closed recognition does not run effect-profile admission, protected-document validation or continuation waiting, and does not impose these context source limits on ordinary observation. It still requires a valid existing authorized target and confirmed absence of its document. Missing/invalid disk targets or unknown open state cannot become already closed.

## 3. Complete public result shape

All top-level fields below are required; nullable records are explicitly null. IDs/version counters retain existing canonical unsigned decimal-string formats, not lossy JSON numbers. Availability, witness, collection and invalidation meanings are those of observation v1.

| Field | Shape / interpretation |
|---|---|
| `schema_version`, `operation`, `request_id` | `1`, `close_gdscript`, checked correlation. |
| `requested_target` | Checked `{project_root, session_id, script_path}` or null when input did not establish selectors. |
| `resolved_target` | Existing resolved observation target identity, including actual project identity, session, engine version/hash and path; null when unauthorized/unresolved. |
| `interval` | Caller-clock `{started_unix_ms, finished_unix_ms, elapsed_us}` for this attempt. |
| `outcome`, `reason`, `stage`, `application` | Closed vocabularies in §4–§5; stage is furthest entered, not completed. |
| `expected` | Null or checked prior target, document identity, source digest/length, current version and provenance (prior request/interval, authority witnesses/collections, prior document/dirty evidence), with `use: matched\|mismatched\|not_applied_to_closed_state\|unavailable`. No source bodies or permission token. |
| `progress` | `closing`, `native_revalidation`, `verification`, each exactly `{state, reason, collection}`. State `not_started\|not_applicable\|entered\|completed\|failed\|unknown`; unused reason/collection null. |
| `before` | Nullable attributable target summary: open state/document IDs, independently acquired source summaries, dirty evidence, Resource-edited/current/saved-version facts and validity. |
| `observation` | Null or `{purpose, interval, snapshot}`; purpose `preparation\|recognition\|verification\|survivor`. Acquisition interval is separate from operation interval. Existing snapshot semantics, replacing observed text with `{sha256, utf8_bytes}`, while retaining actual per-authority witnesses/collections, comparison, availability and invalidation. |
| `resource_state` | `{state, resource_edited, reason, collection}`; state `retained\|unloaded\|unavailable\|invalidated\|not_collected`, flag Boolean or null. Latest actual post-close cache/Resource evidence, not inferred from missing B. |
| `protection` | `{status, revalidation, required_count, completed_count, reason}`; status `not_applicable\|preserved\|unavailable\|invalidated`, revalidation `not_applicable\|pending\|completed\|unavailable\|invalidated`. Counts null when not established. No protected path/source/hash/receipt. |
| `selection` | Null or `{before, after, request_effect}`; relations `target\|other\|no_source_editor\|unknown`, effect `none\|native_fallback\|unknown`. No other-document names. |
| `history` | `{participation: "not_participated", target_buffer, unrelated, reason}`; target buffer `retained\|disposed\|unavailable\|not_applicable`, unrelated `observed\|unavailable\|invalidated\|not_applicable`. |
| `diagnostics` | At most 64 bounded source-free `{stage, reason, code}` records; bounded engine/system code or null, no source snippets/raw parse output. |
| `safe_next_action` | Fixed source-free guidance appropriate to actual effect knowledge. |

A closing snapshot is **not** a reusable source-bearing observation-v1 edit/close basis. Obtain a new ordinary observation for future effectful action. Source digest/length is a summary, not an independent observation. A preparation sample after possible effects remains earlier/invalidated evidence, never relabeled fresh verification. B and buffer dirty are `not_applicable` only after independently confirmed closure; unreadable B remains unavailable, and empty observed text remains a value. Genuinely unloaded R retains observation-v1 unavailable/unloaded semantics and is explained by `resource_state`, without force-loading. Closing does not assert target parse validity; available target errors remain attributable bounded source-free diagnostics, not fabricated clean parsing.

Native selection fallback is permitted only when closing the selected target. A non-selected target must preserve exact private selected identity. Target history disposal means actual old buffer lifetime ended, not enumeration of an opaque native history stack, a source-history action or an undoable-close promise. Unrelated history preservation requires actual witnesses and acceptance exercising usable Undo/Redo.

## 4. Outcomes and exact process exits

| Outcome | Exit | Application and required evidence |
|---|---:|---|
| `verified_newly_closed` | 0 | `applied`: admitted old document removed, no fresh target document, unchanged independent D/file identity, retained R independently unchanged or observed unloaded, native continuation completed/not applicable, all protection and final guards intact. |
| `already_closed_unchanged` | 0 | `not_applied`: fresh exact valid file/session confirmed closed, no effect authorization or source/selection/history change; source-local limitations explicit. |
| `refused` | 3 | `not_applied`: proven no lifecycle effects and no late product entry possible. |
| `applied_unverified` | 4 | `applied` or `partly_applied`: attributable lifecycle effect, but required closure/preservation/completion/verification missing or invalidated. |
| `effects_unknown` | 4 | `unknown`: effects possible, insufficient attributable application or irrevocable discard evidence. |

Malformed CLI/input uses exit **2**, structured `refused`/`invalid_request` when delivery is possible. Unexpected host/result-delivery failure uses exit **1**, not a claimed successfully delivered JSON outcome. `--help` follows existing successful help convention rather than fabricating a close result. Cancellation is reduced using actual effect knowledge, not a special rollback exit.

Exit 0 does not mean parse-valid, all project work clean, Resource unloaded, safe to edit or proof of an earlier interrupted close. A native `OK` or completed callback is insufficient: a separate fresh ordinary observation and independently reacquired Rust D plus native protection/file checks must pass. Known selection/removal effects survive later failure. If authorization may have reached the editor but no effect/discard evidence returns, use `effects_unknown`; lack of acknowledgment is not no effect. No late event upgrades an immutable terminal result.

## 5. Reasons, stages and precedence

Bounded machine reasons (native/system codes refine but do not replace them):

- Positive: `complete`, `already_closed`.
- Input/basis: `invalid_request`, `missing_basis`, `invalid_basis` (missing completeness/provenance/version), `revision_changed`, `identity_changed`.
- Routing/access: `ambiguous_session`, `session_ended`, `session_changed`, `editor_unavailable`, `disconnected`, `denied_access`, `outside_project`.
- Target/evidence: `missing_script`, `invalid_document_kind`, `unsupported_representation`, `unsupported_capability`, `open_state_unknown`, `dirty_conflict`, `resource_edited_conflict`, `source_divergence`, `stale_resource`, `stale_buffer`, `evidence_unavailable`.
- Preparation/protection: `unsafe_editor_context`, `context_limit`, `context_parse_invalid`, `context_validation_unavailable`, `target_changed`, `source_changed`, `context_changed`, `busy`.
- Execution/verification: `protocol_error`, `native_failure`, `native_revalidation_unavailable`, `verification_incomplete`, `verification_failed`, `timeout`, `cancelled`.

Stage vocabulary is `accepted`, `selected`, `inspected`, `prepared`, `validated`, `authorized`, `closing`, `settling`, `verifying`, `terminal`, with meanings/transitions in [model §7](../data-model.md#7-state-transitions-deadline-and-terminal-precedence). Recognition takes `inspected -> verifying -> terminal`, without effect stages.

Check input/scope, authenticated selection and valid exact file/session before recognition or effect authorization. Wrong-session/file binding is not excused by closed state. For open state, basis/identity/source/dirty and protection prerequisites must pass before entry. Preserve the first causal failure and sticky invalidation; incidental later unavailability cannot hide dirty/stale/conflicting evidence. Denied/ambiguous/invalid target authority suppresses source-derived output but not independently attributable source-free effect knowledge. Required post-entry failure defeats success even when closure was seen. A human-reopened target makes closure unverified; never close the new buffer. Before authorization refusal requires no effectful control; afterward only attributable irreversible pre-entry discard can establish no effect. Busy is terminal non-applied, never queued.

## 6. Meaningful consumer-visible boundaries

| Scenario | Required interpretation |
|---|---|
| Null + same valid closed target, cached R divergent/unavailable | Already-closed recognition may succeed with explicit independent source limitations; no load or clean/coherence claim. |
| Null + open target | `missing_basis`, no close. |
| Eligible prior open basis + same file/session now closed | Fresh recognition, expected `not_applied_to_closed_state`; no retroactive certification. |
| Old basis + replaced file or another session, even closed | Binding refusal, not recognition. |
| Equal-text dirty target or Resource-edited target | Conflict refusal; exact bytes alone do not prove safe disposal. |
| Same-text reopened buffer/version change | Identity/revision refusal before entry, or unverified after attributable effects; preserve newer buffer. |
| Missing disk file with surviving buffer | Refuse; never classify file disappearance as closure permission. |
| Safe syntax error, empty source, read-only target | Eligible if all independent clean/identity/effect evidence passes; no target parser/write gate. |
| Dirty unrelated R==B versus R!=B | Equal case may qualify with every guard/receipt and preserved history; unequal case refuses, never waits for synchronization. |
| Last tab / selected tab / non-selected tab | No selected source / reported ordinary fallback / unchanged selected identity respectively. |
| Native call returned but completion event missing | No verified new-close success; deadline produces effect-sensitive unverified/unknown result. |
| Human changes roster/source/selection/configuration during validation or wait | Sticky invalidation; no recapture/retry/reclose or restoration. |

## 7. Time, limits, privacy and compatibility

One controlled local invocation with stdout drained terminates within ten seconds: **9.5 seconds work from before input/selection, 0.5 seconds delivery reserve**. Addon remaining lease is integer 1–9000 ms, never beyond parent cutoff. All helper children, the single continuation wait and cleanup share that cutoff; no queue, waiter lease extension, fixed sleeps, forced parse, reconnect/retry or editor termination. Timing bounds are not a promise that every maximum-sized context succeeds.

Target D/R/B each remain ≤512 KiB UTF-8; effectful close requires exact LF UTF-8 without BOM/CR/NUL or unsupported controls. Existing 4 KiB control, 4 MiB selected source request, 12 MiB response/worker/final result and depth-32 ceilings apply. Source-local limits and availability for recognition remain observation-v1 semantics. Source is never truncated or copied between authorities to assert agreement.

Only uniquely authenticated explicitly authorized target summaries may be public. Protected source/paths/hashes, receipts, native pointers, secrets/proofs, helper paths and raw output never enter results, incidental logs or registry. No target hash/source before unique selection. No telemetry, remote access, new approval, dependency/service or gameplay/export authority is introduced.

Close is not unconditionally idempotent across reopening/session/file replacement. Request IDs are correlation, not replay-cache keys. After possible application, fixed guidance requires a **fresh ordinary observation of the explicit original target before another intentional action**, not automatic retry/reopen/rollback. A new request with null may recognize a still-closed target; it cannot close a subsequently reopened one. A later effectful close requires fresh eligible observation and a new request ID.

Existing public discovery/observe/edit/open v1 schemas, meanings, limits and deadlines stay unchanged: editing refuses closed documents and existing operations never implicitly close. Private peers migrate together to [bridge v5](bridge-protocol.md) and native family revision 3 using the existing `editor_integration` artifacts, with fresh editor lifetime/session. Historical acceptance is not relabeled; exact closing guarantees are limited to the [recorded cumulative acceptance and relevant-input review](../quickstart.md#t003-execution-decision-and-evidence-review).
