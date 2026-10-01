# Research: Safely Discover Project GDScripts

**Feature:** [spec.md](spec.md) | **Research:** 2026-09-30–2026-10-01

**Disposition:** Technical unknowns resolved for the design below. This is planning evidence, not implemented discovery, feature acceptance or a wider compatibility claim. Two read-only investigations covered current integration seams and pinned Godot visibility; owned executable probes checked the important assumptions. The initial constitutional research gate is recorded in [plan.md](plan.md#constitution-check).

## 1. Enumeration authority and freshness

**Decision:** Enumerate fresh directory/name/type metadata in the existing supervised Rust worker, through the selected project's capability-rooted filesystem boundary. Implement the small, exact supported Godot visibility predicate; obtain its effective project-data exclusion from the authenticated editor. Do not enumerate the cached editor tree in the product, read script contents, start a scan or introduce an index.

**Rationale:** The owned stock editor exposed a settled 126-script tree containing the 100-script/ten-folder catalog. After an external fixture file was created, the cached tree still omitted it while `is_scanning` was false and the `filesystem_changed` counter remained 1. Twenty subsequent passive traversals preserved the fixture's dirty buffer and history but could not certify fresh disk coverage. Thus absence of a scan or signal is not sufficient for FR-007/FR-009 and US3.3. A directory walker also exposes unreadable areas that a cached tree may silently omit.

**Alternatives considered:**

- Tree-only enumeration is the simplest code, but the demonstrated stale-negative case can become a false complete/empty result. Reframing the feature as only a cached-tree listing would weaken the accepted specification.
- Always label tree results limited would not deliver the required useful complete path.
- A hybrid tree/disk comparison duplicates traversal and inherits both sides' availability limits. Native visibility parity can instead be established against exact source and fixtures, with one fresh metadata enumeration in production.
- Forced rescan violates FR-006; a watcher, database or index service adds no necessary guarantee.

The selected walker observes an interval, not an atomic filesystem snapshot. Known namespace, scope-policy or session invalidation prevents completeness; undetected transient activity is not claimed to be excluded. Human source edits do not by themselves invalidate a path listing, and file source timestamps/hashes are not revision evidence for this operation.

## 2. Exact visibility policy

**Decision:** Initial policy `godot_project_files_v1` is verified only for the existing official Godot 4.7.2/macOS arm64 candidate. Preserve exact path spelling and use these rules:

1. Exclude dot-prefixed entry names on this Unix/macOS backend. This covers hidden files/directories and dot VCS metadata; do not consult `.gitignore` or another VCS ignore engine.
2. Exclude the effective project-data directory and descendants, normally `res://.godot`, alternatively `res://godot`.
3. For each otherwise visible child directory, exclude its subtree when a regular file named `.gdignore` or `project.godot` exists there. The latter is a nested project. Apply the candidate's filesystem lookup semantics to these literal names; do not parse marker contents or impose artificial case-sensitive lookup on a case-insensitive filesystem. The selected root itself is not a nested-project/marker exclusion.
4. Include ordinary visible addon folders. Production-export filters do not change discovery scope.
5. Include regular files whose final extension is ASCII-case-insensitively `gd`, including empty, syntax-invalid, read-only and tool scripts. Do not read source, class metadata or parse results to classify them.
6. Apply the existing stricter confinement/path-representation boundary independently. Symlinks are never followed, marker aliases are not treated as verified exclusions, and an unsafe/unreadable in-scope area is a reported gap rather than a fabricated empty subtree.

**Rationale and primary sources:** Pinned [`EditorFileSystem::_scan_new_dir`](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/file_system/editor_file_system.cpp#L1161-L1229), [`_should_skip_directory`](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/editor/file_system/editor_file_system.cpp#L3488-L3512), and the same file's recognized-extension processing establish directory and extension rules. Pinned [`DirAccessUnix::is_hidden`](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/drivers/unix/dir_access_unix.cpp#L688-L690) checks dot naming, not macOS `UF_HIDDEN`.

The probe confirmed that both a file and folder with `chflags hidden`/flag value 32768 remain visible in this Godot build. An initial research interpretation that `current_is_hidden()` necessarily honors OS hidden flags was wrong and is not part of the plan. Dot files/folders, `.gdignore` descendants and nested projects were excluded; `.GD`, addon, VCS-ignored and export-excluded controls remained visible.

**Alternatives considered:** Generic OS-hidden filtering would incorrectly omit observed Godot-visible scripts. A cross-platform policy now would be untested scope expansion. Content/type parsing or custom ignore patterns are unnecessary and prohibited by the feature's location-only contract.

## 3. Effective project-data context

**Decision:** The addon obtains `EditorInterface.get_editor_paths().get_project_settings_dir()`. On the pinned profile accept only `res://.godot/editor` or `res://godot/editor`, derive its parent as the excluded data directory, and recheck it at the end of the attempt. Unexpected/missing context is unsupported, never guessed from disk or a default. This is an editor-owned fact, not a new native method.

**Rationale:** The engine initially derives the data directory from `application/config/use_hidden_project_data_directory` ([pinned initialization](https://github.com/godotengine/godot/blob/ed1daf0bf001b61586d9930840f2f1394092c079/core/config/project_settings.cpp#L884-L886)). A live setting value is not necessarily the effective directory. In an owned project started with the setting false, the public editor-path getter returned `res://godot/editor`; after a fixture-only unsaved setting change to true, it still returned `res://godot/editor`. Using only the changed Boolean would exclude the wrong directory.

The public [EditorPaths contract](https://docs.godotengine.org/en/4.7/classes/class_editorpaths.html#class-editorpaths-method-get-project-settings-dir) and the exact generated API (`godot-addon/native/build/extension_api.json`, `EditorInterface.get_editor_paths`) provide the accessor. Direct `EditorPaths.get_project_settings_dir()` failed as a non-static GDScript call; the corrected `EditorInterface` accessor succeeded. The failed owned editor was terminated and is not counted as a successful probe.

**Alternatives considered:** Hard-coding `.godot`, deriving only from the mutable project setting, reading/parsing `project.godot`, or exposing an extra C++ getter are all unnecessary or weaker than the tested public getter.

## 4. Locator-free authenticated selection

**Decision:** Introduce a discovery request with request ID, project root and optional session ID, but no script locator. Extract the existing resolver's common project/session authentication algorithm into a private locator-independent seam. Keep the existing public `target::resolve(&ObservationRequest, ...)` and its real script validation/binding responsibility. Add only the crate-visible project-selection entry consumed by discovery; do not expose another public routing framework.

**Rationale:** [`ObservationRequest` and `ResolvedTarget`](../../mcp-server/src/observation.rs) necessarily contain a script path. [`project_fs::project`](../../mcp-server/src/project_fs.rs) currently validates that locator while establishing the root. [`SelectedSession` and `target::resolve`](../../mcp-server/src/target.rs) then own the authenticated socket and rooted directory. A dummy `res://placeholder.gd` would invent a document and conflate project selection with script admission.

Factor the common root validation and source-free authentication without changing existing script-specific preconditions or public observation/edit/open v1 results. A private selected-editor value owns socket, directory, original/canonical project identity, session, engine and capabilities. Existing script selection binds its real `ResolvedTarget`; discovery binds a project-only target. Moving owned fields does not require duplicate connections, directory trees or source copies.

LSP references to `target::resolve` identified the observation runner, open/edit workers, confinement tests and bridge-boundary tests. The existing public resolver remains a current consumer with distinct script-specific work, not a deprecated alias. Shared implementation changes must exercise those consumers and the fixture-side routing consumer.

**Alternatives considered:** Duplicating the resolver risks authentication/ambiguity precedence drift. Making the observation locator optional would change every existing document contract. A broad Phase 2 core extraction is not needed to share one current selector.

## 5. Filesystem safety and bounded evidence

**Decision:** Add discovery-specific traversal beneath the existing `project_fs` owner. Reuse root/ancestor identity, ownership, mode, deny-only ACL and no-follow open checks; do not call source acquisition or invent a second security policy. The prototype demonstrated `cap_std::fs::Dir::entries`, `symlink_metadata`, checked no-follow child opening and `Dir::from_std_file` with the existing dependency.

The production walker must increment work/depth/result bounds before growing collections, traverse one validated child component at a time, distinguish marker absence from failed lookup, and recheck namespace/permission evidence. Retain only bounded request-local identities and directory membership metadata; retain at most the depth-bounded active directory handles, reopening through the checked root for the final verification instead of keeping one descriptor per visited directory. Source bytes and source hashes are never collected.

A stable eligible file observation records identity/type and access evidence, not file-content revision. Directory membership metadata and marker identities/exclusion context are checked before/after traversal and at final recheck. A changed subtree removes affected entries and reports a gap; root identity/access denial suppresses the entire inventory. No detected invalidation can be hidden by a later matching sample. The full rules and bounds are normative in [data-model.md](data-model.md).

**Alternatives considered:** Ambient recursive filesystem walking bypasses the existing boundary. Reusing `read_disk` reads source unnecessarily. Retaining unbounded names/FDs or sorting an unbounded directory before applying a cap defeats the resource limits. A snapshot service or arbitrary-writer lock is not required by the interval-based contract.

The prototype is not production confinement proof: it exercised metadata traversal and parity, not the complete ACL/authentication/race/deadline boundary. Those obligations remain implementation acceptance, including unsafe markers, directory replacement and partial-evidence suppression.

## 6. Private compatibility and editor ownership

**Decision:** Make a coordinated private bridge v4 cutover, adding exactly one authenticated capability, `discover_gdscripts`, after the seven existing bits. Preserve native API revision 2, native artifact naming and all three existing public v1 contracts. Add a small GDScript discovery owner for scope/liveness facts, not editor-tree enumeration or a second result reducer.

**Rationale:** Current [bridge v3](../003-open-project-gdscript/contracts/bridge-protocol.md) has exact tuple versions and a fixed, transcript-bound capability record. Adding a field without a deliberate version change breaks its authenticated contract. Discovery must not masquerade as `open_enumeration`, which describes existing buffer attribution, or be inferred from unrelated capabilities.

The new owner claims the existing `_active` operation slot, observes public effective context and scan/import/change facts, rechecks them, and releases on finish/abort/expiry/disconnection. It acquires no Script, document or native transaction object. No kit operation is queued or replayed after busy refusal. The Rust worker alone walks the selected directory; the addon never receives its paths or opens a scope supplied by the caller.

**Alternatives considered:** A parallel listener/protocol, feature-specific credential, native ABI bump, shared-result framework or additional scheduler would add cost without new protection. Keeping v3 with an incompatible transcript would conceal a real compatibility break. Version 4 rejects v3 peers instead of providing a fallback shim.

The cutover changes a shared authenticated boundary; complete existing observation, edit and opening suites are therefore affected under repository policy, not merely because those suites exist. Native code/build semantics remain unchanged unless implementation establishes a separate concrete need.

## 7. Caller, deadline and scope limits

**Decision:** Add one `discover-gdscripts` CLI and a narrow typed library request/outcome/runner entry in the existing package. Reuse existing owner-private registry bootstrap, no-source-before-selection, clock-before-parse behavior and supervised same-executable worker. Use a 4.5-second operation budget plus 0.5-second terminal-delivery reserve within the specified five seconds. No configurable timeout, force, include-ignored, glob, search, pagination, executable or offline fallback flag.

Public outcomes are complete listing, limited listing, refused and interrupted; use the existing exit-code convention 0/2/3/4, with 1 reserved for unexpected host failures. The model defines deterministic precedence, safe partial evidence and strict sizes. A new explicit request always collects fresh evidence. The worker uses internal bounded batches so interruption can retain attributable earlier entries; this is not a public paging/replay interface.

The selected profile supports 1024 returned entries, 1024 visited directories including the root, depth 64, 16384 visited directory entries and a 6 MiB final JSON result. Existing root/path/request formats remain unchanged. These bounded choices comfortably exceed the 100-script/ten-folder required fixture without promising arbitrary project scale. Limits produce honest partial coverage; no directory-wide allocation precedes a bound check.

**Alternatives considered:** An in-process directory walk can block the five-second result. Reusing the ten-second editing worker or source-specific observation reducer gives the wrong deadline/semantics. New async runtimes, worker services, persistent caches and retry queues are not needed.

## 8. Research evidence and limitations

Exact exercised executable: official `4.7.2.stable.official.ed1daf0bf`, commit `ed1daf0bf001b61586d9930840f2f1394092c079`, SHA-256 `c7cccbf8fb143e34e02fd6521e09be2c2b974f0d5db080b19071c9c570718ccf`; macOS 26.6.2 arm64. Existing Rust 1.98.1 and locked `cap-std` built artifact were used for the throwaway metadata consumer. No dependency was installed or changed.

Retained synthetic [research summary](file:///Users/petertam/gak-discovery-plan-rbmizhza/evidence/summary.json), SHA-256 `29d4725bb1ae6888ee7e9b5e9c905d78489e79a2219ad448bd27fedce97be77e`, records:

- 23 passive inventory getter invocations, including an initially scanning empty tree and a settled 126-entry tree with 100 catalog scripts.
- Inclusion/exclusion controls for empty/invalid/read-only/tool/uppercase scripts, addons, VCS/export filters, dot names, `.gdignore`, nested project and OS-hidden flags.
- A new on-disk file absent from the idle cached tree with unchanged signal epoch.
- Twenty repeated getter requests preserving before/after live fixture witnesses, followed by successful actual native Undo/Redo history replay. Maximum measured getter-only duration: 261 microseconds; this is not a product discovery latency measurement.
- Three metadata projections: add 127 entries/8324 microseconds; rename 127/3099; removal 126/2356. The latter two path sets were independently compared with the settled tree plus the intentional change. No source bytes were read by that consumer.
- Effective visible data-directory evidence and the corrected public accessor. One failed static-call startup is explicitly retained as a research finding; two successfully exercised owned editors exited 0, and the failed instance was terminated.
- [Owned-window image](file:///Users/petertam/gak-discovery-plan-rbmizhza/evidence/owned-discovery-probe.png) was inspected: the dirty subject buffer and visible filesystem state matched the owned fixture. No other window was captured.

All owned editors are stopped. Temporary projects, registry credentials, probe sources and binaries were removed; only synthetic evidence remains. These local artifacts are provenance, not a prerequisite for another developer. [quickstart.md](quickstart.md) defines reproducible implementation acceptance instead.

No discovery product caller, private v4 exchange, production walker, full deadline/refusal/race/export campaign or feature acceptance was implemented or run. Existing product suites were not rerun for this documentation-only plan. Public API and source evidence plus bounded executable research establish a credible design; they do not satisfy future task completion.

## 9. Complexity and support review

| Present requirement/failure | Simplest credible alternative and its limitation | Selected cost and why justified now |
|---|---|---|
| Unknown paths cannot enter the completed workflow | Manual path lookup leaves Phase 1 discovery absent | One location-only caller and domain reducer, directly consumed now |
| Cached tree missed an observable new file while idle | Tree-only/always-limited/forced scan fails completeness or non-interference | One bounded metadata walker using existing confinement; no second inventory service |
| Discovery has no script locator | Dummy path or copied resolver invents identity or duplicates security policy | Private shared project-selection seam, preserving current script admission |
| New capability must be authenticated | Altering v3 silently or inferring another bit breaks compatibility/meaning | One coordinated v4 cutover and directly affected regression evidence |
| Live setting can differ from effective data directory | Hard-code/default/current Boolean can exclude the wrong tree | One tested public editor-path getter and bounded recheck |
| Filesystem/editor I/O can block | In-process timeout checks cannot guarantee terminal delivery | Reuse existing owned-worker supervision; no service or runtime dependency |

Initial and post-design constitutional checks impose no new approval, CI-provider, runner-topology or operational gate. Support remains the exact tested candidate until the discovery implementation earns its own acceptance. Hostile same-UID software or malicious already-running editor extensions remain outside the inherited threat model; known interference, unsafe roots, missing evidence and project confinement are not waived.
