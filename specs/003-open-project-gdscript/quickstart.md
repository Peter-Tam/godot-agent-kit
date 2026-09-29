# Quickstart: Verify Known-Path GDScript Opening

**Status:** Planning validation guide. `open-gdscript`, bridge v3, native revision 2 and the opening runner described below are **not implemented yet**. Existing commands are identified separately from future commands. The [task list](tasks.md) has passed [granularity review](tasks.md#granularity-review); analysis and approval of one dependency-ready task remain before implementation.

Use the [spec](spec.md), [plan](plan.md), [data model](data-model.md) and [caller](contracts/open-api.md), [bridge](contracts/bridge-protocol.md), [native](contracts/native-integration.md) contracts. A successful process exit, native return or matching pair of sources does not establish new-open success.

## 1. Exact candidate and prerequisites

- Official Godot `4.7.2.stable.official.ed1daf0bf`, full commit `ed1daf0bf001b61586d9930840f2f1394092c079`, executable SHA-256 `c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf`; macOS **26.6.2 arm64** is the initial tested-candidate environment, not a new opening support claim.
- Rust **1.98.1**, edition 2021, tracked Cargo.lock, declared rustfmt/Clippy components; C++17 Apple toolchain/SDK; Python **3.10+**; exact matching Godot export templates. No engine source checkout/patch or new dependency is selected.
- Owned synthetic project/editor/runtime processes, an **unlocked visible desktop** and permission to capture only their windows. Getter diagnostics while the desktop is locked are research, not GUI acceptance.
- Absolute executable paths, new empty mode-0700 artifact directories for each runner invocation, a draining stdout consumer and explicit deadlines. No real developer source/credentials in fixtures or captured artifacts.
- Install the matched native bundle using the existing [build guide](../../godot-addon/native/README.md). At implementation cutover, that guide/build must produce `editor_integration.gdextension`, `libeditor_integration.macos.arm64.dylib` and matching revision-2 provenance; all old kit-owned installed paths must be removed. Restart the owned editor to obtain a fresh v3 descriptor; do not reuse a previous session-bound edit basis.

Current executable identity/build entrypoints, from the repository root:

```sh
"$STOCK_GODOT" --version
rustc +1.98.1 --version
python3 --version
python3 godot-addon/native/build.py --godot "$STOCK_GODOT"
```

The current build still implements the completed edit boundary until the later cutover. A version string or generated manifest alone does not pass native compatibility. Record executable/library/source/ABI/toolchain hashes and actual exercised capabilities; never commit generated binaries, manifests or editor state.

## 2. Baseline and future build checks

From `mcp-server/`, the existing repository baseline remains:

```sh
cargo +1.98.1 fmt --all -- --check
cargo +1.98.1 clippy --all-targets --locked -- -D warnings
cargo +1.98.1 test --locked
cargo +1.98.1 doc --no-deps --locked
```

Default tests include doctests. Do not replace them with `--all-targets` alone or introduce a root workspace/blanket feature matrix. Run applicable existing native-build and workflow checks on changed build/fixture/workflow surfaces; no new provider/runner gate is required.

**Future build command, after the opening binary is implemented:**

```sh
cargo +1.98.1 build --locked --lib --bin open-gdscript \
  --bin observe-gdscript --bin edit-gdscript --example stock_validation_fixture
```

Pure/core and boundary tests must assert outcomes and plausible consumer-visible failures: dirty-equal recognition versus new-open conflict, independent evidence, no false absent R, invalidated identity/source, source/access precedence, staged known/unknown effects, terminal discard and late-message immutability. Cover strict v3 framing/authentication and actual confined file changes. Avoid forwarding/mock echoes, source-text tests or duplicate rows for the same path.

## 3. Future owned opening runner

Reuse existing observation/edit harness setup, authentication, disk/editor/history witnesses, owned-window capture, cleanup and export checks. New opening fixture controls are private test infrastructure, never product opcodes. Cold fixtures may use an ignored directory to prove actual cache absence; that is fixture preparation, not a user installation requirement.

Build a separate fault-enabled artifact using the **existing** native build interface:

```sh
python3 godot-addon/native/build.py --godot "$STOCK_GODOT" \
  --fixture-faults --output-addon "$PRIVATE_FAULT_NATIVE"
```

The normal product library must contain no fault-control callable. The separate `GAK_FIXTURE` artifact may expose bounded stage barriers for owned tests only; it must not replace the normal installed library or enter exports.

**Selected future runner interface — this command becomes runnable with its implementation task:**

```sh
python3 godot-addon/tests/run_script_open.py \
  --godot "$STOCK_GODOT" --opener "$OPEN_CALLER" \
  --observer "$OBSERVER" --editor "$EDIT_CALLER" \
  --stock-validator "$STOCK_VALIDATION_FIXTURE" \
  --native-fault-addon "$PRIVATE_FAULT_NATIVE" \
  --scenario all --artifacts "$OPEN_ARTIFACTS"
```

`OPEN_CALLER`, `OBSERVER`, `EDIT_CALLER` name the absolute built binaries; `STOCK_VALIDATION_FIXTURE` names the test-only Cargo example. `all` requires every input and executes all groups below, serially on the visible desktop. Each group may also be selected by its exact name with the same interface. Do not run GUI groups concurrently or substitute direct native opening for the public caller in product acceptance.

| Group | Specification coverage | Required result |
|---|---|---|
| `new-open` | US1.1–US1.5; FR-001–002, FR-003, FR-005, FR-008–009; SC-001 | Cold/cached, exact same-name routing, empty/read-only/syntax-invalid targets; new actual buffer and independent unchanged D/equal R/B/clean buffer/unedited R. Fresh observe then existing edit succeeds without manual tab preparation. |
| `already-open` | US2.1–US2.2; FR-006, FR-009, FR-015; SC-002 | Clean, non-selected, dirty-different, dirty-equal, divergent and source-limited targets: same identity/source/dirty/selection/history, no load/reload/selection or duplicated buffer. Limited observations remain limited. |
| `preservation` | US2.3–US2.5; FR-003–005, FR-007, FR-013; SC-002–004 | Dirty current R == B and dirty background positive cases retain human work and actual native history. Conflicting current R/B, changed target before entry, pending-drag/stale compiled context and newer human text during verification never cause application/repair of pending work or false success. |
| `routing` | US3.1–US3.5; FR-002–004, FR-007, FR-011, FR-014–015; SC-003 | Ambiguous/ended/replaced sessions, missing/denied/outside/unsupported paths, cached conflict/dirty/unknown state, effect profiles, file/cache/parent races and overlap; no source-before-selection, retargeting, unauthorized effects or late opening after proven refusal. |
| `native-boundary` | US1.1–US1.2, US2.3–US2.5, US3.3–US3.5, US4.1–US4.2; FR-003–008, FR-010–013 | Exact source/path methods, non-takeover cache publication, initial compilation only for new R, file/current-context guards, unedited flag, staged receipts and retained entered ownership. Independent witnesses, not native-response copies. |
| `interruption` | US4.1–US4.3; FR-010–014; SC-004, SC-007 | Cancel/timeout/EOF/disable/worker/helper failure before and after authorization and at every native/verification stage. Known partial/unknown outcomes match survivor state; ≤10-second results; no rollback/retry or loss of newer work. |
| `sequential` | US4.4; FR-004, FR-006, FR-013–014, FR-018; SC-005 | ≥20 actual opening requests, ≥5 successful genuinely closed transitions and ≥5 dirty already-open requests, with human close/reopen/edit interleavings. No duplicates, stale identity results, source/history loss or unexpected late reopening. |
| `composed` | US1.4, US4.5; FR-009, FR-016, FR-018; SC-006 | Product open → separate fresh observation → existing edit, then all applicable A–E and Save/reopen/reparse/rescan/permitted runtime behavior. Existing observation stays read-only; closed-target editing still refuses. |
| `privacy-export` | US4.6; FR-017–018; SC-007 | Authorized/ambiguous/denied/interrupted sentinels, private current-source validation, no incidental/unselected-source leakage, all three export variants exclude tooling and run without active tooling/dependencies. |

These groups cover all **21 story scenarios, FR-001–FR-018 and SC-001–SC-007**. FR-010/result-only interpretation and FR-012/timing apply throughout, not only their named groups. A safe-refusal-only implementation or primitive-only pass is incomplete.

## 4. Boundary recipes and witnesses

Before each case, independently record disk bytes/hash/identity; cache presence and retained Script ID/source/edited flag; actual target ScriptEditorBase/CodeEdit association, text, current/saved versions and attributed dirty state; current selection; and relevant human history. Record actual absence separately from unavailable evidence. After the operation, obtain a separate witness rather than echoing captured inputs or bridge replies. Include visible owned-window evidence and inspect it.

Required focused controls:

1. **Exact methods:** Reproduce the property-assignment prototype's clean-buffer-but-edit-refused condition as a private negative control. Product acquisition must instead use bound source/path methods, preserve Resource edited=false without clearing it, and pass a fresh existing edit. Preserve actual target source and original file identity across opening.
2. **Current source:** Use actual native typing to create dirty R == B after ordinary fixture preparation; retain dirty versions/flags through opening, then exercise actual Undo and Redo of the earlier human change. Separately hold dirty R != B at the entry barrier and require no new lifecycle effect. Product requests must not wait for or force convergence.
3. **Pending export/history:** Prepare actual dragged-export state, Undo/removal and stale compiled tool/base/property/method witnesses. Unsafe/unknown contexts must refuse before native navigation can set an unrelated object property, invoke a dynamic hook or change source. Include primitive, Object/typed-hint and exported property variants sufficient to challenge the selected metadata guard. Do not merely assert a fabricated pending-empty flag.
4. **Effect acquisition:** Instrument a private front loader to establish zero product root-loader calls. Exercise tool/static/const/export/load/preload/global-class/autoload/extension and script-base refusals. Comments/strings containing excluded words must not be mistaken for executable tokens. Target `var =` must open with actual invalid text; invalid current context must refuse with a different reason.
5. **Identity/access:** Replace/rename/delete leaf and parents at controlled boundaries; change D/cache/current document/session; hold a human-open race; test exactly supported source/context limits and one over. Read-only opening is positive. NUL/CR/BOM/control/unsupported lexical representation refuses without conversion or truncation. Never open an outside sentinel under the old target.
6. **Staged uncertainty:** Hold before authorization, before/after cache publication, inside compilation/opening and during observation/recheck. A known published Resource without a buffer is partial application. Lost unacknowledged entry is unknown. Establish irrevocable discard for proven refusal, release all controlled deliveries and prove no late effect. Cancelled entered work retains the operation slot until return.
7. **Overlap:** While A owns the same editor slot, B must return terminal `busy` with no retained opening stage. Release A, observe its actual completion/uncertainty, then prove B cannot open later. After cleanup a newly intentional request must use current identity; no queue or automatic replay.
8. **Post-effect human work:** Type into the new buffer or close/reopen it during verification. Preserve the newer identity/text/history and known request effects; never Save, clear an edited flag, reassert captured text or close the buffer to restore eligibility.

Synchronous native calls may outlive the caller. External elapsed time and the terminal result must still meet ten seconds with stdout drained; fixture teardown may then release a barrier and observe eventual survivor state. Do not turn cleanup timing into a false rollback guarantee or kill a developer-owned editor.

## 5. Existing behavior and composed durability

Opening creates no source-history entry and claims no undoable tab-opening action. Its relevant C obligation is preserving real prior native history and the existing edit's actual Undo/Redo after opening. D and E also remain applicable because this capability supplies the document on which edits operate.

The `composed` group must use the product opener wherever the first useful clean-edit path otherwise requires manual target opening; it must not call a fixture's direct-open helper as a substitute. Then exercise:

- **A:** a real changed edit reaches independently verified D == R == B.
- **B:** dirty-different and dirty-equal human work survives an edit refusal.
- **C:** actual apply → Undo → ordinary Save → Redo → Save, plus earlier human history reachability. Never count registration or `has_undo` alone as this proof.
- **D:** ordinary Save and human close/reopen preserve intended source without reconciliation; cached source identity/ordinary consumers remain usable.
- **E:** twenty fresh-basis edits with the existing unsafe interleavings, no disappearance/divergence/reversion on Save.
- Explicit reparse/rescan and a permitted owned runtime fixture produce the intended changed behavior, not just exit 0. No new runtime or Save/Undo/close product operation is added.

Also run the **existing complete suites on the cutover head**, using separate empty artifact directories:

```sh
python3 godot-addon/tests/run_script_edit.py \
  --godot "$STOCK_GODOT" --observer "$OBSERVER" --editor "$EDIT_CALLER" \
  --stock-validator "$STOCK_VALIDATION_FIXTURE" \
  --native-fault-addon "$PRIVATE_FAULT_NATIVE" \
  --scenario all --artifacts "$EDIT_ARTIFACTS"
python3 godot-addon/tests/run_observation.py \
  --godot "$STOCK_GODOT" --observer "$OBSERVER" \
  --scenario all --artifacts "$OBSERVATION_ARTIFACTS"
```

Existing five-second observation and ten-second edit deadlines remain. These baseline suites alone do not prove their composition with the new opener; the `composed` group supplies that missing evidence. Do not double-count invocation records as distinct cases or run filtered groups and label them full-suite acceptance.

## 6. Result review, privacy and export

For every public opening result, first review only the result against the caller contract: exact target/refusal, new versus already satisfied, known/partial/unknown effects, reached versus completed stages, actual source/dirty/Resource flags, invalidation, history limitations and safe next action. Then compare with independent before/after witnesses and external time. This satisfies SC-007 without inventing an additional approval/CI service.

Use distinct synthetic sentinels for the target, current private-admission document, unrelated open document and another project. Only an authorized requested-target result may contain target source. Verify no current/unselected source or raw child diagnostics in output/logs/registry; private owned validator staging must be removed after success, refusal, timeout and child failure. Record actual child/process-group cleanup, not a policy assertion.

Exercise enabled-addon, disabled-addon and export-hook-only production exports. Inspect actual exported contents for opening scripts, shared native bundle/library/manifest, bridge, fixtures and credentials; launch the export and prove no tool listener, native registration or tooling dependency. Keep the existing export boundary and workflow trust model rather than adding a feature-specific workflow.

## 7. Evidence at planning completion

[Research](research.md#4-candidate-and-executed-evidence) records **50 fixture requests, 20 opening invocations and six clean owned-editor exits**. Two additional invocations of each existing observation/edit caller tested composition. Property assignment failed the existing dirty gate; explicit native methods preserved unedited R and produced `verified_changed` with independently matching intended D/R/B. Source-bound syntax-error/read-only/cached opening and a dirty-current native Undo/Redo control were exercised. Throwaway projects/source/binaries were removed; private evidence remains at the recorded paths/hashes.

The desktop was locked. No visible-window screenshot, new product opening caller, full opening timing/race/history/durability/export campaign or new support claim follows those probes. Planning artifact checks are documentation validation; product Cargo/native/workflow suites were not rerun for this documentation-only change. Implementation completion requires the applicable evidence above on its delivery head, plus the repository's proportional implementation-shape review. Do not mark a task or feature complete merely because planning or a PR exists.
