# Data Model: Guarded Open-GDScript Editing

**Status:** T001's protocol-independent typed core is implemented and verified; native integration, caller execution and mutation acceptance remain pending. This model adds one mutation operation, not a generalized transaction framework. [Spec](spec.md), [caller contract](contracts/edit-api.md), [private bridge](contracts/bridge-protocol.md), and [native primitives](contracts/native-integration.md) define the complete boundary.

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

One authenticated connection/request-local in-memory record owned by the Godot integration. It retains actual object associations, the immutable intended source, fresh native guard witnesses, the pinned project/file descriptors, editor-clock expiry and current stage. It owns the engine document guard, one persistence receipt and bounded validation records. No durable operation registry, replay cache, journal or background service.

Only one active read collection **or** edit attempt is admitted by the existing editor integration slot for the selected editor/session. This also serializes kit mutation entry: only the slot-owning attempt may cross the potentially-applied boundary. Excess edit attempts receive a terminal, source-free `busy` refusal with zero source/history/finalization change; no preparation or apply work is queued for them. Slot release never resumes, retries or replays a rejected attempt. A later edit requires a new request with a freshly observed valid basis; a changed revision cannot be bypassed by compatible or equal desired text.

The admitted attempt owns the slot from preparation through terminal cleanup. Disconnect/expiry/disable prevents new stages, but does not release the slot or live native references/descriptors while already-entered work could still mutate; cleanup waits for that work to return and prevents deferred effects before admitting another attempt. This does not extend the caller deadline or imply rollback. Human editing is not disabled; changed target evidence invalidates the attempt. This is local kit-entry serialization, not coordination across editors/worktrees or exclusion of arbitrary external writers.

## 2. Evidence records

### SurfaceEvidence

Reuses independent authority, availability, identity, collection stamp and invalidation meanings from observation. The mutation reducer obtains exact text independently and compares it internally. The public edit result carries digest/byte length and witnesses rather than duplicate full before/after source strings:

`authority`, `availability`, `source_sha256`, `utf8_bytes`, `collection`, `identity`, `reason`, and optional invalidated evidence.

Only actual observations get hashes. Unavailable is not an empty string or equality. D comes from Rust's confined reader; R/B come from separate actual editor getters. The native writer receipt is not D evidence. Denied/ambiguous targets suppress source-derived evidence and unauthorized paths. Previously valid evidence retains its original interval and invalidation reason after interruption; no replacement-document facts are mixed into it.

### SavedStateEvidence

A separate read-only engine inspection returns document path/mtime baseline, Resource edited/mtime fields, CodeEdit current/saved versions, object association, current native save-format profile and collection interval. Compare with the independent current disk metadata and the intended source. The finalizer's own returned step flags are not this inspection. At verified success, the target is independently attributable as clean, its saved version equals its current intended buffer version, and Resource/document save metadata agrees with current target persistence evidence.

### PersistenceReceipt

Native-only bounded application evidence defined in [Primitive A's contract](contracts/native-integration.md#2-common-bound-state-and-persistence-receipt). Public/bridge results contain a non-authorizing summary: stage/binding, write-start/byte/truncate/flush/readback facts, current attachment, descriptor identity/mtime, and errors/unknowns. Never serialize a reusable native handle or accept a result-shaped receipt from a caller.

### FinalizationResult

`status: complete|rejected|partial_or_unknown`, reason, bound attempt/document, interval, before/after source/version/metadata observations and per-step bookkeeping completion. A rejected finalizer says nothing by itself about whether B/R/D already changed. Partial effects remain evidence; no rollback field is inferred.

### ValidationResult

`status: valid|invalid|unavailable`, reason, purpose, request/session/document binding, exact input path/hash/length, editor-clock invocation interval, dependency/context witnesses, diagnostics and completeness. Actual parser/analyzer completion determines valid/invalid; unsupported effects, incomplete or invalidated context make the result unavailable. Root and dependency diagnostics retain distinct origins and source attribution. Valid does not mean bytecode/gameplay success or immutable dependencies. See [Primitive B](contracts/native-integration.md#4-primitive-b-exact-source-native-gdscript-validation).

### AttemptProgress and EditOutcome

Progress is monotonic stage knowledge, not an assumption that every prior effect completed. Record separate `buffer_application`, `resource_sync`, `persistence`, `finalization`, `validation`, and `verification` states as `not_started`, `entered`, `completed`, `failed`, or `unknown`, with evidence/reason. Keep the greatest safely established application knowledge alongside any later failure.

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
5. **Applying:** editor rechecks actual current target/revision/dirty state, connection/expiry and representation, then enters the document guard. Immediately before the first native operation that can affect source/history (including complex-operation grouping), record application as potentially entered. Apply one native CodeEdit complex operation; record resulting version and synchronize R through the explicit source setter with fresh guards.
6. **Persisting:** recheck current R/B/version/identity and expected D bytes/identity; descriptor-bound write/truncate/flush/readback. Any known new work stops further obsolete writes. A failure retains actual partial changes.
7. **Finalizing:** Primitive A consumes eligible receipt facts and performs guarded target-only saved bookkeeping. Failure stops; do not overwrite newer work to repair it.
8. **Validating:** independently read actual resulting source/identity and perform Primitive B as `post_change`; a preflight result cannot satisfy this stage. Invalid/unavailable/context-changed results prevent success but do not erase applied facts.
9. **Verifying:** Rust independently reads D; integration independently rereads R/B/dirty and saved-state inspection. Recheck target, sources/versions, dependency/context witnesses and deadline. Classify once, only in Rust/core.
10. **Terminal:** finish the caller outcome and ignore late frames; native state/slot cleanup follows the lifetime rule above, not the caller's return time. No automatic retry, reconnect/resume, rollback, or outcome upgrade.

For **unchanged** intent, branch after fresh preparation checks directly to `unchanged` validation and independent verification. Do not dispatch authorization, enter a mutating document guard, write D, tag saved state or add history. If unchanged source is invalid or required saved-state evidence is missing, return a truthful refusal/non-success rather than verified unchanged.

## 4. Deadline, cancellation and application certainty

The edit supervisor uses **9.5 seconds** for acquisition/operation evidence and reserves **0.5 seconds** for bounded result delivery in the controlled consuming caller. Reuse the existing same-binary supervised worker and nonblocking reap strategy; do not kill the user's editor or wait indefinitely for blocked filesystem/editor work. Observation retains its existing 4.5/5-second behavior.

The editor establishes a local monotonic expiry from the remaining budget supplied during preparation (capped at 9000 ms), checks it before every mutating stage and after blocking work, and refuses new stages after expiry. Clock domains are not compared directly. A transport delay or already-entered native call can outlive the caller; report uncertainty, not a false global cancellation guarantee.

| Known facts at termination | Classification obligation |
|---|---|
| Supervisor never authorized mutation | Not applied: preparation/validation are contractually source/history read-only, and worker cannot issue apply independently. |
| Authorization released, but no terminal editor proof | `application_unknown` unless earlier valid evidence already proves a change. Parent state is set before dispatch so a worker crash cannot hide this possibility. |
| Editor permanently rejected/discarded before boundary | `refused`; context is terminal/consumed, duplicate/late apply cannot run. |
| B/R change known; persistence unavailable/failed | `applied_unverified` with actual stage/partial facts. |
| Persistence completed; finalization failed or partial | `applied_unverified`, including saved/dirty evidence as observed, not assumed. |
| Finalization complete; validation/verification unavailable | `applied_unverified`; finalizer acknowledgment is not success. |
| Post-change validation failed or later human activity invalidated evidence | `applied_unverified`; preserve new work, no reassertion/rollback. |

Cancellation/EOF/protocol failure/deadline is terminal for the caller and prevents success. After authorization, only an attributable irreversible pre-boundary discard can prove not applied. An `abort` attempt without that acknowledgment is not proof. Once an effectful stage has entered, cancellation cannot promise that the stage has not or will not finish; stop subsequent stages when control returns. Native guard cleanup does not undo effects.

## 5. Outcome precedence and invariants

First validate identity/order/bounds of each evidence event. Denial/ambiguity suppresses unauthorized source-derived data. Known effects and `may_apply` survive subsequent transport/protocol failures. Reason precedence follows existing denial/target/protocol/interruption distinctions, but no reason may downgrade known application to refused. Dirty/revision/identity failures before boundary refuse; afterward they preserve application facts and invalidate success. Unknown attribution is never silently repaired by a new sample.

Verified success requires all relevant final checks, no overriding terminal condition, exact independent byte equality to intended source, clean/saved-state evidence and valid post-change parse attribution. Do not claim atomic history against arbitrary non-cooperating same-inode writers; known interference still invalidates success even if later bytes match. Native receipt/hash/mtime, absence of errors, or the mutation response alone cannot pass verification.

Implementation tests must cover boundaries and transitions rather than source-text wiring: no authorization/no late application; authorization/lost acknowledgment; changed same-text buffer version; descriptor namespace loss; partial write/bookkeeping; dependency invalidation; unchanged invalid source; wrong-session/source validation; native history and all A–E/durability requirements in real Godot.
