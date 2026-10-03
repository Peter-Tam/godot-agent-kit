# MCP Interface Contract — Schema 1

**Status:** Selected planning contract; no server or client compatibility is implemented/proved by this document. [Model](../data-model.md) defines checked state; [execution](execution.md) defines trusted ownership. The following catalog text is agent-facing; the detailed protocol/engineering rules below it are not copied into tool descriptions.

## Public catalog

Advertise these tools in this fixed order and no other product tools:

| Name | Static description | Annotations |
| --- | --- | --- |
| `discover_scripts` | Find supported scripts in a project and report discovery coverage. | readOnlyHint=true; openWorldHint=false |
| `read_script` | Read script source, current editor state and an edit revision without opening the script. | readOnlyHint=true; openWorldHint=false |
| `edit_script` | Replace one script using its current read revision. The script stays open or closed as observed. | readOnlyHint=false; destructiveHint=true; idempotentHint=false; openWorldHint=false |

No title, long server instruction block, internal module names, repeated safety lecture or full failure catalog is required. Annotation hints are not permissions or safety enforcement. Discover is not required when the caller already knows the target. Native editor lifecycle/history operations are not hidden tool aliases.

### Input schemas

Every input is a JSON Schema 2020-12 object with `additionalProperties: false`. Required fields and types are exact below; optional session omission has its ordinary unique-authenticated-selection meaning. Explicit null is not a session ID. The material parameter descriptions are deliberately short.

| Field | Type / schema rule | Description |
| --- | --- | --- |
| `project_root` | Nonempty string; required on all tools | Absolute project directory. |
| `session_id` | Optional string matching `^[0-9a-f]{32}$` | Editor session ID; omit only when selection is unambiguous. |
| `script_path` | Nonempty string; required on read/edit | Exact project script locator, such as res://scripts/player.gd. |
| `revision` | String matching `^sr1:[0-9a-f]{64}$`; required on edit | Revision returned by read_script for this target. |
| `replacement_source` | String, including empty; required on edit | Exact replacement GDScript source. |

`discover_scripts` permits only project_root/session_id. `read_script` additionally requires script_path. `edit_script` additionally requires revision/replacement_source. No basis/prior-result object, force, lifecycle, timeout, retry, output-format, raw command or transport parameter exists.

Concrete static result definitions may share local `$defs`/`$ref`, not a metadata DSL or generator framework. Edit inputs do not reference the read-output schema or contain internal expected-state evidence. The core independently enforces exact UTF-8 byte, identity, source, applicability and stale-state rules; JSON Schema character lengths do not replace UTF-8 byte checks. Unknown/duplicate keys, excessive nesting and wrong types fail without dispatch. A revision's valid syntax alone grants nothing.

### Example tool call

This is a complete read request; the project/session are synthetic selectors, not defaults:

```json
{
  "jsonrpc": "2.0",
  "id": "read-1",
  "method": "tools/call",
  "params": {
    "name": "read_script",
    "arguments": {
      "project_root": "/fixture/project",
      "session_id": "00112233445566778899aabbccddeeff",
      "script_path": "res://scripts/player.gd"
    }
  }
}
```

A subsequent edit uses the explicit selectors, the read's non-null `result.revision` and `replacement_source`; it does not echo the source/state or complete tool object. The agent treats the revision as opaque, not something to decode or construct. A null revision means the read is informative but cannot supply an edit precondition. Fresh execution compares current state and returns `fresh_read` on stale state; it never accepts the revision as authorization.

For example, with `R` denoting the exact returned revision rather than a literal wire value:

```text
read_script(project_root, script_path, session_id?)
  -> result: {source, revision: R, state}
edit_script(project_root, script_path, revision: R, replacement_source, session_id?)
```

## Structured results

All three output schemas have the exact root fields:

| Field | Meaning |
| --- | --- |
| `schema_version` | Integer constant `1`, independently versioning this tool surface. |
| `operation` | Constant tool name for the particular output schema. |
| `request_id` | Generated checked domain correlation, separate from JSON-RPC ID. |
| `result` | The operation-specific record below, or null when no operation result can be produced. |
| `error` | The typed adapter error below, or null. Exactly one of result/error is non-null. |

Use discriminated schemas with required fields and explicit nullable states, not a bag of optional safety flags. Referenced existing contracts define preserved semantics; the deliberate read/edit projections below are not exports of private evidence layouts or arbitrary JSON. Public schemas model observable distinctions and meaningful bounds without explanatory prose on every leaf.

### Discover result

`result` is the existing [discovery-v1 record](../../004-discover-project-gdscript/contracts/discovery-api.md), preserving its correlation, interval, requested/resolved target, inventory scope/coverage/validity, consistency, diagnostics and source-free selection feedback. No source or edit basis is returned. Partial or interrupted inventory never implies undiscovered paths are absent.

### Read result

`result` has exactly `source`, `revision` and `state`, as defined by [ScriptReadResult](../data-model.md#4-trusted-script-read). Source is exact text or null, distinct from an observed empty string. Revision is the opaque precondition or null with a reason. State preserves trusted observation classification, permitted target/document identity, lifecycle, source provenance and applicable authorities, dirty attribution, comparisons/stability, interval, limitations and invalidated/unavailable facts.

Agreeing authority text is represented by its relation to `source`, not repeated source strings. Distinct or invalidated permitted evidence remains explicit; the main source is never silently chosen as mutation authority. Dirty, closed, unsupported or partly observable targets still return useful facts. No `ObservationOutcome`, `edit_basis`, native collection/epoch/file-revision tuple or full internal expected-state record is returned for the agent to copy into an edit.

### Edit result

`result` has exactly `mode` and `outcome`:

- `mode`: `open`, `closed` or `undetermined` before matching current-state selection.
- `outcome`: a public projection of the existing [open-edit outcome](../../002-edit-open-gdscript/contracts/edit-api.md), or its closed-source counterpart. Preserve actual outcome/reason/stage, application certainty, interval, permitted target/document identity, expected/before/after evidence meaning, validation, persistence/finalization, history, diagnostics and safe next action. Represent the caller precondition by its opaque revision and project meaningful evidence availability/agreement; do not export complete internal expected-state records, native IDs or collection bookkeeping merely because the core holds them. Existing local v1 output remains unchanged.

The closed counterpart additionally has `lifecycle: {admitted: "closed", observed_final: "closed" | "open" | "unknown"}` and explicit R applicability; B/history are `not_applicable_closed` only on confirmed absence. It reports the loaded-class-not-reloaded limitation when R is present. Open history and Feature 002 outcomes remain unchanged. The core's source-free edit evidence projections are reused; replacement source, private validation source and unrelated documents are not echoed.

### Adapter errors and next actions

`error` has exactly `category`, `code`, `stage`, `application`, `requested_target`, `resolved_target`, `next_action`. Categories are `input`, `admission`, `host`. Targets are null until safely attributable; failure cannot disclose candidate source or private routing data.

`next_action` is a discriminated object: `{kind: "correct_request"}`, `{kind: "select_session", candidate_sessions: [...]}`, `{kind: "fresh_read"}`, `{kind: "check_setup"}`, `{kind: "unsupported"}` or `{kind: "none"}`. Include only relevant permitted details. Underlying operation results retain their existing actionable reason/action fields instead of being rewritten into this error record.

Before dispatch, invalid tool arguments report `application: "not_applied"`. Busy at adapter capacity reports `server_busy`, not a falsely observed `editor_busy`. An unexpected host failure after possible dispatch reports retained effect certainty, or `unknown` when no such facts can be recovered; never a fabricated operation result. The client must obtain a fresh read of the original explicit target after possible effects before another intentional edit.

### MCP result carrier

Return the complete permitted public object in **`structuredContent`**, authoritative and conforming to `outputSchema`, including failure variants. The selected default `content` is one terse outcome/next-action text summary derived from that same typed result—not a serialized copy of the object, source or revision. It is not an alternative complete result and cannot make a text-only consumer compatible by itself.

The selected [MCP specification](https://modelcontextprotocol.io/specification/2025-11-25/server/tools#structured-content) says serialized JSON text **SHOULD** accompany structured content for backward compatibility; it does not require unconditional duplication. [Exact-client research](../research.md#7-public-contract-correction-evidence) establishes Codex's structured-content conversion but not completed two-client model use. Before claiming support, prove both selected clients can use source/revision/state and truthful failures through this carrier. If a required client demonstrably needs the full text object, record that case and the extra context/escaping cost, then adopt one identical serialization of the permitted public object as its necessary compatibility fallback. A summary-only or structured-only failure must not be called compatible merely for compactness.

No invented capability bit, client-name heuristic, `_meta`-only data, format tool parameter or generic carrier framework is selected. A required fallback cannot weaken outcomes, disclosure or the authoritative structured schema. Bound the actual complete response after any escaping; never silently truncate safety facts or source to fit.

`isError=true` for adapter errors and genuine operation refusals/failures/partial or unknown effects. Dirty/divergent successful observations, valid not-open reads and honest limited inventory/observation are not execution errors merely because they do not permit editing. Transport success or isError=false never replaces the structured outcome's meaning.

## Protocol and capability contract

Use MCP **2025-11-25 only** over local stdio. Explicitly configure the SDK's supported-version list and returned protocol version; do not inherit its 2026-07-28 default.

1. Initial state permits initialize and source-free protocol ping/version discovery only. No Godot operation is dispatched.
2. On initialize, negotiate the sole supported revision using the selected protocol's fallback rules and return server identity plus `capabilities: {tools: {}}`. Client inability to use that revision means disconnect, not imaginary compatibility.
3. Accept notifications/initialized. Tools are available only after successful selected-revision initialization; this adapter does not pretend the SDK itself enforces that state. Reinitialization or newer-era pre-init tool dispatch is rejected.
4. tools/list returns the fixed catalog without nextCursor. A supplied cursor is invalid, rather than a second catalog page. No listChanged, resources, prompts, tasks, completions, roots, sampling, elicitation or protocol logging is advertised.
5. A modern server/discover compatibility probe may report the sole selected supported revision and tools capability without editor access. It is protocol negotiation, not another product tool or advertised modern-era implementation. Other newer-revision operations fail explicitly.

Unknown JSON-RPC methods yield method-not-found; invalid envelopes yield invalid-request; malformed JSON yields parse-error when a complete frame can be answered; unknown tools/malformed tools-call envelopes yield invalid-params. A valid call with invalid tool arguments yields a source-free typed tool error and no dispatch. These protocol errors do not replace actual editor-operation failures after valid dispatch.

Schema/protocol negotiation never overrides operation capability. Catalog presence means server support; current editor/native compatibility and script eligibility are returned as structured operation information. No editor is launched or selected by current working directory/client roots.

## Connection framing and timing

[Model bounds](../data-model.md#2-supported-profile-and-bounds) apply. The adapter-owned transport is a bounded line collector/parser and writer using SDK message types and service machinery, not a second protocol implementation.

- Reserve stdout for newline-delimited UTF-8 JSON-RPC. Startup/help/errors and source-free diagnostics use stderr; normal server startup emits no incidental stdout.
- Enforce frame/depth/duplicate-key and bounded ID rules before the SDK can allocate unbounded request state. String IDs are at most 128 UTF-8 bytes; numeric IDs are exact signed integers representable by the SDK, never float-rounded. Unknown cancellation IDs are ignored; duplicate live IDs cannot replace existing ownership.
- Malformed complete JSON gets a source-free `-32700` response with null ID. Structurally invalid JSON-RPC uses `-32600`; unknown methods use `-32601`; invalid tools-call envelopes/unknown tools use `-32602`. Never echo offending data or parser line excerpts.
- Oversized, invalid UTF-8 or unterminated frames terminate the connection explicitly after source-free diagnostics; do not truncate into another valid call. A partially received frame has the fixed connection deadline; ordinary idle connections do not expire.
- Capture the operation clock when a complete bounded tools/call frame is accepted, before tool-specific decoding, routing and validation. Parsing/dispatch must not reset it. Pass that same clock to the existing runner budgets. Serialize and flush within its final delivery reserve with output consumed.
- One admitted tool runs off the protocol executor. A concurrent call is refused, not queued. The reader can handle its cancellation while work runs; bounded outstanding-ID/output state prevents unbounded SDK handler growth.
- A blocked writer has a finite delivery deadline. On failure/expiry, close the connection and cancel outstanding owned work without changing retained effects. No late response/replay store is added. Output backpressure tests are separate from consumed-output performance claims.
- Do not install a tracing subscriber that enables SDK message events or honor RUST_LOG for them. Disabling stdout logging alone does not prevent source leakage to stderr.

On notifications/cancelled, set only that call's cancellation flag and keep the bounded supervisor owned until terminal cleanup. The SDK can suppress delivery of its result; record delivery loss, not rollback. On EOF/SIGINT/SIGTERM, prevent new calls/stages, signal active work and wait only within its original cutoff. Bound runtime teardown to one second afterward; if a Tokio stdin reader remains blocked, exit the owned server process rather than claim that reader joined. Do not terminate Godot or reauthorize uncertain work.

## Executable and compatibility surface

The planned executable is `mcp-server/target/debug/godot-agent-kit-mcp --registry "$REGISTRY"`. Registry is required deployment configuration, not an agent-selected endpoint or tool parameter. Reuse the existing registry initialization, exact authenticated engine/validator selection and internal same-binary worker dispatch before MCP startup. No new editor-path, network, arbitrary worker, default-project or force option.

Implement `--help` and `--version` as no-operation invocations. Unknown/duplicate configuration flags fail before protocol/editor work. Keep the existing five local binaries and their schemas/exit meanings unchanged. This plan changes no Cargo file yet.

Schema 1 compatibility requires deliberate versioning/migration for future breaking fields, names or outcome meanings. A private bridge/native cutover does not silently change this public schema or add tool aliases. Concrete Codex/Claude support is claimed only after the [quickstart evidence](../quickstart.md) is actually produced.
