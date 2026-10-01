# Discovery Caller Contract — Version 1

**Status:** Proposed, not implemented. This is a local CLI/typed-library contract, not MCP. [Data model](../data-model.md) owns entities, bounds, evidence and terminal precedence; [private bridge](bridge-protocol.md) owns editor integration. Existing public observation/edit/open v1 remain unchanged.

## 1. Invocation

One operation:

```sh
discover-gdscripts --registry "$REGISTRY" --project "$PROJECT" --session "$SESSION"
```

- `--registry`: required canonicalizable absolute path to the existing owner-private operational registry. It must also be supplied to the addon via `GODOT_AGENT_KIT_REGISTRY`. Reuse `observe-gdscript init-registry --registry "$REGISTRY"`; do not add a second bootstrap command or expose token/endpoint flags.
- `--project`: required absolute project root, validated/canonicalized through the existing confinement and authenticated selection rules. No cwd, focus, display-name or first-session default.
- `--session`: optional exact session ID. Omission is permitted only when the shared source-free selection algorithm establishes exactly one authenticated live candidate; unresolved candidates are not discarded to manufacture uniqueness.
- `--help` and `--version`: no discovery and no editor/project access.

No script/path-prefix, glob, source query, include-ignored, offline, timeout, force, continuation, executable or hidden worker flag is public. Unknown/duplicate flags, missing values, invalid root/session syntax and positional input produce `refused/invalid_request`; no inventory is acquired. The CLI starts its clock before parsing and generates a fresh nonsecret request ID. It does not open a returned script automatically.

The implementation's private same-executable worker mode is not a public interface and must reject an ordinary shell invocation without its owned IPC setup. SIGINT/SIGTERM use the existing bounded cancellation convention, not a crash or a promise of current evidence.

## 2. Request and typed library

Semantic request projection (not an additional stdin-JSON mode):

```json
{
  "schema_version": 1,
  "request_id": "example-discovery-1",
  "project_root": "/fixture/project",
  "session_id": "00112233445566778899aabbccddeeff"
}
```

The current Rust consumer constructs checked `script_discovery::DiscoveryRequest` and calls `runner::discovery::run`, receiving an immutable `DiscoveryOutcome` or the existing style of recoverable host failure. Registry location and cancellation/clock plumbing remain runner inputs, not domain permissions. Only types/accessors needed by that consumer are public; no attempt/witness/codec APIs are exported speculatively.

Request syntax is not selection. The operation must prove one authenticated project/session before enumerating names. No dummy script is supplied to the old observation request to obtain a root handle. Discovery locators retain exact case/Unicode and existing lexical constraints; they do not carry a source revision or authorize later use.

## 3. Terminal results

Emit exactly one UTF-8 JSON object plus newline on stdout. The object contains every top-level field in the data model: `schema_version`, `request_id`, `outcome`, `interval`, `requested_target`, `resolved_target`, `inventory`, `diagnostics`, `selection`. Required unavailable fields are null, not omitted. Normal results are bounded to 6 MiB plus newline. The consuming caller must drain stdout within the controlled timing environment.

| Outcome | Exit | Meaning |
|---|---|---|
| `complete_listing` | 0 | The admitted scope was actually exhausted and required root/path/editor checks completed with no known gap/invalidation. Empty entries can establish an empty scope. |
| `limited_listing` | 2 | Coverage, supported representation or required rechecks are incomplete/invalidated. Safely attributed entries may remain; zero entries does not prove no scripts exist. |
| `refused` | 3 | Invalid/ambiguous/unavailable/unauthorized/unsupported target or busy admission. Distinct structured reasons remain available; no candidate or unauthorized inventory. |
| `interrupted` | 4 | Protocol loss, cancellation, session replacement/disconnection or deadline prevented completion. Only previously validated permissible evidence may remain, explicitly earlier and incomplete. |
| Unexpected host/launcher failure | 1 | The normal contract could not be completed. Emit a fixed safe diagnostic and a structured boundary result when possible; never an invented empty inventory. |

Reasons and simultaneous-failure precedence are normative in [data model §6](../data-model.md#6-terminal-precedence-reasons-and-retention). An exit code alone does not tell whether a needed script is absent or any later mutation is eligible. Every controlled normal request returns within five seconds, including a blocked filesystem/editor. Do not wait beyond the deadline for cleanup acknowledgment; late messages cannot upgrade a delivered result.

Only the authorized requested result can contain its inventory. Stderr and tracing contain bounded correlation/stage/reason data, never serialized requests/results, script paths/inventory, source, tokens, proofs or raw filesystem errors. No inventory is persisted in session descriptors. Requested validated root identity in the result is not candidate-source disclosure.

## 4. Complete semantic example

This is synthetic example data, not an executed result. The object is the full public envelope; the path observations do not imply editor-buffer or mutation state.

```json
{
  "schema_version": 1,
  "request_id": "example-discovery-1",
  "outcome": "complete_listing",
  "interval": {"started_unix_ms": 1000, "finished_unix_ms": 1012, "elapsed_us": 12000},
  "requested_target": {
    "project_root": "/fixture/project",
    "session_id": "00112233445566778899aabbccddeeff"
  },
  "resolved_target": {
    "request_id": "example-discovery-1",
    "project_root": "/fixture/project",
    "project_file_id": {"device": "1", "inode": "2"},
    "session_id": "00112233445566778899aabbccddeeff",
    "godot_version": {
      "version": "4.7.2.stable.official.ed1daf0bf",
      "hash": "ed1daf0bf001b61586d9930840f2f1394092c079"
    }
  },
  "inventory": {
    "scope": {
      "policy": "godot_project_files_v1",
      "project_data_directory": "res://.godot",
      "exclusions": ["dot_names", "project_data_directory", "gdignore_subtrees", "nested_projects", "non_gdscript_files", "non_regular_files", "embedded_or_unsaved_documents"]
    },
    "entries": ["res://actors/player.gd", "res://ui/player.gd"],
    "collection": {"clock_id": "caller", "started_tick_us": "2000", "finished_tick_us": "9000", "received_elapsed_us": 10000},
    "coverage": "complete",
    "validity": "observed",
    "consistency": {"recheck": "completed", "stability": "unknown", "atomic": false},
    "visited_entries": 8,
    "visited_directories": 3
  },
  "diagnostics": [],
  "selection": null
}
```

The schema rejects unknown or duplicate fields where a machine-readable input/private result is decoded, wrong primitive types, mismatched request/session/root, out-of-profile paths, unsorted/duplicate terminal entries, contradictory completeness and over-limit collections. Public output must satisfy the same semantic invariants even though JSON serialization itself succeeds.

## 5. Behavioral contract vectors

| Situation | Required interpretation |
|---|---|
| Empty stable visible scope; all checks completed | Complete, entries empty, no fabricated missing-observation reason. |
| Only dot/`.gdignore`/nested-project/data subtrees contain scripts | Complete empty is allowed because exclusions were established, not because enumeration failed. |
| Visible file is ignored by VCS or excluded from export | Include it; neither filter changes this scope. |
| A `.GD` script is visible | Include its exact round-trippable location; do not widen the separate opening/editing profile or promise support there. |
| One unreadable in-scope directory, others enumerated | Limited with safe other entries and `directory_unreadable`, not complete or project-empty. |
| Name cannot be represented by existing locator syntax | Limited with `unsupported_path` at a safe parent; no normalization, byte-escape alias or raw untrusted filename. |
| First entry beyond result cap | Limited with `entry_limit`, retaining whole permitted entries; no implicit continuation promise. |
| New file observable after earlier cached tree snapshot | Fresh metadata result reflects it or reports actual collection limitations; no cached complete-negative answer. |
| Directory/marker changes during the interval | Remove affected entries, expose `namespace_changed`, withhold complete even if later samples match. |
| Unsafe/replaced root or authentication denial after a batch | Refused; clear inventory rather than preserving names under untrusted attribution. |
| Disconnect/timeout after a checked batch | Interrupted; permitted earlier entries have `earlier_observation`, incomplete coverage and unavailable final checks. |
| Explicit session replaced | Never substitute the replacement; no inventory attributed to it. |
| Existing open/edit owner holds the slot | Busy refusal, no queued later discovery or mutation. |
| Human source buffer changes while metadata remains valid | Preserve it; discovery does not infer dirty state or turn the result into edit authorization. |

## 6. Composing with existing operations

The caller selects a returned path and explicitly performs a separate observation/open/edit request with the same intended project/session. Each later operation resolves its current target and applies its own unchanged supported profile and safety checks. Opening does not inherit discovery authorization, and editing still needs fresh source/revision/dirty evidence. An ended session or replacement file cannot inherit an old discovery identity.

A discovery entry says nothing about parse validity, source availability, openness or cleanliness. Absence from a limited list says nothing about existence. Known-document observation still works under its existing contract even when a document is absent from this inventory, such as a deleted file with a live unsaved buffer or an ignored known path.

Retries are explicit new requests. The operation does not reconnect/retry automatically, queue busy requests, launch an editor or supply a disk-only fallback. There is no stable paging cursor, persistent inventory ID or idempotency store.

## 7. Compatibility and evidence

This is the first discovery public schema, version 1. Required-field, outcome/precedence, scope-policy or identity changes are compatibility changes and need deliberate versioning/migration; additions must not silently change membership or weaken refusal. The existing three public v1 operations are unchanged by private bridge v4.

The [quickstart](../quickstart.md) owns executable acceptance commands, including real positive discovery and non-interference. JSON examples and pure classification tests cannot establish authenticated acquisition, exact filesystem coverage, visible-buffer preservation or product timing. This document does not claim those gates have run.
