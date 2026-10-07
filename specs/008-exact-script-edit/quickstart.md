# Quickstart: Validate Schema 2 Exact Editing

**Status:** T001 is complete: the Schema 2 implementation, scoped VM safety/history/durability evidence and current acceptance in both Codex CLI 0.153.4 and OMP 18.5.1 are recorded below. Codex uses the accepted private official npm distribution, not the known-failing Homebrew companion route. T002 and T003 remain pending; full-feature acceptance is not claimed.

Use the [public contract](contracts/mcp-interface.md), [data model](data-model.md) and [execution precedence](contracts/execution.md#refusal-precedence) as the oracle. Follow [TEST_POLICY.md](../../TEST_POLICY.md), not a blanket historical campaign.

## Prerequisites and support

- Complete task derivation, granularity review, consistency analysis and applicable approval before implementation. Deliver one selected implementation task per PR; this guide does not authorize execution of all tasks.
- Test a committed implementation revision in an owned checkout. The [VM setup guide](../../.github/LOCAL_VM.md) provisions the dedicated graphical guest; the wrapper copies committed source, never a mutable human project or host working tree.
- Retain official Godot `4.7.2.stable.official.ed1daf0bf`, macOS 26.6.2 (25G83) arm64, four vCPUs / 6 GiB and exact engine/native provenance. A different guest is development evidence until support equivalence is deliberately approved. No host-GUI fallback or headless replacement for editor witnesses.
- Use exact **Codex CLI 0.153.4** and **OMP 18.5.1** installations, normal host model authentication/approval and the accepted `gpt-6-astra` model path. Set `CODEX` and `OMP` to those executable paths; do not assume globally installed commands have those versions. Keep credentials off the guest and out of evidence/publication.
- For Codex, reuse the [Phase 3 private official npm execution route](../007-mcp-script-workflow/quickstart.md#real-coding-agent-clients), restoring the exact `@openai/codex@0.153.4` private installation if needed. Set `CODEX` to that installation's `node_modules/@openai/codex/bin/codex.js`, not the Homebrew executable. Phase 3 already records the Homebrew companion's pre-`--help` stall; it is not a new product or authorization prerequisite. Do not change global installations, quarantine, Gatekeeper or personal trust to use the private route.
- Retain source-free startup validator preparation and fresh per-request validation under original budgets. No larger VM or raised timeout is a substitute for the inherited support conditions.

From repository root, inspect versions and the existing wrapper interface without launching a host editor:

```sh
"$CODEX" --version
"$OMP" --version
python3 godot-addon/tests/run_in_vm.py --help
```

## Focused implementation checks

Start with the actual new exact-intent unit cases and directly affected runner/projection tests. Permanent tests must assert transformation, precedence, preservation and effect behavior, not source text or copied catalog wording. Existing wording/incidental tests should not be re-pinned.

The migrated process-test targets are runnable from `mcp-server/`:

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

Use these existing selectors according to the selected task's ownership. T001 has migrated the current MCP callers and added the focused exact-edit baseline; broader adversarial and composed acceptance remains assigned to T002–T003:

- `transport-workflow`: three-profile public positive edits and fresh reads.
- `transport-sources`: exact multiline/Unicode/empty/deletion/no-op cases and supported representation distinctions.
- `transport-bound`: exact source/derived-size bounds and consumed-output timing.
- `preservation`: stale, dirty/equal-text-dirty, divergence, missing evidence and retained-error precedence.
- `preservation-exact-baseline`: T001's matching/input refusals, stale/dirty/missing-evidence precedence and late dirty-Resource guard while a match error is retained.
- `interruption`: affected pre/post-effect cancellation/deadline/output-loss and no-effect diagnostic interruption.
- `composed`: fixture/protocol check of the stateful sequence; **not** actual-agent A–E proof.
- `privacy-export`: affected fragment/error disclosure checks; reuse unchanged export evidence after relevant-input review rather than automatically rerunning every export scenario.

Run only task-owned/affected selectors. For example:

```sh
python3 godot-addon/tests/run_in_vm.py start
REVISION="$(git rev-parse HEAD)"
python3 godot-addon/tests/run_in_vm.py run mcp \
  --revision "$REVISION" --scenario transport-sources --captures
```

Expected result: independent complete-source/lifecycle assertions pass for positive and refusal cases; valid cases stay inside the inherited bounds; artifacts identify exact source/environment and observed surfaces. An exit code, matching substring or native acknowledgment alone is insufficient. Failed/interrupted runs remain failure evidence.

## Real coding-agent workflows

Use a fresh prepared run for each client/profile. The current profiles `workflow`, `sources`, `bound`, `failures`, `reconnect` and `composed` remain the existing ownership locations; `known`/`observations` are available when directly affected. T001's migrated prompts, request/result consumers and independent witnesses cover the exact-edit positives. Broader recovery and composed scenarios require the extensions and acceptance assigned to T002–T003. Do not treat an old prompt or a model's claimed use of new fields as proof.

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

## T001 implementation evidence (2026-10-07)

**T001 completion: complete.** Production and acceptance inputs are at
`6b41fcfd446b14461aafe94303aa2a8eafa0fc98` on
`task/T001-schema2-exact-edit`; subsequent changes record documentation and
completion state only. Both selected clients satisfy the atomic cutover
criterion. No T002/T003 or full-feature completion is claimed.

### Implemented boundary and static checks

The public catalog, strict decoder, output schemas and envelopes now use Schema 2
and one literal `old_string`/`new_string` pair. Current MCP callers and consumers
are migrated together. Local whole-source Rust/CLI/private interfaces retain
their separate contracts. No legacy translator, second mutation engine,
dependency, native rebuild requirement or protocol revision was added.

The checked core counts overlapping starts with at most two substring searches,
checks the combined byte cap before constructing the result, and preserves exact
surroundings. Fresh authenticated acquisition and revision comparison precede
derivation. A retained match error traverses existing unchanged execution and may
be exposed only after independently verified zero-effect results; actual safety,
disclosure, validation, interruption and effect uncertainty take precedence.

Earlier task execution passed Rust formatting, Clippy with warnings denied,
affected unit/MCP process tests, rustdoc and locked binary/example builds.
The aggregate Rust run hit the existing opening-timeout assertion in
`bridge_boundary::late_completion_result_cannot_upgrade_a_delivered_known_partial_timeout`;
that unchanged case passed in isolation and the remaining targets and doctests
passed separately. The aggregate failure is not a passing aggregate run.
Affected Python consumers passed: 56 tests initially, followed by focused
18-, 4-, 11- and 17-test checks after their respective fixture corrections.
These overlapping executions are not added into a distinct-test total.

### Direct live-editor evidence

All runs used the fixed wrapper, official Godot
`4.7.2.stable.official.ed1daf0bf`, macOS 26.6.2 arm64, four vCPUs / 6 GiB.
Environment identity:
`5be872888a3eacb7798de40e70d66e1f438d908e00cf714575d65b2632596ced`.
Private artifact roots below are relative to
`~/.local/state/godot-agent-kit-vm/artifacts/`; source-bearing records and captures
are deliberately not committed.

| Selector | Tested source | Passed records | Maximum consumed edit response | Artifact root |
| --- | --- | ---: | ---: | --- |
| `transport-workflow` | `38e1d1b878c04f09436f00b5572dac502c380f94` | 71 | 5.924257 s | `schema2-workflow-smoke-1/20261007T000406Z-dcde21c8bb88` |
| `preservation-exact-baseline` | `a45a8f1ed165bfb922e78e42ddd90d2ac9e630e4` | 42 | 4.718693 s | `schema2-safety-smoke-3/20261007T001251Z-45030e952eb6` |
| `transport-sources` | `6b41fcfd446b14461aafe94303aa2a8eafa0fc98` | 34 | 6.815210 s | `schema2-sources-smoke-3/20261007T002830Z-c45638ebc06c` |
| `transport-bound` | `6b41fcfd446b14461aafe94303aa2a8eafa0fc98` | 10 | 6.585374 s | `schema2-bound-smoke-1/20261007T002912Z-460add7442e9` |
| `privacy-authorized` | `6b41fcfd446b14461aafe94303aa2a8eafa0fc98` | 21 | 6.521613 s | `schema2-privacy-smoke-1/20261007T003003Z-1316080e252a` |

Workflow witnesses cover complete intended D/R/B and saved state on the same
open document, D/R with continuing B absence for cached closed scripts, and D
with positive R/B absence for uncached closed scripts. The changed path includes
localized replacement/deletion, complete-source deletion, empty unchanged and
empty-to-nonempty edits, real native history/prior-history, and applicable
Save/reopen/reparse/rescan/fresh-runtime durability. The focused preservation
run includes matching/input refusals, stale/dirty/missing-evidence precedence,
invalid complete derived source, and a late dirty-Resource guard overriding a
retained match error. Privacy checks preserve authorized digest evidence without
exposing source fragments or match feedback.

### Actual OMP acceptance

OMP **18.5.1**, retained CLI SHA-256
`fc62b280c50923f779e9af14e0ba12d24cbf3db6b2757106a8ad127b3a3274a9`,
used `openai-codex/gpt-6-astra`, normal `write` approval and a temporary project
MCP configuration in an owned detached checkout. `OMP_MCP_TIMEOUT_MS=15000` and
`OMP_MCP_REQUIRE_READY=1` affect the client relay, not the product's five/ten-second
bounds. All accepted runs used source
`6b41fcfd446b14461aafe94303aa2a8eafa0fc98`.

| Profile | Finalizer records | Correlated model-visible results | Maximum consumed edit response | VM artifact root |
| --- | ---: | ---: | ---: | --- |
| `workflow`, including later-opening reads | 53 | 47 | 7.554557 s | `schema2-omp-workflow-2/20261007T004848Z-88e9e81278bd` |
| `sources` | 26 | 25 | 6.123061 s | `schema2-omp-sources-1/20261007T005425Z-23c820119549` |
| `known` | 9 | 8 | 5.003475 s | `schema2-omp-known-1/20261007T005716Z-90720b62ab98` |

All 80 model-visible results correlate with actual server calls; none was
delivery-unavailable. Private client launch records, JSONL, normalized carrier
events and correlation results are under `schema2-t001-_pgpqenm/omp-workflow-2`,
`omp-sources` and `omp-known`.

OMP refreshed the three-tool Schema 2 catalog, used localized spans and each
latest read revision, and fresh-read after every edit. Each open/cached/absent
target completed all six workflow edits without switching lifecycle. Source
cases preserve multiline/tab/Unicode bytes and anchored insertion; complete
empty-source and unchanged intent are distinguished from missing text.
The known-path workflow skipped discovery, intentionally used the original
stale revision once, consumed `revision_mismatch` / `not_applied` /
`fresh_read`, then read rather than replaying the rejected request.

Qualitative review found the structured carrier sufficient: OMP distinguished
`source_origin` from per-surface provenance, open D/R/B agreement from positively
absent closed surfaces, unchanged from applied results, and source equality from
loaded-class/runtime refresh. Its later-opening reads consumed the now-open
buffer evidence. It explicitly retained non-atomic/stability limitations instead
of inferring unobserved coherence. Model summaries alone were not accepted as
proof; independent complete-source/lifecycle witnesses and call correlation
established the observed results.

### Actual Codex acceptance

The private official **Codex CLI 0.153.4** npm distribution used
**gpt-6-astra**, invocation-scoped untrusted project entries, the read-only shell
sandbox and ordinary `on-request` approval. All 29 edit requests received
individual **Allow** decisions, including the intentional stale-revision probe;
no session-wide or permanent grant was used. Source and VM environment identity
match the accepted OMP runs.

| Profile | Finalizer records | Correlated client results | Maximum consumed edit response | VM artifact root |
| --- | ---: | ---: | ---: | --- |
| `workflow`, including later-opening reads | 54 | 48 | 6.682354 s | `schema2-codex-workflow-2/20261007T050939Z-130a596811e7` |
| `sources` | 26 | 25 | 6.920724 s | `schema2-codex-sources-1/20261007T052740Z-5500f70c05c1` |
| `known` | 9 | 8 | 5.245178 s | `schema2-codex-known-1/20261007T053639Z-16c96c7598f5` |

All 81 actual calls/results correlate with the server records; none was
delivery-unavailable. Selected-thread transcripts, official TUI approval logs,
launch records, normalized carrier evidence and correlation results are under
`schema2-t001-_pgpqenm/codex-workflow-2`, `codex-sources` and `codex-known`.
The refreshed catalog contains exactly the three tools, root output Schema 2
and only the five required exact-edit fields plus optional `session_id`.
Codex requested MCP `2025-06-18` and accepted the server's `2025-11-25`.

The workflow completed all six edits on each open/cached/absent target, followed
by successful later-opening reads. One concurrent read received truthful
`server_busy` / not-queued information; the client issued a separate read and
obtained the final state without replaying a mutation. Source cases establish
multiline/tab/Unicode preservation, anchored insertion and complete-empty-source
semantics. The known-path conversation skipped discovery, made one intentional
stale equal-text request, interpreted `revision_mismatch` / `not_applied` /
`fresh_read`, and then read the unchanged final source.

Carrier review follows the accepted Phase 3 structured-result conversion and
also checks actual code-mode output. The workflow printed 44 full envelopes;
four open-edit results were consumed by model-authored code and printed as
selected outcome/application/history views, with persistence/diagnostics where
selected. Those views were compared against the complete returned client result
and server record, together with subsequent exact arguments, fresh reads and
independent witnesses. Unprinted fields are not claimed as reasoning-context
text. Source and known runs printed all 25 and eight complete envelopes.
Neither raw SDK events nor model assurances alone establish these results.

Qualitative review confirmed correct exact-span/current-revision use, empty
versus unavailable source, changed versus unchanged effects, lifecycle
preservation, and refusal recovery. Codex distinguished `editor_buffer` source
origin from disk provenance, reported applicable D/R/B equality after later
opening, and retained non-atomic/stability and loaded-class limitations.
Real native-history and closed-lifecycle captures were inspected alongside the
independent state witnesses. The finalizers passed and the owned VM was stopped.

### Codex execution route and excluded failures

Use the already accepted private official **0.153.4** npm distribution, not
Homebrew. The retained `t002-codex-0.153.4-7sapgq72/package-lock.json` pins
`@openai/codex@0.153.4` and the official
`@openai/codex@0.153.4-darwin-arm64` platform archive with SHA-512 integrity.
The private installation was restored with `npm ci --ignore-scripts` using that
lock. It reports `codex-cli 0.153.4`; its companion's `--help` returns normally
and its signature verifies. No global/Homebrew installation or macOS security
setting is changed by this accepted route.

Private CLI SHA-256:
`b973d440acac501fd2594a43e7ca9ce41e0a65b9dfb28d0d7a7837c99e1261e3`.
Companion SHA-256:
`d8a2222e017342718d16a5dbe092921c628961f812f62f42036b8d960e1ffe56`,
matching the previously recorded companion bytes. Package archive integrity,
signature/version/help observations and exact paths are retained in
`schema2-t001-_pgpqenm/codex-provenance.json`.

An initial attempt used the wrong, Homebrew execution route and reproduced the
[already documented limitation](../007-mcp-script-workflow/quickstart.md#real-coding-agent-clients)
before any project MCP call. Disabling its host and isolating its catalog did
not provide usable tools. The maintainer-authorized removal of that Homebrew
helper's quarantine attribute also did not resolve it; no other security change
was made. This is excluded setup evidence, not a new product, permission or
acceptance blocker. Only a failure of the accepted private route could establish
a new execution blocker.

Owned client evidence is under `schema2-t001-_pgpqenm/codex-workflow`.
Finalization was attempted and retrieved
`schema2-codex-workflow-1/20261007T011817Z-1ea0f8b2e1a5`, but exited 70 with
`Owned MCP relay/control failed`; no successful finalization or fixture
acceptance is claimed for it. The dedicated VM was subsequently stopped.

Earlier failed safety runs exposed fixture disclosure/descriptor assumptions,
which were corrected before the 42-record pass. Expanded seven-editor source
fixtures hit validation deadlines; the original three-target source scope was
restored without changing product bounds, while workflow retained all required
edits on all three lifecycle profiles. OMP's first workflow disconnected during
the absent target and remains incomplete; the second run used fresh fixtures
and passed. Failed/incomplete runs are retained and excluded from all counts.

### Evidence currentness and implementation shape

The direct workflow's later source deltas concern separate preservation/source
fixture corrections and failure diagnostics, not its accepted workflow path.
The successful safety run already contains its disclosure and optional-descriptor
corrections. Source, bound, privacy and OMP results ran at the current code head.
No production Rust, addon, native, ABI, engine or dependency change followed
those accepted runs. Documentation/status updates do not invalidate them.

Production and fixture native artifact hashes match the accepted Feature 007
source `3067fc690f1b08e28bf7a296468c8105c4122ea1` in the same environment.
The reviewed foundation delta contains only blank-line removal in unchanged
native execution workers; exact intent and MCP behavior were separately
exercised above. `schema2-t001-_pgpqenm/native-reuse.json` records that comparison.
Unchanged confinement/authentication, native/export and local whole-source
foundations retain their historical meaning; no historical Schema 1 client
result is promoted to Schema 2 acceptance.

The shape review found no material accidental complexity: checked text/intent
remain in the existing request module, capture/revision/lifecycle dispatch and
terminal match reduction stay in the existing read runner, and MCP
decoding/schema/projection remain at the adapter. New exact-intent declarations
are crate-private with current callers; there is no speculative public mode,
state store, hook or generic framework. The nonempty-string `expect` is dominated
by an explicit empty branch, not used for unavailable safety evidence. Independent
adapter review found no material correctness, disclosure or shape issue.

This records compliance with Principles I–IV, VII, IX, X and XIII for T001's
implemented boundary and current both-client acceptance. T002's broader
qualification, T003's composed completion and a release remain outside this
task. Approved requirements and remaining task obligations are unchanged:
the initial exact analysis fingerprints established implementation currentness;
these later status, checkbox and evidence updates change no remaining obligation.
No extension configuration was present at pre/post-implementation inspection.

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
