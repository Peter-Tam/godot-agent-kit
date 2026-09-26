# Architecture

This document records the **intended** product boundaries, not a description of an implemented server or addon, a feature specification, or permission to start a roadmap phase. The [constitution](.specify/memory/constitution.md) governs safety and compatibility; the [roadmap](ROADMAP.md) orders capability work. Trees below are conceptual, not existing files or scaffolding instructions. Only `godot-addon/` and `mcp-server/` are durable component boundaries; their internal layouts remain open.

## Repository and component boundaries

```text
godot-agent-kit/
├── godot-addon/       Godot-facing component installed into a Godot project
├── mcp-server/        agent-facing server product and distribution entrypoint
├── .specify/          governance and feature-workflow material
├── AGENTS.md
├── ARCHITECTURE.md
├── ROADMAP.md
└── ...
```

The expected addon shape illustrates how Godot installs an editor plugin; it does not prescribe additional internal files:

```text
godot-addon/
└── addons/
    └── godot_agent_kit/
        ├── plugin.cfg
        ├── plugin.gd
        └── ...
```

The addon owns **Godot Editor integration**: observe Godot-owned state and make native editor mutations through applicable Godot APIs, including `ScriptEditor`, `CodeEdit`, loaded `Resource`/`Script`, `EditorUndoRedoManager`, and `EditorFileSystem` where appropriate. It exposes only the Godot-side capabilities needed by the shared automation model. A thin, Godot-native EditorPlugin is the expected starting point, subject to evidence-backed planning; Godot-specific transaction integration and dirty-human-state/D/R/B protection belong here where the editor exposes them, but must obey the common core semantics. It does **not** own MCP protocol semantics or tool schemas, agent-client behavior, server/core orchestration or business logic, or a second independent transaction/coherence mechanism.

```text
mcp-server/
├── [Rust server and protocol-independent core implementation]
├── [tests]
└── npm/
    ├── [package metadata]
    ├── [thin launcher]
    └── [bundled platform-specific native release binaries]
```

The primarily Rust server is the actual server implementation, not the JavaScript launcher. It owns the shared protocol-independent automation/transaction core (sessions, targets, revisions, transactions, conflicts, and outcomes), structured diagnostics and observability, common safety across adapters, the external MCP adapter (schemas and agent-client behavior), the private addon bridge, and the npm/npx distribution entrypoint. Bracketed entries describe responsibilities, not mandated file or directory names; `npm/` and binary placement are illustrative, not a fixed package layout. Bundling native releases does not imply committing generated binaries to source.

## Flow, dependencies, and correctness

```text
Coding Agent / MCP Client
          │ MCP (external agent protocol)
          ▼
mcp-server/
  MCP Adapter
          │
          ▼
  Protocol-independent Automation / Transaction Core
          │ private editor bridge (not public agent protocol)
          ▼
godot-addon/
  Godot Editor Integration
          │
          ▼
Godot Editor APIs
```

Dependency direction is **MCP adapter → automation/transaction core → Godot integration contract**, not addon → MCP implementation. The core must not import MCP-specific contracts or depend conceptually on MCP-specific behavior; Godot implementation details stay behind integration rather than leaking to external adapters. The npm launcher only launches the native server and duplicates none of the core. There is no circular ownership: the addon can implement necessary Godot-side safety, not a rival core. MCP is the first external adapter, not the architecture. Every mutation path, regardless of external adapter, passes through the common transaction/coherence guarantees.

The [constitution's editor-coherence contract](.specify/memory/constitution.md#i-editor-coherence-is-the-correctness-boundary) applies end to end: **D** is disk source, **R** is loaded Godot `Resource`/`Script` state, and **B** is the visible editor buffer. Splitting responsibilities across server, bridge, and addon does not split the success guarantee. Open-script success requires independently observed `D == R == B` across applicable surfaces, protection of unsaved human work, and safe refusal when required safety or coherence cannot be observed. Request acceptance, mutation application, synchronization, and independent postcondition verification are separate stages. An addon acknowledgment to the server is not success; an addon disk change alone is not completion. Outcomes must expose unavailable observation and partial application rather than assert an unverified guarantee. Constitutional durability checks and applicable real-editor release gates remain required; this diagram does not replace them.

## Private editor bridge

The bridge is local-first, with **local-only operation and no remote access by default** as required by the constitution. Requests must identify the intended editor and project explicitly and refuse ambiguous routing safely. The bridge must carry structured requests/responses and outcomes, make disconnection/reconnection state observable, and preserve the same revision and conflict guarantees across that boundary. Transport details stay outside core semantics. Unselected authentication mechanics do not waive current least-privilege, local-only, permission, or other applicable security obligations. Transport, discovery, wire format, and session mechanics are decisions for later feature plans, not selections made here.

## Distribution and support

The intended eventual launch command is:

```bash
npx godot-agent-kit
```

The current distribution direction, **not constitutional law or a claim that the package is published**, is one npm package containing prebuilt Rust binaries for the platforms it supports:

```text
npx → npm package → thin JS launcher → select bundled current OS/architecture binary
                                      → native Rust MCP server
```

The launcher **MUST remain thin**.

The launcher selects the matching bundled binary, forwards arguments, preserves correct process/stdin/stdout/stderr behavior, reports an unsupported platform clearly, and propagates exit status/signals appropriately. If a stdio MCP transport is selected later, protocol output must remain on stdout and logs on stderr. Unavoidable launch failures (such as no matching binary) are launcher errors; product diagnostics belong to the Rust server/core. The launcher owns no MCP or Godot behavior, transaction/coherence or other business rules, or editor-session product logic. It never falls back to an incompatible binary.

No initial OS/architecture matrix or supported Godot-version list is established here. A bundled binary is not proof of support: advertise platform compatibility only with corresponding build/test and CI/release evidence, and exact Godot versions only with the constitutionally required CI and real-Godot integration evidence. Targets and the CI matrix remain unselected.

The addon, server, and any later runtime helpers are **developer tooling**, not gameplay authority. When implemented and released, their development/production-export boundaries must be explicit and verified at the applicable gates; repository placement or an editor-plugin annotation alone does not prove exclusion from production exports. Implementation of later runtime capabilities follows the roadmap, not this document.

## Decisions reserved for feature planning

- Private bridge transport (TCP, WebSocket, or another option), wire protocol/serialization, ports and discovery, authentication/session-token mechanics, reconnect strategy, and multiplexing/session protocol.
- Cargo workspace/package topology, Rust crates/dependencies, async runtime, and testing framework/harness.
- MCP SDK and specification version, as well as external MCP transport.
- Exact npm layout and final npm package naming/scoping, notwithstanding the intended command above; initial platform matrix, target triples, CI/release implementation, and binary signing/checksums.
- Supported Godot-version matrix and evidence-backed compatibility selections.

The constitution remains authoritative; this document records present, reviewable topology and ownership rather than making these choices immutable constitutional rules. The [roadmap](ROADMAP.md) sets sequencing and capability expansion, feature specifications define user-visible behavior, technical plans select implementation, and tasks govern delivery under [AGENTS.md](AGENTS.md). Later plans may refine internals while preserving these boundaries; a materially better topology can change this document through normal review. Nothing here authorizes early phases or weakens existing safety, security, export, or release obligations.
