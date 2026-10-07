# Quickstart: Validate Schema 2 Exact Editing

**Status:** Feature 008 and T001–T003 are **Complete**. The Schema 2 cutover, both-client interoperability, adversarial qualification and [composed/cumulative acceptance](#t003-composed-and-cumulative-acceptance-2026-10-07) have valid evidence. Codex uses the accepted private official npm distribution, not the known-failing Homebrew companion route. PR delivery is separate from completion; no next feature, Phase 4 work or release is selected.

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

Use these existing selectors for affected verification. T001 owns the migrated exact-edit baseline, T002 adversarial qualification and T003 composed/cumulative acceptance; their executed and reused evidence is recorded below:

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

Use a fresh prepared run for each client/profile. The current profiles `workflow`, `sources`, `bound`, `failures`, `reconnect` and `composed` remain the existing ownership locations; `known`/`observations` are available when directly affected. Their prompts, result consumers and independent witnesses implement the exact-edit positive, recovery and composed scenarios. Do not treat an old prompt or a model's claimed use of new fields as proof.

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

**T001 completion: complete.** Initial production and acceptance inputs are at
`6b41fcfd446b14461aafe94303aa2a8eafa0fc98` on
`task/T001-schema2-exact-edit`. Initial delivery then recorded documentation and
completion state; the post-review adapter correction and evidence-currentness
decision are recorded below. Both selected clients satisfy the atomic cutover
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
corrections. Source, bound, privacy and both-client results ran at the initial
code head. No production change preceded the documentation-only initial delivery.
The later adapter validation correction is assessed separately below.

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
by an explicit empty branch, not used for unavailable safety evidence. Initial
adapter review reported no material issue; the later finding is corrected below.

This records compliance with Principles I–IV, VII, IX, X and XIII for T001's
implemented boundary and current both-client acceptance. T002's broader
qualification, T003's composed completion and a release remain outside this
task. Approved requirements and remaining task obligations are unchanged:
the initial exact analysis fingerprints established implementation currentness;
these later status, checkbox and evidence updates change no remaining obligation.
No extension configuration was present at pre/post-implementation inspection.

### Post-review correction: open matching output validation

PR #70 review identified that a schema-valid open result labeled
`matching` / `refused` / `not_applied` could pass adapter validation despite
history, execution-step or receipt evidence contradicting zero effects.
T001's matching-refusal projection acceptance and the public matching contract
require independent adapter rejection, not reliance on the private producer.

`mcp::output::edit::validate_typed()` now requires `not_participated` history,
`not_started` buffer application, Resource synchronization, persistence and
finalization, and absent persistence/finalization receipts for open matching
results. Existing checks continue to require `refused`, `not_applied`, a matching
reason and `fresh_read`. The producer's `reduce_open_match()`, all closed-path
validation, mutation execution and valid-output serialization are unchanged.

Three deterministic regressions exercise 29 schema-valid contradictions across
terminal facts/guidance, history, all four effect steps (entered, completed,
failed and unknown), and both receipts even without claimed effects. The
positive serialization case retains all four matching reasons, nonparticipating
history, zero-effect fields, `isError: true` and `fresh_read`; it permits completed
validation/verification and does not pin sentence wording. All 26 focused MCP
output tests passed before the broader Rust checks.

An offline smoke replayed the six recorded open edit results from
`schema2-codex-workflow-2/20261007T050939Z-130a596811e7/artifacts/mcp-calls.jsonl`
through the current Rust validator. All original results remained accepted.
Relabeling only outcome/application/stage/reason/action as matching refusals
accepted the two zero-effect unchanged results and rejected all four effectful
results. This is adapter fault-injection evidence from retained records, not
a new Codex/OMP or VM run. The temporary smoke driver was removed.

Post-review Rust completion checks passed from `mcp-server/`:
`cargo fmt --all -- --check`,
`cargo clippy --all-targets --locked -- -D warnings`,
`cargo test --locked -- --test-threads=1` (495 tests; doctest target also passed),
`cargo doc --no-deps --locked`, and `cargo build --locked --bins --examples`.
Serial test execution avoids the previously observed parallel worker-startup
contention without changing any deadline or assertion. The earlier aggregate
failure remains historical; this completion run passed without exclusions.

**Currentness decision:** Reuse the accepted Codex 0.153.4, OMP 18.5.1 and VM
evidence above under `TEST_POLICY.md`. The changed boundary rejects inconsistent
private output only; the unchanged producer already enforces every added
predicate before emitting an open matching refusal. Relevant behavior is
equivalent for accepted outputs, supported by producer review, positive
serialization regressions and the recorded-output smoke—not a claim of
byte-identical rebuilt Rust binaries. Native/addon/ABI, schemas/protocol,
authentication/routing, client/VM fixtures/witnesses/runners, environment and
acceptance requirements are unchanged. No accepted client/VM evidence was
invalidated and no campaign was rerun.

**Shape and constitutional review:** The existing DTO validator still owns this
check. One allocation-free fixed-size predicate is the smallest correction;
producer-only validation cannot catch contradictory producer output, while a new
validation layer would add unnecessary maintenance cost. No new declaration,
public API, dependency, protocol or operational gate is introduced in production.
This preserves Principles III, IV, VII, IX, X and XIII. T001 remains complete.
At that delivery, T002/T003 had not started.

## T002 implementation evidence (2026-10-07)

**T002 completion: complete.** This task qualifies US2.1–US2.9, the intentional
no-match/ambiguity recovery in US4.3, recovery guidance in US4.4 and the affected
US3.4 uncertainty path. T003's composed/cumulative acceptance remains pending.
Only T002 is delivered on `task/T002-exact-edit-qualification`.

### Changed boundary and checks

The existing preservation, interruption, validation, privacy and client-failure
fixtures now exercise literal/overlapping matching, same-text and empty-source
staleness, saved-version/Resource/context guards, complete-source validation,
UTF-8 byte limits, interrupted effects and sticky disclosure. Independent
witnesses retain full source, dirty/saved state, identities, native history,
selection and lifecycle facts rather than accepting reason strings alone.
No Rust, production addon/native, schema, dependency or transaction change was
needed. Public guidance was sufficient in both actual clients and is unchanged.

The final affected consumer command passed **65 tests** from
`godot-addon/tests/`:

```sh
python3 -m unittest test_mcp_failure_acceptance test_mcp_privacy_acceptance \
  test_mcp_preservation_acceptance test_mcp_lifecycle_acceptance \
  test_mcp_composed_acceptance
```

The displayed `--command sh` parser errors are expected negative CLI cases; the
unittest result is `OK`. Earlier overlapping consumer runs are not added to this
count. Changed Python modules parsed. No fresh Rust baseline or historical
native/export campaign was required for fixture-only changes.
One redundant copy/unchanged-input assertion was removed during final shape review;
the real disclosure run supplies the positive evidence and behavioral rejection
regressions remain. The owned client checkout and diagnostic scratch log were
removed, and the dedicated VM was stopped after acceptance.

### Direct real-editor qualification

All runs used the owned VM, official Godot `4.7.2.stable.official.ed1daf0bf`,
macOS 26.6.2 arm64 and the unchanged four-vCPU / 6-GiB profile.
Environment identity:
`5be872888a3eacb7798de40e70d66e1f438d908e00cf714575d65b2632596ced`.
These fifteen groups contain **448 qualified records**, including setup,
reads, refusals and interrupted outcomes—not 448 successful edits or unique
requirements. Artifact roots are relative to
`~/.local/state/godot-agent-kit-vm/artifacts/`. Source abbreviations identify task
commits; each private `provenance.json` retains its full source and build identity.

| Selector | Tested source | Qualified records | Maximum consumed edit response | Artifact root |
| --- | --- | ---: | ---: | --- |
| `preservation-exact-literals` | `bc8477f` | 34 | 4.478628 s | `exact-t002-literals-1/20261007T071135Z-2fc18370c3a1` |
| `preservation-exact-stale` | `bc8477f` | 37 | 0.633285 s | `exact-t002-stale-1/20261007T071441Z-8af357650749` |
| `preservation-exact-empty-stale` | `bc8477f` | 24 | 0.629598 s | `exact-t002-empty-stale-1/20261007T071721Z-f4206a69cac6` |
| `preservation-exact-safety` | `bc8477f` | 31 | 0.656990 s | `exact-t002-safety-1/20261007T072103Z-be420941db21` |
| `interruption-exact-diagnostic` | `bc8477f` | 37 | 9.515915 s | `exact-t002-diagnostic-interrupt-1/20261007T073406Z-ef06d529a9b1` |
| `interruption-exact-effects` | `bc8477f` | 73 | 9.514215 s | `exact-t002-effect-interrupt-1/20261007T074529Z-aff3f7f64130` |
| `privacy-exact` | `bc8477f` | 12 | 6.827380 s | `exact-t002-privacy-1/20261007T074619Z-3ef4ccae12ee` |
| `preservation-exact-diagnostic-guards` | `6637c14` | 17 | 5.270149 s | `exact-t002-guards-2/20261007T081704Z-b2567e9464b1` |
| `preservation-source-context` | `6637c14` | 33 | 6.492210 s | `exact-t002-context-1/20261007T082006Z-922579ce37c1` |
| `preservation-selection-privacy` | `6637c14` | 18 | 0.648626 s | `exact-t002-selection-1/20261007T082104Z-6a7869150fe3` |
| `preservation-session-replacement` | `6637c14` | 8 | 0.693760 s | `exact-t002-session-1/20261007T082151Z-c630e633b0e5` |
| `privacy-interrupted` | `e0826ba` | 19 | 4.805147 s | `exact-t002-late-denial-3/20261007T094750Z-0e2087dcfb86` |
| `preservation-revisions` | `e0826ba` | 19 | 5.868130 s | `exact-t002-revisions-2/20261007T094901Z-98984baa7389` |
| `validation-exact-max-near-match` | `1044b6f` | 13 | 9.243937 s | `exact-t002-max-match-4/20261007T100435Z-3799d0b6b6cf` |
| `validation-exact-source-boundaries` | `50f66ab` | 73 | 6.971910 s | `exact-t002-validation-7/20261007T103843Z-1f65e8f5d1d7` |

Literal and stale groups cover US2.1–US2.4; safety/context/selection/session groups
cover US2.5–US2.6. Source-boundary qualification covers US2.7–US2.8: all twelve
positive fragment/deletion/whitespace/empty-source edits were `verified_changed`,
and malformed complete source, oversized fields and oversized derivations refused
before effects. An exactly 524,288-byte UTF-8 fragment is admitted as an argument
but rejected as `invalid_source` when its complete derivation exceeds that cap;
the one-byte-over-limit UTF-8 field is rejected at input. No truncation is accepted.
Interruption and late-denial groups cover US2.9 and the affected US3.4 behavior.

**Maximum-bound limitation:** The nine repetitive maximum-source diagnostic
requests all refused without effects under the original clock. Open cases
returned `validation_unavailable`; cached-closed cases returned `timeout`;
absent-R cases completed validation and returned the expected two `no_match`
and one `ambiguous_match`. A matching diagnosis must not override unavailable
validation or a deadline. Ordinary literal cases still require their exact
matching reasons. Unknown terminal lifecycle/history is not rewritten as
preserved-state proof; separate editor witnesses establish the surviving state.

### Actual selected-client recovery

Both retained installations used `gpt-6-astra` and normal approval policies.
Codex CLI **0.153.4** used the accepted private official npm route,
`--ask-for-approval on-request --sandbox read-only`; every edit received one
ordinary `Allow` decision, not session-wide or bypass approval.
OMP **18.5.1** used `--approval-mode write`; its 15-second timeout applies to the
owned relay, not the product's original five/ten-second operation bounds.

Client-owned evidence is under `exact-t002-u0a4oqud/`, with launch commands,
prompts, complete actual client transcripts, normalized model-facing records
and correlation results. Codex records come from the identified real TUI threads;
OMP records come from completed model-facing tool-result messages, not merely
raw server envelopes. Server calls and independent finalizers remain separate.

| Client/profile | Tested source | Qualified records | Model-visible results | Withheld results | VM artifact root |
| --- | --- | ---: | ---: | ---: | --- |
| OMP failures | `fe7635e` | 23 | 18 | 0 | `exact-t002-omp-failures-2/20261007T085023Z-9b6d70bbfda4` |
| OMP reconnect | `e0826ba` | 6 | 3 | 1 | `exact-t002-omp-reconnect-2/20261007T090802Z-a42e22284647` |
| Codex failures | `e0826ba` | 23 | 18 | 0 | `exact-t002-codex-failures-1/20261007T092023Z-5a5f5f9f9117` |
| Codex reconnect | `e0826ba` | 5 | 2 | 1 | `exact-t002-codex-reconnect-1/20261007T093535Z-474cf025b415` |

Both clients freshly read before each intentionally missing, corrected,
ambiguous and larger-unique-span request. Both corrections were independently
verified changes, not retries. The clients also recognized unavailable revision,
stale refusal, known partial application and unknown application, then read the
same explicit target without repairing or replaying uncertain edits.

OMP automatically retransmitted its one model-authored lost-response edit once.
The original response was withheld; the retransmission was refused by the old
revision. The model received that refusal, read the surviving intended source
and did not infer that the refusal proved nothing had happened. The automatic
resend is neither a fresh read nor another intentional edit.

Codex reported `Transport closed` for the edit and its attempted recovery read.
That failed read was client-local, not a fabricated server call. Resuming the
same conversation restored the connection; its only new operation was a read of
the original target. The model distinguished observed surviving source from the
still-missing original receipt and continuous-lifecycle evidence. No edit was
resent. All four independent finalizers passed.

### Corrections, excluded attempts and timing investigation

Review corrected preparation cases that had stopped at revision comparison,
parser cases that could pass on an unrelated refusal, recovery checks that
rejected valid additional fresh reads, and missing stderr sentinels for the two
late-denial fragments. A fragment's extra terminal newline initially triggered
`save_would_reformat`; removing that fixture newline made the required
complete-source parser rejection reachable.

The first actual OMP failure run kept four independent editors alive and failed
during a correction. The fixture now reuses the existing serial-ownership
pattern: independently finalize/close one editor before preparing the next.
The accepted client records include singleton live-editor PID witnesses.
The initial OMP reconnect run then exposed a fixture `TypeError`: the composed
observer did not accept the unavailable-delivery keyword. Withheld receipts now
invoke their owning failure observer directly. Both actual reconnect runs
exercise the correction; production behavior did not change.

The maintainer-requested Phase 3 investigation checked the
[serial fixture correction](../007-mcp-script-workflow/plan.md#t004-serial-fixture-correction)
and [prepared native-documentation evidence](../007-mcp-script-workflow/quickstart.md#t004-timing-safe-validation-preparation-2026-10-05).
Source-free phase probes confirmed that native documentation was prepared before
protocol input and seeded into fresh preflight, post-change and unchanged
validators. Preparation was not missing. Probe patches and measurements are
retained privately; the disposable checkouts and instrumentation were removed,
and no probe is promoted to task acceptance.

Failed source-boundary attempts `exact-t002-validation-1` through `-6`,
maximum-match attempts `exact-t002-max-match-1` through `-3`, the first
late-denial/revision attempts, and the first OMP failure/reconnect attempts remain
excluded. Some simple positive attempts exhausted validation; their assertions
and deadlines were not relaxed. The final complete source-boundary run passed.
The initial direct failure smoke is historical and superseded by the accepted
serial actual-client runs, not added to the qualified-group total.

An additional whole-buffer maximum-size rewrite experiment reached post-effect
validation limits, and one attempt reported `protocol_failure` near the deadline.
These are unverified outcomes, not successful edits. Its proposed permissive
consumer and tests were removed rather than broadening success acceptance.
The selected byte-boundary refusal and maximum-source matching tests remain;
T001's accepted positive-bound evidence is reused. No general successful
maximum-size Unicode open-buffer rewrite guarantee is newly claimed.

### Evidence currentness and completion review

The delivered production source is unchanged from merged T001 (`468a0fe`).
Native production/fixture build keys, outputs and generated ABI/header hashes
match the accepted T001 provenance. The final boundary run's complete Rust and
native build provenance also matches the accepted T002 Codex run in the same
environment. T001's documented post-review adapter correction remains the
justification for reusing its earlier client positives; no historical Schema 1
result substitutes for Schema 2 acceptance.

Later fixture changes are confined to their affected groups: corrected
diagnostic-guard setup, boundary oracles, failure-profile ownership and
lost-delivery observation. Their affected groups were rerun as listed above.
The nonempty same-text file perturbation is unchanged; the new empty/saved-version
actions are reached only by the new cases. Existing workflow/source/bound client
fixtures and actual consumer behavior remain unchanged. Shared harness summary
labels and an interruption docstring now use durable responsibilities instead
of historical task numbers; this metadata-only cleanup changes no assertion or
execution. Unchanged T001 client positives, native history/durability,
confinement/authentication, local whole-source operations and export foundations
therefore remain reusable under `TEST_POLICY.md`.

The [implementation-shape and constitutional review](plan.md#t002-implementation-shape-and-constitutional-review)
records the ownership and complexity decisions. All new declarations have current
fixture consumers; no production public API, dependency, alternate writer,
deadline extension or approval gate was added. Principles I–V, VII–X and XIII
are preserved. Approved remaining obligations and the existing analysis
attestation are unchanged by these delivery/evidence updates.

No extension configuration was present at pre/post-implementation inspection.
Only T002 is marked complete; Feature 008 remains in implementation and T003
remains unstarted. Delivery does not authorize merge, another task, Phase 4 or
a release.

## T003 composed and cumulative acceptance (2026-10-07)

**T003 and Feature 008: complete.** The accepted source is
`a3d581b0f477071112320781f842ca62a374da25` on
`task/T003-exact-edit-composed`. Subsequent delivery changes only documentation,
evidence and completion state. T001 and T002 were already complete and delivered
in merged PRs #70 and #71; only T003 is completed by this task PR.

### Executed composed acceptance

Actual **OMP 18.5.1**, `openai-codex/gpt-6-astra`, normal `write` approval and the
fixed VM relay drove `prepare-mcp --profile composed`. The retained CLI SHA-256
is `fc62b280c50923f779e9af14e0ba12d24cbf3db6b2757106a8ad127b3a3274a9`.
The private invocation used an owned detached checkout and temporary project
MCP configuration; credentials remained on the host. No SDK/direct-driver call
or model assertion substitutes for actual model-visible results.

| Profile | Verified fresh-read edits | Dirty refusals | Stale refusals | No-match refusals | Ambiguous refusals |
| --- | ---: | ---: | ---: | ---: | ---: |
| Open | 8 | 3 | 1 | 1 | 1 |
| Cached-R closed | 6 | 0 | 1 | 1 | 1 |
| Absent-R closed | 6 | 0 | 1 | 1 | 1 |

All **80 model-visible calls/results** exactly correlate with actual server
arguments and authoritative returned envelopes; no result was withheld. The
independent guest finalizer passed **87 records**, including setup, operations
and runtime witnesses—not 87 edits. Its `real_client_acceptance: false` field
deliberately does not certify the external model; the separately inspected OMP
transcript, normalized model-facing content and correlation establish that claim.
Maximum consumed responses were **7.101979 s edit**, **0.717909 s read** and
**0.177831 s discover**, within unchanged ten/five-second bounds.

The sequence preserved complete intended source and admitted lifecycle, not
merely the requested return value. Open edits retained the same document,
independently clean/saved D/R/B and unrelated human work. The equal-text dirty
case still refused. Genuine native Undo → Save → Redo → Save exposed the exact
unsaved/persisted source transitions to the model; prior history remained
reachable and each later open edit reversed to its actual previous source.
After each matching refusal the client freshly read and deliberately selected
the corrected unique return line, rather than replaying the failed intent.

Cached closed edits independently retained coherent D/R without a buffer.
Absent-R edits retained positive R/B absence. Closed success was established
before later opening. Final ordinary Save, close/reopen or first opening,
reparse, rescan and fresh runtime preserved the complete intended sources;
owned editor shutdown retained final disk source. The final open source retained
all three human comment lines. Reviewed guest captures include unsaved native
Undo, closed absence and the clean later-opened final script.

Private evidence, relative to `~/.local/state/godot-agent-kit-vm/artifacts/`:

- Guest: `exact-t003-omp-composed-3/20261007T142339Z-87c0268f4087/`,
  with `provenance.json`, finalizer, all calls/delivery timings and independent
  native/lifecycle/source witnesses and window captures.
- Client: `exact-t003-35dy8raz/omp-composed-3/`, with launch/prompt/config receipt,
  actual `omp.jsonl`, normalized `model-visible.jsonl` and `correlation.json`.
- Review index: `exact-t003-35dy8raz/accepted-composition.json` and
  `excluded-runs.json`. Source-bearing artifacts are not committed.

The affected composed/lifecycle/failure consumer command passed **61 tests**:
`python3 -m unittest test_mcp_composed_acceptance test_mcp_lifecycle_acceptance test_mcp_failure_acceptance`
from `godot-addon/tests/`. Expected negative CLI parser errors are not test
failures. After the composed-only schedule correction, its **26 tests** passed
again; the other 35 tests' inputs were unchanged. These overlapping runs are
not added into another distinct-test count. Actual-agent acceptance exercised
the changed fixture/relay path. No new Rust/native baseline or historical GUI
campaign was needed for these Python-only changes.

### Excluded attempts and retained limitations

The first run, `exact-t003-omp-composed-1`, failed before its first product call:
shared positive setup overwrote the composed schedule. The corrected regression
reproduced `ValueError: too many values to unpack`; the composed owner now
retains its own schedule. Open/closed next-action decoding also received a
failing-before/passing-after consumer correction before runtime acceptance.

The second run, `exact-t003-omp-composed-2`, stopped at open edit 5 with
`applied_unverified` / `validation_unavailable`: preflight passed, effects and
finalization completed, but post-change validation did not complete by the
9.5-second work cutoff (reported interval **9.505721 s**). The strict fixture
rejected the non-success and closed its relay. OMP recognized the uncertainty,
attempted reads and did not replay the edit. This is a failed composed attempt,
not verified success or rollback. No unique resource/timing cause was established.
Its 27 delivered results and two client-local failed reads are excluded from
the accepted run's counts.

After finalizing that failure and restarting the owned desktop through the
existing readiness check, the fresh third run passed at the same source,
resources, deadlines and assertions. It used new fixtures and intentional agent
requests, not product retries or continuation of uncertain edits. Earlier
T001/T002 failures remain excluded as recorded in their sections.

Support remains official Godot `4.7.2.stable.official.ed1daf0bf`,
macOS **26.6.2 (25G83) arm64**, four vCPUs / 6 GiB, the existing source/context
bounds and exact supported clients. No arbitrary-load timing, universally
successful maximum-size Unicode open rewrite, active-game/cached-class hot
reload, persistent history after disposal or broader platform support is claimed.
Closed B/history is inapplicable only with positive absence, never missing evidence.
T002's lost receipts remain undelivered and its automatic OMP retransmission is
not new intent or proof of non-application.

### Relevant-input currentness

T003 changes only `mcp_composed_acceptance.py` and its consumer tests; shared
positive/failure fixtures, production paths, schemas, authentication/routing,
native/addon/ABI and VM runner inputs are unchanged. The schedule correction is
confined to composed ownership. It invalidated prior composed proof, which the
new accepted run replaces, not unrelated positive or interruption evidence.

The accepted T003 **complete Rust, native production and native fixture build
receipts**, including output hashes, exactly match accepted T002 source-boundary
run `exact-t002-validation-7/20261007T103843Z-1f65e8f5d1d7`.
Environment and identity also match:
`5be872888a3eacb7798de40e70d66e1f438d908e00cf714575d65b2632596ced`.
MCP executable SHA-256 is
`df280674c3f3683998ea8d6bc97c766f035cfa26ab3389d2243a6b0f9bd9723f`;
native production/fixture keys remain `e456ef92…` / `2df62748…`.
The full receipt retains library, manifest, generated API/ABI/header,
compiler/SDK and exact engine/template hashes—not just version labels.

Independent cumulative review inspected all six T001 client finalizers and
correlations, their Schema 2 catalogs and all-profile exact/deletion/empty-source
server sequences; all fifteen accepted T002 direct summaries; all four actual
failure/reconnect finalizers and representative source/history/lifecycle
witnesses. Both Codex 0.153.4 and OMP 18.5.1 positives remain current. The only
production delta after T001's original client runs is its documented open
matching-output validator correction; its existing producer-equivalence,
regressions and recorded-output replay rationale remains applicable.

Native source/ABI/binary receipts match the accepted Feature 007 foundation.
Reviewed native-history, matched-v6/native4 local-operation and enabled/disabled/
hook-only export records remain valid: no changed execution path invalidates
their guarantees. The older export environment identity differs by provisioning
generation, but its inspected engine/platform/tool/worker/template/native inputs
match; this is evidence reuse, not checkpoint portability. Local whole-source
constructors/dispatch remain genuine unchanged consumers. T001's documented
495-test serial Rust baseline and T002's prior tests are reused, not newly run.

### Cumulative requirement disposition

**New** means the accepted T003 actual-agent composition. **Reused** means accepted
T001/T002/foundation evidence after the relevant-input review above; no historical
Schema 1 client pass establishes Schema 2 interoperability.

| Scenarios | Final evidence disposition |
| --- | --- |
| US1.1–US1.3 | Reused both-client/all-profile exact workflows; new complete-source/lifecycle composition. |
| US1.4 | Reused both clients' multiline/tab/Unicode source profiles. |
| US1.5–US1.6 | Reused localized/all-source deletion, invalid-complete-source refusal, anchored insertion and deterministic boundary matching. |
| US1.7–US1.8 | Reused checked unchanged and all-profile complete-empty-source replacement; new repeated preservation. |
| US2.1–US2.3 | Reused exact literal/overlap/Unicode refusals and both-client correction; new no-match/ambiguity recovery on each profile. |
| US2.4–US2.6 | Reused same-text/empty/session/identity, safety, late-guard, context and confined-selection evidence; new stale/dirty/human-history interleaving. |
| US2.7–US2.8 | Reused strict malformed/legacy/mode checks, source/UTF-8 boundaries, full-source validation, maximum diagnostic refusals and scoped positive-bound evidence. |
| US2.9, US3.4 | Reused exact diagnostic/effect interruption, late denied disclosure and both-client failure/reconnect interpretation. Composition adds no new interruption mechanism or interaction. |
| US3.1–US3.3 | New actual native history/prior-history, durability and twenty-edit composed A–E interaction; reused unchanged native foundations. |
| US4.1–US4.2 | Reused actual Schema 2 catalogs and strict legacy/mixed/mode refusal; current catalog also observed in the new conversation. |
| US4.3–US4.4 | Reused both-client positives/corrections/uncertainty; new correlated composed choices, fresh-read corrections and truthful state interpretation. |

| Requirements / criteria | Final evidence disposition |
| --- | --- |
| FR-001–FR-002, FR-014–FR-015; SC-001, SC-006–SC-007 | Reused single-shape Schema 2 process/catalog/migration and both-client use; new actual composed interface use and qualitative review. |
| FR-003–FR-007, FR-009–FR-010; SC-002–SC-003 | Reused bounded exact derivation, validation, freshness/safety and diagnostic precedence; new repeated full-source and human-work preservation. |
| FR-008; SC-004 | New applicable real-agent A–E, native history and lifecycle/durability, with reused native foundations. |
| FR-011–FR-013; SC-005 | Reused effect/disclosure/refusal and recovery evidence; new source-free matching results and complete call/result correlation. |
| FR-016–FR-018; SC-008 | New supported-condition consumed timing, composition and cumulative provenance review; reused unchanged native/export/local boundaries and both-client compatibility. |

This covers all **25 scenarios, 18 FRs and eight SCs**. No required scenario is
waived; only positively absent closed B/history and explicitly excluded support
claims are inapplicable. The [shape/constitutional review](plan.md#t003-implementation-shape-and-constitutional-review)
passes. The original analysis remains current because completion/evidence/status
updates change no approved obligation. The owned run finalized, its temporary
client configuration was removed, and the dedicated VM was stopped.

Completion documentation validation rendered eight changed surfaces (104 headings
and 23 tables), resolved all 82 local links/anchors, parsed fifteen shell fences
with `bash -n` and one JSON fence, and passed whitespace checks. No extension
configuration existed at pre/post-implementation checks. The owned client
checkout and temporary launcher were removed; private evidence remains retained.

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
