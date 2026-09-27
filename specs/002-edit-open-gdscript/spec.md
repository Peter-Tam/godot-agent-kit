# Feature Specification: Safely Edit Open GDScript

**Feature Branch**: `spec/002-edit-open-gdscript`

**Created**: 2026-09-27

**Status**: Draft — quality-validated and ready for planning; implementation is not authorized.

**Input**: Generated feature description supplied to `/speckit.specify`:

> Safely edit one existing, already-open standalone GDScript in an explicitly selected local Godot project and editor session. A coding-agent caller needs to turn an observed script into an intended source change without losing a developer's unsaved work or leaving disk, the loaded Script, and the visible buffer inconsistent. Build on the completed observation foundation; an earlier observation is evidence, not permission to mutate.

> Require target-bound stale-write protection and fresh, document-specific safety checks at the mutation boundary. Refuse dirty or conflicting human state, changed revisions or identities, ambiguous or replaced sessions, closed or unsupported documents, denied/outside-project access, and missing safety observations rather than guessing, opening a document, force-writing, or reconciling conflicts automatically. After application through native editor semantics, report success only when the intended change is independently observed on disk, in the loaded Script, and in the live buffer, with persistence and dirty/synchronization state established. Distinguish acceptance, application, synchronization, and verification; interrupted or partly applied attempts must expose known changes, uncertainty, diagnostics, and safe next steps without implying rollback or retry safety.

> The developer must be able to reverse and reapply the edit through ordinary Godot Undo/Redo without losing earlier valid history. Prove A–E in real Godot, including dirty-buffer refusal, native history transitions, close/reopen persistence, sequential edits, and applicable Save/reparse/rescan/runtime durability. Native Undo/Redo and subsequent Save are editor interactions for this scoped guarantee, not new general automation commands. Preserve existing read-only observation behavior, local authenticated routing, project confinement, privacy, export isolation, and protocol-independent ownership. Real-editor evidence must establish mutation guarantees rather than borrowing observation-only support claims.

> This advances Phase 1's pending coherent-mutation and durability requirements with one usable open-document edit capability. Exclude script discovery and open/close commands, independent Save or history-control commands, batch/multi-file or concurrent-agent workflows, conflict merging and force writes, new files or embedded scripts, MCP exposure, generalized transaction infrastructure, scene/resource/project-setting authoring, runtime/debugger/LSP tooling, distribution expansion, and new operational or approval prerequisites without a concrete present need. Leave edit representation, APIs, transport evolution, internal design, and other implementation choices to planning.

## User Scenarios & Testing *(mandatory)*

The primary caller is a coding-agent automation layer acting for a developer who may continue working in the live editor. The capability changes **one existing, already-open, standalone project GDScript**. It neither opens a closed script nor creates a new one.

**D** is source on disk, **R** is source held by the loaded Godot Script, and **B** is source in the live editor buffer. They are independent authorities. **Verified edit success** means the intended resulting source has been independently observed as `D == R == B`, the target document is observably clean and synchronized, and its post-change parse result has no parse error. This is not a claim of project-wide or gameplay correctness. A previous complete observation is not authorization to edit.

Stories describe independently verifiable user outcomes, not permission to ship an unsafe subset. Every exposed edit path must enforce the applicable refusal, history, verification, and durability requirements from its introduction.

### User Story 1 - Make One Coherent, Persisted Edit (Priority: P1)

As a coding-agent caller, I want to change a script I have observed while it stays open, so the developer sees the intended result and does not have to repair disagreement between the editor and disk.

**Why this priority**: This is the smallest useful step beyond observation: one real change with an independently verified result, not a disk write or editor acknowledgment.

**Independent Test**: In a real Godot Editor, establish a clean open script and its current revision basis independently. Request a distinctive source change and compare the result with separate disk, loaded-script, buffer, dirty-state, and parse observations.

**Acceptance Scenarios**:

1. **Given** an explicitly selected live session and clean open script with agreeing, independently observable D/R/B, an attributable clean state, and a matching expected revision basis, **When** the caller requests a valid source change, **Then** the change is applied as a native editor edit and persisted; success is returned only after the intended result is independently observed on D, R, and B with clean, synchronized state and no post-change parse error.
2. **Given** a safe target plus a different document containing unsaved human work, **When** the target is edited, **Then** only the selected script changes; the unrelated document is not saved, altered, closed, or stripped of editing history. Unrelated unsaved work is not misattributed to the target.
3. **Given** the selected script is open but not the editor's currently selected tab, **When** an edit is requested, **Then** the same target-bound safety and verification rules apply; editor focus never chooses the destination. If the necessary state cannot be observed or edited safely, the request is refused without source or history changes.
4. **Given** a safe, current target whose requested resulting source is already present, **When** the request is made, **Then** the result explicitly reports independently verified unchanged state, not a newly applied edit, and introduces no source write, save, or editing-history entry.

---

### User Story 2 - Preserve Human Work and Reject Stale Intent (Priority: P1)

As a developer, I want an agent's edit to stop when my work or the intended target has changed, so that an earlier read cannot authorize overwriting what I am doing now.

**Why this priority**: Dirty-state protection, stale-write protection, and exact routing are inseparable from offering unattended mutation.

**Independent Test**: Prepare real dirty, divergent, changed-revision, and changed-session cases with distinctive human text and known history. Attempt an edit and independently compare all affected documents and histories before and after each refusal.

**Acceptance Scenarios**:

1. **Given** the target has an unsaved human edit, including an editor-reported dirty buffer whose text happens to equal disk, **When** an edit is requested, **Then** it is refused as a dirty-buffer conflict; unsaved text, dirty state, disk state, and existing history remain intact. No automatic Save, merge, or force overwrite occurs.
2. **Given** the source revision, document identity, or session has changed since the caller's observation, **When** a request uses that earlier basis, **Then** it is refused with the appropriate revision/identity/session reason; the current state is not overwritten even if the editor now reports clean.
3. **Given** D, R, or B disagree before application, or document-specific dirty state, a required source, or target identity cannot be safely established, **When** the caller requests an edit, **Then** it is refused with explicit conflict or missing-observation evidence; equality on the remaining surfaces does not establish safety. A known stale Resource or buffer is distinguished from unexplained divergence.
4. **Given** a human edit or target change occurs after initial checks but before application, **When** the operation reaches its mutation boundary, **Then** stale preflight evidence cannot authorize the change. The attempt refuses without overwriting the new work; inability to protect this boundary makes the operation unsupported, not optimistically safe.
5. **Given** selection is ambiguous among projects or editor sessions, including same-project sessions, or identifies an ended session, **When** an edit is requested, **Then** no target is guessed and no replacement session is substituted; no candidate source is exposed and no candidate document is mutated.
6. **Given** a closed, missing, non-GDScript, embedded, outside-project, unauthorized, or otherwise unsupported target, or unknown open state, **When** editing is requested, **Then** an actionable, distinguishable refusal occurs before mutation. The request does not open, create, force-load, or repair a document to manufacture eligibility.
7. **Given** the caller omits its expected revision basis or supplies an unusable one, **When** it requests an edit, **Then** the request is rejected before mutation; there is no unconditional-write fallback.

---

### User Story 3 - Understand Interrupted or Unverified Changes (Priority: P1)

As a coding-agent caller, I want to know whether an unsuccessful attempt changed anything and what evidence is missing, so I do not mistake an acknowledgment for success or blindly repeat an edit.

**Why this priority**: Once an edit can change live state, timeout, loss of access, and partial application must not conceal that state or imply rollback.

**Independent Test**: Exercise controlled failures before application, after application, during persistence/synchronization, and during verification. Independently observe the surviving state and compare it with the reported stage, application certainty, limitations, and next steps.

**Acceptance Scenarios**:

1. **Given** the editor acknowledges or applies a request but independent postconditions have not been established, **When** a result is reported, **Then** it does not claim verified success. If persistence or convergence fails, it identifies the affected D/R/B surfaces and the known applied state.
2. **Given** timeout, cancellation, or disconnection occurs before application is confirmed, **When** the attempt terminates, **Then** the result distinguishes proven not-applied from application-unknown. A confirmed pre-application refusal/cancellation must not later apply. Loss of acknowledgment alone is not proof that no change occurred.
3. **Given** a source change has occurred before timeout, cancellation, disconnection, or loss of a required observation, **When** the attempt terminates, **Then** it reports known application or partial application, the last independently observed state and its validity, and what is unknown. It does not claim rollback, invent missing source, automatically retry, or overwrite newer human work to force convergence.
4. **Given** the resulting script has an observed parse error, **When** the attempt returns, **Then** parse error is distinct from invalid target, revision conflict, and synchronization failure; the result identifies relevant diagnostics and whether source changes occurred. Equal D/R/B alone does not turn the attempt into verified success. Unavailable post-change parse evidence likewise prevents success rather than being treated as a clean parse.
5. **Given** the human changes the document after application but before verification, **When** verification runs, **Then** that newer work remains intact, the attempted change is not reasserted over it, and invalidated or divergent postconditions produce an explicit non-success result rather than a success based on earlier samples.
6. **Given** a previously interrupted attempt may have applied, **When** a caller considers another edit, **Then** the result directs it to freshly observe the same explicit target and establish a new revision basis first; the old request is not represented as safe to replay merely because it timed out.

---

### User Story 4 - Reverse and Reapply Through Native History (Priority: P1)

As a developer, I want an agent's edit to behave like a coherent native script edit that I can Undo and Redo, rather than losing my editing history or being left with a hidden disk-only change.

**Why this priority**: Native editor semantics are part of trustworthy live editing. An undoable flag or an unrelated second edit is not evidence of reversibility.

**Independent Test**: Start with a clean open script containing known, previously saved native edit history. Apply the agent edit, invoke actual editor Undo and Redo, independently observe D/R/B and dirty state at every step, and Save each restored/reapplied state to verify persistence.

**Acceptance Scenarios**:

1. **Given** a successfully applied agent edit, **When** the developer invokes one ordinary editor Undo, **Then** the complete pre-edit source is restored in the buffer through native history. The actual loaded-source, disk, and dirty states are independently observed rather than assumed to change with B; after ordinary Save and synchronization, D/R/B agree on the pre-edit source.
2. **Given** that undone edit, including an intervening Save, **When** the developer invokes ordinary editor Redo, **Then** the entire intended agent change returns through the same history; after Save and synchronization, D/R/B agree on the reapplied source. The edit does not disappear or require a new agent mutation to recreate it.
3. **Given** valid native undo history before the agent edit, **When** the new edit is applied, undone, and earlier history is traversed, **Then** earlier undoable changes remain reachable in their original order. Normal editor disposal of an existing redo branch when making a new edit is permitted; clearing unrelated undo history is not.
4. **Given** an edit attempt is refused before application or verified unchanged, **When** native history is subsequently exercised, **Then** it contains no new entry for that attempt and behaves as it did before the request.

Native Undo/Redo and Save here are ordinary developer/editor interactions used to prove the edit's behavior, not new general-purpose automation commands. Undo may legitimately create unsaved state: before Save, report the actual surfaces and dirty state, not a false persisted-coherence claim. A subsequent agent edit must still refuse unresolved dirty state.

---

### User Story 5 - Keep Successful Changes Through Normal Editor Use (Priority: P1)

As a developer, I want successful edits to survive normal editor actions and repeated agent work, so I never have to reconcile changes that disappear or revert later.

**Why this priority**: A momentarily matching snapshot is not a durable edit, and repeated use must not bypass conflict safety.

**Independent Test**: In a real editor, verify a successful edit through Save, close/reopen, reparse, filesystem rescan, and a permitted fixture runtime launch. Repeat changes with fresh revision bases and intervening dirty/stale refusals, checking each applicable authority independently.

**Acceptance Scenarios**:

1. **Given** a verified edit with no later intentional source change, **When** the developer Saves and closes/reopens the script, **Then** its intended source persists and D/R/B agree after reopening; no stale version reappears and no reconciliation prompt caused by the agent's divergence is needed.
2. **Given** that edited script, **When** ordinary reparse and rescan complete and an authorized acceptance fixture is launched, **Then** the intended revision remains intact, applicable D/R/B observations still agree, and the fixture's runtime behavior reflects the edited revision. An unavailable runtime is explicit and does not count as successful runtime proof.
3. **Given** at least 20 sequential successful changes with fresh observations, **When** Save is exercised between changes and at least three dirty-buffer or stale-revision attempts are interleaved, **Then** every successful change is preserved until intentionally superseded, every unsafe attempt refuses without losing human work, native history remains usable, and the final intended source survives close/reopen.
4. **Given** the editing capability is installed, **When** the existing observation workflows run against clean, dirty, divergent, closed, and partly observable scripts, **Then** their existing meanings, source limits, bounded results, and non-interference guarantees remain unchanged. Observing a script never becomes an implicit edit, Save, synchronization, or edit authorization.
5. **Given** the editing capability and synthetic project-content sentinels, **When** authorized, ambiguous, denied, and interrupted requests are exercised and production exports are checked, **Then** only the explicitly selected authorized result may contain its requested source; incidental logs and private routing metadata contain no source, other targets remain untouched, and exported gameplay contains no active editing tooling or dependency on it.

---

### Edge Cases

- The intended change is empty source or differs only in whitespace/line endings: preserve the requested source or refuse an unsupported representation explicitly; never normalize away a real D/R/B disagreement or silently truncate source.
- A source or requested result exceeds declared supported limits: refuse the edit before mutation, retain permissible independently obtained evidence, and leave the existing observation feature's per-source limit behavior unchanged.
- The script is renamed, deleted, replaced, closed, or reopened during an attempt: do not transfer an earlier revision basis to the new identity or combine observations from different documents.
- The disk is not writable or saving fails after B has changed: report the actual application state and persistence failure; do not claim success or erase newer work in an assumed rollback.
- An initially invalid script is being repaired: a syntax error in GDScript does not by itself make it a non-script target. Repair remains subject to the same safety checks and requires a valid observed post-change parse result for success.
- A plugin/editor restart follows an interrupted request: the replacement session is a new target, not authority to replay an old mutation.
- Another actor writes the script or changes its history during synchronization: preserve newer work and expose invalidated evidence or partial application rather than silently enforcing the agent's older intent.
- A valid empty observation is not a missing surface; unknown dirty state is not clean, and a read of stale source is not proof that it is current.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The caller MUST be able to request one source change to an existing, already-open, standalone GDScript in one explicitly selected local project/editor session. The request MUST identify the desired resulting change and the target-bound revision basis on which it relies. The edit representation is a planning decision, not a requirement for multiple alternative edit operations.
- **FR-002**: Target resolution MUST retain the existing authenticated, project-confined, lifetime-bound routing guarantees. Ambiguity, denied access, ended/replaced sessions, and changed document identity MUST NOT select a convenient target or expose source from candidates or unselected projects.
- **FR-003**: Before mutation, the system MUST independently establish that the target is open, D/R/B are available and agree, and the editor reports document-attributable clean state. Missing or invalidated evidence, dirty state, or divergence MUST cause safe refusal, not automatic repair. Source equality MUST NOT substitute for dirty-state observation.
- **FR-004**: The expected revision basis MUST protect against stale read-modify-write intent and remain bound to the document and session actually observed. The operation MUST enforce that basis and all safety preconditions at the mutation boundary, including protection against changes between preflight and application. If equivalent stale-write and human-work protection cannot be ensured, it MUST refuse. Omitted or unusable revision evidence MUST NOT permit an unconditional edit.
- **FR-005**: A pre-application refusal MUST leave source, dirty state, and editing history unchanged by the operation. It MUST NOT save unsaved human work, merge competing versions, force overwrite, open/close/create a document, or force-load missing state to make the request eligible. Independent human/editor activity remains possible and MUST be distinguished from operation-caused changes.
- **FR-006**: Successful edits MUST use native editor mutation and history semantics as the canonical route wherever possible, preserve unrelated documents and prior valid undo history, and form one logically reversible edit. A disk write followed by assumed reload is not sufficient. Any alternative considered in planning MUST meet the same transaction, coherence, and native-history guarantees.
- **FR-007**: Each attempt MUST distinguish request acceptance, source application, persistence/synchronization, and independent postcondition verification. Acceptance, delivery acknowledgment, a completed save, or application alone MUST NOT constitute verified success.
- **FR-008**: Verified edit success MUST require fresh, independently attributed post-change observations showing the intended resulting source as D == R == B, observable document-specific clean state, and completed disk/Resource/editor synchronization. Required evidence that is missing or detectably invalidated MUST prevent success; mutation-path acknowledgments and copies of the submitted text are not independent observations.
- **FR-009**: Successful edits MUST have an observed post-change parse result for the target script without a parse error. A parse error or unavailable required parse evidence MUST be explicit and prevent verified success, while retaining the actual application and coherence state. This does not require project-wide diagnostics or guarantee gameplay correctness.
- **FR-010**: Outcomes MUST expose the requested target and the resolved target, or why no target could be resolved; expected and observed revision information; reached transaction stage; known application state; D/R/B availability and synchronization; dirty state; relevant source-attributed diagnostics; native Undo/Redo participation; and partial failures. Unavailable or invalidated evidence MUST carry a reason and MUST NOT imply that an authority was observed. Evidence MUST remain associated with its actual target and observation interval.
- **FR-011**: Machine-readable outcomes MUST distinguish verified changed, verified unchanged, safe refusal, and applied-but-unverified/partly applied/application-unknown attempts. Reasons MUST distinguish dirty conflict, revision mismatch, other divergence, known stale Resource, known stale buffer, missing safety observation, ambiguous or invalid target, closed/unknown-open state, unavailable/disconnected editor, parse error, persistence failure, timeout, cancellation, unsupported capability, and access/approval denial wherever applicable. Unavailable runtime MUST be explicit when reporting durability evidence. No undifferentiated error may conceal application state.
- **FR-012**: Each controlled local edit attempt MUST return a terminal result within ten seconds, including an unresponsive-editor case. Timeout/cancellation/disconnection MUST NOT imply rollback or definitely-not-applied state. A result may state not-applied only with evidence that application did not occur and cannot occur later from that attempt; otherwise uncertainty MUST be explicit. Timing rules for existing observations remain unchanged.
- **FR-013**: The operation MUST preserve newer human work during application, synchronization, and verification. It MUST NOT force convergence or attempt an implicit rollback by overwriting work that arrived after its original safety check. Loss of the preconditions or validity of the postconditions MUST produce a truthful non-success result with any known changes retained as evidence.
- **FR-014**: Mutation attempts MUST NOT be automatically retried. The documented caller behavior after a possibly applied attempt MUST require fresh target/state observation and a new valid revision basis before another mutation; replay of a failed or interrupted request MUST NOT be advertised as idempotent without proof. This feature does not require a replay cache or recovery service.
- **FR-015**: The requested agent edit MUST be reversible and reapplicable through real Godot editing history. One ordinary Undo MUST restore the pre-edit buffer source and Redo MUST restore the intended edit; actual D/R/B/dirty transitions MUST be independently observed. After ordinary Save and synchronization at each state, D/R/B MUST agree with that restored or reapplied source. Normal unsaved state immediately after a human Undo/Redo MUST NOT be reported as persisted convergence.
- **FR-016**: Pre-existing valid undo entries MUST remain usable in their native order, subject to normal native redo-branch invalidation for a new edit. Refused and unchanged attempts MUST add no history entries. The feature MUST NOT create an unrelated alternate history or substitute a fresh source edit for genuine Undo/Redo.
- **FR-017**: Verified successful edits MUST survive ordinary Save, close/reopen, reparse, filesystem rescan, and runtime launch wherever applicable, absent later intentional changes. Repeated edits MUST retain the same revision/conflict protections, persistence, and history behavior as a single edit. Every claimed guarantee MUST have real-editor evidence; unavailable observability is not an inapplicability justification.
- **FR-018**: An already-satisfied request MUST be identified as verified unchanged only after the same target, revision, safety, and required postcondition checks. It MUST cause no source write, save, or history mutation, and MUST NOT claim that a new edit occurred.
- **FR-019**: The completed observation capability MUST retain its existing read-only semantics and limits, including independent D/R/B evidence, document-specific dirty attribution, distinctions for closed/unknown/unavailable state, fresh observations, and non-interference. A complete observation, including a divergent one, MUST NOT itself authorize mutation or become a conflict error merely because editing now exists.
- **FR-020**: Editing MUST stay local-only and project-confined, preserve existing authentication and source-free private routing metadata, and introduce no arbitrary filesystem access, evaluation, process execution, remote access, or telemetry capability. Source may appear in explicitly requested results, not incidental logs or session metadata. Unrelated projects, documents, and unsaved work MUST remain untouched. Permission or approval MUST NOT substitute for safety checks.
- **FR-021**: The capability MUST preserve the architectural separation between agent-facing interaction, protocol-independent automation/transaction semantics, and Godot integration. It MUST NOT add MCP exposure, a parallel mutation safety model, or gameplay authority. Tooling/export isolation MUST remain explicit and verified for the changed capability.
- **FR-022**: All mutation A–E scenarios and applicable durability checks MUST pass in real Godot before this feature is complete. Pure decision logic and changed boundary/state transitions MUST also receive appropriate deterministic, isolated coverage. Exact supported versions/environments and verified limitations MUST be documented from the required CI and real-editor evidence; prior observation-only evidence does not establish mutation support. Unsupported/refused outcomes for every edit do not satisfy the required positive capability.

### Outcome Interpretation

Outcome names describe user-visible meaning, not a selected schema or transport.

- **Verified changed**: A real source change occurred and all required independently observed postconditions passed. This describes the verified interval, not a promise that the developer cannot edit afterward.
- **Verified unchanged**: The requested result was already present, all required current checks passed, and the operation changed no source or history.
- **Refused, not applied**: The operation established that its mutation did not occur and will not later occur. The reason and any permissible relevant observations are explicit.
- **Applied but unverified / partly applied**: Evidence establishes a change, but one or more required persistence, synchronization, parse, or independent verification conditions did not pass. Known changes and remaining uncertainty stay distinct.
- **Application unknown**: Evidence cannot establish whether or how far the change applied. A missing acknowledgment, timeout, or lost connection is not sufficient to classify it as not applied.

An outcome can retain independently observed partial evidence without becoming a success. Conflicting or unavailable sources are not replaced with the requested text. Re-observation informs a later decision; it is not automatic rollback, repair, or retry.

### Scope and Governance Alignment

This is an open-document mutation slice of **Phase 1 — Live-editor script coherence**, building on the completed [observation specification](../001-observe-gdscript-state/spec.md), its [completed tasks](../001-observe-gdscript-state/tasks.md), and [recorded evidence](../001-observe-gdscript-state/quickstart.md#29-rebased-t008-real-editor-acceptance-and-completion-2026-09-27). It does not redefine observation success as mutation safety.

In scope are one-script revision-guarded native editing, persistence needed for verified edit success, safe refusals, truthful interrupted/partial outcomes, preservation and exercise of native history, and the real-editor proof needed for that capability. Save is part of edit persistence; ordinary developer Save/Undo/Redo/close/reopen actions also serve as acceptance interactions.

Out of scope are script discovery and open/close commands; independent Save or general history-control commands; creating, deleting, renaming, or editing embedded scripts; multi-file/batch edits; conflict merging or force writes; concurrent-agent orchestration; MCP; broad core generalization; scene/resource/project-setting authoring; runtime/debugger/LSP tooling; and distribution/platform expansion. Runtime launch for permitted durability fixtures does not introduce runtime controls or arbitrary execution as product capabilities.

The [constitution](../../.specify/memory/constitution.md), [working agreement](../../AGENTS.md), [roadmap](../../ROADMAP.md#phase-1--live-editor-script-coherence), and [architecture](../../ARCHITECTURE.md) govern this work. This specification requires Principles I–IV's coherence, human-work protection, native semantics, and independent verification; V and X's confinement and truthful outcomes; VI and XII's real-editor mutation coverage; and VII–XI's ownership, export isolation, minimal surface, privacy, and independent implementation. Mutation-semantic and security-sensitive planning/review must explicitly record compliance.

**Principle XIII boundary:** The present failure mode is an agent overwriting newer human work, creating editor/disk divergence, losing history, or reporting an unverified edit as successful. Existing observations are read-only and cannot themselves apply, persist, or verify a mutation. The simplest scope is one guarded open-document edit using the existing routing and observation foundation, not a second framework. Planning must justify any additional mechanism against that present need, the simpler alternative, existing mechanisms' limits, ongoing cost, and why the cost is warranted now. This specification mandates no new service, general transaction engine, approval layer, CI provider, runner topology, or operational prerequisite. Real-editor behavioral evidence remains mandatory; infrastructure ceremony is not a substitute.

All A–E gates apply, including actual native Undo/Redo because this feature explicitly claims it. Evidence must independently observe relevant authorities and state transitions in real Godot, with explicit deadlines and event-based synchronization; retain regression cases for discovered failures. Headless runtime, disk-only checks, mocks, and history registration alone cannot prove visible-buffer coherence or native reversal. Any inapplicable durability surface/scenario needs a recorded substantive justification; lack of access is not a passing result. The feature is not complete until its positive and refusal cases and applicable gates are satisfied. Completing it does not automatically complete Phase 1; remaining roadmap capabilities and exit evidence must be assessed separately.

#### Phase 1 concurrency boundary

FR-004/FR-013 and their acceptance scenarios require guarded mutation in the selected editor: fresh target-bound revision/state checks at source application and before persistence, protection of unsaved human editor work, no retargeting on document/Resource/buffer/session identity change, and an actual persistence write that cannot be redirected by path or parent replacement. Known stale or conflicting state MUST NOT authorize continuing an obsolete edit. Inability to protect the applicable editor mutation boundary or obtain required safety observations remains grounds for refusal.

This feature does **not** promise atomic exclusion or linearizable compare-and-write against an arbitrary non-cooperating external process writing the same filesystem object after the final valid checks during the physical commit interval. It does not claim that every transient external write can be detected or prevented. Stronger coordination across competing actors belongs to later concurrency scope if specified, not a new Phase 1 mechanism or guarantee.

This limitation MUST NOT convert invalidated evidence into success. Known/detected interference that invalidates required target, revision or postcondition evidence, changed identity, missing required evidence, or divergent postconditions MUST produce a truthful non-success outcome even if later text samples match. Before any application, refusal leaves source/history unchanged; after possible application, retain actual changes and uncertainty under FR-007–FR-014, without reasserting older intent or claiming rollback. Independent D/R/B, document-specific dirty/synchronization state, source-attributed parse verification and all applicable native history/durability gates remain required. External changes cannot be ignored or checks disabled to manufacture eligibility or success.

### Key Entities *(include if feature involves data)*

- **Edit target**: One local project, live editor-session lifetime, and already-open standalone script identity; names or focus alone do not establish it.
- **Expected revision basis**: Caller-held evidence of the source state and target identity used to decide the change, checked for freshness and safety at application. It is not permission to bypass dirty-state checks.
- **Edit intent**: The caller's desired source change and resulting source against that basis, limited to one document.
- **Transaction evidence**: Before/after independent D/R/B observations, attributable dirty state, parse/diagnostic evidence, observation intervals, revisions, and their validity or unavailability.
- **Edit outcome**: The reached stages, target, known application state, observed postconditions, refusal/interruption reason, native-history participation, and actionable limitations.
- **Native history transition**: The actual agent edit, its reversal, and its reapplication through the editor's history, with observed state and persistence before and after ordinary Save.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In 100% of supported clean-open edit acceptance cases, the requested resulting source is independently observed on all three authorities before success, with clean/synchronized state and a valid observed parse result; the developer performs no reconciliation.
- **SC-002**: In 100% of dirty, stale-revision, divergent, missing-observation, ambiguous/replaced-target, closed/unsupported-target, and denied-access acceptance cases, unsafe requests make zero operation-caused source/history changes and preserve all unsaved human work. Changes injected between preflight and application are included.
- **SC-003**: In every interruption, persistence-failure, parse-error, and post-application human-edit case, reported stage and application certainty match independent witnesses. There are zero false verified-success, definite-not-applied, or rollback claims; required unavailable facts and distinct failure reasons are visible in the result alone.
- **SC-004**: Every controlled local edit attempt, including an unresponsive editor, returns a terminal result within ten seconds. Existing observation attempts retain their own five-second acceptance deadline.
- **SC-005**: Every native-history acceptance case restores the complete prior source with one Undo, restores the agent result with Redo, and converges to each saved state across applicable authorities. Earlier valid undo history remains usable, and refused/unchanged attempts add zero entries.
- **SC-006**: All applicable Save, close/reopen, reparse, rescan, and runtime durability cases retain the intended successful revision with zero spontaneous reversions or stale reopened buffers. At least one permitted runtime fixture demonstrates the changed behavior; absent applicable runtime evidence is not a pass.
- **SC-007**: A real-editor sequence of at least 20 successful edits with intervening Saves and at least three dirty/stale refusals loses no intended revision or unsaved human work, conceals no conflict, preserves usable native history, and retains the final result after close/reopen.
- **SC-008**: The existing observation acceptance scenarios retain their specified outcomes and non-interference guarantees. For every edit scenario, a reviewer can determine target, changed/unchanged/unknown application state, D/R/B and dirty-state evidence, history participation, and the required safe next action from the result without guessing. Privacy and production-export checks show no new project-content leakage or shipped tooling behavior.

## Assumptions

- Feature 001's observation foundation is complete; its task completion and recorded evidence, not its original draft-generation header or PR delivery labels, establish this dependency. No observation task is being reopened or reimplemented here.
- The developer/caller already knows the intended script and supplies enough target identity. The script is already open in an authorized local editor; discovery and document opening are not prerequisites to add inside this feature.
- One request concerns one script; the feature does not claim safe concurrent-agent orchestration. Human activity and external changes are still realistic interference, not conditions that can be ignored or disabled to claim safety.
- Conflict policy is refusal. A human can resolve unsaved work separately, after which the caller must observe again and provide a fresh revision basis. There is no automatic merge, force path, or permission prompt that replaces this check.
- Native Undo/Redo is claimed as behavior of the new edit, not as a general caller API. Ordinary unsaved states produced by human history actions are valid editor states, not false mutation-success evidence.
- Ten seconds bounds an individual edit result in the controlled local acceptance environment, not sustained throughput, arbitrary project size, or concurrent load. Planning must define reproducible supported fixtures and timing evidence without weakening the required successful path.
- Exact mutation-supported versions and environments must be selected and proven during planning/implementation. The observation feature's tested environment is existing evidence to reuse where applicable, not a new mutation support claim or mandate for broader platform work.
- Planning retains decisions about edit representation, revision mechanisms, APIs, transport evolution, persistence/synchronization integration, internal boundaries, dependencies, limits, and test implementation. New complexity must meet Principle XIII; existing safety mechanisms must not be duplicated without demonstrated need.
- Creating this draft and its quality checklist does not approve implementation or establish any mutation gate as passed. Clarification, planning, task derivation, granularity review, analysis, implementation, and acceptance remain separate work under repository policy.
