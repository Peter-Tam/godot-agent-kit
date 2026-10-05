"""Bounded actual-stdio calls and independent witnesses for adversarial acceptance.

This fixture never supplies an operation result, source witness or mutation authority
inside the product. Public results remain unmodified in private evidence files.
"""
from __future__ import annotations

import json
import os
import re
import secrets
import select
import subprocess
import time

import run_observation as observation
from closed_script_acceptance import source_free
from run_script_close import TARGET


FRAME_LIMIT = 64 * 1024 * 1024


def selectors(project, descriptor, script_path=TARGET, select_session=True):
    result = dict(project_root=str(project), script_path=script_path)
    if select_session:
        result['session_id'] = descriptor['session_id']
    return result


class McpPeer:
    """One owned product process; no automatic retries or result substitution."""

    def __init__(self, harness, label):
        self.harness = harness
        self.label = label
        self.process = None
        self.buffer = bytearray()
        self.diagnostics = None
        self.pending = {}

    def __enter__(self):
        observation.require(self.harness.args.mcp_server is not None, 'actual_MCP_binary_required')
        self.stderr_path = self.harness.artifacts / (self.label + '-mcp-stderr.log')
        self.diagnostics = self.stderr_path.open('wb')
        self.process = subprocess.Popen(
            [str(self.harness.args.mcp_server), '--registry', str(self.harness.registry)],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=self.diagnostics, bufsize=0,
            env=dict(os.environ, RUST_LOG='trace'))
        self.harness.open_processes.append(self.process)
        os.set_blocking(self.process.stdin.fileno(), False)
        os.set_blocking(self.process.stdout.fileno(), False)
        try:
            self.send(dict(jsonrpc='2.0', id='initialize', method='initialize', params=dict(
                protocolVersion='2025-11-25', capabilities={},
                clientInfo=dict(name='adversarial-acceptance', version='1'))))
            response = self.receive(timeout=10)
            observation.require(response.get('id') == 'initialize' and
                                response.get('result', {}).get('protocolVersion') == '2025-11-25' and
                                response['result']['capabilities'] == {'tools': {}},
                                'actual_MCP_selected_protocol_and_capability')
            self.send(dict(jsonrpc='2.0', method='notifications/initialized'))
            return self
        except BaseException:
            self.__exit__(None, None, None)
            raise

    def send(self, value):
        data = json.dumps(value, ensure_ascii=False, separators=(',', ':')).encode() + b'\n'
        self.send_bytes(data)

    def send_bytes(self, data, timeout=10):
        deadline = time.monotonic() + timeout
        stream = self.process.stdin
        observation.require(stream is not None and not stream.closed, 'MCP_input_still_owned')
        view = memoryview(data)
        while view:
            remaining = deadline - time.monotonic()
            if remaining <= 0 or not select.select([], [stream], [], remaining)[1]:
                raise observation.Failure('bounded_MCP_input_write_' + self.label)
            try:
                count = os.write(stream.fileno(), view)
            except BlockingIOError:
                continue
            observation.require(count > 0, 'MCP_input_write_progress')
            view = view[count:]

    def receive(self, timeout=12):
        deadline = time.monotonic() + timeout
        while True:
            newline = self.buffer.find(b'\n')
            if newline >= 0:
                line = bytes(self.buffer[:newline])
                del self.buffer[:newline + 1]
                observation.require(len(line) <= FRAME_LIMIT, 'bounded_MCP_response')
                value = json.loads(line)
                observation.require(isinstance(value, dict), 'MCP_response_object')
                return value
            observation.require(len(self.buffer) <= FRAME_LIMIT, 'bounded_MCP_response_frame')
            remaining = deadline - time.monotonic()
            if remaining <= 0 or not select.select([self.process.stdout], [], [], remaining)[0]:
                raise TimeoutError('MCP response deadline')
            try:
                data = os.read(self.process.stdout.fileno(), 65536)
            except BlockingIOError:
                continue
            if not data:
                observation.require(not self.buffer, 'MCP_no_truncated_terminal_frame')
                raise EOFError('MCP delivery unavailable')
            self.buffer.extend(data)

    def start(self, tool_name, arguments, identity=None):
        identity = identity if identity is not None else 'mcp-' + secrets.token_hex(12)
        observation.require(identity not in self.pending, 'distinct_owned_MCP_ID')
        call = dict(id=identity, started=time.monotonic(), operation=tool_name,
                    arguments=arguments, domain_request_id=None)
        self.pending[identity] = call
        self.send(dict(jsonrpc='2.0', id=identity, method='tools/call',
                       params=dict(name=tool_name, arguments=arguments)))
        return call

    def finish(self, call, label, delivery_optional=False):
        limit = 10 if call['operation'] == 'edit_script' else 5
        remaining = limit - (time.monotonic() - call['started'])
        observation.require(remaining > 0, 'MCP_original_clock_remaining_' + label)
        try:
            response = self.receive(timeout=remaining)
        except EOFError:
            if not delivery_optional:
                raise
            elapsed = time.monotonic() - call['started']
            self.harness.case(label, operation=call['operation'], elapsed_seconds=elapsed,
                              mcp_id=call['id'], request_id=call['domain_request_id'],
                              delivery='unavailable', application='unavailable',
                              mcp_acceptance=True, real_client_acceptance=False)
            self.pending.pop(call['id'], None)
            return None
        except TimeoutError as error:
            raise observation.Failure('MCP_consumed_output_deadline_' + label) from error
        elapsed = time.monotonic() - call['started']
        observation.require(elapsed <= limit and response.get('id') == call['id'] and
                            'error' not in response, 'MCP_bounded_correlated_tool_delivery_' + label)
        carrier = response.get('result')
        observation.require(isinstance(carrier, dict), 'MCP_tool_result_carrier_' + label)
        content = carrier.get('structuredContent')
        observation.require(isinstance(content, dict) and set(content) ==
                            {'schema_version', 'operation', 'request_id', 'result', 'error'} and
                            content['schema_version'] == 1 and content['operation'] == call['operation'] and
                            ((content['result'] is None) != (content['error'] is None)),
                            'MCP_authoritative_typed_envelope_' + label)
        request_id = content['request_id']
        observation.require(isinstance(request_id, str) and
                            re.fullmatch(r'[A-Za-z0-9_-]{1,64}', request_id) is not None and
                            request_id != call['id'], 'MCP_distinct_domain_correlation_' + label)
        if call['domain_request_id'] is not None:
            observation.require(request_id == call['domain_request_id'], 'MCP_barrier_domain_owner_' + label)
        if content['error'] is not None:
            observation.require(carrier.get('isError') is True, 'MCP_adapter_error_summary_' + label)
        raw = json.dumps(response, ensure_ascii=False).encode()
        observation.require(all(secret not in raw for secret in self.harness.secrets),
                            'MCP_no_credential_disclosure_' + label)
        if call['operation'] == 'edit_script':
            observation.require(all(marker not in raw for marker in self.harness.source_markers),
                                'MCP_source_free_edit_evidence_' + label)
        observation.json_file(self.harness.artifacts / (label + '.json'), response)
        self.harness.case(label, operation=call['operation'], elapsed_seconds=elapsed,
                          evidence=label + '.json', mcp_id=call['id'], request_id=request_id,
                          delivery='delivered', mcp_acceptance=True, real_client_acceptance=False)
        self.harness.last_workflow_request = request_id
        if call['operation'] == 'edit_script' and content['result'] is not None:
            self.harness.last_workflow_edit = content['result']
        self.pending.pop(call['id'], None)
        return content

    def call(self, tool_name, arguments, label):
        return self.finish(self.start(tool_name, arguments), label)

    def cancel(self, call):
        self.send(dict(jsonrpc='2.0', method='notifications/cancelled',
                       params=dict(requestId=call['id'], reason='owned acceptance cancellation')))

    def eof(self):
        if self.process.stdin is not None and not self.process.stdin.closed:
            self.process.stdin.close()

    def wait_exit(self, bound=12):
        try:
            return self.process.wait(timeout=bound)
        except subprocess.TimeoutExpired as error:
            raise observation.Failure('MCP_owned_cleanup_deadline_' + self.label) from error
        finally:
            if self.process.poll() is not None and self.process in self.harness.open_processes:
                self.harness.open_processes.remove(self.process)

    def __exit__(self, exc_type, exc, traceback):
        if self.process is None:
            return
        try:
            self.eof()
            try:
                self.wait_exit()
            except observation.Failure:
                self.process.kill()
                self.process.wait(timeout=2)
                if self.process in self.harness.open_processes:
                    self.harness.open_processes.remove(self.process)
                raise
        finally:
            self.process.stdout.close()
            self.diagnostics.close()
        diagnostic = self.stderr_path.read_bytes()
        observation.require(all(marker not in diagnostic for marker in self.harness.source_markers) and
                            all(secret not in diagnostic for secret in self.harness.secrets),
                            'MCP_source_free_incidental_diagnostics_' + self.label)


class McpAdversarialMixin:
    def mcp_read(self, peer, project, editor, descriptor, label, revision=True, source=None):
        before, disks = self.state(editor, project)
        content = peer.call('read_script', selectors(project, descriptor), label)
        result = content['result']
        observation.require(content['error'] is None and isinstance(result, dict) and
                            set(result) == {'source', 'revision', 'state'}, 'MCP_public_read_' + label)
        token = result['revision']
        observation.require((isinstance(token, str) and re.fullmatch(r'sr1:[0-9a-f]{64}', token))
                            if revision else token is None, 'MCP_read_revision_eligibility_' + label)
        if source is not None:
            observation.require(result['source'] == source, 'MCP_exact_read_source_' + label)
        after, now = self.state(editor, project)
        observation.require(source_free(before, disks) == source_free(after, now),
                            'MCP_read_independent_no_effect_' + label)
        self.record_witness(label, before, disks, after, now, editor)
        return result

    def mcp_edit(self, peer, project, descriptor, revision, replacement, label, expected):
        content = peer.call('edit_script', dict(selectors(project, descriptor), revision=revision,
                                              replacement_source=replacement), label)
        return self.mcp_review_edit(content, label, expected)

    def mcp_review_edit(self, content, label, expected):
        result = content['result']
        observation.require(content['error'] is None and isinstance(result, dict) and
                            set(result) == {'mode', 'outcome'} and
                            result['mode'] in ('open', 'closed', 'undetermined'), 'MCP_public_edit_' + label)
        outcome = result['outcome']
        allowed = {expected} if isinstance(expected, str) else set(expected)
        observation.require(outcome['outcome'] in allowed and outcome['application'] in
                            ('not_applied', 'applied', 'partly_applied', 'unknown'),
                            'MCP_expected_effect_outcome_' + label)
        relations = {'verified_changed': 'applied', 'verified_unchanged': 'not_applied',
                     'refused': 'not_applied', 'application_unknown': 'unknown'}
        if outcome['outcome'] in relations:
            observation.require(outcome['application'] == relations[outcome['outcome']],
                                'MCP_outcome_application_relation_' + label)
        effects = outcome.get('effects')
        if effects and any(effects.get(key) for key in ('resource_changed', 'written_bytes', 'truncated')):
            observation.require(outcome['application'] in ('applied', 'partly_applied'),
                                'MCP_sticky_known_effects_' + label)
        if outcome['outcome'] == 'refused':
            observation.require(outcome.get('reason') is not None, 'MCP_specific_refusal_cause_' + label)
        observation.require((isinstance(outcome.get('safe_next_action'), str) if result['mode'] == 'open' else
                             isinstance(outcome.get('next_action'), dict) and
                             isinstance(outcome['next_action'].get('kind'), str)), 'MCP_actionable_result_' + label)
        self.last_workflow_edit = result
        self.cases[-1].update(mode=result['mode'], outcome=outcome['outcome'],
                              application=outcome['application'], reason=outcome.get('reason'))
        return result

    def wait_mcp_barrier(self, editor, stage, peer, call, acquisition=False):
        def reached():
            observation.require(peer.process.poll() is None, 'MCP_alive_at_barrier_' + stage)
            path = editor['control'] / 'closed-event.json'
            if not path.is_file():
                return None
            event = json.loads(path.read_text())
            request_id = event.get('request_id')
            if event.get('stage') != stage or not isinstance(request_id, str) or not request_id:
                return None
            observation.require(len(request_id) <= 64 and request_id != call['id'],
                                'MCP_barrier_separate_domain_ID')
            if not acquisition:
                previous = call['domain_request_id']
                observation.require(previous is None or previous == request_id, 'MCP_immutable_domain_owner')
                call['domain_request_id'] = request_id
            return event
        remaining = (10 if call['operation'] == 'edit_script' else 5) - (time.monotonic() - call['started'])
        observation.require(remaining > 0, 'MCP_original_clock_at_barrier')
        return observation.wait_for(reached, 'actual_MCP_barrier_' + stage, timeout=min(8, remaining))

    def mcp_survivor_read(self, peer, project, editor, descriptor, label):
        before, disks = self.state(editor, project)
        content = peer.call('read_script', selectors(project, descriptor), label)
        after, now = self.state(editor, project)
        observation.require(source_free(before, disks) == source_free(after, now),
                            'MCP_recovery_read_does_not_repair_' + label)
        self.record_witness(label, before, disks, after, now, editor)
        return content

    def assert_mcp_buffer_prefix_survivor(self, before, disks, after, now, label):
        # After an applied B edit, ordinary ScriptEditor validation may copy that
        # already-visible source into R even though the native R stage never ran.
        # Permit only that transition, never a D/B/history/dirty/identity change.
        left, right = source_free(before, disks), source_free(after, now)
        old_docs, new_docs = left['documents'], right['documents']
        observation.require(
            after['cached_R'] in (before['cached_R'], before['target']['B']) and
            after['target']['R'] == after['cached_R'] and
            old_docs.keys() == new_docs.keys() and
            all({k: v for k, v in old_docs[path].items() if path != TARGET or k != 'R'} ==
                {k: v for k, v in new_docs[path].items() if path != TARGET or k != 'R'} for path in old_docs) and
            {k: v for k, v in left.items() if k not in ('documents', 'cached_source_sha256')} ==
            {k: v for k, v in right.items() if k not in ('documents', 'cached_source_sha256')},
            'MCP_only_observed_editor_B_to_R_propagation_' + label)
        self.cases[-1]['observed_editor_resource_propagation'] = (
            self.cases[-1].get('observed_editor_resource_propagation', False) or
            before['cached_R'] != after['cached_R'])
