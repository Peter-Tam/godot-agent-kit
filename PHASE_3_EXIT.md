# Phase 3 Exit — MCP Adapter MVP

**Decision date:** 2026-10-05

**Assessed and closure base:** `705955f156ced2114d80b688ab67fadf87045f19`

## Decision

**Phase 3 — MCP adapter MVP — is complete on the documented support profile.**
The separate roadmap assessment found the exit obligations supported by current
implementation and accepted evidence, not by Feature 007's completion checkbox.

[PR #67][t004-pr] was confirmed merged before assessment. Fetch/prune and
fast-forward verification found clean `main` equal to `origin/main` at the exact
revision above. PR #67's source branch was already absent locally and remotely.
T001–T004 were delivered separately in merged PRs [#64][t001-pr], [#65][t002-pr],
[#66][t003-pr] and [#67][t004-pr]. [Phases 1][phase-one] and [2][phase-two] retain
their accepted guarantees; the relevant-input review below accounts for subsequent
changes rather than presuming their old exit revisions prove MCP behavior.

**Phase 3 completion does not automatically authorize Phase 4 or a v0.1 release.**
This closure selects no next feature, designates no release candidate and starts
no release or later-phase work.

## Scope and support profile

- Official Godot **4.7.2.stable.official.ed1daf0bf**, engine commit
  `ed1daf0bf001b61586d9930840f2f1394092c079`, executable SHA-256
  `c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf`.
- **macOS 26.6.2 (25G83), arm64**, dedicated graphical `VirtualMac2,1` VM,
  **four vCPUs and 6 GiB RAM**. No six-vCPU substitution or host-GUI acceptance.
  Rust/Cargo **1.98.1**, Apple clang **21.0.0**, SDK **26.5**; exact engine,
  API/ABI, native and export-template provenance remains in the accepted records.
- Local stdio **MCP 2025-11-25**, **rmcp 3.5.0**, **Tokio 1.53.1**, public tool
  schema **1**, matched private bridge **v6** and native integration revision **4**.
  Exactly `discover_scripts`, `read_script` and `edit_script` are public tools.
- Actual selected coding-agent clients: **Codex CLI 0.153.4** and **OMP 18.5.1**,
  using **gpt-6-astra**. Both have positive and adversarial workflow evidence;
  the new complete T004 composed conversation used OMP, not both clients.
- Existing [operation/source bounds][bounds] and standalone project GDScript
  profiles remain binding. Open editing preserves the same clean document and
  independently verified intended D/R/B. Closed editing supports coherent clean
  cached R or positively observed absent R, while the document remains absent.
  Unsupported context, dirty/divergent state or missing safety evidence refuses.
- The [documented long-lived setup][setup] uses the optional
  `--validator-engine` configuration to prepare source-free native documentation
  before protocol input. Actual source validation still launches fresh isolated
  stock processes inside each request's original budget. Registry-only callers
  retain their fully timed cold path, not a prepared-readiness guarantee.

The composed run records environment identity
`5be872888a3eacb7798de40e70d66e1f438d908e00cf714575d65b2632596ced`, MCP executable
SHA-256 `332243aab6d9a76432f2968dc225022bc7a93662e83e880cf04b3bc264174047`, and
production native library SHA-256
`b1d10e9ed714dacb2ccc4cb614e146d8ea2059003c52813f4acdd8ff91ee76c1`.
These identify executed artifacts, not a claim that every later rebuild is
byte-identical. [Currentness](#evidence-currentness) establishes applicability to
the assessed revision.

## Roadmap interpretation and public surface

The [unchanged Phase 3 roadmap][roadmap] says its purpose is to make the proven
script workflow usable by real coding agents. Its capability areas name a small
surface for status, capabilities, read/open/edit/save/close, Undo/Redo and state
inspection. Its exact exit is:

> Real MCP clients, including coding agents rather than only an internal harness,
> complete the Phase 1 A–E workflows over MCP. Dirty human edits remain protected,
> safe refusals remain explicit, and successful edits need no human reconciliation.
> This is the likely first useful public release, tentatively **v0.1**, not a broad
> Godot API wrapper.

**The three-tool surface is sufficient for this exit; it does not expose every
named editor action as an MCP command.** This conclusion rests on the wording and
approved interpretation together:

1. The roadmap specifies capability areas and behavioral A–E exit gates, not one
   wire tool per noun or a requirement that the agent initiate every native
   acceptance interaction. Its [Spec Kit relationship][roadmap-specs] assigns
   behavior and architecture to approved specifications/plans.
2. The maintainer's [Feature 007 clarifications][clarifications] explicitly select
   discover/read/edit, preserve open/closed lifecycle, exclude MCP open/close, and
   assign C to ordinary native Undo/Redo/Save and D to editor/fixture close/reopen.
   Its [History and Phase 3 acceptance interpretation][scope-interpretation],
   FR-019 and SC-005 explicitly retain MCP calls/reads plus independent native
   witnesses. This is an approved Phase 3 interpretation, not an inference from
   a completed task or a retrospective weakening to fit the implementation.
3. [Phase 1's Save/history interpretation][save-history] and [Phase 2's explicit
   non-requirements][phase-two-nonrequirements] preserve edit-owned persistence
   and native history without standalone controls. They support, but do not alone
   decide, Phase 3's separate external-adapter obligation. The new actual-agent
   evidence below supplies that obligation.

The agent's product operations and observations use MCP. Fixture/editor actions
establish human work, exercise native history and challenge already-verified
persistence; they are **not MCP-exposed capabilities**, hidden aliases, implicit
open/edit/close machinery or repairs after false success. Closed editing removes
the need to open a tab merely to make a source edit eligible. Session status and
capability information are required behavior delivered through negotiation,
operation state and explicit failures, not standalone tools. No unresolved roadmap
capability or exit ambiguity remains under this approved interpretation.

## Requirements-to-implementation and evidence map

| Phase 3 obligation | Current implementation / behavior | Accepted evidence |
| --- | --- | --- |
| Purpose: usable by real coding agents through the first external adapter | [MCP executable][executable] and [adapter][adapter] dispatch checked operations to the reusable core, not shell glue. | [T002 selected clients][clients] and [T004 actual-agent composition][composed]. |
| Editor/session status and explicit targeting | Explicit project/session selectors; authenticated routing; read state and structured unavailable/incompatible/ambiguous results. Connection success is not editor readiness. | [Public contract][interface], T002 observations and [T003 selection/preservation][adversarial]. |
| Capabilities, consistent schemas, discovery and progressive disclosure | Static three-tool [catalog][catalog], initialization tools capability, typed input/output schemas, source/revision/state and targeted next actions. Unsupported protocol features are not advertised. | T002 negotiated catalog/carrier and [qualitative cumulative review][cumulative]; both agents used the structured results. |
| Script read/edit and diagnostic/state inspection | Trusted read and revision-based [execution][execution]; independent open D/R/B or closed applicable authorities; explicit unavailable/invalidated facts. | [T001 core/native acceptance][foundation], T002 open/cached/absent workflows and T003 refusals/effects. |
| Open/close capability-area references | Existing core opening/closing remain intact. Public MCP edits preserve lifecycle; later ordinary native close/reopen proves persistence. No MCP open/close capability is claimed. | Approved interpretation above; [T004 D/E][composed] plus matched legacy-operation evidence. |
| Save capability-area reference | Successful edit already persists and verifies intended source. Ordinary Save challenges durability and commits deliberately selected human/history state; no standalone MCP Save or automatic saving of dirty work. | T004 pre-Save-intent witnesses; [Feature 006 decision][save-decision] remains assessed, not proceeding. |
| Undo/Redo capability-area reference | Actual native open-document history remains reversible/reapplicable and prior history reachable. No standalone MCP history tool, synthetic replacement edit or invented closed-buffer history. | T004 C below and T003 native-history/dirty-Resource regressions. |
| Structured outcomes/errors without loss of core facts | [Adapter projections][outcomes] preserve target, stage, reason, application certainty, evidence and safe action. Protocol/request errors remain distinct from execution or post-dispatch host failures. | T002 [recovery correction][recovery] and T003 [result-carrier/uncertainty evidence][client-failures]. |
| Cancellation mapping and delivery semantics | Call-scoped cancellation retains bounded supervisor ownership and known/partial/unknown effects. Undelivered results are not rollback, success delivery or permission to replay. | T003 cancellation, original deadlines, lost/blocked output, worker/channel loss, newer work and actual-client reconnect evidence. |
| Adapter/core/integration separation; validation | MCP framing/DTOs remain at the adapter; core owns fresh revision/lifecycle admission, authorization and outcome reduction; Godot integration owns native effects. | [Execution contract][execution], T001–T004 boundary acceptance and [shape/constitutional review][shape]. |
| Released compatible protocol/SDK and transport security | Selected released MCP/rmcp versions; local stdio, explicit registry, authenticated/confined editor access, no extra arbitrary execution/network/force authority. | [Dependency/client research][research], T002 actual negotiation and T003 denied/escaping/ambiguous-target evidence. |
| Protocol/log separation and truthful advertisement | Protocol-only stdout, source-free incidental diagnostics; only authorized requested results disclose permitted selected content. Catalog support never promises target eligibility. | [Privacy/export disposition][cumulative], complete-frame selection/interruption checks and startup/worker checks. |
| Deferred scope | No catalog expansion, scene authoring, runtime/debugger tools or unnecessary protocol features. Durability runtime probes remain acceptance tooling. | Exact catalog/contract and inspected enabled/disabled/hook-only production exports. |
| Exit: real clients complete A–E over MCP | Actual client calls plus independent live native history/lifecycle/durability witnesses, mapped below. | T002/T003 both-client evidence and new T004 complete composed interaction. |
| Exit: dirty work protected; explicit safe refusals | Revision/identity/lifecycle/conflict guards preserve dirty/equal-text/newer work; no force, implicit repair or queued retry. | T003 adversarial/newer-work witnesses and T004's three dirty plus three stale refusals. |
| Exit: success needs no human reconciliation | Independent intended postconditions and saved state precede success. Later native actions challenge success, not establish it retroactively. | T004 A–E witnesses, including pre-Save-source comparison and later runtime values. |
| Tentative v0.1 language and inherited obligations | Likely usefulness is not release authorization. Exact support, privacy/export, constitutional guarantees and release gates remain binding. | This assessment and [TEST_POLICY's separate release rule][release-policy]. |

## Real-agent A–E and durability

[T002][clients] records ten completed profiles across both clients: **127 independent
records and 109 correlated tool results**, including known-target/no-op, Unicode,
empty and full **512 KiB** source, informative reads and later durability.
[T003][client-failures] adds **31 independent records, 25 model-visible results and
two deliberately undelivered results**. Undelivered results are not counted as
model-visible evidence.

T004's actual OMP conversation used source
`3067fc690f1b08e28bf7a296468c8105c4122ea1`. It completed **68 MCP operations**:
three discoveries, 39 reads and 26 edit attempts. This assessment independently
matched every model-visible public object and submitted argument object to the
retained server call, and every delivery record to its request ID. The only other
tool actions were three reads of MCP tool descriptions; no shell, direct-file or
private-bridge operation substituted for a product call. A separate sample review
matched all **18** primary Codex workflow results/arguments to its retained calls.

The independent T004 finalizer has **75 passed records**. Its
`real_client_acceptance: false` is correct: fixture results alone do not prove
model visibility. The separate actual-client transcript supplies that proof.

| Phase 1 gate | Current MCP-path proof |
| --- | --- |
| **A — Clean open-buffer edit** | Eight successful open edits independently match intended D/R/B, clean/saved state and the same Script/editor/buffer identities. Success precedes ordinary Save; the strengthened Save witness compares with pre-Save intended B. |
| **B — Dirty human-buffer conflict** | Three dirty refusals include different-text and equal-text dirty work. Independent before/after snapshots preserve source, dirty state, identities and history. T003 additionally covers newer work after effect entry and dirty loaded R. Human Save after a deliberate conflict is not reconciliation of agent-created divergence. |
| **C — Real Undo/Redo** | Native Undo → Save → Redo → Save, each followed by actual MCP reads and independent witnesses: `D=101, R/B=47, dirty` → `D/R/B=47, clean` → `D=47, R/B=101, dirty` → `D/R/B=101, clean`. Prior history remains reachable; no second edit simulates history. |
| **D — Close/reopen persistence** | Native close/reopen creates a new visible document for the open target. Later opening of already-verified closed targets preserves source. Save, completed reparse/rescan and fresh runtimes retain final values **108, 206, 306**, with live authority witnesses and actual MCP reads. |
| **E — Sequential edits** | Twenty verified fresh-read/revision edits: **8 open / 6 cached-R closed / 6 absent-R closed**, with three stale and three dirty refusals interleaved. All intended changes persist; no skipped/retried failure is counted as a success. |

This assessment checked the independent snapshots for all twenty successes:
intended D and applicable R/B agree, open identity/saved state is retained, cached
R remains coherent, and absent R/B remain absent for closed mutation. All six
refusal snapshots retain the complete captured before state. Closed B/history are
inapplicable because absence is positively observed, not because a getter failed.
Successful edits require no human reconciliation.

## Timing, outcomes and privacy

The documented four-vCPU support environment remains unchanged. All 68 composed
responses meet the original consumed-output bounds: **5 seconds read/discover**,
**10 seconds edit**, without renewing the **4.5/9.5-second** work cutoffs.
This assessment recomputed maxima from retained delivery records:

| Operation | Maximum complete response |
| --- | ---: |
| Discover | 0.239333 s |
| Read | 0.778521 s |
| Edit, including both closed profiles | 6.512634 s |

Native-document preparation took **2.555010 seconds before protocol input**.
It is neither an untimed source preflight nor a prewarmed compiler. Fresh-child
launch, current source/context validation, independent actual-source validation
and delivery remain inside the original request clock. [Focused preparation
proof][preparation] includes ten cold/prepared diagnostic pairs, bounded startup
loss/cleanup, **19** public revision records and **13** before/after-effect
snapshot-loss/corruption records. T003's blocked-editor maxima remain **4.509811 s
read, 4.510608 s discover, 9.520930 s edit**. Lost/blocked output is separate
unavailable-delivery evidence, not a consumed-output timing pass.

[T003's actual-client limitations][client-failures] remain explicit: Codex required
conversation reconnection for a fresh read after transport loss; OMP internally
resent one identical edit, which the revision guards refused. The kit adds no
replay/queue/compensation, and does not promise exactly-once client behavior or
that a missing response means no effects. Structured partial/unknown outcomes
remain non-success; a later observation cannot retroactively verify the old edit.

The assessment inspected retained privacy summaries: **17 completed named
startup/help/worker/authorized/error records**, **18 selection records** and
**10 interrupted/sticky-denial records**. The enclosing run containing the first
group failed later; only its explicitly passed completed group is reused.
Enabled, disabled and hook-only export records list actual ZIP/pack contents and
hashes, zero runtime exit status and zero observed listeners. No addon/native/
fixture tooling appears in those recorded exports. Native-document preparation
is host tooling, not exported game content; no new gameplay authority or telemetry
was added.

## Evidence currentness

Apply [TEST_POLICY's relevant-input rules][reuse], not a historical campaign replay:

- **Phase 1/2 through T001:** retain their accepted A–E/core/disclosure evidence
  with T001's new closed-capability proof and matched **v6/native4** legacy,
  admission, interruption, durability and export witnesses. The coordinated
  private cutover is not dismissed as a version-only change. [T001's review][t001-current]
  identifies affected reruns and later projection-only correction evidence.
- **T002:** the inspected changes to existing production owners are closed-worker
  IPC and adapter exposure. Large-frame/cancellation/deadline tests and actual
  **512 KiB** MCP edits cover the IPC change. The later operation-specific host
  recovery correction has focused caller evidence; [its review][recovery] checks
  that none of the 109 accepted client results used the changed failure branches.
- **T003:** the only non-test production delta from merged T002 is the open-native
  dirty-Resource guard repeated before every stage. Its failing-before/passing-after
  regression, actual clean/history/legacy behavior and interruption witnesses
  cover that boundary with the rebuilt native library. [The 941-record review][t003-current]
  does not presume native byte identity or invalidate unrelated closed behavior.
- **T004:** the changed production boundary is optional source-free documentation
  preparation and fresh-validator launch/staging reuse. Source/ABI/native mutation,
  routing, admission, effect reducers and schemas are unchanged. Newly executed
  cold/prepared semantic comparisons, preparation lifecycle/fault cases and the
  actual composed executable cover the changed path. Strengthened pre-Save and
  complete-frame privacy witnesses have focused regressions and replacement
  affected evidence; earlier insufficient checks do not stand in for them.
- **Executed source to assessed main:** the complete diff from `3067fc6` contains
  evidence/status documentation plus removal of blank lines in three Rust worker
  files. The production diff is empty when blank-line-only changes are ignored;
  no native/addon, fixture, dependency, protocol or support input changed. All four
  recorded MCP acceptance-driver hashes match the current files.
- **Analysis:** all six recorded fingerprints recover exactly from
  `efe38b3c3fb581c97403b746279adcd6d2ff7fbc`. Current spec/constitution/governance
  hashes still match; cumulative plan/task/quickstart drift contains completion,
  evidence and shape-review bookkeeping only. The [passed analysis][analysis]
  remains current under the existing policy; no fingerprint-only renewal is needed.

No uncovered currentness gap was identified. This closure changes documentation
only; no new runtime evidence, Rust tests, native build or GUI campaign was needed.
The accepted historical Rust/Python checks remain those in the feature record,
not checks newly executed here. In particular, the **473 distinct passing Rust
tests** are coverage across documented runs with the existing unchanged timing-case
exception, **not an all-green aggregate invocation**.

The failed serial composed attempt's fifth edit remains
`applied_unverified/validation_unavailable` at **9.464649 seconds**, not verified
success. Its earlier successes, matching source and visible capture do not
complete SC-005. The later complete conversation supplies that proof. Failed,
incomplete and research-only executions are not promoted to accepted passes.
Deliberately undelivered results support delivery-loss assessment, not a claim of
client-visible transaction success.

### Retained artifacts inspected

Private artifacts stay under `~/.local/state/godot-agent-kit-vm/artifacts/`;
source-bearing transcripts and credentials are not published by this closure.
The following SHA-256 values were computed during assessment:

| Artifact, relative to that root | SHA-256 |
| --- | --- |
| `t004-omp-prepared-20261005T140507Z/20261005T142357Z-81078c012cfa/artifacts/summary.json` | `1cc6eeeac662c6eeef20f7b766846f68dd4cb50fe48da815f6368032838cf907` |
| Same VM snapshot, `provenance.json` | `7140a04ba1606cab23e73dce7aedd7e3031f746672163ec8397c42317d7ce2a2` |
| Same VM snapshot, `artifacts/mcp-calls.jsonl` | `e0e89d1f8dade62d12166e49f854002e895beff715f21e2a97fc429cc036f58a` |
| `t004-omp-prepared-20261005T140507Z-4cdg2m6j/model-visible.jsonl` | `89726bc585e57c4507ea6ad45256bb86ea43dad1a5c9bf6a6b3c57bcc06670f1` |

These are retained-run integrity references, not new runtime executions or a new
evidence service. Earlier evidence is reused through its linked currentness chain;
this assessment does not claim to re-audit every historical raw record.

## Limitations and explicit non-claims

- No first-class MCP open/close/Save/Undo/Redo, standalone status tool or general
  capability framework. The three public tools and their actual structured state
  are the supported agent surface; editor/fixture controls are not product tools.
- No loaded-class/static-variable refresh, running-game hot reload, persistent
  history after buffer disposal, universal Resource authoring or arbitrary scripts.
  The [source/effect profiles and bounds][bounds] remain explicit.
- No broader client/version/platform matrix, arbitrary-load latency promise,
  exactly-once delivery, automatic retry, rollback or crash atomicity. The existing
  normal-local-user threat model and stock-validator loopback endpoint limitation
  remain; stronger hostile-same-UID isolation is not claimed.
- No remote product transport, arbitrary evaluation/process/file access, force
  overwrite, scene/project/runtime/debugger tooling or distribution expansion.
  The fixed VM relay is acceptance infrastructure, not a remote product mode.
- No release-readiness or packaging claim beyond this roadmap phase's exit.
  The tentative v0.1 milestone still requires a separately authorized candidate
  and applicable release evidence. Feature 006 remains assessed, not proceeding.

This assessment introduces no product behavior, framework, process gate or new
constitutional exception. **Phase 3 is complete; Phase 4 and v0.1 release work are
not authorized or started by this closure.**

[roadmap]: ROADMAP.md#phase-3--mcp-adapter-mvp
[roadmap-specs]: ROADMAP.md#relationship-to-spec-kit
[phase-one]: PHASE_1_EXIT.md
[phase-two]: PHASE_2_EXIT.md
[save-history]: PHASE_1_EXIT.md#savehistory-interpretation
[phase-two-nonrequirements]: PHASE_2_EXIT.md#explicit-non-requirements
[clarifications]: specs/007-mcp-script-workflow/spec.md#session-2026-10-03
[scope-interpretation]: specs/007-mcp-script-workflow/spec.md#scope-and-governance-alignment
[bounds]: specs/007-mcp-script-workflow/data-model.md#2-supported-profile-and-bounds
[setup]: specs/007-mcp-script-workflow/quickstart.md#build-and-local-connection
[interface]: specs/007-mcp-script-workflow/contracts/mcp-interface.md
[execution]: specs/007-mcp-script-workflow/contracts/execution.md
[research]: specs/007-mcp-script-workflow/research.md#4-released-protocol-dependencies-and-clients
[analysis]: specs/007-mcp-script-workflow/analysis.md
[shape]: specs/007-mcp-script-workflow/plan.md#t004-implementation-shape-and-constitutional-review
[foundation]: specs/007-mcp-script-workflow/quickstart.md#t001-execution-evidence-2026-10-04
[t001-current]: specs/007-mcp-script-workflow/quickstart.md#currentness-implementation-shape-and-constitutional-review
[clients]: specs/007-mcp-script-workflow/quickstart.md#selected-client-records
[recovery]: specs/007-mcp-script-workflow/quickstart.md#post-review-correction-operation-specific-recovery
[adversarial]: specs/007-mcp-script-workflow/quickstart.md#t003-execution-evidence-2026-10-05
[client-failures]: specs/007-mcp-script-workflow/quickstart.md#actual-selected-clients-and-their-limits
[t003-current]: specs/007-mcp-script-workflow/quickstart.md#checks-reuse-and-completion-scope
[preparation]: specs/007-mcp-script-workflow/quickstart.md#t004-timing-safe-validation-preparation-2026-10-05
[composed]: specs/007-mcp-script-workflow/quickstart.md#t004-execution-evidence-2026-10-05
[cumulative]: specs/007-mcp-script-workflow/quickstart.md#cumulative-evidence-disposition
[save-decision]: specs/006-save-project-gdscript/decision.md#decision
[reuse]: TEST_POLICY.md#reusing-evidence-across-commits
[release-policy]: TEST_POLICY.md#constitutional-guarantees-and-release
[executable]: mcp-server/src/bin/godot-agent-kit-mcp.rs
[adapter]: mcp-server/src/mcp/
[catalog]: mcp-server/src/mcp/schema.rs
[outcomes]: mcp-server/src/mcp/output.rs
[t001-pr]: https://github.com/Peter-Tam/godot-agent-kit/pull/64
[t002-pr]: https://github.com/Peter-Tam/godot-agent-kit/pull/65
[t003-pr]: https://github.com/Peter-Tam/godot-agent-kit/pull/66
[t004-pr]: https://github.com/Peter-Tam/godot-agent-kit/pull/67
