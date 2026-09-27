---
description: Select the next dependency-correct project feature and create its Spec Kit specification.
---

## User Input

```text
$ARGUMENTS
```

Empty input is valid: select the feature without asking the user to identify or describe it.
Treat non-empty input as optional steering only, never an override of authoritative repository
constraints. If steering conflicts, follow the repository and briefly note the conflict.

## Goal

Select exactly one next dependency-correct feature from repository state, briefly explain the
selection, then delegate specification creation to the existing `/speckit.specify` and STOP.
`/project.specify-next` is repository-specific orchestration, not a specification engine.

## Authoritative Sources

Before selection, read the latest applicable repository state, not remembered conversation
state. At minimum read:

- `PROJECT_STATUS.md`
- `ROADMAP.md`
- `ARCHITECTURE.md`
- `AGENTS.md`
- `.specify/memory/constitution.md`
- all completed feature specifications relevant to the current roadmap phase

Treat the governing files as authoritative in the repository's defined precedence. Follow
their current completion semantics, dependency sequencing, safety invariants, complexity gate,
and architecture/roadmap boundaries rather than duplicating or weakening those policies here.

Read additional existing specifications, plans, tasks, evidence, or source only as needed to
establish completion, dependencies, pending requirements, or duplication of existing capability.

## Feature Selection

Choose the smallest coherent, independently useful feature that materially advances a pending
roadmap requirement while respecting dependencies and completed work. Stay within the current
roadmap phase unless authoritative project state establishes that phase is complete.

Apply Principle XIII as defined by the current constitution. Prefer direct current-need feature
slices; reject features whose main purpose is speculative setup, infrastructure, or
future-proofing. Do not shrink a feature by omitting safety behavior inseparable from offering
that capability.

## Selection Report

Before specification, briefly report exactly these six decision points in conversation only:

1. **Selected feature**
2. **Roadmap gap closed**
3. **Why dependency-correct now**
4. **Why a materially smaller feature would be incomplete/not independently useful**
5. **Adjacent scope deliberately deferred**
6. **Principle XIII check** — notable complexity deliberately not introduced

Do not create a separate selection artifact.

## Delegate to Spec Kit

Construct a natural-language WHAT/WHY description covering the selected problem, capability,
observable outcomes, required safe refusals, roadmap gap, behavior to preserve, and adjacent
scope excluded. Leave implementation decisions to planning.

Read `.omp/commands/speckit.specify.md` and execute its existing instructions in the same OMP
session with the generated feature description as its user input, not the steering above.
Do not merely print `/speckit.specify` for the user to run manually.

`/speckit.specify` remains responsible for specification creation. Do not modify it or duplicate
its feature-numbering, directory, template, pointer, validation, clarification, or extension-hook
logic.

## Failure / Stop Boundary

Before delegation, stop without creating a feature and report the concrete issue if repository
state is materially contradictory or insufficient to determine the current phase or next
dependency-correct direction. Ordinary feature-boundary judgment is not a blocker: make the
best selection yourself when the repository provides enough direction.

After exactly one specification is created, report:

- selected feature;
- created feature directory/spec path;
- roadmap gap advanced.

Then STOP. Do not begin implementation, create a second feature, or automatically run:

- `/speckit.clarify`
- `/speckit.plan`
- `/speckit.tasks`
- `/speckit.analyze`
- `/speckit.implement`
- another `/project.specify-next`
