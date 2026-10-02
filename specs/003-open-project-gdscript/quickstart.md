# Quickstart: Verify Known-Path GDScript Opening

**Status:** **Feature 003 is complete; T001–T003 are complete.** [Cumulative acceptance](#10-t003-cumulative-acceptance-2026-09-30) and the [implementation-shape review](plan.md#t003-implementation-shape-and-constitutional-review-2026-09-30) passed on the exact supported stock Godot/macOS arm64 environment below. The complete opening, edit and observation campaigns passed with unchanged public observation/edit v1. Completion is independent of PR review/merge; roadmap Phase 1 remains in progress because discovery and independent lifecycle controls remain outside this feature.

Use the [spec](spec.md), [plan](plan.md), [data model](data-model.md) and [caller](contracts/open-api.md), [bridge](contracts/bridge-protocol.md), [native](contracts/native-integration.md) contracts. A successful process exit, native return or matching pair of sources does not establish new-open success.

## 1. Exact candidate and prerequisites

- Official Godot `4.7.2.stable.official.ed1daf0bf`, full commit `ed1daf0bf001b61586d9930840f2f1394092c079`, executable SHA-256 `c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf`; macOS **26.6.2 arm64** is the exact supported opening environment. No other Godot build or platform is claimed.
- Rust **1.98.1**, edition 2021, tracked Cargo.lock, declared rustfmt/Clippy components; C++17 Apple toolchain/SDK; Python **3.10+**; exact matching Godot export templates. No engine source checkout/patch or new dependency is selected.
- Owned synthetic project/editor/runtime processes, an **unlocked visible desktop** and permission to capture only their windows. Getter diagnostics while the desktop is locked are research, not GUI acceptance.
- Canonical absolute executable paths, new empty mode-0700 artifact directories for each runner invocation, a draining stdout consumer and explicit deadlines. The stock validator rejects a symlink executable rather than following it; resolve the supplied Godot path before invoking these commands. No real developer source/credentials in fixtures or captured artifacts.
- Rebuild/install the matched revision-3 native bundle using the [build guide](../../godot-addon/native/README.md): `editor_integration.gdextension`, `libeditor_integration.macos.arm64.dylib` and matching provenance. The builder removes only provenance-matched obsolete kit artifacts and refuses substituted files. Update caller/addon peers together for [private v5](../005-close-project-gdscript/contracts/bridge-protocol.md) and restart the owned editor for a fresh descriptor; do not reuse older descriptors, native revision 2 or previous session-bound edit bases.

Current executable identity/build entrypoints, from the repository root:

```sh
"$STOCK_GODOT" --version
rustc +1.98.1 --version
python3 --version
python3 godot-addon/native/build.py --godot "$STOCK_GODOT"
```

The current cutover contains editing, opening and guarded closing. Each native operation requires its complete matched transport and native family. Closing acceptance is recorded separately in the [closing quickstart](../005-close-project-gdscript/quickstart.md). A version string or generated manifest alone does not pass native compatibility. Record executable/library/source/ABI/toolchain hashes and actual exercised capabilities; never commit generated binaries, manifests or editor state.

## 2. Build checks

From `mcp-server/`, the existing repository baseline remains:

```sh
cargo +1.98.1 fmt --all -- --check
cargo +1.98.1 clippy --all-targets --locked -- -D warnings
cargo +1.98.1 test --locked
cargo +1.98.1 doc --no-deps --locked
```

Default tests include doctests. Do not replace them with `--all-targets` alone or introduce a root workspace/blanket feature matrix. Run applicable existing native-build and workflow checks on changed build/fixture/workflow surfaces; no new provider/runner gate is required.


Build the integrated library, three callers and private validator fixture:

```sh
cargo +1.98.1 build --locked --lib --bin open-gdscript \
  --bin observe-gdscript --bin edit-gdscript --example stock_validation_fixture
```

Pure/core and boundary tests must assert outcomes and plausible consumer-visible failures: dirty-equal recognition versus new-open conflict, independent evidence, no false absent R, invalidated identity/source, source/access precedence, staged known/unknown effects, terminal discard and late-message immutability. Cover strict v5 framing/authentication and actual confined file changes. Avoid forwarding/mock echoes, source-text tests or duplicate rows for the same path. Historical acceptance below retains its exercised v3 provenance; current v5/native-revision-3 cutover evidence belongs to the [closing quickstart](../005-close-project-gdscript/quickstart.md).

## 3. Owned opening runner and caller groups

Reuse existing observation/edit harness setup, authentication, disk/editor/history witnesses, owned-window capture, cleanup and export checks. New opening fixture controls are private test infrastructure, never product opcodes. Cold fixtures may use an ignored directory to prove actual cache absence; that is fixture preparation, not a user installation requirement.

Build a separate fault-enabled artifact using the **existing** native build interface:

```sh
python3 godot-addon/native/build.py --godot "$STOCK_GODOT" \
  --fixture-faults --output-addon "$PRIVATE_FAULT_NATIVE"
```

**Implemented T001 native-boundary command:**

```sh
python3 godot-addon/tests/run_script_open.py \
  --godot "$STOCK_GODOT" --observer "$OBSERVER" --editor "$EDIT_CALLER" \
  --stock-validator "$STOCK_VALIDATION_FIXTURE" \
  --native-fault-addon "$PRIVATE_FAULT_NATIVE" \
  --scenario native-boundary --artifacts "$OPEN_ARTIFACTS"
```

This runs the actual private addon/native owner, independent current-source
helper and existing observation/edit consumers. It does not require `--opener`.
The native group cannot substitute for the six T002 public-caller groups.
No incomplete group is exposed as `all`.

The normal product library must contain no fault-control callable. The separate `GAK_FIXTURE` artifact may expose bounded stage barriers for owned tests only; it must not replace the normal installed library or enter exports.

**Complete public-caller and native-boundary command:**

```sh
python3 godot-addon/tests/run_script_open.py \
  --godot "$STOCK_GODOT" --opener "$OPEN_CALLER" \
  --observer "$OBSERVER" --editor "$EDIT_CALLER" \
  --stock-validator "$STOCK_VALIDATION_FIXTURE" \
  --native-fault-addon "$PRIVATE_FAULT_NATIVE" \
  --scenario all --artifacts "$OPEN_ARTIFACTS"
```

`OPEN_CALLER`, `OBSERVER`, `EDIT_CALLER` name the absolute built binaries;
`STOCK_VALIDATION_FIXTURE` names the test-only Cargo example. `all` runs every
group below exactly once, including `native-boundary`, `sequential` and
`composed`, serially on the visible desktop. Use a new empty private artifact
directory per invocation. A named selector runs only that group and is not
complete-suite evidence. Do not substitute direct native opening for the public
caller in product acceptance.

`sequential` makes 25 product opening requests across two evolving fixtures:
five genuinely closed successes, seven clean and ten dirty recognitions, and
three unsafe refusals. `composed` reuses the existing clean/conflict/history/
durability/sequential edit campaigns with product opening as their initial target
preparation, plus explicit read-only closed observation and closed-edit refusal.
Human history, Save and close/reopen are fixture actions, not product commands.

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

## 8. T001 native-boundary and cutover acceptance (2026-09-30)

T001's private native/current-validation capability and coordinated migration
are complete. This is **not** public-opening-caller or feature acceptance.
The commands in §2, the implemented native-boundary command in §3, and both
unfiltered existing `--scenario all` commands in §5 were exercised serially.

| Gate | Observed result |
|---|---|
| Native opening | **265 records passed**, including cold/cached/empty/read-only/syntax-invalid opening, exact D/R/B and clean/unedited evidence, fresh observe/edit composition, property-assignment negative control, dirty current/background history, LF whitespace preservation, bounded metadata/source profiles, current validator refusals/cleanup, namespace/cache/document/session changes, actual slot contention and terminal/entered-call ownership. |
| Existing edit/native suite | **407 records passed**, including applicable A–E, actual prior-history Undo → Save → Redo → Save, Save/reopen/reparse/rescan/runtime durability, twenty fresh-basis sequential edits, privacy and all three production-export variants. All **114** public edit results received the runner's result-only review. Maximum recorded caller time **9.553 s**. |
| Existing observation suite | **216 records passed** across all 13 quickstart groups, including v3 three-role cross-language proofs, seven-bit capability/native-build tampering, source-free routing, independent surface limits and repeated read-only behavior. Maximum recorded bounded operation **4.803 s**. |
| Local Rust checks | Formatting, Clippy `--all-targets --locked -- -D warnings`, **207 tests** including the doctest phase, rustdoc and locked library/caller/example builds passed. The three authentication unit tests were rerun after removing the redundant vector-length assertion. |
| Native/workflow checks | Normal and isolated fault native builds passed; **9** native-build tests, **22** workflow tests and Actionlint passed. These are local checks, not a claim that optional hosted GUI automation ran. |

Counts are invocation/evidence records, not distinct scenarios; nested group
totals are not added again. The opening group contains 23 owned-window captures
and the edit suite 28. Actual cold, empty, syntax-invalid, dirty-history and
pending-drag opening surfaces, reopened sequential-edit/runtime-durability
surfaces, and observation session/dirty-sequence surfaces were inspected.
The syntax-invalid target remained visibly invalid rather than being repaired;
the dirty current/background tabs retained their dirty markers.

The private helper validated actual admitted R == B rather than target D or an
edit proposal. Wrong purpose/source/identity/settings/guard, incomplete results,
helper loss and deadline paths released no authorization hashes. Actual native
11-second stage stalls retained ownership/references; competing bounded callers
did not imply a responsive editor or rollback. T001 adds no public opening
deadline claim: that caller remains T002's responsibility.

### Regressions resolved during integration

- Godot's category display record carries a path hint. It is not an exported or
  Object-valued script variable. Native/Rust gates now preserve this distinction;
  the failing-before/passing-after Rust regression also rejects a variable
  attempting to use display flags to bypass the property guard.
- Opening incorrectly inherited edit-only Save-format admission. Supported LF
  trailing whitespace/final newlines now survive opening unchanged in both
  target and current documents; real-editor cases prove preservation. Save
  formatting remains private to editing, with no weakened mutation gate.
- Fixture dispatch reread a control file after deciding its operation family,
  allowing the next request to reach the wrong handler. It now delegates the
  already-parsed immutable request. Cold fixtures exclude script indexing while
  explicit global-class controls remain discoverable.
- The large-edit prefix gate retained v2 while the rest of the bridge used v3.
  An actual **12,342-byte** edit failed before the fix and then returned
  `verified_changed` with independent D/R/B agreement. This regression now runs
  in the existing `clean-open` group and complete edit suite.
- The observation proof fixture retained an obsolete transcript-length assertion.
  That redundant assertion was removed, not repinned; all three exact HMAC
  vectors and cross-language/tampering checks remain and passed.

### Provenance, ownership and limits

Exact environment: official Godot `4.7.2.stable.official.ed1daf0bf`, binary and
commit from §1, macOS **26.6.2 arm64**, Rust **1.98.1**, Apple clang
`21.0.0 (clang-2100.3.34.2)`, SDK **27.0**. The exact official export template
SHA-256 was `88df5e2e6fee99088699be66e6d42e4da4fb0c5619d054297d755a49558a4792`.

- Production native build ID: `836fdbd72f7bb176d76d8b029f4a34dabdcbccc1c4215fb2b6360a80ba6acea2`.
- Production library SHA-256: `9bc089277461944ef3dd22644eb134ccb062660e92fe03e0c3c9d41da722d881`.
- Fixture native build ID: `0a9080929dd046f360f5d11824f319283548b8e2de4dde99980bf1d9a80b0a9a`.
- Fixture library SHA-256: `b805d154f610f8f2998657b4d58178d630a33ae0ecd5237d628e74a1c70d5485`.

Private evidence is retained under
`~/.godot-agent-kit-open-T001-mn0df0wp/`; these local paths identify this run,
not contributor prerequisites. Successful summary files and SHA-256:

| Summary | SHA-256 |
|---|---|
| `opening-final-2/summary.json` | `82511afcd4a602c1906a83580d0132e78eb6ea0bbe646c19de6cfaae410be0e7` |
| `edit/summary.json` | `6cb4fdf261b0efc33d6b3f8ebc9f159c2ff6b46d0462597e069c0d3d41c44f70` |
| `observation-final/summary.json` | `99ad5c7b5665fd2d8dbaee112e95d2ee65c53c6e0dc94f4dba0a0a16a72e709a` |

Earlier failed, interrupted and locked-desktop state-diagnostic runs are
separate and are not counted as passing acceptance. After the maintainer
unlocked the desktop, normal visible campaigns supplied the required evidence.
The final proof-fixture correction changed no production behavior or opening/
edit path; their completed runs remain applicable, and observation was rerun
in full after that correction. Owned fixture processes and temporary projects
were cleaned up; generated native artifacts and evidence remain outside version
control. The temporary desktop-awake assertion was released.

Implementation-shape review found current, cohesive consumers: `document_guard`
owns shared descriptor/namespace/document checks; `script_document` owns edit
formatting/writes/history/finalization; `script_open` owns opening lifecycle;
`open_context` and the Rust typed guard/literal modules own private admission.
One native owner variant and addon lifecycle state retain entered work safely.
New Rust visibility is limited to the actual fixture consumer; no unused opening
domain/reducer, generic serializer, duplicate scheduler or validation service was
introduced. The review's large-frame finding was fixed and exercised before
completion.

Constitutional review: independent authorities and verification, preserved human
work/history, native methods, confined read-only opening, effect-aware terminal
facts and real GUI/export evidence satisfy applicable I–VIII/X/XII obligations.
The same pinned dependencies/public ABI and existing supervision/slot satisfy
XI/XIII without a new permission, service or CI boundary. The inherited local
endpoint and already-running-malicious-plugin limitations are unchanged. No
other engine/platform, universal callback isolation, public opening caller,
T002/T003 completion or roadmap Phase 1 completion is claimed.

## 9. T002 public-caller acceptance (2026-09-30)

T002's complete guarded caller, consumed domain, worker/supervisor and
authenticated addon exchange are accepted. Only T002 is newly complete; T003's
new cumulative opening/composed matrix remains pending. Public observation/edit
v1 and their deadlines are unchanged. No broader engine/platform or completed
Feature 003 support claim follows this task.

The §2 Rust commands, both native builds, all six §3 public groups, the separate
native-boundary group and both unfiltered §5 regression suites passed. GUI
campaigns ran serially on the exact §1 stock Godot/macOS arm64 candidate with
owned visible windows. The final Rust baseline passed formatting, warnings-denied
Clippy, **247 tests** with the doctest phase, rustdoc and locked caller/example
builds. **9 native-build tests**, **22 workflow tests** and Actionlint passed.
No native source or workflow behavior changed after those applicable checks.

### Executed groups and observed behavior

| Public group | Records | Actual opening results | Maximum external seconds |
|---|---:|---:|---:|
| `new-open` | 18 | 7 | 3.463944 |
| `already-open` | 12 | 11 | 0.416735 |
| `preservation` | 14 | 13 | 2.399983 |
| `routing` | 34 | 31 | 2.722357 |
| `interruption` | 91 | 78 | 9.510471 |
| `privacy-export` | 8 | 4 | 5.768252 |

The **177 records include 144 actual public opening results**, not 177 distinct
opening invocations. Every public result received result-only interpretation
before comparison with independent witnesses; every timed controlled invocation
returned within ten seconds with stdout drained.

- Cold, retained-cache, empty, read-only, syntax-invalid, no-current-source and
  exact-512-KiB targets opened through the actual caller. Independent D/R/B,
  identity, dirty and Resource-edited facts established the result; target parse
  invalidity remained distinct from an unsafe/invalid departing current source.
- Clean/non-selected, dirty-different, dirty-equal and divergent existing
  documents retained their identities, selection, source and native history.
  Independently oversized D, R and B remained unavailable/`too_large`; the
  caller did not manufacture complete observation or clean state.
- Dirty current R == B and dirty background work survived new opening with
  actual prior Undo/Redo. Current R != B, invalid/stale current metadata and
  pending export drag refused. Human opening, typing, close/reopen and four
  current/background tab changes at verification/recheck boundaries preserved
  newer work; selection was observed, never restored for the proof.
- Exact same-basename routing, two real same-project sessions, ended/replaced
  lifetimes, missing/denied/outside/invalid targets, cached conflict/dirty/wrong
  type, effect profiles, namespace/cache/context races and a genuine held edit
  owner exercised refusal and no-late-entry behavior.
- Six actual helper/worker/stock-child interruption cases independently checked
  process groups, descendant exit and private staging removal. Forty-five peer
  checkpoints and fifteen entered-native cases covered stall, connection loss,
  cancellation, plugin disable and worker loss. Known cache/document effects
  remained partial application; unacknowledged possible effects remained unknown.
  Later barrier release did not amend the terminal result or roll back human work.
- Selected/current/background/other-project sentinels covered authorized,
  ambiguous, denied and interrupted requests. Only authorized selected source
  appeared in product results. Enabled, disabled and export-hook-only exports
  excluded the opening/native/tooling artifacts and ran without active tooling.

The new-open composition used **open → separate fresh observe → existing edit**
without fixture tab preparation. Actual edit Undo → ordinary Save → Redo → Save,
dirty refusal, close/reopen, reparse, rescan and a fresh runtime returning **53**
passed. Opening itself added no source-history operation. The separate retained
native-boundary campaign passed **265 records**. Full existing edit/native
acceptance passed **407 records**, including all A–E gates, twenty fresh-basis
sequential edits and **114** result-only public edit reviews; its maximum timed
call was **9.536002 s**. Full observation passed **216 records**, maximum timed
call **4.769220 s**. These are current integrated regressions, not a substitute
for T003's still-unimplemented complete opening `all`/`sequential`/`composed`.

### Provenance and review

Local evidence root:
`/Users/petertam/.godot-agent-kit-open-T002-ber0hqs2`.
Only the following complete runs are this acceptance record:

| Summary relative to evidence root | SHA-256 |
|---|---|
| `new-open-final/summary.json` | `2485e50d03514dd350072dd082496fe51a28df1a082a497a6bdce3b04818387a` |
| `already-open-final/summary.json` | `b6302ae5086e979f03031698373f9afb22b29809af453e8861f24073c60cc26c` |
| `preservation-final/summary.json` | `076a9e9cecec34cf9e93ca6e8a64c92bc99678b73e24eec23e9edf7763d2acd7` |
| `routing-7/summary.json` | `44e983d25b18189ff36e4d28777e089552284e39a404deeabd693610a5faad3f` |
| `interruption-4/summary.json` | `9d6b91fe4b22d2c3b2af0519fd9624a60ab7e444b94f794b631e2b206ed7aac5` |
| `privacy-export-2/summary.json` | `69ee93b4bb564806879c90d521a9e1789c5b8f68a588eb30acacf76eb5fac913` |
| `native-boundary/summary.json` | `aa4a4c52252c634de2be6228a7738fcd0aed2bd4eb42f2c4bc10e4352c10afa9` |
| `edit-2/summary.json` | `a4a6f1eb159490c10aee0de5a6252b5ff7e47272490fd6f359041c759f641ff1` |
| `observation/summary.json` | `1ed6eea12941f5de825ee1c15e3a85d0010237635d3b313c3f385e789cc058e5` |

All six public groups used the same final `open-gdscript` SHA-256
`398dad0f0f58623223e232ce8cf8d2bb2762bf55959ea26d6cd9362d3357a71e`.
The normal native build ID remains
`836fdbd72f7bb176d76d8b029f4a34dabdcbccc1c4215fb2b6360a80ba6acea2`
(library SHA-256
`9bc089277461944ef3dd22644eb134ccb062660e92fe03e0c3c9d41da722d881`);
the separate fixture build ID is
`0a9080929dd046f360f5d11824f319283548b8e2de4dde99980bf1d9a80b0a9a`
(library SHA-256
`753b883cc43936900ebb7a975d30f7cdab91fe78a7930877cd16207fce3bab2a`).
Manifests/summaries retain ABI/API, source, driver, caller and template hashes.
The native toolchain was Apple clang **21.0.0**, SDK **27.0**, with Rust
**1.98.1** and Python **3.10.9**. Generated artifacts remain outside version
control.

Earlier failed/diagnostic runs are not counted as passing groups. Corrections
included typed missing-disk/refusal provenance, preserved invalidated evidence,
fixture preparation before native navigation, waiting for actual saved-layout
restoration in second editors, and publishing the entered-call witness before
deliberate endpoint shutdown. An ordinary fixture Save failed despite local
Control focus; the harness now presents the actual owned OS window before
sending the shortcut once. A focused real history smoke and the full edit/
observation suites passed afterward; the first three public groups were then
rerun with the final caller and presentation helper.

Owned-window images were inspected for cold/invalid opening, dirty/divergent
recognition, preserved human work/selection, interrupted survivor state, prior
history and reopened/runtime/sequential durability. Fixture processes and
temporary projects were cleaned up, and the owned desktop-awake assertion was
released. Private evidence is retained locally; this does not claim a hosted
GUI run.

The [implementation-shape and constitutional review](plan.md#t002-implementation-shape-and-constitutional-review-2026-09-30)
records current ownership, narrow visibility, typed failure/state handling and
the concrete complexity justification. No new service, dependency, permission,
approval layer or CI execution boundary was introduced. The inherited stock
endpoint and already-running-malicious-plugin limitations remain unchanged;
there is no universal callback isolation or wider-platform claim.

## 10. T003 cumulative acceptance (2026-09-30)

T003 and Feature 003 are complete. The maintainer selected only T003 after T002
merged in [PR #43](https://github.com/Peter-Tam/godot-agent-kit/pull/43).
The unfiltered opening `--scenario all` command in §3, then the complete edit
and observation commands in §5, passed serially on the same implementation.
All 21 story scenarios, FR-001–FR-018, SC-001–SC-007, eight edge cases and
applicable A–E/durability/privacy/export gates retain their mapped coverage.
No production behavior, public contract, dependency or CI execution boundary
changed in this task.

### Complete campaigns

| Campaign | Evidence records | Public result review | Maximum external caller time |
|---|---:|---|---:|
| Opening `all` | **663** | **187 opening** and **61 edit** results | Opening **9.574972 s** |
| Existing edit/native `all` | **407** | **114 edit** results | **9.558940 s** |
| Existing observation `all` | **216** | Existing independent observation assertions | **4.763759 s** |

Records are invocations/witnesses, not distinct scenarios. The opening total
counts the single bootstrap once; nested group totals are not added again.
Every public opening result received result-only interpretation before
independent fixture comparison. Public edit results retain their existing
result-only review. All controlled caller deadlines passed.

| Opening top-level group | Records | Public opening calls |
|---|---:|---:|
| `native-boundary` | 264 | 0 |
| `new-open` | 18 | 7 |
| `already-open` | 11 | 11 |
| `preservation` | 13 | 13 |
| `routing` | 33 | 31 |
| `interruption` | 90 | 78 |
| `sequential` | 26 | 25 |
| `composed` | 200 | 18 |
| `privacy-export` | 7 | 4 |

All nine groups executed exactly once. The sequential group made **25** actual
requests: **five** genuinely closed verified openings, **seven** clean and
**ten** dirty already-open recognitions, and **three** unsafe refusals. Twenty-two
requests shared one evolving target; a separate unsafe-survivor fixture supplied
the three refusals. Deliberate typing, same-text dirty versions, three-way
divergence, human Save/close/reopen and actual earlier Undo/Redo preserved current
identity and history without duplicates or late reopens after refusal.

The composed group used the product opener for **18** initial target transitions,
then separate fresh ordinary observation. It reused the existing clean,
conflict, history, durability and twenty-edit sequential campaigns instead of
cloning them. Dirty-different and dirty-equal refusal, exact earlier history,
apply → Undo → ordinary Save → Redo → Save, ordinary cached-Resource consumption,
close/reopen, completed reparse/rescan and fresh runtime value **23** passed.
The twenty-edit sequence ended at **319**, with six interleaved stale/dirty
refusals. Closed observation remained read-only; editing a subsequently closed
target returned `closed_target`/`not_applied` and did not reopen it.

Selected/current/unrelated/other-project sentinels passed through authorized,
ambiguous, denied and interrupted paths, including actual helper/worker failure
and cleanup. Enabled-addon, disabled-addon and hook-only production exports were
inspected and executed with no shipped tooling/native artifacts, credentials,
active listener/registration or gameplay dependency.

### Integration correction and verification

The first composed history run exposed a fixture Save-input issue: a real
product-opened Script existed while the 3D workspace remained visible. Local
CodeEdit focus alone did not deliver a script Save. The private human Save
action now presents the Script workspace and requires a visible focused buffer
before sending its single ordinary shortcut. The failing witness and corrected
composed/full campaigns are retained. No product focus repair, Save, retry or
source mutation was added. Earlier-history verification now requires the exact
original source rather than merely a different string.

The §2 baseline passed formatting, warnings-denied Clippy, **247 Rust tests**
including the doctest phase, rustdoc and locked library/three-caller/example
builds. Normal and separate fault-native builds passed, as did **9 native-build
tests**, **22 workflow tests** and Actionlint. No Rust/native/workflow source
changed after those checks. The [shape and constitutional review](plan.md#t003-implementation-shape-and-constitutional-review-2026-09-30)
records cohesive current-consumer test modules and reuse without new layers.

The opening campaign retained **176** owned-window captures, edit **28**, and
observation **92**. Actual cold/syntax-invalid opening, preserved human tabs,
fifth reopening, composed runtime/sequential durability, full edit durability
and observation dirty-sequence surfaces were inspected. Independent getters and
history transitions, not screenshot appearance alone, establish D/R/B and dirty
state.

### Exact provenance and limits

Private evidence root:
`/Users/petertam/.godot-agent-kit-open-T003-805m51ti`.
These local paths identify the accepted run, not contributor prerequisites.

| Complete summary | SHA-256 |
|---|---|
| `opening-all/summary.json` | `39fe0973ebbeae5d66d29279a85e867bcb6f5670eddf374566b4630a729010e9` |
| `edit-complete/summary.json` | `6f0ddfb5d21907f47382c678609c6bcc0f5fe12ae6f5bd7e5bfa2797236d822e` |
| `observation-all/summary.json` | `4f5696ab58ea9a4ecc9c8be5f307acac8d9ad9ffb02cd83f808b2fabff21f419` |

The opener SHA-256 was
`4a02683638916f1bd0a484d642fcb45101a7805b97ce004a75c9698c9f15e660`.
Normal native build ID:
`836fdbd72f7bb176d76d8b029f4a34dabdcbccc1c4215fb2b6360a80ba6acea2`;
library SHA-256:
`9bc089277461944ef3dd22644eb134ccb062660e92fe03e0c3c9d41da722d881`.
Separate fixture build ID:
`0a9080929dd046f360f5d11824f319283548b8e2de4dde99980bf1d9a80b0a9a`;
library SHA-256:
`31261d5066cc74a94c511da4944e4e5dde899540208340e7790b8d74bd27a0cd`.
`build-provenance.json`, native manifests and suite summaries retain exact
engine/ABI/API/source/driver/caller/template hashes. Toolchains were Rust
**1.98.1**, Python **3.10.9**, Apple clang **21.0.0**, SDK **27.0**, on the exact
§1 stock/macOS candidate.

The initial locked-desktop attempt and failed Save-fixture smoke are not accepted
campaigns. A combined shell invocation completed opening but hit its one-hour
outer deadline during the following edit suite; that incomplete `edit-all`
directory is excluded. The complete edit rerun and separate observation run
above passed. Owned processes exited, the interrupted owned project was removed,
and the desktop-awake assertion was released. Generated artifacts/evidence remain
outside version control.

Opening itself adds no source-history action or reversible-tab-open promise.
The inherited stock-validator endpoint and already-running-malicious-plugin
limitations remain unchanged; no stronger isolation, untested version/platform,
hosted GUI run or roadmap Phase 1 completion is claimed.
