# PLAN-109: Direct Structural Type Functions

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task. Phase 113 is DESIGN-034 SPEC-E. Do not implement public type-function summary export/import, associated recursive type families, proposition solving, holes, partial constructor application, mutual recursion, or generalized type lambdas under this plan.

**Goal:** Implement [SPEC-061](../spec/SPEC-061-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md) by exposing checked module-local `type fn` declarations over sealed domains, adding source-equation result carriers for marker-constructor RHSs, rejecting public/export leakage until SPEC-F, and integrating total source equations with the SPEC-060 normalizer.

**Architecture:** Phase 113 is source-to-normalizer work. `ash-parser` owns raw `type fn` surface syntax and spans, including rejected visibility prefixes. `ash-core` owns shared type-function/equation carriers, including a result-expression carrier or canonical extension for sealed-domain marker constructors. `ash-typeck` owns source type-expression resolution, provisional local-head registration, kind/domain checking, finite nested residual pattern-matrix coverage/overlap, source-order dependency validation, structural recursion validation, normalizer integration, and diagnostics. `ash-engine` owns module integration/non-interference only; public equation export/import remains SPEC-F.

**Tech Stack:** Rust 2024, `ash-parser::surface`, `ash-core::type_ir`, `ash-core::semantic_summary`, `ash-typeck::TypeEnv`, `ash-typeck::normalizer`, Phase 111 sealed-domain summaries, Phase 112 definitional equality, focused Rust tests, Markdown docs.

---

## Phase 113: Direct Structural Type Functions

**Status:** 📋 Planned
**Spec:** [SPEC-061](../spec/SPEC-061-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md)
**Design:** [DESIGN-034](../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md)
**Depends on:** [SPEC-057](../spec/SPEC-057-UNIFIED-TYPE-MODULE-PIPELINE-AND-SEMANTIC-SUMMARIES.md), [SPEC-058](../spec/SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md), [SPEC-059](../spec/SPEC-059-SEALED-TYPE-LEVEL-DOMAINS.md), [SPEC-060](../spec/SPEC-060-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md)

### Task table

| Task | Description | Type | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-830](tasks/TASK-830-spec-e-spec-plan-packet.md) | Promote DESIGN-034 SPEC-E into SPEC-061/PLAN-109 and register Phase 113 | Docs/Planning | 4 | ✅ Complete |
| [TASK-831](tasks/TASK-831-type-function-audit-gate.md) | Audit live parser/core/typeck/normalizer seams before implementation | Docs/Substrate | 5 | ✅ Complete |
| [TASK-832](tasks/TASK-832-parser-surface-for-type-functions.md) | Add parser surface syntax and raw AST carriers for `type fn` | Parser/Substrate | 6 | ✅ Complete |
| [TASK-833](tasks/TASK-833-core-type-function-equation-carriers.md) | Add core type-function/equation/pattern/result-expression carriers | Core/Substrate | 5 | ✅ Complete |
| [TASK-834](tasks/TASK-834-type-function-lowering-and-registration.md) | Lower source declarations and register module-local type-function heads | Type/Substrate | 6 | ✅ Complete |
| [TASK-835](tasks/TASK-835-type-function-signature-kind-domain-validation.md) | Validate signatures, source resolution, domains, result constraints, and public boundary | Type/Semantic | 6 | ✅ Complete |
| [TASK-836](tasks/TASK-836-type-function-pattern-coverage-overlap.md) | Implement pattern linearity plus finite residual coverage/overlap/default semantics | Type/Semantic | 7 | ✅ Complete |
| [TASK-837](tasks/TASK-837-type-function-structural-recursion.md) | Implement declared decreasing-parameter and structural recursion validation | Type/Semantic | 6 | ✅ Complete |
| [TASK-838](tasks/TASK-838-source-equations-normalizer-integration.md) | Register checked source equations with normalizer reduction/equality | Type/Integration | 7 | ✅ Complete |
| [TASK-839](tasks/TASK-839-engine-module-boundary-and-non-interference.md) | Enforce module-local boundary and engine/import non-interference | Engine/Integration | 5 | 📋 Planned |
| [TASK-840](tasks/TASK-840-type-function-diagnostics-and-acceptance-tests.md) | Add diagnostics and acceptance/non-regression test matrix | Diagnostics/Tests | 6 | 📋 Planned |
| [TASK-841](tasks/TASK-841-spec-e-closeout-docs-and-verification.md) | Reconcile docs/status/changelog and record closeout verification evidence | Docs/Planning | 4 | 📋 Planned |
| [TASK-842](tasks/TASK-842-phase113-review-remediation.md) | Remediate independent post-closeout review findings | Review/Hardening | 6 | 📋 Planned |

Estimated total: 73 hours.

## Tracks

### Track A: Spec Gate and Audit

- TASK-830 creates the normative SPEC-E packet.
- TASK-831 audits live parser dispatch, surface/core carriers, TypeEnv canonicalization/registration, normalizer fixture integration, and engine module boundaries before Rust implementation begins.

### Track B: Syntax and Shared Carriers

- TASK-832 adds the parser-only surface and span-preserving AST.
- TASK-833 adds core-owned semantic carriers for type functions, equations, patterns, and source equation result expressions that can represent sealed-domain marker constructors.

### Track C: Registration and Totality Validation

- TASK-834 lowers declarations, predeclares provisional local computation heads, preserves marker-constructor RHS carriers, enforces source-order publication, and publishes only successfully validated heads.
- TASK-835 validates signatures, source type-expression resolution, kind/domain/arity, marker-constructor ambiguity, RHS pattern-variable scoping/substitution environments, result-domain conformance, no-sealed-scrutinee rejection, source-order dependencies, and public/cross-module restrictions.
- TASK-836 implements finite symbolic pattern matrix coverage/overlap, nested residual spaces, unreachable-row detection, and ordered residual catch-all/default semantics.
- TASK-837 implements structural recursion validation.

### Track D: Normalizer and Module Integration

- TASK-838 connects checked source equations to the SPEC-060 normalizer. Completed with source-backed Append reductions, open/partial neutrality, bound-variable substitution, and normal-form definitional equality coverage.
- TASK-839 enforces module-local engine/import boundaries and non-interference.

### Track E: Diagnostics and Closeout

- TASK-840 adds the full diagnostic/acceptance matrix.
- TASK-841 reconciles status surfaces and broad verification evidence.
- TASK-842 closes the post-review remediation slice.

## Execution Order

1. TASK-830 first.
2. TASK-831 second; no Rust implementation begins before the audit gate lands.
3. TASK-832 precedes semantic lowering because the raw AST contract must be span-stable.
4. TASK-833 depends on TASK-831 and may proceed after parser carrier decisions are stable.
5. TASK-834 depends on TASK-832 and TASK-833, and must use source-order registration plus two-phase predeclare/validate/publish registration so recursive self-references can resolve without publishing invalid heads or allowing later forward references.
6. TASK-835 depends on TASK-834 and Phase 111 sealed-domain registration APIs.
7. TASK-836 depends on TASK-835.
8. TASK-837 depends on TASK-836 because recursion validation consumes bound structural-subcomponent facts from checked patterns.
9. TASK-838 depends on TASK-837 and Phase 112 normalizer APIs.
10. TASK-839 depends on TASK-838 and verifies import/export non-interference.
11. TASK-840 depends on TASK-839 and cites evidence from TASK-832 through TASK-839.
12. TASK-841 depends on TASK-840.
13. TASK-842 depends on independent review after TASK-841.

## Implementation Constraints

1. Phase 113 is module-local `type fn` only; `pub type fn`, public ordinary export leakage of local computation heads, and imported equation normalization remain SPEC-F.
2. Type-function applications must lower to computation-head carriers, and sealed-domain marker-constructor RHSs must lower to a domain-constructor result carrier; neither may be encoded as ordinary nominal constructors. Ambiguous nominal/type-function/marker-constructor heads are rejected.
3. Pattern constructors are sealed-domain marker constructors, not ordinary ADT/runtime constructors, and not matched inside unconstrained `Type` slots.
4. Every accepted type function has at least one sealed-domain scrutinee.
5. RHS checking enforces sealed-domain/domain constraints, not kind equality alone.
6. Catch-all/default rows are ordered residual rows over finite known constructors only; nested residual spaces are split only where explicitly inspected, abstract inputs remain neutral, and recursive domains are not expanded without explicit nested patterns.
7. Accepted recursion requires the recursive decreasing argument to be a direct structural subcomponent from the current equation pattern.
8. Mutual recursion, lexicographic recursion, inferred decreases, equality guards, open catch-all reduction, associated families, proof search, and type-function inversion are out of scope.
9. Fuel/cycle guards are robustness diagnostics, not the semantic termination story.
10. TASK-831 is the single owner of the live seam audit and exact implementation target matrix.
11. TASK-838 is the first task allowed to wire source equations into normalizer reduction.
12. TASK-839 is the single owner of public/import boundary evidence for this phase.

## Verification Strategy

Every implementation task must include focused tests and exact non-regression commands. Phase-level closeout must verify:

1. parser accepts `type fn Append(...) decreases xs { case ... }` with accurate spans;
2. parser dispatches `type fn` before ordinary `type` definitions and rejects malformed case heads, missing semicolons, and `pub type fn` / `pub(crate) type fn` with the SPEC-F handoff diagnostic;
3. core/lowering preserves `TypeComputationHeadId`, parameter metadata, equation order, result expressions, and source anchors;
4. local type-function heads use provisional predeclare/validate/publish registration;
5. source type-expression resolution rejects ambiguous nominal/type-function heads and ambiguous marker-constructor/type-head RHSs or patterns;
6. kind/domain/arity validation rejects malformed definitions;
7. result-domain mismatch is rejected even when kind matches;
8. definitions with no sealed-domain scrutinee are rejected;
9. RHS pattern-variable scoping rejects unknown variables and substitutes bound variables into normalizer equations;
10. pattern linearity, finite symbolic coverage, nested residual coverage/defaults, positive multiple-default residual rows, unreachable rows after defaults, overlap, empty-default, and residual catch-all semantics are enforced;
11. structural recursion accepts direct subcomponents and rejects same/rebuilt/computed/mutual arguments, missing/invalid `decreases`, forward references, and calls nested anywhere in source/canonical RHS children;
12. known-scrutinee Append-style reduction works from source declarations, including marker-constructor RHS lowering;
13. open abstract applications remain neutral;
14. catch-all/default rows reduce known residual constructors but not abstract variables;
15. public ordinary exports that leak local computation heads are rejected before SPEC-F;
16. imported/cross-module type-function normalization is rejected before SPEC-F;
17. Phase 109/110/111/112 behavior remains non-regressed;
18. docs/spec index, PLAN-INDEX, task statuses, and CHANGELOG are reconciled honestly.

## Decision Gates

- D1: Phase 113 exposes direct structural `type fn`, not associated recursive families.
- D2: The first slice is module-local; public/exported type-function semantics and public ordinary leakage of computation heads are SPEC-F.
- D3: Totality is definition-time: partial `Head`-style definitions and no-sealed-scrutinee definitions are rejected before use.
- D4: Catch-all/default rows are ordered residual known-constructor coverage only, including explicitly inspected nested residual spaces, not open-variable reduction.
- D5: Definitional equality remains normalize-and-compare and does not invert type functions.
- D6: Structural recursion is direct-subcomponent only; no mutual/lexicographic/size-change termination in this phase.

## Completion Checklist

- [x] SPEC-061 is drafted and registered in `docs/spec/README.md`.
- [x] PLAN-109 and TASK-830 through TASK-842 are registered in `docs/plan/PLAN-INDEX.md`.
- [ ] TASK-831 audit gate names exact live seams and file targets.
- [ ] `ash-parser` exposes raw `type fn` surface syntax and spans.
- [x] `ash-core` exposes shared type-function/equation/pattern/result-expression carriers that preserve sealed-domain marker-constructor RHSs.
- [ ] `ash-typeck` validates signatures, source resolution, RHS variable scope, marker-constructor ambiguity, result domains, patterns, nested residual coverage, overlap, source-order dependencies, and recursion.
- [ ] Checked source equations substitute pattern variables and reduce through the SPEC-060 normalizer.
- [ ] Public/cross-module type-function use and public ordinary export leakage are rejected until SPEC-F.
- [ ] Acceptance, non-interference, broad cargo gates, and independent review are reconciled.
