# Feature Specification: Safely Discover Project GDScripts

**Feature Branch**: `spec/004-discover-project-gdscript`

**Created**: 2026-09-30

**Status**: Draft — quality-validated and ready for planning; implementation is not authorized.

**Input**: Generated feature description supplied to `/speckit.specify`:

> Safely discover existing standalone GDScripts in one explicitly selected, authorized local Godot project and live editor session. A coding-agent caller can already observe, open and edit a known script, but currently needs someone to supply its path. Add a read-only project-script listing that returns exact, distinguishable project-relative targets suitable for those existing capabilities, including closed scripts and same-named scripts in different folders. Return script locations, not source bodies, symbols or inferred editing permission.

> Establish the selected project/session before exposing its script inventory. Preserve existing authentication, project confinement, source privacy, local-only operation and protocol-independent ownership. Refuse ambiguous, unavailable, replaced or unauthorized targets without selecting by editor focus or exposing candidate inventories. Do not follow outside-project redirects or treat unreadable, unsupported, stale or interrupted enumeration as proof that no scripts exist. Report the request's scope, observation interval, completeness and limitations; an empty complete result must mean a successfully examined empty scope, not a failed search. Results are observations, not durable file identities or authorization for a later operation.

> Discovery must not load or open scripts, read their source for a content search, parse or execute project code, trigger a rescan, save, edit, close or select documents, change history, or reconcile dirty state. Preserve unsaved human work and already-loaded/editor state even for dirty, divergent, non-selected and editor-executing scripts. A caller chooses a returned path and makes a separate fresh observation/open/edit request under the existing safety rules. Changed or removed paths and replaced sessions must not inherit stale discovery authority.

> Prove useful complete listings, unambiguous composition with existing operations, safe target/access refusals, truthful bounded partial and interrupted results, fresh repeated discovery and real-editor non-interference. This advances Phase 1's pending script-discovery requirement without claiming that Phase 1 is complete. Exclude source-content search, symbols and language intelligence, persistent indexing, file creation/rename/delete, embedded scripts, independent close/Save/Undo/Redo controls, batch mutation, concurrent-agent orchestration, MCP, broader runtime/debugger tooling and distribution expansion. Leave interfaces, enumeration mechanisms, limits, internal design and compatibility choices to planning and evidence; introduce no speculative service, framework, dependency, approval layer or operational prerequisite.

## Clarifications

### Session 2026-09-30

- Q: Should discovery exclude scripts hidden or ignored by Godot’s project file view, even when their files exist inside the project? → A: Yes. Follow Godot’s project-file visibility: exclude folders Godot hides or ignores, including `.gdignore` exclusions. Version-control ignore rules and production-export filters do not independently exclude otherwise visible scripts.

## User Scenarios & Testing *(mandatory)*

The primary caller is a coding-agent automation layer acting for a developer working in a live Godot Editor. The caller knows the intended project/editor session but need not know a script path. Discovery supplies locations for a subsequent explicit choice; it does not choose a script for the caller or perform the next operation.

A **discovery entry** identifies one existing standalone project GDScript by its exact project-relative path, associated with the selected project and editor-session lifetime. Different folders may contain the same basename. A listing does not establish whether a script is open, clean, parse-valid, supported for opening/editing or safe to mutate. Those facts remain the responsibility of the existing operations.

In the existing safety contract, **D** is source on disk, **R** is source in the loaded Godot Script, and **B** is source in the live editor buffer. Discovery returns none of these source values and does not need to infer their agreement or dirty state to list a path; its obligation is to leave them undisturbed.

A **complete listing** establishes coverage of the declared script scope during this request with no known coverage gap or detected invalidation. A **limited listing** retains safely attributable entries but makes incomplete coverage or uncertain freshness explicit. Neither is an atomic snapshot of a project frozen against human activity. The stories are independently verifiable outcomes; confinement, non-interference and truthful limitations are inseparable from the first exposed discovery path.

### User Story 1 - Find an Exact Script Without Knowing Its Path (Priority: P1)

As a coding-agent caller, I want to list the project's standalone scripts so I can choose the exact script to inspect or open without asking the developer to locate it manually.

**Why this priority**: The completed workflow starts from a known script path. A usable project-scoped listing closes that gap without adding content search or a language index.

**Independent Test**: Prepare a known inventory of project scripts in multiple folders, with repeated basenames, open and closed files, empty source and syntax-invalid source. Compare discovery with that independently established inventory, then pass a selected entry to the existing observation and opening capabilities.

**Acceptance Scenarios**:

1. **Given** an explicitly selected authorized live project/session with supported standalone scripts in ordinary nested folders and addon folders, **When** discovery is requested, **Then** the caller receives every in-scope script exactly once by its exact path, including closed and never-opened scripts and otherwise visible scripts excluded only by version-control ignore rules or production-export filters. The complete result identifies its scope, project/session and observation interval and contains no script source.
2. **Given** two scripts with the same basename in different folders, **When** discovery returns them and the caller selects one, **Then** their paths remain distinguishable and can be used with the existing explicit-target operations without guessing, lossy renaming or selecting by editor focus.
3. **Given** a project with no in-scope standalone scripts and successful enumeration, including a project whose only script files are in folders Godot hides or ignores, **When** discovery completes, **Then** it returns a complete empty listing. Non-GDScript files, embedded scripts, editor-generated internal state and Godot-hidden/ignored folders, including `.gdignore` exclusions, are not invented as entries.
4. **Given** an in-scope empty, syntax-invalid, read-only or editor-executing script, **When** discovery is requested, **Then** that script remains discoverable by location without source parsing, loading, execution or a claim that later opening/editing is supported. Source size or parse validity alone does not exclude its path.

---

### User Story 2 - Keep Discovery Inside the Intended Project (Priority: P1)

As a developer, I want inventory access to obey the same target and permission boundaries as existing script operations, so an agent cannot enumerate another project or escape the selected one merely by requesting a list.

**Why this priority**: Filenames and project structure are project information even when no source body is returned. Source-free session selection is not permission to disclose candidate inventories.

**Independent Test**: Use distinguishable projects and live sessions, including multiple sessions for one project. Exercise ambiguous, ended, replaced and denied targets, outside-project redirects and path changes; check both results and incidental diagnostics for unauthorized inventory disclosure.

**Acceptance Scenarios**:

1. **Given** insufficient identity to choose one live project/session, **When** discovery is requested, **Then** it refuses with actionable selection information and no candidate script inventory. Existing minimal session-selection feedback remains permitted; focused, first, newest or previously selected sessions are not substituted.
2. **Given** an exact selected session among distinguishable projects or same-project sessions, **When** discovery succeeds, **Then** all entries belong to that authenticated project/session. An ended session is not replaced by a new one, even if the project path is unchanged.
3. **Given** unauthorized access, an outside-project redirect, an unsafe project root or an entry whose confinement cannot be established, **When** discovery is attempted, **Then** it does not traverse or disclose the unauthorized target. A request-wide target/access failure refuses; a confined local coverage gap may retain other authorized entries only as a limited result with a safe reason.
4. **Given** project/session identity or a path's confinement changes during collection, **When** the result is returned, **Then** detected invalidation prevents a complete current listing and prevents attribution to a replacement. Evidence that can no longer be safely associated with the authorized scope is withheld; independently safe earlier evidence, if retained, is explicitly historical or invalidated.

---

### User Story 3 - Distinguish an Empty Project From an Incomplete Search (Priority: P1)

As a coding-agent caller, I want to know what discovery actually covered and why it stopped, so I do not conclude that a needed script is absent from an incomplete or stale result.

**Why this priority**: Bounded discovery is useful only if limits, unreadable areas and loss of the editor cannot masquerade as a complete inventory.

**Independent Test**: Exercise unreadable in-scope folders, unsupported path representations, controlled result limits, changing inventories, an unavailable or disconnected editor and an unresponsive request. Compare returned coverage, retained entries and reasons with independent witnesses.

**Acceptance Scenarios**:

1. **Given** an in-scope directory cannot be enumerated or a candidate path cannot be represented safely, **When** other entries can still be collected, **Then** those entries remain usable with a limited outcome identifying the coverage problem. Zero retained entries is still limited, not a complete empty listing; no path is truncated or rewritten into another target.
2. **Given** enumeration exceeds a documented work or result limit, **When** the request terminates, **Then** it reports the limit and incomplete coverage, retaining permitted entries without claiming that omitted scripts do not exist. The caller can distinguish this from successful exhaustion of the scope without inferring completeness from the entry count.
3. **Given** scripts are created, renamed or removed between requests and those changes are observable, **When** discovery is requested again, **Then** it reflects the new inventory or explicitly reports unavailable/stale freshness rather than presenting an earlier list as freshly complete. Detectable changes during collection invalidate affected entries or coverage claims.
4. **Given** a missing editor, disconnection, cancellation or an unresponsive editor, **When** discovery terminates, **Then** the result distinguishes the actual failure and retains only safely attributable evidence with its validity stated. Every controlled local request returns within five seconds; no case falls back to unbound disk-only discovery or automatically starts another editor.
5. **Given** a discovered path is removed, renamed or replaced before the caller acts, **When** the caller uses it in a later operation, **Then** that operation resolves current state under its existing rules. Discovery is not a revision basis or permission token, cannot select a replacement session and cannot bypass fresh observation, identity, dirty-state or revision checks.

---

### User Story 4 - Discover Without Disturbing Live Work (Priority: P1)

As a developer, I want discovery to leave my editor and unsaved work alone while enabling the agent to continue through the existing safe workflow.

**Why this priority**: A read-only inventory must not become a hidden load, rescan, Save or synchronization path, or weaken the guarantees already earned by Features 001–003.

**Independent Test**: In a real editor, establish clean, dirty, equal-text-but-dirty, divergent and non-selected documents with known native history and selection. Repeatedly discover while independently witnessing their state. Separately exercise discovery followed by existing observation, opening and eligible editing or dirty refusal.

**Acceptance Scenarios**:

1. **Given** live human work and known D/R/B, dirty state, selection and history, **When** discovery runs, **Then** it causes no source write, source synchronization, script load, open/close/selection change, Save, reload/reparse/rescan, execution or history change. Already-loaded state and unloaded-script status are preserved; unrelated documents remain untouched. Human/editor activity independent of the request is distinguished from discovery-caused effects.
2. **Given** at least 20 discovery requests with deliberate between-request inventory changes and human buffer edits, **When** the sequence runs, **Then** each request reports its own observed inventory or explicit limitations, preserves unsaved work and usable history, and introduces no delayed editor or source effects after returning.
3. **Given** a discovered closed script supported by existing opening/editing and a separate discovered dirty open script, **When** the caller explicitly chooses each and uses the existing operations, **Then** the supported clean workflow succeeds without manual path lookup or tab preparation, while an attempted dirty edit still refuses and preserves the human work. Observation remains read-only; discovery neither opens implicitly nor grants edit eligibility.
4. **Given** authorized, ambiguous, denied and interrupted discovery with distinguishable project-path and source sentinels, **When** results, incidental logs, private routing metadata and production exports are inspected, **Then** only authorized requested results contain script inventory, no source bodies are returned by discovery, no inventory is added to incidental logs or routing metadata, and discovery tooling does not become exported gameplay behavior or a gameplay dependency.

---

### Edge Cases

- Duplicate basenames, spaces, Unicode and case-distinct names must retain exact supported path identity; unsupported representations are coverage limitations, never lossy aliases. Distinct valid paths remain distinct even if they refer to the same underlying file.
- A script may be present on disk while its open buffer is dirty, divergent or missing an observable surface. Discovery may return the path without inspecting or inferring D/R/B agreement, open state or cleanliness.
- An open buffer whose file has been deleted, an unsaved document with no file and an embedded script are not existing standalone file entries. Their absence from discovery does not mean the editor has no unsaved work; known-document observation keeps its existing behavior.
- A directory or special file with a script-like name is not a standalone script file. A symlink or path alias must not bypass the existing project-confinement rules or reveal its outside-project destination.
- Godot project-file exclusions and unsupported/denied enumeration are different: folders Godot hides or ignores, including `.gdignore` exclusions, are outside scope; an unreadable area within scope prevents completeness. Version-control ignore rules and production-export filters do not independently remove otherwise visible scripts. An undocumented exclusion must not conceal a failed read.
- A filesystem or editor inventory may be changing or stale. Report what is known about the collection interval and freshness; do not force a rescan, freeze human work or invent an instantaneous snapshot to obtain a complete label.
- A script discoverable by path may exceed source limits or require an unsupported opening context. Discovery success makes no source-availability, parse, execution-safety or mutation-support claim.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The caller MUST be able to discover existing standalone project GDScript paths without supplying a script path, within one explicitly selected authorized local project/editor-session lifetime. Discovery MUST be a read-only operation distinct from observation, opening and editing; it MUST NOT automatically choose or act on a returned script.
- **FR-002**: The declared discovery scope MUST follow the selected editor's ordinary project-file visibility rules for standalone `.gd` files, including visible nested project folders, addon folders, open and closed files. Non-GDScript files, embedded scripts, unsaved documents without files, editor-generated internal state and folders Godot hides or ignores, including `.gdignore` exclusions, are outside that scope. Version-control ignore rules and production-export filters MUST NOT independently exclude otherwise visible scripts. The applied Godot visibility/exclusion rules MUST be documented and identified in the result's scope description; exclusions MUST NOT be invented to hide access failures, supported script paths, parse errors or dirty state. These rules define scope, not proof that an editor inventory is current; known stale or incomplete enumeration remains subject to FR-007–FR-009.
- **FR-003**: Each entry MUST carry an exact, unambiguous project-relative script locator usable with the existing explicit-target operations, bound to the result's selected project/session and observation interval. A stable complete listing MUST contain each in-scope path once, without conflating same-named scripts, truncating paths or silently rewriting unsupported representations. Discovery MUST NOT require source content, symbols, source hashes, open/dirty state or parse results in its entries.
- **FR-004**: Existing authenticated selection and lifetime-bound routing MUST precede script-inventory disclosure. Ambiguous, unauthorized, unavailable, ended or replaced targets MUST NOT produce a guessed inventory, a candidate inventory, automatic editor startup or a substitute session. Existing source-free session-selection feedback remains permitted.
- **FR-005**: Enumeration and returned entries MUST remain within the selected project's existing access and confinement boundaries. Outside-project redirects, changed root identity and unsupported path relationships MUST NOT grant traversal or disclosure. Request-wide selection/access failure MUST remain a refusal; safe partial evidence MUST NOT conceal that refusal or expose unauthorized paths or redirect destinations.
- **FR-006**: Discovery MUST NOT read script source to search or classify contents, load scripts, open/close/select documents, write source, Save, apply or register history actions, synchronize, reload/reparse/rescan, execute project code or resolve conflicts. It MUST preserve unsaved work, loaded state, dirty state, selection and existing native history. Syntax invalidity, empty source, read-only source and potential load-time effects MUST NOT alone prevent safe path discovery.
- **FR-007**: Results MUST identify the requested/resolved project and session or why resolution failed; the declared scope and exclusions; the collection interval; complete versus incomplete coverage; known freshness/invalidation; and actionable reasons for limitations. A complete listing MUST require actual coverage of that scope with no known gap or detected invalidation. An empty complete result MUST mean successful enumeration of a scope containing no eligible entries, never failed, unsupported, denied, timed-out or stale enumeration.
- **FR-008**: An inaccessible in-scope area, unsupported path representation, result/work limit or other local coverage gap MUST prevent a complete claim. Independently safe, authorized entries SHOULD remain available with a limited outcome and the affected scope/reason; inability to retain them safely MUST be explained. No entry or list may be silently truncated and presented as exhaustive. Limits and their caller-visible behavior MUST be documented without requiring arbitrary-project-size support.
- **FR-009**: Each request MUST obtain evidence for its own selected session and interval. Previously collected inventory MUST NOT be presented as newly observed. Known stale enumeration or detected changes to inventory, project/session identity or path confinement MUST invalidate the affected currency/coverage claims. Results MUST distinguish known change from unknown stability; they MUST NOT promise an atomic filesystem snapshot or force editor reconciliation.
- **FR-010**: Outcomes MUST distinguish complete listing, limited listing, target/access refusal and interruption. Reasons MUST distinguish ambiguous target, invalid/unsupported project or path, unavailable/disconnected/replaced editor, denied access, confinement failure, unreadable area, known stale/changed inventory, unsupported representation, limit reached, timeout and cancellation wherever applicable. Safely retained partial entries MUST keep their attribution and validity, never erase an overall refusal/interruption or imply completeness.
- **FR-011**: Every controlled local discovery request MUST return a terminal result within five seconds, including an unresponsive-editor or blocked-enumeration case. Cancellation, timeout or disconnection MUST NOT fabricate an empty inventory, change targets or cause fallback to a disconnected disk-only mode. Discovery MUST have no delayed source/editor mutation and MUST NOT automatically retry an interrupted request to conceal failure.
- **FR-012**: A discovery result MUST NOT act as a source revision, persistent document identity, permission token or guarantee of later existence, coherence or edit eligibility. Subsequent observation, opening and editing MUST independently resolve their actual target and retain all existing identity, revision, dirty-state, effect-safety and verification checks. A replacement file at the same path is not the old observed document; a later operation interprets current state under its own unchanged contract.
- **FR-013**: Discovery MUST return locations rather than source bodies or content-derived intelligence. Project inventory MUST appear only in the authorized requested result, not new incidental logs, telemetry or private session metadata. Diagnostics MUST not disclose unselected inventories, outside-project destinations, source or credentials; pre-existing minimal routing metadata remains governed by its existing contract.
- **FR-014**: The capability MUST preserve local-only operation, existing authentication and permission boundaries, protocol-independent semantics and the separation of caller interaction, common automation rules and Godot integration. It MUST NOT introduce MCP, an arbitrary-filesystem browser, remote/network capability, force access, evaluation or an approval bypass.
- **FR-015**: The completed observation, opening and editing capabilities MUST retain their existing contracts, deadlines and safety behavior. A caller MUST be able to choose a discovered eligible script and complete the existing observe/open/edit workflow, while dirty, stale, unavailable and unsupported states still produce their existing truthful results. Neither a complete listing nor absence from a listing MUST change known-document observation semantics.
- **FR-016**: Real-editor acceptance MUST demonstrate positive complete discovery, exact target selection, safe refusals, bounded partial/interrupted results, repeated-use freshness and non-interference with clean/dirty/divergent/non-selected work and native history. Pure decision logic and changed boundaries MUST receive appropriate deterministic, isolated coverage. Existing applicable coherence/history/durability regressions MUST remain satisfied; refusal or limited results for every normal project do not satisfy the positive capability.
- **FR-017**: Development/export isolation MUST remain explicit and verified for discovery: no active discovery tooling, private state or required gameplay dependency may unintentionally ship in production exports. Exact supported Godot versions/environments and verified limitations MUST be documented from applicable CI and real-editor evidence; previous feature evidence does not establish untested discovery support.

### Outcome Interpretation

These meanings describe observable results, not a selected schema, command syntax or new outcome framework.

- **Complete listing**: The authorized selected scope was covered for this request, with no known gap or detected invalidation; entries may legitimately be empty. Completion describes discovery coverage, not clean editor state, a frozen project or readiness to edit.
- **Limited listing**: Some or all safe entries are available, but scope coverage or freshness is incomplete or unavailable. Limitations remain explicit even when no entries could be retained.
- **Refused**: The request cannot safely target or access the intended scope. No candidate or unauthorized inventory is disclosed; permitted partial evidence cannot turn the refusal into success.
- **Interrupted**: Timeout, cancellation or loss of the selected editor prevented completion. Any retained entries identify their observed interval and limitations, not a complete current inventory.

Unknown stability is not evidence that the project is unchanged. Missing coverage is not proof that a script is absent. The caller may make a new explicit discovery request after a limitation is resolved, but discovery never repairs the project, automatically replays another operation or changes its permission boundary.

### Scope and Governance Alignment

This is the script-discovery slice of **Phase 1 — Live-editor script coherence**. The completed foundations are [Feature 001 observation](../001-observe-gdscript-state/spec.md) and its [tasks](../001-observe-gdscript-state/tasks.md), [Feature 002 editing](../002-edit-open-gdscript/spec.md) and its [tasks](../002-edit-open-gdscript/tasks.md), and [Feature 003 opening](../003-open-project-gdscript/spec.md), [tasks](../003-open-project-gdscript/tasks.md) and [cumulative acceptance](../003-open-project-gdscript/quickstart.md#10-t003-cumulative-acceptance-2026-09-30). Their current task and acceptance records establish completion, not older draft-generation headers or PR delivery states.

In scope are one selected project's read-only script-location listing, exact-path handoff to existing operations, inseparable confinement/privacy, honest coverage/freshness and bounded non-interfering outcomes. A whole-project list within documented limits is sufficient; search expressions, ranking, content matching and multiple discovery modes are not required.

Out of scope are source-content search, symbols, diagnostics or language indexing; persistent/background indexing; project or editor orchestration; file creation/rename/delete; embedded scripts or unsaved-document catalogs; independent close/Save/Undo/Redo controls; conflict resolution/force writes; batch mutation and concurrent-agent orchestration; broad core generalization; MCP; scene/resource/project-setting authoring; runtime/debugger tools; and distribution/platform expansion. These boundaries do not weaken the safety of existing operations. Phase 1 remains in progress after this discovery feature: separate lifecycle controls and remaining phase assessment are not delivered by a listing.

The [constitution](../../.specify/memory/constitution.md), [working agreement](../../AGENTS.md), [roadmap](../../ROADMAP.md#phase-1--live-editor-script-coherence) and [architecture](../../ARCHITECTURE.md) govern the work. Principles I–IV preserve existing independent D/R/B, human-work and verified-mutation semantics; discovery supplies no substitute authority. V and X require confined authenticated access and truthful limited outcomes. VII–IX require protocol independence, tooling isolation and a small composable surface. XI–XII require independent implementation and evidence-backed quality. Security-sensitive planning/review must explicitly record constitutional compliance.

**Principle XIII boundary:** The present gap is that callers cannot obtain unknown script paths through the kit, although they can already observe, open and edit known targets. Requiring manual path lookup leaves that roadmap capability absent; listing only open tabs misses closed targets. The simplest credible scope is a bounded read-only inventory using the existing target, access and safety foundation. Its necessary cost is defining coverage, exact target handoff and non-interference evidence. No index database, watcher service, generic query engine, new dependency, parallel security model, approval layer, CI topology or operational prerequisite is required. Planning must justify any additional mechanism against a concrete current need, simpler alternatives, existing mechanism limits and ongoing cost.

Discovery makes no source/lifecycle mutation or Undo/Redo claim. It therefore does not independently introduce mutation A–E or Save/reparse/rescan/runtime durability outcomes; inventing a reversible list operation would add no value. This is an applicability statement, not a waiver of existing mutation coverage. Real-editor witnesses must prove non-interference and the composed clean and dirty-refusal workflows. Applicable existing A–E/history/durability guarantees remain intact, with affected versus cumulative verification chosen under the repository's real-editor validation rules. Disk-only listing tests cannot prove preservation of buffers or history. Human fixture preparation is not a discovery capability.

### Key Entities *(include if feature involves data)*

- **Discovery target**: The explicitly selected authorized local project and live editor-session lifetime; no script path is needed to request its inventory.
- **Discovery scope**: The eligible standalone project-script domain under Godot's project-file visibility rules, with documented exclusions and supported representation/coverage limits; version-control ignore rules and production-export filters do not independently determine membership.
- **Discovery entry**: One exact project-relative path with selected-target and observation attribution, not source content or edit authorization.
- **Coverage evidence**: The collection interval, enumerated scope, known gaps, freshness/invalidation and safe reasons for unavailable facts.
- **Discovery outcome**: Complete/limited/refused/interrupted status, safely retained entries, target attribution and actionable next steps without a completeness or permission inference.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In every supported stable-project acceptance case, discovery returns exactly the independently established in-scope path set with no duplicate, missing or invented entries. At least one case contains 100 scripts across at least ten folders, including addon and never-opened scripts and repeated basenames. Visibility cases exclude scripts in Godot-hidden and `.gdignore` folders while retaining otherwise visible scripts excluded only by version-control ignore rules or production-export filters. A separate empty-project case, including one containing only Godot-hidden/ignored scripts, returns a genuinely complete empty result.
- **SC-002**: Every ambiguous, replaced, denied or escaping-target case returns zero unauthorized/candidate script entries and selects no substitute project/session. Distinct same-project sessions and same-basename scripts remain correctly attributed in all positive and refusal cases.
- **SC-003**: Every unreadable-area, unsupported-representation, stale/change, limit and interruption case reports its actual coverage and reason, with zero false complete or empty-project claims and zero lossy path substitutions. Every controlled request, including blocked/unresponsive cases, terminates within five seconds.
- **SC-004**: Across at least 20 real-editor discoveries with human edits and between-request create/rename/remove changes, results reflect newly observed state or explicit limitations. Independent witnesses establish zero discovery-caused source, load, document-selection, dirty-state or history changes, and no unsaved work is lost.
- **SC-005**: A caller starting without a script path can choose a discovered closed eligible script and complete the existing observation/open/edit workflow without manual path lookup, tab preparation or reconciliation. A discovered dirty target still produces the existing safe edit refusal with the unsaved text and history intact. Existing applicable coherence and durability guarantees remain satisfied.
- **SC-006**: For every acceptance case, a reviewer using only the result can identify the target or refusal, the scope, whether coverage is complete, which entries remain usable and why another observation/action may be needed. Privacy checks show zero source bodies in discovery and zero new inventory disclosure in incidental logs/private routing metadata; production-export checks show no active discovery tooling or gameplay dependency.

## Assumptions

- Features 001–003 are complete according to the current [project status](../../PROJECT_STATUS.md) and their task/acceptance records. Their implementation is reused rather than reopened. Normal prerequisite PR sequencing still governs any later dependent implementation; generating this specification does not start it.
- The caller can provide enough existing project/session identity for exact authorized selection. Expanding session discovery or starting editors is not part of this feature.
- The useful minimum is discovery of existing standalone files within Godot's project-file visibility, not every live document or arbitrary filesystem object. Godot-hidden/ignored folders are outside scope; inaccessible in-scope areas require explicit incomplete coverage. Planning must document and reproduce the supported editor's visibility rules, not choose a different exclusion policy or treat a stale editor inventory as current.
- Path discovery is not a claim about source readability, editability, parse validity, open state or dirty state. Existing observation/open/edit requests remain the only authority for their respective current behavior.
- Five seconds follows the existing read-only observation deadline in the controlled local acceptance environment. The 100-script/ten-folder case establishes a useful minimum supported inventory, not arbitrary project scale or concurrent throughput. Planning chooses documented bounds without weakening that positive path.
- Human edits and filesystem changes remain possible. The feature must report detected invalidation and preserve attribution, but introduces no cross-editor leases, filesystem snapshot service or exclusion of arbitrary non-cooperating writers.
- Interfaces, ordering, enumeration strategy, exclusion representation, work/result limits, internal layout, dependency choices and exact compatibility/evidence setup remain planning decisions under the existing architecture and complexity gate. No new stack or support matrix is selected here.
- This draft and its requirements-quality checklist do not approve implementation, satisfy real-editor acceptance or complete Phase 1. Clarification if needed, planning, task derivation, granularity review, analysis, implementation and acceptance remain separate authorized work.
