# Quickstart: Validate Live GDScript Observation

**Status:** T001–T008 are complete: Feature 001's **observation foundation** satisfies ordinary hosted native/workflow CI and complete maintainer-operated real-editor acceptance (§2.9). T001–T007 are merged; T008 remains in [PR #18](https://github.com/Peter-Tam/godot-agent-kit/pull/18), awaiting review/merge. Dedicated GUI CI is optional, not a completion prerequisite. Roadmap Phase 1 remains **In progress**; mutation A–E and edit durability remain pending. No broader platform/version or product mutation/UndoRedo claim is made.

This guide covers the entire observation-only specification. It does not implement the feature, derive tasks, or claim a mutation/UndoRedo/Phase 1 exit guarantee. Use the [data model](data-model.md), [caller contract](contracts/observation-api.md), and [bridge contract](contracts/bridge-protocol.md) for normative fields and outcomes instead of inferring semantics from exit status alone.

## 1. Prerequisites

T001's native library requires only Rust 1.98.1 with rustfmt/clippy and its tracked lockfile. The following Godot, GUI, Python, and export prerequisites apply to later live-editor tasks, not to the pure classification engine.

- Reviewed specification/plan and completed relevant one-task-per-PR implementation dependencies. Do not invoke generic implement-all.
- A maintainer-operated real macOS arm64 environment with exact Godot `4.7.2.stable.official.ed1daf0bf`, engine hash `ed1daf0bf001b61586d9930840f2f1394092c079`. Complete acceptance was exercised on macOS 26.6.2; document the actual environment and do not extrapolate to another platform/version.
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

Changing the globally selected Rust toolchain is unnecessary. Do not silently accept a nearby Godot patch version. Record official binary/template provenance and retain collected executable/template checksums as evidence identities, not mandatory hash-match gates. A GitHub GUI environment, self-hosted or ephemeral runner, manual dispatch and environment approval are not prerequisites for maintainer-operated acceptance.

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

#### Optional protected live workflow

[live-editor.yml](../../.github/workflows/live-editor.yml) is optional future
automation for reproducing the same real-editor acceptance in CI. It is not
required to complete T008 or Feature 001. If selected, it is manually dispatched
with the **workflow sourced from `main`** (`--ref main`) and a full
`reviewed_sha` identifying the code to test. For this solo-maintainer repository,
that explicit maintainer dispatch is the authorization step; no PR review or
second-person environment approval is required. The SHA must be the dispatch
revision of `main` or the exact current head of one eligible open, non-draft,
unmerged, same-repository PR targeting `main`. The hosted gate validates PR
association, current head/state and the environment's main-only branch policy
without checking out PR code. It fails closed on unavailable GitHub metadata.

After validation, a dedicated clean one-job macOS/ARM64
`godot-live-editor-ephemeral` runner checks out and verifies the immutable SHA.
The workflow always runs `--scenario all`, never a partial group. The GUI job
inherits only `contents: read`; the hosted gate additionally needs
`actions: read` for environment metadata and `pull-requests: read` for PR
association/details, not reviews. There is no privileged PR trigger, PAT, or
repository/environment secret. Configure exactly the `main` deployment branch
(no tags), no deployment-reviewer rules and no administrator bypass, then
provision the isolated runner. Operating instructions, permission evidence and
validation commands are in
[shared CI and live-editor operations](../../.github/README.md). The ordinary
hosted workflow is [ci.yml](../../.github/workflows/ci.yml).

The following is historical T004 evidence for the earlier gate, **not**
validation of the current maintainer-authorized workflow:
Actionlint passed. A local execution of the hosted gate's Python code accepted
the valid exact-main protection configuration and rejected six unsafe revision/
protection variants using **simulated GitHub metadata**. This was not a CI run.
At that T004 inspection there was no configured environment or GUI runner.
Missing provisioning prevents use of this optional workflow, not completion
based on the required maintainer-operated real-editor/native/export evidence.

Constitutional review: independent observations, request-local rechecks, truthful
limits and no editor mutation preserve I–IV/VII/X/XII; private routing/confinement
and source-free failures preserve V. Exact dependency/tool provenance is in
[research](research.md#t004-collector-and-verification-provenance-2026-09-26).
Mutation A–E and applied-edit durability remain inapplicable. T004 does not complete
Feature 001 or roadmap Phase 1.

### 2.5. T005 dirty and changing-document evidence (2026-09-27)

Run each US2 group against owned disposable GUI editors, using a separate empty,
absolute, mode-0700 artifact directory for each invocation:

```sh
python3 godot-addon/tests/run_observation.py \
  --godot "$GODOT" --observer "$OBSERVER" \
  --scenario dirty-divergent --artifacts "$DIRTY_ARTIFACTS"
python3 godot-addon/tests/run_observation.py \
  --godot "$GODOT" --observer "$OBSERVER" \
  --scenario dirty-unavailable --artifacts "$UNAVAILABLE_ARTIFACTS"
python3 godot-addon/tests/run_observation.py \
  --godot "$GODOT" --observer "$OBSERVER" \
  --scenario changing-document --artifacts "$CHANGING_ARTIFACTS"
```

Observed on macOS **26.6.2 arm64**, exact Godot
`4.7.2.stable.official.ed1daf0bf`, full hash
`ed1daf0bf001b61586d9930840f2f1394092c079`:

| Boundary | Executed evidence |
|---|---|
| Native | fmt, Clippy `-D warnings`, **85 tests**, doctest phase, docs and library/caller build passed. No doctest cases exist. The new caller regression covers nine dirty/divergence/transition event sequences. |
| Dirty/divergent | **9 cases** including bootstrap: selected/non-selected dirty B, independently differing R, equal D/R/B with actual editor dirty state, each partial-equality arrangement, whitespace-only and CRLF-only differences. Complete observations preserve exact independent values and comparisons. **US2.1–2.** |
| Dirty unavailable | **7 cases** including bootstrap: withheld dirty attribution, actual global unsaved work with its association map withheld, unsupported association, an actual empty Resource path, two actual nonunique paths, and mixed script/text/documentation tabs. Readable sources and known open state survive dirty uncertainty; no guessed B, clean state, target ambiguity or disconnection. **US2.3/5.** |
| Changing document | **10 cases** including bootstrap: B/version, dirty, R, D, rename, removal, native close, a retained-R change after close, and same-path replacement. Fixture barriers execute after the caller's sample/D read and immediately before recheck. Affected facts are invalidated; closure is not a fabricated closed snapshot or permission to conceal an independent R change; replacement excludes every original identity-dependent source from current comparisons. **US2.4.** |
| Non-interference | Independent before/after disk and native Script/CodeEdit/unsaved/identity/caret/selection/version/history-availability witnesses pass. Changing cases additionally preserve the post-preparation witness through the observer's recheck. Scoped native screenshots were captured and reviewed for dirty, non-selected, equal-but-dirty and mixed-tab surfaces. This is not T008's twenty-read/history-replay gate. |
| Clean regression | The unmodified `clean-open` entrypoint passes **13 cases**, including empty sources, independent R/B caps, recheck/refusal boundaries and fresh repeat observations. |
| Privacy | Existing source/authentication guards pass in the GUI/export runs. Expanded guards replay **75 incidental logs**, reject **13 source markers**, and pass an actual dirty caller/private-registry check. Synthetic source remains in intentional evidence, not incidental logs or descriptors. |
| Export | **4 cases** including bootstrap: enabled, disabled and hook-only ZIP/PCK inspection and actual exported-app launches pass. The added fixture bridge/collector are physically included under the excluded fixture-driver tree; no tooling code, listener or advertisement ships. |
| Workflow/coverage | Actionlint passes; the protected live workflow now offers the three US2 groups without changing its trust gate. `--scenario all` still exits 1 with the **eight remaining groups**. No hosted GUI run or support claim is implied. |

Passing local evidence is under `~/.t005-acceptance-w55wyas0/`:
`dirty-divergent-final`, `dirty-unavailable-integrated`,
`changing-document-final`, `clean-open-final`, `export-verified`,
`coverage-gate`, and `redaction-replay`. Final US2 caller durations were within
**0.203 s**; clean/cap regression durations were within **0.364 s**. Each group records
its exact binary/driver hashes, selected identities, intentional synthetic
results, authority witnesses and timings.

The initial attempt failed because the desktop was locked and the display asleep.
Separate nonvisual diagnostics were labeled as such, not counted as GUI
acceptance. After the desktop was unlocked, all four unmodified visible-editor
entrypoints passed with scoped screenshots. Earlier failed/diagnostic directories
are retained separately and are not passing visual evidence.

The controlled native-close case reproduced a real bug: assigning a previously
freed ScriptEditorBase to a typed variable aborted recheck and yielded
`protocol_error`. Keeping Node references untyped until validity checks restores
the structured limited/`document_closed` outcome; the retained real-editor
regression now passes. Native tab switching can copy B into R, so fixtures assert
the actual independently observed R rather than pinning it to D. Safe-save can
replace D's inode, and that independently witnessed identity change invalidates
D alone. No source divergence establishes which authority is stale: staleness
remains unknown without causal evidence.

Review also reproduced two attribution errors and retained their regressions:
two actual loaded Scripts at one path were incorrectly reported as R not loaded,
and closure hid an independent change to the retained Resource. The first now
reports `resource_unreadable`; the second rechecks R independently and preserves
both the R change and closure invalidations. Former R cannot enter comparisons.

Constitutional scope and review are recorded in
[plan](plan.md#t005-implementation-compliance); exact tool/dependency evidence is in
[research](research.md#t005-dirty-observation-and-verification-provenance-2026-09-27).
T005 adds no mutation, product UndoRedo capability, wire-schema change, or
dependency. At T005 delivery, T006–T008 cumulative acceptance remained open;
Phase 1 mutation gates remain separate. Dedicated GUI CI is optional.

### 2.6. T006 routing and interruption evidence (2026-09-27)

Run each of `routing`, `session-loss`, `deadline`, `confinement`, and `redaction`
with its own empty absolute mode-0700 artifact directory:

```sh
python3 godot-addon/tests/run_observation.py \
  --godot "$GODOT" --observer "$OBSERVER" \
  --scenario routing --artifacts "$ROUTING_ARTIFACTS"
```

Observed on macOS **26.6.2 arm64**, exact Godot
`4.7.2.stable.official.ed1daf0bf`, full hash
`ed1daf0bf001b61586d9930840f2f1394092c079`:

| Boundary | Executed evidence |
|---|---|
| Native | fmt, Clippy `-D warnings`, **93 tests** (10 unit, 31 bridge/caller, 21 confinement, 31 observation contract), doctest phase, docs and library/caller build passed. No doctest cases exist. Added cases exercise real caller processes, selected lifetime/partial-stage transitions, unavailable/malformed rechecks and already-queued late events; controlled peers are not live R/B proof. |
| Routing | **6 cases**, including bootstrap: two distinguishable same-named projects with the same script path; omitted/exact selectors; two concurrent editors of one project with different actual R/B; source-free ambiguity with the required selector; exact session provenance and no unselected source. Independent disk/native identity, source, dirty, selection, caret, version and history-availability witnesses remain unchanged. **US3.1–2.** |
| Session loss | **9 cases**, including bootstrap: absent project/ID, disable after authentication before sample, renewed session/secret, terminate after validated sample and D, stale descriptor after unclean exit, same-project restart and ended-ID refusal. D retains its original attribution; disconnected R/B/open/dirty facts become invalidated evidence and cannot imply current agreement. **US3.3–5.** |
| Deadline | **4 cases**, including bootstrap: suspend a connected owned editor at recheck; separately stop the owned worker after sample/D. Both return timeout with labeled partial evidence, not disconnection or complete/not-open. The worker is reaped, editors survive, and an explicit later request has fresh identity/evidence. **US3.6.** |
| Confinement | **40 cases**, including bootstrap and reused authentication regressions: wrong/replayed/reflected proofs and changed transcripts, peer expiry/capacity, unsafe registries/descriptors/secrets, request-wide capacity, malformed/oversized frames, request/session/project mismatch, lexical/symlink denial, unresolved candidate liveness, post-sample symlink substitution, and an owned impostor rebinding the ended session's port. Rejected samples never authorize D; denial after partial collection suppresses all source. **US3.7.** |
| Privacy | The executable incremental `redaction` group passes **24 cases**, replaying namesake/same-project source isolation, independent authentication/no-secret-on-wire checks, selected-source results and rebound-port denial. Source markers are checked in descriptors and incidental logs; secret/nonces/proofs are excluded from results/logs. This is not T008's complete privacy replay. |
| Regression/export | `clean-open` passes **13 cases**. `export-boundary` passes **4 cases**, including enabled, disabled and hook-only ZIP/PCK inspection and actual exported-app launches. Fixture barriers/negative response restrictions remain under the excluded driver tree; no listener, descriptor or tooling dependency ships. |
| Workflow/coverage | Actionlint and Python syntax compilation pass. Named groups are available under the unchanged protected live workflow. `--scenario all` still exits 1 naming closed/invalid, surface-limit, sequential-readonly and full redaction coverage; the incremental privacy group is not advertised as full acceptance. |

Passing evidence is under `~/.t006-acceptance-qld7pjvy/`: `routing`,
`session-loss`, `deadline`, `confinement`, `redaction-final`, `clean-open`,
`export-boundary`, and the expected-failing `coverage-gate`.
Every measured caller/routing request in these runs was below **4.617 seconds**;
the source-bearing editor/worker stalls returned within **4.507 seconds**.
Each summary records exact binary/driver hashes, selected identities, stages,
timings, intentional synthetic results and scoped native-window screenshots.
The routing A/C buffers and the interrupted editor's Script surface were
visually reviewed, independently of the result assertions.

The first visual attempt stopped at the locked-desktop window gate. Separate
nonvisual diagnostic runs are explicitly marked as **not acceptance**. After
unlocking, the unmodified entrypoints above passed with actual scoped captures.
Fixture corrections preserve real semantics: each newly started editor is
matched to its newly advertised ID; SIGTERM may leave stale metadata; renaming
the owned source changes `ctime`, while its content/identity/write metadata
and the displaced file remain protected. A parallel native-fixture port
allocation collision was reproduced and fixed by allocating distinct atomic
candidates. No product defect or production behavior change was established.

The production API/protocol, dependencies and architecture remain unchanged.
See the [caller semantics](contracts/observation-api.md#12-t006-source-bearing-routing-and-interruption-outcomes),
[bridge evidence boundary](contracts/bridge-protocol.md#11-t006-source-bearing-lifetime-and-interruption-evidence),
[constitutional review](plan.md#t006-implementation-compliance), and
[tool provenance](research.md#t006-routing-and-interruption-provenance-2026-09-27).
At T006 delivery, T007/T008 acceptance remained pending. Phase 1 mutation gates
remain separate; a dedicated GUI CI run is not required.

### 2.7. T007 closed and partial observation evidence (2026-09-27)

Run each US4 group with its own empty absolute mode-0700 evidence directory:

```sh
python3 godot-addon/tests/run_observation.py \
  --godot "$GODOT" --observer "$OBSERVER" \
  --scenario closed-and-invalid --artifacts "$CLOSED_ARTIFACTS"
python3 godot-addon/tests/run_observation.py \
  --godot "$GODOT" --observer "$OBSERVER" \
  --scenario surface-limits --artifacts "$LIMIT_ARTIFACTS"
```

Observed on macOS **26.6.2 arm64**, exact Godot
`4.7.2.stable.official.ed1daf0bf`, full hash
`ed1daf0bf001b61586d9930840f2f1394092c079`:

| Boundary | Executed evidence |
|---|---|
| Native | fmt, Clippy `-D warnings`, **105 tests** (10 unit, 42 bridge/caller, 21 confinement, 32 observation contract), doctest phase, docs and library/caller build passed. No doctest cases exist. Coverage includes strict metadata attribution, real caller/filesystem transitions, Unicode byte limits, closed D/R limits, disk replacement and stronger refusal/interruption precedence. Controlled peers are not live R/B proof. |
| Closed/invalid | **12 cases**, including bootstrap: cached and truly unloaded closed external scripts; missing versus non-GDScript invalid; actual syntax-invalid and empty open GDScript; native cached/closed and open built-in identities; unresolved built-in identity; container symlink and traversal refusal. D never contains container bytes. **US4.1–2.** |
| Partial/limits | **27 cases**, including bootstrap and reused collector/recheck regressions: known-open unavailable R/B, unsupported association, unknown open state, real mixed text/script/documentation tabs, open missing/unreadable D, inclusive and over-limit D/R/B, four closed cached D/R combinations, closed uncached oversized D, unknown-open oversized D, and separate denial/disconnection/silence. Other and earlier independently observed facts remain intact. **US4.3–6; CHK032.** |
| Non-interference/privacy | Both groups compare independent before/after source, file metadata, Resource/CodeEdit identity, open-list, selection/caret, versions, dirty and history-availability witnesses. The fixture retains its actual prepared CodeEdit reference to check the target buffer when mixed tabs prevent product association. Source/secret sentinels are excluded from incidental logs, descriptors and diagnostics. |
| Incremental privacy replay | `redaction` passes **61 cases**, combining source-attributed routing, both US4 groups, authentication/secret-free traffic and stale-port impersonation checks. This remains an incremental gate, not T008's full-feature privacy/support acceptance. |
| Regression/export | `clean-open` passes **13 cases**. `export-boundary` passes **4 cases**: enabled, disabled and hook-only ZIP/PCK inspection and actual exported-app launches. Addon, fixture drivers and the new built-in/syntax-invalid assets are excluded; no listener, descriptor or tooling dependency ships. |
| Workflow/coverage | Actionlint and Python syntax compilation pass. Both named groups are selectable under the unchanged protected live workflow. `--scenario all` exits 1 naming `sequential-readonly` and full `redaction`; incremental privacy is not full-feature acceptance. |

Passing evidence is under `~/.t007-acceptance-x2ozb8i8/`:
`closed-and-invalid-final`, `surface-limits`, `clean-open`, `export-boundary`,
`redaction`, and the expected-failing `coverage-gate`. Every measured caller request was
below **4.507 seconds**; the silent-editor case took **4.506267 seconds**.
Each summary records exact driver/binary/lockfile identities, selected targets,
timings, intentional synthetic sources/witnesses and scoped native screenshots.
The open built-in Script surface and the preserved buffer with missing D were
visually reviewed, independently of JSON assertions.

Four baseline defects were observed and corrected: built-in scope denial,
non-GDScript `unsupported_observation`, closed uncached oversized D becoming
limited, and a genuinely missing closed target becoming limited. Pre-fix
evidence is in `~/.t007-reproduction-icylf8gx/evidence/`,
`~/.t007-reproduction-icylf8gx/closed-limit-before/`, and the failed initial
`closed-and-invalid` run. Native regression cases now use the real collector's
unknown-validity/confirmed-closed shape, not fabricated editor disk knowledge.
The final US4 groups pass those paths. New unloaded fixtures are created only
after editor startup and independently checked uncached: native import can
cache pre-existing scripts even when no tab is open.

Unobservable/unsafe metadata remains explicit: an uncached unreadable file
without a safely verified handle cannot acquire an invented valid identity.
Known-open missing/unreadable D still retains independent R/B. No observation
opens, selects, force-loads, repairs or saves a document. Metadata presence/absence
is not source text, three-way agreement, mutation authorization or atomicity.
The dependency graph and public version-1 JSON contract are unchanged.

See [caller semantics](contracts/observation-api.md#13-t007-closed-and-partly-observable-documents),
[bridge metadata boundary](contracts/bridge-protocol.md#12-t007-closed-built-in-and-filesystem-evidence),
[constitutional review](plan.md#t007-implementation-compliance), and
[tool provenance](research.md#t007-closed-and-partial-observation-provenance-2026-09-27).
At T007 delivery, T008 cumulative acceptance remained pending. Phase 1 mutation
gates remain separate; a dedicated GUI CI run is not required.

### 2.8. T008 cumulative local acceptance (2026-09-27)

Initial, pre-rebase evidence: the real `--scenario all` invocation passed
**194 cases** on macOS **26.6.2 arm64**, Godot
`4.7.2.stable.official.ed1daf0bf` (full engine hash
`ed1daf0bf001b61586d9930840f2f1394092c079`). All 13 required groups executed;
the following counts exclude the single shared bootstrap case:

| Group | Cases | Evidence boundary |
|---|---:|---|
| `clean-open` | 12 | Independent clean/empty D/R/B, repeat reads and collector/recheck boundaries; US1.1–3. |
| `dirty-divergent` | 8 | Human-unsaved, non-selected, equal-text-dirty and exact whitespace/line-ending differences; US2.1–2. |
| `dirty-unavailable` | 6 | Unavailable/unattributable dirty evidence, mixed tabs and unsafe association; US2.3/5. |
| `changing-document` | 9 | Barrier-controlled source, close, rename, remove and replacement invalidation; US2.4. |
| `routing` | 5 | Distinct namesake projects and same-project sessions, source-free ambiguity and exact source attribution; US3.1–2. |
| `session-loss` | 8 | Absence, authenticated loss, partial facts and ended-session replacement refusal; US3.3–5. |
| `deadline` | 3 | Connected suspended editor, independently stalled worker and fresh post-timeout request; US3.6. |
| `confinement` | 39 | Scope/metadata/authentication, malformed framing, capacity, stale-endpoint impostor and denial; US3.7. |
| `closed-and-invalid` | 11 | Closed cached/unloaded, missing/invalid/syntax-invalid/empty and built-in identities; US4.1–2. |
| `surface-limits` | 26 | Independent unavailable/over-limit D/R/B, unknown open state and stronger terminal precedence; US4.3–6 and CHK032. |
| `sequential-readonly` | 21 | Twenty real observations plus native prior-history replay; FR-011/012 and SC-005. |
| `redaction` | 42 | Additional complete session/authentication/stale-endpoint and executor boundary replay, plus privacy checks over all eleven preceding groups. |
| `export-boundary` | 3 | Enabled, disabled and hook-only ZIP/PCK inspection and actual exported-app launches; no tooling material, listener or advertisement. |

The sequence contains **6 clean, 8 independently prepared human-dirty and
6 closed** requests. Clean and dirty targets include selected and non-selected
documents. Every read compares independently sampled D/R/B, disk content and
metadata, open-document identities/list, current tab, caret/selection, versions,
dirty state and undo/redo availability before/after. All 20 request IDs are unique;
native collection stamps advance. Known prior two-edit history survives and
fixture-only **Redo → Undo → Undo → Redo** reproduces the exact expected states.
This proves preservation of that history, not a product UndoRedo capability.

All **140 timed checks** remain below five seconds: maximum **4.766403 s** for
peer-capacity/unauthenticated expiry; the slowest timed caller outcome is
**4.508021 s**. Result-only review of **126 distinct retained caller results**
checks selected target/refusal, D/R/B availability and collection, and explicit
reason/action fields for missing source and dirty knowledge. All 21 acceptance
scenarios and seven edge cases retain their mappings in §4 and `tasks.md`.

Each group now owns separate working/evidence directories; repeated collector
cases cannot collide or overwrite another group's records. `summary.json`
records group counts and each case's `artifact_directory`, evidence and scoped
screenshots. `all` reuses its actual eleven-group run for full privacy review
instead of repeating it; standalone `redaction` executes that replay itself.
Source/secret checks recurse through all retained group evidence. Only deliberate
synthetic results/witnesses contain source; incidental logs remain source-free.
Native dirty-buffer and closed-editor screenshots were visually reviewed.

Integration reproduced and corrected fixture-only failures: invalid GDScript
history-loop syntax, hidden earlier owned editor windows, colliding shared
fixture names, and an assumption that background import would leave a closed
script uncached. Capture explicitly presents only the owned process; closed R
assertions now use actual cache identity/source witnesses, never timing or D.
No product collector, caller, core, dependency or public schema changed.

The initial native fmt, Clippy `-D warnings`, **105 tests** (10 unit,
42 bridge/caller, 21 confinement, 32 observation contract), doctest phase (zero
cases), docs and library/caller build passed. Actionlint and eleven simulations
of the then-current exact-main trust gate also passed. Those simulations are
historical evidence only: PR #19 later superseded that gate with
reviewed-PR authorization and its own regression suite; the current
solo-maintainer dispatch model supersedes that review requirement. Neither
simulated workflow metadata nor local GUI execution is actual protected GUI CI.
The 59-package archive/license/provenance review and advisory audit are recorded in
[research](research.md#t008-cumulative-dependency-and-compatibility-review-2026-09-27).

Passing full evidence is in `~/.t008-acceptance-kgugzz75/all-final/`; the parent
directory retains dependency/trust-gate/result-review records and the diagnostic
runs. Every group cleans up only its owned editors, projects and metadata.

| Exercised artifact | SHA-256 |
|---|---|
| Godot executable | `c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf` |
| Official `macos.zip` template | `88df5e2e6fee99088699be66e6d42e4da4fb0c5619d054297d755a49558a4792` |
| `observe-gdscript` | `16aed5592c75717a0f8223e6b095594e9a9290850e9979e73820422b8fbf214f` |
| Python acceptance driver | `ec0d0105dc144cf9023fbe0304dc67d1ca7f81b08906cbe393850672f32f336f` |
| GDScript fixture driver | `6d24744fe8d619511f9a2a9d0360f235fbb9a372f74ea3efad20c71ac2d4c720` |
| `Cargo.lock` | `dbba0e851819c51a4623e1b58b8cf7d583ff6ac372b66ea7e9f07b366db02fe1` |

**Compatibility/completion boundary:** These are actual maintainer-operated
real-editor results, not a claim that protected GUI CI executed. The historical
hosted native [run 36310724143](https://github.com/Peter-Tam/godot-agent-kit/actions/runs/36310724143)
tested the merged prerequisite revision only. Section 2.9 records the full
post-rebase replay and final validity review; ordinary hosted native/workflow
checks are still required. Dedicated GUI CI is optional automation, not the
basis required for a support claim. Claims remain limited to the exact exercised
environment and documented guarantees; this evidence does not satisfy any
Phase 1 mutation gate.

### 2.9. Rebased T008 real-editor acceptance and completion (2026-09-27)

Existing [PR #18](https://github.com/Peter-Tam/godot-agent-kit/pull/18) was rebased
onto `main` at `67d4b630769e1f11803ac69da423734735599046`, which merged
[PR #19](https://github.com/Peter-Tam/godot-agent-kit/pull/19). At that rebase,
conflicts were limited to `ci.yml` and `live-editor.yml`; both then matched the
merged `main` versions byte-for-byte, with no workflow diff in T008. The Python
acceptance driver and GDScript fixture driver also remained byte-identical to
PR #18's original implementation. No acceptance assertion or product
functionality changed in that rebase.

At that PR #19 rebase, fresh verification on the same exact macOS 26.6.2 arm64 / Godot 4.7.2 candidate:

- `--scenario all`: **194 passed cases across all 13 groups**, with every
  per-group count unchanged from §2.8. No hidden skip or partial group counts as
  acceptance.
- `sequential-readonly`: **20 distinct requests**, exactly **6 clean / 8 dirty /
  6 closed**. Selected/non-selected cases, fresh native collection, independent
  D/R/B and before/after disk/editor witnesses pass. Fixture-only native
  **Redo → Undo → Undo → Redo** still reproduces the known prior history.
- Full privacy replay passes: all eleven observation groups plus 42
  session/authentication/executor cases. Enabled, disabled and hook-only exports
  pass ZIP/PCK checks and actual app launches with no tooling listener.
- All **140 timed checks** remain under five seconds: maximum **4.769260 s**;
  slowest timed caller outcome **4.519666 s**. Fresh result-only review covers
  **126 distinct caller results** and confirms all 20 unchanged sequence witnesses.
- Native fmt, Clippy `-D warnings`, **105 tests**, doctest phase (zero cases),
  docs and library/caller build pass with Rust 1.98.1.
- At the PR #19 rebase, all **29 then-merged workflow regression tests**
  passed using pinned **PyYAML 6.0.3**, including the then-current
  reviewed-PR/latest-review authorization, changed-head refusal, checkout
  equality, candidate/native preflight failures and complete-suite execution
  ordering. Python syntax, YAML parsing and Actionlint 1.7.12 passed. This is
  historical workflow-simulation evidence, not protected GUI CI or a test of
  the later solo-maintainer authorization model.
- Fresh cargo-audit again reports zero vulnerabilities and warnings against
  RustSec `e2111519ba6d14a5da59a7b2e5c8083ae8a37c01`; all **59/59** registry
  archive checksums match. Every binary/template/driver/lockfile checksum in
  §2.8 is unchanged. Scoped dirty-buffer and closed-editor screenshots were
  visually reviewed.

Retained maintainer-operated real-Mac evidence from that PR #19 rebase:
`~/.t008-rebase-ox24xk8i/all/summary.json`, per-group artifacts under that
directory, and sibling acceptance/result-only/dependency review records.
The environment was macOS **26.6.2 arm64**, Godot
**4.7.2.stable.official.ed1daf0bf**, Rust/Cargo **1.98.1**, and Python **3.10.9**.
The executable, template, harness, fixture and lockfile identities in §2.8 are
retained as provenance, not mandatory executable/template hash gates.

**Required acceptance model:** ordinary hosted GitHub native/workflow checks
pass, and the maintainer executes the complete real-editor `--scenario all`
suite on the documented real Mac with the pinned Godot candidate. All 13 groups,
21 acceptance scenarios, seven edge cases, twenty-read 6/8/6 sequence,
independent D/R/B/dirty evidence, non-interference, native history preservation,
privacy/redaction, export isolation, deadlines, result-only reviewability and
environment/provenance recording remain mandatory.

**Final evidence review:** the retained summary contains 194 passing cases
across all 13 groups. All 20 distinct sequential requests retain equal
before/after disk/editor witnesses and the 6/8/6 distribution; selected and
non-selected documents, native prior-history replay, all privacy/export evidence
and 140 sub-five-second timings remain recorded. The 126-result review and
unchanged-lockfile dependency/provenance review remain applicable. Product,
harness and fixture source trees are unchanged from the exercised post-rebase
revision; their recorded identities remain valid. No expensive real-editor
rerun is needed for this documentation/status correction. Review metadata is
retained in `~/.t008-acceptance-review-s9a8ttlg/retained-evidence-review.json`;
native/workflow validation and the current hosted CI result are recorded in
[PR #18](https://github.com/Peter-Tam/godot-agent-kit/pull/18).

All substantive T008 requirements are satisfied: **T008 is `[X]` and Feature 001's
observation foundation is complete**. Dedicated GitHub GUI CI, its environment,
registered/ephemeral runner, candidate-SHA dispatch and environment approval are
optional infrastructure, not remaining gates. No protected GUI CI run or broader
supported-version/platform claim is asserted. PR #18 remains unmerged.
**Roadmap Phase 1 remains In progress; mutation A–E and edit durability remain pending.**

## 3. Fixture driver entrypoint

The acceptance driver is `godot-addon/tests/run_observation.py`. All 13 groups are implemented; `all` executes the complete required real-editor acceptance matrix on the maintainer-operated real Mac, and `redaction` covers every observation group plus session/executor boundaries. Run with the pinned candidate and record the exercised environment. Dedicated GUI CI is not required:

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

- Exact Godot version/engine hash, OS/architecture, Rust version, harness/driver/lockfile identities, selected project/session/resource identity, and fixture preparation actions. Retain collected executable/template hashes as provenance, not mandatory hash-match gates.
- Per-case observation intervals/durations, source/dirty/open-state authority witnesses, applicable limitations, native screenshots for visible cases, and the failed stage for any non-success.
- All 21 scenarios plus edge cases passing their specified outcomes, twenty-read non-interference, no wrong-target/source leakage, and **every** controlled request ≤5 seconds.
- Passing ordinary hosted native/workflow CI, native baseline and dependency review results, and complete maintainer-operated real-editor/export evidence on the declared candidate matrix. Hosted compile-only CI cannot replace GUI acceptance. Dedicated GUI CI is optional; if used, its privileged runners remain isolated from untrusted PR code and secrets.
- No claim that complete observation means D/R/B convergence, clean state, permission to edit, applied mutation durability, or actual product UndoRedo support.

The original planning command exercised only its bounded GUI feasibility experiment and design-artifact checks. T001–T007 evidence is in §2.1–§2.7; §2.8 records initial T008 acceptance and §2.9 its full post-rebase verification and completion review. The documented maintainer-operated real-editor results plus ordinary hosted/native checks satisfy Feature 001's observation acceptance. Lack of a self-hosted GUI runner does not block completion; no untested version/platform, mutation guarantee or Phase 1 completion follows.
