# PLAN-111: Associated Type-Family Computation

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Implement [SPEC-063](../spec/SPEC-063-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md) so sealed associated type families reduce through the total normalizer without turning ordinary impl search into a hidden solver.

**Architecture:** Keep family identities, checked family scheme/result carriers, normal-form blockers, and public summaries in `ash-core`; keep parser work raw-surface-only while adding the missing typed-parameter/projection/declaration carriers; make `ash-typeck::TypeEnv` own family declaration registration, module ownership, coherence, one-way selected-scheme lookup, and validation; integrate local projection-family reduction into the SPEC-060 normalizer before V4 import adds imported family tables; make `ash-engine` transport V4 public family summaries without owning semantics.

**Tech Stack:** Rust 2024, ash-core, ash-parser, ash-typeck, ash-engine, serde, cargo tests/clippy/doc.

---

**Status:** 🟡 In progress (TASK-857 through TASK-861 complete; TASK-862 through TASK-870 ready)
**Spec:** [SPEC-063](../spec/SPEC-063-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md)
**Design:** [DESIGN-034 §16.7](../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#167-spec-g-associated-type-family-computation)
**Depends on:** [SPEC-057](../spec/SPEC-057-UNIFIED-TYPE-MODULE-PIPELINE-AND-SEMANTIC-SUMMARIES.md), [SPEC-058](../spec/SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md), [SPEC-059](../spec/SPEC-059-SEALED-TYPE-LEVEL-DOMAINS.md), [SPEC-060](../spec/SPEC-060-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md), [SPEC-061](../spec/SPEC-061-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md), [SPEC-062](../spec/SPEC-062-MODULE-SUMMARY-EXPORT-IMPORT-FOR-TYPE-COMPUTATION.md)

## Task Breakdown

| Task | Description | Type | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-857](tasks/TASK-857-spec-g-spec-plan-packet.md) | Promote DESIGN-034 SPEC-G into SPEC-063/PLAN-111 and register Phase 115 | Docs/Planning | 4 | ✅ Complete |
| [TASK-858](tasks/TASK-858-associated-family-audit-gate.md) | Audit live associated projection, impl, normalizer, and summary seams, then bind downstream tasks to exact files/tests/callsites before implementation | Docs/Substrate | 5 | ✅ Complete |
| [TASK-859](tasks/TASK-859-associated-family-surface-and-compat-parser.md) | Add explicit family projection/declaration surface and preserve SPEC-035 compatibility parsing | Parser | 6 | ✅ Complete |
| [TASK-860](tasks/TASK-860-core-associated-family-identity-carriers.md) | Add core associated-family identity/projection/summary carriers and V4 version contract | Core/Substrate | 7 | ✅ Complete |
| [TASK-861](tasks/TASK-861-typeck-family-declaration-registration-coherence.md) | Register sealed family declarations and validate impl-family coherence | Typeck/Semantic | 8 | ✅ Complete |
| [TASK-862](tasks/TASK-862-spec035-substitution-compatibility-bridge.md) | Preserve SPEC-035 selected-impl substitution while bridging family declarations | Typeck/Compatibility | 5 | 🟡 Ready |
| [TASK-863](tasks/TASK-863-unique-generic-impl-family-selection.md) | Reduce unique generic impl-family schemes over concrete and abstract arguments | Typeck/Normalizer | 7 | 🟡 Ready |
| [TASK-864](tasks/TASK-864-rigid-where-bound-projection-boundary.md) | Enforce rigid projection behavior for generic where-bound evidence | Typeck/Equality | 5 | 🟡 Ready |
| [TASK-865](tasks/TASK-865-recursive-associated-family-totality.md) | Validate recursive associated-family coverage, overlap, and decreasingness | Typeck/Semantic | 8 | 🟡 Ready |
| [TASK-866](tasks/TASK-866-normalizer-projection-family-integration.md) | Integrate associated-family reduction into normalizer and definitional equality | Normalizer | 7 | 🟡 Ready |
| [TASK-867](tasks/TASK-867-associated-family-summary-export-import.md) | Export/import public associated-family summaries through V4 semantic summaries | Core/Engine/Typeck | 8 | 🟡 Ready |
| [TASK-868](tasks/TASK-868-associated-family-diagnostics-acceptance-matrix.md) | Add diagnostics and row-by-row acceptance/non-interference matrix | Tests/Diagnostics | 6 | 🟡 Ready |
| [TASK-869](tasks/TASK-869-spec-g-closeout-docs-and-verification.md) | Reconcile docs/status/changelog and record closeout verification | Docs/Planning | 4 | 🟡 Ready |
| [TASK-870](tasks/TASK-870-phase115-review-remediation.md) | Remediate independent post-closeout review findings | Review/Hardening | 6 | 🟡 Ready |

## Execution Tracks

**Track A (Spec Gate and Audit):** 9h. Promote DESIGN-034 SPEC-G to SPEC-063/PLAN-111, then audit live projection, impl, normalizer, and summary seams before Rust changes.

**Track B (Surface + Core Carriers):** 13h. Add typed interface/impl parameters, explicit `<Interface<Args>>::Assoc` family projection syntax, and raw `sealed type family Name: Domain [decreases Param]` declaration syntax while preserving SPEC-035 compatibility, then add core family identities, helper APIs, checked family scheme/result carriers, normal-form blocker support, and concrete V4 summary carriers.

**Track C (Typeck Family Semantics):** 25h. Register sealed family declarations, validate impl-family coherence, preserve simple substitution, implement unique generic scheme selection, and enforce rigid where-bound behavior.

**Track D (Recursion + Normalizer + Summaries):** 23h. Validate recursive family totality/decreasingness, integrate family reduction into the normalizer/equality path, and transport public family summaries across modules.

**Track E (Diagnostics + Closeout):** 16h. Add diagnostics, acceptance/non-interference matrix, closeout verification, and independent remediation.

## Key Decisions

1. Reducible associated families require explicit `sealed type family` declarations. Ordinary `type Name` associated types retain SPEC-035 compatibility behavior only.
2. Explicit computation-grade projection syntax is `<Interface<Args...>>::Assoc`; existing `Base::Assoc` remains accepted and elaborates through compatibility rules when unambiguous.
3. Family equation selection is closed, unique, coherent, and non-inverting. Generic bounds establish evidence/rigid projections; they do not select equations.
4. Unique generic impl schemes may reduce over abstract arguments only by one-way scheme-head matching that binds scheme-owned variables, e.g. `<Iterator<List<X>>>::Item -> X`; queried projection variables/metas and expected outputs are never solved.
5. Recursive families reuse SPEC-061 residual coverage/overlap/decreasingness principles, adapted to impl-family heads and sealed-domain arguments.
6. Public family transport requires concrete V4 semantic summaries. V1/V2/V3 summaries with family facts are malformed and rejected; V4 exports are reducible only when the full validated closed equation set and every dependency are public-summary-visible.
7. `ash-core` owns semantic family identities and summaries; `ash-engine` transports/reconciles them only.
8. Associated-family reduction remains normalize-and-compare. No projection inversion, proof search, or proposition solving lands in Phase 115.
9. Acceptance-matrix ownership is singular: TASK-868 owns every SPEC-063 §13 acceptance row and may cite earlier focused suites.
10. TASK-858 is a hard pre-implementation gate: no TASK-859+ Rust work starts until the audit artifact has patched or explicitly bound each downstream task to exact files, callsites, test targets, and zero-test-safe verification commands.

## Verification Strategy

Each implementation task runs zero-test-safe focused crate tests for the changed layer plus:

```bash
cargo fmt --check
git diff --check
cargo check --workspace
```

Closeout tasks additionally run:

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo doc --workspace --no-deps 2>&1 | tee /tmp/ash-phase115-doc.log
! grep -i '^warning:' /tmp/ash-phase115-doc.log
```

Every task uses subagent-driven development and independent verification before completion. Focused test commands must either use exact integration-test targets created by the task or include a `-- --list`/grep guard proving non-zero matching tests before a passing result is accepted.

## Completion Checklist

- [x] SPEC-063 promoted in `docs/spec/README.md` as Draft / ready for implementation.
- [x] PLAN-111 and TASK-857 through TASK-870 registered in `docs/plan/PLAN-INDEX.md`.
- [x] TASK-857 created and completed as the planning packet.
- [x] TASK-858 audit artifact created before Rust implementation and downstream TASK-859 through TASK-868 file/test/callsite bindings patched or explicitly confirmed.
- [x] Parser surface and compatibility syntax implemented and tested.
- [x] Core associated-family identity/projection/summary carriers implemented and tested.
- [ ] TypeEnv sealed family registration/coherence implemented and tested.
- [ ] SPEC-035 simple associated substitution preserved.
- [ ] Unique generic impl-family reduction and rigid where-bound behavior implemented and tested.
- [ ] Recursive family totality/decreasingness implemented and tested.
- [ ] Normalizer/equality projection-family integration implemented and tested.
- [ ] Public V4 family summary export/import implemented and tested.
- [ ] Acceptance/non-interference matrix maps every SPEC-063 §13 row to focused tests or recorded evidence and passes.
- [ ] Broad workspace verification recorded.
- [ ] Independent review/remediation complete.
