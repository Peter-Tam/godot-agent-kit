# Quickstart: Verify Guarded Open-GDScript Editing

**Status:** Future implementation/acceptance guide. Feature 002's design and [task derivation](tasks.md) are complete; granularity review and read-only analysis passed, including the local-overlap clarification. Implementation has not started. No mutation implementation, engine patch or A–E result is supplied by this design/task PR. Commands involving `edit-gdscript`, the native build entrypoint or `run_script_edit.py` are the planned interfaces implementation must provide, **not commands executed or currently available here**. Existing observation evidence does not establish mutation support.

Use the [spec](spec.md), [plan](plan.md), [data model](data-model.md) and [caller](contracts/edit-api.md), [bridge](contracts/bridge-protocol.md), [native](contracts/native-integration.md) contracts as normative semantics. Do not infer success from process exit, a save acknowledgment or the finalizer's copied fields.

## 1. Prerequisites and exact candidate

- Reviewed spec/plan and the relevant approved one-task/one-PR implementation increment from [tasks.md](tasks.md). Task derivation, granularity review and read-only analysis are complete; implementation remains separately authorized work. This guide does not authorize implement-all.
- Owned GUI macOS **26.6.2 arm64** fixture environment, with visible Script Editor/CodeEdit and permission to capture only its owned window. No real developer project content.
- Patched development editor based on Godot **4.7.2**, exact commit `ed1daf0bf001b61586d9930840f2f1394092c079`, implementing native API family revision **1**, plus matching C++17 GDExtension. Record actual custom version, patch/native build IDs and artifacts; the official stock binary does not contain the new APIs.
- Existing Rust **1.98.1** with rustfmt/clippy and tracked lockfile; Apple command-line C++ toolchain/SDK; Python **3.10+**. Record actual native build-tool versions and provenance. Build engine APIs with the pinned engine's normal build system, not a new distribution service.
- Compatible exact-version export templates; editor-only API additions must not require shipped tooling or a patched gameplay authority. Verify actual exported execution, not only a preset's text.
- Empty absolute mode-0700 artifact directories and a draining stdout consumer. Harness starts/stops only its owned editor/runtime children and records bounded waits and cleanup.

The implementation-owned native build entrypoint is planned as:

```sh
python3 godot-addon/native/build.py --godot "$EDIT_GODOT"
```

It must use the matched public GDExtension C ABI, compile only the owned native source, install the editor-only artifact under the addon, and record build/ABI/provenance information. It must not download/build an arbitrary engine automatically, substitute an unmatched binary, or create a release/update service. Implementation must provide the exact-base engine patch and reproducible application/build instructions alongside that native source. No patch checksum is invented before the patch exists.

Confirm actual candidate/tool identities during implementation:

```sh
"$EDIT_GODOT" --version
rustc +1.98.1 --version
python3 --version
```

The harness must independently check advertised native capabilities/build identity and exercised behavior; a version string alone is insufficient. Stock/unmatched/missing-native cases must refuse mutation source-free while retaining supported observation behavior. No runtime probes, policy-blocked experiments or filter retries are authorized by this planning continuation.

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

Reuse existing observation harness setup, authentication, owned-window capture, independent D/R/B/dirty/history witnesses, process cleanup and artifact handling. Add the focused planned runner with this invocation:

```sh
python3 godot-addon/tests/run_script_edit.py \
  --godot "$EDIT_GODOT" --observer "$OBSERVER" --mutator "$MUTATOR" \
  --scenario all --artifacts "$EDIT_ARTIFACTS"
```

`OBSERVER` and `MUTATOR` identify the newly built callers; the native artifact is installed by the native build into the addon copied to the fixture. The runner accepts the named groups below and `all`. It creates its own registry/project, enables only owned fixture/product plugins, opens the target explicitly during setup, prepares the requested clean/dirty/history states, and drives actual caller requests. Product edit/observation code cannot call the fixture helpers to manufacture eligibility or proof.

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

Exercise wrong thread/context, nested/reentrant validation, changed target/source/dependency/global-class mapping, limits and incomplete result. Confirm automatic validation/function discovery and queued application/export callbacks caused by the edit cannot bypass the controlled path after its explicit return/guard close. Subsequent independent human edits and unrelated ordinary editor behavior must still work normally.

Runtime proof of these new hooks remains required during implementation. A currently disallowed experiment is not retried, rephrased or claimed to pass in this planning continuation; source reasoning does not replace future behavior evidence.

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
