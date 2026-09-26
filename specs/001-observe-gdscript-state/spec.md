# Feature Specification: Observe Live GDScript Editor State Safely

**Feature Branch**: No feature branch created; specification generated on `main`.

**Created**: 2026-09-26

**Status**: Draft — quality-validated and ready for planning; implementation is not authorized.

**Input**: User description: "Observe live GDScript editor state safely. Identify the intended live Godot Editor, project, and script; independently observe disk source, loaded Script source, visible editor-buffer source, and unsaved human work. This feature is observation-only, not the rest of Phase 1."

## User Scenarios & Testing *(mandatory)*

The primary user is a coding-agent automation layer acting for a developer who may be working in the editor. The developer must be able to trust that inspection does not alter or lose their work.

In this specification, **D** means the script's source on disk, **R** means the source held by the already-loaded Godot Resource/Script, and **B** means the source in the live Script Editor buffer. These are independent authorities. A **complete observation** means the requested applicable state was actually observed; it does not mean the three sources agree or that a later edit is safe.

### User Story 1 - Inspect a Clean Open Script (Priority: P1)

As a coding-agent caller, I want to inspect one explicitly identified live script so that I know what each source authority contains and whether the editor reports unsaved work.

**Why this priority**: Later safe editing depends on facts about the live editor rather than assumptions based on a disk read.

**Independent Test**: In a real Godot Editor, open a known clean GDScript and independently establish its disk, loaded-script, buffer, and dirty state. Request an observation and compare each returned fact with the corresponding authority.

**Acceptance Scenarios**:

1. **Given** an unambiguously selected live editor/project and a clean open script with independently readable D, R, B, and dirty state, **When** the caller observes that script, **Then** the result identifies the project, editor session, and script; returns the source observed from each authority; reports the document as open and clean; and reports complete observation and agreement.
2. **Given** that same clean document, **When** it is observed again without intervening changes, **Then** the result contains observations made for the new request, and inspection changes no source, dirty state, open-document selection, or editing history.
3. **Given** an observable source whose text is empty, **When** the script is observed, **Then** that source is represented as an independently observed empty value, not as missing or unreadable.

---

### User Story 2 - See Unsaved Work and Divergence (Priority: P1)

As a coding-agent caller, I want to see unsaved human work and disagreements between source authorities so that I do not mistake disk contents for the developer's current work.

**Why this priority**: Concealing a human edit or falsely reporting a clean buffer would make a later mutation decision unsafe.

**Independent Test**: In a real Godot Editor, make a distinctive human change in an open buffer without saving. Establish each authority separately, observe the script, and verify that the unsaved change remains intact. Also exercise a separately established disk/loaded-source disagreement where the editor permits it.

**Acceptance Scenarios**:

1. **Given** a previously clean open script and a human buffer change that has not been saved, **When** the caller observes it, **Then** B contains that edit, dirty state reports unsaved work, D contains the independently read disk source, and R contains what the loaded Script actually exposes, whether or not it matches either other source. The result reports each known disagreement without treating divergence itself as an observation failure.
2. **Given** independently established differing values on at least two of D, R, and B, **When** all applicable state is observable, **Then** observation is complete, every observed pair is classified as equal or different correctly, and neither equality of the remaining pair nor the complete outcome conceals divergence.
3. **Given** that dirty state cannot be observed even though D, R, and B are readable and equal, **When** the caller observes the script, **Then** dirty state is explicitly unknown with a reason; it is not inferred to be clean, and the result is not a complete observation.
4. **Given** detectable source changes or a document-identity change during observation, **When** a result is returned, **Then** it identifies the affected observations as changed or invalidated, withholds a complete current-snapshot claim, and preserves any usable evidence with its limitations. An older observation is never substituted for a fresh read.
5. **Given** an unambiguously identified open script in an available editor that exposes an unsaved-work indication but not enough evidence to associate it with that document, **When** the script is observed, **Then** its dirty state is unknown, with unavailable document-specific attribution as the reason; neither dirty nor clean is inferred from that indication or from D/R/B comparisons. Independently observed sources remain available, the outcome is limited rather than complete, and the result does not misclassify the known target as ambiguous or the editor as disconnected. Observation does not change or save the human buffer.

---

### User Story 3 - Refuse the Wrong or Unavailable Target (Priority: P1)

As a coding-agent caller, I want target uncertainty and loss of editor access to be explicit so that I never receive source from a guessed editor or an earlier session.

**Why this priority**: Correct source from the wrong project or editor is not a trustworthy observation.

**Independent Test**: Use real editor sessions with distinguishable project/script contents, including two sessions for the same project where supported. Exercise incomplete selection, exact selection, disconnection, and session replacement; verify the selected identities and refusal outcomes.

**Acceptance Scenarios**:

1. **Given** a request that matches more than one live editor/project session, **When** observation is requested, **Then** the operation returns an ambiguous-target outcome explaining what identity information is needed. It does not choose the focused, first, newest, or previously used editor, and it returns no candidate script source.
2. **Given** distinguishable live sessions and enough caller-supplied identity to select exactly one, **When** its script is observed, **Then** every returned observation belongs to that project, session, and script, even if another session has the same project name or script path. Insufficient identity is never filled from editor focus.
3. **Given** an intended editor that is not available when requested, **When** observation is attempted, **Then** the caller receives a structured editor-unavailable outcome rather than cached editor state or automatic editor startup.
4. **Given** a selected editor that disconnects before observation completes, **When** the result is returned, **Then** it reports disconnection and identifies which observations, if any, were obtained before the loss. It does not claim a complete current live-editor observation.
5. **Given** an editor session that has ended and a replacement session for the same project, **When** a caller requests the ended session, **Then** it is reported unavailable or disconnected; the replacement is not silently substituted.
6. **Given** an editor that remains connected but does not answer in time, **When** the observation deadline expires, **Then** the caller receives a timeout outcome, distinct from a known disconnection, with any unavailable or invalidated observations identified.
7. **Given** a target outside the selected project or requiring access outside this feature's local, project-scoped boundary, **When** observation is requested, **Then** it is refused with an actionable reason and without exposing that target's source.

---

### User Story 4 - Understand Closed Documents and Missing Observability (Priority: P2)

As a coding-agent caller, I want closed scripts, missing or invalid targets, and unreadable open buffers to be different outcomes so that I can understand what is known without inventing a buffer or assuming the editor is clean.

**Why this priority**: Explicit limitations make the observation useful beyond the ideal clean-open-document case without expanding it into document opening or recovery.

**Independent Test**: In a real editor/project, prepare a valid closed script, an open script with a deliberately unavailable observation surface, and missing/invalid targets. Compare their results and confirm that observation opens nothing and repairs nothing.

**Acceptance Scenarios**:

1. **Given** a valid project GDScript confirmed not to be open in the Script Editor, **When** it is observed, **Then** the result reports not-open, independently reads D when accessible, reports R only if the Script is already loaded and observable, and reports B and document-buffer dirty state as not applicable because no open buffer exists. It neither fabricates B nor opens or loads the script to supply it.
2. **Given** a missing script with no corresponding open document, or an invalid target such as a non-GDScript file, **When** observation is requested, **Then** the result distinguishes missing from invalid and distinguishes both from a valid closed script, without fabricated source values. A syntax error in otherwise observable GDScript text does not by itself make the target invalid.
3. **Given** a script independently confirmed to be open but whose buffer cannot be read, **When** it is observed, **Then** open state remains open, B is unavailable with a reason, and other independently observable facts remain available. The result is incomplete rather than not-open or complete.
4. **Given** an open script whose disk source is unreadable or has disappeared, **When** it is observed, **Then** D reports the specific limitation, independently readable R and B are retained, and the result does not conceal the existing open document or report full agreement.
5. **Given** a script whose loaded source is not independently observable, **When** it is observed, **Then** R is unavailable with a reason, never copied from D or B, while all independently readable facts remain available.
6. **Given** that the system cannot determine whether a script is open, **When** observation is requested, **Then** open state is unknown, B and dirty-state limitations are explicit, and absence of a buffer observation is not presented as proof that the script is closed.

---

### Edge Cases

- A script is open in a non-selected editor tab: inspect its actual buffer only if observable without changing the user's selection; otherwise distinguish open-but-unobservable from closed.
- Identical project names, repeated script basenames, and separate sessions of the same project must not collapse into one target identity.
- A script is renamed, removed, closed, or replaced during inspection: report the detected identity/state change without combining evidence from different documents.
- Source differences consist only of whitespace or line endings: retain the source text as observed and report the difference, rather than normalizing it into agreement.
- An observable script has no independently attributable disk source: report D's limitation, not another authority's source or unrelated container contents.
- A buffer is edited and then made textually equal to disk: report the editor's observed dirty state, not a dirty state calculated from text equality.
- A loaded Script or buffer is known to lag another authority: report its actual observed value and the evidence of lag. Disagreement alone cannot establish which authority is stale or which source the human intends to keep.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The caller MUST be able to designate one intended project, live editor session, and GDScript document. Results MUST identify the resolved project, session, and script/path sufficiently to distinguish same-named projects/scripts and separate sessions of the same project.
- **FR-002**: The system MUST resolve the intended target unambiguously before reading or returning script source. Ambiguous selection MUST produce a structured refusal and the missing selection information, without guessing or returning source from candidate targets.
- **FR-003**: Observations MUST remain bound to the selected editor-session lifetime and document identity. A restarted editor, different project, or replacement document MUST NOT silently inherit an earlier target or snapshot.
- **FR-004**: The result MUST distinguish open, confirmed not-open, and unknown document-open state. It MUST distinguish a valid closed script from a missing script, an invalid/unsupported target, and an open script with an unavailable buffer. Observable GDScript with syntax errors remains an observation subject; parsing or project diagnostics are not required.
- **FR-005**: The system MUST independently observe D from disk, R from the already-loaded Script, and B from the live editor buffer whenever each is applicable and accessible within the declared observation limits. A source exceeding its own limit remains applicable but MUST be reported unavailable with a structured limit reason; other independently observable sources and their already-gathered evidence MUST be preserved. Returned source MUST preserve the text exposed by its authority, including empty values and source differences; one authority MUST NOT serve as a substitute for another.
- **FR-006**: Each of D, R, B, and dirty state MUST explicitly indicate whether it was observed, unavailable, or not applicable, using the meanings in Outcome Interpretation and giving a reason for unavailable/not-applicable state. Missing or unreadable values MUST NOT be supplied as observed values. Evidence detectably invalidated after collection MAY be retained only as explicitly invalidated evidence, never as current observed state. Unrelated readable facts MUST remain distinguishable from unavailable facts.
- **FR-007**: Dirty state MUST describe unsaved source work in the observed script/document, based on independently observed editor state safely attributable to that document. Dirty means the editor reports unsaved source changes for that document; clean means it reports no such changes. If this document-specific state cannot be established, dirty state MUST be unknown and its observation unavailable with a reason. An editor-wide or project-wide unsaved-work indication without sufficient document attribution MUST NOT make the selected document dirty or clean by inference. For a confirmed closed document, document-buffer dirty state is not applicable. Source equality or inequality alone MUST NOT be used to claim dirty state or cleanliness.
- **FR-008**: The snapshot MUST report D/R, D/B, and R/B comparisons as equal, different, or unknown. Equality/difference requires both values to have been independently observed for this request. Overall agreement requires all three observed values to match; any known unequal pair establishes divergence; otherwise missing evidence leaves agreement unknown. Comparisons describe the observed values, not proof of a simultaneous or future editor state.
- **FR-009**: A complete observation MUST require observed identity, open state, all requested applicable source surfaces, and dirty state, with no detected invalidation of those observations. A complete divergent snapshot MUST remain a successful observation. Missing required observability MUST prevent a complete-success claim; neither dirty state nor divergence alone is an observation error or a mutation conflict.
- **FR-010**: For a confirmed closed script, the result MUST report not-open, preserve independently available D and already-loaded R, and mark B and document-buffer dirty state not applicable. An unloaded R MUST be explicit. A not-open result MUST NOT be presented as successful observation of a visible buffer or as evidence that the project has no unsaved work.
- **FR-011**: Observation MUST NOT write or alter source files, save, patch, synchronize, resolve conflicts, perform or register Undo/Redo, trigger reload/reparse/rescan, start runtime execution, or modify editing history. It MUST NOT open, close, select, or force-load a document to manufacture observability. Independent human/editor activity is not prohibited, but MUST NOT be attributed to the observer as a repair or synchronization step.
- **FR-012**: Each result MUST identify its observation interval and associate its values and limitations with the authority and target actually observed. Values from a previous request or ended session MUST NOT be passed off as current evidence. Source content belongs in the requested observation, not incidental diagnostic logs.
- **FR-013**: Detectable changes, invalidation, and known stale Resource/buffer state MUST be reported with their evidence. If stability during the interval cannot be established, it MUST be described as unknown rather than guaranteed. Reading an actually loaded but stale R is a valid observation of R; this differs from returning an old cached observation. Detected invalidation during collection MUST prevent a complete current-snapshot claim.
- **FR-014**: Outcomes MUST distinguish complete observation, limited observation, confirmed not-open, ambiguous target, missing/invalid target, unavailable editor, disconnected editor, timeout, unsupported observation, and denied access wherever applicable. They MUST identify the failed activity and affected target/surfaces with actionable, machine-readable reasons. A surface-level failure MUST NOT erase unrelated valid observations, and partial evidence MUST NOT conceal an overall refusal or disconnection.
- **FR-015**: Every observation attempt MUST return an observation or a structured non-success outcome within five seconds in the controlled local acceptance environment, including an unresponsive-editor case. Deadline expiry MUST NOT imply that unavailable state was observed or that a disconnected editor is known to be connected.
- **FR-016**: This feature MUST remain local-only and limited to the explicitly selected project's script observations. It MUST NOT enable outside-project access to user project/source content, arbitrary source files, or unrelated filesystem data. The only permitted outside-project access is to minimal owner-private operational/session metadata, confined to the tool's own private state location and used solely for the discovery, identification, and routing required to connect to the intended local Godot editor session. This metadata MUST NOT contain project/source content; its access MUST remain local-only, protected by the existing authentication and privacy requirements, and MUST NOT grant arbitrary filesystem access. Remote access, arbitrary evaluation/execution, and telemetry remain prohibited. Source MUST NOT be retained in incidental logs or exposed from unselected projects; the observation result itself necessarily contains the requested source.
- **FR-017**: Observation semantics MUST be reusable without MCP and MUST respect the existing separation between agent-facing interaction, shared automation semantics, and Godot-owned state. This feature MUST NOT introduce an MCP tool surface, a second safety model, or gameplay authority.
- **FR-018**: Supported Godot versions MUST be exact and backed by appropriate CI and real-editor acceptance evidence before support is claimed. Planning MUST establish that clean-open and dirty-buffer scenarios can actually be observed on the intended versions; treating every buffer or dirty state as unavailable does not satisfy this feature. Verified guarantees and limitations MUST be recorded without claiming untested versions.

### Outcome Interpretation

The outcome names above describe user-visible meaning, not a prescribed serialization or interface.

The following terms distinguish evidence, applicability, uncertainty, and failure; they are not interchangeable overall outcomes or a prescribed protocol schema:

- **Observed**: A value was obtained independently from its own authority during this request, safely attributed to the intended target, and not detectably invalidated for the result. Equal values on different authorities still require separate observations.
- **Unavailable**: An applicable requested observation could not be obtained or safely attributed, so its value is unknown and the reason is explicit. This includes an R with no currently loaded Script or a source exceeding its own declared observation limit. Such a limit does not make the source not applicable. A surface being unavailable does not by itself mean the selected editor is unavailable.
- **Not applicable**: A confirmed document state makes the particular surface/state inapplicable, as with B and document-buffer dirty state for a confirmed closed script. This must not stand in for a failed read or uncertainty about whether a buffer exists.
- **Unknown**: Evidence does not establish the particular fact, such as open state, dirty state, or whether an authority is stale. Unknown is not a source value, proof of absence, or evidence of clean state. Unknown staleness does not make an independently read source unavailable.
- **Stale**: Observable evidence establishes that an authority has not incorporated an identified change; the result states that evidence and what it applies to. Divergence alone cannot establish staleness. Without such evidence, staleness remains unknown; no staleness-detection guarantee is required. A fresh observation of stale source is different from a stale or invalidated observation.
- **Error**: A requested observation could not be performed or safely attributed, with the affected target/surface, failed activity, and reason identified. Dirty state, source divergence, or known stale source content is not an observation error by itself. An error affecting one observation does not erase independently obtained evidence from another.

- **Complete observation**: The applicable open-document facts were independently observed. Dirty or divergent state is valid information, not failure. No permission to edit is implied.
- **Limited observation**: Some facts were read but required state is unavailable, unknown, or detectably invalidated. Each useful fact and each limitation remains explicit; missing evidence never implies agreement or cleanliness.
- **Not-open**: The valid script is confirmed closed. This is different from failing to inspect an open buffer; D and already-loaded R can still be reported independently.
- **Refused or interrupted observation**: Target ambiguity, missing/invalid targets, access restrictions, unavailable/disconnected editors, or timeouts have distinct reasons. Any retained evidence is labeled with when and where it was obtained, not presented as a complete live snapshot.

A per-surface limit MUST affect only that source: other independently observable facts, including evidence already gathered for them, MUST remain available under the existing attribution and safety rules. An otherwise valid open-document observation with an over-limit source is limited, not complete; a confirmed closed document retains the not-open outcome with explicit D/R limitations. Unknown open state remains limited, never inferred closed. Source MUST NOT be truncated or represented as observed empty text to satisfy a limit.

An operation-wide unsupported/refused outcome is permitted only when the request as a whole cannot be performed safely or meaningfully, not merely because one or more D/R/B sources exceed their individual limits. Existing target, access-denial, invalidation, disconnection, and timeout precedence remains unchanged; a source limit neither erases unrelated valid evidence nor overrides a separate refusal/interruption.

A snapshot describes observations over an interval, not a promise that all authorities were frozen at one instant. It cannot establish that a later mutation may proceed without fresh safety checks. Detected source divergence is not itself evidence that one particular authority is stale.

### Scope and Governance Alignment

This is the first **observation-only subset of Phase 1**, not completion of Phase 1. In scope are minimal safe target identification, one-script observation, independent D/R/B and dirty-state evidence, comparisons, and explicit limitations/errors.

Excluded are script editing/patching, file writes, Save, conflict resolution, Undo/Redo, revision-based mutation, script opening/closing as a capability, scene/resource authoring, runtime control, debugger functionality, LSP/project diagnostics, MCP exposure, multi-agent concurrency, broad multi-editor orchestration, and package/distribution work. Using multiple editor sessions to prove target refusal does not authorize orchestration capabilities.

The governing references are the [constitution](../../.specify/memory/constitution.md), [working agreement](../../AGENTS.md), [Phase 1 roadmap](../../ROADMAP.md#phase-1--live-editor-script-coherence), and [architecture](../../ARCHITECTURE.md). This specification preserves independent D/R/B authority, protection of human work, explicit unavailable state, safe routing, least privilege, and protocol-independent semantics. The architecture's convergence requirement concerns successful edits; a successful read need not find equal sources.

Real-Godot evidence is required for claims about live loaded source, visible buffers, dirty state, and editor sessions. Disk-only checks, headless runtime, and mocks cannot prove those observations. Pure classification/comparison behavior and boundary failures also require deterministic, isolated verification at the appropriate level. Acceptance preparation may include human edits, closing documents, changing fixture files, and stopping an editor; these are test actions, not observation capabilities. Evidence must identify the independently checked authorities and exact editor version, use explicit deadlines and event-based synchronization, and retain regression cases for discovered failures.

The mutation-specific A/B/D/E gates, Save/reparse/rescan/runtime durability of an applied mutation, and gate C's actual Undo/Redo transitions are inapplicable to this feature because it applies no mutation and claims no Undo/Redo support. That is a scope justification, not a waiver for later work or permission to omit live-editor observation tests. No Phase 1 mutation, durability, or exit guarantee is earned by this specification. Planning/review must retain applicable constitutional security, compatibility, and tooling/export-isolation obligations without adding distribution or runtime capabilities here.

### Key Entities *(include if feature involves data)*

- **Observation target**: The intended project, live editor-session identity, and script identity/path. Names alone are insufficient if they match multiple targets.
- **Script document state**: Whether the identified GDScript is open, closed, missing/invalid, or not classifiable with available evidence. Existence on disk and presence in the editor are separate facts.
- **Surface observation**: One authority's source text, availability, observation timing, target attribution, and any reason or evidence of unavailability, invalidation, or known stale state.
- **Dirty-state observation**: The target document's independently reported unsaved-work state, or explicit uncertainty/inapplicability. It is not a source comparison.
- **Observation snapshot**: The target, observation interval, document state, independent D/R/B and dirty observations, pairwise comparisons, overall agreement/divergence, completeness, and limitations.
- **Observation outcome**: The structured result of an attempt, including refusal/interruption reasons and any usable partial evidence without a false complete-success claim.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In 100% of clean-open and unsaved-buffer acceptance cases, the caller receives each applicable source exactly as independently checked at its authority, the correct target identity, and the editor's actual dirty state. An unsaved human edit remains intact after observation.
- **SC-002**: In 100% of established divergence cases with full observability, the caller receives a complete observation and the correct comparisons; no disagreement is hidden or misclassified as an observation failure. Missing observability never produces a false equality or clean-state claim.
- **SC-003**: All seven minimum scenario classes—clean open script, dirty unsaved buffer, D/R/B divergence, closed script, unavailable surface, disconnected editor, and ambiguous target—produce the specified distinguishable outcomes. The additional missing/invalid, unknown-open-state, unavailable-editor, timeout, and session-replacement cases also meet their acceptance scenarios, with zero guessed targets or stale snapshots presented as current.
- **SC-004**: Every controlled local acceptance request, including the unresponsive-editor case, produces a result or explicit non-success outcome within five seconds; callers never wait indefinitely to learn that state is unavailable.
- **SC-005**: Across 20 sequential observations spanning clean, human-dirtied, and closed-document fixtures, inspection causes zero source writes, saves, open/close/selection changes, or editing-history changes. Every request reflects its independently checked state or explicitly reports why it could not, rather than reusing a previous snapshot as fresh evidence.
- **SC-006**: For every acceptance scenario, a reviewer using the result alone can identify the selected target or refusal reason, answer what is known about D, R, B, and unsaved work, and identify every unavailable or uncertain fact without guessing one authority from another. Reported limits never imply authorization for an edit.

## Assumptions

- A request concerns one GDScript document in one intended local project/editor session. Batch inspection and broad editor orchestration are not required. Enough target identity can be supplied or obtained to choose one session; otherwise refusal is the correct behavior.
- Standard open-document observation requests cover D, R, B, open state, and dirty state. Selective surface-query capabilities are not required. For a closed document, no visible buffer is fabricated to satisfy that request.
- The human may continue editing. Observation does not freeze the editor or provide a revision-based authorization for later mutation; it reports the interval and any detectable changes or uncertainty.
- Five seconds is the user-facing observation deadline for the controlled local acceptance environment, not a throughput, project-size, or concurrent-load claim. The implementation plan must define reproducible fixtures and timing evidence.
- Planning must select and verify exact Godot versions and establish access to independent live-state evidence. No compatibility version is selected or claimed by this specification; missing observability must remain explicit and cannot replace the required positive clean/dirty scenarios.
- Implementation mechanisms remain for planning: dependencies, transport, ports, bridge protocol, module/addon layout, and implementation APIs are intentionally unselected. The existing architectural ownership boundaries remain authoritative.
- Generation of this specification creates documentation and workflow metadata only. Feature approval, technical planning, task derivation, and implementation remain separate steps under the working agreement's one-implementation-task-per-PR policy.
