# Project Status

## Current

- **Phase:** 1 — Live-editor script coherence
- **Phase state:** In progress
- **Feature:** [001 — Observe Live GDScript Editor State Safely](specs/001-observe-gdscript-state/spec.md)
- **Feature state:** Implementation
- **Implemented tasks:** 5 / 8
- **Merged tasks:** 4 / 8
- **Current task:** T005 — Document-attributed unsaved/divergent and changing-document observations; implemented and locally validated on `task/T005-dirty-changing-observation`, awaiting PR review/merge

## Roadmap status

| Phase | Status | Current evidence |
| --- | --- | --- |
| 0 — Governance and project foundation | Complete | Merged [constitution v1.0.0](.specify/memory/constitution.md) and [working agreement](AGENTS.md). |
| 1 — Live-editor script coherence | In progress | Feature 001 is in implementation; T001–T004 are merged. T005's dirty/changing-document slice is implemented and locally validated, awaiting review/merge. Remaining observation and mutation gates stay open below. |
| 2–13 — Later roadmap phases | Pending | Not started; direction and exit criteria remain in [ROADMAP.md](ROADMAP.md). |

## Active feature

The active feature's [tasks.md](specs/001-observe-gdscript-state/tasks.md) remains
authoritative for task completion, dependencies, scope, and acceptance criteria.
T001–T004 are complete and merged, including [T004 PR #13](https://github.com/Peter-Tam/godot-agent-kit/pull/13).
T005 is checked as implemented in `tasks.md`; its dedicated task branch carries
the [native/live/export evidence](specs/001-observe-gdscript-state/quickstart.md#25-t005-dirty-and-changing-document-evidence-2026-09-27)
and awaits PR review/merge. T006 remains pending under the one-task-one-PR gate;
implementation completion is not merged delivery.

| Task | Status | Evidence |
| --- | --- | --- |
| T001 — Reusable observation evidence and classification engine | Complete | [PR #8](https://github.com/Peter-Tam/godot-agent-kit/pull/8), merged into `main`. |
| T002 — Authenticated, source-free local editor session routing and safe addon lifecycle | Complete | [PR #10](https://github.com/Peter-Tam/godot-agent-kit/pull/10), merged into `main`. |
| T003 — Bounded, project-confined observation execution through the local caller | Complete | [PR #12](https://github.com/Peter-Tam/godot-agent-kit/pull/12), merged into `main`; source branch deletion verified. |
| T004 — Clean-open D/R/B observation end to end | Complete | [PR #13](https://github.com/Peter-Tam/godot-agent-kit/pull/13), merged into `main`; source branch deletion verified. |
| T005 — Document-attributed unsaved/divergent and changing-document observations | In review | Implemented and locally validated; [native/GUI/export evidence](specs/001-observe-gdscript-state/quickstart.md#25-t005-dirty-and-changing-document-evidence-2026-09-27). Dedicated branch `task/T005-dirty-changing-observation` awaits review/merge. |
| T006 — Source-attributed multi-session routing and live interruption outcomes | Pending | No merged completion evidence. |
| T007 — Closed, invalid, and partly observable document inspection | Pending | No merged completion evidence. |
| T008 — Feature-wide non-interference and verified compatibility/evidence baseline | Pending | No merged completion evidence. |

**Remaining for Feature 001:** Review and merge T005, then complete and merge
T006–T008 with their required per-task validation and documentation, including
T008's cumulative read-only acceptance and verified compatibility/evidence
baseline. The feature is not complete.

## Phase 1 exit gates

D = disk source; R = loaded Godot Resource/Script; B = visible editor buffer.

| Gate | Status | Required evidence / boundary |
| --- | --- | --- |
| D/R/B observation foundation | In progress | Semantics, source-free routing, bounded/confined execution and clean-open D/R/B observation are merged. T005 adds locally verified dirty/changing-document observation, awaiting review/merge. Remaining stories, cumulative acceptance and trusted GUI CI are still pending. |
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
