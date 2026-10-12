# MCP Interface: Additive Schema 2 Script Existence Tools

**Status:** Selected Phase 1 design, not implemented. The [Feature 008 contract](../../008-exact-script-edit/contracts/mcp-interface.md) remains authoritative for existing discover/read/edit input and result shapes, exact matching and empty-source behavior. This contract adds two tools; it does not rewrite that historical three-tool delivery.

## Catalog and protocol

The unpaged catalog order becomes `discover_scripts`, `read_script`, `edit_script`, `create_script`, `delete_script`. Preserve the first three tool definitions. Add only:

| Name | Description | Annotations |
| --- | --- | --- |
| `create_script` | Create a new project script from complete source without opening it. Existing targets are refused. | readOnlyHint=false; destructiveHint=false; idempotentHint=false; openWorldHint=false |
| `delete_script` | Delete a script using its current read revision. Clean target documents are removed; references are not repaired. | readOnlyHint=false; destructiveHint=true; idempotentHint=false; openWorldHint=false |

All five advertise and return public Schema **2**. Retain MCP **2025-11-25**, tools-only capability, local stdio framing/initialization, single admitted tool/no queue, cancellation, protocol-only stdout, SDK logging restrictions and structured-result carrier. Hints are not authorization. No new initialization instruction block, transport, user CLI, capability framework, lifecycle tool or hidden mode is added.

## Input schemas

Use JSON Schema 2020-12 objects with `additionalProperties: false`, strict string fields and exact required lists. Duplicate JSON fields fail before map construction; unknown, missing, null or wrong-type required values fail without dispatch. Explicit null for optional `session_id` is invalid.

| Field | Applies to | Rule | Parameter description |
| --- | --- | --- | --- |
| `project_root` | Both, required | Existing absolute authenticated project selector; at most 1024 UTF-8 bytes. | Absolute project directory. |
| `session_id` | Both, optional | `^[0-9a-f]{32}$`; omission requires unambiguous authenticated selection. | Editor session ID; omit only when selection is unambiguous. |
| `script_path` | Both, required | Exact confined supported standalone `res://` path ending in `.gd`; existing 2048-byte cap, no normalization. | Exact project script path, such as res://scripts/player.gd. |
| `source` | Create, required | Complete source, including admissible empty text; at most 524288 UTF-8 bytes; reject NUL/CR/U+FEFF without repair. Existing source/context validation follows. | Complete source for the new script. |
| `revision` | Delete, required | `^sr1:[0-9a-f]{64}$`; must match fresh admissible state for this target. | Current revision returned by read_script for this target. |

Required lists are exactly `project_root`, `script_path`, `source` for create and `project_root`, `script_path`, `revision` for delete. Source byte bounds are checked by the trusted decoder/type, not misrepresented as JSON Schema character counts. No `minLength: 1` on source. Structural source rejection is distinct from complete-script validation failure.

Reject create `revision`, delete `source`, `replacement_source`, `old_string`, `new_string`, `mode`, force, save/discard/close flags, recursive/directory options, timeouts, retries, basis objects and all other unknown fields. Malformed JSON-RPC retains its existing protocol error; a valid tools/call with invalid arguments uses the existing source-free `invalid_arguments` adapter error and `application: not_applied` before dispatch.

## New operation result objects

Keep the exact existing outer fields: `schema_version: 2`, tool-specific constant `operation`, checked domain `request_id`, `result`, `error`. Exactly one of result/error is non-null. Existing tools retain their inner shapes; only create/delete use the following new operation-owned record. All objects are closed schemas; every listed field is required, with nullable values explicitly indicated.

| Result field | Type and meaning |
| --- | --- |
| `target` | Existing checked `{project_root, script_path, session_id}` public selector projection; selected session can be null before selection. Never credentials/endpoints/candidates. |
| `outcome` | `verified_created` for create or `verified_deleted` for delete; otherwise `refused`, `applied_unverified` or `application_unknown`. |
| `application` | `not_applied`, `applied`, `partly_applied` or `unknown`, constrained by the [terminal model](../data-model.md#deterministic-terminal-reduction). |
| `reason` | Null on verified success; bounded source-free cause on non-success. Diagnostics retain finer-grained facts. |
| `basis_revision` | Null for create. For delete, the matched admitted `sr1` when safely established and publishable, otherwise null; never a newly minted post-delete revision. |
| `lifecycle` | `{before, after}`; each is `open`, `closed` or `unknown`, based on actual observations. |
| `stages` | `{admission, preflight, application, synchronization, verification}`; each `not_started`, `completed`, `failed`, `unavailable` or `not_applicable`. Completed application alone is not success. |
| `effects` | Create: exactly `{file_creation, source_persistence}`. Delete: exactly `{document_removal, resource_detachment, file_removal}`. Values are the [effect progress states](../data-model.md#effects-validation-and-verification). |
| `evidence` | `{before, after}`; nullable authority snapshots defined below. Null is not an absence observation. |
| `synchronization` | `{disk, resource, document}`; each `complete`, `incomplete`, `unavailable` or `not_applicable`. |
| `validation` | `{intended_source, current_context, post_source}`; each `{status, reason}` with status `valid`, `invalid`, `unavailable` or `not_applicable`, reason nullable. Scope is fixed by the field, not a whole-project validity assertion. |
| `verification` | `{disk, resource, document, protected_state, post_source}`; each `pending`, `verified`, `failed`, `unavailable` or `not_applicable`. |
| `history` | `{target, unrelated}`; target is `not_participating` or `target_document_disposed`, unrelated is `preserved`, `unverified` or `not_applicable`. No Undo capability follows. |
| `interval` | Existing `{started_unix_ms, finished_unix_ms, elapsed_us}`; monotonic elapsed governs deadlines, not subtraction of wall timestamps. |
| `diagnostics` | At most 64 bounded source-free `{stage, reason, code}` records; code may be null. Reuse applicable existing diagnostic meanings. |
| `limitations` | Bounded list of source-free outcome-relevant codes, not a static essay. Includes retained references/UID metadata, no reference repair/hot reload/Undo when relevant. |
| `next_action` | `{kind}`: `none`, `correct_request`, `correct_source`, `choose_absent_path`, `fresh_read`, `resolve_conflict`, `check_support`, `check_connection` or `inspect_effects`. Guidance never authorizes replay or force. |

An authority snapshot has exactly `disk`, `resource`, `document`, `interval` and `validity`. `validity` is `current`, `invalidated` or `unavailable`; acquisition is non-atomic. Authority records are:

- Disk: `{presence, identity, source, reason}`. Presence is `present`, `absent` or `unavailable`; identity is a permitted opaque string or null; source is `{sha256, utf8_bytes}` or null.
- Resource: `{presence, identity, association, source, dirty, reason}`. Same presence/nullable identity/source rules; association is `target`, `absent` or `unavailable`; dirty is `clean`, `dirty`, `not_applicable` or `unavailable`.
- Document: `{state, identity, source, dirty, current_version, saved_version, reason}`. State is `open`, `closed` or `unavailable`; identity/source nullable as above; versions are canonical decimal strings or null, never native object pointers/IDs. Dirty uses the Resource vocabulary.

All reasons are nullable bounded source-free codes. SHA-256 values are 64 lowercase hex; UTF-8 lengths are bounded nonnegative integers. Present but unreadable source is null with the actual reason, not empty. Absent/closed records have no source/identity/version and dirty is not-applicable where present in that record. Identities use existing opaque observation conventions; they are not client-provided handles or stale-write authority. The schema and trusted output validator enforce these combinations and the success predicates, not only field types. Deleted target source is never echoed. Create source is not repeated in either carrier; use a fresh `read_script` if source is wanted.

## Failure and delivery semantics

Distinguish occupied create, missing delete, missing/unusable/stale revision, changed identity/lifecycle, dirty/conflicting state, unsupported source/context/capability, denied/ambiguous routing, invalid/unavailable validation and interruption. Retain earlier causal and later safety/effect facts. Existing per-surface diagnostics distinguish stale Resource/buffer and unavailable observations; do not collapse them into a generic validation error.

- An occupied create, including equal or empty source, is an error with no successful unchanged branch.
- An absent delete is an error, not idempotent success. A stale request cannot delete a replacement lifetime.
- Any known document removal, Resource detachment or file effect survives a later failure as applied/partly-applied evidence. Unknown entry cannot become a definite not-applied result.
- If output decoding/serialization or host failure prevents an operation result, use the existing typed adapter error with retained effect knowledge, or `unknown` after possible dispatch. Extend this rule to both operations throughout transport and handler code.
- Disclosure denial suppresses unauthorized digests/identities/context even if earlier collection succeeded; preserve source-free effect facts. No source snippets, deleted text, validation staging paths, tokens or unrelated document details in results/logs.

`structuredContent` stays authoritative and schema-conforming. `content` is one terse summary/action derived from the same result, not serialized duplicate JSON or supplied source. `isError=false` only for verified create/delete success; all their refusals/unverified outcomes use true. A response accepted by MCP or consumed by a client is not mutation verification. A lost response is not rollback or permission to resend.

## Caller workflow and compatibility

For create, choose an explicit supported path with an existing parent and submit complete source. No prior read or discovery is required; the operation itself proves absence. For delete, freshly read the intended target and use its non-null revision. Refuse to guess when read cannot issue one. Both need the selected authenticated editor; neither falls back to raw filesystem access.

After implementation delivery, refresh `tools/list`; reconnect a client that caches the catalog. Existing Schema 2 discover/read/edit requests/results remain valid without migration. New callers must understand their new operation result, not treat a tool summary as success. Remove no existing tool, add no aliases, revive no Schema 1 editing, and do not automatically translate/retry an in-flight request. Package/executable version, MCP negotiation and `sr1` stay unchanged; private bridge/native compatibility is not a public schema version.

Both existing actual clients must exercise new operations and the composed create → read → exact edit → fresh read → delete → observe-absence workflow under the [quickstart](../quickstart.md). Raw protocol/SDK cases prove carrier behavior, not real-agent use or D/R/B safety.
