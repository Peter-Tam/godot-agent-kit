# Implementation Plan: Observe Live GDScript Editor State Safely

**Git branch**: `main` (no feature/task branch created) | **Feature identifier**: `001-observe-gdscript-state` | **Date**: 2026-09-26 | **Spec**: [spec.md](spec.md)

**Input**: `specs/001-observe-gdscript-state/spec.md`

**Status**: Phase 0 research and Phase 1 design prepared for review. The specification is ready for planning; neither it nor this plan authorizes implementation. Stop after design; derive tasks separately. The setup helper returned `BRANCH=001-observe-gdscript-state` from its feature pointer, not from Git; the actual checkout remains `main`.

## Summary

Provide one-script, observation-only inspection of an explicitly selected live Godot editor/project. Independently read disk (D), already-loaded GDScript source (R), live CodeEdit text (B), document-open state, and attributable editor dirty state. Return exact text, comparisons, limitations, and bounded structured outcomes without changing files, editor selection, documents, or history. A complete dirty/divergent observation is valid information, not a mutation conflict or permission to edit.

Use one Rust package with protocol-independent evidence/classification semantics and a small local caller, plus a thin Godot EditorPlugin over an authenticated loopback bridge. A real GUI-editor feasibility experiment established the positive clean/dirty/non-selected-buffer API path on the exact candidate Godot version. Full implementation acceptance, CI, and support remain gated.

## Technical Context

**Language/Version**: Rust 1.98.1, edition 2021; GDScript on exact candidate Godot `4.7.2.stable.official.ed1daf0bf` (full engine hash `ed1daf0bf001b61586d9930840f2f1394092c079`). No MSRV promise. Python 3.10+ standard library for acceptance orchestration only. The existing machine's Rust 1.69.0 is not the selected implementation toolchain.

**Primary Dependencies**: `serde =1.0.229` with derive, `serde_json =1.0.151`, `cap-std =4.0.3` without default features, and `ring =0.17.14` without default features and with `std` for T002's mutual authentication; Rust standard library networking/process/deadline primitives; public Godot ScriptEditor/CodeEdit/ResourceLoader/TCPServer/Crypto/EditorExportPlugin APIs. No MCP SDK or transport, async runtime, native extension, database, or CLI framework. See dependency rationale, licenses, provenance, and audit obligations in [research.md](research.md).

**Storage**: No database or stored source snapshots. Owner-private session descriptors outside the project are confined to the tool's own private state location and contain only minimal endpoint/identity/token metadata for discovering and routing to the intended local editor session. This narrowly authorized metadata access grants no access to outside-project source content or unrelated filesystem data; existing authentication/privacy requirements apply. Observation values exist only in request memory, bounded worker transport, and the intentional result. Tests retain only explicit synthetic-fixture evidence; incidental logs never contain source or tokens.

**Testing**: Rust built-in unit/integration tests for semantic invariants and boundaries; isolated Python-driven real-Godot GUI acceptance for source/dirty/session claims. Exact-version export generation and artifact/runtime inspection prove tooling isolation. No test dependency or permanent mock editor framework is selected. [quickstart.md](quickstart.md) defines the planned executable acceptance entrypoint and all scenario groups.

**Target Platform**: Initial candidate is macOS arm64 (`aarch64-apple-darwin`). Native CI targets macOS 15 arm64; real-editor evidence targets macOS 26.6.2 arm64 with a GUI-capable trusted CI/release runner. The planning smoke ran on the latter. Neither an API reference nor native compilation earns OS/editor support; record actual matrix evidence before advertising any supported combination. No Linux/Windows or other Godot-version claim.

**Project Type**: Reusable observation library, minimal local JSON-result caller/worker, and editor addon. This is an observation subset of roadmap Phase 1, not an MCP surface, generalized transaction framework, broad editor orchestrator, or distribution feature.

**Performance Goals**: Every controlled local observation request, including an unresponsive editor, returns a structured terminal result within five seconds. One monotonic deadline begins before target resolution; collection stops at 4.5 seconds to reserve bounded finalization time. No throughput, arbitrary project-size, concurrent-load, or hard-real-time guarantee. See explicit frame/source limits in the bridge contract.

**Constraints**: Local-only, authenticated script observation confined to the selected project; no force-load/open/select/save/write/reload/reparse/rescan/UndoRedo/runtime actions. No source normalization or D/R/B substitution. No source-derived dirty state. Session/document identity and before/after witnesses prevent detected changes from becoming complete snapshots. Uncertainty remains visible. Source access fails closed for ambiguous/denied targets.

**Scale/Scope**: One script and all standard facts per observation. Multiple editor sessions exist only to select/refuse correctly. External `.gd` documents are the positive supported-observation target; already-identifiable built-in GDScript may expose R/B with D explicitly unavailable. Unresolvable built-in identity is a structured unsupported result, never permission to load a scene. Twenty sequential observations form the non-interference acceptance gate.

## Constitution Check

**Initial gate, before Phase 0: PASS for research.** The initial plan explicitly identified API/version/dirty feasibility, dependency/toolchain, session/security, deadline, testing, and export unknowns. It proposed no weakening of any invariant. Research resolved those decisions, including a positive real-editor clean/dirty experiment; no design clarification remains open.

**Post-design gate: PASS for design, not a release/support claim.** The contracts and acceptance obligations below preserve every applicable constitutional requirement. Required implementation tests are not reported as executed.

| Constitutional gate | Design/review evidence and disposition |
|---|---|
| I — Independent D/R/B | Separate typed observations, exact source preservation, invalidation/availability states, three independent comparisons. No disk-only coherence or atomic snapshot claim. |
| II — Human work | Getter-only observation; attributable `get_unsaved_files()` evidence; no equality-derived clean state, repair, overwrite, or force option. |
| III — Native semantics | Public Godot APIs own R/B/dirty. No mutation or UndoRedo capability; native mutation-route requirements remain reserved for later features. |
| IV — Verified outcomes | Acceptance/collection are not complete observation. Known target, all applicable facts, and no detected invalidation are required. Outcome precedence preserves refusal/interruption/partial evidence. No mutation-success state exists. |
| V — Least privilege | Explicit project, mutually authenticated loopback, private descriptors, project-rooted D, unsafe-path refusal, and no telemetry, arbitrary execution, remote access, or outside-project content access. Fresh session-bound HMAC proofs authenticate both peers without transmitting the secret, including after stale-descriptor port reuse. The narrow session-metadata permission is confined to the tool's own private state location under FR-016. Worker execution is fixed internal infrastructure, not a caller command. |
| VI — Real-editor gates | Positive clean/dirty/non-selected feasibility observed. All 21 spec scenarios, edge cases, sequential reads, and exact-version CI remain implementation gates. Mutation A/B/D/E and applied-edit Save/reparse/rescan/runtime durability are inapplicable because no edit is applied; C is inapplicable because no UndoRedo support is claimed. This earns no mutation Phase 1 exit guarantee. |
| VII — Protocol independence | Local caller → common observation core → Godot integration. Core has no MCP, JSON, socket, or Godot-object dependency; transport DTOs convert at the boundary. No second safety model. |
| VIII — Tooling isolation | EditorPlugin only, no gameplay authority/autoload; export hook plus production-preset exclusion, verified with enabled/disabled exports and actual exported launch. |
| IX — Small surface | One observation operation and registry bootstrap; explicit source-free selector feedback, no broad tool catalog, session control, or MCP exposure. |
| X — Actionable diagnostics | Stage/surface-specific outcomes distinguish unavailable editor, disconnect, timeout, attribution failure, missing/invalid targets, and denied access. Diagnostics exclude source and tokens. |
| XI — Independent implementation | Public APIs/pinned source and released manifests; dependency license/provenance review recorded, actual lockfile advisory/license review required before acceptance. No copied third-party implementation. |
| XII — Quality before scope | Safety/identity/deadline invariants have deterministic integration coverage and real-editor proof obligations. Missing observability is a limited result, not a substitute for the required positive path. |
| Architecture/compatibility | Existing `mcp-server/` / `godot-addon/` ownership retained; exact candidate version, deliberate schema version 1, documented migration policy. |
| Workflow | This command writes design artifacts only. No implementation tasks are completed, no `tasks.md` is created, and no PR/commit is made. Later approved implementation selects exactly one task and delivers one PR, then stops. |

**No exceptions:** A failed attribution, confinement, positive-observability, or export gate blocks the affected implementation. Approval cannot override safety or turn a limited observation into complete. If implementation evidence contradicts the pinned API assumptions, revise this plan/specification through review rather than silently broadening privileges or weakening a gate.

### T002 implementation compliance

T002 preserves the source-free session boundary; it adds no observer/worker,
source collector, mutation, MCP, or runtime operation. Core evidence semantics
remain separate from registry, framing, cryptography, and Godot APIs.

The macOS implementation needs ownership, filesystem identity, and ACL metadata
that Godot's filesystem API does not expose. Bootstrap/cleanup therefore invoke
only fixed `/usr/bin/id`, `/usr/bin/stat`, and `/bin/ls` through `OS.execute`,
without a shell, configurable executable, peer-supplied arguments, or source/token
arguments. Inputs are validated local metadata paths; outputs are captured, not
logged. This is fixed internal metadata inspection, not an arbitrary-execution
capability. Rust inspects ACLs on borrowed open descriptors through a small,
documented Darwin FFI boundary. Missing observability refuses access; there is no
mode-only fallback on another platform.

Both boundaries reject access-grant ACLs, unsafe ownership/modes/ancestry, and
in-project registries without repair. Deny-only ACLs remain valid. The private
directory protects the new temporary descriptor during Godot's safe-save write;
the closed file must pass owner/type/0600/ACL checks before atomic advertisement.
The export guard runs before GDScript remapping, and production presets provide
independent exclusion when the plugin is disabled.

These decisions implement constitution V/X/XI/XII without relaxing I–IV or VII–IX.
Mutation A–E and applied-edit durability remain inapplicable: no mutation or
UndoRedo capability exists. Native, live-session, export, and exact-tool evidence
is recorded in [quickstart §2.2](quickstart.md#22-t002-source-free-boundary-evidence-2026-09-26)
and [research](research.md#t002-resolved-authentication-dependencies-and-tool-provenance-2026-09-26).
No source-observation or additional-platform support is earned by these checks.


### T003 implementation compliance

T003 implements only the approved execution/disk/codec boundary. The local caller
supervises its own same-binary read-only worker over inherited private IPC; it
accepts no executable, endpoint, force, or deadline override. SIGINT/SIGTERM set a
lock-free cancellation flag through a documented Darwin signal boundary. Only the
owned worker is killed/reaped, without joining blocked filesystem acquisition.

Constitution V/X: unique authenticated selection precedes source; the selected
channel retains request/project/session binding. Capability-rooted D acquisition
and recheck refuse unsafe roots, symlinks, substituted identities, modes or ACLs.
Framed DTOs enforce bounds and strict attribution; diagnostics contain fixed safe
metadata. I/II/IV/VII/XII: independent D remains separate from editor evidence and
protocol-independent classification, no source is copied between authorities,
and individually validated partial evidence survives interruption. The additive
`Recheck::Partial` representation retains detected changes without a false claim
that all checks completed. It preserves the existing version-1 wire semantics.

No addon collector, Godot mutation, MCP, gameplay or export payload is added.
Mutation A–E and applied-edit durability remain inapplicable; T002's unchanged
exported-addon exclusion remains the applicable boundary. T003 does not earn
positive R/B/dirty or US1 support. Actual native/executor evidence and the
unchanged dependency baseline are recorded in [quickstart §2.3](quickstart.md#23-t003-executor-boundary-evidence-2026-09-26)
and [research](research.md#t003-executor-provenance-2026-09-26).

### T004 implementation compliance

T004 preserves I/II/III/IV/VII: existing public editor getters independently supply
R, actual attributed CodeEdit B, open state and unsaved evidence; the common core
alone classifies completeness. No product mutation, document opening/selection,
forced resource load, Save, reparse/rescan or history operation is added. Array,
identity, source/version and dirty rechecks withhold completeness on detected
change. Unsigned object-ID strings and same-clock interval validation preserve
actual attribution without numeric truncation.

V/X/XII: authenticated unique selection remains source-free; the addon checks
scope around getter passes and the existing Rust directory capability owns D.
Specific source-free scope refusals suppress earlier samples. Per-source limits,
request-local reference cleanup and bounded caller supervision remain in force.
The test driver uses independently sampled synthetic authorities and owned-window
evidence, not the product collector as its oracle.

VIII/XI: the expanded addon/driver remains under existing export exclusions and
requires renewed artifact/actual-export regression evidence. No dependency is
added. The protected manual live workflow is sourced from `main`; the hosted gate
validates the explicitly selected immutable tested revision and main-only
environment policy before a GUI job is eligible. It has no PR trigger, no
persisted checkout credentials, and no repository secrets. The dedicated GUI
runner must be clean, isolated, ephemeral and single-job, not a persistent human
workstation. Missing provisioning is not a passing CI/support gate.

Mutation A–E and applied-edit durability remain inapplicable because no mutation
or product UndoRedo capability exists. T004 is the US1 development increment,
not completion of Feature 001 or roadmap Phase 1. Native/live/export evidence and
limits are recorded in [quickstart §2.4](quickstart.md#24-t004-clean-open-evidence-2026-09-26).

### T005 implementation compliance

T005 is limited to US2's document-attributed dirty/divergent observations and
request-local change detection. I–IV/VII require independent D/R/B reads,
document-specific dirty evidence, preservation of human text and history, and the
existing common reducer's truthful complete/limited outcomes. Same-document
changes invalidate affected facts; replacement invalidates identity-dependent
facts. Closure during collection cannot become a fabricated closed snapshot.
No observer mutation, forced load/open/select, Save, synchronization, or retry
is permitted.

V/X/XII retain the authenticated local/project-confined boundary, source limits,
deadline and source-free diagnostics. Negative acceptance restrictions may
withhold observability only in disposable fixture code; production gains no
bypass flag or fake positive evidence. VI/VIII require real GUI witnesses for
the three US2 groups, non-interference checks, and renewed export exclusion and
actual exported-app evidence. Fixture preparation remains separate from product
collection. IX/XI introduce no additional product operation or dependency.

The protected live workflow may select these named groups on a reviewed revision
under its existing trust gate; this does not authorize untrusted GUI execution
or establish an executed CI/support claim. Mutation A–E and applied-edit
durability remain inapplicable because T005 adds no mutation or product UndoRedo
capability. T006–T008 and roadmap Phase 1 exit gates remain outside this task.

Implementation review: the collector remains observation-specific; shared session,
framing, worker and core-reducer responsibilities are reused without a parallel
implementation. Fixture helpers belong to the acceptance boundary and introduce
no task-named product API or speculative production extension mechanism. All
applicable local native, real-GUI, privacy and export evidence is recorded in
[quickstart §2.5](quickstart.md#25-t005-dirty-and-changing-document-evidence-2026-09-27).

### T006 implementation compliance

T006 covers US3's source-bearing session selection and interruption behavior.
I/II/IV/VII require every retained D/R/B/dirty fact to keep its original
request/project/session/document attribution. Known loss invalidates live
currency; failed or absent final rechecks cannot produce complete/not-open
success. No replacement session, automatic observation retry, forced editor
action, or second classification model is permitted.

V/X/XII retain mutual authentication before unique selection and source access,
private bounded metadata, project-confined D, source-free denial/ambiguity,
bounded frames and a caller deadline independent of the editor/worker. Fault
injection belongs only to owned fixtures; it may interrupt or withhold evidence,
never manufacture positive R/B/dirty facts or expose a production bypass.
VI/VIII require real GUI provenance/partial-stage evidence, independently
witnessed human-state preservation, and renewed production-export exclusion.

The shared routing, bridge, worker and GUI-runner infrastructure keeps its
responsibility-based ownership; no feature/task identifier enters product APIs
or wire fields. Observation-specific assertions remain in the existing acceptance
driver. No dependency, product operation, schema version, or support claim is
added. The protected workflow exposes named groups under its unchanged reviewed
main/environment gate, not automatic privileged PR execution.

Mutation A–E and applied-edit durability remain inapplicable: this task applies
no mutation and claims no product UndoRedo. T007/T008 and roadmap Phase 1 exit
gates remain pending. Completion requires the task's native, actual GUI,
privacy, and export evidence; workflow configuration alone is not executed CI.

Implementation review: no production change or duplicate session/security/worker
infrastructure was needed. Native transition tests remain beside their durable
boundaries; the observation-specific multi-editor oracle stays in the shared
observation driver. Negative fixture controls add no product extension mechanism.
Names/ownership remain responsibility-based, with no speculative generalization.
All required local native, US3 GUI, privacy and export checks pass; exact scope,
timings and the remaining full-feature/CI gates are recorded in
[quickstart §2.6](quickstart.md#26-t006-routing-and-interruption-evidence-2026-09-27).

### T007 implementation compliance

T007 implements US4's closed/invalid/partly observable document boundary and
independent source limits. I/II/IV/VII require actual cached or open GDScript
identity, independent D/R/B, explicit unknown open state, and retention of
unrelated evidence. Closed B/dirty alone are not applicable; syntax-invalid and
empty GDScript remain valid observation subjects. The common reducer retains
version-1 outcome and interruption precedence.

V/X/XII: a built-in locator is validated against the same grammar at both
adapters, and its container remains project-confined before/after collection.
Only existing native editor/cache identity permits R/B observation; no scene
loading, parsing, container-source read or editor selection manufactures facts.
Unresolved built-in identity is unsupported. Per-source limits change only
availability, never authorize operation-wide refusal or discard earlier facts.
Negative fixture restrictions may withhold observability, not create evidence.

VI/VIII require both US4 GUI groups, independent non-interference/privacy
witnesses and renewed enabled/disabled/hook-only export inspection and actual
launches. The protected workflow merely adds named groups under its unchanged
reviewed-main/environment gate; local verification is not executed trusted CI.
No dependency, schema, product operation or supported-version claim is added.

The observation collector and assertions remain capability-specific; shared
framing, routing, confinement, supervision and classification retain their
durable ownership. No duplicate infrastructure or speculative framework is
introduced. Mutation A–E and applied-edit durability remain inapplicable:
this task performs no product mutation or UndoRedo. T008's cumulative acceptance
and roadmap Phase 1 exit gates remain separate.

Implementation review: both US4 groups, the expanded incremental privacy replay,
clean-open regression and three export variants pass locally with independent
native authority and scoped visual evidence. All 105 native tests and required
baseline commands pass. Verified filesystem presence/absence retains its own
collection stamp and never substitutes for unavailable D text; closed/unknown
editor facts remain distinct. The exact evidence and remaining T008/CI gates are
recorded in [quickstart §2.7](quickstart.md#27-t007-closed-and-partial-observation-evidence-2026-09-27).

### T008 implementation compliance

T008 is restricted to cumulative acceptance orchestration, independent fixture
witnesses, and the existing CI/evidence boundary. Product observation semantics,
public version-1 contracts, dependencies and authority ownership remain unchanged.
I–IV/VII require actual D/R/B and attributable dirty evidence, fresh per-request
collection, and preservation of human source, selection and editing history.
Fixture-only preparation and history replay prove non-interference; they do not
introduce product mutation, UndoRedo or applied-edit durability guarantees.

V/VI/VIII/X/XII require the complete real-editor matrix, source/secret-free
incidental evidence, enabled/disabled export exclusion, bounded controlled
requests, and owned-process cleanup. XI requires a fresh actual-lockfile
license/provenance/advisory review. Shared CI keeps responsibility-based naming
and ownership; capability-specific assertions stay in the observation harness.
No duplicate shared infrastructure or speculative abstraction is authorized.

The complete GUI workflow was supplied by merged
[PR #19](https://github.com/Peter-Tam/godot-agent-kit/pull/19);
the solo-maintainer authorization model is in the
[shared operating procedure](../../.github/README.md). The workflow definition
must come from `main`, while its validated immutable checkout may be the exact
dispatch-main SHA or the explicitly selected exact current head of one eligible
same-repository PR. Maintainer dispatch with that SHA authorizes execution:
neither independent PR review nor separate environment approval is required.
The hosted gate validates current PR head/state and the main-only `live-editor`
environment policy (no deployment reviewers or secrets); the clean isolated
ephemeral one-job runner verifies checkout HEAD against the full SHA before the
complete suite. Read-only scoped credentials and no administrator bypass remain
required.

Local acceptance and hosted CI cannot substitute for actual trusted GUI CI.
At the 2026-09-27 PR #19 rebase inspection, no environment or runner was
registered; that historical count is not a statement of current provisioning.
Protected execution of the full suite on the explicitly selected current
eligible head remains pending until observed. T008 stays unchecked; no human
workstation registration, protection bypass, support claim or Phase 1 completion
follows from local evidence.

Implementation review: all 13 local groups pass (194 cases), including twenty
fresh observations with unchanged independent witnesses and actual prior-history
replay. All 105 native tests and required baselines pass; full privacy and three
export variants pass. Result-only review retains explicit authority/target/limit
knowledge. Fixture setup and evidence namespaces are isolated per group; native
cache state is observed, not inferred from import timing. The addon/core/caller
remain unchanged. At the PR #19 rebase, workflow protection simulations and
Actionlint passed, but no actual GUI CI ran; T008 remained unchecked. This
satisfies local verification, not the remaining release/support gate. The full
post-rebase replay preserves those counts and passed all 29 then-merged workflow
regressions without modifying the PR #19 trust implementation. See
[historical local evidence and the protected GUI CI gate](quickstart.md#29-rebased-t008-local-verification-and-trusted-gui-ci-gate-2026-09-27).

## Project Structure

### Documentation (this feature)

```text
specs/001-observe-gdscript-state/
├── spec.md                         # Existing requirements; unchanged
├── checklists/requirements.md      # Existing specification review; unchanged
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
└── contracts/
    ├── observation-api.md          # Local caller and reusable semantic contract
    └── bridge-protocol.md          # Private Godot integration contract
```

`tasks.md` is generated only by the later task-derivation workflow.

### Planned Source Code (repository root)

The following is the **selected future layout**, not existing implementation or scaffolding generated by this command:

```text
mcp-server/
├── Cargo.toml                      # One package, library + one binary
├── Cargo.lock
├── rust-toolchain.toml
├── src/
│   ├── lib.rs
│   ├── observation.rs              # Typed evidence, state transitions, comparisons/outcomes
│   ├── target.rs                   # Project/session selection rules
│   ├── bridge.rs                   # Private frame/authentication DTO boundary
│   ├── project_fs.rs               # Confined disk/descriptor access
│   ├── runner.rs                   # Deadline and owned read-only worker supervision
│   └── bin/observe-gdscript.rs      # CLI/JSON result boundary, internal worker mode
└── tests/
    ├── observation_contract.rs
    ├── bridge_boundary.rs
    └── confinement.rs
godot-addon/
├── addons/godot_agent_kit/
│   ├── plugin.cfg
│   ├── plugin.gd                   # Enable/disable, lifecycle, bridge registration
│   ├── observation.gd              # Passive Godot state collection/attribution
│   ├── bridge.gd                   # Bounded polling/framing/authentication
│   └── export_guard.gd             # EditorExportPlugin exclusion
└── tests/
    ├── run_observation.py          # Owns disposable fixtures/editor processes
    └── fixtures/observation/
        ├── project.godot
        ├── export_presets.cfg
        ├── fixture_driver.gd       # Test-only preparation and independent witness capture
        └── scripts/               # Synthetic sources and scenario variations
```

**Structure Decision:** Reuse the architectural component boundaries without adding a workspace or a separate core crate. Keep protocol-independent domain types and classification free of Serde/transport details; adapter DTOs map into them. Isolate side-effecting test preparation from passive product observation. The internal worker reuses the same binary/library; it is not a daemon, generic process runner, or additional public tool.

## Observation Design

1. **Bootstrap:** A deliberate registry-init command creates/verifies private metadata storage. An explicitly enabled addon with `GODOT_AGENT_KIT_REGISTRY` binds loopback, creates a new session/token, and atomically advertises metadata. Disabling disconnects peers, stops polling/listening, removes only its own descriptor, and unregisters export hooks/signals/nodes. Stale metadata never proves a live session.
2. **Validate/select:** Start the deadline before filesystem/descriptor work. Validate the script locator and project scope; mutually authenticate source-free candidates using fresh nonces and role-separated HMAC-SHA256 proofs. Verify listener identity before trusting hello metadata; never transmit the registry secret or treat endpoint reuse as liveness. Require exactly one intended project/session before any script-source read. Do not choose focus, first, newest, or a cached target.
3. **Attribute/collect:** The addon enumerates existing documents; unique, stable association supplies R, B, and independent dirty evidence without selecting a tab. Rust independently reads D through `cap-std`. Built-in D is unavailable rather than a scene-container substitute. Unreadable facts do not erase unrelated evidence.
4. **Recheck:** Check the same document/session and source/dirty/version witnesses after collection, including a second bounded D comparison. Detect changes/replacement/closure without requesting reload or synchronization. Retain invalidated evidence separately from current observed values; do not retry the whole request to hide instability.
5. **Classify:** Common core applies the [data model](data-model.md) and [caller contract](contracts/observation-api.md). Open/valid/fully observed can be complete despite dirty/divergent text. Confirmed closed is not-open, never a visible-buffer success. Unavailable identity/dirty/B is limited, not guessed. Source equality does not establish freshness or later edit safety.
6. **Return/terminate:** Supervisor returns a bounded structured result; known disconnect differs from deadline expiry. It never waits indefinitely for a blocked read-only worker, kills no editor, ignores late responses, and preserves valid partial evidence with limitations. No result is cached as a later observation.

### Security and compatibility decisions

- The bridge contract fixes explicit framing, mutual authentication, input/output bounds, allowed operations, safe diagnostics, and project-scoped attribution. T002 must prove stale-descriptor/rebound-port refusal, replay/reflection rejection, cross-language proof agreement, and absence of secrets from wire traffic; T006 replays these guarantees on source-bearing paths. Unknown schema majors are refused; breaking semantics require new versions/migration documentation. The corrected authentication exchange remains the initial unimplemented version-1 proposal, not a legacy compatibility path.
- Reject out-of-project paths and symlink escapes before source reads; recheck before disclosure. The security claim covers normal local user isolation, not hostile same-UID processes or malicious project code already running in Godot. Uncertain attribution still fails closed.
- Only the intentional observation result may carry requested source. Capture a unique source sentinel in acceptance to prove it is absent from incidental logs, errors, descriptors, and other sessions' results.
- Export preset exclusion plus `EditorExportPlugin.skip()` must remove the addon tree even when the plugin is disabled. Verify actual pack contents and production launch; `@tool` alone is not a boundary.

## Phase 0 and Phase 1 Deliverables

- [research.md](research.md): selected versions/APIs/dependencies, decisions and rejected alternatives, primary links, actual clean/dirty feasibility evidence, honest evidence limits.
- [data-model.md](data-model.md): entity fields, attribution/timing, source/dirty availability, comparisons, state transitions, terminal precedence, invariants.
- [contracts/observation-api.md](contracts/observation-api.md): reusable observation and local caller request/result semantics, CLI/exit behavior, representative fixtures, migration rules.
- [contracts/bridge-protocol.md](contracts/bridge-protocol.md): private bootstrap/authentication/framing/operations, no-source-before-selection, deadlines, lifecycle, and input/resource bounds.
- [quickstart.md](quickstart.md): prerequisites, planned setup/native/live-editor/export commands, independent evidence and expected results for every acceptance group. Commands targeting future implementation are explicitly labeled.

## Verification and Requirement Traceability

| Requirements / success criteria | Planned proof |
|---|---|
| FR-001–003; US3.1–5; SC-003 | Multiple distinguishable sessions/projects, source-free ambiguity, exact selection, ended-session replacement, stable document IDs. |
| FR-004–006, FR-010; US1.3, US4.1–6 | Closed/cached/unloaded, missing/invalid/syntax-error/empty, open missing D/B/R, unknown open-state fixtures; independent surface statuses. |
| FR-007; US2.1/3/5; SC-001/006 | Unsaved human buffer, equal-text-but-dirty, unavailable/unattributed dirty indications; no D/B inference. |
| FR-008–009; US1.1, US2.2; SC-001/002 | Exact pairwise comparison and complete divergent cases, whitespace/line-ending distinctions, independent clean D/R/B. |
| FR-011–013; US1.2, US2.4; SC-005 | Fresh evidence, mid-read source/identity changes, stale-content distinctions, non-selected tab, twenty-read no-write/no-selection/no-history-change run. |
| FR-014–015; US3.3/4/6; SC-003/004 | Structured timeout/disconnect/unavailable/partial evidence, stalled editor and blocked worker, all controlled results within five seconds. |
| FR-016; US3.7; SC-006 | Traversal/symlink/authentication/private-registry/source-redaction/malformed-frame tests; no remote listener, no unrelated source. |
| FR-017 | Core type/boundary review and reusable classification tests; no MCP DTOs or Godot objects in domain semantics. |
| FR-018; SC-001–006 | Recorded exact-version GUI feasibility now; complete real-editor acceptance and CI evidence before support. |
| Constitution VIII | Enabled/disabled export exclusions, archive inspection, actual export launch with no bridge/tooling activity. |

Required native baseline after implementation: `cargo fmt --all -- --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked` (includes doctests), and `cargo doc --no-deps --locked`. The package is not a workspace. Review the actual lockfile's advisory/license/provenance evidence. No broad `--all-features` promise is introduced.

**Planning verification executed (2026-09-26):**

- Actual GUI-editor feasibility: clean, dirty, and non-selected dirty states; three before/after read pairs preserved the recorded state witnesses. Native owned-window screenshot reviewed. Exact observations and limits are recorded in research.
- All six generated design artifacts passed unresolved-marker and relative-link checks: **29 local links/anchors** resolved.
- All **six JSON contract blocks** parsed; all **11 semantic classification vectors** produced their specified outcomes, pairwise comparisons, and agreement.
- The installed prerequisite helper passed both read-only path resolution and specification/plan validation, discovering research, data model, contracts, and quickstart in the intended feature directory.
- `tasks.md` was not created. No Cargo/product/full-editor acceptance is reported; the product programs do not yet exist. Post-design constitutional review passes for the proposed design, while every implementation/support gate above remains required.

## Complexity Tracking

No constitutional violation or exception is requested. Three costs have direct safety justification rather than architectural expansion:

| Choice | Requirement served | Simpler alternative rejected |
|---|---|---|
| `cap-std` directory capability | Project-confined D under path races | Ambient `canonicalize` followed by `open` has a check/use gap. |
| Supervised read-only worker process | Whole-request deadline including blocked filesystem I/O | Socket timeout cannot bound disk I/O; abandoned threads accumulate across repeated calls. |
| `ring` authentication primitives | Mutual local-session authentication without transmitting the secret | A raw-token hello cannot authenticate a reused endpoint; hand-written HMAC/randomness would add avoidable cryptographic risk. |

No implementation, task batching, automatic merge, package publication, or untested support claim is authorized by completing this plan.
