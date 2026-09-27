# Implementation Plan: Safely Edit Open GDScript

**Branch**: `spec/002-edit-open-gdscript` | **Feature identifier**: `002-edit-open-gdscript` | **Date**: 2026-09-27 | **Spec**: [spec.md](spec.md)

**Input**: `specs/002-edit-open-gdscript/spec.md`

**Status: Planning blocked — native integration remains a promising candidate; the minimum sufficient design is unresolved (State B).** This is a research/gate record, not a completed or approved implementation design. Phase 1 design has not started. Do not derive tasks or implement against this document.

**Evidence boundary:** completed native probes and pinned-source/API findings remain valid, including their negative results. A later native probe did not complete because the available coding environment refused it on cybersecurity-policy grounds. No bypass, evasion or equivalent rephrased retry was attempted. That interruption is a tooling limitation, not technical evidence that native integration failed. The [requirements audit](research.md#10-phase-1-concurrency-boundary-audit-and-corrected-planning-decision) supports guarded single-editor mutation, not arbitrary same-inode writer atomic exclusion. The remaining design prerequisites are guarded target-document finalization and source-attributed native validation; no runtime probes or implementation are resumed here.

## Summary

The required capability remains one revision-guarded, native edit of an existing open standalone GDScript, preserving human work, independently verifying D/R/B convergence and parse state, preserving real native Undo/Redo, and requiring A–E and applicable durability evidence. The completed observation feature remains read-only and unchanged.

[Research and actual feasibility evidence](research.md) establish useful native mechanisms and specific prototype/API limitations; the incomplete investigation does not disprove the native direction:

- In the earlier ResourceSaver-backed fixture, CodeEdit complex operations preserved prior native history; Undo, intervening save/bookkeeping, Redo and close/reopen worked. That evidence does not establish durability of the later descriptor-write candidate.
- `ResourceSaver.save()` wrote the intended source after native R synchronization but did not mark the buffer saved. Explicit saved-version bookkeeping was required in the probe; that bookkeeping must itself be guarded in a real transaction.
- `ScriptEditor.save_all_scripts()` saved the unrelated dirty fixture document. It cannot meet the target-only requirement.
- After a controlled project-directory replacement, the path-based ResourceSaver candidate followed a symlink and overwrote an owned outside-project sentinel while returning `OK`. A read-time pathname check and post-write observation do not bind/prevent that write.
- A valid target returned `OK` from `Script.reload(true)` in the fixture. A general source-attributed parse/diagnostic path and its side effects are not established by that single result.
- The [focused continuation](research.md#8-focused-ab-continuation-and-decision) rejected omitted-path `ResourceSaver.save(script)` too: the same already-loaded Resource overwrote a replacement file at its stored path while reporting `OK`.
- Actual language/editor bindings expose no directly callable source-validator result for the current GDScript addon. Eight new top-level reload attempts and a nested call demonstrated error-code ambiguity, stale validity predicates, static-initialization effects with `keep_state=true`, and reentrant `OK` without a parse.
- The [native continuation](research.md#9-native-integration-continuation-c1c5) compiled and loaded an ABI-only GDExtension in the exact GUI editor. Existing Script/CodeEdit access, native complex edits and descriptor-bound persistence were demonstrated without internal C++ linking, a new permanent dependency or an external writer.
- Controlled leaf/parent replacement, rename and unlink after the final checks did not redirect the descriptor write. Namespace loss returned non-success; replacement/outside sentinels were preserved. A newer same-inode revision written after the last check was overwritten: that negative result demonstrates no arbitrary-writer compare-and-write guarantee, not a newly required Phase 1 coordination mechanism. Known interference still prevents a verified-success claim.
- In a fresh editor, descriptor write plus CodeEdit saved tagging left native document timestamp bookkeeping stale. Actual script-only Cmd+Alt+S after Undo opened the external-change dialog instead of synchronizing D with restored R/B; unrelated dirty work remained untouched. The missing document-level finalization is distinct from Resource/CodeEdit source equality.
- Source inspection identified `GDScriptLanguage::validate` as the genuine source/path parser-plus-analyzer result. It is not a supported built-in GDExtension call, and dependency/cache/resource effects cannot be erased by adding a binding. No direct validator runtime invocation or engine patch was performed.

**Decision: State B — design still unresolved.** Standard GDExtension plus narrow guarded finalization and validation exposure remains a promising candidate. A custom module is not shown necessary. The specification now clarifies the Phase 1 concurrency boundary without weakening its dirty-state, identity, confinement, stale-refusal or verification requirements. `/speckit.plan` is not resumed because the two integration capabilities still need sufficiently concrete semantics, not because a tooling-policy refusal proved them infeasible.

The plan does not substitute a refusal-only feature, weaken confinement or human-work protection, or treat a proposed native API as implemented/proven. It distinguishes positive runtime behavior, negative runtime behavior, source/API findings, unresolved design questions and the policy interruption in [research §10.1](research.md#101-evidence-categories-and-interruption).

## Technical Context

**Language/Version**: Existing Rust **1.98.1**, edition **2021**, and GDScript are unchanged. The native candidate is Godot **4.7.2.stable.official.ed1daf0bf**, full hash `ed1daf0bf001b61586d9930840f2f1394092c079`. Disposable research additionally used C++17, Apple clang **21.0.0** (`clang-2100.3.34.2`), macOS SDK **27.0**, the exact engine-generated C ABI headers, and Python standard-library orchestration. No permanent native language/binding dependency or mutation support is selected by those experiments.

**Primary Dependencies**: Existing `serde =1.0.229` with derive, `serde_json =1.0.151`, `cap-std =4.0.3` without default features, and `ring =0.17.14` without default features and with `std`; Rust standard library and public Godot APIs. No dependency or lockfile change; no `godot-cpp` was needed for the earlier probe. **Unresolved capabilities:** guarded document finalization and a callable source-validator result with settled invocation/effect semantics. The descriptor operation's atomicity limitation is documented, not converted into a current CAS/lease dependency.

**Storage**: Existing owner-private source-free routing metadata and bounded request-local observations remain unchanged. No database, journal, retained project-source cache, or recovery service is selected. Revision/state lifetime decisions depend on the real mutation boundary; an existing observation ID/hash is not a write authority.

**Testing**: Existing deterministic Rust contract/boundary coverage and maintainer-operated real-Godot acceptance remain the baseline. Research exercised exact API generation, native compilation/loading, existing-object calls, controlled descriptor/identity/revision/failure cases and a fresh-editor script-only Save scenario. C4's additional validator findings are source/API inspection, not runtime validation proof. Limits and negative results are in [research.md §9](research.md#9-native-integration-continuation-c1c5). The product acceptance matrix was not implemented or run; no test suite or CI gate was added.

**Current audit validation**: repository/spec/roadmap analysis and documentation checks only. The earlier experiments are retained evidence, not rerun checks. The policy-blocked probe remains uncompleted; no equivalent retry or new runtime/API probe is performed. Spec Kit path resolution, links/anchors, Markdown consistency, whitespace and complete intended/staged diff review apply to this documentation change.

**Target Platform**: Existing macOS arm64 candidate. Current research host reports macOS **26.6.2**, build **25G83**, Apple M2; existing ordinary native CI remains separate from real-editor acceptance. No broader platform/version claim and no new provider/runner prerequisite.

**Project Type**: Existing reusable Rust library/local caller and Godot EditorPlugin. Preserve `caller → protocol-independent core → Godot integration`; no MCP surface or second mutation core. A future edit caller/wire result must be distinct from observation v1, but its concrete contract is not finalized before resolving the application/persistence boundary.

**Performance Goals**: The specification requires a terminal edit result within **ten seconds** in controlled acceptance. Existing observations retain **five seconds**. These are future acceptance obligations, not measurements from the disposable multi-step controller. Worker termination/disconnection must not imply that an editor mutation could not apply later.

**Constraints**: Independent D/R/B and document-specific dirty evidence; independently attributable parse state; stale-write protection at the real mutation boundary; no target guessing, hidden opening/loading, force/merge, automatic mutation retry, unsafe rollback, out-of-project writes, source logging, or gameplay authority. **Blocked evidence route:** reload remains an effectful attempt rather than unconditional fresh parse success. The actual source validator needs supported exposure, source/identity/interval attribution and truthful dependency/effect semantics; a binding alone does not prove the complete safety boundary.

**Scale/Scope**: One already-open standalone project GDScript per attempt. Required: fresh revision/conflict checks, protection of unsaved editor work, exact target/session identity, non-redirectable persistence and independent invalidation-aware verification. Not claimed: atomic serialization against an arbitrary non-cooperating same-inode writer during the final filesystem commit interval. Stronger competing-actor coordination belongs to later scope if specified; even Phase 9 does not currently promise universal exclusion of such writers. No generalized transactions, batch orchestration, MCP or platform expansion is introduced.

## Constitution Check

**Initial gate, before Phase 0: PASS for research only.** The specification preserves Principles I–XIII and explicitly exposes native mutation/persistence/parse, confinement, and interruption unknowns. No safety exception was presumed.

**Post-research gate: BLOCKED — incomplete design/evidence, not a failed native direction.** Earlier unsafe path saves and Save All remain rejected. Descriptor-bound persistence retains its demonstrated positive behavior. The ordinary Undo → Save failure identifies incomplete native saved-state bookkeeping; the genuine validator still needs a supported result exposure with defined effects. The policy interruption supplies no technical verdict about those proposed capabilities. The earlier planning demand for arbitrary same-inode exclusion is withdrawn after the constitutional/roadmap/spec audit; current stale/conflict and non-success protections remain mandatory.

**Post-Phase-1 gate: NOT REACHED.** Phase 1 requires completed research without unresolved material decisions. There is no completed data model, mutation contract, quickstart, implementation plan approval, release, or support claim.

| Constitutional gate | Evidence / current disposition |
|---|---|
| I — Independent D/R/B | Independent surfaces and attributable clean/synchronized state remain required. The same-inode experiment shows that later equality cannot attest every intermediate external write; known/detected invalidation still prevents success. No global atomic-history claim is made. |
| II — Preserve human work | Preserve unsaved editor work, refresh stale/conflict checks at the relevant boundary and refuse known invalidation. Target-only native writes preserved the unrelated dirty fixture; Save All remains rejected. Constitution II's optimistic checks do not mandate arbitrary external-writer exclusion. |
| III — Native transactions/history | Native complex edits, Undo/Redo and earlier history have bounded positive evidence. CodeEdit tagging omits document-level saved bookkeeping; ordinary script Save after Undo failed without reconciliation. No alternate history stack was introduced. |
| IV — Independent verification | ResourceSaver `OK`, successful native `pwrite`/`fsync`, saved tagging and a reload result are not mutation success. Source/identity/failure witnesses and reached stages remain distinct. |
| V — Confinement/least privilege | Earlier path-save candidates failed. Descriptor-bound writes prevented the examined leaf/parent substitutions from redirecting writes and did not treat namespace loss as success. Those protections remain required. The recorded `EPERM` lease result is not a Phase 1 blocker or permission to seek a new privilege. |
| VI — Real-editor gates | A–E and applicable durability remain mandatory. The disposable research sequence is not the product acceptance suite and does not mark any mutation gate complete. |
| VII — Protocol independence | Existing durable ownership is retained. No core/caller/addon changes or parallel safety path were introduced. |
| VIII — Tooling isolation | No product addon/runtime/export changes. Any later mutation implementation must re-prove its affected export boundary. |
| IX — Small surface | One intended edit capability; no extra product operation is introduced to hide the persistence problem. |
| X — Truthful diagnostics | Preserve actual negative results and unknowns. A policy-blocked probe is not a Godot failure, success or unsupported-capability result. No success, rollback, no-application, parse-validity or support claim is invented. |
| XI — Independent implementation | Public Godot documentation, exact runtime metadata, existing repository source and independently written disposable probes were used; no third-party implementation was copied. |
| XII — Correctness first | **BLOCKED:** the finalization and validation integration semantics remain unresolved; they are not assumed safe or declared impossible. No implementation begins on an incomplete design. |
| XIII — Justified complexity | Native code may reduce external composition; C++ is not disqualifying. Do not require locks/leases, a filesystem framework or other later-phase coordination to satisfy an unpromised arbitrary-writer atomicity guarantee. |
| Compatibility/workflow | Observation v1 and its deadlines remain unchanged. One explicit specification scope clarification is recorded in research §10; no functional requirement, acceptance scenario, product schema or task is removed or added. |

### Conditions for resuming Phase 1

1. Specify guarded target-document finalization: the selected editor/document/source/version guards, persistence-result handoff, native modified-time/saved-version/Resource bookkeeping, callback ordering, and failure/identity-change behavior. Retain the demonstrated object-bound writer, fresh revision checks and truthful non-success on invalidation. Do not add arbitrary external-writer CAS as a prerequisite.
2. Specify narrow exposure of `GDScriptLanguage::validate`: selected-editor/project/thread context, exact source/revision/interval attribution, root/dependency diagnostics and effect/refusal semantics consistent with FR-009/FR-010/FR-020. Source inspection identifies the capability but does not complete those decisions.
3. Define application uncertainty, permanent pre-application discard, deadline/disconnection and no-retry behavior around those capabilities using the existing core. Preserve read-only compatibility and privacy.
4. Record a proportionate implementation/verification approach for the actual Phase 1 guarantees and re-evaluate constitutional compliance. Necessary unavailable runtime experiments remain unresolved; do not bypass the tooling filter or claim they passed. Full A–E release acceptance remains required for completion, not as a prerequisite to drafting an otherwise concrete plan.

The [audit](research.md#102-authoritative-evidence-and-interpretation) selects Interpretation A and explains the narrow [specification clarification](spec.md#phase-1-concurrency-boundary). The earlier buffer-only/manual-reconciliation/parse-omission alternatives remain rejected. Planning is blocked by unresolved finalization/validation design, not by a disproven native architecture or an arbitrary-writer atomicity requirement. No constitutional amendment or blanket assumption of a quiet filesystem is made.

## Project Structure

### Documentation (this feature)

```text
specs/002-edit-open-gdscript/
├── spec.md                         # Existing specification with scoped concurrency clarification
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

**Structure decision:** Preserve the existing two component boundaries. No product source, permanent tests or new source layout are changed. Earlier probes ran only in disposable directories and were removed; this continuation performs documentation/design analysis only.

## Complexity Tracking

No constitutional exception or new infrastructure is approved. Current needs are stale/dirty single-editor conflict protection, non-redirectable writes, complete native saved-state bookkeeping and attributed parse evidence. The recorded same-inode overwrite remains an atomicity limitation; requiring its prevention as global serialization was an overbroad reading corrected by the audit. The tool-policy interruption is a separate evidence limit. C++ and an editor build step are assessed by total cost, not language purity.

| Question | Recorded assessment |
|---|---|
| Current requirement/failure | Complete one guarded open-document edit without losing unsaved editor work, redirecting writes, omitting native saved bookkeeping or asserting an unobserved parse result. Do not infer arbitrary-writer serialization from “protect newer work.” |
| Simplest credible alternative | Standard GDExtension with the demonstrated descriptor-bound writer, native history and narrow target-finalization/validation exposure. Reuse the current core rather than compose external writers/validators or add a second safety model. |
| Why existing mechanisms are insufficient | Existing path savers have demonstrated redirection failures; public CodeEdit tagging omits document saved state, and the native validator is unbound. The two proposed integrations are unresolved, not technically disproven by the interruption. |
| Ongoing cost | Native descriptor/object lifetime, partial-write handling, compiler/ABI/export and editor-version maintenance; narrow engine exposure adds editor-build/API upkeep. Broader modules or coordination systems add costs not justified by the audited current scope. |
| Why introduce it now? | Only to close the identified current finalization/validation and Phase 1 safety requirements with fewer moving parts. No daemon, lock manager, journal, generic concurrency framework, new approval or CI infrastructure is introduced to close an unpromised atomicity guarantee. |

No planning resumption or downstream artifact generation follows from this audit. It does not invoke `/speckit.plan` or bypass a hook. These artifacts authorize no implementation, engine patch, task generation, filter bypass or merge. The maintainer authorized this scoped requirements/evidence correction and publication on `spec/002-edit-open-gdscript` in existing PR #24 only.
