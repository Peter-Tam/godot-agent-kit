# Implementation Plan: Safely Edit Open GDScript

**Branch**: `spec/002-edit-open-gdscript` | **Feature identifier**: `002-edit-open-gdscript` | **Date**: 2026-09-27 | **Spec**: [spec.md](spec.md)

**Input**: `specs/002-edit-open-gdscript/spec.md`

**Status: ERROR — planning blocked at the Phase 0 persistence-safety gate.** This is a research/gate record, not a completed or approved implementation design. Phase 1 has not started. Do not derive tasks or implement against this document.

**Resume prerequisite:** Answer both focused Phase 0 questions: **A — target-bound single-script persistence** that preserves the validated target/revision and unrelated human dirty work; **B — parse evidence attributable to the selected target, resulting source/revision, and observation interval**. Planning may resume only if the answers establish a constitutionally safe route; otherwise record the unsupported requirement and keep the gate blocked. Do not generate tasks or downstream design artifacts to conceal either gap.

## Summary

The required capability remains one revision-guarded, native edit of an existing open standalone GDScript, preserving human work, independently verifying D/R/B convergence and parse state, preserving real native Undo/Redo, and requiring A–E and applicable durability evidence. The completed observation feature remains read-only and unchanged.

[Research and actual feasibility evidence](research.md) found a usable native text/history candidate but not a complete safe persistence route:

- Real CodeEdit complex operations preserved the prior native text history; Undo, intervening save/bookkeeping, Redo and close/reopen worked for the owned valid-script fixture.
- `ResourceSaver.save()` wrote the intended source after native R synchronization but did not mark the buffer saved. Explicit saved-version bookkeeping was required in the probe; that bookkeeping must itself be guarded in a real transaction.
- `ScriptEditor.save_all_scripts()` saved the unrelated dirty fixture document. It cannot meet the target-only requirement.
- After a controlled project-directory replacement, the path-based ResourceSaver candidate followed a symlink and overwrote an owned outside-project sentinel while returning `OK`. A read-time pathname check and post-write observation do not bind/prevent that write.
- A valid target returned `OK` from `Script.reload(true)` in the fixture. A general source-attributed parse/diagnostic path and its side effects are not established by that single result.

The plan does not substitute a refusal-only feature, weaken confinement or human-work protection, or name a hypothetical safe-save abstraction as though it were implemented/proven. The missing persistence design is a prerequisite to finalizing its transaction model and contracts.

## Technical Context

**Language/Version**: Retain existing Rust **1.98.1**, edition **2021**, and GDScript. The inspected native candidate is Godot **4.7.2.stable.official.ed1daf0bf**, full hash `ed1daf0bf001b61586d9930840f2f1394092c079`. Python standard-library orchestration was used only for disposable research. No mutation support claim is earned.

**Primary Dependencies**: Existing `serde =1.0.229` with derive, `serde_json =1.0.151`, `cap-std =4.0.3` without default features, and `ring =0.17.14` without default features and with `std`; Rust standard library and public Godot APIs. No new dependency or lockfile change. **NEEDS CLARIFICATION (technical research):** a concrete native/alternative persistence integration that binds the actual write to the selected target and expected revision, preserves newer work, and retains native editor semantics. Neither tested save route is selected.

**Storage**: Existing owner-private source-free routing metadata and bounded request-local observations remain unchanged. No database, journal, retained project-source cache, or recovery service is selected. Revision/state lifetime decisions depend on the real mutation boundary; an existing observation ID/hash is not a write authority.

**Testing**: Existing deterministic Rust contract/boundary coverage and maintainer-operated real-Godot acceptance remain the baseline. Parent research exercised public API inventory and an actual owned GUI editor; the exact limits and negative results are in [research.md](research.md). The complete product acceptance matrix is not implemented or run. No new test suite or CI gate is introduced by this planning attempt.

**Target Platform**: Existing macOS arm64 candidate. Current research host reports macOS **26.6.2**, build **25G83**, Apple M2; existing ordinary native CI remains separate from real-editor acceptance. No broader platform/version claim and no new provider/runner prerequisite.

**Project Type**: Existing reusable Rust library/local caller and Godot EditorPlugin. Preserve `caller → protocol-independent core → Godot integration`; no MCP surface or second mutation core. A future edit caller/wire result must be distinct from observation v1, but its concrete contract is not finalized before resolving the application/persistence boundary.

**Performance Goals**: The specification requires a terminal edit result within **ten seconds** in controlled acceptance. Existing observations retain **five seconds**. These are future acceptance obligations, not measurements from the disposable multi-step controller. Worker termination/disconnection must not imply that an editor mutation could not apply later.

**Constraints**: Independent D/R/B and document-specific dirty evidence; independently attributable parse state; stale-write protection at the real mutation boundary; no target guessing, hidden opening/loading, force/merge, automatic mutation retry, unsafe rollback, out-of-project writes, source logging, or gameplay authority. **NEEDS CLARIFICATION (technical research):** source/revision-attributed parse evidence and the final combined native edit/persistence/verification boundary, including reentrancy/interruption semantics.

**Scale/Scope**: One already-open standalone project GDScript per attempt. No generalized transaction framework, script lifecycle/history-control commands, batch/concurrent-agent orchestration, runtime/debugger/LSP tools, scene/resource authoring, distribution work, or platform expansion.

## Constitution Check

**Initial gate, before Phase 0: PASS for research only.** The specification preserves Principles I–XIII and explicitly exposes native mutation/persistence/parse, confinement, and interruption unknowns. No safety exception was presumed.

**Post-research gate: FAIL for the investigated persistence candidates.** Source evidence and the actual directory-redirection counterexample prevent selecting a pathname-based save as a confined transaction. Save All has an independently observed unrelated-document side effect. Required safety is unresolved, not deferred to optimistic implementation.

**Post-Phase-1 gate: NOT REACHED.** Phase 1 requires completed research without unresolved material decisions. There is no completed data model, mutation contract, quickstart, implementation plan approval, release, or support claim.

| Constitutional gate | Evidence / current disposition |
|---|---|
| I — Independent D/R/B | Separate native R/B and controller D witnesses were used in the fixture. Equal text does not establish target confinement or full feature acceptance. Preserve all independent observations in any resumed design. |
| II — Preserve human work | Native text/history mechanics preserved the unrelated unsaved fixture until the deliberate Save All negative case. Save All is rejected. Guarded saved-version bookkeeping and protection against newer work remain mandatory. |
| III — Native transactions/history | CodeEdit/TextEdit native history has positive bounded evidence. A second history stack, raw write-and-hope reload, or undoable flag is not a substitute. Target-specific persistence integration remains unresolved. |
| IV — Independent verification | ResourceSaver `OK`, `tag_saved_version`, and a reload result are not mutation success. Application certainty and independently observed postconditions must remain separate in the eventual contract. |
| V — Confinement/least privilege | **FAIL for the path-save candidate:** a controlled parent-directory replacement redirected its write outside the selected project. Existing confined reads do not confer their authority on a later path-based save. No new permission gate is used to excuse this. |
| VI — Real-editor gates | A–E and applicable durability remain mandatory. The disposable research sequence is not the product acceptance suite and does not mark any mutation gate complete. |
| VII — Protocol independence | Existing durable ownership is retained. No core/caller/addon changes or parallel safety path were introduced. |
| VIII — Tooling isolation | No product addon/runtime/export changes. Any later mutation implementation must re-prove its affected export boundary. |
| IX — Small surface | One intended edit capability; no extra product operation is introduced to hide the persistence problem. |
| X — Truthful diagnostics | Gate failure and uncertainty are explicit. No success, rollback, no-application, parse-validity, or support claim is invented. |
| XI — Independent implementation | Public Godot documentation, exact runtime metadata, existing repository source and independently written disposable probes were used; no third-party implementation was copied. |
| XII — Correctness first | **BLOCKED:** do not implement a mutation path whose confined persistence/human-work boundary is only assumed. Safe refusals cannot replace the specified positive capability. |
| XIII — Justified complexity | No speculative writer service, extension framework, engine distribution, approval layer, or GUI-CI topology is selected. Any proposed new boundary must answer the concrete cost/alternative questions below and demonstrate the added protection. |
| Compatibility/workflow | Observation v1 and its deadlines remain unchanged; no tasks, product code, new public schema or untested compatibility promise. The current specification is not silently amended. |

### Conditions for resuming Phase 1

1. Establish a concrete persistence mechanism whose actual write remains bound to the selected project/document authority despite filesystem replacement, and whose revision/newer-work semantics satisfy the specification. A second path check or a digest alone is not that mechanism.
2. Integrate it with a guarded native text/history and saved-state transition; demonstrate an ordinary positive edit, preserved unrelated dirty work, native Undo/Redo and saved/reopened coherence. Do not conflate the research candidate's separate steps with one safe transaction.
3. Establish target/source/revision-attributed parse evidence, including invalid-source behavior and explicit side effects/limitations, rather than assuming the valid-fixture reload result covers general scripts.
4. Define application uncertainty, permanent pre-application discard, timeout/disconnection and no-retry behavior against that actual boundary. Preserve existing read-only compatibility and privacy.
5. Record the Principle XIII assessment for any additional mechanism, then re-evaluate the constitutional gate. An alternative native/capability-scoped integration is not ruled out, but its missing guarantees cannot be replaced by a contract assertion.

These are technical blockers, not a request for blanket maintainer approval to bypass safeguards. No constitutional amendment or scope reduction has been requested or applied.

## Project Structure

### Documentation (this feature)

```text
specs/002-edit-open-gdscript/
├── spec.md                         # Existing feature specification; unchanged
├── checklists/requirements.md      # Existing requirements-quality review; unchanged
├── plan.md                         # This blocked planning/gate record
└── research.md                     # Source research and actual feasibility evidence
```

`data-model.md`, `contracts/`, and `quickstart.md` are **not generated**: their Phase 1 prerequisite is unsatisfied. No empty/provisional contract files or pretend runnable mutation commands are created. `tasks.md` remains for a later authorized command after a complete reviewed plan; the repository's granularity-review and one-implementation-task-per-PR gates remain intact.

### Source Code (repository root)

```text
mcp-server/
├── Cargo.toml, Cargo.lock, rust-toolchain.toml
├── src/                            # Existing core/caller/bridge/confined-read implementation
└── tests/                          # Existing observation and boundary coverage

godot-addon/
├── addons/godot_agent_kit/          # Existing editor observation/session/export integration
└── tests/                          # Existing owned-editor observation harness and fixtures
```

**Structure decision:** Preserve the existing two component boundaries. No product source or permanent tests were changed; no new source layout is committed in the absence of a safe persistence design. Research ran only in owned disposable directories outside the repository.

## Complexity Tracking

No constitutional exception or new infrastructure is approved by this document. The present risk is concrete: a real path-save candidate overwrote an owned out-of-project sentinel after namespace replacement, and Save All persisted unrelated unsaved work.

| Question | Recorded assessment |
|---|---|
| Current requirement/failure | A target-only mutation must preserve human work and confine the actual persistence write, not just its initial read. The negative probes demonstrate the missing guarantee. |
| Simplest credible alternative | First reuse public native text/history and persistence facilities. Save All fails target isolation; ResourceSaver plus source checks and saved-version bookkeeping has useful mechanics but not a bound write. A native-buffer/capability-scoped persistence integration remains a research direction, not a selected abstraction. |
| Why existing mechanisms are insufficient | Authenticated selection and `cap-std` reads protect acquisition; the tested Godot saver accepts a path and re-resolves it. Read witnesses and post-write checks do not prevent the observed write. Observation worker cancellation does not revoke an editor operation. |
| Ongoing cost of a proposed additional boundary | Any scoped writer/native integration would add target/handle lifetime management, write-error and competing-revision semantics, editor saved-state integration, compatibility maintenance and real-editor/race regressions. Exact mechanisms and costs cannot be asserted before selecting and demonstrating one. |
| Why introduce it now? | Only a demonstrated mechanism closing the current mutation safety gap would justify that cost. No mechanism is introduced merely to make the plan look complete; planning remains blocked. |

The mandatory post-planning hook check still applies when reporting a planning attempt. These research artifacts authorize no implementation, task generation, or merge. The maintainer has separately authorized preserving this blocked evidence and continuing narrow A/B research on `spec/002-edit-open-gdscript` in existing PR #24; no new PR is authorized.
