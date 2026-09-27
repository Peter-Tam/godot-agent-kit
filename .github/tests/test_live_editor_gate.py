"""Behavioral regressions for the inline, pre-checkout live-editor trust gate."""

import copy
import io
import json
import os
from pathlib import Path
import re
import tempfile
import textwrap
import unittest
from unittest import mock
import urllib.error
import urllib.parse
import urllib.request

import yaml


REPOSITORY = "Peter-Tam/godot-agent-kit"
API_ROOT = f"https://api.github.com/repos/{REPOSITORY}"
WORKFLOW_REF = f"{REPOSITORY}/.github/workflows/live-editor.yml@refs/heads/main"
MAIN_SHA = "b" * 40
PR_SHA = "a" * 40
OTHER_SHA = "c" * 40
ENVIRONMENT = "/environments/live-editor"
BRANCHES = ENVIRONMENT + "/deployment-branch-policies"


class UnexpectedAPI(AssertionError):
    """An unconfigured request is a test error, never an authorization denial."""


class GateTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        workflow = Path(__file__).resolve().parents[1] / "workflows/live-editor.yml"
        with workflow.open(encoding="utf-8") as source:
            steps = yaml.load(source, Loader=yaml.BaseLoader)["jobs"]["trust-gate"]["steps"]
        run = next(step["run"] for step in steps if step.get("id") == "verify")
        match = re.fullmatch(r"python3 - <<'PY'\n(?P<body>.*?)\nPY\n?", run, re.DOTALL)
        if match is None:
            raise AssertionError("trust-gate verify step must be inline Python")
        cls.gate = compile(textwrap.dedent(match["body"]), str(workflow), "exec")

    def setUp(self):
        self.responses = {
            ENVIRONMENT: {
                "deployment_branch_policy": {
                    "custom_branch_policies": True,
                    "protected_branches": False,
                },
            },
            BRANCHES: {
                "total_count": 1,
                "branch_policies": [{"name": "main", "type": "branch"}],
            },
        }
        self.pages = {}
        self.sequence = {}
        self.calls = []
        self.associations = f"/commits/{PR_SHA}/pulls"
        self.pr_path = "/pulls/17"
        self.pr = {
            "number": 17,
            "state": "open",
            "draft": False,
            "merged_at": None,
            "merged": False,
            "base": {"ref": "main", "repo": {"full_name": REPOSITORY}},
            "head": {"sha": PR_SHA, "repo": {"full_name": REPOSITORY}},
        }

    def pr_candidate(self):
        self.pages[self.associations] = {1: [copy.deepcopy(self.pr)]}
        self.sequence[self.pr_path] = [copy.deepcopy(self.pr)]

    def urlopen(self, request, timeout=None):
        url = request.full_url
        parts = urllib.parse.urlsplit(url)
        if (parts.scheme, parts.netloc, parts.path[:len(f"/repos/{REPOSITORY}")]) != (
            "https", "api.github.com", f"/repos/{REPOSITORY}"
        ):
            raise UnexpectedAPI(f"unexpected API origin: {url}")
        path = parts.path[len(f"/repos/{REPOSITORY}"):]
        self.calls.append((path, parts.query))
        if path in self.pages:
            try:
                query = urllib.parse.parse_qs(parts.query, strict_parsing=True)
            except ValueError as exc:
                raise UnexpectedAPI(f"unexpected pagination: {url}") from exc
            if (set(query) != {"per_page", "page"} or query["per_page"] != ["100"]
                    or len(query["page"]) != 1):
                raise UnexpectedAPI(f"unexpected pagination: {url}")
            try:
                page = int(query["page"][0])
                response = self.pages[path][page]
            except (KeyError, IndexError, ValueError) as exc:
                raise UnexpectedAPI(f"unexpected page: {url}") from exc
        elif not parts.query and path in self.sequence:
            if not self.sequence[path]:
                raise UnexpectedAPI(f"unexpected repeated API read: {url}")
            response = self.sequence[path].pop(0)
        elif not parts.query and path in self.responses:
            response = self.responses[path]
        else:
            raise UnexpectedAPI(f"unexpected API read: {url}")
        if isinstance(response, BaseException):
            raise response
        if isinstance(response, bytes):
            return io.BytesIO(response)
        return io.BytesIO(json.dumps(response).encode("utf-8"))

    def gate_result(self, *, reviewed=MAIN_SHA, github_sha=MAIN_SHA, overrides=None):
        environment = {
            "REVIEWED_SHA": reviewed,
            "GH_TOKEN": "test-token",
            "GITHUB_REPOSITORY": REPOSITORY,
            "GITHUB_REF": "refs/heads/main",
            "GITHUB_SHA": github_sha,
            "GITHUB_EVENT_NAME": "workflow_dispatch",
            "GITHUB_WORKFLOW_REF": WORKFLOW_REF,
        }
        environment.update(overrides or {})
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "github-output"
            output.touch()
            environment["GITHUB_OUTPUT"] = str(output)
            with mock.patch.dict(os.environ, environment, clear=True), mock.patch.object(
                urllib.request, "urlopen", side_effect=self.urlopen
            ):
                failure = None
                try:
                    exec(self.gate, {"__name__": "__main__"})
                except UnexpectedAPI:
                    raise
                except (Exception, SystemExit) as exc:
                    failure = exc
            return failure, output.read_text(encoding="utf-8")

    def authorized(self, *, reviewed=MAIN_SHA, github_sha=MAIN_SHA, overrides=None):
        failure, output = self.gate_result(
            reviewed=reviewed, github_sha=github_sha, overrides=overrides
        )
        self.assertIsNone(failure, f"gate rejected an authorized revision: {failure!r}")
        self.assertEqual(output, f"revision={reviewed}\n")

    def denied(self, *, reviewed=PR_SHA, github_sha=MAIN_SHA, overrides=None):
        failure, output = self.gate_result(
            reviewed=reviewed, github_sha=github_sha, overrides=overrides
        )
        self.assertIsNotNone(failure, "gate authorized an untrusted revision")
        self.assertEqual(output, "", "rejected revision must not reach GITHUB_OUTPUT")

    def test_main_dispatch_without_associated_pr(self):
        self.authorized()
        self.assertEqual([path for path, _ in self.calls], [ENVIRONMENT, BRANCHES])

    def test_exact_pr_head_from_manual_dispatch(self):
        self.pr_candidate()
        self.authorized(reviewed=PR_SHA)

    def test_invalid_revision_formats_and_shell_output_injection(self):
        for revision in ("A" * 40, "abc", PR_SHA + "\nrevision=" + MAIN_SHA, PR_SHA + ";echo bogus"):
            with self.subTest(revision=revision):
                self.denied(reviewed=revision)

    def test_unassociated_revision_and_stale_pr_head(self):
        self.pages[self.associations] = {1: []}
        self.denied()
        self.pr_candidate()
        self.pr["head"]["sha"] = OTHER_SHA
        self.pages[self.associations][1] = [copy.deepcopy(self.pr)]
        self.denied()

    def test_ineligible_associations_never_authorize(self):
        mutations = {
            "wrong base branch": lambda pr: pr["base"].update(ref="develop"),
            "foreign base repository": lambda pr: pr["base"]["repo"].update(full_name="elsewhere/repo"),
            "draft": lambda pr: pr.update(draft=True),
            "closed": lambda pr: pr.update(state="closed"),
            "merged timestamp": lambda pr: pr.update(merged_at="2026-09-25T12:00:00Z"),
            "fork": lambda pr: pr["head"]["repo"].update(full_name="elsewhere/repo"),
        }
        for label, mutate in mutations.items():
            with self.subTest(case=label):
                self.pr_candidate()
                associated = copy.deepcopy(self.pr)
                mutate(associated)
                self.pages[self.associations][1] = [associated]
                self.denied()

    def test_fetched_pr_must_still_be_eligible(self):
        mutations = {
            "changed head": lambda pr: pr["head"].update(sha=OTHER_SHA),
            "wrong base": lambda pr: pr["base"].update(ref="feature"),
            "fork": lambda pr: pr["head"]["repo"].update(full_name="fork/repo"),
            "closed": lambda pr: pr.update(state="closed"),
            "merged": lambda pr: pr.update(merged=True),
            "draft": lambda pr: pr.update(draft=True),
        }
        for label, mutate in mutations.items():
            with self.subTest(case=label):
                self.pr_candidate()
                fetched = copy.deepcopy(self.pr)
                mutate(fetched)
                self.sequence[self.pr_path] = [fetched]
                self.denied()

    def test_exactly_one_eligible_association_even_with_other_ineligible_prs(self):
        self.pr_candidate()
        unrelated = copy.deepcopy(self.pr)
        unrelated.update(number=18, state="closed")
        self.pages[self.associations][1].append(unrelated)
        self.authorized(reviewed=PR_SHA)
        self.pr_candidate()
        unrelated["state"] = "open"
        self.pages[self.associations][1].append(unrelated)
        self.denied()

    def test_duplicate_and_malformed_association_metadata_fail_closed(self):
        for label, extra in (
            ("duplicate number", dict(self.pr)),
            ("missing number", {"state": "closed"}),
            ("boolean number", dict(self.pr, number=True)),
            ("missing head", dict(self.pr, number=18, head=None)),
        ):
            with self.subTest(case=label):
                self.pr_candidate()
                self.pages[self.associations][1].append(extra)
                self.denied()

    def test_wrong_dispatch_context_fails_even_for_exact_main_revision(self):
        for label, changes in (
            ("ref", {"GITHUB_REF": "refs/heads/feature"}),
            ("tag", {"GITHUB_REF": "refs/tags/v1"}),
            ("repository", {"GITHUB_REPOSITORY": "attacker/godot-agent-kit"}),
            ("event", {"GITHUB_EVENT_NAME": "pull_request_target"}),
            ("automatic PR event", {"GITHUB_EVENT_NAME": "pull_request"}),
            ("workflow run event", {"GITHUB_EVENT_NAME": "workflow_run"}),
            ("workflow ref", {"GITHUB_WORKFLOW_REF": WORKFLOW_REF.replace("@refs/heads/main", "@refs/heads/feature")}),
            ("workflow repository", {"GITHUB_WORKFLOW_REF": WORKFLOW_REF.replace(REPOSITORY, "attacker/repo")}),
        ):
            with self.subTest(case=label):
                self.denied(reviewed=MAIN_SHA, overrides=changes)

    def test_environment_and_branch_policy_fail_closed(self):
        mutations = {
            "custom policy disabled": lambda: self.responses[ENVIRONMENT]["deployment_branch_policy"].update(custom_branch_policies=False),
            "protected branches enabled": lambda: self.responses[ENVIRONMENT]["deployment_branch_policy"].update(protected_branches=True),
            "no branch rule": lambda: self.responses[BRANCHES].update(total_count=0, branch_policies=[]),
            "extra branch rule": lambda: self.responses[BRANCHES].update(total_count=2, branch_policies=[{"name": "main", "type": "branch"}, {"name": "other", "type": "branch"}]),
            "wrong branch": lambda: self.responses[BRANCHES]["branch_policies"][0].update(name="release"),
            "wildcard branch": lambda: self.responses[BRANCHES]["branch_policies"][0].update(name="*"),
            "tag allowed": lambda: self.responses[BRANCHES]["branch_policies"][0].update(type="tag"),
        }
        for label, mutate in mutations.items():
            with self.subTest(case=label):
                self.setUp()
                mutate()
                self.denied(reviewed=MAIN_SHA)

    def test_missing_environment_does_not_authorize_main(self):
        self.responses[ENVIRONMENT] = urllib.error.HTTPError(
            API_ROOT + ENVIRONMENT, 404, "not found", {}, None
        )
        self.denied(reviewed=MAIN_SHA)

    def test_exact_pr_head_on_second_association_page(self):
        self.pr_candidate()
        unrelated = [
            dict(self.pr, number=number, state="closed")
            for number in range(18, 118)
        ]
        self.pages[self.associations] = {1: unrelated, 2: [copy.deepcopy(self.pr)]}
        self.authorized(reviewed=PR_SHA)

    def test_second_page_cannot_hide_another_eligible_pr(self):
        self.pr_candidate()
        unrelated = []
        for number in range(18, 117):
            item = copy.deepcopy(self.pr)
            item.update(number=number, state="closed")
            unrelated.append(item)
        another = copy.deepcopy(self.pr)
        another["number"] = 117
        self.pages[self.associations] = {1: [copy.deepcopy(self.pr), *unrelated], 2: [another]}
        self.denied()

    def test_http_and_json_failures_do_not_release_a_revision(self):
        failures = (
            ("environment HTTP", ENVIRONMENT, "response", "http"),
            ("environment JSON", ENVIRONMENT, "response", "json"),
            ("branch HTTP", BRANCHES, "response", "http"),
            ("association HTTP", self.associations, "page", "http"),
            ("association JSON", self.associations, "page", "json"),
            ("PR HTTP", self.pr_path, "pr", "http"),
            ("PR JSON", self.pr_path, "pr", "json"),
        )
        for label, path, stage, kind in failures:
            with self.subTest(case=label):
                self.setUp()
                self.pr_candidate()
                broken = (urllib.error.HTTPError(API_ROOT + path, 403, "forbidden", {}, None)
                          if kind == "http" else b"{invalid json")
                if stage == "response":
                    self.responses[path] = broken
                elif stage == "page":
                    self.pages[path][1] = broken
                else:
                    self.sequence[path][0] = broken
                self.denied()


if __name__ == "__main__":
    unittest.main()
