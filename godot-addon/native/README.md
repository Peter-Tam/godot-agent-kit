# Native editor integration (editor only)

T003 implements a private native application/persistence/finalization boundary
on exact official stock Godot 4.7.2 and a separate Rust-owned one-shot stock
LSP source validator. [T003 acceptance](../../specs/002-edit-open-gdscript/quickstart.md#11-t003-native-boundary-acceptance-2026-09-29)
records the previously executed stock primitive/export and observation cases.
T002's patched validator was accepted at the time but is superseded and removed
from HEAD; its implementation and evidence remain in
[PR #29](https://github.com/Peter-Tam/godot-agent-kit/pull/29) and
[historical acceptance](../../specs/002-edit-open-gdscript/quickstart.md#10-t002-native-validation-acceptance-2026-09-28).
T004 adds the guarded `edit-gdscript` caller and bridge-v2 edit exchange.
Its [real-editor caller acceptance](../../specs/002-edit-open-gdscript/quickstart.md#13-t004-caller-acceptance-2026-09-29)
is complete; [T005 cumulative acceptance](../../specs/002-edit-open-gdscript/quickstart.md#14-t005-cumulative-acceptance-2026-09-29)
completes Feature 002 on the exact recorded stock/macOS arm64 candidate. No other
Godot version or platform is claimed.

Feature 003 adds guarded native opening and current-source validation in the
shared revision-2 `editor_integration` bundle. [T001 acceptance](../../specs/003-open-project-gdscript/quickstart.md#8-t001-native-boundary-and-cutover-acceptance-2026-09-30)
established its private native/slot boundary. T002 connects the explicit
`open-gdscript` caller to that same owner; `open_gdscript` is advertised only for
the matched complete family. Its [public-caller acceptance](../../specs/003-open-project-gdscript/quickstart.md#9-t002-public-caller-acceptance-2026-09-30)
and implementation-shape review are complete on the exact recorded candidate.
[T003 cumulative acceptance](../../specs/003-open-project-gdscript/quickstart.md#10-t003-cumulative-acceptance-2026-09-30)
completes Feature 003: full opening/edit/observation campaigns, repeated opening,
composed A–E/durability and all three export modes passed on that exact
environment. No wider support or roadmap Phase 1 completion is implied.

## Build and private integration

Use the exact official `4.7.2.stable.official.ed1daf0bf` executable (SHA-256
`c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf`)
on the studied macOS arm64 environment. From the repository root, provide its
canonical absolute path (not a symlink) as `STOCK_GODOT`:

```sh
python3 godot-addon/native/build.py --godot "$STOCK_GODOT"
```

The build checks the exact binary and generated public GDExtension ABI, requires
the supported single-precision 64-bit `float_64` layout, rejects unsupported
precision/word sizes, extracts method hashes and compiles C++17 against public
generated headers only. It does not download or build Godot. Generated inputs
belong under ignored `godot-addon/native/build/`. The installed
`addons/godot_agent_kit/native/` contains `editor_integration.gdextension`,
`libeditor_integration.macos.arm64.dylib` and `build-manifest.json`. The manifest
records exact binary, ABI/header/API, source and library hashes, compiler/SDK
and the stable 64-hex native build ID; it has no patched-engine mode or patch
hash. The build ID derives from exact engine, ABI and source/configuration
inputs before compilation, so the library hash cannot recursively enter it.
Do not commit generated inputs, manifests or binaries.
The builder removes obsolete kit-owned `script_edit` artifacts only after
matching their exact descriptor/library provenance; substituted or unrelated
files are preserved and cause refusal. Rebuild callers and addon together,
restart the editor and discard old session-bound edit bases. There is no
old-library alias or private-v2 fallback.

The loader checks the running engine version/full commit and executable SHA-256.
`configure(session_id)` binds the actual project and existing 32-lowercase-hex
bridge or owned-fixture session; `close()` and editor shutdown release state.
Unmatched or missing binaries fail closed without changing observation.
Bridge v5 authenticates nine capability bits and the installed revision-3 native
build. Editing, opening and closing require their matched complete native and
transport families; public observation/edit/open/discovery stay v1. The distinct
`close-gdscript` caller uses the authenticated `close_gdscript` capability.
The read-only discovery/observation owners do not require native mutation
availability. Rebuild/install Rust, addon and native peers together, then
restart the editor; v4 peers and revision-2 bundles have no fallback.

The editor-local `godot_agent_kit_native` metadata exposes `configure`, `close`,
`api_revision`, `build_id`, read-only `edit_inspect`, `edit_prepare`,
`edit_advance`, `edit_cancel` and `edit_expire`. The opening family is
`open_inspect`, `open_prepare`, `open_advance`, `open_verify`, `open_recheck`,
`open_finish`, `open_abort` and `open_expire`. The closing family is
`close_inspect`, `close_prepare`, `close_advance`, `close_status`, `close_verify`,
`close_recheck`, `close_finish`, `close_abort` and `close_expire`. Revision `3`
and the matched build ID are authenticated; unavailable tooling cannot
advertise editing or opening. Stock validation stays in the supervised Rust
helper, never a native fallback.
`session.cpp` owns one edit/open/close session owner; `document_guard` owns shared
read/identity/document checks. Editing and its Save-format/write/T0/saved-state
steps remain in `script_document.cpp`; opening stages remain in `script_open.cpp`.
Shared source/effective/compiled-context acquisition belongs to `editor_context`,
with opening-specific orchestration retained in `open_context`; guarded closing
and native continuation witnesses belong to `script_close`. Registration remains
in `extension.cpp`. The [closing native contract](../../specs/005-close-project-gdscript/contracts/native-integration.md),
[opening native contract](../../specs/003-open-project-gdscript/contracts/native-integration.md)
and [editing contract](../../specs/002-edit-open-gdscript/contracts/native-integration.md#7-current-private-implementation-boundary)
distinguish stage facts from independent verification.

The native directory has `.gdignore`: the editor plugin loads its installed
manifest explicitly through `GDExtensionManager`, while an unbuilt checkout
retains observation and read-only scope without native editing/opening. The
existing export hook/preset exclusions prevent
enabled, disabled and hook-only exports from including tooling/native artifacts
or a dangling runtime extension dependency.

## Guarded caller integration

Build `observe-gdscript` and `edit-gdscript` from `mcp-server/` with locked
resolution. Restart the editor with the updated addon to create a new v5 session, obtain a
fresh observation, and submit the complete basis plus replacement source through
stdin using the [caller contract](../../specs/002-edit-open-gdscript/contracts/edit-api.md).
The caller owns validation supervision and a 9.5-second operation budget with
0.5 seconds reserved for result delivery. It never retries, force-writes, opens
the target script or treats a native acknowledgment as verified success.

The caller runner groups are `clean-open`, `conflicts`, `routing`,
`interruption`, `validation`, `history`, `durability`, `sequential` and
`privacy-export`. Each requires the absolute `--editor` path to the built
`edit-gdscript`; `interruption` also requires the separate `--native-fault-addon`
artifact. `--scenario all` runs the complete edit/native/export suite and requires
`--editor`, `--stock-validator` and `--native-fault-addon`. Run the full observation
suite separately afterward. Use a new empty mode-0700 `--artifacts` directory
per invocation and an unlocked visible GUI session. State diagnostics without
owned-window captures do not pass these acceptance gates.

For a known closed script, build `open-gdscript` as well and invoke the distinct
[opening caller](../../specs/003-open-project-gdscript/contracts/open-api.md):

```sh
open-gdscript --registry "$REGISTRY" --project "$PROJECT" \
  --session "$SESSION" --script res://scripts/subject.gd
```

Opening reads no stdin and accepts no source, force, reload, focus or retry
option. An already-open result leaves dirty/divergent state and selection
unchanged; a newly-opened result requires separate D/R/B, dirty, Resource-edited
and protection evidence. A target syntax error is not an opening failure.
Always obtain a new ordinary observation before editing; an opening result is
not an edit basis. Possibly applied results require fresh observation of the
original target, not retry or assumed rollback.

The opening runner accepts `native-boundary`, `new-open`, `already-open`,
`preservation`, `routing`, `interruption`, `sequential`, `composed` and
`privacy-export`. `all` runs each group exactly once. Public groups and `all`
require `--opener` in addition to the existing absolute executable/fault-artifact
paths. The composed campaign uses product opening before the reused edit A–E
workflows; fixture-only human Save/close/history actions remain distinct from
product capabilities. Use the [opening quickstart](../../specs/003-open-project-gdscript/quickstart.md)
for exact commands, separate artifact directories and current acceptance limits.

Build `close-gdscript` for the explicit clean-document closing operation. Its
[caller contract](../../specs/005-close-project-gdscript/contracts/close-api.md)
accepts one bounded stdin object with `schema_version`, a fresh `request_id`,
and the complete prior observation-v1 `basis`. Only a fresh clean, exact-target
basis can authorize closing an open document. Explicit `basis: null` can
recognize an already-closed valid target without loading or closing it.

Closing never saves, discards, repairs, retries or compensates by reopening.
Verified new closure requires actual buffer absence, unchanged independent D,
unchanged original retained R or observed unloading, completed native
revalidation and preserved remaining documents. B is then not applicable,
not empty. Target-buffer native history may be disposed by ordinary closing;
unrelated history remains protected. Obtain a fresh ordinary observation
before any later effectful operation, especially after possible application.

The [closing runner](../../specs/005-close-project-gdscript/quickstart.md#3-owned-native-boundary-and-caller-groups)
provides `native-boundary`, `clean-close`, `already-closed`, `preservation`,
`routing`, `interruption`, `sequential`, `composed` and `privacy-export`. Public
groups require the real `--closer`, `--opener` and `--discoverer` in addition to
observer/editor/validator and separate fixture-fault inputs. Native-only
execution does not need these three callers. Close `all` runs all nine groups
once; complete campaign `--suite close` and `--suite all` are available.
[Cumulative acceptance](../../specs/005-close-project-gdscript/quickstart.md#12-t003-cumulative-acceptance-2026-10-02)
completes Feature 005 on the recorded candidate without a native change.
Local real-editor runs use the [owned VM](../../.github/LOCAL_VM.md), and
historical evidence reuse follows the explicit feature review rather than
unconditional full replay.


## Official-stock boundary and verification

From the repository root, after setting the absolute executable and output paths:

```sh
python3 godot-addon/native/build.py --godot "$STOCK_GODOT"
cargo +1.98.1 build --manifest-path mcp-server/Cargo.toml --locked \
  --lib --example stock_validation_fixture --bin observe-gdscript --bin edit-gdscript --bin open-gdscript
python3 godot-addon/native/build.py --godot "$STOCK_GODOT" \
  --fixture-faults --output-addon "$PRIVATE_FAULT_NATIVE"
python3 godot-addon/tests/run_script_edit.py \
  --godot "$STOCK_GODOT" --observer "$OBSERVER" \
  --scenario native-primitives \
  --stock-validator "$STOCK_VALIDATION_FIXTURE" \
  --native-fault-addon "$PRIVATE_FAULT_NATIVE" --artifacts "$ARTIFACTS"
```

All executable/output paths are absolute. `ARTIFACTS` must be an existing empty
mode-0700 directory. `STOCK_VALIDATION_FIXTURE` names the built Cargo example;
it is a test consumer, not a product CLI. `--fixture-faults` builds a separately
identified `GAK_FIXTURE` test artifact in its own directory; no fault callable
is compiled into the normal product library, and fixture artifacts cannot
replace it.

The native attempt retains exact object associations, source and current/saved
versions, Save formatting profile, project/leaf descriptors and original fd mtime.
Its six guarded stages apply one CodeEdit history operation, set/read Script R,
write/truncate/fsync/read the same fd, restore/read back T0 on that fd, clear the
public edited flag and tag the bound buffer. It then stops. Stage facts are not a
success verdict. Cancellation and synchronous editor callbacks cannot release the
shared collection/edit slot while native work is still entered; newer callback
text is checked before insertion and never overwritten to repair a partial edit.

The live-editor effect profile is deliberately narrower than the isolated
validator: script/global-class inheritance, tool scripts, preload/load references,
global-class registration and exported declarations refuse before history entry.
Ordinary editor export/validation can initialize a cold tool dependency; the
helper's parser-only result does not certify that continuation. Baseline and
desired source must both qualify, preserve exact LF UTF-8 and survive the current
Save profile. CR/NUL/BOM and Save-transforming representations refuse. No private
Godot timestamp, saved-handler call, broad Save, reload or rollback is used.

Rust captures a bounded no-follow source closure and supplied, project/session-
bound effective warning/global-class context before launch. The worker stages
only admitted sources, checks the exact official binary and its loopback listener,
requires each URI's diagnostics before its shaped symbol response, discards all
raw child output and owns teardown. Effective editor context must be freshly
acquired and rechecked by the trusted integration through authenticated
bridge-v5 ProjectSettings observations, not inferred from disk configuration.
Unsupported context and incomplete evidence remain unavailable, not invalid or
valid by omission. The stock endpoint's documented inherited limitation remains;
private staging is context isolation, not an OS sandbox.

Opening uses a read-only retained file and exact source/path methods on a new
unbound GDScript, initial compilation only for that new object, then native
document creation. It never writes source, Saves, clears edited state or tags a
buffer. Retained cached scripts are neither assigned nor recompiled. Its LF
source profile is not editing's Save-format eligibility check: exact whitespace
is preserved rather than normalized or rejected because a later Save may format it.

For a departing current GDScript, the native guard independently obtains equal
R/B plus dirty/history/compiled/context evidence. Purpose `open_context` validates
that private source even when it differs from current D. It rejects edit-proposal
input and binds completed validity to the exact request/session/document/guard.
The helper retains ordinary-string path-confinement checks and owned-child cleanup.
Display-category metadata is not a script variable or exported/Object property;
real variable metadata retains every effect guard and bound.


## Licenses and dependency provenance

Godot and its generated public GDExtension C header carry the Godot MIT license
and copyright notice. The generated header is unmodified build output, not a
vendored interface. The official executable is supplied by the operator;
`build.py` does not fetch Godot. [Godot license](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/LICENSE.txt)
and [public GDExtension ABI definition](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/core/extension/gdextension_interface.json)
record the upstream provenance. Native C++ uses C++17 standard headers,
macOS POSIX/ACL APIs and system CommonCrypto SHA-256; no godot-cpp, external
parser, runtime framework or engine-private binary linkage is required.
The historical T002 engine-build dependency review is archived in
[PR #29](https://github.com/Peter-Tam/godot-agent-kit/pull/29);
SCons and custom-engine builds are not part of current reproduction.
