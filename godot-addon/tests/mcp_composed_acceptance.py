"""T004's fixed composed conversation, witnessed independently in the owned editor.

The relay calls before_composed_call before capturing/forwarding a tool request.
Fixture actions are ordinary human/history/lifecycle actions, never reconciliation.
The direct driver runs exactly this protocol but is NOT real-agent acceptance.
"""
from __future__ import annotations

from contextlib import ExitStack
import json
import re

import run_observation as observation
from closed_script_acceptance import source_free
from close_native_acceptance import documents
from mcp_lifecycle_acceptance import target_receipt
from mcp_peer import McpPeer, selectors
from run_script_close import TARGET, CURRENT

COUNTS = {'open': 8, 'cached': 6, 'absent': 6}


def composed_steps(profile):
    steps = [('discover', None), ('read', None)]
    for index in range(1, COUNTS[profile] + 1):
        steps.extend([('change', index), ('read', None)])
        if index == 1:
            steps.extend([('stale', None), ('read', None)])
            if profile == 'open':
                steps.extend((role, None) for role in ('undo_read', 'undo_saved_read',
                                                      'redo_read', 'redo_saved_read'))
        if profile == 'open' and index in (2, 4, 6):
            steps.extend([('dirty', index), ('dirty_read', None), ('human_saved_read', None)])
    steps.append(('durable_read', None))
    return steps


def tool_for(role):
    return 'discover_scripts' if role == 'discover' else 'edit_script' if role in ('change', 'dirty', 'stale') else 'read_script'


def replacement(source, profile, index):
    value = {'open': 100, 'cached': 200, 'absent': 300}[profile] + index
    text, count = re.subn(r'(?m)^\treturn \d+$', '\treturn ' + str(value), source)
    observation.require(count == 1, 'composed_single_owned_value_function')
    return text


class McpComposedMixin:
    def _is_composed(self):
        return getattr(self, '_composed_active', False)

    def prepare_targets(self, stack, profiles=None):
        if getattr(self.args, 'profile', None) != 'composed' and profiles != ('composed',):
            return super().prepare_targets(stack, profiles)
        receipts = super().prepare_targets(stack, tuple(COUNTS))
        self._composed_active = True
        for target in self.targets.values():
            editor = target['editor']
            self.close_action(editor, 'close_human', path=CURRENT, mutation='dirty_equal')
            self.close_action(editor, 'open_setup', paths=[], path=CURRENT, idle=True)
            if target['name'] == 'open':
                self.close_action(editor, 'close_human', path=TARGET, mutation='select')
                self.close_action(editor, 'close_human', path=TARGET, mutation='dirty_equal')
                self.close_action(editor, 'open_setup', paths=[], path=TARGET, idle=True)
                self._composed_save(target)
            before, disks = self.state(editor, target['project'])
            observation.require(documents(before)[CURRENT]['dirty'] and documents(before)[CURRENT]['has_undo'],
                                'composed_genuine_prior_unrelated_human_history')
            target.update(before=before, disks=disks, steps=composed_steps(target['name']), cursor=0,
                          pending=None, fresh=None, first_revision=None, successes=0, dirty_refusals=0,
                          stale_refusals=0, source=disks[TARGET]['text'], initial_source=disks[TARGET]['text'])
        return receipts

    def prepared_prompt(self):
        if not self._is_composed():
            return super().prepared_prompt()
        targets = []
        for t in self.targets.values():
            item = target_receipt(t['name'], t['project'], t['descriptor'])
            del item['script_path']
            item.update(script_hint='Find subject.gd in the inventory.',
                        requested_values=list(range({'open': 101, 'cached': 201, 'absent': 301}[t['name']],
                                                    {'open': 101, 'cached': 201, 'absent': 301}[t['name']] + COUNTS[t['name']])))
            targets.append(item)
        return (
            'Use only godot_agent_kit MCP tools for these projects, not shell/files/other bridges. '
            'Complete targets in listed order. Discover subject.gd, read it, then change only the return '
            'value of value() through each requested value in order, preserving all other text and comments. '
            'Read after every change and use that latest eligible read for the next change. '
            'For each target retain the initial read revision; immediately after the first change and its read, '
            'try the first replacement once with that old revision, explain the refusal, then read fresh. '
            'For open only, next the developer performs Undo, Save, Redo, Save: read four times in that order '
            'and explain which source is unsaved and which is persisted; never edit unsaved history. '
            'After open changes 2, 4 and 6 and their reads, the developer makes unsaved human work '
            '(the middle case has text equal to disk). Attempt the next requested value once with the '
            'just-read revision, explain the safe refusal, read the human work, then read again after the '
            'developer deliberately saves it. Continue the next intended change from that saved read, '
            'preserving human comments. After each target\'s final change/read, read once more after the '
            'developer closes/reopens the open target or first opens a closed target and checks persistence. '
            'Do not skip, batch, reorder, force, or retry edits blindly. Report actual observations and '
            'safe next actions, not assumptions about success. Targets:\n' + json.dumps(targets) + '\n')

    def _composed_save(self, target):
        state, disks = self.state(target['editor'], target['project'])
        action = 'native_edit_save' if state['target']['dirty'] else 'native_edit_save_clean'
        saved = self.native_action(target['editor'], action)
        observation.require(saved['focused'] and saved['disk_matches'], 'composed_ordinary_delivered_Save')
        after, now = self.state(target['editor'], target['project'])
        serial = target.get('save_serial', 0) + 1
        target['save_serial'] = serial
        label = 'composed-Save-' + target['name'] + '-' + str(serial)
        self.record_witness(label, state, disks, after, now, target['editor'])
        observation.json_file(self.artifacts / (label + '-receipt.json'), saved)

    def _composed_coherent(self, target, source):
        state, disks = self.state(target['editor'], target['project'])
        doc = state['target']
        observation.require(doc['associated'] and doc['B'] == doc['R'] == disks[TARGET]['text'] == source and
                            not doc['dirty'] and not doc['resource_edited'] and
                            doc['version'] == doc['saved_version'], 'composed_independent_clean_saved_D_R_B')
        return state, disks

    def before_composed_call(self, target, request):
        if not self._is_composed():
            return
        ordered = list(self.targets.values())
        first = next((t for t in ordered if t['cursor'] < len(t['steps'])), None)
        observation.require(target is first and target['pending'] is None, 'composed_ordered_one_owned_call')
        role, index = target['steps'][target['cursor']]
        params = request['params']
        args = params['arguments']
        observation.require(params['name'] == tool_for(role) and
                            args.get('session_id') == target['descriptor']['session_id'] and
                            (role == 'discover' or args.get('script_path') == TARGET),
                            'composed_exact_next_operation_and_target')
        if role in ('change', 'dirty', 'stale'):
            token = target['first_revision'] if role == 'stale' else (target['fresh'] or {}).get('revision')
            desired = target['first_replacement'] if role == 'stale' else replacement(
                target['source'], target['name'], index if role == 'change' else index + 1)
            observation.require(isinstance(token, str) and args.get('revision') == token and
                                args.get('replacement_source') == desired, 'composed_exact_fresh_or_deliberately_stale_intent')
        before, disks = self.state(target['editor'], target['project'])
        editor = target['editor']
        if role == 'discover':
            marker = target['project'] / 'scripts/.gdignore'
            if marker.exists():
                self.present_editor(editor)
                scanned = self.native_action(editor, 'native_edit_scan')
                observation.require(scanned['settled'], 'composed_cold_initial_index_settled')
                marker.unlink()
        if role == 'dirty':
            mutation = 'dirty_disk_equal' if index == 4 else 'dirty_equal'
            self.close_action(editor, 'close_human', path=TARGET, mutation=mutation)
            self.close_action(editor, 'open_setup', paths=[], path=TARGET, idle=True)
            human, human_disks = self.state(editor, target['project'])
            doc = human['target']
            observation.require(doc['dirty'] and doc['has_undo'] and doc['version'] != doc['saved_version'] and
                                ((doc['B'] == human_disks[TARGET]['text']) == (index == 4)),
                                'composed_actual_different_and_equal_text_dirty')
        elif role in ('undo_read', 'redo_read'):
            operation = role.split('_')[0]
            self.close_action(editor, 'close_human', path=TARGET, mutation=operation)
            self.close_action(editor, 'open_setup', paths=[], path=TARGET, idle=True)
            state, now = self.state(editor, target['project'])
            expected = target['initial_source'] if operation == 'undo' else target['first_replacement']
            observation.require(state['target']['B'] == state['target']['R'] == expected and
                                state['target']['dirty'] and now == disks, 'composed_genuine_unsaved_' + operation)
        elif role in ('undo_saved_read', 'redo_saved_read', 'human_saved_read'):
            if role == 'human_saved_read':
                self._composed_history(target, TARGET)
            self._composed_save(target)
            state, now = self.state(editor, target['project'])
            target['source'] = state['target']['B']
            self._composed_coherent(target, target['source'])
            if role == 'redo_saved_read':
                self._composed_prior_history(target)
        elif role == 'durable_read':
            observation.require(target['successes'] == COUNTS[target['name']], 'composed_primary_verified_before_lifecycle')
            self._composed_history(target, CURRENT)
            if target['name'] == 'open':
                self._composed_coherent(target, target['source'])
                self.close_action(editor, 'close_human', path=TARGET, mutation='close')
                closed, _ = self.state(editor, target['project'])
                observation.require(TARGET not in closed['open_paths'], 'composed_ordinary_close_observed')
                self.close_action(editor, 'close_human', path=TARGET, mutation='open')
                reopened, _ = self._composed_coherent(target, target['source'])
                observation.require((reopened['target']['editor_id'], reopened['target']['buffer_id']) !=
                                    (before['target']['editor_id'], before['target']['buffer_id']),
                                    'composed_reopen_new_visible_document')
            else:
                observation.require(TARGET not in before['open_paths'], 'composed_closed_primary_not_opened_to_verify')
                self.close_action(editor, 'close_human', path=TARGET, mutation='open')
                target['durability_start'] = len(target['calls'])
            self._composed_coherent(target, target['source'])
            self._composed_save(target)
            parsed = self.native_action(editor, 'native_edit_reparse')
            scanned = self.native_action(editor, 'native_edit_scan')
            observation.require(parsed['parse_completed'] and parsed['parse_error'] == 0 and scanned['settled'],
                                'composed_actual_Save_reparse_rescan')
            self.native_runtime(target['project'], {'open': 100, 'cached': 200, 'absent': 300}[target['name']] + COUNTS[target['name']])
            self._composed_coherent(target, target['source'])
        after, now = self.state(editor, target['project'])
        if role in ('dirty', 'undo_read', 'redo_read', 'undo_saved_read', 'redo_saved_read', 'human_saved_read', 'durable_read'):
            self.record_witness('composed-fixture-' + target['name'] + '-' + str(target['cursor']),
                                before, disks, after, now, editor)
        target['pending'] = (request['id'], role, index)

    def _composed_history(self, target, path, expected_undo=None):
        before, disks = self.state(target['editor'], target['project'])
        original = documents(before)[path]
        self.close_action(target['editor'], 'close_human', path=path, mutation='select')
        self.close_action(target['editor'], 'close_human', path=path, mutation='undo')
        undone, _ = self.state(target['editor'], target['project'])
        self.close_action(target['editor'], 'close_human', path=path, mutation='redo')
        self.close_action(target['editor'], 'open_setup', paths=[], path=path, idle=True)
        after, now = self.state(target['editor'], target['project'])
        observation.require(documents(undone)[path]['B'] != original['B'] and
                            documents(undone)[path]['has_redo'] and documents(after)[path]['B'] == original['B'] and
                            documents(after)[path]['has_undo'] and disks == now,
                            'composed_actual_preserved_human_history_' + path)
        observation.require(expected_undo is None or documents(undone)[path]['B'] == expected_undo,
                            'composed_each_native_edit_reverses_to_its_actual_prior_source')
        self.record_witness('composed-human-history-' + target['name'] + '-' + str(target['cursor']),
                            before, disks, after, now, target['editor'])
        if TARGET in after['open_paths']:
            self.close_action(target['editor'], 'close_human', path=TARGET, mutation='select')

    def _composed_prior_history(self, target):
        editor = target['editor']
        before, disks = self.state(editor, target['project'])
        for expected in (target['initial_source'], target['disks'][TARGET]['text'].replace('# CLOSE_HUMAN_EARLIER\n', '', 1)):
            self.close_action(editor, 'close_human', path=TARGET, mutation='undo')
            state, now = self.state(editor, target['project'])
            observation.require(state['target']['B'] == expected and state['target']['has_redo'] and now == disks,
                                'composed_prior_history_actually_reachable')
        for expected in (target['initial_source'], target['first_replacement']):
            self.close_action(editor, 'close_human', path=TARGET, mutation='redo')
            state, now = self.state(editor, target['project'])
            observation.require(state['target']['B'] == expected and now == disks, 'composed_prior_history_actual_redo')
        self.close_action(editor, 'open_setup', paths=[], path=TARGET, idle=True)
        self._composed_save(target)
        after, now = self._composed_coherent(target, target['first_replacement'])
        self.record_witness('composed-prior-history', before, disks, after, now, editor)

    def observe_call(self, request, response, before):
        if not self._is_composed():
            return super().observe_call(request, response, before)
        target = self.targets[request['params']['arguments']['project_root']]
        pending = target['pending']
        observation.require(pending is not None and pending[0] == request['id'], 'composed_correlated_next_response')
        _, role, index = pending
        # Existing witnesses check source origin, no-effect reads/refusals, native
        # identity, applicable R and genuinely continuing closed B absence.
        super().observe_call(request, response, before)
        content = response['result']['structuredContent']
        result = content['result']
        observation.require(content.get('error') is None and isinstance(result, dict), 'composed_actual_operation_result')
        after, now = self.state(target['editor'], target['project'])
        if role in ('change', 'dirty', 'stale'):
            outcome = result['outcome']
            expected = 'verified_changed' if role == 'change' else 'refused'
            observation.require(outcome['outcome'] == expected and outcome['application'] ==
                                ('applied' if role == 'change' else 'not_applied'), 'composed_required_changed_or_safe_refusal')
            if role == 'change':
                source = request['params']['arguments']['replacement_source']
                state, disks = before
                observation.require(after['selection'] == state['selection'] and
                                    all(documents(after)[path] == doc for path, doc in documents(state).items()
                                        if path != TARGET) and
                                    all(now[path] == disk for path, disk in disks.items() if path != TARGET),
                                    'composed_each_edit_preserves_unrelated_human_work_and_history')
                target['source'] = source
                target['successes'] += 1
                target['fresh'] = None
                if index == 1:
                    target['first_replacement'] = source
                if target['name'] == 'open':
                    self._composed_coherent(target, source)
                    observation.require(after['target']['has_undo'], 'composed_open_native_history_present')
                    self._composed_save(target)
                    self._composed_coherent(target, source)
                    if index > 1:
                        self._composed_history(target, TARGET, expected_undo=state['target']['B'])
                        self._composed_save(target)
                        self._composed_coherent(target, source)
                else:
                    observation.require(result['mode'] == 'closed' and outcome['history'] == 'not_applicable_closed',
                                        'composed_no_invented_closed_history')
            else:
                observation.require(outcome.get('reason') is not None and
                                    source_free(*before) == source_free(after, now), 'composed_refusal_preserves_every_observed_authority')
                action = outcome.get('next_action', {}).get('kind')
                observation.require(action == 'fresh_read' or isinstance(outcome.get('safe_next_action'), str),
                                    'composed_actionable_refusal_not_blind_retry')
                target[role + '_refusals'] += 1
        elif role != 'discover':
            unsaved = role in ('undo_read', 'redo_read', 'dirty_read')
            observation.require((result['revision'] is None) == unsaved, 'composed_unsaved_history_not_edit_permission')
            if unsaved:
                doc = after['target']
                observation.require(result['source'] == doc['B'] and doc['dirty'] and
                                    result['state']['source_origin'] == 'editor_buffer' and
                                    result['state']['dirty']['buffer']['state'] == 'dirty',
                                    'composed_truthful_unsaved_human_or_history_read')
                disk = result['state']['sources']['disk']
                observation.require(disk['current'] and disk['equals_source'] == (now[TARGET]['text'] == doc['B']) and
                                    (disk['text'] == now[TARGET]['text'] if now[TARGET]['text'] != doc['B'] else disk['text'] is None),
                                    'composed_unsaved_read_truthful_independent_persisted_D')
                target['fresh'] = None
            else:
                observation.require(isinstance(result['revision'], str) and result['source'] == target['source'],
                                    'composed_fresh_eligible_intended_source')
                target['fresh'] = result
                if target['first_revision'] is None:
                    target['first_revision'] = result['revision']
        target['cursor'] += 1
        target['pending'] = None

    def finalize_targets(self, *, primary_only=False):
        if not self._is_composed():
            return super().finalize_targets(primary_only=primary_only)
        for target in self.targets.values():
            observation.require(target['pending'] is None and target['cursor'] == len(target['steps']) and
                                target['successes'] == COUNTS[target['name']] and target['stale_refusals'] == 1 and
                                target['dirty_refusals'] == (3 if target['name'] == 'open' else 0),
                                'composed_no_omitted_skipped_or_unwitnessed_operations')
            self._composed_coherent(target, target['source'])
        self.summary.update(coverage_scope='T004_composed_MCP_A_E', mcp_acceptance=True,
                            composed=dict(successful_fresh_read_edits=20, dirty_refusals=3, stale_refusals=3,
                                          profiles=COUNTS, history='actual_Undo_Save_Redo_Save_and_prior_reachability',
                                          durability='ordinary_Save_close_reopen_reparse_rescan_fresh_runtime'))

    def mcp_composed(self):
        """Registerable actual-protocol regression; expressly not a coding agent."""
        self.compile_window_probe()
        with ExitStack() as stack:
            self.prepare_targets(stack, ('composed',))
            with McpPeer(self, 'composed-direct-not-real-agent') as peer:
                for target in self.targets.values():
                    while target['cursor'] < len(target['steps']):
                        role, index = target['steps'][target['cursor']]
                        args = selectors(target['project'], target['descriptor'])
                        if role == 'discover':
                            args.pop('script_path')
                        if role in ('change', 'dirty', 'stale'):
                            args.update(revision=target['first_revision'] if role == 'stale' else target['fresh']['revision'],
                                        replacement_source=target['first_replacement'] if role == 'stale' else
                                        replacement(target['source'], target['name'], index if role == 'change' else index + 1))
                        request = dict(id='composed-' + target['name'] + '-' + str(target['cursor']), method='tools/call',
                                       params=dict(name=tool_for(role), arguments=args))
                        self.before_composed_call(target, request)
                        before = self.state(target['editor'], target['project'])
                        content = peer.call(tool_for(role), args, request['id'])
                        self.observe_call(request, dict(result=dict(structuredContent=content)), before)
            self.finalize_targets()
            self.summary['real_client_acceptance'] = False
