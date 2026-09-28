# Quickstart: Verify Guarded Open-GDScript Editing

**Status:** T001's protocol-independent core and T002's read-only patched-editor native validation primitive are implemented and verified; see [§9](#9-t001-core-acceptance-2026-09-27) and [§10](#10-t002-native-validation-acceptance-2026-09-28). [§18 stock GUI research](research.md#18-preserved-mtime-behavioral-finalization-research-2026-09-28) selects conceptual T1 handler-free same-retained-fd T0 mtime restoration/readback followed by guarded public edited=false and CodeEdit saved-version tagging on the demonstrated macOS arm64 build. No saved-handler discovery, admission or callback-generation tracking is needed for this route. A qualifying explicit exact-source `valid|invalid|unavailable` validator with required effect safety remains separately unresolved. Feature 002 remains in progress: T003 is unstarted, T004/T005 are pending, and the writer/finalizer, edit caller, full-feature runner and mutation A–E/durability acceptance are not implemented or passed. The native build and `run_script_edit.py --scenario native-validation` remain available for T002's reference only.

**Historical recipe boundary:** The patched native API family revision 1, private Resource/document numeric mtime synchronization, older no-callback/tag-last instructions, P1 saved-handler route and mechanism-specific prerequisites, native A cases and caller/bridge recipes below are **historical**, not a production path or final stock wire/API shape. Use the current [stock native §3 contract](contracts/native-integration.md#3-primitive-a-guarded-target-document-saved-transition) and §18's T1 decision for current semantic design, not the old patch recipe or P1 handler. The executed T001/T002 evidence in §9/§10 is unchanged; all independent safety requirements, behavioral scenarios and A–E/durability gates below remain required.

Use the [spec](spec.md), [plan](plan.md), [data model](data-model.md) and [caller](contracts/edit-api.md), [bridge](contracts/bridge-protocol.md), [native](contracts/native-integration.md) contracts as normative semantics. Do not infer success from process exit, a save acknowledgment or the finalizer's copied fields.

## 1. Prerequisites and exact candidate

- Reviewed spec/plan and the relevant approved one-task/one-PR implementation increment from [tasks.md](tasks.md). Task derivation, granularity review and read-only analysis are complete; implementation remains separately authorized work. This guide does not authorize implement-all.
- Owned GUI macOS **26.6.2 arm64** fixture environment, with visible Script Editor/CodeEdit and permission to capture only its owned window. No real developer project content.
- Patched development editor based on Godot **4.7.2**, exact commit `ed1daf0bf001b61586d9930840f2f1394092c079`, implementing native API family revision **1**, plus matching C++17 GDExtension. Record actual custom version, patch/native build IDs and artifacts; the official stock binary does not contain the new APIs.
- Existing Rust **1.98.1** with rustfmt/clippy and tracked lockfile; Apple command-line C++ toolchain/SDK; Python **3.10+**. Record actual native build-tool versions and provenance. Build engine APIs with the pinned engine's normal build system, not a new distribution service.
- Compatible exact-version export templates; editor-only API additions must not require shipped tooling or a patched gameplay authority. Verify actual exported execution, not only a preset's text.
- Empty absolute mode-0700 artifact directories and a draining stdout consumer. Harness starts/stops only its owned editor/runtime children and records bounded waits and cleanup.

The implemented native build entrypoint is:

```sh
python3 godot-addon/native/build.py --godot "$EDIT_GODOT"
```

It uses the matched public GDExtension C ABI, compiles the owned native source, installs the editor-only artifact and records build/ABI/provenance information. The [native build guide](../../godot-addon/native/README.md) supplies exact-base patch application and engine build instructions. It does not download/build an arbitrary engine automatically or create a release/update service. T002 implements only validation, not the complete editing API family.

Confirm actual candidate/tool identities during implementation:

```sh
"$EDIT_GODOT" --version
rustc +1.98.1 --version
python3 --version
```

The harness independently checks native capabilities/build identity and exercised behavior; a version string alone is insufficient. Stock/unmatched/missing-native cases must retain supported observation behavior without granting edit capability. The original planning continuation authorized no runtime experiment; §10 records the separately authorized T002 implementation evidence.

## 2. Rust and boundary checks

From `mcp-server/`, run the repository baseline for the implemented package:

```sh
cargo +1.98.1 fmt --all -- --check
cargo +1.98.1 clippy --all-targets --locked -- -D warnings
cargo +1.98.1 test --locked
cargo +1.98.1 doc --no-deps --locked
cargo +1.98.1 build --locked --lib --bin observe-gdscript --bin edit-gdscript
```

`cargo test --locked` includes doctests. No root workspace or blanket `--all-features` is introduced. Tests must cover consumer-visible invariants and transitions: prior-observation basis eligibility, dirty-equal/stale/same-text-version distinction, independent postconditions, stage/application precedence, pre/post-authorization worker loss, immutable late results, strict v2 authentication/limits/identity, denied-source suppression and actual confined I/O outcomes. Do not add source-text/wiring-copy/mock-echo tests.

The native implementation also needs isolated real-engine regressions for A/B, using actual descriptors, Script/document/CodeEdit objects and independent saved-state reads. An engine integration fault hook may control a test boundary but cannot replace the real writer/validator/finalizer with a successful stub or become a public product control. Distinguish actual I/O failure from an injected status; retain honest evidence for each.

## 3. Owned real-editor runner

Reuse existing observation harness setup, authentication, owned-window capture, independent D/R/B/dirty/history witnesses, process cleanup and artifact handling. T002's implemented validation-only invocation is in §10. The complete caller runner below belongs to T004/T005 and is **not available at T002**:

```sh
python3 godot-addon/tests/run_script_edit.py \
  --godot "$EDIT_GODOT" --observer "$OBSERVER" --mutator "$MUTATOR" \
  --scenario all --artifacts "$EDIT_ARTIFACTS"
```

In that planned full-feature runner, `OBSERVER` and `MUTATOR` identify the newly built callers; the native artifact is installed by the native build into the addon copied to the fixture. It will accept the named groups below and `all`, create its own registry/project, prepare the requested clean/dirty/history states, and drive actual caller requests. Product edit/observation code cannot call fixture helpers to manufacture eligibility or proof.

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

Prepare a non-selected target, another dirty document and known prior native history. Record independent before/after disk content/identity/mtime, Script source/edited/mtime, actual CodeEdit text/current/saved versions, document mtime baseline, dirty attribution, current selection and history reachability.

Exercise:

- Successful native bound write + A: exact target is clean/synchronized, no unrelated save/history/selection change, no broad saved notification/runtime reload.
- Missing/failed/unknown/mismatched receipt; receipt for another attempt/file/document; source or identity mismatch; same-text newer buffer version; closed/replaced document; Resource replacement. No false clean tag.
- New human B after source application or persistence: preserve that B and actual dirty/history state, expose applied/partial result and never reassert the old request.
- File/parent/leaf replacement and rename/unlink around guarded descriptor I/O: never write replacement/outside sentinels; report loss of namespace/current-target coherence even if the retained original object received bytes.
- Changed D known before persistence; short write/truncate/flush failure; loss after real write; each A bookkeeping-step failure. Result flags and independent state identify actual changes, including partial metadata.
- The observed regression **apply → Undo → ordinary script Save → Redo → Save**: document mtime and saved-version handling must eliminate reconciliation caused by stale bookkeeping and retain native history.
- Save formatting profiles: refuse before mutation if baseline or desired source would be changed by native trailing-whitespace/final-newline/indent conversion. Do not change preferences to make product results pass. Compatible representations must retain exact bytes and history through ordinary Save.

The same-inode critical-window counterexample remains an explicit unpromised atomicity limit. Do not require a new lock/CAS system or claim all transient writes can be detected. Where invalidation is actually known to the system, it must prevent success even if later bytes match. A fixture controller's additional knowledge cannot be silently presented as a product observation.

### Same-session overlap (US2.4; T004)

This is T004's real routing/conflict/interruption acceptance, not a new story, task or concurrency service. Use two actual caller requests, the same selected editor/session/script, fixture-owned barriers and independently read D/R/B, current/saved versions, dirty state and native history. Boundary/finalization witnesses must identify the actual attempt; do not infer non-entry only from equal text or a response echo.

1. Observe clean revision X. Start A and hold it after supervisor authorization while its prepared attempt owns the existing slot, before mutation, with an explicit barrier that lets the integration handle B.
2. Submit B with the same basis X. Verify `refused`/`busy`, `application: not_applied`, no B boundary entry and zero B-caused source/history/finalization change while A remains held. Establish that B's attempt is terminal with no retained preparation/apply work, not merely waiting.
3. Release A and require its normal verified edit to Y. Release controlled deliveries/work and verify no late B application or finalization and no retained B work after slot release; source/history changes are attributable only to A.
4. Submit a new request with the old B basis X, including desired text equal to Y: it must remain stale, not `verified_unchanged` or an automatic rebase. Then observe Y afresh and prove a new valid request can edit to Z.

In the existing interruption cases, hold an already-entered A native stage through caller timeout/disconnection. A retains truthful applied/unknown knowledge and its bounded caller result; no other attempt may enter mutation while that stage can still mutate. Release the barrier, observe survivor state and terminal cleanup before a later fresh-basis edit. This is not rollback or permission to block human typing. Existing before/after-application human-edit cases remain mandatory.

### B: real source parser/analyzer and effect boundary

Use exact-source root and project-relative source-backed GDScript dependencies, including non-ASCII text, inheritance and changed dependency context. Include a nested dependency in another directory that itself uses a relative inheritance/preload path; resolve it against that referring script, not the root target. Compare native validity/diagnostics against the actual supplied input and engine context, not another parser. Verify root/dependency paths, one invocation's timing/hash and explicit unavailable cases; a preflight result cannot certify post-change source.

Use controlled safe fixture sentinels to establish that excluded loaders, dynamic project getters, tool/static initialization and constructors are refused **before** executing, and that ordinary allowed parser/cache population is not itself rejected. Check real cache-hit/source-generation attribution, not just the current text of a stale cached object. Validation must not assign root R/B, write source, save, alter unrelated dirty work, or leak diagnostics into logs.

Exercise wrong thread/context, nested/reentrant validation, changed target/source/dependency/global-class mapping, limits and incomplete result. Guard forbidden effects of ordinary edit-generated validation/export continuations, including queued effects after an explicit validator's return/guard close, as well as the explicit validator itself; no P1 saved-handler function-discovery tail, callback-generation admission or deferred-debugger routing is a production prerequisite. Subsequent independent human edits and unrelated ordinary editor behavior must still work normally.

T002's non-mutating B proof is recorded in §10. Establishing a qualifying stock exact-source `valid|invalid|unavailable` validator and its generic effect safety remains separate from T1 saved-state finalization, and requires its own real-editor evidence before implementation acceptance. No historical incomplete probe is converted into acceptance by source reasoning or this implementation.

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

Export checks must cover addon enabled, disabled and existing hook-only isolation where applicable, with the **installed native artifact present**. Inspect actual PCK contents and any bundled libraries/resources, then run the exported fixture. Neither `.gdextension`, native binary/tool registration, fixture bridge nor credential/listener behavior may survive. Runtime must not depend on the editor API patch. Use existing CI/export trust boundaries; no new provider-specific workflow is an acceptance prerequisite.

## 7. Evidence and completion report

Record for each case: exact spec/gate mapping; request and target identity; expected basis and immutable intent digest; controlled injection/action interval; before/after independently observed D/R/B availability/hash/identity, dirty/saved versions and document/Resource metadata; actual native history actions; native source-attributed diagnostics; stage/application result; elapsed time; safe next action; and artifact references. Missing data is explicit, not synthesized from the request. Keep synthetic source-bearing requested evidence distinct from redacted routine logs/private session metadata.

Record exact OS/architecture, Rust and native toolchain, engine base/patch/version/hash, native API/build and binary identities, addon/core commit, export-template provenance, suite commands/results, reviewed screenshots, failure diagnostics and owned-process cleanup. Update verified guarantees and limitations only from passing applicable evidence; no nearby version/platform claims.

Completion requires every positive/refusal/partial case, all A–E, applicable durability including runtime, observation preservation and privacy/export gates. Unsupported/refused output for all edits is not completion. Mark only the selected approved implementation task complete when its own acceptance is satisfied, independently of PR review/merge metadata. Feature/Phase 1 completion remains a separate acceptance assessment. This planning PR marks no implementation task or mutation gate complete.

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

**T002 is complete.** Primitive B runs the real Godot parser/analyzer through
generated public GDExtension C ABI and ClassDB dispatch. This is read-only native
integration evidence, not an edit caller, Primitive A, or mutation acceptance.
T003–T005 remain pending.

### Executed build and runtime evidence

The maintainer-operated candidate was macOS **26.6.2 arm64**, Godot
`4.7.2.stable.custom_build.ed1daf0bf`, Rust **1.98.1**, Apple clang
`21.0.0 (clang-2100.3.34.2)`, SDK **27.0**, and SCons **4.10.1**.
The SDK version is build provenance, not a claim to have tested another host.
The engine build used `target=editor arch=arm64 dev_build=no debug_symbols=no
optimize=none tests=yes vulkan=no metal=no angle=no accesskit=no`.
The native build used C++17, `-O2`, hidden visibility and
`-Wall -Wextra -Werror`. Exact reproduction is in the
[native build guide](../../godot-addon/native/README.md).

| Identity | Recorded value |
|---|---|
| Godot source base | `ed1daf0bf001b61586d9930840f2f1394092c079` |
| Engine patch SHA-256 | `1b1508c4c79dcb77af86685aef828e97b0d5857be6e4285ff2b7b72ecfe75f2a` |
| Patched editor SHA-256 | `45a64b260c8347b4496bf7d0caabcbf2ff2e49530d051bf0434e8866990ba338` |
| Native build ID | `5c76fe0f323be3189970f16351d88adcd27cc5cd4beba477b1bc84094b34c362` |
| Native library SHA-256 | `32e4766e5b47948bf5b0dd5ee79eae10288566b757655a0f1b7c743a1fa521b0` |
| Native build-manifest SHA-256 | `7f02424b31f522642d1a1e856949b871923070e2f66a7a7bd421e8777e95aa5e` |
| Generated C header SHA-256 | `640b48188708ba0016f8d7ace9e0e1d3279a41fa1226c59ff3193b15538bd254` |
| Generated ABI / ClassDB API SHA-256 | `7d8c0a039d9743eb8ebf88681ae0c641d8d3aa5ffca11081745a84da803e09a1` / `ee94edcf1f0f485080a92d8d2e01b4e646103de3df625377f9ec6755a6fbaa87` |
| Observation caller SHA-256 | `ccb9366f5a71ff6fc211e0900df5885c0b13fa19da97a7665058838d492c0d80` |
| Stock Godot / macOS export-template SHA-256 | `c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf` / `88df5e2e6fee99088699be66e6d42e4da4fb0c5619d054297d755a49558a4792` |

Executed from the repository root, with absolute paths and separate empty private
artifact directories:

```sh
python3 godot-addon/native/build.py --godot "$EDIT_GODOT"
python3 godot-addon/tests/run_script_edit.py \
  --godot "$EDIT_GODOT" --observer "$OBSERVER" \
  --scenario native-validation --artifacts "$NATIVE_ARTIFACTS"
python3 godot-addon/tests/run_observation.py \
  --godot "$STOCK_GODOT" --observer "$OBSERVER" \
  --scenario all --artifacts "$OBSERVATION_ARTIFACTS"
```

- **54 native cases passed:** bootstrap, 50 validation/lifecycle cases and three
  actual production exports (enabled, disabled, hook-only). Native state checks
  independently compared D bytes/identity/metadata, R/B text, associations, dirty
  and current/saved versions, caret/selection and prior Undo/Redo. Owned GUI
  before/after captures were byte-identical, SHA-256
  `8dd739e8906d5238d47d9e07839360a8f216bd0b82c6d2ec3843b7c09dc000db`.
- **194 existing observation cases passed**, all 13 groups, on
  `4.7.2.stable.official.ed1daf0bf` with the final native artifact installed.
  This includes the existing twenty-read/history, routing, privacy and export
  coverage; the unsupported native artifact did not widen the stock bridge.
- A separate owned **missing-native smoke passed** after removing only the
  copied fixture's library/build manifest. The editor had no native API metadata;
  the actual caller returned complete observation with independently matched
  D/R/B and an owned-window capture.
- Rust formatting, Clippy with `-D warnings`, `cargo test --locked`
  (**155 tests**, with the doctest target also executed), and rustdoc passed.
  Three ABI refusal regressions, 20 workflow boundary/execution regressions,
  Python compilation, Actionlint and the validation-only CLI help smoke passed.
  The local patched-engine/native build passed. Hosted `ci.yml` now runs that
  build boundary; workflow validation is not a claim of a hosted GUI run.

Final local artifacts are retained under
`/var/folders/2r/m9lt6gb17wgbzcw9zzf6mp_80000gn/T/godot-agent-kit-native-ex09m3nz/`:

| Artifact | Summary SHA-256 |
|---|---|
| `native-acceptance/summary.json` | `f397774e96779c99fdb2d8fa783250182cc789e06773d72a1c3a5c5b65d23a49` |
| `stock-observation-final/summary.json` | `4959c6a0a0fca5d8ffc39fc68097021ca18d6f0f275bb5c9e30639cd961f01dd` |
| `missing-native-evidence/summary.json` | `48535409902874b706162f0dda938dbf219e8a30cfa8fc5a569b343198d7bceb` |

These are local acceptance artifacts, not portable download links or committed
build/editor state. Summaries retain per-case witnesses, artifact hashes and
runtime/export results. Historical failed attempts remain failed evidence.

### Coverage, discovered failures and fixes

The native cases cover valid Unicode/empty source; root parser/analyzer and
dependency errors; missing versus denied/unreadable/symlink/ACL dependencies;
nested relative/transitive resolution; fresh attribution despite stale caches;
actual mid-call dependency invalidation; changed autoload context; native
reentrancy; wrong path/hash/document/session/thread; and close/disable/re-enable.
Exact and over-limit cases cover root/dependency source bytes, dependency count,
4 MiB aggregate bytes, 64 diagnostics, 2048-byte dependency paths and 2048-byte
diagnostic messages. Unsupported binary/remapped inputs and excluded loaders,
tool/static initialization, constructors, getters and diagnostic stringification
have independent refusal/effect evidence.

Review findings were reproduced and fixed before acceptance:

- Duplicate-key diagnostic formatting could call a scripted `_to_string()` after
  effect refusal. The analyzer now guards the value before formatting; the actual
  effect sentinel remains untouched.
- A failed autoload dependency could yield `valid`. Dependency-analysis failures
  now propagate with dependency-attributed diagnostics.
- Cyclic call-local parser references leaked. Cleanup breaks dependency edges
  while retaining the owning map; 32 repeated cyclic validations preserve the
  observed object count.
- The native reader needed a final size/identity check, and generated double or
  missing precision could select undersized ABI storage. Both are checked;
  double/unknown/32-bit ABI regressions fail closed.
- Skipping the native manifest during export still left its path in Godot's
  generated extension registry. `.gdignore` plus explicit editor-plugin loading
  removed automatic discovery; all three real exported applications run without
  tooling, a missing native dependency, or a listener.

Earlier locked-desktop capture failures were not counted as visual proof.
The unlocked final run used a process-scoped display-idle assertion and actual
owned-window capture. Fixture loader sentinels are armed only during the native
call, and ordinary editor R convergence is observed before measuring B's
non-interference; startup/import activity is not mislabeled as a validation effect.

### Implementation shape and constitutional review

`extension.cpp` owns C-ABI registration, exact-build gating and callable lifetime;
`native.hpp` owns the shared opaque-value ABI wrappers and session state;
`validation.cpp` owns the project-bound read-only reader/invocation pipeline.
Its helpers remain private, its idle/running/closing state has one enum, retained
dependency witnesses do not copy full source buffers, and document IDs are
decoded once rather than allocated repeatedly during editor enumeration.
No writer/finalizer declarations or speculative public APIs were added.

The engine's scoped read/parser/effect context and ClassDB result marshaling are
separate small units. Existing large analyzer/cache files retain their original
responsibilities; the patch adds checks at their real resolution/evaluation
sites rather than another parser or a generic execution framework. The existing
plugin owns enable/disable and export lifecycle. Responsibility-based naming is
retained; task IDs occur in acceptance fixtures/docs, not product APIs.

Principles I–IV/VI/X/XII: B reports actual source/context-attributed validation,
preserves independently observed source/dirty/history state and refuses missing,
invalidated or unsupported evidence. This does not verify any mutation.
V/VII/IX/XI: confinement/effect limits stay at the Godot boundary, the bridge v1
and Rust policy remain unchanged, and the implementation uses the public C ABI
without private binary layouts, private-symbol linkage or another parser.
VIII: actual enabled/disabled/hook-only exports prove tooling exclusion.
XIII: the existing [complexity record](plan.md#complexity-tracking) justifies the
native build, scoped context, test-only effect witness and explicit loading
against demonstrated failures. No new service, approval gate, runtime dependency,
provider-specific GUI prerequisite or duplicate transaction policy was introduced.

**Limits:** evidence applies only to the recorded macOS arm64, tests-enabled
patched editor/native build and the separately tested stock observation build.
No other platform/version, `tests=no` native build, patched-editor bridge support,
project-wide compilation, global execution sandbox or atomic same-inode snapshot
is claimed. The full editing family, edit-generated callback guard, bound writes,
finalization, caller/bridge cutover and mutation A–E/Save/reopen/runtime durability
remain pending. Preserving existing Undo/Redo is not proof of agent-edit Undo/Redo.
