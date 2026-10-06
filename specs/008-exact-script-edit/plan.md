# Implementation Plan: Exact Partial Editing for Project GDScripts

**Branch:** `spec/008-exact-script-edit` | **Feature identifier:** `008-exact-script-edit` | **Date:** 2026-10-06 | **Spec:** [spec.md](spec.md)

**Input:** Clarified Feature 008 specification, including the clean Schema 2 cutover and complete-empty-source replacement amendment. The setup helper returned feature identifier `008-exact-script-edit`; the actual dedicated Git design branch is the branch above.

**Status:** Phase 0 research, Phase 1 design and [task derivation](tasks.md) are complete; the task list records its granularity review. Consistency analysis and implementation have not run. This is a proposed implementation plan, not product acceptance or authorization to skip the remaining design workflow. Feature 008 is in **Planning**; Phase 3 remains complete and Phase 4 remains pending.

## Summary

Replace only the public MCP whole-source edit input with one literal `old_string`/`new_string` pair. After fresh authenticated acquisition and current revision comparison, derive the complete intended source from the same frozen admissible capture. Reuse the existing trusted open/closed full-source transactions, validators, history, confinement, independent verification, effect-sensitive outcomes and lifecycle guarantees. Exactly `discover_scripts`, `read_script` and `edit_script` remain.

Nonempty old text must occur once, counting overlaps. Empty old text denotes replacement of the complete freshly acquired empty source only; empty new text supports deletion. No normalization, fuzzy fallback, insertion mode or file creation exists. Schema 2 replaces Schema 1 MCP input without a compatibility period. Independent local whole-source APIs remain genuine current contracts, not a second MCP mode.

One important precedence detail comes from source review: read eligibility does not contain every later saved-version/Resource-edited/context check. A failed derivation therefore retains its match error while the existing unchanged path verifies applicable safety/no effects; only a proven unchanged result may be reduced to that matching refusal. No additional no-op precedes successful derivations, no native/private protocol change is selected, and all work remains inside the original edit clock.

## Technical Context

**Language/Version:** Existing Rust 1.98.1 / edition 2021, GDScript, C++17 public GDExtension integration and Python 3.10+ evidence tooling. Production changes are scoped to Rust intent/orchestration/adapter code; addon/native code remains unchanged by design. No new edition/MSRV claim.

**Primary Dependencies:** Retain exact locked serde 1.0.229, serde_json 1.0.151, cap-std 4.0.3, ring 0.17.14, rmcp 3.5.0 and Tokio 1.53.1, with existing features. Standard-library substring search suffices; no dependency, framework, engine patch or protocol upgrade.

**Storage:** Existing source-free registry and request-local source/capture/evidence. One private complete/exact intent enum and optional retained typed derivation failure; no database, persistent revision/match store, queue or replay cache.

**Testing:** Existing Rust behavior and process tests, focused Python fixture/consumer checks and real-Godot VM witnesses. Both Codex CLI 0.153.4 and OMP 18.5.1 require new Schema 2 positive/deletion/empty-source workflows; at least one actual-agent composed A–E/refusal-recovery workflow is required. [Quickstart coverage](quickstart.md#acceptance-coverage) maps every scenario, FR and SC under [TEST_POLICY.md](../../TEST_POLICY.md).

**Target Platform:** Official Godot `4.7.2.stable.official.ed1daf0bf`, engine commit `ed1daf0bf001b61586d9930840f2f1394092c079`, executable SHA-256 `c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf`; macOS 26.6.2 (25G83) arm64, dedicated graphical VM with four vCPUs / 6 GiB. Local stdio MCP 2025-11-25; no support expansion or active-game/cached-class hot-reload claim.

**Project Type:** Existing one-package Rust library/CLI and thin Godot addon/native integration. Bounded refinement of the delivered script workflow, not a new application/service or roadmap phase.

**Performance Goals:** Five-second read/discover and ten-second edit through consumed response delivery, including matching, rejected-intent safety checks, validation and verification. Retain 4.5/9.5-second work cutoffs and existing startup preparation. At most two bounded linear-time standard substring searches, constant search space, one complete-result construction, no match list or needle-table allocation.

**Constraints:** Existing 512 KiB source/profile bound, exact LF UTF-8 without NUL/CR/BOM normalization, explicit authenticated/confined selection, current stateless revision, human-work protection, fixed admitted lifecycle, independently observed complete postconditions, truthful effects and sticky disclosure. Fragments are not validated as standalone scripts. Complete empty source is not missing/unavailable source.

**Scale/Scope:** Three static tools, one public edit shape, one admitted execution per connection, no queue/retry/force/batch. Public root Schema 2 is independent of unchanged `sr1`, local-operation v1, private bridge v6, native revision 4, MCP revision and package version.

## Constitution Check

*Gate evaluated before Phase 0 research and re-evaluated after Phase 1 design against constitution v1.2.0.*

**Initial disposition: PASS for research.** The maintainer resolved migration and empty-source product choices. Research was restricted to existing boundaries and source-backed design, with no constitutional exception, new mutation writer or assumed implementation pass.

**Post-design disposition: PASS for design.** R1–R3 are resolved. No unjustified violation or unresolved product clarification remains. The selected design preserves constitutional MUST requirements; actual execution/client/time/effect acceptance remains required before implementation completion. Missing future evidence is not a current product support claim.

| Obligation | Selected design / verification ownership |
| --- | --- |
| I — independent coherence | Complete intended D/R/B for open, D/applicable R plus positive continuing absence for closed. Span agreement, worker receipts and disk/runtime agreement alone are insufficient. |
| II — preserve human work | Fresh state-bound revision and all later dirty/saved/Resource/identity/lifecycle guards. Match failures cannot hide preparation-only safety failures; no repair or forced synchronization. |
| III — native transactions | Reuse native whole-source edit/history/persistence. Matching never acquires filesystem or editor authority. Real prior history/Undo/Redo remains required. |
| IV — verified outcomes | Acceptance, preparation, effects, validation and independent postconditions remain distinct. Retained match error can replace only a proven zero-effect unchanged result, never a failure/uncertain attempt. |
| V — least privilege | Existing local stdio, authenticated routing, confinement and permissions. No remote product transport, arbitrary execution, telemetry, outside-project authority or extra approval mechanism. |
| VI — real editor evidence | New actual-client exact-path A–E and durability plus focused affected cases in the dedicated VM; unchanged valid historical foundations may be reused. |
| VII — protocol independence | Exact intent and transformation below MCP; DTOs/catalog at adapter; full-source Godot execution unchanged. Local full-source callers remain independent. |
| VIII — tooling separation | No gameplay/export role changes. Preserve matched tooling exclusion and review reusable export provenance; rerun only affected export obligations. |
| IX — lean effective surface | Exactly three tools, one familiar pair, concise parameter constraints and outcome-time guidance, structured facts. No internal architecture instructions, duplicate safety lectures or description budget. |
| X — truthful diagnostics | Distinguishable source-free match reasons, existing safety/effect precedence and action fields; no source suggestions, false rollback, automatic replay or erased disclosure denial. |
| XI — independent implementation | Existing dependencies/public Rust APIs and repository evidence; no external editor engine copied. External interface familiarity in the specification is not mutation authority. |
| XII — quality before capability | Whole-source validation and existing trusted paths remain mandatory, including empty/equal/deletion cases. Unknown required facts refuse; no disk-write workaround. |
| XIII — proportionate complexity | Concrete source-intent enum, bounded matcher and retained-error reduction only; present need, alternatives and cost below. No generic framework, new dependency, runner or CI/approval gate. |
| Compatibility / workflow | Deliberate Schema 2 migration; old/mixed forms rejected, every current MCP caller migrated, historical Feature 007 contracts retained. Plan ends here; tasks → granularity review → analyze precede one-task/one-PR implementation. |

The existing supported normal-local-user threat model is unchanged. No hostile same-UID isolation, atomic filesystem exclusion, universal callback purity or class hot reload is invented. If implementation discovers it cannot meet the specified guarantees within existing support, record a design correction before dependent work; do not weaken the contract silently.

## Execution and interface decisions

[Research](research.md) records evidence, rationale and alternatives. [Data model](data-model.md) defines entities/bounds/state transitions. [Public contract](contracts/mcp-interface.md) defines Schema 2 input/output/version/migration, and [execution contract](contracts/execution.md) defines fresh-capture ownership and refusal precedence.

- Keep `ScriptEditRequest::new` for actual whole-source local consumers; add a crate-private exact constructor backed by a private concrete enum. Narrow visibility avoids an unnecessary public transformation API or a second MCP representation.
- Match only after fresh capture and revision equality, borrowing source from the same checked authority set used to construct the basis. Use two `str::find` calls, resuming the second at the next UTF-8 boundary after the first start to count overlaps. Check combined byte length before one complete-source allocation.
- Successful derivation reuses the current full-source path once. Derivation failure is withheld until unchanged execution verifies applicable safety and zero effects; propagate every intervening safety/validation/disclosure/cancellation failure instead. Never return successful no-match or infer prior application from new text.
- Keep native/private contracts, source validators, stateless revision commitment and local CLI behavior unchanged. Existing later guards still catch source/identity/lifecycle races after matching; no reacquired basis rescues stale intent.
- Advertise root schema 2 for all three outputs, decode only old/new in MCP, retain structured content plus terse summary and add relevant action guidance. Update all current MCP callers/fixtures/docs in the implementation cutover; do not mechanically rename private whole-source payloads.

## Project Structure

### Documentation for this feature

```text
specs/008-exact-script-edit/
├── spec.md
├── checklists/requirements.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── tasks.md
└── contracts/
    ├── mcp-interface.md
    └── execution.md
```

The planning stage generated neither tasks nor analysis. The subsequent [task list](tasks.md) now derives meaningful capability increments with directly required tests/docs and records their granularity review; it does not introduce file/symbol-only tasks or a separate migration cleanup task. `analysis.md` remains absent until the requested consistency-analysis stage actually runs.

### Implementation ownership in the existing tree

```text
mcp-server/src/
├── script_edit/request.rs        # checked exact pair and complete-source derivation
├── script_edit.rs                # crate-private access only where current callers need it
├── script_read.rs                # private intent representation and preserved local constructor
├── runner/read.rs                # shared capture/dispatch and retained-error interpretation
└── mcp/
    ├── schema.rs / handler.rs    # single Schema 2 input and checked decoding
    └── output.rs / output/       # root version, faithful projections, relevant guidance
mcp-server/tests/
├── mcp_tools.rs                  # actual public process/argument behavior
└── mcp_transport.rs              # affected version/carrier/strict-boundary behavior
godot-addon/tests/
├── mcp_peer.py                   # actual MCP request builder
├── mcp_*_acceptance.py           # current profile/consumer/witness ownership
├── test_mcp_*.py                 # directly affected consumer/selection behaviors
└── run_in_vm.py / run_mcp.py     # reuse existing commands; no new runner
.github/LOCAL_VM.md               # current MCP operating guidance on implementation cutover
```

These are planned locations, not files or scaffolds created by this stage. Existing native/addon code and the local full-source fixture remain outside planned production changes. Fixture data can change where directly required by exact/empty/refusal scenarios; preserve native fault/production export isolation.

**Structure decision:** Bounded text intent belongs beside existing source request validation; capture and terminal orchestration remain in the existing runner. Do not append matching to transport or add a generic transaction/transform layer. Material changes to large production modules must receive cohesion/ownership review; existing large transport/wire/native modules are not refactor targets merely because their callers migrate. Any simple responsibility cleanup introduced by this task belongs within that same task, not automatic follow-up scope.

## Complexity Tracking

No constitutional exception is requested; the template's violation table is inapplicable. Principle XIII still requires present-cost justification:

| Addition and present requirement | Simplest alternative / why existing mechanisms alone are insufficient | Ongoing cost / why justified now |
| --- | --- | --- |
| Bounded literal derivation — FR-004–FR-006 | Existing full-source input or a caller helper still resends unaffected source. Generic patch/regex engines add unwanted behavior. | Small standard-library transformation and edge tests; no dependency or second writer. |
| Private complete/exact intent — independent current callers | Replacing the whole-source constructor would break real fixture/local consumers; duplicate runners would fork acquisition and safety. | Two concrete private variants, one shared dispatch, redacted exact value; no public mode/extension point. |
| Retained-error unchanged assessment — FR-009 | Immediate no-match skips saved-version/Resource-edited/context checks located in existing prepare; duplicating checks or adding a probe RPC creates another policy/contract. | Reuse existing zero-effect validation only on rejected derivations, with focused late-failure/effect tests and original timing; preserves safety reason precedence now. |
| Versioned Schema 2 cutover — FR-014 | Silent Schema 1 break or dual-mode compatibility violates the clarified product decision. | One migration of current MCP callers, schemas, examples and guidance; no compatibility period, alias or release machinery. |
| New actual-client/profile coverage — FR-017–FR-018 | Historical whole-source client passes do not establish exact matching, empty-source handling or model-visible Schema 2 use. | Extend existing VM/fixture/relay consumers and composed run; reuse valid unchanged native evidence, no new infrastructure. |

**Design-shape review:** Every selected type and branch has a current caller. No speculative enum variants, public hooks, generic traits, lifecycle booleans or unsafe code are selected. One owned complete source satisfies the existing mutation path; search is borrowed and allocation-free. Recoverable malformed/unavailable evidence uses typed failure, not `unwrap`/`expect`. The no-effect diagnostic reduction cannot weaken actual uncertainty or disclosure.

## Verification and stage boundary

The [quickstart](quickstart.md) supplies concrete installed entrypoints, scoped future acceptance, all 25 scenario / 18 FR / 8 SC coverage and evidence-reuse rules. Both actual clients must demonstrate the new representation; SDK fixtures and model assurances cannot replace them. No blanket historical GUI, native rebuild, hosted-CI topology or fresh release campaign follows from this plan.

For this documentation-only stage, validate Markdown rendering/structure, local links/anchors, fenced shell/JSON syntax and complete intended/staged diff/whitespace; check pre/post extension hooks and publish the design-stage delta under [SPECKIT_WORKFLOW.md](../../SPECKIT_WORKFLOW.md). No product tests, native builds, Godot or real-agent campaign is claimed. Publication updates the existing draft design PR but neither marks implementation complete nor starts task derivation automatically.
