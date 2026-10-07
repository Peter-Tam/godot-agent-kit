"""Disclosure authorization and actual-export completion regression witnesses."""
import copy
import json
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest
from unittest import mock

import mcp_privacy_acceptance as privacy
from mcp_preservation_acceptance import McpPreservationAcceptanceMixin
from run_script_close import TARGET, CURRENT, SAFE
from run_script_edit import sha


class DisclosureTests(unittest.TestCase):
    def read_response(self):
        return {'result': {'content': [{'type': 'text', 'text': 'Read complete'}],
                           'structuredContent': {'result': {
                               'source': '# SELECTED_SOURCE',
                               'state': {'sources': {'disk': {'text': '# SELECTED_SOURCE'}},
                                         'diagnostics': []}}}}}

    def assert_selected(self, response):
        privacy.assert_disclosure(response, 'read', secrets=(b'credential-secret',),
                                  sources=('SELECTED_SOURCE', 'OTHER_SOURCE', 'other-source-digest'),
                                  inventory=('OTHER_INVENTORY',),
                                  grants={'SELECTED_SOURCE': privacy.READ_TEXT_PATHS})

    def test_requested_source_authorities_are_permitted_without_mutating_response(self):
        response = self.read_response()
        before = copy.deepcopy(response)
        self.assert_selected(response)
        self.assertEqual(response, before)

    def test_authorized_source_copied_to_summary_error_or_diagnostic_is_incidental(self):
        for location in ('summary', 'diagnostic', 'error', 'key'):
            response = self.read_response()
            if location == 'summary':
                response['result']['content'][0]['text'] += ' SELECTED_SOURCE'
            elif location == 'diagnostic':
                response['result']['structuredContent']['result']['state']['diagnostics'] = [
                    {'message': 'SELECTED_SOURCE'}]
            elif location == 'error':
                response['error'] = {'message': 'SELECTED_SOURCE'}
            else:
                response['SELECTED_SOURCE'] = None
            with self.subTest(location=location), self.assertRaises(privacy.observation.Failure):
                self.assert_selected(response)

    def test_other_source_digest_and_credentials_are_forbidden_even_in_read_source(self):
        for sentinel in ('OTHER_SOURCE', 'other-source-digest', 'credential-secret'):
            response = self.read_response()
            response['result']['structuredContent']['result']['source'] += sentinel
            with self.subTest(sentinel=sentinel), self.assertRaises(privacy.observation.Failure):
                self.assert_selected(response)

    def test_denied_read_cannot_reuse_authorized_source_grant(self):
        with self.assertRaises(privacy.observation.Failure):
            privacy.assert_disclosure(self.read_response(), 'denied', sources=('SELECTED_SOURCE',))

    def test_matching_error_cannot_disclose_either_exact_fragment(self):
        old, new = '# PRIVATE_OLD_FRAGMENT\n', '# PRIVATE_NEW_FRAGMENT\n'
        response = {'result': {'structuredContent': {
            'schema_version': 2, 'operation': 'edit_script', 'request_id': 'owned', 'error': None,
            'result': {'mode': 'closed', 'outcome': {
                'outcome': 'refused', 'application': 'not_applied', 'stage': 'matching', 'reason': 'no_match'}}},
            'content': [{'type': 'text', 'text': 'Read again.'}], 'isError': True}}
        privacy.assert_disclosure(response, 'matching', sources=(old, new))
        for fragment in (old, new):
            changed = copy.deepcopy(response)
            changed['result']['structuredContent']['result']['outcome']['diagnostic'] = fragment
            with self.subTest(fragment=fragment), self.assertRaises(privacy.observation.Failure):
                privacy.assert_disclosure(changed, 'matching', sources=(old, new))

    def edit_response(self, outcome='refused', reason='ambiguous_match'):
        return {'result': {'structuredContent': {
            'schema_version': 2, 'operation': 'edit_script', 'request_id': 'owned',
            'error': None, 'result': {'mode': 'closed', 'outcome': {
                'outcome': outcome, 'application': 'not_applied', 'reason': reason,
                'stage': 'matching', 'revision': 'sr1:' + 'a' * 64,
                'validation': {'status': 'valid'}, 'effects': {'written_bytes': False},
                'lifecycle': {'observed_final': 'closed'},
                'next_action': {'kind': 'fresh_read'}}}},
            'content': [{'type': 'text', 'text': 'Read again.'}], 'isError': True}}

    def test_matching_feedback_and_private_payloads_fail_even_without_source_sentinel(self):
        feedback = {
            'candidate_matches': [{'line': 7}], 'candidate_text': 'short excerpt',
            'candidates': [{'line': 7}], 'count': 2, 'matches': [3, 9],
            'match_count': 2, 'occurrence_count': 2, 'match_offset': 12,
            'offset': 12, 'offsets': [12, 20], 'snippet': 'short excerpt',
            'excerpt': 'short excerpt', 'raw_request': {'arguments': {}},
            'request_arguments': {}, 'endpoint': '/tmp/private.sock',
            'token': 'private-token', 'validator_stdout': 'engine diagnostics',
            'validator_stderr': 'engine diagnostics', 'validator_request': {},
            'debug': 'ExactEdit { old: <redacted> }', 'old_string': 'x',
            'new_string': 'y', 'replacement_source': 'x', 'derived_source': 'y',
        }
        for key, value in feedback.items():
            for location in ('outcome', 'nested', 'carrier'):
                response = self.edit_response()
                result = response['result']['structuredContent']['result']
                if location == 'outcome':
                    result['outcome'][key] = value
                elif location == 'nested':
                    result['outcome']['diagnostics'] = [{'details': {key: value}}]
                else:
                    response['result']['extra'] = {key: value}
                with self.subTest(key=key, location=location), self.assertRaises(privacy.observation.Failure):
                    privacy.assert_disclosure(response, 'matching')

    def test_fragments_derived_source_and_escaped_debug_fail_in_all_carriers(self):
        old, new = '# OLD_PRIVATE\n', '# NEW_PRIVATE\n'
        derived = 'extends RefCounted\n' + new
        for source in (old, new, derived):
            for location in ('nested', 'key', 'summary', 'error', 'debug'):
                response = self.edit_response()
                if location == 'nested':
                    response['result']['structuredContent']['result']['outcome']['diagnostics'] = [
                        {'details': {'text': source}}]
                elif location == 'key':
                    response['result']['structuredContent']['result']['outcome'][source] = None
                elif location == 'summary':
                    response['result']['content'][0]['text'] = source
                elif location == 'error':
                    response['result']['structuredContent']['error'] = {'message': source}
                else:
                    response['result']['content'][0]['text'] = 'ExactEdit ' + json.dumps(source)
                with self.subTest(source=source, location=location), self.assertRaises(privacy.observation.Failure):
                    privacy.assert_disclosure(response, 'matching', sources=(old, new, derived))

    def test_denied_late_failure_cannot_restore_hash_in_key_or_nested_evidence(self):
        source = 'extends RefCounted\n# DENIED_PRIVATE\n'
        for location in ('key', 'nested', 'summary'):
            response = self.edit_response('applied_unverified', 'mtime_restore_failed')
            outcome = response['result']['structuredContent']['result']['outcome']
            outcome.update(application='applied', revision=None, evidence=None)
            if location == 'key':
                outcome[sha(source)] = None
            elif location == 'nested':
                outcome['progress'] = [{'diagnostic': {'source_hash': sha(source)}}]
            else:
                response['result']['content'][0]['text'] = sha(source)
            with self.subTest(location=location), self.assertRaises(privacy.observation.Failure):
                privacy.assert_disclosure(response, 'denied-late', sources=(source, sha(source)))

    def test_json_rpc_error_rejects_escaped_raw_fragment_without_typed_envelope(self):
        fragment = '# PRIVATE_UNICODE_é\n'
        for escaped in (False, True):
            response = {'error': {'code': -32602, 'message': json.dumps(fragment, ensure_ascii=escaped)}}
            with self.subTest(escaped=escaped), self.assertRaises(privacy.observation.Failure):
                privacy.assert_disclosure(response, 'protocol-error', sources=(fragment,))

    def test_inventory_grant_is_one_entry_not_all_text_or_candidate_paths(self):
        response = {'result': {'structuredContent': {'result': {'inventory': {
            'entries': ['res://scripts/SELECTED_INVENTORY.gd']}}}}}
        policy = dict(inventory=('SELECTED_INVENTORY', 'OTHER_INVENTORY'),
                      grants={'SELECTED_INVENTORY': (privacy.RESULT + ('inventory', 'entries', 0),)})
        privacy.assert_disclosure(response, 'discover', **policy)
        for leak in ('SELECTED_INVENTORY', 'OTHER_INVENTORY'):
            changed = copy.deepcopy(response)
            changed['result']['content'] = [{'text': leak}]
            with self.subTest(leak=leak), self.assertRaises(privacy.observation.Failure):
                privacy.assert_disclosure(changed, 'discover', **policy)
        response['result']['structuredContent']['result']['inventory']['entries'].append('OTHER_INVENTORY')
        with self.assertRaises(privacy.observation.Failure):
            privacy.assert_disclosure(response, 'discover', **policy)

    def test_credentials_cannot_be_granted_by_any_location(self):
        with self.assertRaises(privacy.observation.Failure):
            privacy.assert_disclosure({'source': 'credential-secret'}, 'credential',
                secrets=(b'credential-secret',), grants={b'credential-secret': (('source',),)})

    def test_denial_scenario_rejects_disclosure_only_in_retained_text_carrier(self):
        with TemporaryDirectory() as directory:
            h = McpPreservationAcceptanceMixin()
            h.artifacts = Path(directory)
            h.secrets = {b'credential-secret'}
            h.source_markers = {b'PRIVATE_TARGET'}
            source = SAFE + '# PRIVATE_TARGET\n'
            body = dict(schema_version=2, operation='read_script', request_id='owned',
                        result=None, error={'category': 'selection', 'code': 'ambiguous_session'})
            witnesses = [({'documents': []}, {TARGET: {'text': source, 'sha256': sha(source)},
                                             CURRENT: {'text': SAFE, 'sha256': sha(SAFE)}})]
            for leak in ('PRIVATE_TARGET', sha(source), CURRENT):
                frame = dict(result=dict(structuredContent=body, content=[dict(type='text', text=leak)]))
                (h.artifacts / 'denied.json').write_text(json.dumps(frame))
                with self.subTest(leak=leak), self.assertRaises(privacy.observation.Failure):
                    h._preservation_private(body, witnesses, [], 'denied')



class CompletionTests(unittest.TestCase):
    def make_harness(self, exports):
        harness = privacy.McpPrivacyAcceptanceMixin()
        harness.summary = {}
        harness.cases = []
        harness._privacy_authorized_and_errors = mock.Mock()
        harness._privacy_exact_disclosure = mock.Mock()
        harness.mcp_preservation_selection_privacy = mock.Mock()
        harness.mcp_interruption_denial = mock.Mock()
        harness.native_export = mock.Mock(side_effect=lambda: harness.cases.extend(exports))
        harness.group = mock.Mock(side_effect=lambda name, method: method())
        return harness

    def exports(self):
        return [dict(case='export_boundary_' + mode, zip_sha256='zip digest', pack_sha256='pack digest',
                     runtime_exit=0, observed_listeners=0) for mode in ('enabled', 'disabled', 'hook-only')]


    def test_no_export_success_from_missing_artifacts_mode_runtime_or_listener(self):
        for damage in ('missing mode', 'zip', 'pack', 'runtime', 'listener'):
            exports = self.exports()
            if damage == 'missing mode':
                exports.pop()
            elif damage == 'zip':
                exports[0].pop('zip_sha256')
            elif damage == 'pack':
                exports[0].pop('pack_sha256')
            elif damage == 'runtime':
                exports[0]['runtime_exit'] = 1
            else:
                exports[0]['observed_listeners'] = 1
            harness = self.make_harness(exports)
            with self.subTest(damage=damage), self.assertRaises(privacy.observation.Failure):
                harness.mcp_privacy_export()
            self.assertNotIn('privacy_export_verified', harness.summary)


if __name__ == '__main__':
    unittest.main()
