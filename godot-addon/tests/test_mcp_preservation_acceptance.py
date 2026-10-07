"""Schema 2 fixture consumers must retain exact intent and truthful refusals."""
from pathlib import Path
from tempfile import TemporaryDirectory
import time
from types import SimpleNamespace
import unittest
from unittest import mock

import run_observation as observation
from mcp_peer import McpPeer
from mcp_preservation_acceptance import McpPreservationAcceptanceMixin


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


if __name__ == '__main__':
    unittest.main()
