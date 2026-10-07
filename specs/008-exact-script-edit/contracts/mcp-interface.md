# MCP Interface Contract — Schema 2

**Status:** Implemented and accepted through the clean public Schema 2 cutover; [both-client and cumulative evidence](../quickstart.md#t003-composed-and-cumulative-acceptance-2026-10-07) is current. [Feature 007's Schema 1 contract](../../007-mcp-script-workflow/contracts/mcp-interface.md) remains historical; its unchanged protocol, carrier, discovery/read semantics, timing and effect/disclosure guarantees are inherited here.

## Public catalog

Advertise exactly these tools in the existing order:

| Name | Description | Annotations |
| --- | --- | --- |
| `discover_scripts` | Find supported scripts in a project and report discovery coverage. | readOnlyHint=true; openWorldHint=false |
| `read_script` | Read script source, current editor state and an edit revision without opening the script. | readOnlyHint=true; openWorldHint=false |
| `edit_script` | Replace exact text in a script using its current read revision. The script stays open or closed as observed. | readOnlyHint=false; destructiveHint=true; idempotentHint=false; openWorldHint=false |

No fourth tool, title/instruction block, new mode or lifecycle/history/Save operation is added. Hints and prose are not permission or safety enforcement. Discovery is optional for known targets.

## Input schemas

JSON Schema 2020-12 objects retain `additionalProperties: false`, exact required types and strict runtime decoding. Every input field below is a string. Missing/wrong-type/null values are invalid, including explicit null for the optional session.

| Field | Applies to | Schema / runtime rule | Parameter description |
| --- | --- | --- | --- |
| `project_root` | Required on all three | Nonempty; existing absolute project selector and 1024-byte cap | Absolute project directory. |
| `session_id` | Optional on all three | `^[0-9a-f]{32}$`; omission requires unambiguous authenticated selection | Editor session ID; omit only when selection is unambiguous. |
| `script_path` | Required on read/edit | Nonempty; existing exact confined locator and 2048-byte cap | Exact project script locator, such as res://scripts/player.gd. |
| `revision` | Required on edit | `^sr1:[0-9a-f]{64}$`; must match fresh admissible state | Revision returned by read_script for this target. |
| `old_string` | Required on edit | String; may be empty; at most 524288 UTF-8 bytes, supported text representation | Exact text occurring once, including overlaps. Empty only to replace a completely empty script. |
| `new_string` | Required on edit | String; may be empty; same fragment bound/representation | Replacement text; empty deletes the matched text. |

Required lists are exactly:

- Discover: `project_root`.
- Read: `project_root`, `script_path`.
- Edit: `project_root`, `script_path`, `revision`, `old_string`, `new_string`.

Fragment byte/representation checks belong to the trusted decoder/type, not a misleading character-count schema. The [model bounds](../data-model.md#bounds-and-exactness) are binding. No `minLength: 1` applies to either fragment. Fragment syntax need not be a valid script; the complete original/intended sources undergo existing admission/validation. Reject NUL, CR and U+FEFF without repair.

Reject `replacement_source` alone or mixed with the new fields, `replace_all`, `mode`, occurrence indexes, force, timeout, retry, basis objects and all other unknown fields. Duplicate fields fail in the strict JSON parser before map construction; no last-value-wins decoding. A malformed call/JSON-RPC envelope retains its existing protocol error; a valid tools/call with invalid arguments returns the existing source-free tool `invalid_arguments` error without editor dispatch.

## Workflow and example

Use `read_script` to obtain exact source, a non-null opaque revision and useful trusted state. Submit one literal pair, not the read object or an internally constructed revision. If read cannot issue a revision, its safe next action applies; guessing text does not make the target editable.

For a script containing one `const SPEED: int = 17`, replacing that span with `const SPEED: int = 23` leaves all surrounding bytes untouched. The complete illustrative edit request below uses synthetic selectors and a syntax-only revision; an actual call must use its own fresh read value:

```json
{
  "jsonrpc": "2.0",
  "id": "edit-1",
  "method": "tools/call",
  "params": {
    "name": "edit_script",
    "arguments": {
      "project_root": "/fixture/project",
      "script_path": "res://scripts/player.gd",
      "revision": "sr1:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      "old_string": "const SPEED: int = 17",
      "new_string": "const SPEED: int = 23"
    }
  }
}
```

Empty new text deletes a unique nonempty span if the complete result is admissible. To add text to a nonempty script, retain a unique nonempty anchor in new text. To replace an existing complete empty script, use empty old text and the intended new text after a fresh read. This creates neither a file nor an editor document. Unavailable source is not empty. Equal old/new requires the same safety, uniqueness/empty-source condition, validation and verification as other intents.

## Structured results and version advertisement

Every tool's advertised `outputSchema` and every emitted tool object uses the same exact root fields:

| Field | Contract |
| --- | --- |
| `schema_version` | Integer constant `2`; public tool-surface version, not protocol/native/local-operation version. |
| `operation` | Constant `discover_scripts`, `read_script` or `edit_script` for that tool. |
| `request_id` | Existing checked domain correlation, not JSON-RPC identity or replay authority. |
| `result` | Existing operation-specific record or null. |
| `error` | Existing typed adapter error or null; exactly one of result/error is non-null. |

Discovery retains its v1 inventory/coverage record, including its separately owned inner version. Read retains exactly `source`, `revision`, `state`, including empty/unavailable/invalidated and source-provenance distinctions. No revision format or read semantics change. Edit retains `mode` and the existing open/closed/pre-dispatch outcome projections: evidence, stages, effects, synchronization, validation, history, lifecycle, revision meaning and limitations are unchanged except for the new matching refusal reasons below.

`structuredContent` remains authoritative and schema-conforming. `content` remains one terse summary/action derived from that same result, without serialized full-object duplication or input source. `isError=true` for matching refusals and all existing genuine errors/non-success, not for informative dirty/closed reads. Transport acceptance and response delivery are not mutation success.

### Matching refusals

All rows require fresh authentication/confinement, current revision and the [applicable no-effect safety check](execution.md#refusal-precedence). Earlier or later safety/freshness/validation/disclosure/effect failures take precedence.

| `reason` | Meaning | Outcome-time guidance |
| --- | --- | --- |
| `no_match` | Nonempty old text has zero literal occurrences. Already-present new text is irrelevant. | Read the script again and quote its exact current text. |
| `ambiguous_match` | At least two starts, including overlaps; no occurrence chosen. | Read again and use a larger span that occurs once. |
| `empty_old_string` | Empty old text against nonempty complete current source, including whitespace-only source. | Read again and use a nonempty span for this script. |
| `invalid_source` | Derivation cannot produce a bounded supported complete source, including combined size overflow. | Read again and submit a replacement within the supported source bounds. |

They use `stage: matching`, `outcome: refused`, `application: not_applied`. Preserve the actual checked open/closed result mode, evidence and no-effect lifecycle/history facts; the internal unchanged eligibility check is not a published successful edit. Open results retain `safe_next_action: fresh_read`; closed results retain `next_action: {kind: fresh_read}`. Basic acquisition failures may still have `mode: undetermined` with the existing action shape. No new shared next-action enum, result variant, native reason or private-wire revision is required merely to express these reasons.

No match count, offset, candidate text, excerpt, fragment, derived complete source, private expected state or extra hash is disclosed. Existing permitted source-free execution evidence retains its meaning. Downstream invalid-script/unsupported-context errors keep their existing reasons and stages, not this table's `invalid_source`. Malformed/over-bound/unsupported fragment requests remain `error.category: input`, `code: invalid_arguments`, `stage: validate_request`, `application: not_applied`, `next_action.kind: correct_request`.

After possible effects, timeout/cancellation/disconnection/output failure retains known/partial/unknown application. No match reason may erase uncertainty. A deliverable result can request a fresh read; a missing response cannot prove rollback, non-application or permission to replay.

## Unchanged protocol and operational boundary

Retain local stdio MCP **2025-11-25**, the existing initialization/discovery compatibility handling, tools-only capability, three-entry unpaged catalog, bounded strict transport, single admitted tool/no queue, per-call cancellation and original operation clock. Retain protocol-only stdout, source-free incidental diagnostics, SDK logging restrictions and current response/backpressure handling.

Schema 2 is discoverable through ordinary tool output schemas/results. Keep package/executable version **0.1.0**, MCP negotiation, `sr1`, local v1, private bridge v6 and native revision 4 independent. No release/package bump, server-name change, hidden capability bit or client-name carrier branch is selected. The executable configuration and source-free validator preparation remain unchanged.

## Migration

On Schema 2 implementation delivery:

1. Refresh `tools/list` and update the expected root output schema version to `2` for all three tools. Reconnect clients that cache the catalog.
2. Remove `replacement_source` and any legacy/mode fallback. Do not automatically translate an old in-flight request.
3. Perform a fresh `read_script` on the original explicit target. Use its exact source and non-null revision.
4. Send one exact `old_string`/`new_string` pair. For a genuine whole-file change, the complete nonempty read source may be the unique old span. For complete empty source, old text is empty under the same safety rules.
5. Consume the structured result. After a refusal or lost response, obtain fresh state before any separate intentional edit. No fallback filesystem write, silent retry or open/close workaround.

All current in-repository MCP callers/examples/prompts must migrate in the implementation cutover, including negative legacy-only and mixed-form coverage. Existing local whole-source Rust/CLI/private contracts remain distinct and unchanged. Feature 007 artifacts remain Schema 1 historical design/evidence, not a second supported MCP mode or proof of Schema 2 compatibility. Both actual clients require the new [acceptance evidence](../quickstart.md).
