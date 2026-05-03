# PLAN-106: Type-Expression IR, Projection Identities, and Kind/Arity Substrate

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task. Phase 110 is SPEC-B from DESIGN-034. Do not implement sealed domains, normalization, public `type fn`, computation-summary export/import, recursive associated type-family computation, propositions, holes, partial type-constructor application, or new public projection spellings under this plan.

**Goal:** Implement [SPEC-058](../spec/SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md) by introducing a shared core-owned `Kind`, a canonical internal type-expression IR, promoted projection identities, rigid/neutral carriers, and explicit kind/arity validation on top of the completed Phase 109 substrate.

**Architecture:** Phase 110 is an internal type-substrate phase. `ash-core` owns the canonical computation-capable type-expression IR, the single shared `Kind` type, and promoted identity carriers. `ash-parser` remains surface-only, but Phase 110 parser work includes both `parse_type_def.rs` and `parse_module.rs` wherever ordinary type syntax must stay aligned on supported associated projections or explicit rejections. `ash-typeck` elaborates current surface/core type syntax into canonical IR, validates kind/arity, canonicalizes projections and transparent aliases, and also performs the source/import identity plumbing needed to resolve canonical projections against the same interface/member IDs in local and imported code. `ash-engine` remains out of scope: computation-grade summary export/import belongs to a later packet.

**Tech Stack:** Rust 2024, `ash-core`, `ash-parser`, `ash-typeck`, current `Kind` substrate, Phase 109 semantic-summary identities, existing associated-type surface syntax, focused Rust tests.

---

## Phase 110: Type-Expression IR, Projection Identities, and Kind/Arity Substrate

**Status:** ✅ Complete
**Spec:** [SPEC-058](../spec/SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md)
**Design:** [DESIGN-034](../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md)
**Syntax grounding note:** Do not use historical syntax-reduction notes as implementation authority for Phase 110. If later work in this area reaches user-facing syntax, validate it against the live parser/surface contracts first.
**Depends on:** [SPEC-003](../spec/SPEC-003-TYPE-SYSTEM.md), [SPEC-034](../spec/SPEC-034-WHERE-BOUNDED-GENERIC-INTERFACE-IMPLEMENTATIONS.md), [SPEC-035](../spec/SPEC-035-ASSOCIATED-TYPES.md), [SPEC-057](../spec/SPEC-057-UNIFIED-TYPE-MODULE-PIPELINE-AND-SEMANTIC-SUMMARIES.md)

### Task table

| Task | Description | Type | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-793](tasks/TASK-793-spec-b-spec-plan-packet.md) | Promote DESIGN-034 SPEC-B into SPEC-058/PLAN-106 and register Phase 110 | Docs/Planning | 4 | ✅ Complete |
| [TASK-794](tasks/TASK-794-type-expression-ir-and-kinding-audit-gate.md) | Audit live type-expression, projection, alias, and kind/arity substrate and freeze the Phase 110 gate | Docs/Substrate | 4 | ✅ Complete |
| [TASK-795](tasks/TASK-795-core-type-computation-identity-carriers.md) | Promote/add ash-core identity carriers and re-home shared `Kind` ownership for computation-grade type IR and projections | Core/Substrate | 6 | 📝 Planned |
| [TASK-796](tasks/TASK-796-core-canonical-type-expression-ir-and-neutral-carriers.md) | Add ash-core canonical type-expression IR plus rigid/neutral carriers | Core/Substrate | 6 | 📝 Planned |
| [TASK-797](tasks/TASK-797-ordinary-type-parser-expression-parity-and-explicit-rejections.md) | Align `parse_type_def.rs` and `parse_module.rs` ordinary type parsing with the Phase 110 parity/rejection boundary | Parser/Substrate | 5 | ✅ Complete |
| [TASK-798](tasks/TASK-798-canonical-type-ir-lowering-from-surface-and-core.md) | Lower current surface/core type syntax into canonical IR and make `TypeEnv` own interface/member identity registries, storage, and source/import registration | Type/Substrate | 7 | 📝 Planned |
| [TASK-799](tasks/TASK-799-kind-and-arity-validation-hardening.md) | Harden kind/arity validation for nominal, projection, and future computation heads | Type/Substrate | 5 | 📝 Planned |
| [TASK-800](tasks/TASK-800-associated-projection-canonicalization-and-rigid-plumbing.md) | Replace every live stringly/sentinel associated-projection surface with canonical identity-backed rigid projection plumbing and own projection-specific diagnostics | Type/Substrate | 7 | 📝 Planned |
| [TASK-801](tasks/TASK-801-transparent-alias-canonicalization-helper.md) | Add transparent-alias canonicalization helpers and readable diagnostic rendering rules | Type/Substrate | 5 | 📝 Planned |
| [TASK-802](tasks/TASK-802-canonicalization-boundary-adoption-for-current-equality-sites.md) | Adopt canonicalization at `TypeEnv::unify_types` / `TypeEnv::types_equivalent_for_equality` via `TypeEnv::canonicalize_type_for_equality`; no pattern/exhaustiveness rollout | Type/Compatibility | 5 | 📝 Planned |
| [TASK-803](tasks/TASK-803-spec-b-diagnostics-negative-tests-and-non-interference.md) | Add diagnostics, negative tests, and non-interference coverage for SPEC-B substrate | Diagnostics/Tests | 6 | ✅ Complete |
| [TASK-804](tasks/TASK-804-spec-b-closeout-docs-and-verification.md) | Reconcile docs/status/changelog and run Phase 110 closeout verification | Docs/Planning | 4 | ✅ Complete |
| [TASK-805](tasks/TASK-805-phase110-review-remediation.md) | Remediate post-closeout review findings for Phase 110 | Review/Hardening | 6 | 📝 Planned |

Estimated total: 70 hours.

## Tracks

### Track A: Spec Gate and Audit

- TASK-793 creates the normative SPEC-B packet.
- TASK-794 audits the current parser/core/typechecker substrate before implementation begins, records contradictions against live specs, and freezes the implementation gate.

### Track B: ash-core Canonical IR Substrate

- TASK-795 promotes/adds the canonical identity carriers needed for computation-grade IR and re-homes the shared `Kind` definition into `ash-core`.
- TASK-796 adds the canonical type-expression IR and rigid/neutral carriers consumed by later packets.

### Track C: Parser + Typechecker Lowering Boundary

- TASK-797 aligns `parse_type_def.rs` and `parse_module.rs` so both parser paths either support the current Phase 110 subset or reject deferred syntax explicitly.
- TASK-798 introduces the main lowering boundary from surface/core type syntax into canonical IR, defines `TypeEnv` interface/member identity registries and storage, and registers those identities from both source lowering and imported ordinary summaries. It stops before replacing existing stringly/sentinel projection consumers or projection-specific diagnostics.
- TASK-799 hardens kind/arity checking over that canonicalized lowering path.

### Track D: Projection and Alias Canonicalization

- TASK-800 consumes the registries landed by TASK-798 and replaces all live stringly/sentinel projection handling surfaces in `ash-typeck` with identity-backed rigid projection elaboration, including projection carriers, unresolved-state handling, and projection-specific diagnostics.
- TASK-801 adds transparent-alias canonicalization helpers while preserving readable diagnostics.
- TASK-802 adopts TASK-800/TASK-801 outputs only at the named current equality boundaries `TypeEnv::unify_types` and `TypeEnv::types_equivalent_for_equality`, both routed through `TypeEnv::canonicalize_type_for_equality`. `check_pattern.rs` and `exhaustiveness.rs` remain out of scope for Phase 110 because their live implementations do not consume `TypeEnv` canonicalization state.

### Track E: Diagnostics and Closeout

- TASK-803 adds negative diagnostics, regression tests, and non-interference coverage.
- TASK-804 reconciles docs, status surfaces, and verification evidence.
- TASK-805 reserves the post-review hardening slice.

## Execution Order

Phase 110 is mostly sequential with limited parallelism:

1. TASK-793 first.
2. TASK-794 second; no Rust implementation begins before the audit gate lands.
3. TASK-795 must land before any canonical IR work because it establishes both the promoted identity carriers and the core-owned shared `Kind`. TASK-797 may run after TASK-794 in parallel with late TASK-795 review/fix work.
4. TASK-796 depends on TASK-795.
5. TASK-798 depends on TASK-796 and TASK-797 and must land canonical lowering entry points plus `TypeEnv` identity registries/storage/registration for source-local and imported interface/member summaries. It does not replace the live stringly/sentinel projection representation or projection diagnostics.
6. TASK-799 depends on TASK-798.
7. TASK-800 depends on TASK-795 through TASK-799 and is the first task allowed to replace live stringly/sentinel projection representations and projection-specific diagnostics with canonical identity-backed plumbing.
8. TASK-801 depends on TASK-798.
9. TASK-802 depends on TASK-800 and TASK-801.
10. TASK-803 depends on TASK-799 through TASK-802.
11. TASK-804 depends on TASK-803.
12. TASK-805 depends on independent review after TASK-804.

## Implementation Constraints

1. `ash-core` owns the shared `Kind`, canonical IR, and promoted identity carriers.
2. `ash-parser` remains surface-only; parser parity work for this phase must cover both `parse_type_def.rs` and `parse_module.rs` without adding speculative public syntax.
3. `ash-typeck` owns elaboration, canonicalization, kind/arity validation, and all `TypeEnv` interface/member identity registry/storage/registration work; TASK-798 lands that substrate, while TASK-800 alone may replace live stringly/sentinel projection consumers and projection diagnostics.
4. No task may reinterpret Phase 109 semantic-summary transport as computation-summary export/import.
5. `base::Assoc` remains the only normative public projection spelling in this phase.
6. No task may add sealed domains, public `type fn`, normalization, recursive associated-family computation, propositions, or proof search.
7. No task may add public kind binder syntax, holes, or partial type-constructor application.
8. Current SPEC-035 simple associated-type substitution must keep working for the already-supported subset.
9. Existing ADT/interface/workflow/capability/resource/do/comprehension behavior must remain unaffected.
10. TASK-797 is the single owner of parser rejection-boundary evidence for Phase 110; later tasks may rerun or cite that suite but must not create a second parser-evidence owner.

## Verification Strategy

Every implementation task must include focused tests for the changed crate and explicit non-regression coverage. The phase-level closeout must verify:

1. canonical type IR exists in `ash-core` and distinguishes nominal heads from computation heads;
2. canonical projection identities replace string interface matching in the active typechecker paths, including ordered argument spines for both unary `S::Assoc` and multi-parameter `Map<K, V>::Entry` projections;
3. wrong kind/arity is rejected before later computation packets could consume the type expression;
4. unsupported projection shapes admitted by the current `base::Assoc` syntax — for example `(S::Item)::Assoc` and `Map<K, V>::Entry::Assoc` — fail with a dedicated unsupported-shape diagnostic rather than ambiguity fallback, placeholder state, or silent acceptance;
5. Phase 110 introduces rigid/neutral carriers only; no current equality or pattern boundary may claim comparison, decomposition, or solving under neutral computation heads;
6. transparent aliases and canonical rigid projections are consumed at `TypeEnv::unify_types` / `TypeEnv::types_equivalent_for_equality` without losing readable diagnostics;
7. the current simple associated-type compatibility path still works;
8. no new projection syntax, `type fn`, holes, or partial type-constructor application is accepted silently;
9. Phase 109 ordinary-type summary/import/export behavior remains intact;
10. docs/spec index, PLAN-INDEX, task statuses, and CHANGELOG are reconciled, and TASK-804 records the exact focused/broad verification commands plus any residual-failure classification.

## Decision Gates

- D1: SPEC-B is an internal IR and validation packet, not a public `type fn` or normalization packet.
- D2: `base::Assoc` remains the only normative public projection spelling in Phase 110.
- D3: `ash-core` owns the single shared `Kind`, canonical type-expression IR, and promoted identity carriers; `ash-typeck` consumes them.
- D4: kind/arity validation is explicit and early, but public kind binder syntax, holes, and partial type-constructor application remain deferred.
- D5: current SPEC-035 simple associated-type substitution is preserved only as a compatibility path; it is not the future general normalizer.
- D6: computation-summary export/import remains deferred to a later packet.
- D7: Before TASK-800, Phase 110 must already have (a) core-owned `Kind`, (b) aligned ordinary-type parser targets in `parse_type_def.rs` and `parse_module.rs`, and (c) source/import plumbing for interface/member identities.

## Completion Checklist

- [ ] SPEC-058 is registered in `docs/spec/README.md`.
- [ ] PLAN-106 and TASK-793 through TASK-805 are registered in `PLAN-INDEX.md`.
- [ ] The shared core-owned `Kind` exists in `ash-core` and is re-used by the canonical IR.
- [ ] Canonical computation-capable type-expression IR exists in `ash-core`.
- [ ] `TypeEnv` owns interface/member identity registries, storage, and source/import registration before projection replacement begins.
- [ ] Canonical projection identities replace stringly interface-name matching in the active typechecker path.
- [ ] Kind/arity validation is explicit for nominal constructors, projections, and future computation-head placeholders.
- [ ] Transparent alias canonicalization helpers exist and are adopted at the named Phase 110 comparison boundaries.
- [ ] Current simple associated-type behavior remains intact for the already-supported subset.
- [ ] Existing ADT/interface/workflow/capability/resource/do/comprehension regressions still pass.
- [ ] Deferred syntax/features remain rejected or explicitly unsupported.
- [x] Docs/status/changelog are reconciled, TASK-804 contains exact verification evidence and residual-failure classification, and independent review findings are closed via TASK-805 before the phase is marked fully complete.
