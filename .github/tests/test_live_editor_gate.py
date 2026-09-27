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
                "protection_rules": [{
                    "type": "required_reviewers",
                    "reviewers": [{"type": "User", "reviewer": {"id": 900, "login": "environment-reviewer"}}],
                    "prevent_self_review": True,
                }],
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
        self.reviews_path = self.pr_path + "/reviews"
        self.pr = {
            "number": 17,
            "state": "open",
            "draft": False,
            "merged_at": None,
            "merged": False,
            "base": {"ref": "main", "repo": {"full_name": REPOSITORY}},
            "head": {"sha": PR_SHA, "repo": {"full_name": REPOSITORY}},
            "user": {"id": 100, "login": "author"},
        }
        self.approval = {
            "id": 1,
            "user": {"id": 200, "login": "independent-reviewer"},
            "state": "APPROVED",
            "commit_id": PR_SHA,
            "submitted_at": "2026-09-25T12:00:00Z",
        }

    def pr_candidate(self):
        self.pages[self.associations] = {1: [copy.deepcopy(self.pr)]}
        self.sequence[self.pr_path] = [copy.deepcopy(self.pr), copy.deepcopy(self.pr)]
        self.pages[self.reviews_path] = {1: [copy.deepcopy(self.approval)]}

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

    def test_exact_pr_head_with_independent_current_approval(self):
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
                self.sequence[self.pr_path] = [fetched, copy.deepcopy(fetched)]
                self.denied()

    def test_no_reviews_wrong_commit_and_author_only(self):
        for label, reviews in (
            ("none", []),
            ("wrong commit", [dict(self.approval, commit_id=MAIN_SHA)]),
            ("author approval", [dict(self.approval, user={"id": 100, "login": "other-login"})]),
        ):
            with self.subTest(case=label):
                self.pr_candidate()
                self.pages[self.reviews_path][1] = reviews
                self.denied()

    def test_latest_review_controls_each_reviewer_approval(self):
        for state, commit in (
            ("CHANGES_REQUESTED", PR_SHA),
            ("DISMISSED", PR_SHA),
            ("COMMENTED", PR_SHA),
            ("APPROVED", OTHER_SHA),
            ("PENDING", PR_SHA),
        ):
            with self.subTest(state=state, commit=commit):
                self.pr_candidate()
                later = dict(self.approval, id=2, state=state, commit_id=commit)
                if state == "PENDING":
                    later.pop("submitted_at")
                else:
                    later["submitted_at"] = "2026-09-25T13:00:00Z"
                self.pages[self.reviews_path][1] = [copy.deepcopy(self.approval), later]
                self.denied()

    def test_submission_time_not_record_order_controls_approval(self):
        self.pr_candidate()
        # A review can be created early and submitted after another review.
        later = dict(self.approval, id=2, state="CHANGES_REQUESTED",
                     submitted_at="2026-09-25T13:00:00Z")
        self.pages[self.reviews_path][1] = [later, copy.deepcopy(self.approval)]
        self.denied()
        self.pr_candidate()
        earlier = dict(self.approval, id=3, state="CHANGES_REQUESTED",
                       submitted_at="2026-09-25T11:00:00Z")
        self.pages[self.reviews_path][1] = [copy.deepcopy(self.approval), earlier]
        self.authorized(reviewed=PR_SHA)

    def test_new_exact_sha_approval_on_second_page_authorizes(self):
        self.pr_candidate()
        earlier = [
            dict(self.approval, id=i + 1, state="CHANGES_REQUESTED", commit_id=OTHER_SHA,
                 submitted_at=f"2026-09-25T12:{i // 60:02d}:{i % 60:02d}Z")
            for i in range(100)
        ]
        renewed = dict(self.approval, id=101, submitted_at="2026-09-25T13:00:00Z")
        self.pages[self.reviews_path] = {1: earlier, 2: [renewed]}
        self.authorized(reviewed=PR_SHA)

    def test_equal_latest_timestamp_is_ambiguous_but_other_reviewer_can_qualify(self):
        self.pr_candidate()
        conflicting = dict(self.approval, id=2, state="CHANGES_REQUESTED")
        self.pages[self.reviews_path][1].append(conflicting)
        self.denied()
        self.pr_candidate()
        self.pages[self.reviews_path][1].extend((
            conflicting, dict(self.approval, id=3, user={"id": 300, "login": "second-reviewer"})
        ))
        self.authorized(reviewed=PR_SHA)

    def test_pending_reviewer_does_not_cancel_another_independent_approval(self):
        self.pr_candidate()
        pending = dict(self.approval, id=2, state="PENDING", user={"id": 300, "login": "pending"})
        pending.pop("submitted_at")
        self.pages[self.reviews_path][1].append(pending)
        self.authorized(reviewed=PR_SHA)

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

    def test_invalid_review_identity_state_and_timestamp_fail_closed(self):
        mutations = {
            "identity missing": lambda r: r.update(user={}),
            "identity bool": lambda r: r.update(user={"id": True}),
            "state missing": lambda r: r.pop("state"),
            "state unknown": lambda r: r.update(state="UNKNOWN"),
            "timestamp missing": lambda r: r.pop("submitted_at"),
            "timestamp invalid": lambda r: r.update(submitted_at="not-a-date"),
            "commit missing": lambda r: r.pop("commit_id"),
        }
        for label, mutate in mutations.items():
            with self.subTest(case=label):
                self.pr_candidate()
                bad = dict(self.approval, id=2)
                mutate(bad)
                independent = dict(self.approval, id=3, user={"id": 300, "login": "another"})
                self.pages[self.reviews_path][1] = [bad, independent]
                self.denied()

    def test_wrong_dispatch_context_fails_even_for_exact_main_revision(self):
        for label, changes in (
            ("ref", {"GITHUB_REF": "refs/heads/feature"}),
            ("tag", {"GITHUB_REF": "refs/tags/v1"}),
            ("repository", {"GITHUB_REPOSITORY": "attacker/godot-agent-kit"}),
            ("event", {"GITHUB_EVENT_NAME": "pull_request_target"}),
            ("workflow ref", {"GITHUB_WORKFLOW_REF": WORKFLOW_REF.replace("@refs/heads/main", "@refs/heads/feature")}),
            ("workflow repository", {"GITHUB_WORKFLOW_REF": WORKFLOW_REF.replace(REPOSITORY, "attacker/repo")}),
        ):
            with self.subTest(case=label):
                self.denied(reviewed=MAIN_SHA, overrides=changes)

    def test_environment_and_branch_policy_fail_closed(self):
        mutations = {
            "no required reviewers": lambda: self.responses[ENVIRONMENT].update(protection_rules=[]),
            "empty reviewers": lambda: self.responses[ENVIRONMENT]["protection_rules"][0].update(reviewers=[]),
            "self review permitted": lambda: self.responses[ENVIRONMENT]["protection_rules"][0].update(prevent_self_review=False),
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

    def test_second_page_cannot_hide_a_newer_revocation(self):
        self.pr_candidate()
        earlier = [dict(self.approval, id=i + 1, submitted_at=f"2026-09-25T12:{i // 60:02d}:{i % 60:02d}Z") for i in range(100)]
        later = dict(self.approval, id=101, state="CHANGES_REQUESTED", submitted_at="2026-09-25T13:00:00Z")
        self.pages[self.reviews_path] = {1: earlier, 2: [later]}
        self.denied()

    def test_pr_head_change_during_review_lookup_invalidates_authorization(self):
        self.pr_candidate()
        changed = copy.deepcopy(self.pr)
        changed["head"]["sha"] = OTHER_SHA
        self.sequence[self.pr_path][1] = changed
        self.denied()

    def test_http_and_json_failures_do_not_release_a_revision(self):
        failures = (
            ("environment HTTP", ENVIRONMENT, "response", "http"),
            ("environment JSON", ENVIRONMENT, "response", "json"),
            ("branch HTTP", BRANCHES, "response", "http"),
            ("association HTTP", self.associations, "page", "http"),
            ("reviews JSON", self.reviews_path, "page", "json"),
            ("PR recheck HTTP", self.pr_path, "recheck", "http"),
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
                    self.sequence[path][1] = broken
                self.denied()


if __name__ == "__main__":
    unittest.main()
