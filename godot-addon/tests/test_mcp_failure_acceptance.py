"""Prevent false real-client acceptance from missing recovery or replayed intent."""
import copy
from contextlib import ExitStack, contextmanager
from pathlib import Path
from tempfile import TemporaryDirectory
from types import SimpleNamespace
import unittest
from unittest import mock

import mcp_failure_acceptance as failure
from mcp_lifecycle_acceptance import review_client_events
from mcp_peer import McpAdversarialMixin


class FailureEvidenceTests(unittest.TestCase):
    def make_harness(self, name='failure_partial'):
        harness = failure.McpFailureMixin()
        initial = dict(name='read_script', structuredContent=dict(result=dict(source='old', revision='sr1:old')))
        edit = dict(name='edit_script', arguments=dict(revision='sr1:old', old_string='old', new_string=failure.DESIRED),
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
        for field in ('revision', 'old_string', 'new_string'):
            harness, target = self.make_harness()
            target['calls'][1]['arguments'][field] = 'substituted'
            with self.subTest(field=field), self.assertRaises(failure.observation.Failure):
                harness.finalize_targets()

    def test_missing_exact_field_or_legacy_extra_cannot_complete(self):
        for field in ('revision', 'old_string', 'new_string'):
            harness, target = self.make_harness()
            del target['calls'][1]['arguments'][field]
            with self.subTest(field=field), self.assertRaises(failure.observation.Failure):
                harness.finalize_targets()
        harness, target = self.make_harness()
        target['calls'][1]['arguments']['replacement_source'] = failure.DESIRED
        with self.assertRaises(failure.observation.Failure):
            harness.finalize_targets()

    def test_unchecked_edit_cannot_be_ignored_as_baseline_failure(self):
        harness, target = self.make_harness()
        bad = copy.deepcopy(target['calls'][1])
        bad['structuredContent']['result'] = None
        target['calls'].insert(2, bad)
        with self.assertRaises(failure.observation.Failure):
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
                    structuredContent={'request_id': 'initial', 'result': {'source': 'actual source', 'revision': 'sr1:' + '1' * 64}})
        lost = dict(name='edit_script', profile='failure_reconnect', delivery='unavailable',
                    arguments={'project_root': '/owned', 'revision': 'sr1:' + '1' * 64,
                               'old_string': 'actual source', 'new_string': failure.DESIRED},
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

    def test_claimed_but_unrecorded_exact_call_is_rejected(self):
        read = dict(name='read_script', profile='failure_reconnect', arguments={'project_root': '/owned'},
                    structuredContent={'request_id': 'read', 'result': {'source': 'old', 'revision': 'basis'}})
        actual = dict(name='edit_script', profile='failure_reconnect',
                      arguments={'project_root': '/owned', 'revision': 'basis', 'old_string': 'old', 'new_string': 'new'},
                      structuredContent={'request_id': 'edit', 'result': {'outcome': 'refused'}})
        claimed = dict(actual['arguments'], old_string='fabricated')
        events = [dict(kind='model_visible_tool_result', content=call['structuredContent']) for call in (read, actual)]
        events.append(dict(kind='tool_call', name='edit_script', arguments=claimed))
        with self.assertRaisesRegex(ValueError, 'missing server record'):
            review_client_events(events, [read, actual])


class MatchingRecoveryEvidenceTests(unittest.TestCase):
    def calls(self):
        source, revision = failure.SAFE, 'sr1:initial'
        calls = [dict(name='read_script', structuredContent=dict(
            result=dict(source=source, revision=revision)))]
        for index, (old, new, terminal, intended) in enumerate(failure.MATCHING_RECOVERY):
            outcome = (dict(outcome='verified_changed') if terminal == 'verified_changed' else
                       dict(outcome='refused', reason=terminal, stage='matching',
                            application='not_applied', next_action=dict(kind='fresh_read')))
            calls.append(dict(name='edit_script', arguments=dict(
                revision=revision, old_string=old, new_string=new),
                structuredContent=dict(result=dict(outcome=outcome)), delivery='delivered'))
            source = intended
            if terminal == 'verified_changed':
                revision = 'sr1:changed-' + str(index)
            calls.append(dict(name='read_script', structuredContent=dict(
                result=dict(source=source, revision=revision)), delivery='delivered'))
        return calls

    def test_refusals_require_fresh_reads_before_distinct_unique_corrections(self):
        calls = self.calls()
        remaining = failure.McpFailureMixin._review_matching_recovery(calls)
        self.assertEqual(remaining, [calls[-1]])
        for read_index in (2, 4, 6, 8):
            missing = copy.deepcopy(calls)
            del missing[read_index]
            with self.subTest(read=read_index), self.assertRaises(failure.observation.Failure):
                failure.McpFailureMixin._review_matching_recovery(missing)

    def test_additional_read_uses_latest_revision_without_replaying_intent(self):
        calls = self.calls()
        extra = copy.deepcopy(calls[2])
        extra['structuredContent']['result']['revision'] = 'sr1:additional-read'
        calls.insert(3, extra)
        calls[4]['arguments']['revision'] = 'sr1:additional-read'
        self.assertEqual(failure.McpFailureMixin._review_matching_recovery(calls), [calls[-1]])
        calls[4]['arguments']['revision'] = 'sr1:initial'
        with self.assertRaises(failure.observation.Failure):
            failure.McpFailureMixin._review_matching_recovery(calls)

    def test_stale_or_substituted_correction_cannot_complete(self):
        for edit_index, field, value in (
                (7, 'revision', 'sr1:initial'),
                (3, 'revision', 'sr1:invented'),
                (3, 'old_string', failure.SAFE),
                (7, 'old_string', 'r'),
                (7, 'new_string', 'return 99')):
            calls = self.calls()
            calls[edit_index]['arguments'][field] = value
            with self.subTest(edit=edit_index, field=field), self.assertRaises(failure.observation.Failure):
                failure.McpFailureMixin._review_matching_recovery(calls)

    def test_false_refusal_success_or_unconsumed_recovery_is_rejected(self):
        for index, field, value in (
                (1, 'outcome', 'verified_unchanged'),
                (5, 'reason', 'no_match'),
                (5, 'application', 'unknown')):
            calls = self.calls()
            calls[index]['structuredContent']['result']['outcome'][field] = value
            with self.subTest(index=index, field=field), self.assertRaises(failure.observation.Failure):
                failure.McpFailureMixin._review_matching_recovery(calls)
        for index in (1, 2, 5, 6):
            calls = self.calls()
            calls[index]['delivery'] = 'unavailable'
            with self.subTest(undelivered=index), self.assertRaises(failure.observation.Failure):
                failure.McpFailureMixin._review_matching_recovery(calls)

    def test_wrong_complete_source_or_missing_revision_is_not_recovery(self):
        for field, value in (('source', failure.SAFE), ('revision', None)):
            calls = self.calls()
            calls[4]['structuredContent']['result'][field] = value
            with self.subTest(field=field), self.assertRaises(failure.observation.Failure):
                failure.McpFailureMixin._review_matching_recovery(calls)


class SerialFailureHarness(failure.McpFailureMixin):
    def __init__(self, work):
        self.work = Path(work)
        self.args = SimpleNamespace(profile='failures')
        self.editors, self.events, self.cases = [], [], []
        self.summary = {}

    @contextmanager
    def closed_fixture(self, name, **options):
        project = self.work / ('close-' + name)
        (project / 'scripts').mkdir(parents=True)
        (project / 'scripts/subject.gd').write_text(failure.SAFE)
        editor = dict(process=SimpleNamespace(pid=len(self.events) + 1))
        self.editors.append(editor)
        self.events.append(('start', name, len(self.editors)))
        try:
            yield project, editor, dict(session_id=name)
        finally:
            self.editors.remove(editor)
            self.events.append(('stop', name, len(self.editors)))

    def state(self, editor, project):
        return dict(open_paths=[]), {failure.TARGET: dict(text=(project / 'scripts/subject.gd').read_text())}

    def case(self, name, **facts):
        self.cases.append(dict(case=name, **facts))

    def assert_no_effect(self, name, project, editor, before, disks):
        failure.observation.require(self.state(editor, project) == (before, disks), 'fixture_survivor')


class SerialFailureFixtureTests(unittest.TestCase):
    def test_unvisited_profiles_do_not_create_editors_and_abandonment_closes_owner(self):
        with TemporaryDirectory() as work:
            harness = SerialFailureHarness(work)
            with ExitStack() as stack:
                harness.prepare_targets(stack)
                self.assertEqual(harness.events, [('start', 'mcp-failure_limited', 1)])
                self.assertEqual(len(harness.editors), 1)
            self.assertEqual(harness.events[-1], ('stop', 'mcp-failure_limited', 0))
            self.assertEqual(harness.editors, [])

    def test_missing_completed_evidence_cannot_close_previous_or_start_next(self):
        with TemporaryDirectory() as work, ExitStack() as stack:
            harness = SerialFailureHarness(work)
            harness.prepare_targets(stack)
            targets = list(harness.targets.values())
            with self.assertRaises(failure.observation.Failure):
                harness.before_failure_call(targets[1])
            self.assertEqual(harness.events, [('start', 'mcp-failure_limited', 1)])
            self.assertIs(harness._failure_current, targets[0])

    def test_verified_transition_closes_previous_before_creating_next(self):
        with TemporaryDirectory() as work, ExitStack() as stack:
            harness = SerialFailureHarness(work)
            harness.prepare_targets(stack)
            targets = list(harness.targets.values())
            targets[0]['calls'].append(dict(name='read_script', structuredContent=dict(
                result=dict(source=failure.SAFE, revision=None))))
            harness.before_failure_call(targets[1])
            self.assertEqual(harness.events, [('start', 'mcp-failure_limited', 1),
                                             ('stop', 'mcp-failure_limited', 0),
                                             ('start', 'mcp-failure_refusal', 1)])
            with self.assertRaises(failure.observation.Failure):
                harness.before_failure_call(targets[0])
            self.assertEqual(len(harness.editors), 1)


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
