# Quickstart: Validate Live GDScript Observation

**Status:** T001–T004 implement the reusable core, authenticated session boundary, bounded caller/worker, confined D and passive R/B/dirty collector. The driver implements `clean-open`, `session-boundary`, `executor-boundary`, and `export-boundary`; remaining story and cumulative groups belong to T005–T008. Evidence is recorded in §2.1–§2.4. No full-feature, mutation or supported-version claim is made.

This guide covers the entire observation-only specification. It does not implement the feature, derive tasks, or claim a mutation/UndoRedo/Phase 1 exit guarantee. Use the [data model](data-model.md), [caller contract](contracts/observation-api.md), and [bridge contract](contracts/bridge-protocol.md) for normative fields and outcomes instead of inferring semantics from exit status alone.

## 1. Prerequisites

T001's native library requires only Rust 1.98.1 with rustfmt/clippy and its tracked lockfile. The following Godot, GUI, Python, and export prerequisites apply to later live-editor tasks, not to the pure classification engine.

- Reviewed specification/plan and completed relevant one-task-per-PR implementation dependencies. Do not invoke generic implement-all.
- macOS arm64, exact Godot `4.7.2.stable.official.ed1daf0bf` with engine hash `ed1daf0bf001b61586d9930840f2f1394092c079`. Planning observed macOS 26.6.2; do not extrapolate that evidence to another platform/version.
- A GUI session with permission to capture the **owned fixture window**; real Script Editor/CodeEdit visibility is required. Headless runtime does not validate B.
- Rust **1.98.1** with rustfmt/clippy, Apple Command Line Tools for ring's C/assembly build, the implemented `mcp-server/Cargo.lock`, Python 3.10+ for the fixture driver, and exact-version Godot export templates for export checks.
- A temporary owner-private work/artifact directory, consuming stdout reader, and no real user project content. The harness starts/stops only its owned editor processes. Source mutation/open/select/save actions occur only in explicit fixture preparation, never in the observation code under test.

Confirm tools before running acceptance:

```sh
godot --version
rustup toolchain install 1.98.1 --profile minimal --component rustfmt --component clippy
rustc +1.98.1 --version
python3 --version
```

Changing the globally selected Rust toolchain is unnecessary. Do not silently accept a nearby Godot patch version. Verify official binary/template provenance and record executable checksums in the evidence report.

## 2. Native checks and caller build

From `mcp-server/`, the native library and boundary checks are:

```sh
cargo +1.98.1 fmt --all -- --check
cargo +1.98.1 clippy --all-targets --locked -- -D warnings
cargo +1.98.1 test --locked
cargo +1.98.1 doc --no-deps --locked
```

`cargo test --locked` includes the doctest phase. There is one package, not a workspace; no blanket `--all-features` is specified. T001 checks evidence semantics; T002 adds actual framing, authentication, routing, and registry/confinement regressions. The [T002 dependency review](research.md#t002-resolved-authentication-dependencies-and-tool-provenance-2026-09-26) records actual licenses, provenance, audit-tool/database revisions, and findings.

Build the caller and the library consumed by the live driver with `cargo +1.98.1 build --locked --lib --bin observe-gdscript`.

### 2.1. T001 native evidence (2026-09-26)

Local host: macOS **26.6.2**, arm64; `rustc 1.98.1 (48a229cea 2026-09-01)`, `cargo 1.98.1 (797e8a9bc 2026-08-05)`. All four commands above passed. The integration suite contains **27 passing tests**, including all 11 caller-contract vectors expanded into full evidence records; the unit and doctest phases currently contain zero cases. `cargo +1.98.1 build --locked --lib` also passed.

A separate temporary Rust binary consumed the built public `godot_agent_kit::observation` API. It was compiled with `rustc +1.98.1 --edition=2021`, `--extern godot_agent_kit=target/debug/libgodot_agent_kit.rlib`, and `-L dependency=target/debug/deps`, then executed. It used explicitly **synthetic** caller/editor-clock stamps and `/synthetic/project` identity, not actual project or Godot state. Its classify/finalize assertions observed:

| Supplied evidence / transition | Observed library result |
|---|---|
| Independently supplied D=R, differing B, dirty | `CompleteObservation`, divergent, dirty preserved |
| D exceeds 512 KiB by one byte | `LimitedObservation`; only D unavailable/`TooLarge` |
| B changes during collection | `LimitedObservation`; original B in invalidated evidence |
| Confirmed closed; D changes | `NotOpen`; D invalidated, closed state retained |
| Known session loss plus timeout | `DisconnectedEditor`; D retained, R/B invalidated |
| Protocol failure after valid partial collection | `ProtocolError`; earlier validated D retained |
| Cancellation plus known disconnection | `Cancelled`; known loss still invalidates R/B |
| Denial after collection/invalidation | `DeniedAccess`; no snapshot or source |
| Reuse of prior-request evidence in the same session | Rejected with `WrongTarget` |

The consumer also accepted backward-adjusted wall-clock endpoints while validating collection against monotonic elapsed time. Temporary consumer source/binary were removed after execution. Public construction examples remain in [the contract tests](../../mcp-server/tests/observation_contract.rs), and the [API notes](contracts/observation-api.md#7-implemented-t001-rust-library) describe adapter obligations.

Integration review reproduced and fixed request-reuse, partial-evidence loss, hidden session loss, dirty-reason bypass, wall-clock adjustment, invalid-target representation, closed-state invalidation, ungrounded target-refusal, and earlier-invalidation-reporting failures. Permanent regressions cover those behaviors.

Constitutional review: independent authorities, dirty-state uncertainty, source-free denial, and interval-only claims preserve I/II/IV/X; the standard-library-only domain module preserves VII. No mutation, filesystem/network access, editor integration, gameplay authority, or UndoRedo capability exists in T001, so native-mutation, live-editor, export, and A–E mutation gates are inapplicable to this task. Dependency provenance/license/advisory review addresses XI. These results establish library semantics only—not five-second execution, live D/R/B observability, editor non-interference, or Godot/platform support. The hosted workflow targets macOS 15 arm64; its actual run status belongs in the task PR and is not inferred from this local run.

### 2.2. T002 source-free boundary evidence (2026-09-26)

At T002 delivery, its executable provided only `init-registry --registry
ABSOLUTE_PATH`, `--help`, and `--version`; T003's caller is documented in §2.3.
Install `godot-addon/addons/godot_agent_kit/` in a disposable project's `addons/`
and deliberately enable it. Use a canonical, owner-private directory outside the
project for `GODOT_AGENT_KIT_REGISTRY`; an absent or unsafe configuration opens no
bridge. On macOS the registry and descriptors require 0700/0600, trusted ancestry,
and no access-grant ACLs. Unsafe state is refused, not repaired.

Run the implemented groups from the repository root, with a separate empty 0700
artifact directory for each:

```sh
python3 godot-addon/tests/run_observation.py \
  --godot "$GODOT" --observer "$OBSERVER" \
  --scenario session-boundary --artifacts "$SESSION_ARTIFACTS"
python3 godot-addon/tests/run_observation.py \
  --godot "$GODOT" --observer "$OBSERVER" \
  --scenario export-boundary --artifacts "$EXPORT_ARTIFACTS"
```

All executable/artifact paths must be absolute. The driver compiles a temporary
Rust library consumer for source-free selection, plus an owned-window identifier
for scoped macOS screenshots. It creates home-private disposable projects,
registries and control directories, and cleans only its owned processes/state.
The test driver is not a second product CLI or a public editor-control interface.

Observed on macOS **26.6.2 arm64**, exact Godot
`4.7.2.stable.official.ed1daf0bf`, full hash
`ed1daf0bf001b61586d9930840f2f1394092c079`:

| Boundary | Executed evidence |
|---|---|
| Native | fmt, Clippy with `-D warnings`, tests including doctest phase, docs, and real caller/library build all passed. **53 tests**: 27 original semantic regressions, 1 three-role proof-vector test, 10 bridge tests, 15 confinement tests. |
| Real editor sessions | **33 cases passed**, including bootstrap; absent configuration; in-project/ACL-granted registry refusal; zero/one/two candidates; exact and absent selectors; an owned suspended candidate; disable/re-enable/restart; ended-session refusal; and rebound-port impersonation. |
| Authentication | Rust and Godot matched all three public synthetic vectors. Python independently verified live server/client/finish proofs and unchanged transcripts. Wrong-secret, replayed, reflected, changed-request, malformed/premature messages, boolean/fractional versions, nonscalar identities, oversized controls, and the 32-peer/unauthenticated-expiry boundary were exercised. Challenge receipts synchronize acceptance instead of assuming TCP backlog entries are accepted peers; EOF and reset both establish refusal. |
| Source/privacy | Product selection acquired no D/R/B. The independent fixture checked unchanged source hash/mtime and open/selected-document witnesses. Owned traffic contained no raw secret; secret/nonces/proofs and the source sentinel were absent from incidental logs. The owned GUI screenshot was inspected. |
| Deadline | Every controlled routing consumer invocation completed within five seconds; the maximum was **4.506 s**, for the unresolved candidate (`Timeout` at `ResolveTarget`). Maximum observed idle lifetime among 32 accepted unfinished handshakes was **4.493 s**. This does not establish T003's blocked-filesystem worker guarantee. |
| Export | Enabled, disabled, and independently exercised hook-only variants passed ZIP and actual release-app PCK inspection, including compiled/remapped resources. All three actual apps ran the minimal one-node scene, without missing dependencies, TCP listeners or registry advertisements, with registry configuration present as a negative control. |
| Coverage gate | `--scenario all` exited 1 and listed all 12 unimplemented story/redaction groups. It did not report a partial suite as feature acceptance. |

`summary.json` records actual synthetic target/session identity, per-case outcomes
and timings, driver/lockfile/binary hashes, and export manifests/hashes. Scoped
screenshots, source-free logs, ZIPs and actual app PCKs are retained separately.
Local evidence is under `~/.t002-acceptance.P1ipPl/` (`session-final-pass`,
`export-final`, and `coverage-gate`); preparatory failed runs are distinct records,
not passing evidence. Binary/template checksums and official provenance are in
[research](research.md#t002-resolved-authentication-dependencies-and-tool-provenance-2026-09-26).

Godot retains inert `editor_plugins/enabled` paths in an enabled export's
`project.binary`; these are editor settings, not shipped addon scripts or runtime
autoloads. The file-entry/remap inspection and actual app's scene, dependency,
listener and advertisement checks establish tooling isolation without rewriting
project settings merely to remove an inert name. The disabled-preset variant
does not depend on the export hook.

Discovered regressions covered here include the macOS ACL/mode discrepancy and
Darwin's non-POSIX `acl_get_entry` return convention, routing outcome precedence,
actual engine-version/JSON-number representations, editor safe-save descriptor
publication, retaining accepted peers when an accept crosses the frame budget,
and export-hook ordering before GDScript remapping. Release stdout is explicitly
flushed in the synthetic fixture so readiness is observed while the actual app
is running, not inferred from buffered output after exit.

Constitutional review: V/X/XI/XII require and receive authenticated, owner-private,
bounded, redacted, cause-specific routing and dependency review; VII keeps the
core independent; VIII receives actual artifact and exported-launch evidence.
No source/mutation/UndoRedo capability exists, so no A–E mutation or applied-edit
durability claim is made. These are local source-free session/export results,
not full Feature 001 acceptance, GUI-CI evidence, or additional-platform support.


### 2.3. T003 executor boundary evidence (2026-09-26)

At T003 delivery, the real CLI accepted the observation invocation, while the
then-current T002 addon returned `unsupported_observation` (exit 2), an
authenticated target and null snapshot. T004 adds the actual collector;
the current executor regression prepares a real cached, closed script explicitly.

Run the dedicated executor boundary, independently of the unimplemented story groups:

```sh
python3 godot-addon/tests/run_observation.py \
  --godot "$GODOT" --observer "$OBSERVER" \
  --scenario executor-boundary --artifacts "$EXECUTOR_ARTIFACTS"
```

The group passed **11 cases** on macOS **26.6.2 arm64**, exact Godot
`4.7.2.stable.official.ed1daf0bf` / full hash
`ed1daf0bf001b61586d9930840f2f1394092c079`, with Rust/Cargo 1.98.1 and Python 3.10.9:

| Boundary | Executed evidence |
|---|---|
| Native | fmt check, Clippy `-D warnings`, tests including doctest phase, API docs, library and real caller build passed. **78 tests**: 8 library/unit, 18 bridge, 21 confinement, 31 observation/caller; zero doctest cases. |
| Actual caller/addon | Bootstrap/help/version, zero sessions, authenticated absent collector, ended-session refusal, two-session ambiguity, and exact-session selection all returned the expected correlated JSON and exit code without source. |
| Independent D library consumer | Authenticated unique selection preceded capability-confined D read and independent recheck. Exact text and inode matched the fixture's independently read disk; no recheck change was detected. This is a library boundary, not a D-only product command. |
| Editor stall | An owned suspended editor produced timeout in **4.510 s**. No disconnect was inferred from silence. |
| Worker stall/cancellation | The real caller's owned worker was separately stopped. Timeout arrived in **4.512 s**; SIGINT cancellation arrived in **0.020 s**. Both workers were reaped, and neither editor was terminated by the supervisor. |
| Non-interference/privacy | Script content hash/mtime, open paths and selected-script witnesses were unchanged. Incidental logs contained neither the synthetic source sentinel nor collected authentication secrets; source appeared only in intentional D evidence. The owned editor-window screenshot was inspected. No visible-buffer observation is claimed. |
| Supported-peer path | Native actual-process caller tests exercised authenticated controlled peers through the worker, independent D, both rechecks, reducer and full result serialization: complete-divergent exit 0 and limited/invalidated exit 2. Simulated R/B values are protocol evidence only. |

Final local artifacts: `~/.t003-acceptance-m5dvbob_/executor-final/summary.json`,
scoped `executor-boundary.png`, safe caller/editor logs, and intentional
`disk-consumer-evidence.json`. The summary records actual request/session/project
identities, binary/driver/lockfile hashes and per-case timings. The earlier
`executor/` run failed at owned-window readiness; it is not passing evidence.
The harness now waits for the native owned-window condition under a deadline
rather than assuming fixture readiness means the window is already visible.

Retained regressions include editor-clock document validity, valid invalidated
source/document/dirty evidence decoding, selected-channel request/project binding,
per-source size/encoding/identity boundaries, and editor changes retained when an
independent disk recheck is interrupted. No global source cache, automatic retry,
editor mutation, or public worker/executable override was added.

The addon/export payload and dependency lockfile are unchanged. T002 export
evidence remains historical evidence for that unchanged boundary; no new export
run, GUI-CI result, R/B/dirty support, US1 completion, or additional platform is
claimed by T003. Mutation A–E and applied-edit durability remain inapplicable.

### 2.4. T004 clean-open evidence (2026-09-26)

Run the usable US1 slice against an owned GUI editor:

```sh
python3 godot-addon/tests/run_observation.py \
  --godot "$GODOT" --observer "$OBSERVER" \
  --scenario clean-open --artifacts "$CLEAN_ARTIFACTS"
```

The artifact directory must be empty, absolute and mode 0700. The driver opens
and presents only its disposable fixture, then brackets real caller observations
with independent disk, GDScript, CodeEdit, unsaved-path, identity, caret, selection,
version/saved-version and Undo/Redo-availability witnesses. Product collection
does none of that preparation and never calls the fixture's helpers.

Observed on macOS **26.6.2 arm64**, exact Godot
`4.7.2.stable.official.ed1daf0bf`, full engine hash
`ed1daf0bf001b61586d9930840f2f1394092c079`:

| Boundary | Executed evidence |
|---|---|
| Native | fmt, Clippy `-D warnings`, **84 tests** (8 library, 24 bridge, 21 confinement, 31 observation/caller), doctest phase, docs and library/caller build passed. No doctest cases exist. |
| Clean-open | **13 cases passed**, including US1.1–3: clean and repeated observations, independently empty D/R/B, actual clean evidence and agreement. Request IDs and editor collection ticks are fresh. |
| Non-interference | Clean/empty reads preserve exact D/R/B, disk hash/metadata, open/current documents, caret/selection, buffer versions, dirty state and prepared history availability. Scoped native screenshots of the clean and empty CodeEdit surfaces were reviewed. This is not T008's twenty-read/history-replay gate. |
| Source limits | Real R and B are separately prepared at **512 KiB** and **512 KiB + 1**. Exact limits remain observed; over-limit records alone become unavailable/`too_large`, retaining other independently observed sources and dirty evidence. Before/after witnesses remain unchanged during the reads. |
| Rechecks/refusals | Actual buffer/dirty changes between authenticated sample and recheck are reported; plugin disable prevents recheck success. An authenticated escaping locator returns source-free refusal. A held collection causes source-free `unsupported_observation`, not a fictitious editor disconnection, while the original collection can still recheck. |
| Executor | **11 cases passed** with the installed collector: explicitly prepared cached/closed state, exact/ambiguous/ended-session outcomes, independent confined D, suspended-editor and stopped-worker timeouts, cancellation and owned-worker reaping. The caller never terminates an editor. |
| Sessions/authentication | **33 cases passed**, including three proof vectors, independent live mutual proofs, malformed/premature/replayed/reflected messages, peer capacity/expiry, unsafe metadata, lifetime renewal and rebound-port impersonation. This remains the source-free boundary group, not T006's full source-bearing routing matrix. |
| Export/privacy | Enabled, disabled and hook-only variants passed ZIP/PCK inspection and actual release-app launches: no addon/driver code, listener, advertisement or gameplay dependency. Synthetic source/authentication sentinels were absent from incidental caller/editor/export logs. |
| Coverage gate | `--scenario all` exits 1 and names the **11 remaining groups**. `clean-open` is no longer missing; no partial run is reported as full-feature acceptance. |

Final local evidence is under `~/.t004-acceptance-0wwik1u6/`: `clean-open-verified`,
`executor-verified`, `session-final`, `export-verified`, `coverage-gate`, and
`workflow-gate-smoke.json`. Each group records exact target/session identity,
source/driver/binary hashes, cases and timings. Earlier failed/diagnostic directories
are retained separately and are not passing evidence.

The largest measured clean-open/cap caller duration was **0.356 s**. Suspended-editor
timeout was **4.505 s**, stopped-worker timeout **4.513 s**, and cancellation
**0.039 s**. The 32-peer test measured maximum unauthenticated idle lifetime
**4.009 s**. The bridge begins handshake expiry at 4.0 seconds to reserve polling
margin for its 4.5-second lifetime bound; source collection retains its separate
4.5-second lease. No deadline was relaxed.

Fixture preparation waits for the actual owned window to be visible, without
requiring keyboard focus or using a fixed sleep. Cap-only fixtures deliberately
use syntax-invalid B so native validation does not copy it over the independently
prepared R; both authorities are real and native processing remains enabled.
The ordinary clean/empty US1 cases are unchanged. The executor fixture explicitly
loads/holds its synthetic cached Script rather than assuming import-time cache
state. None of these fixture actions is a product capability.

The native regressions also cover wrong request/session/document witnesses,
facts outside the sample interval, pre-sample or mismatched rechecks, independent
B limits, empty-source results, and source suppression after authenticated denial.
Those controlled peers establish boundary semantics, not GUI observability.

#### Protected live workflow

[live-editor.yml](../../.github/workflows/live-editor.yml) is manually
dispatched from `main` with the exact reviewed dispatch SHA and one implemented
group. Its hosted trust gate rejects another revision, missing reviewer protection,
self-review, wildcard branch policies and a `main` tag policy. The GUI job requires
the protected `live-editor` environment and a dedicated
`godot-live-editor-ephemeral` macOS/ARM64 runner. Provision a clean one-job GUI
environment with no human projects, credentials or privileged network access;
do not register a persistent human workstation under that label.

The environment must require reviewers with self-review prevented and permit only
the `main` branch. The workflow references no repository secrets, grants only
`contents: read`, pins checkout/upload actions to full SHAs, and does not persist
checkout credentials. It has no PR or privileged PR trigger. Artifacts contain
explicit synthetic evidence only. The existing native workflow remains
[ci.yml](../../.github/workflows/ci.yml).

Actionlint passed. A local execution of the hosted gate's Python code accepted
the valid exact-main protection configuration and rejected six unsafe revision/
protection variants using **simulated GitHub metadata**. This was not a CI run.
The repository currently has no configured environment or GUI runner; the live
workflow fails closed without that provisioning. Actual trusted GUI CI remains
required before any supported-version/platform claim.

Constitutional review: independent observations, request-local rechecks, truthful
limits and no editor mutation preserve I–IV/VII/X/XII; private routing/confinement
and source-free failures preserve V. Exact dependency/tool provenance is in
[research](research.md#t004-collector-and-verification-provenance-2026-09-26).
Mutation A–E and applied-edit durability remain inapplicable. T004 does not complete
Feature 001 or roadmap Phase 1.

## 3. Fixture driver entrypoint

The acceptance driver is `godot-addon/tests/run_observation.py`. T004 adds `clean-open` to the implemented boundary groups above; the following remains the full-feature contract, with `all` deliberately refusing incomplete coverage:

```sh
REPO="$PWD"
GODOT="$(command -v godot)"
OBSERVER="$REPO/mcp-server/target/debug/observe-gdscript"
ARTIFACTS="$(mktemp -d)"
python3 "$REPO/godot-addon/tests/run_observation.py" \
  --godot "$GODOT" --observer "$OBSERVER" \
  --scenario all --artifacts "$ARTIFACTS"
```

Run this block from the repository root. Required driver options: `--godot` and `--observer` absolute executable paths, `--scenario` (`all` or a group ID below), and `--artifacts` an empty private output directory. Exit zero means every selected case passed; nonzero must name actual failed stage, surfaces, target, and expected/observed outcome. It must not turn an unavailable GUI/export dependency into a passing skipped acceptance gate.

The driver must:

1. Copy the synthetic fixture to a disposable directory and install the addon there; never run preparation against the source fixture or an existing human project.
2. Initialize a private registry, set `GODOT_AGENT_KIT_REGISTRY` for its owned editor, deliberately enable the addon, and wait for authenticated readiness/events under explicit startup deadlines.
3. Open the actual Script screen for visible cases during **preparation**. Wait for document/editor readiness events/condition checks, not fixed sleeps. Preparation may seed known text, dirty/closed/divergent states, edit undo history, and stop/suspend owned sessions as required.
4. Independently record D from the fixture file, R from its actual loaded GDScript, B from its actual CodeEdit, document-specific unsaved paths, instance/version/open/selection/history witnesses, and native-window screenshots. These checks must not read the observer's result and treat it as their oracle or call the product's collection helper as the independent witness.
5. Issue the real caller command, capture its intentional JSON result, measure wall/monotonic duration, compare each applicable fact to the independently established authority, and check unchanged state after the read. Never infer R or B from disk/runtime agreement.
6. Write `summary.json`, explicit synthetic per-case observation/witness records, timing/identity metadata, and scoped GUI screenshots. Keep incidental stderr/editor logs separate and source/token-free. Assertions and failure messages identify surfaces/stages without echoing source payloads.
7. Resume/stop only owned suspended/running processes and remove disposable projects/registries even on failure. Retain the explicit evidence directory for review, not a product source cache.

The current manual observation invocation is:

```sh
"$OBSERVER" init-registry --registry "$REGISTRY"
GODOT_AGENT_KIT_REGISTRY="$REGISTRY" "$GODOT" --editor --path "$FIXTURE"
# In this disposable editor, deliberately enable the installed addon and open subject.gd.
"$OBSERVER" --registry "$REGISTRY" --project "$FIXTURE" \
  --script res://scripts/subject.gd
```

Run the observer in a second terminal while the editor is open. `FIXTURE` and `REGISTRY` are the private paths created for this run. With one mutually authenticated session, omission of `--session` is unambiguous; if more than one exists, use a session ID from source-free disambiguation metadata. Never display/copy the private authentication secret to choose a session; it must not appear in wire frames.

## 4. Required scenario groups and expected outcomes

Each group can be rerun using `--scenario <group-id>`. These are acceptance-driver entrypoints selected by this plan, not product capabilities.

| Group | Preparation and actual observation | Expected result / acceptance mapping |
|---|---|---|
| `clean-open` | Show a clean open external GDScript, sample authorities independently, observe twice; include an actually observed empty source fixture. | Complete, D/R/B exact, dirty clean, comparisons correct. Distinct fresh request intervals; no state/selection/history change. Empty string remains observed. **US1.1–3; FR-005/009/012; SC-001.** |
| `dirty-divergent` | Insert distinctive unsaved text into the real CodeEdit; observe before/after selecting another script as preparation. Independently establish at least two differing authorities. Also prepare textually equal-to-D but editor-reported-dirty state. | Complete when all facts readable, even when dirty/divergent. Preserve human B. Report actual R whether equal to D or B; do not infer it. Equal text does not force clean. Non-selected target remains non-selected. **US2.1–2; SC-001/002; edge cases.** |
| `dirty-unavailable` | Real open document with D/R/B available, but test-only boundary deliberately withholds document-specific dirty observability; also supply only an unattributable unsaved indication. | Limited, dirty unknown with attribution reason, independent sources retained. Neither ambiguous target nor disconnected editor; buffer untouched. The injection does not supply fake positive source evidence. **US2.3/5; FR-007.** |
| `changing-document` | Fixture changes B/D or renames/closes/replaces the target between start sample and recheck, using explicit barrier events. | Changed facts invalidated, no complete claim or mixed document identities. Unrelated facts retained; no automatic synchronization/retry. Whitespace/CRLF-only differences preserved. Known-stale content only labeled stale with independent evidence; otherwise staleness unknown. **US2.4; FR-003/013; edge cases.** |
| `routing` | Start distinguishable owned editors/projects, including two sessions for the same project on the candidate version; reuse project names/script basenames with different sentinels. Try omitted and exact session selectors. | Ambiguous with zero candidate source until exact selection; selected project/session only, no focus/first/newest choice. If same-project concurrent sessions cannot be created on a candidate environment, record the acceptance limitation and do not claim that case passed. **US3.1–2; FR-001/002.** |
| `session-loss` | Request absent session; terminate an authenticated owned editor after a known partial stage; restart same project and request ended ID. | Unavailable before authentication; disconnected after known loss, partial evidence labeled; ended ID never replaced. Stage/time recorded. **US3.3–5.** |
| `deadline` | Suspend only an owned editor after authentication so it stays connected but does not answer; separately stall an isolated worker boundary. | Timeout distinct from known disconnect, within five seconds for every case. Caller remains independent of the stalled main thread. Resume/clean up the owned fixture. No stale late response upgrades a terminal result. **US3.6; FR-015; SC-004.** |
| `confinement` | Request traversal, absolute/user/remote/symlink-escaped targets; unsafe registry/secret; malformed/oversized request frames; mismatched request/session identity; request-wide capacity limits preventing safe or meaningful evaluation. Rebind an owned ended editor's advertised port with an owned listener that lacks its secret; attempt forged, replayed, reflected, and transcript-altered proofs. | Source-free denied/invalid/protocol/unsupported outcomes as appropriate to the whole-request cause; no outside-project or unselected source, remote listener, eval, or repairs. An impostor cannot pass mutual authentication, trigger D reads, or supply accepted R/B/dirty evidence; failed proofs return `denied_access`/`authentication_failed`, never a live replacement session. Private session metadata access is limited to the tool's own owner-private state location under FR-016, not arbitrary filesystem data. Per-source limits belong to `surface-limits`, not this refusal group. **US3.7; FR-003/014/016.** |
| `closed-and-invalid` | Confirm valid external script closed with R cached, then closed/unloaded; request missing `.gd`, non-GDScript, and syntactically invalid but observable GDScript. Exercise already-identifiable built-in script. | Not-open B/dirty not applicable; cached R actual or unloaded R unavailable. Missing/invalid distinct; syntax errors do not reject observation. Built-in D unavailable, never scene text; unidentifiable built-in target unsupported without loading. **US4.1–2; FR-004/010.** |
| `surface-limits` | Real open document with unavailable B, missing/unreadable D, or unreadable/unloaded R; separately withhold open-state observability. Include mixed text/script/documentation tabs and unsupported control association. Independently exceed each applicable D/R/B source limit while other facts remain observable; include confirmed closed D/R cases and evidence gathered before the limit is encountered. | Only the over-limit source is unavailable/`too_large`, still applicable, never truncated or observed empty. Preserve all other independently observable facts and their earlier valid evidence. Absent a separate refusal/interruption, open or unknown-open state yields `limited_observation`; confirmed closed state retains `not_open` with D/R limitations and B/dirty not applicable. Per-source limits alone never justify operation-wide refusal. Missing D does not conceal B; unknown open state is not closed; no guessed parallel-array attribution. **US4.3–6; FR-005/006/010/014.** |
| `sequential-readonly` | Twenty observations: six clean, eight spanning independently prepared human-dirty states, six closed. Include selected/non-selected scripts and known prior undo history. | Zero observer-caused source writes, Save/open/close/selection changes, or history changes. Recheck disk metadata/content, open list/current tab/caret, buffer versions, dirty state, and undo/redo availability. After reads, fixture-only Undo/Redo of the prior known history must still produce the expected sequence; this verifies non-interference, not product UndoRedo support. Every result fresh or explicitly limited. **FR-011/012; SC-005.** |
| `redaction` | Unique synthetic source/secret sentinels across selected/unselected projects; observe success and all relevant failures, including authentication failures and stale-endpoint impersonation. Inspect owned handshake traffic without persisting sensitive payloads. | Source appears only in the intentional selected observation/explicit fixture evidence, not stderr, incidental editor logs, descriptors, malformed-payload diagnostics, or other-project results. Raw secrets never appear on the wire; secrets/nonces/proofs never appear in result/log evidence. **FR-016; SC-006.** |
| `export-boundary` | Export identical production fixture presets with addon enabled and disabled; inspect both artifacts and launch actual export. | Entire addon and test-driver trees absent; exported app starts no bridge/listener/descriptor and has no tooling gameplay dependency. See section 5. **Constitution VIII.** |

Every spec acceptance scenario is mapped above: US1 (3), US2 (5), US3 (7), US4 (6), totaling **21**, plus the seven listed edge cases and cross-cutting criteria. Limited/unsupported negative cases cannot replace positive clean-open/dirty-buffer proof. Deterministic fault injection is confined to test setup/ports; no public production bypass flags are added.

## 5. Export isolation

Use exact 4.7.2 export templates and a production fixture preset named `macOS`, configured to exclude `addons/godot_agent_kit/` and fixture-driver material. The EditorExportPlugin additionally skips addon paths when enabled. Repeat both enabled/disabled configurations; verify the actual exclusion pattern against archive contents rather than assuming recursive glob semantics.

For each prepared variant, with `FIXTURE` its disposable project and `OUT` an existing private output directory:

```sh
"$GODOT" --headless --path "$FIXTURE" --export-pack macOS "$OUT/fixture.zip"
unzip -Z1 "$OUT/fixture.zip"
"$GODOT" --headless --path "$FIXTURE" --export-release macOS "$OUT/fixture.app"
open "$OUT/fixture.app"
```

Expected: archive inspection finds **zero** addon/test-driver files, including remapped/compiled resources; inspect the actual release app's resource pack too. Launch the owned exported app with the fixture registry path present in its environment as a negative control: no session advertisement/listener or tooling node starts, and the minimal non-tooling fixture scene runs. A ZIP pack alone does not prove the app was built from the same preset or contains the same files. Record the preset/configuration and inspect both.

Headless export is appropriate for artifact inspection; it is not evidence about the live editor buffer. Export checks do not add a public runtime-control capability. Missing templates/export failure leaves this gate failed, not waived.

## 6. Evidence and completion criteria

A passing implementation report includes:

- Exact Godot version/hash, OS/architecture, binary checksum, Rust version/lockfile, test-driver revision, selected project/session/resource identity, and fixture preparation actions.
- Per-case observation intervals/durations, source/dirty/open-state authority witnesses, applicable limitations, native screenshots for visible cases, and the failed stage for any non-success.
- All 21 scenarios plus edge cases passing their specified outcomes, twenty-read non-interference, no wrong-target/source leakage, and **every** controlled request ≤5 seconds.
- Native baseline and dependency review results; real-editor and export evidence on the declared candidate matrix. Hosted compile-only CI cannot replace GUI acceptance. Keep privileged GUI runners isolated from untrusted PR code and secrets.
- No claim that complete observation means D/R/B convergence, clean state, permission to edit, applied mutation durability, or actual product UndoRedo support.

The original planning command exercised only its bounded GUI feasibility experiment and design-artifact checks. T001–T004 evidence is recorded in §2.1–§2.4. The full product checks above remain prerequisites for later feature/support claims.
