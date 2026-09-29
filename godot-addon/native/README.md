# GDScript validation and guarded editing (editor only)

T002's patched, non-mutating validator remains an accepted semantic oracle. T003 completes the official-stock native application/persistence/finalization boundary and a separate Rust-owned one-shot source validator. [Acceptance evidence](../../specs/002-edit-open-gdscript/quickstart.md#11-t003-native-boundary-acceptance-2026-09-29) records 149 stock primitive/export cases, 194 observation regressions, 54 oracle/export cases and the Rust/build checks. There is no public edit command, edit bridge opcode or mutation capability advertisement; those remain T004 work.

## Reproduce the patched oracle build

Use an owned checkout of Godot `ed1daf0bf001b61586d9930840f2f1394092c079` (4.7.2-stable) and macOS arm64 with Xcode command-line tools. The build does not fetch or distribute Godot. From this directory, substituting your absolute repository and checkout paths:

```sh
GODOT_SOURCE=/absolute/path/to/owned/godot
NATIVE_SOURCE=/absolute/path/to/godot-agent-kit/godot-addon/native
python3 -m venv /absolute/path/to/owned/build-venv
/absolute/path/to/owned/build-venv/bin/python -m pip install 'SCons==4.10.1'
git -C "$GODOT_SOURCE" checkout ed1daf0bf001b61586d9930840f2f1394092c079
git -C "$GODOT_SOURCE" apply --check "$NATIVE_SOURCE/engine-api.patch"
git -C "$GODOT_SOURCE" apply "$NATIVE_SOURCE/engine-api.patch"
(cd "$GODOT_SOURCE" && /absolute/path/to/owned/build-venv/bin/python -m SCons \
  platform=macos target=editor arch=arm64 dev_build=no debug_symbols=no optimize=none tests=yes \
  vulkan=no metal=no angle=no accesskit=no)
python3 "$NATIVE_SOURCE/build.py" --godot "$GODOT_SOURCE/bin/godot.macos.editor.arm64"
```

This command builds the **acceptance candidate** with `tests=yes`, compiling Godot's test code and the `TESTS_ENABLED`-gated fixed fixture Object sentinel needed for dynamic-effect refusal evidence. The fixture helper is absent in `tests=no` builds. Do not treat results for one binary/ABI/build manifest as results for the other; rebuild the extension with `build.py --godot` and rerun applicable acceptance for any different binary. No `tests=no` native runtime evidence is claimed here.

For the oracle, `--godot` must point to the absolute patched executable inside a checkout of the pinned base with the exact patch applied. `build.py` checks the base and reverse-applicability before generating the public ABI from that executable. For the official-stock build, use the command below instead: no source patch or custom editor is required. Both modes require the single-precision 64-bit `float_64` ABI layout, reject double/unknown precision and 32-bit layouts, extract generated method hashes, and compile C++17 against generated public headers only.

Generated inputs live under ignored `godot-addon/native/build/`. The build installs `libscript_edit.macos.arm64.dylib` and `build-manifest.json` next to `script_edit.gdextension` in `addons/godot_agent_kit/native/`. The manifest records exact engine binary SHA-256, ABI/header/API hashes, patch hash, source hashes, compiler/SDK, native library SHA-256 and a stable 64-hex native build ID. That ID derives from exact engine, ABI and source/configuration inputs before compilation; the library's own hash cannot recursively enter its embedded ID. Do not commit generated inputs, manifests or binaries.

The loader checks the engine version/build/full commit and SHA-256 of the running executable. The oracle additionally requires its patched parser and association methods; stock uses only generated public editor methods. `configure(session_id)` binds the actual project directory and existing 32-lowercase-hex bridge or owned-fixture session. `close()` and editor shutdown release native state. An unmatched binary fails closed without changing observation.

Bridge v1 still accepts only observation and still requires its supported official build. The matched stock artifact can configure its private native integration in that session; a patched-oracle artifact cannot substitute for it. Patched-editor bridge support and caller editing remain unimplemented.

The native directory contains `.gdignore`. The editor plugin explicitly loads the manifest through `GDExtensionManager` only when its library exists; an unbuilt checkout remains a working observation-only addon. Keeping the directory out of automatic discovery also prevents a tooling path from entering Godot's generated runtime extension registry. The existing export hook and preset exclusions remain in place. Enabled, disabled and hook-only exports must contain neither the native binary/manifest nor a dangling runtime dependency.

## Integration surface and acceptance

The editor-local `godot_agent_kit_native_validation` metadata contains common `configure`, `close`, `api_revision` and `build_id` callables. The oracle additionally exposes `validate(source, path, correlation)` through its patched binding. Matched stock mode exposes private `edit_prepare`, `edit_advance`, `edit_cancel` and `edit_expire`; stock exact-source validation instead runs in Rust's one-shot worker. Revision `1` is local metadata, not an advertised bridge editing capability. The existing bridge protocol remains v1.

The [native contract](../../specs/002-edit-open-gdscript/contracts/native-integration.md#4-primitive-b-exact-source-native-gdscript-validation) defines statuses, attribution and limits. The oracle uses call-local instances of Godot's parser/analyzer and shallow metadata, resolves nested dependencies relative to their referring script, and rechecks confined source identities/content and project context. Stock uses the separate Rust-owned helper described below, not a native validation fallback. Root D/R/B, dirty state and history are independently checked by the owned fixture; a parser result is not mutation success.

From the repository root, with absolute executable paths and an existing empty mode-0700 artifact directory:

```sh
python3 godot-addon/tests/run_script_edit.py \
  --godot "$EDIT_GODOT" --observer "$OBSERVER" \
  --scenario native-validation --artifacts "$NATIVE_ARTIFACTS"
```

`native-validation` remains the patched oracle suite. `native-primitives` runs the official-stock helper, native finalization and export groups; it requires the fixture inputs below and an unlocked desktop. `all` deliberately refuses until T004/T005 provide their remaining caller and cumulative coverage. Run existing `run_observation.py --scenario all` separately with the stock artifact installed. Hosted `ci.yml` builds both oracle and official-stock ABI boundaries; it does not establish GUI acceptance.

## Official-stock boundary and verification

Use official `4.7.2.stable.official.ed1daf0bf`, executable SHA-256
`c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf`,
on the studied macOS arm64 environment. From the repository root:

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
it is a test consumer, not a product CLI. Fault builds require a separate output
directory, identify themselves in their manifest, and cannot replace the normal
installed product library. No fault callable is compiled into that product library.

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


## Licenses and dependency/security review

Godot source and its generated public GDExtension C header carry the Godot MIT license and copyright notice; the generated header is unmodified build output, not a vendored interface. The Godot checkout/engine binary is supplied by the operator, not downloaded by this build entrypoint. [Godot license](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/LICENSE.txt) and [public GDExtension ABI definition/header generator](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/core/extension/gdextension_interface.json) are the source/ABI provenance. Native C++ uses only C++17 standard headers, macOS system POSIX/ACL APIs and the system CommonCrypto SHA-256 implementation; there is no godot-cpp, external parser, runtime framework or engine-private binary linkage.

The separately installed build-time [SCons 4.10.1 PyPI metadata](https://pypi.org/pypi/SCons/4.10.1/json) lists MIT `license_expression`, Python >=3.7, no mandatory dependencies and an empty published-vulnerabilities field; its wheel SHA-256 is `bd9d1c52f908d874eba92a8c0c0a8dcf2ed9f3b88ab956d0fce1da479c4e7126`. This is a metadata review, not a guarantee that no vulnerabilities exist. [SCons' security guidance](https://scons.org/security.html) states that recipes execute Python and are **not sandboxed**: inspect and build only the selected owned checkout in an isolated venv, without escalated privileges. The [Godot security advisory page](https://github.com/godotengine/godot/security) did not list published advisories at review time; absence of a listing is not proof of safety. An actual release must review the current versions and exercised binary/SDK/patch hashes again.
