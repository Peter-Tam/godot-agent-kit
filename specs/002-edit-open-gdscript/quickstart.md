# Quickstart: Verify Guarded Open-GDScript Editing

**Status:** **T001–T005 and Feature 002 are complete.** [T005 cumulative acceptance](#14-t005-cumulative-acceptance-2026-09-29) records the current complete edit/native and observation runs. Earlier task records below retain their historical scope and completion state at the time. The private bridge remains v2 and public caller/observation schemas remain v1; T005 changes acceptance coverage, not product semantics.

**Current reproduction:** Use the official stock executable and matched standard
public-ABI extension from the [native guide](../../godot-addon/native/README.md).
The caller groups, retained `native-primitives`, cumulative `durability`,
`sequential`, `privacy-export`, and complete `all` mode below are implemented.
Historical patched-family/mtime sketches are superseded by the current caller,
bridge, native and data-model contracts. §10 is an archive summary, not an
oracle recipe.

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
cargo +1.98.1 build --locked --lib --bin observe-gdscript --bin edit-gdscript --example stock_validation_fixture
```

`cargo test --locked` includes doctests. No root workspace or blanket `--all-features` is introduced. Tests must cover consumer-visible invariants and transitions: prior-observation basis eligibility, dirty-equal/stale/same-text-version distinction, independent postconditions, stage/application precedence, pre/post-authorization worker loss, immutable late results, strict v2 authentication/limits/identity, denied-source suppression and actual confined I/O outcomes. Do not add source-text/wiring-copy/mock-echo tests.

The T003 native acceptance used real descriptors, Script/CodeEdit objects and
independent saved-state reads. Fault barriers are compiled only into separate
`GAK_FIXTURE` test artifacts, never the normal product library.

## 3. Owned real-editor runner

Reuse the observation harness's owned setup, authentication, window capture,
independent D/R/B/dirty/history witnesses, process cleanup and artifact handling.
Caller scenarios are `clean-open`, `conflicts`, `routing`, `interruption`,
`validation`, `history`, `durability`, `sequential` and `privacy-export`; each
requires `--editor` naming the absolute built `edit-gdscript` path. The retained
`native-primitives` group uses `--stock-validator` naming the test-only stock
validator. `interruption` and `native-primitives` require the separate fixture-only
native artifact. `all` requires all three inputs and executes every edit/native
group, including production exports, once; it does not substitute for the
separate complete observation runner. There is no `--mutator`.

For each caller group, use a new empty mode-0700 artifact directory:

```sh
python3 godot-addon/tests/run_script_edit.py \
  --godot "$EDIT_GODOT" --observer "$OBSERVER" --editor "$EDIT_CALLER" \
  --scenario "$SCENARIO" --native-fault-addon "$PRIVATE_FAULT_NATIVE" \
  --artifacts "$ARTIFACTS"
```

For complete feature acceptance, use `--scenario all` and additionally pass
`--stock-validator "$STOCK_VALIDATION_FIXTURE"`. Run the complete observation
suite in §6 afterward. Groups own separate projects/evidence subdirectories
and close their editors before the next group; no GUI groups run concurrently.
Every caller outcome has a result-only semantic review and an externally
measured ≤10-second result bound. Independent fixture witnesses remain required.

Real-editor acceptance requires an unlocked visible GUI and owned-window captures.

Getter/state diagnostics while that visual prerequisite is unavailable may expose
bugs, but do not satisfy the task's GUI acceptance or authorize completion.

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

T002 completed the then-approved patched-engine native validation task. T003
later superseded that implementation, which has been removed from HEAD.

Recorded acceptance passed **54 native GUI/lifecycle/export cases**, relevant
observation regressions and the then-required build/regression checks.
[PR #29](https://github.com/Peter-Tam/godot-agent-kit/pull/29) and git history retain
the old implementation and full contemporaneous evidence.

**T002 remains complete**, but is not a current build/runtime dependency or a
reproduction path maintained on HEAD.

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
| Stock and isolated fixture-fault ABI builds | Passed against the exact official binary/public ABI |
| Workflow regression tests, Actionlint, Python compilation and runner help | Passed; 20 workflow tests |
| Official-stock `run_script_edit.py --scenario native-primitives` | **149 cases passed**: bootstrap + 58 helper + 87 native finalization + 3 export |
| Official-stock `run_observation.py --scenario all`, stock artifact installed | **194 cases passed**, all 13 groups |

These are previously executed local results, not a claim that hosted CI ran.
Current stock reproduction and required fixture inputs are in the
[native guide](../../godot-addon/native/README.md).

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

Current stock acceptance summaries, native build/library identities and local
evidence paths are recorded under [Stock-only HEAD verification](#stock-only-head-verification).

Earlier locked-desktop, focused-smoke and incomplete GUI runs remain recorded as
failed or scoped evidence; they were not relabeled as acceptance. The unlocked
run corrected fixture assumptions: request dispatch consumes one parsed message;
rescan is explicitly requested and observed; fresh setup opens both witness
documents; expiry tests forward the actual boundary value; refusal compares
unrelated work across the native call rather than across a preceding human tab
reorder. Existing groups close completed case editors. All post-B/R I/O failures
must retain partial application, even before disk persistence.

The [implementation-shape/constitutional review](plan.md#t003-implementation-shape-and-constitutional-review)
records module responsibility, narrow visibility, ordinary editor-effect limits
and the concrete cleanup/slot hazards addressed. No new dependency, engine
patch, security service or approval gate was introduced. Owned fixture editors,
projects and pre-fix leaked helper scratch were removed; evidence and required
build outputs remain outside tracked source.

### Stock-only HEAD verification

Current reproduction uses only the pinned official Godot executable, its generated
public ABI, the standard native extension and Rust one-shot LSP validator.
`session.cpp` retains project/session/confinement support; private metadata is
`godot_agent_kit_native`. Normal CI downloads and verifies the official
archive/executable and builds the extension without building Godot.

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

The focused fresh-process export rerun verified the existing `fixture_driver`
bundle exclusion; no production code changed after the aggregate run. An earlier
in-process attempt loaded a stale Python module and failed before export; its
`final-export/summary.json` remains failed, not counted as acceptance.

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
These identities belong to the accepted T003 stock-only delivery head.
Evidence is local, not a claim of hosted CI or portable artifact availability.
Owned fixture processes/projects were cleaned up. Historical research/status
snapshots and T004/T005 task sections were preserved; completion at that head was 3/5.

### Remaining feature boundary

T003 completed the private helper/native increment, not Feature 002 or Roadmap
Phase 1. At its accepted head, T004's evidence migration, public caller and
bridge-v2 cutover were not implemented. The current T004 implementation and
remaining acceptance are described at the top of this guide; T005 still owns
cumulative feature acceptance. No MCP tool, wider-platform support, active-runtime
hot-reload or OS sandbox is claimed. The inherited stock-LSP endpoint limitation
remains non-blocking and unchanged.

## 12. T004 working-tree verification — not GUI acceptance

This section preserves the pre-acceptance working-tree diagnostics. The Rust
baseline passed formatting, Clippy with warnings denied, **193 tests across nine
suites**, and rustdoc. Both callers, the library and the stock-validation example
were built with locked dependencies.

Owned real-Godot **state-only** diagnostics passed all six caller groups, the
complete **204-record / 13-group** observation campaign, native finalization and
native export checks. They exercised actual CLI outcomes, independent D/R/B
getters, native history, persistence faults, selected context, confinement and
redaction; they are not mocks or headless substitutes.

**Historical limitation, resolved by §13:** During these diagnostic runs,
CoreGraphics reported `CGSSessionScreenIsLocked=1`, and owned visible-window
capture was unavailable. Temporary diagnostic runners omitted captures and
explicitly recorded `visual_acceptance: false`, `support_claim: false`, and
`state_checks_passed_not_visual_acceptance`. These records remain state-only
evidence; the later GUI run does not retroactively relabel them.

Local evidence root:
`/var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-agent-kit-T004-xnj8rqqe/`.
Each row counts recorded checks, including that invocation's bootstrap record;
these are not counts of distinct user stories. The summaries retain their actual
engine, host, driver and fixture provenance.

| Summary under the evidence root | Records | SHA-256 |
| --- | --- | --- |
| `state-batch-5/clean-open/state-diagnostic.json` | 9 | `6be7c241c57c24432de79404cbdfccb1e0e68cd3377b97f2666f785050e31263` |
| `state-batch-6/conflicts/state-diagnostic.json` | 38 | `eac9fc396db915b6c11060fb9ead598157a982c374a08709ffff367bb0f48b70` |
| `state-batch-6/routing/state-diagnostic.json` | 36 | `7c480712de93957a19c5fbf155fb741a3b1336a2422e93baec9bf09b1ebd5f6a` |
| `state-final/interruption/state-diagnostic.json` | 39 | `f9100a6de211f1faf7d53e261645a242f6fa8a992f74fb590fd9cfbf02086dff` |
| `validation-state-isolated/state-diagnostic.json` | 39 | `e52b99a032262e3b16835fd33750cc0ddc8a3e8d5ed89cd7aaeb1e5a572ba654` |
| `state-batch-9/history/state-diagnostic.json` | 9 | `34d5a04bb28068a1e9e42d134187d34f3001d4932c2ca3ad00ed598fef609364` |
| `state-final/observation/state-diagnostic.json` | 204 | `7d8622bfa8c7aed606a07f7bd7ebf913364f4bffd5601540a8bf18dc78f3e32e` |
| `state-batch-12/native-finalization/state-diagnostic.json` | 88 | `4f9d512e59f3f190c4201c1ddccbcd4f477d655a3c58544eb0982dcaf4030281` |
| `state-batch-12/native-export/state-diagnostic.json` | 4 | `283cb0bb77b8f3b2cdc12755dd6b52eb34d3fed64a928c3782fec0c2b9a5b5fa` |

The stock-helper group also passed 58 checks in
`state-batch-9/native-primitives/state-diagnostic.json`; that enclosing campaign
remains failed because the following native fixture still sent v1 frames. The
later native rows above cover the corrected fixture, not a relabeled old result.
Other earlier failed campaigns remain failed. One earlier validation campaign
lost its returned result at a combined assertion; it is not classified as a
proven deadline defect. The focused repair and isolated full validation reruns
passed. The fixture now preserves credential-checked outcomes before expectation
assertions and closes completed validation editors instead of accumulating them.

The standard GUI campaign below subsequently closed the visible-editor gate.
Only T004 is complete; T005 and the full-feature/wider-platform gates remain
separate.

## 13. T004 caller acceptance (2026-09-29)

**T004 is complete** on `task/T004-guarded-caller-edit`, independently of PR
review or merge. The standard, unmodified runners passed all seven script-edit
groups and the complete observation suite serially on the unlocked owned desktop:
**170 caller records**, **149 native-primitive records**, and **204 observation
records**. Counts include each invocation's bootstrap record, not distinct user
stories. No capture bypass or diagnostic subclass was used.

The exact candidate was official `4.7.2.stable.official.ed1daf0bf`,
engine commit `ed1daf0bf001b61586d9930840f2f1394092c079`, executable SHA-256
`c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf`,
on **macOS 26.6.2 arm64**. Rust was 1.98.1; native provenance records
Apple clang 21.0.0 (`clang-2100.3.34.2`) and SDK 27.0.

### Executed behavior and visual evidence

- `clean-open`: actual public changed/unchanged outcomes, non-current-tab
  targeting, unrelated dirty work, later human Save/reopen and fresh-basis repeat.
- `conflicts` and `routing`: dirty/stale/equal-text/new-version and replaced
  identity refusal, missing evidence, denial precedence/source suppression,
  ended or ambiguous sessions, and busy overlap without queued/late mutation.
- `interruption`: bounded pre/post-authorization cancellation, timeout and
  disconnection; no late apply after proven refusal; retained application,
  survivor state and partial content/T0/finalization evidence.
- `validation`: actual root/analyzer/dependency errors, unavailable context,
  changed warnings, unsupported effects, invalid-source repair, exact Unicode
  and escaped-control transport at the source boundary.
- `history`: real caller apply → native Undo → ordinary Save → Redo → Save,
  preserved earlier human history, and no added entry for refused/unchanged edits.
- `native-primitives` and observation: independent native guards and fault
  receipts, Save/reopen/reparse/rescan/runtime durability, unchanged observation
  behavior, privacy checks, and actual exports excluding tooling/native binaries.

Owned-window captures were produced by the standard screenshot path. Visual
inspection included the caller's clean `subject.gd` showing `return 23` while
`other.gd(*)` remained dirty, the dirty synthetic buffer marker, and the fresh
reparsed editor showing the later human `return 29`. Synthetic parse errors in
the deliberately invalid fixture scripts remain attributable fixture evidence,
not an unreported target failure. D/R/B, dirty/version/history and timing
assertions accompany the captures; images alone are not the transaction oracle.

### Exact local artifacts

Evidence root:
`/var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-agent-kit-T004-xnj8rqqe/gui-acceptance/`.

| Summary | Records | SHA-256 |
| --- | --- | --- |
| `clean-open/summary.json` | 9 | `9751936e3feb4be8aecf55c60f89fa443dab7131d46cca83a1f6334c0b67e27f` |
| `conflicts/summary.json` | 38 | `31e1d731cad2649690ce1bb782da57a76b0128125e03f384f7ef5c45a7e4b736` |
| `routing/summary.json` | 36 | `d88bc07f362a8f142f4a6844222b1e988fe354ed90ac6b5c3e6def682605eb1c` |
| `interruption/summary.json` | 39 | `3015b6f24a898b6cbb0d37deefc30b99d4a2970dfbb0d6b09db7ddcd3d64cbbf` |
| `validation/summary.json` | 39 | `d94fcd32014ac6410109e96d35cf238eacc58706af2178b61adb3613d576cb3b` |
| `history/summary.json` | 9 | `6a60b7a3626b09c5e1269e96c4421824d75d83a1bfb047e41b48cb24cd4f5a4d` |
| `native-primitives/summary.json` | 149 | `2e14f208b6286104f08f0ecdfa2e9295541a0816af0c32193789ce547072a8a6` |
| `observation/summary.json` | 204 | `8401c9bdf669b7d3af6d31506519ee61b157ecf515c4c13bef600c174113d8b1` |

Normal native build ID:
`c36dd094486bbf26dfac0e6025c3d8cb12785e85e4940eef20ee31f73af74b1f`;
library SHA-256:
`656b53708ddb24bfc6678f7bcf06c821c35175fe572d5c6dd124caf60ac117b3`.
Fixture-only native build ID:
`73ec38cb4c5dc56bd9df73da5069b15e42683b8679d9d5dee8725f8a7d792262`;
library SHA-256:
`3298557f8cfa83e4cbe3c205eea7a1fa8504dedede819789916edb5a0e4e08a7`.
The exercised `edit-gdscript` SHA-256 is
`b53aa5985e7e0a77f8170b18a0a01d5f26b6e1387e997184ff614742eb567dad`;
`observe-gdscript` is
`c768b20f6dbc111aa4dc922de777b88982b8f7f476f174c6d657e77f432d0511`.

The Rust baseline in §2 passed **193 tests**, formatting, Clippy and rustdoc.
The native-build suite passed **5 tests**, workflow suites passed **20 tests**,
and Actionlint passed for both existing workflows. Owned processes/projects were
cleaned up. Only documentation, completion markers and two responsibility-based
comment corrections followed the GUI run; no behavioral implementation changed.

This is local acceptance, not a claim that hosted CI or optional GUI CI ran.
T005's cumulative durability, ≥20-edit stress and whole-feature evidence remain
unstarted. Feature 002 and Roadmap Phase 1 are not complete. The inherited stock
LSP endpoint limitation remains non-blocking; no MCP surface, broader platform
support, runtime hot-reload or OS sandbox is claimed.


## 14. T005 cumulative acceptance (2026-09-29)

**T005 and Feature 002 are complete** on `task/T005-cumulative-edit-acceptance`,
independently of PR review/merge. T004's PR #39 merged before this task began;
its local and remote source branches were already absent. Only T005 was selected.

The unmodified complete runners executed serially on the owned visible desktop:
`run_script_edit.py --scenario all` passed **405 records**, then
`run_observation.py --scenario all` passed **204 records**. These are actual
invocation/check records, including one bootstrap per suite, not counts of distinct
stories. No skipped groups, capture bypass, headless substitute for B or partial
`all` mode was used.

### Coverage and observed outcomes

| Edit/native group | Records |
| --- | ---: |
| stock-validation | 58 |
| native-finalization | 87 |
| clean-open | 8 |
| conflicts | 37 |
| routing | 35 |
| interruption | 38 |
| validation | 38 |
| history | 8 |
| durability | 9 |
| sequential | 76 |
| privacy-export | 10 |
| bootstrap | 1 |

The group mapping in §3 covers all 26 story scenarios, 22 functional requirements
and eight success criteria. Current-run evidence includes:

- **A/B:** real changed/unchanged caller outcomes; independent D/R/B, dirty/saved
  and source-attributed validation; dirty/stale/identity/denial/busy refusals;
  non-selected target and unrelated human-work preservation.
- **C:** actual native Undo → ordinary Save → Redo → Save, preserved earlier
  history, and no added entry for refused/unchanged requests. Partial and unknown
  outcomes retain actual stage/application knowledge without retry or rollback.
- **D and durability:** ordinary Save, close/reopen into a new CodeEdit,
  synchronous public `Script.reload()` completion, completed filesystem-change
  rescan, and a fresh fixture runtime reporting `NATIVE_RUNTIME_VALUE=23`.
  The original live editor retained clean intended D/R/B afterward.
- **E:** **20 distinct successful fresh-basis edits**, values 300–319, each
  followed by ordinary Save; **three stale and three dirty refusals** interleaved.
  Human value 29 survived each dirty refusal and an explicit human Save.
  Final native Undo/Save/Redo/Save restored 318/319, then close/reopen retained 319.
- **Result-only review:** all **113 public edit outcomes**, including stalled
  stdin, were inspected without fixture expectations for target, stage, actual
  application certainty, D/R/B availability/hashes, dirty state, history and next
  action. Partial/unknown actions require fresh observation and inspection of
  human work; no result advertised replay or rollback.
- **Timing:** every controlled public edit passed the ≤10-second bound;
  maximum **9.565838 seconds**, including the stalled-stdin case. The complete
  observation run's maximum measured result was **4.650398 seconds**.
- **Compatibility/privacy:** all observation groups, including twenty-read
  non-interference, passed unchanged. Authorized/ambiguous/denied/interrupted
  caller sentinels, unrelated-project state and incidental redaction passed.
  Enabled, disabled and hook-only exports were inspected as actual PCK/bundle
  contents and executed: no active tooling/native registration, listener,
  credentials or missing native dependency survived.

Owned-window visual review included the final reopened clean `subject.gd`
showing `return 319`, the post-runtime clean target showing `return 23`, and
preserved `other.gd(*)` human work. The intentionally invalid unrelated fixture
buffer and its parse indication are synthetic preservation evidence, not a target
parse failure. Images accompany independent state/history witnesses.

### Exact candidate and artifacts

Official `4.7.2.stable.official.ed1daf0bf`, engine commit
`ed1daf0bf001b61586d9930840f2f1394092c079`, executable SHA-256
`c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf`;
macOS **26.6.2**, build **25G83**, **arm64**; Rust **1.98.1**,
Python **3.10.9**, Apple clang **21.0.0** (`clang-2100.3.34.2`), SDK **27.0**.
Export-template SHA-256:
`88df5e2e6fee99088699be66e6d42e4da4fb0c5619d054297d755a49558a4792`.

Evidence root:
`/var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-agent-kit-T005-nd7ltchy/`.

| Summary | Records | SHA-256 |
| --- | ---: | --- |
| `edit-all/summary.json` | 405 | `125817ffa50e6c078ab83e73e592d77b8d417db801c80b04fbb2c49476d93050` |
| `observation-all/summary.json` | 204 | `129b38f965b049682711429e36ae9402b4d86f5a52ef32e7715ff479211be80a` |

Production native build ID:
`c36dd094486bbf26dfac0e6025c3d8cb12785e85e4940eef20ee31f73af74b1f`;
library SHA-256:
`656b53708ddb24bfc6678f7bcf06c821c35175fe572d5c6dd124caf60ac117b3`.
Fixture-only build ID:
`73ec38cb4c5dc56bd9df73da5069b15e42683b8679d9d5dee8725f8a7d792262`;
library SHA-256:
`b70f944566e050a815f58887524c7ff4704bb3e24307c8d514016aadc9353526`.
The complete summary records each build separately, including generated ABI/API,
manifest and acceptance-source hashes.

Exercised caller SHA-256:
`ea3564faf640997f2bd0c278c67ff383f71cee7053bb3efb4b363f1cafbc342b`;
observer:
`9a5df2c93d470d86b5e30a9c82f24e57e531bf0fe6bf2b6db734ee4fc181437a`;
stock-validator test consumer:
`815a3593be1cd48926312e8096c362ef6bca4d44cd52845b7dd31f1a232995ed`.

The §2 Rust commands passed formatting, Clippy, **193 tests** including doctests,
rustdoc and all required binaries/example builds. Both native artifacts built.
Five native-build tests, **22 workflow tests**, and Actionlint passed. Existing
ordinary `ci.yml` already covers these paths; the optional GUI workflow now builds
both native artifacts and runs both complete suites. This records local execution,
not an assertion that the optional hosted/self-hosted workflow ran.

### Development failures, limits and phase assessment

Earlier focused runs remain failed evidence, not relabeled acceptance: the initial
Save assertion incorrectly required inode retention across ordinary Godot Save;
the reparse fixture called a nonexistent `GDScript.is_valid`; a dirty-human setup
incorrectly waited for R/B convergence; and privacy comparisons spanned second-editor
startup/teardown instead of the individual request. These fixture issues were
corrected before the complete run. One early prior-history Save action failed
without a retained response; its cause was not established. Failed-action response
retention was then added, and the final complete run passed the same prior-history
path. Passing focused runs are supplemental, not counted again in the 405 records.

No production mutation semantics, API, dependency or threat model changed.
The [implementation-shape/constitutional review](plan.md#t005-implementation-shape-and-constitutional-review)
records responsibility and complexity decisions. Owned processes/projects were
cleaned up by the runners; explicit private evidence is retained.

Support is confined to the exact candidate and admitted native source/Save profiles:
one already-open standalone GDScript, LF UTF-8 within declared bounds, no tool
script/script inheritance/load-preload/global-class/exported-declaration effects.
Unsupported context or representation refuses; no neighboring-version/platform,
MCP, arbitrary same-inode exclusion, crash atomicity, OS sandbox or live-game
hot-reload guarantee follows. The inherited stock LSP endpoint limitation in
§19.9 remains non-blocking.

Roadmap Phase 1's A–E and applicable durability evidence is now satisfied for this
approved open-script slice. Its broader capability areas still include script
discovery/open and independent lifecycle controls not delivered by this feature;
Phase 1 remains in progress rather than equating feature completion with all
roadmap capability coverage. No next task or feature is selected.
