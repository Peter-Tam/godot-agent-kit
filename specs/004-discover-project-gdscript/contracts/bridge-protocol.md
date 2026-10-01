# Private Editor Bridge — Version 4 and Discovery Integration

**Status:** T001 implements the coordinated v4 cutover and path-free addon scope exchange (§1–§3). T002 implements the inventory worker/events (§4) and [public discovery v1](discovery-api.md). Task-owned acceptance is recorded in the [quickstart](../quickstart.md); T003's cumulative feature gate remains pending. Public observation/edit/open v1 and native API revision 2 are unchanged. This contract supersedes private [v3 transport](../../003-open-project-gdscript/contracts/bridge-protocol.md), not its historical acceptance record.

## 1. Bootstrap and authenticated capability migration

Reuse the existing owner-private source-free registry, per-editor-lifetime session/256-bit secret, canonical project identity, IPv4 loopback endpoint, role-separated mutual HMAC and bounded candidate selection. No new listener, credential, permission class or session service.

Descriptor/hello/request/response version is `4`; reject other private versions without fallback. The capability record has exactly these eight Boolean keys in transcript order:

1. `observe_gdscript`
2. `open_enumeration`
3. `buffer_attribution`
4. `unsaved_paths`
5. `cached_resource_lookup`
6. `edit_open_gdscript`
7. `open_gdscript`
8. `discover_gdscripts`

`open_enumeration` still means the existing document/buffer enumeration support; it is not script discovery. Advertise discovery true only when the matching addon scope owner and supported public getters/profile are available. It does not require a native mutation family: native revision remains 0 for unavailable or 2 for the matched existing family, with its existing build ID validation. Do not change the native ABI or artifact names to add this read-only capability.

Let `F(x) = u32_be(byte_length(x)) || x`. Preserve `HMAC-SHA256(K, F(role) || T)` and the existing server/client/finish role strings. `T` is the concatenation of `F` applied separately to:

- UTF-8 `godot-agent-kit/editor-bridge/v4`;
- request ID; decoded session ID; advertised project root; Godot version; engine hash;
- each of the eight one-byte 0/1 capability values above;
- four-byte big-endian native revision; UTF-8 native build ID;
- decoded client nonce; decoded server nonce.

Authenticate these typed fields, not JSON serialization. Missing/unknown/duplicate/wrong-type capability fields or changed context cannot authenticate. Nonces/proofs/tokens stay private. Unique authenticated selection precedes any project inventory access, even though the new bridge exchange itself contains no inventory.

## 2. Common framing and admission

Preserve big-endian four-byte length-prefixed UTF-8 JSON, depth 32, exact tuple arity/types, strict response decoding, finite integral numeric fields and checked decimal-string identities/ticks. Reject oversized lengths before allocation. Existing selected source requests/responses retain their 4 MiB/12 MiB ceilings; discovery does not raise them. Discovery editor requests/responses are source-free control messages capped at 4 KiB. Worker inventory progress is a separate private IPC projection with [data-model bounds](../data-model.md#2-fixed-supported-profile).

Each discovery tuple begins `[4, opcode, request_id, session_id, advertised_project_root, ...]`. There is deliberately no script path or directory prefix. All identity fields must match the authenticated channel, immutable selected project and one current attempt. A peer cannot ask the addon to enumerate an arbitrary supplied path or invoke a method by name.

The existing `_active` slot admits at most one observation/open/edit/discovery owner. Discovery uses that slot rather than a parallel scheduler. Overlap returns terminal `busy`; do not queue or retry it. Internal rechecks for the admitted owner are allowed. Existing entered-native-owner retention rules remain unchanged; discovery has no entered native stage.

## 3. Discovery exchange

### Begin

Exact tuple:

```json
[4, "discover_begin", "example-discovery-1", "00112233445566778899aabbccddeeff", "/fixture/project", 4000]
```

The trailing integer is remaining budget in milliseconds, 1–4500, bounded by the caller's actual original cutoff. The addon cannot extend that cutoff. The tuple is illustrative, not an executed request.

The addon validates authenticated request identity/capability, claims the existing slot and obtains only:

- `EditorInterface.get_editor_paths().get_project_settings_dir()`; accept the exact demonstrated `res://.godot/editor` or `res://godot/editor`, returning its parent as the effective data exclusion;
- `EditorInterface.get_resource_filesystem().is_scanning()` and `.is_importing()`;
- the owner/session-local `filesystem_changed` counter;
- editor-local start/end ticks and owner expiry.

Do not read source, documents, resource cache, type/class/import metadata or the editor's cached directory tree. Do not call scan/update/reimport, validation helpers, native mutation APIs, Save, focus or startup operations. No generic ProjectSettings snapshot is sent. A public getter failure is unavailable evidence; unexpected context is unsupported, never guessed from the mutable setting or filesystem.

`discover_state` has exactly:

| Field | Value |
|---|---|
| `v`, `kind` | `4`, `discover_state` |
| `request_id`, `session_id`, `project_root` | Same authenticated binding; project is the advertised root |
| `collection` | Existing editor-domain collection stamp; receiver assigns its actual receipt once |
| `status` | `observed`, `unavailable` or `refused` |
| `reason` | Null for fully obtained facts; otherwise a fixed supported context/admission reason |
| `context` | Null unless safely obtained; otherwise the exact object below |
| `expiry_tick_us` | Nullable canonical decimal editor tick, present when this attempt owns the slot |

`context` has exactly `policy`, `project_data_directory`, `filesystem_epoch`, `scanning`, `importing`. Policy is `godot_project_files_v1`; epoch is a nonnegative canonical decimal string. Scan/import Booleans are observed facts, not a verdict that the inventory is complete. Rust decides whether to collect or return limited/unsupported. A busy refusal has no owned expiry/context and cannot release another operation.

The scope owner's closed context/admission reasons are:

| Status | Reason | Context / expiry |
|---|---|---|
| `observed` | null | Exact observed facts; owned expiry |
| `unavailable` | `unavailable_editor_context` | null context; owned expiry |
| `refused` | `unsupported_visibility_policy` | null context; owned expiry |
| `refused` | `unsupported_discovery` | Missing advertised/current scope capability; no context or owned expiry |
| `refused` | `busy` | Another operation owns the slot; no context or owned expiry |

An admitted unavailable/unsupported-context attempt stays owned until finish,
abort, channel loss or expiry. These are private acquisition facts, not public
inventory outcomes. The epoch increments on observed `filesystem_changed`
signals while an attempt is registered and remains local to this owner/session;
it is neither a count of every filesystem write nor proof of an idle current tree.

### Recheck

Exact tuple: `[4, "discover_recheck", request_id, session_id, advertised_project_root]`.

Legal only for the same still-owned unexpired attempt. `discover_rechecked` has the same exact envelope and context fields as `discover_state`, with its own fresh stamp. No expected path list, source, external context blob or caller-supplied verdict is accepted. The core compares fresh scope/lifetime/epoch/scan facts with begin evidence; the addon does not synthesize the public `DiscoveryOutcome`.

This recheck is not a ping acknowledgment alone: it re-observes the effective data path, scan/import state and counter. A changed policy/data directory invalidates the admitted scope. An epoch/scan/import change prevents complete discovery without implying which source/buffer changed. It never forces reconciliation or waits for the editor to become quiet.

### Finish and abort

Exact tuples: `[4, "discover_finish", request_id, session_id, advertised_project_root]` and `[4, "discover_abort", request_id, session_id, advertised_project_root]`.

Responses `discover_finished` / `discover_aborted` carry `v`, `kind`, request/session/project, editor collection stamp and `terminal_discard` Boolean. They release only this attempt's read-only context/slot. An acknowledgment is not inventory verification and cannot restore invalidated facts. Abort is best effort within the original time budget; terminal delivery never waits for it past cutoff.

`terminal_discard` is false for finish and true for abort. Invalid, duplicate,
wrong-owner or expired stage requests close their own peer without a context
reply and cannot release another peer's owner.

Peer closure, expiry, addon disable or owner exit cancels the same read-only attempt and cleans up its signal/registration state. No document/source/history cleanup is performed. Unexpected, repeated, wrong-owner or expired stage tuples cannot create a new owner or revive an old attempt. A late response cannot upgrade a caller result.

## 4. Rust collection and worker events

The worker resolves a project-only selector through the common authenticated algorithm and retains the selected socket and rooted directory. It emits a validated selected-target event, obtains begin context, then invokes the confined metadata walker. The addon never receives the returned inventory and therefore cannot disclose one before unique selection.

The new codec follows the existing owned-socket length-prefixed IPC, same-executable startup and clock conventions. Its strict discriminated events are:

1. `selected`: exact project-only target and authenticated capability facts; legal once after source-free resolution.
2. `scope`: exact validated begin facts and receipt; legal after selected.
3. `entries`: bounded batch ordinal starting at zero, its caller collection stamp, whole checked path entries and bounded visited counters; legal only while enumerating and only under the same admitted scope.
4. `invalidate`: fixed scope/root/namespace reason and safe affected prefix, or global suppression; removes affected previously accepted entries and cannot be reversed by a later batch.
5. `rechecked`: final namespace/access verification facts, coverage/gaps and validated fresh editor-context receipt; legal after enumeration ends.
6. `failed`: fixed typed failure and any already-established safe coverage facts; no raw OS/wire text.
7. `done`: terminal acquisition marker, not a public outcome or proof of completeness.

Every event includes the immutable request binding; field types, allowed states, ordinals, identities, stamps, result/work limits and exact known keys are validated before acceptance. Invalid worker frames cannot add paths, change targets or pass off a copied input as observed evidence. Duplicate entries, repeated/out-of-order batches and post-terminal events are protocol failures. The parent owns terminal reduction and the deadline; the worker cannot supply a final success enum to bypass checks.

These event names are a closed operation-specific codec, not a generic event bus. During implementation, use typed variants and existing framing helpers instead of a second untyped protocol framework. [Data model §5–§6](../data-model.md#5-private-filesystem-evidence-and-coverage-algorithm) owns the evidence required before entries may be emitted and retained after interruption.

Use the existing same-executable Unix socketpair/owned child pattern. No additional shell/process executable is accepted from the caller. At timeout/cancellation, stop/reap the owned worker through existing supervision; losing the selected channel triggers addon expiry/release. Do not kill or restart the developer's editor. A blocked worker cannot hold terminal delivery, and read-only cancellation does not require a mutation-authorization handshake or rollback receipt.

## 5. Coordinated consumer migration

Update the following actual boundaries together when v4 is introduced:

- Rust `bridge.rs`, strict descriptor/handshake and wire parsing/encoding, capability validation/transcript vectors and private worker consumers.
- Addon `bridge.gd` advertisement/handshake/tuple checks, `script_open_transport.gd` and existing edit/open/observation message envelopes, plus plugin wiring for the discovery owner.
- Existing Rust bridge/confinement/caller fixtures, observation harness HMAC/descriptor/premature-operation checks, fixture-driver cross-language vectors, opening/editing callers and private controls that encode/check version 3.
- Current installation/reproduction notices in the existing feature contracts/quickstarts, native operator guide only where addon session/version instructions change, shared CI's real binary build list and fixture/campaign inputs. Keep archived v2/v3 acceptance historical.

Rebuild the Rust callers/library, install the updated addon and restart owned/developer editors deliberately to obtain fresh v4 descriptors. Native API revision 2/library may be reused if its provenance and unchanged source/ABI family still match; this feature does not require a native rebuild merely for a transport version. Stale v3 descriptors/peers are unsupported, not a reason to select another session silently. Prior session-bound edit bases cannot transfer to a new session.

No deprecated aliases, dual transcript domains or compatibility listener remain. Existing source observations, edit persistence/validation, opening semantics and public schema versions do not change.

## 6. Required boundary and privacy evidence

Cross-language fixtures must demonstrate valid v4 authentication and rejection of v3, missing/extra/non-Boolean discovery capability, changed capability/native metadata, wrong roles, replay/reflection and bad identity. A capability false response must not make discovery available or suppress an otherwise valid existing operation.

Exercise path-free tuple arity, malformed/oversized/unknown fields, private IPC batch bounds and state order, no inventory before unique selection, stale/ended sessions, blocked/replaced editor, late responses, same-slot busy interactions with observation/open/edit and cleanup after worker loss/expiry/disable. Establish supported context through the actual public getter, including visible data directory and unsaved setting changes; no echo-only context test.

Privacy checks use synthetic selected/other-project path and source sentinels. Inventory is allowed only in the explicitly authorized discovery result and its bounded worker evidence, never logs, registry or another candidate's output. Exports must exclude the new owner and fixtures in enabled, disabled and hook-only modes and run without tooling dependency.

This shared authenticated cutover can invalidate the complete existing observation/edit/open suites; those full suites plus complete discovery acceptance are required on the final implementation head. No test here authorizes source execution, new CI topology or native ABI changes.
