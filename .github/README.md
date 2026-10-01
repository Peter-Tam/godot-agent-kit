# CI and optional live-editor automation

These workflows are shared execution infrastructure. [ci.yml](workflows/ci.yml)
runs required ordinary hosted Rust/caller, native stock-build and workflow
checks. [live-editor.yml](workflows/live-editor.yml) is **optional** automation
for the real-editor suites, not a separate Feature 001 or Feature 002
completion prerequisite.

**Real-editor acceptance is mandatory; dedicated GUI CI is optional.** A
maintainer-operated real Mac with the pinned stock Godot candidate, the
complete suites required by the approved feature cumulative/release gate, and
honestly recorded evidence can satisfy the GUI requirements alongside ordinary CI. An
observation run alone does not prove guarded mutation A–E, runtime durability,
or caller completion. See the [Feature 002 acceptance
runner](../specs/002-edit-open-gdscript/quickstart.md#3-owned-real-editor-runner)
and the [Feature 001 acceptance
record](../specs/001-observe-gdscript-state/quickstart.md#29-rebased-t008-real-editor-acceptance-and-completion-2026-09-27).

Known-path opening has its own [Feature 003 caller acceptance](../specs/003-open-project-gdscript/quickstart.md#3-owned-opening-runner-and-caller-groups).
The ordinary Rust job builds `open-gdscript` alongside the observer and editor.
The optional GUI workflow below still runs the existing edit and observation
suites; those runs do not substitute for complete opening acceptance, including
repeated opening and composed A–E/durability. At the cumulative gate, run opening,
edit and observation serially on the same final delivery head with private
artifacts. The campaign utility below retains evidence at existing scenario boundaries.

Project-script discovery uses the authenticated read-only scope boundary and
[private bridge v4](../specs/004-discover-project-gdscript/contracts/bridge-protocol.md).
Rebuild Rust consumers, install the matching addon and restart owned editors;
v3 peers have no fallback. The native revision-2 family is unchanged.
The ordinary Rust job also builds `discover-gdscripts`. Its
[six first-caller acceptance groups](../specs/004-discover-project-gdscript/quickstart.md#4-owned-discovery-runner-after-implementation)
run through the public caller with independent inventory and editor witnesses.
Discovery's shared selector integration requires complete observation, edit and
opening regressions as well. The optional GUI workflow still covers only edit
and observation; it does not establish discovery acceptance.

## Focused validation and resumable campaigns

During development, run the failed/affected scenario, fix it, then rerun that
scenario. Task completion needs task-owned scenarios plus directly affected
regressions, not the entire historical GUI universe. Use a complete unfiltered
suite when approved acceptance requires whole-suite interaction, a changed
shared boundary can invalidate the suite (authenticated bridge compatibility,
native ABI/family loading, shared editor-operation ownership), or the task is
the feature cumulative/release gate. The existence of a suite or generic
regression caution does not establish an affected obligation.

The [campaign utility](../godot-addon/tests/run_editor_campaign.py) invokes the
existing runners; they retain ownership of all product, D/R/B, history, privacy
and export assertions. Build the existing acceptance inputs first, including
the library used by observation probes and separate production/fault native
artifacts; see the [opening runner prerequisites](../specs/003-open-project-gdscript/quickstart.md#3-owned-opening-runner-and-caller-groups).
From the repository root, set the same absolute inputs for these examples:

```sh
GODOT=/absolute/path/to/Godot
FAULT_NATIVE=/absolute/path/to/fixture-only-native
set -- --godot "$GODOT" \
  --observer "$PWD/mcp-server/target/debug/observe-gdscript" \
  --editor "$PWD/mcp-server/target/debug/edit-gdscript" \
  --opener "$PWD/mcp-server/target/debug/open-gdscript" \
  --stock-validator "$PWD/mcp-server/target/debug/examples/stock_validation_fixture" \
  --native-fault-addon "$FAULT_NATIVE"
```

Focused development (use a new empty private artifact directory each time):

```sh
ARTIFACTS="$(mktemp -d "$HOME/open-interruption.XXXXXX")"
python3 godot-addon/tests/run_script_open.py \
  --scenario interruption --artifacts "$ARTIFACTS" "$@"
```

Run a selected suite on stable code, using a separate campaign directory per suite:

```sh
CAMPAIGN="$(mktemp -d "$HOME/editor-campaign.XXXXXX")"
python3 godot-addon/tests/run_editor_campaign.py \
  --suite open --campaign-dir "$CAMPAIGN" --keep-going "$@"
```

Resume the same unchanged interrupted campaign, retaining `CAMPAIGN`:

```sh
python3 godot-addon/tests/run_editor_campaign.py \
  --suite open --campaign-dir "$CAMPAIGN" --keep-going --resume "$@"
```

`--suite open`, `edit`, and `observation` select the existing suites. Opening
needs `--opener`; edit needs the remaining inputs above; observation needs only
`--godot` and `--observer`. For discovery, use the same inputs plus its caller:

```sh
DISCOVERY_CAMPAIGN="$(mktemp -d "$HOME/discovery-campaign.XXXXXX")"
python3 godot-addon/tests/run_editor_campaign.py \
  --suite discovery --campaign-dir "$DISCOVERY_CAMPAIGN" --keep-going \
  --discoverer "$PWD/mcp-server/target/debug/discover-gdscripts" "$@"
```

Discovery executes `inventory`, `routing`, `coverage`, `interruption`,
`readonly`, and `privacy-export` exactly once. These are first-caller gates,
not full feature acceptance. `--suite all` explicitly refuses while the
additional cumulative discovery groups are unavailable; it never silently
substitutes those six groups for the full gate.

Campaigns are serial, with one isolated subprocess per opening, edit or
discovery scenario in the runner's authoritative order. Observation deliberately
remains one `run_observation.py --scenario all` step: its cross-group
redaction/replay/boundary assertions must stay together. There is no case-level
resume. Discovery fingerprints include its caller and discovery/open/edit
fixtures as well as shared inputs.

Every execution receives a fresh empty private directory such as
`open/routing/attempt-001/`, with the runner's `summary.json`. Failed and
interrupted attempts remain intact; retries allocate a new directory. An atomic
`manifest.json` records per-step status, exit code, execution fingerprint,
attempt/summary paths, summary SHA-256, command identity and executed/reused
disposition, without copying source, credentials or runner-summary payloads.
Without `--keep-going` the campaign stops at the first failure. With it,
remaining independent steps run; the final PASS / FAIL / REUSED listing still
exits nonzero if any required step failed or did not run.

`--resume` reuses only a passed checkpoint whose summary still exists, says
passed, matches its recorded SHA-256, and has an identical current execution
fingerprint. Fingerprints cover applicable executables, native build artifacts,
runner/helpers, fixtures, addon/runtime inputs and behavioral arguments;
documentation edits and commit IDs are not execution identity. Missing, corrupt,
failed or interrupted evidence is rerun, never promoted to passing.

Focused scenario passes are development evidence, not a substitute for an
approved final-head cumulative gate. After a code/build/runtime change, old-build
passes cannot silently prove the new head: rerun the affected scenario during
development, then run the complete required cumulative campaign once the final
inputs are stable. Resume avoids restarting unchanged completed work, not the
acceptance required by a new execution identity. A–E, feature-completion and
release obligations are unchanged.

The remaining sections describe conditions **only for choosing the optional
self-hosted workflow**. It is sourced from `main` even when the tested
`reviewed_sha` is an unmerged PR head, and has no PR, `pull_request_target` or
`workflow_run` trigger for self-hosted execution. Maintainer dispatch from
trusted `main` with the selected exact SHA authorizes that optional run.

## Establish the boundary before dispatch

If using the optional workflow, a repository maintainer must configure its
GitHub environment `live-editor` as follows:

- No deployment-reviewer rules or separate human approval step. The environment
  is a main-only execution boundary, not a two-person authorization mechanism.
- Selected deployment branches/tags configured with exactly **one** policy:
  name `main`, type **branch**. Use custom branch policies, not “protected
  branches only” or unrestricted deployments. Do not add a `main` **tag**,
  wildcard, or any other branch/tag policy. Disable administrator bypass of
  environment protection rules.
- No environment secrets. Also do not supply repository secrets or a PAT to
  the live workflow. Its hosted gate uses the automatically issued,
  short-lived `GITHUB_TOKEN` for read-only GitHub API requests.

Only for that optional workflow, provision a dedicated, clean, single-job
self-hosted macOS/ARM64 GUI runner with the
`godot-live-editor-ephemeral` label. Do **not** register a persistent human
workstation: no human projects, saved credentials, privileged network access,
or retained cross-job state. Runner labels alone are not isolation. A new
clean environment is needed for each job; the runner must be restricted to
this workload. GitHub [warns that self-hosted runners lack the isolation of
hosted VMs](https://docs.github.com/en/actions/reference/security/secure-use#hardening-for-self-hosted-runners).
The workflow does not create the environment, its policies, or the runner.

The environment's branch restriction applies to the run's `GITHUB_REF`
(`refs/heads/main`), **not** to the candidate SHA. The hosted, no-checkout
`trust-gate` validates that policy through read-only metadata APIs and validates
the candidate's current PR state. GitHub's deployment restriction is an additional
main-only boundary; it does not require another person to approve execution.
The maintainer must inspect the selected SHA rather than mistake the trusted
workflow ref for the code being tested. See
[GitHub's environment branch restrictions](https://docs.github.com/en/actions/reference/workflows-and-actions/deployments-and-environments).

## Exact revision validation

Dispatch only the workflow file on `main`, in `Peter-Tam/godot-agent-kit`,
with a full lowercase 40-hex `reviewed_sha`. The hosted gate requires a
`workflow_dispatch` event, `GITHUB_REF=refs/heads/main`, and
`GITHUB_WORKFLOW_REF=Peter-Tam/godot-agent-kit/.github/workflows/live-editor.yml@refs/heads/main`.

- **Main mode:** if `reviewed_sha == GITHUB_SHA` (the exact dispatch revision
  of `main`), PR lookup is unnecessary. Environment branch-policy validation
  still applies.
- **Unmerged PR mode:** otherwise, the commit-to-PR association must contain
  **exactly one eligible PR** with state open, non-draft, unmerged, base ref
  `main`, base and head repositories both `Peter-Tam/godot-agent-kit` (not a
  fork), and `head.sha == reviewed_sha`. Other unrelated, ineligible
  associations do not count. After reading the association list, the gate fetches
  the selected PR directly to reject a moved head or changed PR state. Multiple
  eligible PRs or malformed/duplicate association data fail.

The gate requests associated PRs with `per_page=100&page=N` through the complete
list, with a bounded 100-page limit; truncated, malformed, unavailable, or
ambiguous results fail closed. It does not query reviews or follow API-provided
URLs or fall back to another credential/source of truth. HTTP or JSON errors,
including inaccessible environment metadata, fail before a revision output is
emitted. The accepted SHA is passed to the GUI job only
after every hosted check succeeds. The GUI job checks out that full immutable
SHA with a pinned checkout action and `persist-credentials: false`, verifies
`git rev-parse HEAD` equals the gate's revision before executing repository
code, then verifies the provisioned candidate and runs the native baseline
before building separate production and fixture-only native artifacts. It runs
complete script-edit acceptance first, then complete observation acceptance;
both require `--scenario all`, with no partial scenario selector. Two
synthetic-evidence artifacts, `script-edit-all-${{ github.run_id }}-${{ github.run_attempt }}`
and `observation-all-${{ github.run_id }}-${{ github.run_attempt }}`, distinguish
reruns. The script-edit evidence includes separate native build manifests
identifying the normal and fixture-only binaries. Uploads run with
`if: always()` and retain available evidence for 14 days.

The PR checks are observations at the hosted gate, not a lock on a PR.
Moving a branch after validation cannot change the immutable checkout; testing
the new head requires a new maintainer dispatch with that exact SHA. Cancel an
older queued run if its selected revision should no longer be tested.

The workflow grants `contents: read` globally; only the hosted trust-gate job
adds `actions: read` (environment and branch-policy introspection) and
`pull-requests: read` (commit association and current PR details only). The GUI job
inherits **only** `contents: read`, not the gate's expanded rights. No write
permission is granted. [GitHub's GitHub App installation-token permissions
table](https://docs.github.com/en/rest/authentication/permissions-required-for-github-apps#repository-permissions-for-actions)
lists environment and deployment-branch-policy GET under Actions read, and
[the Pull requests table](https://docs.github.com/en/rest/authentication/permissions-required-for-github-apps#repository-permissions-for-pull-requests)
lists associated-PR and PR GET under Pull requests read (`IAT` means
installation access token). [Workflow syntax for permissions](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#permissions)
documents job-specific permissions and that specifying permissions sets
unspecified scopes to `none`. The official
[commit association](https://docs.github.com/en/rest/commits/commits#list-pull-requests-associated-with-a-commit)
and [PR details](https://docs.github.com/en/rest/pulls/pulls#get-a-pull-request)
APIs supply the exact-head eligibility inputs. No review API is used.

## Trusted GUI execution contract

The optional complete-suite job has a **180-minute** timeout. The checks below
describe the workflow's provisioned-candidate preflight, not Feature 001
acceptance prerequisites or an additional Feature 002 platform requirement.
The hashes identify this pinned candidate rather than claiming other
environments are supported:

- macOS **26.6.2**, **arm64**.
- Godot **4.7.2.stable.official.ed1daf0bf**; the executable resolved by
  `command -v godot` must have SHA-256
  `c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf`.
- Official macOS export template at
  `$HOME/Library/Application Support/Godot/export_templates/4.7.2.stable/macos.zip`,
  with SHA-256
  `88df5e2e6fee99088699be66e6d42e4da4fb0c5619d054297d755a49558a4792`.
- Rust **rustc 1.98.1 (48a229cea 2026-09-01)**, host tuple
  **aarch64-apple-darwin**, Cargo **1.98.1**, and Python **3.10 or newer**.

Missing or mismatched candidates fail; the workflow does not download or
replace tools/templates. The GUI job sets `RUSTUP_AUTO_INSTALL=0` to
[prevent implicit installation by rustup proxies](https://rust-lang.github.io/rustup/environment-variables.html).
Runner provisioning must already include the required toolchain components.

After preflight, the GUI job runs the following from `mcp-server/`, stopping
before GUI acceptance on any failure:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo doc --no-deps --locked
cargo build --locked --lib --bin observe-gdscript --bin edit-gdscript \
  --example stock_validation_fixture
```

The GUI job also builds the official-stock production GDExtension and a separate
`--fixture-faults` artifact outside the product addon via
`godot-addon/native/build.py`, both against that same checked executable. The
normal artifact remains installed for fixture projects and export checks;
fixture-only hooks remain in their distinct test directory. This native
baseline is additional to ordinary hosted CI, not replaced by it.
No new provider, runner, environment or dependency is required for a
maintainer-operated acceptance run.

## Operate and verify

After inspecting and selecting the exact PR head SHA (or exact current `main`
SHA), the maintainer authorizes execution by dispatching from the repository
root; **`--ref main` selects trusted workflow code, not the tested revision**:

```sh
SHA=<full-lowercase-40-hex-reviewed-commit>
gh workflow run live-editor.yml --repo Peter-Tam/godot-agent-kit --ref main -f reviewed_sha="$SHA"
```

The GUI job runs `run_script_edit.py --scenario all` with the absolute stock
Godot, `observe-gdscript`, `edit-gdscript`, test-only
`stock_validation_fixture` and separate fixture-native directory; only after
that process succeeds does it run `run_observation.py --scenario all` against
the same Godot and observer. Each runner receives its own initially empty,
private artifact directory. The editor must be unlocked and visible for real
owned-window images; neither a headless run nor the shell-stage simulations
below substitute for GUI acceptance. For a maintainer-operated invocation,
use the [Feature 002 runner instructions](../specs/002-edit-open-gdscript/quickstart.md#3-owned-real-editor-runner)
with both all-scenario suites and independent review of their results.

For an optional CI run, inspect the gate output, tested SHA and the two
separate synthetic-evidence artifacts, including each native build manifest
and both suites' summaries. A denied gate or missing runner means that
optional run did not execute; a nonzero result from either `--scenario all`
invocation is not passing evidence. This optional route does not replace or
invalidate valid maintainer-operated acceptance.

For local/hosted workflow fixture and configuration validation (Python 3,
PyYAML **6.0.3**, and Actionlint installed; CI pins its Actionlint Go module):

```sh
python3 -m pip install 'PyYAML==6.0.3'
python3 -m unittest discover -s .github/tests -v
python3 -m unittest discover -s godot-addon/tests -p test_editor_campaign.py -v
actionlint -config-file .github/actionlint.yaml .github/workflows/ci.yml .github/workflows/live-editor.yml
```

The `ci.yml` hosted workflow-validation job runs the trust-gate, shell-stage and
campaign-orchestration regressions without starting a GUI runner. Campaign tests
use stub child processes and temporary evidence; shell-stage tests use simulated
candidate metadata. These prove orchestration and refusal, not real-editor
acceptance. Both main-push and main-targeting PR path filters include
`godot-addon/**`, alongside Rust and workflow paths.

The hosted `editor-native` job is the **Stock native build**: it runs native
ABI refusal regressions, verifies the exact official stock archive SHA-256
`c58a24e31d720be9d62f60cb5627c4e695fb72f21b0cfe1bc9ccaa9a3b3ba63e`
before extraction, verifies the executable SHA-256
`c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf`,
and builds the matching public-ABI extension. The archive comes from the
[official upstream release](https://api.github.com/repos/godotengine/godot-builds/releases/tags/4.7.2-stable).
There is no source checkout, SCons, engine patch or oracle build in hosted CI.
Checkout actions retain full-SHA pins and no credentials. This is a
build/API check, not GUI or export acceptance; see the
[stock native guide](../godot-addon/native/README.md).

Workflow configuration and its regression simulations do not prove live-editor
behavior. Complete maintainer-operated real-editor evidence plus ordinary
hosted CI can satisfy the feature acceptance without a protected GUI-CI run.
Constitutional least privilege, compatibility, truthful outcomes and exact
tested-environment claims still apply. This workflow change does not claim
that the optional job ran or that Feature 002 or Roadmap Phase 1 completed.
