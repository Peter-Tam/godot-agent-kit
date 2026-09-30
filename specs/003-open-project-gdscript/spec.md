# Feature Specification: Safely Open a Known Project GDScript

**Feature Branch**: `spec/003-open-project-gdscript`

**Created**: 2026-09-29

**Status**: Implementation — T001's private native boundary and compatible cutover are complete; the public caller and cumulative acceptance remain pending T002/T003 work. The feature is not complete.

**Input**: Generated feature description supplied to `/speckit.specify`:

> Safely open one known, existing standalone GDScript in an explicitly selected authorized local Godot project and live editor session. A coding-agent caller can already inspect a closed script and safely edit an open one, but currently depends on the developer to open the document manually. Add the missing closed-to-open step in Phase 1 without coupling it to script discovery or changing the existing observation and editing contracts.

> Use the supplied exact target, preserve source and unsaved human work, and independently verify that the intended script is actually open with its own live buffer. For a newly opened clean script, establish independent disk, loaded-Script and buffer agreement and document-attributed clean state; an editor acknowledgment is not success. Loading the selected script and creating its buffer are intentional lifecycle effects, not permission to rewrite source, save, reload conflicting state, execute project code, change unrelated documents, or discard history. If the document is already open, report that fact from fresh evidence without reopening, reloading, selecting another tab, clearing history or changing its dirty state; an already-open dirty or divergent document remains intact and is never represented as ready for editing.

> Refuse ambiguous, missing, replaced, outside-project, unauthorized or unsupported targets and unsafe opening contexts before effects. Unknown open state, conflicting already-loaded source, missing required safety evidence, or unpermitted load-time effects must not be repaired or guessed away. Protect the target and human work through the opening boundary. Distinguish accepted, opening initiated, opened, and independently verified states; timeout, cancellation or disconnection may leave an open document and must retain known effects and uncertainty without implicit rollback, closing newer human work, automatic retries or retargeting. A later edit still requires fresh observation and its existing revision and dirty-state checks.

> Prove real-editor opening, already-open non-interference, dirty and unrelated-document preservation, stale-target/effect refusals, interruptions and repeated use. Exercise the composed open-observe-edit workflow with the existing applicable A–E, native history and Save/reopen/reparse/rescan/runtime durability guarantees. Preserve authenticated local routing, project confinement, source privacy, protocol-independent ownership and production-export isolation. Specify observable behavior and safe limitations; leave APIs, transports, internal layout, admission mechanisms, dependency choices and exact supported environments to planning and evidence.

> Exclude script discovery/indexing, script creation/rename/delete, embedded or non-GDScript documents, standalone close/Save/Undo/Redo controls, automatic conflict resolution and force writes, batch or concurrent-agent orchestration, MCP exposure, broad core generalization, scene/resource/project-setting authoring, runtime/debugger/language-intelligence tools and distribution expansion. No new service, framework, approval layer, CI topology or operational prerequisite is required without a concrete present need under Principle XIII.

## User Scenarios & Testing *(mandatory)*

The primary caller is a coding-agent automation layer acting for a developer using a live Godot Editor. The caller already knows the intended project, editor session and standalone script path. The missing capability is opening that existing script without requiring the developer to prepare a tab manually.

**D** is source on disk, **R** is source in the loaded Godot Script, and **B** is source in the live editor buffer. They remain independent authorities. A **verified newly opened** result establishes that the intended document became open, its independently observed D/R/B agree, and its document-attributed dirty state is clean. This proves the opening result, not parse validity, gameplay correctness or permission to edit. An **already-open, unchanged** result instead describes an existing document whose sources, dirty state, selection and history the request did not change; that document may remain dirty or divergent.

Opening is an editor lifecycle change, not a source edit. Loading an eligible target and creating its editor buffer are intentional effects. Source writes, automatic Save, conflict reconciliation, history changes and unpermitted project-code execution are not. The stories are independently verifiable outcomes of one capability; safety and truthful partial outcomes cannot be postponed beyond its first exposed opening path.

### User Story 1 - Open a Known Closed Script and Continue Safely (Priority: P1)

As a coding-agent caller, I want to open the known script I intend to work on, so I can use the existing observation and guarded-edit capabilities without asking the developer to open it first.

**Why this priority**: Closed-script inspection and already-open editing are complete, but neither supplies the transition between them. One verified opening is independently useful without a script catalog or general document-management interface.

**Independent Test**: In a real editor, start with an existing closed script, independently confirm its source and absence of a buffer, request opening, and inspect the actual resulting document and D/R/B. Follow with a separate existing observation and eligible edit.

**Acceptance Scenarios**:

1. **Given** an exact authorized live session and a supported, existing standalone GDScript that is confirmed closed and not yet loaded, **When** opening is requested, **Then** the intended script opens in that editor and receives a verified newly opened result only after independent open-document, D/R/B and clean-state evidence. D is unchanged; absence of R and B before opening is not confused with unavailable required post-opening evidence.
2. **Given** a closed script with an already-loaded, independently observable R agreeing with D and no unresolved editor-owned source changes, **When** it is opened, **Then** the existing source is preserved and the new buffer agrees with D and R. Opening does not reload or replace conflicting state to make it eligible.
3. **Given** two same-named scripts or distinguishable editor sessions, **When** the caller supplies an exact target, **Then** only that target is opened; every returned fact belongs to it. Editor focus, a basename or a previously used session never chooses the destination.
4. **Given** a verified newly opened script within the existing edit capability's supported profile, **When** the caller separately observes it and requests an edit using fresh revision evidence, **Then** the existing edit rules apply unchanged and the intended edit can succeed without human tab preparation or reconciliation. Opening itself submits no edit and grants no reusable mutation authorization.
5. **Given** an otherwise safe, supported GDScript with a syntax error, **When** opening is requested, **Then** the script can be opened and its actual source and available parse diagnostic reported without repair. Syntax invalidity alone is not a missing/non-GDScript target or an opening failure; no valid-parse or edit-readiness claim follows.

---

### User Story 2 - Keep Existing Buffers and Human Work Intact (Priority: P1)

As a developer, I want opening requests to preserve my unsaved work and native history, including when the requested script is already open, so an agent cannot use opening as a hidden reload or Save.

**Why this priority**: Replacing a dirty buffer or applying unrelated pending work would defeat the safety of the completed observation and editing capabilities.

**Independent Test**: Prepare independently witnessed clean, dirty and divergent open targets, including non-selected tabs and known native history. Repeat requests and open a different closed target while an unrelated document is dirty. Compare source, dirty state, document identity, selection and history before and after.

**Acceptance Scenarios**:

1. **Given** the exact target is already open and clean, including in a non-selected tab, **When** opening is requested, **Then** the result reports already open and unchanged from fresh evidence. It creates no duplicate buffer, does not reopen, reload or select the target, and adds no history entry.
2. **Given** the target is already open with dirty or divergent state, including a dirty buffer whose text equals D, **When** opening is requested, **Then** its buffer, loaded source, disk source, dirty indication and history remain intact. The result reports the existing open state and actual independent observations or their explicit limitations, not newly opened, clean, coherent or safe-to-edit state by inference.
3. **Given** another document has unsaved work and earlier native history, **When** an eligible closed target is opened, **Then** the unrelated document is not saved, closed, reloaded, edited or stripped of history. Native selection of the newly opened target is permitted and reported; selecting other documents to obtain evidence is not.
4. **Given** a human opens or edits the target after initial checks but before the operation's opening boundary, **When** the request proceeds, **Then** stale closed-state evidence cannot authorize replacement or reload. The operation either establishes the same document as already open without changing it or refuses with the detected change; missing protection requires refusal.
5. **Given** opening has begun and the human then edits the resulting buffer, **When** verification occurs, **Then** the newer work remains intact. Invalidated clean/coherence evidence produces an explicit unverified result with known opening effects, not automatic Save, reload, reassertion of older source or closure of the new buffer.

---

### User Story 3 - Refuse Unsafe or Unidentifiable Opening (Priority: P1)

As a coding-agent caller, I want opening to refuse a wrong target or an unsafe load before effects, so supplying a path does not bypass project confinement or authorize project execution.

**Why this priority**: Opening can acquire loaded state and create editor effects even though it is not a source edit. Existing read permission is not blanket permission for every loading consequence.

**Independent Test**: Exercise ambiguous and replaced sessions, missing and outside-project files, changed document identities, conflicting loaded source, unavailable safety evidence and controlled scripts with disallowed load-time effects. Independently witness that refusal neither opens a document nor changes source, human work or history.

**Acceptance Scenarios**:

1. **Given** ambiguous selection, an ended/replaced session, denied access or an outside-project target, **When** opening is requested, **Then** a distinguishable refusal identifies the selection/access problem, returns no unauthorized or candidate source, and opens nothing. No convenient editor or replacement session is substituted or started.
2. **Given** a missing file, non-GDScript file, embedded script or unsupported target representation, **When** opening is requested, **Then** the outcome identifies the relevant missing/invalid/unsupported reason without creating a file, opening a container or silently converting the target.
3. **Given** a closed target whose existing loaded source conflicts with D, whose relevant editor-owned dirty state is unresolved, or whose open state or necessary safety facts cannot be established, **When** opening is requested, **Then** it is refused before lifecycle effects. No reload, forced synchronization, source substitution or dirty-state inference manufactures eligibility.
4. **Given** the file, parent path, document or session changes after selection and before opening, **When** the opening boundary is reached, **Then** detected identity/revision invalidation prevents opening the replacement or combining its evidence with the original target. An overlap with another kit operation must not bypass the same protection or apply later after a terminal refusal.
5. **Given** opening a target would execute project code, access unauthorized dependencies or cause other effects outside this capability's permissions, or safety of that context cannot be established, **When** opening is requested, **Then** the operation reports an unsupported/denied context before triggering those effects. Approval is not a substitute for human-work or coherence checks; this feature adds no approval or arbitrary-execution capability.

---

### User Story 4 - Understand Interruptions and Repeated Use (Priority: P1)

As a coding-agent caller, I want to know whether an unsuccessful opening left a document open and what remains uncertain, so I can inspect current state rather than replay an action or close newer human work blindly.

**Why this priority**: A missing acknowledgment cannot distinguish an unstarted operation from an opened but unverified document.

**Independent Test**: Interrupt requests before opening, after opening starts and during independent verification, with separate editor witnesses. Exercise repeated opens and the composed opening/observation/edit workflow, checking native history, ordinary editor durability and existing privacy/export behavior.

**Acceptance Scenarios**:

1. **Given** an opening request is acknowledged or the target buffer is created but independent verification is incomplete, **When** an outcome is returned, **Then** it does not claim verified newly opened. Known loaded/open/selection effects, reached stage, missing evidence and any actual source changes are explicit.
2. **Given** cancellation, deadline expiry or disconnection occurs, **When** the attempt terminates, **Then** the result distinguishes proven no opening effect from known effects and effects unknown. A proven not-applied refusal cannot open later; loss of response alone does not prove not-applied. The request does not automatically retry or target a new session.
3. **Given** an interrupted attempt may have opened a document, **When** the caller considers another operation, **Then** the result directs it to freshly observe the explicit target first. No rollback is presumed and no buffer is automatically closed, especially if a human has since used it. A later request against the same still-open identity follows the already-open no-change behavior, not a stale replay guarantee.
4. **Given** at least 20 opening requests spanning supported closed, already-open clean, already-open dirty and unsafe targets, with deliberate human close/reopen and edits between requests, **When** the sequence runs, **Then** the results reflect current identities and state, create no duplicate buffers, preserve source and history, and cause no unexpected reopening after refusal. At least five requests begin with a genuinely closed supported target and succeed; at least five exercise dirty already-open preservation.
5. **Given** a script opened by this capability and the completed edit workflow, **When** applicable A–E, ordinary Save, human close/reopen, reparse, rescan and permitted fixture runtime checks run, **Then** existing guarantees remain satisfied, without agent-created divergence or lost work. Existing observation remains read-only and existing editing still refuses a closed or unsafe target rather than opening it implicitly.
6. **Given** authorized, denied, ambiguous and interrupted requests with synthetic source sentinels, **When** results, incidental logs, private routing metadata and production exports are inspected, **Then** only authorized requested results expose selected source, other targets remain untouched, and no active opening tooling or gameplay dependency is exported.

---

### Edge Cases

- Empty source is an observed value, not a missing script. Whitespace and line-ending differences must not be normalized into false agreement; unsupported representation or size limits remain explicit rather than truncating text.
- A script is closed but R is retained from earlier use: distinguish a genuinely absent R from unreadable or conflicting existing state before allowing new load/open effects.
- A non-selected dirty tab is already the target: the request must not select it to manufacture observability or treat equal source as proof of cleanliness.
- A file is renamed, deleted or replaced during the request, including a path redirect outside the project: do not open a replacement under an old identity or return its source as the original's evidence.
- A source is readable but not writable: opening does not require write permission merely because editing might later require it. Ordinary editor metadata is not permission to write project source.
- A script has a syntax error: opening for inspection or later repair is distinct from guaranteeing that it can run or satisfies the existing edit capability's post-change validation.
- A load/open produces a native prompt or cannot finish within the deadline: do not accept a destructive prompt, Save, discard or wait indefinitely; report observed effects and the unresolved state.
- The target is closed/reopened or another tab is selected by the human during verification: do not steal selection back or combine identities; retain usable attributed evidence and invalidate unsupported claims.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The caller MUST be able to open one known, existing standalone project GDScript in one explicitly selected authorized local editor-session lifetime. This MUST be a distinct lifecycle request, not an implicit behavior of observation or editing, a file-creation operation or a requirement to discover scripts first.
- **FR-002**: Target resolution MUST preserve existing authenticated, local, project-confined routing. It MUST refuse ambiguous, ended/replaced, unauthorized or outside-project targets without choosing by focus, returning candidate source, starting an editor or substituting another session. Every effect and observation MUST remain bound to the actual selected identity.
- **FR-003**: Before new loading/opening effects, the system MUST establish current document-open state and the relevant target, source and human-work safety facts. Confirmed absence of an unloaded R or closed B is expected, not fabricated evidence; an existing R whose source or relevant unsaved state cannot be safely established MUST NOT be treated as absent or clean. Conflicting loaded/disk source and unresolved dirty state MUST refuse without repair.
- **FR-004**: Safety checks MUST remain valid at the opening boundary. Detected source/revision, namespace, document or session changes MUST NOT redirect opening or permit stale intent. Kit-operation overlap MUST NOT create duplicate or later-applying effects after terminal refusal. An intervening human open may be reported as already open only after fresh exact-identity checks and without changing that document; otherwise refuse. If the boundary cannot be protected, report unsupported rather than assume safety.
- **FR-005**: Opening MUST use Godot-native editor lifecycle semantics as the canonical route. It MAY load the admitted target and create its buffer, and a new open MAY natively select that target. It MUST NOT write project source, Save, force reload, resolve source conflicts, apply pending edits, close documents or change native editing history. Unrelated documents and human work MUST remain unchanged by the request; any selection change MUST be attributable and reported.
- **FR-006**: An exactly identified already-open target MUST be handled without reopen, reload, duplicate buffer, selection change, source write, dirty-state change or history change. Fresh observations MUST report actual state or explicit limitations. Dirty/divergent existing state MUST be preserved and MUST NOT be described as clean, newly opened or eligible for an edit; no-effect recognition does not require resolving that state.
- **FR-007**: Permission to open MUST NOT imply permission for arbitrary evaluation, project-code execution, unauthorized dependency access or other source/editor effects outside this capability. Such a context, or inability to establish the necessary effect safety before dispatch, MUST cause an actionable unsupported/denied refusal. Existing least-privilege boundaries apply; this feature introduces no force or approval bypass.
- **FR-008**: Verified newly opened MUST require independent post-operation evidence of the exact open document and its buffer, agreeing D/R/B, document-attributed clean state, and no detected invalidation of those facts. D MUST remain the intended source rather than an operation-written replacement. An acknowledgment, successful load return, copied input or equal remaining pair MUST NOT substitute for required observation. Missing, divergent or invalidated required evidence MUST prevent verified success.
- **FR-009**: Already-open recognition MUST distinguish independently established open state from unknown/unobservable state and carry the available D/R/B and dirty evidence with their limitations. It MUST NOT imply complete observation, coherence, parse validity or authorization for later mutation. A later edit MUST obtain its own fresh valid revision basis and satisfy every existing safety condition; opening evidence is not a permission token.
- **FR-010**: Each operation MUST distinguish acceptance, initiation of lifecycle effects, known application and independent verification. Results MUST expose requested/resolved target, reached stage, known loading/open/selection effects, observed revisions and D/R/B availability, dirty/synchronization state, relevant available diagnostics, history non-participation/preservation, evidence interval/validity and actionable limitations. Unknown facts MUST NOT be invented; known partial effects and unexpected source changes MUST NOT be hidden.
- **FR-011**: Outcomes MUST distinguish verified newly opened; already open without request-caused change; refused with no opening effect; applied/partly applied but unverified; and effects unknown. Reasons MUST distinguish ambiguous/changed target, missing/invalid/unsupported document, dirty/conflicting loaded state, known stale Resource or buffer, unknown open state, missing required evidence, unsafe load context, denied access, unavailable/disconnected editor, timeout and cancellation wherever applicable. Syntax errors alone MUST NOT make an otherwise safe supported GDScript unopenable; available parse diagnostics remain separate from lifecycle success and no valid-parse claim is required.
- **FR-012**: Every controlled local opening attempt MUST return a terminal result within ten seconds, including an unresponsive editor. Timeout, cancellation and disconnection MUST NOT imply rollback or definitely-not-applied state. A not-applied result requires evidence that effects did not occur and cannot occur later from that attempt; otherwise the uncertainty MUST be explicit. Existing observation and edit deadlines remain unchanged.
- **FR-013**: Newer human work MUST be preserved during opening and verification. The system MUST NOT overwrite it, Save it, reload it, restore old source or close a buffer to undo a partly completed opening. Invalidated postconditions MUST produce truthful non-success with known effects and usable evidence, not success based on earlier samples.
- **FR-014**: Opening attempts MUST NOT automatically retry, queue a refused request for later application or replay against replacement state. Results for possibly applied attempts MUST direct the caller to fresh observation before another action. Repeating a new request for the same still-open document MUST use FR-006; this conditional no-change behavior MUST NOT be advertised as unconditional replay safety across close/reopen or session replacement.
- **FR-015**: Source limits, source attribution and availability distinctions MUST preserve the existing observation contract. Source MUST NOT be truncated, copied from another authority or represented as empty to satisfy limits. A declared unsupported opening profile MUST refuse before effects where detectable; an unexpected post-opening observation limit MUST produce an unverified outcome with known effects, not a false complete result or loss of unrelated permissible evidence.
- **FR-016**: Existing observation and edit behaviors MUST remain unchanged, including read-only observation, closed-target edit refusal, independent evidence, stale-write and dirty protection, native history and edit durability. Opening MUST NOT become a second mutation policy. The architectural boundary between agent-facing interaction, protocol-independent automation semantics and Godot integration MUST be preserved; no MCP or gameplay authority is added.
- **FR-017**: The capability MUST preserve source privacy, source-free private routing metadata, local-only operation, no telemetry and tested development/export isolation. Only explicitly requested authorized results may contain selected project source; unrelated project content MUST NOT be exposed through results, logs or metadata. Opening adds no arbitrary filesystem, network or execution capability.
- **FR-018**: Before feature completion, real-editor acceptance MUST establish positive closed-to-open behavior, already-open non-interference, human-work preservation, safe refusals, truthful interruptions, repeated use and the composed workflow's applicable A–E/durability guarantees. Pure decision and changed boundary/state-transition behavior MUST receive appropriate deterministic isolated coverage. Supported versions/environments and limitations MUST be exact and backed by required CI and real-editor evidence; refusing every new open or borrowing previous edit evidence alone does not satisfy the feature.

### Outcome Interpretation

These meanings specify caller-visible behavior, not a wire schema or an additional transaction framework.

- **Verified newly opened**: The request caused the intended lifecycle transition and fresh independent postconditions passed. It does not certify parsing or authorize a subsequent edit.
- **Already open, unchanged**: Fresh evidence establishes the exact existing open document; the request caused no lifecycle/source/history/selection change. Dirty, divergent, missing or limited source observations remain explicit, not a clean-open or complete-observation claim.
- **Refused, not applied**: Evidence establishes that the request caused no loading/opening/selection effect and cannot do so later. Its reason and any authorized evidence remain available.
- **Applied or partly applied, unverified**: Some lifecycle effect is known, but required postconditions are not established. A loaded Script without a verified buffer is a possible partial effect, not success or proof of no application.
- **Effects unknown**: Evidence cannot establish whether/how far opening occurred. A lost acknowledgment is insufficient to claim not-applied.

Successful results describe their observed interval, not a freeze on later human activity. No result permits automatic reconciliation, rollback, retry or source editing. If independent observation itself is limited, the operation retains that limitation rather than changing the observation feature's meaning.

### Scope and Governance Alignment

This is one lifecycle slice of **Phase 1 — Live-editor script coherence**. It builds on the completed [observation specification](../001-observe-gdscript-state/spec.md) and [tasks](../001-observe-gdscript-state/tasks.md), and the completed [open-script editing specification](../002-edit-open-gdscript/spec.md), [tasks](../002-edit-open-gdscript/tasks.md) and [cumulative acceptance](../002-edit-open-gdscript/quickstart.md#14-t005-cumulative-acceptance-2026-09-29). Their completion is established by acceptance and task records, not original specification-generation headers or PR delivery labels.

In scope are one known-path closed-to-open transition, no-change recognition of an already-open document, inseparable preservation/effect checks, independently verified outcomes and bounded failure behavior. This removes the manual-opening prerequisite to the existing safe workflow. It does not require discovering an unknown script first.

Out of scope are script discovery/search/indexing; file creation, deletion or renaming; embedded and non-GDScript documents; independent close, Save and Undo/Redo controls; automatic conflict merging or force writes; batch and concurrent-agent orchestration; generalized core consolidation; MCP; scene/resource/project-setting authoring; runtime/debugger/language-intelligence tools; and distribution/platform expansion. Human close/reopen, Save and native history actions and permitted fixture runtime launches are acceptance interactions, not new product commands. Phase 1 remains in progress; completing this feature does not settle its remaining discovery/lifecycle coverage.

The [constitution](../../.specify/memory/constitution.md), [working agreement](../../AGENTS.md), [roadmap](../../ROADMAP.md#phase-1--live-editor-script-coherence) and [architecture](../../ARCHITECTURE.md) govern this work. Principles I–IV require independent authorities, preservation of human work, native lifecycle semantics and verified postconditions. V and X require confinement, effect-aware refusal and truthful uncertainty. VII–IX preserve ownership, tooling isolation and a small surface. VI, XI and XII govern real-editor evidence, independent implementation and quality. Mutation-semantic and security-sensitive planning/review must explicitly record compliance.

**Principle XIII boundary:** The concrete present gap is that a caller with a known closed script cannot use completed editing without manual editor preparation. Reusing read-only observation alone cannot create a buffer, while silently loading/reloading under a read would violate its contract. The simplest credible scope is one explicit guarded open using the existing routing, observation and common safety semantics. The new responsibility costs lifecycle admission/verification and corresponding transition evidence, justified by removing that manual step safely. No project index, background service, duplicate transaction model, generic lifecycle framework, new dependency, approval layer, CI provider or runner prerequisite is selected. Planning must justify any proposed mechanism against the current failure, simpler alternative, insufficiency of existing controls and ongoing cost; hypothetical wider isolation or future extensibility cannot add requirements silently.

Opening itself makes no source edit or native Undo/Redo claim, so it must add no source-history entry and need not invent a reversible tab-open action. Because it leads directly into mutation, the existing applicable A–E regressions must remain passing for the composed open → fresh observation → existing edit workflow: coherent edit, dirty refusal, actual native Undo/Redo, human close/reopen persistence and sequential edits. Save/reparse/rescan/runtime durability remains required where applicable, using permitted fixtures rather than a new execution capability. Real-Godot witnesses must establish the actual buffers and preserved history; disk checks, headless runtime and acknowledgment alone are insufficient. Planning/review must record substantive applicability reasons, not waive a gate because observation is unavailable. Use explicit deadlines and event-based synchronization and retain regressions for discovered failures.

### Key Entities *(include if feature involves data)*

- **Opening target**: The exact local project, editor-session lifetime and known standalone script identity/path; names or focus do not establish it.
- **Opening intent**: A request to make that existing document open, without requesting source change, Save, conflict resolution or execution.
- **Lifecycle evidence**: Before/after open state, actual document/buffer identity, loading and selection effects, independent D/R/B, attributable dirty state, revisions, history preservation and observation validity or limitations.
- **Opening outcome**: Reached stages, newly opened versus previously open state, application certainty, verified postconditions or refusal/interruption reasons and safe next action.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In 100% of supported closed-target acceptance cases, including unloaded and already-loaded clean targets and a safe syntax-error fixture, the intended script becomes open without human tab preparation. Every verified newly opened result matches independent D/R/B and clean-state witnesses, with zero request-caused source changes or guessed targets.
- **SC-002**: All already-open clean, dirty, divergent and non-selected cases preserve source, document identity, dirty state, selection and usable native history, with zero duplicate buffers, reloads, Saves or added history entries. Opening a different target preserves unrelated human work and history in every case.
- **SC-003**: Every unsafe-target, conflicting-loaded-state, missing-safety-evidence and disallowed-effect case refuses before prohibited effects. Race and overlap cases produce no stale retargeting, overwrite or late application after proven refusal, and no candidate or unauthorized source is disclosed.
- **SC-004**: Every interruption and post-opening human-change case reports lifecycle effects and uncertainty consistent with independent witnesses, with zero false verified-open, definite-not-applied or rollback claims. Every controlled attempt, including an unresponsive editor, terminates within ten seconds.
- **SC-005**: At least 20 real-editor opening requests, including at least five successful genuinely closed-to-open transitions and five dirty already-open requests, preserve all human work and source/history state. Deliberate intervening human close/reopen never leads to an old-identity result or unexpected late reopening.
- **SC-006**: The existing observation scenarios and applicable edit A–E, native-history and Save/close-reopen/reparse/rescan/runtime checks pass when exercised with this opening capability. The composed workflow needs no manual opening or repair and does not weaken closed-target edit refusal or fresh revision requirements.
- **SC-007**: For every acceptance case, a reviewer using the result alone can determine the exact target or refusal, whether opening was new/already satisfied/partial/unknown, which observations are usable and the safe next action. Privacy and production-export checks show zero incidental source disclosure or active shipped tooling introduced by this capability.

## Assumptions

- Features 001 and 002 are complete according to their authoritative task and acceptance records. This feature does not reopen their tasks or reinterpret observation success as mutation safety. Normal delivery sequencing still governs later implementation.
- The caller already supplies the script path and enough identity to choose one authorized local editor. Discovery is an independent roadmap gap, not an unstated prerequisite to opening a known target.
- Opening means presence as the intended real Script Editor document with its own buffer, not a promise to focus an already-open tab. A newly opened target may become selected through normal editor behavior; an already-open request leaves selection alone.
- The new-open path admits only contexts whose actual loading/opening effects can satisfy existing confinement, permission and preservation boundaries. Supported profiles and limits are planning/evidence decisions; a useful positive standalone-script path and the stated safe syntax-error case remain required.
- Human activity and external changes remain possible. The operation must honor detected invalidation and preserve work; it does not introduce cross-editor leases, distributed coordination or a claim of atomic exclusion against arbitrary non-cooperating external writers.
- Ten seconds bounds a single request in the controlled local acceptance environment, matching the existing mutation deadline. It is not a throughput, arbitrary-project-size or concurrent-load promise; planning must define reproducible fixtures and limits.
- Exact support claims for this new lifecycle operation require its own applicable evidence. Prior supported builds are a baseline, not permission to claim other versions/platforms or assume new opening behavior is proven.
- APIs, transport evolution, internal modules, admission mechanisms, dependency choices, concurrency protection and evidence collection remain planning decisions under existing architecture and Principle XIII. No implementation or plan is created by this draft.
