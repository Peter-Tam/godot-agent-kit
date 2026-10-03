# Phase 1 Exit — Live-editor Script Coherence

**Decision date:** 2026-10-03

**Assessed and closure base:** `79d30352f8d2f3383b3666ca018c13d10652cfb8`

## Decision

**Phase 1 — Live-editor script coherence — is complete on the documented support
profile.** The requirements-to-evidence exit assessment passed on accepted
cumulative evidence, not merely the presence of implementations.

No product-capability or behavioral acceptance-evidence gap remains. The only
remaining work at the assessment's conclusion was lifecycle/status bookkeeping:
record the Phase 1 exit conclusion and correct stale current-status prose. This
record and its accompanying status updates perform that bookkeeping.

Features 001–005 provide the completed runtime capabilities. Feature 006 is a
completed product decision: its standalone Save proposal is **assessed, not
proceeding**, not an implemented Save capability. That decision merged in
[PR #59](https://github.com/Peter-Tam/godot-agent-kit/pull/59) before this closure.
No next feature or Phase 2 task is selected.

## Scope

- Official Godot **4.7.2.stable.official.ed1daf0bf**, engine commit
  `ed1daf0bf001b61586d9930840f2f1394092c079`.
- **macOS 26.6.2 (25G83), arm64**, with the exact recorded host/guest, toolchain,
  native-build and export-template provenance retained in the acceptance records.
- The individual features' supported standalone project GDScript profiles and
  bounds remain unchanged. The edit proof is for one clean, already-open,
  revision-bound standalone script with admitted LF UTF-8 source and native
  source/Save profiles; it does not admit tool-script, script-inheritance,
  load/preload, global-class or exported-declaration effects. Unsupported
  representation/context and missing safety observations refuse explicitly.

See the [edit profile and limitations][edit-limits] and
[closing execution identity][close-identity]. Observation of dirty, divergent or
partly available state remains valid information, not permission to mutate.

## Original Phase 1 obligations

The authoritative sources are [ROADMAP.md, Phase 1][roadmap], its
[inherited sequence requirements][sequence], and the
[constitution's coherence, safety and real-editor gates][constitution]. The
[original roadmap](https://github.com/Peter-Tam/godot-agent-kit/blob/f51c774f8a9be8a25eb7ded80f9b6e01b1883492/ROADMAP.md#phase-1--live-editor-script-coherence)
requires a reliable live-editor editing slice, not editor-menu-command parity:

- Explicit project/editor/session discovery and selection; script discovery,
  read and open; independent disk (**D**), loaded Script (**R**) and visible
  buffer (**B**) observations; attributable dirty-state detection.
- Revision/identity/stale-write protection; native coherent mutation; independently
  verified intended `D == R == B` before edit success; preserved human work.
- Real-editor **A–E**: clean edit, dirty conflict, native Undo/Redo, close/reopen
  persistence and sequential edit stress; applicable Save/reparse/rescan/runtime
  durability without human reconciliation of agent-created divergence.
- Distinct acceptance, application, synchronization and verification; truthful
  partial/interrupted/failure outcomes and unavailable observations; documented
  limits, local routing/confinement/privacy, protocol-independent ownership,
  tooling/export isolation and exact evidence-backed support claims.

Later status prose is not independent requirement provenance. The roadmap's
purpose, capability areas, A–E definitions and exit criteria are unchanged.

## Requirements-to-evidence conclusion

**Every original Phase 1 obligation is satisfied by accepted evidence.** Proposed
standalone Save/history automation controls were classified as **not required for
Phase 1**, not as missing capabilities. No unresolved Phase 1 requirement ambiguity
or missing behavioral evidence remains.

| Accepted feature record | Contribution to the original obligations |
| --- | --- |
| [001 — Observation, T008 completion][observation] | Explicit authenticated target/session selection; independent D/R/B and attributable dirty evidence; source-free ambiguity/refusal, unavailable facts and read-only preservation. |
| [002 — Editing, T005 cumulative acceptance][editing] | Revision/identity-guarded native edit and persistence, independently verified success, human-work protection, truthful stages/partial outcomes, actual history and A–E/durability. |
| [003 — Opening, T003 cumulative acceptance][opening] | Known-script opening and already-open preservation; repeated opening and composed edit/history/durability with fresh observations. |
| [004 — Discovery, T003 cumulative acceptance][discovery] | Exact scoped script inventories, fresh read-only discovery, truthful incomplete/refused results and discovery-selected composed workflows. |
| [005 — Closing, T003 cumulative acceptance][closing] | Guarded clean closure and separate reopening, dirty/stale/replacement-buffer protection, full composed workflow and reviewed cumulative safety/evidence coverage. |

## Real-editor A–E gates

These are real-editor behavioral results, not registration flags, disk-only
inferences, screenshots alone or successful process exits.

| Gate | Status | Accepted evidence |
| --- | --- | --- |
| **A — Clean open-buffer edit** | **Satisfied** | [F002 A/B outcomes][edit-outcomes] independently establish intended D/R/B, clean/saved state and source-attributed validation; [F005 composition][closing] exercises the public workflow without reconciliation. |
| **B — Dirty human-buffer conflict** | **Satisfied** | [F002 A/B and sequential outcomes][edit-outcomes] preserve dirty-different/equal and unrelated work; [F005 sequence/composition][closing] retains dirty edit/close refusals and newer-buffer protection. |
| **C — Real Undo/Redo** | **Satisfied** | [F002 native history][edit-outcomes] and [F005 cumulative review][close-review] establish apply → Undo → ordinary Save → Redo → Save, reachable prior history and no extra entries for refused/unchanged edits. |
| **D — Close/reopen persistence** | **Satisfied** | [F002 durability][edit-outcomes] and [F005 product close/separate reopen][closing] retain persisted source in a newly identified buffer, including final revision 319. |
| **E — Sequential edit stress** | **Satisfied** | [F002's twenty fresh-basis edits][edit-outcomes], each followed by Save, interleave three stale and three dirty refusals; [F005 stress/composition][closing] repeats the twenty-edit path and preserves the final reopened revision. |

## Durability and safety

[F002's accepted outcomes][edit-outcomes] include ordinary Save, close/reopen,
completed public `Script.reload()` reparse, completed filesystem rescan and a
fresh permitted fixture runtime reflecting the edited revision. The original
live editor retains clean intended D/R/B afterward. [F005 composition][closing]
retains applicable Save/reparse/rescan/runtime and source-persistence evidence.

Human unsaved work, including equal-text-but-dirty state and newer work after
application, remains protected. Timeout, cancellation and disconnection do not
imply rollback or definitely-not-applied state. Outcomes distinguish known
application, partial effects and uncertainty; partial/unknown results are not
verified success or authorization for automatic replay. Accepted result-only
reviews and independent witnesses establish those distinctions.

[The cumulative boundary review][close-review] retains authenticated local
routing, project confinement, source/credential privacy and enabled, disabled
and hook-only production exports with inspected artifacts and actual launches.
[Implementation/constitutional review][close-shape] retains protocol-independent
ownership and the existing small surface. Observation evidence is not substituted
for mutation proof; unavailable observations are not waived as inapplicable.

## Save/history interpretation

- Successful agent editing already owns its required persistence and independently
  verifies the saved intended source. It does not need a later agent Save command.
- Standalone Save of an existing dirty human buffer is not a Phase 1 exit
  requirement. The [Feature 006 decision][save-decision] remains unchanged in
  substance: assessed, not proceeding; no Save capability was implemented.
- Real native Undo/Redo and preserved prior valid history are required and
  demonstrated. [Feature 002 US4 and FR-015/016][native-history] explicitly use
  ordinary developer/editor interactions, not new general-purpose commands.
- Agent-callable standalone Undo/Redo/history controls are not a Phase 1 exit
  requirement. A human Undo may legitimately leave unsaved state; observing that
  state and exercising ordinary Save is not reconciliation of a failed agent edit.
  Source persistence after reopening does not promise Undo-close or persistent
  target-buffer history after native disposal, as [F005 records][close-review].

The subsequent exit assessment independently resolved the history interpretation;
it is not attributed to the narrower Feature 006 Save review.

## Evidence reuse and currentness

This closure uses [TEST_POLICY.md's evidence-validity rules][reuse], not a new
acceptance campaign. F005's [group-by-group cumulative review][close-review]
retains its seven accepted prerequisite close groups and the accepted observation
**273**, edit/native **408**, opening **664** and discovery **429** records.
Its [T003 execution][closing] adds sequential **45**, full composed **121** and
affected clean-close **134** passing records. These are evidence records, not
counts of distinct requirements; nested records are not counted twice.

The review records the relevant-input comparison from T002 source
`7385281b1cf1ff5fb0dded2588efab036746f842` through executed T003 source
`0790385c000e9d95aa05ea2a6cb59bbf8e27a947`, including unchanged product/ABI
boundaries and distinct accepted host/guest build provenance. The completed
read-only exit assessment checked the three T003 summary hashes against their
acceptance record. Changes from that executed source to the assessed/closure
base `79d30352f8d2f3383b3666ca018c13d10652cfb8` are documentation-only.

This closure changes only documentation/lifecycle state: no product, protocol,
fixture, witness, runner, build, dependency, support implementation or acceptance
requirement changes. Existing accepted evidence therefore remains valid. No
requirement exists to replay every historical GUI campaign at the literal
closure commit. Failed, interrupted, locked-desktop and research-only executions
remain excluded from acceptance. This is not a newly designated release candidate.

## Known limits and non-claims

This decision does not claim general Godot-version support, other platforms,
arbitrary script/resource types or unsupported source/effect profiles. Existing
bounds, safe refusals and the documented stock-validator endpoint limitation
remain; no stronger OS sandbox, arbitrary same-inode writer exclusion or crash
atomicity is claimed.

It does not deliver runtime/debugger/language-intelligence tooling, live-game
hot-reload guarantees beyond accepted evidence, standalone Save or Undo/Redo
controls, a broader MCP surface, distribution completion or Phase 2 completion.
No new capability, feature, phase-review framework or operational gate is created.
Dated research/task/acceptance records retain what was known at their own delivery
points; subsequent-status notes distinguish them from current lifecycle truth.

[roadmap]: ROADMAP.md#phase-1--live-editor-script-coherence
[sequence]: ROADMAP.md#how-to-read-the-sequence
[constitution]: .specify/memory/constitution.md#core-principles
[observation]: specs/001-observe-gdscript-state/quickstart.md#29-rebased-t008-real-editor-acceptance-and-completion-2026-09-27
[editing]: specs/002-edit-open-gdscript/quickstart.md#14-t005-cumulative-acceptance-2026-09-29
[edit-outcomes]: specs/002-edit-open-gdscript/quickstart.md#coverage-and-observed-outcomes
[edit-limits]: specs/002-edit-open-gdscript/quickstart.md#development-failures-limits-and-phase-assessment
[opening]: specs/003-open-project-gdscript/quickstart.md#10-t003-cumulative-acceptance-2026-09-30
[discovery]: specs/004-discover-project-gdscript/quickstart.md#11-t003-cumulative-acceptance-2026-10-01
[closing]: specs/005-close-project-gdscript/quickstart.md#12-t003-cumulative-acceptance-2026-10-02
[close-review]: specs/005-close-project-gdscript/quickstart.md#t003-execution-decision-and-evidence-review
[close-identity]: specs/005-close-project-gdscript/quickstart.md#exact-execution-identity-isolation-and-static-evidence
[close-shape]: specs/005-close-project-gdscript/plan.md#t003-implementation-shape-and-constitutional-review--2026-10-02
[save-decision]: specs/006-save-project-gdscript/decision.md#decision
[native-history]: specs/002-edit-open-gdscript/spec.md#user-story-4---reverse-and-reapply-through-native-history-priority-p1
[reuse]: TEST_POLICY.md#reusing-evidence-across-commits

## Exit conclusion

No product-capability or behavioral acceptance-evidence gap remains for Phase 1
on the documented support profile.

**Phase 1 is complete.**
