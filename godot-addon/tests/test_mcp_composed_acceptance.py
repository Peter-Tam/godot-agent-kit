"""Fixed composed protocol's fail-closed ordering/evidence regressions (no editor)."""
import copy
from contextlib import ExitStack
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest
from types import SimpleNamespace
from unittest import mock

import mcp_composed_acceptance as composed
from run_script_close import SAFE, TARGET
from close_native_acceptance import CURRENT
from mcp_lifecycle_acceptance import McpLifecycleMixin
from closed_script_acceptance import ClosedScriptAcceptanceMixin
from run_script_edit import sha


def witness_state(source=SAFE, *, opened=False, cached=False):
    doc = dict(path=TARGET, script_id=1, editor_id=2, buffer_id=3, R=source, B=source,
               version=2, saved_version=2, dirty=False, resource_edited=False,
               has_undo=True, has_redo=False, associated=opened)
    state = dict(documents=[doc] if opened else [], target=doc, cached_id=1 if cached else 0,
                 cached_R=source if cached else None, cached_edited=False if cached else None,
                 cache_has=cached, cache_type='GDScript' if cached else '', selection=TARGET if opened else CURRENT,
                 open_paths=[TARGET] if opened else [], loader_calls=0)
    disk = dict(text=source, sha256=sha(source), device=1, inode=2, mtime_ns=3,
                ctime_ns=4, mode=0o644)
    return state, {TARGET: disk}


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
                           fresh={'source': SAFE, 'revision': 'fresh'}, first_revision='old', first_source=SAFE,
                           first_replacement=composed.replacement(SAFE, profile, 1), source=SAFE,
                           successes=0, dirty_refusals=0, stale_refusals=0,
                           no_match_refusals=0, ambiguous_match_refusals=0)
        self.targets = {str(self.target['project']): self.target}
        self.close_action = mock.Mock()
        self.native_action = mock.Mock()
        self.record_witness = mock.Mock()
        self._composed_coherent = mock.Mock()
        self._composed_save = mock.Mock()
        self.state = mock.Mock(return_value=witness_state())

    def request(self, role, index=None, **overrides):
        args = dict(project_root=str(self.target['project']), session_id=self.target['name'], script_path=TARGET)
        if role in ('change', 'dirty', 'stale', 'no_match', 'ambiguous_match'):
            desired = self.target['first_replacement'] if role == 'stale' else composed.replacement(
                self.target['source'], self.target['name'], index if role == 'change' else index + 1)
            args.update(revision='old' if role == 'stale' else 'fresh',
                        **composed.intent_pair(role, self.target['first_source'] if role == 'stale' else
                                               self.target['source'], desired))
        args.update(overrides)
        return dict(id='owned', params=dict(name=composed.tool_for(role), arguments=args))

    def at(self, role, index=None):
        self.target['cursor'] = self.target['steps'].index((role, index))


class EvidenceHarness(composed.McpComposedMixin, McpLifecycleMixin, ClosedScriptAcceptanceMixin):
    """Exercise the same inherited evidence consumer used by WorkflowHarness."""
    def __init__(self, directory, profile='open'):
        Harness.__init__(self, profile)
        self.artifacts = Path(directory)
        self.cases = []
        self.case = lambda name, **fields: self.cases.append(dict(name=name, **fields))

    request = Harness.request
    at = Harness.at


class ComposedProtocolTests(unittest.TestCase):

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
                                       ('dirty', 2, {'new_string': SAFE}), ('dirty', 2, {'old_string': SAFE})]:
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

    def test_matching_refusal_invalidates_basis_and_requires_intentional_recovery(self):
        for profile in composed.COUNTS:
            for role, index in [('no_match', 2), ('ambiguous_match', 4)]:
                h = Harness(profile)
                h.at(role, index)
                request = h.request(role, index)
                h.before_composed_call(h.target, request)
                state, disks = h.state.return_value
                outcome = dict(outcome='refused', application='not_applied', reason=role, stage='matching')
                if profile == 'open':
                    outcome['safe_next_action'] = 'fresh_read'
                else:
                    outcome['next_action'] = {'kind': 'fresh_read'}
                h.observe_call(request, {'result': {'structuredContent': {
                    'error': None, 'result': {'outcome': outcome}}}}, (state, disks))
                self.assertIsNone(h.target['fresh'])
                self.assertEqual(h.target[role + '_refusals'], 1)
                self.assertEqual(h.target['steps'][h.target['cursor']], ('read', None))
                with self.assertRaises(composed.observation.Failure):
                    h.before_composed_call(h.target, h.request('change', index + 1,
                        old_string='\treturn 47', new_string='\treturn ' + str(
                            {'open': 100, 'cached': 200, 'absent': 300}[profile] + index + 1)))
                # The recovery read must carry both the unchanged complete source and revision.
                h.target['pending'] = ('owned', 'read', None)
                h.observe_call(h.request('read'), {'result': {'structuredContent': {
                    'error': None, 'result': {'source': SAFE, 'revision': 'fresh'}}}}, (state, disks))
                correction = h.request('change', index + 1)
                h.before_composed_call(h.target, correction)
                self.assertEqual(h.target['pending'], ('owned', 'change', index + 1))
                self.assertEqual(correction['params']['arguments']['old_string'], '\treturn 47')

    def test_matching_refusal_rejects_wrong_reason_stage_guidance_or_effect(self):
        valid = dict(outcome='refused', application='not_applied', reason='no_match',
                     stage='matching', next_action={'kind': 'fresh_read'})
        for changed in (dict(reason='ambiguous_match'), dict(reason='revision_mismatch'),
                        dict(stage='prepare'), dict(application='applied'),
                        dict(next_action={'kind': 'retry'}, safe_next_action='read again')):
            h = Harness('cached')
            h.at('no_match', 2)
            request = h.request('no_match', 2)
            h.before_composed_call(h.target, request)
            with self.assertRaises(composed.observation.Failure):
                h.observe_call(request, {'result': {'structuredContent': {
                    'error': None, 'result': {'outcome': dict(valid, **changed)}}}}, h.state.return_value)
            self.assertEqual(h.target['no_match_refusals'], 0)
            self.assertIsNotNone(h.target['pending'])

    def test_matching_recovery_rejects_wrong_read_source_or_revision(self):
        for result in ({'source': SAFE + '# lost basis\n', 'revision': 'fresh'},
                       {'source': SAFE, 'revision': 'old'}):
            h = Harness('absent')
            h.at('no_match', 2)
            h.target.update(cursor=h.target['cursor'] + 1, pending=('owned', 'read', None),
                            fresh=None, recovery_revision='fresh')
            with self.assertRaises(composed.observation.Failure):
                h.observe_call(h.request('read'), {'result': {'structuredContent': {
                    'error': None, 'result': result}}}, h.state.return_value)
            self.assertIsNone(h.target['fresh'])

    def test_correction_rejects_stale_revision_wrong_source_and_whole_source_substitute(self):
        for overrides, basis in [({'revision': 'old'}, SAFE), ({}, SAFE + '# wrong read\n'),
                                 ({'old_string': SAFE, 'new_string': composed.replacement(SAFE, 'cached', 3)}, SAFE)]:
            h = Harness('cached')
            h.at('change', 3)
            request = h.request('change', 3, **overrides)
            h.target['fresh']['source'] = basis
            with self.assertRaises(composed.observation.Failure):
                h.before_composed_call(h.target, request)
            h.close_action.assert_not_called()
            self.assertIsNone(h.target['pending'])

    def test_changed_source_read_cannot_reuse_previous_revision(self):
        h = Harness('cached')
        h.at('read')
        h.target.update(pending=('owned', 'read', None), changed_revision='fresh', fresh=None,
                        source=h.target['first_replacement'])
        with self.assertRaises(composed.observation.Failure):
            h.observe_call(h.request('read'), {'result': {'structuredContent': {
                'error': None, 'result': {'source': h.target['source'], 'revision': 'fresh'}}}},
                h.state.return_value)
        self.assertIsNone(h.target['fresh'])

    def test_every_refusal_discards_previous_edit_permission(self):
        for role, index in [('stale', None), ('dirty', 2)]:
            h = Harness()
            h.at(role, index)
            h.target['pending'] = ('owned', role, index)
            response = {'result': {'structuredContent': dict(error=None, result={'outcome': dict(
                outcome='refused', application='not_applied', reason='revision_mismatch',
                next_action={'kind': 'fresh_read'})})}}
            h.observe_call(h.request(role, index), response, h.state.return_value)
            self.assertIsNone(h.target['fresh'])


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

    def test_final_read_does_not_hide_source_loss_after_owned_shutdown(self):
        with TemporaryDirectory() as directory:
            h = Harness()
            project = Path(directory)
            source = project / 'scripts/subject.gd'
            source.parent.mkdir()
            source.write_text('# lost accepted edit\n')
            h.target.update(project=project, cursor=len(h.target['steps']), successes=8,
                            dirty_refusals=3, stale_refusals=1, no_match_refusals=1,
                            ambiguous_match_refusals=1, completed_read='actual-final-read')
            with self.assertRaises(composed.observation.Failure):
                h.finalize_targets()
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


    def test_save_rejects_coherent_clean_reversion_of_pre_save_human_source(self):
        with TemporaryDirectory() as directory:
            h = Harness()
            h.artifacts = Path(directory)
            human = SAFE + '# unsaved human work\n'
            before = {'target': {'dirty': True, 'B': human}}
            reverted = {'target': dict(associated=True, B=SAFE, R=SAFE, dirty=False,
                                      resource_edited=False, version=2, saved_version=2)}
            h.state.side_effect = [(before, {TARGET: {'text': SAFE}}),
                                   (reverted, {TARGET: {'text': SAFE}})]
            h.native_action.return_value = dict(focused=True, disk_matches=True)
            with self.assertRaises(composed.observation.Failure):
                composed.McpComposedMixin._composed_save(h, h.target)

    def test_save_requires_delivered_native_focus_and_disk_witness(self):
        for receipt in (dict(focused=False, disk_matches=True), dict(focused=True, disk_matches=False)):
            h = Harness()
            h.native_action.return_value = receipt
            with self.assertRaises(composed.observation.Failure):
                composed.McpComposedMixin._composed_save(h, h.target)

    def test_native_history_rejects_simulated_or_wrong_undo_and_disk_writes(self):
        before, disks = witness_state(opened=True)
        prior = SAFE.replace('return 47', 'return 46')
        for undone_source, redo_available, persisted in [(SAFE, True, disks), (prior, False, disks),
                                                        (prior, True, {TARGET: {'text': prior}}),
                                                        (SAFE + '# wrong prior\n', True, disks)]:
            h = Harness()
            undone, _ = witness_state(undone_source, opened=True)
            undone['documents'][0]['has_redo'] = redo_available
            h.state.side_effect = [(before, disks), (undone, disks), (before, persisted)]
            with self.assertRaises(composed.observation.Failure):
                h._composed_history(h.target, TARGET, expected_undo=prior)

    def test_success_claim_rejects_wrong_complete_source_or_missing_native_history(self):
        for changed in ('source', 'history'):
            with TemporaryDirectory() as directory:
                h = EvidenceHarness(directory)
                h.at('change', 1)
                h.target['pending'] = ('owned', 'change', 1)
                before = witness_state(opened=True)
                after, disks = witness_state(h.target['first_replacement'], opened=True)
                if changed == 'source':
                    after['target']['B'] += '# unrequested change\n'
                else:
                    after['target']['has_undo'] = False
                h.state.return_value = after, disks
                response = {'result': {'structuredContent': dict(schema_version=2, request_id='server',
                    error=None, result=dict(mode='open', outcome=dict(
                        outcome='verified_changed', application='applied')))}}
                with self.assertRaises(composed.observation.Failure):
                    h.observe_call(h.request('change', 1), response, before)
                self.assertIsNotNone(h.target['pending'])

    def test_closed_success_cannot_be_proven_only_after_opening(self):
        with TemporaryDirectory() as directory:
            h = EvidenceHarness(directory, 'cached')
            h.at('change', 1)
            h.target['pending'] = ('owned', 'change', 1)
            before = witness_state(cached=True)
            h.state.return_value = witness_state(h.target['first_replacement'], opened=True, cached=True)
            response = {'result': {'structuredContent': dict(schema_version=2, request_id='server',
                error=None, result=dict(mode='closed', outcome=dict(
                    outcome='verified_changed', application='applied', history='not_applicable_closed')))}}
            with self.assertRaises(composed.observation.Failure):
                h.observe_call(h.request('change', 1), response, before)
            self.assertIsNotNone(h.target['pending'])

    def test_read_rejects_incorrect_lifecycle_or_independently_observed_full_source(self):
        for lifecycle, source in [('open', SAFE), ('closed', SAFE + '# imaginary\n')]:
            with TemporaryDirectory() as directory:
                h = EvidenceHarness(directory, 'cached')
                h.at('read')
                h.target['pending'] = ('owned', 'read', None)
                before = witness_state(cached=True)
                h.state.return_value = before
                projection = dict(document={'lifecycle': lifecycle}, source_origin='disk',
                                  sources={'disk': {'equals_source': True, 'text': None}})
                response = {'result': {'structuredContent': dict(schema_version=2, request_id='server',
                    error=None, result=dict(source=source, revision='fresh', state=projection))}}
                with self.assertRaises(composed.observation.Failure):
                    h.observe_call(h.request('read'), response, before)
                self.assertIsNotNone(h.target['pending'])

    def test_matching_refusal_rejects_actual_source_history_or_timestamp_effect(self):
        for surface in ('buffer', 'cache', 'timestamp'):
            h = Harness('cached')
            h.at('no_match', 2)
            h.target['pending'] = ('owned', 'no_match', 2)
            before = witness_state(opened=True, cached=True)
            after, disks = copy.deepcopy(before)
            if surface == 'buffer':
                after['documents'][0]['B'] += '# lost human work\n'
            elif surface == 'cache':
                after['cached_R'] += '# changed cache\n'
            else:
                disks[TARGET]['mtime_ns'] += 1
            h.state.return_value = after, disks
            response = {'result': {'structuredContent': dict(error=None, result={'outcome': dict(
                outcome='refused', application='not_applied', reason='no_match', stage='matching',
                next_action={'kind': 'fresh_read'})})}}
            with self.assertRaises(composed.observation.Failure):
                h.observe_call(h.request('no_match', 2), response, before)
            self.assertEqual(h.target['no_match_refusals'], 0)


    def serial_harness(self, directory):
        h = Harness()
        h.work = Path(directory)
        h.args = SimpleNamespace(profile='composed')
        h.editors = []
        h.events = []
        h.case = mock.Mock()
        current = dict(path=CURRENT, script_id=1, editor_id=2, buffer_id=3,
                       R=SAFE, B=SAFE, version=2, saved_version=1, dirty=True,
                       resource_edited=False, has_undo=True, has_redo=False)
        h.state.return_value = ({'documents': [current]}, {TARGET: {'text': SAFE}})

        def prepare(owner, name):
            project = h.work / ('close-mcp-' + name)
            (project / 'scripts').mkdir(parents=True)
            (project / 'scripts/subject.gd').write_text(SAFE)
            editor = {'process': SimpleNamespace(pid=len(h.events) + 1)}
            h.editors.append(editor)
            h.events.append(('start', name))
            def close():
                h.editors.remove(editor)
                h.events.append(('stop', name))
            owner.callback(close)
            return dict(name=name, project=project, editor=editor,
                        descriptor={'session_id': name}, calls=[])
        h._prepare_target = prepare
        return h

    def test_profiles_never_overlap_and_advance_only_after_completed_final_read(self):
        with TemporaryDirectory() as directory, ExitStack() as stack:
            h = self.serial_harness(directory)
            h.prepare_targets(stack)
            self.assertEqual(h.events, [('start', 'open')])
            self.assertEqual([t['name'] for t in h.targets.values() if 'descriptor' in t], ['open'])
            for target in h.targets.values():
                h.target = target
                target.update(cursor=len(target['steps']) - 1, successes=composed.COUNTS[target['name']],
                              dirty_refusals=3 if target['name'] == 'open' else 0, stale_refusals=1,
                              no_match_refusals=1, ambiguous_match_refusals=1,
                              pending=('owned', 'durable_read', None))
                self.assertEqual(h.editors, [target['editor']])
                h.observe_call(h.request('durable_read'), {'result': {'structuredContent': {
                    'error': None, 'request_id': 'final-' + target['name'],
                    'result': {'revision': 'fresh', 'source': SAFE}}}}, h.state.return_value)
            self.assertEqual(h.events, [(event, profile) for profile in composed.COUNTS for event in ('start', 'stop')])
            self.assertEqual(h.editors, [])
            h.finalize_targets()

    def test_abandoned_serial_workflow_closes_current_and_never_starts_future_profiles(self):
        with TemporaryDirectory() as directory:
            h = self.serial_harness(directory)
            with self.assertRaisesRegex(RuntimeError, 'abandoned'), ExitStack() as stack:
                h.prepare_targets(stack)
                raise RuntimeError('abandoned')
            self.assertEqual(h.editors, [])
            self.assertEqual(h.events, [('start', 'open'), ('stop', 'open')])

if __name__ == '__main__':
    unittest.main()
