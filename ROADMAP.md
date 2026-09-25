# godot-agent-kit product roadmap

This roadmap describes long-term product direction and dependency ordering, not an
implementation plan or authorization to begin a phase. It is subordinate to the
[constitution](.specify/memory/constitution.md) and the delivery workflow in
[AGENTS.md](AGENTS.md). Phase exit criteria supplement, never replace, those requirements.

The intended progression is:

**correctness first → trusted editor transactions → protocol exposure → wider Godot
capability → runtime/debugging → concurrency/security → extensibility/productization**

The first priority is proving that agents can safely edit a live Godot Editor while
preserving human work. Tool count, broad Godot API coverage, and feature parity with
other Godot MCP projects are not early success measures. The eventual goal is a mature,
independent open-source automation platform built on that proof.

## How to read the sequence

- Later capabilities depend on the relevant earlier exit gates. All mutation paths use
  the shared transaction/coherence model; unsupported guarantees require safe refusal.
- Every phase inherits the full constitution. Security, explicit target selection,
  actionable diagnostics, and tooling/gameplay separation apply from the first relevant
  capability. Their later phases deepen support rather than defer baseline obligations.
- Local-only operation and no telemetry are defaults. Evaluation, process execution,
  force writes, outside-project access, and network access require explicit permission
  gates whenever introduced. Approval never replaces conflict or coherence safety.
- Mutation features need deterministic unit and boundary integration coverage plus
  real-Godot live-editor regression evidence for applicable A–E gates. Disk-only checks,
  headless runtime, and mocks cannot prove visible-buffer coherence. Use bounded,
  event-based synchronization and retain regressions for discovered failures.
- Successful edits must survive Save, close/reopen, reparse, rescan, and runtime launch
  wherever applicable. Record reasons for inapplicable surfaces or scenarios; missing
  observability is not an exemption. Supporting durability checks do not authorize a
  general runtime product surface early.
- Claim only exact Godot versions covered by appropriate CI and real-editor testing.
  Each release needs verified guarantees, known limitations, basic installation guidance,
  permission boundaries, and tested production-export isolation. Distribution polish can
  come later; these obligations cannot.

## Phase 0 — Governance and project foundation

**Purpose:** Establish the rules under which technical capability can be trusted.

**Status:** The governance foundation is merged: constitution v1.0.0 and root
`AGENTS.md`. At this roadmap's introduction, the tracked baseline has no product code,
README/docs collection, approved feature specification, implementation plan, or task list.
The Spec Kit workflow is mandated; local tooling installation is not a delivered feature.

**Capability areas:**

- Constitutional authority, spec-driven development, and one implementation task per PR.
- The D/R/B model, A–E release gates, independent postcondition verification, and safe
  refusal as the foundation for later work.
- Independent development from public APIs/specifications and observed behavior, with
  dependency license/provenance review rather than copying unrelated implementations.
- Recorded stack guidance: a primarily Rust server with a thin GDScript EditorPlugin as
  the expected starting point, subject to evidence and approved planning. No separate
  stack-research artifact or tested compatibility matrix is present; the guidance is not
  a claim that an implementation has been proven.

**Deferred:** Product scaffolding, dependency selection, transport decisions, and any
technical capability. Research supporting concrete stack choices belongs in later feature
planning before commitment.

**Exit:** Merged governance establishes authority, safety gates, workflow, and provisional
stack direction. This foundation is present; technical feasibility remains unproven.

## Phase 1 — Live-editor script coherence

**Purpose:** Prove one reliable vertical slice: safe GDScript editing in a live Godot
Editor, with human unsaved work protected.

**Capability areas:** Explicit editor/session discovery and selection; script discovery,
read, and open; independent observation of **D — disk source**, **R — loaded
Resource/Script source**, and **B — visible editor buffer**; dirty-buffer detection;
revision/stale-write protection; coherent mutation; Save; close/reopen; and real Godot
Undo/Redo. Use Godot-native editor semantics as the canonical mutation route, not file
writes followed by hoped-for reloads.

Define deterministic outcomes that separate request acceptance, application,
synchronization, and independently observed verification. Success for an open-script edit
requires observed `D == R == B`. Expose revisions, dirty/synchronization state, diagnostics,
UndoRedo participation, partial application, and unavailable observations. Refuse when
conflicts or missing safety observability prevent a verified result.

**Required real-editor proof:**

- **A — Clean open-buffer edit:** D, R, and B converge without human reconciliation.
- **B — Dirty human-buffer conflict:** Preserve the unsaved edit and refuse safely, or use
  an explicitly designed resolution workflow; never silently overwrite it.
- **C — Real Undo/Redo:** Verify apply → Undo → Redo through Godot's editing history across
  applicable state surfaces, not merely registration of an undoable operation.
- **D — Close/reopen persistence:** Edits reported successful remain present and coherent
  after reopening.
- **E — Sequential edit stress:** Repeated edits do not disappear, diverge, conceal
  conflicts, or revert on Save.

**Deferred:** Scene/resource authoring, project configuration, runtime tooling, debugger
integration, MCP exposure or broad tool coverage, and other feature areas. Do not expand
into them before this coherence proof. Runtime-launch durability verification remains an
applicable correctness check, not runtime-tool development.

**Exit:** A–E pass in real Godot, with Save/reparse/rescan/runtime durability checked where
applicable. Accepted edits reported successful remain coherent, human work is protected,
and limitations/non-success outcomes are documented. No reconciliation of agent-created
editor divergence is required.

## Phase 2 — Protocol-independent automation/transaction core

**Purpose:** Generalize the proven script workflow into reusable semantics without losing
its guarantees.

**Capability areas:** Sessions, explicit targets, revisions, transactions, conflicts,
mutation outcomes, diagnostics, deadlines/timeouts, cancellation, idempotency/retries,
partial failures, and correlated structured tracing with project-content redaction.
Distinguish revision mismatch, dirty buffer, stale Resource or buffer, parse error,
synchronization timeout, disconnection, unsupported capability, approval/permission
requirements, and unavailable runtime where applicable.

Preserve the conceptual boundary:

`agent protocol adapter → automation/transaction core → Godot integration`

Phase 1 already respects this boundary; this phase consolidates demonstrated behavior,
not a speculative framework. Timeout or cancellation does not imply rollback. Outcomes
must expose known application state, and retry safety or non-idempotency must be explicit.

**Scope boundary:** MCP-specific contracts stay out of the core. Generic extension
frameworks, broad Godot capability coverage, and concurrent multi-editor mutation remain
deferred.

**Exit:** The Phase 1 workflow is driven through the reusable core with the same A–E
results, observed postconditions, and safe failure behavior. No adapter can supply a
weaker mutation path.

## Phase 3 — MCP adapter MVP

**Purpose:** Make the proven script workflow usable by real coding agents through the
first external adapter.

**Capability areas:** A deliberately small, composable surface for editor/session status,
capabilities, script read/open/edit/save/close, Undo/Redo, and diagnostic/state inspection.
Provide consistent schemas, structured outcomes/errors, capability discovery and
progressive disclosure, cancellation mapping, and protocol/request versus execution-error
distinctions without losing core outcome detail.

Keep transport and MCP details at the adapter. Validate inputs and outputs, separate
protocol traffic from logging, and advertise only supported capabilities. Select compatible
released MCP specification/SDK versions and any transport in the approved plan, not here;
transport-specific security requirements apply from introduction.

**Deferred:** Tool-catalog expansion, scene authoring, runtime/debugger tools, and protocol
features without a demonstrated agent need.

**Exit:** Real MCP clients, including coding agents rather than only an internal harness,
complete the Phase 1 A–E workflows over MCP. Dirty human edits remain protected, safe
refusals remain explicit, and successful edits need no human reconciliation. This is the
likely first useful public release, tentatively **v0.1**, not a broad Godot API wrapper.

## Phase 4 — Scene and Resource transactions

**Purpose:** Extend trusted mutation to common scene and Resource authoring.

**Capability areas:** Scenes, nodes, properties, signals, reusable Resources, Save/reopen,
and real Undo/Redo through Godot-native editing transactions. Respect dirty editor state,
revisions, shared transaction outcomes, and independent postcondition observation.

**Scope boundary:** `.tscn` or `.tres` text rewriting is not the canonical editing model.
Advanced semantic scene diffs and large batch authoring workflows remain deferred.

**Exit:** Agents can perform common scene/Resource authoring safely. Real-editor
regressions cover clean edits, dirty conflicts, actual Undo/Redo, save/reopen persistence,
and sequential changes comparable in spirit to A–E, with applicable surfaces and any
inapplicability justified. Existing script gates continue to pass.

## Phase 5 — Project configuration

**Purpose:** Remove the need for unsafe raw-file changes to common project configuration.

**Capability areas:** InputMap, autoloads, project settings, layers/groups where appropriate,
and plugin/project configuration. Validate changes, use Godot-authoritative semantics,
report restart requirements, and define observable persistence and failure outcomes.
Document rollback support and its limits; partial application is never implied to have
rolled back. Preserve human changes and use Undo/Redo wherever supported and claimed.

**Excluded:** Blind direct editing of `project.godot` as the primary model, unrestricted
plugin/process execution, and arbitrary environment administration.

**Exit:** Common configuration changes use verified transactions rather than unsafe raw
mutation. Validation, conflicts, partial failures, save/reopen durability, and relevant
restart behavior are covered; applicable coherence gates remain satisfied.

## Phase 6 — Runtime control and inspection

**Purpose:** Enable an agent to verify behavior after trustworthy editor changes.

**Capability areas:** Start/stop, pause/resume, run a scene or the main project, runtime
readiness, runtime scene-tree and node/property inspection, screenshots/visual evidence,
and controlled evaluation only when separately permitted. Distinguish editor state from
runtime observations and report an unavailable runtime explicitly.

Development helpers and probes must not become gameplay authority or required game state.
Their development/export boundary must be explicit and verified, including exclusion from
production exports; a tooling directory or annotation alone is not evidence.

**Scope boundary:** Source-level debugging and profiler integration remain deferred.
Evaluation requires explicit permission; unrestricted evaluation is excluded.

**Exit:** An agent can perform **edit → run → inspect → verify** without manual Godot
operation, while editor guarantees persist and tooling remains outside gameplay authority
and unintended production exports.

## Phase 7 — Diagnostics, LSP, symbols, and navigation

**Purpose:** Enable reliable cross-file understanding of GDScript projects.

**Capability areas:** File/project diagnostics, symbols, definitions, references,
completion/navigation, `class_name` indexing, and explicit parse/index readiness or stale
state. Integrate Godot's existing language/editor services rather than build a second
GDScript language implementation.

Clearly identify **disk diagnostics**, **live-editor diagnostics**, **LSP diagnostics**,
and **runtime errors**, including the observed source/revision when available. Do not
present one source as proof that another is current. Basic transaction diagnostics have
been required since Phase 1; this phase adds developer intelligence.

**Deferred:** A standalone GDScript compiler/language server and higher-level refactoring.

**Exit:** Agents reliably diagnose and navigate cross-file projects, distinguish diagnostic
sources, and recognize unavailable or stale parse/index results instead of trusting them.

## Phase 8 — Native debugger integration

**Purpose:** Support investigation using genuine source-debugger state.

**Capability areas:** Source breakpoints, pause/continue, step into/over/out, stack frames,
locals/variables, and separately permissioned expression evaluation. Keep debugger session
and source state explicit so observations relate to the intended running project.

**Scope boundary:** Profiler/performance captures remain deferred. Frame advancement and
runtime evaluation are not source debugging and cannot substitute for debugger integration.

**Exit:** An agent investigates a gameplay bug through real breakpoints, stepping, stack
frames, and variables, rather than relying primarily on logs or guess-and-rerun loops.

## Phase 9 — Multi-editor, multi-project, and multi-agent operation

**Purpose:** Expand beyond proven single-editor transactions without cross-target damage.

**Capability areas:** Project, editor session, workspace/worktree, and agent session
identity; discovery/selection; explicit routing; leases/locks where appropriate; revision
conflicts; and concurrent mutation handling. Routing identity must remain unambiguous
through a transaction, including disconnect/reconnect and stale-session conditions.

**Core rule:** **ambiguous mutation target → safe refusal**. Never guess an editor or
project. This rule already applies to earlier phases; concurrency increases the scope of
its proof rather than introducing it for the first time.

**Scope boundary:** Distributed/remote orchestration remains deferred. Locks cannot
replace revision checks or human-buffer protection.

**Exit:** Concurrent editors, worktrees, and agents cannot silently cross-mutate targets.
Real-editor and boundary regressions demonstrate isolation, deterministic conflicts, and
preserved coherence under competing edits and session changes.

## Phase 10 — Security and permission model

**Purpose:** Consolidate the security model for wider third-party-agent adoption, not add
security for the first time.

**Capability areas:** Capability permissions, approval classes, project/path sandboxing,
evaluation and process-execution permissions, separately permissioned force writes,
network permissions, authentication/session tokens where applicable, logging redaction,
and least privilege. Local operation and no telemetry remain defaults unless changed
through constitutional governance. Permission never substitutes for transaction safety.

Earlier features and releases already require safeguards for their exposed surfaces.
This phase makes those guarantees coherent across concurrent sessions and broader use
cases, including a documented threat model and permission-boundary verification.

**Excluded:** Remote access by default, blanket approvals, and any force mechanism that
bypasses dirty-state detection or required coherence guarantees.

**Exit:** An explicit, documented and tested security model is suitable for connecting
third-party coding agents, with understandable permissions, denial outcomes, and exposure
limits across supported capabilities.

## Phase 11 — Extensibility

**Purpose:** Allow project/domain-specific automation without parallel unsafe mechanisms.

**Capability areas:** Project-specific tools, domain operations, custom capability
providers, and future protocol adapters. Extensions must reuse the common transactions,
conflict handling, permission boundaries, structured outcomes, and observability.

For example, a project could expose validation of a game-specific simulation using shared
runtime/transaction infrastructure without moving simulation authority into the tooling.

**Excluded:** Unrestricted extension execution or direct-mutation escape hatches.

**Exit:** Projects can extend the kit through documented contracts while preserving core
coherence, security, and compatibility guarantees. Extension verification demonstrates
that adapters/providers cannot introduce weaker mutation semantics.

## Phase 12 — Distribution and open-source productization

**Purpose:** Make installation, maintenance, and contribution predictable for new users.

**Capability areas:** Prebuilt Rust binaries; Windows/macOS/Linux distribution; Godot
Asset Library packaging; installation/configuration helpers; health/doctor diagnostics;
version compatibility checks; checksums/signing where appropriate; release automation;
changelog/migration guidance; contribution documentation; and a security policy.

These are intended outcomes, not selections of packaging systems or release infrastructure.
Basic installation, compatibility, security, and limitation documentation accompanies
earlier releases. Wider platform support is claimed only after corresponding verification;
public schemas, structured errors, and outcomes already require deliberate versioning and
migration guidance for breaking changes. Unsafe behavior is not preserved for compatibility.

**Scope boundary:** Infrastructure choices belong in later plans; untested platform/version
support claims are excluded.

**Exit:** A new user can install, diagnose, upgrade, and remove the kit predictably on each
supported platform. Compatibility and provenance are documented, and release automation
respects least privilege and the repository's dependency/workflow security rules.

## Phase 13 — Advanced agent UX

**Purpose:** Improve sophisticated workflows after correctness and the core product are
established. These are optional enhancements, not prerequisites for useful early releases.

**Capability areas:** Batch transactions, preview/dry-run, semantic scene diff, profiler
integration, richer visual verification, test-runner integration (for example, GUT),
asset/import workflows, higher-level safe refactoring, performance captures, and richer project/domain
automation. Each capability reuses existing safety, runtime, permission, and outcome
contracts; a preview must not be mistaken for proof that a later application is safe.

**Excluded:** New bypass paths, blanket atomicity/rollback claims for batches, and breadth
without a concrete agent workflow.

**Exit:** Each selected enhancement demonstrates an improved end-to-end agent workflow
with applicable regression and permission coverage, without weakening existing guarantees.
The entire optional catalog need not ship together or become a condition of v1.0.

## Tentative release milestones

Versions express direction, not delivery promises or dates. Every release still requires
all applicable constitutional gates; a version label cannot waive an unmet phase exit.

| Milestone | Likely scope | Evidence required |
| --- | --- | --- |
| **v0.1** | Phases 1–3: trustworthy GDScript editing over MCP | Real-agent A–E workflows on the reusable core; baseline security, export isolation, compatibility and installation guidance |
| **v0.2** | Phases 4–5: scenes, Resources, project configuration | Native transaction, conflict, Undo/Redo, and persistence coverage for supported authoring operations |
| **v0.3** | Phases 6–7: runtime and diagnostics/navigation | Repeatable edit/run/inspect/verify and source-aware cross-file intelligence |
| **v0.4** | Phase 8: native debugger | Real breakpoints, stepping, frames, and variables |
| **v0.5** | Phases 9–10: multi-session/concurrency and expanded security model | Cross-target isolation, concurrent conflict handling, and verified permission boundaries |
| **v0.6+** | Phases 11–13: extensibility, distribution, selected advanced UX | Safe extension contracts, predictable installation/upgrades, and demonstrated workflow improvements |
| **v1.0** | Stable public contracts and a dependable open-source product | Proven coherence guarantees, polished installation, documented migrations/security, and a tested compatibility matrix |

## Relationship to Spec Kit

A roadmap phase is **not automatically one Spec Kit feature**. Select individual coherent
capabilities from the current phase and take each through:

**specify → clarify as needed → plan → tasks → implement → verify**

Approved specifications and plans define behavior and architecture; this roadmap does not.
Architecture-changing, mutation-semantic, and security-sensitive planning/review must
record constitutional compliance. Generic template guidance about optional tests or
parallel tasks cannot waive mandatory gates or the delivery policy.

Each individual implementation task remains one focused PR under `AGENTS.md`: satisfy
its dependencies, implement and verify only that task, then stop after opening its PR.
Continue only after review/merge under repository policy, or explicit authorization for
stacked PRs; stacking never combines tasks into one PR. This roadmap itself is a dedicated
documentation change, not a feature specification or a start on Phase 1.

Later-phase needs may inform an earlier architectural boundary, but do not authorize
unnecessary implementation scope. Crates, transports/libraries, ports, wire formats,
module layouts, test frameworks, exact protocol internals, supported version selections,
and release infrastructure remain decisions for evidence-backed feature planning. There
are no scheduled dates or promised versions. If a proposed capability conflicts with
governance, surface the conflict and block it; never weaken an invariant to fit this sequence.
