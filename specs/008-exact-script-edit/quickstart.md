# Quickstart: Validate Schema 2 Exact Editing

**Status:** Implementation validation guide; [tasks.md](tasks.md) assigns acceptance ownership. No Schema 2 product/client acceptance has run. Existing commands below are installed infrastructure; their current Schema 1 scenarios must be migrated/extended before their results can prove this feature. Design/task generation does not implement those scenarios.

Use the [public contract](contracts/mcp-interface.md), [data model](data-model.md) and [execution precedence](contracts/execution.md#refusal-precedence) as the oracle. Follow [TEST_POLICY.md](../../TEST_POLICY.md), not a blanket historical campaign.

## Prerequisites and support

- Complete task derivation, granularity review, consistency analysis and applicable approval before implementation. Deliver one selected implementation task per PR; this guide does not authorize execution of all tasks.
- Test a committed implementation revision in an owned checkout. The [VM setup guide](../../.github/LOCAL_VM.md) provisions the dedicated graphical guest; the wrapper copies committed source, never a mutable human project or host working tree.
- Retain official Godot `4.7.2.stable.official.ed1daf0bf`, macOS 26.6.2 (25G83) arm64, four vCPUs / 6 GiB and exact engine/native provenance. A different guest is development evidence until support equivalence is deliberately approved. No host-GUI fallback or headless replacement for editor witnesses.
- Use exact **Codex CLI 0.153.4** and **OMP 18.5.1** installations, normal host model authentication/approval and the accepted `gpt-6-astra` model path. Set `CODEX` and `OMP` to those executable paths; do not assume globally installed commands have those versions. Keep credentials off the guest and out of evidence/publication.
- Retain source-free startup validator preparation and fresh per-request validation under original budgets. No larger VM or raised timeout is a substitute for the inherited support conditions.

From repository root, inspect versions and the existing wrapper interface without launching a host editor:

```sh
"$CODEX" --version
"$OMP" --version
python3 godot-addon/tests/run_in_vm.py --help
```

## Focused implementation checks

Start with the actual new exact-intent unit cases and directly affected runner/projection tests. Permanent tests must assert transformation, precedence, preservation and effect behavior, not source text or copied catalog wording. Existing wording/incidental tests should not be re-pinned.

The current process-test targets are runnable from `mcp-server/` after migrating their expectations:

```sh
cargo test --locked --test mcp_tools
cargo test --locked --test mcp_transport
```

These prove actual process/catalog/request/carrier boundaries, not live coherence or model use. Include legacy-only/mixed/unknown forms and structural string/revision errors; verify no edit dispatch. Semantic match failures must traverse fresh admission and their no-effect safety check, not a mocked success echo.

At Rust-affecting task completion, use the existing baseline from `mcp-server/`:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo doc --no-deps --locked
cargo build --locked --bins --examples
```

Run the affected existing Python acceptance-consumer/selection tests when their tooling changes. Native builds, ABI checks and historical GUI reruns are conditional on relevant native/fixture changes or unavailable equivalent provenance, not automatic consequences of Schema 2. Do not change CI or run unrelated workflow checks merely for this feature.

## Direct VM scenarios

These existing selectors are execution entrypoints, not completed exact-edit tests. Extend their current owners rather than introduce an exact-edit runner:

- `transport-workflow`: three-profile public positive edits and fresh reads.
- `transport-sources`: exact multiline/Unicode/empty/deletion/no-op cases and supported representation distinctions.
- `transport-bound`: exact source/derived-size bounds and consumed-output timing.
- `preservation`: stale, dirty/equal-text-dirty, divergence, missing evidence and retained-error precedence.
- `interruption`: affected pre/post-effect cancellation/deadline/output-loss and no-effect diagnostic interruption.
- `composed`: fixture/protocol check of the stateful sequence; **not** actual-agent A–E proof.
- `privacy-export`: affected fragment/error disclosure checks; reuse unchanged export evidence after relevant-input review rather than automatically rerunning every export scenario.

Run only task-owned/affected selectors. For example, after migration:

```sh
python3 godot-addon/tests/run_in_vm.py start
REVISION="$(git rev-parse HEAD)"
python3 godot-addon/tests/run_in_vm.py run mcp \
  --revision "$REVISION" --scenario transport-sources --captures
```

Expected result: independent complete-source/lifecycle assertions pass for positive and refusal cases; valid cases stay inside the inherited bounds; artifacts identify exact source/environment and observed surfaces. An exit code, matching substring or native acknowledgment alone is insufficient. Failed/interrupted runs remain failure evidence.

## Real coding-agent workflows

Use a fresh prepared run for each client/profile. The current profiles `workflow`, `sources`, `bound`, `failures`, `reconnect` and `composed` are sufficient ownership locations for extensions; `known`/`observations` remain available when directly affected. The implementation must update their prompts, actual request/result consumers and independent witnesses together. Do not treat an old prompt or a model's claimed use of new fields as proof.

From a clean owned repository checkout:

```sh
REPO="$PWD"
REVISION="$(git rev-parse HEAD)"
RUN_ID="mcp-exact-$(date -u +%Y%m%dT%H%M%SZ)"
ARTIFACTS="$(mktemp -d "$HOME/mcp-exact.XXXXXX")"
python3 godot-addon/tests/run_in_vm.py prepare-mcp \
  --revision "$REVISION" --run-id "$RUN_ID" --artifacts "$ARTIFACTS" \
  --profile workflow
```

Preparation produces the owned guest fixtures, prompt, receipt and fixed relay definition. The actual server, registry, project, Godot and native witnesses stay in the guest; clients and personal authentication stay on the host. No arbitrary remote execution/product transport is added.

### OMP

Use the prepared `mcp.json` as temporary project `.omp/mcp.json` **only** in the owned checkout, preserving existing configuration and `.omp/lsp.yaml`. If a config already exists, do not overwrite it; use another clean owned checkout or a reviewable additive temporary entry with exact restoration.

For a checkout with no existing project MCP file:

```sh
test ! -e "$REPO/.omp/mcp.json" && \
  cp "$ARTIFACTS/mcp.json" "$REPO/.omp/mcp.json"
OMP_MCP_TIMEOUT_MS=15000 OMP_MCP_REQUIRE_READY=1 \
  "$OMP" --cwd "$REPO" --mode json --no-session --approval-mode write \
  < "$ARTIFACTS/prompt.txt"
```

Capture JSONL stdout and stderr separately in private evidence. Normal `write` approval is retained; a prompt-required headless call uses ordinary interactive approval, not bypass flags. Discovery is additive: record the actual selected server/config origin, and require the three `mcp__godot_agent_kit_*` calls to come from the fixed relay. Client-owned output-spill recovery is allowed only as consumption of an already-returned MCP result; shell/file/private-bridge editing is not acceptance.

### Codex

Use a **separate prepared run** and normal interactive authorization; the accepted environment's `exec` policy refused edits and is not the positive mutation recipe. From the owned repository root:

```sh
"$CODEX" --no-alt-screen --ask-for-approval on-request \
  --sandbox read-only --model gpt-6-astra \
  -c "projects={\"$REPO\"={trust_level=\"untrusted\"}}" \
  -c 'mcp_servers.godot_agent_kit.command="python3"' \
  -c "mcp_servers.godot_agent_kit.args=[\"$REPO/godot-addon/tests/run_in_vm.py\",\"mcp-stdio\",\"--run-id\",\"$RUN_ID\"]" \
  -c 'mcp_servers.godot_agent_kit.startup_timeout_sec=10' \
  -c 'mcp_servers.godot_agent_kit.tool_timeout_sec=15'
```

Supply the generated prompt in the TUI. Inspect and approve each intended owned-fixture edit through normal **Allow**, not **Always allow** or global bypass. Keep trust invocation-scoped; for a worktree include the displayed primary repository path in the same untrusted project table where applicable. Preserve this thread's actual tool calls/results, approval events and subsequent model interpretation; do not inspect unrelated histories or authentication files.

### Required agent behavior and finalization

For **each client**, newly demonstrate:

1. Refreshed three-tool Schema 2 discovery and read → localized edit → fresh read on an eligible open, cached-R closed and absent-R closed script. Known targets do not require discovery.
2. Deletion and complete-empty-source → nonempty replacement on all three profiles, not whole-source substitutes for localized changes. Exact multiline/Unicode and unchanged/empty-equal cases may share the `sources` profile.
3. Model-visible consumption of exact source/revision/state and truthful success/refusal/effect results from the existing structured carrier. Do not infer consumption from a raw SDK event or a printed summary.
4. At least one workflow across the clients must intentionally encounter no-match and ambiguity, freshly read, then submit a larger/corrected unique span as a separate intentional request. This cannot be an automatic resend or a scripted private mutation.

After primary `workflow` results, while closed successes have already been verified without opening, obtain the existing durability continuation:

```sh
python3 godot-addon/tests/run_in_vm.py prepare-durability \
  --run-id "$RUN_ID" > "$ARTIFACTS/durability-prompt.txt"
```

Invoke the same actual client/relay with that continuation. Require its post-opening reads, then finalize:

```sh
python3 godot-addon/tests/run_in_vm.py finalize-mcp --run-id "$RUN_ID"
```

Finalization checks required results/durability, cleans owned guest fixtures and retrieves evidence, including failures. Other profiles use their existing finalization directly. Remove only the temporary OMP config entry/file this run owns, preserving human changes, and stop the VM when the acceptance session is finished. Fetching artifacts or stopping the VM alone is not finalization.

## Composed A–E and durability

Prepare `--profile composed` in a new run and drive it with at least one selected actual coding agent. Extend the existing twenty-successful-edit sequence across open/cached/absent profiles to use localized Schema 2 intents with fresh revisions and interleaved dirty, stale, no-match and ambiguous refusals. This sequence is stateful acceptance, not a demand to replay historical independent groups in the same run.

- **A:** Real open exact change, whole intended D/R/B, same document, independently clean/saved state.
- **B:** Dirty and equal-text-dirty human state remains intact; combine missing/ambiguous/empty old text with late preparation-only failures to prove safety precedence.
- **C:** Real native Undo → Save → Redo → Save with independent complete-source/version/dirty observations and reachable prior history. A second edit cannot simulate history.
- **D:** Ordinary close/reopen or later opening, Save where applicable, reparse/rescan and fresh runtime retain the complete source. Closed success must precede later opening; active-game/cached-class hot reload is not claimed.
- **E:** Repeated fresh-revision edits lose no intended change, conceal no conflict and never change the admitted lifecycle. Refusal recovery starts with the original target's fresh read and a new intentional request.

Use existing owned fixture/native actions only for human state, history and durability witnesses; they are not new MCP tools or eligibility repairs. Correlate every model-visible call/result with actual server arguments/outcomes, independently observed source/effects and consumed-output timing. OMP's client-local cancellation is not MCP wire cancellation; use focused direct protocol cases for wire behavior and actual clients for recovery/result interpretation. Automatic reconnect/resend does not authorize replay.

## Acceptance coverage

| Coverage family | Exact scenarios | FR coverage | SC coverage | Evidence required |
| --- | --- | --- | --- | --- |
| Exact transformation and positive lifecycle | US1.1–US1.8 | FR-002–FR-008, FR-010, FR-016–FR-017 | SC-001–SC-002, SC-004, SC-008 | New deterministic edge cases plus both clients' positive/deletion/empty-source workflows; independent full-source and lifecycle witnesses. |
| Literal refusal and bounded source | US2.1–US2.3, US2.7–US2.8 | FR-004–FR-007, FR-010–FR-011, FR-013 | SC-002, SC-005 | Zero/overlapping/multiple matches, whitespace/case/newline/Unicode distinctions, empty-old on nonempty/whitespace source, invalid combined source and cap boundaries; zero-effect preservation. |
| Safety, freshness and uncertainty | US2.4–US2.6, US2.9, US3.4 | FR-003, FR-007, FR-009, FR-011–FR-013, FR-016 | SC-003, SC-005, SC-008 | Changed-path admission and diagnostic no-op precedence, stale-empty/same-text/identity/lifecycle, denied disclosure and affected interruption/effect cases. |
| Native history, durability and composition | US3.1–US3.3 | FR-008, FR-010, FR-012, FR-017–FR-018 | SC-004–SC-005, SC-008 | New actual-agent composed A–E/refusal recovery and relevant independent durability; valid unchanged native foundations may be reused. |
| Single versioned interface and agent usability | US4.1–US4.4 | FR-001–FR-002, FR-014–FR-018 | SC-001, SC-005–SC-008 | New Schema 2 catalog/process/migration checks, both clients using the single shape and outcome guidance; reject legacy/mixed/modes without dispatch. |

This maps all **25 acceptance scenarios, FR-001–FR-018 and SC-001–SC-008**. Routing, source-free disclosure, timing and original-clock/effect ownership apply across relevant rows. The pre-effect diagnostic path additionally needs a late dirty/context/lifecycle/cancel failure while a match error is retained, so tests cannot pass by hiding those failures behind `no_match`.

## Evidence reuse and completion

For every inherited guarantee, record newly executed, valid reused, invalidated-and-rerun or substantively inapplicable evidence with provenance. Inspect actual production paths, native/source/ABI/binary inputs, fixture/witness semantics, runner fingerprints, environment and acceptance meaning. Changed bytes alone or a new task head do not invalidate unrelated evidence.

- Newly required: exact transform and refusal logic, changed public schemas/callers/carrier projections, all required actual-client Schema 2 behaviors and composed interactions, new fragment privacy and original-clock work.
- Directly affected: shared read/edit dispatch and refusal projection consumers, migrated fixture assertions and cancellation/safety boundaries reached by retained errors. Run the affected groups, not every old suite.
- Potentially reusable after review: unchanged native full-source A–E foundations, confinement/authentication/bridge framing, export isolation, source-only validator semantics and independent local discover/observe/open/close/whole-source APIs. Feature 007 client whole-source success does **not** prove Schema 2 use.

No whole historical GUI or fresh release campaign is selected. If an actual change invalidates a shared boundary, name the exact behavior, affected suites and reachable failure before broadening. Native rebuild equivalence cannot be inferred from a matching version string.

Implementation completion requires the full mapped valid evidence plus the proportionate implementation-shape/constitutional review. Evidence belongs here or in the owning task PR, with private source-bearing artifacts excluded from Git. Task/feature completion and PR delivery state remain distinct.

## Planning validation

This section records documentation-stage evidence only. Validate rendered headings/tables, local links/anchors, fenced shell/JSON syntax, complete intended/staged diffs and whitespace. Do not execute the expensive illustrative commands for a design-only change. Product tests, native builds, GUI campaigns and Schema 2 client sessions have not run in this planning stage; later results must be recorded separately from this guide.

**Executed 2026-10-06:** rendered nine changed Markdown surfaces (eight feature
documents plus the changed Current status section): 81 headings and 11 tables.
All 69 local links/anchors resolved; nine shell fences passed `bash -n` and the
one JSON example parsed. Coverage checks found all 25 scenarios, 18 FRs and eight
SCs, with all 16 requirements-quality checkboxes retained and no unresolved
planning/template markers. These are document checks, not execution of the
illustrated product commands.

The installed setup and paths-only prerequisite helpers resolved the verified
Feature 008 directory and plan. Pre/post-plan checks found no extension
configuration, so no registered hook needed dispatch. Constitutional design
review passed without an exception. Publication diff/whitespace evidence is
reported in the design PR; no implementation acceptance or consistency-analysis
attestation is claimed.
