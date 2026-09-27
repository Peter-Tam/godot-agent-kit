---
description: Select the next dependency-correct project feature and create its Spec Kit specification.
---

## User Input

```text
$ARGUMENTS
```

You MUST consider non-empty user input as optional steering context.

The user is not required to identify, name, scope, or describe a feature.
Empty input is valid: select the next feature from repository state without asking for a description.

## Goal

Determine the next Spec Kit feature from the repository's latest authoritative state, then
invoke the existing `/speckit.specify` command with a natural-language description of that
selected feature.

Create exactly one feature.

Do not ask the user to manually choose the next feature unless the repository contains a
genuine unresolved product decision that cannot be derived from its existing roadmap,
status, specifications, architecture, and constitution.

Do not treat ordinary feature-boundary judgment as a reason to ask the user.

## Read the current project state

Before selecting a feature, inspect the repository's latest applicable sources.

At minimum read:

- `PROJECT_STATUS.md`
- `ROADMAP.md`
- `ARCHITECTURE.md`
- `AGENTS.md`
- `.specify/memory/constitution.md`
- all completed feature specifications relevant to the current roadmap phase

Read additional existing specifications, plans, tasks, implementation boundaries, or source
files only when needed to determine:

- what is already complete;
- which dependencies exist;
- which current-phase requirements remain;
- whether a proposed feature would duplicate existing capability.

Use repository state as the source of truth.

Do not rely on remembered conversation state when the repository can answer the question.

## Completion versus delivery

Respect the repository's completion semantics.

A task or feature marked complete by its approved acceptance criteria and evidence remains
complete regardless of historical PR lifecycle wording.

Do not interpret:

- `PR open`
- `awaiting review`
- `awaiting merge`
- `in review`

as evidence that a task or feature is incomplete.

Merge state matters only where repository sequencing requires merged work to serve as the
base for dependent implementation.

Do not select already-completed work merely because historical delivery metadata is stale.

## Select the next feature

Choose the smallest coherent feature that materially advances the **current roadmap phase**
toward its exit criteria while respecting all dependencies.

Do not jump to a later roadmap phase while required current-phase exit criteria remain
unsatisfied.

A roadmap phase is not automatically one Spec Kit feature.

The selected feature MUST:

- deliver a meaningful independently useful capability or behavioral increment;
- have clear user-visible or agent-visible behavior;
- directly close one or more currently pending roadmap requirements or exit gates;
- respect all completed prerequisites;
- fit within the current roadmap phase;
- be large enough to justify its own feature specification;
- avoid bundling unrelated capabilities;
- preserve all existing constitutional safety and coherence invariants.

The feature MUST NOT exist primarily to create setup, abstraction, infrastructure, or
generalization for hypothetical future work.

## Feature-boundary test

Prefer a direct vertical slice over horizontal implementation scaffolding.

Before selecting the feature, test the proposed boundary:

1. Does it produce an independently meaningful capability or verified behavioral result?
2. Does it directly advance a currently pending roadmap requirement?
3. Can it be specified in user/agent-visible WHAT and WHY terms?
4. Would removing speculative future reuse substantially reduce its scope?
5. Does it avoid pulling in later-phase capabilities?

If the answer indicates the proposed feature is mostly setup or future-proofing, choose a
more direct feature boundary.

Do not create one feature per file, module, class, API, protocol layer, or implementation
mechanism.

Do not create an entire roadmap phase as one feature merely for convenience.

## Principle XIII — Justify Complexity With Concrete Present Risk

Apply Constitution Principle XIII aggressively during feature selection.

For every proposed new:

- infrastructure layer;
- abstraction;
- framework;
- service;
- dependency;
- approval mechanism;
- process gate;
- CI topology;
- security control;
- operational prerequisite;
- generic transaction mechanism;
- extension point;

ask:

1. What concrete current requirement or realistic failure mode requires it?
2. What is the simplest credible alternative?
3. Why are existing mechanisms insufficient?
4. What ongoing implementation, maintenance, operational, or contributor cost does it add?
5. Why is that cost justified now?

If those questions do not have concrete answers, do not make that mechanism part of the
selected feature.

Generic best practice, enterprise convention, theoretical defense in depth, speculative
future reuse, or architectural elegance are not sufficient justification.

Reuse existing proven mechanisms and boundaries wherever they already satisfy the current
requirement.

When two feature boundaries provide materially equivalent progress and safety, prefer the
one with:

- fewer moving parts;
- fewer dependencies;
- less operational burden;
- less new infrastructure;
- less maintenance cost.

## Respect architecture without prematurely implementing future phases

Existing architecture boundaries remain authoritative where already established.

However, a responsibility being planned for a later roadmap phase does not authorize
implementing its generalized future form early.

For example:

- do not build a generalized transaction framework merely because a later phase may need one;
- do not add MCP merely because it is the eventual external adapter;
- do not create distribution infrastructure for an unreleased current-phase capability;
- do not add multi-agent coordination for a single-editor current-phase requirement;
- do not generalize capability-specific logic solely for hypothetical reuse.

If the current roadmap requirement can be satisfied through a direct vertical slice using
existing boundaries, prefer that.

## Do not silently weaken safety to make a feature smaller

Smallest coherent feature does not mean weakest feature.

A feature boundary MUST NOT omit behavior that is necessary for the capability to satisfy
existing constitutional safety requirements.

For example, a mutation capability must not be defined as a "happy-path edit" if that would
allow unsaved human work to be overwritten or leave required coherence unverifiable.

When a safety behavior is inseparable from offering the capability at all, include it in the
feature rather than defer it as unrelated future work.

Simplification means removing unnecessary mechanisms, not removing required protections.

## Optional user steering

If `$ARGUMENTS` is non-empty, use it as preference/context during selection.

Examples:

```text
Prefer a smaller vertical slice.
```

```text
Avoid adding new dependencies if existing code can support the feature.
```

```text
Focus on the next mutation requirement.
```

Steering MUST NOT:

- skip required dependencies;
- jump roadmap phases;
- reopen completed work without cause;
- weaken constitutional requirements;
- force unjustified complexity.

If steering conflicts with authoritative repository constraints, follow the repository and
briefly note the conflict.

## Selection report

Before invoking `/speckit.specify`, output a concise selection report containing exactly
these decision points:

1. **Selected feature**
2. **Roadmap gap closed**
3. **Why it is dependency-correct now**
4. **Why a materially smaller feature would be incomplete or not independently useful**
5. **Adjacent scope deliberately deferred**
6. **Principle XIII check** — notable complexity deliberately not introduced

Keep this short.

This is conversational output only.

Do not create a separate:

- selection document;
- feature-ranking table;
- scoring artifact;
- roadmap queue;
- decision database.

Do not compare or rank a large catalog of hypothetical future features unless genuinely
necessary to resolve ambiguity.

## Invoke Spec Kit

After selecting the feature, invoke the existing:

```text
/speckit.specify
```

in the same OMP session.

For this file-based OMP command, read `.omp/commands/speckit.specify.md` and execute its
instructions in this session with the generated feature description as its user input,
not the optional steering input above. Merely printing `/speckit.specify ...` or asking
the user to run it is not invocation.

Construct the natural-language feature description yourself.

The user MUST NOT have to copy the selection report into `/speckit.specify`.

The description passed to `/speckit.specify` should contain enough WHAT and WHY for the
normal Spec Kit command to produce a complete specification.

Include where relevant:

- the user/agent problem being solved;
- the meaningful capability being added;
- required observable behavior;
- required safe-refusal/conflict behavior;
- relevant current-roadmap outcome or gate;
- important existing behavior that must remain preserved;
- major adjacent scope that is explicitly out of scope.

Do not prematurely choose implementation details better left to planning.

In particular, do not select unless already mandated by governing repository artifacts:

- concrete internal APIs;
- Rust structs/enums/modules;
- GDScript class layout;
- exact Godot API call sequence;
- dependency crates/libraries;
- wire-format changes;
- transport mechanics;
- CI infrastructure;
- runner topology;
- task decomposition;
- PR decomposition;
- implementation sequence.

The generated description is an input to `/speckit.specify`, not an implementation plan.

## Delegate rather than duplicate

`/speckit.specify` remains the sole command responsible for normal specification creation.

Do not duplicate its logic for:

- feature numbering;
- feature-directory creation;
- `spec.md` template handling;
- `.specify/feature.json`;
- specification formatting;
- specification quality validation;
- specification clarification markers;
- its extension hooks.

Invoke `/speckit.specify` and let it perform those responsibilities.

Do not modify `/speckit.specify` as part of this command.

## Failure cases

Stop without creating a feature if repository state shows a genuine blocking inconsistency,
such as:

- current roadmap phase cannot be determined;
- required governing artifacts materially contradict each other;
- all current-phase exit requirements appear complete but the phase is still marked active
  without a defined next direction;
- selecting any next feature would require inventing product direction absent from the
  repository.

Report the concrete inconsistency.

Do not manufacture a feature merely to keep the workflow moving.

Ordinary scope judgment does not count as such an inconsistency; make the best
dependency-correct selection yourself.

## Stop boundary

After `/speckit.specify` successfully creates the specification, report:

- selected feature name;
- created feature directory/spec path;
- roadmap requirement or exit gate it advances.

Then STOP.

Do not automatically run:

- `/speckit.clarify`
- `/speckit.plan`
- `/speckit.tasks`
- `/speckit.analyze`
- `/speckit.implement`
- another `/project.specify-next`

Do not begin implementation.

Do not create a second feature.

## Non-goals

This command is not:

- a roadmap manager;
- a feature backlog;
- an implementation orchestrator;
- a planning command;
- a task generator;
- a prioritization scoring engine;
- a project-management framework.

Its only responsibilities are:

```text
select next feature
→ delegate specification to /speckit.specify
```
