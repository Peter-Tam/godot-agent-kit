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
