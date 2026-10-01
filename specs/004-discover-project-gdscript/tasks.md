---
description: "PR-sized implementation tasks for safe project GDScript discovery"
---

# Tasks: Safely Discover Project GDScripts

**Input:** [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md), [data-model.md](data-model.md), [caller v1](contracts/discovery-api.md), [private bridge v4](contracts/bridge-protocol.md), and [quickstart.md](quickstart.md).

**State:** Implementation — **1 / 3 complete**. Only **T001** is complete, with [required acceptance](quickstart.md#9-t001-scope-and-private-v4-acceptance-2026-10-01) and [shape/constitutional review](plan.md#t001-implementation-shape-and-constitutional-review-2026-10-01). Granularity/coverage and post-task consistency analysis passed; merged [planning PR #46](https://github.com/Peter-Tam/godot-agent-kit/pull/46) records analysis of `e7fb197af058c230751e59b2f05c3373adbfbf3b`. T002/T003 remain pending; no subsequent task is selected or authorized by this single-task execution. No discovery inventory support is claimed; Roadmap Phase 1 remains in progress.

**Organization:** Three coherent increments: authenticated read-only editor scope with the coordinated v4 cutover; the complete public discovery caller and its consumed core; cumulative repeated-use/composed-workflow acceptance. Each of the four P1 stories has its own phase and independent acceptance. US2, US3 and first-use US4 safety are inseparable from the first caller and belong to T002, not separate later safety/test PRs. Story references below do not create duplicate task checkboxes.

## Working Agreement and Format

- Each `- [ ] Tnnn [P?] [USn?] Description with exact file paths` row is one Spec Kit implementation task and one PR. Its indented scope, tests, verbatim constraints and acceptance form the executable task description. Internal steps, commands, files and subagents are not extra tasks.
- Follow the [constitution](../../.specify/memory/constitution.md), [AGENTS.md](../../AGENTS.md), approved feature artifacts and this task list. Complete granularity review before `/speckit.analyze`; resolve actual analysis blockers before implementation. Do not invent a stronger threat model or use missing hypothetical proof as a product blocker.
- Select exactly one dependency-ready ID. Preserve existing changes; fetch updated `main`, branch `task/<task-id>-<description>` from the authorized base, and verify `SPECIFY_FEATURE_DIRECTORY` points to `specs/004-discover-project-gdscript` before invoking helpers. Task branch names alone do not select the feature. Inspect intended/staged changes, stage explicit paths, commit, push, open the dedicated PR, report evidence and STOP. No automatic merge or next task; stacking requires explicit authorization. After confirmed merge, safely remove remote/local source branches under repository policy.
- Mark only the selected task `[X]` when its acceptance, required verification and implementation-shape review pass on its delivery head, independently of PR review/merge. Update this file and `PROJECT_STATUS.md` for material lifecycle changes. Prerequisite completion does not bypass normal PR-merge sequencing.
- **Tests are required by FR-016 and the constitutional safety/boundary gates.** Write meaningful failing-before regressions where executable, implement the behavior, and deliver passing tests, actual smoke and directly necessary docs together. No tests-only precursor PR. Test consumer-visible state, evidence, limits and failures, not copied fields, wiring, mock echoes or source text. Universal refusal/limited results do not satisfy positive discovery.
- Retain Rust core semantics, adapter-owned CLI/wire/process concerns and addon-owned Godot facts. No source reads, source hashes, script loads, document actions, scans, parsing, execution, history participation or repair in discovery. An inventory is not D/R/B evidence, source revision, persistent identity or authorization for a later operation.
- Reuse the locked dependencies, local registry/authentication, confinement, owned worker and existing operation slot. No new crate, generic query/traversal framework, parser, daemon, index, watcher, queue, retry service, approval class, CI topology, MCP, platform or native ABI. No speculative public APIs or compatibility shims; public observe/edit/open v1 and native revision 2 remain unchanged.
- Review each materially changed module's responsibility, current-consumer visibility, lifecycle states, speculative declarations and recoverable evidence errors before completion. Large existing `observation.rs`, `runner.rs`, `bridge/wire.rs` and addon `bridge.gd` receive only necessary seams/dispatch; use the planned feature-owned modules rather than adding another unrelated responsibility. Proportionate cleanup introduced by this task belongs in its PR, not a future refactor task.
- Each task owns its current contract/operator changes and reproducible evidence in `specs/004-discover-project-gdscript/quickstart.md`; update `plan.md` with applicable constitutional/shape review and `research.md` only for new findings. Preserve historical acceptance; amend only current installation/reproduction notices. Do not commit credentials, binaries, generated editor state or temporary probe scaffolds.

### Paths, Environment and Shared Verification

All paths are repository-relative. New files are identified with their first real consumer; existing packages/directories need no setup task. A simple private responsibility-based split beneath a planned owner is allowed when justified by cohesion; no one-type-per-file target or generic framework is implied.

Use existing Rust 1.98.1/edition 2021, Python 3.10+, the current locked dependencies and official Godot `4.7.2.stable.official.ed1daf0bf`, commit `ed1daf0bf001b61586d9930840f2f1394092c079`, executable SHA-256 `c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf`, macOS 26.6.2 arm64. The existing matched native revision-2 bundle is required for composed edit/open evidence, not as a new discovery capability prerequisite. No other build/platform is claimed supported.

For affected Rust work, from `mcp-server/`:

```sh
cargo +1.98.1 fmt --all -- --check
cargo +1.98.1 clippy --all-targets --locked -- -D warnings
cargo +1.98.1 test --locked
cargo +1.98.1 doc --no-deps --locked
```

Default tests include doctests. Build actual current consumers with locked resolution using [quickstart §2](quickstart.md#2-implementation-checks-and-build); do not request the discovery binary before T002 creates it. Run existing workflow/campaign/native-build checks only when those surfaces change. A transport-only cutover does not require a native rebuild if the unchanged source/ABI/provenance still match.

Actual owned visible-editor and process evidence is mandatory for changed boundaries. Run the affected scenario first during development; collect all integrated results before broader checks. GUI groups run serially with fresh private artifacts, explicit deadlines, independent witnesses, owned-window proof and cleanup. The v4 shared authentication/operation-slot change requires complete existing observation/edit/open suites in T001; T002's shared selector/caller integration must preserve those full suites. T003 records the complete final-head discovery plus affected old-suite gate. Planning probes, old feature acceptance, headless checks and focused passes cannot stand in for these gates.

## Phase 1: Setup — Reuse the Existing Foundation

**Purpose:** Reuse completed Features 001–003: Cargo package, locked toolchain/dependencies, addon, private registry, authenticated bridge, confinement, native integration, owned-worker and real-editor harnesses. No initialization, directory, formatter, dependency or standalone documentation checkbox is justified.

**Entry criteria:** Reviewed feature artifacts, required post-granularity analysis, and the selected task's actual authorized validation environment. No implementation begins through this planning command. Any necessary fixture/build setup belongs with its first consumer.

## Phase 2: Foundational — Authenticated Read-Only Editor Scope

**Goal:** Establish an independently verifiable, path-free editor-context capability through the existing authenticated slot, with all existing callers functional across the private-v4 cutover. No inventory or public discovery command is introduced yet.

- [X] T001 Deliver the authenticated read-only discovery scope owner and atomic private-v4 cutover in `godot-addon/addons/godot_agent_kit/script_discovery.gd`, `godot-addon/addons/godot_agent_kit/bridge.gd`, `godot-addon/addons/godot_agent_kit/plugin.gd` and `mcp-server/src/bridge.rs`, bundling real scope/compatibility tests in `godot-addon/tests/discovery_scope_acceptance.py`, `godot-addon/tests/run_observation.py` and `mcp-server/tests/bridge_boundary.rs` with migration evidence in `specs/004-discover-project-gdscript/quickstart.md`.
  **Depends on:** Completed Features 001–003 and the reviewed-artifact/analysis entry criteria; no earlier Feature 004 task. This is one real editor-boundary/cutover PR, not independent version, capability-bit, getter or fixture PRs.

  **Deliverable and current consumer:** Create the thin `script_discovery.gd` owner and implement every source-free `discover_begin`, `discover_recheck`, `discover_finish` and `discover_abort` tuple/response in `contracts/bridge-protocol.md`. The existing authenticated Python/fixture bridge exercises this real owner before a public caller exists. Keep feature-specific live probes in new `godot-addon/tests/discovery_scope_acceptance.py`, consumed by the existing `session-boundary` and `executor-boundary` groups in `godot-addon/tests/run_observation.py`; reuse `godot-addon/tests/fixture_bridge.gd` and `godot-addon/tests/fixtures/observation/fixture_driver.gd` for private preparation/control. Do not add public Rust discovery models, an unused worker codec, walker, no-op binary or public inventory scenario in this task.

  **Scope facts and ownership:** Obtain effective settings directory only through `EditorInterface.get_editor_paths().get_project_settings_dir()`, accepting exactly `res://.godot/editor` or `res://godot/editor`. Observe public scan/import facts, a session-local filesystem-change epoch and editor-local stamps/expiry. No project configuration/source read, cached file-tree traversal, document/cache/native inspection, rescan, wait-for-idle or editor action. Claim the existing `_active` slot; wrong-owner, duplicate, expired and late requests cannot revive or release another attempt. Release this read-only owner on finish/abort/channel loss/expiry/disable/exit and disconnect registrations/signals. Preserve entered-native-owner retention for current open/edit requests.

  **Coordinated v4 migration:** Migrate all current descriptor/hello/request/response checks and exact HMAC transcript consumers together. Append exactly `discover_gdscripts` after the seven existing Boolean capability fields, use the `godot-agent-kit/editor-bridge/v4` domain and retain separate server/client/finish roles, typed length-prefix framing, nonces and native metadata authentication. Update `mcp-server/src/bridge.rs`, `mcp-server/src/bridge/wire.rs`, current `mcp-server/src/bridge/wire/` consumers, `godot-addon/addons/godot_agent_kit/bridge.gd`, `script_open_transport.gd`, `script_open.gd`, `script_edit.gd`, `observation.gd` and `plugin.gd` where envelopes/ownership change. Migrate existing Rust bridge/confinement/caller fixtures and actual Python/GDScript HMAC/version consumers in `godot-addon/tests/run_observation.py`, `run_script_open.py`, `run_script_edit.py`, `fixture_bridge.gd` and their fixtures. Locate remaining current v3 consumers before cutover; do not mechanically rewrite archived evidence. Reject v3 without fallback or substitute-session selection. Discovery capability is true only with the matching scope owner and supported getters/profile, independently of native mutation availability; native revision stays 0 or 2 under its existing contract.

  **Meaningful boundary tests:** Extend `mcp-server/tests/bridge_boundary.rs` and `mcp-server/tests/confinement.rs` for actual cross-language v4 authentication and v3, missing/extra/duplicate/wrong-type capability, changed capability/native metadata, wrong-role, replay/reflection and identity rejection. A false discovery capability must not break otherwise supported existing operations. Exercise exact path-free tuple arity/types, depth 32, preallocation rejection, 4 KiB discovery control bounds, remaining begin budget 1–4500 ms and same-request/session/project attribution. Test actual effective `.godot` and visible `godot` contexts, including a live unsaved setting change that must not replace the effective getter value; unavailable/unsupported context, active scan/import, epoch change, stage order, expiry/disable/disconnect and busy interactions in both directions. Acknowledgment is not coverage evidence; fixture getter echoes are insufficient.

  **Independent acceptance:** Pass the focused existing `session-boundary`/`executor-boundary` groups with the real new scope owner, then complete unfiltered observation, edit and opening suites under quickstart §6 on the integrated cutover head. Existing public deadlines/contracts and A–E/history/durability remain satisfied. Independently witness source/selection/dirty/history preservation for these read-only context probes; exercise actual history, not just a flag. Check source/path/credential sentinels, registry/log redaction and enabled/disabled/hook-only export exclusion/execution for the new owner and private controls. Extend `godot-addon/addons/godot_agent_kit/export_guard.gd` only if the existing generic exclusion is insufficient. No inventory support claim follows a scope-only pass.

  **Documentation and delivery:** Update current private-version/session-restart notices in `specs/001-observe-gdscript-state/contracts/bridge-protocol.md`, `specs/001-observe-gdscript-state/quickstart.md`, `specs/002-edit-open-gdscript/contracts/bridge-protocol.md`, `specs/002-edit-open-gdscript/quickstart.md`, `specs/003-open-project-gdscript/contracts/bridge-protocol.md`, `specs/003-open-project-gdscript/quickstart.md`, `godot-addon/native/README.md` and `.github/README.md` where current instructions change, preserving historical v2/v3 evidence. Record actual scope-fixture commands/evidence and partial implementation status in this feature's bridge contract, plan and quickstart. Rebuild/install matching Rust/addon peers and deliberately restart owned editors for fresh v4 descriptors; native revision/artifact names do not change. Mark only T001 complete after constitutional/shape review, publish its dedicated PR and stop.

  **EditorScopeContext constraints (verbatim from data-model.md):**

  > `EditorScopeContext` contains request/project/session binding, exact policy/data-directory, editor collection stamp, `filesystem_changed` epoch, `scanning`, `importing` and owner expiry tick. The Boolean fields are independent external editor facts, not lifecycle flags. Epoch is local invalidation evidence, not a project revision or a cache-freshness proof.
  >
  > Begin and final context must match policy/data directory/session and show no scan/import or epoch invalidation for completeness. Scan/import activity returns a limited result with `editor_inventory_busy`; do not wait/retry or enumerate solely to conceal it. Missing/unsupported policy context refuses with `unsupported_visibility_policy` before traversal. A policy/data-directory change clears inventory because the collection no longer has the admitted membership basis. An epoch change with unchanged scope permits independently rechecked paths only as limited `editor_inventory_changed` evidence.

  **Constraint ownership:** T001 implements the producer/owner and its observed facts, not a competing addon terminal reducer. T002 consumes these facts and owns the public limited/refused/inventory interpretation. All exact fields, nullability, enums, frame sizes, transcript ordering and cleanup rules in the private bridge contract apply now; native mutation availability is not invented as a discovery requirement.

## Phase 3: User Story 1 — Find an Exact Script Without Knowing Its Path (Priority: P1)

**Goal:** One useful bounded public script-location inventory, including closed scripts, with all inseparable confinement, privacy, freshness, interruption and non-interference behavior in its first delivery.

**Independent test:** Through the actual caller, compare exact results with an independently prepared inventory of at least 100 scripts across at least ten folders, including ordinary addons, repeated basenames, never-opened files, empty/invalid/read-only/tool/large-source/case variants. Verify native visibility parity and both genuinely empty and hidden/ignored-only empty scopes; choose a returned eligible path for fresh observation and opening without manual path lookup.

- [ ] T002 [US1] Deliver the complete bounded discovery caller and consumed core in `mcp-server/src/script_discovery.rs`, `mcp-server/src/project_fs/discovery.rs`, `mcp-server/src/runner/discovery.rs`, `mcp-server/src/runner/discovery/worker.rs`, `mcp-server/src/bridge/wire/discovery.rs` and `mcp-server/src/bin/discover-gdscripts.rs`, bundling contract/process/real-editor coverage in `mcp-server/tests/script_discovery_contract.rs`, `mcp-server/tests/script_discovery_caller.rs` and `godot-addon/tests/run_script_discovery.py` with `specs/004-discover-project-gdscript/quickstart.md`.
  **Depends on:** Completed T001 and its merged PR unless stacking is explicitly authorized. Do not reimplement its authenticated scope owner or bypass a failed v4 prerequisite. This task owns US1.1–US1.4, all US2/US3 acceptance, US4.1/US4.4 and first-use US4.3 composition; T003 owns the additional cumulative sequence/matrix, not missing safety behavior.

  **Tests → consumed models → acquisition → caller → integration:** Add meaningful contract/precedence and real-filesystem/process regressions before the behavior they protect where executable; keep private-state tests inside the owning module rather than exporting internals for tests. Implement the checked request, evidence invariants and immutable outcome in `script_discovery.rs`; the actual CLI/library runner consumes them in this task. Add only necessary declarations in `mcp-server/src/lib.rs`, `runner.rs`, `bridge/wire.rs` and the binary entry in `mcp-server/Cargo.toml`. Keep the lockfile/dependencies unchanged unless a separately justified requirement establishes otherwise; no additional dependency is selected here.

  **Source-free selection:** Privately factor the existing root validation in `mcp-server/src/project_fs.rs` and the common authentication algorithm in `mcp-server/src/target.rs`. Both discovery and the existing real-script resolver consume one implementation. Preserve public `target::resolve` and its genuine current observation/open/edit consumers; discovery uses no fabricated locator or optionalized document contract. Retain canonical/root-handle identity, ancestor/owner/mode/deny-only-ACL/no-follow checks, denial/ambiguity/unresolved-candidate precedence and ended-session behavior. No project names are enumerated before unique authenticated selection and supported scope admission. Run LSP references before changing exported symbols and migrate actual affected callers/tests without speculative public routing APIs.

  **Fresh confined metadata:** Implement the complete data-model §5 algorithm in feature-owned `project_fs/discovery.rs`, not an ambient recursive walker or `read_disk` source acquisition. Apply exact candidate-native dot-name, effective-data, regular `.gdignore`/nested-`project.godot`, standalone regular-file and ASCII-case-insensitive final `gd` rules. Root markers do not exclude the root; literal marker lookup follows the actual filesystem's case behavior. OS-hidden flags do not independently hide files on this pinned candidate. VCS/export filters are not consulted. Distinguish absent, unsafe and unavailable markers without reading contents; symlink destinations are neither followed nor printed. Bound encountered names before allocation/filtering; retain depth-bounded handles and bounded witnesses, not one descriptor per visited directory. Recheck root/access/named chains, directory membership, file identity and exclusions; source-only mtime/content change is not path invalidation. Remove known-invalidated entries and never erase an earlier gap with a later matching sample.

  **Worker, codec and reduction:** Use T001's real begin/recheck/finish/abort exchange and the existing same-executable Unix-socketpair child ownership. Implement the closed `selected`, `scope`, `entries`, `invalidate`, `rechecked`, `failed`, `done` IPC events from the bridge contract, with immutable binding, ordinal/state/stamp/identity/bounds validation before acceptance. The parent, not worker/addon, owns the original cutoff and typed terminal precedence. Reject malformed, duplicate, oversized, out-of-order, copied or late evidence without adding paths or upgrading a terminal result. Keep separate editor and caller clock domains, assign receipt time once, and retain earlier batches only under the exact suppression/validity rules below. On timeout/cancel stop/reap only the owned read-only worker and close its channel without awaiting blocked I/O or an addon acknowledgment past cutoff. Do not change native entered-owner retention or kill/restart the developer editor.

  **Caller contract:** Implement `discover-gdscripts --registry <absolute> --project <absolute> [--session <exact>]`, `--help` and `--version` exactly as caller v1; reuse the observer's registry bootstrap. Start the clock before parsing, use a fresh 32-hex request ID, 4500 ms operation cutoff and 500 ms terminal reserve with stdout drained. Unknown/duplicate/positional/invalid/missing flags yield structured `refused/invalid_request` without reflecting unvalidated input. No script, prefix, query, offline, force, timeout override, continuation or executable flag. Reject ordinary shell entry to private worker mode without owned IPC. Handle SIGINT/SIGTERM using existing bounded cancellation. Emit one bounded JSON object plus newline with all required nullable keys, fixed diagnostics and exits complete 0 / limited 2 / refused 3 / interrupted 4 / unexpected host failure 1. Stderr/registry never contain inventory, raw OS errors, source, proofs or credentials; no automatic reconnect/retry, alternate editor startup or disk-only fallback.

  **Deterministic and actual boundary proof:** Bundle tests in `mcp-server/tests/script_discovery_contract.rs`, `script_discovery_caller.rs`, `confinement.rs` and `bridge_boundary.rs`/feature-owned submodules. Cover actual exhaustion versus false empty, refusal suppression versus safe limited/earlier retention, simultaneous-failure precedence, scope/session/root/stamp mismatch, unknown/duplicate fields, exact and one-over every resource bound, diagnostic overflow, distinct hard-link names, case/Unicode/path rejection, and terminal serialization limits. Use real capability-rooted directories, ACL/permission controls, deterministic namespace/marker replacement barriers and a genuinely stalled owned worker. Cancellation after a checked batch must preserve only attributable earlier evidence and deliver within five seconds; release late frames afterward and prove no replay/upgrade or delayed effect. Do not create public witness APIs or source-text assertions merely to make tests possible.

  **Live caller runner:** Create `godot-addon/tests/run_script_discovery.py` and private `godot-addon/tests/fixtures/script_discovery/fixture_driver.gd`/owned fixture sources, reusing existing observation/project/window/history/export facilities and T001's boundary probes. Implement the real public-caller `inventory`, `routing`, `coverage`, `interruption`, `readonly` and `privacy-export` groups in quickstart §4–§5, with result-only interpretation and ≤5-second external timing in every controlled case. The six groups must be runnable individually; do not expose incomplete/no-op `sequential`, `composed` or `all` groups. Independently expected inventories must include all current visible addon/fixture scripts, never silently filter tooling paths out of product results. Actual GUI witnesses, not a private-walker invocation, establish public acceptance.

  **Required first-caller acceptance:** US2 and US3 phases below are T002 gates. Prove both effective data modes, root versus child markers, native literal lookup, visibility/empty controls, legal exact names and unsupported representation, stale-cache-misses-new-file control, create/rename/remove freshness, root/parent/file/marker replacement and safe earlier evidence. Exercise missing/ambiguous/ended/replaced/denied targets, blocked worker/editor, cancel/disconnect/expiry/disable, same-slot observation/open/edit overlap in both directions, and no late replay. Independently preserve clean, dirty, equal-text-dirty, divergent, non-selected and unloaded-script controls; replay actual prior Undo/Redo and observe no delayed effects. A later use of a removed/replaced path gets fresh existing checks; known-document observation for ignored/deleted-live-buffer paths retains its existing semantics.

  **Composition, privacy and compatibility before delivery:** Use a returned closed eligible path for actual fresh observation → product opening → fresh-basis existing editing, and a separate discovered dirty path for safe edit refusal with text/history preserved. Include applicable existing A–E/Save/reopen/reparse/rescan/runtime smoke, then full existing observation/edit/open suites because shared selection and authenticated operation integration changed. Run selected/other-project/source/credential sentinels through authorized, ambiguous, denied and interrupted discovery; only the authorized requested result contains inventory. Inspect and run enabled/disabled/hook-only exports, excluding the new discovery owner and all fixtures/private state. No known non-interference, privacy/export or existing-operation regression is deferred to T003.

  **Current build/campaign consumers and docs:** Add the real discovery binary to `.github/workflows/ci.yml`'s existing locked build list. Register the existing `godot-addon/tests/run_editor_campaign.py` `--suite discovery` / `--discoverer` and actual input fingerprints only alongside this functioning runner; extend meaningful execution/resume/invalidation coverage in `godot-addon/tests/test_editor_campaign.py` and affected `.github/tests/test_live_editor_execution.py` where required. Until T003 supplies every discovery group, campaign-wide `--suite all` must not claim full discovery acceptance; reject a requested unavailable full discovery campaign explicitly rather than silently run a subset. Update `.github/README.md`, caller/bridge contract implementation status, exact focused commands and evidence in the feature quickstart, and project/task lifecycle truth. Mark only T002 complete after constitutional/shape review; the whole feature remains incomplete until T003.

  **Reused value and visibility constraints (verbatim from data-model.md):**

  > - Request ID: existing safe ASCII syntax, at most 64 bytes; CLI generates fresh 32-hex correlation IDs, not idempotency keys.
  > - Session ID: exact 32 lowercase hex characters, bound to the authenticated editor lifetime.
  > - Project root: required absolute input, existing lexical validation, at most 1024 UTF-8 bytes; selected identity is canonical root plus device/inode. Display names, focus and cwd do not select a project.
  > - Entry locator: exact `res://` path, at most 2048 UTF-8 bytes, no empty/dot/dotdot components, backslash, controls, query/fragment/colon decorations, embedded subresource or lossy Unicode conversion. Preserve legal Unicode and case.
  > - Discovery recognizes the final `gd` extension ASCII-case-insensitively. `ResourcePath` validates round-trip syntax, not eligibility for another operation; a case-variant path may remain unsupported by an existing caller's unchanged script profile. Do not change existing `ResourcePath::kind` or widen opening/editing as part of this feature.
  > - Device/inode/ticks/counters use checked decimal strings where existing wire contracts do, never lossy JSON numbers or editor pointer values.
  > - Editor collection ticks and caller monotonic elapsed time have separate clock domains. Receipt time is assigned once per validated frame. Wall-clock endpoints are descriptive; monotonic elapsed time controls the deadline.
  >
  > Public Rust declarations are limited to the request, immutable outcome and fields/accessors consumed by the actual CLI/library runner. Attempt state, filesystem witnesses, scope leases, codecs and project-only routing internals remain crate-private. No generic discovery trait or speculative variant is introduced.

  **Fixed supported profile (verbatim from data-model.md):**

  > | Bound | Value / behavior |
  > |---|---|
  > | Operation cutoff / terminal deadline | 4500 ms / 5000 ms from clock-before-parse; reserve 500 ms for terminal delivery |
  > | Returned eligible entries | 1024; the first additional eligible entry creates `entry_limit`, not silent truncation |
  > | Visited directories | 1024 including root; further otherwise eligible directories create `directory_limit` |
  > | Directory depth | 64 below root; deeper in-scope subtrees create `depth_limit` |
  > | Visited directory entries | 16384 across acquisition; count every encountered name before filtering or allocating retained name storage |
  > | Detailed diagnostics | At most 63, plus one fixed `additional_gaps` aggregate carrying the omitted count when needed |
  > | Worker progress batch | At most 64 entries and 512 KiB encoded; no batch is accepted before its binding and metadata checks |
  > | Final result | At most 6 MiB UTF-8 JSON plus newline, below the existing 12 MiB worker/response ceiling |
  > | Public path/root/request syntax | Existing 2048/1024/64-byte limits above |
  > | Public query modes | Whole selected project only; no paging, filters, include-ignored, source search or configurable safety limits |
  >
  > Bounds apply before allocation/growth, including per-directory name collection; do not first read/sort an unbounded directory. Visit order may follow the filesystem; final retained entries are unique and sorted by exact UTF-8 bytes. A limited result is not guaranteed to be a stable prefix and provides no continuation token. Exactly reaching a limit is not itself proof of exhaustion or overflow: complete requires actually exhausting the scope within every bound. Final serialization size is checked before emission; reaching the result limit retains only complete entries, reports `result_limit` and cannot become complete. The supervisor rejects a worker violating bounds rather than trusting its truncation claim.

  **DiscoveryRequest constraints (verbatim from data-model.md):**

  > Fields: `request_id`, `project_root`, nullable `session_id`. No script, source, target executable, directory prefix, glob, timeout override or permission token.
  >
  > Validation is structural before routing; access/identity is checked by the existing common resolver. A syntactically valid request is not authenticated selection. A malformed request produces `refused/invalid_request` without echoing unvalidated fields.

  **DiscoveryTarget constraints (verbatim from data-model.md):**

  > Fields: `request_id`, canonical `project_root`, `project_file_id {device,inode}`, `session_id`, `godot_version {version,hash}`. It contains no script locator, endpoint, secret, nonce or native pointer.
  >
  > It must match the request, exactly one authenticated candidate, the pinned root handle and selected lifetime. An explicit session mismatch or replacement never binds to another session. Existing `Selection`-style feedback may contain only authenticated public candidate session IDs for the requested project and the missing `session_id` selector, never candidate inventory.

  **DiscoveryScope constraints (verbatim from data-model.md):**

  > Fields: `policy` equal to `godot_project_files_v1`, `project_data_directory` equal to `res://.godot` or `res://godot`, and `exclusions` equal to the fixed ordered identifiers `dot_names`, `project_data_directory`, `gdignore_subtrees`, `nested_projects`, `non_gdscript_files`, `non_regular_files`, `embedded_or_unsaved_documents`.
  >
  > The policy is tied by `DiscoveryTarget` to the exact supported Godot/macOS environment, not an unverified portability claim. It follows [research §2](research.md#2-exact-visibility-policy). A symlink/access/representation failure is a coverage gap, not a silently expanded exclusion policy. No list of hidden/ignored filenames is needed to explain scope. VCS ignore and production-export rules are not consulted.

  **DiscoveryEntry constraints (verbatim from data-model.md):**

  > Public representation: one exact `ResourcePath` string in `entries`. No per-file source, hash, dirty/open/parse state, persistent document ID, edit eligibility, display-name alias or redundant basename. Paths are distinct even when they reference the same underlying regular file through different valid names. A subsequent request must independently resolve current identity and safety.

  **DiscoveryInventory constraints (verbatim from data-model.md):**

  > Required fields:
  >
  > - `scope`: the observed `DiscoveryScope`.
  > - `entries`: bounded exact path strings, unique and sorted at terminal delivery.
  > - `collection`: caller-domain metadata collection interval/stamp, never copied editor time.
  > - `coverage`: `complete`, `partial` or `not_started`; describes the declared scope, not whether entries are nonempty.
  > - `validity`: `observed` or `earlier_observation`. The latter is used when interruption prevents current final checks; it never claims current editor liveness or complete coverage.
  > - `consistency`: `{recheck: "completed"|"unavailable", stability: "unknown"|"changed", atomic: false}`. Completed rechecks do not prove a frozen filesystem; there is no `stable` or atomic-success value.
  > - `visited_entries`, `visited_directories`: bounded acquisition counters, not a promised total project size.
  >
  > Known-invalidated entries are removed rather than exposed as current locators. Diagnostics retain safe attributable affected scope/reasons. A valid empty list with `partial` or `not_started` does not mean an empty project. A complete inventory requires `validity=observed`, `recheck=completed`, `stability=unknown`, actual exhausted scope and no gap or terminal failure. Unknown stability means no stronger stability guarantee was established, not permission to ignore detected invalidation.

  **DiscoveryDiagnostic constraints (verbatim from data-model.md):**

  > Fields: fixed `code`, `stage`, nullable `scope`, fixed `message`, fixed `action`, and nullable bounded `omitted_count` used only by `additional_gaps`. Stages: `validate_request`, `resolve_target`, `authenticate`, `begin_scope`, `enumerate`, `recheck`, `finalize`.
  >
  > `scope` is either `res://` for the selected root, a safely attributed representable project-relative path/subtree, or null. Never print an outside redirect target, undecodable filename bytes, raw OS error, private registry path, credential or candidate project inventory. An unsupported component is described at the nearest safe parent or with null scope. User-owned source text is never diagnostic data.

  **DiscoveryOutcome constraints (verbatim from data-model.md):**

  > Required public fields: `schema_version: 1`, `request_id`, `outcome`, `interval`, nullable `requested_target`, nullable `resolved_target`, nullable `inventory`, `diagnostics`, nullable `selection`.
  >
  > `requested_target` is `{project_root,session_id}` only after lexical validation, otherwise null. `resolved_target` is the authenticated `DiscoveryTarget`, never a guessed target. `inventory` is null before usable scope/evidence or when suppression is required. Required nullable fields are present, not omitted. Outcome and exit meanings are in [the caller contract](contracts/discovery-api.md#3-terminal-results).

  **Private attempt and SelectedEditor constraints (verbatim from data-model.md):**

  > The private state machine is `starting → selected → scoped → enumerating → rechecking → terminal`; a failure may enter terminal from any earlier state. No source/lifecycle application state or rollback model belongs here. A typed state/ordered-event reducer prevents impossible combinations and rejects duplicate, late or out-of-order events.
  >
  > `SelectedEditor` owns the authenticated socket, project directory, original/canonical root identity, engine/session/capability evidence and request binding. It is produced by the common selection algorithm; it cannot be constructed from a path string alone or transferred between request IDs.

  **Private filesystem evidence and coverage algorithm (verbatim from data-model.md):**

  > Private witnesses contain only what the current request needs:
  >
  > - Root: original/canonical namespace identity, retained directory handle and existing ancestor/owner/mode/ACL checks.
  > - Directory: exact relative components, device/inode/type, applicable ownership/mode/ACL, pre/post membership metadata (mtime/ctime), and the namespace chain observed for that path.
  > - Exclusion markers: safely established absence or regular-file identity/access evidence for literal `.gdignore` and `project.godot` in child directories. Marker contents are never read. Root marker lookup is not a subtree exclusion.
  > - File candidate: exact locator, device/inode/regular-file type and applicable access/namespace evidence. Do not collect source bytes, source hash or source mtime as a revision; a source-only edit does not change path membership.
  >
  > Algorithm obligations:
  >
  > 1. Authenticate unique selection and obtain supported idle editor scope before project inventory access. Recheck root identity/access around collection; no offline fallback.
  > 2. Enumerate only through validated directory handles. Bound each encountered name before allocation/filtering. Skip candidate-native dot names and the effective data subtree without traversing them.
  > 3. For visible directories, check/open one component with the existing no-follow/nonblocking/identity/ACL pattern before enumeration. Refuse unsafe root binding; local unsafe/unreadable components become explicit gaps. Never follow a symlink to determine whether it is a directory or expose its target. Do not treat failed stat/open as nonexistent.
  > 4. Check child marker names using capability-relative metadata and the native filesystem's literal lookup behavior. A safe regular marker excludes the child subtree. Directory/special-file markers do not satisfy regular-file existence; aliases/unavailable checks make the subtree a gap rather than guessed visible/ignored. Recheck exclusion evidence so marker removal/replacement during the interval cannot certify a complete list.
  > 5. A regular, representable, in-scope GDScript candidate must satisfy the current namespace/access boundary before entering a progress batch. Reuse necessary descriptor-based metadata/ACL checks without reading file content. Non-GDScript regular files and known non-regular non-directory objects are not script entries. Unsafe visible symlinks are gaps because their possible subtree cannot be established without following them.
  > 6. Retain only depth-bounded open traversal handles. Keep bounded witnesses for final rooted reopening/recheck, not 1024 open directories. Before accepting each batch, validate its root/namespace attribution; a batch is not a final complete result.
  > 7. At final recheck, verify root, traversed directory membership/access, exclusion markers, eligible file identities/access and their named component chains. Known affected subtrees remove their entries and add `namespace_changed`; exhausted enumeration cannot erase that invalidation. Unavailable checks add explicit gaps and prevent complete coverage. No automatic retry restores a complete label.
  > 8. Recheck editor scope/lifetime over the same authenticated channel. Only then can the common reducer report complete. Namespace and editor stamps remain separate interval evidence, not simultaneous atomic proof.
  >
  > A blocked call is bounded by parent supervision, not an assumption that every syscall returns. A worker may be terminated; earlier checked batches can survive only under the terminal retention rules below. This design does not claim linearizable enumeration against arbitrary non-cooperating writers between checks.

  **Terminal precedence, reasons and retention (verbatim from data-model.md):**

  > Apply the following order when several facts are known; retain all permissible diagnostic distinctions. Later success evidence never overrides an earlier terminal decision.
  >
  > | Priority | Outcome / representative reasons | Retention |
  > |---|---|---|
  > | 1 | `refused`: `denied_access`, `unsafe_registry`, `authentication_failed`, `out_of_project`, `project_identity_changed` | Clear inventory; no unauthorized target/path evidence. |
  > | 2 | `refused`: `ambiguous_target` | Clear inventory; only safe session-selection feedback. |
  > | 3 | `refused`: `invalid_request`, `invalid_project`, `unsupported_version`, `unsupported_capability`, `unsupported_visibility_policy`, `editor_unavailable`, `busy` | No inventory from an unstarted/unadmitted attempt. |
  > | 4 | `interrupted`: `protocol_error`, then `cancelled`, `session_replaced`, `disconnected_editor`, `timeout` | Discard rejected payloads. Clear inventory on known binding/session replacement; otherwise independently validated earlier batches may remain as `earlier_observation`, never complete/current liveness. |
  > | 5 | `limited_listing`: local gap or unavailable/invalidated recheck | Keep only safely attributable entries; coverage partial/not-started, reasons explicit. |
  > | 6 | `complete_listing` | Exhausted scope, every required check completed, no known gap/invalidation; zero entries is valid. |
  >
  > Local-gap reasons include `directory_unreadable`, `entry_unreadable`, `unsafe_entry`, `unsupported_path`, `unsupported_marker`, `namespace_changed`, `scope_changed`, `editor_inventory_busy`, `editor_inventory_changed`, `recheck_unavailable`, `entry_limit`, `directory_limit`, `depth_limit`, `work_limit`, `result_limit`, `additional_gaps`. A missing/replaced file encountered during acquisition is `namespace_changed`, not proof it never existed. A directory-local denial does not automatically discard other safe entries; a root/authentication denial does.
  >
  > `scope_changed` clears inventory and records the lost admitted scope; there is no new-policy rewalk in the same request. For interrupted earlier evidence, the result identifies the original interval and unresolved checks. An earlier correctly authorized path is not relabeled as belonging to a replacement editor. A terminal result cannot schedule later enumeration, mutation or replay. Transport closure/expiry merely releases the read-only slot; it is not a rollback claim.

  **Constraint inheritance:** T001's verbatim editor-context constraints apply to T002's domain/worker/reducer. All caller/bridge wire-only required keys, nullability, exact enums, malformed-input handling and size constraints remain normative; these quotes do not replace either contract. No new public declaration is justified solely by a future task or fixture.

## Phase 4: User Story 2 — Keep Discovery Inside the Intended Project (Priority: P1)

**Goal:** Inventory disclosure requires the exact authorized project/session and retains the existing confinement/access boundary.

**Implementation/tests owner:** T001 supplies authenticated v4 context and common-slot behavior; **T002 owns all US2.1–US2.4 public routing, confinement and acceptance**. No separate checkbox: a later “add authorization” PR would ship an unsafe first caller. This phase must pass before T002 is complete.

**Independent test criteria:** Use distinguishable projects and multiple same-project sessions. Missing selectors must give safe authenticated session-only feedback, never candidate inventory or focus/first/newest defaults. An exact selected lifetime must not be substituted after ending/replacement. Use actual unsafe root/ancestor/ownership/mode/ACL and outside-symlink controls plus root/parent/file/marker replacements before enumeration, batch publication and recheck. Require no unauthorized traversal, path/redirect disclosure or replacement attribution. Root/authentication denial suppresses all inventory; local gaps retain only independently safe entries with explicit limited/earlier validity. Inspect requested results, stderr and registry with outside/candidate sentinels.

## Phase 5: User Story 3 — Distinguish an Empty Project From an Incomplete Search (Priority: P1)

**Goal:** Coverage, freshness, limitations and interruption remain actionable; no missing evidence or retained count implies absence/completeness.

**Implementation/tests owner:** **T002 owns all US3.1–US3.5 behavior and acceptance**, using T001's independent scope facts. T003 repeats these guarantees in cumulative sequences. No additional limits/timeout/safety PR or checkbox is permitted after the first caller.

**Independent test criteria:** Exercise unreadable folders/files, unsafe/unavailable markers, unsupported control/non-UTF8/decorated/over-length names, zero-entry limited results, exact/one-over every fixed bound and overflow diagnostics. Independently add/rename/remove scripts across fresh requests and use the settled-cache-misses-new-file negative control. Detected namespace/marker/scope/epoch changes must remove affected entries or clear lost scope and prevent complete coverage even if a later sample matches. Actually stall the worker/editor and interrupt at selection/scope/batch/recheck barriers; compare typed reason, earlier evidence and externally measured ≤5-second delivery. Release delayed frames and prove no terminal upgrade, automatic retry or editor/source effect. A removed/replaced path or ended session used later receives fresh observation/open/edit rules; the old inventory is neither authority nor an edit basis.

## Phase 6: User Story 4 — Discover Without Disturbing Live Work (Priority: P1)

**Goal:** Establish cumulative freshness and non-interference with human work/history, separate safe composition, privacy/export and complete final-head compatibility evidence.

**Ownership:** T002 already proves US4.1, US4.4 and first-use clean/dirty US4.3 before exposure. T003 adds US4.2's full repeated-use campaign, complete US4.3 composed A–E/durability matrix and revalidates every story. It may fix newly discovered in-scope regressions but cannot inherit known incomplete T001/T002 acceptance.

**Independent test criteria:** Establish independently known clean, dirty, equal-text-dirty, divergent, non-selected and unloaded state. Run at least twenty fresh public discovery requests interleaving human edits and create/rename/remove controls; compare each path set or explicit limitation and before/after D/R/B, identity, dirty/current/saved version, selection and actual prior history. Separately choose discovered closed/dirty paths for the existing fresh observe/open/edit workflow. Confirm zero discovery-caused immediate/delayed effects or unsaved loss and all three actual export modes remain tool-free.

- [ ] T003 [US4] Deliver cumulative repeated-discovery and composed-workflow acceptance in `godot-addon/tests/run_script_discovery.py`, `godot-addon/tests/fixtures/script_discovery/fixture_driver.gd` and `godot-addon/tests/run_editor_campaign.py`, reusing `godot-addon/tests/run_observation.py`, `godot-addon/tests/run_script_open.py` and `godot-addon/tests/run_script_edit.py`, and record verified completion in `specs/004-discover-project-gdscript/quickstart.md`, `specs/004-discover-project-gdscript/tasks.md` and `PROJECT_STATUS.md`.
  **Depends on:** Completed T002 and its merged PR unless stacking is explicitly authorized. All first-caller branches, safety and individual baseline groups must already pass. No new product operation, model, native ABI, dependency or permission is introduced.

  **Cumulative harness capability:** Implement meaningful `sequential` and `composed` groups and the final unfiltered `--scenario all` path, executing each of the eight quickstart groups exactly once. Reuse existing history/durability/privacy/export helpers instead of copying suites or introducing a new fixture framework. Complete discovery `--suite all` integration and actual execution fingerprints/resume handling in `godot-addon/tests/run_editor_campaign.py` and `test_editor_campaign.py`; update `.github/README.md` and affected workflow execution tests for the real commands only. A complete campaign may not silently omit discovery or count nested records twice.

  **Twenty-request preservation/freshness sequence:** Use at least twenty actual public discoveries with deliberate between-request create/rename/remove and human-buffer edits, including normal complete positives and truthful limitations/refusals where controlled conditions require them. Each request obtains its own interval/session/path evidence; no cached inventory reuse. Independently witness D/R/B, loaded/unloaded and buffer/Resource identity, dirty indication, current/saved versions, selection and non-selected work. Exercise real prior Undo/Redo after discovery to establish retained history, and delayed observations to detect later effects. Distinguish deliberate fixture/human activity from discovery-caused changes; repeated unchanged mock rows do not satisfy the sequence.

  **Full composition:** Begin without a script locator. The public discovery result supplies the chosen eligible closed path, then separate product observation/opening/fresh-basis editing performs the existing workflow without manual tab preparation or path lookup. Prove A clean D/R/B convergence; B dirty-different/equal refusal with human text/history intact; C actual apply → Undo → Save → Redo → Save and prior-history reachability; D ordinary Save/close-reopen and current-consumer persistence; E the existing sequential fresh-basis edit stress. Reuse the complete existing edit/open acceptance definitions, including reparse/rescan and an owned runtime whose behavior reflects the edit. Also recheck changed/removed paths, ended sessions and known-document observation unaffected by inventory absence. Discovery itself acquires no source, opens nothing and claims no Undo/Redo participation; these are deliberate fixture/existing-operation actions.

  **Final-head evidence:** Run complete discovery, observation, edit and opening suites serially using quickstart §4/§6 commands, separate new private artifact directories, exact candidate/normal and separate fault-bundle provenance and owned-window proof. All eight discovery groups and all affected old suites must pass on the same unchanged implementation head; preserve failures/incomplete campaigns and do not relabel focused or planning evidence as cumulative success. Apply required Rust checks/builds and applicable changed campaign/workflow checks. Correct in-scope regressions in their owning production module with meaningful regression coverage, then rerun affected and required cumulative evidence on the corrected head.

  **Result-only/privacy/export review:** For every public discovery result, identify requested/resolved target or refusal, scope, coverage, usable/earlier entries, missing checks/reasons and safe next action from the result alone; then compare independent witnesses, exact path sets and elapsed time. Require selected/other-project/source/credential sentinel privacy in authorized, ambiguous, denied and interrupted cases; no inventory in incidental logs/registry and no source in discovery. Inspect and run enabled-addon, disabled-addon and hook-only exports for active tooling, private state and gameplay dependencies. Record actual counts without double-counting invocations/scenarios and exact measured deadlines; claim no untested support or arbitrary-writer atomicity.

  **Completion and delivery:** All 17 scenarios, FR-001–FR-017, SC-001–SC-006, seven edge cases, existing applicable A–E/durability and privacy/export gates must pass. Record constitutional and proportionate implementation-shape review, exact provenance and verified limitations in `specs/004-discover-project-gdscript/plan.md`, `quickstart.md`, current contracts/operator docs and `PROJECT_STATUS.md`. Mark only T003 `[X]` and Feature 004 complete when acceptance holds, independently of PR state. Phase 1 remains in progress because separate lifecycle/phase exit gates are not delivered here. Open the dedicated T003 PR and stop; no merge or next feature.

  **Constraint inheritance:** Every verbatim field/state/limit/retention constraint attached to T001/T002, and both contracts, applies to these sequences, results and any regression fix. This task adds cumulative evidence, not a second policy or a weaker supported profile.

## Phase 7: Polish & Cross-Cutting Concerns — Bundled With Their Owners

Necessary documentation, compatibility notices, cleanup of owned temporary scaffolds, redaction/export checks and implementation-shape review travel with T001/T002/T003. No standalone formatting, generic hardening, docs-only or cleanup task is justified. T003 supplies a meaningful cumulative state-transition/compatibility outcome, not permission to defer safeguards from the first caller.

All tasks preserve the exact current threat model, protocol independence, local-only operation, independent evidence and human-work protection. No Phase 1 completion, source-content search, lifecycle control, MCP or distribution expansion is authorized.

## Dependencies & Execution Order

### Task graph

```text
Completed Features 001–003 + reviewed artifacts + required analysis
                              |
                              v
T001: real read-only editor-scope boundary + coordinated v4 cutover
                              |
                              v
T002: complete public caller/walker/core + inseparable story safety
                              |
                              v
T003: repeated-use/composed workflows + final cumulative acceptance
```

Edges require prerequisite acceptance and normal PR-merge sequencing unless stacking is explicitly authorized. No task is marked `[P]`: all have real dependencies and shared integration paths. Setup/polish add no prerequisite PRs. Parallel authoring within the selected task does not authorize parallel task delivery or concurrent GUI campaigns.

### Story completion order and counts

| Story | Primary checkbox count | Owning tasks and completion |
|---|---:|---|
| US1 — exact unknown-path discovery (P1) | 1: T002 | T001 boundary; all US1.1–US1.4 caller acceptance in T002; cumulative regression in T003. |
| US2 — intended project confinement (P1) | 0 additional | T001 authenticated context; all US2.1–US2.4 implemented/proved inside T002 before its completion. |
| US3 — truthful coverage/interruption (P1) | 0 additional | All US3.1–US3.5 implemented/proved in T002; T003 revalidates sequences/composition. |
| US4 — live-work preservation/composition (P1) | 1: T003 | T002 owns US4.1/US4.4 and first-use US4.3; T003 adds US4.2/full composed matrix and completes the story. |
| Shared foundation | 1: T001 | Real scope owner, compatible v4 migration, tests/docs and affected old-suite acceptance. |

**Total: 3 unique tasks.** US1–US3 complete together at T002's acceptance gate; the first-use US4 safeguards also pass there. Full US4/feature completion follows T003. These are independently testable story outcomes, not independently shippable unsafe stages.

## Parallel Execution Examples by Story

These are optional **within-one-selected-task authoring slices**, not extra tasks or permission to bypass dependencies. Agree the already-defined interfaces and disjoint file ownership first. One integration owner gathers all results before builds/tests and runs actual GUI acceptance serially. Examples that name the same files are alternatives, not independent simultaneous batches.

| Story / selected task | Disjoint authoring example | Integration condition |
|---|---|---|
| US1 / T002 | One contributor owns `mcp-server/src/project_fs/discovery.rs` and private filesystem tests; another owns `godot-addon/tests/fixtures/script_discovery/fixture_driver.gd` and independently expected fixture inventories. | Fixed visibility/bounds contract first; integrate with the real caller before executing public inventory acceptance. |
| US2 / T002 | Auth/confinement regressions in `mcp-server/tests/confinement.rs` and `bridge_boundary.rs` can be authored separately from routing/namespace fixture controls in `godot-addon/tests/fixtures/script_discovery/fixture_driver.gd`. | Common exact binding and suppression semantics; no shared-file edits or mocked evidence substituted for real boundaries. |
| US3 / T002 | Contract/precedence cases in `mcp-server/tests/script_discovery_contract.rs` can be authored separately from real process/cancel/output cases in `mcp-server/tests/script_discovery_caller.rs`. | Fixed immutable outcome/runner contract; actual worker stalls and timing run only after integrated implementation. |
| US4 / T003 | Sequential/composed scenarios in `godot-addon/tests/run_script_discovery.py` can be authored separately from discovery campaign/fingerprint/resume integration in `godot-addon/tests/run_editor_campaign.py` and `test_editor_campaign.py`. | Final eight-group contract agreed first; one owner integrates before serial full suites and evidence review. |

T001 can likewise split addon scope/dispatch from Rust transcript/strict-decoder and disjoint Python fixture changes after fixing the bridge contract. Every v4 consumer ships in the same T001 PR; no intermediate incompatible peer pair or additional scheduler is allowed.

## Requirement and Acceptance Coverage

| Required acceptance | Owning task(s) | Required proof |
|---|---|---|
| US1.1–US1.4; FR-001–FR-003; SC-001 | T002; T003 cumulative | Exact ≥100-script/≥10-folder set; duplicate basenames; addons/never-opened/empty/invalid/read-only/tool/large/case paths; native visibility and true-empty controls. |
| US2.1–US2.4; FR-004–FR-005; SC-002 | T001 boundary; T002 end-to-end | Source-free authentication, exact/ambiguous/ended/replaced sessions, confinement/access/races and zero unauthorized inventory or substitution. |
| US3.1–US3.3; FR-007–FR-009; SC-003 | T002 | Actual local gaps/unsupported names/markers, exact/one-over bounds, diagnostic overflow, fresh metadata versus stale cache and known invalidation. |
| US3.4; FR-010–FR-011; SC-003 | T001 owner; T002 process/caller | Real blocked worker/editor, cancellation/loss/expiry/disable, typed precedence/earlier retention, ≤5 seconds and no late replay/fallback. |
| US3.5; FR-012; SC-005 | T002; T003 cumulative | Separate later operations re-resolve removed/replaced paths and session identity; inventory never becomes edit authority. |
| US4.1; FR-006/FR-016; SC-004 | T001 context probes; T002 caller | Independent clean/dirty/equal-text-dirty/divergent/non-selected/unloaded witnesses; actual prior history; zero source/lifecycle/scan effects. |
| US4.2; FR-009/FR-016; SC-004 | T003 | ≥20 discoveries interleaving inventory/human changes, truthful fresh results, history preservation and no delayed effects. |
| US4.3; FR-012/FR-015–FR-016; SC-005 | T002 first composition; T003 complete matrix | Returned path → fresh observe/open/edit; dirty refusal; unchanged known-document semantics and applicable A–E/Save/reopen/reparse/rescan/runtime. |
| US4.4; FR-013–FR-014/FR-017; SC-006 | Every introducing task; T003 cumulative | Result-only interpretation, inventory/source/credential sentinel privacy, all three actual export modes and exact tested support. |
| FR-016; all SCs | T001/T002 task evidence; T003 feature gate | Meaningful deterministic/real filesystem/process/live-editor evidence; full final-head discovery and directly affected observation/edit/open suites. |

All **seven edge cases** have T002 owners and T003 cumulative coverage where applicable: exact spaces/Unicode/case and distinct hard-link paths; discoverable dirty/divergent/unobservable document state without inspecting it; deleted/unsaved/embedded documents absent without changing known-document behavior; script-named directories/special files and unsafe symlinks; legitimate visibility exclusions versus failed reads with VCS/export inclusion; stale/changing inventories without scans or atomic claims; and large-source/unsupported-follow-on paths without source/parse/editability inference. Record filesystem-specific inapplicability for case-distinct fixtures when the actual filesystem cannot represent them, never a fabricated pass.

The matrix assigns owners; it does not narrow any normative scenario, constraint or gate in the linked artifacts.

## Implementation Strategy

### First Useful Slice — MVP

The first useful inventory is **T001 + T002**, two coherent PRs from the current baseline. The suggested MVP is US1's exact-path discovery **with US2/US3 and first-use US4 safety**, not a listing that gains confinement, honest limits or dirty-work preservation later. T001 is a real scope/compatibility boundary consumed by the existing fixture and old callers, not a chain of schema/helper stubs. Feature/release completion waits for T003.

### Incremental Delivery

1. T001 supplies real read-only editor context through a tested compatible v4 boundary; no inventory/caller claim yet.
2. T002 supplies the actual caller, now-consumed domain/walker/supervisor and every safe branch, with task-owned public acceptance and unchanged old capabilities.
3. T003 supplies cumulative freshness/non-interference/composed transitions and complete final-head privacy/export/compatibility evidence; it does not absorb known incomplete safety.
4. Stop after each dedicated task PR. Completion is determined by acceptance/shape/evidence, while permission to begin the next task still follows repository PR sequencing. Do not auto-select another task or merge.

## Granularity Review

**Review state: PASS (2026-10-01).** Parent review plus independent coverage/contract and granularity/dependency reviews found no actionable task-boundary, coverage, compatibility or executability defect. T001 has a real authenticated fixture consumer; T002 consumes its core in the actual caller and includes every first-use safeguard; T003 adds cumulative transitions rather than deferred safety. This completes the required granularity review before `/speckit.analyze`, not that separate analysis command or implementation approval. No new task, approval mechanism or CI gate is introduced.

| Criterion | Reviewed decomposition |
|---|---|
| Meaningful PR-sized increments | Real authenticated scope/cutover; complete caller and all safety; cumulative transitions/compatibility acceptance. |
| No trivial fragments | No directory/model/enum/codec/bit/version/test/docs-only PR. Discovery core and walker ship with the real caller that consumes them. |
| Tests/docs bundled | Every task owns its behavior/boundary tests, actual smoke, implementation-shape review and current docs. |
| Serial path to useful capability | One independently exercised private boundary before the caller; two PRs to useful inventory, three total. |
| Coverage preserved | Four P1 stories, 17 scenarios, 17 FRs, six SCs, seven edge cases and existing applicable coherence/durability/export gates have explicit owners. |
| Clear dependencies | T001 → T002 → T003; no misleading `[P]` or duplicate story checkboxes. |
| Present need and cost | v4/scope solves authenticated effective visibility; one walker solves demonstrated stale cache; private selector reuse avoids dummy locators/duplicate security policy; owned worker bounds blocked I/O. Existing mechanisms and simpler rejected alternatives are recorded in plan/research; no new infrastructure gate. |
| Current consumers/ownership | Existing authenticated fixture consumes T001; real CLI consumes T002. Shared bridge/selection/supervision names stay responsibility-based; domain/walker/fixtures stay discovery-specific. |
| Constitutional compliance | No source/lifecycle effects or substitute authority, preserved human work/history, explicit limited/refused/interrupted evidence, confined local authentication, actual GUI/export proof and unchanged native/public contracts. |

**Artifact validation:** All three unique sequential pending checklist rows have the required checkbox, ID, phase-appropriate story label and exact file paths; no dependent task is marked `[P]`. Seven phases retain all four P1 stories. Thirteen complete data-model constraint blocks are quoted verbatim, covering every field/enum/nullability/bound constraint and terminal precedence. Coverage review accounts for all 17 scenarios, 17 FRs, six SCs and seven edge cases. Local links/anchors, fences, template-marker removal and whitespace checks passed. The installed prerequisite helper found `tasks.md` and all design artifacts under the verified feature override. No product code, Cargo/native/GUI acceptance or `/speckit.analyze` was run by this task-generation workflow.
