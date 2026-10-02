# Isolated local real-editor testing

## Decision and Principle XIII review

The concrete failure is repeated Godot Editor/runtime windows stealing the active
maintainer desktop during long acceptance runs. Existing scenario isolation and
private projects do not isolate macOS WindowServer or foreground activation.

- **Host GUI:** simplest operationally, but retains the demonstrated disruption.
- **Headless Godot:** removes the independently observed visible-editor behavior;
  not an acceptable substitute for existing GUI assertions.
- **Hosted/cloud macOS:** adds provider, credentials and operational dependencies
  for a local desktop-isolation requirement; not required.
- **One local macOS VM (selected):** separates the graphical desktop while retaining
  the existing GUI runners, private synthetic projects and acceptance assertions.

Use **Tart**, one CLI around Apple's Virtualization.framework, and one owned profile:
`godot-agent-kit-live-editor`. No custom virtualization framework, multi-backend
interface, remote-execution service or product API. No VM disks enter Git.

Initial machine inspection found no Tart, Lume, Parallels CLI, UTM CLI or vfkit
installation, and no existing VM in the Tart/Parallels storage directories. The
host has hardware virtualization, arm64, 16 GiB RAM and approximately 75 GiB free
space. Initial profile sizing is 2 virtual CPUs and 6 GiB RAM to leave capacity
for host work. Resource contention is still possible: desktop isolation does not
promise zero CPU, memory or disk impact.

[Tart's CLI](https://tart.run/quick-start/) supports macOS guests, clone/start/stop,
terminal control and copy-on-write local clones. `run --no-graphics` suppresses the
**host viewer**, not the guest display device or Godot GUI. Normal runs must also
disable host audio and clipboard sharing; no directory shares or bridged/public
networking are configured. A viewer is optional, explicit debugging only.

Installation/maintenance cost is one backend, a macOS guest disk, guest toolchain
and exact engine/template installation, plus a narrow Python orchestration boundary.
These costs directly prevent the observed disruption and are reused by all existing
GUI suites. No new CI provider or required CI gate is introduced. The existing
hosted and optional live-editor workflows retain their roles.

The Cirrus Homebrew formula failed during setup with a removed `depends_on :macos`
syntax; use the upstream release archive rather than patching Homebrew or adding a
second backend. Backend release provenance and complete setup commands are recorded
below with the implementation evidence.

## Acceptance and support boundary

Normal local real-editor runs must execute in the guest. VM or prerequisite failure
must stop with an actionable error, never fall back to host Godot. Existing runners
retain scenario selection, deadlines, independent witnesses, result schemas,
cleanup and all assertions; the VM changes where they execute, not what they prove.

Infrastructure acceptance requires a real focused guest GUI run and retrieved
artifacts, with no host Godot Editor/runtime process, test-created application
window, or host foreground-activation request. Guest process/window evidence and
Tart's no-viewer boundary establish isolation; merely backgrounding a host process
does not. A second focused run should demonstrate workspace/build reuse.

The currently recorded candidate is macOS **26.6.2 (25G83), arm64**, official Godot
`4.7.2.stable.official.ed1daf0bf`, full commit
`ed1daf0bf001b61586d9930840f2f1394092c079`, executable SHA-256
`c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf`.
A different guest macOS version is isolated development/regression infrastructure,
not an expansion of supported product environments or equivalent support evidence.
Feature 005 requirements, completion state, native behavior and public contracts
remain unchanged. Focused development and cumulative valid-evidence reuse remain
mandatory; a VM is not justification for replaying historical campaigns.

## One-time setup

Install the pinned [Tart 2.40.1 release](https://github.com/openai/tart/releases/tag/2.40.1)
as a complete app bundle, not a detached executable (its virtualization entitlement
requires the bundle). The upstream repository moved from Cirrus Labs to OpenAI.
This release uses [FSL-1.1-ALv2](https://github.com/openai/tart/blob/2.40.1/LICENSE),
which permits internal use; it is an external local prerequisite, not vendored or
redistributed product code. Apple's macOS virtualization licensing still applies.

From the repository root:

```sh
umask 077
STATE="$HOME/.local/state/godot-agent-kit-vm"
mkdir -p "$STATE/backend"
curl --fail --location \
  https://github.com/openai/tart/releases/download/2.40.1/tart.tar.gz \
  --output "$STATE/tart-2.40.1.tar.gz"
printf '%s  %s\n' \
  363e2701154a8155cbc1bb6d845430c9b42697d2a186bc49574471ca2877db46 \
  "$STATE/tart-2.40.1.tar.gz" | shasum -a 256 -c -
tar -xzf "$STATE/tart-2.40.1.tar.gz" -C "$STATE/backend"

python3 godot-addon/tests/run_in_vm.py setup \
  --godot-app "$HOME/Applications/Godot-4.7.2.app" \
  --export-template "$HOME/Library/Application Support/Godot/export_templates/4.7.2.stable/macos.zip"
```

Supply the actual local official bundle/template paths. Setup checks their hashes
without executing host Godot, then copies only those prerequisites into the guest.
The export template SHA-256 is
`88df5e2e6fee99088699be66e6d42e4da4fb0c5619d054297d755a49558a4792`.
The engine download provenance remains in [ci.yml](workflows/ci.yml) and the
[native build instructions](../godot-addon/native/README.md).

The one pinned image is
`ghcr.io/cirruslabs/macos-tahoe-base@sha256:1b093499716409d29e8b5336844528e1cae375db97d2ad8e5aeff78cf0da201e`.
Initial download is about 27.3 GB compressed with a 50 GB virtual disk.
Allow additional space for toolchains, compiled inputs and retained evidence.
The [base image](https://github.com/cirruslabs/macos-image-templates) supplies
auto-login as `admin`, a graphical desktop, Homebrew, Apple command-line tools
and Tart Guest Agent. Full Xcode is not a new requirement: the native C++ and
window-witness Swift builds need the Apple compiler and macOS SDK.

Setup installs Python >=3.10 if absent and Rust **1.98.1**, with `rustfmt`,
`clippy`, `rust-analyzer` and `rust-src`. It verifies the exact engine/template,
guest hardware, admin console ownership and unlocked Aqua session. It caches
the selected revision's locked Cargo dependencies and saves the stopped
`godot-agent-kit-live-editor-base` APFS clone. Slow prerequisites are not
reinstalled for each scenario. Setup is the explicit dependency-download phase;
no cloud runner, SSH key, hosted account or image publishing service is needed.

Control and file transfer use **Tart Guest Agent over the VM-local vsock** and
private host control socket, not a public TCP service or forwarded SSH agent.
Setup attempts to disable the image's Remote Login and Screen Sharing services;
their actual results are recorded. The image's `admin` account belongs only to
this disposable test desktop; never add personal accounts, credentials or projects.
The image's optional Metal shim is **not enabled**, and no host GPU preferences
or product rendering flags are changed.

## Normal use

```sh
python3 godot-addon/tests/run_in_vm.py start
python3 godot-addon/tests/run_in_vm.py status

# Existing focused scenario, existing assertions; engine/native/caller paths supplied in guest.
python3 godot-addon/tests/run_in_vm.py run close --scenario preservation --captures

# Select a campaign only when its existing acceptance scope is required.
python3 godot-addon/tests/run_in_vm.py campaign open --run-id open-regression --keep-going
python3 godot-addon/tests/run_in_vm.py campaign open --run-id open-regression --keep-going --resume

# Automatic retrieval also runs after a failure. Repeat retrieval without rerunning tests.
python3 godot-addon/tests/run_in_vm.py fetch-artifacts open-regression --captures
python3 godot-addon/tests/run_in_vm.py stop
```

Use `run observation`, `edit`, `open`, `discovery` or `close` with the existing
`--scenario` name. This is a fixed runner allowlist, not an arbitrary shell
executor. Existing restrictions still apply: close `all`, `sequential`, `composed`
and complete close/all campaigns are not implemented merely by adding a VM.
The host terminal remains available; the Tart process is detached and has no
viewer. Foreground activation by the inner harness targets guest WindowServer.

Default source is clean committed `HEAD`. Commit development changes first, or
explicitly choose `--revision <commit>` on `setup`, `run` or `campaign`.
An explicit revision deliberately excludes uncommitted changes. The wrapper
prints the full tested commit, transfers its `git archive` (no host Git credentials,
ignored binaries or mutable shared tree), checks the archive SHA-256, and verifies
every tracked file's bytes/mode before and after execution. Interrupted sync
invalidates the source receipt. Guest source remains in one private stable path;
only previously managed obsolete source is removed.

Native production and fixture-fault artifacts are built separately using the
existing native builder. Rust builds use locked resolution. Relevant input and
output hashes decide reuse; an unchanged focused rerun does not recreate the VM,
reinstall prerequisites or rebuild everything. Changed Cargo dependencies may
require explicit temporary network access:

```sh
python3 godot-addon/tests/run_in_vm.py stop
python3 godot-addon/tests/run_in_vm.py start --bootstrap-network
python3 godot-addon/tests/run_in_vm.py run edit --scenario clean-open
python3 godot-addon/tests/run_in_vm.py stop
python3 godot-addon/tests/run_in_vm.py start
```

Normal runs use Tart `--net-host`; bootstrap uses its local NAT, never bridged
networking or public port forwarding. The product's local-only assumptions are
unchanged. Backend telemetry variables are removed; guest commands receive a
small fixed environment without host credentials or telemetry configuration.

## Evidence and resume

Host state, backend and VM disks are under `~/.local/state/godot-agent-kit-vm/`:

- `tart/vms/`: owned VM and reusable base; `tart/cache/`: downloaded image cache.
- `artifacts/<run-id>/<fetch-id>/`: fresh mode-0700 VM evidence retrievals.
- `<run-id>/*.host.json`: tested revision, wrapper/worker hashes, VM launch boundary
  and sampled host process observations. Launcher output stays in a private log.

Guest state is `/Users/admin/.godot-agent-kit-vm/`: `inputs/`, `workspace/repo/`,
`cache/`, separate `build/fixture-native/`, and `runs/<run-id>/`.
Each focused run creates a fresh private artifact directory. Campaigns retain
their existing fresh-per-attempt directories, failure retention, fingerprints,
summary validation and resume behavior. No checkpoint is imported from the host.
The guest's stable environment adds provisioning and worker identity to the
existing campaign environment fingerprint; volatile transport, run IDs and commit
IDs do not invalidate unchanged passes. Native/tool/engine/source changes still
invalidate the existing affected fingerprints.

Default retrieval includes runner summaries, campaign manifests and VM provenance.
`--captures` explicitly includes guest screenshots, which may show synthetic
source. Raw logs, registry descriptors/tokens, fixture controls and guest project
trees are not automatically copied. All evidence stays private/local; review
before sharing and never put the artifact root in a cloud-synced directory.
Runner failures preserve their nonzero exit status; retrieval failure cannot
turn a failed run into success. Interruption does not imply rollback or cleanup:
inspect the guest before starting another run.

Host process sampling is supporting evidence, not proof against every short-lived
process or an unrelated human-launched application. The actual focus boundary is
the dedicated VM: Tart 2.40.1's [no-viewer path](https://github.com/openai/tart/blob/2.40.1/Sources/tart/Commands/Run.swift)
sets `NSApplication` activation policy to `prohibited`; only guest runners launch
Godot or request focus. Human host applications are never killed or controlled.

## Inspect, recover and destroy

For explicit GUI debugging, stop the wrapper-owned VM first, then open Tart's
viewer manually. Do not run acceptance while using an unreceipted manual launcher:

```sh
python3 godot-addon/tests/run_in_vm.py stop
STATE="$HOME/.local/state/godot-agent-kit-vm"
env -u TRACEPARENT -u TRACESTATE TART_HOME="$STATE/tart" TART_NO_AUTO_PRUNE=1 \
  "$STATE/backend/tart.app/Contents/MacOS/tart" run godot-agent-kit-live-editor \
  --net-host --no-audio --no-clipboard --no-usb-accessories
# Close/stop the manual viewer, then use the normal wrapper start again.
```

The viewer is the guest desktop, not individual host Godot windows. It is never
opened by normal automated execution. Use it for guest login/session/permission
diagnosis only; do not bypass missing observations or weaken runner assertions.

Fetch wanted evidence before a reset:

```sh
python3 godot-addon/tests/run_in_vm.py stop
python3 godot-addon/tests/run_in_vm.py reset --discard-guest-runs
python3 godot-addon/tests/run_in_vm.py start
```

Reset explicitly discards the owned guest workspace/evidence and clones the
saved base; host evidence remains. It is recovery, not the inner loop. Existing
base images are not silently overwritten. To recreate from scratch, stop the VM,
retain any needed host evidence, and explicitly remove **only this dedicated
state directory**, then repeat installation/setup. No VM images are committed.
Do not delete another VM directory or prune unrelated Tart state.

No host fallback exists. Missing/stopped VM, an unowned viewer launch, unavailable
guest agent, missing exact input, stale source, invalid artifacts or a lost GUI
session produces a nonzero result with guidance. Host GUI execution is not an
alternative merely because guest setup is inconvenient.
