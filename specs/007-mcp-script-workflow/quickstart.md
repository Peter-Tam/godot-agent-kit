# Quickstart: Validate the Trusted MCP Script Workflow

**Status:** Planning/validation contract only. The MCP binary, new closed transaction and MCP runner commands below are **implementation deliverables, not present commands at this planning head**. Existing availability checks are identified separately. This guide defines their exact intended invocation and expected evidence; it does not implement them or claim they ran.

Read [plan.md](plan.md), [data-model.md](data-model.md) and [the public contract](contracts/mcp-interface.md). Follow [TEST_POLICY.md](../../TEST_POLICY.md) for scope, evidence reuse and the existing [VM boundary](../../.github/LOCAL_VM.md). No command authorizes a feature task before its own approved tasks/analysis and one-task-one-PR prerequisites.

## Prerequisites and available checks

- Existing exact stock Godot 4.7.2.stable.official.ed1daf0bf, macOS 26.6.2 arm64 profile and engine/native provenance. Keep the recorded support boundary; a different environment is not equivalent support evidence by assumption.
- Rust 1.98.1, edition 2021, tracked locked dependencies; matched private bridge v6/native revision 4 after implementation. Old native revision 3 does not implement closed editing.
- Existing dedicated Tart VM, private synthetic fixture projects, unlocked guest GUI and owned-window captures. Godot/core/registry stay inside the guest for acceptance; no host GUI fallback or personal projects/credentials in the guest.
- Installed independent clients: Codex CLI 0.153.4 and Claude Code 2.1.222. Maintainer-configured model authentication/approval remains client-owned and is never copied into repository/guest evidence. Agent acceptance cannot be replaced with mock calls when model access is unavailable.

The following existing interfaces were exercised during planning and remain runnable from the repository root:

```sh
codex --version
claude --version
node --version
npm --version
python3 godot-addon/tests/run_in_vm.py status
```

Observed versions were Codex 0.153.4, Claude Code 2.1.222, Node 22.22.0/npm 10.9.4; the VM existed with Tart 2.40.1. Its final planning state is stopped. Version/help output establishes availability, not MCP interoperability.

## Build and local connection after implementation

From `mcp-server/`, build the actual new executable and its library with the selected locked feature graph:

```sh
cargo +1.98.1 build --locked --bin godot-agent-kit-mcp
./target/debug/godot-agent-kit-mcp --help
./target/debug/godot-agent-kit-mcp --version
```

Expected: no editor effect, no source/credential output. For ordinary developer use with an already-configured local editor/registry, the client starts that binary with `--registry "$REGISTRY"`. Registry setup/addon enablement/editor startup remain deliberate developer actions. The server does not infer a project from cwd or start Godot.

For implementation verification, start with the smallest affected test target. At Rust-affecting task completion run the applicable baseline from `mcp-server/`:

```sh
cargo +1.98.1 fmt --all -- --check
cargo +1.98.1 clippy --all-targets --locked -- -D warnings
cargo +1.98.1 test --locked
cargo +1.98.1 doc --no-deps --locked
```

Retain default doctests. Native/bridge changes also require their directly affected build/loading/boundary evidence. Do not run these product commands merely because planning documentation changed.

## Owned MCP fixture interfaces after implementation

Extend the existing `run_in_vm.py` fixed suite allowlist with `mcp`, retaining its committed-source, cached-build, provenance, ownership, capture and artifact retrieval semantics. Required groups are `transport`, `closed-native`, `closed-lifecycle`, `preservation`, `interruption`, `composed` and `privacy-export`. They are individually runnable; no new workflow engine or requirement for an aggregate historical `all` mode.

From repository root, on the committed implementation revision:

```sh
python3 godot-addon/tests/run_in_vm.py start
python3 godot-addon/tests/run_in_vm.py run mcp --scenario transport --captures
python3 godot-addon/tests/run_in_vm.py run mcp --scenario closed-native --captures
python3 godot-addon/tests/run_in_vm.py run mcp --scenario closed-lifecycle --captures
```

Run only groups owned/invalidated by the selected task, then the separately required cumulative MCP composition. Expected positive closed results independently establish intended D, same clean loaded R source or confirmed absence, continued absence of B, matching lifecycle/revision and actual validation. A native acknowledgment, disk hash alone or universal refusal is failure.

The `transport` group uses the actual server executable without authorizing an editor mutation for malformed/unsupported cases. It records actual negotiated revision, exact three-tool catalog, result-schema validity, parse/request/tool-error distinctions, UTF-8/depth/size limits, duplicate/unknown IDs, concurrency refusal, stdout purity, blocked-output termination and open-input shutdown. Relevant source/credential sentinels must not occur in stderr or incidental protocol errors.

The fixture controls human state/history/race barriers and independent witnesses, not the MCP tool's success verdict. Use fault artifacts separately from production native artifacts; neither fixtures nor their privileged controls may enter an export.

## Real coding-agent clients after implementation

Add two fixed test-only VM-wrapper commands:

- `prepare-mcp --revision SHA --run-id ID --artifacts DIR`: prepare an owned synthetic project/session and matched binaries; write source-free selected-target setup plus `mcp.json` and `prompt.txt` into the fresh host artifact directory. The run receipt binds actual guest inputs/session and does not contain authentication secrets.
- `mcp-stdio --run-id ID`: validate that prepared run and relay stdin/stdout through the existing private Tart channel to its fixed guest MCP executable/registry. No arbitrary guest command, endpoint or host project is accepted. Protocol bytes are not wrapped in helper status output. Setup/relay diagnostics are source-free stderr.

The relay is test apparatus, not a product remote transport. Host clients retain their normal model access; editor/core effects remain in the owned guest. This is the minimal adaptation needed to combine real clients with the existing VM policy.

Example preparation from repository root:

```sh
REPO="$PWD"
REVISION="$(git rev-parse HEAD)"
RUN_ID="mcp-agent-$(date -u +%Y%m%dT%H%M%SZ)"
ARTIFACTS="$(mktemp -d "$HOME/mcp-agent.XXXXXX")"
python3 godot-addon/tests/run_in_vm.py prepare-mcp \
  --revision "$REVISION" --run-id "$RUN_ID" --artifacts "$ARTIFACTS"
```

The generated `mcp.json` configures the `godot_agent_kit` stdio server as the fixed relay above. The prompt gives the exact owned project/session, requested source change and human intent; it must not teach safety through a long private architecture explanation. Catalog/result semantics must be sufficient on their own.

Observed Claude CLI help supports this invocation-scoped configuration, without changing personal MCP settings:

```sh
claude --strict-mcp-config --mcp-config "$ARTIFACTS/mcp.json" \
  --tools "" \
  --allowedTools mcp__godot_agent_kit__discover_scripts,mcp__godot_agent_kit__read_script,mcp__godot_agent_kit__edit_script \
  --no-session-persistence --output-format stream-json --verbose \
  -p < "$ARTIFACTS/prompt.txt"
```

Codex's observed exec/config interface permits a separate invocation, retaining normal authentication but not changing user configuration:

```sh
codex exec --ignore-user-config --ephemeral --sandbox read-only --json \
  -c 'mcp_servers.godot_agent_kit.command="python3"' \
  -c "mcp_servers.godot_agent_kit.args=[\"$REPO/godot-addon/tests/run_in_vm.py\",\"mcp-stdio\",\"--run-id\",\"$RUN_ID\"]" \
  -c 'mcp_servers.godot_agent_kit.startup_timeout_sec=10' \
  -c 'mcp_servers.godot_agent_kit.tool_timeout_sec=15' \
  - < "$ARTIFACTS/prompt.txt"
```

Use a fresh prepared run for the second client and retain actual tool-call/result transcripts. Client timeout exceeds the server contract so it does not hide the server's deadline result. Deny direct file/shell/private-bridge substitutions; any such substitution invalidates that agent workflow proof. The guest fixture is not a host workspace file path. Do not use global permission-bypass flags. The fixture harness must separately assert state; model claims are not acceptance.

Each client must actually connect, discover the exact catalog and complete representative discover/read/edit calls. At least one real-agent conversation must perform the full composed workflow and applicable A–E below. Add/list/config output alone is not proof. If the selected old-revision negotiation or result visibility fails on a client, report and correct the design/compatibility claim before declaring support; do not silently test only a custom client.

## Acceptance coverage

| Group / evidence | Exact approved scenarios | FR / SC coverage and required observations |
| --- | --- | --- |
| Transport and both independent clients | US1.1–US1.4 | FR-001–FR-004, FR-012–FR-017, FR-021; SC-001, SC-004, SC-006–SC-007. Negotiation, unsupported requests, editor-unavailable versus connection success, bounded errors and no incidental disclosure/effect. |
| Qualitative interface review and actual agent use | US1.5 | FR-002, FR-022; SC-009. Agent chooses operation, supplies required state, interprets structured results and safe next action without private knowledge, redundant warnings or exhaustive static prose. No numerical concision threshold. |
| Closed-native and closed-lifecycle | US2.1, US2.3–US2.5, US5.5 | FR-005–FR-011, FR-014, FR-018–FR-020; SC-002–SC-006, SC-008. Positive cached/absent R, current basis, no opening, no-op recognition, missing/dirty/divergent R refusal, namespace/cache/lifecycle races, independent closed postconditions and later opening durability. |
| Preservation and stale state | US2.2, US3.1–US3.4, US3.7 | FR-004–FR-010, FR-018–FR-020; SC-002–SC-003, SC-008. Open guarantees, dirty/equal-dirty human state/history, same-text versions/file revisions, closed→open→closed epoch, target/session replacement, confinement and unobservable evidence. |
| Interruption and overlapping calls | US3.5–US3.6, US4.1–US4.4 | FR-003–FR-004, FR-007, FR-010–FR-017, FR-020; SC-003–SC-004, SC-006–SC-007. Actual pre/post-authorization cancel/EOF/timeout/output-loss/disable, preserved newer work, exact effect certainty, no late effect after proven refusal, no queue/replay or cross-call cancellation. |
| Real-agent MCP composed A–E | US5.1–US5.4 | FR-019–FR-020; SC-001–SC-005, SC-008–SC-009. A/B clean-open edit and dirty protection; C genuine native Undo/Save/Redo/Save for open edits; D ordinary fixture close/reopen and permitted durability; E twenty fresh-basis successful MCP edits covering open/closed with at least three dirty and three stale refusals. |
| Privacy/export and existing-contract review | US1–US5 relevant error/selection paths and all privacy/export edge cases | FR-016–FR-018, FR-020–FR-021; SC-007–SC-008. Authorized results only, source-free catalog/logs/errors, sticky disclosure denial after causal failure, enabled/disabled/hook-only exports, all existing local contracts unchanged. |

This covers all **26 scenarios, FR-001–FR-022 and SC-001–SC-009**. Cross-cutting routing, timing, privacy and outcome meaning apply to every relevant group, not just one named row. Closed positive cases supplement A, never replace its open-buffer proof.

For C, native Undo/Redo must independently reverse/reapply real open-document source with prior history intact; no second source-edit call substitutes for history. For D, later opening is a durability witness after already-verified closed editing, not the method of obtaining success. Ordinary reparse/rescan/fresh runtime must retain intended source without reconciliation; loaded class metadata is not assumed hot-reloaded. E uses a fresh trusted read/basis for every intentional edit; losing a response requires another fresh read rather than replay.

## Evidence reuse and completion

Reuse [Phase 1 accepted evidence](../../PHASE_1_EXIT.md#evidence-reuse-and-currentness), [Phase 2 ownership/evidence](../../PHASE_2_EXIT.md#evidence-currentness), [Feature 002 acceptance](../002-edit-open-gdscript/quickstart.md#14-t005-cumulative-acceptance-2026-09-29) and [Feature 005 cumulative review](../005-close-project-gdscript/quickstart.md#t003-execution-decision-and-evidence-review) only after relevant-input/provenance review.

The private v6/native revision-4 cutover changes registration/negotiation/owner inputs. Prove affected capability/refusal/slot/cancellation/loading/export compatibility and representative existing open-edit/observation/discovery/lifecycle preservation. If mutation-relevant code or a witness changes, rerun its affected cases. A rebuilt library needs demonstrated relevant equivalence or those affected reruns; unchanged filenames/version strings alone are not equivalence. No blanket historical all-suite campaign is imposed.

Retain exact committed source, engine/platform, SDK/client versions, dependency lock, native/ABI/build provenance, fixture/witness identity, negotiated revision, admission/delivery timings, independent authority/history/lifecycle records and owned captures. Keep raw source-bearing research/acceptance artifacts private and outside Git. Failure/partial/interrupted evidence is not a pass. Record reused/new/invalidated/not-applicable evidence against the matrix before completion.

After an owned run, retrieve artifacts and stop the VM using the existing wrapper; close only owned processes/projects. A complete feature still requires approved implementation tasks, their actual acceptance and shape reviews. This planning guide does not satisfy those future gates.

## Planning evidence only

Planning ran static documentation checks, repository/LSP and released-source research, installed client version/help checks, VM status/start/stop, and the scoped [closed-source mechanics probe](research.md#owned-stock-editor-observations) with visual inspection and fresh-runtime persistence witness. The two incomplete probe attempts remain excluded from the completed observation. No MCP server/client interoperability, product test suite, native build, historical campaign, A–E acceptance, release or Phase 3 exit is claimed.
