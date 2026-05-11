# PLAN-110: Module-Summary Export/Import for Type Computation

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Implement [SPEC-062](../spec/SPEC-062-MODULE-SUMMARY-EXPORT-IMPORT-FOR-TYPE-COMPUTATION.md) so public type-computation summaries can be exported, imported, validated, and used for deterministic downstream normalization.

**Architecture:** Keep semantic ownership in `ash-core::semantic_summary` and `ash-core::type_ir`; make `ash-engine` transport/reconcile summaries; make `ash-typeck::TypeEnv` batch-register public computation summaries before normalization. Parser work is limited to preserving visibility and spans for public type-function declarations.

**Tech Stack:** Rust 2024, ash-core, ash-parser, ash-engine, ash-typeck, serde, cargo tests/clippy/doc.

---

**Status:** 🟢 In Progress (planning packet complete; implementation tasks planned)
**Spec:** [SPEC-062](../spec/SPEC-062-MODULE-SUMMARY-EXPORT-IMPORT-FOR-TYPE-COMPUTATION.md)
**Design:** [DESIGN-034 §16.6](../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#166-spec-f-module-summary-exportimport-for-type-computation)
**Depends on:** [SPEC-057](../spec/SPEC-057-UNIFIED-TYPE-MODULE-PIPELINE-AND-SEMANTIC-SUMMARIES.md), [SPEC-058](../spec/SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md), [SPEC-059](../spec/SPEC-059-SEALED-TYPE-LEVEL-DOMAINS.md), [SPEC-060](../spec/SPEC-060-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md), [SPEC-061](../spec/SPEC-061-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md)

## Task Breakdown

| Task | Description | Type | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-843](tasks/TASK-843-spec-f-spec-plan-packet.md) | Promote DESIGN-034 SPEC-F into SPEC-062/PLAN-110 and register Phase 114 | Docs/Planning | 4 | ✅ Complete |
| [TASK-844](tasks/TASK-844-type-computation-summary-audit-gate.md) | Audit live public summary/export/import/normalizer seams before implementation | Docs/Substrate | 5 | ✅ Complete |
| [TASK-845](tasks/TASK-845-core-public-computation-summary-schema.md) | Add core public type-computation summary schema and V3 version contract | Core/Substrate | 8 | 📝 Planned |
| [TASK-846](tasks/TASK-846-parser-public-type-fn-visibility.md) | Preserve `pub type fn` parser surface for SPEC-F validation | Parser | 4 | 📝 Planned |
| [TASK-847](tasks/TASK-847-typeck-public-export-closure-validation.md) | Validate public/private export closure for public type functions | Typeck/Semantic | 8 | 📝 Planned |
| [TASK-848](tasks/TASK-848-transparent-public-equation-summary-lowering.md) | Lower export-closed public equations into transparent public summaries | Typeck/Core | 8 | 📝 Planned |
| [TASK-849](tasks/TASK-849-engine-summary-transport-reconciliation.md) | Transport/reconcile public computation summaries through engine imports | Engine/Integration | 8 | 📝 Planned |
| [TASK-850](tasks/TASK-850-summary-versioning-cache-invalidation.md) | Add summary version/cache/dedup invalidation for computation facts | Core/Engine | 5 | 📝 Planned |
| [TASK-851](tasks/TASK-851-typeenv-imported-head-registration-normalizer.md) | Batch-register imported public heads/equations and normalize downstream | Typeck/Normalizer | 9 | 📝 Planned |
| [TASK-852](tasks/TASK-852-private-opacity-unavailable-reduction-diagnostics.md) | Add private-opacity and unavailable-reduction diagnostics | Diagnostics | 5 | 📝 Planned |
| [TASK-853](tasks/TASK-853-import-order-reexport-determinism.md) | Prove named/glob/pub-use import-order determinism and identity preservation | Engine/Tests | 6 | 📝 Planned |
| [TASK-854](tasks/TASK-854-spec-f-acceptance-non-interference-matrix.md) | Own the DESIGN-034 §16.6 acceptance/non-interference matrix | Tests | 6 | 📝 Planned |
| [TASK-855](tasks/TASK-855-spec-f-closeout-docs-and-verification.md) | Reconcile docs/status/changelog and record closeout verification | Docs/Planning | 4 | 📝 Planned |
| [TASK-856](tasks/TASK-856-phase114-review-remediation.md) | Remediate independent post-closeout review findings | Review/Hardening | 6 | 📝 Planned |

## Execution Tracks

**Track A (Spec Gate and Audit):** 9h. Promote DESIGN-034 SPEC-F to SPEC-062/PLAN-110, then audit live summary/export/import/normalizer seams before Rust changes.

**Track B (Core Summary Schema):** 13h. Add core V3 summary schema, public type-function summary carriers, version rules, and cache/dedup dimensions.

**Track C (Parser + Typeck Export Closure):** 20h. Preserve public type-function declarations, validate public dependency closure, and lower transparent public equations into public summaries.

**Track D (Engine + TypeEnv Import Consumption):** 28h. Transport computation summaries, reconcile fragmented export carriers, add a TypeEnv batch summary-registration API for ordinary types, sealed domains, interface/member identities, and public computation heads/equations, and integrate normalizer lookup.

**Track E (Diagnostics, Acceptance, Closeout):** 21h. Add private-opacity diagnostics, import-order/re-export determinism tests, acceptance/non-interference matrix, closeout verification, and independent remediation.

## Key Decisions

1. SPEC-062 MVP exports **direct checked public equations** for transparent `pub type fn` definitions.
2. A public type function whose equations depend on private helpers or private domains is rejected at export validation; it is not silently converted into an opaque fact.
3. Opaque stable downstream results are represented as neutral computation normal forms with canonical public heads and blocker reasons.
4. All type-computation summary semantics live in `ash-core`; `ash-engine` is transport/cache/reconciliation only.
5. Imported summaries are registered through a batch/two-pass API before normalizer use. The batch declares ordinary types, sealed domains, interface/member identities, and computation heads across all imported summaries before validating domains/equations, guaranteeing import-order independence.
6. Imported public computation summaries are revalidated for SPEC-061 kind/domain, equation, coverage/overlap, result-domain, and structural-recursion invariants before normalizer registration unless a future trusted-summary/digest model explicitly replaces revalidation.
7. Dependency-closure helper heads may be normalizer-available without becoming source-visible names; aliases affect selected visible names only.
8. V1/V2 summaries with non-empty computation facts are rejected before partial registration; only V3 may carry public computation summaries.
9. Acceptance-matrix ownership is singular: TASK-854 owns the end-to-end DESIGN-034 §16.6 matrix; earlier tasks own focused layer tests but TASK-854 must cite or execute every SPEC-062 acceptance row.

## Verification Strategy

Each implementation task runs focused crate tests for the changed layer plus:

```bash
cargo fmt --check
git diff --check
cargo check --workspace
```

Closeout tasks additionally run:

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo doc --workspace --no-deps 2>&1 | tee /tmp/ash-phase114-doc.log
! grep -i '^warning:' /tmp/ash-phase114-doc.log
```

Every task uses subagent-driven development and independent verification before completion.

## Completion Checklist

- [x] SPEC-062 registered in `docs/spec/README.md` as Draft.
- [x] PLAN-110 and TASK-843 through TASK-856 registered in `docs/plan/PLAN-INDEX.md`.
- [x] TASK-843 created and completed as the planning packet.
- [x] TASK-844 audit artifact created before Rust implementation.
- [ ] Core summary schema/versioning implemented and tested, including V1/V2 non-empty computation-field rejection.
- [ ] Public `pub type fn` export closure implemented and tested.
- [ ] Engine transport/reconciliation implemented and tested, including normalizer-available dependency closure without source-visible helper leakage.
- [ ] TypeEnv imported public computation-head registration implemented and tested, including import-side SPEC-061 invariant revalidation before normalizer registration.
- [ ] Acceptance/non-interference matrix maps every SPEC-062 §13 row to focused tests or recorded evidence and passes.
- [ ] Broad workspace verification recorded.
- [ ] Independent review/remediation complete.
