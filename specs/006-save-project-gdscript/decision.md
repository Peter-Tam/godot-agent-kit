# Feature 006 Product Decision: Do Not Add Standalone Save in Phase 1

**Date:** 2026-10-02

**Status:** Decision recorded — proposal assessed, not proceeding

**Reviewed baseline:** `a985527e90314711c2353915980998d1068ff3c3`

**Retained proposal:** [spec.md](spec.md)

**Delivery:** Existing [PR #59](https://github.com/Peter-Tam/godot-agent-kit/pull/59)

## Decision

**Standalone Save is not currently justified and will not proceed in Phase 1.**
No sufficiently valuable current coding-agent workflow requiring this additional
authority has been established. This is a product-value decision, not an
implementation-feasibility failure or a claim that Save could never be useful.

Feature 006 is retained as the historical/product-decision identity of the
considered proposal; `006` must not be reused for another feature. Its candidate
requirements and historical quality checklist remain useful evidence, but are not
active implementation requirements or authorization for planning, tasks or
implementation. This record is the authoritative disposition of that proposal.

The decision does not weaken dirty-human-work protections, make dirty buffers
unsupported observations, invalidate ordinary Save acceptance evidence, or imply
that Feature 002 persistence is incomplete. No Save capability was implemented.

## Existing completed capability

Feature 002 already provides a persisted agent edit:

- The target is an existing open script with current identity, independently
  agreeing disk source (**D**), loaded Script source (**R**) and editor buffer
  (**B**), document-attributed clean state and a valid expected edit basis.
- The agent supplies new intended source. Editing applies and persists that source
  within the operation; verified success independently establishes intended
  `D == R == B` and clean/saved/synchronized state.
- Dirty human work, including equal-text-but-dirty state, is deliberately refused,
  not automatically saved to make editing eligible.
- Successful edits survive ordinary Save and close/reopen, with accepted applicable
  reparse/rescan/runtime durability. Native Undo/Redo and earlier history remain
  usable; **Undo → ordinary Save → Redo → Save** was exercised.

The normative sources are [Feature 002 FR-003–FR-009 and FR-015–FR-017](../002-edit-open-gdscript/spec.md#functional-requirements),
its [plan summary](../002-edit-open-gdscript/plan.md#summary),
[caller verification/refusal contract](../002-edit-open-gdscript/contracts/edit-api.md#4-verification-and-refusal-rules)
and [native ownership/persistence contract](../002-edit-open-gdscript/contracts/native-integration.md#1-ownership-and-minimum-exposure).
Its [task records](../002-edit-open-gdscript/tasks.md) mark T001–T005 complete.

These are accepted behaviors, not just proposed requirements. The
[T005 coverage and observed outcomes](../002-edit-open-gdscript/quickstart.md#coverage-and-observed-outcomes)
record independent D/R/B and dirty/saved witnesses, actual native history, ordinary
Save/reopen/reparse/rescan/runtime and twenty successful fresh-basis edits with
three stale and three dirty refusals. Claims are limited to the
[recorded exact environment](../002-edit-open-gdscript/quickstart.md#exact-candidate-and-artifacts),
official Godot 4.7.2 on macOS 26.6.2 arm64. Feature 005's later
[cumulative evidence review](../005-close-project-gdscript/quickstart.md#t003-execution-decision-and-evidence-review)
and [T003 execution](../005-close-project-gdscript/quickstart.md#12-t003-cumulative-acceptance-2026-10-02)
retain and supplement the composed workflow's history and durability evidence.

Therefore, **“agent edit requires a later Save command” is not a current capability
gap.** These evidence references do not claim new runtime acceptance for Feature 006.

## Proposed capability increase

The rejected proposal would **persist an already-existing observed dirty buffer
without supplying a new edit**. Its principal additional authority is over
human-originated unsaved work:

- Existing edit: clean current target → agent supplies new intended source → edit
  applies and persists it → success verifies D/R/B and clean/saved state.
- Proposed standalone Save: existing dirty B already contains work → agent supplies
  no replacement source → agent makes that work durable.

The latter converts **temporary/unsaved buffer state into durable project state**.
It is distinct from both persistence inside edit success and ordinary Save used as
an acceptance/durability interaction. Neither of those existing meanings establishes
a need for standalone dirty-buffer automation.

## Candidate workflows reviewed

| Candidate workflow | Disposition |
| --- | --- |
| **1. Save after agent edit** | Already solved by edit persistence and independent verification. No later Save command is needed after verified success. Persistence remains owned by edit. |
| **2. Agent observes human dirty work** | Persistence of that buffer through the kit is technically unsupported. The current observe/report and edit/close refusal boundary is intentional protection, not an established defect. No material need for automation to commit the human's work was shown. |
| **3. Human native Undo/Redo leaves dirty state** | Real native history plus ordinary human Save already works. Acceptance exercised those interactions; it did not establish a need for an agent to persist the human-selected history state. |
| **4. A hypothetical future operation leaves pending work** | No current approved successful operation supplies this dependency. A hypothetical consumer cannot justify present generic authority. Any later producing operation must first establish and own its appropriate persistence semantics. |
| **5. User explicitly asks “save this buffer”** | A technically valid possible intent, whose currently established value is mainly delegating a normal editor Save. Explicit permission can make an action potentially permissible; it does not make the capability valuable enough to build now. No material current workflow constraint was established. |

Dirty states are real and reachable; their acceptance coverage is not a measure of
user demand or workflow frequency. No demonstrated frequency or material cost of
leaving Save/discard to the human supplied the missing product justification.
Observation still accepts dirty/divergent evidence under
[Feature 001's contract](../001-observe-gdscript-state/spec.md#functional-requirements).
Feature 005 deliberately refuses dirty closure and says independent Save
[is not a prerequisite](../005-close-project-gdscript/spec.md#scope-and-governance-alignment).
No implicit save-before-close behavior follows this review.

## Requirement provenance

The original [Phase 1 roadmap](../../ROADMAP.md#phase-1--live-editor-script-coherence)
states its purpose:

> Prove one reliable vertical slice: safe GDScript editing in a live Godot
> Editor, with human unsaved work protected.

Its capability areas mention “coherent mutation; Save; close/reopen; and real Godot
Undo/Redo.” Its exit language is:

> A–E pass in real Godot, with Save/reparse/rescan/runtime durability checked where
> applicable. Accepted edits reported successful remain coherent, human work is protected,
> and limitations/non-success outcomes are documented. No reconciliation of agent-created
> editor divergence is required.

This establishes Save behavior/durability within editor coherence, not by itself a
standalone agent-callable Save command. Feature 002's
[native-history story](../002-edit-open-gdscript/spec.md#user-story-4---reverse-and-reapply-through-native-history-priority-p1)
explicitly calls ordinary Save and Undo/Redo developer/editor interactions used to
prove the edit's behavior, **not new general-purpose automation commands**.
[Phase 3](../../ROADMAP.md#phase-3--mcp-adapter-mvp) separately names save in its
adapter capability areas; that later-phase direction does not create a Phase 1 gate.

The review traced the requirement-fossilization chain:

1. The original roadmap's Phase 1 wording dates to
   [`f51c774`](https://github.com/Peter-Tam/godot-agent-kit/commit/f51c774f8a9be8a25eb7ded80f9b6e01b1883492)
   and mentions Save without requiring the independent command.
2. Feature 003 specification commit
   [`bad0e87`](https://github.com/Peter-Tam/godot-agent-kit/commit/bad0e87c0d1e1b8347d51669df531d735f32e2a7)
   added status prose saying “independent close/Save/history controls remain deferred.”
3. Later status prose described those controls as pending. Feature 005 completion
   commit [`164777d`](https://github.com/Peter-Tam/godot-agent-kit/commit/164777df091f3ae4d6a6cdeecca81dc6a2d6b25b)
   retained “Independent Save/history controls and the phase exit assessment remain pending.”
4. The [original Feature 006 proposal](https://github.com/Peter-Tam/godot-agent-kit/blob/a985527e90314711c2353915980998d1068ff3c3/specs/006-save-project-gdscript/spec.md)
   then treated a “standalone Save gap” as established Phase 1 work.

Those references record an interpretation becoming repeated status and then a
feature premise; they do not independently establish a required current workflow.
This decision supersedes the standalone-Save prerequisite interpretation in older
phase-progress prose, including retained earlier-feature artifacts. It does not
rewrite their completed behavior, task acceptance or historical evidence.
`PROJECT_STATUS.md` must not act as circular provenance for the rejected requirement.
The original roadmap wording remains unchanged: this record and the corrected
current status resolve the interpretation without deleting Save durability obligations.

## Authority and risk

The extra authority is commitment of existing unsaved work, even when B's text is
preserved. It introduces risks of:

- Making temporary, experimental or accidental human work durable.
- Acting on stale intent after further typing, or marking newer work saved.
- Overwriting independently changed disk or Resource state.
- Applying broader native Save effects to other documents or history.
- Describing commitment of human work as mere “preservation,” obscuring the decision
  previously retained by the human.

The existing [stock target-Save research](../002-edit-open-gdscript/research.md#13-stock-target-save-finalization-research-2026-09-28)
provides concrete effect evidence. Its `other_no_final_newline` case changed an
unrelated buffer and its history during single-script Save; `human_in_saver`
produced a false clean marker for newer text; replacement/fallback cases wrote
despite detected invalidation. These are historical experiments on particular
routes, not failures of the completed edit path or proof that every safe standalone
Save design is impossible.

The candidate [requirements](spec.md#functional-requirements) show the necessary
surface beyond a command name: dirty-state/saved-source eligibility, exact revision
and identity authorization, confined persistence and saved-state transitions,
partial-failure evidence, native-history preservation and ongoing regression
coverage. No current demonstrated benefit justifies that added authority and cost.

## Principle XIII conclusion

Applying [Principle XIII](../../.specify/memory/constitution.md#xiii-justify-complexity-with-concrete-present-risk)
and the [requirement/evidence burden](../../AGENTS.md#complexity-gate) symmetrically:

- Exact standalone Phase 1 obligation: **not established**.
- Current required workflow failure: **not established**; successful agent editing
  already persists its intended source.
- Supported dirty-buffer states and local developer/editor actors: **real and
  reachable**.
- Material unmet automation need: **not established** by that reachability or by
  explicit permission alone.
- Simpler sufficient current behavior: **observe dirty work → refuse unsafe
  mutation/close → leave Save/discard authority with the human**.

The product decision is complete; a Save implementation is neither complete nor
planned. Standalone Save is **not required for the currently established Phase 1
exit** and must not be counted as a known implementation gap. Features 001–005 and
their accepted behavior remain complete within their recorded scope.

Phase 1 nevertheless remains **in progress pending a dedicated
requirements-to-evidence exit-gap assessment**. That is the next repository action
after this decision PR, not work performed here. It must recompute any remaining
obligations from original requirements and accepted evidence, without presuming
standalone Save is pending. Independent history controls were **not decided** by
this review and require their own original-provenance and product-value assessment;
they are neither automatically required nor rejected. No next feature is selected.

## Revisit conditions

Standalone Save may be reconsidered only when a concrete current workflow establishes
a material need and its authority, safety and maintenance costs satisfy Principle
XIII. For example, an approved operation might genuinely require persistence of an
existing pending buffer, but the review must first establish that persistence is not
better owned by that producing operation. This is a condition for reconsideration,
not a commitment to any future feature or permission to resume this proposal.

The retained candidate requirements are a record of what was evaluated, not a ready
implementation backlog. Any reconsideration must explicitly revisit the product
decision under existing repository governance; this record introduces no generic
ADR framework, new process or additional approval mechanism.
