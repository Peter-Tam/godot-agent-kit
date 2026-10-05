"""Fixed composed protocol's fail-closed ordering/evidence regressions (no editor)."""
import copy
from pathlib import Path
import unittest
from unittest import mock

import mcp_composed_acceptance as composed
from run_script_close import SAFE, TARGET


class WitnessBase:
    def observe_call(self, request, response, before):
        self.observed.append(request['id'])


class Harness(composed.McpComposedMixin, WitnessBase):
    def __init__(self, profile='open'):
        self._composed_active = True
        self.observed = []
        self.summary = {}
        self.target = dict(name=profile, project=Path('/owned') / profile,
                           descriptor={'session_id': profile}, editor={}, calls=[],
                           steps=composed.composed_steps(profile), cursor=0, pending=None,
                           fresh={'source': SAFE, 'revision': 'fresh'}, first_revision='old',
                           first_replacement=composed.replacement(SAFE, profile, 1), source=SAFE,
                           successes=0, dirty_refusals=0, stale_refusals=0)
        self.targets = {str(self.target['project']): self.target}
        self.close_action = mock.Mock()
        self.native_action = mock.Mock()
        self.record_witness = mock.Mock()
        self._composed_coherent = mock.Mock()
        self._composed_save = mock.Mock()
        self.state = mock.Mock(return_value=({}, {}))

    def request(self, role, index=None, **overrides):
        args = dict(project_root=str(self.target['project']), session_id=self.target['name'], script_path=TARGET)
        if role in ('change', 'dirty', 'stale'):
            args.update(revision='old' if role == 'stale' else 'fresh',
                        replacement_source=self.target['first_replacement'] if role == 'stale' else
                        composed.replacement(SAFE, self.target['name'], index if role == 'change' else index + 1))
        args.update(overrides)
        return dict(id='owned', params=dict(name=composed.tool_for(role), arguments=args))

    def at(self, role, index=None):
        self.target['cursor'] = self.target['steps'].index((role, index))


class ComposedProtocolTests(unittest.TestCase):
    def test_fixed_minima_and_interleaving_not_counts_only(self):
        plans = {name: composed.composed_steps(name) for name in composed.COUNTS}
        self.assertEqual(sum(sum(role == 'change' for role, _ in plan) for plan in plans.values()), 20)
        self.assertEqual(sum(sum(role == 'dirty' for role, _ in plan) for plan in plans.values()), 3)
        self.assertEqual(sum(sum(role == 'stale' for role, _ in plan) for plan in plans.values()), 3)
        for profile, plan in plans.items():
            for position, (role, _) in enumerate(plan):
                if role in ('change', 'dirty', 'stale'):
                    self.assertTrue(plan[position - 1][0].endswith('read'))
            self.assertEqual(plan[-1], ('durable_read', None))
        history = [role for role, _ in plans['open'] if role.startswith(('undo_', 'redo_'))]
        self.assertEqual(history, ['undo_read', 'undo_saved_read', 'redo_read', 'redo_saved_read'])
        self.assertFalse(any(role.startswith(('undo_', 'redo_')) for role, _ in plans['absent']))

    def test_replacement_changes_only_value_and_preserves_human_comments(self):
        source = '# human\n' + SAFE + '# another human\n'
        self.assertEqual(composed.replacement(source, 'open', 3), source.replace('return 47', 'return 103'))
        for invalid in ('# no value\n', SAFE + SAFE):
            with self.assertRaises(composed.observation.Failure):
                composed.replacement(invalid, 'open', 1)

    def test_skipped_read_is_rejected_before_human_or_lifecycle_action(self):
        h = Harness()
        h.at('change', 2)
        h.target['fresh'] = None
        with self.assertRaises(composed.observation.Failure):
            h.before_composed_call(h.target, h.request('change', 2))
        h.close_action.assert_not_called()
        h.native_action.assert_not_called()
        self.assertIsNone(h.target['pending'])

    def test_wrong_sequence_wrong_session_or_replacement_cannot_mutate_fixture(self):
        for role, index, overrides in [('change', 1, {}), ('dirty', 2, {'session_id': 'other'}),
                                       ('dirty', 2, {'replacement_source': SAFE})]:
            h = Harness()
            h.at('dirty', 2)
            with self.assertRaises(composed.observation.Failure):
                h.before_composed_call(h.target, h.request(role, index, **overrides))
            h.close_action.assert_not_called()
            h.native_action.assert_not_called()

    def test_next_target_and_overlapping_call_are_rejected(self):
        h = Harness()
        other = copy.deepcopy(h.target)
        with self.assertRaises(composed.observation.Failure):
            h.before_composed_call(other, h.request('discover'))
        h.target['pending'] = ('inflight', 'discover', None)
        with self.assertRaises(composed.observation.Failure):
            h.before_composed_call(h.target, h.request('discover'))
        h.close_action.assert_not_called()

    def test_stale_challenge_requires_exact_old_basis_not_latest_basis(self):
        h = Harness('cached')
        h.at('stale')
        with self.assertRaises(composed.observation.Failure):
            h.before_composed_call(h.target, h.request('stale', revision='fresh'))
        h.before_composed_call(h.target, h.request('stale'))
        self.assertEqual(h.target['pending'], ('owned', 'stale', None))
        self.assertEqual(h.target['cursor'], h.target['steps'].index(('stale', None)))

    def test_unsaved_history_read_cannot_claim_revision_or_persisted_disk(self):
        for token, disk in [('unsafe-token', dict(current=True, equals_source=False, text=SAFE)),
                            (None, dict(current=True, equals_source=True, text=None))]:
            h = Harness()
            h.at('undo_read')
            h.target['pending'] = ('owned', 'undo_read', None)
            unsaved = SAFE.replace('return 47', 'return 46')
            state = dict(target={'B': unsaved, 'dirty': True})
            disks = {TARGET: {'text': SAFE}}
            h.state.return_value = state, disks
            result = dict(source=unsaved, revision=token, state=dict(source_origin='editor_buffer',
                          dirty={'buffer': {'state': 'dirty'}}, sources={'disk': disk}))
            with self.assertRaises(composed.observation.Failure):
                h.observe_call(h.request('undo_read'), {'result': {'structuredContent': {'error': None, 'result': result}}},
                               (state, disks))
            self.assertEqual(h.target['pending'], ('owned', 'undo_read', None))
            self.assertEqual(h.target['cursor'], h.target['steps'].index(('undo_read', None)))

    def test_truthful_unsaved_history_read_is_observed_without_edit_basis(self):
        h = Harness()
        h.at('undo_read')
        h.target['pending'] = ('owned', 'undo_read', None)
        state = dict(target={'B': SAFE, 'dirty': True})
        disks = {TARGET: {'text': h.target['first_replacement']}}
        h.state.return_value = state, disks
        result = dict(source=SAFE, revision=None, state=dict(source_origin='editor_buffer',
                      dirty={'buffer': {'state': 'dirty'}}, sources={'disk': dict(
                          current=True, equals_source=False, text=disks[TARGET]['text'])}))
        h.observe_call(h.request('undo_read'), {'result': {'structuredContent': {'error': None, 'result': result}}},
                       (state, disks))
        self.assertIsNone(h.target['pending'])
        self.assertEqual(h.observed, ['owned'])
        self.assertIsNone(h.target['fresh'])

    def test_wrong_response_owner_and_host_failure_cannot_advance(self):
        for identity, content in [('other', {'error': None, 'result': {}}),
                                  ('owned', {'error': {'code': 'busy'}, 'result': None})]:
            h = Harness()
            h.target['pending'] = ('owned', 'discover', None)
            request = h.request('discover')
            request['id'] = identity
            with self.assertRaises(composed.observation.Failure):
                h.observe_call(request, {'result': {'structuredContent': content}}, ({}, {}))
            self.assertEqual(h.target['cursor'], 0)
            self.assertIsNotNone(h.target['pending'])

    def test_claimed_counts_do_not_replace_missing_lifecycle_read(self):
        h = Harness()
        h.target.update(cursor=len(h.target['steps']) - 1, successes=8, dirty_refusals=3, stale_refusals=1)
        with self.assertRaises(composed.observation.Failure):
            h.finalize_targets()
        h._composed_coherent.assert_not_called()
        self.assertNotIn('composed', h.summary)

    def test_lifecycle_cannot_begin_before_verified_twenty_edit_protocol(self):
        h = Harness('absent')
        h.at('durable_read')
        h.target['successes'] = 5
        h.state.return_value = ({'open_paths': [], 'target': {}}, {})
        with self.assertRaises(composed.observation.Failure):
            h.before_composed_call(h.target, h.request('durable_read'))
        h.close_action.assert_not_called()
        h.native_action.assert_not_called()


if __name__ == '__main__':
    unittest.main()
