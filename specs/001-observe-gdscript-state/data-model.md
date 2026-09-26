# Data Model: One-Script Observation

**Status:** Phase 1 design, schema version 1. No persistent database or source cache. See [spec.md](spec.md), [research.md](research.md), and [caller contract](contracts/observation-api.md).

## Entities and ownership

### ObservationRequest

| Field | Type / rule |
|---|---|
| `schema_version` | Integer `1`; unknown major versions refused |
| `request_id` | Nonsecret caller correlation string, 1–64 ASCII letters/digits/`-`/`_`; unique for each attempt |
| `project_root` | Explicit absolute local directory; resolve to a filesystem identity before selection |
| `session_id` | Optional 32-character lowercase hexadecimal identifier; omission requires proof of exactly one live matching session |
| `script_path` | Exact `res://` resource path; external `.gd`, or an already-identifiable built-in GDScript subresource |

Every standard request asks for D, R, B, open state, and dirty state. No surface selection, force flag, source replacement, repair, execution, or retry option. A request ID is not an authenticator or future mutation revision.

### EditorSession

One editor/plugin lifetime belongs to one project. Enabling the addon creates a fresh identity even in the same editor process; disabling/restarting never reuses it.

| Field | Type / rule |
|---|---|
| `session_id` | Random 128 bits, serialized as lowercase hex |
| `project_root` | Root reported by addon; Rust canonicalizes and verifies it against caller selection |
| `project_file_id` | Rust-observed device/inode strings for the canonical project directory; scoped to this session/request, not globally permanent |
| `godot_version` | Exact version and engine hash |
| `endpoint` | IPv4 loopback address and bootstrap-selected port, private integration only |
| `token` | Independent random 256-bit pre-shared secret; private descriptor/trusted memory only, never transmitted or included in observation results or diagnostics |
| `lifecycle` | `advertised`, `authenticated`, `ended`, or `unavailable`; descriptors alone do not prove liveness |

Only session metadata persists in an owner-private registry. The core receives authenticated identity, not a dependency on TCP, descriptor layout, Godot objects, or JSON.

The `authenticated` lifecycle state records successful mutual candidate authentication, including listener proof of prior secret possession and verified client/finish proofs under the bridge contract. Echoed descriptor fields or a reused TCP endpoint cannot establish it. It does not by itself select/bind the session to an observation request or authorize source reads.

### ResolvedTarget / DocumentIdentity

A `ResolvedTarget` is established only after exactly one intended authenticated session is selected/bound for this request, not by candidate authentication alone.

`ResolvedTarget` contains canonical `project_root`, `project_file_id`, exact `session_id`, `godot_version`, and exact `script_path`. A `DocumentIdentity` adds:

- `kind`: `external_gdscript` or `builtin_gdscript` when proven; otherwise unknown.
- `script_instance_id`, `editor_instance_id`, `buffer_instance_id`: optional decimal **strings**, sampled from actual objects; never fabricated from path or text.
- `disk_file_id`: optional device/inode strings for D, independently of editor identity.
- `resource_path`: exact observed Resource path; a built-in path retains its container and subresource identifier.

IDs have meaning only within the selected session and observed interval. Closed/unloaded documents legitimately lack editor instance IDs. A changed instance at the same path is replacement, not continuity. A source path alone cannot authorize reuse of an old snapshot.

### ObservationInterval and CollectionStamp

The overall interval contains `started_unix_ms`, `finished_unix_ms`, and monotonic `elapsed_us`. Wall-clock timestamps locate the request; only monotonic elapsed time enforces deadlines.

Each fact has `collection`: `{clock_id, started_tick_us, finished_tick_us, received_elapsed_us}`. Tick counters are decimal strings; `clock_id` identifies `caller` or `editor:<session_id>`. The supervisor stamps `received_elapsed_us`. Never subtract timestamps from different clock domains or imply that source reads were simultaneous. Missing facts have no fabricated collection time.

### SourceObservation (D, R, B)

The snapshot contains one record for each authority, even when unavailable.

| Field | Rule |
|---|---|
| `authority` | Exactly `D`, `R`, or `B`, matching its enclosing field |
| `availability` | `observed`, `unavailable`, or `not_applicable` |
| `text` | Present **only** when observed; exact string, including `""`, whitespace, BOM, and line endings exposed by that authority |
| `collection` | Required for observed text; absent if never collected |
| `reason` | Required for unavailable/not-applicable; machine-readable reason plus safe action |
| `witness` | Relevant identity/metadata/version strings; no claim of a cross-authority or mutation revision |
| `staleness` | `{state: "unknown"}` or `{state: "known_stale", evidence: [...]}`; divergence is not sufficient evidence |
| `invalidated_evidence` | Optional former text, collection, witness, and invalidation reason, only when availability is unavailable |

Each record is attributed to the snapshot's resolved target and original DocumentIdentity. Never place replacement-document values in that snapshot. If text is invalidated, remove it from the current `text` field and optionally preserve it under `invalidated_evidence`; comparisons cannot use it. An unavailable fact may retain collected metadata but cannot imply its missing text was observed.

- **D:** Independent strict-UTF-8 disk read. Missing, unreadable, invalid encoding, excessive size, or no attributable standalone source => unavailable. Built-in scene/resource container bytes are never script D.
- **R:** Fresh read of an already-loaded GDScript. Not loaded => unavailable/`resource_not_loaded`, not not-applicable. A currently loaded but stale Script is still an observed R if its actual source was read.
- **B:** Actual attributed CodeEdit text. Confirmed closed => not-applicable/`document_not_open`; open but unreadable/unassignable => unavailable. Unknown open state never permits not-applicable.

A per-source limit affects availability, not applicability. An over-limit D, R, or B is `unavailable` with reason `too_large` and no observed `text`, never truncated, observed empty, or `not_applicable`. Continue with other independently observable facts under the existing deadline and safety rules, retaining their already-gathered valid evidence. A per-source limit alone is not an operation-wide refusal.

### DocumentState and DirtyObservation

`document` contains `validity` (`valid`, `missing`, `invalid`, `unknown`), `open_state` (`open`, `not_open`, `unknown`), collection stamps/reasons for the evidence, and `identity` as above.

Validity and existence on disk are not equivalent. An open GDScript with missing D remains an observable document. A missing external `.gd` with no open document is `missing`; a non-GDScript target is `invalid`. GDScript syntax errors and empty text do not invalidate the target. A closed built-in target requires an already-present GDScript identity; absent that, report unsupported identity rather than load its container.

`dirty` has `availability`, `state`, `collection`, `reason`, and optional `invalidated_evidence`:

| Availability | Allowed state | Meaning |
|---|---|---|
| `observed` | `dirty` or `clean` | Complete, document-attributed editor unsaved evidence |
| `unavailable` | `unknown` | Unsupported, unreadable, unattributable, or invalidated evidence |
| `not_applicable` | `not_applicable` | Document confirmed closed, no buffer |

Source equality/inequality, global unsaved indications, or absent response fields cannot determine dirty state. Do not return unsaved paths of unrelated documents; the result contains only attribution evidence for the requested script.

### ObservationSnapshot

Fields: `target`, `document`, `sources` with D/R/B records, `dirty`, `comparisons`, `agreement`, `consistency`, and `diagnostics`.

For each pair D/R, D/B, R/B:

1. If both source records are currently observed in this request, compare their exact text: `equal` or `different`.
2. Otherwise return `unknown`, including comparisons involving invalidated evidence or a not-applicable B.

Overall `agreement`:

- `divergent` if **any** independently observed pair differs, even if a third surface is unavailable.
- `agree` only if all three current observed texts are equal.
- `unknown` otherwise. D=R on a closed script does not prove three-way agreement.

`consistency` contains `atomic: false`, `stability: unknown|changed`, `checks: performed|unavailable`, and `detected_changes: []`. Matching before/after witnesses mean no change was detected, not that the editor was frozen. Detected changes set stability to changed and identify affected facts. Unknown staleness or unproven atomicity alone does not turn valid observed values into unavailable values.

### Diagnostic

Fields: `code`, `stage`, `surface` (D/R/B/dirty/document/session or null), safe `message`, and `action`. Stages: `validate_request`, `resolve_target`, `authenticate`, `attribute_document`, `read_disk`, `read_editor`, `recheck`, `finalize`.

Codes include `dirty_attribution_unavailable`, `buffer_attribution_unavailable`, `buffer_unreadable`, `resource_not_loaded`, `disk_missing`, `disk_unreadable`, `invalid_utf8`, `too_large`, `no_standalone_disk_source`, `identity_changed`, `source_changed`, `open_state_unknown`, `unsupported_version`, `session_ended`, `deadline_exceeded`, `out_of_project`, `unsafe_registry`, `authentication_failed`, `invalid_frame`, and `cancelled`.

Messages/actions contain no source snippets, secret tokens, authentication nonces/proofs, raw malformed payloads, or unrelated project paths. Dirty/divergent evidence is not an error code or mutation conflict. Parser/compiler diagnostics are outside scope; do not parse or reload to manufacture them.

### ObservationOutcome

Fields: `schema_version`, `request_id`, `outcome`, `interval`, `resolved_target` (nullable), `snapshot` (nullable), `diagnostics`, and `selection` (only minimal source-free disambiguation metadata).

| Outcome | Required interpretation |
|---|---|
| `complete_observation` | Open, valid, attributed document; D/R/B and dirty independently observed; no detected invalidation. May be dirty/divergent. No permission to edit. |
| `limited_observation` | Target known and not confirmed closed, with one or more required facts unavailable/unknown/invalidated, including a per-source limit; preserve unrelated usable facts. |
| `not_open` | Valid target confirmed closed with B/dirty not applicable. R may be unloaded; any D/R limitation remains explicit. Never a complete visible-buffer observation. |
| `ambiguous_target` | Multiple confirmed candidates; no script source or candidate snapshots. Supply missing selector information. |
| `missing_target` / `invalid_target` | Distinct target failures, not closed or parse-error aliases. |
| `editor_unavailable` | Intended session not available before authenticated observation; never auto-start or switch editors. |
| `disconnected_editor` | Known loss after authentication; identify loss and obtained/invalidated facts. |
| `timeout` | Deadline expires without stronger known result; not proof of a disconnected or connected editor. |
| `unsupported_observation` | The request as a whole cannot proceed safely or meaningfully because of unsupported version/capability/target identity or an operation-wide resource bound. A per-source limit alone never qualifies; no fabricated facts. |
| `denied_access` | Scope/authentication/permission refusal; no source from denied targets. |
| `invalid_request` / `protocol_error` / `cancelled` | Explicit boundary failures; do not collapse into success, missing target, or a surface fact. |

Unknown final outcomes are incompatible until deliberately versioned. Optional metadata may evolve additively, but changes to source/dirty/availability/precedence semantics require a new schema version and migration documentation.

## State transitions and terminal precedence

```text
received → validated → candidate discovery
  per potentially matching candidate:
    discovered → authenticated → evaluated for requested target identity
  across the evaluated candidates:
    exactly one intended live session established
      → target_resolved (selected/bound for this request)
      → document_attributed → collected → rechecked → classified → returned
```

Discovery, candidate authentication, identity evaluation, and unique selection are source-free parts of `resolve_target`. The `authenticate` diagnostic stage identifies candidate authentication within that resolution, not target binding. Only selection/binding permits observation to proceed, subject to document attribution and the existing scope/capability checks. This conceptual flow adds no lifecycle values, diagnostic stages, or outcomes.

Every stage can terminate with a structured reason. Collection can produce individually usable facts before completion. There are **no** mutation-accepted/applied/synchronized/verified states; those terms are reserved for later mutation work.

1. Validate syntax and scope before source acquisition. Known denied scope/authentication suppresses source, including any accidentally collected evidence. Ambiguity never starts source reads.
2. Evaluate authenticated candidate identities against the requested project and any exact session selector before selecting/binding the intended session. Never choose by focus, first response, newest session, or a previous target. With an explicit session, failure never falls back to another descriptor.
   - Exactly one intended live session proven under the supplied selectors => select/bind it as `target_resolved`; authentication alone is insufficient.
   - Multiple confirmed matching candidates that the supplied selectors cannot distinguish => `ambiguous_target`, without source.
   - Unresolved candidate liveness that prevents a unique decision at the deadline => `timeout` at `resolve_target`, without source; do not discard that candidate to claim zero or one target.
   - Zero eligible available sessions => `editor_unavailable`, without source. `missing_target` remains a document/existence classification under rule 3, not a substitute for unavailable editor selection.
   - Preserve specific scope/authentication/permission, protocol, unsupported-capability, or interruption outcomes where applicable; do not collapse those failures into the zero-candidate outcome.
3. Missing/invalid target classification requires attributable document/existence evidence. Unknown open state is not absence. Disk read failures must not hide an existing open document.
4. Known disconnection, protocol failure, cancellation, or timeout prevents complete/not-open success. Retain already obtained facts with their time/target; live facts whose currency was invalidated move to invalidated evidence. Do not downgrade known disconnection to a generic timeout.
5. A same-document surface change invalidates only affected facts; identity replacement invalidates all facts dependent on that identity. A detected close during an open read is invalidation, not a newly fabricated closed snapshot.
6. If no operation-wide refusal/interruption applies and the valid target is confirmed closed, return not-open with explicit D/R limits, including `too_large` where applicable. Otherwise complete requires every open-document fact above; unavailable required source due to a per-source limit yields `limited_observation`, retaining other independently observed facts. One or more per-source limits alone must not become `unsupported_observation` or another operation-wide refusal. Unknown open state remains limited. No automatic re-read of the whole request to conceal a changing document.
7. Once returned, the result is immutable. Ignore late frames and never promote a timeout using a previous or replacement-session observation. An explicit caller retry is a new request, fresh selection, and fresh evidence.

## Deterministic invariants to test

- Pairwise equality is symmetric and transitive for three currently observed texts; missing/invalidated/empty are distinct.
- Three-way agreement cannot arise from only two authorities, even for closed documents.
- Dirty state and agreement are orthogonal to read completeness; empty observed text is not unavailable.
- A surface-level failure preserves unrelated evidence; target denial never exposes source.
- Ended session IDs and replacement document IDs cannot reuse an earlier target/snapshot.
- Every terminal path is bounded and correlated; no outcome falsely reports mutation, synchronization, UndoRedo participation, or edit authorization.
