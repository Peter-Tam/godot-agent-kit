"""Owned real-editor MCP lifecycle fixtures and private connection evidence.

The persistent guest owner survives relay EOF. Only fixed finalization tears down
its projects; reconnecting clients cannot accidentally close an editor.
"""
from __future__ import annotations

from contextlib import ExitStack
import json
import hashlib
from pathlib import Path
import re
import select
import socket
import subprocess
import time

import run_observation as observation
from closed_script_acceptance import CHANGED, SOURCE_LIMIT, source_free
from close_native_acceptance import documents
from run_script_close import TARGET, CURRENT, BACKGROUND, SAFE

PROFILE_GROUPS = {
    'workflow': ('open', 'cached', 'absent'),
    'known': ('known',),
    'sources': ('unicode', 'empty', 'empty_desired'),
    'bound': ('bound',),
    'observations': ('dirty', 'divergent', 'limited', 'invalidated', 'partial'),
}


def lifecycle_profile(name):
    return name if name in ('open', 'cached') else 'absent'


def exact_steps(name, source):
    """Independent intended spans; localized cases cannot use whole-source substitutes."""
    steps = []
    def add(old, new):
        nonlocal source
        intended = new if source == old == '' else source.replace(old, new, 1)
        steps.append(dict(old_string=old, new_string=new, source=intended))
        source = intended
    if name == 'empty':
        add('', '')
        add('', CHANGED)
    elif name == 'bound':
        add('return 47', 'return 47')
        add('\treturn 47\n', '\treturn 47\n# ' +
            'x' * (SOURCE_LIMIT - len(source.encode()) - 3) + '\n')
    else:
        add('return 47', 'return 47')
        if name == 'unicode':
            add('\t# café 雪 𐐀\n\treturn 47', '\t# naïve 雨 𐐀\n\treturn 83')
            add('extends RefCounted\n', 'extends RefCounted\n# anchored insertion\n')
        else:
            add('return 47', 'return 83')
        if name in ('open', 'cached', 'absent'):
            add('# localized deletion witness\n', '')
            add(source, '')
            add('', '')
            add('', CHANGED)
        elif name == 'empty_desired':
            add(source, '')
    return steps


def review_exact_sequence(target, calls):
    """Consume server records, checking intent, fresh basis and complete read state."""
    steps = target['steps']
    cursor = 0
    basis = None
    awaiting_read = False
    revision_change = None
    expected = target['initial_source']
    lifecycle = lifecycle_profile(target['name'])
    for call in calls:
        result = call.get('structuredContent', {}).get('result')
        if call['name'] == 'edit_script' and not isinstance(result, dict):
            raise ValueError('edit has no recorded checked operation result')
        if not isinstance(result, dict):
            continue
        if call['name'] == 'read_script':
            if (result.get('source') != expected or not isinstance(result.get('revision'), str) or
                    re.fullmatch(r'sr1:[0-9a-f]{64}', result['revision']) is None or
                    result.get('state', {}).get('document', {}).get('lifecycle') !=
                    ('open' if lifecycle == 'open' else 'closed')):
                raise ValueError('incorrect complete source, revision or lifecycle')
            if revision_change is not None and (result['revision'] != basis) != revision_change:
                raise ValueError('fresh result read revision contradicts edit outcome')
            revision_change = None
            basis = result['revision']
            awaiting_read = False
        elif call['name'] == 'edit_script':
            outcome = result['outcome']['outcome']
            if outcome == 'refused' and target['name'] == 'known':
                if result['outcome']['application'] != 'not_applied':
                    raise ValueError('stale refusal has effects')
                awaiting_read = True
                continue
            if awaiting_read or basis is None or cursor >= len(steps):
                raise ValueError('missing fresh read before intentional edit')
            args = call['arguments']
            step = steps[cursor]
            old, new = args.get('old_string'), args.get('new_string')
            if (args.get('revision') != basis or not isinstance(old, str) or not isinstance(new, str) or
                    set(args) - {'project_root', 'session_id', 'script_path', 'revision', 'old_string', 'new_string'}):
                raise ValueError('incorrect exact intent or read revision')
            if old:
                start = expected.find(old)
                unique = start >= 0 and expected.find(old, start + 1) < 0
            else:
                unique = expected == ''
            localized = bool(step['old_string']) and step['old_string'] != expected
            if (not unique or (localized and old == expected) or
                    (new if expected == old == '' else expected.replace(old, new, 1)) != step['source']):
                raise ValueError('incorrect exact intent or read revision')
            wanted = 'verified_unchanged' if expected == step['source'] else 'verified_changed'
            revision_change = wanted == 'verified_changed'
            if outcome != wanted:
                raise ValueError('exact intent did not establish expected outcome')
            expected = step['source']
            cursor += 1
            awaiting_read = True
    if cursor != len(steps) or awaiting_read:
        raise ValueError('missing intentional edit or fresh result read')
    return expected


def target_receipt(name, project, descriptor):
    return dict(name=name, project_root=str(project), session_id=descriptor['session_id'], script_path=TARGET)


def control_path(run):
    return run.parents[1] / 'tmp' / ('mcp-' + hashlib.sha256(run.name.encode()).hexdigest()[:24] + '.sock')


def review_client_events(events, calls):
    """Review normalized completed model/tool events, never assistant verdicts.

    Client-specific extraction must retain actual model-facing content (including
    recovered client-owned spill files), call arguments, and result correlation.
    A catalog or raw structuredContent event alone is deliberately insufficient.
    """
    visible = {event['content']['request_id']: event for event in events
               if event.get('kind') == 'model_visible_tool_result' and
               isinstance(event.get('content'), dict) and isinstance(event['content'].get('request_id'), str)}
    actions = [event for event in events if event.get('kind') == 'tool_call']
    if any(a.get('name') not in ('discover_scripts', 'read_script', 'edit_script') and
           not a.get('client_owned_result_recovery', False) for a in actions):
        raise ValueError('non-product operation substitution')
    if not calls or not visible or not actions:
        raise ValueError('missing correlated model-visible tool evidence')
    unmatched = [(call['name'], call['arguments']) for call in calls]
    for action in actions:
        if action.get('name') in ('discover_scripts', 'read_script', 'edit_script'):
            pair = (action['name'], action.get('arguments'))
            if pair not in unmatched:
                raise ValueError('claimed tool call missing server record')
            unmatched.remove(pair)
    first_reads = set()
    undelivered = 0
    for call in calls:
        key = call['structuredContent']['request_id']
        if call.get('delivery') == 'unavailable':
            if key in visible:
                raise ValueError('undelivered result claimed model-visible')
            undelivered += 1
            continue
        if key not in visible or visible[key]['content'] != call['structuredContent']:
            raise ValueError('required authoritative result not model-visible')
        result = call['structuredContent'].get('result')
        if call['name'] == 'read_script' and result and result.get('revision'):
            later = [a for a in actions if a.get('name') == 'edit_script' and
                     a.get('arguments', {}).get('revision') == result['revision']]
            selector = call['arguments']['project_root']
            if selector not in first_reads and call.get('profile') not in ('dirty', 'divergent', 'limited', 'partial'):
                if not later:
                    raise ValueError('model did not use returned revision')
                first_reads.add(selector)
    return {'model_visible_results': len(calls) - undelivered,
            'delivery_unavailable_results': undelivered, 'claims_are_not_evidence': True,
            'qualitative_state_and_next_action_review_required': True}


class McpLifecycleMixin:
    def mcp_transport(self):
        selected = self.args.scenario.removeprefix('transport-')
        groups = PROFILE_GROUPS.values() if selected == 'transport' else [PROFILE_GROUPS[selected]]
        for profiles in groups:
            self._mcp_transport_group(profiles)

    def _mcp_transport_group(self, profiles):
        import threading
        observation.require(self.args.mcp_server is not None, 'actual_MCP_binary_required')
        with ExitStack() as stack:
            self.prepare_targets(stack, profiles)
            client, owner = socket.socketpair()
            stack.callback(client.close)
            stack.callback(owner.close)
            errors = []
            def relay():
                try:
                    self.relay_connection(owner)
                except Exception as error:
                    errors.append(error)
                finally:
                    owner.close()
            thread = threading.Thread(target=relay)
            thread.start()
            client.settimeout(15)
            reader = client.makefile('rb')
            serial = 0
            def call(method, params):
                nonlocal serial
                serial += 1
                client.sendall(json.dumps(dict(jsonrpc='2.0', id=serial, method=method, params=params)).encode() + b'\n')
                response = json.loads(reader.readline(64 * 1024 * 1024 + 1))
                observation.require(response.get('id') == serial and 'error' not in response, 'actual_MCP_response')
                return response['result']
            try:
                initialized = call('initialize', dict(protocolVersion='2025-11-25', capabilities={},
                                                     clientInfo=dict(name='fixed-lifecycle-driver', version='1')))
                observation.require(initialized['protocolVersion'] == '2025-11-25', 'MCP_selected_revision')
                client.sendall(b'{"jsonrpc":"2.0","method":"notifications/initialized"}\n')
                catalog = call('tools/list', {})
                observation.require([t['name'] for t in catalog['tools']] ==
                                    ['discover_scripts', 'read_script', 'edit_script'], 'MCP_exact_catalog')
                for target in self.targets.values():
                    selector = dict(project_root=str(target['project']), session_id=target['descriptor']['session_id'])
                    def tool(name, **fields):
                        return call('tools/call', dict(name=name, arguments=dict(selector, **fields)))['structuredContent']['result']
                    if target['name'] != 'known':
                        tool('discover_scripts')
                    first = tool('read_script', script_path=TARGET)
                    if target['name'] in ('dirty', 'divergent', 'limited', 'invalidated', 'partial'):
                        continue
                    second = tool('read_script', script_path=TARGET)
                    observation.require(first['revision'] == second['revision'], 'MCP_stable_revision')
                    for step in target['steps']:
                        basis = tool('read_script', script_path=TARGET)
                        tool('edit_script', script_path=TARGET, revision=basis['revision'],
                             old_string=step['old_string'], new_string=step['new_string'])
                        tool('read_script', script_path=TARGET)
                    if target['name'] == 'known':
                        tool('edit_script', script_path=TARGET, revision=first['revision'],
                             old_string='return 83', new_string='return 83')
                        tool('read_script', script_path=TARGET)
                if any(t['name'] in ('cached', 'absent') for t in self.targets.values()):
                    # The reply precedes its independent witness; a later ping
                    # drains that witness before main-thread fixture opening.
                    call('ping', {})
                    self.prepare_durability()
                    for target in self.targets.values():
                        if target['name'] in ('cached', 'absent'):
                            call('tools/call', dict(name='read_script', arguments=dict(
                                project_root=str(target['project']), session_id=target['descriptor']['session_id'],
                                script_path=TARGET)))
            finally:
                client.shutdown(socket.SHUT_WR)
                thread.join(timeout=15)
                observation.require(not thread.is_alive(), 'bounded_owned_relay_cleanup')
                reader.close()
            if errors:
                raise errors[0]
            self.finalize_targets()
            self.summary.update(coverage_scope='exact_edit_actual_MCP_transport_lifecycle', mcp_acceptance=True,
                                real_client_acceptance=False)

    def prepare_targets(self, stack, profiles=None):
        self.targets = {}
        for name in profiles if profiles is not None else PROFILE_GROUPS[self.args.profile]:
            target = self._prepare_target(stack, name)
            self.targets[str(target['project'])] = target
        return [target_receipt(t['name'], t['project'], t['descriptor']) for t in self.targets.values()]

    def _prepare_target(self, stack, name):
        source = '' if name == 'empty' else SAFE
        if name == 'unicode':
            source = source.replace('\treturn 47', '\t# café 雪 𐐀\n\treturn 47')
        if name in ('open', 'cached', 'absent') and self.args.profile != 'composed':
            source += '# localized deletion witness\n'
        profile = lifecycle_profile(name)
        if profile == 'open':
            fixture = self.close_fixture('mcp-' + name, source=source,
                paths=[TARGET, CURRENT, BACKGROUND], selected=TARGET)
        else:
            fixture = self.closed_fixture('mcp-' + name, cached=profile == 'cached' or name in ('dirty', 'divergent'),
                                          source=source, faults=name == 'limited')
        project, editor, descriptor = stack.enter_context(fixture)
        if name == 'open' and self.args.profile != 'composed':
            self.close_action(editor, 'close_human', path=TARGET, mutation='dirty_equal')
            self.close_action(editor, 'open_setup', paths=[], path=TARGET, idle=True)
            saved = self.native_action(editor, 'native_edit_save')
            observation.require(saved['focused'] and saved['disk_matches'], 'exact_prior_history_saved')
            source = '# CLOSE_HUMAN_EARLIER\n' + source
        if name in ('dirty', 'divergent'):
            self.close_action(editor, 'closed_resource', mutation=name)
        if name == 'limited':
            self.close_action(editor, 'closed_fault', fault='missing_cache_getters')
        if name == 'partial':
            gap = project / 'unreadable'
            gap.mkdir()
            (gap / 'hidden.gd').write_text(SAFE)
            gap.chmod(0)
            stack.callback(gap.chmod, 0o700)
        before, disks = self.state(editor, project)
        steps = exact_steps(name, source)
        desired = steps[-1]['source']
        return dict(name=name, project=project, editor=editor, descriptor=descriptor,
                    before=before, disks=disks, desired=desired, calls=[],
                    initial_source=source, steps=steps)

    def prepared_intents(self):
        return [dict(name=t['name'], replacement_sha256=hashlib.sha256(t['desired'].encode()).hexdigest(),
                     replacement_utf8_bytes=len(t['desired'].encode()),
                     observation_only=t['name'] in ('dirty', 'divergent', 'limited', 'invalidated', 'partial'))
                for t in self.targets.values()]

    def prepared_prompt(self):
        targets = []
        for target in self.targets.values():
            item = target_receipt(target['name'], target['project'], target['descriptor'])
            if target['name'] != 'known' and target['name'] not in ('dirty', 'divergent', 'limited', 'invalidated', 'partial'):
                del item['script_path']
                item['script_hint'] = 'Find the script named subject.gd in the returned inventory.'
            if target['name'] in ('dirty', 'divergent', 'limited', 'invalidated', 'partial'):
                item['requested_changes'] = None
            elif target['name'] == 'bound':
                item['requested_changes'] = [
                    'Request an unchanged edit of the return 47 span.',
                    'Retain the final return line as an anchor and append one comment: "# ", '
                    'enough ASCII x characters to make the complete source exactly 524288 UTF-8 bytes, then one LF.',
                ]
            elif target['name'] == 'empty':
                item['requested_changes'] = [
                    'Request an unchanged edit of the complete empty script.',
                    'Replace the complete empty script with this exact source: ' + json.dumps(CHANGED),
                ]
            else:
                item['requested_changes'] = ['Request an unchanged edit of the return 47 span.']
                if target['name'] == 'unicode':
                    item['requested_changes'] += [
                        'In one multiline replacement, change the tab-indented comment café 雪 𐐀 '
                        'to naïve 雨 𐐀 and the following return 47 to return 83.',
                        'Retain the extends line as an anchor and insert "# anchored insertion\\n" immediately after it.',
                    ]
                else:
                    item['requested_changes'].append('Replace only return 47 with return 83.')
                if target['name'] in ('open', 'cached', 'absent'):
                    item['requested_changes'] += [
                        'Delete only the "# localized deletion witness\\n" comment, preserving all other source.',
                        'Delete the complete source, leaving the existing script empty.',
                        'Request an unchanged edit of the complete empty script.',
                        'Replace the complete empty script with this exact source: ' + json.dumps(CHANGED),
                    ]
                elif target['name'] == 'empty_desired':
                    item['requested_changes'].append('Delete the complete source, leaving the existing script empty.')
            targets.append(item)
        return ('Use only the godot_agent_kit MCP tools for project operations; do not use shell, files, '
                'or other bridges. Complete each listed target before the next. For positive targets discover '
                '(except known), read twice to compare unchanged revisions, then perform every listed requested_change '
                'in order. Quote exact old text from the latest read, use its revision, and fresh-read after EACH edit. '
                'Use localized spans unless the requested change explicitly concerns the complete source. Preserve all other bytes. '
                'For known, read and edit directly without discovery. Discover partial and explain incomplete '
                'coverage (never infer absence); read it without editing. Read dirty, divergent, limited and invalidated targets, '
                'explain source_origin, document.lifecycle, dirty, consistency, limitations, invalidated historical evidence and null revision; '
                'do not edit them. For known only, after changing it intentionally try the original now-stale '
                'revision once with old_string and new_string both "return 83", explain the refusal/next_action, then fresh-read. '
                'Explain the source, opaque revisions, lifecycle, and effects you actually observed. '
                'Keep exact Unicode/empty/bound source without truncation. Targets and requested changes:\n' +
                json.dumps(targets, ensure_ascii=False) + '\n')

    def observe_call(self, request, response, before):
        params = request.get('params', {})
        name = params.get('name')
        args = params.get('arguments', {})
        target = self.targets.get(args.get('project_root'))
        if target is None:
            return
        content = response.get('result', {}).get('structuredContent')
        observation.require(isinstance(content, dict), 'actual_MCP_authoritative_structured_result')
        observation.require(content.get('schema_version') == 2, 'actual_root_Schema_2_result')
        self.case('mcp-' + str(len(self.cases)), operation=name)
        self.last_workflow_request = content['request_id']
        result = content.get('result')
        state, disks = before
        after, now = self.state(target['editor'], target['project'])
        if not target.get('durability_start') and lifecycle_profile(target['name']) == 'absent' and target['name'] not in ('dirty', 'divergent', 'limited'):
            observation.require(not state['cached_id'] and not after['cached_id'],
                                'MCP_confirmed_absent_R_through_actual_call')
        if name in ('read_script', 'discover_scripts'):
            observation.require(source_free(state, disks) == source_free(after, now), 'MCP_observation_no_native_effect')
        if name == 'discover_scripts' and result and target['name'] != 'partial':
            observation.require(TARGET in (result.get('inventory') or {}).get('entries', []),
                                'MCP_target_present_in_discovery_scope')
        if name == 'read_script' and result:
            observation.require(set(result) == {'source', 'revision', 'state'}, 'MCP_exact_read_projection')
            if target['name'] != 'invalidated':
                observation.require(result['source'] == (after['target']['B'] if TARGET in after['open_paths'] else now[TARGET]['text']),
                                'MCP_independent_source_provenance')
            if target['name'] == 'invalidated':
                authority = result['state']['sources']['disk']
                observation.require(result['source'] is None and result['revision'] is None and
                                    result['state']['revision_unavailable_reason'] is not None and
                                    not authority['current'] and authority['invalidated'] is not None and
                                    authority['invalidated']['current'] is False and
                                    authority['invalidated']['text'] == target['disks'][TARGET]['text'] and
                                    result['state']['source_origin'] != 'disk', 'MCP_invalidated_historical_D_not_current_or_empty')
            observation.require(all(a.get('text') is None for a in result['state']['sources'].values()
                                    if a.get('equals_source') is True), 'MCP_no_duplicate_agreeing_source')
            projection = result['state']
            opened = TARGET in after['open_paths']
            if target['name'] != 'invalidated':
                observation.require(projection['document']['lifecycle'] == ('open' if opened else 'closed') and
                                    projection['source_origin'] == ('editor_buffer' if opened else 'disk'),
                                    'MCP_independent_lifecycle_and_authority_origin')
            observation.require(not response['result'].get('isError', False), 'MCP_informative_read_not_execution_error')
            if not opened and target['name'] != 'limited':
                authority = projection['sources']['loaded_resource']
                dirty = projection['dirty']['loaded_resource']
                if after['cached_id']:
                    observation.require(authority['current'] and
                                        authority['equals_source'] == (after['cached_R'] == result['source']) and
                                        dirty['state'] == ('dirty' if after['cached_edited'] else 'clean') and
                                        (authority['text'] == after['cached_R'] if after['cached_R'] != result['source']
                                         else authority['text'] is None), 'MCP_independent_cached_R_source_and_dirty')
                else:
                    observation.require(not authority['current'] and authority['text'] is None and
                                        dirty['availability'] == 'not_applicable', 'MCP_confirmed_absent_R_not_unavailable')
            if target['name'] == 'limited':
                observation.require(projection['dirty']['loaded_resource']['availability'] == 'unavailable' and
                                    projection['revision_unavailable_reason'] is not None,
                                    'MCP_unavailable_not_fabricated_absent')
        if name == 'edit_script' and result:
            outcome = result['outcome']['outcome']
            self.last_workflow_edit = result
            if outcome == 'refused':
                observation.require(result['outcome']['application'] == 'not_applied' and
                                    source_free(state, disks) == source_free(after, now), 'MCP_refusal_independent_no_effect')
            if outcome in ('verified_changed', 'verified_unchanged'):
                old, new = args['old_string'], args['new_string']
                current = state['target']['B'] if TARGET in state['open_paths'] else disks[TARGET]['text']
                observation.require((old == current == '') or
                                    (bool(old) and current.find(old) >= 0 and
                                     current.find(old, current.find(old) + 1) < 0),
                                    'MCP_exact_unique_original_span')
                desired = new if old == current == '' else current.replace(old, new, 1)
                if TARGET not in state['open_paths']:
                    self.assert_closed_success(target['name'], target['project'], target['editor'], state, disks,
                                               desired, changed=outcome == 'verified_changed')
                else:
                    observation.require(after['target']['B'] == after['target']['R'] == now[TARGET]['text'] == desired and
                                        documents(state).keys() == documents(after).keys() and
                                        not after['target']['dirty'] and not after['target']['resource_edited'] and
                                        after['target']['version'] == after['target']['saved_version'] and
                                        all(state['target'][key] == after['target'][key] for key in
                                            ('script_id', 'editor_id', 'buffer_id')),
                                        'MCP_open_D_R_B_same_document')
                    if outcome == 'verified_unchanged':
                        observation.require(source_free(state, disks) == source_free(after, now), 'MCP_open_noop_zero_effect')
        self.record_witness('mcp-' + str(len(self.cases)), state, disks, after, now, target['editor'])
        record = dict(id=request['id'], name=name, profile=target['name'], arguments=args, structuredContent=content)
        target['calls'].append(record)
        with (self.artifacts / 'mcp-calls.jsonl').open('a') as stream:
            stream.write(json.dumps(record, ensure_ascii=False) + '\n')

    def relay_connection(self, connection, control_server=None):
        connection.settimeout(15)
        diagnostics = (self.artifacts / 'mcp-stderr.log').open('ab')
        process = subprocess.Popen([str(self.args.mcp_server), '--registry', str(self.registry),
                                    '--validator-engine', str(self.args.godot)],
                                   stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                   stderr=diagnostics)
        pending = {}
        buffers = {connection: bytearray(), process.stdout: bytearray()}
        try:
            while True:
                readers = list(buffers)
                if control_server is not None:
                    readers.append(control_server)
                ready, _, _ = select.select(readers, [], [], 15 if pending else None)
                observation.require(bool(ready), 'MCP_consumed_output_deadline')
                for incoming in ready:
                    if incoming is control_server:
                        finalizer, _ = control_server.accept()
                        finalizer.settimeout(15)
                        try:
                            command = finalizer.recv(1)
                            if command in (b'F', b'P'):
                                return finalizer, command
                        except BaseException:
                            finalizer.close()
                            raise
                        finalizer.close()
                        continue
                    data = incoming.recv(65536) if incoming is connection else __import__('os').read(incoming.fileno(), 65536)
                    if not data:
                        return
                    buffers[incoming].extend(data)
                    observation.require(len(buffers[incoming]) <= 64 * 1024 * 1024, 'bounded_relay_frame')
                    while b'\n' in buffers[incoming]:
                        line, _, rest = buffers[incoming].partition(b'\n')
                        buffers[incoming] = bytearray(rest)
                        try:
                            value = json.loads(line)
                            if not isinstance(value, dict):
                                value = {}
                        except (ValueError, UnicodeDecodeError):
                            value = {}
                        if incoming is connection:
                            invalidating = False
                            failure_target = None
                            params = value.get('params')
                            identity = value.get('id')
                            if value.get('method') == 'initialize':
                                version = params.get('protocolVersion') if isinstance(params, dict) else None
                                record = dict(event='initialize_request', protocol_version=version
                                              if isinstance(version, str) and re.fullmatch(r'[0-9]{4}-[0-9]{2}-[0-9]{2}', version)
                                              else None)
                                with (self.artifacts / 'protocol.jsonl').open('a') as stream:
                                    stream.write(json.dumps(record) + '\n')
                            if (value.get('method') == 'tools/call' and isinstance(params, dict) and
                                    params.get('name') in ('discover_scripts', 'read_script', 'edit_script') and
                                    isinstance(params.get('arguments'), dict) and type(identity) in (str, int) and
                                    identity not in pending and len(pending) < 8):
                                project = params['arguments'].get('project_root')
                                target = self.targets.get(project) if isinstance(project, str) else None
                                if target and self.args.profile == 'composed':
                                    self.before_composed_call(target, value)
                                marker = target['project'] / 'scripts/.gdignore' if target else None
                                if (params['name'] == 'discover_scripts' and target and
                                        target['name'] != 'partial' and marker.exists()):
                                    self.present_editor(target['editor'])
                                    scanned = self.native_action(target['editor'], 'native_edit_scan')
                                    observation.require(scanned['settled'], 'MCP_initial_index_settled')
                                    marker.unlink()
                                invalidating = target and target['name'] == 'invalidated' and params['name'] == 'read_script'
                                if invalidating:
                                    observation.require(not target.get('invalidated_after'), 'single_controlled_invalidated_read')
                                    self.close_action(target['editor'], 'closed_arm', kind='closed_state')
                                if target and target['name'].startswith('failure_'):
                                    failure_target = target
                                    self.arm_mcp_failure(target, value)
                                pending[identity] = (value, self.state(target['editor'], target['project']) if target else None,
                                                     time.monotonic())
                            process.stdin.write(line + b'\n')
                            process.stdin.flush()
                            if failure_target is not None:
                                self.after_mcp_failure_send(failure_target, value, process)
                            if value.get('method') == 'tools/call' and invalidating:
                                self.wait_closed_barrier(target['editor'], 'response:closed_state',
                                                         (process, None, 'fixture-no-request'), acquisition=True)
                                self.closed_transition(target['project'], target['editor'], 'same_text')
                                external_state, external_disks = self.state(target['editor'], target['project'])
                                self.record_witness('invalidated-external-change', target['before'], target['disks'],
                                                    external_state, external_disks, target['editor'])
                                target['invalidated_after'] = (external_state, external_disks)
                                pending[identity] = (value, (external_state, external_disks), pending[identity][2])
                                self.close_action(target['editor'], 'closed_release')
                        else:
                            owned_response = pending.get(value.get('id')) if type(value.get('id')) in (str, int) else None
                            if owned_response:
                                request, before, _ = owned_response
                                project = request['params']['arguments'].get('project_root')
                                target = self.targets.get(project)
                                if target and target['name'] == 'failure_reconnect' and self.drop_mcp_delivery(request, value, before):
                                    return
                            connection.sendall(line + b'\n')
                            delivered = time.monotonic()
                            identity = value.get('id')
                            result = value.get('result')
                            control = None
                            if isinstance(result, dict):
                                if 'protocolVersion' in result:
                                    control = dict(event='initialized', protocol_version=result['protocolVersion'],
                                                   capabilities=result.get('capabilities'))
                                elif 'tools' in result:
                                    control = dict(event='catalog', tools=result['tools'])
                                    tools = result['tools']
                                    observation.require([t['name'] for t in tools] ==
                                                        ['discover_scripts', 'read_script', 'edit_script'],
                                                        'refreshed_exact_three_tool_catalog')
                                    observation.require(all(t['outputSchema']['properties']['schema_version']['const'] == 2
                                                            for t in tools), 'refreshed_root_Schema_2_catalog')
                                    edit_schema = tools[2]['inputSchema']
                                    observation.require(set(edit_schema['required']) ==
                                                        {'project_root', 'script_path', 'revision', 'old_string', 'new_string'} and
                                                        set(edit_schema['properties']) ==
                                                        {'project_root', 'session_id', 'script_path', 'revision', 'old_string', 'new_string'} and
                                                        edit_schema['additionalProperties'] is False,
                                                        'refreshed_single_exact_edit_shape')
                                elif 'supportedVersions' in result:
                                    control = dict(event='version_discovery', supported_versions=result['supportedVersions'])
                            if control is not None:
                                with (self.artifacts / 'protocol.jsonl').open('a') as stream:
                                    stream.write(json.dumps(control) + '\n')
                            owned = (pending.pop(identity, None) if type(identity) in (str, int) and
                                     isinstance(result, dict) and isinstance(result.get('structuredContent'), dict) else None)
                            if owned:
                                request, before, started = owned
                                limit = 10 if request['params']['name'] == 'edit_script' else 5
                                observation.require(delivered - started <= limit, 'MCP_original_consumed_output_bound')
                                if before:
                                    self.observe_call(request, value, before)
                                with (self.artifacts / 'delivery.jsonl').open('a') as stream:
                                    stream.write(json.dumps(dict(id=request['id'], request_id=result['structuredContent']['request_id'],
                                                                 relay_elapsed_ms=(delivered-started)*1000))+'\n')
        finally:
            process.stdin.close()
            try:
                process.wait(timeout=12)
            except subprocess.TimeoutExpired:
                process.terminate()
                try:
                    process.wait(timeout=2)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait(timeout=2)
            process.stdout.close()
            diagnostics.close()

    def exact_history_witness(self, target):
        """Ordinary native history, never a second edit or fixture-source repair."""
        editor = target['editor']
        before, disks = self.state(editor, target['project'])
        changed = []
        source = target['initial_source']
        for step in target['steps']:
            if step['source'] != source:
                changed.append((source, step['source']))
            source = step['source']
        serial = 0
        def history(mutation, expected):
            nonlocal serial
            prior, prior_disks = self.state(editor, target['project'])
            self.close_action(editor, 'close_human', path=TARGET, mutation=mutation)
            state, now = self.state(editor, target['project'])
            observation.require(state['target']['B'] == expected and
                                state['target']['has_' + ('redo' if mutation == 'undo' else 'undo')],
                                'exact_native_' + mutation + '_complete_source')
            observation.require(now == prior_disks and
                                (expected == now[TARGET]['text'] or
                                 (state['target']['dirty'] and
                                  state['target']['version'] != state['target']['saved_version'])),
                                'exact_native_history_unsaved_without_disk_write')
            serial += 1
            self.record_witness('exact-history-' + str(serial), prior, prior_disks, state, now, editor)
            return state, now
        def save(expected):
            nonlocal serial
            prior, prior_disks = self.state(editor, target['project'])
            self.close_action(editor, 'open_setup', paths=[], path=TARGET, idle=True)
            state, _ = self.state(editor, target['project'])
            action = 'native_edit_save' if state['target']['dirty'] else 'native_edit_save_clean'
            saved = self.native_action(editor, action)
            state, now = self.state(editor, target['project'])
            observation.require(saved['focused'] and saved['disk_matches'] and
                                state['target']['B'] == state['target']['R'] == now[TARGET]['text'] == expected and
                                not state['target']['dirty'] and
                                state['target']['version'] == state['target']['saved_version'],
                                'exact_history_Save_complete_D_R_B')
            serial += 1
            self.record_witness('exact-history-save-' + str(serial), prior, prior_disks, state, now, editor)
        history('undo', changed[-1][0])
        save(changed[-1][0])
        history('redo', changed[-1][1])
        save(changed[-1][1])
        for old, new in reversed(changed):
            history('undo', old)
        history('undo', target['initial_source'].replace('# CLOSE_HUMAN_EARLIER\n', '', 1))
        history('redo', target['initial_source'])
        for old, new in changed:
            history('redo', new)
        save(target['desired'])
        after, now = self.state(editor, target['project'])
        self.record_witness('exact-native-history', before, disks, after, now, editor)
        target['history_witnessed'] = True

    def finalize_targets(self, *, primary_only=False):
        for target in self.targets.values():
            calls = target['calls']
            if target.get('durability_start') is not None:
                primary_calls = calls[:target['durability_start']]
            else:
                primary_calls = calls
            # Admission/host errors remain recorded attempts, not operation evidence.
            calls = [c for c in primary_calls
                     if isinstance(c.get('structuredContent', {}).get('result'), dict)]
            names = [c['name'] for c in calls]
            observation.require('read_script' in names, 'real_client_read_' + target['name'])
            if target['name'] == 'known':
                observation.require(not any(c['name'] == 'discover_scripts' for c in target['calls']),
                                    'direct_known_target_without_discovery')
            if target['name'] == 'partial':
                discoveries = [c['structuredContent']['result'] for c in calls if c['name'] == 'discover_scripts']
                observation.require(any(d['inventory']['coverage'] == 'partial' for d in discoveries),
                                    'real_client_partial_discovery')
                self.assert_no_effect('partial', target['project'], target['editor'], target['before'], target['disks'])
                continue
            if target['name'] in ('dirty', 'divergent', 'limited', 'invalidated'):
                observation.require(any(c['structuredContent']['result']['revision'] is None for c in calls
                                        if c['name'] == 'read_script'), 'informative_unsafe_null_revision')
                baseline = target.get('invalidated_after', (target['before'], target['disks']))
                self.assert_no_effect(target['name'], target['project'], target['editor'], *baseline)
                continue
            observation.require('edit_script' in names and (target['name'] == 'known' or 'discover_scripts' in names),
                                'real_client_positive_sequence_' + target['name'])
            edits = [c for c in calls if c['name'] == 'edit_script']
            review_exact_sequence(target, primary_calls)
            outcomes = [c['structuredContent']['result']['outcome']['outcome'] for c in edits]
            observation.require('verified_changed' in outcomes and 'verified_unchanged' in outcomes,
                                'real_client_changed_and_noop_' + target['name'])
            if target['name'] == 'known':
                observation.require(any(c['structuredContent']['result']['outcome']['outcome'] == 'refused' and
                                        c['structuredContent']['result']['outcome']['next_action']['kind'] == 'fresh_read'
                                        for c in edits), 'real_client_actionable_stale_refusal')
            reads = [c['structuredContent']['result'] for c in calls if c['name'] == 'read_script']
            observation.require(reads[-1]['source'] == target['desired'] and
                                (reads[-1]['revision'] != reads[0]['revision'] or target['desired'] == target['initial_source']),
                                'real_client_fresh_read_changed_revision')
            if target['name'] != 'known':
                observation.require(len(reads) >= 3 and reads[0]['revision'] == reads[1]['revision'],
                                    'real_client_unchanged_revision_stability')
            if target['name'] == 'open' and not target.get('history_witnessed'):
                self.exact_history_witness(target)
            if target['name'] in ('cached', 'absent') and not primary_only:
                observation.require(target.get('durability_start') is not None, 'fixture_durability_prepared')
                later = target['calls'][target['durability_start']:]
                observation.require(later and all(c['name'] == 'read_script' for c in later),
                                    'post_open_read_only_client_continuation')
                observation.require(any(isinstance(c['structuredContent']['result'], dict) and
                                        c['structuredContent']['result']['source'] == CHANGED and
                                        c['structuredContent']['result']['state']['document']['lifecycle'] == 'open'
                                        for c in later), 'later_open_actual_client_fresh_MCP_read')
                self.native_action(target['editor'], 'native_edit_save_clean')
                parsed = self.native_action(target['editor'], 'native_edit_reparse')
                scanned = self.native_action(target['editor'], 'native_edit_scan')
                observation.require(parsed['parse_completed'] and parsed['parse_error'] == 0 and scanned['settled'],
                                    'later_Save_reparse_rescan')
                self.native_runtime(target['project'], 83)
                state, disks = self.state(target['editor'], target['project'])
                observation.require(state['target']['B'] == state['target']['R'] == disks[TARGET]['text'] == CHANGED,
                                    'later_runtime_D_R_B_durability')
            if target['name'] == 'open' and not primary_only:
                self.close_action(target['editor'], 'close_human', path=TARGET, mutation='reopen')
                parsed = self.native_action(target['editor'], 'native_edit_reparse')
                scanned = self.native_action(target['editor'], 'native_edit_scan')
                observation.require(parsed['parse_completed'] and parsed['parse_error'] == 0 and scanned['settled'],
                                    'exact_open_reopen_reparse_rescan')
                self.native_runtime(target['project'], 83)
                state, disks = self.state(target['editor'], target['project'])
                observation.require(state['target']['B'] == state['target']['R'] == disks[TARGET]['text'] == target['desired'],
                                    'exact_open_fresh_runtime_complete_D_R_B')
        if not primary_only:
            self.summary.setdefault('mcp_profiles_verified', []).extend(t['name'] for t in self.targets.values())


    def prepare_durability(self):
        observation.require(not any(t.get('durability_start') is not None for t in self.targets.values()),
                            'durability_prepare_once')
        observation.require(any(t['name'] in ('cached', 'absent') for t in self.targets.values()),
                            'durability_workflow_profile_required')
        self.finalize_targets(primary_only=True)
        targets = []
        for target in self.targets.values():
            if target['name'] in ('cached', 'absent'):
                target['durability_start'] = len(target['calls'])
                self.public_open(target['project'], target['descriptor'], 'mcp-later-' + target['name'])
                state, disks = self.state(target['editor'], target['project'])
                observation.require(state['target']['B'] == state['target']['R'] == disks[TARGET]['text'] == CHANGED,
                                    'ordinary_open_preserves_verified_primary_source')
                targets.append(target_receipt(target['name'], target['project'], target['descriptor']))
        return ('Use only godot_agent_kit MCP read_script for each target below. Read the now-open script, '
                'explain source, source_origin, lifecycle and agreement of applicable disk/loaded_resource/editor_buffer. '
                'Do not edit, discover, use shell/files or other bridges.\n' + json.dumps(targets) + '\n')


def serve_prepared(harness, run):
    import signal
    def interrupted(signum, frame):
        raise KeyboardInterrupt
    signal.signal(signal.SIGTERM, interrupted)
    socket_path = control_path(run)
    with ExitStack() as stack:
        harness.initialize()
        harness.compile_window_probe()
        targets = harness.prepare_targets(stack)
        server = stack.enter_context(socket.socket(socket.AF_UNIX))
        server.bind(str(socket_path))
        stack.callback(socket_path.unlink, missing_ok=True)
        server.listen(1)
        server.settimeout(3600)
        provenance = json.loads((run / 'provenance.json').read_text())
        observation.json_file(run / 'prepared.json', dict(revision=harness.args.revision, targets=targets,
            registry=str(harness.registry), executable=str(harness.args.mcp_server),
            executable_sha256=observation.digest(harness.args.mcp_server),
            environment_identity=provenance['environment_identity'],
            provenance_sha256=observation.digest(run / 'provenance.json'),
            native_build_id=harness.expected_native_build_id,
            intents=harness.prepared_intents(),
            fixture_driver_sha256=observation.digest(Path(__file__))))
        (run / 'prompt.txt').write_text(harness.prepared_prompt())
        try:
            while True:
                connection, _ = server.accept()
                with connection:
                    connection.settimeout(15)
                    command = connection.recv(1)
                    if command == b'R':
                        handed = harness.relay_connection(connection, server)
                        if handed is None:
                            continue
                        finalizer, command = handed
                    elif command in (b'F', b'P'):
                        finalizer = connection
                    else:
                        raise ValueError('invalid prepared control')
                    with finalizer:
                        if command == b'P':
                            prompt = harness.prepare_durability()
                            (run / 'durability-prompt.txt').write_text(prompt)
                            finalizer.sendall(prompt.encode())
                            continue
                        harness.finalize_targets()
                        stack.close()
                        harness.cleanup()
                        harness.verify_incidental_redaction()
                        harness.prepared_cleanup_complete = True
                        harness.summary.update(status='passed', mcp_acceptance=True, real_client_acceptance=False,
                                               model_visibility_review_required=True)
                        harness.summary['passed_case_count'] = len(harness.cases)
                        observation.json_file(harness.artifacts / 'summary.json', harness.summary)
                        finalizer.sendall(b'OK\n')
                        return
        finally:
            if harness.summary.get('status') != 'passed':
                harness.summary['status'] = 'failed'
            observation.json_file(harness.artifacts / 'summary.json', harness.summary)
            socket_path.unlink(missing_ok=True)
