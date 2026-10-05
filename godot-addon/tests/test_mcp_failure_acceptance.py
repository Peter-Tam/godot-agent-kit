"""Prevent false real-client acceptance from missing recovery or replayed intent."""
import copy
from pathlib import Path
import unittest
from unittest import mock

import mcp_failure_acceptance as failure
from mcp_lifecycle_acceptance import review_client_events
from mcp_peer import McpAdversarialMixin


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


class InterruptedBufferWitnessTests(unittest.TestCase):
    def test_editor_resource_propagation_cannot_hide_other_changes(self):
        target = dict(path=failure.TARGET, script_id="script", editor_id="editor", buffer_id="buffer",
                      R="original", B="already visible", version=3, saved_version=2, dirty=True,
                      resource_edited=True, has_undo=True, has_redo=False)
        other = dict(target, path="res://scripts/other.gd", R="human work", B="human work")
        before = dict(target=target, documents=[target, other], cached_id="script", cached_R="original",
                      cached_edited=True, cache_has=True, cache_type="GDScript", selection=failure.TARGET,
                      open_paths=[target["path"], other["path"]], loader_calls=0)
        disks = {failure.TARGET: dict(sha256="original digest", mtime_ns=1, ctime_ns=2, mode=0o644)}
        after = copy.deepcopy(before)
        after["cached_R"] = after["target"]["R"] = before["target"]["B"]
        harness = McpAdversarialMixin()
        harness.cases = [{}]
        harness.assert_mcp_buffer_prefix_survivor(before, disks, after, disks, "editor propagation")
        self.assertTrue(harness.cases[-1]["observed_editor_resource_propagation"])
        changes = {
            "buffer overwrite": lambda state, disk: state["target"].update(B="overwritten"),
            "dirty cleared": lambda state, disk: state.update(cached_edited=False),
            "history lost": lambda state, disk: state["target"].update(has_undo=False),
            "document replaced": lambda state, disk: state["target"].update(buffer_id="replacement"),
            "cache replaced": lambda state, disk: state.update(cached_id="replacement"),
            "unrelated resource": lambda state, disk: state["documents"][1].update(R="overwritten"),
            "disk write": lambda state, disk: disk[failure.TARGET].update(sha256="different digest"),
            "metadata changed": lambda state, disk: disk[failure.TARGET].update(mtime_ns=3),
        }
        for name, change in changes.items():
            damaged, changed_disks = copy.deepcopy(after), copy.deepcopy(disks)
            change(damaged, changed_disks)
            with self.subTest(change=name), self.assertRaises(failure.observation.Failure):
                harness.assert_mcp_buffer_prefix_survivor(before, disks, damaged, changed_disks, name)
        third_source = copy.deepcopy(after)
        third_source["cached_R"] = third_source["target"]["R"] = "not the visible buffer"
        with self.assertRaises(failure.observation.Failure):
            harness.assert_mcp_buffer_prefix_survivor(before, disks, third_source, disks, "unattributed R")


if __name__ == '__main__':
    unittest.main()
