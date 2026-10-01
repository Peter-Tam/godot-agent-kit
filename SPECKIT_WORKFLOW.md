# Spec Kit Workflow

## Scope and authority

This is repository working policy delegated by [AGENTS.md](AGENTS.md#spec-kit-workflow),
with AGENTS.md-level authority only. It cannot override, in precedence order, the
constitution, approved specification, approved plan, or selected task. All existing
implementation, safety, completion, and delivery rules in `AGENTS.md` remain in force.

This document governs agent actions before and after existing `/project.specify-next`,
`/speckit.specify`, `/speckit.clarify`, `/speckit.plan`, `/speckit.tasks`, and
`/speckit.analyze` workflows, plus analysis-currentness recognition during
`/speckit.implement` prerequisites. It does not redefine the commands' internal work.
Execute the installed commands as defined; do not fork or modify them to implement this
policy. Publication finishes the requested stage; it does not authorize the next stage.

## A. Publish the intended stage delta

Successful completion of a repository-mutating design stage includes publishing its
intended repository delta before reporting completion. This applies to specify, clarify
when it changes files, plan, tasks, and the post-analysis metadata in section F.

At the beginning of each stage, the agent MUST:

1. Inspect `git status --short` and retain a baseline of pre-existing changed and
   untracked paths, including staged changes. Retain enough of their diffs/content to
   distinguish later stage edits from existing user edits in the same files.
2. Identify the current branch and active feature directory using the existing
   `AGENTS.md` feature-directory guidance. For a creation stage, identify the resulting
   directory when Spec Kit produces it; do not infer it from the branch name.
3. Preserve all unrelated and pre-existing user work throughout the stage.

After the command finishes, before publication, the agent MUST:

1. Inspect the complete intended diff, including the contents of newly created files.
2. Stage ONLY files or separable hunks attributable to this stage, using explicit paths.
   NEVER use `git add .` or `git add -A`. A path appearing in the stage output does not
   authorize staging pre-existing user edits in that path.
3. Keep unrelated/pre-existing staged changes out of the commit as well. Preserve their
   content and staging state; refuse publication if the intended delta cannot be safely
   isolated from them or from other user edits.
4. Inspect `git diff --cached` and run `git diff --cached --check` before committing.

Never commit `.specify/feature.json`, credentials, local research evidence, temporary
projects, build/editor artifacts, or unrelated changes. Intended design documents such
as `research.md` are not permission to publish raw private/local research evidence.
Never force-push, rewrite shared history, or auto-merge. If safe publication or a required
remote operation fails, report the concrete incomplete publication step; do not claim
successful completion or discard work.

## B. Use the dedicated feature-design branch

Feature-design work MUST NOT be committed directly to `main`. Before a design-stage
commit, use the dedicated branch `spec/<feature-directory-basename>`.

If `/project.specify-next` or `/speckit.specify` has just created a feature while the
checkout is still on `main`, create or switch to that branch before committing. For
example, `specs/006-save-project-gdscript` maps to `spec/006-save-project-gdscript`.
The directory produced by Spec Kit is authoritative; do not duplicate feature-number
selection logic.

Reuse the correct dedicated feature branch if already on it. When switching from another
branch, preserve the baseline work and do not mix another feature's commits into this
feature's delivery. If that cannot be done safely, report the concrete blocker rather
than commit to the wrong branch or discard changes.

## C. Commit and push each changed design stage

When a stage produces intended repository changes, create one focused commit with a
concise, stage-appropriate message, then push the current branch. Suggested messages
(not mandatory wording):

- `docs(spec): specify <feature>`
- `docs(spec): clarify <feature>`
- `docs(plan): design <feature>`
- `docs(tasks): derive <feature> tasks`
- `docs(analysis): record <feature> consistency analysis`

If the branch has no upstream, use `git push -u origin HEAD`; otherwise use `git push`.
After pushing, verify that the pushed remote branch resolves to the local HEAD, using
`git ls-remote` and `git rev-parse HEAD`, not just a cached remote-tracking ref.

If there is no repository delta, do not manufacture an ordinary content commit merely
to record that a command ran. Section F deliberately permits a post-analysis attestation
commit even though the analyzer itself is read-only; it does not require an empty commit.

## D. Create one draft PR for `/project.specify-next`

After `/project.specify-next` selects one feature, delegates to the existing
`/speckit.specify`, produces a valid specification/checklist, and commits/pushes the
feature branch, the agent MUST ensure one GitHub design PR exists. The delegated specify
stage publishes its delta once; the wrapper does not duplicate that commit.

Look for an **OPEN** PR whose head is the **exact current branch** and whose base is
`main`. If one exists, reuse it. If none exists, create exactly one draft PR with
`gh pr create --draft`, explicitly targeting that head and `main`. Never create a
duplicate PR, turn an existing ready/non-draft PR back into draft, automatically mark a
draft ready, or auto-merge.

The initial body MUST reflect current truth: feature and spec path, requirements/checklist
state, documentation-only status, and whether clarify, plan, tasks, and analyze have run.
Do not claim implementation or product/version support from design artifacts.

Report the PR URL along with the command's required feature/spec/roadmap report, then
**STOP**. Do not automatically continue to clarify, plan, tasks, analyze, implement,
another feature, or another `/project.specify-next`.

## E. Refresh the existing design PR

After clarify, plan, tasks, or analysis publication, look for an **OPEN** design PR with
the exact current feature branch as head and `main` as base. If one exists, refresh its
description from **current repository state**, not an appended chronological transcript.
Keep its existing draft/ready state. Section H covers a no-op clarify invocation.

The refreshed description MUST report what is currently true:

- Feature/spec path and specification/requirements-checklist status.
- Material clarification status; do not infer that clarify ran just from file existence.
- Planning, research, data-model, and contracts status, identifying applicable artifacts.
- Task count and granularity-review status.
- Consistency-analysis status, including whether the section F record is current.
- An explicit statement that implementation has not started for this design-only work;
  never substitute that statement for the actual state if implementation already exists.
- Feature/phase lifecycle truth, separate from GitHub review/merge delivery metadata as
  required by `AGENTS.md`.
- Validation actually executed, without implementation or support claims.

Remove stale claims such as "No plan or task list has been generated" once those artifacts
exist. If a stage's execution cannot be established, say so rather than inventing history.
If there is no open design PR because the developer invoked `/speckit.*` directly,
commit/push normally and do not create one. Only `/project.specify-next` automatically
creates the initial design PR. A refresh never creates a second PR.

## F. Persist the completed analysis result

Do not change `/speckit.analyze`. Its actual analysis pass remains strictly read-only and
produces its normal report. Only **after that pass and report are complete** does the
repository agent write publication metadata to `specs/<feature>/analysis.md`.

This small attestation is workflow metadata, not a fourth semantic design artifact and
not an input to analysis itself. Do not copy the full report or private reasoning. Use
this fixed structure, replacing placeholders with the immediately completed report's
supported values:

```markdown
# Consistency Analysis Record

**Schema:** 1
**Result:** pass | blocked
**Blocking findings:** N
**Analysis commit/input state:** <full HEAD at analysis; clean or modified listed inputs>

## Input fingerprints

| Input | SHA-256 |
| --- | --- |
| spec.md | <sha256> |
| plan.md | <sha256> |
| tasks.md | <sha256> |
| .specify/memory/constitution.md | <sha256> |
| AGENTS.md | <sha256> |
| SPECKIT_WORKFLOW.md | <sha256> |

## Findings summary

- Critical: N
- High: N
- Medium: N
- Low: N
```

Schema 1 requires all displayed fields and exactly these six input rows, once each.
`spec.md`, `plan.md`, and `tasks.md` are relative to the active feature directory; the
other three paths are repository-root-relative. Each fingerprint is a 64-character
hexadecimal SHA-256 of the exact raw file contents, not a Git blob ID. Counts are
non-negative integers for unresolved findings; `Result` is one value, `pass` or `blocked`.
Record the full analysis-time HEAD and whether the listed inputs had uncommitted changes;
the fingerprints identify their exact bytes even when the working tree was not clean.

Implementation-readiness classification MUST follow the completed report:

- Any unresolved CRITICAL or HIGH finding means `Result: blocked`.
- LOW/MEDIUM findings alone are non-blocking unless the existing analysis report explicitly
  establishes otherwise. If it does, add a brief finding identifier/reason to the summary
  and include it in `Blocking findings`; do not invent a new blocker.
- `Blocking findings` counts all unresolved blockers. `pass` requires zero blockers and
  zero Critical/High findings; otherwise use `blocked`. Zero findings means `pass`.

Never record a pass unsupported by the immediately completed analysis. Hash the exact
current contents that were analyzed, after any approved remediation and rerun. If any
input changes between analysis and attestation, rerun analysis on those changed bytes
before certifying them. `analysis.md` is not hashed, so there is no self-reference loop.
The commit/input-state field is provenance, not a requirement that later HEAD or branch
identity match the analysis-time checkout.

Stage the attestation as the analysis publication delta, inspect/check the staged diff,
commit, push, and refresh the existing design PR under sections A–E. Publish a blocked
result truthfully too; publication does not turn it into implementation readiness. This
write happens after the analyzer has finished, not as an edit during its read-only pass.

## G. Recognize current analysis before implementation

Do not modify `/speckit.implement`. Before deciding that analysis needs to run, its
implementing agent MUST inspect the active feature's `analysis.md` and recompute SHA-256
for **every recorded input**, including all six required schema-1 inputs.

Consistency analysis is already satisfied when the record parses correctly, says
`Result: pass`, records no unresolved blockers (including no Critical/High findings),
and every fingerprint matches the current files. Missing/duplicate rows, unsupported
schema, invalid fields/hashes/counts, or contradictory result/counts make the record
malformed; an unavailable input cannot establish a match.

Do NOT rerun `/speckit.analyze` merely because this is a new chat/session, the design PR
merged, its source branch was deleted, the checkout is now on `main`, or implementation
has just started. A stale prose claim that analysis has not run is not grounds for
repeating a valid current pass. Do not require the recorded commit to equal current HEAD
or the old branch to exist.

A fresh analysis is required only when the record is absent, malformed, blocked, or an
input fingerprint no longer matches. A spec, plan, tasks, constitution, `AGENTS.md`, or
`SPECKIT_WORKFLOW.md` change therefore invalidates the prior currentness claim. Do not
backfill a pass from remembered chat results or from hashes calculated without an analysis
of those exact inputs.

The workflow remains **tasks → granularity review → analyze → implement**. Apply the
existing `AGENTS.md` granularity review after task generation or material revision, before
analysis and implementation. Recognizing an unchanged completed gate skips only duplicate
analysis, not artifact approval, task selection, dependencies, or any other existing gate.
Select exactly one dependency-ready task under the one-task-one-PR rules.

## H. Leave no-op clarification alone

If `/speckit.clarify` asks zero questions and changes no files, do not create an empty
ordinary commit or push for the sake of pushing. An existing PR body may remain unchanged
unless repository lifecycle truth changed. If any intended spec, checklist, or status file
changed, publish normally under sections A–E.

## I. Keep remediation outside the read-only pass

When analysis finds an issue, report it normally without silently editing spec, plan, or
tasks during analysis. If the user later authorizes remediation, edit the affected design
files within that scope and publish their intended delta under sections A–E. Changed input
hashes automatically make the old attestation stale. Rerun analysis after remediation, then
replace `analysis.md` with the newly supported result and fingerprints and publish it under
section F. Do not invent a separate remediation task or workflow mechanism.

## J. Leave implementation delivery unchanged

Continue following [AGENTS.md's one-task-one-PR rules](AGENTS.md#one-spec-kit-task-one-pr):
one selected Spec Kit task, its task branch, implementation with directly required
tests/docs, focused commit, push, one implementation PR, then **STOP**. No automatic merge
or next task. This document does not redefine implementation delivery; its only
implementation-related addition is recognition of a current analysis attestation.

## Principle XIII maintenance justification

[Principle XIII](.specify/memory/constitution.md#xiii-justify-complexity-with-concrete-present-risk)
requires a concrete present need and a justified cost. Three failures are demonstrated:

1. Feature 005 analysis had covered 25/25 requirements with no actionable findings,
   ambiguities, constitutional conflicts, or unmapped tasks, but a later session repeated
   it after merge because stale prose said it had not run and no durable currentness
   record was available.
2. Each design phase repeatedly required manual diff/stage/commit/push/PR bookkeeping.
3. Design PR descriptions repeatedly became stale as artifacts advanced.

The simplest credible alternative is the existing manual publication process plus chat
reports or a prose "analysis passed" note. That has already lost analysis state and PR
truth; an unhashed note also cannot establish whether the analyzed inputs changed.
Existing commands produce the design artifacts/report but do not supply this repository's
publication and durable-currentness policy.

The selected cost is one linked repository-policy document, a small per-feature analysis
attestation, and agent-performed Git/PR publication and six-file fingerprint verification.
Keep the details here rather than expanding `AGENTS.md` into a duplicate workflow manual.
Maintainers review this policy and small records; no implementation code, helper, command
fork, hook system, database, service, or CI/provider gate is needed. This cost is justified
now by the repeated failures, without adding an approval gate or weakening any existing
constitutional safety or one-task-one-PR rule.

Reject modifying installed Spec Kit/project commands, adding custom OMP commands, helper
scripts/frameworks, databases, bots/services, or mandatory hosted automation: they add
installation, compatibility, execution, or operational obligations without an additional
required guarantee. Policy uses existing Git/GitHub operations; it creates no workflow
engine, extension configuration/hooks, integration-manifest changes, or CI topology.
