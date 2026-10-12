# Quickstart: Validate Trusted Script Creation and Deletion

**Status:** Planning guide, not executed create/delete acceptance. The [mechanics study](research.md#owned-stock-editor-mechanics-experiment) is bounded research evidence. New selectors/profiles below are the selected extensions to the existing harness and must be implemented before those commands can run; they are not available product capabilities at this design commit.

## Prerequisites

- Complete separately requested task derivation, granularity review and current consistency analysis before implementation. Select one named task and deliver one PR; this guide does not authorize implementing all groups at once.
- Use committed implementation source in an owned checkout and the existing [macOS VM workflow](../../.github/LOCAL_VM.md). Godot Editors, runtimes, window/focus actions and fixtures stay in the dedicated graphical guest. No host-editor/headless substitute or mutable human project.
- Keep official Godot `4.7.2.stable.official.ed1daf0bf`, macOS 26.6.2 / 25G83 arm64, four vCPUs/6 GiB and exact engine/export/native provenance. Different environments are development evidence until equivalence is deliberately approved, not expanded support.
- Use Codex CLI **0.153.4** through the already accepted private official npm route and OMP **18.5.1**, with normal host-owned model authentication/approval. Do not reopen the known Homebrew companion stall, copy credentials into the guest, or change global trust/quarantine settings.
- Build the matched bridge-v7/native-revision-5 product and separate fixture-fault artifacts through existing tooling when the selected implementation task changes native inputs. Record source/generated ABI/API/build/library provenance; matching version strings alone do not establish equivalence.

From repository root, inspect existing interfaces and client versions without launching a host editor:

```sh
"$CODEX" --version
"$OMP" --version
python3 godot-addon/tests/run_in_vm.py --help
```

`CODEX` and `OMP` identify the exact accepted executables, not arbitrary global installations. Validator startup preparation remains source-free; per-request validation remains fresh and inside the original mutation deadline.

## Focused Rust and process checks

Start with selected-task behavior: absent-root admission, exact source/byte boundaries, unchanged revision eligibility/comparison, deterministic effect reduction and consumer-visible input/output errors. Tests must catch plausible safety or API failures, not pin description wording, copy schema source, count fields without behavior, or echo mocks. Do not add a generic test framework.

Include the occupied-at-entry race: an attributable exclusive `openat` `EEXIST` result must retain `failed_no_effect` and yield `refused/not_applied` when no other effect occurred. Losing that return after possible entry must instead preserve uncertainty. A failed unlink after known document/Resource removal must retain partial application.

Existing process boundaries, from `mcp-server/`:

```sh
cargo test --locked --test mcp_tools
cargo test --locked --test mcp_transport
```

Extend them to exercise real five-tool discovery/output schemas, valid new decoding, rejection of missing/null/wrong-type/duplicate/unknown/legacy/mixed fields before dispatch, and effect-sensitive failure after dispatch for each new operation. Prove that an output/worker error cannot relabel possible create/delete as not-applied. These process tests are not live-editor or real-agent acceptance.

At Rust-affecting task completion, retain the normal applicable baseline:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo doc --no-deps --locked
cargo build --locked --bins --examples
```

Apply [TEST_POLICY.md](../../TEST_POLICY.md) for narrower development checks, doctests, actual consumers, native provenance, and affected harness selection tests. Markdown-only publication does not execute these commands.

## New owned VM groups

Register these independently runnable selectors in the existing MCP runner and expose them through `run_in_vm.py run mcp`; do not add a second runner or make an aggregate `all` mode mandatory:

| Selector | Required behavior and witnesses |
| --- | --- |
| `existence-create` | Exact Unicode/whitespace/empty and supported source-boundary creation; no live placeholder on failed validation; positive absent/newly coherent R branches with B absent; independently observed actual-source post-validation; occupied empty/equal/regular/directory/link controls, missing parents and unsafe UID companions. |
| `existence-delete` | Fresh-revision deletion of clean open, closed cached-R and closed absent-R targets; empty existing file; retained original R alive but detached; exact file/document/cache disappearance; no source/reference repair or forced lifecycle workaround. |
| `existence-preservation` | Same-text revisions, file/parent/session/Resource/document replacement, close/open ABA, cache-branch change, dirty/equal-text dirty/divergent/unknown state, read-revision unavailability, target orphan B/R, unsupported contexts and denied/confined routing. Observe original/newer targets and unrelated history, not just the error code. |
| `existence-interruption` | Before/after every effect boundary, lost authorization/receipt, partial write/flush, document removed before failed detach/unlink, late cancellation, helper/context failure, recreated paths and output/backpressure/EOF loss. Known effects stay known; unknown stays unknown; no cleanup/replay/late entry after proved discard. |
| `existence-composed` | Stateful create → read → exact edit → fresh read → delete → independent absence, then same-path recreation and old-basis rejection. Repeat across admitted lifetimes/profiles; old text, UID reuse or coincidental absence cannot certify/replay an old request. |

For an implementation revision that registers the selected group, run only the owned/affected selector, for example:

```sh
python3 godot-addon/tests/run_in_vm.py start
REVISION="$(git rev-parse HEAD)"
python3 godot-addon/tests/run_in_vm.py run mcp \
  --revision "$REVISION" --scenario existence-create --captures
```

Expected evidence includes complete actual D bytes/metadata, independently read cached Script source/path/edited state or positive absence, complete ScriptEditor/CodeEdit association and dirty/version/history observations or positive B absence, correlated before/after intervals and actual effect facts. A native receipt, process exit, inventory omission or agreement with runtime alone is insufficient.

Use deterministic fixture barriers at preparation/entry/post-effect boundaries, never timing sleeps as race proof. Absence-R setup must be positively observed after startup; ordinary startup scanning can load a disk fixture. No fixture may make the product pass by closing, unloading, saving, discarding or repairing its target during the operation. Fault entrypoints stay in the separate fixture artifact and never the public catalog/export.

## Actual coding-agent clients

Add `existence` and `existence-composed` profiles to the existing `prepare-mcp` prompt/receipt/witness ownership, not another client launcher. For **each** client, use fresh owned runs:

- `existence`: real complete-source creation and fresh-read-based deletion on all three clean profiles, plus model-visible refusal/recovery. Keep a separately created survivor for later ordinary durability checks. No client raw file write/delete or private native call counts as tool use.
- `existence-composed`: the full stateful five-tool workflow and same-path new lifetime. Known targets need not discover first. The normal composed workflow works closed without hidden open/close steps; separate prearranged open-target fixtures prove open deletion.

Example preparation, from an owned clean checkout of the implemented profile:

```sh
REPO="$PWD"
REVISION="$(git rev-parse HEAD)"
RUN_ID="mcp-existence-$(date -u +%Y%m%dT%H%M%SZ)"
ARTIFACTS="$(mktemp -d "$HOME/mcp-existence.XXXXXX")"
python3 godot-addon/tests/run_in_vm.py prepare-mcp \
  --revision "$REVISION" --run-id "$RUN_ID" --artifacts "$ARTIFACTS" \
  --profile existence
```

The fixed relay keeps server/registry/project/native witnesses in the guest. Clients and personal authentication remain on the host. Preserve normal client authorization and actual request/result consumption; a printed success summary or raw SDK event is not proof the model used the new tool/result.

### OMP

In an owned checkout with no pre-existing project MCP file, install only this run's generated entry. Otherwise use another clean owned checkout or an additive temporary entry with exact restoration; never overwrite a user's configuration.

```sh
test ! -e "$REPO/.omp/mcp.json" && \
  cp "$ARTIFACTS/mcp.json" "$REPO/.omp/mcp.json"
OMP_MCP_TIMEOUT_MS=15000 OMP_MCP_REQUIRE_READY=1 \
  "$OMP" --cwd "$REPO" --mode json --no-session --approval-mode write \
  < "$ARTIFACTS/prompt.txt"
```

Capture this run's JSONL/stdout and stderr privately. Use ordinary interactive approval when required, not bypass flags. OMP discovery is additive; verify actual tool origin is the prepared relay, not another configured server. Preserve `.omp/lsp.yaml` and unrelated configuration.

### Codex

Use a separately prepared run and the accepted interactive TUI route; the known `exec` edit-policy refusal is not a positive mutation recipe:

```sh
"$CODEX" --no-alt-screen --ask-for-approval on-request \
  --sandbox read-only --model gpt-6-astra \
  -c "projects={\"$REPO\"={trust_level=\"untrusted\"}}" \
  -c 'mcp_servers.godot_agent_kit.command="python3"' \
  -c "mcp_servers.godot_agent_kit.args=[\"$REPO/godot-addon/tests/run_in_vm.py\",\"mcp-stdio\",\"--run-id\",\"$RUN_ID\"]" \
  -c 'mcp_servers.godot_agent_kit.startup_timeout_sec=10' \
  -c 'mcp_servers.godot_agent_kit.tool_timeout_sec=15'
```

Supply the prepared prompt. Use normal **Allow** for the intended owned-fixture calls, not persistent/global bypass. Include the displayed primary repository in the invocation-scoped untrusted project table when using a worktree. Capture only this owned thread's tool calls/results and approval/consumption evidence, not unrelated histories or authentication files.

### Durability and finalization

Extend the existing durability continuation for the `existence` profile's created survivor and deleted targets. First establish create/delete success **without** opening, scanning or saving to manufacture it. Only afterward exercise ordinary opening/Save, reparse/rescan and a fresh owned runtime. Confirm exact created source and absent deleted paths after ordinary remaining-document Save; old detached R or history must not silently recreate the deleted file. A deliberately separate later create is a new lifetime, not rollback.

```sh
python3 godot-addon/tests/run_in_vm.py prepare-durability \
  --run-id "$RUN_ID" > "$ARTIFACTS/durability-prompt.txt"
```

Drive that continuation with the same actual client/relay, then finalize:

```sh
python3 godot-addon/tests/run_in_vm.py finalize-mcp --run-id "$RUN_ID"
```

For the composed profile, use its registered finalizer after its complete stateful sequence. Finalization must verify required evidence, clean owned guest fixtures and retrieve both success/failure records. Preserve human changes while removing only temporary config this run owns; stop the VM at session end. Artifact fetching/stopping alone is not finalization.

Both clients must refresh the five-tool Schema 2 catalog and retain existing discover/read/edit meanings. Exercise occupied/missing/stale/dirty and at least one uncertain-result recovery across the actual-client runs: fresh observation and a separate deliberate request, never automatic resend or filesystem fallback. Direct protocol cases additionally own wire cancellation; OMP client-local cancellation is not automatically an MCP cancellation event.

## Constitutional and inherited evidence

| Obligation | Feature 009 treatment |
| --- | --- |
| A — clean open editing | Existing exact-edit contract remains; new composed reads/edits and directly affected shared-owner/validator regressions must preserve D/R/B. Reuse unaffected accepted open-edit evidence only with provenance review. |
| B — dirty human work | New create occupancy/orphan and all delete dirty/equal-text/Resource conflicts, with independent preservation of unrelated dirty work across native close continuation. |
| C — real Undo/Redo | No create/delete Undo claim. Preserve existing edit Undo/Redo and usable unrelated history; exercise actual unrelated history after open deletion, not only availability flags. Rerun existing edit history if its relevant owner/context path changed. |
| D — persistence | New create later open/Save/reparse/rescan/runtime; delete stays absent after applicable ordinary activity. Missing B after verified deletion is a required disappearance postcondition, not waived observation. |
| E — sequential state | New repeated existence/recreation and composed exact-edit workflows; distinct lifetimes, stale old revisions, no silent reversion/replay. Historical sequential-edit evidence is not proof of these new transitions. |

Apply relevant-input review, not blanket suite replay:

| Changed boundary | Exact affected callers/evidence | Reachable failure and proportional check |
| --- | --- | --- |
| v7 authentication/capability record and revision-5 registration | Observation, discovery, open edit, open, close, closed edit, MCP read/edit and new operations | Rejection/misrouting/matched-family errors: exercise strict proof/version mismatch and a real supported request through each affected family. Broaden only if the shared change invalidates more than that boundary. |
| Shared native owner/teardown or extracted close protection | Existing open/edit/close/closed owners that reach the changed lifetime path; especially old close and new open-delete | Wrong-owner dispatch, use-after-teardown or lost continuation/protection: focused overlap/cancel/disable and close preservation cases plus new delete cases. A filename change alone is not full-suite invalidation. |
| Proposed-root validator/source-context helper | Existing preflight/post-change and opening/closing/closed-source purposes actually reaching modified admission/context code | Wrong root identity or altered source/profile/receipt meaning: affected deterministic admission cases and real valid/invalid old-purpose controls. Do not replay all historical editor scenarios. |
| New DTO/output/transport mutating classification | MCP process and new operation interruption/privacy groups; existing edit shared error paths | Invalid inputs dispatched, known effects erased, source disclosure or carrier mismatch: process cases and focused actual-client consumption/interruption. |
| New addon/native files and built artifacts | Existing enabled/disabled hook-only export/privacy modes and affected normal/fixture provenance | Tooling or fault methods shipped, credentials/source leaked: inspect actual exports and new request/failure logs. Reuse only demonstrably unaffected artifacts/evidence. |

Native rebuild/relink needs relevant equivalence or affected reruns, not just a version string. No release candidate, new CI gate, full historical `all` execution or settled-environment security campaign follows from this feature. Record reuse/rerun rationale and actual inputs in this guide's later acceptance evidence, not a new database.

## Acceptance coverage

| Coverage | Scenarios | Requirements | Success criteria |
| --- | --- | --- | --- |
| Validated exact creation and applicable R/B | US1.1, US1.4–5, US1.7 | FR-001, FR-003–005, FR-010–012, FR-017–018 | SC-001, SC-004–005, SC-008 |
| Occupancy, namespace and stale creation | US1.2–3, US1.6 | FR-002–003, FR-009–013 | SC-002, SC-004 |
| Three-profile deletion, empty files and limits | US2.1, US2.6–7 | FR-001, FR-006–011, FR-021 | SC-001, SC-004–006 |
| Current basis, dirty work and unavailable safety | US2.2–5 | FR-002, FR-006–010, FR-012–013 | SC-003–005 |
| Interrupted effects and deliberate recovery | US3.1–4 | FR-010–013, FR-017–018 | SC-004, SC-006, SC-008 |
| Five-tool compatibility and composed workflow | US4.1–5 | FR-001, FR-014–016, FR-019–020 | SC-001, SC-006–007 |
| Timing/privacy/export and cumulative coverage | Cross-cutting all stories | FR-017–020 | SC-008 |

This covers all **23 scenarios, FR-001–FR-021 and SC-001–SC-008**. Cumulative completion needs every item marked newly executed, validly reused, rerun after invalidation or substantively inapplicable with justification. An unavailable observation is never inapplicability. Failed/interrupted/unverified executions are not reusable passes. Measure through complete delivered/model-consumed output for client evidence: five seconds for read/discover, ten for edit/create/delete, with no clock reset or arbitrary-load guarantee.

## Planning evidence and publication boundary

Only the mechanics research and documentation validation are executed at this stage. No product acceptance, new supported version, task completion or feature completion is claimed. The requirement checklist remains a 16/16 requirements-quality record, not execution proof. Planning publication stops before task derivation, granularity review, consistency analysis, implementation, merge, release, Phase 4 or another feature.
