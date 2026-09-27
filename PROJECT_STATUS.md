# Project Status

## Current

- **Phase:** 1 — Live-editor script coherence
- **Phase state:** In progress
- **Feature:** [002 — Safely Edit Open GDScript](specs/002-edit-open-gdscript/spec.md)
- **Feature state:** Tasks derived — granularity review passed; analysis pending
- **Tasks:** 0 / 5 complete; implementation not started
- **Previous feature:** [001 — Observe Live GDScript Editor State Safely](specs/001-observe-gdscript-state/spec.md) — Complete; 8 / 8 tasks implemented and merged
- **Current task:** None — no implementation task selected
- **Mutation A–E and edit durability:** Pending

## Roadmap status

| Phase | Status | Current evidence |
| --- | --- | --- |
| 0 — Governance and project foundation | Complete | Merged [constitution v1.1.0](.specify/memory/constitution.md) and [working agreement](AGENTS.md). |
| 1 — Live-editor script coherence | In progress | Feature 001 is complete. Feature 002's design and five-task decomposition are complete; task-granularity review passed. Analysis, implementation, mutation A–E and durability evidence remain pending; no mutation support is claimed. |
| 2–13 — Later roadmap phases | Pending | Not started; direction and exit criteria remain in [ROADMAP.md](ROADMAP.md). |

## Active feature

Feature 002's [specification](specs/002-edit-open-gdscript/spec.md) and
[requirements-quality checklist](specs/002-edit-open-gdscript/checklists/requirements.md)
define a one-script, already-open editing capability with stale-write protection,
independently verified D/R/B convergence, native Undo/Redo, and durability.
The [implementation plan](specs/002-edit-open-gdscript/plan.md) is now **design-complete**,
with a [data model](specs/002-edit-open-gdscript/data-model.md),
[caller contract](specs/002-edit-open-gdscript/contracts/edit-api.md),
[private bridge contract](specs/002-edit-open-gdscript/contracts/bridge-protocol.md),
[native primitive contracts](specs/002-edit-open-gdscript/contracts/native-integration.md)
and [verification guide](specs/002-edit-open-gdscript/quickstart.md).
The [selected design](specs/002-edit-open-gdscript/research.md#11-concrete-native-design-and-resumed-planning)
uses the existing Rust/core and private bridge, standard GDExtension, and narrow
engine APIs for guarded target-document saved-state finalization and exact-source
GDScript validation. Core policy and independent verification remain in Rust.

Completed [native research](specs/002-edit-open-gdscript/research.md#9-native-integration-continuation-c1c5)
and its negative results remain evidence, not A–E acceptance. The later
tooling-policy interruption remains an uncompleted probe, not a native failure;
no bypass or equivalent rephrased retry occurred. The existing
[concurrency audit](specs/002-edit-open-gdscript/research.md#10-phase-1-concurrency-boundary-audit-and-corrected-planning-decision)
still excludes arbitrary non-cooperating same-inode serialization while preserving
fresh stale/dirty checks, exact identity/confinement and non-success on invalidation.

`/speckit.plan` completed its design artifacts and constitutional planning check.
The subsequent [task list](specs/002-edit-open-gdscript/tasks.md) contains five
pending PR-sized increments, with its required
[granularity review](specs/002-edit-open-gdscript/tasks.md#granularity-review) passed.
Three verifiable core/native foundations precede the first complete caller edit;
US2–US4 safety and history are bundled with US1, followed by cumulative US5 acceptance.
All 22 requirements, 26 scenarios and eight success criteria retain explicit owners.

Analysis has not run, and no implementation task is selected or started.
No product implementation, engine patch, mutation acceptance or support claim was
produced by task generation. Reviewed-artifact/analysis and one-task/one-PR gates
remain required. **Feature 002 is not implemented or complete.**
Delivery metadata: design and task-generation changes are carried by existing
[PR #24](https://github.com/Peter-Tam/godot-agent-kit/pull/24), independently of feature completion.

## Completed observation foundation

Feature 001's [tasks.md](specs/001-observe-gdscript-state/tasks.md) remains
authoritative for its task completion, dependencies, scope, and acceptance criteria.
T001–T008 are complete; all eight task PRs are merged, including [T008 PR #18](https://github.com/Peter-Tam/godot-agent-kit/pull/18).
T008's [post-rebase real-editor evidence and completion review](specs/001-observe-gdscript-state/quickstart.md#29-rebased-t008-real-editor-acceptance-and-completion-2026-09-27)
preserves all 13 groups and 194 cases, including twenty-read 6/8/6
non-interference and native prior-history preservation. The complete suite ran
on the maintainer-operated macOS 26.6.2 arm64 / Godot 4.7.2 / Rust 1.98.1
environment, with independent witnesses, privacy/export/timing evidence and
recorded provenance. The exercised implementation is unchanged and ordinary
hosted native/workflow CI passes; all substantive T008 requirements are met.
Dedicated GUI CI is optional future automation, not a completion prerequisite.
Completion of the observation foundation is distinct from PR merge, broader
version/platform support and Phase 1 mutation capabilities.

| Task | Status | Evidence |
| --- | --- | --- |
| T001 — Reusable observation evidence and classification engine | Complete | [PR #8](https://github.com/Peter-Tam/godot-agent-kit/pull/8), merged into `main`. |
| T002 — Authenticated, source-free local editor session routing and safe addon lifecycle | Complete | [PR #10](https://github.com/Peter-Tam/godot-agent-kit/pull/10), merged into `main`. |
| T003 — Bounded, project-confined observation execution through the local caller | Complete | [PR #12](https://github.com/Peter-Tam/godot-agent-kit/pull/12), merged into `main`; source branch deletion verified. |
| T004 — Clean-open D/R/B observation end to end | Complete | [PR #13](https://github.com/Peter-Tam/godot-agent-kit/pull/13), merged into `main`; source branch deletion verified. |
| T005 — Document-attributed unsaved/divergent and changing-document observations | Complete | [PR #15](https://github.com/Peter-Tam/godot-agent-kit/pull/15), merged into `main`; source branch absence verified locally and remotely. |
| T006 — Source-attributed multi-session routing and live interruption outcomes | Complete | [PR #16](https://github.com/Peter-Tam/godot-agent-kit/pull/16), merged into `main`; source branch absence verified locally and remotely. |
| T007 — Closed, invalid, and partly observable document inspection | Complete | [PR #17](https://github.com/Peter-Tam/godot-agent-kit/pull/17), merged into `main`; source branch absence verified locally and remotely. |
| T008 — Feature-wide non-interference and verified compatibility/evidence baseline | Complete | [PR #18](https://github.com/Peter-Tam/godot-agent-kit/pull/18), merged into `main`; [complete maintainer-operated real-editor evidence and final review](specs/001-observe-gdscript-state/quickstart.md#29-rebased-t008-real-editor-acceptance-and-completion-2026-09-27), with ordinary hosted native/workflow CI. |

**Feature 001 completion:** The observation foundation is complete and T008 is
checked in `tasks.md`. No substantive behavioral/evidence requirement remains
missing. This completion does not authorize mutation implementation. A protected
GitHub GUI environment, registered/ephemeral runner,
manual dispatch or GUI-CI result is not required. The existing workflow remains
[optional automation](.github/README.md); real-editor acceptance itself remains
mandatory, with claims limited to the documented exercised environment.

## Phase 1 exit gates

D = disk source; R = loaded Godot Resource/Script; B = visible editor buffer.

| Gate | Status | Required evidence / boundary |
| --- | --- | --- |
| D/R/B observation foundation | Complete | T001–T008 satisfy the observation acceptance matrix, including real-editor independent witnesses, twenty-read/history preservation and hosted checks. Observation evidence does not establish mutation safety. |
| A — Clean open-buffer edit | Pending | After an actual edit, D, R, and B converge without human reconciliation. |
| B — Dirty human-buffer conflict | Pending | An attempted edit preserves unsaved human work and refuses safely or uses an explicitly designed resolution workflow. |
| C — Real Undo/Redo | Pending | Apply → Undo → Redo produces verified transitions through Godot's editing history across applicable surfaces. |
| D — Close/reopen persistence | Pending | Successful edits survive closing and reopening without stale or reverted state. |
| E — Sequential edit stress | Pending | Repeated edits do not disappear, diverge, conceal conflicts, or revert on Save. |
| Save/reparse/rescan/runtime durability | Pending | Successful edits survive these actions wherever applicable. |

A–E and applied-edit durability are outside Feature 001's observation-only scope;
its observation and history-preservation checks do not satisfy mutation gates.
**Completing Feature 001 does not complete Phase 1.** Phase 1 remains **In progress**
until its own [roadmap exit criteria](ROADMAP.md#phase-1--live-editor-script-coherence)
are met, including real-editor mutation proof and documented limitations/non-success
outcomes.
