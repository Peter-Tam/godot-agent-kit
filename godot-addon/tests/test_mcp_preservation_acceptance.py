"""Schema 2 fixture consumers must retain exact intent and truthful refusals."""
import copy
from pathlib import Path
from tempfile import TemporaryDirectory
import time
from types import SimpleNamespace
import unittest
from unittest import mock

import run_observation as observation
from mcp_peer import McpPeer
from mcp_preservation_acceptance import McpPreservationAcceptanceMixin
from mcp_validation_acceptance import McpValidationMixin, SOURCE_LIMIT
from run_script_edit import sha


class ExactConsumerTests(unittest.TestCase):
    def harness(self):
        harness = McpPreservationAcceptanceMixin()
        harness.cases = [{}]
        return harness

    def result(self, mode='open', **overrides):
        outcome = dict(outcome='refused', application='not_applied', reason='no_match', stage='matching')
        outcome.update(safe_next_action='fresh_read') if mode == 'open' else outcome.update(next_action={'kind': 'fresh_read'})
        outcome.update(overrides)
        return dict(schema_version=2, operation='edit_script', request_id='domain', error=None,
                    result=dict(mode=mode, outcome=outcome))

    def test_matching_refusal_cannot_escape_as_unchanged_success_or_wrong_action(self):
        for mode in ('open', 'closed'):
            invalid = [dict(outcome='verified_unchanged'), dict(stage='verifying'),
                       dict(application='unknown'),
                       dict(safe_next_action='none') if mode == 'open' else dict(next_action={'kind': 'none'})]
            for fields in invalid:
                harness = self.harness()
                with self.subTest(mode=mode, fields=fields), self.assertRaises(observation.Failure):
                    harness.mcp_review_edit(self.result(mode, **fields), 'matching', ('refused', 'verified_unchanged'))
            result = self.harness().mcp_review_edit(self.result(mode), 'matching', 'refused')
            self.assertEqual(result['outcome']['stage'], 'matching')

    def test_actual_carrier_rejects_schema_one_and_false_success_flag(self):
        with TemporaryDirectory() as directory:
            harness = SimpleNamespace(artifacts=Path(directory), secrets=set(), source_markers=set(),
                                      case=mock.Mock())
            for version, is_error, accepted in ((1, True, False), (2, False, False), (2, True, True)):
                peer = McpPeer(harness, 'carrier')
                call = dict(id='wire', started=time.monotonic(), operation='edit_script', domain_request_id=None)
                peer.pending['wire'] = call
                root = self.result()
                root['schema_version'] = version
                peer.receive = mock.Mock(return_value=dict(id='wire', result=dict(structuredContent=root, isError=is_error)))
                with self.subTest(version=version, is_error=is_error):
                    if accepted:
                        self.assertEqual(peer.finish(call, 'carrier'), root)
                    else:
                        with self.assertRaises(observation.Failure):
                            peer.finish(call, 'carrier')


class BoundDeadlineEvidenceTests(unittest.TestCase):
    desired = "#" + "x" * (SOURCE_LIMIT - 1)

    def result(self, mode):
        outcome = dict(outcome="applied_unverified", application="applied",
                       reason="timeout", stage="validating" if mode == "open" else "verifying")
        if mode == "open":
            outcome["progress"] = {step: dict(state="completed") for step in
                                   ("buffer_application", "resource_sync", "persistence", "finalization")}
            outcome["validation"] = [
                dict(purpose="preflight", status="valid", cleanup_confirmed=True,
                     input=dict(sha256=sha(self.desired), utf8_bytes=SOURCE_LIMIT)),
                dict(purpose="post_change", status="unavailable", cleanup_confirmed=True)]
        else:
            outcome["evidence"] = dict(validation=dict(original=True, desired=True, actual=False))
            outcome["effects"] = dict(written_bytes=SOURCE_LIMIT, authorized=True,
                                      disk_entered=True, flushed=True, readback=True, mtime_restored=True)
        return dict(mode=mode, outcome=outcome)

    def test_deadline_does_not_accept_other_failures_or_uncertain_application(self):
        for mode in ("open", "closed"):
            for key, value in (("outcome", "refused"), ("application", "not_applied"),
                               ("application", "partly_applied"), ("application", "unknown"),
                               ("reason", "parse_error"), ("reason", "write_failed"), ("stage", "preflight")):
                result = self.result(mode)
                result["outcome"][key] = value
                with self.subTest(mode=mode, key=key, value=value), self.assertRaises(observation.Failure):
                    McpValidationMixin._review_bound_deadline(result, self.desired, "bounds")

    def test_incomplete_or_wrong_source_evidence_cannot_qualify_the_bound(self):
        for mode in ("open", "closed"):
            base = self.result(mode)
            changes = [
                (("validation", 0, "status"), "unavailable"),
                (("validation", 0, "input", "sha256"), sha("different source")),
                (("validation", 0, "input", "utf8_bytes"), SOURCE_LIMIT - 1),
                (("validation", 1, "cleanup_confirmed"), False),
            ] if mode == "open" else [
                (("evidence", "validation", "original"), False),
                (("evidence", "validation", "desired"), None),
                (("effects", "written_bytes"), SOURCE_LIMIT - 1),
            ]
            changes += ([(("progress", step, "state"), "unknown") for step in base["outcome"]["progress"]]
                        if mode == "open" else
                        [(("effects", step), False) for step in base["outcome"]["effects"] if step != "written_bytes"])
            for path, value in changes:
                result = copy.deepcopy(base)
                current = result["outcome"]
                for key in path[:-1]:
                    current = current[key]
                current[path[-1]] = value
                with self.subTest(mode=mode, path=path), self.assertRaises(observation.Failure):
                    McpValidationMixin._review_bound_deadline(result, self.desired, "bounds")
            with self.subTest(mode=mode, wrong_bound=True), self.assertRaises(observation.Failure):
                McpValidationMixin._review_bound_deadline(base, self.desired[:-1], "bounds")


if __name__ == '__main__':
    unittest.main()
