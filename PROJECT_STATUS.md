# Project Status

## Current

- **Phase:** 1 — Live-editor script coherence
- **Phase state:** In progress
- **Feature:** [001 — Observe Live GDScript Editor State Safely](specs/001-observe-gdscript-state/spec.md)
- **Feature state:** Implementation
- **Completed tasks:** 2 / 8
- **Current task:** T003 — Bounded, project-confined execution

## Roadmap status

| Phase | Status | Current evidence |
| --- | --- | --- |
| 0 — Governance and project foundation | Complete | Merged [constitution v1.0.0](.specify/memory/constitution.md) and [working agreement](AGENTS.md). |
| 1 — Live-editor script coherence | In progress | Feature 001 is in implementation; T001 and T002 are merged. Observation and mutation exit gates remain open below. |
| 2–13 — Later roadmap phases | Pending | Not started; direction and exit criteria remain in [ROADMAP.md](ROADMAP.md). |

## Active feature

The active feature's [tasks.md](specs/001-observe-gdscript-state/tasks.md) remains
authoritative for task completion, dependencies, scope, and acceptance criteria.
Completed tasks below have merged PR evidence. T003 is maintainer-reported work in
progress in a separate worktree, not a merged implementation or completion claim.

| Task | Status | Evidence |
| --- | --- | --- |
| T001 — Reusable observation evidence and classification engine | Complete | [PR #8](https://github.com/Peter-Tam/godot-agent-kit/pull/8), merged into `main`. |
| T002 — Authenticated, source-free local editor session routing and safe addon lifecycle | Complete | [PR #10](https://github.com/Peter-Tam/godot-agent-kit/pull/10), merged into `main`. |
| T003 — Bounded, project-confined observation execution through the local caller | In progress | Known active work; no merged completion evidence. |
| T004 — Clean-open D/R/B observation end to end | Pending | No merged completion evidence. |
| T005 — Document-attributed unsaved/divergent and changing-document observations | Pending | No merged completion evidence. |
| T006 — Source-attributed multi-session routing and live interruption outcomes | Pending | No merged completion evidence. |
| T007 — Closed, invalid, and partly observable document inspection | Pending | No merged completion evidence. |
| T008 — Feature-wide non-interference and verified compatibility/evidence baseline | Pending | No merged completion evidence. |

**Remaining for Feature 001:** Complete and merge T003–T008 with their required
per-task validation and documentation, including T008's cumulative read-only
acceptance and verified compatibility/evidence baseline. The feature is not complete.

## Phase 1 exit gates

D = disk source; R = loaded Godot Resource/Script; B = visible editor buffer.

| Gate | Status | Required evidence / boundary |
| --- | --- | --- |
| D/R/B observation foundation | In progress | Feature 001: semantics and source-free routing are delivered; end-to-end observation and cumulative acceptance remain pending. |
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
