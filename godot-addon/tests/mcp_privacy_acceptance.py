"""Actual MCP disclosure boundaries and installed-production export witnesses.

Raw responses stay in the private artifact directory. These checks grant source
only to the requested read's source authorities, never to a text summary or error.
Existing focused denial scenarios supply the independent survivor/history checks.
"""
from __future__ import annotations

import json
import os
import subprocess
from contextlib import ExitStack

import run_observation as observation
from closed_script_acceptance import CHANGED
from mcp_peer import McpAdversarialMixin, McpPeer, selectors
from run_script_close import TARGET, SAFE
from run_script_edit import sha


WORKER_FLAGS = (
    '--internal-observation-worker', '--internal-discovery-worker',
    '--internal-edit-worker', '--internal-closed-edit-worker',
    '--internal-closed-acquisition-worker', '--internal-stock-validation-worker',
)
PRIVACY_GROUPS = {
    'privacy-authorized': '_privacy_authorized_and_errors',
    'privacy-selection': 'mcp_preservation_selection_privacy',
    'privacy-interrupted': 'mcp_interruption_denial',
    'privacy-production-exports': 'native_export',
}
RESULT = ('result', 'structuredContent', 'result')
READ_TEXT_PATHS = (RESULT + ('source',),) + tuple(
    RESULT + ('state', 'sources', authority, 'text')
    for authority in ('disk', 'loaded_resource', 'editor_buffer'))


def assert_disclosure(response, label, *, secrets=(), sources=(), inventory=(), grants=None):
    """Check exact JSON locations, including keys; do not redact or rewrite evidence.

    ``grants`` maps a sentinel to exact permitted leaf paths. It is intentionally
    not a subtree exemption: a permitted source copied into diagnostics still fails.
    Inventory locations are the actual authorized entries returned by discovery.
    """
    grants = grants or {}
    protected = tuple(dict.fromkeys((*secrets, *sources, *inventory)))

    def visit(value, path):
        if isinstance(value, dict):
            for key, child in value.items():
                visit(key, path + ('<key>',))
                visit(child, path + (key,))
        elif isinstance(value, list):
            for index, child in enumerate(value):
                visit(child, path + (index,))
        elif isinstance(value, str):
            for sentinel in protected:
                text = sentinel.decode('utf-8') if isinstance(sentinel, bytes) else sentinel
                if text and text in value:
                    observation.require(sentinel not in secrets and path in grants.get(sentinel, ()),
                                        'MCP_no_incidental_disclosure_' + label)
    visit(response, ())


class _PrivacyPeer(McpPeer):
    """Retain initialization as well as tool/error responses without rewriting."""

    def __init__(self, harness, label):
        super().__init__(harness, label)
        self.received = []

    def receive(self, timeout=12):
        response = super().receive(timeout)
        observation.json_file(self.harness.artifacts /
                              ('privacy-frame-' + str(len(self.received)) + '.json'), response)
        self.received.append(response)
        return response


class McpPrivacyAcceptanceMixin(McpAdversarialMixin):
    def _privacy_response(self, label, **policy):
        response = json.loads((self.artifacts / (label + '.json')).read_text())
        assert_disclosure(response, label, secrets=self.secrets, **policy)
        return response

    def _privacy_rpc(self, peer, method, params, label, *, sources, inventory):
        peer.send(dict(jsonrpc='2.0', id=label, method=method, params=params))
        response = peer.receive(timeout=5)
        observation.json_file(self.artifacts / (label + '.json'), response)
        observation.require(response.get('id') == label, 'privacy_raw_RPC_correlation_' + label)
        assert_disclosure(response, label, secrets=self.secrets, sources=sources, inventory=inventory)
        self.case(label, operation=method, evidence=label + '.json',
                  mcp_acceptance=True, real_client_acceptance=False)
        return response

    def _privacy_entrypoints(self, sources, inventory):
        # Live registry and source sentinels are present, but public help and private
        # workers invoked over ordinary pipes must not select or expose any owner.
        payload = json.dumps(dict(private_source=sources[0], private_inventory=inventory[0],
                                  private_credential=next(iter(self.secrets)).decode('utf-8'))).encode()
        commands = [('help', ['--help'], 0), ('version', ['--version'], 0),
                    ('startup-error', ['--registry', sources[0]], 2)]
        commands.extend(('worker-' + flag.removeprefix('--internal-'), [flag], 1)
                        for flag in WORKER_FLAGS)
        before = sorted((str(path), observation.digest(path)) for path in self.registry.rglob('*.json'))
        for label, arguments, expected in commands:
            result = subprocess.run([str(self.args.mcp_server), *arguments], input=payload,
                                    capture_output=True, timeout=12,
                                    env=dict(os.environ, RUST_LOG='trace'))
            # Preserve bytes, including unexpected output, before asserting privacy.
            (self.artifacts / ('privacy-' + label + '-stdout.log')).write_bytes(result.stdout)
            (self.artifacts / ('privacy-' + label + '-stderr.log')).write_bytes(result.stderr)
            observation.require(result.returncode == expected and not result.stdout,
                                'privacy_entrypoint_not_MCP_or_worker_authority_' + label)
            assert_disclosure(result.stderr.decode('utf-8'), label, secrets=self.secrets,
                              sources=sources, inventory=inventory)
            self.case('privacy-' + label, exit_code=result.returncode,
                      stdout_evidence='privacy-' + label + '-stdout.log',
                      stderr_evidence='privacy-' + label + '-stderr.log')
        observation.require(before == sorted((str(path), observation.digest(path))
                            for path in self.registry.rglob('*.json')),
                            'privacy_help_worker_error_never_change_live_registry')

    def _privacy_authorized_and_errors(self):
        selected = SAFE + '# MCP_PRIVACY_SELECTED_SOURCE\n'
        proposed = CHANGED + '# MCP_PRIVACY_PROPOSED_SOURCE\n'
        unselected = SAFE + '# MCP_PRIVACY_UNSELECTED_SOURCE\n'
        self.source_markers.update(marker.encode() for marker in
                                   ('MCP_PRIVACY_SELECTED_SOURCE', 'MCP_PRIVACY_PROPOSED_SOURCE',
                                    'MCP_PRIVACY_UNSELECTED_SOURCE'))
        selected_inventory = 'res://scripts/MCP_PRIVACY_SELECTED_INVENTORY.gd'
        with ExitStack() as stack:
            project, editor, descriptor = stack.enter_context(self.closed_fixture(
                'mcp-privacy-selected', cached=True, source=selected,
                setup=lambda root: (root / selected_inventory.removeprefix('res://')).write_text(SAFE)))
            other_project, other_editor, _ = stack.enter_context(self.closed_fixture(
                'mcp-privacy-unselected', cached=True, source=unselected))
            private_path = other_project / 'scripts/MCP_PRIVACY_INVENTORY_ONLY.gd'
            private_path.write_text(unselected)
            sources = (selected, proposed, unselected, sha(selected), sha(proposed), sha(unselected),
                       'MCP_PRIVACY_SELECTED_SOURCE', 'MCP_PRIVACY_PROPOSED_SOURCE',
                       'MCP_PRIVACY_UNSELECTED_SOURCE')
            inventory = (str(other_project), 'MCP_PRIVACY_INVENTORY_ONLY', 'MCP_PRIVACY_SELECTED_INVENTORY')
            marker = project / 'scripts/.gdignore'
            if marker.exists():
                self.present_editor(editor)
                scanned = self.native_action(editor, 'native_edit_scan')
                observation.require(scanned['settled'], 'privacy_initial_index_settled')
                marker.unlink()
            witnesses = [(owner, root, *self.state(owner, root)) for owner, root in
                         ((editor, project), (other_editor, other_project))]
            self.close_action(editor, 'closed_timeline_start')
            stack.callback(lambda: observation.json_file(
                self.artifacts / 'diagnostic-witness.json',
                self.close_action(editor, 'closed_timeline_stop')))
            self._privacy_entrypoints(sources, inventory)
            with _PrivacyPeer(self, 'privacy-authorized') as peer:
                assert_disclosure(peer.received[0], 'privacy-initialize', secrets=self.secrets,
                                  sources=sources, inventory=inventory)
                listed = self._privacy_rpc(peer, 'tools/list', {}, 'privacy-tools-list',
                                          sources=sources, inventory=inventory)
                observation.require({tool['name'] for tool in listed['result']['tools']} ==
                                    {'discover_scripts', 'read_script', 'edit_script'},
                                    'privacy_exact_supported_static_tools')
                basis = self.mcp_read(peer, project, editor, descriptor, 'privacy-selected-read', source=selected)
                grants = {value: READ_TEXT_PATHS for value in (selected, 'MCP_PRIVACY_SELECTED_SOURCE')}
                self._privacy_response('privacy-selected-read', sources=sources, inventory=inventory, grants=grants)
                args = selectors(project, descriptor)
                args.pop('script_path')
                discovered = peer.call('discover_scripts', args, 'privacy-selected-discovery')
                observation.require(discovered['error'] is None and
                                    TARGET in discovered['result']['inventory']['entries'],
                                    'privacy_requested_inventory_is_actually_returned')
                entries = discovered['result']['inventory']['entries']
                observation.require(selected_inventory in entries, 'privacy_selected_inventory_sentinel_returned')
                self._privacy_response('privacy-selected-discovery', sources=sources, inventory=inventory,
                    grants={'MCP_PRIVACY_SELECTED_INVENTORY':
                            (RESULT + ('inventory', 'entries', entries.index(selected_inventory)),)})
                # Invalid params, unknown methods/tools, and unknown fields must not
                # echo source, credentials or candidate inventory from the request.
                for method, params, label in (
                    ('unsupported', {'private_source': unselected}, 'privacy-method-error'),
                    ('tools/call', {'name': 'unsupported', 'arguments': {'private_source': proposed}},
                     'privacy-tool-error'),
                ):
                    response = self._privacy_rpc(peer, method, params, label, sources=sources, inventory=inventory)
                    observation.require(response.get('error') is not None, 'privacy_actual_protocol_error_' + label)
                invalid = peer.call('read_script', dict(selectors(project, descriptor),
                                    private_source=proposed, private_inventory=inventory[1],
                                    private_credential=next(iter(self.secrets)).decode('utf-8')),
                                    'privacy-input-error')
                observation.require(invalid['error'] is not None and invalid['result'] is None,
                                    'privacy_actual_input_error_no_operation_result')
                self._privacy_response('privacy-input-error', sources=sources, inventory=inventory)
                for owner, root, before, disks in witnesses:
                    self.assert_no_effect('privacy-nonmutating-paths', root, owner, before, disks)
                self.present_editor(editor)
                before, disks = self.state(editor, project)
                self.mcp_edit(peer, project, descriptor, basis['revision'], proposed,
                              'privacy-authorized-edit', 'verified_changed')
                self._privacy_response('privacy-authorized-edit',
                                       sources=(selected, proposed, unselected,
                                                'MCP_PRIVACY_SELECTED_SOURCE', 'MCP_PRIVACY_PROPOSED_SOURCE',
                                                'MCP_PRIVACY_UNSELECTED_SOURCE', sha(unselected)), inventory=inventory)
                self.assert_closed_success('privacy-authorized-edit', project, editor, before, disks,
                                           proposed, changed=True)
                after_basis = self.mcp_read(peer, project, editor, descriptor,
                                            'privacy-authorized-survivor', source=proposed)
                observation.require(after_basis['revision'] != basis['revision'], 'privacy_fresh_authorized_revision')
                self._privacy_response('privacy-authorized-survivor', sources=sources, inventory=inventory,
                                       grants={value: READ_TEXT_PATHS for value in
                                               (proposed, 'MCP_PRIVACY_PROPOSED_SOURCE')})
                _, _, other_before, other_disks = witnesses[1]
                self.assert_no_effect('privacy-unselected-survives-edit', other_project, other_editor,
                                      other_before, other_disks)
            assert_disclosure(peer.stderr_path.read_text(), 'privacy-authorized-stderr',
                              secrets=self.secrets, sources=sources, inventory=inventory)
            observation.require((project / selected_inventory.removeprefix('res://')).read_text() == SAFE and
                                private_path.read_text() == unselected,
                                'privacy_inventory_sentinel_scripts_survive_all_calls')
            for path in self.registry.rglob('*.json'):
                assert_disclosure(json.loads(path.read_text()), 'privacy-registry', sources=sources,
                                  inventory=('MCP_PRIVACY_INVENTORY_ONLY', 'MCP_PRIVACY_SELECTED_INVENTORY'))
            assert_disclosure(json.dumps(self.summary), 'privacy-summary', secrets=self.secrets,
                              sources=(selected, proposed, unselected, 'MCP_PRIVACY_SELECTED_SOURCE',
                                       'MCP_PRIVACY_PROPOSED_SOURCE', 'MCP_PRIVACY_UNSELECTED_SOURCE'),
                              inventory=('MCP_PRIVACY_INVENTORY_ONLY', 'MCP_PRIVACY_SELECTED_INVENTORY'))

    def mcp_privacy_export(self):
        self.group('privacy-authorized-startup-help-worker-error', self._privacy_authorized_and_errors)
        # These focused existing scenarios keep their raw MCP responses and actual
        # retained-fd, native-history and survivor witnesses. Never replay the full
        # interruption family merely to prove sticky denial after a causal failure.
        self.group('privacy-denied-ambiguous-selection', self.mcp_preservation_selection_privacy)
        self.group('privacy-interrupted-sticky-denial', self.mcp_interruption_denial)
        # All owned editors have ended before exports assert an empty registry.
        # native_export verifies production provenance, actual zip/app packs and
        # gameplay runtimes in enabled, disabled and hook-only modes; fixture-fault
        # libraries remain confined to their separate owned projects.
        self.group('privacy-production-exports', self.native_export)
        exports = [case for case in self.cases if case['case'].startswith('export_boundary_')]
        observation.require({case['case'] for case in exports} ==
                            {'export_boundary_enabled', 'export_boundary_disabled', 'export_boundary_hook-only'} and
                            all(case.get('zip_sha256') and case.get('pack_sha256') and
                                case.get('runtime_exit') == 0 and case.get('observed_listeners') == 0
                                for case in exports), 'privacy_three_actual_production_artifact_witnesses')
        self.summary['privacy_export_verified'] = dict(
            authorized=True, denied=True, ambiguous=True, interrupted_sticky_denial=True,
            startup_help_worker_error=True, production_exports=['enabled', 'disabled', 'hook-only'],
            real_client_acceptance=False)
