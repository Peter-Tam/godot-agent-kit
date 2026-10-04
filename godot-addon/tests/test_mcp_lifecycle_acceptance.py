"""Prepared workflow contracts; no VM/product execution."""
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

    def test_discovery_first_prompt_has_no_known_locator(self):
        harness = lifecycle.McpLifecycleMixin()
        harness.targets = {name: dict(name=name, project=Path('/owned') / name,
            descriptor={'session_id': name}, desired='desired')
            for name in lifecycle.PROFILE_GROUPS['workflow']}
        prompt = harness.prepared_prompt()
        self.assertNotIn(lifecycle.TARGET, prompt)

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
        def read(source, revision):
            return dict(name='read_script', structuredContent={'result': dict(source=source, revision=revision)})
        def edit(outcome):
            return dict(name='edit_script', structuredContent={'result': {'outcome': {'outcome': outcome}}})
        calls = [dict(name='discover_scripts'), read('old', 'r1'), read('old', 'r1'),
                 edit('verified_unchanged'), edit('verified_changed'), read(lifecycle.CHANGED, 'r2')]
        target = dict(name='cached', calls=calls, desired=lifecycle.CHANGED, durability_start=len(calls))
        harness.targets = {'owned': target}
        harness.native_action = mock.Mock()
        with self.assertRaises(lifecycle.observation.Failure):
            harness.finalize_targets()
        harness.native_action.assert_not_called()
