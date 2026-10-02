# Quickstart: Verify Safe Clean-Document GDScript Closing

**Status:** T001 and T002 are complete. The public `close-gdscript` caller, six first-use caller/safety groups and inherited `native-boundary` group have [passed task acceptance](#10-t002-public-caller-acceptance-2026-10-02). Full `sequential`/`composed`/`all` closing coverage remains T003 work; campaign `--suite close` and `--suite all` explicitly refuse rather than certify an incomplete suite. Feature 005 and roadmap Phase 1 remain incomplete.

Read [spec.md](spec.md), [plan.md](plan.md), [data-model.md](data-model.md) and [research.md](research.md). The [constitution](../../.specify/memory/constitution.md), [architecture](../../ARCHITECTURE.md) and [working agreement](../../AGENTS.md) govern acceptance. An editor/native acknowledgment, matching source pair, successful process exit or fixed sleep is not verified closure.

## 1. Exact candidate and owned prerequisites

- Official Godot **`4.7.2.stable.official.ed1daf0bf`**, full commit **`ed1daf0bf001b61586d9930840f2f1394092c079`**, executable SHA-256 **`c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf`**; tested host **macOS 26.6.2, build 25G83, arm64**. Guarantees are limited to the recorded task acceptance in §§9–10; no broader version/platform support is claimed.
- Rust **1.98.1**, edition 2021, tracked Cargo.lock and existing rustfmt/Clippy components; existing C++17 Apple compiler/SDK; Python **3.10+**. Use matching official export templates; the existing candidate template SHA-256 is `88df5e2e6fee99088699be66e6d42e4da4fb0c5619d054297d755a49558a4792`. No engine patch, new dependency or package installation is selected.
- An unlocked visible desktop, explicitly owned synthetic projects and editor/runtime processes, and permission to capture only owned windows. Inspect actual captures; headless/getter evidence alone does not prove visible buffer/history preservation.
- Canonical absolute executable/input paths; fresh empty mode-0700 artifact directories; a draining stdout consumer; no human source or credentials in fixtures. Resolve a symlink Godot executable before supplying the existing stock validator.
- Use the [native build/operator guide](../../godot-addon/native/README.md) and separate production versus fixture-fault bundles. Install matched **native family revision 3**, retaining `editor_integration.gdextension` and `libeditor_integration.macos.arm64.dylib`; revision 2 is not a close-compatible bundle.
- Install all Rust/addon peers together for **private bridge v5**. Its ninth authenticated `close_gdscript` bit requires the complete installed close transport and matched native family. Restart owned editors for fresh descriptors; never reuse v4 descriptors or earlier session-bound bases. Public observation/edit/open/discovery v1 and their existing limits/deadlines remain unchanged. No compatibility fallback is retained.

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

These interfaces build the current revision-3 integration. The normal library must not contain fixture-fault controls; the separate fixture artifact must never replace it in production or enter an export.

From `mcp-server/`, existing checks and build commands are:

```sh
cargo +1.98.1 fmt --all -- --check
cargo +1.98.1 clippy --all-targets --locked -- -D warnings
cargo +1.98.1 test --locked
cargo +1.98.1 doc --no-deps --locked
cargo +1.98.1 build --locked --lib --bin observe-gdscript \
  --bin edit-gdscript --bin open-gdscript --bin discover-gdscripts \
  --bin close-gdscript --example stock_validation_fixture
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

## 2. Build and caller smoke

From `mcp-server/`, build the integrated consumers:

```sh
cargo +1.98.1 build --locked --lib --bin close-gdscript \
  --bin observe-gdscript --bin edit-gdscript --bin open-gdscript \
  --bin discover-gdscripts --example stock_validation_fixture
```

Use production and separate fault native bundles from the existing §1 commands. Unchanged bundles may be reused only after verifying their exact engine, ABI, source, manifest and library provenance. Observe actual matched loading and capability negotiation, not just successful compilation.

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

`CLOSER` is the canonical absolute built close binary. Its selector flags reuse existing spelling; exactly one stdin JSON value plus EOF contains only `schema_version`, `request_id` and `basis`. The basis is the complete prior observation-v1 result, not a new token or an edited source override. A fresh checked request ID must differ from the observation ID.

```sh
CLOSER="$REPO/mcp-server/target/debug/close-gdscript"
python3 -c 'import json,sys; print(json.dumps({"schema_version":1,"request_id":"close-smoke-1","basis":json.load(open(sys.argv[1]))}))' \
  "$SMOKE_ARTIFACTS/basis.json" | \
  "$CLOSER" --registry "$REGISTRY" --project "$PROJECT" \
    --session "$SESSION" --script "$SCRIPT"
```

Expected positive result: schema 1, `operation: close_gdscript`, `verified_newly_closed`, attributed removal of the original document, fresh confirmed target absence, unchanged admitted D/file identity, independently unchanged retained R or observed unloaded R, completed/not-applicable required native continuation and preserved protected documents. B and document-buffer dirty state become **not applicable**, not empty/clean. A selected target may cause reported ordinary native fallback selection; a non-selected target must leave the current selection intact. No target parse-validity or write-permission prerequisite is added.

Then separately use the existing observer to witness closed state. No-effect recognition smoke:

```sh
printf '%s\n' '{"schema_version":1,"request_id":"close-smoke-2","basis":null}' | \
  "$CLOSER" --registry "$REGISTRY" --project "$PROJECT" \
    --session "$SESSION" --script "$SCRIPT"
```

Expected: `already_closed_unchanged`, no load/open/close, context validation or selection/history/source effect. A null basis must refuse if the target has become open. A valid supplied old open basis can recognize the same valid file/session as closed only with `expected.use: not_applied_to_closed_state`; it cannot transfer to a replacement file/session/buffer. Closed recognition is not proof of project-wide cleanliness or of an earlier interrupted attempt's success. Reopen only as a separate intentional existing opener call, then obtain fresh observation for any edit/close.

Exits: **0** for either verified new closure or already-closed recognition; **3** for proven refused/not applied; **4** for `applied_unverified` or `effects_unknown`; **2** for malformed CLI/input; **1** for unexpected host delivery failure. Inspect structured evidence, not exit code alone. Possibly applied outcomes require fresh observation of the explicit original target, never automatic retry, rollback or compensating reopen.

## 3. Owned native boundary and caller groups

The implemented `godot-addon/tests/run_script_close.py` reuses existing harness ownership, authenticated selection, independent source/history/selection witnesses, owned-window capture, private controls, cleanup and export checks. The inherited `native-boundary` controls invoke the real shared-slot owner and actual Rust source validation; no primitive receipt is a public success verdict.

Native-only command (new empty private artifacts for each invocation; no public closer required):

```sh
CLOSE_ARTIFACTS="$(mktemp -d "$HOME/close-native-boundary.XXXXXX")"
python3 godot-addon/tests/run_script_close.py \
  --godot "$STOCK_GODOT" --observer "$OBSERVER" --editor "$EDIT_CALLER" \
  --stock-validator "$STOCK_VALIDATION_FIXTURE" \
  --native-fault-addon "$PRIVATE_FAULT_NATIVE" \
  --scenario native-boundary --artifacts "$CLOSE_ARTIFACTS"
```

T002 implements `clean-close`, `already-closed`, `preservation`, `routing`, `interruption` and `privacy-export` through the actual public caller. Supply all three public executable paths, in addition to the native command's inputs:

```sh
CLOSE_ARTIFACTS="$(mktemp -d "$HOME/close-clean.XXXXXX")"
python3 godot-addon/tests/run_script_close.py \
  --godot "$STOCK_GODOT" --observer "$OBSERVER" --editor "$EDIT_CALLER" \
  --closer "$CLOSER" --opener "$OPENER" --discoverer "$DISCOVERER" \
  --stock-validator "$STOCK_VALIDATION_FIXTURE" \
  --native-fault-addon "$PRIVATE_FAULT_NATIVE" \
  --scenario clean-close --artifacts "$CLOSE_ARTIFACTS"
```

Run each of the other five public groups with its own fresh private directory.
`clean-close` includes first-use discovery/open/observe/edit/fresh-observe/close/
reopen and inherited A–E/history/durability regressions. T003 still owns the
new repeated-close sequence and full composed matrix. `sequential`, `composed`
and `all` explicitly refuse today; they never skip missing cases.

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

**Future cumulative campaign:** T002 registers the actual close inputs and fingerprints, including absolute `--closer`, but explicitly rejects campaign `--suite close` and `--suite all` until T003 implements all nine close groups. The following command is the future complete-suite interface, not current task acceptance:

```sh
CLOSE_CAMPAIGN="$(mktemp -d "$HOME/close-campaign.XXXXXX")"
python3 godot-addon/tests/run_editor_campaign.py \
  --suite close --campaign-dir "$CLOSE_CAMPAIGN" --keep-going \
  --godot "$STOCK_GODOT" --closer "$CLOSER" \
  --observer "$OBSERVER" --opener "$OPENER" --editor "$EDIT_CALLER" \
  --discoverer "$DISCOVERER" --stock-validator "$STOCK_VALIDATION_FIXTURE" \
  --native-fault-addon "$PRIVATE_FAULT_NATIVE"
```

For the future final cumulative gate, use that command with a new campaign directory and `--suite all`. Today both full-coverage requests explicitly refuse before execution or checkpoint changes. Run the seven implemented closing groups directly and the four existing complete suites separately. Existing `--resume` retains the same campaign directory/arguments and reuses only intact passed summaries with unchanged execution fingerprints. Changed binaries/native/addon/fixtures/behavioral inputs require affected reruns; missing/corrupt/failed/interrupted evidence is never promoted. Each rerun receives fresh artifacts; keep failed attempts. Nested composed records belong to their runner, not extra campaign steps or duplicate scenario counts.

## 7. Result-only review, privacy, cleanup and export

For **every public close result**, first review only the result: intended/resolved target or refusal; new/already-satisfied/refused/applied-unverified/unknown classification; expected-basis use; reached versus completed close/revalidation/verification stages; known selection and history effects; applicable/unavailable/earlier D/R/B and dirty facts; retained/unloaded/unobservable R; preservation/continuation status; interval/invalidation; and safe next action. Then compare with independent disk/editor/history witnesses, owned-window evidence and elapsed time. Do not derive fresh state from an earlier preparation sample or infer no effect from process failure.

Use distinct synthetic sentinels for target, each private protected source, another project and denied/ambiguous candidates. Closing output uses target source **digests/lengths**, not full bodies or reusable observation-v1 snapshots. No protected paths, sources, hashes, receipts, pointer values or raw helper diagnostics may appear in public results, stderr, incidental logs, routing metadata or campaign manifests. Registry authentication material stays source-free/private. Validate actual authorized/denied/interrupted paths rather than relying on a redaction policy assertion.

Inspect actual owned validator staging and process-group cleanup after success, refusal, timeout, cancellation and child failure. Remove private staged sources/diagnostics, reap owned children, disconnect request-local signal witnesses and release owned references/slot under the entered-call lifetime rules. Do not cancel Godot's ordinary scheduled validation or alter source/selection/history as cleanup. Capture only owned windows and retain only private synthetic evidence needed for provenance; remove temporary projects, controls, registries/credentials and throwaway helpers. Preserve failed/incomplete summaries separately; never publish raw source-bearing logs as incidental evidence.

Exercise **enabled-addon, disabled-addon and export-hook-only** production exports. Inspect actual packs for close/shared addon scripts, bridge, normal/fault native libraries/registration/manifests, fixtures, staging and credentials; then run each exported game and establish no active tooling listener, native tooling registration or gameplay dependency. A preset or hook assertion alone is insufficient. Preserve the existing export boundary and optional automation trust model; no close-specific CI environment/approval service is required.

Record actual coverage, exact provenance, measured deadlines and substantive limitations. Before implementation completion, perform the existing proportional implementation-shape/visibility/state/evidence-error review and applicable constitutional assessment. Support and feature completion require positive useful behavior, every required refusal/interruption/preservation/composed/privacy/export case and the final affected cumulative evidence—not a build, planning document, native acknowledgment or PR state.

## 8. Planning evidence actually executed

[Research §3–§4](research.md#3-exact-candidate-and-observed-provenance) records **seven native `ScriptEditor.close_file` calls: six positive stock controls and one deliberately unsafe unrelated-source negative**, across **four owned visible editors**, all reaped with exit **0**. Independent source/identity/dirty/selection witnesses, actual native validation events, unrelated Undo/Redo and three inspected owned-window captures support the selected guard design. Research retains exact local evidence hashes and cleanup provenance; those local artifacts are not contributor prerequisites and are not duplicated here.

The user accepted **guarded native close-triggered revalidation**, not unguarded source application. The selected complete protection set, per-purpose private validation receipts, event-fenced continuation and separate fresh verification are proposed implementation obligations. The seven probes did **not** exercise a guarded public closer, bridge v5/native revision 3, close runner/campaign, full timing/race/sequential/composed/export acceptance or a new platform-support guarantee. No close product acceptance was executed by this documentation-only planning work.

## 9. T001 native-boundary acceptance (2026-10-02)

**T001 is complete**, independently of GitHub review/merge state. This is an
accepted private native/validation capability and coordinated compatibility
cutover, not a public closer or Feature 005 completion. T002/T003 are unstarted.
The ninth authenticated `close_gdscript` capability remains false.

### Executed scope

- The real owner performs one generated-bound close under exact clean D/R/B,
  identity, revision, complete protection-set and effective-context guards.
  Selected/non-selected/last-tab, stable two-document, empty, safe syntax-invalid
  and read-only targets passed. Retained-original R and genuinely destroyed,
  unloaded original R have separate controls. Already-closed recognition has
  no close or validation child.
- Dirty-current/background R==B preserves exact unsaved text, versions, flags
  and actual earlier Undo/Redo; post-close Undo/Redo repeats those real
  transitions. Dirty/equal-dirty targets and the demonstrated unrelated R!=B
  case refuse before entry. Owned Script-screen images were inspected.
- Exact/one-over document, source and metadata bounds; invalid/missing/misbound
  source-purpose receipts; file/roster/session/configuration races; native visit,
  missing/wrong/premature/renewed completion; callback/disable/cancel/expiry
  lifetime; duplicate wait/advance; shared-slot exclusion and no late close
  passed. Independent verification deliberately fails after native OK and
  completion when actual postconditions change; survivor acquisition never
  renews authority or closes a newer document.
- Source privacy, cleanup and actual enabled-addon, disabled-addon and
  export-hook-only pack inspection/runtime execution passed. Complete existing
  observation/edit/open/discovery suites passed after the shared cutover,
  including their applicable A–E, native history, Save/reopen/reparse/rescan/
  runtime and privacy/export regressions.

Commands were the current native command in §3 and all four unfiltered
`--scenario all` commands in §6, run serially with the same production runtime
inputs. The native total includes one bootstrap record plus 122 boundary
records; nested groups/composed records are not extra suite invocations.

Private evidence root:
`/Users/petertam/.godot-agent-kit-close-T001-mw18ukyk/`.
These retained local artifacts identify this run, not contributor prerequisites.

| Accepted summary, relative to that root | Passed records | SHA-256 |
| --- | ---: | --- |
| [native-final/summary.json](file:///Users/petertam/.godot-agent-kit-close-T001-mw18ukyk/native-final/summary.json) | 123 | `54fd091bf65be13660396edb27e3cad9079dcc2bf118e57b382f888b8fa1130b` |
| [regression-observation/summary.json](file:///Users/petertam/.godot-agent-kit-close-T001-mw18ukyk/regression-observation/summary.json) | 273 | `9a20c065cb8e2804e68f86f721e02d1a263db692498867a5e94be5a781bdb5c3` |
| [regression-edit/summary.json](file:///Users/petertam/.godot-agent-kit-close-T001-mw18ukyk/regression-edit/summary.json) | 408 | `6fae7fa9ec8d46a9b1839f52c015f73233e3cea3d1065a30fb3205f76de76468` |
| [regression-open-complete/summary.json](file:///Users/petertam/.godot-agent-kit-close-T001-mw18ukyk/regression-open-complete/summary.json) | 664 | `1741e30bf407256b05baa94f13526f79fae82c921d276d45a68e13d2fc91692a` |
| [regression-discovery/summary.json](file:///Users/petertam/.godot-agent-kit-close-T001-mw18ukyk/regression-discovery/summary.json) | 429 | `dc6d6f37ced8e4f1e344c31bbbe8243f75e180d7f6ac04ca632b6933bcce528b` |

Opening includes **187 public calls**, maximum **9.510375 s**; discovery includes
**158 public calls**, maximum **4.507051 s**. The existing caller deadline gates
remain unchanged. Native controls do not claim the future public close deadline
or terminal-result reducer.

### Exact provenance and implementation checks

Tested environment: the exact official Godot **4.7.2** commit and executable
hash in §1, **macOS 26.6.2 arm64**, Rust **1.98.1**, Apple clang
**21.0.0 (clang-2100.3.34.2)**, SDK **27.0**. No broader version/platform claim.
The summary records contain caller/validator, addon/fixture, Cargo.lock,
generated API/ABI and matching export-template fingerprints.

| Native artifact | Build ID | Library SHA-256 |
| --- | --- | --- |
| Production revision 3 | `89d9027522bcf175d55fe4e8cd63fab5e933b7e4b08f031c736696c4f329c122` | `80bfedb1d639f10c66a2530e26003f4d5e84fc01b21299c0136c64155ced9f1a` |
| Separate fixture-fault revision 3 | `cce41410ffbb538a2ca281c015b3ab6ae19f545a11791fceb28ad3627fd99c44` | `67d49b46151eab76f9fee12bf412095f8eca2b4a06be87a7b806a224f884c971` |

The unchanged native acceptance implementation
`godot-addon/tests/close_native_acceptance.py` has SHA-256
`343664d81842e4715abf93e73936dda1f784d84486933bbc0e2603c23e8b5af6`.
After the full run, the runner gained an explicit field recording that module's
hash; actual constructor execution verified it. That reporting-only addition,
documentation/status updates and deletion of a wording-only unit test do not
change the exercised production or acceptance behavior.

Passed: §1 Cargo formatting, Clippy, **317 default tests including doctests**,
rustdoc and locked real-consumer builds; **2 example pre-allocation boundary
tests**; normal and separate fault native builds; **13 native-build tests** and
**22 workflow tests**. The selected task's [shape and constitutional
review](plan.md#implementation-shape-review) covers module ownership, private
visibility, one-owner lifecycle, evidence failure paths and justified decoder
complexity. No dependency, new CI/provider/approval gate or public close API
was added. No extension hooks were configured.

Earlier diagnostic/failing runs and the outer-tool-timeout `regression-open`
directory are retained but excluded from acceptance. No owned editor remained
after that interruption; its attributed private staging directory was removed,
and the independent complete opening run above replaced it. Temporary projects,
registries and throwaway diagnostic helpers were cleaned up. Close's permitted
target-local history disposal is not an Undo-close claim; public close
composition, result-only review and cumulative feature acceptance remain
T002/T003 obligations, not waived gates.

## 10. T002 public-caller acceptance (2026-10-02)

**T002 is complete.** All seven implemented close groups passed through the real
caller/native boundaries: **639 records, 206 public close results**, maximum
externally measured close time **9.509884 s** with stdout drained. Every public
result received result-only interpretation and comparison with independent
source, identity, dirty/version, history and owned-window evidence.

### Executed scope

- Selected, non-selected and last-document closure; unchanged independently
  acquired D; unchanged retained original R or genuine unloading; actual B
  absence and required native completion passed. Empty, read-only and safe
  syntax-invalid sources remain positive cases without repair or a target
  parse-success gate.
- Fresh no-effect recognition passed with null and valid old open bases.
  Source-local limitations stay explicit; wrong-file/session and ineligible
  bases do not become null or authorize a newer buffer.
- Dirty/disk-equal dirty targets, stale identities, unsupported association,
  unsafe protection contexts and all exact/one-over bounds were exercised.
  Unrelated clean/dirty documents, selection and actual Undo/Redo were preserved.
  Target-local history disposal is not an Undo-close claim.
- Real routing/namespace races and cross-operation ownership passed. Timeout,
  SIGINT/SIGTERM, stalled input/editor/native calls, malformed/lost replies,
  worker/helper/engine loss and newer reopened work retained truthful known or
  uncertain effects. Owned child groups/private staging were independently
  observed gone; terminal delivery did not permit a late product close.
- First-use discovery → open → observe → edit → fresh observe → close → reopen,
  dirty refusal, applicable A–E/history, Save/reparse/rescan/runtime durability,
  source/credential privacy and enabled/disabled/hook-only exports passed.
  The complete affected observation/edit/open/discovery suites also passed.

Commands were all seven §3 groups and the four unfiltered §6 `--scenario all`
commands, run serially. Preservation and routing were refreshed after the final
strict public-input decoder change; every accepted public close group used
caller SHA-256
`b890b30dc74c50ad716528feef2bf03485a43528bd436a34b3ec9c45112141e3`.
Nested/composed records are not additional suite invocations.

Private evidence root:
`/Users/petertam/.godot-agent-kit-close-T002-3ja4t_90/`.
These retained artifacts identify this run, not contributor prerequisites.

| Accepted summary, relative to that root | Passed records | SHA-256 |
| --- | ---: | --- |
| [native-boundary-acceptance/summary.json](file:///Users/petertam/.godot-agent-kit-close-T002-3ja4t_90/native-boundary-acceptance/summary.json) | 123 | `4be557ccd765a58008109ef1d0bc10b1ef793533552dc6708d41ab7fc91ff078` |
| [clean-close-accepted-head/summary.json](file:///Users/petertam/.godot-agent-kit-close-T002-3ja4t_90/clean-close-accepted-head/summary.json) | 134 | `dd45eb6dc125e56f199190cd9ad514016f26a0fec587f4149a8ae4bb0289d992` |
| [already-closed-accepted-head/summary.json](file:///Users/petertam/.godot-agent-kit-close-T002-3ja4t_90/already-closed-accepted-head/summary.json) | 14 | `5fb93b6424c54b182ec6ac7b20ac49e18cf5845db3d168b5d4fca69aeedb9aee` |
| [preservation-final-head/summary.json](file:///Users/petertam/.godot-agent-kit-close-T002-3ja4t_90/preservation-final-head/summary.json) | 152 | `b03b035a2eb1253d355012d97bb4b78465e35754626ae7890bccbe6d0f4f3993` |
| [routing-final-head/summary.json](file:///Users/petertam/.godot-agent-kit-close-T002-3ja4t_90/routing-final-head/summary.json) | 56 | `3e1128b515eaed390251165224f3aa02dbff9f39967a5a3be5798f49923e2526` |
| [interruption-acceptance/summary.json](file:///Users/petertam/.godot-agent-kit-close-T002-3ja4t_90/interruption-acceptance/summary.json) | 149 | `bbdfcc6757ab0a9656fe83e0fa2f3a23f7e7e21c9d51376e5dd676e0354a8e0a` |
| [privacy-export/summary.json](file:///Users/petertam/.godot-agent-kit-close-T002-3ja4t_90/privacy-export/summary.json) | 11 | `09fa43ad572222c8536e898147e7edfc7668457acaf9733b149dcd7c3c90f892` |
| [regression-observation-ready/summary.json](file:///Users/petertam/.godot-agent-kit-close-T002-3ja4t_90/regression-observation-ready/summary.json) | 273 | `93d1e2c0dd1b40a004fb88858643220586236f50f558c5ce8b81f4d71bd2805c` |
| [regression-edit-final-head/summary.json](file:///Users/petertam/.godot-agent-kit-close-T002-3ja4t_90/regression-edit-final-head/summary.json) | 408 | `3ad2278d3ed96f2fd6cfc8111258c03acf8af89bf95ffaf1bbcace4da4a406f5` |
| [regression-open/summary.json](file:///Users/petertam/.godot-agent-kit-close-T002-3ja4t_90/regression-open/summary.json) | 664 | `c681c7ba4f28b6597778de42114e26d382cd6c75bd4181e30398d3a00dc291d7` |
| [regression-discovery/summary.json](file:///Users/petertam/.godot-agent-kit-close-T002-3ja4t_90/regression-discovery/summary.json) | 429 | `f7344009d6a9042cd22d193de035a66c5eeda26d94c1fe7dec4192addd20b3f8` |

### Exact provenance and implementation checks

Tested environment: the exact official Godot **4.7.2** commit/executable in §1,
**macOS 26.6.2 arm64**, Rust **1.98.1**, Apple clang
**21.0.0 (clang-2100.3.34.2)** and SDK **27.0**. Summaries retain addon/fixture,
caller/validator, Cargo.lock, generated API/ABI and export-template fingerprints.

| Native artifact | Build ID | Library SHA-256 |
| --- | --- | --- |
| Production revision 3 | `91c2bf88384ab35a029cecbaea2142a966871ea1a4c598ddd60fc7e5834d9b90` | `9a0865642270ff3503c21e9e1533e8f87aa2e2422000652f73fb1610f4d88fac` |
| Separate fixture-fault revision 3 | `515fb7cdd3e3d6b585cbad56f5ebde8220badb27fd29605f81eca2d694412e55` | `703f581245260e192aee0d97e5858690c514460228c79aea20922b22e3baa4f5` |

Passed: Rust formatting, Clippy, **367 default tests including doctests**,
**2 example boundary tests**, rustdoc and locked library/caller builds; normal
and separate fault native builds; **38 Python native/harness tests**,
**22 workflow tests** and Actionlint. Real campaign `--suite close`/`all`
invocations refused incomplete coverage before creating evidence directories.
No extension hooks were configured.

The [implementation-shape/constitutional review](plan.md#t002-implementation-shape-and-constitutional-review--2026-10-02)
records checked intent, private channel/codec ownership, one-owner lifecycle,
typed evidence failures and proportionate reuse. It also records regression
fixes for actual observation readiness and completed-editor fixture lifetimes.
The observation fixture change passed fresh visible controls and its complete
suite; editor teardown changes passed focused conflict/routing checks and the
complete 408-record suite without changing assertions or product deadlines.

Earlier failed, diagnostic and pre-final decoder runs are retained but excluded
from the accepted summaries above. Temporary projects, registries and diagnostic
helpers were cleaned up; generated binaries/evidence are not committed.
T003's repeated/composed closing matrix, full close campaign and feature
cumulative gate remain pending and unstarted. Independent Save/history controls
and roadmap Phase 1 exit assessment are outside this task.
