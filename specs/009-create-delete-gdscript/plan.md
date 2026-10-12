# Implementation Plan: Create and Delete Project GDScripts Through the Trusted Workflow

**Branch:** `spec/009-create-delete-gdscript` | **Date:** 2026-10-12 | **Spec:** [spec.md](spec.md)

**Input:** Clarified Feature 009 requirements and the four accepted C1–C3 answers.

**Status:** Phase 0 research and Phase 1 design complete. No tasks, granularity review, consistency analysis or product implementation have started. This plan is a design deliverable, not implementation approval or create/delete support evidence.

## Summary

Extend the current local MCP Schema 2 catalog with explicit `create_script` and `delete_script`. Keep adapter → protocol-independent core → Godot integration. Create validates complete supplied source in the existing private stock validator, acquires an absent file exclusively, and independently verifies exact D with absent B and absent or newly coherent R. Delete matches the existing fresh `read_script` revision, supports all three eligible clean lifecycle profiles, explicitly removes its admitted document and Resource path association, then unlinks only the guarded original target. Independent postconditions and truthful partial/unknown effects determine every outcome.

The [research](research.md) resolves filesystem primitives, public editor mechanics, validator admission and integration compatibility. The owned stock-guest experiment supports those mechanics; it does not establish guarded implementation, public clients, ten-second timing or product acceptance. No generic file manager, lifecycle tool, dependency repair, rollback, trash or delete Undo is selected.

## Technical Context

- **Languages/toolchain:** Existing Rust 1.98.1 / edition 2021, C++17 public GDExtension boundary, thin GDScript EditorPlugin, Python 3.10+ evidence tooling. No new MSRV or toolchain claim.
- **Dependencies:** Retain locked serde 1.0.229, serde_json 1.0.151, cap-std 4.0.3, ring 0.17.14, rmcp 3.5.0 and Tokio 1.53.1. No new dependency, SDK, binding generator or engine patch.
- **Platform:** Official Godot `4.7.2.stable.official.ed1daf0bf`, macOS 26.6.2 (25G83) arm64, recorded four-vCPU/6-GiB profile; Codex CLI 0.153.4 and OMP 18.5.1. No expanded support.
- **Transport/compatibility:** Local stdio MCP 2025-11-25; additive public Schema 2 and unchanged `sr1`/existing local contracts/package version. Internal bridge v7/native revision 5 migrate together; no old-peer fallback.
- **Storage:** Existing owner-private source-free registry and bounded request-local evidence. Project effects are the specified `.gd` existence/document/association changes. Existing `.gd.uid` files are not deleted; ordinary engine bookkeeping/reference limitations are disclosed.
- **Bounds:** [Data model](data-model.md#ownership-and-bounds) retains source/context/frame limits and one admitted operation/no queue. Create/delete finish within ten seconds including delivered output under supported conditions, with the existing 9.5-second work cutoff; read/discover remain five seconds and edit ten.
- **Testing:** Existing Rust/process boundaries, native fixture mechanisms and dedicated graphical macOS VM. Actual coding-agent clients plus independent live-editor observations are required; no new runner/CI/approval topology.

## Selected architecture

### Trusted core and supervision

Add operation-owned creation and deletion request/evidence/reducer modules following the current `script_closed_edit` and `script_close` layouts, and corresponding `runner/create` / `runner/delete` workers using existing same-binary supervision. Share the existing `AttemptClock`, routing/confinement, slot ownership, cancellation and source-free diagnostics; do not route through `edit_script`, call public close, or build a generic mutation dispatcher/framework.

Delete reuses `runner/read` capture and unchanged `ScriptRevision` commitment. It then freezes a deletion-specific basis; an open target never becomes eligible by silently switching to a new closed basis. Create uses a dedicated absent-target basis, not a read revision or an existing-empty-file fiction. Domain records have no MCP DTOs or JSON-RPC IDs.

The adapter adds two strict input/output projections and appends two catalog entries. All mutating-operation classifications in handler, transport, cancellation and post-dispatch error projection must include them. Existing discover/read/exact-edit fields and outcomes stay unchanged, including Feature 008's empty-source distinctions.

### Source validation

Extend the existing stock-validator capture with an explicit proposed-absent-root branch and create-preflight purpose. Build its context from actual selected project/effective/source-profile observations without allocating a live Script or CodeEdit. Stage source only in the existing private child project; refuse invalid/unavailable/unsupported source before creating anything in the live project. Recheck absence/context after preflight and again at native entry.

After creation, independently capture actual D, passive R/B and current context, then run actual-source post-validation under the same deadline. Delete has no replacement-source or post-deletion target parse; retain its existing read eligibility and the close path's protected-source/context validation. Keep the accepted validator threat boundary, bounded staging, source-free logs, per-URI fences and owned-child cleanup. No live parser/reload or alternative validator is introduced.

### Native existence changes

Use `script_create.cpp` and `script_delete.cpp` under the existing native `Session` owner and addon operation slot. `document_guard` owns retained parent/leaf identity and no-follow filesystem primitives. `editor_context` owns only genuinely shared passive source/effective-context acquisition. Extract the close protection/continuation helper from the current close implementation where needed by both close and open-delete; do not copy its scanner or invoke a second close transaction.

Creation's first effect is exclusive `openat`; descriptor-bound exact writes/flush follow. Deletion's effects are guarded native document removal when open, exact retained Resource `set_path("")` when present, and parent-relative identity-checked `unlinkat`. Each stage rechecks phase-appropriate state; a failed partial operation stops rather than compensating. No `ResourceSaver`, forced load/reload, explicit filesystem scan, editor Save, source clearing, pathname overwrite or UID cleanup is part of either operation.

The parent performs fresh filesystem and editor acquisition after native receipts, with context and preservation rechecks. Native facts alone are never a success verdict. Retained Resource instances may survive deletion with no target path; references and active-game state are not repaired. The [integration contract](contracts/existence-transactions.md) owns stage order, private exchange and invalidation semantics.

## Constitution Check

**Initial gate:** Passed for scoped research, as recorded before R1–R3 investigation. No feasibility or product acceptance was presumed.

**Post-design gate: PASS for the selected design; no constitutional exception requested.** This is mutation-semantic/security-sensitive planning. Implementation must still prove every required positive profile and safety property; the mechanics experiment is not a waiver.

| Principles | Design disposition |
| --- | --- |
| I, IV — coherence and independent verification | Explicit D/R/B presence versus unavailable facts; exact-source creation and deletion disappearance predicates; receipts, inventory omission and runtime agreement alone cannot establish success. Ordinary later durability remains required. |
| II — human work and stale state | Existing `sr1` fresh comparison; native dirty/equal-text-dirty/identity/context guards; protected remaining documents/history; no save/discard/force/repair fallback or cleanup of replacement targets. |
| III — native transactions | One live-editor-aware native owner; public document-removal and Resource-path APIs where available. Godot exposes no equivalent exclusive/identity-guarded file primitive, so descriptor operations remain inside the same transaction and verification boundary, never a raw-file adapter fallback. No Undo claim. |
| V, X, XII — confinement, effects and correctness | Existing authentication/no-follow/permission/disclosure controls, explicit absent-parent/file binding, original deadline, typed refusal/partial/unknown outcomes and safe next actions. Missing safety evidence refuses; required positives cannot be replaced with universal refusal. |
| VI, VIII — real editor and export isolation | New existence, interruption and composed/client evidence in the owned VM; A–E coverage by new/affected/reusable evidence; new addon/native owners remain editor-only and export exclusion is tested. No release campaign inferred. |
| VII, IX — architecture and small surface | Core owns semantics; MCP owns schemas; Godot owns object effects. Exactly five public tools with concise descriptions. No Save/open/close/history mode or generic CRUD. |
| XI — independent implementation | Existing code/acceptance, pinned public Godot/platform sources and observable owned mechanics; no copied third-party implementation or new dependency. |
| XIII — proportionate complexity | Each necessary addition is justified below. No new isolation service, authorization ceremony, framework, persistent token store or CI boundary. Existing supported threat/concurrency limits remain explicit. |

Compatibility is additive publicly and a clean matched-family cutover privately. Features 001–008 and Phase 1–3 completion remain truthful history; Phase 4, Feature 006, another feature and release work stay unselected.

Independent native-safety and public-contract design reviews found one outcome-model inconsistency: exclusive acquisition can enter and return `EEXIST` with provably no effect. The model and integration contract now retain `failed_no_effect`, permit `refused/not_applied` only when every effect is accounted for, and retain uncertainty when the return is lost; the validation guide includes both cases. No other concrete blocking finding was reported. This proportionate planning review is not `/speckit.analyze` or product acceptance.

## Project Structure

### Design artifacts

- [research.md](research.md): resolved R1–R3, primary sources, mechanics evidence and limitations.
- [data-model.md](data-model.md): intent/basis, authority evidence, state transitions and deterministic outcomes.
- [contracts/mcp-interface.md](contracts/mcp-interface.md): five-tool additive Schema 2 inputs/results/carrier/compatibility.
- [contracts/existence-transactions.md](contracts/existence-transactions.md): trusted/native boundaries, private exchange, validation and positive postconditions.
- [quickstart.md](quickstart.md): runnable validation workflow, requirement coverage and relevant-input reuse rules.

No `tasks.md` or `analysis.md` is produced by planning.

### Implementation ownership selected for later tasks

| Location | Coherent responsibility |
| --- | --- |
| `mcp-server/src/mcp/{handler,schema,output,transport}.rs` and operation-owned output submodules | Strict new DTO/catalog/projection and correct mutating-dispatch/error ownership; no Godot safety decisions. |
| `mcp-server/src/script_create/`, `script_delete/` | Checked intent/basis, evidence and terminal reduction; private visibility unless a current consumer requires otherwise. |
| `mcp-server/src/runner/create/`, `runner/delete/` and existing binary wiring | Same-binary supervised orchestration, current guards, one-shot exchanges, independent post-acquisition. No new user CLI. |
| `mcp-server/src/runner/stock_validation/`, `project_fs.rs`, `project_fs/validation.rs` | Explicit proposed absent root and confined actual-source/absence acquisition; existing validators/callers retain their contracts. |
| `godot-addon/addons/godot_agent_kit/` bridge plus operation-owned create/delete transports | Authenticated fixed-stage dispatch and original response lifetime in the existing slot. No source-authority decisions in forwarding code. |
| `godot-addon/native/` create/delete owners, shared document/context/close-protection helpers, registration/builder | Public Godot effects, exact object/descriptor guards and bounded continuation evidence; matched revision-5 provenance. |
| Existing Rust process tests and `godot-addon/tests/` MCP/VM fixtures | Consumer-visible new behavior and selected regression evidence; no new test framework. |

Names describe script creation/deletion or durable shared responsibility, not a feature/task number. No source directories or scaffolds are created in this design stage.

## Implementation-shape constraints

Keep request/evidence/reduction separate from supervision and native calls where they are already separate responsibilities. The existing large close module must not gain an entire deletion implementation. A private helper shared by the two present close consumers may own protection/continuation state, but it is not a reusable transaction framework. Review cohesion when materially changed modules approach 800–1000 lines; size alone is not a split requirement.

Use one phase enum per operation, with independent factual effect progress where necessary. Remove speculative variants/exports, avoid safety-path `unwrap`/`expect`, and expose only worker symbols genuinely needed by the existing executable. Reuse captured digest/length and owned source buffers instead of serializing/copying source for another commitment. Postcondition reacquisition is intentional independent work, not a cached-copy optimization opportunity.

## Complexity Tracking

| Necessary addition | Present requirement/failure and insufficient existing mechanism | Simplest alternative and why rejected | Ongoing cost and why justified now |
| --- | --- | --- | --- |
| Absent-leaf parent binding and exclusive create | FR-003/004: current read/edit assumes an existing file; a placeholder mutates before validation and an ordinary writer can overwrite. | Raw write or edit-empty fallback violates existence semantics and no-overwrite. | One guarded absent-root branch and explicit new-file descriptor lifetime; directly enables requested creation. |
| Creation validator context/root variant | FR-004/005: current validator captures a disk root; nonexistent source has no real inode/Script/buffer. | Fake empty file/IDs or live load to validate would fabricate evidence or change lifecycle. | A bounded typed projection and new purpose in the existing validator, with affected-root tests; no new validator/process architecture. |
| Delete effect owner and reused close protection | FR-007/008: unlink alone leaves B/R; public close then delete changes the admitted basis and loses partial-effect ownership. | Closed-only deletion fails the accepted three-profile requirement; duplicating close machinery increases safety maintenance. | One delete state machine and narrow present-consumer helper extraction; preserves explicit effects, original lifetime and remaining human work. |
| Target UID companion admission | FR-002/017: source-established ordinary Godot bookkeeping can follow the new script's companion path; source-leaf guards alone do not cover a pre-existing unsafe companion. | Ignore a known companion link or add generic metadata cleanup. | Reuse one no-follow owner/type check, with focused unsafe-companion controls; no new mutation surface or UID framework. |
| Matched v7/revision-5 family and two public projections | FR-010/011/015: old private peers cannot perform/prove existence transitions; current adapter only classifies edit as mutating. | Reuse old handshake semantics or hide modes in edit risks unadvertised behavior and false not-applied errors. | Update current producer/consumer family and bounded operation-owned DTOs together; public Schema 2 and old tool contracts stay stable. |
| New focused VM/client evidence | FR-019/020: inherited tests do not exercise creation/deletion or their composition. | Historical suite replay cannot prove new behavior; mocks cannot prove B/R or real model use. | Extend existing owned fixtures and independently runnable groups, with relevant-input reuse; no new provider, runner or approval gate. |

## Verification and next stage

The [quickstart](quickstart.md) maps all 23 scenarios, FR-001–FR-021 and SC-001–SC-008. New existence semantics, both-client use, sequential lifetimes and applicable durability need new evidence. Shared private-family changes require affected authentication/ABI/owner regressions across reachable existing callers; they do not by themselves require complete historical GUI suites. Reuse unchanged evidence only after reviewing actual execution/native/fixture/environment inputs under [TEST_POLICY.md](../../TEST_POLICY.md).

This planning stage validates documents, links/anchors/fences, illustrative snippet syntax, intended/staged diff and publication. The Phase 0 mechanics run is separately recorded research, not a product suite. No Rust test/build/native campaign is required by the documentation delta. No extension hooks are registered in `.specify/extensions.yml`.

Publish this stage to the existing draft design PR, then stop. A separately requested task stage must derive coherent PR-sized capability increments, bundle their direct tests/docs, perform the required granularity review, then obtain current consistency analysis before one-task/one-PR implementation. This plan does not derive task IDs, authorize stacked work, merge or begin implementation.
