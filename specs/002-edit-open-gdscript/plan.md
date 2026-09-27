# Implementation Plan: Safely Edit Open GDScript

**Branch**: `spec/002-edit-open-gdscript` | **Feature identifier**: `002-edit-open-gdscript` | **Date**: 2026-09-27 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/002-edit-open-gdscript/spec.md`

**Status**: **Planning/design complete; implementation in progress.** Design artifacts were merged in [PR #24](https://github.com/Peter-Tam/godot-agent-kit/pull/24). T001's reusable core is complete with [acceptance evidence](quickstart.md#9-t001-core-acceptance-2026-09-27); T002–T005 remain pending. No native/CLI mutation support or A–E acceptance is claimed.

## Summary

Deliver one native, revision-guarded whole-source edit to an existing open standalone GDScript in one explicitly selected local editor. A caller supplies a checked prior-observation basis and intended source; Rust/core freshly checks independent D/R/B, dirty state and identities, authorizes application, and independently verifies resulting source, clean/saved state and source-attributed parse evidence before success. Refused/unchanged requests add no source/history changes; partial/uncertain changes never imply rollback or retry safety.

Selected boundary:

```text
edit-gdscript / typed caller
  -> existing Rust automation core and supervised worker
  -> existing authenticated private editor bridge (planned v2)
  -> Godot integration and standard C-ABI GDExtension
  -> native CodeEdit history + bound-descriptor persistence
     + narrow engine document-finalization / GDScript-validation APIs
  -> independent Rust D and fresh editor R/B/dirty/saved-state verification
```

The two previously unresolved capabilities now have concrete [native contracts](contracts/native-integration.md): A guards successful persistence and updates exact target document/Resource/saved-version bookkeeping; B returns exact-source native parser/analyzer results under a confined, non-evaluating dependency/effect policy. Small inspection/target-guard companions make those guarantees observable and prevent automatic editor callbacks from bypassing them. Standard GDExtension alone supplies object access, native text/history calls and descriptor I/O, but not the two unbound engine capabilities.

The [research decision](research.md#11-concrete-native-design-and-resumed-planning) preserves every completed positive/negative observation. The policy-interrupted probe is not a technical failure or a successful experiment. No runtime probe or engine patch was performed in this continuation. The [Phase 1 concurrency boundary](spec.md#phase-1-concurrency-boundary) now explicitly distinguishes required local kit-entry serialization from excluded broader coordination; the same existing integration slot supplies it.

## Technical Context

**Language/Version**: Existing Rust **1.98.1**, edition 2021; GDScript for the thin editor integration; **C++17** for the small standard-ABI extension. Python **3.10+** remains the owned-fixture driver environment.

**Primary Dependencies**: Reuse existing locked `serde =1.0.229`, `serde_json =1.0.151`, `cap-std =4.0.3`, `ring =0.17.14` and existing system toolchain. No new Rust crate, godot-cpp layer, parser, process service or transport. Generate/use the pinned engine's public GDExtension C interface and opaque object/Variant dispatch, preserving upstream SDK notices. Native OS I/O uses the demonstrated macOS descriptor APIs.

**Engine candidate**: Godot **4.7.2**, base commit `ed1daf0bf001b61586d9930840f2f1394092c079`, plus the **planned editor/GDScript API family revision 1** and matched extension. This is a future patched development build, **not** the already-tested official stock binary. Record base commit, exact patch digest, reported engine version/hash, API/native build ID, compiler/SDK and actual binary hashes in implementation evidence. The patch digest/binary do not exist yet and are not fabricated. Support is limited to exact builds that later pass CI and real-editor acceptance; no wildcard patch/platform support or unsafe override.

**Target Platform**: Initial mutation candidate is maintainer-operated **macOS 26.6.2 arm64** with a GUI editor, matching the prior bounded research environment. No Linux/Windows or distribution expansion. Stock Godot retains observation-only capability under the migrated bridge; edit is unavailable without the matched native API. The patched editor must independently pass observation regression before its observation compatibility is claimed.

**Storage**: Existing owner-private source-free session registry; one existing project script; request-local memory/descriptors/evidence. No new persistent mutation token store, source cache, replay cache, backup/journal or recovery service. A descriptor-bound write is not crash-atomic storage.

**Testing**: Pure core boundary/transition tests; real supervised-worker/private-bridge tests; native primitive regressions; actual visible-Godot A–E plus Save/reopen/reparse/rescan/runtime and privacy/export/observation regression. Reuse existing owned-editor harness facilities, independent witnesses and deadlines. The [quickstart](quickstart.md) is the future execution/evidence contract, not a passing-results record.

**Project Type**: Existing local Rust library/CLI plus editor addon/native integration. No MCP exposure.

**Performance Goals**: Controlled edit result ≤10 seconds (9.5-second acquisition/operation cutoff + 0.5 delivery reserve), including an unresponsive editor. Observation remains ≤5 seconds with its existing 4.5-second supervision. No sustained-throughput or arbitrary-project-size claim.

**Constraints**: One open standalone script; explicit target/session and expected basis; exact independently agreeing D/R/B and attributable clean state; native single-step history; target-bound non-redirectable persistence; no arbitrary evaluation/execution/outside-project access; truthful partial outcomes; no automatic retry/rollback. Success is an observed interval, not a frozen editor or arbitrary same-inode serialization.

**Scale/Scope**: Root/replacement and each source ≤512 KiB UTF-8; bounded 32 dependency sources / 4 MiB dependency bytes; one active editor collection/edit slot; inherited 32-peer/descriptor bounds. Full-source LF representation, explicit unsupported NUL/CR/BOM refusal, and native Save-profile eligibility preserve exact source rather than normalize it. Caller/bridge/diagnostic bounds are in the contracts.

## Selected Design

### 1. Core intent, revision and compatibility

Add a focused script-edit module to the existing Rust library, using existing checked selectors, identity/collection/source evidence and classification patterns. The core consumes protocol-independent evidence; no JSON, TCP, Godot object or native ABI dependency enters its policy. Do not build a general Phase 2 transaction/revision framework.

One new `edit-gdscript` caller accepts the selectors and bounded stdin JSON in [edit-api.md](contracts/edit-api.md). Whole-source intent avoids patch-coordinate ambiguity; source does not enter argv or an arbitrary file-reading option. Expected basis extracts project/session/document/file identity, agreed source digest/length, exact current CodeEdit version and attributable clean state from prior observation. Observation v1 has no saved-version field; read that separately during fresh preparation and guard the prepared value through A. Fresh checks, not that snapshot, authorize the write.

Keep observation's public v1 meaning, timing and per-surface availability unchanged. Private bridge **v2** is a coordinated clean cutover with authenticated edit/native capability fields, changed HMAC domain and strict tuples. Migrate all Rust/addon/worker/fixture peers and docs; no dual-protocol shim or v1 mutation fallback. Restart/re-enable the addon and observe again after upgrade.

### 2. Native application and Primitive A

Use one native CodeEdit complex operation on the exact bound buffer; preserve previous undo history and unrelated tabs, with normal native redo-branch invalidation permitted. Do not use `set_text`/clear-history as an edit shortcut or create an alternate EditorUndoRedoManager history for TextEdit's existing text history. Never choose the destination by focus.

Immediately recheck current B/version and original R, then use the explicit supported **`Script.set_source_code` method**, not property `_set`, `apply_code`, export propagation or reload. Native source synchronization is not a compiled-class/Inspector/live-game update claim. Pending dragged exports or an unsafe Save representation refuse before application; do not alter editor preferences or scene properties.

The C++ integration pins the selected project and existing file, independently confirms its identity against the Rust basis, and writes/truncates/flushes only that retained object after fresh D/R/B/identity guards. Short-write/error/namespace-loss facts remain explicit. Replacement/parent redirection never changes the write destination. Arbitrary same-inode exclusion remains unclaimed; known changed source/identity still stops obsolete work.

A consumes the private same-attempt persistence receipt only after fresh receipt/file/target/version/source guards. The narrow engine hook updates descriptor-derived Resource/document mtime, Resource edited state for that exact source, and CodeEdit saved version last; no signals/callouts in the guarded span. Refresh only target saved display without broader save notifications. A's independent read-only companion exposes actual save metadata for later observation. Failed/unknown persistence cannot mark saved; changed/new human text is never tagged or restored over.

Do not emit `resource_saved`, call Save All/current-tab save, reload running scripts, clear unrelated docs/history or claim normal scene-save effects. The exact inputs, step flags, rejection/partial semantics and source references are in [Primitive A](contracts/native-integration.md#3-primitive-a-guarded-target-document-finalization).

### 3. Primitive B and automatic editor effects

Expose existing `GDScriptLanguage::validate` → parser → analyzer on immutable exact source/path in the selected editor main-thread/project context. Bind actual input hash, target/session/request, invocation interval and root/dependency diagnostics. Return completed valid, completed invalid, or unavailable; no reload/log/silence proxy.

Permit normal parser/dependency-cache activity and confined source-backed GDScript analysis. A small call-scoped read/effect context gives the existing parser/cache/analyzer actual confined bytes/current hashes, including cache-hit attribution and live project/class/autoload mappings. It refuses before non-GDScript loader hooks, full loading/instantiation or dynamic project-object property access. Do not add a second parser, dependency-world snapshot/lock, cache-purity mandate or general compiler/evaluator service.

The engine's target-local guard covers ordinary validation/function-discovery/application work caused by the agent edit, including queued callbacks, so the effect boundary is not bypassed by native background validation after return. Restrict or consume only that work; preserve later independent human and unrelated editor behavior. No duplicate R/export application, deferred reload or saved tagging may escape. Diagnostic source belongs only in the requested bounded result, not incidental native/Rust logs.

Preflight **changed** intent to avoid preventable invalid-source mutation; after A, read actual resulting source and invoke B afresh. The post-change result, not preflight, contributes to success. An invalid original script remains repairable if all original safety/representation checks hold and the resulting source validates. Unchanged intent runs one non-mutating validation/verification path. Full contract and effects: [Primitive B](contracts/native-integration.md#4-primitive-b-exact-source-native-gdscript-validation).

### 4. Stage ordering, deadlines and independent verification

Use the [ordered state machine](data-model.md#3-ordered-attempt-state-machine):

```text
accept/select
  -> fresh safety + proposed-source validation (changed intent)
  -> supervisor records may_apply, then authorizes its existing worker
  -> editor's fresh boundary guards
  -> native B edit / guarded R source synchronization
  -> target-bound persistence
  -> guarded finalization A
  -> actual-source validation B
  -> independent D/R/B + dirty/saved-state + context rechecks
  -> one Rust-classified terminal outcome
```

The first native call that can change source/history is the local potentially-applied boundary. The supervisor becomes conservative earlier, **before** releasing its worker's one-shot authorization, so a lost worker/acknowledgment cannot become a false not-applied result. Before that handoff, preparation cannot apply; afterward only an attributable irrevocable pre-boundary discard can prove refusal. A completed B/R/D change remains known application despite later loss. Reuse the existing supervised worker rather than create a helper service.

Reuse the existing one-active-collection/edit slot for the selected editor/session: claim it during preparation, require its ownership at native mutation entry, and retain it through verification/terminal cleanup. Another edit receives a source-free terminal busy refusal before the potentially-applied boundary, with no source/history/finalization change and no queued or replayable work. Releasing the slot cannot auto-apply rejected intent. A later request needs a freshly observed valid basis; old-basis text compatibility cannot override a changed revision. Human typing remains unblocked. Caller termination does not release this slot while entered native work can still mutate; its uncertainty/deadline semantics remain unchanged. T004 owns the real same-session overlap case under US2.4.

An editor-local remaining-time expiry stops new stages when control returns. Native parser/I/O work can block; the caller's supervised deadline still returns a truthful bounded outcome. Timeout/abort cannot revoke an entered stage or imply rollback; late replies cannot upgrade the result. No retry, reconnect/resume or repair write.

The finalizer/writer response is not independent evidence. Rust reads/rechecks D through its own project capability; the existing collector freshly reads actual R and B and document-attributed dirty state, plus the separate native saved-state getter. Recheck source/identity/version and validation dependency/context/save-profile witnesses. Missing, changed, known-stale or divergent evidence prevents success even if an earlier/later sample happens to agree. Return explicit changed/unchanged/refused/applied-unverified/application-unknown and actionable reasons.

### 5. Implementation and evidence boundaries

Keep the engine adaptation in a small reviewable patch against the pinned base, owned alongside addon native integration. It adds supported API exposure and the necessary local effect/inspection guards; it is not a custom module or engine distribution project. Build it using the engine's existing build system; record actual build tools/dependencies/provenance and preserve applicable licenses. A local patched development editor is necessary to implement/test the selected APIs; publishing/bundling a custom editor, updater or release channel is not part of this feature.

The native library must fail closed on unavailable/mismatched APIs, unload/disable cleanly, retain no gameplay registration outside the editor, and remain excluded from production exports even when the addon is disabled. Retain lifecycle ownership and private session cleanup. No public execution/force/outside-project permission surface is introduced.

Implementation must prove the selected source-backed positive path and all mandatory cases, not declare every edit unsupported. Exact hook code, build artifacts, controlled fault injection and regression results are implementation work. If they reveal a genuine unsupported guarantee, fix the integration or report that concrete blocker; never substitute a weaker success definition.

## Constitution Check

**Initial research gate:** Passed for bounded design research; historical runtime routes remained non-qualifying where recorded. **Post-design gate:** **PASS for planning**, with no known constitutional MUST left without a defined design/verification obligation. This is not an implementation/acceptance pass, plan approval or feature completion.

| Governing obligation | Design and future evidence |
|---|---|
| I / IV — independent D/R/B and observed postconditions | Separate actual D and editor R/B/dirty/saved reads; attributed post-change B; immutable failure/partial evidence. A/receipt cannot certify success. A–E and durability remain mandatory. |
| II — preserve human work and stale intent | Target-bound expected basis, independently clean/agreeing start, fresh native guards, version/identity/save-profile checks, no force/repair; reject or retain partial facts on new work. No new arbitrary-writer serialization promise. |
| III — native editor semantics | Actual CodeEdit complex history and explicit native Script setter; narrow engine finalizer fills demonstrated public-API gaps. Alternative descriptor persistence retains the same required coherence/history guarantees, to be proven rather than inferred. |
| V / X — confinement and truthful outcomes | Existing authentication/private metadata; native project-bound descriptor/dependency reads; no evaluation path; separate stage/application/parse/dependency/permission/timeout failures. Controlled unknown after authorization/loss. |
| VI / XII — layered verification and explicit support | Pure transition/boundary tests plus real visible-editor A–E, history, durability, timing, privacy/export and complete observation regression on exact recorded builds. No stock/other-version mutation support extrapolation. |
| VII / IX — protocol-independent, small composable surface | One typed/core edit and one CLI, existing private bridge, no MCP; core owns policy, native owns engine-local primitives. No duplicate transaction model. |
| VIII — tooling/gameplay isolation | Editor-only native integration; real exports and runtime checked with enabled/disabled addon and native artifacts, not reliance on `addons/` or `@tool`. |
| XI — independent implementation and dependency review | Public engine source/ABI, own narrow integration, existing locked Rust dependencies. Preserve SDK notices, review exact new native build inputs/advisories/licenses before their implementation delivery. No unreviewed copied framework. |
| XIII — proportional complexity | Each selected mechanism has a present gap, simpler rejected alternative and ongoing cost below and in research §11.5. No daemon/lock/journal/approval/CI machinery for unpromised guarantees. |
| Architecture/compatibility/workflow | Durable component ownership, explicit private v2 migration/public edit v1, observation v1 preserved. No tasks/implementation/merge in this planning continuation; later one approved task/PR at a time. |

All A–E are applicable, including C because native Undo/Redo is claimed. At least one permitted runtime fixture must prove changed behavior. No missing runtime/buffer/parse observation is an inapplicability justification. Feature and task completion depend on acceptance evidence, separately from GitHub review/merge metadata. Feature completion would not automatically complete Roadmap Phase 1.

## Project Structure

### Documentation (this feature)

```text
specs/002-edit-open-gdscript/
+-- spec.md                         existing behavioral specification; unchanged
+-- checklists/requirements.md      existing specification-quality review
+-- research.md                     retained evidence + current design decisions
+-- plan.md                         this implementation plan
+-- data-model.md                   identity, evidence, stages and outcomes
+-- contracts/
|   +-- edit-api.md                 typed/caller input, output, compatibility
|   +-- bridge-protocol.md          private v2 and supervised authorization
|   +-- native-integration.md       finalizer/validator/inspection/guard contracts
+-- quickstart.md                   future execution and evidence guide
+-- tasks.md                        subsequent task derivation and granularity review
```

The subsequent [task list](tasks.md) records five PR-sized increments and its completed granularity review. It was generated by `/speckit.tasks`, not the earlier planning command. Read-only `/speckit.analyze` completed after the local-overlap clarification with no genuine consistency findings. T001 is complete; the remaining implementation increments retain their original scope and dependencies.

### Source Code (repository root)

Existing files are retained; `planned` entries below are implementation ownership, not files created by this planning PR:

```text
mcp-server/
+-- Cargo.toml / Cargo.lock         existing single package, locked dependencies
+-- src/
|   +-- lib.rs                     exports reusable core
|   +-- observation.rs             existing read-only semantics
|   +-- script_edit.rs             planned focused edit intent/evidence/reducer
|   +-- runner.rs                  extend existing supervised execution boundary
|   +-- project_fs.rs              existing independent D/confinement helpers
|   +-- bridge.rs / bridge/        existing session/authentication/framing boundary
|   +-- bridge/wire.rs             existing caller/worker/bridge JSON codecs
|   +-- bin/edit-gdscript.rs        planned thin caller entrypoint
+-- tests/                         required core/boundary behavioral regressions

godot-addon/
+-- addons/godot_agent_kit/
|   +-- plugin.gd / bridge.gd       existing lifecycle/routing; v2 cutover
|   +-- observation.gd             existing getter-only observations
|   +-- script_edit.gd             planned thin native-attempt integration
|   +-- export_guard.gd            extend native tooling exclusion
|   +-- native/                    planned installed editor-only extension/artifact
+-- native/                        planned small C++ source/build + engine API patch
+-- tests/
    +-- run_observation.py         retain all existing behavior acceptance
    +-- run_script_edit.py         planned focused mutation acceptance runner
    +-- fixtures/                  reuse owned-editor setup/witness facilities
```

**Structure Decision:** No new Rust package or top-level product component. Native code/engine adaptation belongs to `godot-addon/`; agent intent/core remains `mcp-server/`. Reuse the existing `Harness`, owned fixture setup, window capture, auth and independent witness facilities rather than clone the large observation harness. Extract only helpers actually needed by both runners under responsibility-based names if direct reuse is insufficient; no generic plugin/test framework. Existing shared artifacts must not acquire feature/task IDs in public APIs, transport, logs or ownership.

**Ownership review:** A/B are genuinely GDScript/editor capability-specific; bridge, supervision, confinement and fixture lifecycle are shared and keep responsibility-based names. No duplicate future infrastructure is built, and the script-edit domain is not generalized beyond present needs.

## Complexity Tracking

No constitutional violation/waiver is requested. Required Principle XIII decisions are recorded here rather than hiding added cost behind “native” or “security-sensitive.”

| Present mechanism/gap | Simplest alternative and insufficiency | Ongoing cost and present justification |
|---|---|---|
| Standard-ABI native library / bound persistence | Public Save paths can follow replacement paths; external writer adds a separate synchronization boundary. | C++ build, ABI/OS/lifetime tests; justified by demonstrated in-editor object/descriptor behavior. No godot-cpp/crate framework. |
| Narrow engine finalizer + saved-state/Save-profile inspection + target guard | CodeEdit tag omits document mtime; broad save/format/callback paths can affect more than the intended source/history. | Pinned editor patch and native-history/metadata/callback maintenance; required by actual Save failure and exact-source durability. |
| Existing validator + scoped read/effect context | Reload is not attributed parse success; naked binding can traverse effectful/unconfined dependency paths; subprocess loses context. | Small parser/cache/analyzer adaptation and dependency/effect regression coverage; closes FR-009/FR-020 without another language implementation. |
| Supervisor authorization/stage extension | Read-only worker termination cannot prove a sent mutation never applies. | One control handoff and certainty state in existing supervision; required by FR-012, not a recovery service. |
| Two kit callers based on X enter the same mutation path and overwrite/order changes | Reuse the existing collection/edit slot plus fresh revision checks; that mechanism is sufficient, so no new coordination layer is needed. | Make existing admission/cleanup ownership explicit and maintain T004's barrier regression. This closes a present stale-intent failure with no lock manager, lease service, queue, registry, replay system, merge algorithm or Phase 9 infrastructure. |
| Bridge v2 / edit stdin contract | Strict v1 shape/authentication cannot silently grow; source in argv or arbitrary file options adds exposure. | Coordinated codecs/fixtures/docs migration; one small operation, no compatibility shims or new protocol service. |
| Exact patched-editor build recipe and edit acceptance runner | Stock bindings and observation-only proof are insufficient. | Reproducible development build/provenance plus reused real-editor fixtures; no custom distribution, provider-specific CI or new approval layer. |

## Verification and Planning Completion

[quickstart.md](quickstart.md) maps all 22 requirements, 26 scenarios and eight success criteria to implementation verification, including all A–E, ≥20 successful sequential edits and ≥3 interleaved unsafe refusals. Existing Rust baseline checks and complete observation acceptance remain required where affected. Real GUI evidence may be maintainer-operated; existing hosted/protected workflow boundaries suffice. No CI provider/runner topology is introduced as a new prerequisite.

The original plan-generation pass performed only feature-path/workflow resolution, pinned-source/API review, specification/design coverage and local-link/anchor/Markdown/whitespace/full-diff checks. No Cargo/product suites, engine build, runtime probe, filter retry, A–E acceptance, task generation or analysis command was run by that pass. The installed `setup-plan.sh --json` was executed with the verified Feature 002 directory and retained the existing plan; the installed template was resolved before completing these artifacts. Before/after-plan hook checks found no `.specify/extensions.yml`, so no hook was registered or bypassed. That pass's read-only native/flow contract review findings were corrected without changing the then-current specification.

**Remaining material design questions:** None. Implementing the specified small APIs/effect guards, resolving exact build identities and proving behavior are implementation/acceptance work, not missing architectural decisions. This design neither asserts they are already proven nor requires a finished implementation before a plan can exist. Follow the normal reviewed-plan → tasks → granularity review → analysis → one-task/one-PR sequence only when separately requested/authorized; this continuation stops after updating PR #24.
