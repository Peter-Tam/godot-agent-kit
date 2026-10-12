# Feature Specification: Create and Delete Project GDScripts Through the Trusted Workflow

**Feature Branch**: `spec/009-create-delete-gdscript`

**Created**: 2026-10-12

**Status**: Draft — clarification and [Phase 0/1 planning](plan.md) complete. Four accepted answers resolve the deletion basis, create/delete lifecycle/Resource postconditions and additive Schema 2 compatibility. Task derivation, analysis and product implementation have not started.

**Input**: Generated WHAT/WHY description supplied to `/speckit.specify`:

> Let coding agents create and delete supported project GDScript files through the existing trusted script workflow. Creation takes an explicit confined res:// path and complete intended source, validates that source, never overwrites an occupied target, and independently verifies the result without opening, closing or saving editor documents to make the operation succeed. Deletion uses current trusted target/state information and stale protection, preserves human work and editor/Resource safety, and independently verifies disappearance before claiming success.
>
> Add create_script and delete_script alongside discover_scripts, read_script and edit_script. Preserve authenticated explicit project/session routing, confinement, identity/lifecycle checks, supported validation, independent observations, truthful known/partial/unknown effects, and existing timing/support/export/privacy boundaries. Creation is not editing an empty existing file; removal of source text is not file deletion. This is one feature, not a generic filesystem framework, a revision of Feature 008 history, or a start on Phase 4. Specify WHAT and WHY; record material questions without resolving them or starting a later design stage.

## Clarifications

### Session 2026-10-12

- Q: Should `delete_script` require the current revision returned by `read_script`, refusing deletion when that revision is unavailable? → A: Yes. Require the existing non-null `read_script` revision for the exact target, matched against freshly acquired trusted state. Missing, unavailable or stale revisions refuse; existing read eligibility and all deletion-specific safety and verification checks remain mandatory. No separate deletion-basis fallback is added.
- Q: Should `delete_script` support only closed, unloaded scripts, or also scripts that Godot still has loaded or open? → A: Support eligible clean open and closed scripts, including closed scripts with a clean loaded Resource or confirmed absent Resource. Removing the admitted target document and its Resource association is an explicit part of deletion, not a preliminary close/unload workaround. Dirty/conflicting state still refuses. Success independently verifies file absence, target-document absence and absence of a loaded Resource association with the deleted path; physical destruction of every retained Resource instance is not required or promised.
- Q: After `create_script` succeeds, may Godot retain a newly loaded Script Resource for the new file, provided its source is independently verified and no editor document is opened? → A: Yes. Creation may leave R independently confirmed absent or establish a new Resource association as a necessary creation effect, with its identity and exact intended source independently verified. The target document stays closed throughout; no forced load/unload, open/close, Save or other lifecycle manipulation may manufacture success.
- Q: Should adding `create_script` and `delete_script` retain public Schema 2, or move all five tools to public Schema 3? → A: Retain Schema 2 as an additive extension. All five tools advertise and return public Schema 2; existing discover/read/edit input/result shapes and meanings stay unchanged. Document catalog refresh/reconnection where needed to discover the two new tools, without a schema-version migration, parallel compatibility mode or revival of Schema 1.

## User Scenarios & Testing *(mandatory)*

The primary user is a coding agent acting for a developer with the supported live
Godot Editor. The delivered workflow handles existing files; adding or removing a
script currently has no trusted public operation. Both existence changes belong to
this feature, although their eventual implementation tasks may be separate.

**D** is disk source/existence, **R** loaded Script/Resource state and association,
and **B** the editor document/visible buffer. They are independent authorities.
Confirmed absence is not empty source, an unavailable observation or absence from a
limited discovery result. All stories inherit the same safety requirements.

The scenarios below establish required behavior, not executed acceptance.
[C1](#material-clarification-points) requires the existing read revision for deletion.
C2 selects eligible clean open, clean cached-R closed and absent-R closed deletion.
Creation keeps the document closed and permits either confirmed absent R or a newly
associated, independently coherent R. C3 retains public Schema 2 additively.

### User Story 1 - Create a New Script Without Overwriting Work (Priority: P1)

As a coding agent, I want to create a supported script at the exact project path I
choose, with its complete intended source, without falling back to raw file writes.

**Why this priority**: Exact editing deliberately requires an existing file; it
cannot express a new script without violating the existing edit contract.

**Independent Test**: Through a supported real coding-agent client, create at a
fresh eligible path and read the result. Independently verify exact disk source,
continued document absence, and either confirmed absent R or the new Resource's
identity/association and exact source. Challenge occupied/stale paths, invalid
source and unavailable/divergent postconditions while preserving human/editor state.

**Acceptance Scenarios**:

1. **Given** an authenticated selected project, an eligible absent `res://` script path with an existing parent, and admissible complete source, **When** the agent creates the script, **Then** verified success requires the exact intended D, continued document/B absence, and either independently confirmed absent R or a newly associated R whose identity and exact intended source are independently verified. A new R is permitted as a necessary creation effect, not an excuse to force-load the target. No implicit open, close, Save, unload or human reconciliation makes creation succeed.
2. **Given** an existing target, including an empty file or a file already containing the requested source, **When** creation is requested, **Then** it refuses as occupied without writing, editing, truncating, replacing or claiming an unchanged successful create.
3. **Given** the target was absent, **When** a file or conflicting editor/Resource association appears, the parent/target identity changes, or required state becomes stale before effects, **Then** creation refuses without overwriting or redirecting into the replacement target. A race after possible effects follows US3 rather than a false no-effect refusal.
4. **Given** invalid, unsupported or over-bound complete source/context, **When** creation is requested, **Then** trusted validation/admission refuses before project mutation. The operation leaves no placeholder file, partial script, implicit parent directory or editor document merely to perform validation.
5. **Given** supported complete source containing Unicode, whitespace or an admissible empty source, **When** creation succeeds, **Then** source is preserved exactly without normalization, truncation or template insertion. Empty source still creates an actual new file; unavailable source is not empty.
6. **Given** a missing/denied parent, an escaping or unsupported locator, an ambiguous/ended session, or an orphaned document at the requested path, **When** creation is requested, **Then** it refuses or reports the applicable unsupported/conflicting state without selecting another path/session, adopting human work or manufacturing absence.
7. **Given** a file or new Resource association may have been created, **When** required Resource identity/source evidence is unavailable, divergent or no longer attributable to the intended target, **Then** the operation reports effect-sensitive non-success rather than ignoring R, treating an unavailable getter as absence, or loading/unloading/replacing state to manufacture verification.

### User Story 2 - Delete the Observed Script, Not Newer Work (Priority: P1)

As a coding agent, I want to remove an existing supported script using current
trusted state, without deleting a replacement file or discarding human changes.

**Why this priority**: Removing all source text leaves a file in the project.
Deletion is a separate destructive intent with different existence postconditions.

**Independent Test**: For each inherited read-eligible profile—clean open document,
closed with clean coherent cached R, and closed with confirmed absent R—obtain a
non-null current revision from `read_script` and request deletion through a real
client. Independently verify file/document/Resource-association disappearance and
preserved unrelated work. Interleave missing/unavailable/stale revisions, dirty
state and missing-evidence attempts against before/after witnesses.

**Acceptance Scenarios**:

1. **Given** a supported script in any of the three eligible clean profiles and its non-null `read_script` revision matching freshly acquired trusted state, **When** deletion is requested with safety preconditions satisfied, **Then** it supports verified deletion with D absent, no target document/B and no loaded Resource associated with the deleted path. For an open target, removal of that admitted document is an explicit deletion effect; for a loaded target, its target-path association is removed. A native acknowledgment, disk-only absence or missing discovery entry is insufficient.
2. **Given** a missing, unavailable or stale read revision, same-text replacement/version change, changed file/Resource/document identity, changed session or changed lifecycle after observation, **When** deletion is attempted, **Then** it refuses before effects and requires fresh trusted state. A matching pathname or source text cannot authorize deletion of newer work. A file that cannot receive a usable revision under existing read eligibility is not deletable through this feature.
3. **Given** dirty human B, equal-text dirty state, dirty/conflicting R, divergent authorities or unavailable required safety observations, **When** deletion is attempted, **Then** it safely refuses and preserves human source, dirty/saved state, identities and history. It does not Save, discard, force-close, reload or repair state to become eligible.
4. **Given** required safety/disappearance evidence is unavailable or a target cannot be removed without discarding human work or acting on a replacement document/Resource, **When** deletion is requested, **Then** it refuses before effects rather than deleting D and calling the surviving authority inapplicable. It does not first close or unload the target to switch to another eligibility profile. Failure after any explicit deletion effect follows US3; safe refusal does not replace the required positive acceptance for all three supported profiles.
5. **Given** the file is already absent, or becomes absent/replaced before the authorized effect boundary, **When** deletion is requested, **Then** it reports the observed missing/stale state rather than success for this request or proof that an earlier uncertain request succeeded. A surviving buffer is not an absent target.
6. **Given** a supported existing empty file, **When** an eligible deletion succeeds, **Then** the file ceases to exist. This is distinct from the unchanged empty file produced by an exact edit.
7. **Given** scenes or other scripts refer to the target, **When** an otherwise eligible deletion completes, **Then** the result does not claim reference repair, dependency cleanup, project-wide parse validity or running-game hot reload. No unrelated source/scene is rewritten; observable relevant limitations remain truthful.

### User Story 3 - Understand Interrupted and Unverified Effects (Priority: P1)

As a developer, I want an interrupted existence change to say what is actually known,
so neither the agent nor I mistake a timeout for rollback or replay permission.

**Why this priority**: A create/delete effect may occur before verification or result
delivery fails. Incorrect certainty could destroy newer work on a repeated request.

**Independent Test**: Interrupt before and after possible create/delete effects and
invalidate observations during verification. Compare delivered outcomes with
independent filesystem and live-editor witnesses, including later human work.

**Acceptance Scenarios**:

1. **Given** a refusal before effect entry with non-application established, **When** a result is delivered, **Then** it may report not-applied and identifies the actionable reason without mutation or lifecycle/history effects.
2. **Given** creation/deletion may have begun, **When** cancellation, timeout, disconnection, validation/context invalidation or verification failure occurs, **Then** the result preserves known applied/partial or unknown effects and identifies missing postconditions. It never infers rollback or not-applied from a lost acknowledgment.
3. **Given** newer human work or a replacement file/document appears after effects may have begun, **When** completion cannot be safely verified, **Then** further mutation stops and the new work is preserved. No cleanup, retry, restoration or second delete repairs the attempt automatically.
4. **Given** result delivery fails or a later read observes the intended state, **When** the agent considers recovery, **Then** neither fact retroactively certifies the earlier request. Fresh trusted observation is required before a separate deliberate operation; the system never silently replays it.

### User Story 4 - Use One Small Script Surface (Priority: P2)

As a coding agent, I want discover/read/edit/create/delete to be distinct, composable
intents with concise descriptions, not hidden modes or a general file manager.

**Why this priority**: The agent must choose the right operation and interpret its
results without learning a second safety model or weakening existing exact edits.

**Independent Test**: Exercise the five-tool Schema 2 catalog and a create → read →
exact edit → fresh read → delete → observe absence workflow in both existing
supported clients, with independent editor witnesses. Existing discover/read/edit
calls retain their input/result shapes and meanings; the workflow must not need
hidden lifecycle actions to make its targets eligible.

**Acceptance Scenarios**:

1. **Given** a client using the existing Schema 2 discover/read/edit contract, **When** it refreshes the expanded catalog, **Then** exactly `discover_scripts`, `read_script`, `edit_script`, `create_script` and `delete_script` describe the available script intents, all advertising and returning public Schema 2. Existing discover/read/edit input/result shapes and meanings remain unchanged; no generic CRUD, alternate edit mode, Save, history, open or close tool appears.
2. **Given** a newly verified script, **When** the agent discovers/reads it and makes an admissible exact edit, **Then** existing source, revision, lifecycle, validation and verification semantics still hold. A later eligible delete removes the file rather than blanking it; complete fresh discovery/read observations reflect its absence without treating partial coverage as proof.
3. **Given** an existing complete empty script, **When** `edit_script` uses empty `old_string` under Feature 008's rule, **Then** it edits only that existing source. Against a missing file it cannot create; empty `new_string` removes source text, never the file.
4. **Given** invalid, legacy, mixed or unsupported arguments, **When** a call is made, **Then** the public contract refuses without dispatching a hidden create/delete mode or reviving Schema 1 editing. Adding the two tools requires catalog refresh where cached, not a schema-version migration or parallel compatibility mode.
5. **Given** a known target path, **When** the agent creates or uses `read_script` to obtain the required non-null revision before deletion, **Then** discovery is optional. Discovery alone grants neither creation absence proof nor deletion authority; results give the relevant safe next action without exposing private source/context or routing data.

### Edge Cases

- A directory, link, unsupported file, existing empty script or same-source file can occupy the requested create path; none authorizes replacement. Missing parent directories are not created implicitly.
- Confined identity checks must survive leaf/parent substitution, same-path recreation and disappearance/reappearance. A lexical `res://` prefix alone proves neither confinement nor current identity.
- A missing disk file can coexist with B or R; an unloaded Resource, absent buffer, empty source and failed getter are different facts. Source equality alone does not establish clean state.
- Ordinary discovery can be limited, stale or omit Godot-hidden/ignored paths. It cannot establish exclusive creation rights or verified deletion. Existing project-visibility/support rules remain applicable.
- Create → delete → create at the same path produces distinct target lifetimes. Old read/deletion bases cannot authorize the new file, even with identical source.
- The source-only validation profile and bounds still apply to creation. Deletion has no intended replacement source; applicable current-context safety validation is not a promise of whole-project dependency analysis. Deletion requires the existing read eligibility: if `read_script` cannot issue a usable revision, deletion refuses rather than introducing a separate basis or bypassing that limitation.
- Ordinary subsequent Save, reparse, rescan, close/reopen and fresh runtime activity must not undo a verified existence result where applicable. Fixtures may challenge persistence; the product operation cannot use those actions to establish eligibility or repair a false success.
- Deleting a clean open script explicitly ends the admitted target document's lifetime; it is not an implicit close used to qualify a different deletion profile. Target-history recovery or delete Undo is not promised; unrelated usable history remains protected.
- A Resource instance retained by another owner need not be physically destroyed, but its association with the deleted target path must be independently confirmed absent. Retained references are not repaired, and unknown or surviving target associations cannot be reported as verified disappearance.
- Removal of the target document or Resource association followed by failed file deletion/verification is an attributable deletion effect, not a no-effect refusal or automatic rollback.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: This feature MUST add explicit `create_script` and `delete_script` intents alongside the three current Schema 2 script tools. Both existence operations belong to this single feature and apply only to supported standalone project GDScripts, one exact target per call.
- **FR-002**: Every operation MUST enforce the existing authenticated explicit project/session selection, project confinement, permission and disclosure boundaries. Ambiguous, unavailable, replaced or denied authority MUST refuse without fallback to editor focus, working directory, basename or another session/project.
- **FR-003**: Create MUST accept an explicit confined `res://` path and complete intended source. It MUST establish current target absence and retain stale/identity protection through effect entry; an occupied or newly occupied target MUST NOT be overwritten, adopted or edited as a fallback, even when its source matches.
- **FR-004**: Create MUST apply existing trusted complete-source and supported-context admission/validation guarantees before project mutation. Invalid, unsupported, unavailable or over-bound validation MUST refuse without creating a placeholder. Exact source representation and limits MUST be preserved without repair or truncation.
- **FR-005**: Create MUST preserve target-document/B absence throughout; it MUST NOT implicitly open, close, Save, reload, force-load/unload or otherwise manipulate editor lifecycle to make creation succeed. Success MUST independently verify exact intended D and either positively confirmed absent R or a new Resource associated with the target as a necessary creation effect, whose identity and exact intended source are independently verified. This permission MUST NOT adopt a pre-existing conflicting Resource or force-load one merely for verification. Missing/divergent required evidence MUST produce non-success, never invented absence or coherence.
- **FR-006**: Delete MUST require the existing non-null revision returned by `read_script` for the exact supported project GDScript. It MUST freshly acquire trusted target/state information and match that revision before authorizing effects, retaining all later mutation-boundary and deletion-specific safety checks against stale source, identity, session and lifecycle changes. Missing, unavailable or stale revisions MUST refuse. Existing read source/context eligibility remains binding; there is no separate deletion-basis fallback. Discovery, a pathname, a caller's assertion of cleanliness or the revision alone MUST NOT grant mutation authority.
- **FR-007**: Delete MUST protect dirty/conflicting human/editor and Resource state, including equal-text dirty work. Missing required safety evidence MUST cause refusal. It MUST NOT implicitly Save, discard human work, force-close, reload or guess, and MUST preserve unrelated documents, sources, identities and usable history.
- **FR-008**: Delete MUST support the three inherited read-eligible profiles: clean open document, closed with clean coherent cached R, and closed with confirmed absent R, subject to FR-006/FR-007. Verified success MUST independently establish D absent, no target document/B, and no loaded Resource associated with the deleted path. Removal of the admitted clean target document and its Resource association MUST be explicit deletion effects, not preliminary close/unload actions to manufacture eligibility. Closed targets MUST never be opened. Physical destruction of Resource instances retained elsewhere is not required or promised; surviving or unavailable target associations MUST produce non-success rather than be relabeled inapplicable. Disk unlink/acknowledgment alone MUST NOT suffice, and missing pre-effect safety evidence MUST refuse.
- **FR-009**: A missing target before an authorized deletion effect MUST remain a missing/stale refusal, not idempotent deletion success or retroactive proof of an earlier request. Neither create nor delete may retarget a replacement object or refresh stale intent silently.
- **FR-010**: Both operations MUST retain the common trusted transaction/coherence semantics: explicit preconditions, effect boundary, synchronization, independent verification and terminal outcome. Native/editor-aware guarantees MUST NOT be replaced by blind filesystem mutation or a second weaker path. Permitted existence-related transitions are creation's optional new Resource association while B stays absent under FR-005, and deletion's removal of the admitted clean target document and Resource association under FR-008. No other lifecycle change or implicit open/close/load/unload step may manufacture eligibility or verification; newer/replacement state remains protected.
- **FR-011**: Results MUST distinguish acceptance, effect entry/application, synchronization and independent verification. They MUST report relevant current/invalidated revisions and identity, observed/unavailable/absent D/R/B, dirty/synchronization state, validation, diagnostics, history participation and known/partial/unknown effects without inventing observations or recovery guarantees.
- **FR-012**: Structured outcomes MUST distinguish occupied create target, missing delete target, stale basis/identity/lifecycle, dirty/conflicting state, unsupported source/context/capability, denied/ambiguous routing, validation failure/unavailability and interrupted/unverified effects where applicable. Safe next actions MUST depend on actual effect knowledge; uncertainty MUST NOT become a generic no-effect error.
- **FR-013**: Cancellation, timeout, disconnection, newer work and delivery failure MUST retain existing effect-sensitive semantics. After possible effects, unknown or partial state MUST NOT become a definite not-applied claim. No automatic queue/retry/replay, compensation, rollback or cleanup of a replacement target is permitted.
- **FR-014**: Existing discover/read/exact-edit meanings MUST remain intact, including non-mutating reads, current opaque revisions, supported lifecycle profiles and independent complete-source verification. Empty `old_string` MUST retain only Feature 008's complete-existing-empty-source meaning; empty `new_string` MUST remain source-text removal. Neither can create/delete a file or invoke a hidden mode.
- **FR-015**: This feature MUST add create/delete as an additive public Schema 2 extension. All five tools MUST advertise and return public Schema 2; existing discover/read/edit input/result shapes and meanings MUST remain unchanged. Documentation MUST explain the expanded catalog and refresh/reconnection requirements for clients that cache it, without requiring a schema-version migration. No aliases, dual edit modes, Schema 1 revival or unsafe fallback are permitted. Any future incompatible public change requires deliberate versioning and migration rather than silently altering this contract; no MCP protocol-version change is implied.
- **FR-016**: Public descriptions MUST remain concise and task-oriented under Constitution Principle IX, sufficient to distinguish existence changes from editing, supply current basis and understand limitations/results. Structured schemas and outcomes MUST carry necessary distinctions; prose MUST NOT substitute for trusted enforcement or expose private architecture.
- **FR-017**: Existing source/context, exact engine/platform/client, local transport, permission, resource and export/privacy boundaries MUST remain intact. Complete source, deleted source and validation context MUST NOT leak through incidental logs/errors or unauthorized outcomes. No broader support, telemetry, arbitrary execution, active-game hot reload or new external access authority is added.
- **FR-018**: Read/discover MUST retain their supported-condition five-second bound and edit its ten-second bound. Create/delete MUST also terminate within ten seconds under the same supported local conditions, including required acquisition, validation, verification and deliverable results, without restarting the request clock. This inherited mutation-budget assumption is a requirement to prove, not an existing create/delete performance claim.
- **FR-019**: Actual use in both existing supported coding-agent clients MUST establish each new operation and their composed workflow, with independent live-editor observations. Applicable constitutional A–E, human-work, sequential-operation and durability obligations MUST remain covered; missing observability is not an inapplicability justification. Delete Undo or other recovery MUST NOT be claimed.
- **FR-020**: Verification MUST follow [TEST_POLICY.md](../../TEST_POLICY.md): new existence semantics and directly affected regressions need evidence; unchanged historical evidence is reusable after relevant-input review. No blanket historical campaign, new CI provider/approval gate, release candidate or broader environment investigation is required merely by this feature.
- **FR-021**: Cross-file reference repair, scene rewriting, automatic dependency cleanup, rollback, trash/recycle-bin recovery, delete Undo, generic Resource mutation and generic file management MUST remain outside this feature. Relevant limits MUST be disclosed; deletion success MUST NOT imply that all dependents still parse or run.

### Public capability boundary

The five-tool surface is an additive extension of public Schema 2. Discover
finds existing supported scripts; read observes source/state and current basis;
edit transforms an existing script's source; create establishes a new script at an
absent path; delete removes the current observed script. Only the last two are new.
Exact argument names, new operation-specific result layouts and implementation
mechanisms belong to later planning within these clarified outcomes. Existing
discover/read/edit shapes and meanings are not reopened by that planning.

No generic filesystem CRUD, directory management, rename/move, arbitrary files,
batches, scene/general Resource authoring or file-management framework is selected.
No Save/Undo/Redo/open/close tools are added. Deletion's explicit removal of its
admitted clean target document and Resource association is part of file deletion,
not a standalone close/unload capability or a preliminary eligibility workaround.
Existing private/local opening and closing capabilities remain distinct. No
reference repair, target-history recovery or delete Undo is implied.

### Scope and constitutional alignment

This is a bounded extension of the delivered script-agent workflow, addressing the
user-requested file-existence gap. [Phase 3 remains complete](../../PHASE_3_EXIT.md#decision);
create/delete are not retroactive exit requirements. [Phase 4 scene/Resource
transactions](../../ROADMAP.md#phase-4--scene-and-resource-transactions) and release
work remain pending. Feature 006 remains [assessed, not proceeding](../006-save-project-gdscript/decision.md).

Principles I–IV require independent authorities, human-work protection, native/editor
semantics and verified effects. V, X and XII retain confinement, truthful failures
and safe refusal; VI requires live-editor evidence; VII retains adapter → core →
Godot separation; VIII retains export isolation; IX limits the public surface; XI
retains independent implementation. C1–C3 cannot waive those invariants. Planning
and mutation/security-sensitive review must explicitly record compliance; this
specification does not select a new architecture or amend the constitution.

**Principle XIII:** The current need is trusted creation/removal of project scripts.
Keeping only discover/read/edit is simpler but cannot express either existence
change; asking the agent to write/delete raw files bypasses the required safety
contract. Two explicit intents reuse the existing trusted boundaries. Their costs
are existence-specific admission/postconditions, compatibility documentation and
focused behavioral/client evidence. Those costs serve a present requested workflow;
they do not justify a generic file framework, new dependency/service, approval gate,
security layer or new test infrastructure. Any additional mechanism must earn its
cost in the later plan using existing evidence first.

### Key Entities *(include if feature involves data)*

- **Create intent**: Explicit selected project/session, exact absent script path and complete intended source; never an edit or overwrite instruction.
- **Delete intent and current basis**: Explicit selected project/session/path plus the existing non-null `read_script` revision for that target, matched against freshly acquired trusted state. It preserves existing read eligibility and stale protection; it is not authorization or a substitute for deletion-specific safety checks.
- **Target lifetime**: The specific file and applicable Resource/document identities, distinct from another object later occupying the same path.
- **Authority/existence evidence**: Independent D/R/B observations with actual presence/absence, source where applicable, dirty state, validity interval and unavailable facts. Creation verifies exact D, absent B and either confirmed absent R or the new R's identity/association and exact source. Deletion verifies absence of the file, target document and loaded Resource association with the deleted path, without asserting destruction of every retained Resource object.
- **Existence-change outcome**: Verified created/deleted, safe refusal or effect-sensitive non-success, with applicable evidence, certainty and safe next action.
- **Public compatibility boundary**: The additive five-tool Schema 2 surface, preserving existing discover/read/edit shapes and meanings while requiring catalog refresh to discover new capabilities. It is independent of MCP transport version and implementation delivery state.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Both existing supported coding-agent clients complete creation from full source and read-revision-based deletion across clean open, clean cached-R closed and absent-R closed profiles, plus the create → read → exact edit → fresh read → delete → observe absence workflow. Independent witnesses establish exact created source and the deleted file/document/Resource-association absence; no raw-file or hidden lifecycle workaround substitutes for a tool call.
- **SC-002**: Every exercised occupied/same-source/empty-file create attempt and stale replacement race preserves the existing/newer target. There are zero overwrite fallbacks, redirected mutations, placeholder files on pre-effect validation refusal or implicit directory/document creation.
- **SC-003**: Every exercised stale, dirty/equal-text-dirty, divergent, unavailable-evidence, routing/confinement and unsupported-state delete attempt preserves the required human/editor state. There are zero forced Save/discard/reload/close actions and zero old-basis deletions of a new target lifetime.
- **SC-004**: Every reported create/delete success has fresh independent applicable-authority evidence, not only acknowledgment or inventory omission. Create verifies exact D, continued B absence and either confirmed absent R or the new R's identity/association and exact source. Delete verifies file, target-document and target-path Resource-association absence for all three supported profiles without claiming physical destruction of every Resource instance. Interrupted/adverse cases, including partial document/association removal, produce zero false success, definite-not-applied, rollback or replay-permission claims and preserve later work.
- **SC-005**: Supported empty, Unicode, representation and source/context-boundary cases retain exact source and the existing validation guarantees. No invalid/unsupported create source is accepted by bypassing validation. Deletion refuses whenever the existing read workflow cannot issue a usable revision, even when the disk file exists; no alternate-basis fallback succeeds.
- **SC-006**: Sequential create/delete/recreate and composed exact-edit use preserve distinct target lifetimes and applicable durability under ordinary later editor/runtime activity. Existing applicable A–E/history guarantees remain supported by valid evidence; deletion does not invent Undo or recovery.
- **SC-007**: Catalog/compatibility acceptance establishes exactly five public script tools, all advertising and returning Schema 2, with unchanged existing discover/read/edit input/result shapes and Feature 008 edit meanings. Clients discover the added tools through catalog refresh without a schema-version migration. All exercised hidden-mode/legacy/mixed requests cause zero unintended effects; concise descriptions suffice for correct real-agent use without private implementation knowledge.
- **SC-008**: Supported-condition operations meet FR-018's five-/ten-second bounds through delivered results. Privacy/export checks establish zero prohibited content/credential disclosure and no unintended tooling export; relevant-input review accounts for inherited evidence without claiming untested support.

## Assumptions

- Features 001–005, 007 and 008 are complete; Feature 006 is an assessed non-proceeding proposal, not completed Save implementation. Their acceptance history and Phase 1–3 exit decisions remain unchanged.
- Current public behavior is [Feature 008 Schema 2](../008-exact-script-edit/contracts/mcp-interface.md). Its exact-empty-source edit rule and prior [clarifications](../008-exact-script-edit/spec.md#clarifications) remain truthful history, not authorization for create/delete.
- The inherited target environment is official Godot 4.7.2.stable.official.ed1daf0bf, macOS 26.6.2 (25G83) arm64 on the recorded four-vCPU/6-GiB profile, local stdio MCP 2025-11-25, Codex CLI 0.153.4 and OMP 18.5.1. This draft does not claim create/delete are already supported there or change local GUI execution policy.
- Existing standalone-script visibility and [source/context bounds](../007-mcp-script-workflow/data-model.md#2-supported-profile-and-bounds) apply; complete create source is at most 512 KiB in the supported representation, rejecting NUL/CR/BOM without repair. An existing supported parent is required; directory creation is out of scope.
- Creation requires no fake read revision for a nonexistent file. It must acquire trusted absence/current state itself and guard the effect boundary. Deletion instead requires the existing non-null `read_script` revision and retains that read's eligibility rules.
- Ten seconds is the draft create/delete budget, matching current bounded mutation/lifecycle behavior; five-second read/discover and ten-second edit remain unchanged. No arbitrary-load or maximum-context-success promise follows.
- Existing supported threat-model and validator limitations remain as recorded, including the [capability-delta readiness evidence](../002-edit-open-gdscript/research.md#199-capability-delta-readiness-review-2026-09-29). New existence behavior needs new evidence; it does not by itself reopen settled endpoint isolation or require a broader adversary/platform model.

### Material clarification points

C1–C3 are resolved by four accepted answers in the session above. No material
product ambiguity remains. Exact native APIs, filesystem mechanisms, module
layout and new operation-specific result layouts remain planning decisions,
not unresolved clarification questions or permission to weaken these outcomes.

1. **C1 — Delete basis and source eligibility (FR-006, US2), resolved:** Require the existing non-null `read_script` revision and refuse when it is missing, unavailable or stale. Retain the existing [read/revision contract](../007-mcp-script-workflow/data-model.md#revision-precondition), including source/context eligibility, fresh acquisition/comparison and later safety guards. A valid revision does not by itself establish deletion eligibility; deletion-specific checks and independent verification remain mandatory. No alternate deletion basis is selected.
2. **C2 — Eligible lifecycle states and authority postconditions (FR-005, FR-008, US1/US2), resolved:** Creation keeps B absent and verifies exact D plus either confirmed absent R or a newly associated R with independently verified identity and exact source as a necessary creation effect; no forced load/unload or other lifecycle manipulation manufactures success. Delete supports eligible clean open, clean cached-R closed and absent-R closed scripts, explicitly removes the admitted target document and Resource association, and independently verifies file/document/target-path Resource-association absence. Closed targets are never opened, dirty/conflicting state refuses, and no preliminary close/unload step manufactures eligibility. Retained Resource instances need not be physically destroyed; references are not repaired and surviving/unknown target associations defeat verified success.
3. **C3 — Public compatibility/version declaration (FR-015, US4), resolved:** Retain public Schema 2 for the additive five-tool catalog and its advertised schemas/results. Existing discover/read/edit input/result shapes and meanings remain unchanged. Document catalog refresh/reconnection where needed to discover create/delete; no schema-version migration, parallel compatibility mode, Schema 1 revival or MCP protocol-version change is selected.

### Existing evidence and limits

- [Feature 001 observation](../001-observe-gdscript-state/spec.md) distinguishes independent D/R/B, dirty state, missing targets and unavailable observations; none is an existence-mutation capability.
- [Feature 002 guarded editing](../002-edit-open-gdscript/spec.md) and its [validation/effect research](../002-edit-open-gdscript/research.md#199-capability-delta-readiness-review-2026-09-29) establish trusted source/context validation and effect-sensitive safety, not permission to overwrite an absent or replaced target.
- [Feature 003 opening](../003-open-project-gdscript/spec.md) operates on existing files. Creating an in-memory Resource to open a script is not creation of a project file.
- [Feature 004 discovery](../004-discover-project-gdscript/spec.md) supplies confined inventory with explicit coverage; inventory omission is not verified absence or mutation authority.
- [Feature 005 closing](../005-close-project-gdscript/contracts/close-api.md#6-meaningful-consumer-visible-boundaries) distinguishes missing D with surviving B, confirmed closure and retained/unloaded R. Its accepted close behavior neither deletes files nor grants permission to close a target as a delete workaround.
- [Feature 006's decision](../006-save-project-gdscript/decision.md), [Feature 007's completed workflow](../007-mcp-script-workflow/spec.md) and [Feature 008's cumulative acceptance](../008-exact-script-edit/quickstart.md#t003-composed-and-cumulative-acceptance-2026-10-07) retain the small public surface, fixed lifecycle, privacy/timing/support limits and exact-edit distinctions. They are reusable antecedents, not create/delete acceptance.

### Stage boundary

Specify and clarification are complete; the four accepted answers resolve C1–C3
without changing the accepted product scope. The separately
requested [planning stage](plan.md) has produced research, a data model, contracts
and a validation guide. Those artifacts record a selected design and bounded
mechanics evidence, not product acceptance or implementation approval.
The [quality checklist](checklists/requirements.md), project status and existing
draft design PR reflect this state. Task derivation, granularity review, consistency
analysis, implementation, merge, release, Phase 4 and another feature have not started.
