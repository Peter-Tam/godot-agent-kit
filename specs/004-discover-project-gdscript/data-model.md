# Data Model: Project GDScript Discovery

**Status:** Proposed contract for [Feature 004](spec.md); not implemented. The [caller](contracts/discovery-api.md) and [bridge](contracts/bridge-protocol.md) contracts define transport projections. This document owns semantic invariants and fixed bounds.

## 1. Reused identities and current consumers

Reuse checked `RequestId`, `ProjectRoot`, optional `SessionId`, `ResourcePath`, `FileIdentity`, decimal counters, `ObservationInterval` and `CollectionStamp` value semantics from the existing [observation model](../001-observe-gdscript-state/data-model.md). Do not reuse a document snapshot, `ResolvedTarget` with a fabricated script, mutation revision, source hash or edit outcome.

- Request ID: existing safe ASCII syntax, at most 64 bytes; CLI generates fresh 32-hex correlation IDs, not idempotency keys.
- Session ID: exact 32 lowercase hex characters, bound to the authenticated editor lifetime.
- Project root: required absolute input, existing lexical validation, at most 1024 UTF-8 bytes; selected identity is canonical root plus device/inode. Display names, focus and cwd do not select a project.
- Entry locator: exact `res://` path, at most 2048 UTF-8 bytes, no empty/dot/dotdot components, backslash, controls, query/fragment/colon decorations, embedded subresource or lossy Unicode conversion. Preserve legal Unicode and case.
- Discovery recognizes the final `gd` extension ASCII-case-insensitively. `ResourcePath` validates round-trip syntax, not eligibility for another operation; a case-variant path may remain unsupported by an existing caller's unchanged script profile. Do not change existing `ResourcePath::kind` or widen opening/editing as part of this feature.
- Device/inode/ticks/counters use checked decimal strings where existing wire contracts do, never lossy JSON numbers or editor pointer values.
- Editor collection ticks and caller monotonic elapsed time have separate clock domains. Receipt time is assigned once per validated frame. Wall-clock endpoints are descriptive; monotonic elapsed time controls the deadline.

Public Rust declarations are limited to the request, immutable outcome and fields/accessors consumed by the actual CLI/library runner. Attempt state, filesystem witnesses, scope leases, codecs and project-only routing internals remain crate-private. No generic discovery trait or speculative variant is introduced.

## 2. Fixed supported profile

| Bound | Value / behavior |
|---|---|
| Operation cutoff / terminal deadline | 4500 ms / 5000 ms from clock-before-parse; reserve 500 ms for terminal delivery |
| Returned eligible entries | 1024; the first additional eligible entry creates `entry_limit`, not silent truncation |
| Visited directories | 1024 including root; further otherwise eligible directories create `directory_limit` |
| Directory depth | 64 below root; deeper in-scope subtrees create `depth_limit` |
| Visited directory entries | 16384 across acquisition; count every encountered name before filtering or allocating retained name storage |
| Detailed diagnostics | At most 63, plus one fixed `additional_gaps` aggregate carrying the omitted count when needed |
| Worker progress batch | At most 64 entries and 512 KiB encoded; no batch is accepted before its binding and metadata checks |
| Final result | At most 6 MiB UTF-8 JSON plus newline, below the existing 12 MiB worker/response ceiling |
| Public path/root/request syntax | Existing 2048/1024/64-byte limits above |
| Public query modes | Whole selected project only; no paging, filters, include-ignored, source search or configurable safety limits |

Bounds apply before allocation/growth, including per-directory name collection; do not first read/sort an unbounded directory. Visit order may follow the filesystem; final retained entries are unique and sorted by exact UTF-8 bytes. A limited result is not guaranteed to be a stable prefix and provides no continuation token. Exactly reaching a limit is not itself proof of exhaustion or overflow: complete requires actually exhausting the scope within every bound. Final serialization size is checked before emission; reaching the result limit retains only complete entries, reports `result_limit` and cannot become complete. The supervisor rejects a worker violating bounds rather than trusting its truncation claim.

The prototype timings are feasibility evidence only. Full caller routing, safety checks, output delivery, blocked-I/O and limits require implementation acceptance.

## 3. Public entities

### DiscoveryRequest

Fields: `request_id`, `project_root`, nullable `session_id`. No script, source, target executable, directory prefix, glob, timeout override or permission token.

Validation is structural before routing; access/identity is checked by the existing common resolver. A syntactically valid request is not authenticated selection. A malformed request produces `refused/invalid_request` without echoing unvalidated fields.

### DiscoveryTarget

Fields: `request_id`, canonical `project_root`, `project_file_id {device,inode}`, `session_id`, `godot_version {version,hash}`. It contains no script locator, endpoint, secret, nonce or native pointer.

It must match the request, exactly one authenticated candidate, the pinned root handle and selected lifetime. An explicit session mismatch or replacement never binds to another session. Existing `Selection`-style feedback may contain only authenticated public candidate session IDs for the requested project and the missing `session_id` selector, never candidate inventory.

### DiscoveryScope

Fields: `policy` equal to `godot_project_files_v1`, `project_data_directory` equal to `res://.godot` or `res://godot`, and `exclusions` equal to the fixed ordered identifiers `dot_names`, `project_data_directory`, `gdignore_subtrees`, `nested_projects`, `non_gdscript_files`, `non_regular_files`, `embedded_or_unsaved_documents`.

The policy is tied by `DiscoveryTarget` to the exact supported Godot/macOS environment, not an unverified portability claim. It follows [research §2](research.md#2-exact-visibility-policy). A symlink/access/representation failure is a coverage gap, not a silently expanded exclusion policy. No list of hidden/ignored filenames is needed to explain scope. VCS ignore and production-export rules are not consulted.

### DiscoveryEntry

Public representation: one exact `ResourcePath` string in `entries`. No per-file source, hash, dirty/open/parse state, persistent document ID, edit eligibility, display-name alias or redundant basename. Paths are distinct even when they reference the same underlying regular file through different valid names. A subsequent request must independently resolve current identity and safety.

### DiscoveryInventory

Required fields:

- `scope`: the observed `DiscoveryScope`.
- `entries`: bounded exact path strings, unique and sorted at terminal delivery.
- `collection`: caller-domain metadata collection interval/stamp, never copied editor time.
- `coverage`: `complete`, `partial` or `not_started`; describes the declared scope, not whether entries are nonempty.
- `validity`: `observed` or `earlier_observation`. The latter is used when interruption prevents current final checks; it never claims current editor liveness or complete coverage.
- `consistency`: `{recheck: "completed"|"unavailable", stability: "unknown"|"changed", atomic: false}`. Completed rechecks do not prove a frozen filesystem; there is no `stable` or atomic-success value.
- `visited_entries`, `visited_directories`: bounded acquisition counters, not a promised total project size.

Known-invalidated entries are removed rather than exposed as current locators. Diagnostics retain safe attributable affected scope/reasons. A valid empty list with `partial` or `not_started` does not mean an empty project. A complete inventory requires `validity=observed`, `recheck=completed`, `stability=unknown`, actual exhausted scope and no gap or terminal failure. Unknown stability means no stronger stability guarantee was established, not permission to ignore detected invalidation.

### DiscoveryDiagnostic

Fields: fixed `code`, `stage`, nullable `scope`, fixed `message`, fixed `action`, and nullable bounded `omitted_count` used only by `additional_gaps`. Stages: `validate_request`, `resolve_target`, `authenticate`, `begin_scope`, `enumerate`, `recheck`, `finalize`.

`scope` is either `res://` for the selected root, a safely attributed representable project-relative path/subtree, or null. Never print an outside redirect target, undecodable filename bytes, raw OS error, private registry path, credential or candidate project inventory. An unsupported component is described at the nearest safe parent or with null scope. User-owned source text is never diagnostic data.

### DiscoveryOutcome

Required public fields: `schema_version: 1`, `request_id`, `outcome`, `interval`, nullable `requested_target`, nullable `resolved_target`, nullable `inventory`, `diagnostics`, nullable `selection`.

`requested_target` is `{project_root,session_id}` only after lexical validation, otherwise null. `resolved_target` is the authenticated `DiscoveryTarget`, never a guessed target. `inventory` is null before usable scope/evidence or when suppression is required. Required nullable fields are present, not omitted. Outcome and exit meanings are in [the caller contract](contracts/discovery-api.md#3-terminal-results).

## 4. Private attempt and editor context

The private state machine is `starting → selected → scoped → enumerating → rechecking → terminal`; a failure may enter terminal from any earlier state. No source/lifecycle application state or rollback model belongs here. A typed state/ordered-event reducer prevents impossible combinations and rejects duplicate, late or out-of-order events.

`SelectedEditor` owns the authenticated socket, project directory, original/canonical root identity, engine/session/capability evidence and request binding. It is produced by the common selection algorithm; it cannot be constructed from a path string alone or transferred between request IDs.

`EditorScopeContext` contains request/project/session binding, exact policy/data-directory, editor collection stamp, `filesystem_changed` epoch, `scanning`, `importing` and owner expiry tick. The Boolean fields are independent external editor facts, not lifecycle flags. Epoch is local invalidation evidence, not a project revision or a cache-freshness proof.

Begin and final context must match policy/data directory/session and show no scan/import or epoch invalidation for completeness. Scan/import activity returns a limited result with `editor_inventory_busy`; do not wait/retry or enumerate solely to conceal it. Missing/unsupported policy context refuses with `unsupported_visibility_policy` before traversal. A policy/data-directory change clears inventory because the collection no longer has the admitted membership basis. An epoch change with unchanged scope permits independently rechecked paths only as limited `editor_inventory_changed` evidence.

## 5. Private filesystem evidence and coverage algorithm

Private witnesses contain only what the current request needs:

- Root: original/canonical namespace identity, retained directory handle and existing ancestor/owner/mode/ACL checks.
- Directory: exact relative components, device/inode/type, applicable ownership/mode/ACL, pre/post membership metadata (mtime/ctime), and the namespace chain observed for that path.
- Exclusion markers: safely established absence or regular-file identity/access evidence for literal `.gdignore` and `project.godot` in child directories. Marker contents are never read. Root marker lookup is not a subtree exclusion.
- File candidate: exact locator, device/inode/regular-file type and applicable access/namespace evidence. Do not collect source bytes, source hash or source mtime as a revision; a source-only edit does not change path membership.

Algorithm obligations:

1. Authenticate unique selection and obtain supported idle editor scope before project inventory access. Recheck root identity/access around collection; no offline fallback.
2. Enumerate only through validated directory handles. Bound each encountered name before allocation/filtering. Skip candidate-native dot names and the effective data subtree without traversing them.
3. For visible directories, check/open one component with the existing no-follow/nonblocking/identity/ACL pattern before enumeration. Refuse unsafe root binding; local unsafe/unreadable components become explicit gaps. Never follow a symlink to determine whether it is a directory or expose its target. Do not treat failed stat/open as nonexistent.
4. Check child marker names using capability-relative metadata and the native filesystem's literal lookup behavior. A safe regular marker excludes the child subtree. Directory/special-file markers do not satisfy regular-file existence; aliases/unavailable checks make the subtree a gap rather than guessed visible/ignored. Recheck exclusion evidence so marker removal/replacement during the interval cannot certify a complete list.
5. A regular, representable, in-scope GDScript candidate must satisfy the current namespace/access boundary before entering a progress batch. Reuse necessary descriptor-based metadata/ACL checks without reading file content. Non-GDScript regular files and known non-regular non-directory objects are not script entries. Unsafe visible symlinks are gaps because their possible subtree cannot be established without following them.
6. Retain only depth-bounded open traversal handles. Keep bounded witnesses for final rooted reopening/recheck, not 1024 open directories. Before accepting each batch, validate its root/namespace attribution; a batch is not a final complete result.
7. At final recheck, verify root, traversed directory membership/access, exclusion markers, eligible file identities/access and their named component chains. Known affected subtrees remove their entries and add `namespace_changed`; exhausted enumeration cannot erase that invalidation. Unavailable checks add explicit gaps and prevent complete coverage. No automatic retry restores a complete label.
8. Recheck editor scope/lifetime over the same authenticated channel. Only then can the common reducer report complete. Namespace and editor stamps remain separate interval evidence, not simultaneous atomic proof.

A blocked call is bounded by parent supervision, not an assumption that every syscall returns. A worker may be terminated; earlier checked batches can survive only under the terminal retention rules below. This design does not claim linearizable enumeration against arbitrary non-cooperating writers between checks.

## 6. Terminal precedence, reasons and retention

Apply the following order when several facts are known; retain all permissible diagnostic distinctions. Later success evidence never overrides an earlier terminal decision.

| Priority | Outcome / representative reasons | Retention |
|---|---|---|
| 1 | `refused`: `denied_access`, `unsafe_registry`, `authentication_failed`, `out_of_project`, `project_identity_changed` | Clear inventory; no unauthorized target/path evidence. |
| 2 | `refused`: `ambiguous_target` | Clear inventory; only safe session-selection feedback. |
| 3 | `refused`: `invalid_request`, `invalid_project`, `unsupported_version`, `unsupported_capability`, `unsupported_visibility_policy`, `editor_unavailable`, `busy` | No inventory from an unstarted/unadmitted attempt. |
| 4 | `interrupted`: `protocol_error`, then `cancelled`, `session_replaced`, `disconnected_editor`, `timeout` | Discard rejected payloads. Clear inventory on known binding/session replacement; otherwise independently validated earlier batches may remain as `earlier_observation`, never complete/current liveness. |
| 5 | `limited_listing`: local gap or unavailable/invalidated recheck | Keep only safely attributable entries; coverage partial/not-started, reasons explicit. |
| 6 | `complete_listing` | Exhausted scope, every required check completed, no known gap/invalidation; zero entries is valid. |

Local-gap reasons include `directory_unreadable`, `entry_unreadable`, `unsafe_entry`, `unsupported_path`, `unsupported_marker`, `namespace_changed`, `scope_changed`, `editor_inventory_busy`, `editor_inventory_changed`, `recheck_unavailable`, `entry_limit`, `directory_limit`, `depth_limit`, `work_limit`, `result_limit`, `additional_gaps`. A missing/replaced file encountered during acquisition is `namespace_changed`, not proof it never existed. A directory-local denial does not automatically discard other safe entries; a root/authentication denial does.

`scope_changed` clears inventory and records the lost admitted scope; there is no new-policy rewalk in the same request. For interrupted earlier evidence, the result identifies the original interval and unresolved checks. An earlier correctly authorized path is not relabeled as belonging to a replacement editor. A terminal result cannot schedule later enumeration, mutation or replay. Transport closure/expiry merely releases the read-only slot; it is not a rollback claim.

## 7. Meaningful validation boundaries

Deterministic tests cover complete versus false-empty, partial retention versus denial suppression, fixed precedence, data/context mismatches, exact/one-over resource limits, path identity and later invalidation. Filesystem integration must use actual confined directories, permission/ACL controls and namespace replacements, not mocked echoes. Process tests must stall a real owned worker and verify terminal delivery/cleanup. Editor acceptance must observe actual dirty/selection/history preservation and the separate composed workflow.

The public output intentionally omits source/document state. A list result cannot be reused as an edit basis, and a path absent from a limited listing cannot be treated as nonexistent. The existing observation/open/edit contracts remain the authority for the later operation.
