# Repository testing policy

## Authority and purpose

Agents MUST read this policy before planning verification, selecting scope, validating
implementation, or deciding whether to repeat evidence. [AGENTS.md](AGENTS.md) delegates
AGENTS.md-level authority only: the [constitution](.specify/memory/constitution.md), approved
specification, approved plan, and selected task take precedence, in that order. Conflicting
approved artifacts require an explicit design correction before dependent execution; this
policy does not silently waive an approved obligation. It defines verification scope and
local GUI execution location, not a new evidence database, approval mechanism, or CI gate.

Verification establishes behavior and acceptance, not ritual command completion. Select
task-owned acceptance, directly affected regressions, and applicable static/build checks.
Future design and task-granularity review MUST reject unjustified broad execution requirements
before `/speckit.analyze`, using this policy rather than duplicating test-scope rules in each
artifact. An approved requirement remains binding until explicitly corrected.

## Operational scope

### Documentation and governance

For documentation/governance-only changes, validate Markdown structure/rendering, local
links and anchors, code fences, and the intended diff (including whitespace/diff checks).
Check syntax of changed executable snippets using their applicable interpreter or parser.
Do not execute expensive illustrative workflows merely because a command appears in a
document. Do not run Rust product tests, native builds, or Godot GUI campaigns unless
executable tooling also changes; then validate that tooling's affected behavior. A wording,
checkbox, status, link, or evidence-record change alone does not invalidate product evidence.

### Rust

During development, run the smallest affected unit or integration test. On failure, use
**failed test → fix → affected test** before broadening; do not repeatedly run the full Rust
suite after every edit. Mutation-semantic changes require integration coverage of affected
state transitions; pure logic belongs in deterministic unit tests. At Rust-affecting task
completion, normally run the existing baseline from `mcp-server/`:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo doc --no-deps --locked
```

Build the affected actual consumers (binaries, libraries, examples, or integrations), not
only an isolated library that avoids their contract. A concrete task-specific exception
MAY be documented with requirement provenance, rationale, and equivalent evidence; it cannot
waive constitutional guarantees or silently override approved artifacts. These are Rust
implementation checks, not mandatory commands for Markdown-only tasks.

Track `Cargo.lock` and use locked dependency resolution in CI and applicable validation.
Document toolchain, edition, and MSRV claims; test any promised MSRV. Test valid approved
feature combinations, not blanket `--all-features`. Add `--workspace` where coverage of
actual workspace members requires it, not by assumption. Default `cargo test` includes
doctests; `cargo test --all-targets` alone does not, so run `cargo test --doc --locked`
separately if using that mode. Commands depend on the approved Cargo setup; do not invent
crates, tools, feature sets, or a root workspace. Existing toolchain selection and Rust LSP
instructions remain in [AGENTS.md](AGENTS.md#rust-lsp-development-tooling).

### Native integration

Native builds/tests apply when native source, generated ABI/API or headers, native family
or revision, build provenance, or a requirement for a newly built binary changes. Exercise
loading and affected native behavior as well as compilation. Transport-only, docs-only, or
unrelated Rust changes do not require native rebuilding when relevant native provenance
is demonstrably reusable. A rebuilt or relinked artifact needs demonstrable relevant
equivalence; a matching version string or manifest claim alone is
not sufficient. Distinguish production and fixture-fault artifacts and preserve their
export isolation.

### Campaign and workflow tooling

Runner, fingerprint, checkpoint, or resume changes require affected campaign tests proving
selection, identity, invalidation, and resume behavior as applicable. Product behavior alone
does not require retesting an unchanged campaign orchestrator. Fixture registration changes
need affected registration/selection checks, not historical GUI executions merely to populate
an aggregate mode. Workflow/configuration checks, including Actionlint, apply when the
corresponding configuration or workflow changes or the selected task explicitly owns them;
they are not a generic tax on unrelated changes.

### Real-editor development

Use the failed/affected scenario first, then the smallest affected group that establishes the
changed behavior. Do not routinely run `--scenario all`, historical groups, or a full campaign
for confidence. Broaden only for the concrete invalidation or owned interaction described
below. Tests must use deterministic, isolated fixtures, explicit deadlines, and event-based
synchronization; arbitrary sleeps or process success are not correctness evidence. Record
regression cases for discovered failures and diagnostics identifying transaction stages and
actually observed D/R/B surfaces.

### Local graphical execution boundary

Local real-editor acceptance MUST use the dedicated **macOS arm64 VM** through
[`godot-addon/tests/run_in_vm.py`](godot-addon/tests/run_in_vm.py). Follow the
[one-time setup and maintainer workflow](.github/LOCAL_VM.md). The VM is the test
desktop: Godot Editors, owned runtimes, window capture and focus requests remain in its
logged-in graphical session. The host supplies committed source and command/control only;
no shared human projects or mutable host working tree. `--no-graphics` hides the VM viewer,
not Godot's GUI; headless Godot is not a replacement for real-editor acceptance.

- **Normal development:** the failed/affected scenario and smallest affected real-editor
  group run inside the VM.
- **Task completion:** task-owned plus directly affected real-editor regressions run
  inside the VM, alongside the same applicable static/build checks.
- **Cumulative/release:** required expensive campaign portions run inside the VM, using
  existing checkpoints and cumulative valid evidence rather than automatic full replay.
- **Host GUI:** not the default, and MUST NOT be used merely because VM setup is
  inconvenient. The wrapper has no host fallback. A specifically chosen host-only
  debugging experiment must be explicit; it is not automatic acceptance substitution.

This changes **where**, not **which**, tests execute. Do not add historical GUI reruns
because an isolated desktop is now available. Existing hosted CI and optional dedicated
live-editor CI remain unchanged; no hosted provider or new CI gate is required.

Missing/stopped VM, failed control, stale source, mismatched engine/native inputs or a lost
graphical session MUST fail before acceptance can be claimed. Preserve failure evidence.
Record VM/guest identity, exact tested source and observed engine/tool/native provenance.
If guest macOS differs from the recorded supported candidate, label results isolated
development/regression evidence until support equivalence is deliberately approved;
do not broaden a product support claim. The infrastructure's own acceptance requires
a focused real guest GUI run, retrieved artifacts, and process/window evidence supporting
the VM boundary with no test-launched host Godot/window/focus requests.

## Completion scope and justified broadening

A task completes with its task-owned acceptance, directly affected regressions, and applicable
static/build checks. Whole-suite execution is justified only when:

1. The task inherently owns a whole-suite interaction whose required behavior cannot be
   established by independent groups and valid prior evidence. Identify the cross-group
   semantics and why that complete execution is necessary. A label such as “cumulative”
   is not itself such an interaction.
2. A changed shared boundary invalidates evidence across that suite. Record all three:
   **the exact changed behavior/boundary; the exact affected suite(s); the concrete failure
   mode that can invalidate their prior results**. Broaden only to those affected suites.

An explicitly justified whole-suite/release requirement is also binding under the authority
order above. The existence of `--scenario all` or `--suite all` does not make executing it
mandatory. Such modes may be implemented and tested as convenience interfaces. They become
mandatory only for uniquely necessary documented cross-group semantics or an explicit,
independently justified whole-suite/release obligation.

The following are insufficient broadening reasons by themselves: a shared file was edited;
a shared integration was touched; existing callers must continue to work; the PR has a new
final head; cumulative confidence; a suite already exists; caution; or theoretical
completeness. Existing-caller correctness remains mandatory, but the execution set must map
to reachable affected behavior rather than an unbounded claim that everything might fail.

Plans and tasks MUST NOT create contrary broad-suite obligations without a concrete present
justification. Before approving task granularity and before `/speckit.analyze`, review every
instruction to execute a complete historical suite. Require the three-part shared-boundary
justification above, or a concrete independently justified whole-suite interaction/release
exception. If neither is present, rewrite the instruction to focused evidence before analysis.
Do not let generic task wording acquire higher-priority authority by accident.

Examples of affected boundaries:

- A close-only input decoder change invalidates decoder and close-caller cases reaching that
  decoder, not unrelated observation/edit/open/discovery evidence.
- A close reducer/outcome change invalidates the close transitions, refusal/success outcomes,
  and composed flows reaching that reducer. It does not automatically invalidate other tools.
- Shared authentication/framing, protocol/ABI compatibility, or shared editor owner/context
  changes may affect multiple callers. Identify their reachable failure and exact suites;
  genuinely suite-wide invalidation requires that wider evidence, not a close-only pass.
- Fixture or witness changes invalidate results whose setup or independent observation
  semantics changed. An unrelated fixture addition does not invalidate unchanged groups.
- Runner changes invalidate affected selection, fingerprint, resume, or execution semantics.
  Retest those semantics and any behavioral checkpoints whose evidence they compromise,
  not every GUI scenario by default.

## Reusing evidence across commits

Evidence is valid for the current delivered behavior, not merely for the literal commit
where it ran. Reuse accepted passes after reviewing all relevant inputs:

- production execution paths and actual caller behavior;
- installed addon behavior and native/Rust binary behavior and provenance, including source,
  generated ABI/API/header, manifest/build identity, native family/revision, and linked inputs;
- protocol and ABI contracts, authentication, routing, owner/context, and negotiation;
- fixtures, independent witnesses, and the meaning of recorded observations;
- runner identity/fingerprints and applicable checkpoint semantics;
- supported environment, including exact Godot engine, platform, toolchain, and export inputs;
- the acceptance requirement and the surfaces/transitions it requires.

Changed relevant inputs invalidate **affected evidence only**. Rebuilt/relinked artifacts
require demonstrable relevant equivalence or affected reruns, not mere version claims.
Review provenance using existing hashes and records; do not introduce a new database.

Docs, checkboxes, status, links, and evidence prose do not themselves invalidate results.
Neither do unrelated groups, registration additions, test-code additions, or later
feature-specific code outside the relevant execution path. These are not categorical
exemptions: a changed test assertion, registration, fixture, or fingerprint that changes
what an accepted witness proves is a relevant input and requires affected review/rerun.

Failed, incomplete, interrupted, or unverified executions are never reusable passes.
If relevant equivalence is uncertain, rerun the affected group/suite, not the universe.
Record provenance and the reuse/invalidation rationale in the existing feature quickstart's
evidence section; PR evidence may link to it. Accepted historical counts and completion states
remain historical facts; later input changes may require new affected evidence, not retroactive
rewriting of what was accepted.

## Cumulative feature acceptance

Cumulative acceptance means complete **valid evidence coverage**, not reexecution at a literal
final commit. Review every scenario, functional requirement (FR), and success criterion (SC)
and identify it as newly executed, reused with valid relevant inputs, rerun after invalidation,
or substantively inapplicable with recorded justification. Missing observability is not an
inapplicability argument. Complete coverage must include newly introduced composed/stateful
behavior and independently justified release executions; individual passes do not substitute
for a required interaction that none of them observed.

For expensive suites, use individually runnable groups and existing checkpoints/resume
facilities where available. Preserve unchanged successful checkpoints; rerun failed/interrupted
checkpoints and those whose relevant inputs changed. A late failure does not invalidate earlier
unchanged passes or make the whole suite atomic. Existing resume identity/fingerprint rules
remain mandatory: never promote an invalid checkpoint. Acceptance-evidence validity is a
separate review of the accepted run's relevant inputs, not permission to bypass campaign checks.
The VM wrapper preserves the existing runner and checkpoint semantics. Its stable guest
execution identity participates in the campaign environment fingerprint; moving execution
into a guest must not authorize stale host checkpoints or weaken relevant-input review.

For Feature 005 T003, newly execute sequential and full composed groups, directly affected
regressions, and applicable cheap/build/campaign checks for the actual changes. The seven
accepted close groups and accepted observation/edit/open/discovery evidence may be reused
following relevant-input review. Fixture/campaign registration/fingerprint/docs-only changes
do not demand unchanged historical GUI reruns. T001's bridge v5, native revision 3, and shared
owner/context changes legitimately justified broad evidence; T002's accepted completion facts
are retained. This policy neither starts T003 nor changes task dependencies or product scope.

## Constitutional guarantees and release

Maintain real-Godot live-editor regression coverage for mutation-related functionality, and
satisfy applicable A–E evidence before feature completion or release. C applies whenever
UndoRedo is claimed:

- **A — Clean open-buffer edit:** disk (D), loaded Resource/Script (R), and visible buffer (B)
  converge without human reconciliation.
- **B — Dirty human-buffer conflict:** preserve unsaved human work; safely refuse or follow
  an explicitly designed resolution workflow, never silently lose it.
- **C — Real Undo/Redo:** independently verify apply → Undo → Redo across applicable surfaces
  through Godot's actual editing history, not merely registration.
- **D — Close/reopen persistence:** successful edits survive reopening without stale/reverted
  state.
- **E — Sequential edit stress:** repeated edits do not disappear, diverge, conceal conflicts,
  or revert on Save.

Also verify Save/reparse/rescan/runtime durability wherever applicable. Observe D/R/B
independently in real Godot: disk-only checks, Resource getters alone, mocks, headless runtime,
acknowledgments, or agreement between disk and runtime cannot establish visible-buffer coherence.
Tests must preserve human-work protection, truthful structured failures, confinement, and
transaction verification. Record justification for inapplicable scenarios/surfaces and state
verified guarantees and limitations, not aspirations. Keep exact supported Godot versions
explicit and covered by appropriate CI and real-editor tests; do not claim untested support.
Explicitly test the development/export boundary: `addons/` and `@tool` alone do not keep editor
tooling out of production exports.

A fresh full release campaign is separate from ordinary feature-task completion. Execute it
at an explicitly designated release candidate, once for that candidate's relevant inputs;
if those inputs subsequently change, rerun the invalidated obligations. Do not turn ordinary
task heads or documentation commits into implicit release candidates or repeatedly run a full
release campaign for generic confidence. Existing constitutional release guarantees remain
binding; valid coverage and release execution are distinct questions.

## Six scope decisions

1. **Close decoder only:** run affected decoder/close cases and applicable Rust checks/builds;
   reuse unchanged observation/edit/open/discovery and native evidence.
2. **Shared authentication:** name the changed authentication behavior, exact callers/suites,
   and concrete rejection/misrouting/compatibility failure; run that wider affected coverage.
3. **Sequential/composed completion:** newly run sequential and full composed groups plus
   applicable campaign/build checks; review and reuse valid accepted groups for cumulative
   FR/SC coverage. Aggregate `all` mode alone creates no mandatory execution.
4. **Composed bug changes close:** rerun the failed composed flow and affected close groups;
   broaden only if the fix changes a relevant shared boundary, not because composition failed.
5. **Docs/governance only:** check Markdown/rendering, links/anchors/fences, diff, and changed
   snippet syntax; no GUI, Rust, or native execution absent executable-tooling changes.
6. **Explicit release candidate:** run the independently justified full release campaign once
   at that RC; retain valid checkpoints and rerun invalidated obligations if relevant inputs
   change, without making every feature task a release campaign.
