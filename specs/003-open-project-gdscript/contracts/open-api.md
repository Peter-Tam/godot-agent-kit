# Open Project GDScript — Caller Contract v1

**Status:** Implemented and accepted on the exact recorded stock Godot/macOS arm64 environment; [T003 cumulative acceptance](../quickstart.md#10-t003-cumulative-acceptance-2026-09-30) completes Feature 003. Governed by [spec.md](../spec.md), the [data model](../data-model.md) and [native contract](native-integration.md). This opening v1 and existing public observation/edit v1 contracts are unchanged by T003. No wider support is claimed.

## 1. Invocation and input

```sh
open-gdscript --registry "$REGISTRY" --project "$PROJECT" \
  --session "$SESSION" --script res://scripts/subject.gd
```

One invocation attempts to open one existing known standalone `.gd` in the selected live editor. `--registry`, `--project` and `--script` are required. `--session` is optional only when existing authenticated resolution selects exactly one session. The existing checked project/path/session limits apply. The registry is the existing owner-private routing location, not arbitrary source access.

Use the existing CLI conventions for `--help`, argument errors, signal handling and generated request IDs. Normal invocations emit exactly one bounded UTF-8 JSON result plus newline to stdout. Stderr is source-free. Stdin is unused; no source, edit basis or input document is read. Unknown/duplicate flags, positional arguments, force/reload/focus/retry options and arbitrary operations are invalid. No editor is launched, no script is discovered/created and no source is written.

The library exposes a checked opening request and immutable outcome through the existing package; transport DTOs and native receipts are not its domain API. An opening result is not an edit basis. Invoke the existing observer separately before an edit.

## 2. Result shape

Every normal result uses the following top-level fields. Null means no attributable record, not an empty successful observation. Unknown fields/types are rejected by the private integration; public consumers should use the deliberate version contract rather than infer meaning from omitted data.

| Field | Type / meaning |
|---|---|
| `schema_version` | Integer `1`. |
| `operation` | String `open_gdscript`. |
| `request_id` | Generated checked request ID. |
| `requested_target` | Checked `{project_root, session_id, script_path}`; null if arguments could not establish it. |
| `resolved_target` | Existing observation-v1 target identity or null; actual project file ID, session, engine version/hash and exact script path. |
| `interval` | Existing `{started_unix_ms, finished_unix_ms, elapsed_us}` from the caller clock. |
| `outcome` | One of the five outcomes below. |
| `reason` | Bounded machine reason from §4; never raw exception/editor output. |
| `stage` | Furthest entered stage from the data model; not a claim of stage completion. |
| `application` | `not_applied`, `applied`, `partly_applied` or `unknown`. Lifecycle effects, not source-write status. |
| `progress` | `resource_binding`, `initial_compilation`, `document_open`, `verification`, each a stage fact described below. |
| `before` | Target summary or null: confirmed open/cache mode, attributed document identity, source availability/hash/length, buffer dirty and Resource-edited evidence. No duplicate source bodies. |
| `observation` | Null or `{purpose, snapshot}`. `purpose` is `preparation`, `recognition` or `verification`; `snapshot` preserves the existing observation-v1 snapshot fields and adds an opening-only `interval` for that acquisition, with the same `{started_unix_ms, finished_unix_ms, elapsed_us}` shape. Independent D/R/B, collection stamps, comparisons and invalidation keep their existing meanings. |
| `resource_edited` | Latest separately acquired target Resource flag: `{availability, value, collection, witness, reason}`. `value` is boolean or null. Does not redefine observation v1's buffer dirty field. |
| `target_parse` | `{state, origin, code, collection, diagnostics}`; states `valid`, `invalid`, `unavailable`, `not_collected`; origin `initial_compilation` or null. Only actual attributable evidence. |
| `protection` | `{status, reason}`; status `not_applicable`, `preserved`, `unavailable` or `invalidated`. No current/unrelated source, path, hash or private validation receipt. |
| `history` | `{participation: "not_participated", preservation, reason}`. Preservation `observed`, `unavailable`, `invalidated` or `not_applicable`; scoped to observed source/history-version witnesses, not enumeration of the opaque history stack. |
| `selection` | Null or `{before, after, request_effect}`. Relations `target`, `other`, `no_source_editor`, `unknown`; effect `none`, `selected_target`, `unknown`. No other-document names. |
| `diagnostics` | At most 64 source-free `{stage, reason, code}` records; nullable bounded engine/system code. Source changes and missing evidence remain distinguishable. |
| `safe_next_action` | Fixed source-free guidance appropriate to the outcome. |

A stage fact is `{state, reason, collection, script_instance_id, editor_instance_id, buffer_instance_id}`. State is `not_started`, `not_applicable`, `entered`, `completed`, `failed` or `unknown`; unused identities/collection/reason are null. `resource_binding` additionally has `mode: created|reused|null`; a completed reuse is not itself application. A completed compilation may be invalid; `target_parse` records the result. Actual document association and independent verification are distinct stages. No source is repeated in progress events.

Source summaries replace each observed text with `{sha256, utf8_bytes}` while preserving its authority, availability, witness, collection and invalidation. They do not upgrade an unavailable surface. The latest snapshot retains its original interval and purpose; if effects may have invalidated a preparation sample, mark that explicitly rather than calling it current post-state. A source from a denied/ambiguous attribution is never included.

The existing observer places its interval beside its snapshot. Opening retains
that acquisition interval inside its opening-only snapshot wrapper so the outer
operation interval cannot be mistaken for the latest sample's interval. This
does not change observation/edit v1 or create another source-bearing record.

`target_parse` diagnostic records are bounded by the existing validator diagnostic limits and contain only target logical path, severity, code and available line/column, not source snippets or raw compiler messages. A bare engine parse error may have no line/column. Cached opening does not reload merely to obtain diagnostics: `not_collected` is truthful. Invalid target parsing does not preclude verified opening. Private current-document validation failure is instead an opening-context refusal and does not disclose that other document's diagnostic content.

## 3. Outcomes, exit codes and interpretation

| Outcome | Exit | Application / required evidence |
|---|---:|---|
| `verified_newly_opened` | 0 | Applied; exact new buffer, independently agreeing D/R/B equal to admitted D, observed clean buffer and Resource edited=false, required identity/context rechecks intact. |
| `already_open_unchanged` | 0 | Not applied; exact existing open identity freshly established, no lifecycle authorization or request-caused source/history/selection change. Source/dirty observation may be limited or divergent. |
| `refused` | 3 | Not applied; no lifecycle effect and no possible future stage from this attempt. Specific reason retained. |
| `applied_unverified` | 4 | Applied or partly applied; known cache/document/selection effects but absent/invalid postconditions. |
| `effects_unknown` | 4 | Unknown; effects were possible but neither application nor irrevocable no-effect termination is established. |

Malformed CLI arguments use exit 2 with a structured `refused`/`invalid_request` result. Output delivery failure uses the existing non-success process convention; no successful JSON delivery can be inferred from a missing result. Signal cancellation uses the same effect-sensitive result reduction rather than treating a signal as rollback.

Exit 0 does not mean safe-to-edit, parse-valid or complete observation. In particular, `already_open_unchanged` can correctly report dirty B equal to D, divergent sources, an unavailable surface or a source-limit refusal on one authority. Selection/path authorization failure suppresses source. Once exact authorized selection is established, source-local availability follows observation v1; for example, unavailable D does not fabricate or erase independently authorized R/B. A new-open attempt, unlike recognition, requires readable admitted D before effects.

Known effects take precedence over later timeout/disconnect/malformed/missing evidence: retain them in `applied_unverified`. If no effect is known after authorization, lack of response yields `effects_unknown`, not no-effect refusal. Only an attributable irrevocably discarded pre-effect attempt can restore a proven not-applied result. Results cannot be upgraded by late evidence.

## 4. Reasons and non-success precedence

Opening reasons use the following vocabulary; structured native/system codes may refine a reason without changing it:

- Positive: `complete`, `already_open`.
- Request/routing: `invalid_request`, `ambiguous_session`, `session_ended`, `session_changed`, `editor_unavailable`, `disconnected`, `denied_access`, `outside_project`.
- Document/evidence: `missing_script`, `invalid_document_kind`, `unsupported_representation`, `unsupported_capability`, `open_state_unknown`, `cached_source_conflict`, `dirty_conflict`, `stale_resource`, `stale_buffer`, `evidence_unavailable`.
- Entry/protection: `unsafe_editor_context`, `current_parse_invalid`, `current_validation_unavailable`, `target_changed`, `source_changed`, `revision_changed`, `context_changed`, `busy`.
- Execution/verification: `protocol_error`, `native_failure`, `compilation_unavailable`, `verification_incomplete`, `verification_failed`, `timeout`, `cancelled`.

Check input/scope and authenticated target first; never replace missing target authority with a plausible document. On the new-open branch, access/source acquisition, cached-source/open attribution, context admission and entry rechecks must pass before effects. Preserve the first causally established failure and all known later effects; incidental missing evidence cannot hide a known conflict or retargeting. Within independent recognition observations retain each authority's actual availability rather than invent a single global failure.

An entered native compilation that does not return an attributable result is unavailable/unknown, not an invalid source that can automatically advance. A completed `ERR_PARSE_ERROR` is attributable invalid target parsing and can continue to opening. Other native failures retain actual partial effects and stop the sequence.

## 5. Deadlines, privacy and compatibility

One controlled invocation, with stdout drained, returns within ten seconds: 9.5 seconds for selection/admission/effects/verification and 0.5 seconds reserved for bounded delivery. The timer starts before parsing/resolution. Existing five-second observation and ten-second edit contracts are unchanged. No automatic retry, reconnect, retargeting, queue, rollback, Save or buffer closure follows timeout.

For any possibly applied outcome, guidance requires a fresh observation of the explicit original target before another intentional action. Already-open guidance states that observed dirty/limited state remains and a later edit needs a new valid observation basis. A failed observation is not permission to force an edit.

512 KiB per source and existing 12 MiB final-result/depth-32 limits apply without truncation. No target source/hash before unique authentication. Current/unrelated source used privately for admission stays out of results, logs, registry metadata and routine diagnostics; only the selected target snapshot may contain source. Native pointers, credentials, proofs and worker/helper paths are never public.

This is one new public v1 contract. Private [bridge v3](bridge-protocol.md) and native API revision 2 migrate together, with unchanged public observe/edit v1. Older private peers are rejected; no automatic compatibility fallback opens or edits a document. Exact support begins only with the feature's own real-editor acceptance.
