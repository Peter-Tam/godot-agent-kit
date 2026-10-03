# Phase 2 Exit — Protocol-independent Automation/Transaction Core

**Decision date:** 2026-10-03

**Assessed and closure base:** `0b8ebb0d1808a4ba89330489a175381561433aaa`

## Decision

**Phase 2 — Protocol-independent automation/transaction core — is complete on the
documented support profile.** The completed requirements-to-evidence exit
assessment found no product-capability, reusable-core, or behavioral
acceptance-evidence gap. This record preserves that conclusion and performs the
remaining lifecycle/status bookkeeping; it is not a feature or another campaign.

[PR #61][correction] is merged into `main`, and its merge commit is the assessed
revision above. Latest `main` was unchanged when this closure began, so no
production/core change invalidated the assessment. [Phase 1 remains complete][phase-one];
[Feature 006 standalone Save remains assessed, not proceeding][save-decision].
Phase 3 is not started, and no next feature is selected by this closure.

## Scope

- Official Godot **4.7.2.stable.official.ed1daf0bf**, engine commit
  `ed1daf0bf001b61586d9930840f2f1394092c079`.
- **macOS 26.6.2 (25G83), arm64**, retaining the recorded host/guest, toolchain,
  native-build and export-template provenance.
- The [existing operation-specific standalone project GDScript profiles and
  limits][support] remain unchanged. In particular, editing requires a clean,
  already-open, revision-bound target with admitted LF UTF-8 source and native
  source/Save profiles. Unsupported effects, representations and missing safety
  observations refuse explicitly; dirty/divergent observation is not mutation
  permission.

This does not claim general Godot support, other platforms, arbitrary
Resource/document types, runtime/debugger/language-intelligence tooling, MCP
completion or release readiness beyond the recorded roadmap meaning. Existing
limitations and export boundaries remain in force; this is not a release candidate.

## Phase 2 purpose

The [original roadmap obligation][roadmap] is to “Generalize the proven script
workflow into reusable semantics without losing its guarantees.” It explicitly
states: “Phase 1 already respects this boundary; this phase consolidates
demonstrated behavior, not a speculative framework.”

The assessment established reusable protocol-independent ownership of the proven
workflow, not a requirement to replace it with generic infrastructure. Constitutional
[protocol independence][core-principle] and [Principle XIII][complexity] remain the
basis for retaining the smallest sufficient design without weakening safety.

## Requirements-to-ownership conclusion

All original Phase 2 capability areas are satisfied. **Some semantics intentionally
remain operation-specific; operation-specific ownership is not itself a Phase 2
gap.** The table links the assessed owners and accepted evidence rather than
introducing new contracts. “Reusable” and “operation-specific” both mean satisfied;
no remaining implementation or assessment/evidence gap was identified.

| Roadmap capability | Assessed ownership | Contract / accepted evidence |
| --- | --- | --- |
| Sessions | Reusable [authenticated resolver][routing]; exact session or uniquely authenticated selection, never focus/first-candidate selection. | [Observation routing][observation-contract]; [accepted observation][observation] and [cumulative routing review][close-review]. |
| Explicit targets | Reusable checked target models, [project confinement][confinement] and [native identity/descriptor guards][native-guards]. | [Observation contract][observation-contract]; [cumulative confinement evidence][close-review]. |
| Revisions | Operation-specific [expected revision basis][basis], edit/close admission and native boundary rechecks; prior evidence is not authorization. | [Edit contract][edit-contract] and [stale/identity acceptance][edit-outcomes]; [close evidence][closing]. |
| Transaction/admission lifecycle | Operation-specific reducers and supervised runners, with the [shared editor operation slot][editor-slot] and guarded native attempt ownership. | [Edit][edit-outcomes], [opening][opening] and [closing][close-review] boundary/overlap evidence. |
| Conflicts | Operation-specific edit/open/close admission preserves dirty work, including equal-text dirty state, and refuses stale/divergent or unobservable safety state. | [Edit conflict evidence][edit-outcomes], [opening preservation][opening] and [close preservation][close-review]. |
| Mutation outcomes | Operation-specific [edit][edit-results], [opening][open-reducer] and [closing][close-reducer] reduction separates acceptance, effects and independent verification. | [Edit result review][edit-outcomes], [opening contract][open-contract] and [close result review][closing]. |
| Diagnostics | Operation-specific reasons plus [structured stage/surface diagnostics][observation-model]; parse, stale, denied, unsupported and unavailable facts stay distinct where applicable. | [Observation][observation-contract], [edit][edit-contract], [open][open-contract] and [close][close-contract] result contracts and accepted result reviews. |
| Deadlines/timeouts | Reusable [clock and owned-worker supervision][supervision], with bounded operation-specific execution and delivery budgets. | [Bounded observation execution][bounded-observation]; [edit timing][edit-outcomes], [opening][opening] and [close interruption evidence][close-review]. |
| Cancellation | Operation-specific effect-sensitive reduction under supervised cancellation and native owner/expiry rules; cancellation is not rollback. | [Edit interruptions][edit-outcomes], [opening interruptions][opening] and [close late-barrier/newer-work evidence][close-review]. |
| Idempotency/retry semantics | Operation-specific one-attempt contracts and safe-next-action guidance: no automatic mutation replay; fresh observation/basis before a new intentional action. | [Observation retries][observation-contract], [opening deadline/retry contract][open-retry] and [closing contract][close-contract]. |
| Partial/unknown effects | Operation-specific reducers retain known application and distinguish partial effects, possible effects and proven no-effect termination. | [Edit outcomes][edit-outcomes], [opening contract][open-contract], [close interruptions][close-review] and [PR #61 regression][disclosure]. |
| Correlated structured tracing | Reusable [request IDs, clocks, collection stamps and diagnostics][observation-model], with [operation progress][edit-results] and request-bound worker/bridge events. | [Observation result contract][observation-contract] and [edit stage/timing/result-only review][edit-outcomes]; no tracing backend is required. |
| Project-content redaction | Operation-specific disclosure reduction, source-free routing errors and bounded codecs; opening denial is sticky independently of causal precedence. | [Cumulative privacy/export evidence][close-review] and [focused opening disclosure regression][disclosure]. |

## Reusable-core boundary

The supported execution flow is:

```text
future adapter
    -> checked request + supervised reusable operation
    -> authenticated/confined Godot integration
    -> independent evidence reduction
    -> structured operation outcome
```

Reducers alone are not the entire core. The [Rust library][library] includes domain
models/reducers, [supervised runners][supervision], authenticated session routing,
project confinement and guarded integration. Independent acquisition/verification
and truthful structured outcomes remain below future protocol adapters, consistent
with the [conceptual architecture][architecture].

Existing reusable entrypoints are `runner::run` for observation,
[`runner::discovery::run`][discovery-runner], [`runner::open::run`][open-runner],
[`runner::edit::run`][edit-runner] and [`runner::close::run`][close-runner]. Existing
CLI consumers already dispatch them. Opening/closing summaries do not become edit
permission; effectful requests retain their ordinary observation/basis requirements.
The [accepted discovery-selected composition][discovery] and [full close/reopen
workflow][closing] exercise these paths.

## Thin-adapter conclusion

A future protocol adapter needs protocol input/output mapping and ordinary host
wiring: configured registry and request correlation, starting/passing the existing
clock, connecting protocol cancellation to existing cancellation state, dispatching
existing worker/helper entrypoints, and integrating synchronous supervision with
its protocol runtime. It maps the returned structured outcome without weakening it.

It does **not** reconstruct session selection, target/project confinement,
revision/stale protection, dirty-work/conflict policy, operation serialization or
admission, may-apply/effect certainty, deadline semantics, cancellation
interpretation, partial/unknown-effect reduction, retry safety, source disclosure
or terminal outcome truthfulness. This is the architectural property Phase 2 was
intended to establish. Remaining protocol/host wiring is a Phase 3 concern, not
unfinished Phase 2 core work.

## PR #61 correction

The one concrete Phase 2 entry gap was an **existing Feature 003 opening-contract
regression**, not a new Phase 2 capability:

```text
earlier causal failure -> later disclosure denial
    -> previously retained source could survive terminal reduction
```

[PR #61][correction] corrected the [existing opening reducer][open-reducer] with
sticky `source_scope_invalid`, separately from first-failure precedence. The first
causal failure stays truthful; later disclosure denial remains sticky;
prohibited `before` source summaries and `observation` source evidence are
suppressed, including invalidated evidence. Known effects/application certainty
and permitted source-free diagnostics, stage and target metadata remain available.

The [Feature 003 regression record][disclosure] establishes focused
failing-before/passing-after behavior and the existing denial scope. No
adapter-side redaction policy, schema change, new capability or generic redaction
framework was introduced. The entry gap is resolved.

## Phase 2 exit criterion

Each clause of the [unchanged roadmap exit][roadmap] is satisfied:

1. **Reusable execution/core paths — satisfied.** The completed observe, discover,
   open, edit and close workflow uses the library entrypoints above, not CLI-only
   safety policy.
2. **Same A–E behavior — satisfied.** The [accepted Phase 1 gates][gates] and
   [full composed workflow][closing] retain coherent editing, human-work protection,
   actual native history, close/reopen persistence and sequential-edit stress.
3. **Same observed postconditions — satisfied.** Independent D/R/B, document/file
   identity, dirty/saved state, effect certainty and operation-specific verification
   remain owned below adapters. Native acknowledgment is not verified success.
4. **Safe failure behavior — satisfied.** [Accepted interruption/partial-effect
   evidence][close-review] retains truthful timeout, cancellation, disconnection,
   stale/dirty refusal and partial/unknown results without implied rollback/replay.
5. **No weaker supported mutation path — satisfied.** Public reusable mutation
   entrypoints are supervised; authenticated channels and mutation RPCs remain
   private/crate-private, and guarded native ownership remains mandatory. Public
   evidence/reducer APIs cannot themselves mutate Godot. This assesses supported
   adapters, not hostile in-process Rust or arbitrary private-module modification.

## Evidence currentness

Apply [TEST_POLICY's relevant-input rules][reuse], not literal final-commit replay:

- **Changed terminal disclosure behavior:** PR #61's focused deterministic reducer
  and actual-caller/authenticated-boundary regressions cover the late-denial
  ordering. Its record reports 40 focused passing tests and applicable checks.
  Historical GUI acceptance is **not** claimed to have covered that new ordering.
- **Unchanged native/editor behavior:** the [Phase 1 evidence chain][phase-one-evidence]
  and [Feature 005 cumulative review][close-review] retain accepted Features 001–005
  acquisition, admission, effects, A–E/durability, confinement and privacy/export
  evidence. PR #61 did not change those paths, native artifacts, bridge/ABI,
  GUI fixtures, campaign tooling, dependencies or supported environment.
- **Documentation/lifecycle updates:** this closure changes no product, protocol,
  runner, reducer, test/fixture, build/CI, dependency or runtime input. It does not
  invalidate accepted evidence and requires no product suite or VM campaign.

The completed exit assessment compared executed source
`0790385c000e9d95aa05ea2a6cb59bbf8e27a947` through the assessed base: intervening
changes were documentation-only apart from PR #61's reducer correction and focused
regressions. It also inspected the retained close sequential **45**, composed
**121** and clean-close **134** passing-record summaries and matched their hashes
to the acceptance record. These are historical evidence-record counts, not new
executions or distinct requirement counts. No uncovered invalidated evidence remains.

## Explicit non-requirements

Phase 2 completion does not require the following; these are **not unfinished
Phase 2 checklist items**:

- A generic transaction manager, `GenericOperation<T>` or workflow engine.
- Automatic retries, a replay cache or a generic cancellation framework.
- A generic redaction engine, tracing backend or telemetry system.
- A generic capability registry or broad API reshaping.
- Standalone Save or standalone Undo/Redo. [Edit-owned persistence and actual
  native history][save-history] remain demonstrated; [Feature 006][save-decision]
  remains assessed, not proceeding.
- Concurrent multi-editor mutation or broader Godot/platform support.
- MCP implementation.

No new infrastructure, feature, phase-review framework or acceptance campaign is
created. Phase 3 and Feature 007 are not started; a future phase/feature requires
its own separate product-value/current-state decision. Completion does not select
an MCP SDK/specification, transport, tool schema or release scope.

[library]: mcp-server/src/lib.rs
[routing]: mcp-server/src/target.rs
[confinement]: mcp-server/src/project_fs.rs
[native-guards]: godot-addon/native/document_guard.cpp
[basis]: mcp-server/src/script_edit/request.rs
[editor-slot]: godot-addon/addons/godot_agent_kit/bridge.gd
[observation-model]: mcp-server/src/observation.rs
[edit-results]: mcp-server/src/script_edit/outcome.rs
[open-reducer]: mcp-server/src/script_open/attempt.rs
[close-reducer]: mcp-server/src/script_close/attempt.rs
[supervision]: mcp-server/src/runner.rs
[discovery-runner]: mcp-server/src/runner/discovery.rs
[open-runner]: mcp-server/src/runner/open.rs
[edit-runner]: mcp-server/src/runner/edit.rs
[close-runner]: mcp-server/src/runner/close.rs
[roadmap]: ROADMAP.md#phase-2--protocol-independent-automationtransaction-core
[architecture]: ARCHITECTURE.md#flow-dependencies-and-correctness
[core-principle]: .specify/memory/constitution.md#vii-keep-the-core-protocol-independent
[complexity]: .specify/memory/constitution.md#xiii-justify-complexity-with-concrete-present-risk
[phase-one]: PHASE_1_EXIT.md#decision
[support]: PHASE_1_EXIT.md#scope
[gates]: PHASE_1_EXIT.md#real-editor-ae-gates
[save-history]: PHASE_1_EXIT.md#savehistory-interpretation
[phase-one-evidence]: PHASE_1_EXIT.md#evidence-reuse-and-currentness
[save-decision]: specs/006-save-project-gdscript/decision.md#decision
[observation]: specs/001-observe-gdscript-state/quickstart.md#29-rebased-t008-real-editor-acceptance-and-completion-2026-09-27
[observation-contract]: specs/001-observe-gdscript-state/contracts/observation-api.md
[bounded-observation]: specs/001-observe-gdscript-state/contracts/observation-api.md#9-implemented-t003-bounded-execution
[edit-outcomes]: specs/002-edit-open-gdscript/quickstart.md#coverage-and-observed-outcomes
[edit-contract]: specs/002-edit-open-gdscript/contracts/edit-api.md
[opening]: specs/003-open-project-gdscript/quickstart.md#10-t003-cumulative-acceptance-2026-09-30
[open-contract]: specs/003-open-project-gdscript/contracts/open-api.md
[open-retry]: specs/003-open-project-gdscript/contracts/open-api.md#5-deadlines-privacy-and-compatibility
[discovery]: specs/004-discover-project-gdscript/quickstart.md#11-t003-cumulative-acceptance-2026-10-01
[closing]: specs/005-close-project-gdscript/quickstart.md#12-t003-cumulative-acceptance-2026-10-02
[close-review]: specs/005-close-project-gdscript/quickstart.md#t003-execution-decision-and-evidence-review
[close-contract]: specs/005-close-project-gdscript/contracts/close-api.md
[correction]: https://github.com/Peter-Tam/godot-agent-kit/pull/61
[disclosure]: specs/003-open-project-gdscript/quickstart.md#11-post-completion-disclosure-regression-correction-2026-10-03
[reuse]: TEST_POLICY.md#reusing-evidence-across-commits

## Exit conclusion

No product-capability, reusable-core, or behavioral acceptance-evidence gap remains
for Phase 2 on the documented support profile.

**Phase 2 is complete.**
