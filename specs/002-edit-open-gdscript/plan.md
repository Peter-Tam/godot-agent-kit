# Implementation Plan: Safely Edit Open GDScript

**Branch**: `spec/002-edit-open-gdscript` | **Feature identifier**: `002-edit-open-gdscript` | **Date**: 2026-09-27 | **Spec**: [spec.md](spec.md)

**Input**: `specs/002-edit-open-gdscript/spec.md`

**Status: ERROR — planning blocked at the Phase 0 native-correctness gate.** This is a research/gate record, not a completed or approved implementation design. Phase 1 design has not started. Do not derive tasks or implement against this document.

**Resume prerequisite:** A/B and the native C1–C5 continuation have been investigated, but no complete safe route is established. **A:** standard GDExtension can perform descriptor-bound writes without pathname redirection, but the examined primitive can overwrite a competing same-inode revision and does not complete native document saved-state bookkeeping. **B:** the existing parser/analyzer source validator is unexposed through the inspected extension ABI; a narrow engine API must expose its result with explicit project/dependency/effect semantics. Planning may resume only when both required guarantees, including revision protection and editor durability, are established without weakening safety. No task generation or downstream design artifacts are authorized.

## Summary

The required capability remains one revision-guarded, native edit of an existing open standalone GDScript, preserving human work, independently verifying D/R/B convergence and parse state, preserving real native Undo/Redo, and requiring A–E and applicable durability evidence. The completed observation feature remains read-only and unchanged.

[Research and actual feasibility evidence](research.md) found useful native text/history and descriptor-write mechanisms, but not a complete safe persistence/validation route:

- In the earlier ResourceSaver-backed fixture, CodeEdit complex operations preserved prior native history; Undo, intervening save/bookkeeping, Redo and close/reopen worked. That evidence does not establish durability of the later descriptor-write candidate.
- `ResourceSaver.save()` wrote the intended source after native R synchronization but did not mark the buffer saved. Explicit saved-version bookkeeping was required in the probe; that bookkeeping must itself be guarded in a real transaction.
- `ScriptEditor.save_all_scripts()` saved the unrelated dirty fixture document. It cannot meet the target-only requirement.
- After a controlled project-directory replacement, the path-based ResourceSaver candidate followed a symlink and overwrote an owned outside-project sentinel while returning `OK`. A read-time pathname check and post-write observation do not bind/prevent that write.
- A valid target returned `OK` from `Script.reload(true)` in the fixture. A general source-attributed parse/diagnostic path and its side effects are not established by that single result.
- The [focused continuation](research.md#8-focused-ab-continuation-and-decision) rejected omitted-path `ResourceSaver.save(script)` too: the same already-loaded Resource overwrote a replacement file at its stored path while reporting `OK`.
- Actual language/editor bindings expose no directly callable source-validator result for the current GDScript addon. Eight new top-level reload attempts and a nested call demonstrated error-code ambiguity, stale validity predicates, static-initialization effects with `keep_state=true`, and reentrant `OK` without a parse.
- The [native continuation](research.md#9-native-integration-continuation-c1c5) compiled and loaded an ABI-only GDExtension in the exact GUI editor. Existing Script/CodeEdit access, native complex edits and descriptor-bound persistence were demonstrated without internal C++ linking, a new permanent dependency or an external writer.
- Controlled leaf/parent replacement, rename and unlink after the final checks did not redirect the descriptor write. Namespace loss returned non-success; replacement/outside sentinels were preserved. However, a newer same-inode revision written after the last check was overwritten: the primitive is not compare-and-write.
- In a fresh editor, descriptor write plus CodeEdit saved tagging left native document timestamp bookkeeping stale. Actual script-only Cmd+Alt+S after Undo opened the external-change dialog instead of synchronizing D with restored R/B; unrelated dirty work remained untouched. The missing document-level finalization is distinct from Resource/CodeEdit source equality.
- Source inspection identified `GDScriptLanguage::validate` as the genuine source/path parser-plus-analyzer result. It is not a supported built-in GDExtension call, and dependency/cache/resource effects cannot be erased by adding a binding. No direct validator runtime invocation or engine patch was performed.

**Decision: Outcome 3D — no sufficient safe native route established.** Standard GDExtension is useful and was not rejected for using C++ or a build step. GDExtension plus narrow validation/finalization API exposure is the next candidate, not a proven minimum sufficient level; a custom module is neither shown necessary nor a cure for the disk revision race. The specification is unchanged, and `/speckit.plan` is not resumed.

The plan does not substitute a refusal-only feature, weaken confinement or human-work protection, or name a hypothetical safe-save abstraction as though it were implemented/proven. The missing persistence design is a prerequisite to finalizing its transaction model and contracts.

## Technical Context

**Language/Version**: Existing Rust **1.98.1**, edition **2021**, and GDScript are unchanged. The native candidate is Godot **4.7.2.stable.official.ed1daf0bf**, full hash `ed1daf0bf001b61586d9930840f2f1394092c079`. Disposable research additionally used C++17, Apple clang **21.0.0** (`clang-2100.3.34.2`), macOS SDK **27.0**, the exact engine-generated C ABI headers, and Python standard-library orchestration. No permanent native language/binding dependency or mutation support is selected by those experiments.

**Primary Dependencies**: Existing `serde =1.0.229` with derive, `serde_json =1.0.151`, `cap-std =4.0.3` without default features, and `ring =0.17.14` without default features and with `std`; Rust standard library and public Godot APIs. No dependency or lockfile change; no `godot-cpp` was needed for the probe. **Blocked capability:** the demonstrated descriptor operation supplies object binding, not complete expected-revision enforcement or editor finalization. A hypothetical narrow engine API must not be represented as already available.

**Storage**: Existing owner-private source-free routing metadata and bounded request-local observations remain unchanged. No database, journal, retained project-source cache, or recovery service is selected. Revision/state lifetime decisions depend on the real mutation boundary; an existing observation ID/hash is not a write authority.

**Testing**: Existing deterministic Rust contract/boundary coverage and maintainer-operated real-Godot acceptance remain the baseline. Research exercised exact API generation, native compilation/loading, existing-object calls, controlled descriptor/identity/revision/failure cases and a fresh-editor script-only Save scenario. C4's additional validator findings are source/API inspection, not runtime validation proof. Limits and negative results are in [research.md §9](research.md#9-native-integration-continuation-c1c5). The product acceptance matrix was not implemented or run; no test suite or CI gate was added.

**Target Platform**: Existing macOS arm64 candidate. Current research host reports macOS **26.6.2**, build **25G83**, Apple M2; existing ordinary native CI remains separate from real-editor acceptance. No broader platform/version claim and no new provider/runner prerequisite.

**Project Type**: Existing reusable Rust library/local caller and Godot EditorPlugin. Preserve `caller → protocol-independent core → Godot integration`; no MCP surface or second mutation core. A future edit caller/wire result must be distinct from observation v1, but its concrete contract is not finalized before resolving the application/persistence boundary.

**Performance Goals**: The specification requires a terminal edit result within **ten seconds** in controlled acceptance. Existing observations retain **five seconds**. These are future acceptance obligations, not measurements from the disposable multi-step controller. Worker termination/disconnection must not imply that an editor mutation could not apply later.

**Constraints**: Independent D/R/B and document-specific dirty evidence; independently attributable parse state; stale-write protection at the real mutation boundary; no target guessing, hidden opening/loading, force/merge, automatic mutation retry, unsafe rollback, out-of-project writes, source logging, or gameplay authority. **Blocked evidence route:** reload remains an effectful attempt rather than unconditional fresh parse success. The actual source validator needs supported exposure, source/identity/interval attribution and truthful dependency/effect semantics; a binding alone does not prove the complete safety boundary.

**Scale/Scope**: One already-open standalone project GDScript per attempt. No generalized transaction framework, script lifecycle/history-control commands, batch/concurrent-agent orchestration, runtime/debugger/LSP tools, scene/resource authoring, distribution work, or platform expansion.

## Constitution Check

**Initial gate, before Phase 0: PASS for research only.** The specification preserves Principles I–XIII and explicitly exposes native mutation/persistence/parse, confinement, and interruption unknowns. No safety exception was presumed.

**Post-research gate: FAIL — Outcome 3D.** The earlier path-save and Save All failures remain rejected. The native descriptor candidate prevents the examined pathname redirections but fails same-object competing-revision protection; direct CodeEdit saved tagging also fails ordinary Undo → Save synchronization because native document bookkeeping is missing. The built-in validator/result and guarded document finalization remain unexposed. Neither a narrow API proposal nor private module access proves the remaining mutation guarantees. This is not a completed plan or permission to defer safety to implementation.

**Post-Phase-1 gate: NOT REACHED.** Phase 1 requires completed research without unresolved material decisions. There is no completed data model, mutation contract, quickstart, implementation plan approval, release, or support claim.

| Constitutional gate | Evidence / current disposition |
|---|---|
| I — Independent D/R/B | Native R/B and separate controller D witnesses were used. The same-inode interleaving ended with equal D/R/B after losing a newer revision; equality alone cannot prove preservation or editor durability. |
| II — Preserve human work | Target-only native writes preserved the unrelated dirty fixture. Save All remains rejected; the competing same-inode overwrite fails newer-work protection. Revision enforcement is still a prerequisite. |
| III — Native transactions/history | Native complex edits, Undo/Redo and earlier history have bounded positive evidence. CodeEdit tagging omits document-level saved bookkeeping; ordinary script Save after Undo failed without reconciliation. No alternate history stack was introduced. |
| IV — Independent verification | ResourceSaver `OK`, successful native `pwrite`/`fsync`, saved tagging and a reload result are not mutation success. Source/identity/failure witnesses and reached stages remain distinct. |
| V — Confinement/least privilege | Earlier path-save candidates failed. Descriptor-bound writes prevented the examined leaf/parent substitutions from redirecting writes and refused namespace loss as success. The ordinary-user write-lease attempt returned `EPERM`; no privilege or permission workaround was introduced. Complete safe persistence remains unestablished. |
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

1. Establish a concrete persistence mechanism whose actual write remains bound to selected project/document authority and protects expected revisions/newer work. The descriptor prototype establishes object binding, but its same-inode race must be closed or equivalently protected—not hidden behind a cooperative-writer assumption, a later hash, or post-write equality.
2. Integrate it with guarded native text/history and complete document saved-state finalization. The exact `edited_file_data.last_modified_time` versus CodeEdit saved-version distinction is now established. Demonstrate ordinary Undo/Save/Redo and applicable durability, preserved unrelated dirty work and no false clean state on failure.
3. Expose the existing native source validator through a supported boundary with fresh target/source/revision/interval attribution, root/dependency diagnostics and explicit effects consistent with the specification. A proposed binding or custom-language validation hook is not present built-in access or runtime proof.
4. Define application uncertainty, permanent pre-application discard, timeout/disconnection and no-retry behavior against that actual boundary. Preserve existing read-only compatibility and privacy.
5. Record the Principle XIII assessment for any additional mechanism, then re-evaluate the constitutional gate. An alternative native/capability-scoped integration is not ruled out, but its missing guarantees cannot be replaced by a contract assertion.

These are technical blockers, not a request for blanket maintainer approval to bypass safeguards. The [earlier feature-boundary assessment](research.md#85-principle-xiii-and-feature-boundary-decision) still rejects buffer-only success, manual reconciliation, assumed path stability, omitted parse proof and a refusal-only cut-down. The [native comparison and decision](research.md#95-integration-level-comparison--c5) establish no sufficient integration level or independent safe narrowing. No constitutional amendment or specification change was applied.

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

No constitutional exception or new infrastructure is approved by this document. Current failures are specific: wrong-object path saves, unrelated saves, a same-object revision overwrite, incomplete native saved-state bookkeeping and a missing supported source-validation result. C++ and an editor build step are not rejected on language-purity grounds; total architecture cost and demonstrated protection govern the decision.

| Question | Recorded assessment |
|---|---|
| Current requirement/failure | One target-only edit must bind its write, protect newer revisions, complete native editor synchronization and provide attributed parse evidence. Research now distinguishes object binding from revision atomicity and document saved bookkeeping from a CodeEdit tag. |
| Simplest credible alternative | Public native text/history plus a small in-editor descriptor operation eliminates an external writer and associated cross-process synchronization. Narrow engine exposure for validation and guarded document finalization is the next candidate; not a selected complete design. |
| Why existing mechanisms are insufficient | Path savers re-resolve names; the native descriptor candidate still permits the demonstrated same-inode overwrite. Public CodeEdit tagging omits native document timestamp state, and the genuine validator is not bound. These gaps are not cured merely by changing the implementation language. |
| Ongoing cost | A native library adds descriptor/object lifetime, partial-write handling, compiler/SDK/ABI/export and exact-version regression obligations. Narrow engine APIs add a patched-editor build/distribution and API maintenance burden; a module adds private editor/GDScript coupling. None automatically supplies disk compare-and-write. |
| Why introduce it now? | A native boundary would be justified if it closes these current MUSTs more directly than external composition. The demonstrated partial protection justifies the research, not selecting an unproven library/module or starting a custom-engine distribution project. No broad locks, journal, rollback service, subprocess validator, new approval gate or CI topology is introduced. |

No planning resumption or downstream artifact generation follows from this record. The installed post-planning-hook check found no extension hook configuration; no hook was bypassed. These artifacts authorize no implementation, engine patch, task generation or merge. The maintainer authorized narrow A/B and C1–C5 research/publication on `spec/002-edit-open-gdscript` in existing PR #24 only.
