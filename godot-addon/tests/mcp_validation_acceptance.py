"""Prepared native-documentation loss must never authorize or verify an edit."""
from __future__ import annotations

import run_observation as observation
from mcp_interruption_acceptance import DESIRED, PROFILES
from mcp_peer import McpPeer, selectors
from run_script_close import SAFE, TARGET


class McpValidationMixin:
    def mcp_validation_preparation(self):
        self.summary.update(coverage_scope='prepared_native_documentation_effect_safety',
                            mcp_acceptance=True, real_client_acceptance=False,
                            mcp_server_sha256=observation.digest(self.args.mcp_server))
        for profile in PROFILES:
            for boundary in ('before', 'after'):
                label = 'native-docs-' + profile + '-' + boundary
                scratch = self.work / (label + '-scratch')
                scratch.mkdir(mode=0o700)
                with self._interruption_fixture(label, profile) as (project, editor, descriptor):
                    with McpPeer(self, label, temporary_root=scratch) as peer:
                        namespaces = list(scratch.iterdir())
                        observation.require(len(namespaces) == 1, 'one_prepared_owned_namespace_' + label)
                        snapshot = namespaces[0] / 'home/Library/Caches/Godot/editor_doc_cache-4.7.res'
                        observation.require([p for p in namespaces[0].rglob('*') if p.is_file()] == [snapshot],
                                            'only_native_snapshot_survives_preparation_' + label)
                        if boundary == 'after':
                            call = self._interruption_start(peer, project, editor, descriptor, profile,
                                                            'verify:post_change', label)
                            before, disks = self.state(editor, project)
                            observation.require(disks[TARGET]['text'] == DESIRED,
                                                'actual_effect_precedes_artifact_loss_' + label)
                            snapshot.unlink()
                            self._interruption_release(editor, profile)
                        else:
                            basis = self.mcp_read(peer, project, editor, descriptor, label + '-basis', source=SAFE)
                            before, disks = self.state(editor, project)
                            # Same size, different bytes: size/shape alone must not admit a snapshot.
                            with snapshot.open('r+b') as stream:
                                first = stream.read(1)
                                stream.seek(0)
                                stream.write(bytes([first[0] ^ 1]))
                            call = peer.start('edit_script', dict(selectors(project, descriptor),
                                               revision=basis['revision'], replacement_source=DESIRED))
                        root = peer.finish(call, label)
                        result = self.mcp_review_edit(root, label,
                                                     'refused' if boundary == 'before' else 'applied_unverified')
                        observation.require(result['outcome']['reason'] == 'validation_unavailable',
                                            'artifact_loss_is_validation_unavailable_' + label)
                        after, now = self._interruption_survivor(project, editor, before, disks, label,
                                                                 no_effect=True)
                        expected = SAFE if boundary == 'before' else DESIRED
                        observation.require(now[TARGET]['text'] == expected,
                                            'artifact_failure_never_changes_or_rolls_back_D_' + label)
                        if profile == 'open':
                            observation.require(after['target']['B'] == after['target']['R'] == expected,
                                                'artifact_failure_retains_actual_open_authorities_' + label)
                        else:
                            observation.require(TARGET not in after['open_paths'] and
                                                after['cached_R'] == (expected if after['cached_id'] else None),
                                                'artifact_failure_preserves_closed_applicability_' + label)
                    observation.require(not list(scratch.iterdir()), 'prepared_namespace_removed_on_EOF_' + label)
                scratch.rmdir()
