# Quickstart: Verify Safe Clean-Document GDScript Closing

**Status: Phase 1 design, not implemented or accepted.** This guide separates existing runnable baselines from **proposed, not implemented** close builds, caller and runner interfaces. There is currently no `close-gdscript`, `run_script_close.py`, campaign `--suite close` or campaign `--closer` flag. Native research does not establish product completion, and completing this feature does not complete roadmap Phase 1.

Read [spec.md](spec.md), [plan.md](plan.md), [data-model.md](data-model.md) and [research.md](research.md). The [constitution](../../.specify/memory/constitution.md), [architecture](../../ARCHITECTURE.md) and [working agreement](../../AGENTS.md) govern acceptance. An editor/native acknowledgment, matching source pair, successful process exit or fixed sleep is not verified closure.

## 1. Exact candidate and owned prerequisites

- Official Godot **`4.7.2.stable.official.ed1daf0bf`**, full commit **`ed1daf0bf001b61586d9930840f2f1394092c079`**, executable SHA-256 **`c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf`**; research host **macOS 26.6.2, build 25G83, arm64**. These identify the candidate to validate, not an already supported closing environment or broader platform claim.
- Rust **1.98.1**, edition 2021, tracked Cargo.lock and existing rustfmt/Clippy components; existing C++17 Apple compiler/SDK; Python **3.10+**. Use matching official export templates; the existing candidate template SHA-256 is `88df5e2e6fee99088699be66e6d42e4da4fb0c5619d054297d755a49558a4792`. No engine patch, new dependency or package installation is selected.
- An unlocked visible desktop, explicitly owned synthetic projects and editor/runtime processes, and permission to capture only owned windows. Inspect actual captures; headless/getter evidence alone does not prove visible buffer/history preservation.
- Canonical absolute executable/input paths; fresh empty mode-0700 artifact directories; a draining stdout consumer; no human source or credentials in fixtures. Resolve a symlink Godot executable before supplying the existing stock validator.
- Use the existing [native build/operator guide](../../godot-addon/native/README.md) and separate production versus fixture-fault bundles. After implementation, install matched **native family revision 3**, retaining `editor_integration.gdextension` and `libeditor_integration.macos.arm64.dylib` artifact names. The current revision-2 bundle is baseline evidence only, not close capability.
- Install all Rust/addon peers together for the proposed **private bridge v5**, which appends `close_gdscript` as the ninth capability. Restart owned editors for new descriptors; never reuse v4 descriptors or earlier session-bound bases. Public observation/edit/open/discovery v1 and their existing limits/deadlines remain unchanged. No compatibility fallback is proposed.

Record actual executable, library, manifest/build ID, generated ABI/API, source, addon, fixture, caller, validator and toolchain hashes for each accepted run. Version strings/manifests alone do not establish native compatibility. Generated binaries, editor state, credentials and evidence are not committed.

### Existing runnable baseline commands

From the repository root, set `STOCK_GODOT` to the actual canonical absolute executable and `PRIVATE_FAULT_NATIVE` to a separate owned fixture-only output directory:

```sh
"$STOCK_GODOT" --version
rustc +1.98.1 --version
python3 --version
python3 godot-addon/native/build.py --godot "$STOCK_GODOT"
python3 godot-addon/native/build.py --godot "$STOCK_GODOT" \
  --fixture-faults --output-addon "$PRIVATE_FAULT_NATIVE"
```

These are existing interfaces. Running them today builds the current integration, **not revision-3 closing**. The normal library must not contain fixture-fault controls; the separate fixture artifact must never replace it in production or enter an export.

From `mcp-server/`, existing checks and build commands are:

```sh
cargo +1.98.1 fmt --all -- --check
cargo +1.98.1 clippy --all-targets --locked -- -D warnings
cargo +1.98.1 test --locked
cargo +1.98.1 doc --no-deps --locked
cargo +1.98.1 build --locked --lib --bin observe-gdscript \
  --bin edit-gdscript --bin open-gdscript --bin discover-gdscripts \
  --example stock_validation_fixture
```

Default tests include doctests; `--all-targets` alone is not a replacement. Apply existing native-build/workflow checks when their surfaces change, without inventing a new provider or approval prerequisite.

Back at the repository root, existing binary variables and smoke commands are:

```sh
REPO="$PWD"
OBSERVER="$REPO/mcp-server/target/debug/observe-gdscript"
OPENER="$REPO/mcp-server/target/debug/open-gdscript"
EDIT_CALLER="$REPO/mcp-server/target/debug/edit-gdscript"
DISCOVERER="$REPO/mcp-server/target/debug/discover-gdscripts"
STOCK_VALIDATION_FIXTURE="$REPO/mcp-server/target/debug/examples/stock_validation_fixture"
"$OBSERVER" --help
"$OPENER" --help
"$DISCOVERER" --help
```

A command being runnable is not a statement that this planning change executed it or passed its acceptance.

## 2. Proposed build and caller smoke after implementation

**Future contract, not runnable today:** from `mcp-server/`, build the integrated consumers:

```sh
cargo +1.98.1 build --locked --lib --bin close-gdscript \
  --bin observe-gdscript --bin edit-gdscript --bin open-gdscript \
  --bin discover-gdscripts --example stock_validation_fixture
```

Rebuild production and separate fault native bundles using the existing §1 commands after revision-3 sources/build contracts exist. Observe actual matched loading and capability negotiation, not just successful compilation.

For manual smoke, use an explicitly owned fixture project with the updated addon installed/enabled. `PROJECT`, `REGISTRY` and `STOCK_GODOT` are canonical absolute paths; set `GODOT_AGENT_KIT_REGISTRY` to the same private registry before deliberately launching its editor. Existing setup/caller syntax:

```sh
export GODOT_AGENT_KIT_REGISTRY="$REGISTRY"
"$OBSERVER" init-registry --registry "$REGISTRY"
"$STOCK_GODOT" --editor --path "$PROJECT"
```

In a separate terminal, select the exact authenticated session (or omit `--session` only when selection is unambiguous). Use discovery to choose a supported exact `res://` script locator. Existing commands:

```sh
"$DISCOVERER" --registry "$REGISTRY" --project "$PROJECT" --session "$SESSION"
"$OPENER" --registry "$REGISTRY" --project "$PROJECT" \
  --session "$SESSION" --script "$SCRIPT"
SMOKE_ARTIFACTS="$(mktemp -d "$HOME/close-smoke.XXXXXX")"
"$OBSERVER" --registry "$REGISTRY" --project "$PROJECT" \
  --session "$SESSION" --script "$SCRIPT" > "$SMOKE_ARTIFACTS/basis.json"
```

First review the observation: it must be a usable clean, target-bound observation-v1 basis with performed consistency checks, no detected invalidation and independently agreeing D/R/B; it does not promise atomic stability. A failed/limited/closed observation is not effectful authorization. Do not Save, synchronize or discard to make the request eligible. The runner in §3 supplies controlled positive fixtures without manual tab preparation.

**Proposed caller contract, not implemented:** `CLOSER` is the canonical absolute built close binary. Its selector flags reuse existing spelling; exactly one stdin JSON value plus EOF contains only `schema_version`, `request_id` and `basis`. The basis is the complete prior observation-v1 result, not a new token or an edited source override. A fresh checked request ID must differ from the observation ID.

```sh
CLOSER="$REPO/mcp-server/target/debug/close-gdscript"
python3 -c 'import json,sys; print(json.dumps({"schema_version":1,"request_id":"close-smoke-1","basis":json.load(open(sys.argv[1]))}))' \
  "$SMOKE_ARTIFACTS/basis.json" | \
  "$CLOSER" --registry "$REGISTRY" --project "$PROJECT" \
    --session "$SESSION" --script "$SCRIPT"
```

Expected positive result: schema 1, `operation: close_gdscript`, `verified_newly_closed`, attributed removal of the original document, fresh confirmed target absence, unchanged admitted D/file identity, independently unchanged retained R or observed unloaded R, completed/not-applicable required native continuation and preserved protected documents. B and document-buffer dirty state become **not applicable**, not empty/clean. A selected target may cause reported ordinary native fallback selection; a non-selected target must leave the current selection intact. No target parse-validity or write-permission prerequisite is added.

Then separately use the existing observer to witness closed state. **Future no-effect recognition smoke**:

```sh
printf '%s\n' '{"schema_version":1,"request_id":"close-smoke-2","basis":null}' | \
  "$CLOSER" --registry "$REGISTRY" --project "$PROJECT" \
    --session "$SESSION" --script "$SCRIPT"
```

Expected: `already_closed_unchanged`, no load/open/close, context validation or selection/history/source effect. A null basis must refuse if the target has become open. A valid supplied old open basis can recognize the same valid file/session as closed only with `expected.use: not_applied_to_closed_state`; it cannot transfer to a replacement file/session/buffer. Closed recognition is not proof of project-wide cleanliness or of an earlier interrupted attempt's success. Reopen only as a separate intentional existing opener call, then obtain fresh observation for any edit/close.

Proposed exits: **0** for either verified new closure or already-closed recognition; **3** for proven refused/not applied; **4** for `applied_unverified` or `effects_unknown`; **2** for malformed CLI/input; **1** for unexpected host delivery failure. Inspect structured evidence, not exit code alone. Possibly applied outcomes require fresh observation of the explicit original target, never automatic retry, rollback or compensating reopen.

## 3. Proposed owned closing runner after implementation

**Not implemented:** `godot-addon/tests/run_script_close.py` reuses existing observation/open/edit/discovery harness ownership, authentication, source/history/selection witnesses, owned-window capture, private barriers, cleanup and export checks. Controls remain fixture-only, never arbitrary method dispatch or product opcodes. `--closer` names an absolute close binary; all other input flags below reuse existing interfaces.

**Future focused command** (new empty private artifacts for every invocation):

```sh
CLOSE_ARTIFACTS="$(mktemp -d "$HOME/close-native-boundary.XXXXXX")"
python3 godot-addon/tests/run_script_close.py \
  --godot "$STOCK_GODOT" --closer "$CLOSER" \
  --observer "$OBSERVER" --opener "$OPENER" --editor "$EDIT_CALLER" \
  --discoverer "$DISCOVERER" --stock-validator "$STOCK_VALIDATION_FIXTURE" \
  --native-fault-addon "$PRIVATE_FAULT_NATIVE" \
  --scenario native-boundary --artifacts "$CLOSE_ARTIFACTS"
```

Proposed groups are `native-boundary`, `clean-close`, `already-closed`, `preservation`, `routing`, `interruption`, `sequential`, `composed`, `privacy-export`, plus `all`. The same command with a new directory and `--scenario all` must execute every group once, serially. No incomplete/no-op group may be advertised as complete acceptance. Private native controls cannot substitute for the public closer in product cases.

| Group | Specification coverage | Required observable proof |
|---|---|---|
| `native-boundary` | US1.1–US1.2/US1.4; US2.4–US2.5; US3.1–US3.2/US3.5; FR-003–FR-006/FR-008–FR-014; SC-001–SC-003/SC-006 | Exact-path generated-bound close, final target/complete-roster guards, source-only per-purpose receipts, one-shot entry/discard/owner retention, actual visited/completed native event fencing and independent final witnesses. |
| `clean-close` | US1.1–US1.2/US1.4–US1.5; FR-001/FR-003/FR-005–FR-006/FR-008–FR-010/FR-014; SC-001 | Selected/non-selected/last-tab, empty/read-only/syntax-invalid target positives; no first-select, source write or repair; actual closed state and separately intentional observe/reopen with persisted source. Include a useful stable two-document positive. |
| `already-closed` | US1.3; US2.3; FR-007–FR-010/FR-014; SC-002/SC-006 | Cached, genuinely unloaded and source-limited recognition; zero lifecycle/history/selection/source effects; no effect-profile validation; null and valid non-applied old basis distinctions, never unknown/missing file as closed. |
| `preservation` | US2.1–US2.5; FR-003–FR-005/FR-008/FR-012/FR-014; SC-002/SC-006 | Dirty-different/equal-text-dirty target refusal, divergent/stale/unavailable surfaces, absent/unusable/stale basis, same-text reopened IDs/versions, boundary races, dirty-current and background R==B positives, real unrelated history; unsafe pending-source application refuses. |
| `routing` | US2.6; US3.5; FR-001–FR-002/FR-004/FR-010/FR-013–FR-016; SC-002/SC-006 | Ambiguous/ended/replaced/unauthorized sessions, missing/denied/outside/embedded/non-GDScript/custom/unsafe contexts and namespace races; no candidate source, fallback targeting, implicit startup, queue or late effects. |
| `interruption` | US3.1–US3.5; FR-004/FR-009–FR-013; SC-003 | Real cancellation/timeout/EOF/disable/child/channel loss before entry, inside close, during native settling and fresh verification; unresponsive editor; truthful known/partial/unknown effects within ten seconds, survivor newer work and irrevocable no-late-entry refusal. |
| `sequential` | US4.2; FR-004/FR-007–FR-008/FR-011–FR-014/FR-017; SC-004 | At least 20 actual public close requests including ≥5 actual clean closes, ≥5 already-closed recognitions and ≥5 dirty refusals including equal-text-dirty, with human edits/reopenings and stale/unsafe requests. Zero lost work, source changes or old-request closure of new buffers. |
| `composed` | US4.1/US4.3–US4.4; FR-015/FR-017–FR-018; SC-005–SC-006 | Product discovery → open → observe → edit → fresh observe → close → existing reopen; independent persisted D/R/B and applicable A–E, history and ordinary durability; existing read-only/closed-edit refusal and deadlines unchanged. |
| `privacy-export` | US4.5; FR-002/FR-009–FR-010/FR-016–FR-018; SC-007 | Result-only review, authorized/denied/ambiguous/interrupted source sentinels, all private child staging/cleanup, no incidental/unselected source or credentials, actual enabled/disabled/hook-only production exports inspected and run. |

The matrix covers all **21 story cases (5 + 6 + 5 + 5), FR-001–FR-018 and SC-001–SC-007**. FR-009/result evidence, FR-010/outcome precedence, FR-011/timing, FR-016/privacy and SC-007/result-only interpretation apply to every relevant case, not only their named row. Positive useful closing is mandatory; universal refusal or primitive-only success is incomplete.

## 4. Boundary recipes and independent witnesses

Before/after each case, independently acquire disk bytes/hash/stat/file identity; actual retained Script identity/source/edited state; exact ScriptEditorBase/CodeEdit association, text, current/saved versions and attributed dirty state; complete supported open-document roster; selected identity; and actual relevant native history. Distinguish absent, unavailable, invalidated and earlier evidence. Do not echo preparation or native replies as post-state proof, force-load R, or reopen to verify closure. Release target-only attempt Resource references before ordinary post-close R observation so retention/unloading is not manufactured.

### Complete effect context and source-application refusal

Guard **every remaining supported open script document**, not just the selected tab: native history navigation and script-list sorting can visit others. Exercise the fixed bounds at the boundary and one over: eight open documents including target; each target authority ≤512 KiB; remaining retained R==B source aggregate ≤512 KiB; effect metadata aggregate ≤256 KiB; existing 4 KiB control, 4 MiB selected source-bearing request and 12 MiB response/input/output ceilings. Include unsupported mixed/custom contexts and representation/availability limits. Never omit a candidate, normalize/truncate source or invent an empty value to fit admission.

Use dirty-but-current **R == B** selected and background positives with document-attributed flags/versions and real history. Separately recreate the research's **R != B pending-source application negative** in an owned primitive-only control; the product regression must refuse **before close entry**, preserve the target open and preserve unrelated R/B/source/dirty/history. The unsafe primitive outcome is evidence of the risk, not allowed product behavior. Include actual pending exported-property/stale compiled/effectful tool/base/static/load/preload/global/autoload contexts sufficient to challenge reused lexical, compiled-property/method and effective-context guards; no fictional pending-empty getter.

For each remaining document, verify exactly bound source-only `close_context` validation receipts, including wrong-purpose (`open_context`/edit), wrong source/identity/settings/guard, invalid/incomplete results, diagnostics/parser-symbol fences, child failure and cleanup. One child runs at a time, sharing the original deadline. Target parse validity is **not** required; a safe invalid-source target must close without repair. No remaining document means no helper. Already-closed recognition does not require effect-context admission.

### Actual native continuation fencing

At the admitted one-shot native boundary, observe attempt-owned `editor_script_changed` visits and exact protected editor `edited_script_changed` validation completions. Test pre-entry events, wrong/reopened editor events, missing callbacks, repeated visits and a later visit after an earlier completion. Only actual attributable completion covering each surviving visited editor discharges the obligation; a later visit invalidates an earlier completion until a new qualifying event. Out-of-set visits or source/identity/version/flag/roster/configuration changes invalidate, never expand/revalidate the set under old authority.

Then separately reacquire fresh target absence, D/file identity, retained R or observed unloading, and protected source/dirty/selection/history evidence. Native return `OK`, equal sources, timer duration or a copied ledger count alone cannot pass. Do not force/flush validation, call a parser, pump events inside close, stop timers or use fixed sleeps as completion proof. A pending/missing/invalidated continuation at the deadline yields applied-unverified when effects are known, not success or rollback.

### Identity, newer work and real history

Use same-text human close/reopen, same-text version changes, typing/Save after preparation, path/parent/leaf replacement, session replacement and newly opened/removed tabs at private deterministic barriers. Before entry, invalidation refuses; after closure, human reopening/editing during settling/verification must retain the new identity/text/history and report known effects with non-success. Never reclose, Save, restore old source or compensate by reopening.

Verify the intended roster transition rather than rejecting every successful close: post-state equals the original roster minus exactly the closed target's editor/buffer, with protected identities intact. An extra removal, new/reopened document or same-path cached Script replacement remains invalidation even if source bytes match. Include a retained-original-R and a genuinely unloaded-R control; do not hold the target alive solely to avoid the latter.

Exercise actual Undo and Redo in unrelated documents after successful close, refusal and already-closed recognition, including dirty equal-text work. A history-available flag or action registration is not proof of reachability. Successful closing may dispose of **target-local** history only as ordinary native buffer destruction; refusal/recognition dispose of none. No synthetic source action or undoable-tab-close guarantee is introduced.

### Interrupted/overlapping attempts

Hold real entry/return/continuation/verification deliveries; exercise SIGINT/SIGTERM, known EOF, disable/expiry, malformed/late frames, child loss, held native calls and an unresponsive editor. Measure externally with stdout drained: **9.5 seconds work + 0.5 seconds delivery**, at most ten seconds total; all children share the original clock. Retain entered native/callback ownership until return. Release controlled barriers after terminal results and witness survivor state; missing acknowledgment is never proof of no effect.

Hold the same slot with real observation/edit/open/discovery/close in both applicable ownership directions. Busy refusal is irrevocable, not queued. Late frames cannot upgrade a terminal result or close a newer document. Cleanup may release owned barriers and reap children; it must not kill/restart a developer's editor, roll back source/selection or claim that native activity ceased solely because the caller timed out.

## 5. Composed A–E and ordinary durability

The `composed` group must use public discovery for the path, the product opener for preparation, the existing fresh observer/edit basis for editing, a **new observation after edit** for closing, the product closer for closure and existing opener for deliberate reopening. A close summary is not an observation-v1 edit basis. No fixture direct-close helper or manual reconciliation can stand in for product closure.

- **A:** real clean edit independently converges D == R == B before a fresh-basis close.
- **B:** dirty-different and equal-text-dirty work survives existing edit refusal and close refusal.
- **C:** while the relevant buffer exists, actual apply → Undo → ordinary Save → Redo → Save establishes all applicable native/coherence transitions; unrelated human histories remain usable through close. Do not promise target history after its ordinary destruction.
- **D:** product close followed by existing reopen preserves the intended persisted revision and independently coherent sources.
- **E:** the existing twenty fresh-basis edit stress/interleaving cases retain source without loss/divergence/reversion on Save, alongside the separate closing-request minima in §3.

Ordinary Save, human history interactions, explicit fixture reparse/rescan and permitted owned runtime launch must preserve intended behavior, not just exit 0. These are authorized acceptance actions, **not close product commands**. Existing discovery/observation remain non-interfering, editing still refuses closed targets, and no existing operation implicitly closes. Record substantive inapplicability; missing access or observation is not a pass.

## 6. Focused development versus cumulative acceptance

Follow the existing [focused/campaign policy](../../.github/README.md#focused-validation-and-resumable-campaigns). During development, run the changed/failing close group and directly affected regressions first. Do not rerun all historical GUI suites merely because they exist. However, the actual **v5 authenticated cutover, native revision-3 family loading and shared edit/open/close slot/lifetime changes** can invalidate existing callers: their cutover gate requires complete affected existing observation/edit/open/discovery suites. Final feature acceptance additionally requires **all nine close groups** and composed/privacy/export evidence on stable unchanged runtime inputs.

Existing complete-suite interfaces below are runnable today with built baseline inputs; on the eventual cutover head they are required regression evidence. Every artifact variable is a distinct fresh empty private directory:

```sh
OBSERVATION_ARTIFACTS="$(mktemp -d "$HOME/close-regression-observation.XXXXXX")"
EDIT_ARTIFACTS="$(mktemp -d "$HOME/close-regression-edit.XXXXXX")"
OPEN_ARTIFACTS="$(mktemp -d "$HOME/close-regression-open.XXXXXX")"
DISCOVERY_ARTIFACTS="$(mktemp -d "$HOME/close-regression-discovery.XXXXXX")"
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
python3 godot-addon/tests/run_script_discovery.py \
  --godot "$STOCK_GODOT" --discoverer "$DISCOVERER" \
  --observer "$OBSERVER" --opener "$OPENER" --editor "$EDIT_CALLER" \
  --stock-validator "$STOCK_VALIDATION_FIXTURE" \
  --native-fault-addon "$PRIVATE_FAULT_NATIVE" \
  --scenario all --artifacts "$DISCOVERY_ARTIFACTS"
```

Run serially on the visible owned environment; existing observation's five-second and edit/open ten-second bounds remain. These suites alone do not prove close composition.

**Future campaign contract, not implemented today:** register all nine close groups in the existing campaign utility, reuse its input fingerprints and add absolute `--closer`. `--suite close` runs close once; future `--suite all` includes complete **open, edit, observation, discovery and close** suites, each once, serially. No new runner/service/provider topology is introduced.

```sh
CLOSE_CAMPAIGN="$(mktemp -d "$HOME/close-campaign.XXXXXX")"
python3 godot-addon/tests/run_editor_campaign.py \
  --suite close --campaign-dir "$CLOSE_CAMPAIGN" --keep-going \
  --godot "$STOCK_GODOT" --closer "$CLOSER" \
  --observer "$OBSERVER" --opener "$OPENER" --editor "$EDIT_CALLER" \
  --discoverer "$DISCOVERER" --stock-validator "$STOCK_VALIDATION_FIXTURE" \
  --native-fault-addon "$PRIVATE_FAULT_NATIVE"
```

For the future final cumulative gate, use that command with a new campaign directory and `--suite all`. Today `--suite all` includes **only four existing suites** and cannot accept `--closer`; do not confuse it with this future gate. Future `--resume` retains the same campaign directory/arguments and reuses only intact passed summaries with unchanged execution fingerprints. Changed binaries/native/addon/fixtures/contracts/behavioral inputs require affected reruns; missing/corrupt/failed/interrupted evidence is never promoted. Each rerun receives fresh artifacts; keep failed attempts. Nested composed records belong to their runner, not extra campaign steps or duplicate scenario counts.

## 7. Result-only review, privacy, cleanup and export

For **every public close result**, first review only the result: intended/resolved target or refusal; new/already-satisfied/refused/applied-unverified/unknown classification; expected-basis use; reached versus completed close/revalidation/verification stages; known selection and history effects; applicable/unavailable/earlier D/R/B and dirty facts; retained/unloaded/unobservable R; preservation/continuation status; interval/invalidation; and safe next action. Then compare with independent disk/editor/history witnesses, owned-window evidence and elapsed time. Do not derive fresh state from an earlier preparation sample or infer no effect from process failure.

Use distinct synthetic sentinels for target, each private protected source, another project and denied/ambiguous candidates. Closing output uses target source **digests/lengths**, not full bodies or reusable observation-v1 snapshots. No protected paths, sources, hashes, receipts, pointer values or raw helper diagnostics may appear in public results, stderr, incidental logs, routing metadata or campaign manifests. Registry authentication material stays source-free/private. Validate actual authorized/denied/interrupted paths rather than relying on a redaction policy assertion.

Inspect actual owned validator staging and process-group cleanup after success, refusal, timeout, cancellation and child failure. Remove private staged sources/diagnostics, reap owned children, disconnect request-local signal witnesses and release owned references/slot under the entered-call lifetime rules. Do not cancel Godot's ordinary scheduled validation or alter source/selection/history as cleanup. Capture only owned windows and retain only private synthetic evidence needed for provenance; remove temporary projects, controls, registries/credentials and throwaway helpers. Preserve failed/incomplete summaries separately; never publish raw source-bearing logs as incidental evidence.

Exercise **enabled-addon, disabled-addon and export-hook-only** production exports. Inspect actual packs for close/shared addon scripts, bridge, normal/fault native libraries/registration/manifests, fixtures, staging and credentials; then run each exported game and establish no active tooling listener, native tooling registration or gameplay dependency. A preset or hook assertion alone is insufficient. Preserve the existing export boundary and optional automation trust model; no close-specific CI environment/approval service is required.

Record actual coverage, exact provenance, measured deadlines and substantive limitations. Before implementation completion, perform the existing proportional implementation-shape/visibility/state/evidence-error review and applicable constitutional assessment. Support and feature completion require positive useful behavior, every required refusal/interruption/preservation/composed/privacy/export case and the final affected cumulative evidence—not a build, planning document, native acknowledgment or PR state.

## 8. Planning evidence actually executed

[Research §3–§4](research.md#3-exact-candidate-and-observed-provenance) records **seven native `ScriptEditor.close_file` calls: six positive stock controls and one deliberately unsafe unrelated-source negative**, across **four owned visible editors**, all reaped with exit **0**. Independent source/identity/dirty/selection witnesses, actual native validation events, unrelated Undo/Redo and three inspected owned-window captures support the selected guard design. Research retains exact local evidence hashes and cleanup provenance; those local artifacts are not contributor prerequisites and are not duplicated here.

The user accepted **guarded native close-triggered revalidation**, not unguarded source application. The selected complete protection set, per-purpose private validation receipts, event-fenced continuation and separate fresh verification are proposed implementation obligations. The seven probes did **not** exercise a guarded public closer, bridge v5/native revision 3, close runner/campaign, full timing/race/sequential/composed/export acceptance or a new platform-support guarantee. No close product acceptance was executed by this documentation-only planning work.
