# Research: Exact Partial Editing for Project GDScripts

**Date:** 2026-10-06 | **Source baseline:** `b0f0a4cdcaa4cd6bb1e3d158b5cea1870f001dd6`

**Disposition:** R1–R3 from the initial plan are resolved. Three independent read-only investigations covered trusted execution, public migration and acceptance tooling. Integration review checked the relevant source and Rust LSP references. This is a design decision record, not Schema 2 implementation, real-client acceptance or a new support claim.

## R1 — Derive intent below MCP, before existing execution

**Decision:** Extend the existing protocol-independent revision-based request with a crate-private exact-text constructor and a private source-intent enum. Keep the existing public whole-source constructor for its actual local consumers. The MCP decoder uses only the exact constructor. Both representations share the current fresh-acquisition/revision gate and the same full-source dispatch; only exact intent needs transformation.

**Evidence:** [runner/read.rs](../../mcp-server/src/runner/read.rs), lines 140–209 at the baseline, already reacquires current state, refuses unavailable eligibility/cancellation/timeout/revision mismatch, then constructs the open or closed request from that frozen capture. [script_read.rs](../../mcp-server/src/script_read.rs), lines 196–218, defines `ScriptEditRequest`; lines 448–469 construct the private whole-source open input without another live capture. The LSP reference query returned 15 references, including the MCP handler, [local workflow fixture](../../mcp-server/examples/script_workflow_fixture.rs) and read/closed core tests. These are genuine independent consumers, not a reason to retain Schema 1 MCP input.

**Rationale:** Matching an adapter-side read, acquiring again after matching, or accepting a caller's source snapshot creates stale-intent or split-capture hazards. Deriving from the one current admitted source makes both the basis and complete intended source refer to the same state. Existing open and closed paths already perform complete-source admission, stock validation, immediate guards, persistence and independent verification. No native capability, private wire, source writer, revision algorithm or Godot integration change is required by this design.

**Alternatives considered:** Replace the public core constructor and migrate unrelated local whole-source interfaces; duplicate acquisition/dispatch in a new runner; perform replacement in MCP; implement partial native writes; introduce a generic transformation trait. All add compatibility cost, duplicate safety policy or unnecessary mutation authority. The private enum has exactly two current consumers and is not a public mode switch or extension framework.

### Preserve safety precedence for rejected transformations

The initial capture is not the entire edit preflight. In particular,
`ExpectedRevisionBasis::from_observation` checks observed dirty/coherence/version
facts, while [bridge preparation](../../godot-addon/addons/godot_agent_kit/bridge.gd)
lines 1225–1282 additionally checks saved-version state, `Resource.edited`,
effective context and Save preservation. Returning a match error immediately
after read would bypass these existing reasons; equal-text dirty R is a concrete
case, not a hypothetical isolation requirement.

**Decision:** Keep a derivation failure private until the existing **unchanged**
full-source path has checked the same frozen basis/current source and returned
independently verified unchanged with proven zero effects. Only then reduce that
result to the retained matching refusal. Any safety, validation, race, timeout,
disclosure or uncertain-effect result wins unchanged. Successful derivation runs
the ordinary intended-source path once; it does not run an extra no-op first.
This is a no-effect eligibility/verification use of an existing path, never a
repair, a second attempted edit or successful no-match recognition.

**Rationale and cost:** This preserves FR-009 precedence without cloning the
editor's saved-state/context checks into Rust or adding a private preflight RPC.
Only rejected derivations incur the existing unchanged validation/verification
work; it remains inside the original ten-second bound. No new native authority
or protocol is introduced. Native effect-only admission remains on actual
changed-source execution; the diagnostic check must satisfy all applicable
unchanged admission and postcondition checks. Tests must combine absent,
ambiguous and empty-search failures with equal-text dirty R, stale state and
unavailable evidence, and exercise a guard change during the diagnostic check.

**Alternatives considered:** Immediate match refusal loses existing safety
precedence; duplicating saved/context checks creates a second policy; a new
probe API and private cutover add maintenance without another guarantee.
Retaining a candidate error cannot override any result other than the proven
zero-effect `verified_unchanged` result.

### Literal search and allocation

**Decision:** For nonempty old text, use at most two standard-library `str::find` substring searches. Find the first start; search again from the next UTF-8 character boundary after that start, **not** from the end of the first match. A second hit is enough to establish ambiguity. Empty old text is handled separately by testing complete source emptiness.

The installed Rust 1.98.1 `rust-src` implementation (`core/str/pattern.rs`, lines 976–978, 1082–1111, 1284 onward) uses the constant-space Two-Way substring searcher. The [standard search source](https://doc.rust-lang.org/src/core/str/pattern.rs.html) and [stable `str::find` API](https://doc.rust-lang.org/std/primitive.str.html#method.find) establish the selected mechanism. Two searches keep work linear in bounded source/needle length without a prefix table or new dependency. This is source-established design, not a measured latency pass.

The next-character-boundary rule counts `aa` twice in `aaa` and `éé` twice in `ééé`; valid UTF-8 literal occurrences cannot begin on continuation bytes. Do not use `match_indices`/`matches` counting, which omits overlaps, advance by needle length, rescan inserted text, or search at every byte offset. There is no case folding, normalization, token/grapheme interpretation or fuzzy fallback.

Before constructing the full result, checked arithmetic verifies `current_bytes - old_bytes + new_bytes <= 512 KiB`. Build prefix/new/suffix once at the exact capacity; reuse owned new text for a complete-source replacement where possible. No vector of matches, source clone for searching, proportional search table or duplicate complete-source serialization is needed. Existing unavoidable whole-source worker ownership is retained. Unchanged intent still goes through existing validation/verification, never an early success shortcut.

**Alternatives considered:** A custom KMP matcher is bounded but adds an avoidable needle-sized allocation and algorithm maintenance. Naive sliding comparisons can become quadratic. Regex, patch parsers and extra search crates add capabilities or dependencies with no current need.

## R2 — One Schema 2 surface and source-free refusals

**Decision:** Change only the public tool-surface version to `2`, across all three `outputSchema` declarations and every success/error envelope. Edit requires `project_root`, `script_path`, `revision`, `old_string`, `new_string`, with optional `session_id`; reject legacy, mixed and unknown forms. Preserve MCP 2025-11-25, opaque `sr1` revisions, local-operation v1, private bridge v6, native revision 4 and Cargo package version 0.1.0. Schema 2 is observable through ordinary `tools/list` and result envelopes; no extra version tool, negotiation bit or release is needed.

**Evidence:** [schema.rs](../../mcp-server/src/mcp/schema.rs) owns the three static input schemas; [handler.rs](../../mcp-server/src/mcp/handler.rs) extracts checked arguments and rejects residual fields. [output.rs](../../mcp-server/src/mcp/output.rs), baseline lines 240–244, 290 and 342–344, owns success/failure envelope versions and their schema constant. [service.rs](../../mcp-server/src/mcp/service.rs) owns the independent protocol/package declarations. Strict duplicate/depth/framing checks already belong to [framing.rs](../../mcp-server/src/mcp/framing.rs), not the argument map.

**Decision:** Exact derivation returns typed core refusals mapped to `no_match`, `ambiguous_match`, `empty_old_string` or `invalid_source`, at `stage: matching`. After the precedence check above, reduce the checked unchanged result to `outcome: refused`, `application: not_applied`, preserving its actual mode, validation/evidence, lifecycle and nonparticipating history. Closed history remains `not_applicable_closed`. Open `safe_next_action` and closed `next_action.kind` both identify `fresh_read`; early acquisition refusals retain the existing `undetermined` shape. Concrete guidance appears only with the relevant failure; no candidate text, offsets, counts or fragments are returned.

Malformed syntax, wrong/missing/null/unknown fields, malformed revisions and unsupported fragment representation/bounds remain source-free adapter `invalid_arguments` errors. A structurally valid empty old string cannot be rejected at decode time: its validity depends on the fresh complete source. Combined-result bounds are checked after successful matching and produce `invalid_source`; downstream full-script validation keeps its existing detailed outcome.

**Rationale:** [edit refusal DTOs](../../mcp-server/src/mcp/output/edit.rs) already carry reason/stage/effects/actions without a new result variant, and [projection](../../mcp-server/src/mcp/output/edit/projection.rs) suppresses revisions when disclosure was denied. Fresh safety/revision failures must win over all match diagnoses; after possible effects, no match reason may replace actual uncertainty. Keep the authoritative structured result and terse text summary, not another duplicated carrier or static failure lecture.

**Alternatives considered:** Adapter invalid-arguments for a legitimate no-match; extending global native reason/wire enums; a fourth tool; `oneOf` legacy/new catalog; a schema-version request flag; source-bearing suggestions; response-wide version renumbering. None is required. Nested discovery/local records retain their own existing versions; the root tool-surface declaration is the compatibility boundary.

### Migration inventory

Implementation must distinguish current MCP consumers from retained local whole-source contracts:

- Migrate `mcp-server/src/mcp/{schema,handler,output}.rs`, directly affected output projections/guidance, and process tests `mcp-server/tests/mcp_tools.rs` / `mcp_transport.rs`.
- Migrate actual MCP calls, prompts and outcome checks in `godot-addon/tests/mcp_peer.py`, `mcp_lifecycle_acceptance.py`, `mcp_failure_acceptance.py`, `mcp_composed_acceptance.py`, `mcp_preservation_acceptance.py`, `mcp_interruption_acceptance.py`, `mcp_validation_acceptance.py` and their affected tests/fixture consumers. Follow their call paths rather than globally replacing text.
- Update current operating guidance in `.github/LOCAL_VM.md` and any current MCP examples that advertise the old shape. The new [public contract](contracts/mcp-interface.md#migration) is the migration reference.
- Do **not** convert `edit-gdscript`, `caller_edit_acceptance.py`, private `bridge/wire/edit.rs`, native/worker full-source payloads, or the direct-runner `script_workflow_fixture.rs` merely because they contain `replacement_source`. Keep their current whole-source semantics and tests. `closed_script_acceptance.py` includes reusable direct-core machinery; change only portions actually serving MCP acceptance.
- Leave Feature 007 artifacts and completed phase-exit evidence as explicitly Schema 1 history. Their unchanged semantics may inform inherited guarantees, never prove the new public representation.

## R3 — Reuse the VM and prove the changed agent path

**Decision:** Extend existing MCP fixture profiles and their independent witnesses; do not create a new runner or CI boundary. Both selected clients must perform localized replacement, deletion and complete-empty-source replacement in each open/cached-R-closed/absent-R-closed profile. At least one real-agent composed workflow additionally establishes A–E, prior native history, refusal recovery and durability through Schema 2. A direct protocol driver establishes deterministic/adversarial edges, not model-visible use.

**Evidence:** [run_in_vm.py](../../godot-addon/tests/run_in_vm.py) already exposes `prepare-mcp`, `mcp-stdio`, `prepare-durability` and `finalize-mcp`. [Feature 007's actual-client guide](../007-mcp-script-workflow/quickstart.md#real-coding-agent-clients) records the fixed guest relay, scoped OMP project config and interactive Codex approval. [Phase 3 support](../../PHASE_3_EXIT.md#scope-and-support-profile) fixes the engine, guest, four-vCPU/6-GiB profile and clients. OMP's current global executable is not assumed to be the accepted 18.5.1; use and record the exact selected client installation.

**Rationale:** The changed request representation and match semantics invalidate whole-source client proof, not every historical native guarantee. The [acceptance coverage](quickstart.md#acceptance-coverage) maps all 25 scenarios, 18 FRs and eight SCs to new or affected evidence and inherited evidence review. Native code/ABI/authentication/exports are not changed by this plan; rebuild/retest them only for relevant changed inputs or demonstrable inability to reuse accepted provenance under [TEST_POLICY.md](../../TEST_POLICY.md).

**Alternatives considered:** SDK-only calls, fabricated model transcripts, full historical GUI replay, new scenario orchestration services, host GUI fallback, relaxed client permissions, unconditional rerun on a documentation commit. These either fail the specified proof or add cost without another guarantee. At least one composed client run is necessary; duplicating the entire composed campaign in both clients is not an additional blanket requirement.

## Dependencies, support and remaining obligations

**Decision:** Retain the exact current Cargo graph and features: serde 1.0.229, serde_json 1.0.151, cap-std 4.0.3, ring 0.17.14, rmcp 3.5.0 and Tokio 1.53.1. The manifest and lockfile agree. No new package, runtime, parser, license surface, approval mechanism or operational dependency is selected.

**Rationale:** The existing standard library and checked core types suffice. [Feature 007 dependency provenance](../007-mcp-script-workflow/research.md#dependencies-and-provenance) and its accepted completion remain historical evidence; this planning invocation did not rerun an advisory audit or claim absence of newly published advisories. Existing local-user threat, source-only admission, disclosure, export and active-game hot-reload limits remain intact.

All planning unknowns are resolved. Implementation must still prove the selected behavior, including complete empty-source admission on all three profiles, original-clock timing, effect/disclosure precedence and actual-client results. No Godot, native build, product test or client acceptance was executed during this research. The planning validation recorded in the quickstart covers documents only.
