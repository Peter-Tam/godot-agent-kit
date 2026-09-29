# Data Model: Known-Path GDScript Opening

**Status:** Phase 1 design, not implemented types or acceptance evidence. Implements [spec.md](spec.md) under the [plan](plan.md). JSON belongs to the [caller](contracts/open-api.md) and [bridge](contracts/bridge-protocol.md), not the domain reducer.

## 1. Shared values and ownership

Reuse the existing checked `RequestId`, `ProjectRoot`, optional `SessionId`, external `ResourcePath`, `ResolvedTarget`, file/document identities, source/dirty availability, collection stamps, invalidation and observation interval semantics. Their formats are defined by the existing [observation data model](../001-observe-gdscript-state/data-model.md) and [caller contract](../001-observe-gdscript-state/contracts/observation-api.md).

- Request IDs: existing safe ASCII format, ≤64 characters; CLI generates a fresh 32-hex ID. IDs are correlation, not idempotency keys.
- Session IDs: 32 lowercase hexadecimal characters, bound to the actual authenticated editor lifetime.
- Canonical absolute project root ≤1024 UTF-8 bytes; exact `res://` path ≤2048 bytes. No basename/focus selection, URI decoding, path normalization into a different target or outside-project escape.
- Source hashes: SHA-256 of exact UTF-8 bytes, with independently checked byte length. Empty source is observed empty, not unavailable.
- File/device/inode/object/version counters retain existing checked decimal-string formats at JSON boundaries. Native/editor and supervisor clocks are distinct; no cross-clock numerical equality is assumed.
- Source availability remains observed/unavailable/not-applicable with a reason; invalidated evidence is retained only as invalidated, not current. Equality does not imply dirty-state knowledge, freshness or parse validity.

Public Rust consumers need the checked opening request, immutable outcome and runner entrypoint. Attempt state, context source, guards, receipts and codec models remain internal/crate-visible unless an actual current consumer requires more. Move or borrow existing evidence; do not copy full source into every progress event or final before/after record.

## 2. Entities

### OpenRequest

| Field | Meaning / validation |
|---|---|
| `request_id` | Fresh checked request correlation. |
| `project_root` | Explicit selected project; same checked type as observation. |
| `session_id` | Exact lifetime if supplied; omission requires unique authenticated selection. |
| `script_path` | Standalone external `.gd` locator, exact spelling. |

No replacement source, prior edit basis, force/reload, focus control, permission override, retries or arbitrary command. The adapter may privately wrap/borrow an `ObservationRequest` to reuse target resolution; it must not turn observation into opening.

### OpeningBinding

The resolved target plus authenticated bridge/native identities, existing connection/request slot ownership, request-local deadline and file identity. It binds the actual project device/inode, session, engine/native build, exact resource path and attempt. A new session/cache/document cannot inherit it. This is private transaction correlation, not a public mutation token or additional transaction/session identifier.

### CapturedSource

Selected-path D text, SHA-256/length, file identity/stat witness, read interval, confinement/namespace recheck evidence and availability. The worker acquires it independently through the selected project capability. The native boundary independently pins/read-checks the same file before consuming the capture. No caller can provide these bytes as an arbitrary source payload.

A cold new-open requires an in-limit supported capture. A cached target requires separately observed R == D plus attributable unedited Resource state. Read-only files are eligible; no write descriptor or writable-mode check belongs to opening. Missing/denied/changed source before effects refuses rather than using cached text as D.

### TargetState

| State | Required facts |
|---|---|
| `open` | Unique actual Script/ScriptEditorBase/CodeEdit association, exact path and object IDs; independently observed open state. |
| `closed_cached` | Confirmed no target buffer; exact cached GDScript reference, source and Resource edited-state witness. |
| `closed_uncached` | Confirmed no target buffer and passive cache lookup establishes absence. Absence is not inferred from a failed lookup. |
| `unknown` | Missing/ambiguous open or cache attribution; reason retained. No new opening permitted. |

Wrong-type cache occupants, duplicate document paths, changed objects, embedded/non-GDScript paths and unsupported associations have distinguishable refusal reasons. Already-open recognition may retain unavailable D/R/B; target/access-denial precedence and source-suppression rules still apply. A confirmed closed missing file is a missing target, not a valid closed source.

### OpeningContext — private

The minimum editor state needed because native new-open navigation can apply or validate a departing document:

- `kind`: `no_source_editor`, `current_gdscript` or `unavailable`. `get_current_editor`/`get_current_script` and actual association establish it; a missing source getter does not mean no current editor. Unknown/mixed unsupported source-editor context refuses.
- For a current GDScript: exact path/project affiliation, Script/editor/buffer IDs, independently acquired R and B, hashes/lengths, current/saved versions, attributable dirty state and history availability. Require R == B without inferring clean state.
- Effective internal-editor configuration, source profile, project global-class/autoload names relevant to admission, built-in/extension class bindings and warning/context evidence needed by existing stock validation. All are bounded and rechecked, not assumed from `project.godot` alone.
- Passive compiled witnesses: tool flag, script-base reference, property/method metadata. Reject unsupported exported/Object-valued/dynamic-property/stale-base context rather than assert an unobservable pending-export flag. Exact rules are in the native contract.
- `validation`: not-applicable only for a proven no-source context; otherwise complete valid source-only validator evidence bound to this exact current source/path/session/request/context. Invalid/unavailable current validation refuses before authorization.

Current/unrelated source and paths stay private to safety processing and owned temporary validation. Public results report only a bounded context status/reason; they never disclose another document's source as target evidence. A dirty current document may be supported; no product operation waits for its R/B to converge or changes it to make it eligible.

### PreparedOpening

Immutable binding, target mode, captured D, retained cached R if any, native file/reference guards, context witness, optional private current-validation receipt and deadline. It owns no source write or published new Resource. Preparation must be incapable of applying after a terminal cancellation unless a separately recorded authorization was already released; in that case uncertainty is retained until irrevocable discard is proved.

### NativeLifecycleFacts

Monotonic facts, independently attributed to the existing request/session/connection owner:

| Fact | Meaning |
|---|---|
| `cache_binding` | Not started, reused existing R without effect, new Resource published, failed before publication, or unknown. New publication is a lifecycle effect. |
| `initial_compilation` | Not applicable for retained R; for a new object: not started, completed valid, completed invalid with an attributable engine error, unavailable/failed, or unknown. Never reload existing R. |
| `document_open` | Not started, entered, actual new document association obtained, failed/unverified, or unknown. Entry/acknowledgment is not independent final verification. |
| `selection` | Native selection effect known/unknown and requested-target/other/no-source relation; no unselected source or project path is exposed. |
| `terminal_discard` | Attempt cannot accept any future lifecycle stage; identifies the owned request and whether any effect was already known. |

An unbound temporary native object is not the target R. Native code may prove no target effect if it fails before cache publication/opening and irreversibly discards all work. Once publication/document/selection effect is established, later failure cannot reduce application to not-applied. Releasing references is ordinary ownership cleanup, not a claimed rollback of editor state.

### VerificationEvidence

Fresh target observation from the existing collector, independent fresh D, same-identity/namespace/source/context rechecks, actual open/buffer identity and dirty state, observation interval and detected invalidation. Neither the captured source nor native initialization receipts may stand in for these reads.

For new opening, require all applicable D/R/B observed and exactly equal to the admitted source; target open with the exact expected Script and actual buffer; clean attributable buffer dirty state and separately observed Resource edited=false; stable required file/session/document/protection witnesses. Source/profile/context changes that invalidate authorization prevent success even if later text happens to match. Resource edited state is opening-specific supplementary evidence, not a change to observation v1.

Already-open recognition requires exact currently established open identity, no lifecycle authorization/effects, and the necessary identity/open-state recheck. Its source observations may be dirty, divergent, limited or explicitly invalidated; they do not become complete/coherent merely because the document was recognized. If open identity itself becomes unknown/changed, return a no-effect refusal/interruption instead.

### OpeningOutcome

Immutable public domain result containing:

- Schema/operation version at the adapter, request and requested/resolved target identity.
- Overall outcome, reason, reached stage and application certainty.
- Bounded per-stage effect/progress facts and actual observation interval.
- Before summaries (availability/hash/length/identities, not duplicate source bodies) and at most one latest authorized target source snapshot.
- Target parse evidence if actually obtained, separate from opening success. Cached R is not recompiled to fabricate a parse result. Missing parse evidence is explicit, not a blocker to otherwise verified opening.
- Private-context disposition without current/unrelated source; detected changes and safe source-free diagnostics.
- History participation `not_participated` and relevant observed preservation evidence/limitations; selection effect and safe next action.

A parse-invalid newly opened target can be a successful open. An invalid current context is an admission refusal, not a target parse error. Only the existing edit operation promises valid post-edit parse evidence.

## 3. Ordered attempt states

```text
Accepted -> Selected -> Prepared
                         |
                         +-- already open -> Observe/recheck identity -> Terminal
                         |
                         +-- closed -> Validate current context (or proven N/A)
                                      -> Authorized
                                      -> Bind/reuse Resource
                                      -> Compile new Resource only
                                      -> Open document
                                      -> Independently observe/recheck
                                      -> Terminal
```

Stages exposed by the caller are `accepted`, `selected`, `prepared`, `context_validating`, `authorized`, `binding`, `compiling`, `opening`, `verifying`, `verified`. Omitted stages are not fabricated: cached R skips compilation, no-source current context skips validation, and already-open skips every authorization/effect stage.

Use one attempt-state enum carrying the data valid for that state, not several booleans that allow impossible lifecycle combinations. Native session ownership likewise admits exactly one read/edit/open owner and prevents multiple live effectful attempt pointers. No generalized transaction framework is introduced.

Preparation remains non-applying. Parent authorization changes knowledge to **may apply** before the control write. Child/bridge/native progress must be monotonically ordered and bound to the same attempt. A queued, stale, expired or mismatched stage is not a new prepare and cannot resurrect a discarded attempt. Native `entered` ownership survives cancellation/disable until the synchronous call returns.

## 4. Terminal reduction and precedence

| Outcome | Required meaning |
|---|---|
| `verified_newly_opened` | Request caused a new open; fresh independent target and protection postconditions passed. Application is applied. |
| `already_open_unchanged` | Exact existing document recognized; no lifecycle authorization/effect; observations retain their own limitations. Application is not-applied. |
| `refused` | No lifecycle effect occurred and none can occur later from this attempt. Specific input/target/context/evidence/capability reason. |
| `applied_unverified` | Some lifecycle effect is known but verification is incomplete/invalidated or the operation only partly applied. Preserve known effects and missing facts. |
| `effects_unknown` | Authorization/dispatch may have led to effects, but their occurrence/extent is not established. No false no-effect or rollback claim. |

Reduction rules:

1. Never disclose source before exact authorized target selection; ambiguous/denied attribution suppresses unauthorized source even if a malformed frame supplied it.
2. Validate every fact's request/session/target/connection ownership/clock/sequence before use. Invalid later messages cannot erase earlier valid known effects or upgrade terminal success.
3. If effects are known, any timeout, disconnection, protocol failure, missing observation, context change or divergence produces `applied_unverified`, not `refused`/`effects_unknown` merely because later evidence is missing.
4. If authorization was released and no valid effect or irrevocable no-effect proof exists, use `effects_unknown`.
5. Before authorization, or after an attributable irrevocable pre-effect discard, report `refused` for failures. A refused busy request is never retained/queued for later entry.
6. Only evaluate positive outcomes after all of their distinct verification conditions. Already-open does not require clean/equal/full source; newly opened does.
7. A known source/context invalidation cannot be erased by eventual equality. Preserve newer human work; do not reassert captured bytes, Save or close a buffer to repair the result.

`application` is `not_applied`, `applied`, `partly_applied` or `unknown`. It describes actual lifecycle knowledge, not source bytes written. `partly_applied` includes a new cached Resource without a verified document. `stage` is the furthest entered stage, not proof that it completed.

## 5. Deadlines, cancellation and retry

The existing supervisor clock starts before CLI parsing/resolution. Acquisition/effects/verification share 9.5 seconds; final delivery has a 0.5-second reserve with a draining stdout consumer. Native deadlines use editor-local remaining-time limits; the parent never numerically compares unrelated clocks.

Timeout does not cancel a synchronous native call. Parent terminates/reaps only owned worker/helper processes; editor/native ownership remains safe until entered work returns. Each new stage must check expiry/identity/source/context before effects. Late evidence cannot amend a delivered outcome. There is no automatic reconnect, retry, queued resume or response replay cache.

After any possibly applied result, freshly observe the original explicit target and inspect human work before another intentional operation. A new request for a still-open document may then be satisfied without effects. This is conditional no-change behavior, not unconditional idempotency across close/reopen, replacement or session restart.

## 6. Privacy, size and compatibility invariants

Public output includes only the authorized requested target's source; private current-source validation never becomes a second source result. Preserve actual empty values and per-surface unavailable reasons. Never truncate text to fit a frame or infer cleanliness from equality. Use before summaries plus one final snapshot to avoid unnecessary source copies and result inflation.

Retain 512 KiB/source, 4 MiB selected-peer request, 12 MiB response/worker frame, 4 KiB source-free control and JSON depth 32 limits. Bounded source/profile/compiled/context collections are specified in the contracts. A limit during new-open preparation refuses before effects where observable; a newly discovered post-effect limit produces an unverified result. Already-open recognition retains the existing independent source-limit semantics.

Opening v1 is a new public operation. Observe/edit v1 semantics stay unchanged. Private v3/native revision 2 is a coordinated clean cutover, including the shared native bundle rename. No obsolete aliases, old mutation fallback or feature/task IDs in shared APIs. This model changes no current implementation until an approved task is selected.
