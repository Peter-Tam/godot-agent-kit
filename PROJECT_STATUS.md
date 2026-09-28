# Project Status

## Current

- **Phase:** 1 — Live-editor script coherence
- **Phase state:** In progress
- **Feature:** [002 — Safely Edit Open GDScript](specs/002-edit-open-gdscript/spec.md)
- **Feature state:** Core/patched validation complete; P1 stock saved-state design selected, mutation integration pending
- **Tasks:** 2 / 5 complete
- **Previous feature:** [001 — Observe Live GDScript Editor State Safely](specs/001-observe-gdscript-state/spec.md) — Complete; 8 / 8 tasks implemented and merged
- **Current task:** T003 not started; on hold for independent stock validation/effect confinement and design review
- **Mutation A–E and edit durability:** Pending; no product mutation support or T003 readiness claim

## Roadmap status

| Phase | Status | Current evidence |
| --- | --- | --- |
| 0 — Governance and project foundation | Complete | Merged [constitution v1.1.0](.specify/memory/constitution.md) and [working agreement](AGENTS.md). |
| 1 — Live-editor script coherence | In progress | Feature 001 is complete. Feature 002's edit core and patched read-only native validation are complete. [Stock post-persistence research §14](specs/002-edit-open-gdscript/research.md#14-stock-post-persistence-saved-transition-research-2026-09-28) selects P1 for the saved-state question only: guarded descriptor persistence followed by the actual public incoming-edge ScriptEditor native Callable on stock. T003 is unstarted/on hold for independent stock validation/effect confinement and review; caller mutation, A–E and durability remain pending. |
| 2–13 — Later roadmap phases | Pending | Not started; direction and exit criteria remain in [ROADMAP.md](ROADMAP.md). |

## Active feature

Feature 002's [specification](specs/002-edit-open-gdscript/spec.md) and
[requirements-quality checklist](specs/002-edit-open-gdscript/checklists/requirements.md)
define a one-script, already-open editing capability with stale-write protection,
independently verified D/R/B convergence, native Undo/Redo, and durability.
The [implementation plan](specs/002-edit-open-gdscript/plan.md),
[native contract](specs/002-edit-open-gdscript/contracts/native-integration.md)
and [§14 stock research](specs/002-edit-open-gdscript/research.md#14-stock-post-persistence-saved-transition-research-2026-09-28)
select P1 for the **post-persistence saved-state transition only**. The prior
patched-engine finalizer is a historical baseline, not an approved product
route. The [data model](specs/002-edit-open-gdscript/data-model.md),
[caller contract](specs/002-edit-open-gdscript/contracts/edit-api.md),
[private bridge contract](specs/002-edit-open-gdscript/contracts/bridge-protocol.md)
and [quickstart recipes](specs/002-edit-open-gdscript/quickstart.md) retain
behavioral requirements, but their patched-family-1/finalizer-specific API,
bridge and build instructions are **historical**, not the current T003 stock
recipe. The [current native §3 contract](specs/002-edit-open-gdscript/contracts/native-integration.md#3-primitive-a-guarded-target-document-saved-transition)
selects the stock saved transition. No final production native/bridge wire
schema or editing API has been implemented by this research. Product
mutation targets official stock Godot, addon and bundled standard GDExtension.
Rust retains core policy and independent verification; T002's patched native
validation evidence remains complete and separate from the stock saved-state
decision, and does not establish stock validation/effect confinement.

The [F3 stock-save research](specs/002-edit-open-gdscript/research.md#13-stock-target-save-finalization-research-2026-09-28)
demonstrates successful ordinary stock target Save performs real native saved-state
bookkeeping/history, including a descriptor-backed saver. It also demonstrates
that non-OK custom-saver failures fall through to builtin path writes (observed
outside-project redirection), and target Save applies every open script, changing
an unrelated dirty buffer/history. Direct `ResourceSaver.save` does not complete
the ScriptEditor saved transition. The audited routes therefore do not qualify;
this is **historical F3 evidence**, not a conclusion that every stock route fails.
[§14 post-persistence research](specs/002-edit-open-gdscript/research.md#14-stock-post-persistence-saved-transition-research-2026-09-28)
instead establishes P1: exact CodeEdit native edit, explicit Script R sync and
retained-descriptor D persistence followed by fresh same-attempt receipt,
namespace, exact open-document/Script/CodeEdit/session/path/source and
post-complex-edit frozen **current** plus preparation-time **saved** version
guards. Preparation checks baseline current/saved (for example **6/6**);
pre-edit guards use current **6**, then the complex edit freezes current **8**
while saved remains **6**. Persistence and both pre-tag guards require **8/6**;
the native tag yields **8/8**. Human current **10** or same-text **12**
invalidates pre-tag authorization (without undoing known B/R/D effects).
Public `EditorInterface.set_object_edited` clears Resource edited state;
another guard precedes a direct call on the
actual existing native ScriptEditor saved-handler Callable discovered through
canonical ScriptEditor public incoming-signal connections. No signal emission,
private handler name lookup, CodeEdit-only tag, custom saver or engine A patch.
Native order tags CodeEdit saved version before document path mtime. Private
Resource mtime is not numerically read/set; the tested standalone-script
Save/reopen behavior across a natural timestamp-second change is bounded
evidence, not a universal metadata guarantee. The handler can execute native
validation even when automatic runtime reload is disabled, and schedules
deferred debugger reload only when enabled: independent stock
validation/effect confinement remains unresolved, with no running-game or A–E claim.

After the Callable, independent target/version/dirty/namespace rechecks precede
eventual separate stock validation and final independent D/R/B verification.
Known B/R/D changes with failed receipt or partial bookkeeping remain
applied-unverified; no newer human work is restored or tagged saved.

`/speckit.plan` completed the earlier design artifacts and planning check. The
[task list](specs/002-edit-open-gdscript/tasks.md) retains five PR-sized
increments; T001–T002 are complete, T003–T005 pending, with the same dependencies
and ownership of all 22 requirements, 26 scenarios and eight success criteria.
The earlier [granularity review](specs/002-edit-open-gdscript/tasks.md#granularity-review)
still applies to the unchanged decomposition. [Prior `/speckit.analyze` §13
result](specs/002-edit-open-gdscript/plan.md#stock-research-artifact-review)
identified an unresolved saved-state mechanism **then**, but is historical
after P1. The [post-persistence analysis](specs/002-edit-open-gdscript/plan.md#post-persistence-design-review)
retains 100% requirement/scenario ownership, with no critical constitutional
conflict and one acknowledged HIGH stock validation/effect-confinement
blocker. T003 is still held, not ready. US2.4 still requires T004's same-session
barrier/witness acceptance through the existing single-active collection/edit slot.

**T001 is complete:** the reusable Rust core checks clean revision eligibility,
fresh evidence, application certainty and independent verification, and emits
all five typed outcomes without acquiring or mutating editor state.

[T001 acceptance](specs/002-edit-open-gdscript/quickstart.md#9-t001-core-acceptance-2026-09-27):
48 edit-core regressions, 153 full-suite tests in the serial run, formatting,
Clippy, rustdoc and an external library consumer exercising all five outcomes.
The initial parallel run's existing socket-test `AddrInUse` failure is recorded;
no transport behavior was changed.
T001's numeric `SavedStateEvidence` Resource/document mtime fields and reducer's
disk-mtime equality/finalizer-step checks are its **historical implemented
interface**, not stock P1 evidence. T004 owns migration of affected typed core
evidence/reducer and caller/native/bridge contracts before integrating P1;
unavailable private Resource mtime must not be invented. T001 acceptance remains
valid against its then-approved contract; no replacement wire/API shape is
decided here and no sixth task is added.

**T002 is complete:** the matched C++17/public-ABI integration invokes Godot's real
parser/analyzer with confined, source-attributed dependency reads and effect
refusals. [T002 acceptance](specs/002-edit-open-gdscript/quickstart.md#10-t002-native-validation-acceptance-2026-09-28)
records 54 native cases, 194 stock-observation cases with the native artifact
installed, a missing-native D/R/B smoke, 155 Rust tests, three ABI regressions,
20 workflow regressions and the exact patched-editor/native build identities.
Prior native history and dirty D/R/B state remain unchanged; enabled, disabled
and hook-only exported games exclude tooling and execute without native dependencies.

**Feature 002 remains incomplete.** T003's native writes/stock saved transition
and edit guards, T004's caller/bridge cutover and T005's cumulative mutation
acceptance remain pending. The patched validation revision does not advertise
the editing API family. T003 has not started and must not implement historical
patched A; its independent stock native validation and edit-generated effect
confinement problem remains unresolved, including the saved handler's
validation and conditional deferred debugger reload. Review affected design
artifacts and actual analysis before implementation. The spec and acceptance
guarantees are unchanged. Standard-extension topology/version, receipt and
guard maintenance is proportionate against custom editor installation/trust,
per-version patched binary distribution and team replacement; correctness
cannot be traded for reduced distribution cost.

Delivery metadata: design artifacts merged in
[PR #24](https://github.com/Peter-Tam/godot-agent-kit/pull/24);
T001 delivered by [PR #25](https://github.com/Peter-Tam/godot-agent-kit/pull/25);
T002 delivered by merged [PR #29](https://github.com/Peter-Tam/godot-agent-kit/pull/29);
the preceding [PR #30](https://github.com/Peter-Tam/godot-agent-kit/pull/30)
is merged evidence. Task completion is independent of PR merge/review status.

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
