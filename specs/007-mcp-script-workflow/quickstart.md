# Quickstart: Validate the Trusted MCP Script Workflow

**Status:** T001–T003 are **complete**, with [core/native evidence](#t001-execution-evidence-2026-10-04), [positive MCP/two-client acceptance](#t002-execution-evidence-2026-10-05), and [adversarial safety/two-client failure acceptance](#t003-execution-evidence-2026-10-05). Codex CLI 0.153.4 and OMP 18.5.1 completed their positive and failure/reconnect profiles. T004 remains pending and unselected; feature completion and Phase 3 exit are not claimed.

Read [plan.md](plan.md), [data-model.md](data-model.md) and [the public contract](contracts/mcp-interface.md). Follow [TEST_POLICY.md](../../TEST_POLICY.md) for scope, evidence reuse and the existing [VM boundary](../../.github/LOCAL_VM.md). No command authorizes a feature task before its own approved tasks/analysis and one-task-one-PR prerequisites.

## Prerequisites and available checks

- Existing exact stock Godot 4.7.2.stable.official.ed1daf0bf, macOS 26.6.2 arm64 profile and engine/native provenance. Keep the recorded support boundary; a different environment is not equivalent support evidence by assumption.
- Rust 1.98.1, edition 2021, tracked locked dependencies; matched private bridge v6/native revision 4 after implementation. Old native revision 3 does not implement closed editing.
- Existing dedicated Tart VM, private synthetic fixture projects, unlocked guest GUI and owned-window captures. Godot/core/registry stay inside the guest for acceptance; no host GUI fallback or personal projects/credentials in the guest.
- Selected installed independent coding-agent clients: **Codex CLI 0.153.4** and **Oh My Pi (OMP) 18.5.1**, re-observed for this correction. Host-owned model authentication/approval is never copied into repository/guest evidence. Unavailable real model access cannot be replaced with mock calls or another client. See [source-backed suitability and limits](research.md#8-codex-and-omp-client-correction-evidence).

These lightweight installed-client checks were executed for the correction and remain runnable from the repository root:

```sh
codex --version
codex exec --help
omp --version
omp --help
omp config --help
```

Observed outputs: `codex-cli 0.153.4` and `omp/18.5.1`. The retained Codex version is the current installed executable, not the newer upstream 0.160.0 release; no installation was changed. OMP package/source and authoritative docs establish structural MCP suitability, not interoperability. The original planning VM/Tart observations remain historical evidence; no VM/status/model/MCP acceptance run was repeated for this correction.

## Build and local connection

From `mcp-server/`, build the actual new executable and its library with the selected locked feature graph:

```sh
cargo +1.98.1 build --locked --bin godot-agent-kit-mcp
./target/debug/godot-agent-kit-mcp --help
./target/debug/godot-agent-kit-mcp --version
```

Expected: no editor effect, no source/credential output. For ordinary developer use with an already-configured local editor/registry, the client starts that binary with `--registry "$REGISTRY"`. Registry setup/addon enablement/editor startup remain deliberate developer actions. The server does not infer a project from cwd or start Godot.

For implementation verification, start with the smallest affected test target. At Rust-affecting task completion run the applicable baseline from `mcp-server/`:

```sh
cargo +1.98.1 fmt --all -- --check
cargo +1.98.1 clippy --all-targets --locked -- -D warnings
cargo +1.98.1 test --locked
cargo +1.98.1 doc --no-deps --locked
```

Retain default doctests. Native/bridge changes also require their directly affected build/loading/boundary evidence. Do not run these product commands merely because planning documentation changed.

## T001 protocol-independent consumer

T001 adds `script_read::{ScriptRevision, ScriptEditRequest, ScriptReadResult}`
and supervised `runner::read::{run, edit}`. Read returns exact permitted source,
useful editor state and a nullable `sr1:` revision. Edit freshly authenticates and
acquires state, compares that revision, then freezes the matching open or closed
basis under the original clock. The legacy open-only caller still refuses closed
targets. Loaded Script source coherence does not imply class or runtime hot reload.

The example is acceptance apparatus, not a new product CLI or MCP adapter:

```sh
cd mcp-server
cargo +1.98.1 build --locked --lib --bins --example script_workflow_fixture
printf '%s\n' '{"operation":"read","request_id":"read-example"}' |
  ./target/debug/examples/script_workflow_fixture \
    --registry "$REGISTRY" --project "$PROJECT" --script "$SCRIPT" --session "$SESSION"
```

An intentional edit supplies only `operation: "edit"`, a new `request_id`,
the returned non-null `revision` and exact `replacement_source`, with the same
explicit selectors. A null revision or failed/uncertain edit is not permission
to retry or change document lifecycle. Native RPCs and private captures are not
fixture consumer arguments.

Install matched Rust/addon **bridge v6/native revision 4** peers together and
restart/re-enable for a new authenticated session. Rebuild production and separate
fixture-fault native artifacts; never install the latter for ordinary use.
Old v5/revision-3 sessions and bases have no fallback.

### T001 validation scheduling review

The first exact-512-KiB VM edit applied all bytes but returned
`applied_unverified` after 9.079435 seconds because actual-source validation was
unavailable. Original and desired preflight validation were serialized, each
starting the existing isolated stock validator. Keep both checks, but run these
independent preflight validations concurrently under the same attempt clock and
cancellation flag; neither can authorize effects alone. Post-effect validation
and the native main-thread sequence remain independent and unchanged.

The simpler serial route failed this supported boundary. A new multi-source LSP
protocol would add more state and compatibility work than reusing two existing
bounded workers. The current cost is one scoped coordination thread and at most
two simultaneous isolated validation children, with their existing provenance,
cleanup and deadlines. No new dependency, extended lease or retained validator
service is introduced. Exact unchanged intent validates the shared original/desired
source once during preflight; both parties require byte equality before reusing
that verdict. The accepted positive group verified all 30 cases; the exact-512-KiB
changed edit completed in 8.638174 seconds with independently observed postconditions.

Focused reruns also expose the existing named subgroups through the runner's
fixed scenario table. A getter-only failure otherwise replayed 30 already-passing
positive cases before reaching its own group; the two aggregate selectors could
not isolate that failure. This reuses the existing close-runner dispatch, capture,
provenance and cleanup conventions, costing one fixed selector table rather than
a new campaign/resume mechanism. It neither skips assertions nor changes the
requirements for cumulative coverage.

## T001 execution evidence (2026-10-04)

T001 is complete on `task/T001-trusted-script-execution`. **324 accepted cases**
across the 11 groups below cover the task-owned core/native behavior. Counts exclude
repeated runner bootstrap checks. No MCP SDK/server, selected-client model session,
MCP accepted-frame timing, composed real-agent A–E or Phase 3 exit is claimed.

Every run used `godot-addon/tests/run_in_vm.py run mcp --scenario … --captures`
with committed source and owned synthetic projects. All listed subgroup names are
also directly runnable through the fixed scenario selector. Artifacts are under
`~/.local/state/godot-agent-kit-vm/artifacts/<run>/<snapshot>/`, containing
`provenance.json`, `artifacts/summary.json` and retrieved owned-window captures.
Raw source-bearing records and independent detailed witnesses remain in the private
guest run directory; they are not committed or published as protocol output.

| Accepted group | Cases | Tested source | Run / snapshot |
| --- | ---: | --- | --- |
| `closed-positives` | 30 | `ca4d3024c476` | `t001-closed-native-6` / `20261004T132114Z-562393ac67b4` |
| `closed-refusals` | 39 | `7e6fc2206baa` | `t001-closed-refusals-1` / `20261004T133125Z-03de857cf09b` |
| `closed-revisions-and-boundary-races` | 75 | `6535c292f0d2` | `t001-closed-revisions-2` / `20261004T134417Z-fd5314a7631c` |
| `closed-effect-faults` | 22 | `edf8c3a1768a` | `t001-closed-effects-2` / `20261004T140058Z-012c5f31d339` |
| `closed-acquisition-invalidation-and-selection` | 12 | `2c815415e873` | `t001-closed-acquisition-3` / `20261004T141603Z-eeed4317f2fe` |
| `closed-authenticated-wire-boundaries` | 22 | `5b5953b3f967` | `t001-closed-wire-4` / `20261004T142855Z-4d13c467e9d1` |
| `closed-save-profile-and-shared-slot` | 7 | `5b5953b3f967` | `t001-closed-slot-1` / `20261004T142943Z-be4ea725ff4b` |
| `closed-cancel-and-newer-work` | 74 | `5b5953b3f967` | `t001-closed-lifecycle-1` / `20261004T143844Z-98bda4f0a650` |
| `closed-later-durability-and-history` | 8 | `4143e73b9507` | `t001-closed-durability-1` / `20261004T144452Z-fc2fd54287b1` |
| `matched-v6-native4-legacy-preservation` | 13 | `4143e73b9507` | `t001-closed-legacy-1` / `20261004T144541Z-770c22205993` |
| `closed-privacy-export` | 22 | `4143e73b9507` | `t001-closed-privacy-1` / `20261004T144834Z-f4888f57fcfc` |

The positive and cancellation groups completed before later groups failed in their
aggregate runs. Only their completed group records are reused; neither failed
aggregate is represented as a passing run. Every failed group was subsequently
corrected and executed successfully in its own scope.

### Environment, provenance and timings

- Guest: macOS **26.6.2 / 25G83**, arm64 `VirtualMac2,1`, four CPUs, 6 GiB;
  Godot **4.7.2.stable.official.ed1daf0bf**,
  engine commit `ed1daf0bf001b61586d9930840f2f1394092c079`.
- Official binary SHA-256:
  `c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf`;
  export-template SHA-256:
  `88df5e2e6fee99088699be66e6d42e4da4fb0c5619d054297d755a49558a4792`.
- Rust/Cargo **1.98.1**; Apple clang **21.0.0**, SDK **26.5**. The generated API,
  ABI/header, native manifests, binaries, fixtures and runner hashes are retained
  in each run's existing provenance/summary, including cache receipts.
- Final production native build:
  `7203d78018cae3b6b507efbefac688100dd8dd4f8032ab10a6ccfc2f3941ad71`;
  library SHA-256 `03911a9420ddb663b3717fa1de8cd15c6ac1700bd7469022400dd0c58050dbcb`.
  Separate fixture build:
  `66df032e755067eb46b18a498bee023e63975d74d8a615170ee4c839a4f2dba6`;
  library SHA-256 `d97489b0d6b1b30e9b89503725fb7c4d12ff7595b884b16ba0ce3b1b88f6538a`.
- Across accepted groups, 152 supervised reads completed in **0.260089–1.515744 s**
  and 135 supervised edits/refusals in **0.004107–9.516107 s**. The longest edit
  was the deliberate silent-editor refusal; the longest verified success was the
  **512 KiB** changed edit at **8.638174 s**. Existing 5/10-second caller bounds,
  4.5/9.5-second work cutoffs and nine-second native lease remain unchanged.
- Actual captures show the target remaining closed during mutation, then visible
  `return 83` after later opening/Save/reparse/rescan while the unrelated human
  script remains dirty. Fresh runtime separately returned 83. Real Undo/Redo was
  exercised on the preserved human document, not manufactured for closed editing.
  Owned editors/helpers were cleaned up; the dedicated Tart VM was stopped and
  `status` confirmed `running: false`, `state: stopped`.

### Required checks and resolved regressions

Formatting and Clippy (`--all-targets --locked -- -D warnings`) passed.
`cargo test --locked -- --test-threads=1` passed **406 tests across 20 suites**,
including the default doctest targets. Rustdoc and actual library, binary and
example builds passed with locked resolution. The ordinary parallel test run hit
the existing open late-compilation test's ten-second wall-clock assertion; its
isolated execution and the complete serial baseline passed without changing the
assertion, deadlines or production timing semantics.

The affected VM/native-build tests passed **38 tests**, and workflow execution/gate
fixtures passed **22 tests**. Changed Python entrypoints compiled and the actual
runner help/selection surface was exercised. No unrelated workflow/CI configuration
changed, so no new Actionlint or hosted-provider gate was introduced.

Executed regression work corrected the native dictionary-comparison ABI call,
serialized preflight cost at the source limit, getter failures falsely reported
as source/lifecycle changes, missing disk coexisting with a retained Script,
partial-source evidence incorrectly requiring a still-admissible profile, and
sticky disclosure after earlier native failure. The incomplete-profile model
regression failed before correction and passed afterward; it still rejects such
evidence as mutation authority. Existing guards were not weakened to obtain passes.

Fixture corrections preserve the actual contracts: absent R has no invented source;
a revision is not a parse verdict; unused autoload names alone are supported by the
source profile; epoch races occur after the first captured epoch; fresh acquisition
uses a distinct request ID; ordinary-read busy behavior retains its existing v1
meaning; human idle parsing settles before preservation baselines. Strict tuples,
real effect prefixes and independent witnesses remain enforced.

### Currentness, implementation shape and constitutional review

Evidence reuse follows [TEST_POLICY.md](../../TEST_POLICY.md#reusing-evidence-across-commits),
not a literal-final-commit replay. Later getter/missing-disk repairs affect refusal
and acquisition paths, which were rerun. The native partial-state correction leaves
normal admission/effect guards intact and adds read-only terminal collection;
partial effects were rerun, and the final native build also executed successful
slot-owner edits, cancellation, durability, legacy operations and exports. Later
fixture changes affect only the specifically rerun wire/acquisition/durability
cases. In particular, clearing stale barrier files does not alter earlier exact-ID
barrier assertions. Documentation/status changes do not invalidate these inputs.
No full historical observation/edit/open/discovery/close campaign was replayed.

The implementation-shape review found cohesive owners: `script_read` owns checked
intent/revision/presentation; `runner/read` owns fresh acquisition and fixed branch
selection; closed model/effect reduction, strict wire codecs, supervisor and worker
are separate. The 760-line closed worker was explicitly reviewed: its capture,
validation, native sequencing and disclosure helpers belong to one owned child
execution boundary. A further split would add visibility/import surface without
reducing the next adapter task's review cost. The read coordinator remains one
210-line module rather than creating an unused `runner/read/` directory.

The native owner and addon lifecycle use existing session/operation admission.
Effect flags describe independently observable progress, not interchangeable
lifecycle booleans. Raw captures, worker controls and effect authority remain
private/crate-private; public declarations have current supervised consumers.
Typed failure paths retain missing/unsafe evidence rather than panicking. Existing
profile/validator/confinement machinery is reused, with no duplicated feature
infrastructure, speculative hooks, extra dependency or compatibility fallback.

Constitutional review preserves independent D/R/B applicability, human work,
stale/lifecycle refusal, canonical native mutation, independently observed success,
sticky partial/unknown effects and late denial, local routing/confinement and
enabled/disabled/hook-only export isolation. Closed history remains not applicable;
loaded class/runtime hot reload is not promised. Existing open native history and
Phase 1 A–E evidence remain valid, supplemented by the affected matched-peer/open
workflow witnesses. The new composed MCP A–E and both real-client obligations remain
T002–T004 work. Complexity additions and their present justification are recorded
in the scheduling review above. Only T001 is marked complete.

### Post-review correction: stable detected-change projection

`ScriptReadResult::encode` now presents `state.consistency.detected_changes` as
the established observation-v1 array of `{"surface": "...", "code": "..."}`
objects, using an exhaustive local match rather than Rust `Debug` strings:

| Internal change | `surface` | `code` |
| --- | --- | --- |
| `Source(D)` | `D` | `source_changed` |
| `Source(R)` | `R` | `source_changed` |
| `Source(B)` | `B` | `source_changed` |
| `Dirty` | `dirty` | `source_changed` |
| `DocumentClosed` | `document` | `document_closed` |
| `DocumentIdentityReplaced` | `document` | `identity_changed` |
| `SessionReplaced` | `session` | `identity_changed` |
| `SessionEnded` | `session` | `session_ended` |
| `DiskIdentityReplaced` | `D` | `identity_changed` |

This supersedes the pre-review Debug-string projection on this **unmerged T001**
delivery; it is not a legacy v1 or revision-contract change. The private
`bridge/wire.rs::change_out` remains unchanged and private. The smallest local
mapping satisfies Principles IX/XIII without a generic schema layer, added
dependency or wider public visibility. Only the caller-facing encoder and
deterministic regression additions change code; hashing/commitment, private
matching input, source disclosure, acquisition, branch selection and effect
state remain unchanged.

The **324 accepted cases**, **406 Rust tests** and **38 + 22 (60) Python tests**
above remain historical evidence, not newly executed results for this correction.
The retained `t001-closed-legacy-1/20261004T144541Z-770c22205993/provenance.json`
and `artifacts/summary.json` were reviewed: they retain the tested revision,
native/API/ABI hashes, environment, fixture/runner/acceptance hashes and the
13-case matched-v6-native4 legacy group. Their production native library hash
matches the native provenance recorded above. The prior 324-case/native evidence
is reusable for its existing guarantees because production mutation/acquisition,
native/addon/ABI, authentication/routing, fixtures/witnesses, runner/environment
and acceptance inputs are unchanged by this projection-only correction; it does
not establish the corrected JSON shape. No VM/native campaign is rerun, and no
new MCP/T002 scope or acceptance claim is introduced.

Focused validation passed under Rust 1.98.1: both new detected-change regressions
(all nine values, accumulated change ordering and encoded Debug-name rejection),
all **13** `script_read` library tests and the public `script_read` integration test.
A throwaway harness reused the existing authenticated boundary peer to launch the
actual rebuilt `script_workflow_fixture` consumer in four read scenarios: changed
R, changed B/dirty, document closure and document replacement. Its stdout contained
the exact structured change arrays, limited state and null revisions; fixture disk
source was unchanged. This is deterministic caller/serialization smoke, not new
real-Godot evidence. The harness and executable were removed.

`cargo +1.98.1 fmt --all -- --check`,
`cargo +1.98.1 clippy --all-targets --locked -- -D warnings`,
`cargo +1.98.1 doc --no-deps --locked` and
`cargo +1.98.1 build --locked --lib --bins --examples` passed. LSP diagnostics
reported no issues in either changed Rust file.

**Task-specific Rust baseline exception:** The normal
`cargo +1.98.1 test --locked` was attempted again and **failed (exit 101)**:
`script_open::a_late_compilation_result_cannot_upgrade_a_delivered_known_partial_timeout`
hit the unchanged `elapsed <= Duration::from_secs(10)` assertion at
`tests/bridge_boundary/open.rs:807`; that boundary suite had 62 passes and one
failure. Its preceding outcome/effect assertions passed. `invoke_open` measures
process launch through consumed output; under default parallel execution that
wall-clock interval exceeded ten seconds. The isolated exact test passed, then
the complete `cargo +1.98.1 test --locked -- --test-threads=1` baseline passed
**408 tests across 20 suites**, including the default doctest targets.
Parallel contention is the inferred scheduling cause, not a measured scheduler
diagnosis or a claim that the failed run passed.

This applies the concrete-exception provision in [TEST_POLICY.md § Rust](../../TEST_POLICY.md#rust)
to the [T001 Rust completion requirement](tasks.md#verification-and-completion-rules-for-every-task):
the same unchanged timing/effect assertion passes in isolation and in the complete
serial suite, while the affected projection receives exhaustive deterministic and
actual-caller evidence. Those results provide equivalent behavioral coverage for
this serialization-only correction; the earlier parallel failure remains historical
development evidence. No test assertion, production timeout, operation deadline
or lease was relaxed. This is not a global serial-test rule for future tasks.

## T002 execution evidence (2026-10-05)

T002 is complete on `task/T002-mcp-script-workflow`. The production corrections
are committed at `2ddc16d33f4cd9d0a9a347ac68bd47ad694c2006`;
`12ac3c0ddc56e9833aa8e17eef4c30d5b01a2e14` adds only the fixture
admission-error reduction correction. Both selected clients completed all five
profiles: **127 independent records / 109 correlated tool results**. T003–T004
remain pending; this is the positive MCP MVP, not feature/phase completion.

### Runtime and required checks

- Rust formatting and Clippy (`--all-targets --locked -- -D warnings`) passed.
  Default Rust coverage is **455 passing tests** across the completed runs:
  224 library, 63 bridge-boundary, 153 remaining integration and 15 MCP transport/
  tool tests. The aggregate `cargo test --locked` run failed the unchanged
  `script_open::a_late_compilation_result_cannot_upgrade_a_delivered_known_partial_timeout`
  elapsed-time assertion, also observed before these corrections. Its exact
  isolated invocation passed with the original ten-second assertion; the other
  62 bridge cases passed in the aggregate run. This is not a claim that the
  aggregate invocation passed. Remaining integration targets, default doctest
  target and `cargo doc --no-deps --locked` passed.
- The corrected closed-worker channel transfers complete source frames without
  per-chunk polling sleeps. Stalled-peer cancellation, incomplete-frame deadline,
  invalid length and consecutive-frame regressions pass. No work/output deadline,
  source limit or Godot per-frame network budget changed.
- The actual stdio/real-Godot `transport-bound` smoke passed **7 cases** on
  `2ddc16d`: `t002-bound-corrected-ipc/20261004T231655Z-2328484169f7`.
  The unchanged request returned `verified_unchanged` / `not_applied` in
  **4.708534 s**; the exact **524,288-byte** replacement returned
  `verified_changed` / `applied` in **7.302625 s**. Independent disk/cache/roster
  witnesses retained absent R and closed B; the captured editor surface kept only
  the unrelated scripts open. Public closed B is `not_applicable_closed`.

The fixed finalizer distinguishes adapter admission/host errors from operation
results while retaining every attempt and its independent witness. The first
corrected Codex workflow completed primary edits and later actual reads, including
an explicitly retried read after `server_busy`, but finalization dereferenced that
error's null result. Its failed evidence remains at
`t002-codex-workflow-4/20261004T232658Z-e9669c07c02f`; no durability pass is inferred.
The regression failed before the fix and all **9** fixture tests passed afterward.
An error alone still cannot authorize later Save/reparse/rescan/runtime checks.

### Selected-client records

These completed profiles use source `12ac3c0`. Run/snapshot paths below are
relative to `~/.local/state/godot-agent-kit-vm/artifacts/`; raw source, approval
events, client history/JSONL, normalized completed model-facing results, call
arguments and independent screenshots/witnesses remain private there. Each
listed result is correlated by domain request ID and checked against subsequent
model arguments and state/action interpretation—not accepted from a model claim
or raw server object alone.

| Client | Profile | Independent cases | Correlated tool results | Run / snapshot | Client evidence directory |
| --- | --- | ---: | ---: | --- | --- |
| Codex CLI 0.153.4 | workflow + later reads | 26 | 21 | `t002-codex-workflow-5/20261004T234257Z-c43db2b8b12f` | `t002-codex-workflow5-r986gp7t` |
| Codex CLI 0.153.4 | known | 5 | 4 | `t002-codex-known-2/20261004T234748Z-acfe85183ef7` | `t002-codex-known2-rtwcwhvu` |
| Codex CLI 0.153.4 | sources | 19 | 18 | `t002-codex-sources-2/20261004T235845Z-bd4ef5d0773d` | `t002-codex-sources2-z3coog9g` |
| Codex CLI 0.153.4 | bound | 7 | 6 | `t002-codex-bound-3/20261005T000844Z-77b88e7d574a` | `t002-codex-bound3-4x5azl4g` |
| Codex CLI 0.153.4 | observations | 7 | 6 | `t002-codex-observations-2/20261005T001154Z-3772fa1223a6` | `t002-codex-observations2-eegkxpl2` |
| OMP 18.5.1 | workflow + later reads | 25 | 20 | `t002-omp-workflow-4/20261005T002019Z-b87431ed92f8` | `t002-omp-workflow4-imv_gobl` |
| OMP 18.5.1 | known | 5 | 4 | `t002-omp-known-2/20261005T002412Z-c6393de0d6a3` | `t002-omp-known2-noh4qkwa` |
| OMP 18.5.1 | sources | 19 | 18 | `t002-omp-sources-2/20261005T003033Z-534566877a63` | `t002-omp-sources2-kxtrzefy` |
| OMP 18.5.1 | bound | 7 | 6 | `t002-omp-bound-3/20261005T004315Z-0b5ef92b8449` | `t002-omp-bound3-a4ijnnma` |
| OMP 18.5.1 | observations | 7 | 6 | `t002-omp-observations-3/20261005T004807Z-7ff5fe660f7a` | `t002-omp-observations3-76p1lvd8` |

Codex requested **2025-06-18**, accepted the server's sole **2025-11-25**
revision, then discovered the exact three-tool catalog. Its ordinary `on-request`
read-only-sandbox TUI obtained individual **Allow** decisions for fixture edits;
read-only continuations used its normal `exec` path. The full 524,288-byte edit
returned verified success in **7.183628 s**. Submitted and client-returned source
matched SHA-256 `3585e03f3e268dff2d067be620d777a894a5aa87da2bd2945a00fd160dfb8467`.
The client preserved the initial source, appended the requested comment, used the
opaque revision and interpreted the resulting closed/absent state.

OMP requested and selected **2025-11-25**. Its boundary conversation used ordinary
per-call interactive approval for model-authored in-memory JavaScript and MCP
calls. The executed code compared the entire fresh-read source with the
constructed replacement, checked all 524,288 UTF-8 bytes and SHA-256, and retained
the full revision/state plus a compact proof record. It did not substitute a
Godot-file read or shorten the source. Verified edit time was **7.490375 s**.
`t002-omp-bound3-a4ijnnma/programmatic-consumption.json` retains the executed code,
outputs and exact-equality proof; `model-visible.jsonl` explicitly labels the
materialized complete-source value rather than pretending the compact display
contained the entire source.

All **109** relay delivery records correlate with actual tool results and satisfy
their original bounds. Maximum request-forwarding-through-consumed-output times
were **1.226140 s** discovery, **3.025456 s** read and **7.579999 s** edit. This
encloses server accepted-frame processing and delivery; it is not a model-turn
latency claim. No deadline, source bound, client version or approval policy was
relaxed. The failed aggregate Rust invocation and failed client/diagnostic runs
above remain recorded, not recounted as passes.

OMP's workflow used normal `write` approval policy and an owned temporary project
configuration; the file remained unchanged and was removed afterward. Actual
completed model-facing JSON blocks identify `godot_agent_kit` / native / OMP.
Both clients distinguished absent Resource from missing file, null unavailable
source from observed empty source, invalidated historical text from current text,
clean-but-divergent Resource from agreement, and partial inventory from absence.
Ordinary opening and actual selected-client reads preceded independent
Save/reparse/rescan/fresh-runtime checks in each workflow.

### Implementation shape and constitutional review

The adapter keeps framing, dispatch/input checking, schema/catalog construction,
operation projections and connection ownership separate. Transport's roughly
800 production lines share ID, write-delivery, cancellation and supervisor-lifetime
invariants; its larger total includes the deterministic race tests. Keeping that
one connection owner avoids a second scheduler or split ownership. Initialization
delivery is an explicit state, reply storage shares the existing eight-ID bound,
and tool work is never queued. New projection types have private visibility.

The closed channel reuses the existing stock-validator I/O pattern within its
existing owner; native routing, authorization, independent D/R/B or closed-state
verification, stale/human-work refusal, disclosure and retained effects remain
in their original production owners. No source cache, retry, force path, public
lifecycle operation, dependency, native ABI or export change was introduced by
these corrections. T001's native/export and unaffected legacy evidence remains
reusable; the actual MCP binary SHA-256 is identical across `2ddc16d` and the
fixture-only `12ac3c0` correction:
`0fe5afd97282d941c6d630ff43125292d57ac21d817c738e2e9802f2a3aa77c1`.
The native build remains `7203d78018cae3b6b507efbefac688100dd8dd4f8032ab10a6ccfc2f3941ad71`.
The [plan's concrete-cost review](plan.md#t002-bounded-worker-channel-correction)
records why these changes are needed now. No T003/T004 or phase-exit claim follows.

### Post-review correction: operation-specific recovery

The focused correction after `c0f1d7a` changes only adapter failure guidance in
`mcp/output.rs`, on the same T002 branch and PR #65. `HostAfterDispatch` and
`InvalidOutput` previously shared edit-oriented text; the latter also gave
discovery `fresh_read`, while host failure gave read `check_setup`.

Both branches now select the action and terse summary together:

| Operation | Next action | Guidance |
| --- | --- | --- |
| `discover_scripts` | `check_setup` | Discovery result unavailable; check setup before trying again. |
| `read_script` | `fresh_read` | Read result unavailable; read that target again. |
| `edit_script` | `fresh_read` | Edit result unavailable; read the original target before another edit. |

Failure classification is unchanged: invalid arguments, server busy and
pre-dispatch host failure retain their existing category/stage/action; dispatched
host failure remains `host_failure` / `execute`; rejected output remains
`invalid_output` / `deliver`. Discovery/read stay `not_applied`. An edit host
failure stays `unknown`; rejected edit output retains established
`applied` / `partly_applied` / `unknown` certainty rather than becoming not-applied.
Editor refusals remain operation results, not adapter or JSON-RPC errors.

Six focused regressions were added, and the existing edit invalid-output
effect-retention test now also checks recovery guidance. They inspect serialized
`CallToolResult` envelopes and actual terse content, including discovery's absence
of script-read/revision/edit instructions. Four failed before the correction;
all **17 output tests** and **15 MCP process/tool tests** passed afterward.
A throwaway actual-executable smoke initialized/listed tools, removed only its
private executable copy to make same-binary worker launch fail, and submitted
valid discover/read/edit calls. All three delivered the required host-error
application/action/summary through stdio. It launched no Godot process and added
no product fault switch; its process and temporary files were removed.

**Rust validation:** `cargo +1.98.1 fmt --all -- --check` and
`cargo +1.98.1 clippy --all-targets --locked -- -D warnings` passed. The normal
`cargo +1.98.1 test --locked` again failed (exit 101) only the unchanged
`script_open::a_late_compilation_result_cannot_upgrade_a_delivered_known_partial_timeout`
ten-second elapsed assertion at `tests/bridge_boundary/open.rs:807`; 230 library
and 62 bridge-boundary tests passed in that invocation. Under the existing T002
exception, the exact isolated case passed with its original assertion, then the
12 previously unexecuted integration targets passed **153 tests**. Together with
the already-passing 15 MCP tests, this supplies **461 passing tests** across the
documented runs, not a passing aggregate invocation. The default doctest target
(zero tests), `cargo +1.98.1 doc --no-deps --locked` and
`cargo +1.98.1 build --locked --bin godot-agent-kit-mcp` passed. No assertion,
production deadline or exception policy changed; no new scheduling diagnosis is
claimed.

**Relevant-input review:** All **109** accepted selected-client records were
inspected: 16 discovery, 60 read and 32 edit operation results, plus one read
`server_busy` admission error. None traversed either changed failure branch.
Success/operation-refusal projection, catalog descriptions, input/output schema
shape, carrier, revision semantics, execution, transport, cancellation and
closed-worker IPC are unchanged. Fixtures, independent witnesses, runner
identity, client configuration, supported environment and native/ABI/export
inputs are unchanged. The rebuilt Rust executable is not claimed byte-identical
to the earlier binary: relevant successful-path equivalence follows from this
bounded change and the affected projection/process checks. Accordingly, the
Codex/OMP five-profile, real-Godot lifecycle, 512 KiB and T001 native evidence is
reused under [TEST_POLICY.md](../../TEST_POLICY.md#reusing-evidence-across-commits);
no Codex/OMP or VM campaign was rerun. Historical evidence counts remain historical.

**Shape and constitutional review:** One private explicit match on the fixed
operation enum pairs structured and text guidance for the two existing error
branches. No new public API, generic policy/framework, retry or tool-description
change was introduced. This applies Principle IX and FR-022/SC-009 without
weakening effect truthfulness (IV/X, FR-003/FR-011/FR-012) or adding speculative
complexity (XIII). This T002 completion did not itself satisfy T003 or T004.

## T003 execution evidence (2026-10-05)

The adversarial implementation on `task/T003-mcp-adversarial-safety` has
**941 accepted records across 30 focused groups**: 272 preservation, 642
interruption, 14 fixed failure/reconnect transport-driver and 13 representative
legacy records. These are acceptance records, not 941 edits or coding-agent calls.
The final matrix/client source is
`d3d2adbf413c952995816970a62401e7746ef65a`; the production correction is
`19a132d052e7562e6818a577223c2a29785ccfcf`.

Artifacts remain private under
`~/.local/state/godot-agent-kit-vm/artifacts/<run-id>/<snapshot>/`.
Each accepted group's `summary.json` and `provenance.json` retain its exact
commit/archive, fixtures, binaries, native inputs, environment and independent
witnesses. Completed named groups in an earlier aggregate run remain valid;
that run's later failed group is excluded, not promoted to a pass.

| Group | Records | Accepted run ID |
| --- | ---: | --- |
| `preservation-open-history` | 12 | `t003-preservation-1` |
| `preservation-prior-native-history` | 4 | `t003-native-history-after-1` |
| `preservation-revisions` | 18 | `t003-preservation-1` |
| `preservation-identities` | 39 | `t003-identities-3` |
| `preservation-cache-absence` | 3 | `t003-cache-absence-1` |
| `preservation-resource-safety` | 27 | `t003-resource-safety-1` |
| `preservation-session-replacement` | 4 | `t003-session-replacement-1` |
| `preservation-source-context` | 32 | `t003-source-context-2` |
| `preservation-active-reconfigure` | 2 | `t003-active-reconfigure-1` |
| `preservation-equality-races` | 75 | `t003-equality-races-3` |
| `preservation-late-resource-dirty` | 18 | `t003-late-resource-after-2` |
| `preservation-native-entry` | 16 | `t003-native-entry-1` |
| `preservation-selection-privacy` | 17 | `t003-selection-privacy-2` |
| `preservation-shared-slot` | 5 | `t003-shared-slot-2` |
| `interruption-cancellation` | 45 | `t003-cancellation-2` |
| `interruption-original-deadline` | 45 | `t003-deadline-2` |
| `interruption-acquisition-clock` | 18 | `t003-acquisition-clock-2` |
| `interruption-unresponsive-editor` | 15 | `t003-unresponsive-1` |
| `interruption-disconnect-and-signals` | 81 | `t003-shutdown-1` |
| `interruption-editor-channels` | 75 | `t003-editor-channels-1` |
| `interruption-known-effects` | 45 | `t003-known-effects-2` |
| `interruption-newer-work` | 132 | `t003-newer-work-1` |
| `interruption-sticky-denial` | 9 | `t003-sticky-denial-1` |
| `interruption-lost-output` | 63 | `t003-lost-output-1` |
| `interruption-worker-loss` | 27 | `t003-worker-loss-1` |
| `interruption-blocked-output` | 63 | `t003-blocked-output-1` |
| `interruption-malformed-private-reply` | 24 | `t003-malformed-private-1` |
| `transport-failures` | 10 | `t003-transport-failures-1` |
| `transport-reconnect` | 4 | `t003-transport-reconnect-1` |
| `matched-v6-native4-legacy-preservation` | 13 | `t003-native-legacy-after-1` |

### Fault-revealed correction and independently observed effects

At `9fa7e4fca42a72f6d6f932c41e8cda7c253bdf14`, changing an open Script's
Resource to equal-text-but-dirty after preparation allowed buffer removal before
the native guard noticed the dirty Resource. The failing
`t003-late-resource-before/20261005T031633Z-6fa6ee8bdae0` witness records an
empty B and R, unchanged D, and newly altered buffer history. MCP truthfully
reported partial application, but the native precondition should have prevented it.

`script_document.cpp::guard` now checks the existing public per-Resource edited
state before every native stage, including the first removal. The six-stage
regression passes: pre-effect dirtiness refuses without changing D/R/B/history;
later dirtiness stops subsequent native stages and retains truthful known effects.
Clean native edits, prior real Undo/Redo history and the existing local caller
also pass with the rebuilt library. Captures were inspected, including the
unchanged visible `return 47` and preserved unrelated dirty document.

After an already-entered B effect, normal ScriptEditor validation can still copy
that visible B into R after the native request stops. Raw witnesses retain this
transition; they do not force reconciliation or claim zero effects. The bounded
comparison permits only that exact R transition while preserving D/B, dirty flags,
history, identities, roster and selection. It also covers fixture-controlled newer
human B typing; it does not permit replacing an independently changed R with an
arbitrary source. A deterministic regression rejects hidden disk/metadata writes,
buffer/history loss, identity changes, unrelated changes and a third R source.

### Timing, environment and native provenance

Consumed focused-driver maxima were **4.509811 s read**, **4.510608 s discover**
and **9.520930 s edit**, including unresponsive-editor cases. Acquisition handoff
spent 1.160/1.630/1.798 seconds before release and reached open/cached/absent apply
at 4.347/4.995/5.282 seconds; terminal output still arrived at
9.510/9.503/9.504 seconds from the original call. No renewed work budget,
5/10-second limit increase or output-reserve change was used.

Blocked/lost output records unavailable delivery and bounded owned cleanup,
not a consumed-output performance pass or rollback. Real partial writes, R-only
effects, lost acknowledgments, failed timestamp restoration, malformed private
replies and late permission denial were independently witnessed. Unknown/duplicate
cancel IDs did not disturb another owner; shared-slot rejection was not queued.
The existing observation-side occupied-slot refusal remains
`unsupported_observation`, rather than inventing a new legacy outcome.

Environment remains the documented Godot **4.7.2.stable.official.ed1daf0bf**,
macOS **26.6.2 / 25G83 arm64** VM, Rust/Cargo **1.98.1**, Apple clang **21.0.0**
and SDK **26.5**. Engine/template hashes match the T001 record above.
Both rebuilt native artifacts retain API SHA-256
`d0e4c08c03b165156dabe6bfb6a906baf0069189f62035341230a246c86d6986`
and ABI SHA-256
`7d8c0a039d9743eb8ebf88681ae0c641d8d3aa5ffca11081745a84da803e09a1`.

- Production build: `67d0e5a3ea5e7b73019ef159cb8ac664da54fe5de2a944c4b150ecc9460fc05e`;
  library `b1d10e9ed714dacb2ccc4cb614e146d8ea2059003c52813f4acdd8ff91ee76c1`.
- Separate fixture build: `62078cac2c041df08244c03e47c482f87e717a4073d66bb8057ff72afcd48335`;
  library `e1be513a1fd33332489e52aec87b7081d4700f94452aa6ec1423c04cbea1ebc6`.

### Actual selected clients and their limits

Both pinned clients used **gpt-6-astra** and the unchanged structured-content plus
terse-summary carrier. Four fresh profile runs produced **31 independent records**,
**25 correlated model-visible results**, and **two deliberately undelivered edit
results**. Every intentional edit used its actual preceding read revision.

| Client/profile | Independent records | Model-visible results | VM snapshot |
| --- | ---: | ---: | --- |
| Codex 0.153.4, failures | 11 | 10 | `t003-codex-failures-1/20261005T053503Z-bb2b0553d64a` |
| Codex 0.153.4, reconnect | 4 | 2 | `t003-codex-reconnect-1/20261005T054203Z-ffcba96131d1` |
| OMP 18.5.1, failures | 11 | 10 | `t003-omp-failures-1/20261005T054725Z-7d8f5ce7d85b` |
| OMP 18.5.1, reconnect | 5 | 3 | `t003-omp-reconnect-1/20261005T055120Z-a68c84586021` |

Private transcript directories are `t003-codex-failures-1-5h9s9zrv`,
`t003-codex-reconnect-1-rzgpv1x2`, `t003-omp-failures-1-ttl3hcmk` and
`t003-omp-reconnect-1-_bazw0rg` under the same artifact root. They retain actual
terminal items/JSONL and normalized model-facing content, not only raw server
details. Codex history was read only for threads
`01a10a8a-8c76-7443-9591-e11f21148478` and
`01a10a90-ba23-72d1-867f-98a2e0f259f4`.

Both models distinguished null revision/unavailable R dirtiness, zero-effect stale
refusal, a 38-byte partial disk write with divergent R, and an unknown operation
whose later read observed the complete replacement. Neither claimed runtime/class
reload or retroactive verified transaction success. No intentional edit was replayed
or repaired. All failure-profile responses were fully visible; no output spill,
JSON text duplication or source truncation was needed.

Codex used ordinary on-request, one-call **Allow** decisions and read-only sandboxing.
It did **not** automatically reconnect after the dropped public response: both the
edit and immediate read reported `Transport closed`. Closing/resuming the same
conversation with unchanged MCP configuration restored the connection for one
read-only recovery. The model kept the missing edit response distinct from the
new observation. There was no edit resend.

OMP used normal `write` approval mode and preserved `.omp/lsp.yaml`. Its client
internally reconnected and resent the identical edit once, although the model
issued only one edit action. The original verified result was not delivered;
the duplicate reached unchanged revision guards and was refused. The model saw
that refusal, not the hidden delivery loss, then read the replacement and correctly
left its causal attribution uncertain. An automatic resend is not a new intentional
edit or fresh read, and this does not establish exactly-once client behavior.
The kit added no retry, queue, compensation or persistent outcome mechanism.

Real-client delivered-output maximum was **6.260048 s**. Temporary project MCP
configuration was removed after exact ownership comparison, the owned client
worktree was removed, and VM `stop`/`status` confirmed `running: false`.
No host editor was controlled or closed.

### Checks, reuse and completion scope

The task's Rust checks passed: formatting, Clippy, **465 default tests** including
the default doctest targets, rustdoc and locked library/binary/example builds.
The focused output and process/tool checks passed **19** and **17** tests.
That output run followed removal of recovery-summary assertions, incorrectly
classified as incidental wording checks; the focused review correction below
restores their consumer-visible semantic coverage.
The final affected Python/native-build command passed **61 tests**. Actual runner
dispatch, production/fixture native loading, CLI tool calls and visible editor
surfaces were exercised; unit tests are not substituted for those runs.

Relevant-input review retains unchanged T001/T002 direct guards, positive clients,
512 KiB/source-bound, durability and export evidence. The only production change
is the repeated open-native dirty-Resource guard; its affected clean/refusal/history
and interruption paths were newly exercised. Closed mutation, protocol/ABI,
authentication, carrier/schema, stock validation and export isolation are unchanged.
Rebuilt libraries are not claimed byte-identical: the source/ABI review and actual
matched loading/legacy/closed/open behavior establish relevant equivalence.
Fixture corrections affect only their named groups; failed executions remain
excluded. No historical full campaign was replayed merely to obtain a final SHA.

The [implementation-shape and constitutional review](plan.md#t003-implementation-shape-and-constitutional-review)
preserves independent authorities, newer human work, native history, truthful
effect/delivery distinctions, confinement and lean interfaces. This completes
T003 acceptance, not T004's composed A–E/cumulative interaction, Feature 007,
Phase 3 exit or release.

### Focused review correction: recovery-summary alignment

The PR #66 review correction changes only
`mcp-server/src/mcp/output/tests.rs` and this evidence record. The existing
`assert_unavailable_result` helper again checks actual serialized terse `content`
alongside the structured action, using recovery concepts rather than full-sentence
snapshots:

- **Discover:** `check_setup`; summary identifies discovery and checking setup,
  with no fresh/script read, revision, original-target or edit guidance.
- **Read:** `fresh_read`; summary says to read the target again, without edit
  instructions or an implication that an edit was applied.
- **Edit:** `fresh_read`; summary requires reading the original target before
  another edit.

These assertions cover the existing host-after-dispatch and invalid-output cases,
including oversized edit results and malformed effect receipts. Existing
classification, source-redaction and certainty checks remain intact:
discover/read are `not_applied`, dispatched edit host failure is `unknown`, and
invalid edit output retains `applied` / `partly_applied` / `unknown`.
The helper remains test-only; no summary-policy framework was introduced.

**Validation, in order:** `cargo test --locked --lib mcp::output::tests` passed
all **19** focused tests before the broader checks. `cargo fmt --all -- --check`
and `cargo clippy --all-targets --locked -- -D warnings` passed. The normal
`cargo test --locked` **failed (exit 101)** at the previously documented
`script_open::a_late_compilation_result_cannot_upgrade_a_delivered_known_partial_timeout`
ten-second assertion (`tests/bridge_boundary/open.rs:807`); its 232 library tests
and other 62 bridge cases passed. The exact isolated case then passed unchanged;
the remaining 14 integration targets passed **170 tests**.
`cargo test --doc --locked` (zero doctests), `cargo doc --no-deps --locked` and
`cargo build --locked --bin godot-agent-kit-mcp` passed. This supplies **465
distinct passing tests across the recorded runs**, not a passing aggregate run.
It applies the existing concrete timing-case exception under
[TEST_POLICY.md § Rust](../../TEST_POLICY.md#rust) to this test-only correction:
no production or timing-test input changed, no assertion/deadline was relaxed,
and no new scheduling cause is claimed.

A throwaway actual-executable stdio smoke initialized/listed tools, removed only
its owned executable copy to force same-binary worker launch failure, and verified
all three operations' host-error action, application and summary. Its process
exited and temporary files were removed; no Godot process was launched.

**Evidence currentness:** Only these Rust test assertions and evidence prose
changed. Production MCP behavior, descriptions, schemas, native behavior,
fixtures/witnesses, runner/fingerprints, client configuration, supported environment
and T003 acceptance scope are unchanged. Fresh output/actual-caller checks cover
the restored assertion surface. The **941-record VM evidence and Codex/OMP
campaigns remain valid and were not rerun**, under
[the relevant-input reuse policy](../../TEST_POLICY.md#reusing-evidence-across-commits).
T004 remains pending and unselected; this correction stays on the existing T003 PR.

## Owned MCP fixture interfaces

Extend the existing `run_in_vm.py` fixed suite allowlist with `mcp`, retaining its committed-source, cached-build, provenance, ownership, capture and artifact retrieval semantics. Required groups are `transport`, `closed-native`, `closed-lifecycle`, `preservation`, `interruption`, `composed` and `privacy-export`. They are individually runnable; no new workflow engine or requirement for an aggregate historical `all` mode.

Group names are not task IDs: T001 owns direct core/native `closed-native`/`closed-lifecycle` proof; T002 adds transport and the full positive real-agent lifecycle workflow through actual MCP; T003 owns the combined adversarial purpose across separately runnable `preservation`, `interruption` and affected lifecycle cases; T004 owns `composed`, cumulative `privacy-export` and valid-evidence coverage. Use the [explicit scenario/FR/SC ownership tables](tasks.md#requirement-entity-and-contract-coverage); consolidation does not collapse test files or execution groups.

From repository root, on the committed implementation revision:

```sh
python3 godot-addon/tests/run_in_vm.py start
python3 godot-addon/tests/run_in_vm.py run mcp --scenario transport --captures
python3 godot-addon/tests/run_in_vm.py run mcp --scenario closed-native --captures
python3 godot-addon/tests/run_in_vm.py run mcp --scenario closed-lifecycle --captures
```

Run only groups owned/invalidated by the selected task, then the separately required cumulative MCP composition. Expected positive closed results independently establish intended D, same clean loaded R source or confirmed absence, continued absence of B, matching lifecycle/revision and actual validation. A native acknowledgment, disk hash alone or universal refusal is failure.

The `transport` group uses the actual server executable without authorizing an editor mutation for malformed/unsupported cases. It records actual negotiated revision, exact three-tool catalog, result-schema validity, parse/request/tool-error distinctions, UTF-8/depth/size limits, duplicate/unknown IDs, concurrency refusal, stdout purity, blocked-output termination and open-input shutdown. Relevant source/credential sentinels must not occur in stderr or incidental protocol errors.

The fixture controls human state/history/race barriers and independent witnesses, not the MCP tool's success verdict. Use fault artifacts separately from production native artifacts; neither fixtures nor their privileged controls may enter an export.

## Real coding-agent clients

The fixed test-only VM-wrapper commands are:

- `prepare-mcp --revision SHA --run-id ID --artifacts DIR`: prepare an owned synthetic project/session and matched binaries; write source-free target setup, `mcp.json` and `prompt.txt` into the fresh host artifact directory. `mcp.json` is an OMP-native project definition for the fixed relay below, with actual absolute paths/run ID. For an OMP run, attach it as the owned repository checkout's temporary `.omp/mcp.json` only when that path is absent; record ownership/content and remove it after the run only if unchanged. Never overwrite pre-existing config or human changes; use a clean owned checkout of the tested commit when needed. The receipt binds guest inputs/session and the selected host config without authentication secrets.
- `mcp-stdio --run-id ID`: validate that prepared run and relay stdin/stdout through the existing private Tart channel to its fixed guest MCP executable/registry. No arbitrary guest command, endpoint or host project is accepted. Protocol bytes are not wrapped in helper status output. Setup/relay diagnostics are source-free stderr.
- `prepare-durability --run-id ID`: for `workflow` only, require the already delivered primary discover/read/no-op/edit/fresh-read evidence before ordinary fixture opening of cached/absent scripts. Keep the owner alive and emit a read-only continuation prompt to stdout. Feed that prompt to a second invocation of the **same selected real client**, using the same fixed relay/config/run ID. This fixture-only action is not a public MCP operation.
- `finalize-mcp --run-id ID`: require those post-opening actual-client reads, independently verify Save/reparse/rescan/runtime durability, clean up only owned fixtures, then retrieve private evidence. Relay EOF does not destroy prepared projects; it only drains that MCP connection. Finalize before selecting another committed guest source.

T002 positive preparation accepts `--profile workflow|known|sources|bound|observations`
(default `workflow`). Use a fresh run ID/artifact directory and fresh conversation
for **each of those five groups with each selected client**; one group is not full
positive acceptance. `workflow` covers open/cached/absent targets without any exact script
locator in its initial prompt. The separate `known` conversation covers direct
read/edit and stale refusal without discovery; never preload its locator into
the discovery-first conversation. `sources` covers Unicode/observed-empty source
and empty desired source; `bound` isolates the 512 KiB boundary.
`observations` covers dirty/divergent/unavailable/**invalidated**/partial facts.
For `invalidated`, one actual client read is paused at T001's closed-state
acquisition barrier, the fixture performs a same-text disk write changing ctime,
and then releases acquisition. Historical disk source must not be current or
carry a usable revision. Independent pre/post-transition witnesses distinguish
that external write from the read's effects; normal no-effect assertions compare
the read with the post-transition baseline, not with the superseded setup.
The same groups are directly runnable as `run mcp --scenario transport-workflow`,
`transport-known`, `transport-sources`, `transport-bound` and
`transport-observations`. `transport` runs them sequentially. These fixed
transport drivers also perform post-opening reads but **are not real agents**.

T003 adds fixed `--profile failures|reconnect` selections, each in a fresh run and
conversation with each pinned client. `failures` supplies an informative limited
read, a real same-text stale revision refusal, a native partial write and loss of
the genuine private applied reply. The model must interpret the actual structured
facts and freshly read the original explicit target rather than repeat or repair
the mutation. `reconnect` drops one actual public edit response after independent
effect observation; the undelivered server result is not model-visible evidence.
Record any client reconnect/resend separately, require existing stale-state guards
on that duplicate, and correlate the client's subsequent fresh read and explanation.
Neither profile opens a document or weakens approval. After the required recovery
read, finalize without the positive profile's durability continuation. Codex's
observed closed transport required ordinary same-thread CLI resume with unchanged
MCP configuration and a read-only recovery request; do not replay the edit.
OMP's automatic resend and model-visible limits are recorded in
[the actual-client evidence](#actual-selected-clients-and-their-limits).

`transport-failures` and `transport-reconnect` exercise the same fixture/relay with
a bounded protocol driver, **not a coding agent**. `preservation` and `interruption`
contain their separately runnable `preservation-*` and `interruption-*` groups
listed by `run_mcp.py --help`. After a failure, retain completed groups and rerun
only the affected group and remaining obligations. No historical `all` campaign
or T004 composed interaction is implied.

Each consumed public result records its original forwarding-to-delivery clock.
Blocked/lost-output runs instead record delivery unavailable and bounded owned
cleanup, not consumed-output performance or rollback. Independent retained disk,
Resource, complete roster and actual human/native-history witnesses remain the
acceptance boundary; fixture faults cannot become production export artifacts.

The relay is test apparatus, not a product remote transport. Host clients retain their normal model access; editor/core effects remain in the owned guest. This is the minimal adaptation needed to combine real clients with the existing VM policy.

After the primary `workflow` client invocation, run
`python3 godot-addon/tests/run_in_vm.py prepare-durability --run-id "$RUN_ID" > "$ARTIFACTS/durability-prompt.txt"`.
Invoke the same selected client again with that prompt and the unchanged relay
configuration, for read-only consumption of now-open D/R/B. Only then run
`python3 godot-addon/tests/run_in_vm.py finalize-mcp --run-id "$RUN_ID"`.
For the other profiles, finalize directly after the client exits. Both controls
can reach an idle live relay without waiting for client EOF: the existing host
lock is shared only for relay/prepare-durability/finalization, exclusive for setup
and source-changing operations. Guest source/provenance and active-owner checks
remain required. Preserve failed evidence as well as successful evidence.
The guest summary reports MCP behavior, not model-visible client acceptance;
correlate call records with actual completed tool results and subsequent model
actions in both workflow invocations. T004 alone owns full composed conversations.

Example preparation from the root of a clean, owned host checkout of the tested repository commit (`REPO` is that checkout, not a human Godot project). The OMP preparation must preserve `.omp/lsp.yaml` and normal repository-root launch semantics:

```sh
REPO="$PWD"
REVISION="$(git rev-parse HEAD)"
RUN_ID="mcp-agent-$(date -u +%Y%m%dT%H%M%SZ)"
ARTIFACTS="$(mktemp -d "$HOME/mcp-agent.XXXXXX")"
python3 godot-addon/tests/run_in_vm.py prepare-mcp \
  --revision "$REVISION" --run-id "$RUN_ID" --artifacts "$ARTIFACTS"
```

The generated `mcp.json` uses this documented OMP shape; the example paths/ID are illustrative, while preparation writes their actual values:

```json
{
  "mcpServers": {
    "godot_agent_kit": {
      "type": "stdio",
      "command": "python3",
      "args": [
        "/owned/godot-agent-kit/godot-addon/tests/run_in_vm.py",
        "mcp-stdio",
        "--run-id",
        "owned-run-id"
      ],
      "cwd": "/owned/godot-agent-kit",
      "timeout": 15000,
      "instructions": false
    }
  }
}
```

The prompt gives the owned project/session, requested change and human intent, not private architecture coaching. The server contributes no long instruction block; static tool descriptions and structured outcomes must suffice.

After the generated definition is attached as temporary project `.omp/mcp.json`, launch the actual installed OMP from that repository root:

```sh
OMP_MCP_TIMEOUT_MS=15000 OMP_MCP_REQUIRE_READY=1 \
omp --cwd "$REPO" --mode json --no-session --approval-mode write \
  < "$ARTIFACTS/prompt.txt"
```

This uses ordinary host-owned authentication and OMP's supported `write` approval mode, not yolo/auto-approve. MCP tools have the write tier; explicit user/tool denies and prompt policies still apply. A prompt-required headless call fails closed: use the ordinary interactive OMP with the same project config and non-bypass policy to obtain the required human approval/private session transcript, rather than loosening that policy. Do not use `--no-lsp` or modify `.omp/lsp.yaml`. OMP's `--config` accepts settings overlays, not a replacement MCP server file.

Project MCP discovery is additive to user/profile/imported definitions; this command does not claim to disable them. Record the actual server/config origin and verify the `godot_agent_kit` relay owns `mcp__godot_agent_kit_discover_scripts`, `mcp__godot_agent_kit_read_script` and `mcp__godot_agent_kit_edit_script`. Do not mutate global personal configs to manufacture a clean result. Any other tool that substitutes shell/file/private-bridge access for a product operation invalidates the workflow proof; ordinary consumption of a client-owned artifact containing an already-returned MCP result is not such a substitution.

Keep OMP JSONL stdout and stderr separate in private evidence. `message_end` carries completed messages; correlate actual tool execution/call/result events and subsequent model actions with relay/server records and independent guest witnesses. Raw `details.structuredContent`, a catalog listing or an assistant's unsupported success claim is not proof the model obtained the required fields.

The guest's private `protocol.jsonl` records the actual requested/selected revision,
version discovery and returned static catalog without client metadata or raw
requests. `delivery.jsonl` correlates consumed-output timing by domain request ID.

Codex's observed exec/config interface permits read-only checks with normal host
authentication and no user-configuration changes. GPT-5.5 uses direct tools. An
earlier Homebrew companion stalled before `--help`; a private installation of the
official **0.153.4** npm distribution started normally and ran the **gpt-6-astra**
code-mode workflow. The companion's SHA-256 matched and signature verification
passed; no binary, quarantine or global trust setting was changed. OMP's global
command had advanced to 18.6.0, so acceptance likewise uses a private **18.5.1**
installation rather than changing global packages. Substitute those actual pinned
executable paths for `codex`/`omp` below and retain their version/config provenance.

```sh
codex exec --ignore-user-config --ephemeral --sandbox read-only --json --model gpt-5.5 \
  -c 'mcp_servers.godot_agent_kit.command="python3"' \
  -c "mcp_servers.godot_agent_kit.args=[\"$REPO/godot-addon/tests/run_in_vm.py\",\"mcp-stdio\",\"--run-id\",\"$RUN_ID\"]" \
  -c 'mcp_servers.godot_agent_kit.startup_timeout_sec=10' \
  -c 'mcp_servers.godot_agent_kit.tool_timeout_sec=15' \
  - < "$ARTIFACTS/prompt.txt"
```

Use a fresh prepared run for the second client and retain actual tool-call/result transcripts. Client timeout exceeds the server contract so it does not hide the server's deadline result. Deny direct file/shell/private-bridge substitutions; any such substitution invalidates that agent workflow proof. The guest fixture is not a host workspace file path. Do not use global permission-bypass flags. The fixture harness must separately assert state; model claims are not acceptance.

In the exercised environment, `exec` refused an edit because its approval policy
was `never`. Full mutation acceptance therefore uses the ordinary interactive
CLI with `--ask-for-approval on-request --sandbox read-only --model gpt-6-astra`.
Obtain human authorization for the owned fixture edits, inspect each requested
target/change, and select **Allow** for that call. Do not select **Always allow**
or use a permission-bypass flag.

Keep directory trust invocation-scoped too. The tested TUI accepted a top-level
inline TOML override, `-c 'projects={"/absolute/owned/checkout"={trust_level="untrusted"}}'`;
include the canonical checkout path and, for a worktree, its displayed primary
repository root as applicable. This deliberately leaves project-local
configuration/hooks untrusted and leaves the normal MCP approval prompts enabled.
Do not accept a prompt that would persist trust in personal configuration.
Quoted path segments in dotted `-c` keys did not set the intended entry in the
exercised client; the inline table did.

`--no-alt-screen` retains the ordinary TUI surface. Its official
`CODEX_TUI_RECORD_SESSION=1` and `CODEX_TUI_SESSION_LOG_PATH` controls can retain
private approval-event evidence. Preserve the selected thread's completed
`mcpToolCall` and `agentMessage` items from the client's normal local history,
without reading unrelated threads or authentication files. Correlate those
results, subsequent model arguments/state interpretations and the independent
guest witnesses; a raw result or a success claim alone still is not acceptance.

For T002, **each client** must actually connect, discover the exact three-tool catalog and perform `discover → read → edit → fresh read` for an eligible open script, cached-clean-R closed script and confirmed-absent-R closed script. Also cover known-target editing without mandatory discovery, unchanged intent and representative informative dirty/limited/non-editable reads. Model-visible source/revision/state and lifecycle-preserving independent postconditions are required, not deferred to a separate acceptance PR. Later opening of a successfully closed-edited target witnesses durability only. T003 adds adversarial/refusal/partial/unknown result interpretation; at least one real-agent conversation supplies T004's full composed A–E. If selected-version negotiation or required result visibility fails, report the actual limitation before claiming support; do not fall back to a custom SDK client or silently drop OMP.

### Revision and result-carrier checks

Exercise the familiar public flow: read the explicit target, use `result.source` and relevant `result.state`, then submit only its non-null `result.revision`, the selectors and replacement. No full read/observation object is accepted as edit input; ordinary informative reads with no safe revision remain useful.

Focused deterministic/boundary and real-editor evidence must establish:

- Reacquiring unchanged eligible state produces the same revision despite new request/collection IDs and intervals. Meaningful source, target/session/document identity, same-text buffer-version or lifecycle changes invalidate it.
- Existing closed same-text file-revision, Resource branch/identity/dirty and close-epoch ABA cases invalidate the public revision as well as the internal basis. The native closed design and its positive coverage are unchanged.
- Edit freshly authenticates/resolves/acquires before comparison, freezes exactly the matching capture and constructs the internal request from that evidence. A later race still reaches the existing guards; no refreshed basis, branch switch or clock renewal rescues stale intent.
- Missing/malformed/wrong-target/stale revisions fail without mutation. Even a fabricated matching precondition cannot bypass current dirty, permission, confinement, Resource, lifecycle or validation checks. After possible effects, the next intentional edit requires a fresh read, not replay.
- Public state preserves exact source/provenance, empty versus unavailable, dirty/divergent/invalidated authority facts and limitations without private expected-state payloads or redundant agreeing source copies.

For **each selected real client**, demonstrate model-visible use of source, opaque revision, relevant state and failure/next-action facts from the authoritative structured result—not merely a raw SDK event or displayed summary. Include multiline/Unicode/empty source, informative limited reads and partial/unknown edit outcomes within the applicable existing bounds. Do not silently truncate to pass. Start with the selected structured-content plus terse-summary carrier; do not preload a full JSON text copy by assumption. If a client requires it, record the failing smaller carrier and the sufficient full-text fallback, update the carrier decision with its compatibility/context cost and preserve the same authorized object/schema. Unavailable model access is not a pass and does not justify silently dropping that client.

Codex's inspected conversion prefers the structured object; OMP's inspected bridge renders it into model-facing JSON alongside the terse summary. Keep that single server carrier. OMP can spill large output to client-owned artifacts: record preview/complete-result recovery and show actual use of required source/revision/state/error fields, rather than assume raw transcript visibility equals model visibility. No unconditional JSON duplication or silent source truncation is a remedy for client output caps.

OMP's stdio cancellation is client-local abandonment and does not send `notifications/cancelled`; use the focused protocol driver for actual wire-cancellation cases, while retaining required real-agent workflows and outcome interpretation. OMP also has a source-established reconnect/resend path after retryable EOF/closure. T003 records actual duplicates and survivor state, verifies unchanged product admission/stale/lifecycle enforcement and never counts an automatic resend as a new intentional edit/fresh read. Neither client-local timeout nor reconnect proves rollback or safe replay. The product adapter's no-retry/no-queue/no-compensation contract remains unchanged.

## Acceptance coverage

| Group / evidence | Owning task(s) | Exact approved scenarios | FR / SC coverage and required observations |
| --- | --- | --- | --- |
| Transport and both independent clients | T002; T003 failure extensions | US1.1–US1.4 | FR-001–FR-004, FR-012–FR-017, FR-021; SC-001, SC-004, SC-006–SC-007. Negotiation, unsupported requests, editor-unavailable versus connection success, bounded errors and no incidental disclosure/effect. |
| Qualitative interface review and actual agent use | T002; T003 failure guidance; T004 composition | US1.5 | FR-002, FR-022; SC-009. Agent chooses discover/read/edit, uses source/revision/state, submits revision/replacement without internal evidence knowledge, and interprets structured results/actions. No redundant warnings or numerical concision threshold; actual carrier visibility and any necessary fallback cost are recorded. |
| Closed-native and closed-lifecycle | T001 direct proof; T002 positive public workflows; T003 adversarial cases | US2.1, US2.3–US2.5, US5.5 | FR-005–FR-011, FR-014, FR-018–FR-020; SC-002–SC-006, SC-008. Positive cached/absent R, current basis, no opening, no-op recognition, missing/dirty/divergent R refusal, namespace/cache/lifecycle races, independent closed postconditions and later opening durability. |
| Preservation and stale state | T002 positive open workflow; T003 combined safety increment | US2.2, US3.1–US3.4, US3.7 | FR-004–FR-010, FR-018–FR-020; SC-002–SC-003, SC-008. Open guarantees, dirty/equal-dirty human state/history, same-text versions/file revisions, closed→open→closed epoch, target/session replacement, confinement and unobservable evidence. |
| Interruption and overlapping calls | T003 combined safety increment | US3.5–US3.6, US4.1–US4.4 | FR-003–FR-004, FR-007, FR-010–FR-017, FR-020; SC-003–SC-004, SC-006–SC-007. Actual pre/post-authorization cancel/EOF/timeout/output-loss/disable, preserved newer work, exact effect certainty, no late effect after proven refusal, no adapter queue/replay or cross-call cancellation. |
| Real-agent MCP composed A–E | T004 | US5.1–US5.4 | FR-019–FR-020; SC-001–SC-005, SC-008–SC-009. A/B clean-open edit and dirty protection; C genuine native Undo/Save/Redo/Save for open edits; D ordinary fixture close/reopen and permitted durability; E twenty fresh-basis successful MCP edits covering open/closed with at least three dirty and three stale refusals. |
| Privacy/export and existing-contract review | T001–T003 direct obligations; T004 cumulative coverage | US1–US5 relevant error/selection paths and all privacy/export edge cases | FR-016–FR-018, FR-020–FR-021; SC-007–SC-008. Authorized results only, source-free catalog/logs/errors, sticky disclosure denial after causal failure, enabled/disabled/hook-only exports, all existing local contracts unchanged. |

This covers all **26 scenarios, FR-001–FR-022 and SC-001–SC-009**. Cross-cutting routing, timing, privacy and outcome meaning apply to every relevant group, not just one named row. Closed positive cases supplement A, never replace its open-buffer proof.

For C, native Undo/Redo must independently reverse/reapply real open-document source with prior history intact; no second source-edit call substitutes for history. For D, later opening is a durability witness after already-verified closed editing, not the method of obtaining success. Ordinary reparse/rescan/fresh runtime must retain intended source without reconciliation; loaded class metadata is not assumed hot-reloaded. E uses a fresh trusted read/revision for every intentional edit; losing a response requires another fresh read rather than replay.

## Evidence reuse and completion

Reuse [Phase 1 accepted evidence](../../PHASE_1_EXIT.md#evidence-reuse-and-currentness), [Phase 2 ownership/evidence](../../PHASE_2_EXIT.md#evidence-currentness), [Feature 002 acceptance](../002-edit-open-gdscript/quickstart.md#14-t005-cumulative-acceptance-2026-09-29) and [Feature 005 cumulative review](../005-close-project-gdscript/quickstart.md#t003-execution-decision-and-evidence-review) only after relevant-input/provenance review.

The private v6/native revision-4 cutover changes registration/negotiation/owner inputs. Prove affected capability/refusal/slot/cancellation/loading/export compatibility and representative existing open-edit/observation/discovery/lifecycle preservation. If mutation-relevant code or a witness changes, rerun its affected cases. A rebuilt library needs demonstrated relevant equivalence or those affected reruns; unchanged filenames/version strings alone are not equivalence. No blanket historical all-suite campaign is imposed.

Retain exact committed source, engine/platform, SDK/client versions, dependency lock, native/ABI/build provenance, fixture/witness identity, negotiated revision, admission/delivery timings, independent authority/history/lifecycle records and owned captures. Keep raw source-bearing research/acceptance artifacts private and outside Git. Failure/partial/interrupted evidence is not a pass. Record reused/new/invalidated/not-applicable evidence against the matrix before completion.

After an owned run, retrieve artifacts and stop the VM using the existing wrapper; close only owned processes/projects. A complete feature still requires approved implementation tasks, their actual acceptance and shape reviews. This planning guide does not satisfy those future gates.

## Planning evidence only

Planning ran static documentation checks, repository/LSP and released-source research, installed client version/help checks, VM status/start/stop, and the scoped [closed-source mechanics probe](research.md#owned-stock-editor-observations) with visual inspection and fresh-runtime persistence witness. The two incomplete probe attempts remain excluded from the completed observation. Those planning-only checks did not establish MCP interoperability, product-suite/native-build acceptance, A–E coverage, release or Phase 3 exit; later implementation evidence is recorded above.

**Historical public-contract probe:** The [earlier correction](research.md#7-public-contract-correction-evidence) inspected Codex result conversion and attempted an isolated synthetic carrier probe with the then-selected Codex + Claude pair. Expired Claude OAuth and an unavailable Codex code-mode tool path prevented model-visible results. Those failures remain excluded; Claude is no longer a current acceptance requirement. The [Codex + OMP planning correction](research.md#8-codex-and-omp-client-correction-evidence) was source/configuration research, not runtime acceptance. [T002 positive workflows](#t002-execution-evidence-2026-10-05) and [T003 adversarial interpretation](#t003-execution-evidence-2026-10-05) are now complete. T004 composed/cumulative acceptance remains pending.
