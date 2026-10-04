# Trusted Execution and Private Compatibility Contract

**Status:** Planning only. [MCP interface](mcp-interface.md) owns public protocol mapping; [closed native contract](closed-edit.md) owns the new effect path. Existing operation-specific safety is reused, not replaced by a generic transaction API.

## 1. Rust responsibilities

| Owner | Required change / unchanged behavior |
| --- | --- |
| `mcp-server/src/bin/godot-agent-kit-mcp.rs` | New executable wiring, explicit registry, current-thread runtime, existing internal worker dispatch, owned process cancellation. No editor launch. |
| `mcp-server/src/mcp/` | Concrete catalog/input/output DTOs, selected-version handler, bounded stdio and per-call supervision/delivery. Separate framing, schema/projection and host ownership where cohesion requires; no one-type-per-file rule. |
| `observation`, `script_discovery`, `script_edit` and current runners | Existing source/state reduction, open expected basis, policy, authentication, confinement, source validation, effect outcomes and disclosure remain authoritative. Existing public signatures/local contracts are not replaced. |
| `script_read` / `runner/read` | Trusted internal capture, eligibility and fixed stateless revision computation; public source/state projection. Reuse ordinary observation acquisition and the original clock, with only the selected private closed supplement/rechecks. |
| `script_closed_edit` / `runner/closed_edit` | New checked closed expected basis/request, finite effect/evidence reduction and bounded supervisor/worker. No open/close composition or external production file writer. |
| `bridge/wire` and addon transport | Coordinated private v6 capture/closed-mutation DTOs, exact semantic validation and authenticated capability negotiation. Raw RPCs remain crate-private. |
| Native editor integration | Existing session owner plus the new closed attempt and close-epoch witness, as specified in closed-edit.md. |

Public request/result constructors and supervised functions are justified by the actual binary/library consumers. Keep parsers, mutation channels, native method selectors, raw captures and state machines private. Existing `ExpectedRevisionBasis` has current edit/close consumers; do not rewrite it into a speculative lifecycle framework.

## 2. Dispatch and clocks

The adapter validates its envelope/selectors, creates a fresh checked domain request ID and passes the original accepted-frame clock into execution. It does not perform its own candidate search, filesystem write, editor eligibility decision or outcome reduction.

- Discover calls the existing `runner::discovery::run` with its checked `DiscoveryRequest`.
- Read uses shared observation acquisition and existing open eligibility; closed eligibility additionally requires native capability, agreeing closed collection and final rechecks. Issue the opaque revision from checked stable facts; return a meaningful source/state projection rather than the private capture.
- Edit validates ordinary selectors, revision syntax and replacement, then execution freshly resolves the authenticated target and reacquires relevant editor/disk/Resource/lifecycle state under the original edit clock. Compare its recomputed revision; stale/changed/unavailable state refuses with an appropriate reason and `fresh_read`, never a new branch or silently refreshed intent.
- Only after equality and current eligibility does execution build the trusted open/closed request. Freeze the exact matching capture. Open execution keeps the existing strict decoder, `ExpectedRevisionBasis::from_observation`, `EditRequest` and `runner::edit::run`; its bounded observation-shaped worker input comes from that fresh internal capture, not a model-supplied read object. The capture has its own observation ID distinct from the edit ID. Do not invoke an external CLI or repeatedly round-trip JSON.
- Closed execution uses its own supervised worker and the same irreversible parent-before-worker authorization pattern. Blocking editor/filesystem/validator work remains in owned bounded workers, not on the protocol executor.
- Revision comparison does not replace any existing prepare, pre-authorization, immediate native or postcondition check. A change after comparison follows existing refusal/effect semantics; no automatic reacquisition into a new expected basis or deadline renewal.

For read/discover retain the 4.5-second work cutoff and 0.5-second consumed-output reserve; for edit retain 9.5/0.5 seconds. The closed path must earn its own timing evidence. Input validation and additional capture do not start a second clock. A native lease never outlives the existing maximum nine-second effect lease or the original request cutoff.

The existing supervisor is bounded synchronous work. Run it off the Tokio event thread with a retained handle; cancellation sets its existing atomic rather than aborting the await. Keep one admitted tool execution per connection and bounded protocol bookkeeping. This host resource limit is not a replacement for the editor's shared admission slot across all callers.

## 3. Read assembly and source disclosure

A read returns the useful Feature 001 source/state semantics even when editing is unsupported. It does not load/open a Script to acquire R, obtain a revision or validate source. A closed observation may be informational while `revision` is null with an explicit eligibility reason. Private observation/expected-state evidence is retained only within the current call and is not echoed through MCP.

The private closed-state supplement is acquired in the same selected session, under the same read interval/deadline. Check the supplement's target, disk witness, R identity/source and absence against ordinary independent observations before claiming a usable basis. If these change, retain truthful observation limitations/invalidation and refuse basis eligibility. Do not join unrelated acquisition intervals and call them an atomic snapshot.

Native/main-thread witnesses and Rust confined D acquisition are independent postcondition sources. No field copied from the request, native prepare receipt or desired text becomes an observed source. Disallowed source, hashes, diagnostics and inventory remain suppressed by existing domain disclosure precedence, including denial after an earlier causal error. SDK/transport logs must not reintroduce suppressed data.

## 4. Private bridge v6 and native family revision 4

This is a coordinated future cutover, not an assertion that v6 exists at this planning head.

- Increment the matched private bridge version from 5 to **6** and native integration revision from 3 to **4**. Preserve durable `editor_integration` artifact names.
- Add one authenticated capability, **`edit_closed_gdscript`**, meaning the complete read-basis, validation, native mutation and independent-verification family is present. Partial installation must not advertise it. Ordinary read/discovery remain available without native mutation support.
- Reuse existing role-separated authentication, session lifetime, registry ownership, framing and exact build/engine checks. Extend the authentication transcript/strict tuple decoders in both languages together. New fields cannot be accepted as old tuples.
- Add private closed inspection/preparation/application/verification/recheck/finish/cancel/expiry operations. Their method/stage is chosen by trusted execution code, never a public MCP argument. Encode exact byte/identity/counter fields with existing conventions and limits.
- Read-only closed inspection includes actual target absence, cache state, edited/profile facts, file revision and close epoch. Mutation messages are request/session/attempt-bound and share the one active editor slot with existing operations.
- Extend same-binary worker dispatch for script-read/closed-edit before protocol startup; worker stdout remains its assigned private socket, never the MCP stdout stream. Preserve null/safe child stderr and existing owned-child cleanup.

Update all current Rust/addon/native/fixture consumers and cross-language vectors together, including matched manifest/build provenance and operator install guidance. No v5 listener, revision-3 closed fallback, compatibility alias or dual implementation remains after cutover. Existing public observation/edit/open/discovery/close v1 contracts and refusal meanings—including closed refusal by the old edit caller—remain unchanged. Install peers together, restart/re-enable for a new authenticated session and discard old bases.

## 5. Effects, cancellation and terminal errors

The supervisor records possible application before authorizing the worker to send the single native effect request. Before that boundary, cancellation can be proven not-applied only if authorization is irreversibly prevented. Afterward, missing acknowledgment is unknown unless stronger evidence proves a terminal refusal or actual effects.

On cancellation, disable or expiry, prevent subsequent native stages and preserve a currently entered synchronous call's owner until it returns. Retain known writes/Resource changes; never overwrite newer human state to restore the old appearance. Worker/channel loss cannot cancel the user's Godot process and cannot turn partial application into rollback.

If the MCP peer cancels delivery or disappears, the bounded core still produces/retains whatever terminal facts it can obtain while owned cleanup completes. No persistent result service is added. Future action requires a fresh read of the original explicit target; an MCP ID is not an idempotency key. A private decoder failure after effects is a host/operation failure with effect knowledge, not an invalid client request.

## 6. Scope of verification and complexity

This design changes adapter framing and new closed semantics. It does not invalidate every old evidence record by file or build identity alone. The private version/native-family/owner cutover does require affected cross-language admission, loading, cancellation, export and representative existing-operation preservation checks, with behavioral equivalence reviewed for reusable native evidence.

New complete MCP A–E and closed positive/race/interruption proof remain mandatory. Ordinary Save/history/close/reopen are independent witnesses, not public tools. Use [the coverage guide](../quickstart.md#acceptance-coverage), the existing VM and TEST_POLICY.md; no additional runner provider or generic workflow is selected.

Concrete additions are the external protocol boundary, a closed-source domain/owner, the selected capture supplement, one stateless revision encoding and bounded transport. Existing mechanisms supply enforcement but not the new public read/edit precondition or closed mutation. The simpler alternatives—CLI shell glue, opening targets, full internal-read echo, unbounded SDK stdio and prose safety instructions—fail current requirements. Existing hashing/acquisition avoids a key/store/cache; no generic transaction/capability/token/description/tracing/retry infrastructure is introduced.
