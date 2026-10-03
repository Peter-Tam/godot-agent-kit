# Implementation Plan: Safely Open a Known Project GDScript

**Branch**: `spec/003-open-project-gdscript` | **Feature identifier**: `003-open-project-gdscript` | **Date**: 2026-09-29 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/003-open-project-gdscript/spec.md`.

**Status:** Design, implementation and Feature 003 acceptance are complete. Granularity and consistency analysis passed; T001/T002 merged in PRs [#42](https://github.com/Peter-Tam/godot-agent-kit/pull/42)/[#43](https://github.com/Peter-Tam/godot-agent-kit/pull/43). The maintainer selected only T003; its [cumulative acceptance](quickstart.md#10-t003-cumulative-acceptance-2026-09-30) and [implementation-shape review](#t003-implementation-shape-and-constitutional-review-2026-09-30) passed independently of delivery review/merge. Roadmap Phase 1 is **complete** under the subsequent [phase exit decision](../../PHASE_1_EXIT.md). The initial B1 callback-isolation hold remains withdrawn under the inherited threat model; see [scope correction](research.md#6-scope-correction-and-callback-limitation). Historical research is not substituted for implementation acceptance.

## Summary

Add one explicit `open-gdscript` caller for an existing known standalone project script. Reuse exact authenticated routing, independent observations, project confinement and the common editor operation slot. Keep observer and editor semantics unchanged: observation never opens, editing still refuses closed targets, and opening never authorizes a subsequent edit.

There are two branches:

1. **Already open:** fresh identity/open-state evidence and ordinary source/dirty observation/recheck; no lifecycle call, source profile, validation helper, tab selection, Save or history action. Dirty/divergent/limited observations remain explicit.
2. **Closed:** independently capture confined D and inspect cached R plus the actual native opening context. Refuse unresolved loaded source or unsafe/unknown context. Privately validate the exact current editor source when one will be visited by native navigation. After one-shot authorization, either retain the existing clean matching cached Script or create a native GDScript from the exact capture, claim its path without takeover and initially compile only that new object. Then call native `EditorInterface.edit_script` with focus enabled. Independently observe and recheck target D/R/B/clean state and protection witnesses before reporting success.

The source-bound cold branch avoids ordinary root ResourceLoader dispatch and unbound source acquisition. Native new-resource compilation is not a reload/repair of an existing R. Source initialization and acknowledgments are not observations: the final R and B come from their real authorities, separately from D.

## Technical Context

**Language/Version:** Existing Rust 1.98.1, edition 2021; thin GDScript editor integration; C++17 standard public-ABI GDExtension; Python 3.10+ owned-editor fixtures.

**Primary Dependencies:** Existing locked `serde =1.0.229`, `serde_json =1.0.151`, `cap-std =4.0.3`, `ring =0.17.14`, matching stock Godot and the system native toolchain. Reuse the existing bounded one-shot stock validator only for current-document admission when needed. No new crate, binding framework, parser implementation, daemon, MCP SDK or engine fork.

**Storage:** Existing owner-private source-free session registry, selected project files, request-local descriptors/references/evidence. Current-document validation may use the existing private disposable source-only helper project; raw child output is discarded and its files are removed. No project source is written, no source database/index or replay store is added.

**Testing:** Deterministic opening-policy and changed boundary tests, actual native lifecycle/fault fixtures, real-editor positive/refusal/interruption/history cases, twenty-request opening stress, and the composed observation/edit A–E/durability/privacy/export gates. Existing Cargo/native/workflow checks remain applicable. The [quickstart](quickstart.md) separates current baseline commands from future opening validation commands.

**Target Platform:** Initial candidate is official `4.7.2.stable.official.ed1daf0bf`, full commit `ed1daf0bf001b61586d9930840f2f1394092c079`, executable SHA-256 `c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf`, macOS 26.6.2 arm64. Native API/ABI generation and executable checks remain exact. Opening support is claimed only after implementation acceptance; no other platform/version is selected.

**Project Type:** Existing local Rust library/CLI plus Godot editor addon/native integration. No MCP or general editor-management product surface.

**Performance Goals:** One controlled result within ten seconds, including an unresponsive editor: 9.5-second operation budget from before parsing/resolution and 0.5-second delivery reserve. Existing five-second observation and ten-second edit contracts are unchanged. No arbitrary project-size, throughput or concurrent-load promise.

**Constraints:** Exact target/session lifetime; independent D/R/B; source and native-history preservation; no force/Save/reload of existing state, root loader callback, target instantiation, arbitrary evaluation, or automatic retry/rollback. Refuse unavailable safety evidence. Timeout bounds the caller result, not synchronous native completion. Malicious code already running in the editor and hostile same-UID software remain outside the inherited threat model; no stronger isolation guarantee is invented.

**Scale/Scope:** One standalone `.gd` per request, one active editor collection/edit/open slot, 512 KiB per source, existing request/project/path limits, 4 KiB source-free control, 4 MiB selected-peer request, 12 MiB response/worker frame and depth-32 JSON bounds. The new-open representation is exact LF UTF-8, no BOM/CR/NUL or other non-tab/newline controls. The explicit effect profile and metadata/context limits are in [native integration](contracts/native-integration.md). Already-open observations retain existing independent per-source limits rather than inheriting new-open eligibility restrictions.

## Constitution Check

*Gate: evaluate before Phase 0 and re-evaluate after Phase 1 design.*

**Initial disposition:** PASS for bounded research against the existing architecture and constitutional invariants. Two independent investigations and owned synthetic source/editor probes were appropriate to the actual new lifecycle boundary; no production implementation or added infrastructure was authorized.

**Post-design disposition:** PASS for the bounded proposed design, not implementation or release acceptance. The in-scope native dirty-tab effect has explicit admission and verification rules. A blanket B1 hold against an intentionally buffer-writing, already-running extension did not meet the inherited in-scope-actor test and was withdrawn. Source/effect checks and actual human-work protection remain mandatory.

**Planning verification (2026-09-29):** Reviewed the final domain/caller/bridge/native separation, five outcome meanings, independent verification, human-work and entered-call lifetime rules, private-context handling, exact-method Resource cleanliness, compatibility cutover and current-need complexity costs against this table. Artifact checks passed for all seven planning documents plus current project status: 71 local/evidence references and Markdown anchors, balanced fences, no unresolved template markers, all 21 story scenarios mapped with 18 functional requirements and seven success criteria retained, correct active feature and no generated task list. All four recorded research-summary hashes matched retained evidence. The installed prerequisite helper discovered research, data model, contracts and quickstart. No extension hook configuration was present. These checks validate planning artifacts, not product acceptance; the locked-desktop limitation and remaining implementation evidence are explicit below.

| Obligation | Design and verification |
|---|---|
| I / IV — independent coherence and observed postconditions | Fresh Rust D plus editor R/B/dirty/identity and rechecks; new open needs exact agreement/clean state. Native initialization/acknowledgment cannot certify success. Already-open recognition never claims clean/coherent state by implication. |
| II — human-work preservation | Refuse conflicting/unknown cached R and unsafe departing context. Dirty current R == B is eligible only with independent dirty/version/semantic evidence; never Save/tag/reload it. Protect exact references through effects and verification; preserve newer work and return unverified outcomes on invalidation. |
| III — native semantics | Real native GDScript objects and ScriptEditor buffers; public non-takeover path binding; initial compilation only for a new Resource; no alternate editor/history, hidden disk writer or forced cache replacement. |
| V — least privilege | Source-bound confined capture, bounded effect profile, local authentication, no outside-project source, target execution or arbitrary operation input. Current-document helper reuses the existing permission/lifetime boundary. Known unsafe context refuses; approval never replaces guards. |
| VI / XII — quality and real-editor gates | Require full new-opening acceptance plus unchanged observation and composed edit A–E/durability. Primitive/getter research and a locked desktop do not pass visible-buffer acceptance. |
| VII — protocol-independent core | Opening semantics and terminal reduction stay in Rust; JSON/auth/worker transport and Godot object APIs remain at their existing boundaries. No rival addon outcome policy. |
| VIII — tooling isolation | Shared native bundle, opening node and fixtures remain development-only. Verify enabled, disabled and hook-only production exports and actual exported execution. |
| IX — composable surface | One known-path open operation; no discovery/index, close, Save/history controls, MCP, batch or runtime tools. |
| X — deterministic diagnostics/privacy | Distinct target/context/parse/timeout/disconnect/partial reasons, request/stage correlation and unavailable evidence. Current/unrelated source stays private to admission; source appears only for the authorized requested target, never incidental logs or registry metadata. |
| XI — provenance | Existing dependency/SDK provenance and independently developed public-API integration; pinned official sources in research. No new dependencies. |
| XIII — proportionality | Reuse existing mechanisms; add only current opening policy, effect guards and capability migration. No generic callback isolation, registry service, approval layer or CI topology. Complexity decisions below name present failures and costs. |
| Compatibility and workflow | Private v3/native revision 2 coordinated cutover, unchanged public observe/edit v1, one new public opening v1. No aliases/fallback. Future tasks remain one task/PR, bundled tests/docs, granularity review before analysis; no implementation task generated here. |

Any implementation finding that violates these requirements must be fixed or held with exact requirement, in-scope actor/failure and reachable evidence. The selected design is not permission to label every new request unsupported and claim completion.

**T001 execution planning (2026-09-30):** Implement the selected native/current-source boundary and coordinated private-v3/native-revision-2 migration as one task. Native file/document guards, thin addon ownership, existing Rust validator admission and current compatibility/fixture consumers have disjoint authoring owners and one integration owner. Integrate before validation; run visible-editor campaigns serially. The existing constitutional mapping and complexity decisions apply unchanged: no second operation scheduler, validation service, dependency, approval or workflow is added. Keep the authenticated product opening capability false until T002 supplies its complete caller. Acceptance must independently prove source/history preservation and effect-aware termination; review module cohesion and visibility before completion.

**T002 execution planning (2026-09-30):** The maintainer selected the next task only,
T002, after T001 merged in PR #42. Implement the complete Rust caller/domain/
supervisor, authenticated addon exchange, and public-caller fixture groups as
disjoint authoring slices under the existing contracts. One integration owner
collects all slices before builds and runs visible-editor campaigns serially.
The constitutional mapping and Principle XIII decisions above remain unchanged:
reuse the current validator, collector, confinement, single operation slot and
owned-worker supervision; add no dependency, permission, service or workflow.
Preservation, effect-aware interruption, privacy/export and unchanged observation/
edit acceptance are T002 completion gates. T003's cumulative sequences and feature
completion remain out of scope.

**T003 execution planning (2026-09-30):** Selected only T003 after T002 merged in
PR #43. Add repeated-opening and composed open/observe/edit acceptance as
disjoint harness slices, then integrate the complete `all` selector and run the
opening, edit and observation GUI campaigns serially. Reuse existing fixture,
history, durability, result-review, privacy and export helpers; the concrete gap
is cumulative current-identity/history proof after product opening, not a missing
production mechanism. No new dependency, service, permission or CI boundary is
needed. Constitutional I–VI/XII require independent D/R/B, preserved human work,
real history and durability; VII–X/XIII retain existing boundaries, source
privacy, truthful effects and the smallest current-consumer harness extension.
Completion requires every T003 acceptance gate and a proportional shape review;
Phase 1's separate discovery/lifecycle gaps remain out of scope.

## Native Opening Composition

The [native contract](contracts/native-integration.md) controls details.

### Preparation and no-effect recognition

Authenticate and bind one project/session before source disclosure. Worker and addon use the existing source-free selection and shared operation slot. Rust independently captures/rechecks D; the editor reports exact open association and cached-R state without loading.

For an exact already-open document, use the normal collector and identity/open-state recheck. Return `already_open_unchanged` with actual source availability and dirty/divergent state. Do not invoke native opening, change selection or insist on the new-open source profile. A denied/ambiguous target retains source-suppression precedence; uncertainty about actual open identity cannot become recognition.

For a closed target, retain an existing matching clean cached R or record cache absence. Prepare read-only file identity/content guards; cached conflicts, known stale R, unknown dirty/open state and unsupported representation refuse. Capture the effective current editor, external-editor setting, bounded source/context profile and passive compiled metadata. No project or editor setting is changed to create eligibility.

### Current-document admission

Native new-open navigation can call `apply_code` and validation on the departing current document. If there is an applicable current GDScript, require exact R == B, observed independent dirty/current/saved state, supported source/compiled context and a completed **valid** current-source result from the existing source-only validator. This input is private guard evidence, not requested target source. Bind the helper result to the same request/session/current identity, source hash and effective warning/context witness; recheck all of them at entry. A known no-source current editor needs no helper. Unknown or unsupported current editor state refuses.

The requested target may itself be syntax-invalid and still open successfully. Invalid/unavailable **current-context** validation is a different refusal; it must not be reported as a target parse error. No product wait/repair loop seeks eventual human convergence. Passive compiled metadata plus the source profile excludes stale tool/script-base/export/object-property/dynamic-getter contexts without fabricating a private pending-export flag. Pending drag/Undo histories remain explicit regressions.

### Authorized lifecycle

The parent records possible effects before releasing a one-shot authorization. Under fresh native identity/source/context/deadline guards:

- **Cold:** construct a native GDScript and initialize it through the bound `Script.set_source_code` method using captured bytes; claim the exact path through the bound `Resource.set_path` method's non-takeover semantics; verify cache identity/path/source and unedited Resource state; initially compile it with `Script.reload(false)`. Property assignment/`Object.set` is not equivalent: the composed prototype then failed the existing edit's dirty gate. Never clear an edited flag to repair opening. Root parse/compile diagnostics are explicit and do not by themselves prohibit opening an otherwise admitted script. Never call `take_over_path`, `set_path_cache`, root `ResourceLoader.load` or instantiate the requested script.
- **Cached:** retain the existing Script and leave its source/path/compiled state untouched. No source assignment or reload.
- Open that exact Script with `EditorInterface.edit_script(script, -1, 0, true)`. New selection is expected and reported; the caller does not choose focus as a target. Keep admitted departing and target references, source/version/dirty/context witnesses and the operation slot.

Use bounded staged facts for cache binding, initial compilation and document opening so the supervisor can retain known effects before a later stall. A failure after cache publication is not a false no-effect refusal. Cancellation never closes a newly opened buffer or restores source. Native ownership survives entered calls until they return; late replies cannot upgrade an already-returned result.

### Independent verification

After native effects, use fresh collector evidence plus independent Rust D and file/namespace recheck. Verify the exact target is open and its actual D/R/B agree with admitted source, with attributable clean state and no invalidation. Separately recheck the admitted protection context and report any unexpected source/dirty/history-version change; never copy a receipt's source into an observation or repair newer work. A caller's later edit requires its own fresh observation/basis.

Normal native display callbacks run inside the inherited Godot trust boundary. No universal exclusion of already-running malicious plugins is claimed. Known incompatible context or detected interference is not ignored; the [research scope correction](research.md#6-scope-correction-and-callback-limitation) does not waive any in-scope preservation check.

## Project Structure

### Documentation

```text
specs/003-open-project-gdscript/
├── spec.md
├── checklists/requirements.md
├── plan.md
├── research.md
├── data-model.md
├── contracts/
│   ├── open-api.md
│   ├── bridge-protocol.md
│   └── native-integration.md
└── quickstart.md
```

The approved [tasks.md](tasks.md) records granularity review, dependencies and implementation completion separately from PR delivery.

### Selected implementation boundaries

```text
mcp-server/
├── Cargo.toml / Cargo.lock / rust-toolchain.toml  # Existing package and lockfile
├── src/
│   ├── script_open.rs                          # Opening domain/state/outcome policy
│   ├── script_open/                           # Private attempt/evidence/outcome implementation
│   ├── runner/open.rs                         # Opening worker/supervisor integration
│   ├── runner/open/                           # Owned worker and parent supervisor
│   ├── runner/stock_validation/                # Reuse exact current-source validation
│   ├── bin/open-gdscript.rs                    # Small local CLI, no source input
│   ├── bridge.rs / bridge/wire/open.rs         # v3 authentication and opening DTOs
│   ├── bridge/wire/open/                      # Borrowed public output and codec regressions
│   ├── observation.rs / target.rs / project_fs.rs
│   └── existing edit modules                  # Preserve semantics during cutover
└── tests/                                     # Domain/boundary/CLI behavior

godot-addon/
├── addons/godot_agent_kit/
│   ├── script_open.gd                         # Thin native/collector lifecycle owner
│   ├── script_open_transport.gd               # Authenticated opening tuple/collector adapter
│   ├── plugin.gd / bridge.gd / observation.gd / export_guard.gd
│   └── native/editor_integration.gdextension   # Shared installed bundle naming
├── native/
│   ├── script_open.cpp                        # Native opening stages and guards
│   ├── script_document.cpp                    # Existing edit responsibility
│   ├── document_guard.hpp / document_guard.cpp # Only helpers with actual edit/open consumers
│   ├── session.cpp / native.hpp / extension.cpp
│   └── build.py                               # Generated exact public ABI; shared bundle
└── tests/
    ├── run_script_open.py
    ├── caller_open_acceptance.py              # Public opening campaigns
    ├── cumulative_open_acceptance.py          # Repeated opening/human/history transitions
    ├── composed_open_acceptance.py            # Product preparation for existing edit A–E
    ├── open_result_review.py                  # Result-only interpretation
    ├── opening_fixture_witness.py             # Shared fixture sources and witness projections
    ├── fixtures/script_open/                  # Owned opening preparations/witnesses
    └── existing observation/edit fixtures      # Reused lifecycle/capture/export support
```

**Structure decision:** Keep opening policy, transport, supervision and native effects separately owned. Do not append a second lifecycle machine to the already-large edit implementation/wire module. Extract only current shared document/source/descriptor guards with behavior-preserving existing consumers; keep writing/saved-state/history mutation in the edit module. Likewise, reuse or narrowly extract existing owned-child/framing cleanup helpers, not the edit validator/finalizer state machine or a generic backend framework.

The native artifact now serves two lifecycle domains. Rename the bundle/library/entry symbol to `editor_integration.gdextension`, `libeditor_integration.macos.arm64.dylib` and `editor_integration_library_init`, retaining the generic `godot_agent_kit_native` metadata key. Migrate build, plugin loader, manifest generation, native/fixture consumers, export checks, workflows and current native documentation together. Remove obsolete kit-owned bundle paths; preserve unrelated files and archived historical evidence. No duplicate native library or legacy entrypoint alias.

Expose only types/functions needed by current CLI/library callers: checked opening request, immutable outcome and runner entry. Keep reducer state, private context source, native receipts and DTO validation internal or crate-visible. Use one state/owner enum rather than independent lifecycle booleans or simultaneous edit/open attempt pointers. Shared/core names describe responsibility, not feature/task IDs.

## Coverage and Validation Plan

| Requirements | Owned implementation/evidence |
|---|---|
| FR-001–002 | Checked opening CLI/request, existing exact routing, v3 capability/auth; cold/cached/same-name routing and refusal fixtures. |
| FR-003–004 | Confined capture, cached-state/current-context preparation and boundary rechecks; dirty/divergent/cache/namespace/session/overlap barriers. |
| FR-005–007 | Native source-bound construction, non-takeover binding, current-source/profile/compiled-context guards, no-effect recognition; real source/history/pending-drag/refusal witnesses. |
| FR-008–009 | Independent post-observation/recheck and strict reducer; target equality/cleanliness versus limited already-open semantics; subsequent fresh-basis edit. |
| FR-010–014 | Typed stages/effects/outcomes, parent authorization/deadline, entered-slot lifetime, immutable terminal reduction; actual partial/unknown/cancel/disconnect/late-attempt cases. |
| FR-015 | Existing per-source observation limits and explicit new-open representation bounds; exact boundary/over-limit/empty/read-only/Unicode cases. |
| FR-016–017 | Unchanged observe/edit contracts, shared v3 cutover, private current-source handling, source-free logs/registry and all three export variants. |
| FR-018 / SC-001–007 | All 21 story scenarios, at least twenty opening requests (five genuinely closed, five dirty already-open), complete existing observation/edit A–E and applicable durability, timing and result-only review. |

Opening creates no source-history entry and claims no undoable tab-opening action. Native Undo/Redo must nevertheless preserve pre-existing human history, and actual existing edit apply/Undo/Redo/Save must remain correct after opening. Verify current dirty R == B as a positive case, conflicting R/B as a pre-effect refusal, and background dirty preservation. Pure tests cover actual outcomes, evidence invalidation, precedence and state transitions, not field forwarding or source-text assertions.

Use the existing private fixture framework and native `GAK_FIXTURE` build for controlled faults. No fault/evaluation operation is exposed in the production caller or authenticated product capabilities. Run GUI campaigns serially in owned projects with independent disk/editor/history witnesses, event-based waits and explicit deadlines. The supported head must pass Cargo fmt/Clippy/tests including doctests/rustdoc plus applicable native/workflow checks, as documented in [quickstart.md](quickstart.md).

## Research Evidence and Limits

[Research](research.md#4-candidate-and-executed-evidence) records **50 fixture requests and 20 opening invocations**, including rejected loader/focus/property-assignment routes, callback boundary probes, source-bound valid/invalid/read-only/cached opening and a dirty-current native Undo/Redo preservation control. Two additional invocations of each existing observation/edit caller tested composition: explicit source/path methods preserved unedited R and produced a verified edit with independently matching D/R/B. Six owned editor processes exited cleanly and all throwaway source/projects/binaries were removed.

The desktop was locked: no visible-window screenshot or new GUI acceptance was obtained. These are real stock-editor state/primitive and focused existing-caller observations, not new opening-caller, timing, full prior-history, namespace-race, export or complete durability proof. This environment limitation does not silently become a new infrastructure prerequisite; actual unlocked-GUI acceptance remains mandatory before feature completion. No product Rust/build/test suite was rerun for documentation-only planning.

The Source/GUI tests required by this plan remain future implementation evidence, not unresolved architecture choices. If a selected guard fails those tests, correct the implementation/design under the same specification before marking work complete. Do not preserve unsafe behavior or broaden the threat model merely to simplify proof.

## Complexity Tracking

No constitutional exception is selected.

| Mechanism | Concrete requirement/failure; simpler alternative | Current cost and why justified |
|---|---|---|
| Opening-specific policy and native state | Read-only observation cannot create B; edit has incompatible persistence/history semantics. Reuse existing boundaries instead of a generic lifecycle framework. | One current operation, explicit stages and tests; no speculative API. |
| Source-bound cold acquisition | Ordinary root loading invokes loaders and consumes a pathname rather than the exact capture. | Small native constructor/path/compile stages using existing ABI; avoids a loader proxy, arbitrary source input or engine patch. |
| Current-source admission | Native tab switching applies/validates current source and can process export state. Equality/focus flags alone do not establish safety; a source-only digest cannot bind helper evidence to changed identity, versions or effective warnings. | Bounded private context, one reused source-only validator and a guard fingerprint using the existing length-prefix/hash primitives. The narrow cross-language encoding costs deterministic vectors, avoids an added canonical-JSON dependency or repeated large context controls, and exposes no general serialization API, parser/service or permission class. |
| Shared guard extraction and bundle rename | Edit and open now consume the same native session/file/document boundary. | Narrow responsibility-based reuse and mechanical migration; not a broad refactor or second library. |
| Private v3/native revision 2 | A new capability changes an exact authenticated contract. Same-version extension or dual fallback is unsafe/needlessly costly. | One clean caller/addon/fixture/docs cutover; public observe/edit v1 preserved. |
| No universal callback isolation | Deliberately harmful already-running editor code is outside the inherited threat model. | Preserve documented limits and in-scope refusal/verification; no sandbox, registry monitor, plugin shutdown, extra approval or hosted-GUI ceremony. |

Before completion of each eventual implementation task, perform the repository's proportional cohesion/ownership/visibility/state review. Bundle directly required tests/docs with meaningful capability increments; do not derive trivial file/type-only tasks. Task derivation → granularity review → analysis remains required and is not run by this planning command.

## T002 implementation-shape and constitutional review (2026-09-30)

The public caller has current, separate owners rather than a second edit state
machine. `script_open` owns checked intent, immutable outcomes and pure ordered
attempt reduction; its private evidence/outcome declarations stay with that
policy. `runner/open` owns process wiring, the killable I/O worker and the parent
supervisor's authorization, deadline and helper lifetime. `bridge/wire/open`
owns strict private decoding and the borrowed public JSON projection.
`stock_validation/opening_context` validates the closed private guard shape.
The addon transport owns opening tuples/collector integration; `script_open.gd`
retains native-call/slot lifetime. The existing native effect implementation is
unchanged.

The large attempt/supervisor modules and pre-existing bridge were explicitly
reviewed for cohesion. The split keeps source-independent policy out of framing
and Godot calls without creating a generic lifecycle framework. State/phase
enums represent progression; native evidence booleans are validated facts, not
parallel lifecycle switches. Public visibility is limited to the request,
immutable result and runner consumed by the CLI/library boundary. Current
source, guards, attempts and receipts remain internal. Malformed or unavailable
evidence produces typed failure rather than safety-path panics. No speculative
API, enum family, extension hook or alternate mutation route was retained.

Review fixes preserved ordinary invalidation and the first decisive failure
across later native/connection faults, distinguished invalid document kinds from
malformed arguments, and attributed the latest snapshot to its own acquisition
interval. Live routing also exposed missing D being flattened into unavailable
editor evidence; a failing-before/passing-after real-caller regression now
protects the distinction. Acceptance code separates result-only interpretation,
independent fixture witnesses and public campaigns. The shared visible-window
helper is reused before a single ordinary Save shortcut: local Control focus
alone had not established the actual owned OS interaction surface.

Constitutional I–VI/X/XII review requires independent D/R/B and clean/unedited
facts, human-history preservation, native effects, effect-sensitive interruption
and the existing A–E/durability gates. VII/IX keep the distinct opening caller
composable with a fresh ordinary observation and unchanged editing; opening
grants no edit authority. VIII and XI retain tested export isolation and the
same public ABI/dependencies.

Under XIII, the present requirement is one bounded, guarded closed-to-open
operation, including cleanup when its worker dies. Reusing observation alone
cannot create a buffer; reusing edit's mutation machine would add persistence
and history effects. Letting the killable worker own the validation helper would
orphan private staging/processes on worker loss. The selected cost is one private
opening reducer/transport and parent-owned reuse of the existing validator,
framing, confinement and editor slot. Actual failure/cleanup tests justify that
cost now. No dependency, daemon, sandbox, approval mechanism, registry monitor
or CI trust boundary was added. The inherited local-endpoint and already-running
malicious-plugin limits remain unchanged.

## T003 implementation-shape and constitutional review (2026-09-30)

No production API, policy, native effect, dependency or workflow changed.
`cumulative_open_acceptance.py` owns repeated product requests and independently
witnessed human/history/identity transitions. `composed_open_acceptance.py`
owns product-opening preparation for the existing edit campaigns and the
closed-observation/edit and cache-consumer composition checks. The runner owns
group selection and evidence accounting; `all` dispatches each of its nine
groups once. Nested group records are not added again to invocation totals.

The existing edit fixture has one consumed preparation seam: the ordinary edit
runner keeps direct preparation, while the opening runner uses the actual
product caller and a separate fresh observation. Existing A–E implementations
are reused rather than copied. Sequence helpers are private; no speculative
production visibility, lifecycle flags, framework, extension points or future
API were introduced. The large pre-existing runner/edit harnesses retain their
responsibilities; current additions live in cohesive acceptance modules rather
than another growing production module.

The new composed history scenario exposed a real fixture-input precondition:
an opened Script can exist while the scene workspace is visible. Local CodeEdit
focus did not make the ordinary Save shortcut a script Save. The private human
Save action now selects the Script workspace and checks actual buffer visibility
before sending its single shortcut. The failing-before witness and passing
composed campaign are retained; no product Save, retry or source-repair path was
added. Earlier-history acceptance now requires the exact original source, not
merely text different from the latest edit.

Constitutional I–VI/X/XII are exercised through independent D/R/B, dirty and
Resource-edited witnesses, actual prior Undo/Redo, human-work preservation,
effect-aware refusal and composed durability. VII–IX retain existing product
boundaries and tooling-only fixture controls. XIII's concrete gap was cumulative
identity/history behavior after product opening; the simpler reused campaigns
plus one preparation seam close it without duplicate suites or new operational
infrastructure. Exact support, privacy/export and full-suite completion evidence
remain governed by the [quickstart](quickstart.md); this shape review alone is
not behavioral acceptance or a broader isolation claim.
