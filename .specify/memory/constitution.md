# godot-agent-kit Constitution

godot-agent-kit makes coding-agent interaction with a live Godot Editor safe, coherent,
observable, and predictable. Its value is enabling project changes while the editor stays open
without silently losing human edits, diverging editor/disk state, or reporting success before
convergence. Capability count is not the mission. MCP is the first external protocol, not the
architecture's definition.

MUST, MUST NOT, SHOULD, and MAY have their RFC 2119 meanings. Departures from SHOULD requirements
MUST document rationale and evidence; they MUST NOT weaken a MUST requirement.

## Core Principles

### I. Editor Coherence Is the Correctness Boundary

Mutations MUST treat **D — disk source**, **R — Godot loaded Resource / Script state**, and
**B — visible editor buffer**, such as CodeEdit text, as independent state authorities.
For an open-script edit, success MUST establish `D == R == B` across applicable, observable
surfaces. Writing bytes to disk alone MUST NOT count as mutation success.

Unavailable observations MUST be explicit, not treated as evidence of coherence. If required
coherence cannot be established, the operation MUST return a structured non-success outcome,
such as conflict, stale editor, synchronization timeout, partial application, or unsupported
state. A successful mutation MUST survive ordinary Save, close/reopen, reparse, filesystem
rescan, and runtime launch actions wherever applicable. Explicit failure takes precedence over
false success.

### II. Preserve Human Work and Refuse Conflicts Safely

Human unsaved editor state MUST take priority over unattended mutation. Before changing an
editor-owned artifact, operations MUST detect dirty or conflicting state wherever Godot exposes
it. Unresolved human edits MUST NOT be silently overwritten; default conflict handling MUST
refuse safely and return structured information identifying the conflict. If available APIs
cannot establish that proceeding is safe, the operation MUST refuse rather than assume clean
state.

Read-modify-write operations SHOULD use revision/hash-based optimistic concurrency to detect
changes since observation; alternatives MUST provide equivalent stale-write protection.
Force overwrite MUST NOT be the default. Any future force mechanism MUST be explicit and
separately permissioned.

### III. Use Godot-Native Editor Transactions

Godot editor APIs and editor transaction semantics MUST be the canonical mutation route wherever
possible. Script editing MUST be designed around the live editor; scene and resource editing
MUST preserve normal Godot editor semantics. The editing model MUST NOT rely on
`write file → hope Godot reloads`.

An UndoRedo claim MUST mean actual, observable state reversal and reapplication through Godot's
editing history, not merely an undoable flag. When Godot APIs cannot provide a required
guarantee, the capability MUST expose the limitation instead of reporting success. Alternative
mutation routes MUST meet the same transaction and coherence guarantees.

### IV. Verify Mutations Before Reporting Success

Every mutating operation MUST define its preconditions, mutation boundary, post-mutation
synchronization, verification, and deterministic structured outcomes. Request acceptance,
mutation application, synchronization completion, and verification completion MUST be distinct
states; acceptance or application alone MUST NOT imply success.

Success MUST require observed postconditions, not an acknowledgment from the mutation path.
Outcomes MUST expose relevant revisions, disk synchronization, editor synchronization, Resource
synchronization, dirty state, diagnostics, UndoRedo participation, and partial failures.
Unavailable state MUST be identified explicitly. Retries MUST either be safe or be explicitly
documented as non-idempotent so agents can avoid unsafe repetition.

### V. Earn Unattended Capability Through Safety

Unattended mutation MUST be justified by demonstrated correctness and the applicable release
gates. Safe refusal MUST take precedence over unsafe capability. Approval gates MUST NOT
substitute for dirty-buffer detection, synchronization, transaction safety, or conflict
handling.

Capabilities MUST follow least privilege. Arbitrary code evaluation, process execution, force
writes, filesystem access outside the project, and network access MUST be explicitly gated.
Local-only operation MUST be the default; remote access and telemetry MUST NOT be enabled by
default.

### VI. Gate Releases on Real Editor-Coherence Tests

Mutation-related functionality MUST maintain real-Godot regression coverage for the following
scenarios. These are release gates, not optional demonstrations; C applies whenever UndoRedo
support is claimed.

- **A — Clean open-buffer edit:** Edit a clean script currently open in Godot. Disk, loaded
  Resource, and visible buffer MUST converge without human reconciliation.
- **B — Dirty human-buffer conflict:** Attempt an agent edit while the human has an unsaved
  change to the same source. The human edit MUST remain intact; the operation MUST refuse safely
  or follow an explicitly designed conflict-resolution workflow without silent loss.
- **C — Real Undo/Redo:** Apply → Undo → Redo MUST produce verified transitions across all
  applicable editor state surfaces through Godot's editing history.
- **D — Close/reopen persistence:** An accepted mutation reported successful MUST survive script
  close/reopen without reverting or becoming stale.
- **E — Sequential edit stress:** Repeated edits MUST NOT lose mutations, leave stale buffers,
  revert on Save, diverge editor/disk state, or conceal conflicts.

Tests MUST observe the applicable live editor state. Runtime agreement with disk, or disk-only
validation, MUST NOT count as editor coherence while the visible editor remains stale.

### VII. Keep the Core Protocol-Independent

The architecture MUST conceptually separate
`Agent protocol adapter → automation/transaction core → Godot integration`.
Transactions, revisions, conflicts, coherent state, diagnostics, session routing, and mutation
outcomes MUST NOT depend unnecessarily on MCP. MCP-specific details MUST remain at the adapter
boundary so other protocols, CLI interfaces, test harnesses, or official Godot automation APIs
can reuse the same core semantics. Adapters MUST NOT introduce divergent mutation guarantees.

### VIII. Separate Tooling from Gameplay

Editor plugins, development helpers, runtime probes, and agent bridges are tooling
infrastructure and MUST NOT become authoritative gameplay state. Games MUST NOT be required to
structure gameplay around MCP or store gameplay authority in tooling autoloads. Tooling MUST NOT
unintentionally ship in production builds. Development/export boundaries MUST be explicit and
testable, with verification that production exports respect those boundaries.

### IX. Keep the Tool Surface Small and Composable

Tool design MUST prioritize reliable agent reasoning and task completion with low orchestration
friction, not tool count. Tools SHOULD be small, stable, composable primitives rather than
near-duplicate operations; departures MUST justify a distinct agent need. Public tools MUST use
consistent naming and structured schemas, with capability discovery and progressive disclosure
so agents can identify available operations without navigating an inflated tool catalog.

### X. Make Diagnostics Deterministic and Actionable

Failures MUST be actionable and machine-readable. Structured outcomes MUST distinguish revision
mismatch, dirty editor buffer, stale Resource, stale visible buffer, parse error, synchronization
timeout, disconnected editor, unsupported capability, permission or approval requirement, and
runtime unavailable when applicable. Different failure conditions MUST NOT collapse into generic
success or an indistinguishable error.

Logs and tracing MUST make multi-step editor transactions diagnosable while minimizing exposure
of project contents. Validation evidence MUST identify the surfaces actually observed;
disk-only checks MUST NOT be presented as proof of editor coherence.

### XI. Maintain an Independent Open-Source Implementation

Implementation decisions MUST rest on public Godot APIs and documentation, public MCP
specifications and SDKs, independently defined requirements, interoperability testing, and
observable behavior. Other Godot agent/MCP projects MAY inform user needs, compatibility
expectations, benchmarks, and ecosystem context; project implementation MUST be developed
independently rather than copied from unrelated projects. Dependency licenses and provenance
MUST be documented and compatible with public open-source distribution.

### XII. Put Quality Before Feature Count

Correctness, safety, and maintainability MUST take precedence over capability count and feature
velocity. Concurrency and synchronization changes MUST be treated as correctness-critical;
mutation-semantic changes MUST include integration coverage of affected state transitions.
Features MUST NOT bypass the common transaction/coherence mechanism because direct filesystem
mutation is easier. A capability that cannot preserve this safety contract MUST remain
unsupported until it can.

## Architecture and Compatibility

- The external agent/server side SHOULD be primarily Rust. A thin GDScript EditorPlugin is the
  expected starting point for editor integration; plans MAY select another mechanism when
  evidence favors its fit to Godot Editor APIs and the same guarantees are preserved.
- Protocol/transport handling MUST remain separate from the transaction/coherence core.
  Godot-specific APIs MUST be isolated behind the editor-integration boundary. All mutation
  tools MUST pass through the common transaction/coherence mechanism.
- Multi-editor and multi-project routing MUST refuse safely when the intended target is
  ambiguous; it MUST NOT guess a mutation destination.
- Specific crates, socket implementations, ports, wire formats, and equivalent implementation
  details belong in later specifications or technical plans, not this constitution.
- Supported Godot versions MUST be explicit and covered by appropriate CI and real-Godot
  integration testing. Untested versions MUST NOT be claimed as supported.
- Machine-readable tool schemas, transaction outcomes, and structured error behavior MUST be
  treated as public compatibility surfaces. Backward-incompatible public behavior MUST receive
  deliberate versioning and migration documentation. Evolution SHOULD be additive to reduce
  disruption, but unsafe behavior MUST NOT be preserved solely for compatibility.

## Development Workflow and Quality Gates

Meaningful features MUST follow the Spec Kit, spec-driven sequence:

1. Specify WHAT the feature does and WHY it is needed.
2. Resolve material ambiguities before technical commitment.
3. Create the technical plan and evaluate constitutional compliance.
4. Derive testable tasks with observable acceptance criteria.
5. Implement according to the approved specification and plan.
6. Verify against the specification, plan, tests, and this constitution.

Architecture-changing work, mutation-semantic changes, and security-sensitive features MUST
explicitly record constitutional compliance during planning and review. Implementation
convenience MUST NOT weaken an invariant; a genuine change of principle requires a deliberate
constitutional amendment before conflicting work proceeds.

**Definition of Done for mutation features:** All of the following MUST be satisfied:

- Conflict behavior and synchronization behavior are defined.
- Failure states and partial-application outcomes are structured.
- Accepted mutations receive independent verification against observed post-mutation state;
  acknowledgment alone is not evidence of success.
- Relevant D/R/B surfaces and their transitions are tested in real Godot.
- Save and close/reopen behavior are tested wherever applicable.
- Dirty-human-state behavior is tested wherever applicable.
- UndoRedo behavior is tested if claimed.
- Sequential-edit behavior is tested, and applicable A–E release gates pass.
- Documentation states verified guarantees and known limitations rather than aspirations.

Any inapplicable surface or scenario MUST have a recorded justification; missing observability
MUST NOT be used to claim an unverified guarantee.

## Governance

This constitution is authoritative over conflicting specifications, plans, tasks, and
convenience-driven practices. Maintainers and reviewers MUST check compliance during planning
and before merge; unresolved violations MUST block the affected work. A specification or plan
MUST NOT grant an exception to a constitutional MUST.

Amendments MUST be proposed as reviewable constitution changes with rationale, the proposed
version, affected principles and artifacts, and compatibility or migration consequences.
Changes to core safety/coherence principles MUST explicitly explain why the change is
necessary, what protections change, and what evidence supports the resulting guarantees.
Maintainer approval MUST precede adoption; implementation difficulty alone is not sufficient
rationale to weaken a safety invariant.

The constitution MUST use semantic versioning:

- **MAJOR:** Backward-incompatible governance changes or principle removals/redefinitions.
- **MINOR:** New principles or sections, or materially expanded guidance without incompatible
  redefinition.
- **PATCH:** Clarifications, wording corrections, or other non-semantic refinements.

Amendments MUST retain the original ratification date and update the last-amended date.
Maintainers MUST identify and propagate amendments into affected Spec Kit specifications,
plans, tasks, checklists, and project guidance before dependent implementation proceeds.
Templates and commands consume the constitution at runtime and MUST NOT be rewritten as part
of the constitution-update workflow. Amendment reviews MUST verify that affected artifacts are
aligned or explicitly tracked as blocking dependent work.

**Version**: 1.0.0 | **Ratified**: 2026-09-25 | **Last Amended**: 2026-09-25
