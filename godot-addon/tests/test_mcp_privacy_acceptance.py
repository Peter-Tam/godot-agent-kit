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
            body = dict(schema_version=1, operation='read_script', request_id='owned',
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
