# Observation Caller Contract — Version 1

**Status:** T001 implements the reusable Rust semantic library described in §7. The CLI/JSON and bridge interfaces remain planned; no caller executable exists yet. This is **not MCP**. [Data model](../data-model.md) defines the normative result fields, evidence variants, comparisons, and outcome precedence; [bridge contract](bridge-protocol.md) is private integration.

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
