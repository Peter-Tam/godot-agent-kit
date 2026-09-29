# Quickstart: Verify Guarded Open-GDScript Editing

**Status:** T001–T003 are complete. Historical core/oracle acceptance remains in [§9](#9-t001-core-acceptance-2026-09-27) and [§10](#10-t002-native-validation-acceptance-2026-09-28); [§11](#11-t003-native-boundary-acceptance-2026-09-29) records the stock helper/native boundary's real-editor acceptance. T004/T005, public caller/bridge/core migration, full-feature A–E and cumulative acceptance remain pending. Feature 002 is still in progress.

**Current reproduction:** Use the official stock executable, the standard
public-ABI extension and the Rust one-shot validator as in the
[stock native guide](../../godot-addon/native/README.md#official-stock-boundary-and-verification).
Sections 1–8 retain feature-wide T004/T005 acceptance requirements; historical
patched-family/mtime/caller sketches within them are not instructions for
current T003 reproduction. §10 is an archive summary, not an oracle recipe.

Use the [spec](spec.md), [plan](plan.md), [data model](data-model.md) and [caller](contracts/edit-api.md), [bridge](contracts/bridge-protocol.md), [native](contracts/native-integration.md) contracts as normative semantics. Do not infer success from process exit, a save acknowledgment or the finalizer's copied fields.

## 1. Prerequisites and exact candidate

- Reviewed spec/plan and the relevant approved one-task/one-PR implementation increment from [tasks.md](tasks.md). Task derivation, granularity review and read-only analysis are complete; implementation remains separately authorized work. This guide does not authorize implement-all.
- Owned GUI macOS **26.6.2 arm64** fixture environment, with visible Script Editor/CodeEdit and permission to capture only its owned window. No real developer project content.
- Official stock Godot **4.7.2.stable.official.ed1daf0bf**, executable SHA-256 `c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf`, plus a matched standard public-ABI C++17 GDExtension. No source checkout or engine patch.
- Existing Rust **1.98.1** with rustfmt/clippy and tracked lockfile; Apple command-line C++ toolchain/SDK; Python **3.10+**. Record actual native build-tool versions and provenance.
- Compatible exact-version export templates; editor-only tooling must not enter production exports. Verify actual exported execution, not only a preset's text.
- Empty absolute mode-0700 artifact directories and a draining stdout consumer. Harness starts/stops only its owned editor/runtime children and records bounded waits and cleanup.

The implemented native build entrypoint is:

```sh
python3 godot-addon/native/build.py --godot "$EDIT_GODOT"
```

It generates the matched public GDExtension C ABI, compiles the owned native
source, installs the editor-only artifact and records build/ABI provenance.
The [native build guide](../../godot-addon/native/README.md) has the current
stock-only primitive reproduction. The build does not download/build Godot.

Confirm actual candidate/tool identities during implementation:

```sh
"$EDIT_GODOT" --version
rustc +1.98.1 --version
python3 --version
```

The harness checks native capabilities/build identity and exercised behavior;
a version string alone is insufficient. Unmatched/missing native binaries
retain supported observation behavior without granting edit capability.

## 2. Rust and boundary checks

From `mcp-server/`, run the repository baseline for the implemented package:

```sh
cargo +1.98.1 fmt --all -- --check
cargo +1.98.1 clippy --all-targets --locked -- -D warnings
cargo +1.98.1 test --locked
cargo +1.98.1 doc --no-deps --locked
cargo +1.98.1 build --locked --lib --bin observe-gdscript --example stock_validation_fixture
```

`cargo test --locked` includes doctests. No root workspace or blanket `--all-features` is introduced. Tests must cover consumer-visible invariants and transitions: prior-observation basis eligibility, dirty-equal/stale/same-text-version distinction, independent postconditions, stage/application precedence, pre/post-authorization worker loss, immutable late results, strict v2 authentication/limits/identity, denied-source suppression and actual confined I/O outcomes. Do not add source-text/wiring-copy/mock-echo tests.

The T003 native acceptance used real descriptors, Script/CodeEdit objects and
independent saved-state reads. Fault barriers are compiled only into separate
`GAK_FIXTURE` test artifacts, never the normal product library.

## 3. Owned real-editor runner

Reuse existing observation harness setup, authentication, owned-window
capture, independent D/R/B/dirty/history witnesses, process cleanup and
artifact handling. **Current T003 reproduction is only `native-primitives`**
with the exact stock fixture inputs in the [native guide](../../godot-addon/native/README.md#official-stock-boundary-and-verification).
The public caller and whole-feature runner groups below are T004/T005
acceptance requirements, **not implemented scenarios or CLI options today**.
There is no `--mutator` or `--scenario all` in the current script-edit runner.

A request uses a newly returned complete clean agreeing observation as `basis`; the runner sends the input JSON directly to `edit-gdscript` stdin. Do not put source/credentials in argv or use a temp-source filename as a product escape hatch. Use a new request ID each attempt and preserve original request/session/document attribution in artifacts. Native Save/Undo/Redo/open/reparse/rescan/runtime actions are explicit **fixture/developer** interactions, not new product commands.

### Required groups and coverage

| Group | Spec scenarios / requirements | Required observable result |
|---|---|---|
| `clean-open` | US1.1–US1.4; FR-001, FR-003, FR-006–FR-009, FR-018; SC-001 | Real distinctive edit, non-current-tab target, unrelated dirty document, and unchanged intent. Separate D/R/B/dirty/saved/parse proof; one native history entry only for changed intent; no focus-based targeting or reconciliation. |
| `conflicts` | US2.1–US2.4, US2.7; FR-003–FR-005, FR-010–FR-011, FR-013; SC-002 | Dirty-different and dirty-equal, stale clean basis, same-text newer buffer version, divergence, known stale R/B, missing facts, changed identity and between-preflight/application human work all preserve operation-unmodified source/dirty/history. US2.4 also requires T004's same-session overlap barrier case below: busy/non-applied, no queued/late mutation, and fresh-basis recovery. Missing/unusable basis has no unconditional fallback. |
| `routing` | US2.5–US2.6; FR-002, FR-005, FR-011, FR-020; SC-002 | Exact/omitted session, two live sessions, unresolved liveness, ended/restarted session, closed/missing/non-GDScript/built-in/unknown-open/denied/outside-project/unsupported target. No source-before-selection, guessing, force-load/open, replacement session or mutation. |
| `interruption` | US3.1–US3.3, US3.5–US3.6; FR-007–FR-008, FR-010–FR-014; SC-003–SC-004 | Fail/cancel/lose connection before authorization, after authorization, during native application/persistence/finalization and verification. Actual survivor state matches reported applied/partial/unknown; no late application after proven refusal, false rollback or retry. New human work remains intact. |
| `validation` | US3.4 and invalid-source repair edge; FR-009–FR-011, FR-020; SC-003 | Actual valid root and GDScript dependency, invalid root/analyzer/dependency, dependency change, missing/unreadable/unconfined dependency, unavailable/unsupported effect. Native input hash/path/interval attribution is correct, not reload/log inference. Repair an initially invalid but coherent/clean source; never classify syntax error as non-script target. |
| `native-primitives` | US1.1–US1.3, US2.4, US3.1–US3.5; FR-004, FR-006–FR-013, FR-020 | Actual A receipt/binding/version/bookkeeping and B effect/guard boundaries described in §4; independent inspections, not result echoes. |
| `history` | US4.1–US4.4; FR-006, FR-015–FR-016, FR-018; SC-005 | One actual Undo restores full original B; ordinary Save makes original D/R/B agree; Redo survives that Save, then Save makes intended D/R/B agree. Earlier undo entries remain reachable; refused/unchanged add zero entries. |
| `durability` | US5.1–US5.2; FR-008–FR-009, FR-017; SC-006 | Ordinary Save, close/reopen, reparse and rescan retain intended source without reconciliation. At least one permitted runtime fixture produces the intended changed behavior, not merely launches. |
| `sequential` | US5.3; FR-004–FR-008, FR-013, FR-015–FR-017; SC-007 | ≥20 successful edits, each based on a fresh observation, with intervening Saves and ≥3 interleaved dirty/stale refusals. No lost human work/revision/history; final source survives close/reopen. |
| `privacy-export` | US5.5; FR-020–FR-021; SC-008 | Authorized/ambiguous/denied/interrupted source sentinels, no incidental leakage, untouched unrelated targets; enabled/disabled-addon production export excludes actual native/tooling files and behavior. |
| Existing complete observation runner | US5.4; FR-019, FR-022; SC-004, SC-008 | Preserve all original observation scenarios, per-source limits, ≤5-second results and non-interference on the changed integration/exact candidate. Native capability installation never turns observation into validation, save or mutation. |

These rows cover all **26** story scenarios, **FR-001–FR-022** and **SC-001–SC-008**; FR-022 also governs the whole acceptance run. All five stories remain P1; passing a subset is not feature completion.

## 4. Native primitive and edge-boundary cases

### A: actual target save bookkeeping

Prepare a non-selected target, another dirty document and known prior native
history. Record independent D bytes/identity and retained-fd T0 mtime,
Script source/public edited flag, actual CodeEdit text/current/saved versions,
document-attributed dirty state, current selection and history reachability.
No private Resource/document numeric mtime equality is required.

Exercise:

- Successful native bound write + A: exact target is clean/synchronized, no unrelated save/history/selection change, no broad saved notification/runtime reload.
- Missing/failed/unknown/mismatched receipt; receipt for another attempt/file/document; source or identity mismatch; same-text newer buffer version; closed/replaced document; Resource replacement. No false clean tag.
- New human B after source application or persistence: preserve that B and actual dirty/history state, expose applied/partial result and never reassert the old request.
- File/parent/leaf replacement and rename/unlink around guarded descriptor I/O: never write replacement/outside sentinels; report loss of namespace/current-target coherence even if the retained original object received bytes.
- Changed D known before persistence; short write/truncate/flush failure; loss after real write; each A bookkeeping-step failure. Result flags and independent state identify actual changes, including partial metadata.
- The observed regression **apply → Undo → ordinary script Save → Redo → Save**: same-fd T0 restoration/readback before the saved tag must avoid false outside-change reconciliation and retain native history.
- Save formatting profiles: refuse before mutation if baseline or desired source would be changed by native trailing-whitespace/final-newline/indent conversion. Do not change preferences to make product results pass. Compatible representations must retain exact bytes and history through ordinary Save.

The same-inode critical-window counterexample remains an explicit unpromised atomicity limit. Do not require a new lock/CAS system or claim all transient writes can be detected. Where invalidation is actually known to the system, it must prevent success even if later bytes match. A fixture controller's additional knowledge cannot be silently presented as a product observation.

### Same-session overlap (US2.4; T004)

This is T004's real routing/conflict/interruption acceptance, not a new story, task or concurrency service. Use two actual caller requests, the same selected editor/session/script, fixture-owned barriers and independently read D/R/B, current/saved versions, dirty state and native history. Boundary/finalization witnesses must identify the actual attempt; do not infer non-entry only from equal text or a response echo.

1. Observe clean revision X. Start A and hold it after supervisor authorization while its prepared attempt owns the existing slot, before mutation, with an explicit barrier that lets the integration handle B.
2. Submit B with the same basis X. Verify `refused`/`busy`, `application: not_applied`, no B boundary entry and zero B-caused source/history/finalization change while A remains held. Establish that B's attempt is terminal with no retained preparation/apply work, not merely waiting.
3. Release A and require its normal verified edit to Y. Release controlled deliveries/work and verify no late B application or finalization and no retained B work after slot release; source/history changes are attributable only to A.
4. Submit a new request with the old B basis X, including desired text equal to Y: it must remain stale, not `verified_unchanged` or an automatic rebase. Then observe Y afresh and prove a new valid request can edit to Z.

In the existing interruption cases, hold an already-entered A native stage through caller timeout/disconnection. A retains truthful applied/unknown knowledge and its bounded caller result; no other attempt may enter mutation while that stage can still mutate. Release the barrier, observe survivor state and terminal cleanup before a later fresh-basis edit. This is not rollback or permission to block human typing. Existing before/after-application human-edit cases remain mandatory.

### B: exact-source stock LSP validation and live-editor effect boundary

The T003 Rust worker captures bounded no-follow root and literal transitive
`.gd` dependencies, resolving paths against each referring source, and stages
only the admitted closure in a private disposable source-only project. One
matching official Godot LSP child per immutable pass supplies per-URI
diagnostics followed by shaped parser symbols. Require all root/dependency
fences before `valid`/`invalid`; missing evidence, unsupported context and
unsafe effects are `unavailable`, not a false valid result.

The helper does not mutate the selected editor's D/R/B or start a broad
`--check-only` execution path. Separately guard ordinary live-editor
validation/export effects: T003 native admission refuses tool scripts,
inheritance, load/preload, class registration and exported declarations
before native history entry. A parser-only result does not certify a queued
editor continuation. T004 must bind fresh effective context and independent
post-change validation to the authenticated public caller; T002's oracle
evidence in §10 is historical and not a current gate.

### Additional representation, availability and access edges

- Empty source is observed empty, not unavailable. Exercise empty replacement and an unchanged empty source where native Save representation is compatible.
- Unicode, tabs, whitespace/final-newline differences, exactly 512 KiB and 512 KiB + 1; preserve exact supported text or refuse unsupported NUL/CR/BOM/save-transforming text before mutation. Never silently normalize/truncate.
- Exactly supported dependency/diagnostic bounds and one over each bound; unavailable/limited diagnostics must not become valid. No broad project scan to fill a missing dependency.
- Read-only file before application and real persistence failure after source change; distinguish refusal from partial application. Unchanged verification performs no write and does not require write access.
- Restart/disable, document rename/delete/reopen, missing R/B/dirty/saved-state observations and permission loss; preserve identity attribution and application facts rather than mix replacement observations.
- Original syntax-invalid source repaired successfully under the same clean/coherent/basis rules; post-change parse failure/unavailability remains explicit and blocks success.

## 5. A–E and durability witness protocol

Every accepted result is bracketed by **independent** fixture witnesses, separate from mutation return fields. Include an owned-window image showing the actual selected CodeEdit and dirty indication at key steps; review its actual surface, not merely existence of an image file. Record native history operations through real editor actions, not calls that set text to expected undo/redo values.

| Gate | Required sequence |
|---|---|
| A | Clean open basis → actual caller edit → independently observed intended D/R/B, clean/saved metadata, fresh native valid parse; no reconciliation. |
| B | Distinctive unsaved human source and dirty-equal case → edit refusal → exact human state/history preserved. Also inject change between preparation and boundary. |
| C | Known prior history → actual agent edit → one Undo → observe actual D/R/B/dirty → ordinary Save → converge original → Redo → observe → Save → converge intended → reach earlier native entries. |
| D | Verified edit → ordinary Save → close/reopen fixture script → fresh independent target/source observations, no stale/reverted buffer. |
| E | ≥20 fresh-basis successful changes with Saves and ≥3 unsafe interleavings → independently check each transition → usable history and final close/reopen persistence. |

Reparse/rescan must finish via actual events before sampling. Launch an authorized fixture runtime whose observable result encodes the changed source revision; record that result and the editor's surviving D/R/B state. At least one applicable runtime proof is mandatory. An unavailable runtime, failed launch or no behavior witness is non-passing, not “not applicable.” No fixture action exposes runtime control in the product API.

Time **every** controlled edit externally from caller start through terminal-result receipt, including hanging stdin/editor/native/I/O cases as applicable. Every edit ≤10 seconds and observation ≤5 seconds; no percentiles/subset timing or excluding failed cases. After a proven pre-application refusal/cancellation, keep the owned editor alive long enough to release any controlled pending work and prove that no late apply occurs. After uncertain results, release controlled stalls and independently inspect survivor state without assuming rollback; cleanup only owned processes.

## 6. Observation and production-export regression

Run the existing full observation suite using the migrated caller/addon against the exact changed candidate:

```sh
python3 godot-addon/tests/run_observation.py \
  --godot "$EDIT_GODOT" --observer "$OBSERVER" \
  --scenario all --artifacts "$OBSERVATION_ARTIFACTS"
```

Retain clean/dirty/divergent/closed/partial/limit/routing/interruption/privacy/export and native-history non-interference behavior, including the existing twenty-read sequence. Update private v2 peers without re-pinning tests to changed incidental text or weakening assertions. Record the actual result count; the prior 194-case evidence is historical, not a new passing run.

Export checks cover addon enabled, disabled and hook-only isolation with the
installed stock native artifact. Inspect PCK contents and run exported fixtures;
neither tooling binaries nor a dangling extension dependency may ship. The
accepted T003 stock exports passed these private gates; T005 repeats complete
feature compatibility. No patched gameplay/editor authority or new CI workflow
is required.

## 7. Evidence and completion report

Record exact spec/gate mapping, request/target and observed stage identity,
before/after independent D/R/B availability/hash/identity, current/saved
versions, public edited/dirty state, retained-fd T0/readback, actual native
history actions, source-attributed helper diagnostics, elapsed time and owned
artifacts. Missing data remains explicit, not synthesized from intent or
native receipts. Keep requested source-bearing evidence distinct from redacted
routine output and private session metadata. Record exact OS/architecture,
Rust/native toolchain, official executable SHA-256/version, ABI/native build
ID, addon/core commit and export-template provenance. Historical patched
build identities belong only to §10/PR #29, not current reproduction.

Completion of T004/T005 requires the applicable positive/refusal/partial
cases, A–E, runtime durability, observation preservation and privacy/export
gates. Unsupported/refused output for all edits is not completion. Task
acceptance is separate from PR merge; feature/Phase 1 completion requires its
own assessment. T001–T003 are complete, not full-feature mutation acceptance.

## 8. Planning-only verification boundary

The following command records the earlier plan-generation path check. Subsequent task generation and its granularity review are recorded in [tasks.md](tasks.md); neither workflow establishes product/runtime acceptance. Resolve the feature directory through the installed helper with the verified override:

```sh
SPECIFY_FEATURE_DIRECTORY="$PWD/specs/002-edit-open-gdscript" \
  bash .specify/scripts/bash/check-prerequisites.sh --json --paths-only
```

Validate local documentation links/anchors, Markdown/code fences, spec/design coverage and whitespace, then inspect complete intended/staged diffs and publish only these design artifacts on the existing branch/PR. Check installed before/after-plan extension hooks rather than bypass them. No product build/suite, native probe, tasks or implementation is included in these planning checks; no A–E result is claimed.

## 9. T001 core acceptance (2026-09-27)

**Complete:** protocol-independent eligibility, evidence and terminal interpretation
under `mcp-server/src/script_edit/`, exposed through the `mcp-server/src/script_edit.rs`
facade and the existing library. The
[public Rust API](contracts/edit-api.md#7-implemented-rust-core) is reusable without a
CLI, JSON, sockets, native objects or a second policy. T002–T005 remain pending.

### Executed evidence

Environment: macOS **26.6.2**, build **25G83**, **arm64**;
`rustc 1.98.1 (48a229cea 2026-09-01)`, host `aarch64-apple-darwin`;
`cargo 1.98.1 (797e8a9bc 2026-08-05)`. No Godot binary was exercised by this task.

From `mcp-server/`:

| Command | Observed result |
|---|---|
| `cargo +1.98.1 fmt --all -- --check` | Passed. |
| `cargo +1.98.1 clippy --all-targets --locked -- -D warnings` | Passed. |
| `cargo +1.98.1 test --locked --test script_edit_contract` | 48 passed. |
| `cargo +1.98.1 test --locked -- --test-threads=1` | 153 passed: 10 library, 42 bridge, 21 confinement, 32 observation and 48 edit-core tests; binary/doctest targets also completed. |
| `cargo +1.98.1 doc --no-deps --locked` | Passed. |

The first default-parallel full test run encountered `AddrInUse` at
`bridge_boundary.rs`'s existing `rebound_ended_port_without_the_original_secret_cannot_release_project_source`
release-and-rebind window. The complete serial run passed that case and every other
case without changing bridge code or weakening its assertions. The initial source
contract tests failed before implementation because `script_edit` did not exist.
Newly discovered reducer regressions were observed failing and then passing.

A throwaway **external Cargo consumer**, depending on this package by path, ran with
`cargo +1.98.1 run --locked --offline` after offline lockfile generation. It supplied
synthetic, separately attributed typed evidence through the public API and asserted:

```text
Refused            application=NotApplied  reason=DirtyConflict
VerifiedUnchanged  application=NotApplied  reason=Complete
ApplicationUnknown application=Unknown     reason=Deadline
AppliedUnverified  application=Applied     reason=Disconnection
VerifiedChanged    application=Applied     reason=Complete
```

This was an executed library-consumer smoke, **not actual source mutation or a
live-editor witness**. The disposable consumer is not part of the product.

### Acceptance and constitutional review

The 48 regressions cover eligible/ineligible and missing prior basis; actual current
version versus absent prior saved version; dirty-equal text; empty/exact UTF-8 and
512-KiB limits; stale same-text versions; each missing/invalidated source; wrong
request/session/document/clock and overlapping collection intervals; save-profile
and metadata guards; native attribution and permanent pre-boundary discard;
authorization/entry/lost replies; independently known partial effects without
invented earlier completion; bounded source-attributed validation; absent/invalid
post-change parse; dependency/context invalidation; independent saved/D/R/B readback;
source suppression under later timing/limit failure; all five outcomes and
source-free disambiguation. Terminal reduction consumes the attempt.

Review corrected two particularly consequential errors: terminal timing failure
must not clear denial, and A's entry source/version must describe the already-applied
document rather than the pre-edit basis. Known native effects are retained even if
earlier acknowledgments are absent; they are never independent verification.

Principles I–IV/X/XII: independent source/dirty/saved/parse evidence and stale-work
refusal remain mandatory; interrupted/partial application never becomes rollback or
safe replay. V/VII/IX/XI/XIII: existing checked observation types and locked `ring`
are reused in one focused core module; no transport, crate, service, native runtime,
permission surface or general transaction framework was added. The existing
observation implementation and public behavior remain unchanged.

Ownership review: script-edit policy is capability-specific; shared selection,
identities, clocks and observation remain in their existing responsibility-based
components. No feature/task IDs enter product APIs and no duplicate future
infrastructure is introduced.

**Limits:** typed facts still require authentic acquisition by the future adapter
and native integration. No CLI edit, bridge-v2 migration, engine patch, actual
Undo/Redo, mutation timing, A–E, export/durability or Godot-support claim follows from
T001. Those feature gates remain mandatory and pending, not inapplicable or passed.

## 10. T002 native validation acceptance (2026-09-28)

**Historical acceptance — complete at the time, superseded by T003.** T002
implemented a confined non-mutating GDScript parser/analyzer validator in a
patched Godot 4.7.2 editor with a matched public-ABI GDExtension. The engine
patch, oracle implementation/build, `native-validation` runner path and
patched-editor CI have since been removed from HEAD; this is **not a current
reproduction recipe**. The implementation and full contemporaneous evidence
remain in [merged PR #29](https://github.com/Peter-Tam/godot-agent-kit/pull/29)
and repository history. Do not build or run the historical oracle for the
stock T003/T004/T005 path.

The executed T002 candidate was macOS **26.6.2 arm64**, custom Godot
`4.7.2.stable.custom_build.ed1daf0bf` on base
`ed1daf0bf001b61586d9930840f2f1394092c079`, Rust **1.98.1**,
Apple clang **21.0.0**, SDK **27.0**, SCons **4.10.1**, with the
tests-enabled patched editor SHA-256
`45a64b260c8347b4496bf7d0caabcbf2ff2e49530d051bf0434e8866990ba338`.
The patch SHA-256 was
`1b1508c4c79dcb77af86685aef828e97b0d5857be6e4285ff2b7b72ecfe75f2a`;
native build ID was
`5c76fe0f323be3189970f16351d88adcd27cc5cd4beba477b1bc84094b34c362`.
These identify *past* evidence, not runnable artifacts in the current tree.

**Observed at T002 acceptance:** 54 native GUI/lifecycle/export cases passed
(bootstrap, 50 validation/lifecycle, three real exports); 194 official-stock
observation cases passed with the native artifact installed; the separate
missing-native D/R/B smoke passed. Rust formatting, Clippy, rustdoc and
155 tests passed, as did three native ABI refusal and 20 workflow regressions.
The matched local editor/extension build passed; hosted CI was a build check,
not hosted GUI acceptance. Native checks covered source/dependency and
diagnostic attribution, confined/effect refusal, cache and context changes,
wrong session/thread, reentrancy, lifecycle and input/diagnostic bounds.
Independent source/dirty/history witnesses and enabled/disabled/hook-only
exports established non-mutation and tooling isolation, **not** edit support.

The accepted candidate did not implement an edit caller, guarded persistence,
finalization, agent Undo/Redo, mutation A–E or feature durability. Those limits
remain historically accurate; T003's later stock acceptance is in §11.

## 11. T003 native-boundary acceptance (2026-09-29)

**Task state: Complete.** Only T003 was selected, on
`task/T003-stock-validation-native-edit` from updated `main`. Completion is based
on the implemented boundary, implementation-shape review and executed acceptance,
not PR merge state. T004/T005 and the public edit caller remain unimplemented.

### Implemented boundary

- Rust `runner::stock_validation` owns the same-executable supervised helper.
  Private admission, protocol and ownership modules separate bounded confined
  capture/context, per-source completion fences and process lifecycle. The
  existing worker reaper is shared; no new crate/service is introduced.
- The helper stages only admitted exact `.gd` sources. Dot-relative/transitive
  references resolve at their referring source; source aliases and staging
  collisions refuse. Effective selected-editor warning overrides and bounded
  global-class names are explicit context, not inferred from stripped disk
  configuration. Every source requires diagnostics before its symbol response.
- The standard-ABI native attempt applies CodeEdit → Script R → same-fd D
  persistence → same-fd T0 restoration/readback → public edited=false → saved tag,
  then stops. It retains stage facts on partial failure and shares the existing
  observation slot, including synchronous cancel/shutdown callbacks.
- Live-editor effect admission is narrower than helper admission: tool sources,
  script/global-class inheritance, preload/load references, class registration
  and exported declarations refuse before source/history work. Parser inertness
  does not certify queued editor export effects.
- Worker loss leaves unknown spawn/reap facts nullable. Duplicate configured
  stdio descriptors are dropped after spawn so EOF is observable; bounded
  cleanup acknowledgment uses the existing reaper within the operation budget.
  Validity cannot be reported without confirmed scratch cleanup.

### Executed checks

Environment: macOS **26.6.2 (25G83), arm64**, official Godot
`4.7.2.stable.official.ed1daf0bf`, executable SHA-256
`c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf`;
Rust **1.98.1**, Apple clang **21.0.0**, SDK **27.0**.

| Executed validation | Result |
| --- | --- |
| Rust formatting, all-target Clippy `-D warnings`, locked serial test suite, documentation build | Passed; 176 tests, plus the doctest target (0 doctests) |
| Stock and isolated fixture-fault ABI builds; historical patched-oracle build (archived in PR #29) | Previously passed; three ABI refusal tests also passed |
| Workflow regression tests, Actionlint, Python compilation and runner help | Passed; 20 workflow tests |
| Official-stock `run_script_edit.py --scenario native-primitives` | **149 cases passed**: bootstrap + 58 helper + 87 native finalization + 3 export |
| Official-stock `run_observation.py --scenario all`, stock artifact installed | **194 cases passed**, all 13 groups |
| **Historical only:** matched patched-oracle `native-validation` campaign (removed from HEAD) | **54 cases passed then**: bootstrap + 50 validation + 3 export; not a current gate |

These are previously executed local results, not a claim that hosted CI ran.
The normal stock artifact was restored after the historical oracle run.
Current stock reproduction and required fixture inputs are in the
[native guide](../../godot-addon/native/README.md); the patched recipe is
archived in [PR #29](https://github.com/Peter-Tam/godot-agent-kit/pull/29).

The helper cases include Unicode/empty/error sources, relative/transitive and
unused-preload dependencies, same-size dependency changes, warning/directory
overrides, global-class context, all-source attribution, limits and prelaunch
refusals. Three additional-client cases preserve the owner's verdict despite
opposite root/dependency overlays on another connection. Actual worker loss and
stopped-child deadlines terminate the owned engine and remove private scratch,
without changing the selected editor or unrelated dirty work.

Native GUI cases independently observe D/R/B, current/saved versions, dirty state,
object association and exact T0 restoration. They cover clean/non-selected edits,
empty and 512 KiB Unicode source, invalid-script repair, short writes, real
read-only/EBADF failures, injected I/O failures, mismatched restoration, changed
human text/versions, close/reopen identity changes, leaf/parent replacement,
reentrant cancellation/shutdown, native Undo → Save → Redo → Save and earlier
history. Explicit completed filesystem rescan, fresh editor reparse and fresh
runtime launch preserve the persisted revision. A later human Save actually
persists new text without false outside-change reconciliation.

After independently verified native application, changing the selected effective
warning policy produces a real invalid post-change result; a stopped separate
post-change helper produces unavailable. Both preserve the already-applied
source/history and unrelated human work. No rollback or final caller outcome is
invented; T004 owns integration with the common outcome reducer.

Enabled, disabled and hook-only exports exclude native/tooling artifacts and
dangling dependencies; their launched applications expose no tooling listener or
native registration. Source-free incidental output and requested bounded
diagnostics pass the existing redaction checks.

### Evidence and review

Local evidence root:
`/var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-agent-kit-edit-vwzsf4w7/`.
These paths are local evidence, not portable downloads.

| Final summary | SHA-256 |
| --- | --- |
| `gui-cutover-stock/summary.json` | `dbdd35ea2263d2c1878a78ede88557f97c7693724f2450f3606e320bfe797900` |
| `gui-cutover-observation/summary.json` | `fe437e4966174bf6ef5dfe60e93dbb2c6bb0783d06312de0d7e0e7e9dc4ec3bc` |
| `gui-cutover-oracle/summary.json` (**historical, not current gate**) | `8a90fd92b4f0148f9f86a41137581a553fbb4530bfe3e41cb5d3ea2e55bf24cd` |

The stock native build ID is
`6837dd50b3c91a2de7e2e4a26df074703177de4e501826797ab9ad13f02cac9c`;
tested library SHA-256 is
`81fa84219ca8a34401f4131b23033f3e63c1af784c5ac5afa03a6efba3fddbeb`.
The historical oracle build ID was
`4823e8980898792e6ceb0be88ab187d879c5847a6acb4b88135011838e410c23`
against T002's then-unchanged patched engine SHA-256
`45a64b260c8347b4496bf7d0caabcbf2ff2e49530d051bf0434e8866990ba338`.
These are archived measurements, not HEAD build inputs. The summaries retained
exact source, driver, ABI, manifest and artifact hashes.

Earlier locked-desktop, focused-smoke and incomplete GUI runs remain recorded as
failed or scoped evidence; they were not relabeled as acceptance. The unlocked
run corrected fixture assumptions: request dispatch consumes one parsed message;
rescan is explicitly requested and observed; fresh setup opens both witness
documents; expiry tests forward the actual boundary value; refusal compares
unrelated work across the native call rather than across a preceding human tab
reorder. Existing groups close completed case editors. All post-B/R I/O failures
must retain partial application, even before disk persistence.
That earlier campaign's capability cutover removed an unsupported stock-native
validator stub and unused fixture-build flag alias. All three historical GUI
campaigns above ran against that earlier cutover; the subsequent stock-only
cleanup does not rerun or depend on the oracle.

The [implementation-shape/constitutional review](plan.md#t003-implementation-shape-and-constitutional-review)
records module responsibility, narrow visibility, ordinary editor-effect limits
and the concrete cleanup/slot hazards addressed. No new dependency, engine
patch, security service or approval gate was introduced. Owned fixture editors,
projects and pre-fix leaked helper scratch were removed; evidence and required
build outputs remain outside tracked source.

### Stock-only HEAD verification

The same T003 PR removed the superseded T002 implementation rather than keep a
second runnable architecture. Current reproduction uses only the pinned official
Godot executable, its generated public ABI, the standard native extension and
Rust one-shot LSP validator. `session.cpp` retains project/session/confinement
support; private metadata is `godot_agent_kit_native`. The engine patch, patched
validator/bindings, oracle build mode, runner group and exclusive fixtures are
gone. Normal CI downloads and verifies the official archive/executable and builds
that extension; it does not check out or compile Godot or install SCons.

Post-cleanup local verification on that official executable:

| Current verification | Result |
| --- | --- |
| `cargo fmt --all -- --check`, all-target locked Clippy `-D warnings`, locked serial tests and rustdoc | Passed; **176 tests**, including the doctest target (0 doctests) |
| Normal and isolated `GAK_FIXTURE` native builds | Passed against the exact official binary/public ABI |
| Native ABI/binary refusal tests | **5 passed**; actual build CLI also refused an unknown executable with exit 2 |
| Workflow regressions, Actionlint, Python compilation, stock-only runner help | Passed; **20 workflow tests** |
| Stock `native-primitives` | **149 passed**: bootstrap + 58 helper + 87 native finalization + 3 export |
| Complete stock observation regression | **194 passed**, all 13 groups |
| Final focused export rerun | Bootstrap + **3 actual export variants passed**, with fixture-file exclusion retained |

The final review restored the existing `fixture_driver` bundle exclusion while
removing only the obsolete `libreentry` check. The focused fresh-process rerun
exercised that final assertion; no production code changed after the aggregate
run. An earlier in-process export attempt loaded a stale pre-cutover Python
module and failed before export; its `final-export/summary.json` remains failed,
not counted as acceptance. Current-source execution is recorded separately.
Neither the patched editor nor its historical 54-case suite was rebuilt or run.

Local evidence root:
`/var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-agent-kit-stock-cutover-dq71esjm/`.

| Summary | SHA-256 |
| --- | --- |
| `native-primitives/summary.json` | `47f9614d102fcdc1ad3b38972662fe77bece4dea472f3ec00837fa8786d16f67` |
| `observation/summary.json` | `cf711ba7db0484be7befecbe1cb7a8bbfe2f3251e592c51ac242a6ee2c7822a7` |
| `final-export-current/summary.json` | `5658e606af7ba9f8167c3f252395e949adfd96d37dfa88c8225efb17128a550d` |

Normal native build ID:
`841a099537cd59447bd2da98109f837c6c91f0b8e8392490d2ca85a30bf52a21`;
library SHA-256:
`6919285c071347863a72c0ed6da3a45afddd784c02579416bc44636eb16b4b3b`.
Fixture-only build ID:
`4bcb6299b1524d9eef1634cc5ddc91529680c9a3330f5b29c34a1be40154f3eb`.
These are the stock-only cleanup candidates, not the earlier identities above.
Evidence is local, not a claim of hosted CI or portable artifact availability.
Owned fixture processes/projects were cleaned up. Historical research/status
snapshots and T004/T005 task sections were preserved; completion stays 3/5.

### Remaining feature boundary

T003 completes the private helper/native increment, not Feature 002 or Roadmap
Phase 1. T004 must migrate the historical typed-core evidence and implement the
authenticated caller/bridge path. T005 owns cumulative feature acceptance.
There is no public edit command, bridge-v2 cutover, MCP tool, wider-platform
support, active-runtime hot-reload claim or OS sandbox. The approved inherited
stock-LSP endpoint limitation remains non-blocking and unchanged.
