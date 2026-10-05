"""Prevent false real-client acceptance from missing recovery or replayed intent."""
import copy
from pathlib import Path
import unittest
from unittest import mock

import mcp_failure_acceptance as failure
from mcp_lifecycle_acceptance import review_client_events


class FailureEvidenceTests(unittest.TestCase):
    def make_harness(self, name='failure_partial'):
        harness = failure.McpFailureMixin()
        initial = dict(name='read_script', structuredContent=dict(result=dict(source='old', revision='sr1:old')))
        edit = dict(name='edit_script', arguments=dict(revision='sr1:old', replacement_source=failure.DESIRED),
                    structuredContent=dict(result=dict(outcome=dict(outcome='applied_unverified'))), delivery='delivered')
        recovery = dict(name='read_script', structuredContent=dict(result=dict(source='survivor', revision=None)))
        target = dict(name=name, project=Path('/owned'), editor=object(), calls=[initial, edit, recovery],
                      desired=failure.DESIRED, delivery_dropped=False)
        harness.targets = {'/owned': target}
        harness.summary = {}
        harness.state = mock.Mock(return_value=(dict(open_paths=[]), {failure.TARGET: dict(text='survivor')}))
        return harness, target

    def test_partial_outcome_without_later_actual_read_cannot_complete(self):
        harness, target = self.make_harness()
        target['calls'].pop()
        with self.assertRaises(failure.observation.Failure):
            harness.finalize_targets()
        self.assertNotIn('mcp_profiles_verified', harness.summary)

    def test_second_edit_after_uncertainty_cannot_count_as_recovery(self):
        harness, target = self.make_harness()
        target['calls'].insert(2, copy.deepcopy(target['calls'][1]))
        with self.assertRaises(failure.observation.Failure):
            harness.finalize_targets()
        self.assertNotIn('mcp_profiles_verified', harness.summary)

    def test_result_consumption_requires_exact_returned_revision_and_intent(self):
        for field in ('revision', 'replacement_source'):
            harness, target = self.make_harness()
            target['calls'][1]['arguments'][field] = 'substituted'
            with self.subTest(field=field), self.assertRaises(failure.observation.Failure):
                harness.finalize_targets()

    def test_reconnect_requires_loss_and_refused_identical_resend(self):
        harness, target = self.make_harness('failure_reconnect')
        with self.assertRaises(failure.observation.Failure):
            harness.finalize_targets()
        target['delivery_dropped'] = True
        duplicate = copy.deepcopy(target['calls'][1])
        target['calls'].insert(2, duplicate)
        with self.assertRaises(failure.observation.Failure):
            harness.finalize_targets()
        duplicate['structuredContent']['result']['outcome']['outcome'] = 'refused'
        harness.finalize_targets()
        self.assertEqual(harness.summary['reconnect_edit_resends'], 1)
        self.assertTrue(harness.summary['reconnect_requires_client_transcript_attribution'])
        self.assertEqual(harness.summary['mcp_profiles_verified'], ['failure_reconnect'])

    def test_recovery_source_must_match_independent_survivor(self):
        harness, target = self.make_harness()
        target['calls'][-1]['structuredContent']['result']['source'] = failure.DESIRED
        with self.assertRaises(failure.observation.Failure):
            harness.finalize_targets()
        self.assertNotIn('mcp_profiles_verified', harness.summary)

    def test_lost_result_is_distinct_from_model_visible_recovery(self):
        read = dict(name='read_script', profile='failure_reconnect', arguments={'project_root': '/owned'},
                    structuredContent={'request_id': 'initial', 'result': {'revision': 'opaque-initial'}})
        lost = dict(name='edit_script', profile='failure_reconnect', delivery='unavailable',
                    arguments={'project_root': '/owned', 'revision': 'opaque-initial'},
                    structuredContent={'request_id': 'lost', 'result': {'outcome': 'verified_changed'}})
        recovered = dict(name='read_script', profile='failure_reconnect', arguments={'project_root': '/owned'},
                         structuredContent={'request_id': 'recovery', 'result': {'revision': 'opaque-after'}})
        events = [dict(kind='model_visible_tool_result', content=read['structuredContent']),
                  dict(kind='tool_call', name='edit_script', arguments=lost['arguments']),
                  dict(kind='model_visible_tool_result', content=recovered['structuredContent'])]
        result = review_client_events(events, [read, lost, recovered])
        self.assertEqual(result['model_visible_results'], 2)
        self.assertEqual(result['delivery_unavailable_results'], 1)
        events.append(dict(kind='model_visible_tool_result', content=lost['structuredContent']))
        with self.assertRaisesRegex(ValueError, 'undelivered'):
            review_client_events(events, [read, lost, recovered])


if __name__ == '__main__':
    unittest.main()
