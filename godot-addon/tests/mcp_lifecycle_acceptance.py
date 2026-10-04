"""Owned real-editor MCP lifecycle fixtures and private connection evidence.

The persistent guest owner survives relay EOF. Only fixed finalization tears down
its projects; reconnecting clients cannot accidentally close an editor.
"""
from __future__ import annotations

from contextlib import ExitStack
import json
import hashlib
from pathlib import Path
import select
import socket
import subprocess
import time

import run_observation as observation
from closed_script_acceptance import CHANGED, SOURCE_LIMIT, source_free
from close_native_acceptance import documents
from run_script_close import TARGET, CURRENT, BACKGROUND, SAFE

PROFILE_GROUPS = {
    'workflow': ('open', 'cached', 'absent', 'known'),
    'sources': ('unicode', 'empty', 'empty_desired'),
    'bound': ('bound',),
    'observations': ('dirty', 'divergent', 'limited', 'partial'),
}


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
    first_reads = set()
    for call in calls:
        key = call['structuredContent']['request_id']
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
    return {'model_visible_results': len(calls), 'claims_are_not_evidence': True,
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
                    if target['name'] in ('dirty', 'divergent', 'limited', 'partial'):
                        continue
                    second = tool('read_script', script_path=TARGET)
                    observation.require(first['revision'] == second['revision'], 'MCP_stable_revision')
                    tool('edit_script', script_path=TARGET, revision=first['revision'], replacement_source=first['source'])
                    tool('edit_script', script_path=TARGET, revision=first['revision'], replacement_source=target['desired'])
                    if target['name'] == 'known':
                        tool('edit_script', script_path=TARGET, revision=first['revision'], replacement_source=target['desired'])
                    tool('read_script', script_path=TARGET)
            finally:
                client.shutdown(socket.SHUT_WR)
                thread.join(timeout=15)
                observation.require(not thread.is_alive(), 'bounded_owned_relay_cleanup')
                reader.close()
            if errors:
                raise errors[0]
            self.finalize_targets()
            self.summary.update(coverage_scope='T002_actual_MCP_transport_lifecycle', mcp_acceptance=True,
                                real_client_acceptance=False)

    def prepare_targets(self, stack, profiles=None):
        self.targets = {}
        for name in profiles if profiles is not None else PROFILE_GROUPS[self.args.profile]:
            source = '' if name == 'empty' else SAFE
            if name == 'open':
                fixture = self.close_fixture('mcp-' + name, source=source,
                    paths=[TARGET, CURRENT, BACKGROUND], selected=TARGET)
            else:
                fixture = self.closed_fixture('mcp-' + name, cached=name in ('cached', 'dirty', 'divergent'),
                                              source=source, faults=name == 'limited')
            project, editor, descriptor = stack.enter_context(fixture)
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
            desired = CHANGED
            if name == 'unicode':
                desired += '# café 雪 🐳\n'
            if name == 'empty':
                desired = SAFE
            if name == 'empty_desired':
                desired = ''
            if name == 'bound':
                desired = SAFE + '# ' + 'x' * (SOURCE_LIMIT - len(SAFE.encode()) - 3) + '\n'
            self.targets[str(project)] = dict(name=name, project=project, editor=editor,
                descriptor=descriptor, before=before, disks=disks, desired=desired, calls=[])
        return [target_receipt(t['name'], t['project'], t['descriptor']) for t in self.targets.values()]

    def prepared_prompt(self):
        targets = []
        for target in self.targets.values():
            item = target_receipt(target['name'], target['project'], target['descriptor'])
            if target['name'] == 'bound':
                item['requested_change'] = ('Keep the current source unchanged and append one comment: "# ", '
                    'enough ASCII x characters to make the complete source exactly 524288 UTF-8 bytes, then one LF.')
            else:
                item['replacement_source'] = target['desired'] if target['name'] not in ('dirty', 'divergent', 'limited', 'partial') else None
            targets.append(item)
        return ('Use only the godot_agent_kit MCP tools for project operations; do not use shell, files, '
                'or other bridges. For the listed targets only, use the three tools. For open/cached/absent/unicode/empty/empty_desired/bound targets, '
                'discover, read twice to compare unchanged revisions, submit an already-satisfied edit using '
                'the exact returned source/revision, edit to the requested replacement, then fresh-read. '
                'For known, read and edit directly without discovery. Discover partial and explain incomplete '
                'coverage (never infer absence); read it without editing. Read dirty, divergent and limited targets, '
                'explain source_origin, document.lifecycle, dirty, consistency, limitations and null revision; '
                'do not edit them. For known only, after changing it intentionally try the original now-stale '
                'revision once with the same desired source, explain the refusal/next_action, then fresh-read. '
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
        self.case('mcp-' + str(len(self.cases)), operation=name)
        self.last_workflow_request = content['request_id']
        result = content.get('result')
        state, disks = before
        after, now = self.state(target['editor'], target['project'])
        if name in ('read_script', 'discover_scripts'):
            observation.require(source_free(state, disks) == source_free(after, now), 'MCP_observation_no_native_effect')
        if name == 'read_script' and result:
            observation.require(set(result) == {'source', 'revision', 'state'}, 'MCP_exact_read_projection')
            observation.require(result['source'] == (after['target']['B'] if TARGET in after['open_paths'] else now[TARGET]['text']),
                                'MCP_independent_source_provenance')
            observation.require(all(a.get('text') is None for a in result['state']['sources'].values()
                                    if a.get('equals_source') is True), 'MCP_no_duplicate_agreeing_source')
            projection = result['state']
            opened = TARGET in after['open_paths']
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
                desired = args['replacement_source']
                if TARGET not in state['open_paths']:
                    self.assert_closed_success(target['name'], target['project'], target['editor'], state, disks,
                                               desired, changed=outcome == 'verified_changed')
                else:
                    observation.require(after['target']['B'] == after['target']['R'] == now[TARGET]['text'] == desired and
                                        documents(state).keys() == documents(after).keys() and
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

    def relay_connection(self, connection):
        connection.settimeout(15)
        diagnostics = (self.artifacts / 'mcp-stderr.log').open('ab')
        process = subprocess.Popen([str(self.args.mcp_server), '--registry', str(self.registry)],
                                   stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                   stderr=diagnostics)
        pending = {}
        buffers = {connection: bytearray(), process.stdout: bytearray()}
        try:
            while True:
                ready, _, _ = select.select(list(buffers), [], [], 15 if pending else None)
                observation.require(bool(ready), 'MCP_consumed_output_deadline')
                for incoming in ready:
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
                            params = value.get('params')
                            identity = value.get('id')
                            if (value.get('method') == 'tools/call' and isinstance(params, dict) and
                                    params.get('name') in ('discover_scripts', 'read_script', 'edit_script') and
                                    isinstance(params.get('arguments'), dict) and type(identity) in (str, int) and
                                    identity not in pending and len(pending) < 8):
                                project = params['arguments'].get('project_root')
                                target = self.targets.get(project) if isinstance(project, str) else None
                                pending[identity] = (value, self.state(target['editor'], target['project']) if target else None,
                                                     time.monotonic())
                            process.stdin.write(line + b'\n')
                            process.stdin.flush()
                        else:
                            connection.sendall(line + b'\n')
                            delivered = time.monotonic()
                            identity = value.get('id')
                            result = value.get('result')
                            owned = (pending.pop(identity, None) if type(identity) in (str, int) and
                                     isinstance(result, dict) and isinstance(result.get('structuredContent'), dict) else None)
                            if owned:
                                request, before, started = owned
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

    def finalize_targets(self):
        for target in self.targets.values():
            calls = target['calls']
            names = [c['name'] for c in calls]
            observation.require('read_script' in names, 'real_client_read_' + target['name'])
            if target['name'] == 'known':
                observation.require('discover_scripts' not in names, 'direct_known_target_without_discovery')
            if target['name'] == 'partial':
                discoveries = [c['structuredContent']['result'] for c in calls if c['name'] == 'discover_scripts']
                observation.require(any(d['inventory']['coverage'] == 'partial' for d in discoveries),
                                    'real_client_partial_discovery')
                self.assert_no_effect('partial', target['project'], target['editor'], target['before'], target['disks'])
                continue
            if target['name'] in ('dirty', 'divergent', 'limited'):
                observation.require(any(c['structuredContent']['result']['revision'] is None for c in calls
                                        if c['name'] == 'read_script'), 'informative_unsafe_null_revision')
                self.assert_no_effect(target['name'], target['project'], target['editor'], target['before'], target['disks'])
                continue
            observation.require('edit_script' in names and (target['name'] == 'known' or 'discover_scripts' in names),
                                'real_client_positive_sequence_' + target['name'])
            edits = [c for c in calls if c['name'] == 'edit_script']
            outcomes = [c['structuredContent']['result']['outcome']['outcome'] for c in edits]
            observation.require('verified_changed' in outcomes and (target['name'] == 'known' or 'verified_unchanged' in outcomes),
                                'real_client_changed_and_noop_' + target['name'])
            if target['name'] == 'known':
                observation.require(any(c['structuredContent']['result']['outcome']['outcome'] == 'refused' and
                                        c['structuredContent']['result']['outcome']['next_action']['kind'] == 'fresh_read'
                                        for c in edits), 'real_client_actionable_stale_refusal')
            reads = [c['structuredContent']['result'] for c in calls if c['name'] == 'read_script']
            observation.require(reads[-1]['source'] == target['desired'] and reads[-1]['revision'] != reads[0]['revision'],
                                'real_client_fresh_read_changed_revision')
            if target['name'] != 'known':
                observation.require(len(reads) >= 3 and reads[0]['revision'] == reads[1]['revision'],
                                    'real_client_unchanged_revision_stability')
            if target['name'] in ('cached', 'absent'):
                self.public_open(target['project'], target['descriptor'], 'mcp-later-' + target['name'])
                durable_read = self.durability_read(target)
                observation.require(durable_read['source'] == CHANGED and
                                    durable_read['state']['document']['lifecycle'] == 'open',
                                    'later_open_actual_fresh_MCP_read')
                self.native_action(target['editor'], 'native_edit_save_clean')
                parsed = self.native_action(target['editor'], 'native_edit_reparse')
                scanned = self.native_action(target['editor'], 'native_edit_scan')
                observation.require(parsed['parse_completed'] and parsed['parse_error'] == 0 and scanned['settled'],
                                    'later_Save_reparse_rescan')
                self.native_runtime(target['project'], 83)
                state, disks = self.state(target['editor'], target['project'])
                observation.require(state['target']['B'] == state['target']['R'] == disks[TARGET]['text'] == CHANGED,
                                    'later_runtime_D_R_B_durability')
        self.summary.setdefault('mcp_profiles_verified', []).extend(t['name'] for t in self.targets.values())


    def durability_read(self, target):
        process = subprocess.Popen([str(self.args.mcp_server), '--registry', str(self.registry)],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
        def exchange(identity, method, params):
            process.stdin.write(json.dumps(dict(jsonrpc='2.0', id=identity, method=method, params=params)).encode() + b'\n')
            process.stdin.flush()
            observation.require(bool(select.select([process.stdout], [], [], 10)[0]), 'later_MCP_read_deadline')
            value = json.loads(process.stdout.readline(64 * 1024 * 1024 + 1))
            observation.require(value.get('id') == identity and 'error' not in value, 'later_MCP_actual_response')
            return value['result']
        try:
            exchange(1, 'initialize', dict(protocolVersion='2025-11-25', capabilities={},
                                          clientInfo=dict(name='fixture-durability-witness', version='1')))
            process.stdin.write(b'{"jsonrpc":"2.0","method":"notifications/initialized"}\n')
            process.stdin.flush()
            value = exchange(2, 'tools/call', dict(name='read_script', arguments=dict(
                project_root=str(target['project']), session_id=target['descriptor']['session_id'], script_path=TARGET)))
            observation.json_file(self.artifacts / ('later-' + target['name'] + '-mcp.json'),
                                  dict(evidence_origin='fixture_later_durability', response=value))
            return value['structuredContent']['result']
        finally:
            process.stdin.close()
            try:
                process.wait(timeout=2)
            except subprocess.TimeoutExpired:
                process.terminate()
                try:
                    process.wait(timeout=2)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait(timeout=2)
            process.stdout.close()


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
            intents=[dict(name=t['name'], replacement_sha256=hashlib.sha256(t['desired'].encode()).hexdigest(),
                          replacement_utf8_bytes=len(t['desired'].encode()),
                          observation_only=t['name'] in ('dirty', 'divergent', 'limited', 'partial'))
                     for t in harness.targets.values()],
            fixture_driver_sha256=observation.digest(Path(__file__))))
        (run / 'prompt.txt').write_text(harness.prepared_prompt())
        try:
            while True:
                connection, _ = server.accept()
                with connection:
                    connection.settimeout(15)
                    command = connection.recv(1)
                    if command == b'R':
                        harness.relay_connection(connection)
                    elif command == b'F':
                        harness.finalize_targets()
                        stack.close()
                        harness.cleanup()
                        harness.verify_incidental_redaction()
                        harness.prepared_cleanup_complete = True
                        harness.summary.update(status='passed', mcp_acceptance=True, real_client_acceptance=False,
                                               model_visibility_review_required=True)
                        harness.summary['passed_case_count'] = len(harness.cases)
                        observation.json_file(harness.artifacts / 'summary.json', harness.summary)
                        connection.sendall(b'OK\n')
                        return
                    else:
                        raise ValueError('invalid prepared control')
        finally:
            if harness.summary.get('status') != 'passed':
                harness.summary['status'] = 'failed'
            observation.json_file(harness.artifacts / 'summary.json', harness.summary)
            socket_path.unlink(missing_ok=True)
