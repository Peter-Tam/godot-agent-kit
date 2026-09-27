# CI and trusted live-editor operations

These workflows are shared execution infrastructure, not Feature 001-specific release
machinery. [ci.yml](workflows/ci.yml) runs ordinary hosted checks;
[live-editor.yml](workflows/live-editor.yml) is a separate, manually dispatched
trust boundary for real GUI Godot acceptance. The live workflow is sourced from
`main` even when the **tested** `reviewed_sha` is the head of an unmerged PR.
It has no PR, `pull_request_target`, or `workflow_run` trigger that would
run PR code automatically on a self-hosted machine.

## Establish the boundary before dispatch

A repository maintainer must create the GitHub environment `live-editor` and
configure all of the following **in GitHub**:

- Required deployment reviewer(s), with **prevent self-review** enabled. A
  separate eligible person must approve the environment job; the person who
  initiated the run cannot approve their own deployment.
- Selected deployment branches/tags configured with exactly **one** policy:
  name `main`, type **branch**. Use custom branch policies, not “protected
  branches only” or unrestricted deployments. Do not add a `main` **tag**,
  wildcard, or any other branch/tag policy. Disable administrator bypass of
  environment protection rules.
- No environment secrets. Also do not supply repository secrets or a PAT to
  the live workflow. Its hosted gate uses the automatically issued,
  short-lived `GITHUB_TOKEN` for read-only GitHub API requests.

A maintainer must independently provision a dedicated, clean, single-job
self-hosted macOS/ARM64 GUI runner with the
`godot-live-editor-ephemeral` label. Do **not** register a persistent human
workstation: no human projects, saved credentials, privileged network access,
or retained cross-job state. Runner labels alone are not isolation. A new
clean environment is needed for each job; the runner must be restricted to
this workload. GitHub [warns that self-hosted runners lack the isolation of
hosted VMs](https://docs.github.com/en/actions/reference/security/secure-use#hardening-for-self-hosted-runners).
The workflow does not create the environment, its policies, or the runner.

GitHub itself holds the GUI job that names `environment: live-editor` until
its deployment approval and branch restriction are met. Those rules apply to
the run's `GITHUB_REF` (`refs/heads/main`), **not** to the candidate SHA.
The hosted, no-checkout `trust-gate` separately checks the environment's
configuration through read-only metadata APIs, and checks the candidate's
PR/review state; this validation does **not** create or enforce the
environment approval. Reviewers of the waiting deployment must inspect the
validated candidate SHA, not mistake the `main` workflow ref for the code
being tested. Administrator bypass must remain disabled by maintainer policy;
metadata validation is not a substitute for that setting.
[GitHub's environment rules](https://docs.github.com/en/actions/reference/workflows-and-actions/deployments-and-environments)
explain the independent approval, self-review, branch/tag matching, and bypass
semantics.

## Exact revision and review gate

Dispatch only the workflow file on `main`, in `Peter-Tam/godot-agent-kit`,
with a full lowercase 40-hex `reviewed_sha`. The hosted gate requires a
`workflow_dispatch` event, `GITHUB_REF=refs/heads/main`, and
`GITHUB_WORKFLOW_REF=Peter-Tam/godot-agent-kit/.github/workflows/live-editor.yml@refs/heads/main`.

- **Main mode:** if `reviewed_sha == GITHUB_SHA` (the exact dispatch revision
  of `main`), PR lookup is unnecessary. Environment configuration validation
  and the independent GitHub environment approval still apply.
- **Unmerged PR mode:** otherwise, the commit-to-PR association must contain
  **exactly one eligible PR** with state open, non-draft, unmerged, base ref
  `main`, base and head repositories both `Peter-Tam/godot-agent-kit` (not a
  fork), and `head.sha == reviewed_sha`. Other unrelated, ineligible
  associations do not count. The gate fetches the selected PR directly and
  re-fetches it after reading reviews to detect a moved head or changed PR
  state. Multiple eligible PRs or malformed/duplicate association data fail.
- An **independent** reviewer, identified by numeric GitHub user ID rather
  than mutable login and different from the PR author, must have a **current
  submitted APPROVED review whose `commit_id` is exactly `reviewed_sha`**.
  For each user the gate considers their latest submitted review by UTC
  `submitted_at` across *all* states and SHAs, not only approvals for the
  candidate. A later CHANGES_REQUESTED, COMMENTED, DISMISSED, or approval
  for another SHA supersedes that user's older approval. A PENDING review
  conservatively disqualifies that reviewer; a missing/invalid timestamp,
  identity, or state fails closed, and tied latest timestamps cannot certify
  that reviewer's approval. Another independently qualifying reviewer can
  satisfy the gate. Review of an earlier commit is not approval of a moved
  head.

The gate requests associated PRs and reviews with `per_page=100&page=N`
through each complete list, with a bounded 100-page limit; truncated,
malformed, unavailable, or ambiguous results fail closed. It does not follow
API-provided URLs or fall back to another credential/source of truth. HTTP or
JSON errors, including inaccessible environment metadata, fail before a
revision output is emitted. The accepted SHA is passed to the GUI job only
after every hosted check succeeds. The GUI job checks out that full immutable
SHA with a pinned checkout action and `persist-credentials: false`, verifies
`git rev-parse HEAD` equals the gate's revision before executing repository
code, then runs the existing OS/Godot/Rust checks and the real-editor driver
with mandatory `--scenario all`. There is no partial scenario selector for
this protected run. Its evidence artifact is `observation-all-${{ github.run_id }}`.

The PR/review checks are observations at the hosted gate, not a lock on a PR.
Moving a branch while environment approval is pending cannot change the
validated immutable checkout; testing the new head requires a new dispatch and
a qualifying review of that new SHA. Cancel an older queued run if its
reviewed revision should no longer be tested.

The workflow grants `contents: read` globally; only the hosted trust-gate job
adds `actions: read` (environment and branch-policy introspection) and
`pull-requests: read` (association, PR details, and reviews). The GUI job
inherits **only** `contents: read`, not the gate's expanded rights. No write
permission is granted. [GitHub's GitHub App installation-token permissions
table](https://docs.github.com/en/rest/authentication/permissions-required-for-github-apps#repository-permissions-for-actions)
lists environment and deployment-branch-policy GET under Actions read, and
[the Pull requests table](https://docs.github.com/en/rest/authentication/permissions-required-for-github-apps#repository-permissions-for-pull-requests)
lists associated-PR, PR, and review GET under Pull requests read (`IAT` means
installation access token). [Workflow syntax for permissions](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#permissions)
documents job-specific permissions and that specifying permissions sets
unspecified scopes to `none`. The official [commit association](https://docs.github.com/en/rest/commits/commits#list-pull-requests-associated-with-a-commit),
[PR details](https://docs.github.com/en/rest/pulls/pulls#get-a-pull-request),
and [review list](https://docs.github.com/en/rest/pulls/reviews#list-reviews-for-a-pull-request)
APIs define the reviewed-state inputs; the review list includes `commit_id`
and `submitted_at`, while PENDING reviews are not submitted.

## Operate and verify

After reviewing the exact PR head SHA (or selecting the exact current `main`
SHA), the maintainer dispatches from the repository root; **`--ref main`
selects trusted workflow code, not the tested revision**:

```sh
SHA=<full-lowercase-40-hex-reviewed-commit>
gh workflow run live-editor.yml --repo Peter-Tam/godot-agent-kit --ref main -f reviewed_sha="$SHA"
```

Then an eligible deployment reviewer checks the run's gate output and input
SHA, reviews the actual candidate and approves the **environment** job in
GitHub. Do not accept a PR review as a substitute for this independent
approval. A denied gate, missing runner, or nonzero `--scenario all` result
is not acceptance; inspect the synthetic evidence only after a real GUI run.

For local/hosted workflow fixture and configuration validation (Python 3,
PyYAML **6.0.3**, and Actionlint installed; CI pins its Actionlint Go module):

```sh
python3 -m pip install 'PyYAML==6.0.3'
python3 -m unittest discover -s .github/tests -v
actionlint -config-file .github/actionlint.yaml .github/workflows/ci.yml .github/workflows/live-editor.yml
```

The `ci.yml` hosted workflow-validation job runs the fixture/config checks
without starting a GUI runner. Run normal hosted CI separately before
proposing infrastructure changes; fixture tests and Actionlint do not prove
an actual protected deployment or a real-editor result.

At the time of this correction, read-only repository API inspection found
**zero `live-editor` environments and zero GUI runners**; maintainer
provisioning remains outstanding. No protected GitHub GUI run has been
performed by this change. The current `main` acceptance harness still fails
`--scenario all` for missing coverage: this workflow must not be treated as
a completed full-feature acceptance gate, supported Godot/platform evidence,
or completion of T008. Constitution V's least privilege is preserved by
read-only scoped credentials and refusing untrusted runner execution; VI/X
require actual complete live-editor evidence and truthful failed/missing
coverage, not simulated or partial success. Product protocol/editor boundaries
and constitutional VII remain unchanged; these workflows add no product
capabilities or mutation guarantees.
