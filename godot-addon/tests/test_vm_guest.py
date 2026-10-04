"""Pure filesystem tests: no VM, engine, compiler, or network execution."""
import argparse
import hashlib
import io
import json
import os
from pathlib import Path
import tarfile
import tempfile
import unittest
from unittest.mock import patch

import vm_guest as guest


class GuestTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        os.chmod(self.root, 0o700)

    def archive(self, files):
        stream = io.BytesIO()
        with tarfile.open(fileobj=stream, mode='w') as archive:
            for name, content in files.items():
                info = tarfile.TarInfo(name)
                info.size = len(content)
                info.mode = 0o644
                archive.addfile(info, io.BytesIO(content))
        data = stream.getvalue()
        return data, hashlib.sha256(data).hexdigest()

    def sync(self, files, revision='a' * 40):
        data, digest = self.archive(files)
        return guest._sync(self.root, revision, digest, io.BytesIO(data))

    def test_active_prepared_owner_prevents_source_replacement(self):
        self.sync({'tracked.txt': b'original'})
        (self.root / 'tmp').mkdir()
        (self.root / 'tmp/mcp-owned.sock').touch()
        with self.assertRaisesRegex(guest.GuestError, 'finalize'):
            self.sync({'tracked.txt': b'replacement'}, 'b' * 40)
        self.assertEqual((self.root / 'workspace/repo/tracked.txt').read_bytes(), b'original')

    def test_prepared_export_keeps_private_calls_not_project_or_credentials(self):
        run = self.root / 'runs/owned'
        artifacts = run / 'artifacts'
        artifacts.mkdir(parents=True, mode=0o700)
        run.chmod(0o700)
        (run / 'provenance.json').write_text('{}')
        (run / 'prepared.json').write_text('{}')
        (artifacts / 'mcp-calls.jsonl').write_text('private result')
        (artifacts / 'target-witness.json').write_text('{}')
        (artifacts / 'credentials.json').write_text('never export')
        (artifacts / 'project.godot').write_text('never export')
        names = {p.name for p in guest._export_paths(run)}
        self.assertIn('mcp-calls.jsonl', names)
        self.assertIn('target-witness.json', names)
        self.assertNotIn('credentials.json', names)
        self.assertNotIn('project.godot', names)

    def test_sync_removes_obsolete_tracked_source_preserves_outputs(self):
        self.sync({'mcp-server/src/old.rs': b'old', 'README.md': b'first'})
        target = self.root / 'workspace/repo/mcp-server/target/debug/tool'
        target.parent.mkdir(parents=True)
        target.write_bytes(b'cached')
        self.sync({'mcp-server/src/new.rs': b'new', 'README.md': b'second'}, 'b' * 40)
        self.assertFalse((self.root / 'workspace/repo/mcp-server/src/old.rs').exists())
        self.assertEqual(target.read_bytes(), b'cached')
        self.assertEqual(guest._verify_source(self.root, 'b' * 40)['revision'], 'b' * 40)
        with self.assertRaises(guest.GuestError):
            guest._verify_source(self.root, 'a' * 40)

    def test_modified_source_or_mode_cannot_authorize_run(self):
        self.sync({'source.rs': b'original'})
        path = self.root / 'workspace/repo/source.rs'
        path.write_bytes(b'tampered')
        with self.assertRaises(guest.GuestError):
            guest._verify_source(self.root, 'a' * 40)
        path.write_bytes(b'original')
        path.chmod(0o755)
        with self.assertRaises(guest.GuestError):
            guest._verify_source(self.root, 'a' * 40)

    def test_bad_archive_hash_keeps_previous_revision_intact(self):
        self.sync({'source.rs': b'old'})
        data, _ = self.archive({'source.rs': b'new'})
        with self.assertRaises(guest.GuestError):
            guest._sync(self.root, 'b' * 40, '0' * 64, io.BytesIO(data))
        self.assertEqual(guest._verify_source(self.root, 'a' * 40)['revision'], 'a' * 40)
        self.assertEqual((self.root / 'workspace/repo/source.rs').read_bytes(), b'old')

    def test_source_sync_handles_committed_file_directory_transitions(self):
        self.sync({'entry': b'file'})
        self.sync({'entry/nested': b'nested'}, 'b' * 40)
        self.assertEqual((self.root / 'workspace/repo/entry/nested').read_bytes(), b'nested')
        self.sync({'entry': b'restored'}, 'c' * 40)
        self.assertEqual((self.root / 'workspace/repo/entry').read_bytes(), b'restored')

    def test_failed_runner_keeps_status_and_private_evidence(self):
        script = (b'import json,sys\nfrom pathlib import Path\n'
                  b'p=Path(sys.argv[sys.argv.index("--artifacts")+1])\n'
                  b'(p/"summary.json").write_text(json.dumps({"status":"failed"}))\n'
                  b'(p/"private.json").write_text(json.dumps({"source":"never-export"}))\n'
                  b'raise SystemExit(3)\n')
        environment = {'gui': 'available'}
        guest._save(self.root / 'provision.json', {'environment': environment, 'identity': 'stable', 'generation': 'generation'})
        import subprocess
        for suite, runner, scenario in (('edit', 'run_script_edit.py', 'clean-open'),
                                        ('mcp', 'run_mcp.py', 'closed-native')):
            with self.subTest(suite=suite):
                self.sync({'godot-addon/tests/' + runner: script})
                args = argparse.Namespace(action='run', revision='a' * 40, run_id=suite,
                                          suite=suite, scenario=scenario)
                with patch.object(guest, '_snapshot', return_value=environment), patch.object(guest, '_build', return_value={}), patch.object(guest, '_runner_options', return_value=[]), patch.object(guest, '_command', return_value=subprocess.CompletedProcess([], 1, '', '')), patch('sys.stdout', new=io.StringIO()):
                    status = guest._run(self.root, args)
                self.assertEqual(status, 3)
                run = self.root / 'runs' / suite
                provenance = json.loads((run / 'provenance.json').read_text())
                self.assertEqual(provenance['exit_code'], 3)
                self.assertEqual(provenance['revision'], 'a' * 40)
                self.assertEqual(provenance['suite'], suite)
                self.assertEqual(json.loads((run / 'artifacts/summary.json').read_text()), {'status': 'failed'})
                self.assertEqual(os.stat(run).st_mode & 0o777, 0o700)
                exported = {p.relative_to(run).as_posix() for p in guest._export_paths(run)}
                self.assertEqual(exported, {'provenance.json', 'artifacts/summary.json'})

    def test_archive_traversal_and_symlink_are_rejected(self):
        for name in ('../escape', '/escape', 'a/../escape', 'a\\escape'):
            data, digest = self.archive({name: b'unsafe'})
            with self.assertRaises(guest.GuestError):
                guest._sync(self.root, 'a' * 40, digest, io.BytesIO(data))
        stream = io.BytesIO()
        with tarfile.open(fileobj=stream, mode='w') as archive:
            info = tarfile.TarInfo('link')
            info.type = tarfile.SYMTYPE
            info.linkname = '/Users/admin'
            archive.addfile(info)
        data = stream.getvalue()
        with self.assertRaises(guest.GuestError):
            guest._sync(self.root, 'a' * 40, hashlib.sha256(data).hexdigest(), io.BytesIO(data))

    def test_sync_cannot_follow_existing_directory_symlink(self):
        repo = self.root / 'workspace/repo'
        repo.mkdir(parents=True)
        elsewhere = self.root / 'elsewhere'
        elsewhere.mkdir()
        (repo / 'src').symlink_to(elsewhere, target_is_directory=True)
        data, digest = self.archive({'src/source.rs': b'unsafe'})
        with self.assertRaises(guest.GuestError):
            guest._sync(self.root, 'a' * 40, digest, io.BytesIO(data))
        self.assertFalse((elsewhere / 'source.rs').exists())

    def test_native_cache_invalidates_inputs_and_library_tampering(self):
        files = {'godot-addon/native/build.py': {'sha256': 'one', 'mode': 0o644},
                 'godot-addon/native/README.md': {'sha256': 'doc', 'mode': 0o644}}
        inputs = {'engine': 'engine', 'sdk': 'sdk', 'fixture': False}
        key = guest._cache_key(files, ('godot-addon/native/',), inputs)
        library = self.root / 'library.dylib'
        library.write_bytes(b'library')
        receipt = {'key': key, 'outputs': {str(library): guest._digest(library)}}
        self.assertTrue(guest._cache_valid(receipt, key, [library]))
        docs_changed = dict(files, **{'godot-addon/native/README.md': {'sha256': 'new prose', 'mode': 0o644}})
        self.assertTrue(guest._cache_valid(receipt, guest._cache_key(docs_changed, ('godot-addon/native/',), inputs), [library]))
        for field in ('engine', 'sdk', 'fixture'):
            changed = dict(inputs, **{field: 'different'})
            other = guest._cache_key(files, ('godot-addon/native/',), changed)
            self.assertFalse(guest._cache_valid(receipt, other, [library]))
        changed = dict(files, **{'godot-addon/native/build.py': {'sha256': 'two', 'mode': 0o644}})
        self.assertNotEqual(key, guest._cache_key(changed, ('godot-addon/native/',), inputs))
        library.write_bytes(b'tampered')
        self.assertFalse(guest._cache_valid(receipt, key, [library]))
        library.unlink()
        self.assertFalse(guest._cache_valid(receipt, key, [library]))

    def make_run(self):
        run = self.root / 'runs/sample'
        run.mkdir(mode=0o700, parents=True)
        artifacts = run / 'artifacts'
        artifacts.mkdir(mode=0o700)
        (run / 'provenance.json').write_text('{"revision":"known"}')
        (artifacts / 'summary.json').write_text('{"status":"failed","groups":{"clean-open":{"artifact_directory":"clean-open"}}}')
        (artifacts / 'manifest.json').write_text('{}')
        (artifacts / 'registry.json').write_text('{"token":"never-export"}')
        (artifacts / 'view.png').write_bytes(b'png')
        group = artifacts / 'clean-open'
        group.mkdir()
        (group / 'view.png').write_bytes(b'group capture')
        (group / 'private.json').write_text('{"token":"never-export"}')
        attempt = artifacts / 'open/new-open/attempt-001'
        attempt.mkdir(mode=0o700, parents=True)
        (attempt / 'summary.json').write_text('{}')
        (attempt / 'private.json').write_text('{"token":"never-export"}')
        (attempt / 'view.png').write_bytes(b'png')
        private = artifacts / 'registry'
        private.mkdir()
        (private / 'summary.json').write_text('{"token":"never-export"}')
        (private / 'view.png').write_bytes(b'private')
        return run

    def test_export_maps_only_allowlisted_evidence_and_explicit_captures(self):
        run = self.make_run()
        default = {p.relative_to(run).as_posix() for p in guest._export_paths(run)}
        self.assertEqual(default, {'provenance.json', 'artifacts/summary.json', 'artifacts/manifest.json',
                                   'artifacts/open/new-open/attempt-001/summary.json'})
        captures = {p.relative_to(run).as_posix() for p in guest._export_paths(run, True)}
        self.assertEqual(captures - default, {'artifacts/view.png', 'artifacts/clean-open/view.png',
                                             'artifacts/open/new-open/attempt-001/view.png'})
        output = io.BytesIO()
        guest._export(self.root, 'sample', False, output)
        with tarfile.open(fileobj=io.BytesIO(output.getvalue())) as archive:
            self.assertEqual(set(archive.getnames()), default)
            for entry in archive:
                self.assertNotIn(b'never-export', archive.extractfile(entry).read())

    def test_failed_group_captures_survive_without_completed_group_entry(self):
        run = self.make_run()
        (run / 'artifacts/summary.json').write_text(json.dumps({
            'status': 'failed', 'groups': {},
            'cases': [{'artifact_directory': 'clean-open', 'screenshot': 'view.png'}]}))
        captures = {path.relative_to(run).as_posix() for path in guest._export_paths(run, True)}
        self.assertIn('artifacts/clean-open/view.png', captures)
        self.assertNotIn('artifacts/clean-open/private.json', captures)

    def test_export_rejects_symlink_in_evidence_tree(self):
        run = self.make_run()
        (run / 'artifacts/sneaky.json').symlink_to(run / 'provenance.json')
        with self.assertRaises(guest.GuestError):
            guest._export_paths(run)

    def test_argument_surface_rejects_commands_paths_and_shell_text(self):
        parser = guest._parser()
        for argv in (['exec', 'id'], ['export', '--run-id', '../private'],
                     ['run', '--revision', 'HEAD', '--run-id', 'ok', '--suite', 'edit', '--scenario', 'clean-open'],
                     ['run', '--revision', 'a' * 40, '--run-id', 'ok', '--suite', 'edit', '--scenario', 'clean-open;id'],
                     ['campaign', '--revision', 'a' * 40, '--run-id', 'ok', '--suite', 'mcp'],
                     ['campaign', '--revision', 'a' * 40, '--run-id', 'ok', '--suite', 'edit', '--headless']):
            with self.subTest(argv=argv), self.assertRaises(SystemExit), patch('sys.stderr', new=io.StringIO()):
                parser.parse_args(argv)
        # Existing campaign remains the owner of unsupported close/all rejection.
        args = parser.parse_args(['campaign', '--revision', 'a' * 40, '--run-id', 'ok', '--suite', 'close'])
        self.assertEqual(args.suite, 'close')

    def test_host_guard_stops_before_guest_mutation(self):
        with patch.object(guest.platform, 'system', return_value='Darwin'), patch.object(guest.platform, 'machine', return_value='arm64'), patch.object(guest, '_probe', side_effect=['Mac16,1', '0']):
            with self.assertRaisesRegex(guest.GuestError, 'refusing host'):
                guest._guest_guard()

    def test_stable_environment_excludes_transport_and_run_state(self):
        with patch.dict(os.environ, {'SSH_AUTH_SOCK': 'secret', 'TRACEPARENT': 'volatile', 'RUN_ID': 'first'}):
            first = guest._environment('generation')
        with patch.dict(os.environ, {'RUN_ID': 'second', 'SSH_AUTH_SOCK': 'different'}):
            self.assertEqual(first, guest._environment('generation'))
        self.assertNotIn('SSH_AUTH_SOCK', first)
        self.assertNotIn('RUN_ID', first)
        self.assertNotEqual(first, guest._environment('other-generation'))


if __name__ == '__main__':
    unittest.main()
