"""Prepared real-agent failure interpretation using actual product effects.

Faults are confined to the existing fixture-only native build and owned relay.
The client still obtains every script operation through the public MCP process.
"""
from __future__ import annotations

import json
import socket
import threading
from contextlib import ExitStack

import run_observation as observation
from closed_script_acceptance import CHANGED, source_free
from close_native_acceptance import documents
from mcp_lifecycle_acceptance import target_receipt
from run_script_close import TARGET, SAFE


FAILURE_GROUPS = {
    'failures': ('failure_limited', 'failure_refusal', 'failure_partial', 'failure_unknown'),
    'reconnect': ('failure_reconnect',),
}
FAILURE_PROFILES = frozenset(name for group in FAILURE_GROUPS.values() for name in group)
DESIRED = '# MCP_FAILURE_REPLACEMENT\n' + CHANGED


class McpFailureMixin:
    def prepare_targets(self, stack, profiles=None):
        selected = tuple(profiles) if profiles is not None else FAILURE_GROUPS.get(self.args.profile)
        if selected is None or not any(name in FAILURE_PROFILES for name in selected):
            return super().prepare_targets(stack, profiles)
        observation.require(all(name in FAILURE_PROFILES for name in selected), 'fixed_failure_profile_set')
        self.targets = {}
        for name in selected:
            project, editor, descriptor = stack.enter_context(self.closed_fixture(
                'mcp-' + name, cached=True, faults=name in ('failure_limited', 'failure_partial')))
            before, disks = self.state(editor, project)
            self.targets[str(project)] = dict(name=name, project=project, editor=editor,
                descriptor=descriptor, before=before, disks=disks, desired=DESIRED, calls=[],
                fault_armed=False, delivery_dropped=False)
        self.summary.update(coverage_scope='T003_actual_MCP_failure_interpretation',
                            mcp_acceptance=True, real_client_acceptance=False)
        return [target_receipt(t['name'], t['project'], t['descriptor']) for t in self.targets.values()]

    def prepared_prompt(self):
        if not any(t['name'] in FAILURE_PROFILES for t in self.targets.values()):
            return super().prepared_prompt()
        targets = [dict(target_receipt(t['name'], t['project'], t['descriptor']),
                        intended_source=None if t['name'] == 'failure_limited' else t['desired'])
                   for t in self.targets.values()]
        return ('Use only godot_agent_kit MCP tools for project operations, not shell, files or other bridges. '
                'Complete one target at a time. Read each explicit target. For failure_limited, explain the '
                'observed source, unavailable state and revision; do not edit. For each other target, '
                'if read supplies an edit revision, send its exact complete source as old_string and the listed intended_source as new_string once with that revision. '
                'Interpret the actual structured outcome, target, effect certainty, evidence availability '
                'and next action. After that attempt, read the same explicit target to inspect surviving '
                'source and lifecycle, then stop editing it. If delivery fails, report that separately '
                'from whether the edit happened. Do not repair or repeat the edit. Explain what remains '
                'known or unknown and what a future intentional action would require. Targets:\n' +
                json.dumps(targets) + '\n')

    def arm_mcp_failure(self, target, request):
        """Fixture perturbations precede frame acceptance; no hidden product call."""
        name = target['name']
        operation = request['params']['name']
        if name == 'failure_limited' and operation == 'read_script':
            self.close_action(target['editor'], 'closed_fault', fault='missing_cache_getters')
        if operation != 'edit_script' or target['fault_armed']:
            return
        target['fault_armed'] = True
        if name == 'failure_refusal':
            before, disks = self.state(target['editor'], target['project'])
            self.closed_transition(target['project'], target['editor'], 'same_text')
            after, now = self.state(target['editor'], target['project'])
            observation.json_file(self.artifacts / (name + '-external-witness.json'),
                                  dict(before=source_free(before, disks), after=source_free(after, now)))
            target['before'], target['disks'] = after, now
        elif name == 'failure_partial':
            self.close_action(target['editor'], 'closed_fault', fault='partial_write')
        elif name == 'failure_unknown':
            self.close_action(target['editor'], 'closed_arm', kind='closed_applied')

    def after_mcp_failure_send(self, target, request, process):
        if target['name'] != 'failure_unknown' or request['params']['name'] != 'edit_script' or target.get('channel_lost'):
            return
        event = self.wait_closed_barrier(target['editor'], 'response:closed_applied',
                                        (process, None, 'mcp-client-ID-is-not-domain-ID'), acquisition=True)
        target['effect_request_id'] = event['request_id']
        actual, disk = self.state(target['editor'], target['project'])
        observation.require(disk[TARGET]['text'] == target['desired'] and
                            actual['cached_R'] == target['desired'] and TARGET not in actual['open_paths'],
                            'real_effect_before_private_acknowledgment_loss')
        self.close_action(target['editor'], 'closed_disconnect')
        self.close_action(target['editor'], 'closed_release')
        target['channel_lost'] = True

    def observe_call(self, request, response, before, delivery='delivered'):
        args = request.get('params', {}).get('arguments', {})
        target = self.targets.get(args.get('project_root'))
        if target is None or target['name'] not in FAILURE_PROFILES:
            return super().observe_call(request, response, before)
        name = request['params']['name']
        content = response.get('result', {}).get('structuredContent')
        observation.require(isinstance(content, dict), 'failure_actual_structured_carrier')
        state, disks = before
        after, now = self.state(target['editor'], target['project'])
        result = content.get('result')
        observation.require(content['request_id'] != request['id'], 'failure_separate_MCP_domain_ID')
        self.case('mcp-failure-' + str(len(self.cases)), operation=name, request_id=content['request_id'])
        if name in ('read_script', 'discover_scripts') or content.get('error') is not None:
            observation.require(source_free(state, disks) == source_free(after, now), 'failure_read_or_admission_no_effect')
        if name == 'read_script' and result:
            observation.require(result['source'] == now[TARGET]['text'] and
                                result['state']['document']['lifecycle'] == 'closed',
                                'failure_model_result_matches_actual_closed_survivor')
            if target['name'] == 'failure_limited':
                observation.require(result['revision'] is None and
                                    result['state']['dirty']['loaded_resource']['availability'] == 'unavailable',
                                    'limited_R_is_not_fabricated_absent_or_editable')
        if name == 'edit_script' and result:
            outcome = result['outcome']
            observation.require(response['result'].get('isError') is True or
                                (target['name'] == 'failure_reconnect' and not target['delivery_dropped']),
                                'failure_isError_preserves_structured_facts')
            observation.require(documents(state) == documents(after) and
                                state['selection'] == after['selection'] and
                                all(now[path] == value for path, value in disks.items() if path != TARGET) and
                                TARGET not in after['open_paths'], 'failure_preserves_unrelated_history_and_closed_lifecycle')
            if target['name'] in ('failure_refusal', 'failure_reconnect') and outcome['outcome'] == 'refused':
                observation.require(outcome['reason'] == 'revision_mismatch' and
                                    outcome['application'] == 'not_applied' and
                                    outcome['next_action']['kind'] == 'fresh_read' and
                                    source_free(state, disks) == source_free(after, now),
                                    'actual_stale_refusal_no_replay_or_repair')
            elif target['name'] == 'failure_partial':
                observation.require(outcome['outcome'] == 'applied_unverified' and
                                    outcome['application'] == 'partly_applied' and
                                    outcome['effects']['resource_changed'] and
                                    0 < outcome['effects']['written_bytes'] < len(target['desired'].encode()) and
                                    now[TARGET]['text'] not in (SAFE, target['desired']) and
                                    after['cached_R'] == target['desired'] and
                                    outcome['next_action']['kind'] == 'fresh_read',
                                    'actual_partial_R_D_effects_visible_to_client')
            elif target['name'] == 'failure_unknown':
                observation.require(outcome['outcome'] == 'application_unknown' and
                                    outcome['application'] == 'unknown' and
                                    content['request_id'] == target['effect_request_id'] and
                                    now[TARGET]['text'] == after['cached_R'] == target['desired'] and
                                    outcome['next_action']['kind'] == 'fresh_read',
                                    'lost_native_reply_unknown_not_false_not_applied')
            elif target['name'] == 'failure_reconnect':
                observation.require(outcome['outcome'] == 'verified_changed' and
                                    now[TARGET]['text'] == after['cached_R'] == target['desired'],
                                    'real_edit_before_relay_delivery_loss')
        self.record_witness('mcp-failure-' + str(len(self.cases)), state, disks, after, now, target['editor'])
        record = dict(id=request['id'], name=name, profile=target['name'], arguments=args,
                      structuredContent=content, delivery=delivery)
        target['calls'].append(record)
        with (self.artifacts / 'mcp-calls.jsonl').open('a') as stream:
            stream.write(json.dumps(record) + '\n')

    def drop_mcp_delivery(self, request, response, before):
        args = request.get('params', {}).get('arguments', {})
        target = self.targets.get(args.get('project_root'))
        if (target is None or target['name'] != 'failure_reconnect' or target['delivery_dropped'] or
                request['params']['name'] != 'edit_script' or
                not isinstance(response.get('result', {}).get('structuredContent', {}).get('result'), dict)):
            return False
        self.observe_call(request, response, before, delivery='unavailable')
        target['delivery_dropped'] = True
        self.cases[-1].update(delivery='unavailable', server_result_delivered=False)
        with (self.artifacts / 'delivery.jsonl').open('a') as stream:
            stream.write(json.dumps(dict(id=request['id'], request_id=target['calls'][-1]['structuredContent']['request_id'],
                                         delivery='unavailable', relay_closed_after_effect=True)) + '\n')
        return True

    def finalize_targets(self, primary_only=False):
        if not any(t['name'] in FAILURE_PROFILES for t in self.targets.values()):
            return super().finalize_targets(primary_only=primary_only)
        for target in self.targets.values():
            calls = target['calls']
            reads = [c for c in calls if c['name'] == 'read_script' and c['structuredContent']['result'] is not None]
            observation.require(bool(reads), 'failure_real_client_read')
            if target['name'] == 'failure_limited':
                observation.require(not any(c['name'] == 'edit_script' for c in calls), 'limited_no_blind_edit')
                self.assert_no_effect(target['name'], target['project'], target['editor'], target['before'], target['disks'])
                continue
            edits = [c for c in calls if c['name'] == 'edit_script']
            observation.require(all(isinstance(c['structuredContent']['result'], dict) for c in edits),
                                'failure_no_unchecked_or_legacy_edit_substitute')
            observation.require(bool(edits) and calls.index(reads[0]) < calls.index(edits[0]) < calls.index(reads[-1]),
                                'failure_fresh_read_before_and_after_intent')
            initial = reads[0]['structuredContent']['result']
            observation.require(isinstance(initial.get('source'), str) and isinstance(initial.get('revision'), str) and
                                edits[0]['arguments'].get('revision') == initial['revision'] and
                                edits[0]['arguments'].get('old_string') == initial['source'] and
                                edits[0]['arguments'].get('new_string') == target['desired'] and
                                not (set(edits[0]['arguments']) -
                                     {'project_root', 'session_id', 'script_path', 'revision', 'old_string', 'new_string'}),
                                'failure_model_uses_returned_revision_and_requested_source')
            if target['name'] != 'failure_reconnect':
                observation.require(len(edits) == 1, 'no_blind_replay_after_refusal_or_uncertainty')
            else:
                observation.require(target['delivery_dropped'], 'actual_reply_delivery_loss')
                duplicates = edits[1:]
                observation.require(all(c['arguments'] == edits[0]['arguments'] and
                                        c['structuredContent']['result']['outcome']['outcome'] == 'refused'
                                        for c in duplicates), 'client_resend_cannot_inherit_old_intent')
                self.summary['reconnect_edit_resends'] = len(duplicates)
                self.summary['reconnect_requires_client_transcript_attribution'] = True
            final, disks = self.state(target['editor'], target['project'])
            observation.require(reads[-1]['structuredContent']['result']['source'] == disks[TARGET]['text'] and
                                TARGET not in final['open_paths'], 'failure_actual_fresh_survivor_read')
        self.summary.setdefault('mcp_profiles_verified', []).extend(t['name'] for t in self.targets.values())

    def mcp_transport(self):
        selected = self.args.scenario.removeprefix('transport-')
        if selected not in FAILURE_GROUPS:
            return super().mcp_transport()
        with ExitStack() as stack:
            self.prepare_targets(stack, FAILURE_GROUPS[selected])
            errors = []
            def connection():
                client, owner = socket.socketpair()
                stack.callback(client.close)
                stack.callback(owner.close)
                def relay():
                    try:
                        self.relay_connection(owner)
                    except Exception as error:
                        errors.append(error)
                    finally:
                        owner.close()
                thread = threading.Thread(target=relay)
                thread.start()
                client.settimeout(12)
                return client, client.makefile('rb'), thread
            client, reader, thread = connection()
            serial = 0
            def call(method, params):
                nonlocal serial
                serial += 1
                client.sendall(json.dumps(dict(jsonrpc='2.0', id=serial, method=method, params=params)).encode()+b'\n')
                line = reader.readline(64 * 1024 * 1024 + 1)
                if not line:
                    raise EOFError('fixture delivery unavailable')
                response = json.loads(line)
                observation.require(response.get('id') == serial and 'error' not in response, 'failure_MCP_response')
                return response['result']
            def initialize():
                call('initialize', dict(protocolVersion='2025-11-25', capabilities={},
                                       clientInfo=dict(name='fixed-failure-driver', version='1')))
                client.sendall(b'{"jsonrpc":"2.0","method":"notifications/initialized"}\n')
            try:
                initialize()
                for target in self.targets.values():
                    args = target_receipt(target['name'], target['project'], target['descriptor'])
                    args.pop('name')
                    first = call('tools/call', dict(name='read_script', arguments=args))['structuredContent']['result']
                    if target['name'] == 'failure_limited':
                        continue
                    edit_args = dict(args, revision=first['revision'], old_string=first['source'], new_string=target['desired'])
                    try:
                        call('tools/call', dict(name='edit_script', arguments=edit_args))
                    except EOFError:
                        observation.require(target['name'] == 'failure_reconnect', 'only_injected_delivery_loss')
                        reader.close()
                        client.close()
                        thread.join(timeout=12)
                        observation.require(not thread.is_alive(), 'first_relay_owned_cleanup')
                        client, reader, thread = connection()
                        initialize()
                        call('tools/call', dict(name='edit_script', arguments=edit_args))
                    call('tools/call', dict(name='read_script', arguments=args))
                call('ping', {})
            finally:
                client.shutdown(socket.SHUT_WR)
                thread.join(timeout=12)
                reader.close()
                observation.require(not thread.is_alive(), 'failure_relay_owned_cleanup')
            if errors:
                raise errors[0]
            self.finalize_targets()
