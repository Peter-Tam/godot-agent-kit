# Implementation Plan: Safely Discover Project GDScripts

**Branch**: `spec/004-discover-project-gdscript` | **Feature identifier**: `004-discover-project-gdscript` | **Date**: 2026-10-01 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/004-discover-project-gdscript/spec.md`, including the accepted Godot project-file visibility clarification.

**Status:** Design, granularity/coverage and post-task consistency analysis passed; merged [planning PR #46](https://github.com/Peter-Tam/godot-agent-kit/pull/46) records analysis of `e7fb197af058c230751e59b2f05c3373adbfbf3b`. **T001 and T002 are complete**, with [scope acceptance](quickstart.md#9-t001-scope-and-private-v4-acceptance-2026-10-01), [public caller/full affected-suite acceptance](quickstart.md#10-t002-public-caller-acceptance-2026-10-01) and the implementation-shape/constitutional reviews below. T003's cumulative discovery gate remains pending; Feature 004 and Phase 1 are not complete.

## Summary

Add one `discover-gdscripts` caller and narrow protocol-independent discovery semantics. Authenticate an exact local project/editor lifetime without inventing a script path, obtain the editor's effective visibility context, enumerate fresh confined filesystem metadata in the existing owned-worker boundary, recheck identity/coverage/context, and return a bounded complete/limited/refused/interrupted inventory. Each returned exact path is input for a later independently checked operation, not an edit basis or permission token.

The [research](research.md) demonstrated that an idle, unchanged `EditorFileSystem` cache can miss a newly created file. Therefore production does not use that cache as an exhaustive inventory, force a scan or duplicate it with another index. One metadata walker follows the pinned native visibility rules. A thin addon owner supplies effective project-data and liveness/change facts through the existing operation slot; Rust owns traversal and terminal interpretation. Private bridge v4 adds authenticated discovery capability; native revision 2 and all existing public observation/edit/open v1 contracts remain unchanged.

## Technical Context

**Language/Version:** Existing Rust 1.98.1, edition 2021; thin GDScript editor integration; Python 3.10+ real-editor harness. Existing C++17 native integration is unchanged by the selected design.

**Primary Dependencies:** Existing locked `serde =1.0.229`, `serde_json =1.0.151`, `cap-std =4.0.3`, `ring =0.17.14`, stock Godot and the existing native bundle for composed edit/open acceptance. No new crate, parser, SDK, async runtime, service or engine patch.

**Storage:** Existing owner-private source-free registry; selected-project directory metadata; bounded request-local path/identity evidence. No source bodies, persistent inventory, database, watcher or replay store. Temporary owned acceptance projects/evidence are test infrastructure, not product storage.

**Testing:** Deterministic core outcome/limit/invalidation tests, actual confined filesystem and process/bridge boundary tests, and the owned real-editor discovery runner. Reuse the existing independent buffer/history witnesses, privacy/export checks and campaign facilities. See [quickstart.md](quickstart.md) for focused versus complete affected suites; planning probes are not those tests.

**Target Platform:** Official `4.7.2.stable.official.ed1daf0bf`, commit `ed1daf0bf001b61586d9930840f2f1394092c079`, executable SHA-256 `c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf`, macOS 26.6.2 arm64. This reuses the existing candidate and macOS ACL implementation. Discovery support requires its own acceptance; no additional version/platform is selected.

**Project Type:** Existing local Rust library/CLI plus Godot editor addon. No MCP adapter or general filesystem browser.

**Performance Goals:** Terminal result within five seconds: clock starts before argument parsing, operation cutoff at 4.5 seconds and 0.5-second delivery reserve with stdout drained. The owned metadata projection handled the required 100-script/ten-folder fixture, but full authenticated product timing remains acceptance work. No arbitrary-project-size or concurrent-throughput promise.

**Constraints:** Source-free authenticated selection; exact rooted namespace; no source reads, editor loads, scans, parses, lifecycle/history mutations, force, retry queue or offline fallback. Retain only safely attributed evidence and never turn missing/invalidated checks into completeness. Existing human-work/coherence guarantees and native ownership remain unchanged.

**Scale/Scope:** One selected project/session and one bounded standalone-script inventory. [Data-model bounds](data-model.md#2-fixed-supported-profile) are normative: 1024 returned entries, 1024 directories including root, depth 64, 16384 visited entries, existing exact path limits, bounded internal batches and a 6 MiB terminal result. Larger/unsupported areas yield explicit gaps or refusal, never silent truncation.

## Constitution Check

*Gate: evaluated before Phase 0 research; re-evaluated after Phase 1 design.*

**Initial disposition:** PASS for bounded research inside the existing architecture. No new infrastructure prerequisite or constitutional exception was selected.

**Post-design disposition:** PASS for this proposed design, not implementation or release acceptance. Independent contract/coverage and confinement/complexity reviews found no actionable defects. The safeguards below remain mandatory implementation obligations; no constitutional exception or stronger hypothetical threat-model gate was introduced.

| Obligation | Selected design and required verification |
|---|---|
| I / IV — independent authority and verification | Inventory proves only path observations. No D/R/B, source revision, clean-state or edit-readiness inference; complete coverage needs actual metadata exhaustion and relevant rechecks. |
| II — human work | No buffer/resource/source/history/lifecycle APIs in discovery. Real-editor dirty, divergent, non-selected and prior-history witnesses prove preservation. |
| III — native semantics | Existing edits/opening retain their native route; discovery introduces no alternate mutation route or pretend undoable listing. |
| V — least privilege | Same local authentication, project capability, ancestor/owner/mode/ACL and no-follow guards. No source-before-selection, outside traversal, arbitrary execution or permission bypass. |
| VI / XII — real evidence | New discovery positive/refusal/non-interference/composed gates plus complete directly affected v4 observation/edit/open suites before feature completion. No disk-only or getter-only substitute for preservation evidence. |
| VII — ownership | Domain reduction in Rust; wire/CLI/process concerns at adapters; public editor-path/scan facts in addon. No Godot API in core policy or rival addon result reducer. |
| VIII — tooling isolation | Extend existing exclusions/actual enabled, disabled and hook-only export validation to the discovery script and fixtures. No gameplay dependency. |
| IX — small surface | One project-only operation; no search language, include-ignored mode, source catalog, pagination API or independent lifecycle controls. |
| X — truthful diagnostics | Distinct complete/limited/refused/interrupted outcomes, fixed reasons/stages, explicit gaps and earlier evidence, bounded diagnostics, inventory-free logs/registry. |
| XI — independent implementation | Exact public source/API and owned behavior establish rules; reuse existing locked dependency provenance. No unrelated implementation copied. |
| XIII — complexity | One current-need walker, private project-selector seam and deliberate authenticated v4 cutover. Costs and rejected simpler alternatives are recorded in research and below. |
| Compatibility / workflow | Public observe/edit/open v1 unchanged; new discovery v1; coordinated private v4 with no fallback; native revision 2 unchanged. Later implementation remains one approved task per PR with bundled tests/docs and shape review. |

The inherited threat model excludes hostile same-UID software and malicious already-running editor extensions. It does not permit ignoring detected interference, unsafe roots or missing evidence. No atomic exclusion against arbitrary filesystem writers is claimed. Real-editor acceptance is required; a specific hosted provider/runner topology is not newly required.

**Planning validation (2026-10-01):** All six planning documents passed heading/fence/template/whitespace checks; 40 local document links/anchors and two retained-evidence links resolved, and all three JSON examples parsed. The retained research-summary hash matched the recorded evidence. The installed prerequisite helper discovered the intended spec/plan/research/model/contracts/quickstart under the verified feature directory. All 17 FRs, 17 story scenarios and six SCs have acceptance coverage. Owned probe processes and temporary sources/projects/binaries were cleaned up. No product implementation, task list, full product acceptance or untested support claim was produced.

## Execution Design

### Project-only selection

Reuse checked `RequestId`, `ProjectRoot`, `SessionId`, `ResourcePath`, decimal counters and interval/stamp primitives without modifying existing document contracts. `DiscoveryRequest` contains no script. Privately factor `project_fs` root validation and `target` candidate authentication so both the existing script resolver and discovery consume one algorithm. Existing script resolution still validates its real locator and produces its real `ResolvedTarget`; discovery receives project identity and an authenticated channel without a dummy locator.

Do not clone the selector or expose speculative routing APIs. Preserve authentication-denial/ambiguity/unresolved-candidate precedence and exact-ended-session behavior. Public `target::resolve` remains useful for current observation/open/edit callers; it is not a compatibility shim. The new project-only seam and selected-editor internals remain crate-visible.

### Read-only editor scope

The new addon owner claims the existing single operation slot and returns an independently obtained scope context: exact session/project, `godot_project_files_v1`, effective data directory from `EditorInterface.get_editor_paths().get_project_settings_dir()`, scan/import state, session-local filesystem-change counter and editor-local stamp. Accept only the two demonstrated project-settings paths for this exact candidate. Do not derive the effective path from a mutable setting, read `project.godot`, enumerate the cached tree, inspect documents or use the stock validator.

An active scan/import or unavailable context cannot yield complete discovery; the core returns an explicit limited or unsupported result as defined by the contracts. The owner does not wait for background work to settle, force a scan or use editor focus. Begin/recheck/finish/abort are private operations, not additional public tools.

### Fresh confined metadata

Use one feature-owned walker beneath `project_fs`, not a general traversal framework. Metadata checks and visibility rules are specified in [data-model.md](data-model.md#5-private-filesystem-evidence-and-coverage-algorithm). One component is checked/opened at a time through an already validated directory handle; initial/final namespace identity must match. Do not resolve a symlink and then inspect its destination. Marker checks distinguish absent, regular, unsafe and unavailable, without reading contents.

The walker emits bounded, checked progress batches. It keeps only names/identities/access and membership witnesses required for this request, never source bytes or an inventory for another request. Source-file mtime/content changes alone do not determine path validity. Directory/marker/context changes, failed access or missing recheck evidence prevent complete coverage; unsafe root/project binding clears all inventory.

### Recheck, terminal interpretation and cleanup

After enumeration or a local gap, recheck gathered namespace and permission evidence and request fresh editor scope/lifetime facts. A complete result requires exhausted supported scope, no gap/known invalidation, valid root/path checks and a matching idle editor context. Unknown atomic stability remains explicit. Limited results may retain independently valid entries; interrupted results may retain earlier observations only with their earlier validity stated. Authentication/root denial and ambiguity suppress inventory.

The parent owns the original cutoff, request binding and typed terminal reduction. Late/malformed/out-of-order worker messages cannot upgrade a returned result. On cancellation/timeout, stop the owned read-only worker and close its selected channel without awaiting blocked I/O or addon acknowledgment. The addon releases only this attempt's slot on finish/abort/channel loss/expiry; there is no entered native mutation whose lifetime must be modeled. Do not change the retained-native-owner rules for concurrent existing open/edit attempts.

### Compatibility and implementation shape

Private v4 appends one capability bit, `discover_gdscripts`, to the existing role-separated transcript and migrates descriptor/tuple version checks and all consumers together. It does not alter native ABI revision 2, artifact names or public v1 results. The [bridge contract](contracts/bridge-protocol.md) defines exact migration and discovery messages. Old installed addons must be updated and their editor session restarted; no silent fallback.

Keep each production owner coherent:

- `script_discovery`: request, evidence invariants and terminal interpretation; narrow public request/outcome types only for the real caller.
- `project_fs::discovery`: confined metadata traversal and namespace evidence; crate-private, not an arbitrary path browser.
- `target`: source-free unique authentication and selected capabilities; common project selection, separate current script binding.
- `runner::discovery`: bounded child supervision; its worker and codec own I/O, not competing semantic policy.
- addon `script_discovery.gd`: scoped public-editor facts and existing-slot lifetime, no Rust policy or file tree.

The existing `observation.rs`, `runner.rs`, `bridge/wire.rs` and addon `bridge.gd` are already large. Add feature-owned modules and only necessary dispatch/seam changes rather than appending another independent responsibility. Preserve current-consumer public APIs; no public attempt/witness/codec variants just for hypothetical later work. Avoid booleans for mutually exclusive attempt states, recoverable-evidence `unwrap`/`expect`, generic traits, or one-field-per-file fragmentation. Implementation must review cohesion and perform proportionate cleanup introduced by this work before task completion.

## Project Structure

### Documentation (this feature)

```text
specs/004-discover-project-gdscript/
├── spec.md
├── checklists/requirements.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── tasks.md
└── contracts/
    ├── discovery-api.md
    └── bridge-protocol.md
```

[tasks.md](tasks.md) derives three coherent increments: the authenticated read-only editor-scope/v4 boundary, the complete public caller with all inseparable story safety, and cumulative repeated-use/composed acceptance. Tests/docs remain bundled with their owners; US2/US3 retain independent story phases without unsafe later safety PRs. Granularity/coverage and subsequent consistency analysis passed. This execution selects only T002, after T001's completed acceptance and merged PR #47; T003 is not authorized by it.

### Source Code (repository root)

The following is the selected implementation layout, not files created by this planning command:

```text
mcp-server/
├── Cargo.toml / Cargo.lock                 # one additional binary; no new dependency
├── src/
│   ├── script_discovery.rs                 # new consumed discovery domain
│   ├── project_fs.rs                       # existing guard owner, private root seam
│   ├── project_fs/discovery.rs             # new confined metadata walker
│   ├── target.rs                          # private project-only selection reuse
│   ├── bridge.rs / bridge/wire.rs          # coordinated v4 authentication/dispatch
│   ├── bridge/wire/discovery.rs            # bounded discovery codec
│   ├── runner.rs                          # reuse existing clock/owned-worker primitives
│   ├── runner/discovery.rs                 # new discovery supervisor
│   ├── runner/discovery/worker.rs          # acquisition and editor-context exchange
│   ├── bin/discover-gdscripts.rs            # one new caller
│   └── lib.rs                             # current-consumer exports only
└── tests/
    ├── script_discovery_contract.rs        # new behavior/boundary contract cases
    ├── script_discovery_caller.rs          # new process/deadline/output cases
    ├── confinement.rs                     # shared selector/guard regressions
    └── bridge_boundary.rs + submodules     # v4 and real boundary evidence

godot-addon/
├── addons/godot_agent_kit/
│   ├── script_discovery.gd                 # new scope/context owner
│   ├── bridge.gd / plugin.gd               # v4, capability and shared-slot dispatch
│   └── export_guard.gd                     # existing export boundary
└── tests/
    ├── run_script_discovery.py             # new owned discovery scenarios
    ├── fixtures/script_discovery/          # owned fixture controls only
    ├── run_observation.py                  # reused harness/witness and v4 consumers
    ├── run_script_edit.py / run_script_open.py
    └── run_editor_campaign.py              # add discovery only as a current runner consumer
```

**Structure Decision:** Retain the two existing component boundaries and one Cargo package. No native source change, new crate, external service or feature-specific CI workflow. The current `.github/workflows/ci.yml` builds/checks the additional real binary; shared transcript consumers and current operator/contract reproduction notices migrate with v4. Update the existing campaign's runner/fingerprint mapping only when the discovery runner actually exists; do not expose a no-op scenario or unused registration.

## Verification and Requirement Coverage

| Requirements / scenarios | Required evidence |
|---|---|
| FR-001–FR-003; US1.1–US1.4; SC-001 | Exact independently prepared inventory, 100 scripts/ten folders, duplicate basenames, empty/invalid/read-only/tool/case variants; visible addons/VCS/export controls; hidden/marker/nested/effective-data exclusions and true empty scope. |
| FR-004–FR-005; US2.1–US2.4; SC-002 | Authenticated project-only routing, ambiguous/ended/replaced sessions, unsafe roots/redirects, marker/namespace races; no unauthorized inventory. |
| FR-007–FR-011; US3.1–US3.4; SC-003 | Local gaps, unsupported representation, exact/one-over bounds, fresh create/rename/delete, stale editor-cache negative control, blocked worker/editor, timeout/cancel/disconnect and no false complete-empty. |
| FR-006 / FR-009 / FR-016; US4.1–US4.2; SC-004 | Twenty fresh discoveries with independent live D/R/B/dirty/selection/history witnesses, exact target attribution and zero request-caused effects. |
| FR-012 / FR-015; US3.5 / US4.3; SC-005 | Separate discovery → choose path → fresh observe/open/edit; later path replacement does not inherit authority; dirty edit refusal and applicable existing A–E/durability preserved. |
| FR-013–FR-014 / FR-017; US4.4; SC-006 | Result-only review, path/source/credential sentinels, no incidental inventory disclosure, enabled/disabled/hook-only export inspection and execution. |

All seven spec edge cases have boundary coverage in the quickstart. Existing public deadlines and source limits remain unchanged. The v4 shared authenticated cutover requires complete unfiltered observation, edit and opening acceptance on the same final implementation head, plus the full discovery scenarios; focused runs are for development and do not replace that cumulative gate.

## Complexity Tracking

No constitutional exception is proposed. Principle XIII's current-need review applies even without an exception:

| Addition | Current requirement / rejected simpler alternative | Ongoing cost and justification |
|---|---|---|
| Metadata walker | Idle cached inventory demonstrably misses a new file; forced scan is forbidden | Small pinned predicate and bounded namespace tests, necessary to deliver honest fresh coverage |
| Private project-only selector seam | Existing requests require a script; dummy path or copied selector is wrong | One shared implementation and direct regressions, not a new public routing abstraction |
| Private bridge v4 and context owner | One new capability must be authenticated; v3 is a fixed transcript | Coordinated consumer migration and affected full suites; no new listener, scheduler, ABI or permission |
| Existing worker reuse for discovery | Blocking filesystem I/O must not defeat five seconds | Operation-specific event/state handling using already-present supervision, not a new worker service |

The exact decisions, primary sources, executed research and rejected alternatives are in [research.md](research.md). Planning may not promote an untested hypothetical stronger isolation property into a product blocker; a real blocker must satisfy the repository's requirement/failure/actor/reachable-evidence test.

## T001 implementation-shape and constitutional review (2026-10-01)

The integrated task keeps the selected boundary and introduces no inventory,
worker codec, public discovery model, selector refactor or caller ahead of T002.

- `script_discovery.gd` owns only public editor scope facts, signal registration,
  a session-local epoch and the read-only lease. It never receives a script
  locator, traverses a tree, reads source, inspects documents/cache or enters
  native mutation. Its internal getter seams are private; current bridge calls
  consume `configure`, `available`, `handle`, `cancel_owned` and the existing
  owner's `is_active_stage` interface.
- The already-large `bridge.gd` retains transport/authentication/framing and
  shared-slot ownership. Its necessary additions create/tear down the scope
  child, authenticate the eighth bit and dispatch discovery tuples. The new
  owner is not appended as another policy/reducer responsibility. Existing
  `plugin.gd` start/stop/free wiring already covers that child; `script_open.gd`
  has no private-version encoder, so neither needs a change.
- Rust bridge/wire and existing worker-startup consumers receive the coordinated
  version/capability migration only. `Capabilities.discover_gdscripts` is part of
  the currently consumed authenticated record, not a speculative routing API.
  Public observation/edit/open schema 1 and native revision 0/2 are unchanged.
  Strict typed decoding rejects missing/extra/duplicate/non-Boolean capability
  fields; no new recoverable-evidence panic or `unsafe` path was introduced.
- The single `_active` slot remains authoritative. A peer's one-begin marker and
  the exact owner/peer association prevent restart/revival; read-only cancellation
  releases only its owner. There is no entered-native discovery stage, queue or
  retry. Existing native entered-owner retention stays at its existing boundary.
- `discovery_scope_acceptance.py` owns feature-specific probes, while existing
  boundary groups retain execution, privacy, window capture and cleanup.
  Getter-fault and deliberate scan/import preparation remain private fixtures;
  ordinary positive contexts use actual public editor getters. Existing generic
  export exclusions and campaign input hashing cover the new files without a
  second export policy or campaign registration.

**Principle XIII:** The present need is authenticated effective visibility context
for a locator-free request. Reusing document observation would acquire forbidden
source/document facts; deriving the data path from the mutable setting was
demonstrably wrong. One small owner plus the coordinated fixed-transcript v4
cutover satisfies the requirement using the existing listener, credentials, slot
and runtime. Its cost is one bounded owner and directly affected regression
coverage, justified by the current contract. No new dependency, service, native
ABI, generic framework, permission, process gate or hosted-runner requirement is
introduced.

**Constitutional disposition:** The implementation preserves independent D/R/B
authority, human work, native mutation semantics, local confined authentication,
source privacy and export isolation. Context acknowledgments cannot establish an
inventory or authorize a later edit. [Live acceptance](quickstart.md#9-t001-scope-and-private-v4-acceptance-2026-10-01)
passed the focused scope probes and complete observation/edit/open gates on the
unchanged integrated implementation. Together with this shape review, that
evidence completes T001 only; discovery inventory and feature acceptance remain
T002/T003 work.

## T002 implementation-shape and constitutional review (2026-10-01)

The implemented caller follows the approved adapter → protocol-independent
semantics → editor-integration boundary. Independent Rust correctness/shape
review found no actionable production defect. Acceptance-harness review found
two fixture prerequisites to correct: marker-excluded directories still count
as visited, and intentionally invalid dirty B must not be required to converge
with R. The focused live groups exercise those corrections rather than treating
review or authored assertions as passing evidence.

- `script_discovery.rs` owns the checked request and immutable public outcome.
  Private `events` and `attempt` modules separate bounded evidence decoding from
  ordered acceptance, terminal precedence and sticky invalidation. Public
  declarations are consumed by the actual CLI/library boundary; no attempt,
  filesystem witness, generic query interface or future-task hook is public.
- `project_fs::discovery` owns only capability-rooted metadata acquisition and
  rechecks. Its production portion is about 656 lines; the larger file includes
  colocated private filesystem regressions. Traversal, marker and namespace
  witnesses form one coherent responsibility; no framework or arbitrary
  file-size split is warranted. Handles remain depth-bounded and retained
  witnesses/name batches have fixed limits. Source contents are never acquired.
- `target` privately shares the existing source-free authentication algorithm,
  while `project_fs::project_root` shares existing root validation. The genuine
  public document resolver retains real locator validation and existing
  observe/open/edit callers. No dummy document, optionalized old contract,
  copied security policy or compatibility shim is introduced.
- `bridge::wire::discovery` owns strict editor/control and worker framing.
  `runner::discovery` owns the original cutoff, child lifetime and parent-side
  reduction; its worker owns blocking acquisition. The existing owned-child
  and shared addon-slot mechanisms remain authoritative. Read-only cancellation
  does not enter native mutation or change retained native-owner rules.
- Attempt ordering uses an enum. Independent facts such as namespace change,
  exhausted coverage and observed scan/import state are not substituted for
  lifecycle state or atomicity. Recoverable evidence errors use typed failures,
  not safety-path panics; no new dependency, service, native ABI or permission
  mechanism is added.
- The discovery runner owns CLI/result checking; its case module owns scenario
  preparation and independent assertions. Existing project/window/history,
  cleanup and export helpers are reused. Private GDScript fixtures hold actual
  scope replies and inject negative controls without exposing product hooks.
  The existing campaign registers the six current groups, fingerprints their
  execution inputs and rejects unavailable full-feature acceptance.

**Principle XIII:** The present requirement is fresh unknown-path discovery
without source acquisition or editor effects. The planning probe established
that the idle cached editor inventory can miss a new file. One bounded rooted
walker, the shared project-only selector seam and reused owned worker address
that failure with fewer mechanisms than a parallel cache, parser, scan or
service. Their maintenance cost is the directly required evidence/recheck code
and regression coverage, not a speculative infrastructure layer.

**Constitutional disposition:** The production shape preserves independent
D/R/B authority, human work/history, confined authenticated selection and
source/export privacy. Inventory entries carry no source revision, durable
identity or later editing authority. Complete coverage requires actual
exhaustion and rechecks; known denial/binding loss suppresses inventory, and
permitted interrupted evidence remains explicitly earlier. [Runtime acceptance](quickstart.md#10-t002-public-caller-acceptance-2026-10-01)
passed all six task-owned discovery groups and complete affected existing
suites. Together with this shape review, it completes T002 only; T003's
cumulative discovery/feature gate remains pending.
