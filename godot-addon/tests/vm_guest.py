#!/usr/bin/env python3
"""Private Tart macOS guest worker; existing GUI runners own all assertions."""
import argparse
import fcntl
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import platform
import pwd
import re
import shutil
import stat
import subprocess
import sys
import tarfile
import tempfile
import urllib.request
import uuid

ROOT = Path('/Users/admin/.godot-agent-kit-vm')
GODOT_SHA = 'c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf'
TEMPLATE_SHA = '88df5e2e6fee99088699be66e6d42e4da4fb0c5619d054297d755a49558a4792'
ENGINE = 'ed1daf0bf001b61586d9930840f2f1394092c079'
VERSION = '4.7.2.stable.official.ed1daf0bf'
RUST = '1.98.1'
SUITES = ('observation', 'edit', 'open', 'discovery', 'close')
RUNNERS = {'observation': 'run_observation.py', **{s: 'run_script_' + s + '.py' for s in SUITES[1:]},
           'mcp': 'run_mcp.py'}
BINS = ('observe-gdscript', 'edit-gdscript', 'open-gdscript', 'discover-gdscripts', 'close-gdscript')
COMPONENTS = ('rustfmt', 'clippy', 'rust-analyzer', 'rust-src')


class GuestError(RuntimeError):
    pass


def _digest(path):
    h = hashlib.sha256()
    with path.open('rb') as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b''):
            h.update(chunk)
    return h.hexdigest()


def _json(path):
    return json.loads(path.read_text())


def _save(path, value):
    path.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
    fd, name = tempfile.mkstemp(dir=path.parent, prefix='.receipt-')
    try:
        with os.fdopen(fd, 'w') as stream:
            json.dump(value, stream, sort_keys=True, indent=2)
            stream.write('\n')
        os.replace(name, path)
    finally:
        Path(name).unlink(missing_ok=True)


def _private(path, create=False):
    if create:
        path.mkdir(mode=0o700, parents=True, exist_ok=True)
    info = path.lstat()
    if not stat.S_ISDIR(info.st_mode) or info.st_uid != os.getuid() or stat.S_IMODE(info.st_mode) != 0o700:
        raise GuestError('directory must be owned, nonsymlink and private: ' + str(path))


def _environment(identity='unprovisioned'):
    # Campaign fingerprints every value: do not inherit transport, credentials,
    # terminal state, run IDs, revisions, or environment injected by the host.
    return {'HOME': '/Users/admin', 'USER': 'admin', 'LOGNAME': 'admin',
            'PATH': '/Users/admin/.cargo/bin:/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin',
            'LANG': 'en_US.UTF-8', 'TMPDIR': str(ROOT / 'tmp') + '/',
            'GODOT_AGENT_KIT_VM_ID': identity, 'RUSTUP_AUTO_INSTALL': '0'}


def _command(argv, *, cwd=None, env=None, capture=True, check=True):
    result = subprocess.run([str(a) for a in argv], cwd=cwd, env=env or _environment(),
                            capture_output=capture, text=True)
    if check and result.returncode:
        raise GuestError('command failed (%s): %s\n%s' % (result.returncode, argv[0], (result.stderr or '')[-4000:]))
    return result


def _probe(argv):
    return _command(argv).stdout.strip()


def _guest_guard():
    if sys.version_info < (3, 10):
        raise GuestError('Python >=3.10 required; install python@3.12 with Homebrew inside the guest')
    if platform.system() != 'Darwin' or platform.machine() != 'arm64':
        raise GuestError('worker requires a macOS arm64 VirtualMac guest')
    model = _probe(['/usr/sbin/sysctl', '-n', 'hw.model'])
    marker = _probe(['/usr/sbin/sysctl', '-n', 'kern.hv_vmm_present'])
    if not model.startswith('VirtualMac') or marker != '1':
        raise GuestError('refusing host execution: VirtualMac and hypervisor guest marker required')
    if pwd.getpwuid(os.getuid()).pw_name != 'admin' or Path.home() != Path('/Users/admin'):
        raise GuestError('Tart guest agent must execute as admin in /Users/admin')
    if os.stat('/dev/console').st_uid != os.getuid():
        raise GuestError('admin must own the active graphical console session')
    _command(['/bin/launchctl', 'print', 'gui/' + str(os.getuid())])
    # CLT Swift suffices. Check an unlocked, logged-in Aqua session, not merely
    # the existence of a launchd GUI domain. No focus or security bypass calls.
    script = ('import CoreGraphics\nimport Foundation\n'
              'guard let s = CGSessionCopyCurrentDictionary() as? [String: Any], '
              's[kCGSessionOnConsoleKey as String] as? Bool == true, '
              's[kCGSessionLoginDoneKey as String] as? Bool == true, '
              's["CGSSessionScreenIsLocked"] as? Bool != true else { exit(1) }\n'
              'print(CGDisplayPixelsWide(CGMainDisplayID()), CGDisplayPixelsHigh(CGMainDisplayID()))\n')
    try:
        result = _command(['/usr/bin/xcrun', 'swift', '-e', script], env={**_environment(), 'TMPDIR': '/tmp/'})
        display = [int(value) for value in result.stdout.split()]
        if len(display) != 2 or min(display) <= 0:
            raise GuestError('graphical desktop dimensions unavailable')
    except (OSError, ValueError, GuestError) as exc:
        raise GuestError('Apple CLT Swift and an unlocked logged-in admin GUI session are required: ' + str(exc)) from exc
    return {'hardware_model': model, 'hypervisor_guest': True, 'console_uid': os.getuid(),
            'uid': os.getuid(), 'display_pixels': display}


def _tools():
    try:
        return {'clang': _probe(['/usr/bin/clang++', '--version']),
                'sdk': _probe(['/usr/bin/xcrun', '--show-sdk-version']),
                'sdk_path': _probe(['/usr/bin/xcrun', '--show-sdk-path']),
                'swift': _probe(['/usr/bin/xcrun', 'swift', '--version']),
                'rustc': _probe(['/Users/admin/.cargo/bin/rustc', '+' + RUST, '-vV']),
                'cargo': _probe(['/Users/admin/.cargo/bin/cargo', '+' + RUST, '--version'])}
    except (OSError, GuestError) as exc:
        raise GuestError('missing Apple CLT/SDK/Swift or pinned Rust prerequisite: ' + str(exc)) from exc


def _godot(root):
    return root / 'inputs/Godot.app/Contents/MacOS/Godot'


def _pinned_inputs(root):
    godot = _godot(root)
    template = Path('/Users/admin/Library/Application Support/Godot/export_templates/4.7.2.stable/macos.zip')
    for path, expected in ((godot, GODOT_SHA), (template, TEMPLATE_SHA)):
        if not path.is_file() or path.is_symlink() or _digest(path) != expected:
            raise GuestError('missing or modified pinned official input: ' + str(path))
    if _probe([godot, '--version']) != VERSION:
        raise GuestError('Godot version does not match the pinned official engine')
    return {'godot_version': VERSION, 'godot_commit': ENGINE, 'godot_sha256': GODOT_SHA,
            'template_sha256': TEMPLATE_SHA}


def _install_rust():
    rustup = Path('/Users/admin/.cargo/bin/rustup')
    if not rustup.exists():
        # Official bootstrap only, downloaded into private guest-local storage.
        with urllib.request.urlopen('https://sh.rustup.rs', timeout=60) as response:
            bootstrap = response.read()
        fd, name = tempfile.mkstemp(dir=ROOT / 'tmp', suffix='.sh')
        try:
            with os.fdopen(fd, 'wb') as stream:
                stream.write(bootstrap)
            _command(['/bin/sh', name, '-y', '--profile', 'minimal', '--default-toolchain', RUST])
        finally:
            Path(name).unlink(missing_ok=True)
    installed = _probe([rustup, 'toolchain', 'list']).splitlines()
    if not any(line.split()[0].startswith(RUST + '-') for line in installed):
        _command([rustup, 'toolchain', 'install', RUST, '--profile', 'minimal'])
    present = _probe([rustup, 'component', 'list', '--toolchain', RUST, '--installed']).splitlines()
    for component in COMPONENTS:
        if not any(line == component or line.startswith(component + '-') for line in present):
            _command([rustup, 'component', 'add', '--toolchain', RUST, component])


def _snapshot(root):
    guard = _guest_guard()
    return {**guard, **_pinned_inputs(root), 'os': _probe(['/usr/bin/sw_vers', '-productVersion']),
            'os_build': _probe(['/usr/bin/sw_vers', '-buildVersion']), 'arch': platform.machine(),
            'kernel': platform.release(), 'python': sys.version, 'tools': _tools(),
            'cpu_count': int(_probe(['/usr/sbin/sysctl', '-n', 'hw.ncpu'])),
            'memory_bytes': int(_probe(['/usr/sbin/sysctl', '-n', 'hw.memsize']))}


def _provision(root, generation):
    generation = str(uuid.UUID(generation))
    _guest_guard()
    # Fail before installation if the Apple build tools are absent.
    _probe(['/usr/bin/xcrun', '--show-sdk-version'])
    _probe(['/usr/bin/clang++', '--version'])
    _pinned_inputs(root)
    _install_rust()
    snapshot = _snapshot(root)
    service_results = {}
    for name, argv in (
        ('remote_login', ['/usr/bin/sudo', '-n', '/bin/launchctl', 'disable', 'system/com.openssh.sshd']),
        ('screen_sharing', ['/usr/bin/sudo', '-n', '/bin/launchctl', 'disable', 'system/com.apple.screensharing'])):
        result = _command(argv, check=False)
        service_results[name] = {'exit_code': result.returncode, 'disabled': result.returncode == 0}
        if result.returncode:
            print('warning: unable to disable ' + name + '; inspect guest service configuration', file=sys.stderr)
    identity = hashlib.sha256(json.dumps({'generation': generation, 'environment': snapshot}, sort_keys=True).encode()).hexdigest()
    receipt = {'generation': generation, 'identity': identity, 'environment': snapshot,
               'development_only': (snapshot['os'], snapshot['os_build']) != ('26.6.2', '25G83'),
               'service_disable': service_results}
    _save(root / 'provision.json', receipt)
    return receipt


def _execution_identity(provision):
    return hashlib.sha256((provision['identity'] + ':' + _digest(Path(__file__).resolve())).encode()).hexdigest()


def _safe_relative(name):
    path = PurePosixPath(name)
    if not name or path.is_absolute() or '..' in path.parts or '\\' in name or str(path) != name or name == '.':
        raise GuestError('unsafe archive path: ' + name)
    return path


def _regular(path):
    return path.exists() and stat.S_ISREG(path.lstat().st_mode) and path.stat().st_uid == os.getuid()


def _safe_destination(root, relative, obsolete=()):
    path = root.joinpath(*_safe_relative(relative).parts)
    current = root
    if root.is_symlink():
        raise GuestError('source root must not be a symlink')
    for part in _safe_relative(relative).parts[:-1]:
        current /= part
        removable = current.relative_to(root).as_posix() in obsolete and _regular(current)
        if current.is_symlink() or (current.exists() and not current.is_dir() and not removable):
            raise GuestError('unsafe source ancestor: ' + str(current))
    if path.is_symlink():
        raise GuestError('unsafe source destination: ' + str(path))
    return path


def _verify_source(root, revision):
    receipt = _json(root / 'source.json')
    if receipt['revision'] != revision:
        raise GuestError('requested revision differs from synced source receipt')
    repo = root / 'workspace/repo'
    for name, expected in receipt['files'].items():
        path = _safe_destination(repo, name)
        if not _regular(path) or _digest(path) != expected['sha256'] or stat.S_IMODE(path.stat().st_mode) != expected['mode']:
            raise GuestError('tracked source changed or missing: ' + name)
    return receipt


def _sync(root, revision, archive_hash, stream):
    repo = root / 'workspace/repo'
    if (root / 'workspace').is_symlink() or repo.is_symlink():
        raise GuestError('workspace must not be a symlink')
    repo.mkdir(mode=0o700, parents=True, exist_ok=True)
    previous = _json(root / 'source.json') if (root / 'source.json').exists() else {'files': {}}
    managed_path = root / 'managed-source.json'
    managed = set(_json(managed_path)) if managed_path.exists() else set(previous['files'])
    with tempfile.TemporaryFile() as archive:
        digest = hashlib.sha256()
        for chunk in iter(lambda: stream.read(1024 * 1024), b''):
            digest.update(chunk)
            archive.write(chunk)
        if digest.hexdigest() != archive_hash:
            raise GuestError('source archive SHA-256 mismatch; previous workspace unchanged')
        archive.seek(0)
        with tarfile.open(fileobj=archive, mode='r:') as tar:
            members = tar.getmembers()
            files = {}
            names = set()
            for member in members:
                name = member.name.rstrip('/') if member.isdir() else member.name
                _safe_relative(name)
                if not (member.isdir() or member.isfile()) or name in names:
                    raise GuestError('source archive must contain only unique regular files and directories')
                names.add(name)
                if member.isfile():
                    files[member.name] = {'sha256': hashlib.sha256(tar.extractfile(member).read()).hexdigest(),
                                          'mode': 0o755 if member.mode & 0o111 else 0o644}
            if not files or any(any(parent.as_posix() in files for parent in PurePosixPath(name).parents if str(parent) != '.') for name in files):
                raise GuestError('empty or conflicting source archive')
            obsolete = managed - files.keys()
            for member in members:
                _safe_destination(repo, member.name.rstrip('/') if member.isdir() else member.name, obsolete)
            _save(managed_path, sorted(managed | files.keys()))
            # Invalidate the receipt before mutation; interrupted sync can never
            # authorize mixed revisions. Only previously managed source is removed.
            (root / 'source.json').unlink(missing_ok=True)
            for name in managed - files.keys():
                path = _safe_destination(repo, name)
                if path.exists():
                    if not _regular(path):
                        raise GuestError('obsolete tracked path is no longer a regular owned file')
                    path.unlink()
            # Remove only empty ancestors of obsolete managed files. This also
            # permits a committed file/directory transition without deleting
            # ignored outputs or any unrelated contents.
            parents = {parent for name in obsolete for parent in (repo / name).parents if parent != repo and repo in parent.parents}
            for parent in sorted(parents, key=lambda p: len(p.parts), reverse=True):
                try:
                    parent.rmdir()
                except OSError:
                    pass  # Missing/nonempty directories are not owned source.
            for member in members:
                if not member.isfile():
                    continue
                path = _safe_destination(repo, member.name)
                path.parent.mkdir(parents=True, exist_ok=True)
                fd, temporary = tempfile.mkstemp(dir=path.parent)
                try:
                    with os.fdopen(fd, 'wb') as output:
                        shutil.copyfileobj(tar.extractfile(member), output)
                    os.chmod(temporary, files[member.name]['mode'])
                    os.replace(temporary, path)
                finally:
                    Path(temporary).unlink(missing_ok=True)
    _save(managed_path, sorted(files))
    receipt = {'revision': revision, 'archive_sha256': archive_hash, 'files': files}
    _save(root / 'source.json', receipt)
    return _verify_source(root, revision)


def _cache_key(files, prefixes, inputs):
    selected = {name: value for name, value in files.items()
                if any(name.startswith(p) for p in prefixes)
                and Path(name).suffix not in ('.md', '.markdown', '.rst')}
    return hashlib.sha256(json.dumps({'sources': selected, 'inputs': inputs}, sort_keys=True).encode()).hexdigest()


def _cache_valid(receipt, key, outputs):
    if not receipt or receipt.get('key') != key or set(receipt.get('outputs', {})) != {str(p) for p in outputs}:
        return False
    return all(_regular(p) and _digest(p) == receipt['outputs'][str(p)] for p in outputs)


def _build(root, source, provision):
    repo = root / 'workspace/repo'
    tools = provision['environment']['tools']
    env = _environment(_execution_identity(provision))
    receipts = {}
    cargo = repo / 'mcp-server'
    rust_outputs = [cargo / 'target/debug' / name for name in BINS] + [
        cargo / 'target/debug/examples/stock_validation_fixture',
        cargo / 'target/debug/examples/script_workflow_fixture',
        cargo / 'target/debug/libgodot_agent_kit.rlib']
    rust_key = _cache_key(source['files'], ('mcp-server/',), tools)
    cache_path = root / 'cache/rust.json'
    old = _json(cache_path) if cache_path.exists() else None
    if not _cache_valid(old, rust_key, rust_outputs):
        _command(['/Users/admin/.cargo/bin/cargo', '+' + RUST, 'build', '--locked', '--lib', '--bins',
                  '--example', 'stock_validation_fixture', '--example', 'script_workflow_fixture'], cwd=cargo, env=env)
        if not rust_outputs or not all(_regular(p) for p in rust_outputs):
            raise GuestError('Rust build did not produce required executables/library')
        _save(cache_path, {'key': rust_key, 'outputs': {str(p): _digest(p) for p in rust_outputs}})
    receipts['rust'] = _json(cache_path)
    generated = repo / 'godot-addon/native/build'
    # native_abi_sizes.h embeds fixture-specific build IDs and is overwritten by
    # each build; the immutable public generated inputs are shared and validated.
    api_names = ('gdextension_interface.h', 'gdextension_interface.json', 'extension_api.json')
    for fixture in (False, True):
        addon = root / 'build/fixture-native' if fixture else repo / 'godot-addon/addons/godot_agent_kit/native'
        outputs = [addon / name for name in ('build-manifest.json', 'libeditor_integration.macos.arm64.dylib', 'editor_integration.gdextension')] + [generated / name for name in api_names]
        key = _cache_key(source['files'], ('godot-addon/native/',), {'tools': tools, 'engine': GODOT_SHA, 'fixture': fixture})
        cache_path = root / ('cache/native-fixture.json' if fixture else 'cache/native-production.json')
        old = _json(cache_path) if cache_path.exists() else None
        if not _cache_valid(old, key, outputs):
            argv = [sys.executable, repo / 'godot-addon/native/build.py', '--godot', _godot(root)]
            if fixture:
                argv += ['--fixture-faults', '--output-addon', addon]
            _command(argv, cwd=repo, env=env)
            manifest = _json(addon / 'build-manifest.json')
            if manifest['fixture_only'] != fixture or manifest['engine_sha256'] != GODOT_SHA or manifest['base_commit'] != ENGINE or manifest['native_library_sha256'] != _digest(outputs[1]):
                raise GuestError('native build provenance mismatch')
            _save(cache_path, {'key': key, 'outputs': {str(p): _digest(p) for p in outputs}})
        receipts['fixture' if fixture else 'production'] = _json(cache_path)
    _verify_source(root, source['revision'])
    return receipts


def _runner_options(root, suite):
    target = root / 'workspace/repo/mcp-server/target/debug'
    values = {'godot': _godot(root), 'observer': target / BINS[0], 'editor': target / BINS[1],
              'opener': target / BINS[2], 'discoverer': target / BINS[3], 'closer': target / BINS[4],
              'stock-validator': target / 'examples/stock_validation_fixture',
              'workflow': target / 'examples/script_workflow_fixture',
              'native-fault-addon': root / 'build/fixture-native'}
    names = ['godot', 'observer']
    if suite != 'observation':
        names += ['editor', 'stock-validator', 'native-fault-addon']
    if suite in ('open', 'discovery', 'close', 'mcp', 'all'):
        names.append('opener')
    if suite in ('discovery', 'close', 'mcp', 'all'):
        names.append('discoverer')
    if suite in ('close', 'mcp', 'all'):
        names.append('closer')
    if suite == 'mcp':
        names.append('workflow')
    return [item for name in names for item in ('--' + name, str(values[name]))]


def _run(root, args):
    provision = _json(root / 'provision.json')
    current = _snapshot(root)
    if current != provision['environment']:
        raise GuestError('guest environment changed: reprovision before execution')
    source = _verify_source(root, args.revision)
    run = root / 'runs' / args.run_id
    if args.action == 'campaign' and args.resume:
        _private(run)
        old = _json(run / 'provenance.json')
        if old['suite'] != args.suite or old['action'] != 'campaign':
            raise GuestError('resume run identity does not match campaign suite')
    else:
        run.mkdir(mode=0o700, parents=True, exist_ok=False)
    artifacts = run / 'artifacts'
    _private(artifacts, create=True)
    provenance = {'action': args.action, 'suite': args.suite, 'scenario': getattr(args, 'scenario', None),
                  'revision': args.revision, 'archive_sha256': source['archive_sha256'],
                  'generation': provision['generation'], 'environment_identity': _execution_identity(provision),
                  'worker_sha256': _digest(Path(__file__).resolve()), 'guest_godot_pids': [],
                  'environment': current, 'worker_pid': os.getpid(), 'runner_pid': None, 'exit_code': None}
    _save(run / 'provenance.json', provenance)
    exit_code = 70
    try:
        provenance['builds'] = _build(root, source, provision)
        tests = root / 'workspace/repo/godot-addon/tests'
        if args.action == 'campaign':
            argv = [sys.executable, str(tests / 'run_editor_campaign.py'), '--suite', args.suite, '--campaign-dir', str(artifacts)]
            if args.resume:
                argv.append('--resume')
            if args.keep_going:
                argv.append('--keep-going')
        else:
            argv = [sys.executable, str(tests / RUNNERS[args.suite]), '--scenario', args.scenario, '--artifacts', str(artifacts)]
        argv += _runner_options(root, args.suite)
        _verify_source(root, args.revision)
        # Output is private guest evidence; don't accidentally export logs.
        with (run / 'runner.log').open('a') as log:
            process = subprocess.Popen(argv, cwd=root / 'workspace/repo', env=_environment(_execution_identity(provision)), stdout=log, stderr=subprocess.STDOUT)
            provenance['runner_pid'] = process.pid
            _save(run / 'provenance.json', provenance)
            while True:
                probe = _command(['/usr/bin/pgrep', '-x', 'Godot'], check=False)
                provenance['guest_godot_pids'] = sorted(set(provenance['guest_godot_pids']) | {int(pid) for pid in probe.stdout.split()})
                try:
                    exit_code = process.wait(timeout=1)
                    break
                except subprocess.TimeoutExpired:
                    continue
        _verify_source(root, args.revision)
        if _snapshot(root) != current:
            raise GuestError('GUI or guest environment changed during execution')
    except (GuestError, OSError, ValueError) as exc:
        provenance['infrastructure_error'] = str(exc)
        # Never turn a runner failure into success; preserve its actual status.
        if exit_code == 0:
            exit_code = 70
    finally:
        provenance['exit_code'] = exit_code
        _save(run / 'provenance.json', provenance)
    print(json.dumps({'run_id': args.run_id, 'exit_code': exit_code, 'provenance': str(run / 'provenance.json')}, sort_keys=True))
    return exit_code if exit_code >= 0 else 128 - exit_code


def _export_paths(run, captures=False):
    _private(run)
    provenance = run / 'provenance.json'
    if not _regular(provenance):
        raise GuestError('run provenance unavailable or unsafe')
    result = [provenance]
    artifacts = run / 'artifacts'
    _private(artifacts)
    capture_directories = set()
    for directory, dirs, files in os.walk(artifacts, followlinks=False):
        base = Path(directory)
        for name in dirs + files:
            if (base / name).is_symlink():
                raise GuestError('refusing artifact export with symlinks')
        relative = base.relative_to(artifacts).parts
        permitted = not relative or (len(relative) == 3 and relative[0] in RUNNERS and re.fullmatch(r'[a-z][a-z0-9-]*', relative[1]) and re.fullmatch(r'attempt-[0-9]{3,}', relative[2]))
        if captures and permitted:
            capture_directories.add(base)
            summary = base / 'summary.json'
            if _regular(summary):
                report = _json(summary)
                # Failed groups do not get a groups entry, but their completed
                # cases still identify captured evidence. Preserve those too.
                for record in (*report.get('groups', {}).values(), *report.get('cases', [])):
                    relative_group = record.get('artifact_directory')
                    if relative_group:
                        parts = _safe_relative(relative_group).parts
                        if any(part in ('registry', 'workspace', 'control') for part in parts):
                            raise GuestError('summary references a private capture directory')
                        capture_directories.add(base.joinpath(*parts))
        for name in files:
            allowed = ((permitted and name == 'summary.json') or
                       (not relative and name == 'manifest.json') or
                       (captures and base in capture_directories and name.endswith('.png')))
            if allowed:
                path = base / name
                if not _regular(path):
                    raise GuestError('export requires owned regular files')
                result.append(path)
    return sorted(result)


def _export(root, run_id, captures, stream):
    run = root / 'runs' / run_id
    with tarfile.open(fileobj=stream, mode='w|') as archive:
        for path in _export_paths(run, captures):
            name = path.relative_to(run).as_posix()
            _safe_relative(name)
            archive.add(path, arcname=name, recursive=False)


def _token(value):
    if not re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9_-]{0,79}', value):
        raise argparse.ArgumentTypeError('requires a safe identifier (letters, digits, underscore, dash; <=80)')
    return value


def _sha(value):
    if not re.fullmatch(r'[0-9a-f]{40}|[0-9a-f]{64}', value):
        raise argparse.ArgumentTypeError('requires a full lowercase hexadecimal SHA')
    return value


def _parser():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='action', required=True)
    provision = commands.add_parser('provision')
    provision.add_argument('--generation', required=True)
    commands.add_parser('status')
    sync = commands.add_parser('sync')
    sync.add_argument('--revision', required=True, type=_sha)
    sync.add_argument('--archive-sha256', required=True, type=_sha)
    for action in ('run', 'campaign'):
        command = commands.add_parser(action)
        command.add_argument('--revision', required=True, type=_sha)
        command.add_argument('--run-id', required=True, type=_token)
        command.add_argument('--suite', required=True, choices=tuple(RUNNERS) if action == 'run' else (*SUITES, 'all'))
        if action == 'run':
            command.add_argument('--scenario', required=True, type=_token)
        else:
            command.add_argument('--resume', action='store_true')
            command.add_argument('--keep-going', action='store_true')
    export = commands.add_parser('export')
    export.add_argument('--run-id', required=True, type=_token)
    export.add_argument('--captures', action='store_true')
    return parser


def main(argv=None):
    args = _parser().parse_args(argv)
    os.umask(0o077)
    try:
        _guest_guard()  # All actions, including sync/export, are guest-only.
        _private(ROOT, create=True)
        _private(ROOT / 'tmp', create=True)
        with (ROOT / '.worker-lock').open('a') as lock:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
            if args.action == 'provision':
                result = _provision(ROOT, args.generation)
            elif args.action == 'status':
                result = {'environment': _snapshot(ROOT), 'provision': _json(ROOT / 'provision.json') if (ROOT / 'provision.json').exists() else None,
                          'source': _json(ROOT / 'source.json') if (ROOT / 'source.json').exists() else None,
                          'worker_pid': os.getpid(), 'worker_sha256': _digest(Path(__file__).resolve())}
            elif args.action == 'sync':
                result = _sync(ROOT, args.revision, args.archive_sha256, sys.stdin.buffer)
                result = {key: result[key] for key in ('revision', 'archive_sha256')}
            elif args.action in ('run', 'campaign'):
                return _run(ROOT, args)
            else:
                _export(ROOT, args.run_id, args.captures, sys.stdout.buffer)
                return 0
        print(json.dumps(result, sort_keys=True))
        return 0
    except (GuestError, OSError, ValueError, KeyError, tarfile.TarError) as exc:
        print('VM guest infrastructure error: ' + str(exc), file=sys.stderr)
        return 70


if __name__ == '__main__':
    raise SystemExit(main())
