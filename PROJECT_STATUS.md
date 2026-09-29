# Project Status

## Current

- **Phase:** 1 — Live-editor script coherence
- **Phase state:** In progress
- **Feature:** [002 — Safely Edit Open GDScript](specs/002-edit-open-gdscript/spec.md)
- **Feature state:** [§18 T1](specs/002-edit-open-gdscript/research.md#18-preserved-mtime-behavioral-finalization-research-2026-09-28) still selects same-retained-fd preserved-mtime, handler-free finalization. [§19 final L2](specs/002-edit-open-gdscript/research.md#19-one-shot-stock-lsp-validation-research-2026-09-28) withdraws the intermediate blanket additional-client/all-input/effect-exclusion hold: source-attributed owner parser results, diagnostics, symbols and disk-based dependencies held in the examined profile. Its **one remaining hold** is the existing FR-020 normal-local-user privacy boundary for the *new* helper-authority source-read endpoint, not owner correctness or a second connection itself. The earlier installed L1 analysis and blanket L2 assessment are intermediate; final installed analysis of narrowed L2 is complete with one HIGH FR-020 endpoint-privacy hold. This is completed technical research, not completed Feature 002, product mutation/validator or A–E acceptance.
- **Tasks:** 2 / 5 complete
- **Previous feature:** [001 — Observe Live GDScript Editor State Safely](specs/001-observe-gdscript-state/spec.md) — Complete; 8 / 8 tasks implemented and merged
- **Current task:** T003 is **unstarted and not implementation-ready** as a whole while the one FR-020 helper-endpoint privacy gap remains unresolved. Once a supported normal-local-user access boundary qualifies, T003 still owns a bounded exact-source stock helper alongside the unchanged §18 T1 native finalizer; no helper implementation, module placement or PR is selected now. T004 owns typed caller/bridge/core evidence migration and integration; T005 cumulative acceptance. Five tasks, the dependency graph, normal one-task/PR sequencing and generic forbidden-effect/human-work invariants remain unchanged.
- **Mutation A–E and edit durability:** Pending; no product mutation support or wider-platform claim follows the completed research.

## Roadmap status

| Phase | Status | Current evidence |
| --- | --- | --- |
| 0 — Governance and project foundation | Complete | Merged [constitution v1.1.0](.specify/memory/constitution.md) and [working agreement](AGENTS.md). |
| 1 — Live-editor script coherence | In progress | Feature 001 complete; Feature 002 T001 core and T002 patched read-only validation complete (T002 is an oracle/reference, not product stock validation). [§18 T1](specs/002-edit-open-gdscript/research.md#18-preserved-mtime-behavioral-finalization-research-2026-09-28) remains selected. [§19 final L2](specs/002-edit-open-gdscript/research.md#19-one-shot-stock-lsp-validation-research-2026-09-28) establishes scoped owner attribution but leaves one FR-020 normal-local-user helper source-read endpoint privacy boundary unqualified, so T003 is unstarted/not implementation-ready. Prior L1 and blanket L2 assessments are intermediate; final installed narrowed-L2 analysis is complete with one HIGH FR-020 endpoint-privacy hold; T004/T005, A–E and durability remain pending. §14–§17 alternative routes remain historical; other platforms unresolved. |
| 2–13 — Later roadmap phases | Pending | Not started; direction and exit criteria remain in [ROADMAP.md](ROADMAP.md). |

## Selected §18 finalization — research/design only

[§18 preserved-mtime behavioral finalization research](specs/002-edit-open-gdscript/research.md#18-preserved-mtime-behavioral-finalization-research-2026-09-28)
selects **T1**: prepare an exact clean target/Save profile and retained project
and exact-file descriptors; record original target fd mtime T0 and exact
Script/editor/CodeEdit/source/current/saved identities; one native CodeEdit
complex B edit → guarded explicit Script R setter/readback → guarded same-fd
D write/truncate/fsync/pread → fresh receipt/source/version/namespace guards
→ `futimens` on the **same retained fd** with atime `UTIME_OMIT`, mtime T0
→ fstat/readback/identity/namespace checks → fresh guards → public Resource
edited=false → fresh guards → CodeEdit saved-version tag → **STOP**. It uses
no saved-handler discovery/call, `ResourceSaver`, `resource_saved` broadcast,
broad Save, private Godot mtime setter, forced reload or unrelated edits.

The accepted stock GUI evidence is **16 distinct case observations**:
15 in the final campaign plus **one separate corrected current-build settled
history case**, not a passing 16/16 campaign or product A–E acceptance.
The candidate gave independently clean intended D/R/B23 and native history,
later human B29 ordinary Save without the false external-change dialog,
Undo18 → Save18 → Redo23 → Save23 → Undo18 → Undo17 → Redo18 → Redo23,
reopen and fresh-launch durability, ordinary scan/reparse and fresh consumer
behavior on the tested fixture. The stock control passed; [§17's no-restore
negative](specs/002-edit-open-gdscript/research.md#17-minimal-behavioral-finalization-research-2026-09-28)
still records the false later Save dialog and an additional history step.
The historical extra step's causal reconciliation path was **not** proved.
Failed same-fd restoration after successful content write or failed T0
readback is **known partial applied-unverified**, never false not-applied,
clean success or implicit rollback; a retained-fd `EBADF` case left D/R/B23
dirty with no saved tag. Leaf/parent namespace replacements did not redirect
writes or retimestamp replacement files. Generic forbidden-effect,
newer-human-work, non-selected/unrelated-document and identity guards remain
mandatory, not a license for broad callback effects.

This is **research/design only** on official Godot 4.7.2
`ed1daf0bf` (binary SHA-256
`c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf`)
and macOS 26.6.2 arm64; other platforms are unresolved. No production T003
native attempt/API, callback, wire schema or Feature mutation acceptance
was implemented or exercised.
[§19 final L2](specs/002-edit-open-gdscript/research.md#19-one-shot-stock-lsp-validation-research-2026-09-28)
does not select the bounded exact-source stock validator for implementation:
the new helper source-read endpoint lacks a qualified normal-local-user FR-020
privacy boundary, although examined owner parser results remain attributable.
If resolved, production capture/admission and helper implementation remain T003 work. The T002
patched validator is reference/hazard inventory, not this stock solution.
Existing outcomes, guards, normal Save/history/reparse/rescan/fresh-launch
and privacy/export gates are unchanged. Private Resource/cache timestamp
parity, full handler replay and active-game hot reload are not implied.
The [§14 P1 handler mechanics](specs/002-edit-open-gdscript/research.md#14-stock-post-persistence-saved-transition-research-2026-09-28),
[§16 H3 handler-admission limits](specs/002-edit-open-gdscript/research.md#16-stock-saved-handler-admission-research-2026-09-28)
and [§17 M3 failed no-restore route](specs/002-edit-open-gdscript/research.md#17-minimal-behavioral-finalization-research-2026-09-28)
are retained as historical research, **not current production prerequisites**.
Raw local §18 artifacts are evidence provenance, not a public durable
reproduction or T003 acceptance run.

## §19 stock LSP validation — narrowed L2 privacy hold, not product selection

[§19 final L2](specs/002-edit-open-gdscript/research.md#19-one-shot-stock-lsp-validation-research-2026-09-28)
supersedes the unpublished tentative L1 design selection **and withdraws**
the intermediate blanket additional-client/all-input/effect-exclusion hold.
One fresh matching official Godot `--headless --editor` LSP child per immutable
captured source closure and validation pass remains a researched candidate,
not an approved T003 implementation route until its new helper-authority
source-read endpoint is kept within FR-020's normal-local-user privacy boundary.
Another connection alone does not invalidate an owner's result.

The research exercised **69 distinct helper invocations**: **65** official-stock
source/closure study invocations (44 + 10 + 11), **one** separate discard-I/O
privacy control and **three** additional-client probes (67–69). This is not
69 product passes or a new GUI acceptance gate. The final 10
closure cases supplied **21 independently checked exact-source per-URI
fences** (six valid, four invalid), plus one root-only negative control.
The privacy control added two separate source fences and is retained in
[its verified proof](file:///var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-one-shot-lsp-39y7q6v3/evidence/privacy-verified.json).

That separately verified root-valid/helper-invalid control launched stock
Godot without `--log-file` and discarded child stdout/stderr; inspection found
no helper `.log` or captured output files in the owned project/HOME/XDG and
the private diagnostic sentinel only in the intended helper source. Unlike
the 65 study launches, this control did not claim a new live GUI target witness.
The earlier logged synthetic research launches remain evidence, not the
production launch recipe.

An unused preloaded dependency can be root-valid while its own URI has an
error: every root/dependency URI requires both its publishDiagnostics and
parser-symbol response before valid/invalid is reported. Symbols or empty
diagnostics alone are insufficient. These are research observations, not a
production scanner, implemented T003 boundary or A–E acceptance.

Invocation 67 observed two same-UID synthetic clients retain **distinct**
same-URI text/version, diagnostic and symbol results: owner valid from captured
root v1, second invalid from its different root v37. Before supplying source,
the second client requested symbols for the staged root and a separate
harness-owned `.gd` **outside the helper project**; stock Godot returned
source-derived constants for both. Inverse probes 68/69 then opened
opposite-text versions of the *same helper dependency URI*: disk/helper `int`
with second-client `String` yielded owner valid, while disk/helper `String`
with second-client `int` yielded owner invalid with its expected type error.
Each examined owner root and required source retained URI-specific
publishDiagnostics **and** shaped documentSymbol fences. Pinned stock source
routes notifications through per-peer managed files and parse results and
obtains dependencies from staged disk, not another peer's overlay. No false
valid, false invalid or false completed owner result was observed. This
establishes **scoped owner semantic attribution**, not global exclusion of
independent clients, arbitrary inputs or every Godot method.

The separate gap is FR-020 privacy for the **new helper-authority source-read
endpoint**: reviewed stock LSP paths admit another connection and can open an
unmanaged absolute `.gd` under helper authority and return source-derived
symbols without a caller identity check. Invocation 67 directly exercised
that capability only with the same synthetic harness UID and owned fixtures;
no different-local-user reachability/disclosure, cross-UID exploitation,
selected-editor D/R/B mutation or new GUI witness was measured. The earlier
65 study cases retain their separately scoped selected-target nonmutation
evidence. Listener-PID verification/private staging alone do not qualify a
normal-local-user access boundary for the new endpoint. Hostile same-UID
software is outside the inherited threat model, and independent trusted
clients are not disqualifying. No public stock boundary sufficient for this
specific source-read path was selected/proved; this does **not** establish
that every public composition is impossible or that owner results are
unavailable simply because a second connection exists. A further proposed
experiment was declined; no retry/reformulation or Godot failure is inferred.
Further cross-UID runtime evidence is unavailable in this pass as a tooling/
environment limit, not a demonstrated negative result.

Once a normal-local-user boundary for that endpoint qualifies, T003's
existing supervised Rust worker can own bounded no-follow exact capture/hash
checks, conservative prelaunch admission, owned source-only clone/context
staging, matching child/endpoint checks, per-URI diagnostics/parser-symbol
fences, bounded diagnostics, deadline and owned-child terminate/kill/reap.
Literal confined standalone `.gd` closures, effective warnings/directory
rules and uncertain-context refusal remain candidate scope, not an admitted
production scanner. Startup scanning before didOpen still requires prelaunch
source/effect admission for the studied source-only profile. No broad project
copy, patched engine, generic LSP layer, new crate, persistent process or
new security mechanism is selected. The unresolved obligation is qualifying
normal-local-user confinement for **this new helper endpoint**, not imposing
all-input ownership or specifying another experiment.

T003 remains one helper-plus-unchanged-§18-native task, **unstarted and not
implementation-ready**; helper module placement is conditional, not a
`mcp-server/src/gdscript_validation.rs` requirement. T004 retains migration
of historical T001 numeric-mtime and T002 patched-validator evidence/contracts
and typed caller/bridge integration; T005 retains cumulative gates. No final
production wire schema/API is set by this research. The prior [installed §19
analysis review](specs/002-edit-open-gdscript/plan.md#one-shot-stock-lsp-validation-design-review)
covered 30/30 buildable requirements (22 FRs, eight SCs), 26/26 scenarios
and 8/8 edge cases with task ownership and corrected I1 logging. That was an
**intermediate L1 assessment**, not a current narrowed-L2 implementation-readiness
pass. Final installed analysis of the narrowed L2 artifacts is complete with one HIGH FR-020 endpoint-privacy hold.
Five tasks with only T001/T002 checked, the unchanged dependency graph and
normal one-task/PR sequence remain in force; there is no product edit,
Feature 002 completion, Phase 1 completion or wider-platform support.

This research/design update changes only `research.md`, `plan.md`,
`contracts/native-integration.md`, `tasks.md` and `PROJECT_STATUS.md`;
`data-model.md`, `contracts/edit-api.md`, `contracts/bridge-protocol.md` and
`quickstart.md` remain the unchanged downstream baseline. There is no
specification, production code, API, schema or module implementation change.

## Active feature

Feature 002's [specification](specs/002-edit-open-gdscript/spec.md) and
[requirements-quality checklist](specs/002-edit-open-gdscript/checklists/requirements.md)
define a one-script, already-open editing capability with stale-write protection,
independently verified D/R/B convergence, native Undo/Redo, and durability.
The [implementation plan](specs/002-edit-open-gdscript/plan.md) and
[native contract](specs/002-edit-open-gdscript/contracts/native-integration.md)
retain [§18 T1's preserved-mtime handler-free design](specs/002-edit-open-gdscript/research.md#18-preserved-mtime-behavioral-finalization-research-2026-09-28).
[§19 final L2](specs/002-edit-open-gdscript/research.md#19-one-shot-stock-lsp-validation-research-2026-09-28)
does not select a stock-LSP helper for T003 implementation until the new
source-read endpoint's FR-020 normal-local-user privacy boundary qualifies. The native sequence
retains exact target and source/version guards, same-fd T0 restoration/readback before
edited=false/CodeEdit tag, and no saved-handler call or broad Save.
[§14 P1](specs/002-edit-open-gdscript/research.md#14-stock-post-persistence-saved-transition-research-2026-09-28),
[§16 H3](specs/002-edit-open-gdscript/research.md#16-stock-saved-handler-admission-research-2026-09-28)
and the former patched-engine finalizer remain historical route evidence,
**not current handler admission, topology, callback-generation or
deferred-debugger prerequisites**. [§17 M3](specs/002-edit-open-gdscript/research.md#17-minimal-behavioral-finalization-research-2026-09-28)
records the earlier measured failure **without** T0 restoration. Generic
forbidden-effect safety, identity/session/version and namespace guards,
human/unrelated-work preservation and independent verification still apply.
No stock exact-source valid/invalid/unavailable helper is selected or
implemented; neither a clean saved tag nor research alone authorizes a
product edit. Conditional T003 stock-helper obligations include prelaunch
capture/admission, qualifying the new helper source-read endpoint's normal-local-user
boundary and independently fenced validation snapshots; a second client alone
does not turn an attributable owner result into unavailable.
The [data model](specs/002-edit-open-gdscript/data-model.md),
[caller contract](specs/002-edit-open-gdscript/contracts/edit-api.md),
[private bridge contract](specs/002-edit-open-gdscript/contracts/bridge-protocol.md)
and [quickstart recipes](specs/002-edit-open-gdscript/quickstart.md) describe
existing models and historical patched-family-1/API/bridge/build mechanics
where identified; **no final production native/bridge wire schema or editing
API has been implemented** by this research. The product target remains
official stock Godot, addon and bundled standard GDExtension; Rust retains
core policy and independent verification. T002's patched native validator
remains completed semantic oracle/reference and hazard inventory, not a
product patched dependency or proof of implemented stock helper safety.

The [F3 stock-save research](specs/002-edit-open-gdscript/research.md#13-stock-target-save-finalization-research-2026-09-28)
demonstrates successful ordinary stock target Save performs real native saved-state
bookkeeping/history, including a descriptor-backed saver. It also demonstrates
that non-OK custom-saver failures fall through to builtin path writes (observed
outside-project redirection), and target Save applies every open script, changing
an unrelated dirty buffer/history. Direct `ResourceSaver.save` does not complete
the ScriptEditor saved transition. The audited routes therefore do not qualify;
this is **historical F3 evidence**, not a conclusion that every stock route fails.
[§14 post-persistence research](specs/002-edit-open-gdscript/research.md#14-stock-post-persistence-saved-transition-research-2026-09-28)
historically established P1's **now-superseded production candidate** saved-state mechanics: exact CodeEdit native edit, explicit Script R sync and
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

For historical P1, post-Callable target/version/dirty/namespace rechecks
preceded eventual separate stock validation and final independent D/R/B
verification. Known B/R/D changes with failed receipt or partial bookkeeping
remained applied-unverified; no newer human work was restored or tagged saved.
These facts remain research evidence, not a selected saved-handler product
route or a new prerequisite. §17's later ordinary Save/history failure was
specific to the un-restored no-handler route and is addressed by §18 T1.

`/speckit.plan` completed the earlier design artifacts and planning check. The
[task list](specs/002-edit-open-gdscript/tasks.md) retains five PR-sized
increments; T001–T002 are complete, T003–T005 pending, with the same dependencies
and ownership of all 22 requirements, 26 scenarios and eight success criteria.
The earlier [granularity review](specs/002-edit-open-gdscript/tasks.md#granularity-review)
is historical; the updated T003 stock-helper-plus-native boundary preserves
its five-task decomposition. The earlier [installed §19 analysis
review](specs/002-edit-open-gdscript/plan.md#one-shot-stock-lsp-validation-design-review)
assessed the intermediate I1-corrected L1 design as implementation-ready;
the subsequent blanket L2 additional-client hold was also intermediate and
withdrawn after invocations 67–69 established scoped owner attribution.
Final installed analysis of the narrowed FR-020 endpoint-privacy L2 is complete with one HIGH FR-020 endpoint-privacy hold,
and T003 is unstarted/not implementation-ready.
The [prior `/speckit.analyze`
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
At that §16 review U1 was an intentional HIGH implementation-readiness hold.
Evidence/source review identified material research corrections, and semantic review identified
a current research-introduction inconsistency. Those six documentation
corrections were verified against pinned source and retained evidence;
Markdown, local links and source anchors passed for that §16 review. This
historical research/design assessment was complete, **not** proof of a safe
stock admission architecture, explicit validator, mutation acceptance,
§17 behavioral finalization or T003 readiness. T003 remains unstarted.

The [§17 minimal behavioral finalization design
review](specs/002-edit-open-gdscript/plan.md#minimal-behavioral-finalization-design-review)
completed the installed `/speckit.analyze` research assessment of the
then-current M3/F-blocking disposition with unchanged task ownership and an
intentional HIGH U1 hold. It is **historical after §18 T1** and is not a
T003 implementation-readiness pass, new T1 analysis result or mutation
acceptance.

The [§18 preserved-mtime design review](specs/002-edit-open-gdscript/plan.md#preserved-mtime-behavioral-finalization-design-review)
reran the installed `/speckit.analyze` workflow with the verified feature
override and no registered hooks. After correcting three publication
inconsistencies, 30/30 buildable requirements, 26/26 scenarios and 8/8 edge
cases retain ownership, with no remaining CRITICAL conflict or unmapped task.
U1 was then HIGH solely for exact-source valid/invalid/unavailable stock
validation. The tentative L1 selection and its analysis, then the blanket
L2 additional-client assessment, were superseded by §19's narrower FR-020
helper-endpoint privacy hold. This §18 result is historical, not product
implementation or a current implementation-readiness pass.

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
interface**, not the selected stock §18 finalization evidence. T004 owns
migration of affected typed core evidence/reducer, historical T002 patched
validator fields and caller/native/bridge contracts before integrating a
qualified T003 helper and finalizer; unavailable private Resource mtime,
editor-clock parity and engine-computed source hashes must not be invented.
T001 acceptance remains valid against its then-approved contract; no
replacement wire/API shape is decided here and no sixth task is added.

**T002 is complete:** the matched C++17/public-ABI integration invokes Godot's real
parser/analyzer with confined, source-attributed dependency reads and effect
refusals. [T002 acceptance](specs/002-edit-open-gdscript/quickstart.md#10-t002-native-validation-acceptance-2026-09-28)
records 54 native cases, 194 stock-observation cases with the native artifact
installed, a missing-native D/R/B smoke, 155 Rust tests, three ABI regressions,
20 workflow regressions and the exact patched-editor/native build identities.
Prior native history and dirty D/R/B state remain unchanged; enabled, disabled
and hook-only exported games exclude tooling and execute without native dependencies.

**Feature 002 remains incomplete.** T003 remains one conditional stock
helper-plus-unchanged-§18-native task; no stock helper is selected for
implementation until the single FR-020 normal-local-user source-read endpoint
privacy gap is resolved. T004 owns caller/bridge/core evidence migration and
integration; T005 owns cumulative mutation acceptance. The patched T002
revision does not advertise editing. T003 is **unstarted and not
implementation-ready** as a whole. The earlier [installed §19 design
review](specs/002-edit-open-gdscript/plan.md#one-shot-stock-lsp-validation-design-review)
is an intermediate I1-corrected L1 assessment, and the subsequent blanket L2
assessment is withdrawn; final installed narrowed-L2 analysis is complete with one HIGH FR-020 endpoint-privacy hold.
Normal task selection, review, merge and one-task/PR sequencing remain
required. This one HIGH FR-020 privacy hold does not weaken generic
forbidden-effect and human-work safety or select patched A, held P1 handler
or the failed §17 route.
[§15](specs/002-edit-open-gdscript/research.md#15-stock-validation-and-effect-confinement-research-2026-09-28)
records measured stock limits: `reload(false)` skips export refresh but still
parses, analyzes, compiles and can initialize; the broad editor-mode
`--check-only --script` helper fully loaded before EditorNode disabled
scripting, executed tool and non-tool static initializers, and exited 0 for
observed invalid inputs. Public ClassDB/global-class/autoload metadata is
available but not a full semantic or callback proof. At the time of §15,
restricted helper and public-ABI adaptation were unselected, unexhausted
options; §19 final L2 leaves bounded stock LSP validation **unselected**
pending the new helper source-read endpoint's normal-local-user privacy
boundary, not §15's broad check-only helper or a generic `@tool`-only refusal policy.
[§16 H3](specs/002-edit-open-gdscript/research.md#16-stock-saved-handler-admission-research-2026-09-28)
historically found no useful safe profile for the tested stock saved-handler/
public-observer composition: same-path diagnostics followed **current disk
type semantics** despite old compiled method metadata, so stale parser
consumption was not demonstrated; no public consumed parser/dependency
generation witness or pre-dispatch source/export/current-editor/deferred-work
effect closure was established. The active game changed from 17 to 31 under
metadata=false/menu-unchecked startup; only disposable-project effective
menu setter experiments bounded runtime off at 17 and on at 31, not a product
scheduling fence. These are **historical route-specific limits**, not
handler-generation gates for §18's selected sequence.
The [§16 H3 design review](specs/002-edit-open-gdscript/plan.md#saved-handler-admission-design-review)
retained a HIGH U1 hold for its then-current route without weakening the spec;
the [§15 V2 review](specs/002-edit-open-gdscript/plan.md#stock-validation-and-effect-design-review)
is likewise historical. [§17 M3](specs/002-edit-open-gdscript/research.md#17-minimal-behavioral-finalization-research-2026-09-28)
found the no-restore handler-free candidate initially clean but blocked later
human ordinary Save and introduced an additional history step in its corrected
history case. Its [installed §17 assessment](specs/002-edit-open-gdscript/plan.md#minimal-behavioral-finalization-design-review)
preceded T1; neither historical route describes the current disposition.
[§18 T1](specs/002-edit-open-gdscript/research.md#18-preserved-mtime-behavioral-finalization-research-2026-09-28)
selects same-retained-fd T0 restoration with readback **before** guarded
public edited=false and CodeEdit tag, without handler topology/callback
machinery. [§19 final L2](specs/002-edit-open-gdscript/research.md#19-one-shot-stock-lsp-validation-research-2026-09-28)
establishes owner diagnostic/symbol and disk-dependency attribution in the
examined source-only profile with per-URI exact-source fences; it does not
select a bounded helper until the new source-read endpoint's normal-local-user
FR-020 boundary qualifies. T003 is unstarted/not implementation-ready; the
previous [installed L1 analysis review](specs/002-edit-open-gdscript/plan.md#one-shot-stock-lsp-validation-design-review)
and blanket L2 assessment are intermediate, and final installed narrowed-L2
analysis is complete with one HIGH FR-020 endpoint-privacy hold. General
forbidden-effect safety and all behavior, history, human-work and durability
gates remain mandatory.
Neither research completion nor patched T002 validation makes a stock helper
product or confers mutation support.
No private numeric timestamp/cache parity or full handler replay is required
by itself.

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
