# Private Editor Observation Bridge — Version 1

**Migration:** This document records Feature 001's accepted private v1 boundary.
The current caller/addon implementation uses
[private bridge v2](../../002-edit-open-gdscript/contracts/bridge-protocol.md)
for the coordinated edit integration. Public observation remains schema v1 and
read-only. Update private peers together and restart the addon for a fresh session;
there is no v1 mutation negotiation or compatibility shim.

**Status:** T002 implements source-free bootstrap, authentication, routing, lifecycle and export isolation. T003 implements the Rust executor boundary (§8); T004 installs real observe/recheck collection (§9); T005 extends request-local attribution and invalidation (§10); T006 verifies source-bearing session/loss behavior (§11); T007 adds closed/built-in and partial-surface evidence (§12). This is private local integration, not MCP, a remote API, or a separate safety model. The [caller contract](observation-api.md) and [data model](../data-model.md) own user-visible semantics.

This is the initial version-1 implementation. It has no raw-token hello or legacy authentication path. No arbitrary code, object deserialization, process command, source write, Save, open/select, reload, rescan, or runtime operation is representable on the bridge.

## 1. Bootstrap and session lifetime

The registry is the tool's own owner-private state location, not an arbitrary filesystem-access permission. Only minimal local operational/session metadata needed to discover, identify, and route to the intended editor session is permitted there under Spec FR-016. It contains no project/source content or unrelated filesystem data; the authentication, ownership, confinement, and privacy requirements below remain mandatory.

1. The local caller's `init-registry` operation creates/checks an explicit absolute directory outside the project. On the initial macOS target it must be owned by the effective user, mode 0700, with no symlink/unsafe writable ancestor substitution; unsafe existing paths are refused rather than chmod-repaired. No token/source goes in command arguments or project settings.
2. A deliberately enabled EditorPlugin receives `GODOT_AGENT_KIT_REGISTRY`. Missing configuration leaves observation unavailable with no listener. Bootstrap verifies private-directory access/permissions and never falls back to an unprotected location.
3. Generate independent random session ID (16 bytes) and token (32 bytes) with Godot `Crypto.generate_random_bytes`. Encode lowercase hex. A plugin disable/re-enable or editor restart always gets a new pair, even for the same PID/project.
4. Bind `TCPServer.listen(port, "127.0.0.1")` only. Choose a cryptographically random candidate port in 49152–65535, with at most 16 bind-collision attempts. Failure leaves no advertised endpoint. Do not use the wildcard default, a configurable remote address, or assume port-zero semantics.
5. Publish a temporary descriptor in the private registry, set/check mode 0600, then atomically rename it to `<session_id>.json` after the listener is ready. On failure, stop the listener and remove only owned incomplete metadata. The Rust reader additionally checks ownership, regular-file type, no symlink, permissions, length, and fields before trusting any descriptor.
6. Disable/exit stops polling, closes peers/listener, removes only the matching owned descriptor, releases request-held references, and unregisters export hooks/signals/nodes. A stale descriptor proves neither liveness nor ownership of a reused TCP port; the mutual proof exchange below must reject a replacement listener that lacks the secret. The observer never kills/restarts an editor or deletes another session's descriptor.

Descriptor fields, all required; no extra source-bearing fields:

```json
{
  "v": 1,
  "session_id": "00112233445566778899aabbccddeeff",
  "project_root": "/fixture/project",
  "godot_version": "4.7.2.stable.official.ed1daf0bf",
  "engine_hash": "ed1daf0bf001b61586d9930840f2f1394092c079",
  "host": "127.0.0.1",
  "port": 53124,
  "token": "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"
}
```

Values above are synthetic examples, not usable credentials. `project_root` is the addon's absolute globalized project root. Rust canonicalizes it and the explicitly requested root, compares filesystem identities, and pins that identity for this request. Godot's string path alone is not an OS confinement guarantee. Handshake echoes use the exact advertised string; public results use Rust's verified canonical root.

Security scope: private registry/token authentication and project confinement protect normal local-user boundaries. They do not defend against malicious same-UID software that can read the token or hostile code already executing inside the selected editor. Do not silently extend this to remote trust or an adversarial same-UID sandbox claim.

## 2. Framing, parser rules, and limits

Each frame is a four-byte unsigned **big-endian byte length**, followed by exactly that many UTF-8 JSON bytes. No newline delimiters, compression, Godot Variant objects, eval, HTTP, TLS fallback, or binary Resource serialization.

| Limit | Version 1 value / behavior |
|---|---|
| Descriptor | 4 KiB maximum, regular private file |
| Registry selection | At most 32 descriptors per request; excess/unsafe enumeration returns source-free unsupported/configuration failure, never chooses the first subset |
| Request/control and handshake response frame | 4 KiB maximum; reject before allocating announced length |
| Response/worker result frame | 12 MiB maximum; reject oversized/truncated/malformed frames |
| Each source | 512 KiB UTF-8 maximum; over limit => only that surface unavailable/`too_large`, still applicable; never truncated or a reason by itself for operation-wide refusal |
| Control string sizes | request ID ≤64 ASCII characters, project root ≤1024 UTF-8 bytes, resource path ≤2048 UTF-8 bytes |
| Control nesting | Flat tuple only, exact arity and types; no object/array-valued arguments |
| Response JSON | Typed known shape; ≤32 nesting levels, bounded collections/strings, duplicate required fields rejected by Rust decoding |
| Editor peers | At most 32 peers including incomplete/unauthenticated handshakes; one active source collection at a time; reject excess, never queue beyond deadline |
| Main-thread network work | Partial I/O, at most 64 KiB or 1 ms of network processing per frame, whichever is reached first; source acquisition is a separate bounded getter pass |

Limits are selected engineering bounds, not measured throughput guarantees. A per-source limit changes only that source's availability, not its applicability. Continue with other independently observable facts and retain their already-gathered valid evidence under the existing deadline and safety rules. Never substitute truncated or observed-empty text.

For a known target, an over-limit source yields `limited_observation` when the document is open or its open state is unknown, or `not_open` with explicit D/R limitations when confirmed closed, unless a separate refusal/interruption takes precedence. A per-source limit alone must not produce `unsupported_observation` or another operation-wide refusal.

Operation-wide unsupported/refused outcomes apply only when the request as a whole cannot proceed safely or meaningfully, such as a request-wide capacity bound preventing unambiguous selection or safe request evaluation. Preserve the existing cause-specific access, input/protocol, and interruption outcomes; do not reclassify those failures as surface limitations. Size/capacity reasons remain distinct from absent/empty source or an unavailable editor. The controlled positive fixtures are far below these caps; larger inputs exercise the applicable per-surface or operation-wide limitation path.

Rust emits strict JSON. Private control messages use fixed-position tuples so Godot's object parser cannot silently resolve duplicate security keys. Godot must validate exact arity, scalar types (including integral `v=1`), UTF-8 validity, field bounds, allowed enum, and identity **after parsing**, without coercing numbers/booleans into strings. Godot's JSON parser tolerates some nonstandard syntax; accepting syntactic tolerance does not relax these semantic/authentication checks. Strings never become executable code or ambient paths. Do not claim that `JSON.parse()` alone supplies strict RFC validation.

The addon uses `StreamPeerTCP.poll()`, available-byte checks, `get_partial_data()`, and `put_partial_data()` with incremental framing. No blocking `get_data()`/`put_data()` loop on the editor thread. Rust uses explicit IPv4 loopback `SocketAddr`, `connect_timeout`, read/write remaining-time bounds, and bounded incremental frame decoding. EOF/reset is disconnection; an incomplete silent peer at deadline is timeout. Late/extra/unexpected frames cannot update a returned result.

## 3. Source-free selection and authentication

For each potentially matching descriptor, mutually authenticate one source-free connection. The descriptor's 32-byte `token` is a pre-shared secret, **never a wire field**. A TCP connection, echoed identities, or private descriptor alone cannot authenticate its current listener.

1. The caller generates a fresh 32-byte cryptographic `client_nonce` for this connection and sends the flat tuple `[version, "hello", request_id, session_id, advertised_project_root, client_nonce]`. Nonces and proofs use exactly 64 lowercase hex characters on the wire; session IDs remain 32 lowercase hex characters. Synthetic example:

```json
[1,"hello","example-1","00112233445566778899aabbccddeeff","/fixture/project","202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f"]
```

2. The addon checks exact current session/project and field bounds, generates a fresh independent 32-byte `server_nonce`, and retains one pending transcript on this connection. It sends an object containing exactly `v`, `kind:"challenge"`, `request_id`, `session_id`, `project_root`, `godot_version`, `engine_hash`, `capabilities`, `client_nonce`, `server_nonce`, and `server_proof`. `capabilities` is an object with exactly five boolean fields: `observe_gdscript`, `open_enumeration`, `buffer_attribution`, `unsaved_paths`, and `cached_resource_lookup`. Advertise only installed operations and available APIs; no mutation/MCP/runtime capabilities.
3. The caller checks types, bounds, its own nonce/request identity, and descriptor session/project/version/hash, then verifies `server_proof` against the transcript below. Until that succeeds, no returned identity, capability, or liveness claim is trusted. Only then send `[version, "authenticate", request_id, session_id, advertised_project_root, client_nonce, server_nonce, client_proof]`:

```json
[1,"authenticate","example-1","00112233445566778899aabbccddeeff","/fixture/project","202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f","404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f","8516bc474f397be9deae2a70c4ae5577041eb6753cb1dde328278978c2f8b5e8"]
```

4. The addon requires that exact pending connection/transcript and verifies `client_proof`. Only on success does it mark the peer authenticated and send `kind:"hello"` with the same identity, capability, and nonce fields as the challenge, replacing `server_proof` with `finish_proof`. The caller requires unchanged fields and a valid finish proof before treating the session as authenticated. A hello acknowledgment alone is not authentication.

For byte-exact interoperability, define `F(x) = u32_be(byte_length(x)) || x`. Let transcript `T` be the concatenation of `F` applied to these fields in this exact order:

- UTF-8 strings `godot-agent-kit/observation-bridge/v1`, `request_id`;
- the decoded 16-byte `session_id`;
- UTF-8 strings `advertised_project_root`, `godot_version`, `engine_hash`, exactly as advertised and checked against the descriptor;
- five one-byte booleans (`00` false, `01` true), in capability order `observe_gdscript`, `open_enumeration`, `buffer_attribution`, `unsaved_paths`, `cached_resource_lookup`;
- decoded 32-byte `client_nonce`, then decoded 32-byte `server_nonce`.

Do not HMAC JSON serialization, normalize strings, or concatenate unframed variable-width fields. With decoded descriptor secret `K`, compute full 32-byte proofs as `HMAC-SHA256(K, F(role) || T)`, with ASCII role `server`, `client`, or `finish` respectively. Separate roles prevent reflection; both nonces and connection-local pending state prevent replay. Rust uses the planned `ring::hmac` sign/verify and `ring::rand::SystemRandom`; Godot uses `Crypto.hmac_digest(HashingContext.HASH_SHA256, ...)`, `Crypto.constant_time_compare` on decoded fixed-length proofs, and `Crypto.generate_random_bytes`. Randomness/crypto failure refuses authentication; never substitute deterministic entropy or a plain hash.

The examples use the synthetic descriptor above, all five capabilities true, and the displayed nonces. Their transcript is 270 bytes; expected proofs are:

| Role | HMAC-SHA256 (lowercase hex) |
|---|---|
| `server` | `31cd95cdc00afe815371791207ddedda98cb79e1d6bf181bd4fb8ac3cc8d2ed1` |
| `client` | `8516bc474f397be9deae2a70c4ae5577041eb6753cb1dde328278978c2f8b5e8` |
| `finish` | `ff0f3f3507b79628be82e0cd67539765a174d9d0323aff470d62bbecbf12dd32` |

Allow only one challenge/authenticate exchange per connection, within the caller's existing deadline; the addon also expires unauthenticated peers within 4.5 seconds of accept. Reject wrong order, replayed/cross-connection nonces, reflected roles, changed identity/capabilities, and malformed or incorrect proofs. A raw-token legacy hello can never authenticate a peer without the required mutual proof exchange. A failed proof yields source-free `denied_access`/`authentication_failed` at `authenticate` and connection close; malformed frames remain `protocol_error`, silence remains bounded timeout. No failure may be discarded to select another candidate. A reused port cannot establish liveness merely by echoing hello fields. No script source is read or sent during any handshake step, and no nonce/proof becomes a public result or diagnostic.

A descriptor plus successful hello does not yet authorize a source read: the **caller** first proves exactly one intended session. With two authenticated matching sessions it returns `ambiguous_target` and closes both without an observe frame. An exact requested ended session never falls back to a replacement. A live candidate plus unresolved second candidate is not unambiguous; deadline expiry yields `timeout` at `resolve_target`, without source. Zero eligible available sessions yields `editor_unavailable`. Keep any source-free candidate diagnostics distinct from document/surface limitations.

## 4. Observation exchange

After selection, retain the selected authenticated connection (or reauthenticate that exact same session if the connection must be recreated before collection; do not retry an issued observation).

### `observe`

```json
[1,"observe","example-1","00112233445566778899aabbccddeeff","/fixture/project","res://scripts/subject.gd"]
```

Positions are `[version, operation, request_id, session_id, advertised_project_root, script_path]`. All identity fields must agree with hello. The addon independently validates the project-relative locator and document attribution. No other operation string is accepted. The reader must not force-load, open, select, or repair the target.

The addon takes a bounded getter-only start sample:

- Enumerate open scripts/editors, uniquely identify the requested Resource, and verify the pinned-version association checks described in [research.md](../research.md).
- Capture open state, object identities, R.source_code, B.text, independently attributable dirty state, relevant CodeEdit versions, and collection stamps.
- If closed, B/dirty are not applicable; R may be read only through an already-cached GDScript reference. If identity/open state cannot be established, preserve other available facts and explicit reasons.
- For mixed text tabs or unsupported controls, return unavailable attribution, not a wrong document. Dirty state from an unattributed unsaved indication is unknown, not an ambiguous-target or disconnected-editor outcome.

Return a typed `sample` object with `v`, `kind:"sample"`, request/session/project/resource identity, collection stamp, `document`, `R`, `B`, `dirty`, and diagnostics. Each fact follows the data model. The addon does not supply D, calculate final three-way comparisons, or decide final core completeness. It retains request-local original witnesses/text only until recheck/close; no cross-request snapshot cache.

Rust validates this sample before retaining evidence. It reads D independently through the pinned project's directory capability, using bounded strict UTF-8 and regular-file checks. No disk read precedes authenticated unambiguous target selection. A missing D must not discard an already-established open R/B; missing target without an open document is classified separately.

### `recheck`

```json
[1,"recheck","example-1","00112233445566778899aabbccddeeff","/fixture/project","res://scripts/subject.gd"]
```

The addon must still hold the same request-local sample on this connection. Re-enumerate/re-read only necessary identity, open/source/dirty/version witnesses without mutating editor state. Compare against the original sample; a changed object/path/order invalidates affected attribution, a changed source/version invalidates affected source evidence, and a changed dirty indication invalidates dirty evidence. Disagreement between D/R/B is not an invalidation: only detected changes **during collection** are.

Return `recheck` with echoed identities, collection stamp, `checks`, and structured `detected_changes` (surface, code, evidence without source snippets). No atomicity or future-stability flag may be true. The Rust side also performs bounded D content/identity recheck. A known same-document limitation preserves unrelated facts; a changed document invalidates the original identity-dependent sample rather than replacing it with new-document values.

Only the common core assembles public comparisons, overall agreement, precedence, and terminal outcome. The addon acknowledgment is evidence of no more than the fields actually read. Close the connection after finalization and release temporary samples.

## 5. Deadline, cancellation, and partial evidence

The caller starts a monotonic deadline before registry/path operations. It supervises one owned, read-only internal worker process; the worker performs potentially blocking filesystem/network acquisition and streams individually validated evidence events. The supervisor's result assembly and timeout path do not depend on Godot's main thread or on joining blocked filesystem I/O.

At 4.5 seconds, stop collection and finalize the strongest known terminal result, reserving up to 0.5 seconds for bounded serialization/delivery in the controlled fixture. Send cancellation/close sockets as possible, terminate only the owned worker, and reap without delaying the required result for an indefinitely blocked operation. Never kill or save the user's editor. No arbitrary executable path/arguments are accepted as a public operation; worker mode runs the same trusted binary with private IPC.

- Known EOF/reset after authentication => `disconnected_editor`; no reply by deadline without known loss => `timeout`.
- Retain previously validated D and other evidence only with original attribution/time and explicit interruption/invalidation limits. No complete current-live claim survives a failed final recheck.
- Malformed/mismatched response => `protocol_error`; never copy values from the wrong session or promote unvalidated payload bytes to evidence.
- Cancellation is terminal. The addon checks peer/deadline state before new work and drops unsent/late responses. It has no mutations to roll back.
- Requests are not transparently retried or resumed. A new caller attempt gets new timestamps, identity resolution, and evidence.

## 6. Confinement, diagnostics, and development boundary

The Rust filesystem boundary owns path validation/capability-based D reads. Addon Resource paths and B/R identities must correspond to that selected project. Reject symlink/parent escapes and unsafe attribution before disclosure; recheck confinement. A `res://` prefix alone is insufficient. Missing disk for a known open document is a surface limitation, not automatic scope denial.

Logs may include request ID, selected-session ID, activity, surface, reason code, timing, and bounded safe project-relative identifiers. Never log tokens, authentication nonces/proofs, source text, entire response/request payloads, raw parser failures containing payloads, unrelated unsaved paths, or candidate scripts. The raw secret remains only in the private descriptor and trusted process memory, never on the wire. Authentication/permission failures and capability absence are distinct structured reasons. There is no telemetry or outbound network activity beyond the selected loopback connection.

Neither `@tool` nor the addon directory proves export exclusion. The planned export hook skips all `res://addons/godot_agent_kit/` files; each production preset must independently exclude that tree. Test enabled and disabled addon exports and actual exported execution. Observation fixture drivers are outside the installed addon tree and excluded from production presets too.

Version changes to framing, identity/authentication interpretation, or required fields require explicit negotiation/version increment; unknown major versions fail closed. No alternate legacy/unsafe bridge path is retained.

## 7. T002 implementation and verified limits

At T002 delivery, the addon installed only the source-free authentication exchange
and advertised every collector capability as false. T004 adds those operations.
Rust still retains the connection only after canonical, unambiguous selection and
requires the finish proof, not an echoed hello.

The exact engine gate validates `major`, `minor`, `patch`, `status`, `build`, and
the full hash from `Engine.get_version_info()`. Its display `string` is
`4.7.2-stable (official)`, not the CLI version identifier above. Godot's JSON
parser represents `1` as a float; the tuple gate therefore requires a numeric
value exactly equal to 1, rejecting booleans, nonintegral numbers, wrong arity,
nested control values, and mismatched identities.

On macOS, owner/mode/type/identity/ACL checks use fixed native metadata operations
because public Godot file APIs do not expose the complete information. There is
no shell, remote target, configurable executable or execution frame. Empty and
deny-only ACLs are accepted; access-grant or unobservable ACLs are refused.
The addon compares directory device/inode ancestry to reject an in-project
registry before creating credentials/listening. Rust separately pins canonical
project and registry identities. Same-UID malicious code remains outside the
stated threat boundary; unsafe detectable metadata still fails closed.

Godot editor safe-save can defer installation of the named temporary file until
close. Descriptor bytes stay within the already-verified 0700 directory; the
closed new file must pass owner/regular-file/0600/ACL checks before atomic rename.
Disable/exit checks owned file identity before removal and disconnects all peers.

The export guard is named to run **before** the built-in GDScript compiler/remapper;
production presets independently exclude product and fixture-driver trees. The
acceptance driver tests enabled, disabled, and hook-only variants, inspects actual
ZIP/PCK entries including compiled/remapped resources, and launches each real app.
Godot may retain inert editor-plugin path settings in `project.binary`; no addon
code, runtime autoload/dependency, tooling node, listener or descriptor survives.

See [T002 evidence](../quickstart.md#22-t002-source-free-boundary-evidence-2026-09-26)
for real-editor routing, all three synthetic proof vectors, independent live
proof checks, refusal/privacy/peer limits and export results. The source-free
`session-boundary` group does not replace T006's later source-bearing `routing`,
`session-loss`, `deadline` or `confinement` acceptance.

## 8. T003 Rust executor boundary

`mcp-server/src/bridge/wire.rs` implements §4's control tuples and typed sample/
recheck responses on T002's retained authenticated connection. The channel is
bound to the original request ID, requested project selector, canonical project
identity, session and locator; another request cannot reuse it to acquire source.
The five advertised capabilities are not invented by the caller. T002's absent
observer is a structured unsupported result, without issuing an observe command.

Sample payloads contain `document`, `R`, `B`, `dirty`, and safe diagnostics, plus
the version/kind, echoed identities and editor collection stamp. Source records
carry authority/availability, exact observed text, stamp, witness, staleness,
reason, and optional invalidated evidence. Document validity/open facts use
`value`, `collection`, `reason`, and optional invalidated evidence. Reasons are
`{code, action}` with fixed safe actions; diagnostics additionally have fixed
message/stage/surface fields. Instance IDs and tick counters are decimal strings.
An editor-supplied disk identity or D witness is rejected: Rust supplies D.

Recheck payloads use `checks`, `detected_changes: [{surface, code}]`, and a nullable
reason. An unavailable recheck may retain changes from checks that did finish;
it does not claim every required check ran. D changes belong only to the separate
Rust disk check. Source limits remain independent and never convert dirty or
divergent evidence into a refusal.

Private worker IPC uses the same four-byte framing bound and versioned correlated
events: `selected`, `sample`, `disk`, `rechecked`, `disk_checked`, then `done`.
`failed` terminates any stage. The supervisor validates each event and its order,
rejects alleged final success supplied by a worker, stamps receipt time, and runs
the common reducer itself. It retains earlier valid partial evidence on a later
malformed frame or interruption; denial still suppresses every source.
`rechecked` is emitted before independently checking D, so a blocked D recheck
cannot conceal earlier editor changes. Late frames cannot upgrade a terminal result.

The addon code and export boundary are unchanged by T003. Native controlled-peer
tests prove codec and precedence behavior, not live R/B. Actual T002-editor caller
and confined-D consumer evidence is recorded in
[quickstart §2.3](../quickstart.md#23-t003-executor-boundary-evidence-2026-09-26).

## 9. T004 live collector boundary

`observation.gd` holds one request-local original sample. The exact-version bridge
advertises installed public editor APIs, binds observe/recheck to the authenticated
request/session/project/locator, and admits one active collection. Getter passes
are separate from the existing shared 64-KiB/1-ms network budget. Controls remain
4 KiB; source-bearing responses are bounded at 12 MiB. Recheck, disconnect, expiry
and plugin disable release the original references; late replies are dropped.

Before and after collection, the addon checks the locator and filesystem-component
identities without reading disk source. Symlink/parent escapes and unsafe scope
cannot disclose collected R/B. This complements, not replaces, Rust's
capability-rooted D boundary and its permission/ACL checks. The same-UID/project
code threat limit remains unchanged.

The sample's `collection` contains every editor fact's collection stamp, and the
recheck interval follows it in the same editor clock. Instance IDs use
`String.num_uint64`, not signed or floating-point formatting. B/dirty association
requires stable complete arrays, unique paths and actual ScriptEditorBase/CodeEdit
objects. R and B are separate getter reads even for equal or empty text.

The additive source-free failure envelope is:

```json
{"v":1,"kind":"failure","request_id":"example-1","session_id":"00112233445566778899aabbccddeeff","project_root":"/fixture/project","script_path":"res://scripts/subject.gd","code":"out_of_project","stage":"read_editor"}
```

`stage` must match the pending operation (`read_editor` or `recheck`); every
identity must match the selected request. The Rust decoder accepts this specific
shape with no source or extra payload fields. `out_of_project` maps to
`denied_access` and suppresses all prior source. `unsupported_observation` is
accepted only at `read_editor` for the one-active-collection capacity refusal
or an unresolved built-in identity; it does not misreport the live editor as
disconnected. Malformed or
mismatched refusals remain protocol errors. Framing, authentication and public
outcomes retain version-1 semantics; deploy this collector with its updated Rust
decoder rather than relying on an older decoder's generic protocol refusal.

Initial real-editor acceptance covers clean-open, repeated and empty observations,
collector rechecks and independent R/B caps. The wider dirty/routing/closed matrix
and full support claim remain T005–T008 work.

## 10. T005 request-local attribution and invalidation

The operations, fields, bounds and authentication exchange remain version 1.
`observation.gd` retains its original enumeration and any changes already
detected during collection until recheck. It never substitutes a replacement
document's source or automatically rereads the request to hide instability.

Before/after arrays must independently justify the target's unique
script/editor association. Unrelated tab movement alone need not invalidate an
unchanged target association; incomplete, nonunique or unsupported arrays cannot
authorize a buffer. Unsaved paths are attributable only through the actual
document map. If that attribution becomes unavailable, `checks: unavailable`
retains any separately detected changes instead of claiming a complete check.

Recheck distinguishes an original Resource whose path changed from a closed
document, detects replacement objects, and validates editor/buffer references
before dereferencing them. Source/version/dirty changes remain surface-specific.
The existing Rust decoder, worker events and reducer propagate this evidence;
no new Rust production API or outcome is introduced. The common core alone
invalidates current facts and recomputes comparisons.

The fixture-only bridge's transition barrier runs after the real caller receives
the sample and reads D, immediately before the real editor recheck. Independent
native witnesses bracket the preparation and subsequent read-only work. Negative
collector restrictions only remove observability; they never supply fabricated
positive source/dirty evidence and are not installed as a production capability.

## 11. T006 source-bearing lifetime and interruption evidence

No production operation, capability, framing, authentication or schema change
is introduced. The existing retained authenticated channel carries real native
R/B/dirty evidence only after unique selection. Same-project sessions with
different actual Resource and CodeEdit contents remain distinct even when
their project name and resource path match.

The disposable fixture bridge can hold a genuine pending `observe` or `recheck`
and publish a source-free stage event. The external driver then disables,
terminates or suspends only its owned editor, or stops only the caller's owned
worker. A pending recheck proves the real sample and independent D read already
occurred; the supervisor still validates and stamps each event separately.
Negative-only restrictions corrupt a genuine response's identity or replace
it with malformed/oversized framing. They never supply accepted synthetic
positive source/dirty evidence and are absent from the production addon.

These cases verify EOF/loss versus silence, original partial attribution,
unavailable rechecks, source suppression after scope denial, and rejection of
late/wrong-session evidence. A stale advertisement cannot authenticate an
impostor reusing an ended editor's port. Replay/reflection, changed transcript,
capacity/expiry, unsafe metadata and secret-free traffic checks are reused
with the source-bearing caller. Fixture material remains under the excluded
driver tree; enabled/disabled/hook-only exports and actual app launches
continue to exclude it. See [T006 evidence](../quickstart.md#26-t006-routing-and-interruption-evidence-2026-09-27).

## 12. T007 closed, built-in and filesystem evidence

The addon validates external and built-in locators independently of the Rust
adapter. A built-in requires a `GDScript_` ASCII alphanumeric/underscore identifier
and a `.tscn`, `.tres`, `.scn` or `.res` container. Scope witnesses apply to the
container's filesystem components before/after getter passes, retaining all
symlink/escape and replacement refusals. No container text is read as script D.

The collector accepts only an existing enumerated or cached GDScript with the
exact requested resource path. Unknown tab association does not discard
independently attributable cached R. Built-in identities use `builtin_gdscript`;
unresolved identity uses the existing source-free `unsupported_observation`
failure envelope at `read_editor`. Failure cleanup releases request references.

Private worker `disk` events now carry nullable `file_identity` and
`file_collection` in addition to the D source record. A caller-clock collection
with an identity is permitted only for external D unavailable because of size,
encoding or read limitations after safe metadata acquisition. A collection
without an identity proves absence only with `disk_missing`. Invalid combinations,
wrong clocks and missing required attribution are protocol errors. Observed D
retains its ordinary source witness; no duplicate metadata or extra allocation
is needed for that path.

The supervisor combines those independent filesystem facts with the original
editor open-state evidence. The worker rechecks confinement and identity even
when D text was unavailable. Metadata never fabricates source text, loaded
Resource identity or editor state. These are same-executable private IPC changes,
not a new editor operation or public JSON schema.

Fixture-only surface restrictions withhold R, B, dirty or open-state attribution;
positive evidence remains actual native getters. Held preparation-time CodeEdit
references let the independent fixture oracle check human-buffer preservation
when mixed tabs prevent the product from assigning a buffer. Built-in/syntax
fixtures and all preparation code remain excluded from production exports.
See [US4 evidence](../quickstart.md#27-t007-closed-and-partial-observation-evidence-2026-09-27).
