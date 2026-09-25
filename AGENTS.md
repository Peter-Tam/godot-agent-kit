# Agent working agreement

## Authority and scope

Read applicable guidance before changing a component. Resolve conflicts in this order:

1. `.specify/memory/constitution.md`
2. Approved feature specification
3. Approved implementation plan
4. The selected task definition
5. This repository `AGENTS.md`
6. Surrounding-code conventions

Nested guidance cannot waive constitutional safety. On conflict with the constitution,
**stop and surface it**; never silently weaken an invariant. A change of principle requires
a reviewable constitutional amendment and maintainer approval before dependent work.
The approved specification and plan, not this file, determine architecture.

For meaningful features, follow **specify what/why → clarify material ambiguities → plan
and check constitutional compliance → derive testable tasks → implement → verify against
approved artifacts and the constitution**. Verification is a phase, not a presumed command.
Architecture-changing, mutation-semantic, and security-sensitive work must explicitly
record constitutional compliance in planning and review.

## One Spec Kit task, one PR

Every completed individual Spec Kit implementation task in the active approved
`tasks.md` MUST be delivered as its own pull request. Commands, reasoning steps, and subagent work are not separate Spec Kit tasks.
Do not batch prerequisites: finish and deliver each separately. Governance/bootstrap
changes may form one coherent dedicated PR rather than artificial task PRs.

1. Select exactly one task ID; confirm its approved feature, specification, plan, acceptance criteria, and satisfied dependencies. If a prerequisite remains, do not silently implement it in this task.
2. Inspect existing changes, fetch the base, and branch `task/<task-id>-<short-description>` from the updated primary development branch (`main` here), unless repository policy sets another base. Preserve unrelated work; use isolation if needed.
3. Implement only that task and directly necessary tests/docs. Run applicable required validation; mark **only that completed task** in `tasks.md`. If safety, acceptance criteria, or required validation cannot be met, report the blocker instead of claiming completion.
4. Inspect status and the complete intended diff; stage explicit paths, inspect the staged diff, and make a focused commit. Do not commit secrets, transient build/editor state, or unrelated user changes.
5. Push the dedicated branch and open a GitHub PR to the selected base. Include feature/spec reference, task ID/title, what and why, validation, applicable constitutional considerations, and known limitations/follow-up. Follow-up does not excuse unmet acceptance criteria.
6. Report the PR number/URL and validation, then **STOP**. Do not automatically merge or start another task. Proceed only after this PR is reviewed/merged, or the user explicitly authorizes a stacked-PR workflow. Stacking still means one task and one PR per task.

After any PR is confirmed merged, its source branch MUST be deleted from its remote
and from the working clone if present. Switch off the branch before local deletion;
preserve unmerged work and do not disrupt another worktree or dependent open PR.
If safe deletion is blocked, report the blocker rather than discard work or silently
skip cleanup. Verify branch deletion and include it in the merge report. This cleanup
requirement does not authorize automatic merging.

Never force-push without explicit authorization, rewrite shared history, auto-merge, bypass failing required checks, or silently discard either side of a conflict. Do not assume CI or branch protections enforce these rules.

Constrain `/speckit.implement` to **one named task**. Its generic implement-all instruction
does not override this policy: leave all other tasks pending, even if marked `[P]`.
`[P]` is dependency metadata, not permission to batch tasks or bypass the post-PR gate.
If the command cannot honor this scope, implement only the selected task against approved
artifacts without invoking implement-all. Keep applicable mandatory gates/hooks; a template
calling tests optional cannot waive constitutional test gates.

On task branches, explicitly verify the active feature directory before using installed
Spec Kit helpers. Task branch names do not select it: the helper checks
`SPECIFY_FEATURE_DIRECTORY`, then `.specify/feature.json`; `SPECIFY_FEATURE` identifies
a feature but does not set its directory. Set `SPECIFY_FEATURE_DIRECTORY` to the approved
directory when needed. For read-only inspection, if installed, use
`.specify/scripts/bash/check-prerequisites.sh --json --paths-only` with a verified override.
`--paths-only` does not persist the pointer; ordinary checks may persist the override.
Do not require an untracked local helper on a clone without it.

## Transaction and editor safety

The boundary is **agent protocol adapter → protocol-independent automation/transaction
core → Godot integration**. MCP is the first adapter, not the definition of the core.
Keep protocol/transport details at the adapter, Godot APIs at the integration boundary,
and every mutation on common transaction/coherence semantics. Keep tools small,
composable, consistently schematized, and discoverable. Route to the intended
project/editor explicitly; refuse ambiguous targets.

Treat **D** (disk source), **R** (loaded Godot Resource/Script), and **B** (visible editor
buffer) as independent authorities. For open-script edits, success requires observed
`D == R == B` across applicable surfaces. Protect unsaved human edits: detect dirty or
conflicting editor state wherever exposed, refuse unresolved conflict or missing safety
observability, and guard stale read-modify-write revisions (or equivalent protection).
No default force overwrite. Godot editor APIs and native editor transaction semantics
MUST be the canonical mutation route wherever possible; alternatives MUST preserve the
same transaction and coherence guarantees. A file write, reload, rescan, parse, or
runtime agreement alone does not prove coherence.

Define preconditions, mutation boundary, synchronization, verification, and deterministic
structured outcomes per mutating operation. Distinguish acceptance, application,
synchronization, and **independently observed** postcondition verification; acknowledgment
is not success. Report revisions, observed or unavailable D/R/B surfaces, dirty/sync state,
diagnostics, UndoRedo participation, and partial application. Distinguish conflict, stale
Resource/buffer, revision mismatch, parse error, timeout, disconnection, unsupported
capability, approval requirement, and unavailable runtime where applicable. Refuse instead
of inventing guarantees; timeout/cancellation does not imply rollback, and retries must
be safe or explicitly documented as non-idempotent. Success must survive Save,
close/reopen, reparse, rescan, and runtime launch wherever applicable.

## Verification and release gates

Layer deterministic, isolated tests: unit tests for pure logic, integration tests for protocol/editor boundaries and changed state transitions, **real-Godot live-editor tests** for coherence. Use explicit deadlines and event-based synchronization, record regression cases for discovered failures, and make diagnostic evidence identify the actual transaction stage and observed D/R/B surfaces. Disk-only checks, headless runtime, and mocks cannot establish visible-buffer coherence.

Mutation-related functionality MUST maintain real-Godot live-editor regression coverage for scenarios A–E, and all applicable gates MUST pass before a mutation feature is complete or released; C applies whenever UndoRedo is claimed:

- **A — Clean open-buffer edit:** D, R, and B converge without human reconciliation.
- **B — Dirty human-buffer conflict:** preserve the unsaved edit; refuse safely or use an explicitly designed resolution workflow, never silently lose it.
- **C — Real Undo/Redo:** verify apply → Undo → Redo transitions across applicable surfaces through Godot's editing history.
- **D — Close/reopen persistence:** reported successful edits survive reopening without stale/reverted state.
- **E — Sequential edit stress:** repeated edits do not disappear, diverge, conceal conflicts, or revert on Save.

Also verify Save/reparse/rescan/runtime durability wherever applicable. Record a justification for an inapplicable surface/scenario; lack of observability is not proof. Document verified guarantees and limitations, not aspirations. State exact supported Godot versions and cover them in CI and real-editor testing before claiming support; do not claim untested versions.

## Conditional implementation guidance

Apply these only when the relevant implementation exists under an approved plan; do not infer a runnable Cargo/Godot project or select crates, SDKs, ports, transport, wire format, package topology, test framework, license, versions, MSRV, bots, or release infrastructure here.

- **Godot:** Use public editor APIs. Observe `ScriptEditor`/`ScriptEditorBase`/`CodeEdit`
  independently of Script resources and disk; `EditorFileSystem` scans/signals and
  `ResourceLoader` caches do not guarantee visible-buffer convergence. Use
  `EditorUndoRedoManager` where appropriate and prove actual Undo/Redo, not registration.
  If a GDScript EditorPlugin is selected, use conventional `addons/<name>/plugin.cfg` and
  `@tool`, clean up registrations/signals/nodes on disable/exit, and keep editor tooling
  out of gameplay authority. Neither `addons/` nor `@tool` excludes tooling from
  production exports: explicitly test the development/export boundary.
- **Rust:** The server should be primarily Rust; a thin GDScript EditorPlugin is the
  expected start, but an alternative MAY be selected only when evidence favors its fit
  to Godot Editor APIs and it preserves the same guarantees. Follow approved package
  boundaries and standard Cargo layout; separate reusable logic from executable wiring
  without gratuitous crates. Use `Result` for recoverable errors, meaningful public
  error contracts/docs (Errors, Panics, Safety where applicable), and avoid unjustified
  `unsafe`; document safety invariants for any justified `unsafe`. Validate serialized/
  schema inputs semantically at adapters. Keep blocking work off async workers; define
  cancellation-safe mutation boundaries explicitly without assuming rollback. Produce
  correlated, structured, redacted diagnostics.
- **MCP:** Select released compatible specification/SDK versions in the plan. Keep
  transport, DTOs, and schemas at the adapter; validate machine-readable inputs and
  structured outputs, advertise only supported capabilities, and distinguish
  protocol/request errors from tool execution failures without losing core outcome detail.
  For stdio, reserve stdout for protocol and send logs to stderr. If HTTP is selected,
  follow the selected specification's Origin/auth requirements.

Once a Cargo application exists, track `Cargo.lock` and use locked dependency resolution in CI; document any toolchain/edition/MSRV claim, test any promised MSRV, and test valid approved feature combinations, not blanket `--all-features`. Run applicable baseline checks (add `--workspace` when coverage of actual workspace members requires it):

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo doc --no-deps --locked
```

Default `cargo test` includes doctests; `cargo test --all-targets` alone does **not**. If using `--all-targets`, run documentation tests separately with `--doc`. Commands are conditional on the later planned Cargo setup, not claims of currently configured checks.

## Dependencies, security, compatibility

Justify dependencies against requirement and plan; review maintenance, security
advisories, compatible licenses, and provenance. Develop independently from public
APIs/specifications rather than copying unrelated projects. Gate arbitrary
execution/evaluation, force writes, outside-project access, and network access explicitly.
Default to local-only operation and no telemetry; approvals never replace dirty-buffer
detection or transaction safety. Minimize project-content exposure in logs. If GitHub
workflows are introduced, grant tokens least permissions, pin actions to full commit SHAs,
and never run untrusted PR code with secrets.

Treat public tool schemas, structured errors, and outcomes as compatibility surfaces: deliberately version and document migration for breaking changes, evolve additively where safe, and do not preserve unsafe behavior just for compatibility. Keep changes task-scoped and leave unrelated user files alone.
