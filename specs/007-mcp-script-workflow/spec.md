# Feature Specification: Use the Trusted GDScript Workflow Through MCP

**Feature Branch**: `spec/007-mcp-script-workflow`

**Created**: 2026-10-03

**Status**: Draft — requirements-quality review passed; implementation is not authorized.

**Input**: Generated WHAT/WHY description supplied to `/speckit.specify`:

> Expose the completed trusted GDScript workflow to real coding agents through MCP. A developer already has a supported live Godot Editor and the completed observation, discovery, opening, editing and closing capabilities, but a coding agent cannot discover and invoke them through MCP. Provide a small composable external surface for those five operations so the agent can find a script, open it, independently inspect actual editor/disk state, submit a guarded persisted edit, freshly observe, safely close and deliberately reopen it without shell glue or manual tab preparation.

> Preserve explicit authenticated project/session routing, project confinement, independent disk/loaded-Script/visible-buffer evidence, document-attributed dirty state, target-bound revisions and fresh mutation-boundary checks. Observation and discovery remain read-only; dirty or divergent observations remain useful information rather than edit permission. Opening and closing retain their distinct lifecycle and no-change recognition semantics. Edit success retains edit-owned persistence, independent intended D/R/B convergence, clean/saved state, required source validation and real native history. Refuse ambiguous or replaced targets, stale or missing bases, dirty work including equal-text dirty state, unavailable safety evidence, denied access and unsupported effects without repair, force, implicit Save, retargeting or automatic replay.

> Supply discoverable consistent machine-readable tool contracts, accurate supported-capability information, structured outcomes and actionable protocol/request versus operation-execution errors. Map cancellation and connection loss without claiming rollback or concealing known/partial/unknown effects. Preserve existing bounded operation timing, request correlation, source disclosure rules, logging separation and production-export isolation. MCP details stay above the reusable automation and Godot integration; reuse the complete trusted execution paths rather than constructing a second safety policy.

> Prove real external MCP interoperability and real coding-agent use of the composed workflow, including new MCP-path A–E evidence. Ordinary human/editor Undo, Redo and Save remain the existing acceptance interactions for native history and durability, while the agent's product operations and observations go through MCP; this feature adds no standalone history or Save commands. Reuse valid unchanged native/editor evidence under TEST_POLICY.md and newly exercise adapter behavior and affected composed paths instead of replaying historical campaigns by default.

> This is the first selected Phase 3 capability, not a declaration of phase completion or a release. Preserve the exact existing Godot/platform and operation-specific support boundaries. Exclude standalone Save/Undo/Redo, Feature 006 revival, arbitrary commands/evaluation/filesystem access, remote access, new Godot capabilities, broader support, scenes/resources/project settings, runtime/debugger/LSP tools, batch/concurrent-agent orchestration, tool-catalog expansion, broad API reshaping and generic transaction/workflow/retry/cancellation/redaction/tracing/capability frameworks. Basic local connection guidance is required; distribution expansion is not. Leave released MCP version, SDK/library, transport, process/runtime shape, schema details and interoperability selections to evidence-backed planning.

## User Scenarios & Testing *(mandatory)*

The primary user is a coding agent acting for a developer who may continue working
in a live Godot Editor. Features 001–005 already supply the trusted local operations;
this feature makes them discoverable and callable through the Model Context Protocol
(MCP), without requiring the agent to construct shell commands or integrate a private
editor protocol. Developer installation, client configuration and deliberate editor
startup remain setup, not agent tools.

**D** is disk source, **R** is loaded Script source, and **B** is the visible editor
buffer. They remain independent authorities. A protocol response is not evidence of
editor success: every operation retains its own verified-success, recognition,
refusal, partial-effect and unavailable-evidence meanings.

Stories describe separately verifiable outcomes of one adapter capability. Safety,
privacy, bounded execution and truthful results are inseparable from every exposed
operation, not optional later stories.

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

1. **Given** documented local setup and a supported external client, **When** the client connects and requests available tools, **Then** it discovers the five supported operations—observe, discover, open, edit and close—with consistent machine-readable inputs, output meanings, effect descriptions, limits and actionable prerequisites. The catalog contains no project source, inventory, credentials or unsupported Save/history tools.
2. **Given** a client that cannot agree on a supported protocol or required capability, **When** it connects or requests an unsupported protocol feature, **Then** it receives an explicit compatible failure without dispatching a Godot operation or claiming a successful connection with nonexistent capabilities.
3. **Given** the protocol service is reachable but the selected editor is unavailable, incompatible or lacks an operation capability, **When** the agent requests that operation, **Then** the result distinguishes editor availability/support from protocol connection success. Tool presence never guarantees that a particular editor or script is eligible.
4. **Given** a malformed or out-of-bound tool request or an unknown tool, **When** the client submits it, **Then** it receives the appropriate machine-readable request or execution failure without unsafe effects, payload echo or a fabricated operation result. Valid requests remain subject to the ordinary core checks.

---

### User Story 2 - Complete the Trusted Script Workflow Through MCP (Priority: P1)

As a coding agent, I want to find a script, open and inspect it, make one coherent
persisted edit, and close or deliberately reopen it through MCP, so the developer
need not supply command-line glue or manage tabs for my work.

**Why this priority**: This delivers the existing useful editing workflow to a real
agent, rather than only exposing connection infrastructure or a prearranged demo.

**Independent Test**: Ask a real coding agent to change an eligible script in an
explicitly selected project without supplying its exact path. Observe the agent's MCP
calls and independently witness the actual editor, disk, dirty state and identities.
No shell invocation, direct file write or private fixture operation may substitute for
an agent-facing operation in this workflow.

**Acceptance Scenarios**:

1. **Given** a supported local project/session and an eligible closed script among distinguishable paths, **When** the agent discovers scripts, selects the exact path, opens it, obtains an ordinary observation and submits an edit with that basis, **Then** the intended change is independently verified and persisted through the existing operations. There is no manual path lookup, tab preparation or source reconciliation.
2. **Given** a verified edit, **When** the agent obtains a new ordinary observation, requests clean closure and separately requests opening and observation, **Then** the persisted intended source survives with independently verified lifecycle states. Close/open results are not substituted for an ordinary mutation basis, and closure does not fabricate a post-close buffer.
3. **Given** a known target or an already-open document, **When** the agent directly invokes an applicable operation, **Then** discovery is not a prerequisite and no compound workflow runs implicitly. Already-open recognition preserves dirty/divergent state and selection; equal-source edits and already-closed recognition retain their existing checked no-change meanings.
4. **Given** dirty, divergent, closed or partly observable state, or an incomplete discovery scope, **When** the agent observes or discovers through MCP, **Then** source authority, dirty attribution, availability, invalidation and coverage remain distinguishable. A complete read is not editing permission, and a limited empty list is not proof that no scripts exist.

---

### User Story 3 - Keep Human Work and Target Boundaries Intact (Priority: P1)

As a developer, I want the MCP route to enforce the same safety as the completed
operations, so a new agent interface cannot overwrite my work, close a newer buffer
or disclose another project's contents.

**Why this priority**: An adapter with weaker input, routing, disclosure or mutation
semantics would invalidate the product's existing value.

**Independent Test**: Submit MCP requests against independently witnessed dirty,
stale, ambiguous, replaced, denied and unsupported targets. Include changes between
observation and effect entry, and inspect both permitted results and preserved state.

**Acceptance Scenarios**:

1. **Given** a dirty target, including equal-text-but-dirty state, **When** edit or effectful close is requested, **Then** the operation refuses while preserving text, dirty state and native history. The adapter does not Save, discard, merge or force the document to make it eligible.
2. **Given** a missing, unusable, wrong-target or stale expected basis, including same-text version changes or a closed/reopened buffer, **When** edit or effectful close is requested, **Then** existing revision and identity checks prevent stale action. A freshly observed already-closed target retains its separate recognition rules; a missing basis cannot close an open target.
3. **Given** ambiguous same-project sessions, an ended explicit session, denied access or an escaping path, **When** a tool is called, **Then** no focused, convenient or replacement target is selected and no candidate/unauthorized source or inventory is exposed. Source-free selection feedback remains usable.
4. **Given** unavailable required safety evidence, conflicting loaded source or an unsupported source/effect/context profile, **When** an effectful operation is requested, **Then** it refuses according to that operation's existing rules rather than force-loading, normalizing, repairing or weakening a check. Safe observation, opening and closing are not falsely made parse-validity tests.
5. **Given** newer human work appears after an effect begins, **When** verification detects invalidation, **Then** newer work survives and the agent receives the actual non-success and effect knowledge. There is no source reassertion, compensating reopen, reclose or rollback claim.
6. **Given** overlapping requests reach the same editor, **When** existing admission refuses one, **Then** that refused request is not queued or replayed later. Request correlation stays distinct, and cancellation of one call does not become authority to cancel or retarget another.

---

### User Story 4 - Understand Failures and Interrupted Effects (Priority: P1)

As a coding agent, I want protocol failures and editor-operation outcomes to remain
interpretable, so I can stop or re-observe instead of blindly repeating a change.

**Why this priority**: A successful protocol exchange can carry an unsafe or unverified
operation result, and a lost exchange cannot establish that no effect occurred.

**Independent Test**: Exercise invalid requests, ordinary refusals, host/delivery
failure, cancellation and connection loss before and after effect authorization.
Compare every deliverable result with independent survivor-state witnesses and verify
that the client can make a separate fresh observation where the connection permits.

**Acceptance Scenarios**:

1. **Given** a valid tool call whose operation refuses, partially applies or cannot verify its result, **When** a response is delivered, **Then** the client receives the operation's target, reason, stage, effect certainty, permitted evidence and safe next action in structured form. Transport success or a generic error flag does not replace those facts.
2. **Given** cancellation or deadline expiry before or after possible effects, **When** the attempt terminates, **Then** existing proven-not-applied, known-applied, partly-applied and unknown distinctions are preserved. A proven refusal cannot apply later; a missing acknowledgment is not no-effect evidence.
3. **Given** the client disconnects, cancels delivery or stops reading a result, **When** the adapter cannot deliver the terminal evidence, **Then** it does not claim successful delivery, rollback or safe replay. The documented recovery is a fresh observation of the original explicit target before another intentional action; no persistent outcome service or automatic retry is implied.
4. **Given** an unresponsive editor or blocked operation under supported local conditions, **When** a request is made with output consumed, **Then** the existing operation deadline produces a truthful terminal result. Protocol handling remains able to receive cancellation; subordinate work or adapter orchestration cannot renew the operation budget.

---

### User Story 5 - Retain Native History and Durable Results in Real Agent Use (Priority: P1)

As a developer, I want an MCP-originated edit to retain the same native history,
persistence and ordinary-editor behavior as the proven local workflow.

**Why this priority**: Internal-harness success or an advertised undoable flag does not
prove that a real agent's new protocol path preserves those guarantees.

**Independent Test**: Drive the product calls of A–E through a real coding-agent MCP
client, with independent live-editor witnesses. Human/editor interactions create dirty
work and invoke ordinary Undo, Redo and Save; they do not replace an agent-facing call
or repair divergence caused by a reported successful edit.

**Acceptance Scenarios**:

1. **Given** clean and dirty-human-buffer fixtures, **When** a coding agent edits through MCP, **Then** A establishes intended independent D/R/B and clean/saved state, while B preserves the human work and returns the existing safe refusal, including dirty-equal-text cases.
2. **Given** a verified MCP-originated edit and pre-existing native history, **When** the developer performs ordinary Undo, Save, Redo and Save, with MCP observations and independent witnesses at the relevant transitions, **Then** C verifies genuine native reversal/reapplication and retained prior history. Unsaved history states remain truthful observations, not falsely persisted convergence or permission for another edit.
3. **Given** a successful MCP edit, **When** the agent uses separate close/open calls and observation, and applicable ordinary Save/reparse/rescan/permitted-runtime durability checks occur, **Then** D and durability retain the intended source without manual reconciliation. Target-buffer history persistence after disposal is not promised.
4. **Given** twenty sequential successful MCP edits with fresh observations/bases, ordinary Save between edits and at least three dirty and three stale refusals interleaved, **When** the agent completes the sequence and closes/reopens the final target, **Then** E loses no intended revision or human work, conceals no conflict and retains the final intended source and applicable native-history guarantees.

### Edge Cases

- Protocol connection success, tool advertisement and editor compatibility are separate facts; none proves operation eligibility.
- A client serializes source, paths, identities, versions or an observation basis differently: reject unsupported or lossy input rather than normalize it into another target or usable authorization.
- An empty observed source, unavailable source, invalidated evidence and absent buffer must not collapse into the same protocol value or display summary.
- A large source, basis, inventory or outcome reaches existing limits: preserve bounded refusal/limitation semantics without silent truncation, dropped safety fields or false completeness.
- A discovery path is no longer present, or a session/buffer is replaced between calls: every subsequent operation resolves current state under its own contract.
- An earlier causal failure is followed by disclosure denial: prohibited source summaries and bodies remain suppressed without erasing known effects.
- A client presents the same request again after losing its reply: correlation is not a replay guarantee, and the adapter supplies no automatic replay path.
- Cancellation arrives after authorization or while result delivery fails: loss of a response must not be translated into definite non-application.
- Help, startup failures, worker output and diagnostic logging must not contaminate protocol traffic or expose project content.
- Human Undo may leave dirty state legitimately; neither automatic Save nor a second source edit may substitute for genuine history traversal.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: A real external MCP client MUST be able to connect using documented local setup and invoke the completed script workflow. Compatible released protocol/library choices and concrete client compatibility MUST be established in planning and acceptance, not inferred from an internal harness or a directory name.
- **FR-002**: The external tool surface MUST expose the five distinct completed operations—observe, discover scripts, open, edit and close—with consistent names, machine-readable inputs/results and discoverable descriptions of effects, prerequisites, limits and outcomes. Required information MUST be available without an inflated catalog or unrelated tools. There is no implicit compound operation.
- **FR-003**: Inputs and outputs MUST receive syntactic and semantic validation appropriate to their boundary. Unsupported tools, malformed values, wrong types, out-of-bound data and incompatible negotiation MUST fail explicitly. Exact source, paths, identity/version values and basis meaning MUST not be silently truncated, normalized or weakened; invalid output after possible effects MUST not become false success or false not-applied state.
- **FR-004**: Every operation MUST retain explicit authenticated project/session selection, confinement and lifetime binding. Ambiguity, denied access, ended/replaced sessions and changed target identity MUST not select by focus, working directory, earlier basis or convenient fallback, disclose unauthorized data or launch an editor.
- **FR-005**: Observation MUST preserve Feature 001's independent D/R/B, dirty, identity, interval, comparison and availability/invalidation semantics. Dirty/divergent observations remain valid reads, not mutation conflicts. Observation MUST remain read-only, including supported closed and partly observable states, without forced opening/loading or inferred cleanliness.
- **FR-006**: Discovery MUST preserve Feature 004's exact scoped, source-free, fresh inventory and complete/limited/refused/interrupted distinctions. A listing is not a source read, edit basis, current document identity or permission to act; subsequent operations MUST make their own current checks.
- **FR-007**: Opening MUST preserve Feature 003's guarded native lifecycle, independent newly-opened verification and already-open no-change recognition. No source write, Save, conflict repair or history action is introduced. Opening evidence MUST not substitute for a fresh ordinary observation before an edit.
- **FR-008**: Editing MUST preserve Feature 002's checked single-script intent, target-bound expected basis, fresh boundary checks, native source/history semantics, edit-owned persistence and independently verified intended D/R/B, clean/saved and required validation evidence. Dirty, stale, divergent, unsupported or unobservable safety state MUST refuse; no unconditional or alternate writer is exposed.
- **FR-009**: Closing MUST preserve Feature 005's basis-bound clean closure, fresh safety/context checks, unrelated-work protection, independent absence/unchanged-source verification and separate already-closed recognition. It MUST not Save, discard, close a newer buffer or reopen as compensation. Closing summaries MUST not become future mutation bases.
- **FR-010**: MCP MUST remain an adapter above the existing protocol-independent automation and Godot integration. Routing, confinement, revision/conflict checks, admission, independent acquisition/verification, effect certainty, deadline/cancellation interpretation and disclosure reduction MUST remain owned by the existing trusted execution paths, not reconstructed as MCP-specific policy or accepted as client-provided safety assertions.
- **FR-011**: Deliverable results MUST preserve the underlying operation's requested/resolved target, correlation, stages, verified versus recognized versus refused outcome, known/partial/unknown effects, applicable evidence, unavailable/invalidated facts, diagnostics and safe next action. Protocol success, acknowledgment or serialization alone MUST not become operation success.
- **FR-012**: Protocol/request errors, tool-execution outcomes and host/delivery failures MUST remain distinguishable. A private editor-protocol failure after possible effects MUST retain effect knowledge rather than collapse into a generic malformed-client-request response. Known stale Resource/buffer, dirty conflict, revision mismatch, parse/validation failure, disconnection, timeout and unsupported/denied capability remain distinct where applicable.
- **FR-013**: Protocol cancellation and connection/process lifecycle MUST preserve existing effect-sensitive termination, newer-human-work protection and owned-work cleanup. Cancellation is scoped to the intended request, not another request or the user's editor. When the chosen protocol permits result delivery, retained terminal facts MUST remain available; when delivery is impossible or suppressed by protocol cancellation semantics, no successful-delivery, rollback or definite-no-effect claim is permitted.
- **FR-014**: Existing controlled local operation bounds MUST remain: five seconds for observation/discovery and ten seconds for open/edit/close, including operation-input validation, selection, execution and bounded delivery with the client consuming output. Planning MUST define reproducible MCP admission/delivery measurement and bounded connection behavior without excluding operation work or renewing its cutoff. No concurrent-throughput or arbitrary-project-size promise is added.
- **FR-015**: The adapter MUST not automatically retry, queue a refused mutation for later application, reconnect to a replacement editor, compensate or replay an uncertain action. After possible effects, safe guidance requires fresh ordinary observation of the original explicit target before a new intentional action with an eligible basis where required. Request correlation is not an idempotency or recovery token.
- **FR-016**: Only supported protocol/tool capabilities may be advertised. Server tool support MUST be distinguishable from current editor/session/script availability and eligibility; an unavailable editor or unsupported operation MUST yield truthful non-success. Existing selected-target and diagnostic information MUST support agent decisions without exposing private endpoints, credentials or candidate project contents.
- **FR-017**: Protocol traffic MUST remain separate from incidental logs and diagnostics. Requested authorized results alone may disclose their permitted selected source/inventory/evidence. Raw requests, source, unauthorized paths, credentials, private validation content and source-derived records suppressed by the core MUST not leak through protocol errors, logging, startup output or tool discovery. Existing operation-specific disclosure precedence remains unchanged.
- **FR-018**: The existing public local-operation contracts, source/effect profiles, bounds, refusal meanings and exact support boundary MUST remain unchanged. New MCP schemas and structured outcomes are deliberate compatibility surfaces; breaking changes require documented versioning/migration. No wider Godot/platform/script support, new permission or stronger isolation/atomicity claim follows adapter exposure.
- **FR-019**: Real external-client interoperability and real coding-agent completion of the product workflow MUST be demonstrated. New MCP-path A–E evidence MUST retain coherent editing, dirty refusal, actual native history, separate product close/reopen and sequential-edit stress, with applicable durability and independent live-editor witnesses. Refusing every mutation, internal-harness-only execution, disk-only checks or copied request values cannot satisfy this requirement.
- **FR-020**: Valid unchanged native/editor and existing-operation evidence MUST be reused according to TEST_POLICY.md after relevant-input review. New adapter contracts, mapping, cancellation, advertisement, traffic/privacy behavior and composed MCP execution need new evidence; invalidated boundaries need affected regressions. A new adapter or feature head alone MUST not require replay of all historical GUI/VM campaigns.
- **FR-021**: Local-only operation, no telemetry, preservation of human work, existing permission/confinement boundaries and tested tooling/export isolation MUST remain intact. Basic client connection/setup, exact compatibility and verified limitations MUST be documented before support is claimed. This feature adds no arbitrary evaluation/process/filesystem capability, remote access, gameplay authority or distribution expansion.

### Scope and Governance Alignment

This is the first selected capability of **Phase 3 — MCP adapter MVP**. The current
[source manifest](../../mcp-server/Cargo.toml) exposes local callers, while the
[Phase 2 exit](../../PHASE_2_EXIT.md#thin-adapter-conclusion) establishes that trusted
reusable execution already exists. The gap is external agent access, not missing
transaction, history or persistence behavior.

The completed semantic baselines remain:

| Operation | Preserved requirement source | User-facing distinction |
| --- | --- | --- |
| Observe | [Feature 001](../001-observe-gdscript-state/spec.md#functional-requirements) | Independent read; dirty/divergent completeness is not edit permission. |
| Discover | [Feature 004](../004-discover-project-gdscript/spec.md#functional-requirements) | Scoped locations and honest coverage, not source or mutation authority. |
| Open | [Feature 003](../003-open-project-gdscript/spec.md#functional-requirements) | Verified new lifecycle versus existing-open preservation. |
| Edit | [Feature 002](../002-edit-open-gdscript/spec.md#functional-requirements) | Basis-bound native persisted edit with independent verification. |
| Close | [Feature 005](../005-close-project-gdscript/spec.md#functional-requirements) | Safe clean closure versus already-closed recognition. |

Their phase-local exclusions of MCP describe those earlier features, not a prohibition
on this later external adapter. No earlier product behavior or acceptance obligation
is weakened or reopened.

**History and Phase 3 acceptance interpretation:** For this feature, the agent-facing
product calls in A–E run over MCP, including fresh state inspection. Human/editor
interactions still create unsaved work and perform ordinary Undo/Redo/Save, as defined
by [Feature 002 US4](../002-edit-open-gdscript/spec.md#user-story-4---reverse-and-reapply-through-native-history-priority-p1)
and the [Phase 1 interpretation](../../PHASE_1_EXIT.md#savehistory-interpretation).
These are independently witnessed acceptance actions, not standalone MCP capabilities
or manual repair of an agent-created divergence. This explicit feature boundary does
not declare Phase 3 complete; its roadmap exit is assessed against delivered evidence.
The [Feature 006 decision](../006-save-project-gdscript/decision.md#decision) remains
assessed, not proceeding, and supplies no prerequisite or implementation backlog.

**Excluded:** standalone Save/Undo/Redo/history controls; save-and-close/discard; compound
or batch workflows; file creation/rename/delete; new discovery/search/indexing;
scenes/resources/project settings; runtime/debugger/LSP tools; force/conflict merging;
remote or concurrent-agent orchestration; broader platform/Godot support; tool-catalog
expansion; package/release automation and distribution expansion. Reparse/rescan/runtime
fixture actions prove durability, not new agent tools. Session status and capability
information are required behaviors, not authorization for a separate tool per noun.

The [constitution](../../.specify/memory/constitution.md),
[working agreement](../../AGENTS.md), [architecture](../../ARCHITECTURE.md) and
[Phase 3 roadmap](../../ROADMAP.md#phase-3--mcp-adapter-mvp) govern this work.
Principles I–IV preserve independent authorities, human work, native history and verified
outcomes. V and X preserve local confinement and truthful diagnostics. VI and XII require
real-editor evidence; VII–IX keep protocol ownership separate and the tool surface small,
with no gameplay authority. XI requires released public protocol/library research and
independent interoperability. Architecture-changing and security-sensitive planning/review
must explicitly record constitutional compliance.

**Principle XIII check:** The concrete unmet workflow is a real coding agent's inability
to invoke the completed operations through MCP. Continuing with shell commands is the
simplest alternative, but requires agent-specific command/payload orchestration and does
not provide the selected discoverable MCP contract. Existing core mechanisms already
satisfy editor safety, so a second safety framework would add cost without closing this
gap. The justified new responsibility is bounded protocol/host integration and its
schema, compatibility and real-client evidence maintenance. That cost supplies useful
external access now. No generic transaction manager, workflow engine, retry/replay cache,
cancellation/redaction/tracing framework, extensible capability registry, broad API
reshaping, new approval gate or CI/runner topology is required. Any additional mechanism
must earn its cost against a concrete current requirement during planning.

**Evidence boundary:** The new composed MCP workflow must be exercised because no
historical local-caller campaign observed that protocol path. Unchanged native history,
confinement, privacy/export and existing operation evidence is reusable under
[TEST_POLICY.md](../../TEST_POLICY.md#reusing-evidence-across-commits); uncertain or changed
relevant inputs require affected checks, not unconditional full replay. Local graphical
acceptance uses the policy's existing VM boundary. A future release candidate is a
separate decision, not a consequence of creating or completing this specification.

### Key Entities *(include if feature involves data)*

- **Client connection**: An external MCP client's supported protocol relationship with the kit, distinct from a selected Godot editor lifetime.
- **Tool description**: Discoverable operation inputs, result meanings, effects, limits and prerequisites; not project content or authority to bypass guards.
- **Operation request**: One explicit action with target identity, correlation and any required prior ordinary observation/intended source.
- **Expected basis**: Existing target/revision provenance used by edit or effectful close, never standalone permission or a replay token.
- **Operation result**: The core's attributed outcome, stages, effect certainty, evidence, limitations and safe next action as exposed through the external protocol.
- **Cancellation/delivery state**: Whether a client can still receive an attempt's result, kept distinct from whether native effects occurred.
- **Compatibility profile**: The released protocol/client combinations and unchanged operation-specific editor/platform support actually demonstrated.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: At least one real coding-agent client and one other independent external MCP client connect using the documented setup, discover the supported five-operation surface and complete representative read-only and effectful calls. Unsupported negotiation/tool requests cause zero unintended editor effects; internal harnesses alone do not count as external clients.
- **SC-002**: A real coding agent starting without an exact script path completes discover → open → observe → edit → fresh observe → close → separate open/observe on the supported fixture through MCP, with zero shell/direct-file/private-bridge substitutions, manual tab preparation or reconciliation. Independent witnesses confirm the intended persisted source and lifecycle states.
- **SC-003**: Every dirty, stale/basis, changed-identity, ambiguous, denied, unsupported and missing-safety-evidence acceptance case retains its specified result and preservation behavior. There are zero unsafe overwrites, discarded human changes, guessed targets, late effects after proven refusal or unauthorized data disclosures.
- **SC-004**: Every delivered success/refusal/interruption result preserves the applicable operation facts and can be interpreted without guessing application, evidence availability, target or safe next action. There are zero false success, definite-not-applied, rollback or safe-replay claims, including cancellation after authorization and loss of result delivery.
- **SC-005**: The MCP-driven A–E cases pass in real Godot, including genuine Undo/Save/Redo/Save while the buffer exists and twenty fresh-basis successful edits with at least three dirty and three stale refusals. The final intended source survives product close/reopen and applicable Save/reparse/rescan/runtime durability; required unavailable observations are not counted as passes.
- **SC-006**: Every controlled local observation/discovery call completes within five seconds and open/edit/close within ten seconds under the documented admission/delivery measurement, including blocked/unresponsive cases. The protocol remains responsive to request cancellation without extending operation budgets or claiming rollback.
- **SC-007**: Inspection of authorized, denied, ambiguous, interrupted and startup/error paths finds zero incidental source/inventory/credential leakage or protocol traffic contamination. Only permitted requested results expose selected information; capability discovery never claims unsupported tools or guaranteed editor eligibility.
- **SC-008**: Existing operation contracts and support limits remain unchanged, with valid evidence coverage for every affected requirement and existing tooling/export isolation. Reuse and new execution are identified separately; no product, Phase 3 exit or release claim rests on specification quality, protocol acknowledgment or unverified historical equivalence.

## Assumptions

- Features 001–005 and Phases 1/2 are complete according to current task, acceptance and exit records, not older draft-generation headers. This specification adds an external adapter and does not reopen those implementation tasks.
- The developer deliberately configures the client and already-installed tooling and starts a supported local editor. The client can supply explicit project/session information; ambiguity must still refuse. Automated editor startup, installation management and broader session orchestration are excluded.
- Each operation keeps its own existing target/source/context profile. The composed mutation workflow uses eligible standalone project GDScript; observation is not artificially restricted to only clean mutation-eligible documents.
- The support baseline remains official Godot 4.7.2.stable.official.ed1daf0bf on macOS 26.6.2 arm64 and the recorded operation limits. New MCP support needs its own evidence on that baseline; this is not a claim that the adapter is already implemented or verified.
- Ordinary developer history/Save and synthetic acceptance preparation are permitted test interactions, not MCP tools. No standalone Save/history authority is needed to perform a successful persisted edit, and none is selected here.
- Two independent external clients, one a real coding agent, establish a minimal interoperability check rather than a broad client-support matrix. Planning selects concrete released versions and reproducible evidence; unavailable clients cannot be replaced by mock-only proof.
- SDK/library, released protocol version, transport, process/runtime shape, wire/schema details, tool names, client-version choices and test implementation remain for evidence-backed planning. Their absence from this WHAT/WHY specification is not a missing user requirement.
- This draft and its requirements checklist establish one selected feature's requirements only. Approval, clarification if needed, planning, task derivation, granularity review, analysis, implementation and acceptance remain separate stages. No implementation or next stage starts automatically.
