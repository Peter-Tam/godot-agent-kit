# Exact-source GDScript validation (editor only)

This directory implements T002's non-mutating validation primitive. It does not provide an edit command, writer, finalizer or authenticated bridge operation. The patched Godot parser/analyzer owns the validation result; the extension supplies an editor-bound native reader and checks its project/session/document/source binding. Real-editor, lifecycle, privacy and production-export evidence is recorded in [Feature 002's acceptance guide](../../specs/002-edit-open-gdscript/quickstart.md#10-t002-native-validation-acceptance-2026-09-28). Mutation A–E and edit durability remain pending.

## Reproduce the matched build

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

Use the actual executable filename output by SCons if it differs; `--godot` must point to that absolute patched editor executable inside the pinned source checkout. Do not use the stock installed editor or an unrelated engine binary for ABI generation. `build.py` checks the checkout base and reverse-applicability of the exact patch, queries the binary version, and generates `gdextension_interface.h`, `gdextension_interface.json` and `extension_api.json` **from that binary**. It requires the single-precision engine's 64-bit `float_64` ABI layout, rejects double/unknown precision and 32-bit layouts, extracts generated method hashes, and compiles both C++17 units against those generated public headers only.

Generated inputs live under ignored `godot-addon/native/build/`. The build installs `libscript_edit.macos.arm64.dylib` and `build-manifest.json` next to `script_edit.gdextension` in `addons/godot_agent_kit/native/`. The manifest records exact engine binary SHA-256, ABI/header/API hashes, patch hash, source hashes, compiler/SDK, native library SHA-256 and a stable 64-hex native build ID. That ID derives from exact engine, ABI and source/configuration inputs before compilation; the library's own hash cannot recursively enter its embedded ID. Do not commit generated inputs, manifests or binaries.

The loader checks the exact engine major/minor/patch/status/build/full commit **and SHA-256 of the editor executable currently running**, against the build-time pinned binary. `api_revision` is `1` only when the generated static GDScript parser API and editor association binding are present at their matched hashes. Missing methods make validation unavailable; stock observation remains separate. `configure(session_id)` takes only the existing authentic 32-lowercase-hex bridge/owned-fixture session ID and binds the current editor ProjectSettings project directory. The native metadata is editor-only and `close()`/editor shutdown release the session. A fixture without the product plugin may configure its own authentic session, not impersonate the bridge. Product plugin enable/disable does not add a bridge capability. Validation reads only confined existing `.gd` files and probes `.gd.remap` existence for fail-closed remap handling; it does not open arbitrary paths or invoke ResourceLoader itself.

The patched editor reports a different build label (normally `custom_build`) than the unpatched official binary. Existing observation bridge v1 explicitly requires the official build and its original descriptor; T002 does **not** widen, spoof or migrate that bridge. Run native GUI acceptance with the owned fixture's source-free session and without a bridge registry; exercise existing observation separately on its supported stock editor. A native artifact installed into a stock editor must fail closed with no configured native session while stock observation continues to work. Patched-editor observation or caller edit support is **not** claimed at T002.

The native directory contains `.gdignore`. The editor plugin explicitly loads the manifest through `GDExtensionManager` only when its library exists; an unbuilt checkout remains a working observation-only addon. Keeping the directory out of automatic discovery also prevents a tooling path from entering Godot's generated runtime extension registry. The existing export hook and preset exclusions remain in place. Enabled, disabled and hook-only exports must contain neither the native binary/manifest nor a dangling runtime dependency.

## Integration surface and acceptance

The editor-local `Engine` metadata key `godot_agent_kit_native_validation` holds native C-ABI callables `configure`, `close`, `validate`, `api_revision` and `build_id`. `validate(source, source_path, correlation)` delegates to the matched ClassDB binding with a native-only reader. This validation revision `1` is **not** the complete editing API family or an advertised bridge capability. The existing bridge v1 is unchanged.

The [native contract](../../specs/002-edit-open-gdscript/contracts/native-integration.md#4-primitive-b-exact-source-native-gdscript-validation) defines statuses, attribution and limits. The implementation uses call-local instances of Godot's existing parser/analyzer and shallow metadata, resolves nested dependencies relative to their referring script, and rechecks confined source identities/content and project context. It refuses unsupported effects rather than executing loaders, dynamic getters, scripted diagnostic stringification, initializers or constructors. Root D/R/B, dirty state and history are independently checked by the owned fixture; a parser result is not mutation success.

From the repository root, with absolute executable paths and an existing empty mode-0700 artifact directory:

```sh
python3 godot-addon/tests/run_script_edit.py \
  --godot "$EDIT_GODOT" --observer "$OBSERVER" \
  --scenario native-validation --artifacts "$NATIVE_ARTIFACTS"
```

Only `native-validation` is implemented in this runner. It includes native validation/lifecycle/privacy checks and actual native-present production exports. The complete edit runner, `native-primitives` aggregate, mutator and `all` remain later task scope, not successful aliases or stubs. Run the existing `run_observation.py --scenario all` separately on its supported stock editor. Hosted `ci.yml` builds the pinned patched engine and native ABI; it does not claim GUI acceptance. Exact executed build IDs, regression findings and support limits are in the acceptance guide.

## Licenses and dependency/security review

Godot source and its generated public GDExtension C header carry the Godot MIT license and copyright notice; the generated header is unmodified build output, not a vendored interface. The Godot checkout/engine binary is supplied by the operator, not downloaded by this build entrypoint. [Godot license](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/LICENSE.txt) and [public GDExtension ABI definition/header generator](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/core/extension/gdextension_interface.json) are the source/ABI provenance. Native C++ uses only C++17 standard headers, macOS system POSIX/ACL APIs and the system CommonCrypto SHA-256 implementation; there is no godot-cpp, external parser, runtime framework or engine-private binary linkage.

The separately installed build-time [SCons 4.10.1 PyPI metadata](https://pypi.org/pypi/SCons/4.10.1/json) lists MIT `license_expression`, Python >=3.7, no mandatory dependencies and an empty published-vulnerabilities field; its wheel SHA-256 is `bd9d1c52f908d874eba92a8c0c0a8dcf2ed9f3b88ab956d0fce1da479c4e7126`. This is a metadata review, not a guarantee that no vulnerabilities exist. [SCons' security guidance](https://scons.org/security.html) states that recipes execute Python and are **not sandboxed**: inspect and build only the selected owned checkout in an isolated venv, without escalated privileges. The [Godot security advisory page](https://github.com/godotengine/godot/security) did not list published advisories at review time; absence of a listing is not proof of safety. An actual release must review the current versions and exercised binary/SDK/patch hashes again.
