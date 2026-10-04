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
space. The profile uses 4 virtual CPUs and 6 GiB RAM, requests a 1280×800 display
with viewer refitting disabled, and records the actual guest display dimensions.
An initial 2-CPU run (1440×900 requested) had a close operation refuse with
`context_validation_unavailable` after 8.9 seconds; no product deadline or assertion
was changed. CPU, memory and display
observations participate in guest execution identity. Resource contention is still
possible: desktop isolation does not promise zero CPU, memory or disk impact.

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
artifacts, with no **test-created** host Godot Editor/runtime process, application
window, or foreground-activation request. Independently running human host editors
are recorded as a baseline and left untouched; the maintainer explicitly chose
this baseline-aware criterion during implementation. Guest process/window evidence
and Tart's no-viewer boundary establish isolation; merely backgrounding a host
process does not. A second focused run should demonstrate workspace/build reuse.

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
`--scenario` name. `run mcp --scenario closed-native` and `closed-lifecycle`
exercise the protocol-independent read/revision/closed-edit fixture consumer;
these groups do not claim MCP or real-agent interoperability. The wrapper
builds `script_workflow_fixture` and supplies only its fixed guest path.
No `mcp` campaign is registered. This remains a fixed runner allowlist,
not an arbitrary shell executor. Close `sequential`, `composed`, `all` and complete close/all campaigns
are now implemented; [Feature 005 acceptance](../specs/005-close-project-gdscript/quickstart.md#12-t003-cumulative-acceptance-2026-10-02)
records the focused runs and reviewed historical evidence. Complete campaign
availability is not a requirement to replay unchanged historical GUI suites.

Feature 007's `run mcp --scenario transport` uses the actual stdio executable.
For host-owned coding-agent clients, `prepare-mcp --revision SHA --run-id ID
--artifacts DIR` creates an owned guest fixture and writes private `receipt.json`,
`mcp.json` and `prompt.txt`. `mcp-stdio --run-id ID` relays only its fixed
executable/registry over Tart; it accepts no arbitrary guest command or endpoint.
Keep model credentials on the host and preserve existing project client settings.
For `workflow`, run `prepare-durability --run-id ID` after the primary client
invocation, then feed its read-only prompt to the same selected client using the
unchanged relay. This first validates primary success, then opens the verified
cached/absent targets for an actual agent read. `finalize-mcp --run-id ID`
requires those reads, verifies independent durability, cleans up owned fixtures
and fetches evidence. Other profiles finalize directly after their client exits.
Connection EOF alone does not tear down the prepared projects. Finalize before
changing guest source. See the [client workflow](../specs/007-mcp-script-workflow/quickstart.md#real-coding-agent-clients)
for scoped configuration and the distinction between raw MCP records and actual
model-visible acceptance.
Preparation's fixed `--profile workflow|known|sources|bound|observations` selector
keeps independent fixtures small; the 512 KiB case runs alone, and the known path
does not leak into the discovery-first prompt. Full selected-client acceptance
requires all five groups, with a fresh conversation/run ID/artifact directory and
finalization for each. `run mcp --scenario transport-<profile>` isolates the same
groups for affected regressions; aggregate `transport` runs them sequentially.
Relay and fixed continuation/finalization controls share the existing host lock;
setup/source-changing operations remain exclusive. Finalization can reach an idle
live relay without waiting for client EOF.

The host terminal remains available; the Tart process is detached and has no
viewer. Foreground activation by the inner harness targets guest WindowServer.

`start` also observes guest CPU headroom within its existing **180-second**
readiness deadline. A reachable guest agent and logged-in Aqua session alone do
not establish a settled desktop during cold boot. Five measured one-second CPU
intervals must each show at least 75% idle; the initial cumulative `top` report
does not count. This leaves three of the profile's four CPUs for editor,
validator and control work. The one-minute load average is recorded, not used
as a gate: it can remain high after current work has finished.

The observation is saved in the owned launcher receipt and included by `status`
and host run provenance. Missing or failed startup readiness refuses `run` and
`campaign` before source transfer or acceptance; use `start` to establish it.
Stop and artifact retrieval remain available. No warm-up editor, product retry,
new fixed sleep, resource/profile change or deadline extension is performed.

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
`workers/<sha256>.py`, `cache/`, separate `build/fixture-native/`, and `runs/<run-id>/`.
The host pins helper bytes for each operation and uploads that exact helper even
for standalone artifact retrieval. A restored base cannot silently select an old
helper, and a new version cannot overwrite an interrupted worker's executable.
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

Reset discards changes since the saved base and restores that complete image,
including its saved workspace state; host evidence remains. It is recovery, not
the inner loop. Normal stop synchronizes the guest filesystem before stopping Tart;
unavailable sync is reported as failure even if the owned VM was stopped, and setup
must not save a supposedly durable base afterward. Existing
base images are not silently overwritten. To recreate from scratch, stop the VM,
retain any needed host evidence, and explicitly remove **only this dedicated
state directory**, then repeat installation/setup. No VM images are committed.
Do not delete another VM directory or prune unrelated Tart state.

No host fallback exists. Missing/stopped VM, an unowned viewer launch, unavailable
guest agent, missing exact input, stale source, invalid artifacts or a lost GUI
session produces a nonzero result with guidance. Host GUI execution is not an
alternative merely because guest setup is inconvenient.

## Implementation evidence and limitations

On 2026-10-02 the pinned image was installed, provisioned, stopped, cloned as a
base, reset from that base and restarted **host-only, without a viewer**. Observed:
`VirtualMac2,1`, macOS **26.6.2 (25G83)**, arm64, 4 CPUs, 6 GiB RAM,
1024×768 reported guest display pixels, Python **3.14.7**, Rust **1.98.1**
(`48a229cea`), Apple clang **21.0.0**, Swift **6.3.3**, SDK **26.5**.
The exact engine/commit/executable/template hashes above matched. The OS matches
the existing candidate; this is not a claim that all historical acceptance has
been recertified for the VM or for rebuilt native artifacts.

Validation:

- **47 deterministic orchestration tests passed**: 10 host orchestration, 14 guest
  worker, and 23 Tart lifecycle tests; the **20 existing campaign tests passed**.
- Existing `run_script_edit.py --scenario clean-open`: **passed**, 10 group cases
  plus bootstrap, at `949919e2f2d90dae53fa66387194fbe5d8d11f09`.
  After immutable helper deployment was added, the unchanged group passed again
  against `2e64e4088bb72c06b51fd95193c1e5295683bb56`, using the current host wrapper
  and the recorded unchanged guest helper bytes; no native/Rust rebuild occurred.
- Existing `run_script_edit.py --scenario durability`: **passed**, 9 group cases
  plus bootstrap, at `2e64e4088bb72c06b51fd95193c1e5295683bb56`.
  This exercised ordinary Save, close/reopen, reparse, rescan, and its existing
  headless runtime-value witness without changing that witness or the GUI editor.
- A separate throwaway **non-headless runtime** displayed an independently
  observed guest window, produced a capture, and exited normally. This is
  infrastructure window evidence, not another semantic acceptance suite.
- The initial two passing groups yielded seven retrieved editor captures;
  representative editor and runtime images were visually inspected. A failed
  close group's completed-case capture was also retrieved and inspected without
  promoting the group to passed.
- Normalized host observations recorded **no new host Godot processes**, and
  passive host window observations found no Godot/Tart viewer windows during the
  observed runs. Independent baseline host processes were left untouched as
  explicitly requested. Guest provenance recorded the engine PIDs, graphical
  session and VM hardware; after the smokes no guest Godot process remained.
  Host terminal commands remained usable. No host focus API or viewer launch
  was invoked by automated execution.
- Native production/fault and Rust output hashes were unchanged between the
  focused runs; build receipts retained their original 17:59 UTC modification
  times. No VM recreation, tool reinstall or native/Rust rebuild occurred between
  the passing groups.
- `campaign close --keep-going` retained the existing unsupported-complete-close
  refusal and exit **2**, before GUI execution. No historical campaign was run.
- The VM was stopped after validation. Temporary runtime projects/control scripts
  and host observation binaries were removed; private evidence and the reusable
  VM/base remain.

The final helper-deployment change received its own focused GUI smoke and artifact
retrieval check with the obsolete mutable helper removed. Subsequent documentation
does not invalidate those executed inputs under the existing evidence-reuse policy.

Private evidence is under `~/.local/state/godot-agent-kit-vm/`:
`artifacts/vm-smoke-edit/`, `artifacts/vm-smoke-durability/`,
`artifacts/vm-smoke-close-4cpu/`, `artifacts/vm-campaign-refusal/`,
`artifacts/vm-smoke-worker-pinned/`, and `validation/runtime-window/`.
Raw project-bearing evidence is not committed or
uploaded with the PR.

**Historical PR #56 close failure:** `close --scenario clean-close` did not pass
as a complete group during infrastructure acceptance. At four CPUs the
selected-close, closed-observation and intentional-reopen cases passed, but the
next nonselected close failed the unchanged
`public_close_expected_outcome_public_close_nonselected` assertion. Its complete
result remained guest-private; the failed summary and completed selected-case
capture were retrieved. The two-CPU attempt had refused
`context_validation_unavailable` after 8.9 seconds. That PR did not establish the
remaining failure's cause or certify close acceptance.

The subsequent [focused cold-start regression investigation](../specs/005-close-project-gdscript/quickstart.md#11-vm-cold-start-close-regression)
reproduced the exact refusal and corrected VM startup readiness without changing
product code, assertions or deadlines. Retain the original failed evidence;
the new acceptance supplements it rather than rewriting the infrastructure run.

Bootstrap also exposed a concrete lifecycle bug: stopping Tart immediately after
dependency downloads left an incomplete cache after reboot. Guest filesystem
sync before stop corrected the observed recovery path; all 59 cached crates,
including `ring`, survived restoring the updated base and offline builds passed.
The image's unnecessary SSH and Screen Sharing services were disabled through
guest launchd. No host sharing, credential forwarding, Metal shim, GPU preference
override, hosted provider or CI requirement was added.

Implementation-shape review: `run_in_vm.py` owns the host CLI/source/evidence
boundary, `vm_tart.py` owns this one backend's lifecycle and vsock transport, and
`vm_guest.py` owns guest preflight/build/run/export. Existing runners own all
acceptance semantics. Helpers remain internal to test tooling, with no product
API, duplicated suite, generic executor interface or speculative backend.

### Cold-start readiness correction

An unforced cold-start nonselected close reproduced the original failure:
`unsafe_editor_context` at `validated`, `not_applied`, in 7.859 s. Private native
evidence identified `idle_parse_delay_exceeds_lease`: validation completed with
only 1.333 s left in the native lease, less than the unchanged 1.5 s configured
idle-parse delay. Independent target/protected documents, D/R/B, flags, versions
and selection were unchanged; no native close entered. The same binaries and
fixture passed in the settled guest. Guest CPU measurements identified competing
ordinary boot services, not a stale binary, wrong target or continuation defect.

The owning fix is in Tart startup readiness, not the close product. It reuses
the existing bounded loop and observes current headroom before admitting timed
acceptance. An initial one-minute-load predicate was rejected after its lag caused
an infrastructure timeout; no readiness deadline was extended to accommodate it.
Readiness is an admission observation, not a promise that arbitrary later load
cannot produce an existing truthful bounded product failure.

Principle XIII: the concrete failure is admitting timing-sensitive positive
fixtures during guest boot contention. Fixed sleeps do not establish readiness;
warming an extra editor or retrying the product would obscure the measured path;
extending the close budget or removing its native guard would weaken safety.
One existing-loop condition, provenance and focused lifecycle tests address that
failure without a new dependency, service, scheduler, product branch or supported
profile exclusion. Cost is a measured startup wait within the existing bound and
a small parser for standard guest CPU observations.

After the correction, the focused cold-start nonselected close passed in
**6.941 s**. The complete unchanged cold-start `clean-close` group passed
**134 records / 10 public close results**, nonselected **5.890 s**, maximum close
**6.278 s**. **37 focused VM orchestration tests** passed. The exact original VM
Rust and production/fault native build receipts were reused; no rebuild occurred.
The feature quickstart records source revisions, summary hashes, inspected
captures, no-host-disruption evidence and the scope of historical evidence reuse.
Feature 005 T003 remains pending and unstarted.
