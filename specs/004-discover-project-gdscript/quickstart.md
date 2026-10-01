# Quickstart: Verify Safe Project GDScript Discovery

**Status:** **T001 is complete**: authenticated read-only editor scope and the coordinated private-v4 cutover passed [focused and complete affected-suite acceptance](#9-t001-scope-and-private-v4-acceptance-2026-10-01). `discover-gdscripts`, its inventory/worker and the public discovery runner remain T002/T003 work. Sections 3–5 describe those future validation surfaces, not executed discovery inventory acceptance. Feature 004 and Roadmap Phase 1 remain in progress.

Use [spec.md](spec.md), [plan.md](plan.md), [data-model.md](data-model.md), [caller v1](contracts/discovery-api.md) and [private bridge v4](contracts/bridge-protocol.md). No successful process exit, nonempty list or editor acknowledgment proves complete discovery.

## 1. Candidate and owned prerequisites

- Existing official Godot `4.7.2.stable.official.ed1daf0bf`, full commit `ed1daf0bf001b61586d9930840f2f1394092c079`, executable SHA-256 `c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf`; macOS 26.6.2 arm64. Other builds/platforms are not discovery support claims.
- Rust 1.98.1/edition 2021, tracked Cargo.lock and existing compiler/native tooling; Python 3.10+. No new package/runtime dependency.
- Unlocked visible desktop and owned synthetic project/editor processes for buffer/selection/history evidence; capture only owned windows. Headless metadata checks cannot establish non-interference with visible buffers.
- Canonical absolute executable paths and fresh empty mode-0700 artifact directories. Resolve a symlink Godot executable before invoking the existing stock-validator/composed harness.
- Existing matched production native revision-2 bundle for the composed opening/editing scenarios, matching export templates and a separate existing fixture-fault bundle for the affected mutation suites. Discovery itself adds no native method/ABI requirement. Follow the existing [native build/operator guide](../../godot-addon/native/README.md) and [opening prerequisites](../003-open-project-gdscript/quickstart.md#1-exact-candidate-and-prerequisites).
- Updated v4 addon and Rust callers must be installed together after implementation; restart the owned editor to create a fresh session descriptor. Do not reuse a v3 descriptor or an earlier lifetime's edit basis.

From the repository root, set `STOCK_GODOT` to the actual absolute executable and `PRIVATE_FAULT_NATIVE` to the separate fixture-only addon/native output. These are local validation inputs, not new product flags or operational services.

**Existing baseline commands:**

```sh
"$STOCK_GODOT" --version
rustc +1.98.1 --version
python3 --version
mcp-server/target/debug/observe-gdscript --help
```

The observer must already be built for the final line; existing build instructions are in the opening guide. Planning exercised the actual observer help/version/bootstrap and exact Godot version, not a new discovery executable.

## 2. Implementation checks and build

For affected Rust work, from `mcp-server/`:

```sh
cargo +1.98.1 fmt --all -- --check
cargo +1.98.1 clippy --all-targets --locked -- -D warnings
cargo +1.98.1 test --locked
cargo +1.98.1 doc --no-deps --locked
```

Default tests include doctests; do not replace them with `--all-targets` alone. Use existing native/workflow/campaign checks when those surfaces change, not a blanket new CI prerequisite.

**T001 build**, from `mcp-server/`:

```sh
cargo +1.98.1 build --locked --lib --bin observe-gdscript \
  --bin open-gdscript --bin edit-gdscript --example stock_validation_fixture
```

**T001 focused scope validation**, from the repository root, with canonical
absolute `STOCK_GODOT` and `OBSERVER` paths and a fresh private directory for
each invocation:

```sh
SCOPE_SESSION_ARTIFACTS="$(mktemp -d "$HOME/discovery-scope-session.XXXXXX")"
python3 godot-addon/tests/run_observation.py \
  --godot "$STOCK_GODOT" --observer "$OBSERVER" \
  --scenario session-boundary --artifacts "$SCOPE_SESSION_ARTIFACTS"
SCOPE_EXECUTOR_ARTIFACTS="$(mktemp -d "$HOME/discovery-scope-executor.XXXXXX")"
python3 godot-addon/tests/run_observation.py \
  --godot "$STOCK_GODOT" --observer "$OBSERVER" \
  --scenario executor-boundary --artifacts "$SCOPE_EXECUTOR_ARTIFACTS"
```

The existing groups consume `discovery_scope_acceptance.py`; there is no new
public runner or inventory command in T001. These focused runs precede the
complete existing observation/edit/open commands in §6 on the integrated
cutover. The generic campaign fingerprint already includes direct Python/GDScript
helpers, observation fixtures and addon sources, so no discovery campaign
registration is needed for this boundary task.

**After T002 implementation**, build the additional current consumer:

```sh
cargo +1.98.1 build --locked --lib --bin discover-gdscripts \
  --bin observe-gdscript --bin open-gdscript --bin edit-gdscript \
  --example stock_validation_fixture
```

Back at the repository root, use absolute paths:

```sh
REPO="$PWD"
DISCOVERER="$REPO/mcp-server/target/debug/discover-gdscripts"
OBSERVER="$REPO/mcp-server/target/debug/observe-gdscript"
OPENER="$REPO/mcp-server/target/debug/open-gdscript"
EDIT_CALLER="$REPO/mcp-server/target/debug/edit-gdscript"
STOCK_VALIDATION_FIXTURE="$REPO/mcp-server/target/debug/examples/stock_validation_fixture"
```

Pure/contract tests must cover real semantic boundaries: no false empty on failed traversal, exactly-at/one-over bounds, entry/marker/root invalidation, local-gap retention versus global suppression, case/Unicode identity, deadline/cancellation and typed outcome precedence. Use actual confined directories and an actually stalled worker for I/O/namespace claims. Do not test only wiring, copied fields, mocked inventory echoes or source text.

## 3. Caller smoke after implementation

**After T002 implementation:** For an explicitly owned fixture project with the updated addon installed/enabled and `GODOT_AGENT_KIT_REGISTRY` set to the same private directory, initialize the existing registry and start its editor deliberately. `PROJECT` and `REGISTRY` must be absolute paths. Discovery does not install/start the editor itself.

```sh
"$OBSERVER" init-registry --registry "$REGISTRY"
"$STOCK_GODOT" --editor --path "$PROJECT"
```

In a separate terminal, with the exact session from authenticated selection or unambiguous omission:

```sh
"$DISCOVERER" --registry "$REGISTRY" --project "$PROJECT" --session "$SESSION"
```

Expected: one bounded JSON result with schema 1, exact selected project/session, observed scope, sorted exact paths, coverage/validity/rechecks and actionable diagnostics. A complete empty listing is permitted only for an independently empty admitted scope. Confirm no inventory on stderr/registry and no source bodies anywhere in discovery output.

Select a returned eligible lowercase-`.gd` path and explicitly call the existing observer and opener. Use a fresh existing edit basis for editing. A returned uppercase/case-variant locator remains exact but does not expand another operation's existing supported profile. Dirty or unsupported follow-on work must retain its existing refusal behavior; discovery is never its authorization.

This manual smoke is not the whole acceptance gate. The owned runner below prepares expected inventory and independent editor witnesses without requiring manual synthetic file construction.

## 4. Owned discovery runner after implementation

**After T002/T003 implementation:** New runner `godot-addon/tests/run_script_discovery.py` must reuse the existing observation harness for owned projects, registry/authentication, real editor actions, independent D/R/B/dirty/selection/history witnesses, capture, cleanup and export checks. Its fixture controls stay private under `fixtures/script_discovery/`, never in product opcodes. It must include the current addon/fixture scripts in independently expected inventory rather than silently filtering them out of the product result.

Focused example with a fresh private artifact directory:

```sh
DISCOVERY_ARTIFACTS="$(mktemp -d "$HOME/discovery-inventory.XXXXXX")"
python3 godot-addon/tests/run_script_discovery.py \
  --godot "$STOCK_GODOT" --discoverer "$DISCOVERER" \
  --observer "$OBSERVER" --opener "$OPENER" --editor "$EDIT_CALLER" \
  --stock-validator "$STOCK_VALIDATION_FIXTURE" \
  --native-fault-addon "$PRIVATE_FAULT_NATIVE" \
  --scenario inventory --artifacts "$DISCOVERY_ARTIFACTS"
```

All paths are absolute. The runner owns actual public caller invocation, final result parsing, elapsed-time measurement and independent witnesses. No fixture invocation of a private walker may substitute for the new caller. T002 supplies the six first-use groups; T003 adds `sequential`, `composed` and `--scenario all`, which runs every group below exactly once. Do not register an incomplete/no-op group or count nested records twice.

| Group | Coverage | Observable proof |
|---|---|---|
| `inventory` | US1.1–US1.4; FR-001–FR-003/FR-006–FR-007; SC-001 | Independently exact set with at least 100 scripts/ten folders, duplicates by basename, closed/never-opened, empty/invalid/read-only/tool/case variants; native visibility parity and complete empty scope. |
| `routing` | US2.1–US2.4; FR-004–FR-005/FR-014; SC-002 | Unique project-only authentication, ambiguity/ended/replaced sessions, unsafe roots and redirects, exact candidate refusal, no inventory before selection and no substitution. |
| `coverage` | US3.1–US3.3/US3.5; FR-007–FR-010/FR-012; SC-003 | Unreadable areas, unsupported paths/markers, exact/one-over bounds, current create/rename/delete, stale editor-cache control, namespace/exclusion races, explicit partial retention versus false empty. |
| `interruption` | US3.4; FR-010–FR-011; SC-003 | Real worker/editor blocking, cancel/disconnect/expiry/disable, malformed/late frames, same-slot overlap, ≤5-second result and owned cleanup with no implicit replay. |
| `readonly` | US4.1; FR-006/FR-016; SC-004 | Clean/dirty/equal-text-dirty/divergent/non-selected documents and unloaded controls retain independent source, selection, dirty state and actual history. |
| `sequential` | US4.2; FR-009/FR-016; SC-004 | At least 20 fresh discoveries interleaving intentional inventory and human-buffer changes; exact new sets or truthful limitations, zero lost work or delayed effects. |
| `composed` | US3.5/US4.3; FR-012/FR-015–FR-016; SC-005 | Public discovery supplies chosen path, then separate fresh observation/open/edit; dirty refusal and existing applicable A–E/Save/reopen/reparse/rescan/runtime durability remain correct. |
| `privacy-export` | US4.4; FR-013–FR-014/FR-017; SC-006 | Selected/other-project inventory and source sentinels, result-only review, no incidental content/credential leak, actual enabled/disabled/hook-only exports inspected and run. |

All 17 story scenarios, 17 FRs and six SCs are covered. Result-only interpretation and terminal timing apply across groups, not only their named rows. No test may weaken the useful complete path into universal limited/refused results.

## 5. Required controls and expected distinctions

### Visibility and current scope

Prepare dot files/folders, `.gdignore` parent/descendant, nested `project.godot`, both effective data-directory modes, a visible ordinary addon and files excluded only by VCS/export settings. On this exact macOS candidate, a `chflags hidden` file/folder remains Godot-visible; do not mistake Finder metadata for Godot visibility. Root `.gdignore` is not a child-subtree exclusion. Use the native filesystem's literal marker lookup behavior, including the candidate filesystem's case behavior.

Compare a stable owned tree with native path/name getters as the independent policy oracle, then add/rename/remove files outside Godot without requesting a rescan. Include the demonstrated idle-cache-misses-new-file negative control. The product must return fresh metadata membership or actual limitations, not a falsely complete cached list. Change the live `use_hidden_project_data_directory` setting after startup and prove the effective editor-path getter still governs the actual data exclusion; never read source/project configuration to manufacture it.

Empty/invalid/large-source scripts remain discoverable by path; their contents must not be read as part of discovery. A directory named `something.gd` is not itself an entry, though its visible child scripts remain eligible. An unsaved/deleted-file buffer or embedded script is not a standalone existing-file entry, but its native state and the existing observer behavior remain intact.

### Namespace, permission and representation

Use independently owned outside sentinels and actual symlink/file/parent replacements at deterministic fixture barriers before enumeration, before batch publication and during recheck. Verify no link target or outside inventory is read/returned. A symlink marker cannot certify that a subtree is safely excluded. Distinguish access failure from absence; a safe marker's contents are irrelevant.

Use unreadable folders/files, granted ACL/unsafe ownership or writable-component controls, and root replacement. Local gaps retain other safe entries with limitations; root/authentication failure clears inventory. Replace/remove `.gdignore` or nested-project markers during traversal and require invalidation even if later samples look like the original. No arbitrary-writer atomicity is claimed.

Cover legal spaces/Unicode, duplicate basenames, case-distinct paths where the filesystem supports them, and control/non-UTF8/decorated/over-length names. Exact locators round-trip; unsupported names cause a reason at a safe parent, not a lossy replacement. Distinct valid hard-link names are distinct paths. State a filesystem-specific inapplicability when a case-distinct fixture cannot exist; do not claim it passed.

Exercise each [fixed profile bound](data-model.md#2-fixed-supported-profile) at the boundary and one beyond, including deep/large directories before sorting/allocation. Include a zero-entry limited result and diagnostic overflow. Never infer completeness from count alone or silently reduce the expected set to fit a limit.

### Interruption and preservation

Use actual process/connection barriers before selection, after scope, between checked batches and before/after recheck. SIGINT/SIGTERM, known EOF, held worker syscall, unresponsive editor, owner expiry and disable must produce the specified reason/earlier evidence within five seconds. Malformed frames never contribute entries. Release held deliveries afterward and prove no late terminal upgrade, replay, source or editor effect.

While one real observation/open/edit holds the existing slot, discovery must return busy without queueing; reverse ownership and verify those callers cannot bypass the same slot. Discovery cancellation must not disturb an existing native entered owner's retention rules. Terminate only owned fixture children; never kill or restart a user's editor for a timeout.

For non-interference, independently witness D/R/B, buffer/resource identity, dirty indication, selection, current/saved versions and prior native history before/after. Include equal-text-but-dirty and non-selected dirty tabs. Exercise actual Undo/Redo after discovery to establish history reachability, not merely `has_undo`. Fixture preparation/history replay is deliberate test activity, never a discovery capability.

## 6. Cumulative affected-boundary gate

During development, run the affected discovery group and directly affected regressions first. For final feature acceptance, the v4 authenticated cutover affects all existing callers, so run complete discovery, observation, edit and opening suites serially on the same unchanged implementation head with separate private artifacts. Preserve failed/incomplete evidence; do not relabel focused passes as the full gate.

**After implementation**, the complete discovery invocation is the command in §4 with `--scenario all` and a fresh directory. Existing full suites use:

```sh
python3 godot-addon/tests/run_observation.py \
  --godot "$STOCK_GODOT" --observer "$OBSERVER" \
  --scenario all --artifacts "$OBSERVATION_ARTIFACTS"
python3 godot-addon/tests/run_script_edit.py \
  --godot "$STOCK_GODOT" --observer "$OBSERVER" --editor "$EDIT_CALLER" \
  --stock-validator "$STOCK_VALIDATION_FIXTURE" \
  --native-fault-addon "$PRIVATE_FAULT_NATIVE" \
  --scenario all --artifacts "$EDIT_ARTIFACTS"
python3 godot-addon/tests/run_script_open.py \
  --godot "$STOCK_GODOT" --opener "$OPENER" \
  --observer "$OBSERVER" --editor "$EDIT_CALLER" \
  --stock-validator "$STOCK_VALIDATION_FIXTURE" \
  --native-fault-addon "$PRIVATE_FAULT_NATIVE" \
  --scenario all --artifacts "$OPEN_ARTIFACTS"
```

Each named artifact variable must be a new empty mode-0700 directory. The existing [campaign utility](../../.github/README.md#focused-validation-and-resumable-campaigns) may wrap these runs. Its planned discovery registration adds `--suite discovery` and `--discoverer` only alongside the real new runner, updates execution fingerprints for its actual inputs, and extends `--suite all` to include discovery. Until that registration exists, its current `all` does not mean discovery coverage. No separate workflow or approval service is introduced.

The composed workflow must independently preserve applicable A–E and Save/close-reopen/reparse/rescan/runtime behavior. Runtime launch, source preparation, Save and history interactions belong only to authorized fixtures and the existing guarantees, not new product controls. Document substantive inapplicability; missing access is not passing evidence.

## 7. Evidence review and completion

For every public result, first answer from the result alone: intended/resolved target or refusal; scope; completeness versus limitation; usable versus earlier entries; unavailable checks; and safe next action. Then compare with independent namespace and real-editor witnesses and elapsed time. Review exact path sets, not only nonempty/count assertions. Preserve source-free diagnostics and distinguish known invalidation from unknown stability.

Record executable/library/addon/fixture/contract provenance, exact environment, actual scenario coverage and measured deadlines. Inspect actual enabled, disabled and hook-only production export contents and run them to establish no active discovery tooling or gameplay dependency. Keep public compatibility changes intentional and the supported profile exact.

Before any implementation task is marked complete, perform the repository's proportional responsibility/visibility/state/evidence-error review, including the large shared files touched by v4 and selector reuse. No new task or separate approval gate is invented for this review. Feature completion requires positive discovery, all required refusals/non-interference/composed/privacy/export gates and affected cumulative evidence—not a draft PR or a successful build.

## 8. Planning evidence actually executed

[Research §8](research.md#8-research-evidence-and-limitations) records the exact owned runtime observations, local evidence paths/hash and limitations: 23 getter invocations, 100-script catalog, stale-cache negative control, twenty non-interfering repetitions with native history replay, three metadata projections, effective project-data getter evidence and an inspected owned-window image. The failed direct static getter call and its correction are recorded rather than hidden.

All temporary projects, registry credentials, probe source/binaries and owned editor processes were removed/stopped. Synthetic evidence remains local for provenance, not a contributor prerequisite. No product discovery, v4, full acceptance suite or wider support claim follows this planning work. The proposed commands above become runnable deliverables of the subsequent approved implementation tasks.

## 9. T001 scope and private-v4 acceptance (2026-10-01)

**T001 is complete**: the real source-free scope owner and coordinated private-v4
cutover passed focused scope probes, complete unfiltered observation/edit/open
suites and implementation-shape review. All three complete suites ran serially
with unchanged production/fixture inputs. T002/T003 and the public discovery
inventory remain pending; PR review/merge state does not determine completion.

### Executed boundary and regression proof

| Run | Observed evidence |
|---|---|
| Focused `session-boundary` | **74 records passed**: three-role Rust/Python/GDScript vectors, eight-bit/native metadata authentication, v3/reflection/replay/identity refusals, strict discovery tuples, 4096-byte admission/4097-byte refusal and bounded framing. |
| Focused `executor-boundary` | **35 records passed** after the evidence-field correction below: real hidden/visible effective data paths, unsaved setting distinction, real scan/import sampling, getter-negative cases, lease/stage cleanup and actual prior history. |
| Complete observation | **271 records passed**, including **53 scope-prefixed records**, all 13 quickstart groups, source-free routing, partial authorities, repeated read-only use, privacy and all three actual export modes. Maximum recorded bounded operation **4.764 s**. |
| Complete edit/native | **408 records / 114 public edit results passed**, including discovery busy refusal during a real edit owner, existing A–E/history/durability and privacy/export behavior. Every public edit result received result-only review. Maximum recorded operation **9.526 s**. |
| Complete opening | **664 records / 187 public opening results passed**, including discovery busy refusal during a real opening owner, native entered-owner retention, repeated opening, composed A–E/Save/reopen/reparse/rescan/runtime and all three exports. Every public opening result received result-only review. Maximum recorded caller time **9.511 s**. |
| Rust baseline | Formatting, warnings-denied Clippy, **247 tests** including the doctest phase, rustdoc and locked library/three-caller/validator-example build passed. |

Counts are invocation/evidence records, not distinct user scenarios. Nested
groups and focused runs are not added to complete-suite totals.

The effective-path positives use the actual `EditorInterface` getter for
`res://.godot/editor` and `res://godot/editor`; a live unsaved configuration change
does not replace that effective path. The visible-data fixture has **no native
bundle installed** and authenticates native revision 0 with edit/open false while
discovery scope remains available. A separate false-discovery fixture still
completes ordinary observation/recheck.

Scan/import positives sample actual public `is_scanning()` / `is_importing()`
facts during explicit **fixture-owned** scan and synchronous reimport activity.
Separate synthetic getter-negative cases are labeled `getter_fault_fixture`;
the epoch test deliberately emits the public `filesystem_changed` signal. None
is a product rescan, parser, inventory or exhaustive-freshness claim.

Independent disk bytes/mtime and actual Script/CodeEdit identity, source,
current/saved versions, dirty state, tab/selection and cached-resource witnesses
remain unchanged across scope calls. Each data profile replays four genuine
native Undo/Redo transitions. Owned before/after window captures were inspected;
the invalid-source diagnostic is deliberate fixture state, not a discovery parse.
Finish/abort, duplicate/wrong-owner stages, expiry/late requests, channel loss,
disable/re-enable and same-slot exclusions receive real bridge coverage.
The 1 ms endpoint proves queued admission and the exact 1000 µs lease through a
private witness; it does not require a reply after the lease has expired.

Observation's enabled, disabled and hook-only packs exclude the new owner and
fixture scripts; each exported runtime exits 0 with **zero observed listeners**.
The existing generic guard/preset and campaign fingerprint already cover the new
files. No native source, ABI, build mechanism, workflow or campaign registration
changed, so unrelated native-build/workflow suites were not rerun.

### Corrections, provenance and limits

The initial Rust run found two old v3 expectations in the opening fixture's
multiline verify/recheck tuples. They were migrated; all **14** affected opening
boundary tests and then the complete Rust suite passed. The initial executor
smoke completed its assertions but failed the group gate because context
`status` overwrote the harness's pass marker. Renaming that evidence key to
`context_status` corrected the record; `scope-executor-fixed` passed. The
original failed `scope-executor` is retained and is not acceptance.

Evidence root:
`/Users/petertam/.godot-agent-kit-discovery-T001-wghzy3sk`.
These local paths identify this run, not contributor prerequisites.

| Accepted summary | SHA-256 |
|---|---|
| `scope-session/summary.json` | `df80f775a63591c1691348c39197733590f4e0ab0e945c24bb4865b836f5dc75` |
| `scope-executor-fixed/summary.json` | `a6db0c6060bf89181acb549a5b8f2b260b04e65f702a173f606f793b9a7b522d` |
| `observation-all/summary.json` | `9d9a040400f7ff7e03057dc42e3d457005c52da213ee49b57f59606a829b3014` |
| `edit-all/summary.json` | `c2267ce62ab16b4b4a0fb6a3b855fb27950c8817ef9020decd7c731f271804ca` |
| `opening-all/summary.json` | `27b2466cae241418adcd03ba1470f83e2d6ca5c160b8267b4bae220498132c37` |

The exact §1 stock executable, macOS 26.6.2 arm64, Rust 1.98.1 and Python 3.10.9
were exercised. `build-provenance.json` records reuse of the unchanged normal and
separate fault-native revision-2 bundles after checking every recorded native
source hash, generated ABI/API/header and library hash. Production native build
ID is `836fdbd72f7bb176d76d8b029f4a34dabdcbccc1c4215fb2b6360a80ba6acea2`;
fault build ID is
`0a9080929dd046f360f5d11824f319283548b8e2de4dde99980bf1d9a80b0a9a`.
No transport-only native rebuild was needed.

The caller SHA-256 values are:

- Observer: `d62818b718c3f3b2efc434e31bd97126e24314c54808d8fe0f6b0d3690e789eb`.
- Editor: `8403c154ad76f13f297029d729b22a92fb7ed71cc091399c87b70f1922529542`.
- Opener: `5c1ab1518883cbc68d584f69b7a6c615c07be36e31b20f107f9abfbc1d6b931d`.

The runners completed owned editor/worker/runtime cleanup; the owned desktop-awake
process was stopped and reaped. Evidence remains outside version control. No
extension pre/post hooks were registered.

The [shape/constitutional review](plan.md#t001-implementation-shape-and-constitutional-review-2026-10-01)
records the current-consumer module boundaries and unchanged native ownership.
No inventory, public discovery timing, stronger arbitrary-writer isolation,
untested version/platform or Feature 004/Phase 1 completion is claimed.
