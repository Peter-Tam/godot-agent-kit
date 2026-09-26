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
