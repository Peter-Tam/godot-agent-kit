# Research: Trusted GDScript Workflow Through MCP

**Date:** 2026-10-03 | **Baseline:** `d8fa1b6085fcaf0d8aba19ff5593f506822ed9b1`

**Disposition:** R1–R5 have selected designs below. This is source-backed planning with a scoped stock-editor mechanics experiment, not an implemented closed-edit capability, MCP interoperability pass or product acceptance. The maintainer approved constitution v1.2.0 and the aligned specification before this planning invocation.

## 1. Research questions and evidence

Three independent investigations covered native closed editing, reusable execution/contracts and released MCP/SDK/client choices. Integration review checked relevant source, used Rust LSP references, observed installed client versions/help, and performed the owned VM experiment in §2. No repository product code, dependencies or permanent tests changed.

| Question | Decision |
| --- | --- |
| R1 — closed route, Resource applicability, races, postconditions | Godot-main-thread guarded native closed-source transaction: passive cache/absence evidence, explicit source setter only for an existing clean retained Script, confined descriptor persistence, independent source/absence verification. |
| R2 — protocol, SDK, transport and runtime | MCP 2025-11-25, local stdio, official `rmcp =3.5.0`, minimal Tokio `=1.53.1`; bounded adapter transport rather than the SDK's unbounded convenience reader. |
| R3 — basis, models and ownership | Self-contained complete prior read result, discriminated open/closed eligibility; existing open basis and execution retained. New closed revision/epoch evidence belongs below MCP. No basis-token store or capability registry. |
| R4 — clients and acceptance | Codex CLI 0.153.4 and Claude Code 2.1.222; existing owned VM and fixture witnesses, with a narrow test-only stdio relay for host clients. New adapter/closed evidence and MCP A–E, not automatic historical replay. |
| R5 — deadlines, cancellation and delivery | Original absolute operation clock, bounded off-executor supervision, cooperative per-call cancellation, owned cleanup and deadline-aware output. SDK cancellation/draining is not transaction completion. |

## 2. Closed editing: selected route and limitations

### Source-established primitives

The exact supported engine remains official Godot `4.7.2.stable.official.ed1daf0bf`, full commit `ed1daf0bf001b61586d9930840f2f1394092c079`.

- [Public Script source contract](https://docs.godotengine.org/en/4.7/classes/class_script.html#class-script-property-source-code) explicitly says setting source does not reload the class implementation. [Pinned direct setter](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/modules/gdscript/gdscript.cpp#L446-L460) only changes source and source-changed bookkeeping. The private `_script_source` property path additionally reloads; it is not the selected method.
- Existing [native opening](../../godot-addon/native/script_open.cpp) pairs `ResourceLoader.has_cached` and `get_cached_ref`, validates types/path/identity and distinguishes confirmed absence from unavailable/inconsistent evidence without loading. [Native closing](../../godot-addon/native/script_close.cpp) already observes retained or absent R and actual document absence.
- Existing [document guards](../../godot-addon/native/document_guard.cpp) pin confined project/file descriptors and recheck namespace attachment. [Open-edit native persistence](../../godot-addon/native/script_document.cpp) already performs exact-descriptor write/truncate/fsync/pread and same-descriptor original-mtime restoration. Its open-document guard is not reused unchanged for closed targets.
- [Pinned native close](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/script/script_editor_plugin.cpp#L540-L600) calls `notify_script_close` before disposing each closing script document. The public [script_close signal](https://docs.godotengine.org/en/4.7/classes/class_scripteditor.html#class-scripteditor-signal-script-close) supports a session-bound close epoch: a closed→open→closed interval cannot silently reuse a pre-close basis. Current roster checks detect a target that remains open. Epoch reset/overflow invalidates old bases; this is one counter and an owned signal registration, not a history service.
- The existing `editor_effects_inert` admission rejects tool scripts, exported defaults, global-class registration, non-built-in bases and load/preload references. Reuse the applicable source/effective/compiled-context checks and exact-source validator, not a new parser or a claim that parse validity alone implies safe effects.

### Owned stock-editor observations

Executed one completed research run, `closed-source-94wofo8i`, in the existing Tart 2.40.1 VM on **macOS 26.6.2 (25G83), arm64**. Exact engine executable SHA-256: `c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf`. A disposable EditorPlugin supplied public getters and an explicit `Script.set_source_code` call; a fixture-only Python child performed retained-descriptor persistence in that same editor turn. This child is experimental apparatus, not the selected production writer.

The initial and intended sources were an inert `extends Node` script with `const VALUE: int = 17` and a method returning that constant, changed to `23`. Initial UTF-8 SHA-256: `c950ac91d2a9bfaf5bcd6f2b0fe1797b3cbcf6e39a57cf36546fa60d12952458`; intended SHA-256: `108068471f7bfde76a3760c3818b46d547daacec8247f7b1bb6ee1f11078b356`.

| Observation | Actual result |
| --- | --- |
| Clean cached target, never opened | Explicit setter plus descriptor persistence produced independent D/R source agreement, same Script identity, edited=false, unchanged inode and restored original mtime. No open script or unsaved document appeared immediately or on the next frame. |
| Confirmed uncached target, created after startup | Paired cache getters remained false/null before and after persistence; intended D was observed and no buffer or Resource was manufactured. |
| Negative control: cached target, descriptor write without setter | D changed but retained R source stayed old. This demonstrates why disk-only writing is not the closed-edit route. |
| Later ordinary fixture opening | Both positive targets yielded intended D/R/B. Opening occurred only after the closed-state observations, not to make editing eligible or verify its initial result. |
| Fresh runtime from persisted source | A separate owned headless runtime loaded both scripts and observed `VALUE == 23`, exit 0, empty stderr. This is a persistence witness, not visible-editor evidence. |
| Loaded class implementation | Its cached constant map still contained `17` after source assignment and the immediate opening witness. Source agreement is not a claim of live class reload or running-game hot reload. |

The owned guest screenshot was inspected; it shows the disposable real editor without an open script document during the closed-state observation. Evidence JSON SHA-256: `1c6409d7d15adf0f759fb60f9b88c7b724ab19f7087731cb847db7878750f71f`; screenshot SHA-256: `953f099714d7e5358da318a4f468102e71b9ec062a18a036b9c50585a8d02289`; experiment source SHA-256: `1e785ece2f2a9777ddb30a05a3c7163a1add50e692b6c538b89b7bee3bd82152`.

Two preceding incomplete probe runs are retained separately: one fixture type-inference error and one capture wait timeout. Neither is a passing campaign or product acceptance. Disposable projects/control scripts were removed; private logs/results/capture remain outside Git. All owned editors/runtimes exited and the VM was restored to stopped state. No host Godot was launched, native library built or historical campaign replayed.

### Decision

Select a narrowly scoped native closed-source owner in the existing editor integration. Support both (a) existing readable clean cached GDScript with actual R source equal to D and (b) positively observed canonical cache absence. Capture expected D identity/content/revision, actual lifecycle and loaded-R applicability; validate intended source in the existing isolated stock validator; recheck on the Godot main thread before effects. For present R, use only the explicit public source setter on the retained object. Persist inside the native editor boundary using the existing confined descriptor mechanics and restore original mtime; do not introduce an external production writer. Independently reacquire D/R/applicability and actual absence before success.

The absent-R branch is not a disk-write fallback: it requires live Godot-authoritative absence/lifecycle/epoch admission, the same validation, native ownership, confinement, fresh guards and independent postconditions. An unavailable Resource witness refuses instead. Any change of branch after read or admission refuses/invalidate; no automatic open/close, load, cache eviction, retry, compensation or hoped-for reload.

As [the specification](spec.md#user-scenarios--testing-mandatory) defines it, R is loaded **Script source**. No live bytecode/static-state refresh is promised. Preserve this limitation explicitly; later ordinary reparse/rescan and fresh runtime durability require their own acceptance. Do not call reload/update_exports/Save or clear dirty flags to manufacture agreement. If implementation cannot meet these specified guards/postconditions, return to the product/spec decision instead of shipping refusal-only closed editing.

**Alternatives rejected:** unchanged Feature 002 open-only execution; implicit open-edit-close; ResourceSaver/pathname writes (existing confined-write research already demonstrated wrong-target/fallback risks); an external writer followed by rescan/reload; forced Resource loading; engine patches; generic transaction machinery. The selected composition has demonstrated primitives and a concrete native location, but the complete guarded state machine, races, failures and timing remain construction/acceptance obligations—not already-proven behavior.

## 3. Reusable core and basis

**Decision:** Preserve all existing local v1 contracts and public open-only basis semantics. Reuse `runner::run`, `runner::discovery::run` and `runner::edit::run`, their authenticated resolution, confined acquisition, bounded children, effect reduction and disclosure. The parent LSP reference check found three references to edit `run` and 34 to `ExpectedRevisionBasis`; do not casually replace these public contracts or duplicate their policies.

A new protocol-independent script-read result combines the existing observation with an explicit edit-basis eligibility record and, for closed targets, independently acquired native closed-state evidence. The caller passes that complete read result unchanged as `basis`. Open requests use the existing strict observation decoder and checked basis; closed requests use a distinct checked closed basis and native path. Native witnesses never become caller-selected methods. Read remains useful when no edit basis can be issued.

**Rationale:** A full self-contained prior observation already has defined stale/conflict/disclosure semantics. Reusing it avoids a new opaque-token store, mutable per-client revision cache, expiry/replay protocol or generic operation facade. No extra copy of that observation is returned as a second basis payload. Adapter code may borrow the actual observation portion when constructing the existing bounded open-worker input; serialization required by the worker boundary is not permission to repeatedly parse/clone large source trees.

**Closed revision:** Session/project/path, independently observed file identity and exact source witness, nanosecond mtime/ctime, confirmed absence of B, session close epoch, and present clean R identity/source or confirmed absence. Native admission rechecks all applicable facts. Closed bases carry no invented buffer version/history or dirty=false buffer. A close signal anywhere in the session conservatively invalidates prior closed bases; this avoids a per-target registry. Same-text file writes still change ctime. Unsupported/unobservable mutation histories are not claimed atomically excluded.

**Alternatives rejected:** treating a digest as permission, trusting client cleanliness assertions, converting closed observations into open bases, stateful basis-token infrastructure solely to shorten arguments, or changing Features 001–005 local JSON meanings. Safety remains current-boundary checks below MCP, regardless of how a caller supplies the expected facts.

## 4. Released protocol, dependencies and clients

### Protocol and process

**Decision:** One client-owned local stdio server binary, `godot-agent-kit-mcp`, in the existing Cargo package. Select published **MCP 2025-11-25** only. It is a compatibility target, not the latest revision. Use its initialize/version negotiation and normal tools/list/tools/call; advertise only tools, not logging/resources/prompts/tasks/sampling/elicitation/roots. A fixed catalog has no listChanged or dynamic paging. Exact names: `discover_scripts`, `read_script`, `edit_script`.

[2026-07-28](https://modelcontextprotocol.io/specification/2026-07-28/changelog.md) removes initialize, introduces per-request metadata/server-discover and changes cancellation/result semantics. Do not accidentally enable that era through SDK defaults, conflate it with the selected handshake or claim both versions supported. A modern discovery probe may receive source-free selected-version information, but cannot dispatch an editor operation; concrete client fallback must be verified before compatibility is claimed.

**Alternatives:** HTTP brings listener, Origin/authentication, HTTP/session and additional dependency obligations without a local workflow need. A handwritten MCP implementation transfers versioned negotiation/message/cancellation maintenance into this repository. Supporting multiple revisions immediately adds unrequired compatibility paths. The released official SDK with one explicit revision is the smaller maintained choice.

### Dependencies and provenance

| Selection for later implementation | Evidence and reason |
| --- | --- |
| `rmcp = "=3.5.0"`, default features off, `server` | [Stable release](https://github.com/modelcontextprotocol/rust-sdk/releases/tag/rmcp-v3.5.0), [manifest](https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.5.0/crates/rmcp/Cargo.toml), [package provenance](https://docs.rs/crate/rmcp/3.5.0/source/.cargo_vcs_info.json), commit `0cde3c5cf3e6aff0cc852ce6045f107e95991f48`. Official Rust SDK; current handlers/catalog implemented directly, no tool macros. `server` already includes async-rw/schemars/pastey/uuid transitively. No separate `transport-io` feature is needed by the custom transport. |
| `tokio = "=1.53.1"`, default features off, `rt`, `time`, `sync`, `io-util`, `io-std` | [Release](https://github.com/tokio-rs/tokio/releases/tag/tokio-1.53.1), [manifest](https://github.com/tokio-rs/tokio/blob/tokio-1.53.1/tokio/Cargo.toml). A current-thread protocol runtime; synchronous bounded supervisors run off-executor and remain owned. Reuse the existing Darwin atomic signal-handler pattern rather than add signal/network/process runtime features. |

rmcp declares edition 2024/MSRV 1.88; Tokio declares edition 2021/MSRV 1.71. The existing Rust 1.98.1 application can consume these without changing its edition or claiming a new MSRV. SDK serde/serde_json ranges admit the repository's existing pins. This is manifest compatibility, not compiled dependency resolution; no Cargo files were changed or packages installed.

The [SDK LICENSE](https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.5.0/LICENSE) describes an MIT→Apache-2.0 transition, retaining MIT for unconsented historical contributions; preserve both applicable notices, not an inaccurate all-Apache claim. Documentation licensing is separately CC-BY-4.0. Tokio is MIT. Both software license families are permissive; inspect the eventual locked transitive graph and retain required notices before distribution. No project-license change is made here.

Public advisory research reviewed SDK [session leak](https://github.com/modelcontextprotocol/rust-sdk/security/advisories/GHSA-9pj6-vhgr-3mwh), [OAuth metadata SSRF](https://github.com/modelcontextprotocol/rust-sdk/security/advisories/GHSA-c9xm-49cp-xcr9), [cross-origin header leak](https://github.com/modelcontextprotocol/rust-sdk/security/advisories/GHSA-9g45-5xwm-f3wc), [DNS rebinding](https://github.com/modelcontextprotocol/rust-sdk/security/advisories/GHSA-89vp-x53w-74fx) and [OAuth resource validation](https://github.com/modelcontextprotocol/rust-sdk/security/advisories/GHSA-33f5-2c5q-wgwj). Version 3.5.0 is outside their retrieved affected ranges, and HTTP/OAuth features are excluded. The reviewed [Tokio named-pipe advisory](https://github.com/tokio-rs/tokio/security/advisories/GHSA-7rrj-xr53-82p7) concerns older Windows code. This is not a full locked-graph audit or a claim of no vulnerabilities. The crates.io API returned 403; published docs.rs provenance and official release/tag sources supplied the version evidence instead.

### Concrete clients

Select [Codex CLI 0.153.4](https://github.com/openai/codex/releases/tag/rust-v0.153.4) and [Claude Code 2.1.222](https://github.com/anthropics/claude-code/releases/tag/v2.1.222). Their installed versions and relevant help were observed, alongside Node 22.22.0/npm 10.9.4. [Codex MCP documentation](https://developers.openai.com/codex/mcp) and [Claude MCP documentation](https://code.claude.com/docs/en/mcp) document local stdio configuration. These are independent clients and both are real coding agents; their installed presence is not handshake, model-access or tool-use acceptance.

Use isolated, invocation-scoped client configuration. Actual agent calls use the guest server over a **test-only stdio relay** through the existing owned Tart control channel; Godot, the Rust core, registry and synthetic projects stay in the guest. This avoids host GUI execution or copying personal model credentials into the disposable VM. Extend the existing VM wrapper with fixed MCP fixture/server commands, not an arbitrary remote-execution product. This cost is justified by FR-019 plus the existing VM boundary; it adds no network transport, product remote support or new VM/service. No Inspector installation is required when these two clients satisfy interoperability.

## 5. SDK boundary findings and selected remedies

| Source-established behavior in released SDK | Required narrow adapter behavior |
| --- | --- |
| [Async reader](https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.5.0/crates/rmcp/src/transport/async_rw.rs#L133-L199) uses unbounded read-until; codec defaults to unlimited and outbound encoder does not enforce its decoder limit. | Bounded newline collection before parsing, strict frame/depth checks, bounded actual serialized responses and finite writes. No truncation or dropped safety fields. |
| The [compatibility parser](https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.5.0/crates/rmcp/src/transport/async_rw.rs#L326-L353) can log the raw line; service debug/info events include request/result/notification/client data. | Do not install an SDK tracing subscriber or honor RUST_LOG for those targets. Emit only adapter-owned source-free stderr diagnostics; never Debug-print received messages/errors. |
| Default parsing silently ignores malformed JSON. | Return source-free parse/invalid-request errors when a frame can be answered; explicitly close uncorrelatable oversized/incomplete framing. |
| [Cancellation](https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.5.0/crates/rmcp/src/service.rs#L1589-L1659) cancels the request token/removes response registration, but the handler keeps running cooperatively. | Signal the request's existing cancellation flag; retain and await bounded supervision. Do not abort/drop the mutation owner. A suppressed response is not delivered terminal evidence. |
| [SDK close/drain](https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.5.0/crates/rmcp/src/service.rs#L1763-L1815) has its own timeout and does not prove handler cleanup. | Process owner cancels and finishes/reaps its bounded work independently before exit. Never kill the user's editor or claim rollback. |
| [Initialization](https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.5.0/crates/rmcp/src/service/server.rs#L568-L691) has no built-in finite handshake timeout and admits newer-era pre-init requests. | Explicit supported version and a small connection state; no Godot dispatch before selected negotiation. Initial connection deadline does not renew operation deadlines. |
| [Tokio stdin](https://docs.rs/tokio/1.53.1/tokio/io/fn.stdin.html) uses an uncancellable blocking reader. | Keep core supervision separate, close connection on terminal transport failure, use bounded runtime shutdown after owned core cleanup and then exit the owned server process. Do not claim the blocked stdin thread was joined. |

The [transport contract](contracts/mcp-interface.md#connection-framing-and-timing) defines concrete byte/operation bounds and error mapping. They are input/output safety bounds, not token or description-length budgets. The adapter transport owns framing and delivery only; the SDK still owns protocol types/correlation and core/native boundaries still own product safety.

## 6. Interface and acceptance decisions

**Decision:** Static deliberate tool/parameter text plus operation-specific structured results. [The catalog](contracts/mcp-interface.md#public-catalog) is sufficient to choose discover/read/edit and supply the complete prior read basis without exposing reducers, bridge internals or repeated D/R/B ownership lectures. Necessary lifecycle/current-state limitations remain explicit. Failure-specific facts/actions appear in outcomes. No quantitative concision score or metadata framework.

[Tools 2025-11-25](https://modelcontextprotocol.io/specification/2025-11-25/server/tools) supports structuredContent/outputSchema and recommends text serialization for clients that only consume content. Return the same permitted operation object in structuredContent and one serialized JSON text block; no parallel explanatory transcript. Validate both from the same typed result and account for duplicate JSON escaping in the frame bound.

New evidence must cover closed positive/refusal/race/interruption behavior, actual clients, lean-interface use, transport/privacy and MCP-composed A–E. The [quickstart](quickstart.md#acceptance-coverage) maps all 26 scenarios, 22 FRs and nine SCs. Existing accepted open/native and lifecycle evidence remains reusable only after relevant-input review; changed native family/bridge/owner behavior requires affected compatibility and preservation checks, not automatic complete historical replay. Missing real-agent or closed-positive proof blocks implementation completion, not this research design selection.

**Alternatives rejected:** copying all safety rationale into every description, exhaustive static failure catalogs, opaque token/description-generation infrastructure, adding lifecycle tools, or claiming SDK/client versions alone prove support. Planning selects construction and verification obligations; it does not waive them.
