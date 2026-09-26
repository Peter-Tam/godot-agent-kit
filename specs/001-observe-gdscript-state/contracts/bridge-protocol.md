# Private Editor Observation Bridge — Version 1

**Status:** Phase 1 design. Private local integration, not MCP, a remote API, or a separate safety model. The [caller contract](observation-api.md) and [data model](../data-model.md) own user-visible semantics. No arbitrary code, object deserialization, process command, source write, Save, open/select, reload, rescan, or runtime operation is representable here.

This is the initial, unimplemented version-1 proposal. The planning review replaces its earlier raw-token hello with the mutual proof exchange below; no deployed protocol or legacy compatibility path exists.

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
