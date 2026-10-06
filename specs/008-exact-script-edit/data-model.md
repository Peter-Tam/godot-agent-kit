# Data Model: Exact Partial Script Editing

**Status:** Phase 1 design; not implemented or accepted. [Specification](spec.md), [research](research.md), [public contract](contracts/mcp-interface.md) and [execution contract](contracts/execution.md) define the selected behavior.

## Ownership and inherited records

MCP owns input/output schemas and transport. The protocol-independent core owns checked exact intent, fresh capture/revision comparison, derivation and terminal interpretation. Existing open/closed workers and Godot integration own complete-source validation, immediate guards, effects and independent verification. A literal match conveys no mutation authority.

Reuse [Feature 007's trusted read, revisions and closed-state model](../007-mcp-script-workflow/data-model.md), existing open expected-state evidence and local operation outcomes. Source/evidence are request-local; no database, match index, revision store, replay token, result cache or journal is added.

## Entities and relationships

| Entity | Fields / relationship | Validation and ownership |
| --- | --- | --- |
| Public exact edit arguments | `project_root`, `script_path`, `revision`, `old_string`, `new_string`; optional `session_id` | One strict MCP object, no mode or legacy field. Decoder checks types, selectors, revision syntax and fragment bounds/representation. |
| Exact replacement intent | Owned bounded `old_string` and `new_string` | Crate-private checked core value; redacted Debug/errors. Empty old is syntactically valid but requires a fresh empty complete source. Fragments are not standalone scripts. |
| Revision-based request | Checked observation selectors/request ID, opaque `ScriptRevision`, private source-intent enum | Existing public whole-source constructor remains for local consumers; crate-private exact constructor serves MCP. Enum has only `Complete(ReplacementSource)` and `Exact(ExactReplacement)` or equivalent names, not public wire modes. |
| Fresh trusted capture | Existing observation plus open basis or closed supplement, revision, interval and permitted source | Authenticated/confined acquisition; unknown, invalidated or dirty state cannot become admissible merely because text matches. One frozen capture supplies source and expected basis. |
| Derived complete source | One owned `ReplacementSource` containing prefix + replacement + suffix, or exactly new text for empty complete source | Existing full-source byte/representation bounds apply; existing workers perform script/context admission and required validation. Outside bytes are unchanged. |
| Derivation failure | Typed `NoMatch`, `AmbiguousMatch`, `EmptyOldString`, `InvalidSource` | Request-local, source-free, withheld until applicable unchanged safety/verification succeeds. It has no candidate/source/offset/count payload. |
| Checked execution result | Existing open/closed result or earlier acquisition refusal | Preserves stage, application, evidence, validation, dirty/synchronization/history/lifecycle and disclosure. Only proven zero-effect unchanged may be reduced to a retained derivation refusal. |
| Public tool envelope | `schema_version: 2`, `operation`, `request_id`, `result`, `error` | Exactly one of result/error non-null. Nested existing local versions keep their own meanings. |

Names above constrain responsibility and visibility, not a new public Rust API. A new declaration becomes public only if a current external consumer actually requires it. Existing complete-source APIs are real independent contracts, not compatibility aliases for MCP.

## Bounds and exactness

| Value | Bound / interpretation |
| --- | --- |
| Project/script/session selectors | Retain 1024-byte project root, 2048-byte resource locator and optional 32-lowercase-hex session ID rules. |
| Revision | Retain opaque `sr1:` plus 64 lowercase hex. No reissue or renumbering merely because the public schema changes. |
| Each decoded fragment | At most 524288 UTF-8 bytes, LF representation; reject NUL, CR and U+FEFF without normalization. Empty strings are values, not missing data. |
| Original and complete intended source | Retain 524288-byte independent-authority/result limits and existing source/context admission. Bounds are bytes, not JSON Schema character counts. |
| Public framing/output | Retain 16 MiB complete input frame/tool object, 64 MiB complete response, depth 64, bounded IDs and strict duplicate/unknown handling. Existing inner worker bounds remain separate. |
| Work and delivery | Read/discover 5 seconds, edit 10 seconds; existing 4.5/9.5-second work cutoffs. Matching, refusal safety checks, validation, serialization and consumed delivery use the original clock. |
| Retained state | Only current strings, one capture, one derived source or pending error and existing attempt state; at most two substring searches, no match collection. |

The two fragment caps do not expand the complete-source cap. A pair of individually valid fragments may produce an over-bound source and must refuse before effects. Admissible empty source means a complete, observed, safe existing script; whitespace, missing files, unavailable source, empty B with nonempty D/R or incomplete acquisition do not qualify.

## Transformation rules

For nonempty old text, find the first literal UTF-8 substring occurrence. No first hit means `NoMatch`. A search beginning at the next character boundary after that start must find no second hit; otherwise `AmbiguousMatch`. This counts overlapping starts, including multibyte text, without searching inside a UTF-8 code point. Match location is internal only.

A unique occurrence defines `prefix + new_string + suffix`. Check result length using checked arithmetic before allocating; construct once and do not search newly inserted text. Matching the entire original source remains ordinary exact replacement. Empty new text removes the unique span, including the entire source if the empty result is admissible. Adjacent insertion is expressed by retaining a nonempty anchor in new text.

Empty old text has exactly one interpretation: if complete current source is empty, intended source is exactly new text; otherwise `EmptyOldString`. No file creation, zero-length position matching, implicit prepend/append or lifecycle change follows.

Equal old/new text still requires unique matching or the complete-empty-source condition. It then uses existing fully validated/independently verified unchanged execution. Finding new text without old text is not success. No trim, case folding, newline conversion, final newline insertion, Unicode normalization, reindentation, fuzzy/regex/AST matching or occurrence selector exists.

## State transitions

1. **Decoded:** strict checked request, original clock/cancellation owner; malformed input ends with a source-free input refusal before dispatch.
2. **Freshly acquired:** authenticate/select/confine the explicit target and acquire current state. Missing safety evidence, dirty/divergent state, cancellation/deadline or revision mismatch ends through existing refusal semantics.
3. **Intent resolved:** borrow complete admitted capture source and either derive one full replacement or retain a typed derivation failure. No mutation has been authorized and no match diagnosis is yet published.
4. **Existing execution:** a valid derivation uses its intended source. A failed derivation uses the exact frozen current source only for the existing unchanged safety/validation/verification path. Both retain the same capture-derived expected basis and original budget; neither refreshes the basis or switches lifecycle.
5. **Terminal:** propagate every safety/validation/disclosure/effect failure. A normal valid intent retains the existing verified-changed/unchanged result. A pending derivation failure may replace **only** independently verified unchanged with zero effects, producing a refusal and retaining actual evidence/lifecycle/history.

The unchanged diagnostic path exists because saved-version/Resource-edited/context checks occur after basic read eligibility in the existing editor preparation. It must not Save, repair, invoke setters/writers/history or authorize changed-source entry. An unexpected effect/uncertain result is not reduced to matching refusal. Native effect-only checks still govern every changed edit. [Execution precedence](contracts/execution.md#refusal-precedence) defines this boundary in detail.

## Outcome and disclosure invariants

- The [public reason table](contracts/mcp-interface.md#matching-refusals) defines new identifiers and actionable guidance. Existing invalid-script, safety, timeout, disconnect and effect-sensitive distinctions remain intact.
- A match refusal has `outcome: refused`, `application: not_applied` and no source/history/lifecycle effects; its successful no-effect assessment is not published as an edit success.
- Open history is not participated for a diagnostic check; closed B/history are not applicable only where absence is positively established. Unknown final state is never preserved-lifecycle proof.
- After possible effects, only actual known/partial/unknown facts govern. No rollback, replay, automatic retry, queue, compensation or definite non-application is inferred from a timeout or lost response.
- Neither fragment nor derived source appears in Debug, incidental logs, error text or match diagnostics. Permitted read source remains in the existing read result. Existing source-free digests/evidence do not become a new match-feedback channel.
- Denied/unavailable disclosure is sticky through projection and output failure. A retained match error cannot restore a revision, target or other source-derived fact that a later safety result suppresses.
