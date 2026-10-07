"""Prepared workflow contracts; no VM/product execution."""
import copy
import json
import queue
import socket
from pathlib import Path
import tempfile
import threading
import unittest
from unittest import mock

import mcp_lifecycle_acceptance as lifecycle
import run_in_vm as host
import vm_guest as guest


def sequence_records(name='cached'):
    source = '' if name.startswith('empty') and name != 'empty_desired' else lifecycle.SAFE + '# localized deletion witness\n'
    if name.startswith('unicode'):
        source = lifecycle.SAFE.replace('\treturn 47', '\t# café 雪 𐐀\n\treturn 47')
    target = dict(name=name, initial_source=source, steps=lifecycle.exact_steps(name, source))
    calls = [dict(name='discover_scripts', arguments={}, structuredContent={'result': {
        'inventory': {'entries': [lifecycle.TARGET]}}})]
    serial = 1
    def read():
        calls.append(dict(name='read_script', arguments={}, structuredContent={'result': dict(
            source=source, revision='sr1:' + format(serial, '064x'),
            state={'document': {'lifecycle': 'open' if lifecycle.lifecycle_profile(name) == 'open' else 'closed'}})}))
    read()
    read()
    for step in target['steps']:
        changed = source != step['source']
        calls.append(dict(name='edit_script', arguments=dict(
            revision='sr1:' + format(serial, '064x'), old_string=step['old_string'], new_string=step['new_string']),
            structuredContent={'result': {'outcome': {
                'outcome': 'verified_changed' if changed else 'verified_unchanged'}}}))
        source = step['source']
        serial += int(changed)
        read()
    target.update(calls=calls, desired=source)
    return target, calls

class PreparedWorkflowTests(unittest.TestCase):
    def test_fixed_commands_accept_no_endpoint_or_command(self):
        for parser in (host._parser(), guest._parser()):
            for command in ('mcp-stdio', 'prepare-durability', 'finalize-mcp'):
                args = parser.parse_args([command, '--run-id', 'owned-run'])
                self.assertEqual(args.run_id, 'owned-run')
                with self.assertRaises(SystemExit):
                    parser.parse_args([command, '--run-id', 'owned-run', '--command', 'sh'])


    def test_client_claim_without_tool_evidence_is_not_visibility(self):
        with self.assertRaises(ValueError):
            lifecycle.review_client_events([{'type': 'message_end', 'text': 'Everything passed'}], [])

    def test_receipt_excludes_private_session_credentials_and_source(self):
        descriptor = {'session_id': 'a' * 32, 'token': 'PRIVATE_TOKEN_SENTINEL',
                      'source': 'PRIVATE_SOURCE_SENTINEL', 'port': 45678}
        receipt = lifecycle.target_receipt('open', Path('/guest/owned'), descriptor)
        self.assertEqual(receipt['session_id'], descriptor['session_id'])
        self.assertNotIn('PRIVATE_', json.dumps(receipt))
        self.assertNotIn('port', receipt)


    def test_relay_eof_drains_owned_process_without_fixture_teardown(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            product = root / 'owned-consumer'
            drained = root / 'drained'
            product.write_text('#!/usr/bin/env python3\nimport sys\nfrom pathlib import Path\n'
                               'sys.stdin.buffer.read()\nPath(' + repr(str(drained)) + ').write_text("EOF")\n')
            product.chmod(0o700)
            harness = lifecycle.McpLifecycleMixin()
            harness.args = mock.Mock(mcp_server=product)
            harness.registry = root / 'registry'
            harness.artifacts = root
            harness.cleanup = mock.Mock()
            client, owner = socket.socketpair()
            try:
                client.shutdown(socket.SHUT_WR)
                harness.relay_connection(owner)
                self.assertEqual(drained.read_text(), 'EOF')
                harness.cleanup.assert_not_called()
            finally:
                client.close()
                owner.close()

    def test_controls_drain_idle_relay_without_waiting_for_client_eof(self):
        for command in (b'F', b'P'):
            with self.subTest(command=command):
                self.check_control_drains_idle_relay(command)

    def check_control_drains_idle_relay(self, command):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            product = root / 'owned-consumer'
            drained = root / 'drained'
            product.write_text('#!/usr/bin/env python3\nimport sys\nfrom pathlib import Path\n'
                               'sys.stdin.buffer.read()\nPath(' + repr(str(drained)) + ').write_text("EOF")\n')
            product.chmod(0o700)
            harness = lifecycle.McpLifecycleMixin()
            harness.args = mock.Mock(mcp_server=product)
            harness.registry = root / 'registry'
            harness.artifacts = root
            harness.cleanup = mock.Mock()
            result = queue.Queue()
            client, owner = socket.socketpair()
            with socket.socket(socket.AF_UNIX) as listener, socket.socket(socket.AF_UNIX) as control:
                listener.bind(str(root / 'control'))
                listener.listen(1)
                control.settimeout(3)
                control.connect(str(root / 'control'))
                control.sendall(command)
                def relay():
                    try:
                        result.put(harness.relay_connection(owner, listener))
                    except BaseException as error:
                        result.put(error)
                thread = threading.Thread(target=relay)
                thread.start()
                try:
                    handed = result.get(timeout=3)
                    if isinstance(handed, BaseException):
                        raise handed
                    finalizer, received_command = handed
                    with finalizer:
                        self.assertEqual(received_command, command)
                        self.assertEqual(drained.read_text(), 'EOF')
                        harness.cleanup.assert_not_called()
                        finalizer.sendall(b'OK\n')
                        self.assertEqual(control.recv(16), b'OK\n')
                finally:
                    client.shutdown(socket.SHUT_RDWR)
                    thread.join(timeout=3)
                    client.close()
                    owner.close()
                self.assertFalse(thread.is_alive())

    def test_long_run_identity_keeps_fixed_control_socket_within_unix_limit(self):
        run = Path('/Users/admin/.godot-agent-kit-vm/runs') / ('x' * 80)
        self.assertLess(len(str(lifecycle.control_path(run)).encode()), 104)
        self.assertNotEqual(lifecycle.control_path(run), lifecycle.control_path(run.with_name('other')))

    def test_consumer_accepts_complete_exact_sequence_on_each_lifecycle(self):
        for name in lifecycle.PROFILE_GROUPS['workflow'] + lifecycle.PROFILE_GROUPS['sources'] + ('bound',):
            with self.subTest(name=name):
                target, calls = sequence_records(name)
                self.assertEqual(lifecycle.review_exact_sequence(target, calls), target['desired'])

    def test_consumer_rejects_wrong_missing_and_reused_revision(self):
        target, original = sequence_records()
        for revision in (None, 'sr1:' + 'f' * 64):
            calls = copy.deepcopy(original)
            calls[3]['arguments']['revision'] = revision
            with self.assertRaisesRegex(ValueError, 'read revision'):
                lifecycle.review_exact_sequence(target, calls)
        calls = copy.deepcopy(original)
        calls[7]['arguments']['revision'] = calls[3]['arguments']['revision']
        with self.assertRaisesRegex(ValueError, 'read revision'):
            lifecycle.review_exact_sequence(target, calls)
        calls = copy.deepcopy(original)
        calls[6]['structuredContent']['result']['revision'] = calls[4]['structuredContent']['result']['revision']
        with self.assertRaisesRegex(ValueError, 'revision contradicts'):
            lifecycle.review_exact_sequence(target, calls)

    def test_consumer_rejects_edit_attempt_without_operation_evidence(self):
        target, calls = sequence_records()
        calls[3]['structuredContent'] = {'result': None, 'error': {'code': 'invalid_arguments'}}
        with self.assertRaisesRegex(ValueError, 'checked operation result'):
            lifecycle.review_exact_sequence(target, calls)

    def test_consumer_rejects_whole_source_substitute_for_localized_change(self):
        target, calls = sequence_records()
        edit = calls[5]
        edit['arguments'].update(old_string=target['initial_source'],
            new_string=target['initial_source'].replace('return 47', 'return 83'))
        with self.assertRaisesRegex(ValueError, 'exact intent'):
            lifecycle.review_exact_sequence(target, calls)

    def test_consumer_rejects_legacy_mixed_and_missing_pair(self):
        target, original = sequence_records()
        for mutation in ('legacy', 'mixed', 'missing'):
            calls = copy.deepcopy(original)
            args = calls[3]['arguments']
            if mutation != 'missing':
                args['replacement_source'] = target['initial_source']
            if mutation != 'mixed':
                del args['old_string']
            with self.assertRaisesRegex(ValueError, 'exact intent'):
                lifecycle.review_exact_sequence(target, calls)

    def test_consumer_rejects_incorrect_complete_source_lifecycle_and_null_revision(self):
        target, original = sequence_records()
        for field, value in (('source', lifecycle.CHANGED + '# LOST_OUTSIDE_SPAN\n'),
                             ('revision', None), ('state', {'document': {'lifecycle': 'open'}})):
            calls = copy.deepcopy(original)
            calls[-1]['structuredContent']['result'][field] = value
            with self.assertRaisesRegex(ValueError, 'complete source'):
                lifecycle.review_exact_sequence(target, calls)

    def test_consumer_rejects_missing_post_edit_read(self):
        target, calls = sequence_records()
        with self.assertRaisesRegex(ValueError, 'fresh result read'):
            lifecycle.review_exact_sequence(target, calls[:-1])

    def test_consumer_accepts_equivalent_unique_localized_context(self):
        target, calls = sequence_records()
        calls[5]['arguments'].update(old_string='\treturn 47\n', new_string='\treturn 83\n')
        self.assertEqual(lifecycle.review_exact_sequence(target, calls), target['desired'])

    def test_consumer_rejects_claimed_unrecorded_call(self):
        call = dict(name='read_script', arguments={'project_root': '/owned'},
                    structuredContent={'request_id': 'observed', 'result': None})
        events = [dict(kind='model_visible_tool_result', content=call['structuredContent']),
                  dict(kind='tool_call', name='edit_script', arguments={'revision': 'invented'})]
        with self.assertRaisesRegex(ValueError, 'missing server record'):
            lifecycle.review_client_events(events, [call])

    def test_failed_primary_validation_cannot_open_or_mark_durability(self):
        harness = lifecycle.McpLifecycleMixin()
        harness.targets = {'owned': {'name': 'cached', 'calls': []}}
        harness.public_open = mock.Mock()
        with self.assertRaises(lifecycle.observation.Failure):
            harness.prepare_durability()
        harness.public_open.assert_not_called()
        self.assertNotIn('durability_start', harness.targets['owned'])

    def test_durability_requires_post_open_client_read_before_native_checks(self):
        harness = lifecycle.McpLifecycleMixin()
        target, calls = sequence_records()
        target['durability_start'] = len(calls)
        harness.targets = {'owned': target}
        harness.native_action = mock.Mock()
        with self.assertRaises(lifecycle.observation.Failure):
            harness.finalize_targets()
        harness.native_action.assert_not_called()
        calls.append(dict(name='read_script', arguments={}, structuredContent={
            'result': None, 'error': {'category': 'admission', 'code': 'server_busy'}}))
        with self.assertRaises(lifecycle.observation.Failure):
            harness.finalize_targets()
        harness.native_action.assert_not_called()
        target.update(project=Path('/owned'), editor=object())
        later_read = dict(name='read_script', arguments={}, structuredContent={'result': dict(
            source=lifecycle.CHANGED, revision='sr1:' + 'f' * 64,
            state={'document': {'lifecycle': 'open'}})})
        calls.append(later_read)
        harness.native_action.side_effect = RuntimeError('reached_independent_durability')
        with self.assertRaisesRegex(RuntimeError, 'reached_independent_durability'):
            harness.finalize_targets()
