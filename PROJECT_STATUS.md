# Project Status

## Current

- **Phase:** 1 — Live-editor script coherence
- **Phase state:** In progress
- **Feature:** [001 — Observe Live GDScript Editor State Safely](specs/001-observe-gdscript-state/spec.md)
- **Feature state:** Implementation
- **Implemented tasks:** 7 / 8
- **Merged tasks:** 7 / 8
- **Current task:** T008 — Cumulative observation acceptance; local implementation and validation pass on `task/T008-cumulative-observation-evidence`, but completion is blocked on actual trusted GUI CI

## Roadmap status

| Phase | Status | Current evidence |
| --- | --- | --- |
| 0 — Governance and project foundation | Complete | Merged [constitution v1.0.0](.specify/memory/constitution.md) and [working agreement](AGENTS.md). |
| 1 — Live-editor script coherence | In progress | T001–T007 are merged. T008's full local observation/privacy/export matrix passes; required trusted GUI CI is unavailable. Feature completion and Phase 1 mutation gates remain open. |
| 2–13 — Later roadmap phases | Pending | Not started; direction and exit criteria remain in [ROADMAP.md](ROADMAP.md). |

## Active feature

The active feature's [tasks.md](specs/001-observe-gdscript-state/tasks.md) remains
authoritative for task completion, dependencies, scope, and acceptance criteria.
T001–T007 are complete and merged, including [T007 PR #17](https://github.com/Peter-Tam/godot-agent-kit/pull/17).
T008's [rebased local evidence](specs/001-observe-gdscript-state/quickstart.md#29-rebased-t008-local-verification-and-review-gate-2026-09-27)
preserves all 13 groups and 194 cases, including twenty-read non-interference.
Merged PR #19 supplies the trusted reviewed-PR dispatch infrastructure without
any workflow diff in T008. It remains unchecked: independent exact-head review,
protected environment/runner provisioning, environment approval and successful
actual trusted GUI CI remain outstanding. Local or hosted verification is not
task completion, merged delivery or version support.

| Task | Status | Evidence |
| --- | --- | --- |
| T001 — Reusable observation evidence and classification engine | Complete | [PR #8](https://github.com/Peter-Tam/godot-agent-kit/pull/8), merged into `main`. |
| T002 — Authenticated, source-free local editor session routing and safe addon lifecycle | Complete | [PR #10](https://github.com/Peter-Tam/godot-agent-kit/pull/10), merged into `main`. |
| T003 — Bounded, project-confined observation execution through the local caller | Complete | [PR #12](https://github.com/Peter-Tam/godot-agent-kit/pull/12), merged into `main`; source branch deletion verified. |
| T004 — Clean-open D/R/B observation end to end | Complete | [PR #13](https://github.com/Peter-Tam/godot-agent-kit/pull/13), merged into `main`; source branch deletion verified. |
| T005 — Document-attributed unsaved/divergent and changing-document observations | Complete | [PR #15](https://github.com/Peter-Tam/godot-agent-kit/pull/15), merged into `main`; source branch absence verified locally and remotely. |
| T006 — Source-attributed multi-session routing and live interruption outcomes | Complete | [PR #16](https://github.com/Peter-Tam/godot-agent-kit/pull/16), merged into `main`; source branch absence verified locally and remotely. |
| T007 — Closed, invalid, and partly observable document inspection | Complete | [PR #17](https://github.com/Peter-Tam/godot-agent-kit/pull/17), merged into `main`; source branch absence verified locally and remotely. |
| T008 — Feature-wide non-interference and verified compatibility/evidence baseline | Blocked | Full post-rebase local GUI/native/workflow/privacy/export gates pass; [fresh evidence and review/GUI CI gate](specs/001-observe-gdscript-state/quickstart.md#29-rebased-t008-local-verification-and-review-gate-2026-09-27). Exact-head independent approval and actual protected GUI CI remain required. |

**Remaining for Feature 001:** Obtain independent approval of the exact current
head of [PR #18](https://github.com/Peter-Tam/godot-agent-kit/pull/18), provision
the protected `live-editor` environment and isolated runner, then dispatch the
trusted `main` workflow against that reviewed SHA. Separate environment approval
and a successful complete GUI run are required before final evidence/status
completion. Merged [PR #19](https://github.com/Peter-Tam/godot-agent-kit/pull/19)
provides this unmerged-PR dispatch model; no merge-first gate remains.
No protection bypass, automatic dispatch/merge or supported-version claim is
authorized. T008 and the feature remain incomplete.

## Phase 1 exit gates

D = disk source; R = loaded Godot Resource/Script; B = visible editor buffer.

| Gate | Status | Required evidence / boundary |
| --- | --- | --- |
| D/R/B observation foundation | In progress | T001–T007 observation behavior is merged. T008's cumulative local acceptance passes, including twenty-read history preservation; actual trusted GUI CI and T008 delivery remain incomplete. |
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
