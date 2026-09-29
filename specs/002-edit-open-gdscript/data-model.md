# Data Model: Guarded Open-GDScript Editing

**Status:** T001–T003 are complete. [T003 acceptance](quickstart.md#11-t003-native-boundary-acceptance-2026-09-29) records the selected §18 T1 / §19 L1 private boundary's real-editor evidence. The semantic obligations below remain normative, while numeric saved-state and patched-validator layouts describe T001's historical implementation. T004 owns migration to truthful stock helper/native facts; T003 does not manufacture private mtimes, editor-clock parser evidence or a public wire cutover. See the [implemented private boundary](contracts/native-integration.md#7-current-private-implementation-boundary).

## 1. Requests, identity and revisions

### EditRequest

| Field | Rule |
|---|---|
| `schema_version`, `operation` | `1`, `edit_open_gdscript`; separate contract from observation v1. |
| `request_id` | Existing nonsecret RequestId format: 1–64 ASCII letters/digits/`-`/`_`, new for each attempt. No transaction UUID. |
| `project_root`, optional `session_id`, `script_path` | Existing explicit routing/selectors and confinement; only existing open standalone `.gd` is editable. An omitted session still requires unique source-free selection. |
| `expected` | Required ExpectedRevisionBasis derived from a prior complete, clean, agreeing open observation; never an authorization token. |
| `replacement_source` | One whole resulting source string, ≤512 KiB UTF-8. No patches/batches/force/retry options. |

A prior complete dirty/divergent/limited observation is valid observation information but cannot supply an eligible edit basis. The CLI accepts that prior observation as input and rejects an ineligible basis; typed consumers can extract the same checked fields. Caller-supplied evidence can be forged or stale, so all meaningful facts must be freshly acquired and compared before application. No observation ID alone grants write authority.

Supported text is exact UTF-8, including empty text, tabs, Unicode and final-newline differences, representable by the native CodeEdit operation. The initial implementation explicitly rejects NUL and carriage-return/BOM representations before mutation rather than normalize them; LF text is preserved byte-for-byte across D/R/B. Existing observation continues to preserve/read its broader encoding/line-ending evidence without adopting these mutation input restrictions. Native representation checks and post-read equality remain mandatory.

Preparation additionally proves that the baseline and desired source are unchanged by the target's current native Save formatting profile. Otherwise refuse explicitly; do not change text or editor preferences to manufacture Save/history durability. Recheck that profile at application and verification.

### ExpectedRevisionBasis

| Field group | Meaning |
|---|---|
| Selected identity | Canonical project root/device/inode; exact session; script resource path; native Script, ScriptEditorBase and CodeEdit IDs; standalone disk device/inode. |
| Source basis | SHA-256 and UTF-8 byte length of the independently agreeing prior D/R/B source. The adapter computes it from actual prior text, not an advertised generic revision token. |
| Editor version | Exact prior CodeEdit current version (`B.witness.source_version`), a decimal string, plus attributable prior clean/open state. Observation v1 does not expose saved version; acquire it independently during fresh preparation and guard that prepared value afterward. Versions are local witnesses, not cross-authority clocks or a global revision. |
| Provenance | Prior observation request ID and interval, original identity/source witnesses. Correlation is not authority and supplies no freshness lifetime exemption. |

Reuse Feature 001 identity/witness types and decimal-string counters. No Resource UID, mtime or source hash alone identifies the filesystem write target. Fresh project/session/document/Resource/buffer/leaf identity must match. Closure/reopening or replacement invalidates the basis even if source text matches. Dirty state is independently observed, never inferred from hashes. Known stale R/B or unavailable required observations refuse mutation. Phase 1's [concurrency boundary](spec.md#phase-1-concurrency-boundary) remains unchanged.

### NativeAttempt

One authenticated connection/request-local in-memory record owned by the Godot integration. It retains actual Script/ScriptEditorBase/CodeEdit instance IDs and still-open association, the immutable intended source, preparation-time saved and frozen post-application current CodeEdit versions, guarded source/namespace witnesses, the pinned project and exact-file descriptors, the original exact-file `fstat` mtime T0, editor-clock expiry and current stage. It owns an attempt-local persistence/metadata-restoration receipt; no saved-handler Callable, Signal-edge topology, cached-Script handler witness or callback-generation tracking is required. Bounded explicit exact-source validation remains a separate unresolved stock obligation. No durable operation registry, replay cache, journal or background service.

Only one active read collection **or** edit attempt is admitted by the existing editor integration slot for the selected editor/session. This also serializes kit mutation entry: only the slot-owning attempt may cross the potentially-applied boundary. Excess edit attempts receive a terminal, source-free `busy` refusal with zero source/history/finalization change; no preparation or apply work is queued for them. Slot release never resumes, retries or replays a rejected attempt. A later edit requires a new request with a freshly observed valid basis; a changed revision cannot be bypassed by compatible or equal desired text.

The admitted attempt owns the slot from preparation through terminal cleanup. Disconnect/expiry/disable prevents new stages, but does not release the slot or live native references/descriptors while already-entered work could still mutate; cleanup waits for that work to return. Generic confinement of forbidden effects from edit-generated ordinary editor continuations and the separately invoked validator remains required; the slot alone does not prove it, but there is no selected saved-handler callback/deferred-debugger path to admit. This does not extend the caller deadline or imply rollback. Human editing is not disabled; changed target evidence invalidates the attempt. This is local kit-entry serialization, not coordination across editors/worktrees or exclusion of arbitrary external writers.

## 2. Evidence records

### SurfaceEvidence

Reuses independent authority, availability, identity, collection stamp and invalidation meanings from observation. The mutation reducer obtains exact text independently and compares it internally. The public edit result carries digest/byte length and witnesses rather than duplicate full before/after source strings:

`authority`, `availability`, `source_sha256`, `utf8_bytes`, `collection`, `identity`, `reason`, and optional invalidated evidence.

Only actual observations get hashes. Unavailable is not an empty string or equality. D comes from Rust's confined reader; R/B come from separate actual editor getters. The native writer receipt is not D evidence. Denied/ambiguous targets suppress source-derived evidence and unauthorized paths. Previously valid evidence retains its original interval and invalidation reason after interruption; no replacement-document facts are mixed into it.

### SavedStateEvidence

Selected **read-only available evidence** is the exact still-open Script/editor/CodeEdit association and resource path, current public `EditorInterface.is_object_edited(Script)` flag, CodeEdit current/saved versions and dirty state, actual R/B, independently obtained D/namespace/metadata, save-format profile and collection interval. Preparation captures clean baseline current/saved versions and T0 from `fstat` on the retained exact-file descriptor. After one native complex edit, freeze the **actual post-edit current** version with the **original preparation-time saved** version; guard that pair through explicit R synchronization, same-descriptor content persistence, T0 mtime restoration/readback and every pre-tag check. Following guarded edited=false and guarded direct `CodeEdit.tag_saved_version()`, independently observe saved version equal to that post-edit current version. A finalizer acknowledgment or its own step flags are **not** independent inspection. At verified success the target must independently remain attributable as clean with public Resource edited false, intended current/saved versions equal, and D/R/B, descriptor attachment and namespace agreeing without invalidation. The earlier private `inspect_script_document` numeric Resource/document mtime proposal and earlier saved-state candidates in §§14–17 (including §17's handler-free un-restored direct tag) are historical, not selected prerequisites; T0 preservation is checked as filesystem metadata, not forged private-field parity. §18 provides bounded behavioral native Save/history/reopen evidence for the tested standalone Script lifecycle, not a general engine/platform guarantee or Feature 002 acceptance.

**T001 implementation/layout boundary:** The completed then-approved typed core still defines `SavedStateEvidence.resource_mtime` and `document_mtime` in [evidence.rs](../../mcp-server/src/script_edit/evidence.rs#L57-L72); its [preparation/verification reducer](../../mcp-server/src/script_edit/attempt/observations.rs#L158-L166) requires both to equal independent D metadata mtime, and its [finalization checks](../../mcp-server/src/script_edit/attempt.rs#L582-L600) require corresponding before/after numeric fields. Stock T1 does **not** provide those private values, so the existing T001 reducer cannot consume this selected semantic evidence as-is. T001's historical acceptance remains complete. Later T004 integration must coherently migrate the affected typed evidence, reducer and contracts before using T1 stock evidence; do not fabricate private mtime values or claim a finalized replacement Rust/wire shape now.

### PersistenceReceipt

Native-only bounded application evidence defined in [Primitive A's contract](contracts/native-integration.md#2-common-bound-state-and-persistence-receipt). Public/bridge results contain a non-authorizing summary: attempt/target binding, write-start/byte/truncate/flush/content-readback facts, same retained descriptor/namespace identity, captured T0, actual timestamp-restoration attempt/error and `fstat`/content readback, and unknowns. `futimens` on that retained descriptor restores mtime T0 while leaving atime unchanged (`UTIME_OMIT`); namespace attachment, identity, source and restored mtime must be freshly verified before any saved tag. A successful content write without verified restoration is known partial application, not eligible persistence. Never serialize a reusable native handle or accept a result-shaped receipt from a caller.

### FinalizationResult

This is a **semantic** local finalization result, not a finalized new native API/bridge schema: record whether the complete same-attempt content-and-T0-restoration receipt and fresh binding/namespace/source/current-and-saved-version guards passed, whether the public edited flag was cleared, whether renewed guards passed and the direct public CodeEdit saved-version tag was entered/completed, plus any independently observed public state or unknowns. `complete|rejected|partial_or_unknown` describes this stage only, with bound attempt/document and interval; do not invent private numeric mtime fields or infer success from the setter/tag return. A rejected finalizer says nothing by itself about whether B/R/D or metadata already changed; a cleared flag followed by failed pre-tag recheck is partial. Preserve new human work and actual effects; no rollback field is inferred.

### ValidationResult

`status: valid|invalid|unavailable`, reason, purpose, request/session/document binding, exact input path/hash/length, editor-clock invocation interval, dependency/context witnesses, diagnostics and completeness. Actual parser/analyzer completion determines valid/invalid; unsupported effects, incomplete or invalidated context make the result unavailable. Root and dependency diagnostics retain distinct origins and source attribution. Valid does not mean bytecode/gameplay success or immutable dependencies. See [Primitive B](contracts/native-integration.md#4-primitive-b-exact-source-native-gdscript-validation).

### AttemptProgress and EditOutcome

Progress is monotonic stage knowledge, not an assumption that every prior effect completed. Record separate `buffer_application`, `resource_sync`, `persistence`, `finalization`, `validation`, and `verification` states as `not_started`, `entered`, `completed`, `failed`, or `unknown`, with evidence/reason. Keep the greatest safely established application knowledge alongside any later failure; content persistence, T0 mtime restoration, edited-flag clearing, direct saved tagging and separate exact-source validation must not be conflated.

Public EditOutcome has these required top-level fields; nullable fields never invent unavailable facts:

| Field | Value |
|---|---|
| `schema_version`, `operation`, `request_id` | `1`, `edit_open_gdscript`, current request. |
| `requested_target`, `resolved_target` | Explicit request selectors; verified resolved identity or null. |
| `interval`, `outcome`, `reason`, `stage` | Actual overall timing, terminal enum, cause and furthest reached stage. |
| `application`, `progress` | Application knowledge: `not_applied`, `applied`, `partly_applied` or `unknown`; per-stage AttemptProgress. |
| `expected` | Checked ExpectedRevisionBasis or null when missing/invalid/suppressed. |
| `before`, `after` | Nullable independently collected EditEvidence records described below. |
| `persistence`, `finalization`, `validation` | Nullable persistence summary/A result; validation records with their explicit purposes, empty when not invoked. |
| `history` | `not_participated`, `native_complex_edit` or `unknown`; participation is not proof of actual Undo/Redo acceptance. |
| `diagnostics`, `selection`, `safe_next_action` | Bounded cause/action records; nullable source-free disambiguation metadata; safe next action. |

Each EditEvidence contains `document` (open/identity/validity evidence), `sources` with D/R/B SurfaceEvidence, independent `dirty`, `saved_state`, `comparisons`, `agreement`, and `consistency` (actual rechecks and detected invalidations). Unavailable records retain reasons; current and invalidated evidence stay separate. Thus dirty state and uncertainty are visible without inferring them from source equality. Denial/ambiguity suppress source-derived fields, including expected hashes, while preserving non-source application/stage knowledge if an earlier effect was already established.

| Outcome | Required meaning |
|---|---|
| `verified_changed` | A real native source edit occurred; all fresh independent source, clean/saved-state and post-change validation conditions passed without invalidation. |
| `verified_unchanged` | Desired text was already current; all safety/validation/verification checks passed with no source, save, bookkeeping or history mutation. |
| `refused` | Proven not applied and irrevocably unable to apply later from this attempt; reason identifies input/target/conflict/capability/error/interruption. |
| `applied_unverified` | A source/history/persistence/bookkeeping change is known, but required later evidence failed, is missing or was invalidated. Includes partly applied cases. |
| `application_unknown` | A mutating command may have reached the editor but actual application cannot be determined. Missing acknowledgment is not not-applied proof. |

Reason categories include busy admission, dirty conflict, revision mismatch, identity/session change, divergence, known stale R/B, unavailable observation, closed/unsupported target, ambiguity/access denial, unsupported engine/effect/representation, persistence failure/unknown, partial finalization, parse/dependency error, validation unavailable, deadline, cancellation, disconnection and protocol failure. Keep stage and reason separate; a parse error after a write is not a clean pre-application refusal.

## 3. Ordered attempt state machine

1. **Accepted:** parse/bound input; start caller monotonic deadline before blocking work. No mutation authority yet.
2. **Selected:** existing source-free mutual authentication and unique project/session resolution; verify required native capabilities and exact candidate identity. No convenient replacement session.
3. **Prepared:** obtain fresh independent D/R/B, dirty/open/identity and saved-state witnesses; compare with expected basis. Pin native descriptors without writing and read back native baseline. For changed intent, preflight-validate proposed source under the selected effect policy; bad/unsupported proposed validation refuses before source application. Unchanged intent skips that duplicate validation and follows its separate branch below.
4. **Authorized:** only after validated preparation, the supervisor sets `may_apply` **before** sending a private one-shot authorization to its worker. No editor mutation request can precede that authorization. This is a dispatch safety boundary, not user approval or a new transaction framework.
5. **Applying:** editor rechecks actual current target/revision/dirty state, connection/expiry and representation. Immediately before the first native operation that can affect source/history (including complex-operation grouping), record application as potentially entered. Apply one native CodeEdit complex operation; freeze its current version and the independently prepared saved version, then synchronize R through the explicit `Script.set_source_code` setter with fresh guards. Generic forbidden-effect safety for ordinary edit-generated editor work remains mandatory; no saved handler is invoked.
6. **Persisting:** recheck current R/B/current-and-saved versions/identity and expected D bytes/identity; same retained exact-file descriptor write/truncate/flush/content-readback and no-follow current namespace check. With fresh guards and an eligible same-attempt content receipt, restore **that descriptor's** mtime to its preparation-time T0 via `futimens` (`UTIME_OMIT` for atime), then `fstat`/read back and recheck attachment, identity, namespace, source and versions. Do not retarget or tag on a missing/failed/unverified restore. Known content changes followed by timestamp failure remain `applied_unverified` with actual partial facts, not `refused`, success or implicit rollback; no second writer or saver fallback.
7. **Finalizing:** only with a complete same-attempt content-and-restored-T0 receipt and fresh exact still-open Script/editor/CodeEdit/path/source/current-and-saved-version/namespace guards, call `EditorInterface.set_object_edited(Script, false)`. Renew the guards, call public `CodeEdit.tag_saved_version()` for that exact buffer and **stop**; do not discover/invoke/emit a saved-handler Callable/Signal, Save, reload or dispatch another write. Reject or report partial on invalidation, never restore newer text.
8. **Validating:** independently read actual resulting source/identity and obtain qualifying stock explicit exact-source `post_change` `valid|invalid|unavailable` validation. Preflight or incidental ordinary editor validation is not a substitute. Its exact-source attribution, dependency/context integrity and generic effect safety remain the separate unresolved architecture blocker; invalid/unavailable/context-changed results prevent success without erasing applied facts.
9. **Verifying:** Rust independently reads D/attachment; integration independently rereads R/B, public edited flag, dirty/current/saved versions and association. Recheck target, sources, dependency/context witnesses and deadline. Do not demand nonexistent numeric private Resource/document mtime inspection or treat the content/metadata receipt or A's returned status as independent evidence. Classify once, only in Rust/core.
10. **Terminal:** finish the caller outcome and ignore late frames; native state/slot cleanup follows the lifetime rule above, not the caller's return time. No automatic retry, reconnect/resume, rollback, or outcome upgrade.

For **unchanged** intent, branch after fresh preparation checks directly to `unchanged` validation and independent verification. Do not dispatch authorization, write D, restore metadata, clear edited state, tag saved state or add history. If unchanged source is invalid or required publicly available saved-state evidence is missing, return a truthful refusal/non-success rather than verified unchanged.

## 4. Deadline, cancellation and application certainty

The edit supervisor uses **9.5 seconds** for acquisition/operation evidence and reserves **0.5 seconds** for bounded result delivery in the controlled consuming caller. Reuse the existing same-binary supervised worker and nonblocking reap strategy; do not kill the user's editor or wait indefinitely for blocked filesystem/editor work. Observation retains its existing 4.5/5-second behavior.

The editor establishes a local monotonic expiry from the remaining budget supplied during preparation (capped at 9000 ms), checks it before every mutating stage and after blocking work, and refuses new stages after expiry. Clock domains are not compared directly. A transport delay or already-entered native call can outlive the caller; report uncertainty, not a false global cancellation guarantee.

| Known facts at termination | Classification obligation |
|---|---|
| Supervisor never authorized mutation | Not applied: preparation/validation are contractually source/history read-only, and worker cannot issue apply independently. |
| Authorization released, but no terminal editor proof | `application_unknown` unless earlier valid evidence already proves a change. Parent state is set before dispatch so a worker crash cannot hide this possibility. |
| Editor permanently rejected/discarded before boundary | `refused`; context is terminal/consumed, duplicate/late apply cannot run. |
| B/R change known; persistence unavailable/failed | `applied_unverified` with actual stage/partial facts. |
| Content persisted but T0 restoration/readback failed or is unavailable | `applied_unverified` when actual changed content is known; keep the persistence/finalization stage and error/partial metadata facts. Never tag or claim rollback/refusal/success. |
| Persistence completed; finalization failed or partial | `applied_unverified`, including saved/dirty evidence as observed, not assumed. |
| Finalization complete; validation/verification unavailable | `applied_unverified`; finalizer acknowledgment is not success. |
| Post-change validation failed or later human activity invalidated evidence | `applied_unverified`; preserve new work, no reassertion/rollback. |

Cancellation/EOF/protocol failure/deadline is terminal for the caller and prevents success. After authorization, only an attributable irreversible pre-boundary discard can prove not applied. An `abort` attempt without that acknowledgment is not proof. Once an effectful stage has entered, cancellation cannot promise that the stage has not or will not finish; stop subsequent stages when control returns. Attempt-local cleanup does not undo effects.

## 5. Outcome precedence and invariants

First validate identity/order/bounds of each evidence event. Denial/ambiguity suppresses unauthorized source-derived data. Known effects and `may_apply` survive subsequent transport/protocol failures. Reason precedence follows existing denial/target/protocol/interruption distinctions, but no reason may downgrade known application to refused. Dirty/revision/identity failures before boundary refuse; afterward they preserve application facts and invalidate success. Unknown attribution is never silently repaired by a new sample.

Verified success requires all relevant final checks, no overriding terminal condition, exact independent byte equality to intended source, actual still-open association/namespace, verified same-descriptor restoration/readback of T0 for changed intent, public edited flag false, current and saved buffer versions equal at the intended source, clean dirty-state observation and qualifying stock post-change exact-source validation/effect safety. Do not claim atomic history against arbitrary non-cooperating same-inode writers; known interference still invalidates success even if later bytes match. Native receipt/hash/mtime, absence of errors, or the mutation/finalizer response alone cannot pass verification. The historical numeric private Resource/document mtime comparison is not a selected success condition.

Implementation tests must cover boundaries and transitions rather than source-text wiring: no authorization/no late application; authorization/lost acknowledgment; changed same-text buffer version; descriptor namespace loss; partial write/bookkeeping; dependency invalidation; unchanged invalid source; wrong-session/source validation; native history and all A–E/durability requirements in real Godot.
