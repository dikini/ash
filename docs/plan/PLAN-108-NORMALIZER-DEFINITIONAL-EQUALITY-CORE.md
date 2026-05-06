# PLAN-108: Normalizer and Definitional Equality Core

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task. Phase 112 is DESIGN-034 SPEC-D. Do not implement public `type fn` syntax, source equation parsing, associated type-family computation, proposition solving, module-summary export/import of equations, holes, partial constructor application, or new public projection syntax under this plan.

**Goal:** Implement [SPEC-060](../spec/SPEC-060-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md) by adding an internal total normalizer, canonical normal forms, fixture equation tables, a structured definitional equality API, non-inverting equality diagnostics, and narrow forcing-point adoption on top of the completed Phase 109/110/111 substrates.

**Architecture:** Phase 112 is internal-first. `ash-core` owns shared normal-form/domain-constructor carriers where they cross crate or summary boundaries. `ash-typeck` owns the normalizer, fixture registry, definitional equality API, forcing points, and diagnostics. `ash-parser` has no public syntax work in this phase. `ash-engine` has no equation-export work in this phase except non-regression checks around existing semantic summaries.

**Tech Stack:** Rust 2024, `ash-core::type_ir`, `ash-core::semantic_summary`, `ash-typeck::TypeEnv`, existing canonical `Kind`, Phase 111 sealed-domain summaries, focused Rust tests, Markdown docs.

---

## Phase 112: Normalizer and Definitional Equality Core

**Status:** 🚧 In Progress
**Spec:** [SPEC-060](../spec/SPEC-060-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md)
**Design:** [DESIGN-034](../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md)
**Depends on:** [SPEC-057](../spec/SPEC-057-UNIFIED-TYPE-MODULE-PIPELINE-AND-SEMANTIC-SUMMARIES.md), [SPEC-058](../spec/SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md), [SPEC-059](../spec/SPEC-059-SEALED-TYPE-LEVEL-DOMAINS.md)

### Task table

| Task | Description | Type | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-816](tasks/TASK-816-spec-d-spec-plan-packet.md) | Promote DESIGN-034 SPEC-D into SPEC-060/PLAN-108 and register Phase 112 | Docs/Planning | 4 | ✅ Complete |
| [TASK-817](tasks/TASK-817-normalizer-defeq-audit-gate.md) | Audit live canonicalization/equality/forcing seams before implementation | Docs/Substrate | 4 | ✅ Complete |
| [TASK-818](tasks/TASK-818-core-normal-form-and-domain-constructor-carriers.md) | Add core normal-form and sealed-domain constructor normal-form carriers | Core/Substrate | 5 | ✅ Complete |
| [TASK-819](tasks/TASK-819-typeck-normalizer-api-skeleton.md) | Add `ash-typeck` normalizer module, options, outcome, and identity behavior | Type/Substrate | 5 | ✅ Complete |
| [TASK-820](tasks/TASK-820-internal-fixture-equation-registry.md) | Add internal fixture equation registry for test-only/internal-test computation heads | Type/Test Substrate | 5 | ✅ Complete |
| [TASK-821](tasks/TASK-821-closed-computation-head-reduction.md) | Implement closed fixture reduction to domain-constructor normal forms | Type/Semantic | 6 | ✅ Complete |
| [TASK-822](tasks/TASK-822-open-neutral-and-partial-normalization.md) | Implement open neutral/stuck forms and partial prefix normalization | Type/Semantic | 6 | ✅ Complete |
| [TASK-823](tasks/TASK-823-rigid-projection-and-alias-normalization.md) | Normalize aliases plus neutral/rigid projection argument spines without associated-family computation | Type/Semantic | 5 | 📝 Planned |
| [TASK-824](tasks/TASK-824-definitional-equality-api.md) | Add structured normalize-and-compare definitional equality API | Type/Semantic | 6 | ✅ Complete |
| [TASK-825](tasks/TASK-825-non-inverting-unification-boundary.md) | Enforce non-inversion/no-solving-under-neutral computation heads | Type/Semantic | 5 | 📝 Planned |
| [TASK-826](tasks/TASK-826-typeenv-forcing-point-rollout.md) | Adopt definitional equality at named `TypeEnv` forcing points only | Type/Integration | 7 | 📝 Planned |
| [TASK-827](tasks/TASK-827-normalizer-diagnostics-and-non-interference.md) | Add diagnostics, negative tests, and Phase 109/110/111 non-interference coverage | Diagnostics/Tests | 6 | 📝 Planned |
| [TASK-828](tasks/TASK-828-spec-d-closeout-docs-and-verification.md) | Reconcile docs/status/changelog and record closeout verification evidence | Docs/Planning | 4 | 📝 Planned |
| [TASK-829](tasks/TASK-829-phase112-review-remediation.md) | Remediate post-closeout review findings for Phase 112 | Review/Hardening | 6 | 📝 Planned |

Estimated total: 74 hours.

## Tracks

### Track A: Spec Gate and Audit

- TASK-816 creates the normative SPEC-D packet.
- TASK-817 audits the live `TypeEnv` equality/canonicalization/forcing seams, produces an exact forcing-point matrix, and freezes exact file targets before Rust implementation begins. Completed audit: [TASK-817 normalizer/defeq audit](audits/TASK-817-normalizer-defeq-audit.md).

### Track B: Normal-Form and Normalizer Substrate

- TASK-818 adds shared normal-form/domain-constructor carriers.
- TASK-819 adds the normalizer module and identity behavior.
- TASK-820 adds internal fixture equation tables used by tests only.

### Track C: Reduction Semantics

- TASK-821 implements closed reduction.
- TASK-822 implements open neutral/stuck and partial normalization.
- TASK-823 integrates aliases plus neutral and rigid projections without associated-family computation.

### Track D: Equality and Forcing Points

- TASK-824 adds structured definitional equality.
- TASK-825 preserves the non-inverting unification boundary.
- TASK-826 adopts the new API only at named `TypeEnv` forcing points.

### Track E: Diagnostics and Closeout

- TASK-827 adds diagnostic/non-interference coverage.
- TASK-828 reconciles status surfaces and verification evidence.
- TASK-829 closes the post-review remediation slice.

## Execution Order

1. TASK-816 first.
2. TASK-817 second; no Rust implementation begins before the audit gate lands.
3. TASK-818 must precede normalizer work because closed fixture reduction needs a domain-constructor normal-form carrier.
4. TASK-819 depends on TASK-818.
5. TASK-820 depends on TASK-819 and Phase 111 domain lookup APIs; it must remain test-fixture/internal-test setup only.
6. TASK-821 depends on TASK-820.
7. TASK-822 depends on TASK-821.
8. TASK-823 depends on TASK-822 and must cover both `ProjectionRigidity::Neutral` and `ProjectionRigidity::Rigid`.
9. TASK-824 depends on TASK-823.
10. TASK-825 depends on TASK-824.
11. TASK-826 depends on TASK-825 and the forcing-point matrix produced by TASK-817.
12. TASK-827 depends on TASK-826 and cites reduction/equality evidence from TASK-821 through TASK-825.
13. TASK-828 depends on TASK-827.
14. TASK-829 depends on independent review after TASK-828.

## Implementation Constraints

1. Phase 112 uses internal fixture equations only; no source `type fn` parser/lowering lands here.
2. Marker constructors remain separate from ordinary constructors and must be represented by Phase 111 domain constructor identities.
3. Neutral computation heads are not decomposed for unification and are never inverted to solve inputs from outputs.
4. Ordinary same-headed nominal constructor unification remains compatible with current `TypeEnv::unify_types` behavior.
5. Rigid projections may normalize argument spines but must not perform associated-family computation or impl-search recursion.
6. Cycle/fuel guards are implementation robustness diagnostics, not the semantic termination story for accepted future type functions.
7. TASK-817 is the single owner of the live forcing-point audit and exact callsite matrix for Phase 112.
8. TASK-826 is the only task allowed to adopt the new definitional equality API into live `TypeEnv` forcing points.
9. No task may add module-summary export/import for type-function equations; that belongs to SPEC-F.

## Verification Strategy

Every implementation task must include focused tests and exact non-regression commands. Phase-level closeout must verify:

1. closed fixture reduction normalizes to sealed-domain constructor normal forms;
2. open fixture reduction produces canonical neutral/stuck normal forms;
3. partial open reduction preserves reduced prefixes and neutral tails;
4. aliases and neutral/rigid projection argument spines normalize structurally;
5. definitional equality succeeds after normalization;
6. neutrality-blocked equality produces structured non-inverting evidence/diagnostics;
7. ordinary constructor unification still decomposes nominal constructors;
8. no solving occurs underneath neutral computation heads;
9. forcing-point adoption is limited to named `TypeEnv` seams;
10. Phase 109/110/111 semantic-summary, canonical projection, and sealed-domain behavior remains non-regressed;
11. docs/spec index, PLAN-INDEX, task statuses, and CHANGELOG are reconciled honestly.

## Decision Gates

- D1: Phase 112 is internal normalizer/equality work only; public `type fn` syntax and source equation validation remain SPEC-E.
- D2: Closed reduction uses internal fixtures and Phase 111 domain constructor IDs, not ordinary ADT constructors.
- D3: Definitional equality is normalize-and-compare, not proof search and not type-function inversion.
- D4: Rigid associated projections stay rigid unless current simple SPEC-035 substitution already selects an associated output; recursive associated-family normalization is deferred.
- D5: Wider forcing-point rollout requires a future task/spec after this phase proves the core API.

## Completion Checklist

- [x] SPEC-060 is registered in `docs/spec/README.md` as Draft.
- [x] PLAN-108 and TASK-816 through TASK-829 are registered in `docs/plan/PLAN-INDEX.md`.
- [x] `ash-core` exposes normal-form/domain-constructor carriers needed by `ash-typeck`.
- [x] `ash-typeck` exposes a normalizer API with weak-head/full/demand-aware options.
- [ ] Internal fixture equation tables support closed/open Append-style tests.
- [ ] Closed and partial-open normalization tests pass.
- [ ] Definitional equality returns structured equality/mismatch/neutral-blocked outcomes.
- [ ] Current ordinary constructor unification remains non-regressed.
- [ ] Docs/status/changelog are reconciled and review findings are closed via TASK-829.
