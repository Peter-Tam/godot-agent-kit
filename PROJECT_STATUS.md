# Project Status

## Current

- **Phase:** 1 — Live-editor script coherence
- **Phase state:** In progress
- **Feature:** [002 — Safely Edit Open GDScript](specs/002-edit-open-gdscript/spec.md)
- **Feature state:** [§17 M3/F-blocking](specs/002-edit-open-gdscript/research.md#17-minimal-behavioral-finalization-research-2026-09-28): tested minimal no-handler finalization fails later ordinary Save/history; held P1 saved-handler candidate remains unsafe under §16 H3. Core/patched validation complete; mutation pending.
- **Tasks:** 2 / 5 complete
- **Previous feature:** [001 — Observe Live GDScript Editor State Safely](specs/001-observe-gdscript-state/spec.md) — Complete; 8 / 8 tasks implemented and merged
- **Current task:** T003 unstarted/not implementation-ready; supported behavioral finalization and safe effect boundaries unresolved, with a separate explicit exact-source stock-validator obligation
- **Mutation A–E and edit durability:** Pending; no product mutation support or T003 readiness claim

## Roadmap status

| Phase | Status | Current evidence |
| --- | --- | --- |
| 0 — Governance and project foundation | Complete | Merged [constitution v1.1.0](.specify/memory/constitution.md) and [working agreement](AGENTS.md). |
| 1 — Live-editor script coherence | In progress | Feature 001 is complete; Feature 002's edit core and patched read-only native validation are complete. [§17 M3/F-blocking](specs/002-edit-open-gdscript/research.md#17-minimal-behavioral-finalization-research-2026-09-28) rejects the tested minimal handler-free finalization for false later Save/history behavior, not every handler-free route. [§14 P1](specs/002-edit-open-gdscript/research.md#14-stock-post-persistence-saved-transition-research-2026-09-28) remains a held saved-state-only handler candidate under [§16 H3](specs/002-edit-open-gdscript/research.md#16-stock-saved-handler-admission-research-2026-09-28), without a useful safe admission profile; the explicit exact-source stock validator is separately unresolved. T003 unstarted, T004/T005, A–E and durability pending. |
| 2–13 — Later roadmap phases | Pending | Not started; direction and exit criteria remain in [ROADMAP.md](ROADMAP.md). |

## Current §17 disposition

[Minimal behavioral finalization research §17](specs/002-edit-open-gdscript/research.md#17-minimal-behavioral-finalization-research-2026-09-28)
tests guarded native CodeEdit B edit, explicit Script R sync, retained-descriptor
D persistence, public Resource edited=false and direct CodeEdit saved-version tag,
**without a saved handler**. Independent getters initially found intended
D=R=B, clean current/saved version and stable target/unrelated state. After
newer human typing, a real stock ordinary Save instead displayed **“Files have
been modified outside Godot”** and did not persist the newer text; a separate
Undo → ordinary Save → Redo → Save sequence encountered the dialog and an
extra history step. The valid unrelated dirty no-final-newline fixture was
unchanged during the candidate interval; clean reopen and fresh-launch
durability observations do not cancel the Save/history failure. The source
audit found no bounded supported narrow behavioral reconciliation respecting
live identity, history and effect constraints. The **M3/F-blocking** result
is specific to this tested route, **not** proof all handler-free routes fail,
the full handler is required, or private numeric timestamp/cache parity is required.

Behavioral acceptance requires independently clean intended D/R/B; guards
against stale or newer human edits, redirected persistence and unrelated or
forbidden effects; truthful partial/unknown outcomes; usable native
Undo → ordinary Save → Redo → Save and earlier history; later human ordinary
Save without false external-change reconciliation; reopen and applicable
persistence/durability. Intermediate D/R/B/internal divergence is permitted
within the guarded attempt; there is no externally atomic every-step
requirement. Private writable timestamp equality, native cache generation
parity and replay of the whole Save/handler callback history are **not**
acceptance substitutes. Runtime hot reload was not observed in the tested
active game and is not required by the specification; runtime-effect safety
and normal fresh-launch durability still matter. The [§14 P1 route](specs/002-edit-open-gdscript/research.md#14-stock-post-persistence-saved-transition-research-2026-09-28)
remains held for saved state only: [§16 H3](specs/002-edit-open-gdscript/research.md#16-stock-saved-handler-admission-research-2026-09-28)
still blocks callback admission, and explicit exact-source valid/invalid/
unavailable stock validation is independently unresolved. Neither candidate
is a production route; T003 has not started and no A–E gate is passed. The
local retained §17 experiment artifacts are not a public durable reproduction
or Feature acceptance run.

## Active feature

Feature 002's [specification](specs/002-edit-open-gdscript/spec.md) and
[requirements-quality checklist](specs/002-edit-open-gdscript/checklists/requirements.md)
define a one-script, already-open editing capability with stale-write protection,
independently verified D/R/B convergence, native Undo/Redo, and durability.
The [implementation plan](specs/002-edit-open-gdscript/plan.md),
[native contract](specs/002-edit-open-gdscript/contracts/native-integration.md)
and [§14 saved-state research](specs/002-edit-open-gdscript/research.md#14-stock-post-persistence-saved-transition-research-2026-09-28)
retain P1 for the **post-persistence saved-state question only**, on hold
under §16 H3 rather than approved as the product finalizer. The prior
patched-engine finalizer is historical, not an approved product route.
[§15 V2](specs/002-edit-open-gdscript/research.md#15-stock-validation-and-effect-confinement-research-2026-09-28)
posed a focused callback-admission question. [§16 stock saved-handler admission
research](specs/002-edit-open-gdscript/research.md#16-stock-saved-handler-admission-research-2026-09-28)
concludes **H3: no useful safe profile established for the demonstrated
official-stock saved-handler/public-observer composition**. The minimum
preload and literal-extends source/dependency cases proved meaningful P1
saved-state mechanics but not a qualifying safety rule. Independent
source/dependency/parser and shallow/export generation (including reverse
consumers), current-editor/function-discovery and ordinary callback closure,
and shared deferred debugger routing/completion remain unresolved before
dispatch. An explicit exact-source stock validator remains independently
unselected/unproved. Neither a successful saved tag nor post-effect observation
authorizes T003; no accepting predicate or alternative unsafe-tail replacement
is selected.
The [data model](specs/002-edit-open-gdscript/data-model.md),
[caller contract](specs/002-edit-open-gdscript/contracts/edit-api.md),
[private bridge contract](specs/002-edit-open-gdscript/contracts/bridge-protocol.md)
and [quickstart recipes](specs/002-edit-open-gdscript/quickstart.md) are
intentionally unchanged: their patched-family-1/finalizer-specific API,
bridge and build instructions remain **historical**, not the current T003
stock recipe. The [native §3 contract](specs/002-edit-open-gdscript/contracts/native-integration.md#3-primitive-a-guarded-target-document-saved-transition)
keeps the held P1 saved sequence for its saved-state-only question. No final production native/bridge
wire schema or editing API has been implemented by this research. The product
target remains official stock Godot, addon and bundled standard GDExtension;
Rust retains core policy and independent verification. T002's patched native
validator remains completed semantic oracle/reference and hazard inventory,
not a product patched dependency or proof of stock validation/effect safety.

The [F3 stock-save research](specs/002-edit-open-gdscript/research.md#13-stock-target-save-finalization-research-2026-09-28)
demonstrates successful ordinary stock target Save performs real native saved-state
bookkeeping/history, including a descriptor-backed saver. It also demonstrates
that non-OK custom-saver failures fall through to builtin path writes (observed
outside-project redirection), and target Save applies every open script, changing
an unrelated dirty buffer/history. Direct `ResourceSaver.save` does not complete
the ScriptEditor saved transition. The audited routes therefore do not qualify;
this is **historical F3 evidence**, not a conclusion that every stock route fails.
[§14 post-persistence research](specs/002-edit-open-gdscript/research.md#14-stock-post-persistence-saved-transition-research-2026-09-28)
established the held P1 candidate's saved-state mechanics: exact CodeEdit native edit, explicit Script R sync and
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
validation even when automatic debugger reload is disabled, and may schedule
deferred debugger reload conditionally. Its names-update tail transitively
calls `get_functions()` on the **current** editor, which can validate current B,
not necessarily the selected target. Separate ordinary editor validation
and export handlers can mutate R/exports or apply pending dragged properties;
direct saved handling does **not** unconditionally refresh target exports.
Neither a handler return, next idle tick nor observable public debugger
session proves that agent-originated work for a specific source version
drained. A persisted metadata=false/unchecked menu state is not a supported
effective scheduling fence: §16 observed active runtime behavior change
from 17 to 31 despite that displayed state. No running-game safety,
A–E or callback-confinement claim follows P1.

For held P1, post-Callable target/version/dirty/namespace rechecks precede
eventual separate stock validation and final independent D/R/B verification.
Known B/R/D changes with failed receipt or partial bookkeeping remain
applied-unverified; no newer human work is restored or tagged saved. These
mechanics do not discharge §17's later ordinary Save/history behavioral gate.

`/speckit.plan` completed the earlier design artifacts and planning check. The
[task list](specs/002-edit-open-gdscript/tasks.md) retains five PR-sized
increments; T001–T002 are complete, T003–T005 pending, with the same dependencies
and ownership of all 22 requirements, 26 scenarios and eight success criteria.
The earlier [granularity review](specs/002-edit-open-gdscript/tasks.md#granularity-review)
still applies to the unchanged decomposition. The [prior `/speckit.analyze`
§13 result](specs/002-edit-open-gdscript/plan.md#stock-research-artifact-review)
identified an unresolved saved-state mechanism **then** and is historical
after P1. The [post-persistence analysis](specs/002-edit-open-gdscript/plan.md#post-persistence-design-review)
also predates §15: it retained 100% requirement/scenario ownership and one
acknowledged HIGH stock validation/effect blocker but did **not** assess the
later V2 or H3 decisions. The [stock validation and effect design review](specs/002-edit-open-gdscript/plan.md#stock-validation-and-effect-design-review)
records the installed `/speckit.analyze` workflow against **§15 V2** and
remains historical. The [saved-handler admission design
review](specs/002-edit-open-gdscript/plan.md#saved-handler-admission-design-review)
records the actual installed read-only workflow for **§16 H3**: verified
Feature 002 override, passed prerequisite, no registered hooks, 30/30
buildable requirements, 26/26 numbered scenarios and 8/8 edge cases with
task ownership, five tasks with two complete, no unmapped task, new
requirement ambiguity, harmful duplication or CRITICAL constitutional conflict.
U1 remains an intentional HIGH implementation-readiness hold. Evidence/source
review identified material research corrections, and semantic review identified
a current research-introduction inconsistency. Those six documentation
corrections were verified against pinned source and retained evidence;
Markdown, local links and source anchors passed for that §16 review. This
historical research/design assessment was complete, **not** proof of a safe
stock admission architecture, explicit validator, mutation acceptance,
§17 behavioral finalization or T003 readiness. T003 remains unstarted.

The [current §17 minimal behavioral finalization design
review](specs/002-edit-open-gdscript/plan.md#minimal-behavioral-finalization-design-review)
completed the installed `/speckit.analyze` research assessment of M3/F-blocking
with unchanged task ownership and an intentional HIGH U1 hold. It is not a
T003 implementation-readiness pass or mutation acceptance.

US2.4 still requires T004's same-session barrier/witness acceptance through
the existing single-active collection/edit slot.

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

**Feature 002 remains incomplete.** T003's native writes/behavioral saved-state
finalization and edit guards, T004's caller/bridge cutover and T005's cumulative
mutation acceptance remain pending. The patched validation revision does not
advertise the editing API family. T003 has not started and must not implement
historical patched A or the measured failing §17 minimal route.
[§15](specs/002-edit-open-gdscript/research.md#15-stock-validation-and-effect-confinement-research-2026-09-28)
records measured stock limits: `reload(false)` skips export refresh but still
parses, analyzes, compiles and can initialize; the broad editor-mode
`--check-only --script` helper fully loaded before EditorNode disabled
scripting, executed tool and non-tool static initializers, and exited 0 for
observed invalid inputs. Public ClassDB/global-class/autoload metadata is
available but not a full semantic or callback proof. Restricted helper and
public-ABI adaptation remain unselected, unexhausted options; conservative
source-scanner/refusal proposals are not a new `@tool`-only policy or validator.
[§16 H3](specs/002-edit-open-gdscript/research.md#16-stock-saved-handler-admission-research-2026-09-28)
found no useful safe profile for the tested stock/public-observer composition:
same-path diagnostics followed **current disk type semantics** despite old
compiled method metadata, so stale parser consumption was not demonstrated;
no public consumed parser/dependency generation witness or pre-dispatch
source/export/current-editor/deferred-work effect closure was established.
The active game changed from 17 to 31 under metadata=false/menu-unchecked
startup; only disposable-project effective menu setter experiments bounded
runtime off at 17 and on at 31, not a product scheduling fence. Separately,
an explicit exact-source stock validator must be selected and proved.
Neither §15 nor §16 made T003 ready. The [actual §16 H3 design
review](specs/002-edit-open-gdscript/plan.md#saved-handler-admission-design-review)
retained U1 HIGH without weakening the spec or acceptance guarantees; the
earlier [§15 V2 review](specs/002-edit-open-gdscript/plan.md#stock-validation-and-effect-design-review)
is historical. [§17 M3](specs/002-edit-open-gdscript/research.md#17-minimal-behavioral-finalization-research-2026-09-28)
adds a distinct blocked behavioral finalization route; the [current §17
analysis](specs/002-edit-open-gdscript/plan.md#minimal-behavioral-finalization-design-review)
assesses that disposition but establishes no safe alternative. Supported normal
Save/history behavior, safe confinement of any unavoidable callbacks and
independently proved exact-source validation remain prerequisites to T003.
The smaller no-handler candidate avoided callback topology but failed the
actual Save/history invariants; held P1 carries Callable topology/version
and callback/deferred-effect costs under H3. A custom editor would add trusted
installation, patched binary distribution and team replacement, with no
justified waiver of human-work, history or effect safety. No private numeric
timestamp/cache parity or full handler replay is a requirement by itself.

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
