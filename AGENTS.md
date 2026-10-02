# Agent working agreement

## Authority and scope

Read applicable guidance before changing a component. Resolve conflicts in this order:

1. `.specify/memory/constitution.md`
2. Approved feature specification
3. Approved implementation plan
4. The selected task definition
5. This repository `AGENTS.md`
6. Surrounding-code conventions

Nested guidance cannot waive constitutional safety. On conflict with the constitution,
**stop and surface it**; never silently weaken an invariant. A change of principle requires
a reviewable constitutional amendment and maintainer approval before dependent work.
The approved specification and plan, not this file, determine architecture.

For meaningful features, follow **specify what/why → clarify material ambiguities → plan
and check constitutional compliance → derive testable tasks → implement → verify against
approved artifacts and the constitution**. Verification is a phase, not a presumed command.
Architecture-changing, mutation-semantic, and security-sensitive work must explicitly
record constitutional compliance in planning and review.

## Spec Kit workflow

Before executing repository feature-design workflows involving `/project.specify-next`,
`/speckit.specify`, `/speckit.clarify`, `/speckit.plan`, `/speckit.tasks`,
`/speckit.analyze`, or the prerequisite portion of `/speckit.implement`, agents MUST first
read and follow [SPECKIT_WORKFLOW.md](SPECKIT_WORKFLOW.md).

That document is repository working policy delegated by `AGENTS.md`, with AGENTS.md-level
authority only. It cannot override the constitution, approved specification, approved plan,
or selected task. The one-task-one-PR implementation rules below remain unchanged.

## Complexity gate

Before introducing a new infrastructure layer, process gate, approval mechanism, dependency,
abstraction, service, workflow, security control, or operational prerequisite, agents MUST
record in the existing plan or review the concrete current requirement or failure mode,
the simplest credible alternative, why existing mechanisms are insufficient, the ongoing
implementation, maintenance, operational, or contributor cost, and why that cost is justified
now. Without that justification, the complexity MUST NOT be introduced. Apply
[Principle XIII](.specify/memory/constitution.md#xiii-justify-complexity-with-concrete-present-risk);
generic best practice is not sufficient justification. `[P]`, security-sensitive work, and
“future extensibility” MUST NOT bypass this gate.

When both establish the same required guarantee, behavioral evidence SHOULD be preferred
over infrastructure ceremony, including provider-specific CI or runner requirements.
Proportional security means identifying the actual failure or attacker capability a control
prevents and requiring demonstrated additional protection to justify its ongoing cost, not
weakening editor coherence, preservation of human work, transaction verification, confinement,
truthful diagnostics, or other constitutional safety invariants.

An implementation-readiness blocker MUST cite (1) exact existing requirement provenance,
(2) a concrete in-scope product failure or capability increase, (3) an actor/environment
inside the supported threat model, and (4) demonstrated or source-established reachable
evidence. If any element is absent, classify the issue as a non-blocking limitation,
hardening opportunity, future work, or tooling/environment limitation. The burden of proof
is on promotion to blocker. A stronger hypothetical isolation property MUST NOT silently
become a product requirement merely because it simplifies proof.

## One Spec Kit task, one PR

Every completed individual Spec Kit implementation task in the active approved
`tasks.md` MUST be delivered as its own pull request. Commands, reasoning steps, and subagent work are not separate Spec Kit tasks.
Do not batch prerequisites: finish and deliver each separately. Governance/bootstrap
changes may form one coherent dedicated PR rather than artificial task PRs.

[PROJECT_STATUS.md](PROJECT_STATUS.md) primarily tracks product/feature/task truth. Update
it when feature/phase lifecycle state or current-task position materially changes. The
active feature's `tasks.md` remains authoritative for task completion; GitHub PRs identify
delivery and may carry acceptance evidence.
`ROADMAP.md` holds long-term direction and exit gates, not routine task progress.
Normally include a status transition in the task PR that causes it, alongside its
`tasks.md` update; no separate status-only PR is required. Feature completion does not
imply roadmap-phase completion: that phase's own exit gates must be satisfied.

1. Select exactly one task ID; confirm its approved feature, specification, plan, acceptance criteria, and satisfied dependencies. If a prerequisite remains, do not silently implement it in this task.
2. Inspect existing changes, fetch the base, and branch `task/<task-id>-<short-description>` from the updated primary development branch (`main` here), unless repository policy sets another base. Preserve unrelated work; use isolation if needed.
3. Implement only that task and directly necessary tests/docs. Run applicable required validation; mark **only that completed task** in `tasks.md`. If safety, acceptance criteria, or required validation cannot be met, report the blocker instead of claiming completion.
4. Inspect status and the complete intended diff; stage explicit paths, inspect the staged diff, and make a focused commit. Do not commit secrets, transient build/editor state, or unrelated user changes.
5. Push the dedicated branch and open a GitHub PR to the selected base. Include feature/spec reference, task ID/title, what and why, validation, applicable constitutional considerations, and known limitations/follow-up. Follow-up does not excuse unmet acceptance criteria.
6. Report the PR number/URL and validation, then **STOP**. Do not automatically merge or start another task. Proceed only after this PR is reviewed/merged, or the user explicitly authorizes a stacked-PR workflow. Stacking still means one task and one PR per task.

After any PR is confirmed merged, its source branch MUST be deleted from its remote
and from the working clone if present. Switch off the branch before local deletion;
preserve unmerged work and do not disrupt another worktree or dependent open PR.
If safe deletion is blocked, report the blocker rather than discard work or silently
skip cleanup. Verify branch deletion and include it in the merge report. This cleanup
requirement does not authorize automatic merging.

Never force-push without explicit authorization, rewrite shared history, auto-merge, bypass failing required checks, or silently discard either side of a conflict. Do not assume CI or branch protections enforce these rules.

Constrain `/speckit.implement` to **one named task**. Its generic implement-all instruction
does not override this policy: leave all other tasks pending, even if marked `[P]`.
`[P]` is dependency metadata, not permission to batch tasks or bypass the post-PR gate.
If the command cannot honor this scope, implement only the selected task against approved
artifacts without invoking implement-all. Keep applicable mandatory gates/hooks; a template
calling tests optional cannot waive constitutional test gates.

On task branches, explicitly verify the active feature directory before using installed
Spec Kit helpers. Task branch names do not select it: the helper checks
`SPECIFY_FEATURE_DIRECTORY`, then `.specify/feature.json`; `SPECIFY_FEATURE` identifies
a feature but does not set its directory. Set `SPECIFY_FEATURE_DIRECTORY` to the approved
directory when needed. For read-only inspection, if installed, use
`.specify/scripts/bash/check-prerequisites.sh --json --paths-only` with a verified override.
`--paths-only` does not persist the pointer; ordinary checks may persist the override.
Do not require an untracked local helper on a clone without it.

### Completion versus delivery state

Task and feature completion MUST be determined by their approved acceptance criteria and
required evidence, not by GitHub review or merge state.

When a task satisfies its specification, plan, task acceptance criteria, required
validation, the implementation-shape completion gate below, and applicable constitutional
gates on its current delivery head, it MUST be marked complete (`[X]`) even if its pull
request is still open or unmerged. If that task completes the feature, the feature MUST
likewise be recorded as complete.

Task completion evidence is not itself a task-definition change. Apply the delegated
[analysis-currentness policy](SPECKIT_WORKFLOW.md#g-recognize-current-analysis-before-implementation)
to distinguish delivery-only updates from changes to approved remaining obligations.

`Awaiting review`, `awaiting merge`, `in review`, `PR open`, and similar GitHub states
MUST NOT be used as task or feature lifecycle states and MUST NOT keep otherwise-complete
work marked pending or in implementation.

GitHub state is delivery metadata only. Status documentation MAY identify the PR carrying
completed work, but MUST keep that separate from completion state. After a completed task
PR merges, do not require a dedicated status-only PR solely to change `PR open` to
`PR merged` unless that metadata is materially useful.

For example:

- `T008 — Complete | Delivery: PR #18 open`
- `Feature 001 — Complete`

Do not use forms such as:

- `T008 — Pending merge`
- `Feature 001 — Implementation, awaiting PR`
- `Current task — T008 awaiting review/merge`

Merge itself is not acceptance evidence and MUST NOT be treated as the event that makes a
task or feature correct.

If the delivery head later changes in a way that can invalidate previously satisfied
acceptance criteria or evidence, reevaluate the affected acceptance. Changes that do not
affect those requirements do not make completed work incomplete merely because the PR head
changed.

Completion state and authorization to begin dependent work are separate concerns. The
one-task-one-PR sequencing rule remains unchanged: dependent implementation waits for the
completed task's PR to merge unless the maintainer explicitly authorizes a stacked-PR
workflow. Stacking still means one task and one PR per task.

### Implementation-shape completion gate

Every implementation task MUST receive a proportionate implementation-shape review before
being marked complete. Passing tests, linting, formatting, and behavioral acceptance is
not by itself sufficient to mark a task complete if it introduced clear accidental
structural complexity. This review applies
[Principle XIII](.specify/memory/constitution.md#xiii-justify-complexity-with-concrete-present-risk)
within current-task completion; it is not a new Spec Kit task, a separate refactor PR by
default, an approval or CI gate, or an architecture-review ceremony. The objective is
clear responsibility and the smallest necessary surface, not stylistic purity.

A materially changed production file approaching roughly 800–1000 lines should trigger
an explicit cohesion/ownership review. A substantially larger file is a stronger review
signal, but size alone does not require splitting: it is not a hard LOC limit or
compliance metric. Several independently nameable responsibilities also trigger review,
even in smaller modules. Diagnostic examples include request/input models, evidence
models, validation, lifecycle/state machines, terminal outcome reduction,
transport/protocol, filesystem/persistence, and native/editor integration; these are not
a required module taxonomy.

Before completing a materially changed implementation task, answer proportionately:

1. What coherent responsibility does each materially changed production module own?
2. Are multiple independently nameable responsibilities combined only because they belong
   to the same feature?
3. Which new declarations truly need their current visibility?
4. Are public APIs exposed only because a future task might use them?
5. Are several booleans representing one lifecycle/state concept and allowing impossible
   combinations?
6. Are there speculative enum variants, extension points, hooks, or abstractions without
   a current requirement or current caller?
7. Are safety/evidence paths using `unwrap`/`expect` where malformed or unavailable
   evidence can practically be represented as a typed failure?
8. Would a simple responsibility-based split now materially reduce the implementation
   or review cost of the next dependency-ready task?

These questions guide judgment, not a new checklist process; they do not require a
written report for every trivial change.

New declarations MUST use the narrowest visibility required by current consumers. A
possible future consumer or later task is not by itself justification for making an API
public today. Public API carries ongoing maintenance/compatibility cost under Principle
XIII; retain public contracts genuinely needed by the current task without artificial
encapsulation.

When simple behavior-preserving cleanup of accidental complexity introduced by the
current task is concretely justified, agents MUST perform it within that task before
marking it complete rather than automatically defer it as future refactoring. Examples,
where concretely justified:

- Split a large feature module into responsibility-based submodules, retaining the same
  architecture, behavior, and external contract.
- Replace several lifecycle booleans with one small state enum.
- Remove an unused speculative public variant until there is a current requirement.
- Replace a safety/evidence-path `unwrap`/`expect` with typed failure where practical.

Do not extract generic frameworks, add traits solely for extensibility, target one type
per file or arbitrary small files, rewrite architecture, introduce layers merely to look
cleaner, refactor unrelated pre-existing code, or build speculative reusable infrastructure.
A split must clarify existing responsibilities; file size alone does not justify a new
framework.

The selected task remains exactly one task and one PR:
**review and proportionate cleanup → mark selected task `[X]` → publish task PR → STOP**.
Do not create another Spec Kit task merely to refactor the selected task; a separate
refactoring task requires genuinely separate, independently valuable scope, not automatic
deferral of completing this task correctly. Review does not authorize work owned by the
next task.

### Task granularity and review

**One-task-one-PR does NOT mean one-file-one-task, one-symbol-one-task, or
one-layer-one-task.** Spec Kit implementation tasks MUST be sized as meaningful,
independently reviewable implementation increments. Prefer coherent capability slices
over file-, symbol-, layer-, or command-oriented fragments.

A task SHOULD normally:

- deliver one clear capability, behavior, or independently verifiable architectural increment;
- include the directly required tests and documentation needed to prove that increment;
- be reviewable as one focused PR;
- have acceptance criteria sufficient to determine whether it is complete.

Do not create separate tasks merely to:

- create one file;
- add one enum, struct, class, method, schema, or similar implementation fragment;
- add tests that naturally belong with the implementation being tested;
- add documentation required by the same implementation;
- split work solely because it touches different files or components.

Setup/foundation work MAY be a separate task only when it establishes an independently
useful and verifiable boundary needed by later work. Otherwise, combine setup with the
first capability that consumes it.

Apply this review heuristic:

> After this task is merged, can its contribution be described as a meaningful capability
> or verifiable architectural outcome rather than merely an implementation fragment?

If not, the task SHOULD normally be consolidated with an adjacent task. A large number
of serial PRs required before the first independently useful vertical slice is a
task-granularity design smell and MUST trigger a task-list granularity review before
implementation begins. There is no hard maximum number of tasks or PRs: the goal is
coherent PR-sized increments, not the fewest tasks possible. Do not combine unrelated
capabilities merely to reduce task count.

**Workflow gate:** After `/speckit.tasks` generates or materially revises `tasks.md`,
the task list MUST receive a granularity review before `/speckit.analyze` and before
implementation begins: **`/speckit.tasks` → granularity review → `/speckit.analyze` →
implementation**.

The review MUST check:

- meaningful PR-sized increments;
- no trivial file/symbol-only tasks;
- tests/docs bundled with their directly related implementation where appropriate;
- a reasonable number of serial PRs before the first useful vertical slice;
- preserved requirement and acceptance-scenario coverage;
- clear dependency ordering.

If granularity is poor, refine `tasks.md` before continuing. This planning gate does not
authorize batching approved tasks into a PR, bypassing dependencies, or weakening the
post-PR STOP rule.

## Shared infrastructure and feature scope

Shared infrastructure SHOULD be named, designed, and owned around its durable
responsibility, execution boundary, or architectural role, not the first roadmap feature
that introduces it. This applies to CI workflows, trusted/live-editor environments and
runner labels, session discovery/routing, authentication, bridge framing, worker
supervision, common errors/outcomes, reusable test harnesses, and generic
project-confinement/security boundaries.

For genuinely shared components, avoid names such as `feature-001-*`, `observation-live`,
or `scene-auth`; prefer accurate responsibility names such as `ci`, `live-editor`,
`session-routing`, `bridge-auth`, or `worker-supervision`. These are examples, not a fixed
naming taxonomy.

Generalize only when the responsibility is already cross-cutting or the approved
architecture clearly assigns it to shared infrastructure, not merely because a future
roadmap phase might reuse it. Feature-specific domain logic SHOULD remain feature-specific
when it belongs to that capability, such as observation collection/classification. Do not
introduce speculative frameworks, indirection, generic modules, or extension systems solely
to avoid feature-specific names.

Use this heuristic:

> If the current feature disappeared tomorrow, would this component still make
> architectural sense as shared infrastructure?

- If yes, prefer responsibility/boundary-based naming and ownership.
- If no, feature-specific naming is probably appropriate.

GitHub Actions workflows SHOULD split by execution/trust/security boundary where practical,
not roadmap feature: ordinary hosted checks in `ci.yml`, trusted GUI Godot checks in
`live-editor.yml`, and release/publishing in a dedicated workflow only when required.
Capability-specific checks should normally be jobs or steps in the appropriate shared
workflow; do not create one workflow per feature when execution and security models match.

Shared/core logic MUST NOT encode feature IDs, task IDs, or roadmap-phase names/numbers
(for example, `T004` or `Feature001`) in public APIs/types, protocol fields, generic
session/security machinery, transport framing, reusable infrastructure, module ownership,
logs, or compatibility surfaces unless the identifier is genuinely part of the product
contract.

Implementation review MUST explicitly ask:

1. Is this artifact genuinely feature-specific?
2. If not, is its name/ownership based on a durable responsibility?
3. Are we accidentally building duplicate infrastructure for a later feature?
4. Conversely, are we over-generalizing capability-specific code without current evidence?

Correct unnecessary feature coupling within the current task before merge when the fix
does not materially expand scope. If it requires architectural redesign outside the
approved task, report the issue rather than silently broadening scope.

## Transaction and editor safety

The boundary is **agent protocol adapter → protocol-independent automation/transaction
core → Godot integration**. MCP is the first adapter, not the definition of the core.
Keep protocol/transport details at the adapter, Godot APIs at the integration boundary,
and every mutation on common transaction/coherence semantics. Keep tools small,
composable, consistently schematized, and discoverable. Route to the intended
project/editor explicitly; refuse ambiguous targets.

Treat **D** (disk source), **R** (loaded Godot Resource/Script), and **B** (visible editor
buffer) as independent authorities. For open-script edits, success requires observed
`D == R == B` across applicable surfaces. Protect unsaved human edits: detect dirty or
conflicting editor state wherever exposed, refuse unresolved conflict or missing safety
observability, and guard stale read-modify-write revisions (or equivalent protection).
No default force overwrite. Godot editor APIs and native editor transaction semantics
MUST be the canonical mutation route wherever possible; alternatives MUST preserve the
same transaction and coherence guarantees. A file write, reload, rescan, parse, or
runtime agreement alone does not prove coherence.

Define preconditions, mutation boundary, synchronization, verification, and deterministic
structured outcomes per mutating operation. Distinguish acceptance, application,
synchronization, and **independently observed** postcondition verification; acknowledgment
is not success. Report revisions, observed or unavailable D/R/B surfaces, dirty/sync state,
diagnostics, UndoRedo participation, and partial application. Distinguish conflict, stale
Resource/buffer, revision mismatch, parse error, timeout, disconnection, unsupported
capability, approval requirement, and unavailable runtime where applicable. Refuse instead
of inventing guarantees; timeout/cancellation does not imply rollback, and retries must
be safe or explicitly documented as non-idempotent. Success must survive Save,
close/reopen, reparse, rescan, and runtime launch wherever applicable.

## Verification and release gates

Layer deterministic, isolated tests: unit tests for pure logic, integration tests for protocol/editor boundaries and changed state transitions, **real-Godot live-editor tests** for coherence. Use explicit deadlines and event-based synchronization, record regression cases for discovered failures, and make diagnostic evidence identify the actual transaction stage and observed D/R/B surfaces. Disk-only checks, headless runtime, and mocks cannot establish visible-buffer coherence.

Real-editor validation MUST use the narrowest evidence set that proves the selected
task: task-owned scenarios and directly affected regressions. During development,
rerun the failed/affected scenario before broader cumulative validation. A complete
unfiltered suite is REQUIRED when the approved task acceptance inherently requires
whole-suite interaction, a changed shared boundary can invalidate that complete
suite (for example authenticated bridge compatibility, native ABI/family loading,
or shared editor-operation ownership), or the task is the feature cumulative/release
gate. Do not invent wider reruns without a concrete affected obligation: an existing
suite or generic regression caution is not justification. A–E, feature-completion,
and release requirements remain unchanged; focused passes cannot establish feature
completion where the approved cumulative gate requires complete final-head evidence.

Mutation-related functionality MUST maintain real-Godot live-editor regression coverage for scenarios A–E, and all applicable gates MUST pass before a mutation feature is complete or released; C applies whenever UndoRedo is claimed:

- **A — Clean open-buffer edit:** D, R, and B converge without human reconciliation.
- **B — Dirty human-buffer conflict:** preserve the unsaved edit; refuse safely or use an explicitly designed resolution workflow, never silently lose it.
- **C — Real Undo/Redo:** verify apply → Undo → Redo transitions across applicable surfaces through Godot's editing history.
- **D — Close/reopen persistence:** reported successful edits survive reopening without stale/reverted state.
- **E — Sequential edit stress:** repeated edits do not disappear, diverge, conceal conflicts, or revert on Save.

Also verify Save/reparse/rescan/runtime durability wherever applicable. Record a justification for an inapplicable surface/scenario; lack of observability is not proof. Document verified guarantees and limitations, not aspirations. State exact supported Godot versions and cover them in CI and real-editor testing before claiming support; do not claim untested versions.

## Conditional implementation guidance

Apply these only when the relevant implementation exists under an approved plan; do not infer a runnable Cargo/Godot project or select crates, SDKs, ports, transport, wire format, package topology, test framework, license, versions, MSRV, bots, or release infrastructure here.

- **Godot:** Use public editor APIs. Observe `ScriptEditor`/`ScriptEditorBase`/`CodeEdit`
  independently of Script resources and disk; `EditorFileSystem` scans/signals and
  `ResourceLoader` caches do not guarantee visible-buffer convergence. Use
  `EditorUndoRedoManager` where appropriate and prove actual Undo/Redo, not registration.
  If a GDScript EditorPlugin is selected, use conventional `addons/<name>/plugin.cfg` and
  `@tool`, clean up registrations/signals/nodes on disable/exit, and keep editor tooling
  out of gameplay authority. Neither `addons/` nor `@tool` excludes tooling from
  production exports: explicitly test the development/export boundary.
- **Rust:** The server should be primarily Rust; a thin GDScript EditorPlugin is the
  expected start, but an alternative MAY be selected only when evidence favors its fit
  to Godot Editor APIs and it preserves the same guarantees. Follow approved package
  boundaries and standard Cargo layout; separate reusable logic from executable wiring
  without gratuitous crates. Use `Result` for recoverable errors, meaningful public
  error contracts/docs (Errors, Panics, Safety where applicable), and avoid unjustified
  `unsafe`; document safety invariants for any justified `unsafe`. Validate serialized/
  schema inputs semantically at adapters. Keep blocking work off async workers; define
  cancellation-safe mutation boundaries explicitly without assuming rollback. Produce
  correlated, structured, redacted diagnostics.
- **MCP:** Select released compatible specification/SDK versions in the plan. Keep
  transport, DTOs, and schemas at the adapter; validate machine-readable inputs and
  structured outputs, advertise only supported capabilities, and distinguish
  protocol/request errors from tool execution failures without losing core outcome detail.
  For stdio, reserve stdout for protocol and send logs to stderr. If HTTP is selected,
  follow the selected specification's Origin/auth requirements.

Once a Cargo application exists, track `Cargo.lock` and use locked dependency resolution in CI; document any toolchain/edition/MSRV claim, test any promised MSRV, and test valid approved feature combinations, not blanket `--all-features`. Run applicable baseline checks (add `--workspace` when coverage of actual workspace members requires it):

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo doc --no-deps --locked
```

Default `cargo test` includes doctests; `cargo test --all-targets` alone does **not**. If using `--all-targets`, run documentation tests separately with `--doc`. Commands are conditional on the later planned Cargo setup, not claims of currently configured checks.

### Rust LSP development tooling

Launch OMP from the repository root. `.omp/lsp.yaml` selects the nested Cargo project:
its POSIX `sh` command enters `mcp-server/` and uses `rustup which rust-analyzer`
to select the analyzer from `mcp-server/rust-toolchain.toml`.
That toolchain declares both `rust-analyzer` and `rust-src` for standard-library navigation.

OMP's protocol `rootUri`/`workspaceFolders` remain the repository root; `linkedProjects`
restricts rust-analyzer's Cargo workspace to `mcp-server/Cargo.toml`. Keep that value
in both `initOptions` and `settings.rust-analyzer`: the later `workspace/configuration`
response replaces initialization settings rather than merging them. Omitting it from
runtime settings causes workspace-discovery errors despite an initially loaded project.
No root Cargo workspace is needed. After changing LSP configuration, use OMP's LSP
`reload` action with `file: "*"`. Run the Cargo baseline commands above from `mcp-server/`.

## Dependencies, security, compatibility

Justify dependencies against requirement and plan; review maintenance, security
advisories, compatible licenses, and provenance. Develop independently from public
APIs/specifications rather than copying unrelated projects. Gate arbitrary
execution/evaluation, force writes, outside-project access, and network access explicitly.
Default to local-only operation and no telemetry; approvals never replace dirty-buffer
detection or transaction safety. Minimize project-content exposure in logs. If GitHub
workflows are introduced, grant tokens least permissions, pin actions to full commit SHAs,
and never run untrusted PR code with secrets.

Treat public tool schemas, structured errors, and outcomes as compatibility surfaces: deliberately version and document migration for breaking changes, evolve additively where safe, and do not preserve unsafe behavior just for compatibility. Keep changes task-scoped and leave unrelated user files alone.
