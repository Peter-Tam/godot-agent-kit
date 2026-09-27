# Phase 0 Research: Live GDScript Observation

**Date**: 2026-09-26 | **Specification**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md)

All initial technical unknowns are resolved into the decisions below. This is a planning result, not implementation approval, full acceptance, or a compatibility claim. Research used public Godot/Rust documentation, pinned Godot source, released dependency metadata, and an isolated real GUI-editor experiment. No implementation from another agent/MCP project was used.

## 1. Public editor APIs and exact candidate version

**Decision:** Target Godot `4.7.2.stable.official.ed1daf0bf`, full engine hash `ed1daf0bf001b61586d9930840f2f1394092c079`, initially on macOS arm64. Use a thin GDScript EditorPlugin. The measured planning host was macOS **26.6.2**, arm64; Darwin's version is not the macOS product version. Other Godot versions return an unsupported-version outcome until separately verified.

Read `EditorInterface.get_script_editor()`, then `get_open_scripts()`, `get_open_script_editors()`, and `get_unsaved_files()` without opening/selecting a document. For the pinned engine, both open arrays follow the same tab order, but the Script array filters non-Script resources. Index association is permitted only when counts match, the script path is unique, every participating Script is valid, and before/after identities and ordering agree. Do not assume arbitrary future versions share this implementation detail. If association fails, preserve independently known open state and R, but report B/dirty attribution unavailable where it cannot be established. Never use node names, displayed titles, source similarity, or editor focus as identity.

- **R:** Read the enumerated `GDScript.source_code`. For a confirmed closed target, `ResourceLoader.get_cached_ref(path)` may return an already-loaded GDScript; a null reference means unavailable/unloaded. Never call `load()` to obtain R.
- **B:** Read the matched `ScriptEditorBase.get_base_editor()` only when it is an actual `CodeEdit`; read its text separately from R.
- **Dirty:** Match the safely attributed document path against `ScriptEditor.get_unsaved_files()`. The pinned implementation uses each editor's unsaved state and edited Resource path. A complete, attributable list can establish dirty or clean. An empty/nonunique path, incomplete capability, or unassignable indication means unknown, not clean. CodeEdit versions are additional witnesses, not the dirty-state definition.
- **D:** In the planned product, the Rust integration reads bounded UTF-8 bytes through a project-rooted filesystem capability. It does not rely on Godot's text decoding/line-ending behavior. The feasibility fixture's separate FileAccess read establishes only its known UTF-8/LF case.

**Rationale:** These are independent authorities available through public APIs, including an actual unsaved-work API. No GDExtension, editor-node traversal, source normalization, or forced resource load is required for positive clean/dirty external-script cases.

**Alternatives considered:** Current-tab-only reads miss non-selected documents. Comparing D and B invents dirty semantics. `ResourceLoader.load()` manufactures R. Guessing parallel-array alignment when text tabs exist can return another document. A native extension adds a build boundary without improving the demonstrated case.

**Sources:** [ScriptEditor 4.7](https://docs.godotengine.org/en/4.7/classes/class_scripteditor.html), [ScriptEditorBase](https://docs.godotengine.org/en/4.7/classes/class_scripteditorbase.html), [Script.source_code](https://docs.godotengine.org/en/4.7/classes/class_script.html#class-script-property-source-code), [ResourceLoader.get_cached_ref](https://docs.godotengine.org/en/4.7/classes/class_resourceloader.html#class-resourceloader-method-get-cached-ref), [pinned unsaved-state implementation](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp#L2408-L2418), [pinned enumeration implementation](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp#L3549-L3576), [official release](https://godotengine.org/download/archive/4.7.2-stable/).

## 2. Actual planning feasibility evidence

**Decision:** The positive clean/dirty planning gate in FR-018 is satisfied for the exact candidate above. Retain the following bounded claim; full implementation acceptance remains required.

An isolated project named `Isolated Observation Feasibility` was launched as an actual editor using:

```sh
godot --version
godot --editor --path "$FIXTURE" --single-window --display-driver macos \
  --rendering-driver opengl3 --log-file "$FIXTURE/engine.log"
```

The version output was `4.7.2.stable.official.ed1daf0bf`; `Engine.get_version_info()` supplied the full hash above. The GUI used the Compatibility renderer on Apple M2. A native screenshot of only the owned editor window was reviewed: the Script screen visibly showed `subject.gd(*)` and the unsaved fixture comment. This was not a headless-runtime or hidden-control substitute for visible-buffer evidence.

Fixture preparation, separate from observer reads, opened scripts and inserted the unsaved comment into CodeEdit. Define synthetic fixture strings:

- `S = "extends Node\nvar source_marker = \"DISK_CLEAN\"\n"`
- `U = S + "\n# FIXTURE_UNSAVED_CHANGE"`

| Prepared state | Independently read D | Independently read R | Independently read B | Unsaved paths | CodeEdit version / saved version |
|---|---|---|---|---|---|
| Clean, selected `subject.gd` | S | S | S | `[]` | `2 / 2` |
| Dirty, selected `subject.gd` | S | S | U | `["res://subject.gd"]` | `3 / 2` |
| Dirty, `other.gd` selected by fixture | S | U | U | `["res://subject.gd"]` | `3 / 2` |

Each state was passively read twice. Those read pairs preserved target Script/ScriptEditorBase/CodeEdit instance IDs, buffer text, version and saved version, caret, selection, undo/redo availability, and selected-script path. The dirty document retained `has_undo=true`. The change in R between prepared states demonstrates why R cannot be inferred from D or B; selecting the other script was a fixture action, not an observer action.

**Limits:** Six passive reads do not establish SC-005's twenty-read acceptance gate or every private navigation-history invariant. The fixture did not validate bridge authentication, confinement, deadlines, mixed text tabs, built-in scripts, missing/invalid/empty files, equal-text-but-dirty state, identity races, export exclusion, or CI. It proves the positive clean/dirty API feasibility only. Its editor process was stopped and disposable project removed; no product source or test suite was created.

## 3. Source, identity, time, and classification semantics

**Decision:** Use the evidence model in [data-model.md](data-model.md). Exact text comparisons operate on independently obtained Unicode values without trimming, newline conversion, or normalization. D is strict UTF-8 with bytes preserved by decoding, including a BOM if present; malformed encoding is unavailable, not lossy text. Empty strings are observed values.

Bind every request to a project filesystem identity, fresh editor-session ID, exact resource path, and observed document instance IDs when available. IDs and large counters serialize as strings, avoiding JSON floating-point truncation of Godot's 64-bit instance IDs. Do not create a reusable edit revision or authorization token.

Collect start/end witnesses for source, document identity/open state, dirty evidence, and CodeEdit version. Re-read D in bounded chunks for a direct comparison and compare its file identity/metadata before and after. Recheck editor state after D collection. Mark detected changes explicitly invalidated; do not mix a replacement document with the original. Even unchanged witnesses give only `no_change_detected`, not an atomic snapshot or future edit permission. Staleness stays unknown unless specific evidence identifies an unapplied change; divergence alone cannot do so.

**Rationale:** FR-008/009 distinguish complete observation from source agreement. Dirty and divergent are legitimate complete observations. Closed scripts have no B/dirty surface, but unloaded R is unavailable rather than not applicable.

**Alternatives considered:** A single aggregate success Boolean, source-equality-derived dirty state, previous-request caches, content hashes treated as mutation permissions, and atomic-snapshot claims all conceal uncertainty or expand scope.

## 4. Rust boundary and dependencies

**Decision:** One Rust package under `mcp-server/`, edition 2021, with a reusable library and the small `observe-gdscript` local caller. Pin toolchain **1.98.1** in `rust-toolchain.toml`; install rustfmt/clippy there. No MSRV promise. Track `Cargo.lock` and resolve/build/test with `--locked` after the initial reviewed lockfile is generated.

| Direct dependency | Selected baseline/features | Purpose | License/provenance |
|---|---|---|---|
| `serde` | `=1.0.229`, `derive` plus defaults | Typed adapter DTOs | MIT OR Apache-2.0; serde-rs |
| `serde_json` | `=1.0.151`, default `std` | Bounded JSON framing/results | MIT OR Apache-2.0; serde-rs |
| `cap-std` | `=4.0.3`, default features disabled | Project-directory-scoped D reads | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT; Bytecode Alliance |
| `ring` | `=0.17.14`, default features disabled, `std` enabled | HMAC-SHA256 proofs, constant-time verification, OS-backed caller nonces at T002's authentication boundary | Apache-2.0 AND ISC; briansmith/ring |

Use the standard library for TCP, monotonic deadlines, process supervision, and channels. Godot `Crypto` generates session IDs/secrets/nonces and HMAC-SHA256 proofs; Rust uses `ring::hmac` and `ring::rand::SystemRandom` at the private bridge boundary, not in the observation core. No MCP SDK, async runtime, HTTP/WebSocket, database, CLI framework, separate hash crate, or extra workspace crate is justified.

**Rationale:** Local Rust/Cargo 1.69.0 was observed, but is not the implementation target. The chosen Serde releases require newer Rust. Rust 1.98.1 fixes the preceding release's vtable miscompilation. `cap-std` avoids inventing security-sensitive directory traversal using hand-written unsafe syscalls. `ring` supplies the authentication primitives absent from Rust's standard library without hand-written cryptography or separate random/HMAC/hash dependencies; its C/assembly build requires the platform C toolchain.

**Dependency review:** Published manifests and upstream provenance were reviewed. The Windows `cap-primitives` sandbox advisory RUSTSEC-2024-0445 was fixed in 3.4.1; the proposed cap-std line depends on cap-primitives 4.x. The selected ring manifest declares Apache-2.0 AND ISC; retain its required notices and review transitive licenses. RUSTSEC-2025-0009 is patched in ring >=0.17.12, and the older-line maintenance advisory RUSTSEC-2025-0010 excludes >=0.17; 0.17.14 is outside both affected ranges. Upstream's latest-release-only patch policy requires checking current release/advisory status again when T002 introduces ring. This is not a resolved-lockfile vulnerability audit or project-license selection. Implementation must record transitive licenses/provenance and run an advisory scan on the actual lockfile before dependency acceptance. No project license is invented here.

**Alternatives considered:** Hand-written JSON, one crate per module, Tokio for a single bounded request, or unconfined `canonicalize` followed by ordinary `open` add risk or complexity without a requirement.

**Sources:** [Rust 1.98.1 release](https://blog.rust-lang.org/2026/09/03/Rust-1.98.1/), [rustup overrides](https://rust-lang.github.io/rustup/overrides.html), [serde manifest](https://docs.rs/crate/serde/1.0.229/source/Cargo.toml), [serde_derive manifest](https://docs.rs/crate/serde_derive/1.0.229/source/Cargo.toml), [serde_json manifest](https://docs.rs/crate/serde_json/1.0.151/source/Cargo.toml), [cap-std manifest](https://docs.rs/crate/cap-std/4.0.3/source/Cargo.toml), [cap-std Dir](https://docs.rs/cap-std/4.0.3/cap_std/fs/struct.Dir.html), [RUSTSEC-2024-0445](https://rustsec.org/advisories/RUSTSEC-2024-0445.html).

Authentication dependency sources: [ring 0.17.14 manifest](https://docs.rs/crate/ring/0.17.14/source/Cargo.toml), [ring HMAC sign/verify and random examples](https://docs.rs/ring/0.17.14/ring/hmac/index.html), [RUSTSEC-2025-0009](https://rustsec.org/advisories/RUSTSEC-2025-0009.html), [RUSTSEC-2025-0010 maintenance scope](https://rustsec.org/advisories/RUSTSEC-2025-0010.html), [Godot HMAC-SHA256](https://docs.godotengine.org/en/4.7/classes/class_crypto.html#class-crypto-method-hmac-digest).

### T001 resolved dependency and native-CI evidence (2026-09-26)

`cargo +1.98.1 generate-lockfile` created the tracked version-4 `mcp-server/Cargo.lock` (SHA-256 `e91b98922776713110b9c2f411a3b91b3da26fcd9646511c1a1ac3ac0ab5bd95`). `cargo +1.98.1 metadata --locked --format-version 1` resolved **one local library package and 50 registry packages**, including non-macOS transitives; the sole target is `godot_agent_kit` (lib), not a binary. Enabled features are `serde`: default/derive/std, `serde_json`: default/std, and `cap-std`: none (its defaults disabled). No additional crate is required for T001's reusable native semantics; `ring` remains T002-only, and the planned caller, authentication, disk reading, and Godot integration are not implemented by this lockfile.

The following groups enumerate **all 50 actual locked registry packages**. License expressions and upstream repository URLs come from each fetched package's Cargo metadata, not from the project's undecided license. `A` = `MIT OR Apache-2.0`; `B` = `Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT`; `C` = `Apache-2.0 WITH LLVM-exception`; `D` = `Unlicense OR MIT`; `E` = `(MIT OR Apache-2.0) AND Unicode-3.0`. The `AND` in E requires preserving the Unicode notice; C requires retaining the LLVM-exception terms. All indicated options are recognized open-source licenses; this does not choose a license for this project or replace a distribution-time notice review.

| Locked packages (exact versions) | License | Declared upstream repository |
|---|---|---|
| `serde`, `serde_core`, `serde_derive` 1.0.229 | A | [serde-rs/serde](https://github.com/serde-rs/serde) |
| `serde_json` 1.0.151 | A | [serde-rs/json](https://github.com/serde-rs/json) |
| `cap-std`, `cap-primitives` 4.0.3 | B | [bytecodealliance/cap-std](https://github.com/bytecodealliance/cap-std) |
| `ambient-authority` 0.0.2 | B | [sunfishcode/ambient-authority](https://github.com/sunfishcode/ambient-authority) |
| `fs-set-times` 0.20.3 | B | [bytecodealliance/fs-set-times](https://github.com/bytecodealliance/fs-set-times) |
| `io-extras` 0.19.0 | B | [sunfishcode/io-extras](https://github.com/sunfishcode/io-extras) |
| `io-lifetimes` 2.0.4 and 3.0.1 | B | [sunfishcode/io-lifetimes](https://github.com/sunfishcode/io-lifetimes) |
| `rustix` 1.1.5 | B | [bytecodealliance/rustix](https://github.com/bytecodealliance/rustix) |
| `rustix-linux-procfs` 0.1.1 | B | [sunfishcode/rustix-linux-procfs](https://github.com/sunfishcode/rustix-linux-procfs) |
| `linux-raw-sys` 0.12.1 | B | [sunfishcode/linux-raw-sys](https://github.com/sunfishcode/linux-raw-sys) |
| `winx` 0.36.4 | C | [sunfishcode/winx](https://github.com/sunfishcode/winx) |
| `bitflags` 2.13.2 | A | [bitflags/bitflags](https://github.com/bitflags/bitflags) |
| `errno` 0.3.14 | A | [lambda-fairy/rust-errno](https://github.com/lambda-fairy/rust-errno) |
| `ipnet` 2.12.2 | A | [krisprice/ipnet](https://github.com/krisprice/ipnet) |
| `itoa` 1.0.18 | A | [dtolnay/itoa](https://github.com/dtolnay/itoa) |
| `libc` 0.2.189 | A | [rust-lang/libc](https://github.com/rust-lang/libc) |
| `maybe-owned` 0.3.4 | A | [rustonaut/maybe-owned](https://github.com/rustonaut/maybe-owned) |
| `memchr` 2.8.3 | D | [BurntSushi/memchr](https://github.com/BurntSushi/memchr) |
| `once_cell` 1.21.4 | A | [matklad/once_cell](https://github.com/matklad/once_cell) |
| `proc-macro2` 1.0.107, `quote` 1.0.47, `syn` 3.0.6 | A | [dtolnay/proc-macro2](https://github.com/dtolnay/proc-macro2), [dtolnay/quote](https://github.com/dtolnay/quote), [dtolnay/syn](https://github.com/dtolnay/syn) respectively |
| `unicode-ident` 1.0.26 | E | [dtolnay/unicode-ident](https://github.com/dtolnay/unicode-ident) |
| `zmij` 1.0.23 | MIT | [dtolnay/zmij](https://github.com/dtolnay/zmij) |
| `windows-link` 0.2.1; `windows-sys` 0.59.0, 0.60.2, 0.61.2; `windows-targets` 0.52.6, 0.53.5; `windows_{aarch64,i686,x86_64}_{gnullvm,msvc}` 0.52.6 and 0.53.1; `windows_{i686,x86_64}_gnu` 0.52.6 and 0.53.1 (**22** entries) | A | [microsoft/windows-rs](https://github.com/microsoft/windows-rs) |

Every one of these 50 entries names `registry+https://github.com/rust-lang/crates.io-index` and has an individual SHA-256 `checksum` in `Cargo.lock`; all 50 fetched `.crate` archives in Cargo's crates.io cache were independently hashed and matched their respective lockfile checksums (50/50, zero missing/mismatches). Windows and Linux-only packages were included in this check, **not compiled or platform-tested**. The lockfile records both `io-lifetimes` versions and all Windows versions rather than concealing target-specific resolution. This is registry archive integrity and declared-repository provenance, not an independent audit of every upstream release process or a native compilation result.

Separately installed Homebrew `cargo-audit` **0.22.2** (its `cargo audit --version` printed `cargo-audit-audit 0.22.2`) ran `cargo audit --file Cargo.lock --json` against [RustSec advisory-db commit `e2111519ba6d14a5da59a7b2e5c8083ae8a37c01`](https://github.com/RustSec/advisory-db/commit/e2111519ba6d14a5da59a7b2e5c8083ae8a37c01), last updated 2026-09-25 19:51:57 +02:00: 1,271 known advisories; 51 lockfile packages; **0 reported vulnerabilities and no unmaintained/unsound/notice warnings**, with no ignored advisories. The previously reviewed `cap-primitives` Windows sandbox advisory affects older versions, not locked 4.0.3; absence of a warning is database-specific, not proof of all-platform safety. No warning has been suppressed. Adding `ring` at T002 requires a new full-lockfile license/provenance/advisory review.

Native workflow uses [GitHub's documented `macos-15` arm64 hosted label](https://docs.github.com/en/actions/reference/runners/github-hosted-runners), verifies product OS major, CPU architecture, compiler version, and Rust host tuple at runtime; [upstream checkout tag `v4.3.1`](https://api.github.com/repos/actions/checkout/git/ref/tags/v4.3.1) resolves to the pinned full commit `34e114876b0b11c390a56381ad16ebd13914f8d5`. It uses a read-only token with checkout credentials not persisted, no secrets or privileged GUI runner, and the exact locked native fmt/clippy/test/docs baselines. **These are workflow configuration and dependency-fetch/audit observations only; no native checks, CI run, GUI-editor test, or platform/editor support are claimed here.** Sources for review method: [Cargo metadata](https://doc.rust-lang.org/cargo/commands/cargo-metadata.html), [Cargo lockfiles](https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html), [RustSec cargo-audit](https://github.com/rustsec/rustsec/tree/main/cargo-audit).

### T002 resolved authentication dependencies and tool provenance (2026-09-26)

T002 adds only the planned `ring =0.17.14`, with default features disabled and
`std` enabled (`alloc` is enabled transitively by `std`). The resolved lockfile
contains one local package and **59 registry packages**; SHA-256
`dbba0e851819c51a4623e1b58b8cf7d583ff6ac372b66ea7e9f07b366db02fe1`.
The nine additional registry entries relative to the T001 review are:

| Package | Version | Declared license | Declared upstream |
|---|---|---|---|
| `ring` | 0.17.14 | Apache-2.0 AND ISC | [briansmith/ring](https://github.com/briansmith/ring) |
| `untrusted` | 0.9.0 | ISC | [briansmith/untrusted](https://github.com/briansmith/untrusted) |
| `getrandom` | 0.2.17 | MIT OR Apache-2.0 | [rust-random/getrandom](https://github.com/rust-random/getrandom) |
| `cc` | 1.5.1 | MIT OR Apache-2.0 | [rust-lang/cc-rs](https://github.com/rust-lang/cc-rs) |
| `find-msvc-tools` | 0.1.14 | MIT OR Apache-2.0 | [rust-lang/cc-rs](https://github.com/rust-lang/cc-rs) |
| `shlex` | 2.0.1 | MIT OR Apache-2.0 | [comex/rust-shlex](https://github.com/comex/rust-shlex) |
| `cfg-if` | 1.0.5 | MIT OR Apache-2.0 | [rust-lang/cfg-if](https://github.com/rust-lang/cfg-if) |
| `wasi` | 0.11.1+wasi-snapshot-preview1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | [bytecodealliance/wasi](https://github.com/bytecodealliance/wasi) |
| `windows-sys` | 0.52.0 | MIT OR Apache-2.0 | [microsoft/windows-rs](https://github.com/microsoft/windows-rs) |

`cargo metadata --locked --format-version 1` supplied these licenses, repositories,
and enabled features. All **59/59** downloaded registry archives matched the
individual SHA-256 checksums in `Cargo.lock`; non-macOS packages were included in
integrity/license review, not platform execution. Ring's packaged VCS metadata
declares commit `2723abbca9e83347d82b056d5b239c6604f786df` with `dirty: true`:
the archive checksum, not an assertion of a pristine Git checkout, identifies the
reviewed release. Its notices require retaining `LICENSE`, `LICENSE-other-bits`,
`LICENSE-BoringSSL`, and the bundled once_cell notices when distributing those
components. This does not choose the repository's license or add distribution work.

`cargo +1.98.1 audit --file Cargo.lock --json`, using cargo-audit **0.22.2** and
RustSec database commit `e2111519ba6d14a5da59a7b2e5c8083ae8a37c01` (1,271
advisories), reported **0 vulnerabilities and no warnings**, with no ignored
advisories, for all 60 lockfile packages. The [upstream release notes](https://github.com/briansmith/ring/blob/main/RELEASES.md)
list 0.17.14 as released and 0.17.15 as TBD at review time.
[RUSTSEC-2025-0009](https://rustsec.org/advisories/RUSTSEC-2025-0009.html) is
patched by >=0.17.12; [RUSTSEC-2025-0010](https://rustsec.org/advisories/RUSTSEC-2025-0010.html)
excludes >=0.17. The latest-release-only patch policy still requires future
advisory/release review; a clean database result is not a cryptographic audit.

Local tools: Rust/Cargo 1.98.1, Python 3.10.9, Apple clang 21.0.0
(`clang-2100.3.34.2`), macOS **26.6.2**, arm64. The exact Godot executable
SHA-256 is `c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf`.
Its application passed `codesign --verify --deep --strict` and `spctl --assess`:
notarized Developer ID **Prehensile Tales B.V. (6K46PWY5DM)**.

The missing macOS template was installed from the [official 4.7.2 release asset](https://github.com/godotengine/godot-builds/releases/tag/4.7.2-stable).
The complete `Godot_v4.7.2-stable_export_templates.tpz` SHA-256 matched the
release API's published digest
`f298490b8d44d934be425a5a65a51bf15f422428b229a06a6e11d9ffea248011`.
Its `templates/version.txt` is `4.7.2.stable`; extracted `macos.zip` SHA-256 is
`88df5e2e6fee99088699be66e6d42e4da4fb0c5619d054297d755a49558a4792`.
That archive supplies universal debug/release binaries, not an arm64-named
template. The fixture uses the universal preset and executes only on the tested
arm64 host; this earns no x86_64 support claim.


### T003 executor provenance (2026-09-26)

No dependency, feature, toolchain or package-version change accompanies T003.
The lockfile remains SHA-256
`dbba0e851819c51a4623e1b58b8cf7d583ff6ac372b66ea7e9f07b366db02fe1`;
T002's resolved license/provenance/advisory review therefore covers the same
dependency graph. This is not a claim that a new advisory audit ran.

The final executor smoke recorded Rust/Cargo 1.98.1, Python 3.10.9, macOS 26.6.2
arm64 and the same exact Godot version/full engine hash used by T002.
Godot executable SHA-256 remains
`c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf`;
its prior official-binary provenance is recorded above. The exercised observer
SHA-256 is `f1c0de92dc7424aaa0637bdfd827c682caca4631d5bac288cb8fa969aedfe564`,
and driver SHA-256 is
`2d834a5ad6ebac7c845f05df47af29e715fbe35414fc556bf621d92b44eb222b`.
No export templates or addon source changed in this task.

The owned GUI/session/executor evidence is in
[quickstart §2.3](quickstart.md#23-t003-executor-boundary-evidence-2026-09-26).
It establishes bounded refusal/execution and confined D, not real-editor source
collection or a wider supported-version/platform matrix.

### T004 collector and verification provenance (2026-09-26)

No dependency, feature, toolchain or package-version change accompanies T004.
The tracked lockfile still hashes to
`dbba0e851819c51a4623e1b58b8cf7d583ff6ac372b66ea7e9f07b366db02fe1`;
T002's recorded license/provenance/advisory review covers that unchanged graph.
No new advisory audit or additional platform support is claimed.

The exercised tools are Rust/Cargo 1.98.1, Python 3.10.9 and exact Godot
`4.7.2.stable.official.ed1daf0bf`, full engine hash
`ed1daf0bf001b61586d9930840f2f1394092c079`, on macOS 26.6.2 arm64.
Godot SHA-256 remains
`c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf`,
and the official `macos.zip` template remains
`88df5e2e6fee99088699be66e6d42e4da4fb0c5619d054297d755a49558a4792`.
Their official provenance is recorded under T002; these are fresh checksum
comparisons, not a new signing assessment. The built observer SHA-256 is
`09e34011ea641f298ffff1c1377c5cf0af1b9e5708972a4141c20622bda08e93`.
Per-run summaries identify the exact Python-driver hash exercised by each group.

Real GDScript instance IDs exposed their reference-counted high bit as negative
GDScript integers. [String.num_uint64](https://docs.godotengine.org/en/4.7/classes/class_string.html#class-string-method-num-uint64)
preserves that complete ObjectID as a decimal string accepted by the Rust model;
converting through float or stripping a sign would not. The independent fixture
also uses this representation. Clean-open acceptance catches the previous
signed-string rejection on actual Script objects.

Initial cap preparation observed native R changes during background validation.
The pinned [ScriptTextEditor::_validate_script](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_text_editor.cpp#L841-L880)
copies valid non-tool B into R. The cap-only fixtures therefore use a deliberately
syntax-invalid B: its failed native validation does not replace the independently
prepared R. Both R and B remain actual Godot objects read through the unchanged
product getters, with strict before/after source, dirty, version and history
assertions. No native processing is disabled and no source limit is bypassed.
Normal clean/empty US1 fixtures are unchanged. Visibility waits use the actual
owned native-window condition, not a keyboard-focus assumption or fixed sleep.

The new live workflow uses the existing full checkout SHA and
`actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02`,
resolved from the upstream `v4.6.2` tag. Actionlint 1.7.12 validates both workflows;
the dedicated self-hosted label is declared in `.github/actionlint.yaml`.
[Environment protection metadata](https://docs.github.com/en/rest/deployments/environments#get-an-environment)
is checked before any GUI job is eligible. Provisioning must follow
[GitHub's isolated, single-job runner guidance](https://docs.github.com/en/actions/reference/security/secure-use#hardening-for-self-hosted-runners).
At implementation time the repository had **zero environments and zero registered
runners**. No settings were changed and no trusted GUI-CI run is claimed.
See [quickstart §2.4](quickstart.md#24-t004-clean-open-evidence-2026-09-26) for local
behavioral evidence and the remaining CI/support boundary.

### T005 dirty-observation and verification provenance (2026-09-27)

T005 changes no dependency, feature, toolchain or package version. The tracked
lockfile remains SHA-256
`dbba0e851819c51a4623e1b58b8cf7d583ff6ac372b66ea7e9f07b366db02fe1`;
the existing T002 license/provenance/advisory review still describes that graph.
No new advisory audit or additional-platform support is claimed.

The exercised tools are Rust/Cargo 1.98.1, Python 3.10.9 and exact Godot
`4.7.2.stable.official.ed1daf0bf`, full hash
`ed1daf0bf001b61586d9930840f2f1394092c079`, on macOS 26.6.2 arm64.
Fresh Godot and `macos.zip` template checksums match T002's documented official
artifacts: respectively
`c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf`
and `88df5e2e6fee99088699be66e6d42e4da4fb0c5619d054297d755a49558a4792`.
The observer SHA-256 is
`257ff839ef5a8994ad0a07e9efd8f47be221889c27450b0daac8e951d49e1e74`.
The final four visible groups exercised driver
`2749cb9d057460ace64ace6b78cdc161da81574bae0545dfb011c07289d5ad8a`;
the earlier expanded privacy-guard replay and actual dirty-caller/registry
smoke exercised driver
`4bd45b03391e960e2837abbeb5b8334bd982c94c86f6432bfd3904a9203a764c`.
Every run summary records its own exact driver identity.

The real-editor evidence confirms three native details important to attribution:
tab switching may copy B into R even while B is dirty; opening deduplicates an
already-open path; closing a tab frees its ScriptEditorBase/CodeEdit. Fixtures
therefore sample actual R, prepare duplicate paths only after opening distinct
real resources, and verify native closure/replacement through actual object IDs.
The product tests Node validity before a typed dereference, avoiding the observed
freed-instance recheck failure. Fixture-only restrictions remove attribution,
not source truth, and the collector never uses those controls in production.
A native-close transition can coexist with a change to the independently held
Resource. Its live regression confirms that both changes must be reported;
closure cannot retain changed R as current. Nonunique loaded Resources retain
an unreadable-attribution reason rather than a false unloaded-state claim.

The added helper scripts are copied into the already excluded
`addons/fixture_driver/` tree in every disposable project, including all three
export variants. Actual ZIP/PCK inspection and release-app execution confirm
the boundary. Native tests and owned-window screenshots establish only the
documented local development increment; the unchanged protected trust boundary
still requires actual trusted GUI CI before support is advertised. See
[quickstart §2.5](quickstart.md#25-t005-dirty-and-changing-document-evidence-2026-09-27)
for counts, timing, the closure regression and explicit visual-evidence limits.

### T006 routing and interruption provenance (2026-09-27)

T006 changes no production API/protocol, dependency, feature, toolchain or
package version. The tracked lockfile remains SHA-256
`dbba0e851819c51a4623e1b58b8cf7d583ff6ac372b66ea7e9f07b366db02fe1`;
T002's license/provenance/advisory review covers that unchanged graph. No new
advisory audit or additional-platform support is claimed.

The exercised tools are Rust/Cargo 1.98.1, Python 3.10.9, Actionlint 1.7.12
and exact Godot `4.7.2.stable.official.ed1daf0bf`, full engine hash
`ed1daf0bf001b61586d9930840f2f1394092c079`, on macOS 26.6.2 arm64.
Fresh binary/template checksums match T002's documented official artifacts:
Godot `c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf`;
`macos.zip` `88df5e2e6fee99088699be66e6d42e4da4fb0c5619d054297d755a49558a4792`.
The built observer SHA-256 is
`a48dd733d318efc2d5e802f0fb03c77442965ef581ed2746daf003c71028b24a`.
Final privacy and clean-open runs exercised driver
`998bb8c1364dc4e0a39cdbc4054fd3f1edf7aad4e2d260458dedc47cfbc64665`;
the four US3 runs exercised
`636acd9c0f2f66a043eabcb90c6c20e4cf16f81cebced077a2e899dd7c31664c`.
Each run records its own exact driver identity; the final privacy replay adds
the namesake source markers to descriptor checks and checks the impostor's
nonce/proof against result/log leakage.

Real concurrent editors of the same project exposed distinguishable native
Resource/CodeEdit identities and contents under separate authenticated session
lifetimes. Fixture setup prepares those actual objects; it does not supply
positive values through a simulated bridge. Stage barriers at pending observe
and recheck permit deterministic loss/suspension without changing the product.
Native parser/editor behavior can update R from B, so syntax-invalid synthetic
B keeps the deliberately distinct routing authorities stable while getters
still read the real objects.

An unclean process stop can leave a descriptor, unlike normal plugin teardown;
the current listener must still prove secret possession before selection.
The live symlink-race preparation changes filesystem `ctime` by renaming the
owned source; separate displaced/restored-file and editor witnesses distinguish
that fixture action from observer interference. Exact deadline, privacy,
visual/export evidence and limits are in
[quickstart §2.6](quickstart.md#26-t006-routing-and-interruption-evidence-2026-09-27).
This remains local development evidence, not an executed trusted GUI-CI or
whole-feature supported-version claim.

### T007 closed and partial observation provenance (2026-09-27)

T007 changes no dependency, feature, toolchain or package version. The tracked
lockfile remains SHA-256
`dbba0e851819c51a4623e1b58b8cf7d583ff6ac372b66ea7e9f07b366db02fe1`;
T002's recorded license/provenance/advisory review covers this unchanged graph.
No fresh dependency audit or additional-platform support is claimed.

The exercised tools are Rust/Cargo 1.98.1, Python 3.10.9, Actionlint 1.7.12 and
Godot `4.7.2.stable.official.ed1daf0bf`, full engine hash
`ed1daf0bf001b61586d9930840f2f1394092c079`, on macOS 26.6.2 arm64.
Fresh checksums match the recorded official artifacts: Godot
`c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf`;
`macos.zip` `88df5e2e6fee99088699be66e6d42e4da4fb0c5619d054297d755a49558a4792`.
Passing US4 runs exercised observer
`a24b2c97752f461d2e4d8d32559b5940387f101d18162f749c834a2d1cecc294`
and driver
`cbb044743be1cc71c36e8f8d4b55272686f1f96005d3c2cded35c333cc58d8d2`.
Each retained summary records its actual identities.

Native GUI observation proved both open and cached closed built-in GDScript
identity without product loading; unresolved identity remains unsupported.
The container is a confinement locator, never D. Independent filesystem
presence/absence collection fixes closed/missing classification without
inventing source or editor knowledge. Source-limit metadata is retained from
the same safe read, avoiding a redundant filesystem acquisition.

Import-time caching made a pre-startup oversized fixture unsuitable for proving
unloaded R. Post-startup creation plus independent `get_cached_ref` witnesses
established the genuine closed/unloaded cases. Test-only surface restrictions
remove observability; native source and dirty state are never fabricated.
Synthetic syntax-invalid assets are excluded from enabled/disabled/hook-only
production exports. Exact regression, privacy, native-window/export evidence
and limitations are in
[quickstart §2.7](quickstart.md#27-t007-closed-and-partial-observation-evidence-2026-09-27).
This is local development evidence, not executed trusted GUI CI or a complete
supported-version matrix.

### T008 cumulative dependency and compatibility review (2026-09-27)

The dependency graph, package, toolchain and public version-1 contracts are
unchanged. A fresh `cargo +1.98.1 metadata --locked --format-version 1` resolves
one local package and **59 registry packages**, all with declared licenses and
upstream repositories. All **59/59** downloaded registry archives independently
match their `Cargo.lock` SHA-256 checksums. The lockfile remains
`dbba0e851819c51a4623e1b58b8cf7d583ff6ac372b66ea7e9f07b366db02fe1`.
The complete license/provenance inventory is the T001 table plus T002's nine
authentication dependencies above; no new license or distribution claim is made.

Fresh cargo-audit **0.22.2** reports **0 vulnerabilities, no warnings, and no
ignored advisories** for all 60 lockfile packages against RustSec commit
[`e2111519ba6d14a5da59a7b2e5c8083ae8a37c01`](https://github.com/RustSec/advisory-db/commit/e2111519ba6d14a5da59a7b2e5c8083ae8a37c01),
containing 1,271 advisories. The rechecked
[ring release notes](https://github.com/briansmith/ring/blob/main/RELEASES.md)
still list 0.17.14 as released and 0.17.15 as TBD. T002's required ring/Unicode/LLVM
notice obligations remain. Archive integrity and an advisory database result
are not audits of every upstream implementation or evidence of untested platforms.

The exercised official Godot executable again passes strict deep codesign
verification and notarized Developer ID assessment, with **Prehensile Tales B.V.
(6K46PWY5DM)** as signing identity. Its SHA-256 remains
`c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf`.
The installed official 4.7.2 `macos.zip` export template matches
`88df5e2e6fee99088699be66e6d42e4da4fb0c5619d054297d755a49558a4792`;
its release-archive provenance is recorded in T002 above.

Repository API inspection after the PR #19 rebase still found **zero environments
and zero registered runners**. No protection, registration or security setting
was changed. Both workflow files now come unchanged from merged PR #19:
the trusted workflow definition is sourced from `main`, and its tested checkout
can be the exact dispatch-main SHA or an independently approved eligible
same-repository PR head. Approval must name that exact SHA; it does not replace
the separate protected environment approval or isolated GUI runner. The former
main-only tested-revision restriction is no longer the blocker. See the
[shared CI procedure](../../.github/README.md). Hosted native/workflow checks
cannot establish visible-buffer or macOS 26.6.2 GUI support; actual trusted GUI
execution remains an unmet T008 completion gate.

The complete initial matrix and exact artifact identities are recorded in
[quickstart §2.8](quickstart.md#28-t008-cumulative-local-acceptance-and-unmet-ci-gate-2026-09-27);
[§2.9](quickstart.md#29-rebased-t008-local-verification-and-review-gate-2026-09-27)
records the full post-rebase rerun, unchanged checksums, fresh clean advisory
audit and the still-required exact-head review and protected GUI execution.
No schema, core-module ownership or product capability was added by this
acceptance cutover or rebase.

## 5. Local bridge, session bootstrap, and confinement

**Decision:** A per-editor `TCPServer` listens only on `127.0.0.1`. Choose a random high port and retry bounded bind collisions during plugin bootstrap; do not rely on undocumented port-zero behavior. The caller initializes a private registry outside the project. The addon must be explicitly enabled and receive that registry path through `GODOT_AGENT_KIT_REGISTRY`; absent/unsafe configuration disables the listener, not authentication.

A fresh 128-bit session ID and independent 256-bit pre-shared secret (`token` in the descriptor) are created on every plugin enable/editor lifetime. Publish only endpoint/identity/authentication metadata in a 0600 descriptor inside a verified 0700 owner-only registry; publish atomically after successful bind. No project source is persisted there. The Rust caller validates descriptor ownership, permissions, regular-file type, size, and schema. Both sides refuse unsafe configuration. The secret remains in the private descriptor and trusted memory, never a handshake field. The [bridge contract](contracts/bridge-protocol.md#3-source-free-selection-and-authentication) defines fresh client/server nonces and role-separated HMAC-SHA256 server, client, and finish proofs over a byte-exact session/project/request/version/capability-bound transcript. Verify the listener's prior possession of the secret before trusting its metadata, and complete mutual authentication before unique selection can authorize source observation. Under FR-016, this minimal local metadata is confined to the tool's own private state location and used only for discovery, identification, and routing to the intended editor session. It grants no access to outside-project source content or unrelated filesystem data.

**Planning security correction:** The former raw-token hello authenticated callers but not listeners. After an unclean editor exit, another OS user could bind the stale descriptor's released port, receive the secret/identity fields, and fabricate hello and editor evidence without reading the private registry. The mutual proof exchange closes that design gap; stale endpoint reuse, wrong/replayed/reflected proofs, changed transcript fields, and secret-free wire behavior are mandatory T002 regressions, replayed with source-bearing cases in T006. This revises an unimplemented initial version-1 proposal; no shipped protocol, legacy fallback, or new product capability is introduced.

**Planning-only proof-vector check:** Python's standard-library HMAC and OpenSSL 1.1.1t independently produced all three documented server/client/finish proofs for the 270-byte synthetic transcript. Twenty negative comparisons covered one change to each of its 13 fields (including both nonces), six cross-role substitutions, and a wrong secret; none matched the expected proof. This checks the documented byte encoding and role/field binding, not the future Rust/Godot bridge, endpoint-rebinding refusal, or live-editor acceptance. T002 still owes cross-language and actual-connection regression evidence.

The caller requires an explicit project root; session ID is optional only when metadata/liveness resolution proves exactly one candidate. Resolve all matching candidates without reading script source. Ambiguity returns only needed selection metadata. An unresolved candidate does not get silently discarded to select another. A requested ended session never resolves to its replacement. Listener lifecycle, descriptors, and exact selection are not broad multi-editor orchestration.

For D, use a project-root `cap_std::fs::Dir` and relative components, not ambient file opens after a lexical prefix check. Reject absolute/user/remote paths, traversal, symlink components, non-regular source files, and unsafe cross-user-writable roots/components. Revalidate identity/confinement before returning source. Open documents whose D has disappeared retain independently attributable R/B. Built-in paths use their in-project container for confinement, but its scene/resource text is never D; only already-present GDScript identity may establish such a target.

**Threat boundary:** Authentication protects against unauthenticated local connections and other OS users with normal permission isolation. It does not sandbox a malicious process already running as the same user or hostile project code already executing inside Godot. No adversarial same-UID B/R filesystem-race-proof guarantee is claimed: public Godot APIs expose Resource paths, not OS directory handles. Detectable/uncertain cross-project attribution must still be denied, not excused by this limit.

**Alternatives considered:** Wildcard/remote listeners, fixed global ports, unauthenticated loopback, raw-token client-only authentication, focus/PID-as-identity, tokens in project settings, and silently choosing the newest session are rejected. Echoing a secret after the caller disclosed it does not prove the listener knew it beforehand. Public GDScript TCP/Crypto APIs permit mutual proof without introducing native Unix-socket integration or a remote/TLS platform.

**Sources:** [TCPServer](https://docs.godotengine.org/en/4.7/classes/class_tcpserver.html), [Crypto](https://docs.godotengine.org/en/4.7/classes/class_crypto.html), [FileAccess permissions](https://docs.godotengine.org/en/4.7/classes/class_fileaccess.html), [DirAccess](https://docs.godotengine.org/en/4.7/classes/class_diraccess.html).

## 6. Deadline and framing

**Decision:** Use versioned, length-prefixed UTF-8 JSON with fixed bounds, one observation per connection, no cached results, and no automatic observation retry. [bridge-protocol.md](contracts/bridge-protocol.md) defines framing and identity validation. The addon polls partial nonblocking reads/writes with per-frame budgets, never blocking `get_data()`/`put_data()` on the editor thread.

The caller starts one monotonic deadline before resolution and supervises an isolated read-only worker process. At **4.5 seconds**, stop accepting new evidence and produce a timeout unless a stronger already-known refusal/disconnection applies; reserve the remaining interval for bounded result serialization/delivery so controlled requests finish within five seconds. The supervisor must not synchronously join potentially blocked disk I/O before returning. It terminates/reaps only its owned worker; no editor process is killed. Partial observations arrive as individually validated worker events before terminal assembly. The built-in worker is not arbitrary process execution exposed to a caller.

**Rationale:** Socket deadlines alone do not bound blocked filesystem calls. A long-lived unjoinable-thread-per-request design leaks work. An isolated worker is the small extra cost required to make the specified end-to-end deadline testable without an async platform or cancellation fiction. Core evidence/classification remains reusable and protocol-independent; supervision is host integration.

**Limits:** The five-second requirement is for a controlled local fixture with a consuming caller, not a hard-real-time operating-system guarantee or arbitrary blocked stdout sink. A per-source size limit makes only that source unavailable/`too_large`, without changing its applicability, truncating it, or discarding other independently observable facts or their earlier valid evidence. Absent a separate refusal/interruption, the result is `limited_observation` for an open or unknown-open document and `not_open` with explicit D/R limitations for a confirmed closed document. Operation-wide unsupported/refused outcomes are reserved for inability to perform the whole request safely or meaningfully, never per-source limits alone. Existing access-denial, invalidation, disconnection, and timeout precedence remains unchanged. Stale late replies cannot upgrade a terminal outcome. Cancellation does not imply rollback; this feature has no mutation to roll back.

**Sources:** [StreamPeer partial I/O](https://docs.godotengine.org/en/4.7/classes/class_streampeer.html), [StreamPeerTCP](https://docs.godotengine.org/en/4.7/classes/class_streampeertcp.html), [Godot JSON parsing caveats](https://docs.godotengine.org/en/4.7/classes/class_json.html), [serde_json deserializer](https://docs.rs/serde_json/1.0.151/serde_json/de/struct.Deserializer.html).

## 7. Testing, support, and export boundary

**Decision:** Use Rust unit/integration tests for model invariants, routing, framing, confinement, and deadline transitions; use a separate real-Godot GUI fixture driver for all live-state claims. Its preparation actions are not addon observation operations. Python 3.10+ standard library is sufficient for the acceptance driver; no Python runtime dependency is added to the product. Test entrypoints and outcomes are defined in [quickstart.md](quickstart.md).

Candidate coverage: `aarch64-apple-darwin` on macOS 15 hosted CI for native checks, plus exact-editor interactive acceptance on macOS 26.6.2 arm64, including a trusted GUI-capable CI/release runner. Do not claim either OS/editor combination supported until its applicable evidence is recorded. Pin any introduced GitHub actions to full commit SHAs, set least token permissions, and never run untrusted PR code on a privileged persistent GUI runner or with secrets.

Register an `EditorExportPlugin` that skips every addon-tree file. Production fixture presets also exclude the addon tree so disabling the editor plugin does not bypass exclusion. Verify both enabled and disabled pack exports, inspect artifacts, and launch the actual export to prove no listener/descriptor/tooling starts. Headless export generation is appropriate for this boundary, not proof of B. No gameplay autoload, runtime probe, npm package, signing, or distribution is added.

**Alternatives considered:** Mocks/headless runtime as live-buffer evidence, inferring export exclusion from `@tool`/`addons/`, broad version matrices based on similar APIs, or mutation A–E tests for an observer are rejected. Mutation A–E and applied-edit durability remain inapplicable only because this feature performs no mutation and claims no UndoRedo.

**Sources:** [EditorExportPlugin](https://docs.godotengine.org/en/4.7/classes/class_editorexportplugin.html), [EditorPlugin export registration](https://docs.godotengine.org/en/4.7/classes/class_editorplugin.html#class-editorplugin-method-add-export-plugin), [exporting projects](https://docs.godotengine.org/en/4.7/tutorials/export/exporting_projects.html), [command-line exports](https://docs.godotengine.org/en/4.7/tutorials/editor/command_line_tutorial.html), [GitHub hosted runners](https://docs.github.com/en/actions/reference/runners/github-hosted-runners).

## Resolution and evidence status

- Resolved: language/toolchain, direct dependencies and review obligations, API/dirty feasibility, private transport/authentication, project/session identity, bounded collection, initial platform targets, test framework, and export-isolation design.
- Observed now: exact local tool versions/platform; positive GUI clean/dirty/non-selected reads and native screenshot; read-pair non-interference for the listed witnesses.
- Required later, not unresolved design choices: implementation, resolved-lockfile audit, native checks, full 21 acceptance scenarios and edge cases, twenty-read stress, bridge/deadline/security/export tests, and exact-version CI. They remain mandatory gates before support or feature-completion claims.
