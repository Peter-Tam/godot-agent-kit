# Data Model: Script Existence Transactions

**Status:** Phase 1 design, not implemented. [Specification](spec.md), [public contract](contracts/mcp-interface.md) and [integration contract](contracts/existence-transactions.md) govern the two new operations. Existing read/discover/edit records and `sr1` commitments remain unchanged.

## Ownership and bounds

MCP owns wire arguments, schemas and public projection. The core owns checked intents, fresh revision comparison, evidence validity and terminal reduction. Native/editor integration owns exact objects, operation lifetime and Godot effect facts. Filesystem observations remain independent of Resource/buffer observations.

Retain the [existing supported profile and bounds](../007-mcp-script-workflow/data-model.md#2-supported-profile-and-bounds): 1024-byte project selector, 2048-byte locator, 512 KiB per source, strict LF UTF-8 without NUL/CR/U+FEFF, 16 MiB MCP input/tool-object cap, JSON depth 64, 64 MiB serialized response cap, one admitted tool/no queue, eight outstanding protocol IDs. The supported source-only profile excludes unsafe tool/global-registration/dependency/effective contexts. Creation additionally requires all parent directories already exist. Known-path admission does not require inventory membership or a discovery call; discovery's own visibility/coverage rules stay unchanged.

Create/delete use the original ten-second caller clock and 9.5-second work cutoff. Authentication, acquisition, native waits, validation, cleanup and result delivery consume that clock; no stage restarts it. Existing validator budgets only shorten the remaining allowance. Open deletion inherits the close protection limits: eight total script documents, 512 KiB aggregate protected source, 256 KiB non-source context metadata. Exhaustion is not permission to omit an authority/document.

## Checked intent and basis

| Record | Required content and invariant |
| --- | --- |
| `CreateScriptRequest` | Checked project/session/path plus one owned complete source value. No revision, replacement mode, directory instruction or overwrite flag. |
| `DeleteScriptRequest` | Checked project/session/path plus existing `ScriptRevision`. No supplied source or lifecycle selector. |
| `CreationBasis` | Authenticated session/project identity; retained existing parent chain/leaf name; successful no-follow D-absence witness; positively absent B and R; native close epoch, source-only/effective/Save profile; requested source digest/length; correlation and deadline. No fake file/Resource/document ID. |
| `DeletionBasis` | Fresh `TrustedScriptCapture` whose unchanged `sr1` equals the submitted revision, plus one of `Open`, `ClosedCached`, `ClosedAbsent`. Preserve that admitted branch throughout. Bind original file/source/metadata, actual R and B identities/clean state where present, close epoch where applicable, native context and protected-document evidence. |
| `ParentBinding` | Retained project and component descriptors with existing identity, attachment and permission/ACL checks. A pathname-only missing-parent result is not a reservation. |
| `FileBinding` | Original or newly created regular-file descriptor, device/inode, source digest/length and metadata, with positive current namespace attachment. Identity values follow existing canonical decimal conventions. |

The three delete branches are actual states, not independent booleans. A closed-to-open or cached-to-absent change before entry invalidates the old basis. Open deletion's authorized removal of its original document is not a new closed-target admission; later verification compares the original lifetime and the phase-specific expected transition. A same-path/same-text new file, Resource or buffer is a new lifetime. No per-path history/revision store is introduced.

`CreationContext` represents supplied source for an absent root explicitly. It reuses current project/warning/ClassDB/source-profile acquisition and bound private-validator receipts, without pretending to be an observed Script or CodeEdit. Root capture distinguishes `ExistingFile` from `ProposedAbsentRoot`; dependencies, if admissible under the existing profile, still require actual confined captures. Creation preflight must recheck live root absence and context after helper completion. The post-create validation record instead binds independently captured actual file source and current applicable R/context.

## Authority observations

Each public before/after authority sample has an acquisition interval and a current/invalidated/unavailable validity classification. Native object IDs and private protected sources are not exported. A public identity is an opaque session-bound commitment using the existing tagged/length-delimited hash conventions, not a mutation handle or new revision format.

- **Disk:** `present`, `absent` or `unavailable`; present includes permitted opaque file identity and source digest/UTF-8 length. Absent requires a successful checked-parent leaf lookup. Permission failure or a missing/replaced parent is unavailable, not absent. No source text is returned by create/delete.
- **Resource:** `present`, `absent` or `unavailable`, plus applicable identity, target-path association, digest/length and independently observed `clean`, `dirty`, `not_applicable` or `unavailable` edited state. Cache absence requires both passive getters to succeed and agree. The detached original object is separately witnessed privately; an empty Resource path is not an empty script.
- **Document:** `open`, `closed` or `unavailable`, with applicable opaque document identity, buffer digest/length, current/saved-version evidence and dirty state. Closed is independently established complete target absence, not a failed CodeEdit lookup. Create requires continued closed state; delete requires removal of the original document without replacement.

Each unavailable/invalidated sample carries a bounded source-free reason. Digests/identities remain subject to current disclosure permission; previously collected evidence cannot bypass a later denial. `null` means no publishable value, never empty source or a zero identity. A sample is an interval, never an atomic global snapshot.

## Transaction transitions

Creation follows `Admitted → Prepared → Entered → Persisted → Verifying → Terminal`. Exclusive leaf acquisition is the first possible effect. Persistence covers exact descriptor writes, flush and readback; failure after acquisition leaves an actual partial/empty file with truthful effects. No rollback cleanup follows.

Deletion follows `Admitted → Prepared → RemovingDocument → DetachingResource → RemovingFile → Verifying → Terminal`. A branch may skip an inapplicable document/Resource step, never re-admit under another branch. `RemovingDocument` includes the inherited close return/settling ledger before Resource/file effects can advance. Each next stage requires intact original target, protection, namespace, session and deadline guards. Cancellation or invalidation stops new stages; entered synchronous work remains owned until return.

A terminal attempt cannot be re-entered. Native callbacks report facts and cannot authorize an effect. Parent possible-entry state becomes conservative before sending each one-shot authorization. An attributable pre-entry refusal/discard can prove no entry; a completed primitive with an attributable no-effect failure instead records that it entered but changed nothing. One state enum per owner plus monotonic factual progress replaces combinations of lifecycle booleans.

## Effects, validation and verification

Create records `file_creation` and `source_persistence`. Delete records `document_removal`, `resource_detachment` and `file_removal`. Every named effect has `not_applicable`, `not_entered`, `entered`, `completed`, `failed_no_effect` or `unknown` progress; this describes execution facts, not verified success. `failed_no_effect` requires an attributable completed primitive whose semantics prove no effect, such as exclusive `openat` returning `EEXIST`; a generic native error or missing return does not qualify. A known created file or detached Resource cannot disappear from the record because later evidence or delivery failed. Actual write length/call error may be retained privately and projected as bounded source-free diagnostics when useful.

Validation separates intended-source preflight, current/protected context and independently observed post-source validation. Each has `valid`, `invalid`, `unavailable` or `not_applicable`, with scope/reason explicit. Delete has no intended or surviving target-source parse obligation; that is `not_applicable`, not `valid`. It still requires existing read eligibility, current safety and protected-context validation. Missing required preflight/context evidence refuses before effects.

Verification records `pending`, `verified`, `failed`, `unavailable` or `not_applicable` separately for disk, Resource, document, protected state and actual-source validation. Synchronization is `complete`, `incomplete`, `unavailable` or `not_applicable` for each applicable authority. History distinguishes `not_participating` from `target_document_disposed`; neither promises delete/create Undo. Unrelated native history remains protected and must be independently exercised.

Success predicates are conjunctions, not majority votes:

- **Created:** same authenticated project/session and newly acquired file lifetime, exact intended flushed D, continued B absence, R positively absent or newly associated with exact intended source/clean supported state, successful fresh actual-source validation, unchanged required context/protected state, and no sticky invalidation.
- **Deleted:** original lifetime admitted by the matched revision, all applicable deletion effects completed, fresh D absence, no target document/B, no loaded target-path Resource association, detached retained original R when applicable, completed close-continuation/preservation evidence, and no sticky invalidation. It says nothing about physical destruction of other references or global dependency validity.

Native receipts, helper acknowledgments, cache omission and inventory omission cannot satisfy those predicates alone. Independent post-acquisition follows the mutation route; re-reading the receipt is not independent evidence.

## Deterministic terminal reduction

| Outcome | Application | Required knowledge |
| --- | --- | --- |
| `verified_created` / `verified_deleted` | `applied` | The operation's entire success predicate is satisfied within the original deadline. |
| `refused` | `not_applied` | Every applicable step is proved not entered/discarded before entry or completed with an attributable `failed_no_effect` result. No known or possibly unaccounted effect remains. |
| `applied_unverified` | `applied` or `partly_applied` | Some actual effects are known, but one or more success obligations failed or are unavailable. `partly_applied` means known partial progress; any uncertainty about remaining steps is retained per effect, not asserted absent. |
| `application_unknown` | `unknown` | Entry may have occurred but usable facts cannot establish actual application. Never downgrade to a no-effect adapter error. |

Preserve first attributable failure and later safety/effect/disclosure facts. Input rejection may be `not_applied` only before dispatch. Known invalidation defeats later coincidental equality/absence; a later read cannot retroactively verify an earlier failed request. Cancellation/timeout/disconnection/serialization loss dominates any tentative success not yet established, without erasing known effects. Disclosure denial always suppresses forbidden source-derived evidence regardless of causal failure precedence.

A bounded reason distinguishes `target_occupied`, `target_missing`, `revision_mismatch`, `identity_changed`, `lifecycle_changed`, `dirty_conflict`, `resource_conflict`, `evidence_unavailable`, routing/permission denial, unsupported target/source/context/capability, invalid/unavailable validation and interrupted/unverified effects. Keep existing finer-grained shared diagnostics where applicable. Next actions are operation-specific: correct request/source, choose an absent path, fresh read, resolve human conflict, check support/connection, or inspect uncertain effects. None authorizes automatic retry, Save, force-close or filesystem fallback.

## Visibility and persistence

All new intent/evidence/reducer/helper declarations are private or crate-private unless a present consumer requires more. Only same-binary worker entrypoints/constants require executable-facing visibility. Do not expose a public create/delete CLI or Rust extension API solely for future use. Request-local receipts, retained descriptors and signal ledgers are released safely; no database, journal, replay cache or global per-path lifecycle service is stored. Existing `.gd.uid` metadata is not deleted, and its survival must not be presented as recovery or UID-based lifetime protection.
