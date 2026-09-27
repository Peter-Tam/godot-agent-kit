# Observation Caller Contract — Version 1

**Status:** T001 implements the reusable Rust semantic library (§7); T002 implements source-free routing (§8); T003 implements the bounded caller/worker and confined disk boundary (§9); T004 adds the passive live-editor collector and clean-open vertical slice (§10); T005 extends document-attributed dirty/divergent and changing-document observation (§11); T006 verifies source-bearing multi-session routing and interruption outcomes (§12). This is **not MCP**. [Data model](../data-model.md) defines normative results; [bridge contract](bridge-protocol.md) defines private integration.

## 1. Operations and invocation

Exactly one source-observation operation exists:

```sh
observe-gdscript --registry "$REGISTRY" --project "$PROJECT" \
  --session "$SESSION" --script res://scripts/subject.gd
```

- `--project`: required absolute local project root. The caller canonicalizes and validates it; display name and current working directory are not substitutes.
- `--script`: required exact resource path. External `.gd` or an attributable built-in GDScript resource path such as `res://scene.tscn::GDScript_abc`; never an ambient absolute filename, `user://`, remote URI, glob, basename, or focused-document alias.
- `--session`: optional exact session ID. Omission is allowed only if source-free mutual authentication and selection prove one live session for that project. Listener identity requires prior-secret-possession proof, not echoed fields or a connected descriptor endpoint. Two live sessions return source-free disambiguation metadata. Unresolved liveness at deadline is timeout, not permission to discard a candidate.
- `--registry`: explicit owner-private bootstrap metadata directory outside the project, dedicated to the tool's own private state. Access is limited to minimal operational/session metadata for discovery, identification, and routing to the intended local editor under FR-016; it grants no access to project/source content or unrelated filesystem data. Existing authentication/privacy requirements apply. No untrusted remote endpoint flag. The same path must have been supplied to the addon through `GODOT_AGENT_KIT_REGISTRY`.

Bootstrap is separate and reads no source:

```sh
observe-gdscript init-registry --registry "$REGISTRY"
```

It creates/verifies owner-private metadata storage without overwriting an unsafe existing path. It returns a small JSON confirmation or structured permission/configuration error. Addon installation/enabling and editor startup are deliberate developer/test setup, not operations the observer performs.

`--help`/`--version` perform no observation. Unknown/duplicate flags, absent required values, or malformed selectors return `invalid_request`; no positional default, implicit project, or automatic retry. CLI generates a unique nonsecret request ID per attempt. Secret tokens remain in private descriptor/trusted memory, never wire frames, CLI arguments, or result fields; authentication nonces/proofs remain private bridge details and are excluded from results/logs.

## 2. Request semantics

The CLI maps to this protocol-independent request shape; a future adapter may supply the same semantic fields without adopting CLI/JSON:

```json
{
  "schema_version": 1,
  "request_id": "example-clean-1",
  "project_root": "/fixture/project",
  "session_id": "00112233445566778899aabbccddeeff",
  "script_path": "res://scripts/subject.gd"
}
```

Validate lengths, types, normalized components, exact session syntax, and scope before source acquisition. Reject empty/dot/dotdot path components, backslashes, NUL/control characters, URI query/fragment decorations, symlink components, and external path aliases. Preserve legal case and Unicode; do not case-fold/Unicode-normalize a caller's locator into a different document. Canonical project identity handles legitimate root aliases without using a name as identity.

A built-in locator retains both its project-confined container and subresource identity. It must resolve to an already-present GDScript through public editor/cache evidence. No scene parsing or force-loading is permitted to resolve it. Container text is never returned as D. Syntax errors in an external GDScript do not invalidate the target.

All requests ask for open state, D/R/B, and dirty state. There is no force/write/open/save/reload/reconcile/run/evaluate command, no mutation revision token, and no option to relax safety/deadline requirements.

## 3. Result and process behavior

The observation command emits exactly one UTF-8 JSON object plus newline on **stdout**. The object has all top-level fields below:

| Field | Contract |
|---|---|
| `schema_version` | `1` |
| `request_id` | Current request correlation |
| `outcome` | One terminal enum from the data model |
| `interval` | Actual start/end wall-clock milliseconds plus monotonic elapsed microseconds |
| `resolved_target` | Verified identity, or null before resolution; no secret token/endpoint |
| `snapshot` | Data-model snapshot, or null if none may safely be returned |
| `diagnostics` | Structured stage/surface/reason/action entries; empty is allowed |
| `selection` | Null normally; source-free candidate session IDs/missing selector fields for the explicitly requested project when needed |

A source-bearing `snapshot` has `target`, `document`, `sources` (D/R/B), `dirty`, `comparisons` (D/R, D/B, R/B), `agreement`, `consistency`, and `diagnostics`. See [data-model.md](../data-model.md) for field definitions and validity constraints. This contract and that model are normative; examples below are semantic projections, not alternate abbreviated wire schemas.

Exit codes:

- **0:** `complete_observation` or `not_open`. Callers must inspect the outcome and individual limitations; exit zero alone does not prove an open buffer, agreement, cleanliness, or edit safety.
- **2:** `limited_observation` or `unsupported_observation`.
- **3:** Target/source-access/input refusal: ambiguous, missing, invalid, unavailable editor, denied access, or invalid request.
- **4:** Disconnection, timeout, protocol error, or explicit cancellation before process termination.
- **1:** Unexpected host/launcher failure preventing the normal contract. Emit a safe diagnostic; emit a structured boundary failure when possible, never fabricated source.

The consuming caller must drain stdout in the controlled acceptance environment. Logs go to stderr, but stderr must contain only correlated safe metadata, not serialized requests, responses, tokens, source, or unrelated project content. Bootstrap JSON, help, and version output are not observation outcomes.

Retries are read-only and may be explicitly initiated by the caller, but every retry is a **new observation** with a fresh interval, identity checks, and reads. No automatic reconnect to a replacement editor and no use of an older snapshot as current evidence.

## 4. Completeness and failures

An open complete result requires observed target identity and open state, independently observed D/R/B, document-specific dirty state, and no detected invalidation. Dirty/divergent is a successful read. Observability of only D, or D=R, or an editor acknowledgment cannot satisfy the contract.

A valid confirmed closed script returns `not_open`, B/dirty not-applicable, and explicit available/unavailable D/R; it never loads/opens a document to fill the result. Unknown open state means limited, not closed. An open document whose file disappeared remains observable with D unavailable. Missing target without an open document is a distinct failure. Invalid GDScript syntax remains readable source, not an invalid-target error.

A per-source limit yields `unavailable`/`too_large` for that source only and preserves other independently observable facts, including their earlier valid evidence. For an otherwise valid known target, return `limited_observation` when open state is open or unknown, or `not_open` with D/R limitations when confirmed closed. Per-source limits alone never justify an operation-wide refusal; that requires a separate inability to perform the whole request safely or meaningfully. Existing refusal/interruption and invalidation precedence remains in force.

Source-level errors preserve independently observed unrelated facts. Refusal/interruption still controls the top-level outcome. Denied/ambiguous targets return no source; known disconnected editors are not mislabeled as timeouts. Invalidated evidence is kept only in the explicitly invalidated field and excluded from comparisons. A response from a different request/session/document is a protocol/identity failure, not usable source.

The result is an interval observation. `consistency.atomic` is always false; absence of detected changes is not proof of stability. Unknown staleness does not invalidate an independently read source. Conversely, text disagreement cannot identify which authority is stale.

## 5. Executable semantic examples

These compact vectors specify meaningful classification cases for contract tests. `sources` abbreviates only current observed text; a JSON null here means unavailable, **not an actual source value or the full wire record**. `dirty` is independently supplied editor evidence. `invalidated` identifies formerly collected facts that cannot enter current comparisons. The implementation must expand each vector into the full evidence model with actual identities/times/reasons.

```json
[
  {"case":"clean","open":"open","sources":{"D":"S","R":"S","B":"S"},"dirty":"clean","invalidated":[],"expect":{"outcome":"complete_observation","pairs":["equal","equal","equal"],"agreement":"agree"}},
  {"case":"dirty-divergent","open":"open","sources":{"D":"S","R":"S","B":"U"},"dirty":"dirty","invalidated":[],"expect":{"outcome":"complete_observation","pairs":["equal","different","different"],"agreement":"divergent"}},
  {"case":"dirty-equal","open":"open","sources":{"D":"S","R":"S","B":"S"},"dirty":"dirty","invalidated":[],"expect":{"outcome":"complete_observation","pairs":["equal","equal","equal"],"agreement":"agree"}},
  {"case":"dirty-unknown","open":"open","sources":{"D":"S","R":"S","B":"S"},"dirty":"unknown","invalidated":[],"expect":{"outcome":"limited_observation","pairs":["equal","equal","equal"],"agreement":"agree"}},
  {"case":"missing-R-with-known-divergence","open":"open","sources":{"D":"S","R":null,"B":"U"},"dirty":"dirty","invalidated":[],"expect":{"outcome":"limited_observation","pairs":["unknown","different","unknown"],"agreement":"divergent"}},
  {"case":"closed-unloaded","open":"not_open","sources":{"D":"S","R":null,"B":null},"dirty":"not_applicable","invalidated":[],"expect":{"outcome":"not_open","pairs":["unknown","unknown","unknown"],"agreement":"unknown"}},
  {"case":"closed-cached","open":"not_open","sources":{"D":"S","R":"S","B":null},"dirty":"not_applicable","invalidated":[],"expect":{"outcome":"not_open","pairs":["equal","unknown","unknown"],"agreement":"unknown"}},
  {"case":"empty-observed","open":"open","sources":{"D":"","R":"","B":""},"dirty":"clean","invalidated":[],"expect":{"outcome":"complete_observation","pairs":["equal","equal","equal"],"agreement":"agree"}},
  {"case":"line-endings","open":"open","sources":{"D":"x\r\n","R":"x\n","B":"x\n"},"dirty":"clean","invalidated":[],"expect":{"outcome":"complete_observation","pairs":["different","different","equal"],"agreement":"divergent"}},
  {"case":"invalidated-buffer","open":"open","sources":{"D":"S","R":"S","B":"U"},"dirty":"dirty","invalidated":["B"],"expect":{"outcome":"limited_observation","pairs":["equal","unknown","unknown"],"agreement":"unknown"}},
  {"case":"open-unknown","open":"unknown","sources":{"D":"S","R":null,"B":null},"dirty":"unknown","invalidated":[],"expect":{"outcome":"limited_observation","pairs":["unknown","unknown","unknown"],"agreement":"unknown"}}
]
```

Pair order is D/R, D/B, R/B. Additional refusal/transport/session cases are exercised by [quickstart.md](../quickstart.md), not implied by these pure-classification vectors.

## 6. Compatibility

Version 1 is the first proposed contract; there are no legacy aliases to preserve. Adapters may add optional non-semantic metadata only when existing consumers can safely ignore it. Changed outcome precedence, authority attribution, dirty semantics, source encoding, required fields, or removal/renaming of outcomes requires a new schema version, explicit capability/version negotiation, and migration documentation. Never preserve unsafe behavior for compatibility.

## 7. Implemented T001 Rust library

The single `mcp-server/` package exports `godot_agent_kit::observation`. The [public contract tests](../../../mcp-server/tests/observation_contract.rs) construct complete typed evidence, including all §5 vectors. Generate local API documentation with `cargo +1.98.1 doc --no-deps --locked` from `mcp-server/`.

1. Build `ObservationRequest` from checked `RequestId`, `ProjectRoot`, optional `SessionId`, and `ResourcePath`. Roots/locators receive lexical validation only. A syntactically valid non-GDScript resource locator can be represented so attributable document evidence can produce `InvalidTarget`, rather than conflating document type with malformed request syntax.
2. After independently proving unique authenticated selection, call `ResolvedTarget::for_request` with canonical project identity, session, and engine version. The target retains a private request binding; another request cannot reuse that target/evidence, even in the same editor session. This constructor does not perform authentication or filesystem identity checks.
3. Construct `DocumentState`, D/R/B `Sources`, and `DirtyObservation` from independently acquired facts and witnesses. `DocumentFact::Unknown` represents missing validity/open-state knowledge; it cannot stand in for observed closed state. `DecimalCounter` preserves decimal IDs/ticks without floating-point conversion. `CollectionStamp` checks same-clock tick order and received elapsed time; R/B/dirty/open stamps must name the selected editor clock, while D uses the caller clock. Wall-clock endpoints can move backwards without overriding monotonic elapsed time.
4. Supply `Recheck::performed` only after all relevant checks actually ran, with every detected change; otherwise supply its explicit unavailable reason. `SourceObservation::invalidate` moves owned text into invalidated evidence without copying or replacing other authorities. A later limit preserves earlier invalidated evidence and its known change. Identity replacement invalidates identity-dependent facts; closure during an open read never manufactures closed-document success.
5. Consume the request, interval, optional selected target/evidence, terminal signals, and safe diagnostics with `ObservationOutcome::classify`. It returns validated, read-only-accessor outcome/snapshot objects or `EvidenceError`. Reject a failed validation at the adapter: do not publish the rejected payload. Wire version/type/duplicate-field validation, acquisition, authentication, scope confinement, and actual deadlines remain adapter/integration obligations.

`SourceObservation::observed` consumes exact UTF-8 text. At more than 512 KiB it reports only that authority unavailable/`TooLarge`; no truncation, empty substitution, or whole-operation refusal. D/R cannot be not-applicable; B/dirty can only be not-applicable for an observed closed document. Exact current texts drive comparisons, never dirty state or staleness. Known stale content requires independent `StalenessEvidence::unapplied_change`: a known incorporated revision and a newer revision of the **same referenced authority**, with attributed evidence. It is not a comparison of unrelated D/R/B counters or a cached read. Without that causal evidence, use unknown staleness.

Refusals/interruption outrank success. When several terminal signals are known, the reducer uses: denial, ambiguity, invalid request, protocol error, cancellation, disconnection, missing target, invalid target, operation-wide unsupported, unavailable editor, timeout. Missing/invalid outcomes require matching attributable document evidence; a signal cannot override a valid open document. Recheck evidence of session loss or deadline expiry contributes its terminal cause, and cancellation does not conceal known live-evidence invalidation. Denial/ambiguity suppress all source, including invalidated text. Protocol failure retains earlier separately validated partial evidence, never the malformed/mismatched payload. Diagnostics use fixed safe messages/actions and retain known causes without source interpolation.

Without a stronger outcome, valid confirmed closed state with applicable rechecks yields `NotOpen` even when D/R are limited or changed. Complete open observation requires every required current fact and recheck, with no detected change. Dirty/divergent observations can be complete; unavailable rechecks prevent complete/not-open success. No result claims atomicity, future stability, mutation authorization, or actual live-editor acquisition. See [T001 verification evidence](../quickstart.md#21-t001-native-evidence-2026-09-26).

## 8. Implemented T002 source-free routing

`godot_agent_kit::target::resolve(&ObservationRequest, &Path, Instant)` validates
project/locator metadata and the owner-private registry, authenticates matching
candidates, and returns `SelectedSession` only after unique selection.
`SelectedSession::target()` exposes canonical project/device/inode identity,
the exact session, engine and locator; `capabilities()` exposes five booleans.
The authenticated socket and pinned directory stay private and are dropped with
the selection. No source/container bytes are opened or returned by this API.

`RoutingFailure` contains core `OutcomeKind`, a safe `Diagnostic`, and optional
`Selection`. Authentication denial outranks ambiguity; established ambiguity
outranks lower-priority protocol/version/timeout failures. Unknown candidate
identity/liveness is not discarded to select a convenient survivor. Exact ended
IDs never fall back to a replacement. Private endpoint, secret, nonce, proof and
raw parser payloads do not enter these error objects or their debug/display output.

`project_fs::init_registry(&Path)` creates/verifies the explicit directory and
returns its canonical path. It never repairs unsafe existing permissions or ACLs.
T002 introduced the three forms below; T003 also implements the observation invocation in §1:

```sh
observe-gdscript init-registry --registry /absolute/private/registry
observe-gdscript --help
observe-gdscript --version
```

Bootstrap emits `{"schema_version":1,"status":"ready"}` and exits 0 on success;
unsafe metadata emits source-free `denied_access`/`unsafe_registry` JSON and
exits 3. Invalid/duplicate/missing flags emit `invalid_request` and exit 3.
The addon must separately be installed/enabled with the same registry environment
path, outside its project. T004 installs the source collector on that boundary.

The initial filesystem/ACL integration is macOS-specific and fails closed when
ACL observability is not implemented. The supplied monotonic deadline bounds
network work and is checked between filesystem operations; it is **not** T003's
process-isolated guarantee against blocked filesystem I/O. Selection authenticates
an editor lifetime, not a future-stability or script-observability promise.
All collector capabilities are false in T002.

See [live/native/export evidence](../quickstart.md#22-t002-source-free-boundary-evidence-2026-09-26).

## 9. Implemented T003 bounded execution

The real observation invocation in §1 now parses exact selectors, generates a fresh
request ID, and emits the full version-1 result plus newline. Required top-level
nullable fields are always present. A source's current `text` and `collection`
appear only when observed; former text remains only under `invalidated_evidence`.
The structured exit-code contract in §3 is implemented, including host failure 1
and SIGINT/SIGTERM cancellation 4. No public executable, worker, timeout, or force
override flag exists. The private worker entrypoint rejects ordinary shell pipes.

`runner::AttemptClock::start()` runs before flag parsing or resolution.
`runner::run` launches the same executable over an inherited private Unix socket
pair. Only the worker performs filesystem/editor acquisition. The supervisor
accepts typed, request-bound events until 4.5 seconds, classifies once using T001,
and terminates only its owned worker. A shared nonblocking reaper does not delay
result delivery on a blocked child. The five-second claim applies to the controlled
consuming caller, not an arbitrary blocked stdout sink or a hard-real-time OS.

`project_fs::read_disk(&SelectedSession, Instant)` and
`recheck_disk(&SelectedSession, &SourceObservation, Instant)` are blocking library
boundaries, used inside that worker. The selected handle cannot be constructed
without T002's authenticated unique selection. D reads use the pinned directory,
strict UTF-8, regular-file checks, a 512-KiB limit, and fresh confinement/content/
identity checks. BOM, empty text, whitespace, and line endings are preserved.
Built-in containers and non-GDScript external locators never supply script D.
Missing, unreadable, invalid-UTF-8, and oversized D remain per-surface limitations;
unsafe scope suppresses all source. There is no second disk-only CLI operation.

The wire boundary validates editor and worker evidence before retaining it:
identities, clocks, duplicate fields, source applicability, invalidated evidence,
and bounded framing/collections. Separately emitted editor and disk recheck events
prevent a later blocked disk read from erasing already detected editor changes.
`Recheck::Partial` records those changes while final `checks` remains `unavailable`.
Rust consumers with exhaustive `Recheck` matches must handle this new variant;
the JSON version, outcome names, and existing availability semantics remain v1.

At T003 delivery, the then-current T002 addon advertised `observe_gdscript: false`.
The caller returned `unsupported_observation` with an authenticated target and
null snapshot, without reading D. Its controlled-peer tests established execution
and result semantics, not actual R/B/dirty observability. T004 adds that real
collector without changing the caller flags or public version-1 outcomes.
See [T003 evidence](../quickstart.md#23-t003-executor-boundary-evidence-2026-09-26).

## 10. Implemented T004 clean-open observation

The enabled addon now serves the standard request through actual editor getters.
It independently samples the existing GDScript's `source_code`, the attributed
CodeEdit's text, and document-specific `get_unsaved_files()` evidence. Rust then
reads confined D, requests an editor recheck, independently rechecks D, and runs
the same core reducer. The observer never opens/selects/loads/saves a document.

An open-script/editor array mismatch, duplicate/empty path, unsupported control,
or unstable association cannot manufacture B or clean state. Applicable source
limits remain independent: exactly 512 KiB is readable; above that, only that
source becomes unavailable/`too_large`. Oversized text is not retained or hashed
for a synthetic revision. Rechecks compare originally observed text, document
identity/order, buffer versions and attributable dirty state; changed facts are
invalidated by the core. No atomicity, mutation permission or future stability
is claimed.

Rust integration consumers must now retain `bridge::wire::EditorSample.collection`
and pass that stamp to `wire::recheck` between the request and clock arguments.
Every editor fact must fall within its sample's editor-clock interval; the
recheck must start no earlier than the sample finishes. Private worker events
carry the sample interval through supervision. Update callers directly; no old
signature alias is retained. Public JSON fields, exit codes and outcome meanings
remain version 1.

Godot object IDs are encoded as unsigned decimal strings without floating-point
conversion, preserving the high bit of reference-counted Script IDs. An
authenticated addon scope refusal is source-free `denied_access`, including when
it occurs after a valid sample; no earlier source survives that denial.

The `clean-open` driver owns all opening, presentation and synthetic preparation.
Its independent disk/Script/CodeEdit/dirty/history witnesses are not obtained from
the product collector. See [T004 evidence](../quickstart.md#24-t004-clean-open-evidence-2026-09-26).
This is a development slice, not full-feature or supported-version acceptance.

## 11. T005 dirty/divergent and changing-document observation

The caller and version-1 result contract are unchanged. Independently readable
dirty or divergent D/R/B can still yield `complete_observation`. Every pair uses
the exact independently read values, including whitespace/line endings. Equal
text never overrides editor-reported dirty state. Native editor activity may
copy B into R; the observer reports actual R, not an expected copy of D or B.
Staleness remains unknown without independently attributable causal evidence.

Unavailable document-specific unsaved evidence yields dirty `unknown` with
`dirty_attribution_unavailable`, preserving observed sources and open state.
An unassignable global indication does not become target ambiguity, a dirty or
clean inference, or an editor disconnection. Mixed/unsupported or nonunique
script/editor associations withhold B/dirty rather than guess a buffer.
Multiple already-loaded Scripts at the requested path make R
`unavailable`/`resource_unreadable`, not `resource_not_loaded`.

Collection and recheck retain the original document identity. Same-document
changes invalidate affected facts only. A renamed or replaced document
invalidates identity-dependent facts; a detected close invalidates the original
open/buffer/dirty facts instead of manufacturing a `not_open` snapshot. Former
text remains only in `invalidated_evidence` and never enters current comparisons.
Closing a tab can free its native editor/buffer nodes; the collector checks those
references without assigning a freed object to a typed Node variable.
The held Resource is rechecked independently of tab closure: a separately
detected R change is invalidated alongside the closed buffer/open/dirty facts.

The disposable acceptance bridge can restrict attribution or prepare a native
transition immediately before recheck. These helpers live under the fixture
driver's excluded tree; no restriction/transition option exists in the product
caller or bridge. See [US2 evidence](../quickstart.md#25-t005-dirty-and-changing-document-evidence-2026-09-27).
These observations grant no mutation permission or whole-feature support claim.

## 12. T006 source-bearing routing and interruption outcomes

The production caller, core and version-1 contract are unchanged. Native
caller/worker regressions and independent real-editor acceptance now exercise
the existing integration with distinguishable same-named projects, identical
script paths, and concurrent sessions of one project. Exact selection binds
every source/dirty/identity fact to the selected lifetime; omitted selection
with two live sessions returns only the required `session_id` selector and
candidate IDs, never a source-bearing snapshot.

Absence before authentication yields `editor_unavailable`. Disable/re-enable
and restart generate a new session and secret; requesting the ended ID never
substitutes the replacement. An unclean process termination may leave private
metadata behind. That descriptor is not liveness evidence: a dead endpoint is
unavailable, while a rebound listener without the original secret is denied
with `authentication_failed`, before source acquisition.

Known post-authentication loss yields `disconnected_editor`. Before a validated
sample, no source can be returned. After sample/D collection, independent D
remains attributed to its original file/request, while R/B, open state and
dirty evidence whose live currency ended move to `invalidated_evidence`.
Their former text cannot enter current comparisons. Silence without known loss
yields `timeout`, retaining the original interval's usable facts with
`checks: unavailable` and `deadline_exceeded`, not fabricated disconnection.
Malformed or mismatched replies yield `protocol_error`; only earlier validated
facts survive. Denial at recheck suppresses all earlier source.

The supervisor does not wait on a suspended editor or stopped owned worker,
does not terminate editors, and never promotes queued late events after a
terminal result. A subsequent successful attempt is caller-initiated and has
fresh request identity and evidence. No interrupted/failed recheck produces
complete/not-open success. See [US3 evidence](../quickstart.md#26-t006-routing-and-interruption-evidence-2026-09-27)
for actual stages, deadlines, privacy, export checks and limitations.
