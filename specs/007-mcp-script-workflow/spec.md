# Feature Specification: Use the Trusted GDScript Workflow Through MCP

**Feature Branch**: `spec/007-mcp-script-workflow`

**Created**: 2026-10-03

**Status**: Requirements approved — planning authorized by the maintainer; implementation is not authorized.

**Input**: Generated WHAT/WHY description supplied to `/speckit.specify`:

> Expose the completed trusted GDScript workflow to real coding agents through MCP. A developer already has a supported live Godot Editor and the completed observation, discovery, opening, editing and closing capabilities, but a coding agent cannot discover and invoke them through MCP. Provide a small composable external surface for those five operations so the agent can find a script, open it, independently inspect actual editor/disk state, submit a guarded persisted edit, freshly observe, safely close and deliberately reopen it without shell glue or manual tab preparation.

> Preserve explicit authenticated project/session routing, project confinement, independent disk/loaded-Script/visible-buffer evidence, document-attributed dirty state, target-bound revisions and fresh mutation-boundary checks. Observation and discovery remain read-only; dirty or divergent observations remain useful information rather than edit permission. Opening and closing retain their distinct lifecycle and no-change recognition semantics. Edit success retains edit-owned persistence, independent intended D/R/B convergence, clean/saved state, required source validation and real native history. Refuse ambiguous or replaced targets, stale or missing bases, dirty work including equal-text dirty state, unavailable safety evidence, denied access and unsupported effects without repair, force, implicit Save, retargeting or automatic replay.

> Supply discoverable consistent machine-readable tool contracts, accurate supported-capability information, structured outcomes and actionable protocol/request versus operation-execution errors. Map cancellation and connection loss without claiming rollback or concealing known/partial/unknown effects. Preserve existing bounded operation timing, request correlation, source disclosure rules, logging separation and production-export isolation. MCP details stay above the reusable automation and Godot integration; reuse the complete trusted execution paths rather than constructing a second safety policy.

> Prove real external MCP interoperability and real coding-agent use of the composed workflow, including new MCP-path A–E evidence. Ordinary human/editor Undo, Redo and Save remain the existing acceptance interactions for native history and durability, while the agent's product operations and observations go through MCP; this feature adds no standalone history or Save commands. Reuse valid unchanged native/editor evidence under TEST_POLICY.md and newly exercise adapter behavior and affected composed paths instead of replaying historical campaigns by default.

> This is the first selected Phase 3 capability, not a declaration of phase completion or a release. Preserve the exact existing Godot/platform and operation-specific support boundaries. Exclude standalone Save/Undo/Redo, Feature 006 revival, arbitrary commands/evaluation/filesystem access, remote access, new Godot capabilities, broader support, scenes/resources/project settings, runtime/debugger/LSP tools, batch/concurrent-agent orchestration, tool-catalog expansion, broad API reshaping and generic transaction/workflow/retry/cancellation/redaction/tracing/capability frameworks. Basic local connection guidance is required; distribution expansion is not. Leave released MCP version, SDK/library, transport, process/runtime shape, schema details and interoperability selections to evidence-backed planning.

The generated Input above is retained verbatim as historical provenance. Its original
surface and scope assumptions are superseded by the following maintainer decisions
and the clarified normative requirements below.

## Clarifications

### Session 2026-10-03

The maintainer supplied these product decisions directly; no additional questions
were asked. The five entries group the ten supplied decisions without changing them.

- Q: Which coding-agent intents belong in the public MCP surface? → A: Discover scripts, read/inspect one script and edit one script. Exact tool names remain for planning. Opening/closing are not MCP product intents; Features 003/005 remain completed capabilities and reusable evidence, and ordinary editor or fixture lifecycle actions remain available for acceptance.
- Q: What does reading a script mean? → A: Expose trusted Feature 001 observation semantics, including source, target/document identity, revision/edit-basis evidence, open/closed state, dirty attribution, applicable D/R/B, divergence, availability and invalidation. Reading is non-mutating, does not open a closed document and never authorizes editing by itself.
- Q: How must an edit relate to document lifecycle? → A: Preserve the observed lifecycle: an eligible unchanged open document stays open; an eligible unchanged closed target is edited without opening a document and stays closed. Changed revision, identity, editor state or required safety evidence requires refusal and a fresh read/basis, not implicit opening, closing or branch switching to manufacture eligibility.
- Q: What is new about closed-script editing, and what history does it require? → A: It is a new scoped capability, not Feature 002 behavior. Planning must research a minimum Godot-authoritative route and loaded-Resource applicability, independently verify coherent persistence and report effects truthfully. Confirmed closed B is not applicable, and no buffer history entry is required; open edits retain Feature 002 native history. If no safe route exists, planning must report the blocker and return to the product/spec decision, never substitute direct writing, force loading, hoped-for reload, implicit opening or weaker verification.
- Q: How must acceptance, evidence and exclusions change? → A: MCP product calls are discover/read/edit. A/B prove trusted editing and human-work protection; C uses ordinary native Undo/Redo/Save for applicable open edits plus MCP reads and independent witnesses; D uses editor/fixture close/reopen; E uses sequential fresh-basis MCP edits and dirty/stale refusals. Closed mutation needs new focused deterministic/core/boundary and applicable real-Godot evidence; unchanged existing evidence follows TEST_POLICY.md. Standalone Save/history tools, Feature 006 revival, broader Godot/platform or unrelated capability expansion and generic frameworks remain excluded; protocol/SDK/transport/runtime/schema choices remain planning research.

## User Scenarios & Testing *(mandatory)*

The primary user is a coding agent acting for a developer who may continue working
in a live Godot Editor. The public MCP surface represents three semantic intents:
**discover scripts**, **read/inspect one script**, and **edit one script**. Read
means trusted editor observation, not a filesystem-only read. Existing capabilities
supply discovery, observation and already-open editing; safe closed-script editing
is a new requirement of this clarified feature, not an existing Feature 002 capability.
Developer installation, client configuration and deliberate editor startup remain
setup, not agent tools. Opening and closing editor documents are not public MCP
operations in this feature.

**D** is disk source, **R** is loaded Script source, and **B** is the visible editor
buffer. They remain independent authorities. An open edit retains independently
verified intended `D == R == B`, clean/saved state and native history. A closed edit
requires independently verified coherent persistence across all applicable authorities
and confirmed continued document absence; B is not applicable, never invented.
Loaded R must not be left stale, and inability to observe it is not proof of absence.
A protocol response or disk write alone is not editor success.

Stories describe separately verifiable outcomes of this agent-facing capability.
Safety, privacy, bounded execution and truthful results are inseparable from every
exposed operation, including the new closed-script mutation path.

### User Story 1 - Connect and Discover the Supported Workflow (Priority: P1)

As a coding-agent user, I want to connect my existing MCP client to the kit and learn
which script operations it actually supports, so I can use the live editor without
private integration knowledge.

**Why this priority**: The current library and one-operation callers do not provide an
external MCP connection or discoverable tool contracts.

**Independent Test**: Configure an independent external MCP client against the supported
local setup, connect, discover the tools, inspect their input/result descriptions and
make a read-only request. Exercise incompatible negotiation and unavailable-editor
cases without causing an editor operation accidentally.

**Acceptance Scenarios**:

1. **Given** documented local setup and a supported external client, **When** the client connects and requests available tools, **Then** it discovers the three semantic operations—discover scripts, read/inspect one script and edit one script—with consistent machine-readable inputs, result meanings, effect descriptions, limits and prerequisites. Exact tool names are a planning/schema decision. The catalog contains no project source, inventory, credentials or first-class open/close/Save/history operations.
2. **Given** a client that cannot agree on a supported protocol or required capability, **When** it connects or requests an unsupported protocol feature, **Then** it receives an explicit compatible failure without dispatching a Godot operation or claiming a successful connection with nonexistent capabilities.
3. **Given** the protocol service is reachable but the selected editor is unavailable, incompatible or lacks an operation capability, **When** the agent requests that operation, **Then** the result distinguishes editor availability/support from protocol connection success. Tool presence never guarantees that a particular editor or script is eligible.
4. **Given** a malformed or out-of-bound tool request or an unknown tool, **When** the client submits it, **Then** it receives the appropriate machine-readable request or execution failure without unsafe effects, payload echo or a fabricated operation result. Valid requests remain subject to the ordinary core checks.
5. **Given** the public descriptions and machine-readable contracts, **When** a coding agent chooses and uses discover/read/edit and interprets a refusal or uncertain result, **Then** it can identify required inputs, current-basis/lifecycle prerequisites, material limitations, outcome meaning and safe next action without internal architecture knowledge or repeated safety explanations. Structured distinctions and targeted outcome details supply progressive disclosure rather than exhaustive static failure catalogs; safety does not depend on the agent obeying prose.

---

### User Story 2 - Read and Edit Without Managing Editor Documents (Priority: P1)

As a coding agent, I want to find, inspect and edit a script without deciding whether
the developer should have its document open, so the requested source change does not
create or remove editor tabs.

**Why this priority**: The product intent is a safe source change, not orchestration
of the internal operations. Existing open-document editing cannot satisfy a closed
target without a new safely bounded capability.

**Independent Test**: Ask a real coding agent to discover, read and edit eligible
initially open and initially closed scripts in an explicit project/session. Compare
source, revisions, applicable authorities and document identities independently.
No shell/direct-file/private-bridge action may replace an agent-facing call, and no
opening or closing may be used to make the edit eligible or conceal its effects.

**Acceptance Scenarios**:

1. **Given** an eligible closed script among distinguishable project paths, **When** the agent discovers it, reads its trusted state and submits an edit with a current closed-target basis, **Then** success establishes the intended persisted source and coherence of every applicable loaded authority while the document remains closed throughout the operation. No editor document or buffer is created, no Resource is force-loaded to manufacture evidence, and no automatic open/edit/close composition is used.
2. **Given** an eligible clean open document that remains the same document at the mutation boundary, **When** the agent reads, edits with a current basis and reads again, **Then** the existing trusted live-editor edit independently establishes intended D/R/B, clean/saved state and required validation while preserving that open document and its native-history guarantee.
3. **Given** dirty, divergent, closed or partly observable state, or an incomplete discovery scope, **When** the agent reads or discovers through MCP, **Then** source, identity, revision/edit-basis evidence, open state, dirty attribution, applicable D/R/B, divergence, availability and invalidation remain distinguishable. A closed read opens nothing, an unavailable R is not asserted absent, and a read or complete listing does not authorize editing.
4. **Given** a known target and usable current read/basis, **When** the agent directly requests an edit, **Then** discovery is not a prerequisite. Already-satisfied intent requires the same lifecycle, target, safety and applicable-postcondition checks and causes no source, history or lifecycle change.
5. **Given** a verified closed-script edit, **When** the developer or controlled acceptance fixture later opens the script and the agent reads it, **Then** the intended persisted source is retained and applicable D/R/B agree. That later lifecycle action witnesses durability; it is neither an MCP tool nor the closed edit's method of verification.

---

### User Story 3 - Keep Human Work and Target Boundaries Intact (Priority: P1)

As a developer, I want agent editing to protect human work, target identity and the
document's observed lifecycle, so an agent cannot overwrite newer state or change
tabs merely to satisfy an implementation precondition.

**Why this priority**: Reducing the tool surface must not weaken safety, and adding
closed-script editing must not turn disk access or lifecycle changes into a bypass.

**Independent Test**: Submit MCP requests against independently witnessed dirty,
stale, ambiguous, replaced, denied and unsupported targets. Include changes between
observation and effect entry, and inspect both permitted results and preserved state.

**Acceptance Scenarios**:

1. **Given** an open target has unsaved human work, including equal-text-but-dirty state, **When** editing is requested, **Then** it refuses while preserving text, dirty state, open identity and native history. The adapter does not Save, discard, merge or force the document to make it eligible.
2. **Given** a missing, unusable, wrong-target or stale expected basis, including same-text version changes or a closed/reopened buffer, **When** editing is requested, **Then** target-bound revision and identity checks prevent stale action. A closed read is not silently converted to Feature 002's open-document basis; a fresh read/basis is required.
3. **Given** ambiguous same-project sessions, an ended explicit session, denied access or an escaping path, **When** a tool is called, **Then** no focused, convenient or replacement target is selected and no candidate/unauthorized source or inventory is exposed. Source-free selection feedback remains usable.
4. **Given** unknown open state, missing safety evidence, dirty or conflicting loaded state, or an unsupported source/effect/context profile, **When** editing is requested, **Then** it refuses rather than force-loading, normalizing, repairing or weakening verification. For closed targets, confirmed Resource absence and an unreadable existing Resource remain distinct; the absence of B does not imply a clean or current R.
5. **Given** newer human work or a lifecycle change appears after an effect begins, **When** verification detects invalidation, **Then** newer state survives and the agent receives truthful non-success with known/partial/unknown effects. The operation does not reassert old source or open/close a document to restore the previous appearance.
6. **Given** overlapping requests reach the same editor, **When** existing admission refuses one, **Then** that refused request is not queued or replayed later. Request correlation stays distinct, and cancellation of one call does not become authority to cancel or retarget another.
7. **Given** the target changes from closed to open or open to closed between the read and mutation boundary, **When** the old edit is attempted, **Then** it refuses and requires a fresh read/basis instead of silently using another lifecycle branch. Same-path replacement and newly opened buffer identities cannot inherit the old intent.

---

### User Story 4 - Understand Failures and Interrupted Effects (Priority: P1)

As a coding agent, I want protocol failures and editor-operation outcomes to remain
interpretable, so I can stop or read fresh state instead of blindly repeating an edit.

**Why this priority**: A successful protocol exchange can carry an unsafe or unverified
operation result, and a lost exchange cannot establish that no effect occurred.

**Independent Test**: Exercise invalid requests, refusals, host/delivery failure,
cancellation and connection loss before and after effect authorization for open and
supported closed edits. Compare deliverable results with independent survivor-state
witnesses and use a separate MCP read where the connection permits.

**Acceptance Scenarios**:

1. **Given** a valid tool call whose operation refuses, partially applies or cannot verify its result, **When** a response is delivered, **Then** the client receives the operation's target, reason, stage, effect certainty, permitted evidence and safe next action in structured form. Transport success or a generic error flag does not replace those facts.
2. **Given** cancellation or deadline expiry before or after possible effects, **When** the attempt terminates, **Then** existing proven-not-applied, known-applied, partly-applied and unknown distinctions are preserved. A proven refusal cannot apply later; a missing acknowledgment is not no-effect evidence.
3. **Given** the client disconnects, cancels delivery or stops reading a result, **When** the adapter cannot deliver terminal evidence, **Then** it does not claim successful delivery, rollback or safe replay. Recovery requires a fresh read of the original explicit target before another intentional action; no persistent outcome service or automatic retry is implied.
4. **Given** an unresponsive editor or blocked operation under supported local conditions, **When** a request is made with output consumed, **Then** the bounded read/discover/edit contract produces a truthful terminal result. Protocol handling remains able to receive cancellation; subordinate work or orchestration cannot renew the budget. Closed-edit timing is a new requirement to prove, not inherited execution evidence.

---

### User Story 5 - Retain Native History and Durable Results in Real Agent Use (Priority: P1)

As a developer, I want MCP editing to preserve native history where an open document
exists and coherent durable source where the target remains closed.

**Why this priority**: Internal-harness success or an advertised undoable flag does not
prove that a real agent's new protocol path preserves those guarantees.

**Independent Test**: Drive discover/read/edit through a real coding-agent MCP client,
with independent live-editor witnesses. Ordinary developer/editor or controlled fixture
actions establish dirty work and invoke Undo/Redo/Save, close/reopen, reparse/rescan
and permitted runtime checks. They witness guarantees rather than serve as MCP tools,
make a closed edit eligible or repair divergence after reported success.

**Acceptance Scenarios**:

1. **Given** clean open and dirty-human-buffer fixtures, **When** a coding agent edits through MCP, **Then** A establishes intended independent D/R/B and clean/saved state, while B preserves the human work and returns safe refusal, including dirty-equal-text cases. Separate positive closed-edit evidence complements rather than replaces A.
2. **Given** a verified MCP edit of an open document and pre-existing native history, **When** ordinary editor Undo, Save, Redo and Save occur with MCP reads and independent witnesses, **Then** C verifies genuine reversal/reapplication and preserved prior history. Unsaved history states remain truthful reads, not falsely persisted convergence or edit permission. Closed edits require no invented editor-buffer history entry.
3. **Given** a successful open or closed MCP edit, **When** ordinary editor or controlled fixture lifecycle actions exercise applicable close/reopen, with MCP reads and independent Save/reparse/rescan/permitted-runtime witnesses, **Then** D and durability retain the intended source without reconciliation. No MCP open/close call is required; target-buffer history persistence after disposal is not promised.
4. **Given** twenty sequential successful MCP edits covering supported open and closed target profiles, each with a fresh read/basis and at least three dirty and three stale refusals interleaved, **When** ordinary Save is exercised where applicable and editor/fixture lifecycle actions check final persistence, **Then** E loses no intended revision or human work and conceals no conflict. Every edit preserves its admitted lifecycle; native-history checks apply to open edits.
5. **Given** the closed-edit profiles established by planning, **When** focused deterministic/core/boundary and real-Godot cases exercise positive edits, loaded-Resource presence or confirmed absence, stale/lifecycle races, missing evidence and interruption, **Then** independently observed applicable authorities and continued buffer absence match each reported result. Unsupported Resource-state profiles refuse explicitly; refusing every closed edit or reusing Feature 002's open-edit passes does not establish this new capability.

### Edge Cases

- Protocol connection success, tool advertisement and editor compatibility are separate facts; none proves operation eligibility.
- Source, path, identity, revision or state-bound basis values must not be normalized into another target or usable authorization; closed targets do not acquire invented buffer-version fields.
- Empty source, unavailable source, invalidated evidence, confirmed Resource absence and confirmed buffer absence must remain distinguishable. Closed B is not applicable; missing required R evidence is not.
- A large source, basis, inventory or outcome reaches supported bounds: preserve truthful refusal/limitation semantics without truncation or dropped safety fields. Planning must establish closed-edit limits rather than infer support from open-edit evidence.
- A discovered path disappears, or a session, Resource, file or buffer is replaced between calls: every edit rechecks its actual target and observed lifecycle; it does not silently choose the alternate open/closed route.
- An earlier causal failure is followed by disclosure denial: prohibited source summaries and bodies remain suppressed without erasing known effects.
- A client presents the same request again after losing its reply: correlation is not a replay guarantee, and the adapter supplies no automatic replay path.
- Cancellation arrives after authorization or while result delivery fails: loss of a response must not be translated into definite non-application.
- Help, startup failures, worker output and diagnostic logging must not contaminate protocol traffic or expose project content.
- Human Undo may leave dirty state legitimately; neither automatic Save nor a second source edit may substitute for genuine history traversal.
- A closed target has a loaded Script whose source or unsaved state conflicts with D: no disk-only write, cache eviction, force load or hoped-for reload may conceal that authority.
- A human opens a closed target during editing, or closes an open target: protect the newer state, report invalidation/effects, and never close or open it back as compensation.
- Closed-script safe execution proves unavailable in planning: report the concrete blocker and revisit the product/spec decision, not an open-first workaround or an implementation that refuses every supported positive case.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: A real external MCP client MUST be able to connect using documented local setup and invoke the clarified discover/read/edit workflow. Compatible released protocol/library choices and concrete client compatibility MUST be established in planning and acceptance, not inferred from an internal harness or existing local callers.
- **FR-002**: The public MCP surface MUST represent exactly three semantic operations: discover scripts, read/inspect one script and edit one script. Descriptions and machine-readable contracts MUST expose their effects, inputs, results, limits and prerequisites consistently. Exact wire/tool names remain a planning/schema decision. Opening and closing are not first-class MCP operations or implicit eligibility steps in this feature.
- **FR-003**: Inputs and outputs MUST receive syntactic and semantic validation appropriate to their boundary. Unsupported tools, malformed values, wrong types, out-of-bound data and incompatible negotiation MUST fail explicitly. Exact source, paths, identity/version values and basis meaning MUST not be silently truncated, normalized or weakened; invalid output after possible effects MUST not become false success or false not-applied state.
- **FR-004**: Every operation MUST retain explicit authenticated project/session selection, confinement and lifetime binding. Ambiguity, denied access, ended/replaced sessions and changed target identity MUST not select by focus, working directory, earlier basis or convenient fallback, disclose unauthorized data or launch an editor.
- **FR-005**: Read-script MUST expose trusted Feature 001 observation semantics, not a filesystem read: source, target/document identity, revision/witness and edit-basis evidence, open/closed state, attributed dirty state, applicable D/R/B, divergence, availability, invalidation and observation interval. Reading a closed target MUST not open or force-load it. Dirty/divergent and limited states remain valid information, never editing authority; confirmed absence MUST remain distinct from missing required evidence.
- **FR-006**: Discovery MUST preserve Feature 004's exact scoped, source-free, fresh inventory and complete/limited/refused/interrupted distinctions. A listing is not a source read, edit basis, current document identity or permission to act; subsequent operations MUST make their own current checks.
- **FR-007**: Edit-script MUST use a fresh state-bound expected revision/identity basis and recheck target, observed editor state, lifecycle and required safety facts at the mutation boundary. An eligible open target MUST remain the same open document; an eligible closed target MUST remain closed without opening a document. Changed revision, identity, relevant editor state, lifecycle or required safety evidence MUST refuse with a fresh-read/basis next action. No implicit open, post-edit close, force load or automatic branch switching may manufacture eligibility or restore appearances.
- **FR-008**: For a target that remains the same eligible clean open document, editing MUST preserve Feature 002's trusted live-editor behavior: target-bound stale protection, native mutation/history, edit-owned persistence, independently verified intended D == R == B, clean/saved state and required source validation. The operation MUST leave that document open. Dirty, stale, divergent, unsupported or unobservable safety state MUST refuse without an alternate writer.
- **FR-009**: Safe closed-script editing MUST be treated as a new narrowly scoped capability, not Feature 002 behavior; Feature 002 intentionally refuses closed targets. Planning MUST research the minimum Godot-authoritative route, identify loaded Script/Resource states, establish a checked closed-target basis and define independently verifiable applicable postconditions. Success MUST establish intended persistence, coherent applicable R and continued document absence, with B explicitly not applicable. Dirty/conflicting R or missing required evidence MUST not be bypassed; a disk write that can leave observable R stale is insufficient. No editor-buffer history entry is required for a target that remains closed. If no safe supported route exists, planning MUST report the blocker and return to the product/spec decision, never silently substitute direct writing, implicit opening, force loading, hoped-for reload or weaker verification.
- **FR-010**: MCP MUST remain above the protocol-independent automation and Godot integration. Reuse existing routing, confinement, revision/conflict policy, admission, acquisition/verification, effect reporting and disclosure ownership. Any narrowly required closed-edit capability and state-bound basis MUST belong below the adapter under the same safety invariants, not be simulated by MCP orchestration or client-provided safety assertions. This does not justify generic framework extraction or redesign of completed operations.
- **FR-011**: Deliverable results MUST preserve requested/resolved target, correlation, lifecycle state, relevant stages, complete/limited reads or inventories, verified changed/unchanged edits, refusals, known/partial/unknown effects, applicable evidence, unavailable/invalidated facts, diagnostics and safe next action. Closed-edit results MUST state actual loaded-Resource and buffer applicability without inventing history participation. Protocol success, acknowledgment or serialization alone MUST not become operation success.
- **FR-012**: Protocol/request errors, tool-execution outcomes and host/delivery failures MUST remain distinguishable. A private editor-protocol failure after possible effects MUST retain effect knowledge rather than collapse into a generic malformed-client-request response. Known stale Resource/buffer, dirty conflict, revision mismatch, parse/validation failure, disconnection, timeout and unsupported/denied capability remain distinct where applicable.
- **FR-013**: Protocol cancellation and connection/process lifecycle MUST preserve existing effect-sensitive termination, newer-human-work protection and owned-work cleanup. Cancellation is scoped to the intended request, not another request or the user's editor. When the chosen protocol permits result delivery, retained terminal facts MUST remain available; when delivery is impossible or suppressed by protocol cancellation semantics, no successful-delivery, rollback or definite-no-effect claim is permitted.
- **FR-014**: Controlled local MCP calls MUST retain five-second read/discover bounds and a ten-second edit bound, including operation-input validation, selection, execution and bounded delivery with the client consuming output. The edit bound is retained for open edits and a new requirement to establish for closed edits, not a claim of existing closed-edit performance. Planning MUST define reproducible admission/delivery measurement and bounded connection behavior without excluding work or renewing cutoffs. No throughput or arbitrary-project-size promise is added.
- **FR-015**: The adapter MUST not automatically retry, queue a refused edit, reconnect to a replacement editor, compensate or replay an uncertain action. After possible effects, guidance requires a fresh MCP read of the original explicit target and a new eligible state-bound basis before another intentional edit. Request correlation is not an idempotency or recovery token.
- **FR-016**: Only supported protocol/tool capabilities may be advertised. Server tool support MUST be distinguishable from current editor/session/script availability and eligibility; an unavailable editor or unsupported operation MUST yield truthful non-success. Existing selected-target and diagnostic information MUST support agent decisions without exposing private endpoints, credentials or candidate project contents.
- **FR-017**: Protocol traffic MUST remain separate from incidental logs and diagnostics. Requested authorized results alone may disclose their permitted selected source/inventory/evidence. Raw requests, source, unauthorized paths, credentials, private validation content and source-derived records suppressed by the core MUST not leak through protocol errors, logging, startup output or tool discovery. Existing operation-specific disclosure precedence remains unchanged.
- **FR-018**: Existing Features 001–005 public local-operation contracts, support profiles, bounds and refusal meanings MUST remain valid and unchanged, including Feature 002's closed-target refusal. Feature 007's new closed-edit behavior MUST have its own explicit supported profile and evidence; it is not retroactive expansion of Feature 002 acceptance. New MCP schemas/outcomes are deliberate compatibility surfaces with documented versioning/migration for breaking changes. No broader Godot/platform support, arbitrary authority or stronger isolation/atomicity claim follows.
- **FR-019**: Real external-client interoperability and real coding-agent discover/read/edit execution MUST be demonstrated. New MCP A/B prove trusted open editing and human-work protection; C proves native history for applicable open edits through ordinary Undo/Redo/Save plus MCP reads and independent witnesses; D uses editor/fixture close/reopen; E uses fresh-basis sequential MCP edits and dirty/stale refusals. Applicable durability remains required without MCP lifecycle/history tools. Positive lifecycle-preserving closed edits MUST also be proved; refusal-only, disk-only or mock-only behavior is insufficient.
- **FR-020**: Valid unchanged open-document observation/edit, historical opening/closing and other accepted evidence MUST be reused under TEST_POLICY.md after relevant-input review. Closed mutation MUST receive new focused deterministic/core/boundary and applicable real-Godot evidence for authority applicability, coherent persistence, lifecycle preservation, races, refusals and effect-sensitive interruption; Feature 002 acceptance does not prove it. Adapter mapping, cancellation, advertisement, traffic/privacy and composed MCP behavior also need new evidence. Changed relevant inputs require affected regressions, not automatic historical full campaigns.
- **FR-021**: Local-only operation, no telemetry, preservation of human work, existing permission/confinement boundaries and tested tooling/export isolation MUST remain intact. Basic client connection/setup, exact compatibility and verified limitations MUST be documented before support is claimed. This feature adds no arbitrary evaluation/process/filesystem capability, remote access, gameplay authority or distribution expansion.
- **FR-022**: Public MCP tool/parameter descriptions, capability text, result guidance and equivalent caller-facing prose MUST be concise, task-oriented and non-redundant, communicating only what a coding agent needs to choose discover/read/edit, supply valid required inputs, interpret structured results and take the appropriate safe next action.
  Material prerequisites, limitations and operation distinctions MUST remain explicit: discover identifies supported script targets; read returns trusted current script/editor-aware state and the basis needed for editing without opening a closed target; edit requires a current trusted basis and preserves the admitted lifecycle.
  Interfaces SHOULD prefer structured schemas/outcomes, explicit typed/state distinctions and progressive disclosure over prose duplicating structural information or common rules. Failure details and next-action guidance SHOULD be targeted to the returned outcome rather than exhaustively preloaded into static tool descriptions. Unnecessary internal terminology/architecture, repeated safety rationales and duplicated caveats MUST NOT burden public descriptions.
  Lean prose MUST NOT weaken implementation-enforced authenticated routing, confinement, target/revision/stale checks, dirty-work protection, permissions, lifecycle preservation, disclosure/redaction or truthful effect semantics; prose MUST NOT compensate for missing enforcement.

### Scope and Governance Alignment

This is a clarified capability of **Phase 3 — MCP adapter MVP**. The current
[source manifest](../../mcp-server/Cargo.toml) exposes local callers, and the
[Phase 2 exit](../../PHASE_2_EXIT.md#thin-adapter-conclusion) establishes reusable
trusted execution. That conclusion remains valid for completed operations; it does
not claim that the newly requested closed-script mutation already exists.

The public surface is organized by coding-agent intent, not by mirroring internal
operations:

| MCP semantic operation | Existing foundation and new obligation | User-facing distinction |
| --- | --- | --- |
| Discover scripts | [Feature 004](../004-discover-project-gdscript/spec.md#functional-requirements) | Scoped locations and honest coverage, not source or edit authority. |
| Read/inspect one script | [Feature 001](../001-observe-gdscript-state/spec.md#functional-requirements) | Trusted source/state and revision evidence; closed reads do not open, and no read authorizes editing by itself. |
| Edit one script | [Feature 002](../002-edit-open-gdscript/spec.md#functional-requirements) for eligible open documents; new researched closed-edit capability under FR-007–FR-010 | Preserve admitted lifecycle; independently verify intended persistence across applicable authorities. |

[Feature 003 opening](../003-open-project-gdscript/spec.md) and
[Feature 005 closing](../005-close-project-gdscript/spec.md) remain valid completed
core/product capabilities and evidence. They are not deleted, weakened or rewritten,
but are not advertised as first-class Feature 007 MCP operations. Ordinary editor
or controlled fixture actions may exercise opening/closing for acceptance without
making them agent tools or using them to implement a closed edit.

**New capability boundary:** Feature 002 intentionally edits only already-open scripts
and refuses closed targets. Closed editing is therefore new scoped core/integration
work required by the lifecycle-preserving product intent, not adapter wiring alone.
Planning must establish which loaded Script/Resource states are supported and how
their applicable preconditions, dirty/conflict checks, revision/identity binding,
source validation, synchronization and independent postconditions are proved without
opening a document. Confirmed B absence makes B not applicable; unknown R state or an
unreadable loaded R cannot be relabeled absent. If no safe Godot-authoritative route
can satisfy the requirement, planning must stop dependent work, report the concrete
blocker and return to the product/spec decision. Direct writing, implicit opening,
force loading, hoped-for reload and weaker verification are not substitute scopes.

**History and Phase 3 acceptance interpretation:** Agent-facing calls in A–E are
discover/read/edit. A/B retain trusted editing and human-work protection. C retains
[Feature 002 US4](../002-edit-open-gdscript/spec.md#user-story-4---reverse-and-reapply-through-native-history-priority-p1)
for open-document edits using ordinary native Undo/Redo/Save plus MCP reads and
independent witnesses. No buffer history entry is required for closed edits because
no buffer exists; planning must record actual applicable authority and history
limitations, not fabricate a history guarantee. D uses ordinary editor or controlled
fixture close/reopen; E uses sequential MCP edits with fresh bases and required
dirty/stale refusals. Ordinary Save, reparse, rescan and permitted runtime actions
witness durability, not MCP tools or manual repair after false success. New closed-edit
proof supplements, rather than substitutes for, applicable A–E.

The [Phase 1 interpretation](../../PHASE_1_EXIT.md#savehistory-interpretation) and
[Feature 006 decision](../006-save-project-gdscript/decision.md#decision) remain valid:
standalone Save/history tools are excluded, and Feature 006 is assessed, not proceeding.
This draft does not declare Phase 3 complete; its exit requires delivered evidence.

**Excluded:** first-class MCP open/close and standalone Save/Undo/Redo/history controls;
implicit open-edit-close, save-and-close/discard and batch operations; file
creation/rename/delete; new discovery/search/indexing; scenes/resources/project-settings
authoring; runtime/debugger/LSP tools; force/conflict merging; remote or concurrent-agent
orchestration; broader Godot/platform support; unrelated tool-catalog expansion; package
or release automation and distribution expansion. Handling an applicable loaded
GDScript Resource for closed-edit coherence is not general Resource authoring.
Session status and capability information remain required behavior, not a tool per noun.

The [constitution](../../.specify/memory/constitution.md),
[working agreement](../../AGENTS.md), [architecture](../../ARCHITECTURE.md) and
[Phase 3 roadmap](../../ROADMAP.md#phase-3--mcp-adapter-mvp) govern this work.
Principles I–IV preserve independent authorities, human work, native history and verified
outcomes. V and X preserve local confinement and truthful diagnostics. VI and XII require
real-editor evidence; VII–VIII keep protocol ownership separate and forbid gameplay authority.
The maintainer-approved constitution v1.2.0 expansion of [Principle IX](../../.specify/memory/constitution.md#ix-keep-the-tool-surface-small-and-composable)
requires both a small composable catalog and lean, agent-effective interfaces with structured,
progressive disclosure (FR-022, SC-009).
XI requires released public protocol/library research and independent interoperability.
Architecture-changing and security-sensitive planning/review must explicitly record
constitutional compliance.

**Principle XIII check:** The concrete unmet workflow is an agent discovering, reading
and editing source without managing editor documents or disturbing the developer's
chosen lifecycle state. Existing observation/discovery and open editing provide reusable
safety but cannot mutate a closed target. Continuing with shell glue lacks the selected
MCP contract; requiring manual opening changes the requested intent, and implicit
opening/closing is expressly rejected. The smallest credible scope is three agent
intents, reuse of the existing trusted core, and only the new closed-mutation behavior
needed to preserve lifecycle. Its justified cost is researching and maintaining that
bounded route, its applicable-authority/effect evidence and protocol/client contracts.
That cost addresses the current specified gap, not future extensibility. No generic
transaction manager, workflow engine, retry/replay, cancellation, redaction, tracing
or extensible capability framework, broad API reshaping, new approval gate or CI/runner
topology is required. Any additional mechanism must earn its cost in planning.

For the agent-interface requirement, deliberate static descriptions and existing structured
contracts are the simplest sufficient approach; a small catalog alone does not limit redundant
prose. Ordinary planning and acceptance review adds no separate process gate. No
description-generation framework, metadata DSL, prompt compiler, generic capability registry
or documentation abstraction layer is justified by this requirement.

**Evidence boundary:** Existing open-document observation/edit and historical
opening/closing evidence remain reusable where relevant inputs are unchanged under
[TEST_POLICY.md](../../TEST_POLICY.md#reusing-evidence-across-commits). Historical
Feature 002 passes do not prove closed mutation. The new closed path needs focused
deterministic/core/boundary and applicable real-Godot proof, including loaded-Resource
coherence or confirmed absence, continued buffer absence, stale/lifecycle changes and
partial/unknown effects. The MCP path needs its own contract and real-client/agent
evidence. Changed relevant inputs invalidate affected evidence only; no automatic
historical full GUI/VM replay follows this clarification. Local graphical acceptance
uses the existing VM boundary. Release and Phase 3 exit remain separate evidence decisions.

### Key Entities *(include if feature involves data)*

- **Client connection**: An external MCP client's supported protocol relationship with the kit, distinct from a selected Godot editor lifetime.
- **Tool description**: Discoverable operation inputs, result meanings, effects, limits and prerequisites; not project content or authority to bypass guards.
- **Operation request**: One discover, read or edit intent with explicit target identity, correlation and any required prior read/basis and intended source; no document-lifecycle command.
- **Expected edit basis**: Target, revision, observed lifecycle and applicable-authority provenance from a trusted read, rechecked before effects. Open editing retains its existing basis; a closed-target basis requires scoped design, not fabricated B versions or reuse of Feature 002's open-only eligibility.
- **Operation result**: Attributed read/inventory or edit outcome, lifecycle, stages, effect certainty, applicable evidence, limitations and safe next action. A closed result distinguishes a genuinely absent buffer or Resource from unavailable observation.
- **Cancellation/delivery state**: Whether a client can still receive an attempt's result, kept distinct from whether native effects occurred.
- **Compatibility profile**: Demonstrated protocol/client and exact editor/platform combinations, retaining existing operation profiles and explicitly identifying the new closed-edit profile and its limitations.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: At least one real coding-agent client and one other independent external MCP client connect using documented setup, discover the three-intent surface and complete representative discover/read/edit calls. No first-class open/close/Save/history tool is advertised; unsupported requests cause zero unintended effects. Internal harnesses alone do not count as external clients.
- **SC-002**: A real coding agent starting without an exact script path completes discover → read → edit → fresh read for eligible initially open and initially closed fixtures through MCP, with zero shell/direct-file/private-bridge substitutions or agent-caused lifecycle changes. Open edits retain the same open document; closed edits create no document and leave the target closed, with intended persistence and applicable authorities independently verified. Later editor/fixture lifecycle actions witness durability rather than enable the edit.
- **SC-003**: Every dirty, stale/basis, changed-identity/lifecycle, ambiguous, denied, unsupported and missing-safety-evidence case meets its specified refusal or effect-sensitive non-success and preservation behavior. There are zero unsafe overwrites, discarded human changes, guessed targets, lifecycle workarounds, late effects after proven refusal or unauthorized disclosures.
- **SC-004**: Every delivered success/refusal/interruption result preserves the applicable operation facts and can be interpreted without guessing application, evidence availability, target or safe next action. There are zero false success, definite-not-applied, rollback or safe-replay claims, including cancellation after authorization and loss of result delivery.
- **SC-005**: MCP-driven A–E pass in real Godot through discover/read/edit: A/B retain coherent open editing and dirty protection; C uses genuine ordinary Undo/Save/Redo/Save for open edits with MCP reads and witnesses; D uses editor/fixture close/reopen; E includes twenty fresh-basis successful edits covering supported open and closed profiles with at least three dirty and three stale refusals. Intended source survives applicable Save/reparse/rescan/runtime checks. Closed edits have independent applicable-authority/lifecycle proof, not an invented buffer-history requirement.
- **SC-006**: Every controlled local MCP read/discover completes within five seconds and edit within ten seconds under documented admission/delivery measurement, including blocked/unresponsive cases. Both open and supported closed edits receive timing evidence; existing open-edit timing is not proof of the new closed path. Cancellation remains responsive without extended budgets or rollback claims.
- **SC-007**: Inspection of authorized, denied, ambiguous, interrupted and startup/error paths finds zero incidental source/inventory/credential leakage or protocol traffic contamination. Only permitted requested results expose selected information; capability discovery never claims unsupported tools or guaranteed editor eligibility.
- **SC-008**: Existing Features 001–005 contracts and support boundaries remain valid, with unchanged evidence reused after relevant-input review and affected regressions identified. New closed mutation has positive focused deterministic/core/boundary and applicable real-Godot evidence, not borrowed Feature 002 acceptance. Tooling/export isolation remains established; specification quality or unresolved feasibility cannot be reported as product, Phase 3 exit or release completion.
- **SC-009**: Planning review and later real-agent acceptance establish that public descriptions and structured contracts are understandable without unnecessary prose and sufficient to choose and correctly use discover/read/edit, provide valid inputs, interpret results and take safe next actions without private implementation knowledge. Required current basis/state, lifecycle and material limitations remain explicit; typed/state distinctions and outcome-specific guidance provide progressive disclosure rather than duplicated warnings or exhaustive static failure catalogs. Correctness and safety remain implementation-enforced, not dependent on long instructions. Evaluation is qualitative: no token/word/character budget, description-length threshold or concision score is imposed.

## Assumptions

- Features 001–005 and Phases 1/2 remain complete. Feature 007 adds the MCP surface and a new scoped closed-edit capability; it does not reopen those tasks or alter Feature 002's existing closed-target refusal.
- The developer deliberately configures the client and already-installed tooling and starts a supported local editor. The client can supply explicit project/session information; ambiguity must still refuse. Automated editor startup, installation management and broader session orchestration are excluded.
- Discovery and read retain their existing supported profiles, including dirty/divergent/closed/limited observations. Open editing retains Feature 002's supported clean-document profile. Closed editing requires a separately established supported profile and state-bound basis under the same safety invariants; read availability alone does not imply edit support.
- The engine/platform baseline remains official Godot 4.7.2.stable.official.ed1daf0bf on macOS 26.6.2 arm64. Neither MCP nor closed-edit support is already proved. Planning may identify unsupported closed-state profiles, but refusing every closed target does not satisfy the required positive capability; if no safe supported route exists, return to the product/spec decision.
- Ordinary developer/editor or controlled fixture Undo/Redo/Save, close/reopen and durability actions are acceptance interactions, not MCP tools or an implicit editing implementation. Open edits preserve native history; closed edits require applicable-authority coherence without a new editor-buffer history entry. Standalone Save/history authority remains excluded.
- Two independent external clients, one a real coding agent, establish a minimal interoperability check rather than a broad client-support matrix. Planning selects concrete released versions and reproducible evidence; unavailable clients cannot be replaced by mock-only proof.
- Protocol/SDK/library versions, transport, process/runtime shape, wire/schema details, exact tool names, client versions and test implementation belong in the [technical plan](plan.md) and [research](research.md), not this product specification. Planning must establish the minimum safe Godot-authoritative closed-edit route, applicable loaded-Resource states, basis and independent postconditions. An unsupported route requires returning to the product/spec decision, not a weaker implementation.
- This approved specification and its quality checklist establish requirements, not product acceptance. Planning research/design is recorded separately; primitive feasibility evidence does not claim an implemented guarded closed-edit capability or approve the new technical plan. Task derivation, granularity review, analysis, implementation and acceptance remain separate stages; none starts automatically after planning.
