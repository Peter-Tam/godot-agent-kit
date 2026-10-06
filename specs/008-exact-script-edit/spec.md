# Feature Specification: Exact Partial Editing for Project GDScripts

**Feature Branch**: `spec/008-exact-script-edit`

**Created**: 2026-10-06

**Status**: Draft — clarification complete; planning and implementation have not run.

**Input**: Generated WHAT/WHY description supplied to `/speckit.specify`:

> Let coding agents make a small, exact source change to an existing project GDScript without resending the complete replacement script. Evolve the existing edit_script interaction to accept the current read revision and one literal old_string/new_string pair. Require one unambiguous occurrence in freshly acquired admissible current source; refuse missing, ambiguous or stale intent without mutation. Allow deletion through an empty new_string, but give empty old_string no insertion or creation meaning.
>
> Keep exactly discover_scripts, read_script and edit_script. Preserve the trusted workflow's authenticated routing, confinement, opaque stateless revisions, human-work protection, source validation, independently verified complete-source postconditions, effect-sensitive outcomes, disclosure restrictions and admitted open/closed lifecycle. Open edits retain native history; both supported closed profiles remain closed. Select a deliberate versioned public-contract migration rather than accumulating modes. Compare current agent interfaces for familiar ergonomics, not mutation authority. This is a new feature after completed Feature 007, not a rewrite of its historical whole-source contract or a start on scene authoring, release work, standalone Save/history or implementation planning.

## Clarifications

### Session 2026-10-06

- Q: Should the revised edit tool replace the old whole-source input immediately, or provide a temporary compatibility period for existing callers? → A: Clean Schema 2 cutover: accept only `old_string`/`new_string`, reject `replacement_source`, and require callers to migrate without a temporary compatibility period.
- Q: Should an empty `old_string` always be rejected, even when the script itself is empty? → A: Reject every empty `old_string`; accept that existing empty scripts cannot be populated through this tool.

## User Scenarios & Testing *(mandatory)*

The primary user is a coding agent working for a developer with a supported live
Godot Editor. The existing whole-source workflow is already safe and usable;
this feature changes how the agent expresses an edit, not who may mutate source.
Discovery remains optional when the target is already known. A read returns exact
source, an opaque revision and trusted state; it never grants mutation authority.

**D** means disk source, **R** loaded Script/Resource source and **B** visible editor
buffer. The [completed support profile](../../PHASE_3_EXIT.md#scope-and-support-profile)
includes an eligible clean open document, a closed script with clean coherent
cached R, and a closed script with positively observed absent R. Missing evidence
is not absence. Every story inherits the same safety and disclosure requirements.

### User Story 1 - Make One Exact Change Without Resending the Script (Priority: P1)

As a coding agent, I want to quote the source span I intend to change and its
replacement, so a small correction does not require regenerating unrelated source.

**Why this priority**: Feature 007 requires a complete `replacement_source` even
for a small change. Exact text already returned by read is a familiar, bounded way
to express intent while keeping trusted source acquisition authoritative.

**Independent Test**: Through a real coding-agent client, read and edit each
supported lifecycle profile, then freshly read the result. Independently compare
the complete intended source with every applicable live authority and witness
that no document was opened or closed by the operation.

**Acceptance Scenarios**:

1. **Given** an eligible clean open script and its current read revision, **When** the agent replaces a uniquely occurring source span, **Then** success establishes the complete intended D/R/B, clean/saved state and the same open document. Source outside the selected span remains exactly unchanged.
2. **Given** an eligible closed script with readable clean R agreeing with D, **When** the agent makes an exact replacement using its current read revision, **Then** intended D and R independently agree and the script remains closed throughout. Loaded-class/runtime hot reload is not implied.
3. **Given** an eligible closed script with positively observed absent R, **When** the agent makes an exact replacement, **Then** intended D is independently verified, R remains absent and no document/buffer is created. B and buffer history are explicitly not applicable, not unobserved substitutes for success.
4. **Given** a unique span containing multiple lines, tabs and Unicode text, **When** it is replaced by exact new text, **Then** those characters and the surrounding source are preserved without trimming, reindentation, newline conversion, Unicode normalization or implicit formatting.
5. **Given** a unique nonempty span whose removal leaves an admissible complete script, **When** `new_string` is empty, **Then** that span is deleted and the same full-source validation, coherence and lifecycle guarantees hold. If removal makes the complete source invalid or unsupported, mutation refuses instead.
6. **Given** a unique nonempty anchor at the beginning, middle or end of a script, **When** the replacement retains the anchor and adds adjacent text, **Then** the requested insertion is expressed as ordinary replacement. No empty-search sentinel, line coordinate or insertion mode is needed.
7. **Given** a current admissible target with exactly one occurrence and `old_string == new_string`, **When** the request is evaluated, **Then** a verified unchanged result is possible only after the same current-state and postcondition checks. Source, history and lifecycle do not change; stale, dirty or ambiguous input cannot become a successful no-op.

### User Story 2 - Refuse an Unclear or Outdated Change (Priority: P1)

As a developer, I want exact matching to reject uncertainty rather than guess,
so a plausible-looking match cannot overwrite newer work or the wrong occurrence.

**Why this priority**: A shorter edit request must not become a weaker precondition
or an alternate safety policy.

**Independent Test**: Exercise zero/multiple matches, exact-text boundaries and
freshness/safety failures against independently captured before/after state.
Check machine-readable distinctions and preservation, not only error messages.

**Acceptance Scenarios**:

1. **Given** fresh admissible source and a matching revision, **When** `old_string` has no occurrence, **Then** the operation refuses as no match with no mutation or history/lifecycle change. Presence of `new_string` does not establish that this request already succeeded.
2. **Given** the same preconditions but multiple occurrences, including overlapping occurrences, **When** a replacement is requested, **Then** it refuses as ambiguous without choosing the first or last occurrence. After obtaining current source/revision and supplying a larger unique span, a separate intentional request can succeed.
3. **Given** a search differs only in case, spaces versus tabs, indentation, newline characters, trailing whitespace or canonically equivalent Unicode spelling, **When** no literal occurrence exists, **Then** no match is reported without fuzzy matching, normalization or relocation to similar code. Unsupported source/request text may instead receive its existing input/profile refusal; it is never repaired into a match.
4. **Given** source or another revision-bound fact changed after read while the quoted old text still occurs, **When** the old revision is submitted, **Then** revision/state protection refuses and requires a fresh read. Matching text cannot override same-text version changes, file/Resource/document replacement, session changes or lifecycle changes. This also holds when the quoted text no longer occurs or occurs multiple times.
5. **Given** dirty human B, equal-text dirty state, dirty/stale R, divergent D/R/B, or unavailable required safety observations, **When** a request contains otherwise matching text, **Then** existing protection refuses and preserves human text, dirty state, identities and history. It does not Save, reload, force-load, open, close or repair state to manufacture eligibility.
6. **Given** denied or escaping paths, ambiguous/ended sessions, replaced targets or unsupported source/context, **When** an edit is requested, **Then** existing authenticated selection, confinement and support restrictions refuse without a fallback target or unauthorized match/source feedback.
7. **Given** a malformed request, missing revision or empty `old_string`, **When** submitted, **Then** it fails without mutation. Empty `old_string` never means append, prepend, create, select the empty file or replace the whole file. An already-empty script cannot be populated by this interface.
8. **Given** a unique literal span whose replacement makes the complete script exceed supported bounds or fail required validation, **When** the request is evaluated, **Then** it refuses before mutation. Fragments need not be valid standalone scripts; the complete original/intended sources remain subject to the existing admission and validation rules.
9. **Given** newer human work, identity/lifecycle invalidation, interruption or loss of required evidence after effects may have begun, **When** success cannot be independently established, **Then** the existing known/partial/unknown-effect outcome is retained. No rollback, definite non-application, replay safety or restored lifecycle is invented; newer work is not overwritten to reassert intent.

### User Story 3 - Keep Native History and Durable Results (Priority: P1)

As a developer, I want a partial-edit request to behave like the trusted existing
edit in Godot, so later Save, history and lifecycle actions do not undo its guarantee.

**Why this priority**: Matching a span is not proof that a native transaction applied
the correct complete source or that the visible editor agrees with persistence.

**Independent Test**: Drive the revised MCP interaction with actual coding-agent
calls and independently observed live editor state. Ordinary editor/fixture
Undo/Redo/Save and lifecycle actions witness behavior; they are not new MCP tools,
edit eligibility workarounds or repairs after reported success.

**Acceptance Scenarios**:

1. **Given** a verified open partial edit and pre-existing native history, **When** genuine native Undo, Save, Redo and Save are exercised with MCP reads and independent witnesses, **Then** the exact pre-edit/intended sources and dirty/saved transitions are observed and prior history remains reachable. A second edit must not simulate Undo/Redo.
2. **Given** successful open, cached-R closed and absent-R closed edits, **When** applicable ordinary Save, close/reopen or later opening, reparse, rescan and fresh runtime launch challenge durability, **Then** the complete intended source persists without human reconciliation. Closed-edit verification already succeeded before later opening; active-game or cached-class hot reload remains outside support.
3. **Given** repeated edits across all three profiles, each based on a fresh read/revision, **When** a real coding-agent workflow interleaves successful edits with dirty, stale, no-match and ambiguous refusals, **Then** it loses no intended change or human work, never changes admitted lifecycle and never automatically retries a refusal. The changed public path demonstrates applicable A–E, including real open-document history, rather than borrowing whole-source client evidence as proof of partial editing.
4. **Given** cancellation, timeout, disconnection or lost response before/after possible effects, **When** the agent observes a deliverable result or reconnects, **Then** structured uncertainty remains truthful and recovery starts with a fresh read of the original explicit target. An automatic client retransmission, if one occurs, is not a new read or a new intentional edit and cannot bypass existing guards.

### User Story 4 - Use One Clear, Versioned Editing Contract (Priority: P1)

As a coding-agent user, I want one discoverable edit representation and a clear
migration from the old contract, so I do not need a custom patch language or guess
which of several overlapping editing modes applies.

**Why this priority**: Renaming public inputs is a compatibility break. Hiding it
under the old version or keeping redundant modes would undermine predictable use.

**Independent Test**: Inspect the versioned catalog and use the revised interface
in both existing supported coding-agent clients. Exercise legacy/mixed requests
and migration guidance without shell/direct-file editing or private tool knowledge.

**Acceptance Scenarios**:

1. **Given** the revised server's ordinary capability/tool discovery, **When** a client inspects the public contract, **Then** it sees exactly `discover_scripts`, `read_script` and `edit_script`, a distinguishable Schema 2 surface and a single edit shape: `project_root`, `script_path`, `revision`, `old_string`, `new_string`, with optional `session_id`. Discover/read behavior remains unchanged apart from the declared surface version.
2. **Given** a Schema 1 caller sends `replacement_source`, alone or mixed with new fields, or supplies a mode/replace-all flag, **When** the revised server receives it, **Then** it rejects the unsupported request without mutation. It does not infer a mode, silently ignore legacy fields, expose a fourth tool or claim Schema 1 compatibility.
3. **Given** updated migration guidance and refreshed tool discovery, **When** each of the existing supported Codex and OMP clients is asked to make a localized edit, **Then** actual agent calls use the returned source/revision to perform replacement and deletion on supported open/closed targets and consume truthful results. At least one workflow demonstrates recovery from no-match and ambiguous refusals by a fresh read and a corrected intentional request. An SDK harness, generated sample call or model claim alone is insufficient.
4. **Given** the revised public descriptions, **When** agents choose an operation, supply its inputs and interpret a failure, **Then** required current revision, unique nonempty search, deletion behavior, lifecycle preservation and safe next actions are discoverable without internal terminology, duplicated safety lectures, exhaustive static failure catalogs or a numeric description budget.

### Edge Cases

- Count distinct possible match starts, including overlaps: searching `aa` in `aaa` is ambiguous. Matching is against the original fresh source once; text introduced by `new_string` is never searched again in the same request.
- Match anywhere within supported text, including part of a line, a comment or a string literal. No line, token, identifier, grapheme or syntax-aware selection is implied.
- Different Unicode encodings of visually identical text remain different. No case folding, Unicode normalization, line-ending conversion or automatic final newline is added.
- Matching the entire nonempty source remains an ordinary unique-span replacement, not a second whole-source mode. Deleting all source is allowed only when the resulting empty script is otherwise admissible; a later request cannot repopulate it with an empty search.
- Wrong-type/null/missing strings, unknown fields and inherited request/source limits retain truthful input refusals. A source unavailable to read is not an empty source.
- A revision is neither a match locator nor a stored permission token. A unique span after an unrelated edit, native Undo or a close/reopen is not grounds to reuse an obsolete revision.
- Pre-effect match refusals preserve source, dirty/saved state, history and lifecycle. After possible effects, later matching or delivery failure cannot relabel the attempt as never applied.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The MCP surface MUST retain exactly `discover_scripts`, `read_script` and `edit_script`. The revised edit MUST accept the single caller shape in US4.1; no fourth tool, alternate edit mode or lifecycle control is added.
- **FR-002**: The normal workflow MUST remain read exact source plus opaque revision/trusted state, submit one old/new pair with that revision, then inspect the result or read fresh state. Read/discover retain their current non-mutating behavior; discovery is not required for a known target.
- **FR-003**: Every well-formed edit candidate MUST freshly authenticate, resolve and acquire its explicit target using the existing trusted workflow. The supplied revision MUST match fresh admissible state before match evaluation can authorize any transformation. Existing later mutation-boundary guards remain authoritative; neither matching text nor caller-supplied state can replace them.
- **FR-004**: Matching MUST be literal UTF-8 text comparison, case-sensitive and exact for spaces, tabs, line endings and Unicode spelling. No trimming, normalization, fuzzy/regex matching, similar-code relocation, line guessing or syntax interpretation is permitted.
- **FR-005**: Exactly one occurrence of nonempty `old_string` MUST exist in the fresh admitted source, counting overlapping occurrences. Zero and multiple matches MUST produce distinguishable structured no-match and ambiguous refusals with no mutation. Callers resolve ambiguity by supplying a larger exact span in a separate intentional request, not by selecting an occurrence number.
- **FR-006**: Replacing that one occurrence with `new_string` MUST define one complete intended source whose prefix/suffix outside the occurrence remain byte-for-byte unchanged. Empty `new_string` MUST support deletion when the complete result is admissible. Empty `old_string` MUST always be rejected, including for an existing script whose freshly acquired source is empty; no initialization exception or implicit meaning is permitted.
- **FR-007**: Complete-source admission, required validation, persistence, synchronization and independent postcondition verification MUST remain mandatory. Fragment-only validation or agreement with only the changed span is insufficient. Required validation failure or an exceeded existing bound before effects MUST refuse without mutation; source MUST NOT be truncated or rewritten to fit.
- **FR-008**: An eligible open edit MUST preserve the same open document, native history, clean/saved success state and independently observed complete intended `D == R == B`. Both supported closed profiles MUST preserve document absence throughout and independently verify intended D and every applicable R. Confirmed absent R MUST not be force-loaded; confirmed absent B/history are not applicable, not invented observations.
- **FR-009**: Existing dirty/equal-text-dirty, stale/divergent, revision/identity/lifecycle, missing-evidence, unsupported-context, authenticated routing and confinement protections MUST remain enforced. A safety/freshness refusal MUST NOT be replaced by match success or a misleading no-match explanation. No automatic repair, Save, force load, open/close or alternate writer may manufacture eligibility.
- **FR-010**: Matching unchanged old/new text MUST obey FR-003–FR-009, including uniqueness, before a verified unchanged outcome is possible. It MUST cause no source/history/lifecycle effect. A missing old span with already-present new text MUST remain a no-match refusal, not idempotent-success recognition.
- **FR-011**: Structured results MUST preserve existing revision/evidence meaning, relevant stages, lifecycle, validation, synchronization, dirty state, history participation, diagnostics and known/partial/unknown effects. New match distinctions MUST be actionable and source-free; no-match suggests checking fresh exact source, ambiguity a larger unique span, and stale state a fresh read/revision. Exact reason identifiers and result schemas belong to planning.
- **FR-012**: Cancellation, timeout, disconnection, delivery failure and newer work after possible effects MUST retain the existing effect-sensitive semantics. No automatic retry, queue, replay, compensation, rollback guarantee or weaker revision policy is introduced. A response acknowledgment alone MUST NOT count as independently verified success.
- **FR-013**: Existing disclosure restrictions MUST cover both input fragments, derived complete source and match diagnostics. Unauthorized source, candidate matches, unrelated context, private routing/validation data and raw request bodies MUST NOT leak through errors, logs or catalog text. Existing disclosure precedence MUST remain authoritative even after an earlier failure or possible effects.
- **FR-014**: This feature MUST deliberately evolve public tool-surface Schema 1 to Schema 2 through a clean cutover with no temporary compatibility period. From Schema 2 delivery, editing MUST accept only the new old/new representation, reject `replacement_source` and mixed/unknown forms without mutation, retain the tool names and provide migration documentation. No dual-mode support, hidden legacy alias or `oneOf` compatibility catalog is permitted.
- **FR-015**: Public descriptions MUST satisfy Constitution Principle IX: concise, task-oriented, stable and non-redundant, sufficient to choose and safely use the tool. Material constraints remain explicit through the input contract and relevant structured outcomes. Safety MUST be implementation-enforced, not dependent on agent instructions; no quantitative token/word/character budget is introduced.
- **FR-016**: The existing exact engine/platform, source/context, local transport, permission, timing, resource and export-isolation boundaries MUST remain intact. Complete edit processing, including new match work, MUST remain within the existing ten-second supported-condition edit bound; read/discover retain five-second bounds. No broader support, arbitrary-load timing, active-game hot reload or new mutation authority is claimed.
- **FR-017**: Both existing supported coding-agent clients MUST demonstrate actual use of the revised representation on all three lifecycle profiles. New real-agent MCP-path evidence MUST establish applicable A–E, exact transformation results, matching/refusal semantics, migration and model-visible results, with independent live-editor witnesses. Shell/file/SDK substitutions and model assurances MUST NOT replace that evidence.
- **FR-018**: Acceptance MUST follow [TEST_POLICY.md](../../TEST_POLICY.md): new representation/composed behavior and directly affected regressions need evidence; valid unchanged historical evidence is reusable after relevant-input review. No blanket historical GUI campaign or new CI/approval mechanism follows from this feature. Feature 007's completed contracts and evidence MUST remain truthful history, not be retroactively rewritten.

**Draft edit description:** “Replace exact text in a script using its current read
revision.” Parameter descriptions carry the unique nonempty search and empty-new-text
deletion rules; existing lifecycle guidance stays concise. Final wording must meet
FR-015, not reproduce this specification in the tool catalog.

### Compatibility and migration decision

**Clarified migration result: a clean Schema 2 cutover with no compatibility period.**
The [existing public contract](../007-mcp-script-workflow/contracts/mcp-interface.md#executable-and-compatibility-surface)
explicitly requires versioning and migration for a breaking schema change. The
repository has completed Schema 1 client workflows but no stated compatibility
window; the public GitHub release listing was empty when assessed. That does not
prove there are no external consumers and is not a claim that compatibility is free.
The deliberate documented break, rather than silent compatibility or retained modes,
is the selected tradeoff for the current small surface.

Schema 2 identifies the revised public tool surface, including its existing result
version declaration. It is not an MCP protocol-version change, a release designation,
a new revision-token authority or an instruction to renumber private integration
contracts. Discover/read keep their meanings; only their declared surface version
and directly necessary edit guidance change. Existing independent local-operation
contracts and trusted whole-source execution remain outside this public cutover.

Migration MUST tell callers to update their expected public schema, refresh tool
discovery, perform a fresh `read_script`, and send its revision with one exact old/new
pair. Remove `replacement_source`; do not automatically translate an old stale
request or silently fall back to filesystem writing. A caller can use the complete
nonempty read source as `old_string` when genuinely replacing all of it, but localized
edits need not resend the rest. All in-repository current MCP callers, examples and
acceptance tooling must migrate when implemented; historical Feature 007 artifacts
remain explicitly Schema 1 history.

**Accepted limitation:** because every empty search is invalid, Schema 2 cannot
populate an already-empty script, including one emptied by a prior admissible
deletion. Read still distinguishes empty from unavailable source. The maintainer
accepted this loss relative to the whole-source input; it must not be masked by an
empty-search convention, initialization exception, fourth tool or retained mode.

### External interface evidence

Reviewed on 2026-10-06. These are public documentation and versioned source
observations, not runtime acceptance of this feature or a universal industry standard.
They inform familiarity only; this project's selected semantics are FR-003–FR-010.

| Interface and evidence | Observed convention | Difference from this feature |
| --- | --- | --- |
| [Claude Code Edit reference](https://code.claude.com/docs/en/tools-reference#edit-tool-behavior), current docs with v2.1.208+ behavior identified | `old_string`/`new_string`, exact text including whitespace, exactly one occurrence unless `replace_all` is explicit; longer context resolves ambiguity. | Current docs allow some unread/changed-file edits when an exact unambiguous match remains and permission permits. This feature always requires the current trusted revision; it does not adopt that relaxation or replace-all. Empty-value behavior is not established by that reference. |
| Cursor [Agent tools](https://cursor.com/docs/agent/overview#tools), [afterFileEdit event](https://cursor.com/docs/hooks#afterfileedit), and [2026-01-08 CLI release note](https://cursor.com/changelog/cli-jan-08-2026) | Current hook documentation exposes `edits` with `old_string: <search>` / `new_string: <replace>`; the release note confirms previous-content reporting. Agent tools are tuned per model. | A post-edit event is not the agent input contract. Reviewed official docs do not guarantee strict uniqueness, normalization or empty-input behavior across Cursor clients/models. A [versioned user report](https://forum.cursor.com/t/entire-installation-is-broken-due-to-apple/149135/19) records `search_replace` empty/no-match/equal-text errors, but is not independently verified current behavior. Cursor is not used to assert universal exact-match semantics. |
| [OMP edit reference](https://github.com/can1357/oh-my-pi/blob/main/docs/tools/edit.md), current public docs, also observed in this session's installed documentation | Supports `replace` with `path`, `old_string`, `new_string`, optional `replace_all`; also `hashline` (default), `patch`, `apply_patch` and `sloppy`. | OMP is not exclusively a string-replacement tool. Its other grammars, snapshot recovery and richer operations are not adopted. The current documentation is not a release-pinned runtime proof. |
| Codex CLI [patch grammar](https://github.com/openai/codex/blob/rust-v0.153.4/codex-rs/apply-patch/src/parser.rs) and [matcher](https://github.com/openai/codex/blob/rust-v0.153.4/codex-rs/apply-patch/src/seek_sequence.rs), released `rust-v0.153.4` | `*** Begin Patch`, file operations, context hunks and changed lines. Matching tries exact lines, then whitespace-tolerant and Unicode-punctuation-normalized forms. | A patch language and tolerant contextual matching, not the selected single unique literal span. No grammar/parser or fallback behavior is copied. |
| [Aider edit formats](https://aider.chat/docs/more/edit-formats.html) and [v0.86.0 edit-block source](https://github.com/Aider-AI/aider/blob/v0.86.0/aider/coders/editblock_coder.py) | Whole-file, SEARCH/REPLACE `diff`, `diff-fenced`, simplified `udiff` and editor variants; format selection varies by model. Released `diff` code tries exact first-match blocks then indentation/blank-line and elision handling. | Search/replacement is familiar, but Aider's delimiters, multiple blocks and permissive matching are not this contract. Its edit-distance helper is unreachable after an unconditional return in the inspected path, not evidence of an active fuzzy fallback. |

The installed local Cursor executable reports 0.40.3, too old to validate current
Cursor agent behavior; no current Cursor runtime claim is made. No Claude, Cursor,
Codex or Aider editing engine was executed for this comparison. Existing Codex/OMP
MCP acceptance establishes the foundation's clients, not acceptance of Schema 2.

### Scope and constitutional alignment

This is a bounded refinement of the delivered Phase 3 script-agent workflow under
Principle IX. [Phase 3 remains complete](../../PHASE_3_EXIT.md#decision); partial
editing is not a newly invented exit obligation. The next broader roadmap capability,
[Phase 4 scene/Resource transactions](../../ROADMAP.md#phase-4--scene-and-resource-transactions),
remains pending. This specification neither starts that phase nor authorizes a release
or the broader Phase 13 enhancement catalog.

Principles I–IV retain independent authorities, human-work protection, native
history and verified mutation. V, X and XII retain confinement, truthful diagnostics
and safety over convenience. VI requires actual live-editor evidence, VII keeps
protocol details out of the trusted core, VIII retains export isolation, and XI
requires independent implementation rather than importing another editor's engine.
Architecture-changing or mutation/security-sensitive planning and review must
explicitly record constitutional compliance; this draft changes no principle.

**Principle XIII:** The present requirement is expressing small edits without
resending unaffected source. Keeping the whole-source input is the simplest
alternative but does not solve that caller cost; a caller helper still sends it.
One old/new pair reuses the existing small catalog and trusted guarantees. Its
ongoing costs are exact-match refusals, one versioned migration, documentation and
focused behavioral/client coverage. Those costs address the requested current
workflow; they do not justify another mutation engine, dependency, framework,
service, approval gate or infrastructure layer. Planning must justify any additional
mechanism against a concrete current need.

**Excluded:** replace-all; custom patch or line-coordinate grammars; unified-diff
parsing; fuzzy, regex or AST editing; occurrence indexes; heterogeneous/batched or
multi-file edits; macros; generic transformation frameworks; automatic retries;
force flags; new lifecycle or Save/Undo/Redo tools; Feature 006 revival; file
creation/rename/removal; scene/general Resource/project-setting authoring; runtime,
debugger or LSP product tools; support/distribution expansion; release automation.
Removing source text is not deleting its file. Full-source trusted execution is a
foundation, not a second public edit representation.

### Key Entities *(include if feature involves data)*

- **Read revision**: The existing opaque stateless, target/state-bound stale-intent precondition. It is not authentication, a stored edit basis, match position or replay token.
- **Exact replacement intent**: Explicit project/script/session selectors, current revision, one nonempty old span and its new text, which may be empty.
- **Admitted current source**: Fresh trusted source whose identity, lifecycle, applicable authorities and safety state permit the requested edit.
- **Complete intended source**: The entire admitted source after the one literal replacement; the independently verified result must agree with this, not merely contain the new span.
- **Edit outcome**: Existing verified-changed/unchanged, refused or effect-sensitive non-success facts, augmented by distinguishable matching refusals and safe next actions.
- **Public contract version**: The declared tool-surface compatibility boundary, independent of transport negotiation, historical acceptance and implementation delivery state.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Actual coding agents in both existing supported clients complete read → exact edit → fresh read on each of the three supported lifecycle profiles. Localized replacement and deletion do not require sending unaffected whole-script source, a patch language or a fourth tool. Independent witnesses establish the exact complete intended source, applicable authority agreement and zero agent-caused lifecycle changes.
- **SC-002**: Multiline/Unicode, whitespace/newline-sensitive, boundary-position, deletion and unchanged-intent cases meet US1/US2 exactly. Every tested zero/multiple/overlapping match or invalid empty search refuses without effects; no tolerance or first-match fallback succeeds accidentally.
- **SC-003**: Every exercised stale, dirty/equal-text-dirty, divergent, missing-evidence, identity/lifecycle, routing, confinement, validation and support failure preserves its required refusal/protection. A stale revision is refused even when old text still matches. There are zero unsafe overwrites, guessed targets or success claims based only on matching.
- **SC-004**: The revised public interaction demonstrates applicable real-agent A–E and durability on the supported real editor, including genuine Undo/Redo and prior-history preservation for open edits, sequential edits/refusals across all profiles and applicable later Save/reopen/reparse/rescan/runtime agreement. No human reconciliation, synthetic history or invented closed-buffer authority is required.
- **SC-005**: Every delivered result distinguishes the tested no-match, ambiguous, freshness/safety, invalid-source and interrupted-effect cases sufficiently for the correct safe next action. There are zero false success, rollback, definite-not-applied or authorized-replay claims and zero prohibited fragment/source disclosures.
- **SC-006**: Catalog/migration checks establish exactly three tools and one Schema 2 edit shape. Legacy-only, mixed and unsupported-mode requests produce zero mutations. Current repository MCP callers and instructions use the revised contract, while Feature 007 history still truthfully describes Schema 1 whole-source editing.
- **SC-007**: Real-agent use establishes that the short descriptions and structured contracts suffice to choose tools, quote exact source, use current revisions, resolve ambiguity and interpret uncertainty without custom grammar or private architecture knowledge. This is qualitative interface acceptance, not a token/character benchmark.
- **SC-008**: Supported-condition calls retain existing five-second read/discover and ten-second edit bounds, including matching and required validation/verification/delivery. Relevant-input review accounts for every inherited guarantee and limitation using new, affected or valid reusable evidence; design quality alone is never reported as product or release acceptance.

## Assumptions

- Feature 007 T001–T004 and Phases 1–3 are complete. Their historical whole-source contracts, accepted evidence and separate phase-exit decisions remain unchanged; Feature 008 owns the public migration and its new acceptance.
- The inherited environment remains official Godot 4.7.2.stable.official.ed1daf0bf on macOS 26.6.2 (25G83) arm64 with the documented four-vCPU/6-GiB profile, local stdio MCP 2025-11-25 and existing Codex CLI 0.153.4 / OMP 18.5.1 compatibility targets. This draft does not claim those clients already support Schema 2.
- Existing standalone-project-GDScript admission, source/context limitations and [bounds](../007-mcp-script-workflow/data-model.md#2-supported-profile-and-bounds) remain binding: complete source is limited to 512 KiB, with no normalization of rejected NUL/CR/BOM input. Exact matching does not expand supported scripts or make CRLF admissible. Loaded source coherence does not promise cached class or running-game hot reload.
- The developer still supplies deliberate supported local setup and editor startup. A client can present explicit selectors and consume structured results. No special agent extension, new client-side safety store or configuration framework is assumed.
- The maintainer selected a clean Schema 2 cutover without a temporary compatibility period. Callers must migrate when Schema 2 is delivered; this is a deliberate breaking change, not a claim of no external usage.

### Material clarification review points

Clarification completed on 2026-10-06 with two accepted answers:

1. **Compatibility result — resolved:** Schema 2 replaces Schema 1 editing without a dual-mode period. Callers must migrate; no temporary compatibility is required by this feature.
2. **Empty-source consequence — resolved:** Reject every empty `old_string`, including when the target is already empty. The inability to populate an empty script through this tool is an accepted limitation, not deferred initialization work.
3. **Exactness edge defaults — clear:** Overlapping occurrences remain ambiguous; identical old/new text may produce a fully checked verified-unchanged outcome. Existing uniqueness and unchanged-intent requirements already settle these cases, so no additional question or behavior change was needed.

The Clarifications section records only the two actual maintainer answers. No
material product ambiguity remains; technical choices below belong to planning.
Clarification completion does not authorize implementation or claim acceptance.

### Decisions reserved for planning

Planning must determine the smallest implementation consistent with these outcomes:
where exact intent is checked and complete source is derived, how existing trusted
whole-source execution is reused, concrete input/output schemas and reason codes,
compatibility/version advertisement details, and affected acceptance ownership.
Deriving one complete intended source for the existing trusted mutation path is
acceptable to evaluate, not a mandated module, algorithm or second partial-mutation
engine. No plan, data model, technical contract, task list or implementation is
selected by this specification.
