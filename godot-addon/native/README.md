# Stock Godot GDScript validation and guarded editing (editor only)

T003 implements a private native application/persistence/finalization boundary
on exact official stock Godot 4.7.2 and a separate Rust-owned one-shot stock
LSP source validator. [T003 acceptance](../../specs/002-edit-open-gdscript/quickstart.md#11-t003-native-boundary-acceptance-2026-09-29)
records the previously executed stock primitive/export and observation cases.
T002's patched validator was accepted at the time but is superseded and removed
from HEAD; its implementation and evidence remain in
[PR #29](https://github.com/Peter-Tam/godot-agent-kit/pull/29) and
[historical acceptance](../../specs/002-edit-open-gdscript/quickstart.md#10-t002-native-validation-acceptance-2026-09-28).
There is no public edit command, bridge edit opcode or mutation advertisement:
T004 owns that caller integration.

## Build and private integration

Use the exact official `4.7.2.stable.official.ed1daf0bf` executable (SHA-256
`c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf`)
on the studied macOS arm64 environment. From the repository root, provide its
absolute path as `STOCK_GODOT`:

```sh
python3 godot-addon/native/build.py --godot "$STOCK_GODOT"
```

The build checks the exact binary and generated public GDExtension ABI, requires
the supported single-precision 64-bit `float_64` layout, rejects unsupported
precision/word sizes, extracts method hashes and compiles C++17 against public
generated headers only. It does not download or build Godot. Generated inputs
belong under ignored `godot-addon/native/build/`. The installed
`addons/godot_agent_kit/native/` contains `script_edit.gdextension`,
`libscript_edit.macos.arm64.dylib` and `build-manifest.json`. The manifest
records exact binary, ABI/header/API, source and library hashes, compiler/SDK
and the stable 64-hex native build ID; it has no patched-engine mode or patch
hash. The build ID derives from exact engine, ABI and source/configuration
inputs before compilation, so the library hash cannot recursively enter it.
Do not commit generated inputs, manifests or binaries.

The loader checks the running engine version/full commit and executable SHA-256.
`configure(session_id)` binds the actual project and existing 32-lowercase-hex
bridge or owned-fixture session; `close()` and editor shutdown release state.
Unmatched or missing binaries fail closed without changing observation.
Bridge v1 still exposes observation only; the matched stock artifact configures
its private integration within that session.

The editor-local `godot_agent_kit_native` metadata exposes `configure`, `close`,
`api_revision`, `build_id`, `edit_prepare`, `edit_advance`, `edit_cancel` and
`edit_expire`. Revision `1` is local metadata, not a public edit capability.
Stock exact-source validation runs in Rust's one-shot worker, not a native
validation fallback. `session.cpp` owns session/project/confinement support;
the native attempt is in `script_document.cpp` and registration in
`extension.cpp`. The [native contract](../../specs/002-edit-open-gdscript/contracts/native-integration.md#7-current-private-implementation-boundary)
records the bounded stages and result distinction.

The native directory has `.gdignore`: the editor plugin loads its installed
manifest explicitly through `GDExtensionManager`, while an unbuilt checkout
remains observation-only. The existing export hook/preset exclusions prevent
enabled, disabled and hook-only exports from including tooling/native artifacts
or a dangling runtime extension dependency.

## Official-stock boundary and verification

From the repository root, after setting the absolute executable and output paths:

```sh
python3 godot-addon/native/build.py --godot "$STOCK_GODOT"
cargo +1.98.1 build --manifest-path mcp-server/Cargo.toml --locked \
  --example stock_validation_fixture --bin observe-gdscript
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
acquired and rechecked by the trusted integration; the fixture uses public
ProjectSettings getters. T004 owns the authenticated caller/bridge binding.
Unsupported context and incomplete evidence remain unavailable, not invalid or
valid by omission. The stock endpoint's documented inherited limitation remains;
private staging is context isolation, not an OS sandbox.

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
