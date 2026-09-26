# Quickstart: Validate Live GDScript Observation

**Status:** Validation/run guide for the approved implementation that will follow this design. The repository currently has no Cargo package, addon, or acceptance driver. Commands below that reference those planned paths become runnable when those artifacts are implemented; they are **not commands reported as passed during planning**. The actual planning GUI experiment and its exact results are recorded in [research.md](research.md#2-actual-planning-feasibility-evidence).

This guide covers the entire observation-only specification. It does not implement the feature, derive tasks, or claim a mutation/UndoRedo/Phase 1 exit guarantee. Use the [data model](data-model.md), [caller contract](contracts/observation-api.md), and [bridge contract](contracts/bridge-protocol.md) for normative fields and outcomes instead of inferring semantics from exit status alone.

## 1. Prerequisites

- Reviewed specification/plan and completed relevant one-task-per-PR implementation dependencies. Do not invoke generic implement-all.
- macOS arm64, exact Godot `4.7.2.stable.official.ed1daf0bf` with engine hash `ed1daf0bf001b61586d9930840f2f1394092c079`. Planning observed macOS 26.6.2; do not extrapolate that evidence to another platform/version.
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

Changing the globally selected Rust toolchain is unnecessary. Do not silently accept a nearby Godot patch version. Verify official binary/template provenance and record executable checksums in the evidence report.

## 2. Native checks and caller build

From `mcp-server/`, after implementation and committed lockfile:

```sh
cargo +1.98.1 fmt --all -- --check
cargo +1.98.1 clippy --all-targets --locked -- -D warnings
cargo +1.98.1 test --locked
cargo +1.98.1 doc --no-deps --locked
cargo +1.98.1 build --locked --bin observe-gdscript
```

`cargo test --locked` includes doctests. There is one package, not a workspace; no blanket `--all-features` is specified. Expected: all checks pass, including comparison/availability/precedence invariants, source-free refusal, malformed input, session replacement, confinement, and deadline transitions. Review the actual lockfile's dependency licenses/provenance and run a separately installed `cargo audit` against that lockfile, recording tool/advisory-database revision and findings. The planning manifest review is not a substitute.

## 3. Fixture driver entrypoint

The planned acceptance driver is `godot-addon/tests/run_observation.py`. Its contract is:

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

For one manual observation after the harness or developer has prepared an editor:

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

- Exact Godot version/hash, OS/architecture, binary checksum, Rust version/lockfile, test-driver revision, selected project/session/resource identity, and fixture preparation actions.
- Per-case observation intervals/durations, source/dirty/open-state authority witnesses, applicable limitations, native screenshots for visible cases, and the failed stage for any non-success.
- All 21 scenarios plus edge cases passing their specified outcomes, twenty-read non-interference, no wrong-target/source leakage, and **every** controlled request ≤5 seconds.
- Native baseline and dependency review results; real-editor and export evidence on the declared candidate matrix. Hosted compile-only CI cannot replace GUI acceptance. Keep privileged GUI runners isolated from untrusted PR code and secrets.
- No claim that complete observation means D/R/B convergence, clean state, permission to edit, applied mutation durability, or actual product UndoRedo support.

During this planning command only the bounded GUI feasibility experiment and design-artifact checks are eligible to be reported as exercised. Full product checks above are prerequisites for later feature/support claims, not an unresolved choice in this plan.
