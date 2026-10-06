# Exact Intent and Trusted Execution Contract

**Status:** Phase 1 design. This changes intent derivation and public interpretation, not native mutation authority. [Research](../research.md) records source evidence and alternatives; the [data model](../data-model.md) defines values and bounds.

## Responsibility and visibility

- `mcp/handler.rs` decodes the single Schema 2 shape into checked exact intent. It neither reads source to construct a replacement nor writes a file.
- `script_edit/request.rs` owns the bounded literal pair and transformation, beside existing `ReplacementSource`; it needs no filesystem, Godot, MCP, async or clock dependency. New exact types/helpers are crate-private with source-redacting Debug.
- `script_read.rs` retains the existing revision-based whole-source constructor for actual independent local consumers. Add a crate-private exact constructor and private complete/exact enum. No public source-mode field, unused generic transformation API or speculative variant is added.
- `runner/read.rs` owns one acquisition/revision gate, fixed lifecycle dispatch, original clock and interpretation of a retained derivation error. Reuse or extract its existing concrete dispatch body locally; do not duplicate routing, create another supervisor or reset the clock.
- Existing open/closed workers, validator, addon and native owners continue receiving complete source and their existing frozen expected-state evidence. No exact spans, MCP types or public schema number enter native execution.
- `mcp/output` projects actual core results and concise action guidance, without inventing success, disclosure or effect certainty.

The existing local `edit-gdscript`, full-source fixture entry and native/private payloads remain unchanged. This is not a public MCP compatibility shim: only the private enum's exact branch is reachable from Schema 2 decoding.

## Request to complete source

1. Validate bounded JSON and selectors/types/revision/fragment representation without content-bearing diagnostics. Empty old is allowed through structural decoding.
2. Acquire the explicit target through the existing authenticated, confined, supervised route under the original edit clock. Use the existing open observation or closed supplement; missing evidence cannot select an empty/absent branch.
3. Preserve capture/eligibility failure, cancellation and deadline precedence. Compare the supplied opaque revision with that capture's recomputed revision. Mismatch refuses before matching even if old text is absent, ambiguous, still unique or both source and old text remain empty.
4. Freeze that capture. Obtain complete source only from the checked applicable authority set: agreeing D/R/B for open, D plus coherent cached R or positively absent R for closed. Use the same capture for expected basis and source; no presentation fallback from an unavailable authority and no separate read-to-repair.
5. Resolve the literal intent with the [transformation rules](../data-model.md#transformation-rules). Hold either a complete `ReplacementSource` or a typed source-free derivation failure. Check cancellation/deadline around bounded matching/construction; no step receives a new budget.
6. For a complete replacement, dispatch the established full-source path once with its original basis. All original/intended-source support, validation, context, Save-preservation, human-state and immediate effect guards remain required. Later changed state must refuse or report actual effects, not refresh the basis or switch lifecycle.
7. For a derivation failure, follow the no-effect path below before publishing a match diagnosis.

Matching is at most two standard-library substring searches; the second begins at the next UTF-8 character boundary after the first start. Use checked result-size arithmetic before one exact-capacity construction. Do not allocate match vectors or copy current source merely to search. Complete/empty replacement may transfer owned new text; existing downstream owned-source requirements still apply.

## Refusal precedence

A read revision establishes current observed eligibility, not every later mutation preflight fact. Existing open preparation additionally inspects actual saved version, Resource-edited state, effective context and Save behavior. These checks cannot be bypassed by returning `no_match` from the initial read alone.

For `NoMatch`, `AmbiguousMatch`, `EmptyOldString` or derived `InvalidSource`:

1. Retain the typed error privately. Feed the **exact original frozen current source** to the existing unchanged full-source path, with the same original expected basis, clock and cancellation owner.
2. This path must use its established no-effect admission, source/context validation and independent unchanged verification. It must not call a changed-source authorizer, setter, writer, saved-state finalizer or history mutation. It cannot Save, reload, force-load, open/close or repair eligibility.
3. If it refuses, times out, is cancelled, loses observation/disclosure or reports any effect uncertainty, return that actual outcome. Do not replace its safety reason, evidence, action or disclosure decision with the pending match error.
4. Only `verified_unchanged` with proven zero effects may become the pending `refused`/`not_applied` matching result. Preserve actual mode, interval, permitted revision/evidence, validation, lifecycle and nonparticipating history. Set the matching reason/stage and the relevant fresh-read action/summary. Never publish the intermediate unchanged success to the caller.
5. Cancellation/deadline before terminal publication and sticky disclosure still take precedence. Any unexpected effect is retained as non-success with actual certainty, never relabeled as a no-effect match failure.

This is existing unchanged execution used to establish applicable safety, not an attempted corrective edit or a retry. It runs only for rejected derivations; successful exact intent does not incur an additional no-op transaction. Native effect-specific admission continues to guard changed source. Do not clone effect or Save policy into the matcher or add another private preflight protocol merely to report a diagnosis.

Core terminal reduction uses checked discriminants/effect evidence, never a success message, nonempty payload, `isError` flag or model assertion. Matching errors are local to this orchestration boundary; no native/global wire reason-enum extension is needed. Result serialization must preserve existing open/closed compatibility and source-free failure behavior.

## Lifecycle and postconditions

**Open:** Preserve the same admitted document and native history. Changed success requires independently observed complete intended D/R/B and clean/saved state; unchanged success requires equivalent complete observation without any source/history effect. Native Undo/Redo and pre-existing history remain real editor operations.

**Closed with cached R:** Preserve actual document absence throughout; use the existing native explicit source setter only on the retained clean matching Script, then existing confined persistence and independent D/R verification. Loaded class/runtime hot reload is not implied.

**Closed with absent R:** Preserve positive Resource absence and document absence; verify intended D independently. Do not force-load a Resource or create a B/history to prove success. Unknown absence refuses. Later ordinary opening challenges durability only after closed success was independently established.

**All profiles:** Existing original/intended validation, guard rechecks, exact namespace/file identity, partial-effect reduction, Save/reparse/rescan/runtime durability and disclosure remain authoritative. Empty scripts do not bypass any gate. A missing file is not an empty script; no create/rename/delete authority exists.

## Timing, interruption and compatibility

Complete edit handling—including matching, a rejected-intent unchanged assessment where needed, validators, verification and consumed response delivery—retains the supported ten-second bound and 9.5-second work cutoff. Read/discover retain five seconds and 4.5-second work cutoffs. No extra cancellation state machine, background queue, compensation or replay store is added.

Preserve known/partial/unknown effects after entry. Lost/cancelled output is not proof that the native operation was cancelled or rolled back. An automatic client retransmission cannot become a fresh intentional request or bypass revision/native guards. Recovery is a fresh read of the original explicit target followed by a separately intended request.

The public schema changes to 2; MCP 2025-11-25, `sr1`, local operation v1, private bridge v6 and native revision 4 do not change. No native rebuild or historical campaign is inherently required by this source-intent refinement; actual implementation changes may invalidate affected evidence under [TEST_POLICY.md](../../../TEST_POLICY.md). New exact-path real-agent A–E and client compatibility remain mandatory, independent of inherited native evidence.

## Focused behavioral obligations

- Deterministic transformation boundaries: overlapping ASCII/multibyte starts, zero/one/many matches, source-start/end, multiline/tabs/Unicode, canonical-equivalent nonmatches, empty/deletion/equal cases, exact cap/overflow and bounded repetitive near-match input.
- Precedence: dirty/equal-text-dirty R/B, divergence/unavailable evidence, stale same-text revisions, target/session/namespace/lifecycle replacement paired with unique, absent, ambiguous and empty old text. No match feedback may defeat disclosure.
- Rejected derivation: the unchanged diagnostic path must preserve source, saved/dirty state, identities, history and lifecycle; late guard/cancel/deadline/validation failure must win over the retained match error.
- Valid derived source: the real changed/unchanged open and both closed routes validate and independently compare the **entire** source. Invalid output and bounds refuse before effects; no fragment-only proof.
- Interruption: new intent/diagnostic state cannot change existing application certainty, owner lifetime or original deadline. Exercise affected cancellation/output-loss points, not every historical campaign by default.
- Compatibility: actual Schema 2 process/catalog/client calls and old/mixed request refusal; independent local whole-source APIs remain callable with unchanged meanings.

These obligations guide later task derivation; they are not tasks, a new approval gate or completed implementation evidence.
