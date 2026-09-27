# CI and optional live-editor automation

These workflows are shared execution infrastructure. [ci.yml](workflows/ci.yml)
runs the required ordinary hosted native/workflow checks.
[live-editor.yml](workflows/live-editor.yml) is **optional future automation**
for reproducing the same real-editor acceptance in CI, not a T008 or Feature 001
completion prerequisite.

**Real-editor acceptance is mandatory; dedicated GUI CI is optional.** A
maintainer-operated real Mac with the pinned Godot candidate, the complete
passing `--scenario all` suite and honestly recorded environment/evidence
satisfies the GUI requirement. Lack of a self-hosted runner does not block
T008 when that substantive evidence and ordinary hosted CI pass. See
[Feature 001 acceptance](../specs/001-observe-gdscript-state/quickstart.md#29-rebased-t008-real-editor-acceptance-and-completion-2026-09-27).

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
before the real-editor driver with mandatory `--scenario all`. There is no
partial scenario selector. Evidence is named
`observation-all-${{ github.run_id }}-${{ github.run_attempt }}` so reruns have
distinct artifacts; uploads still run with `if: always()` and retain evidence
for 14 days.

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
describe its existing preflight, not Feature 001 acceptance prerequisites.
In particular, executable/template hashes remain useful recorded provenance;
matching these historical hashes is not a T008 completion gate. This correction
does not change the optional workflow's implementation:

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
cargo build --locked --lib --bin observe-gdscript
```

This native baseline is additional to ordinary hosted CI, not replaced by it.
It validates the tested immutable revision on the exact GUI candidate.

## Operate and verify

After inspecting and selecting the exact PR head SHA (or exact current `main`
SHA), the maintainer authorizes execution by dispatching from the repository
root; **`--ref main` selects trusted workflow code, not the tested revision**:

```sh
SHA=<full-lowercase-40-hex-reviewed-commit>
gh workflow run live-editor.yml --repo Peter-Tam/godot-agent-kit --ref main -f reviewed_sha="$SHA"
```

For an optional CI run, inspect the gate output, tested SHA and actual synthetic
evidence. A denied gate or missing runner means that optional run did not
execute; a nonzero `--scenario all` result is not passing real-editor evidence.
None replaces or invalidates valid maintainer-operated acceptance.

For local/hosted workflow fixture and configuration validation (Python 3,
PyYAML **6.0.3**, and Actionlint installed; CI pins its Actionlint Go module):

```sh
python3 -m pip install 'PyYAML==6.0.3'
python3 -m unittest discover -s .github/tests -v
actionlint -config-file .github/actionlint.yaml .github/workflows/ci.yml .github/workflows/live-editor.yml
```

The `ci.yml` hosted workflow-validation job runs the trust-gate and shell-stage
regressions without starting a GUI runner. Shell-stage tests use simulated
candidate metadata to prove refusal and execution ordering, not real-editor
acceptance. Both main-push and main-targeting PR path filters include
`godot-addon/tests/**`: harness/fixture changes rerun workflow validation and
native hosted checks. This does not imply hosted product-addon coverage;
`godot-addon/addons/**` is not included. Fixture tests and Actionlint do not
prove an actual protected deployment or real-editor result.

Workflow configuration and its regression simulations do not prove live-editor
behavior. Complete maintainer-operated real-editor evidence plus ordinary
hosted CI can complete Feature 001 without a protected GUI-CI run. Constitutional
VI/X and compatibility requirements preserve actual observed editor evidence,
truthful outcomes and exact tested-environment claims; they do not require a
particular CI provider. If this optional self-hosted workflow is used,
Constitution V's least-privilege boundary still requires trusted dispatch,
read-only scoped credentials and isolated execution. Product protocol/editor
boundaries remain unchanged; no mutation or Phase 1 completion is implied.
