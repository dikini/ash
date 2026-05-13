# PLAN-112: Constraint and Proposition Layer

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task. Phase 116 is DESIGN-034 SPEC-H. Do not implement type-function inversion, unrestricted SMT/proof search, HKT, holes, partial type-constructor application, value-level workflow predicates, or runtime capability-policy solving under this plan.

**Goal:** Implement [SPEC-064](../spec/SPEC-064-CONSTRAINT-PROPOSITION-LAYER.md) so Ash can represent and conservatively discharge type-level equality, disequality, interface-bound, and named-predicate propositions over normalized type expressions.

**Architecture:** Keep shared proposition/evidence carriers and V5 summary schema in `ash-core`; keep parser work raw-surface-only and limited to audited proposition clause sites; make `ash-typeck::TypeEnv` own proposition environments, generated obligations, and conservative solving; reuse the SPEC-060 normalizer for equality/disequality; make `ash-engine` transport proposition summary facts without owning proof semantics.

**Tech Stack:** Rust 2024, ash-core, ash-parser, ash-typeck, ash-engine, serde, cargo tests/clippy/doc.

---

**Status:** 🟡 Ready (TASK-871 through TASK-874 complete; TASK-875 through TASK-884 ready/planned)
**Spec:** [SPEC-064](../spec/SPEC-064-CONSTRAINT-PROPOSITION-LAYER.md)
**Design:** [DESIGN-034 §16.8](../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#168-spec-h-constraintproposition-layer)
**Depends on:** [SPEC-057](../spec/SPEC-057-UNIFIED-TYPE-MODULE-PIPELINE-AND-SEMANTIC-SUMMARIES.md), [SPEC-058](../spec/SPEC-058-CANONICAL-TYPE-EXPRESSION-IR-PROJECTION-IDS-KIND-ARITY-SUBSTRATE.md), [SPEC-059](../spec/SPEC-059-SEALED-TYPE-LEVEL-DOMAINS.md), [SPEC-060](../spec/SPEC-060-NORMALIZER-DEFINITIONAL-EQUALITY-CORE.md), [SPEC-061](../spec/SPEC-061-DIRECT-STRUCTURAL-TYPE-FUNCTIONS.md), [SPEC-062](../spec/SPEC-062-MODULE-SUMMARY-EXPORT-IMPORT-FOR-TYPE-COMPUTATION.md), [SPEC-063](../spec/SPEC-063-ASSOCIATED-TYPE-FAMILY-COMPUTATION.md)

## Task Breakdown

| Task | Description | Type | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-871](tasks/TASK-871-spec-h-spec-plan-packet.md) | Promote DESIGN-034 SPEC-H into SPEC-064/PLAN-112 and register Phase 116 | Docs/Planning | 4 | ✅ Complete |
| [TASK-872](tasks/TASK-872-proposition-layer-audit-gate.md) | Audit live parser/core/typeck/normalizer/engine proposition seams and bind downstream tasks before Rust changes | Docs/Substrate | 5 | ✅ Complete |
| [TASK-873](tasks/TASK-873-core-proposition-carriers.md) | Add core canonical proposition/evidence/refutation/deferred carriers and V5 summary version contract | Core/Substrate | 7 | ✅ Complete |
| [TASK-874](tasks/TASK-874-parser-proposition-surface.md) | Add raw parser surface for audited proposition clauses and explicit named predicate declarations | Parser | 7 | ✅ Complete |
| [TASK-875](tasks/TASK-875-typeenv-proposition-environment.md) | Add TypeEnv proposition environment, obligation generation, and canonical lowering | Typeck/Substrate | 7 | ✅ Complete |
| [TASK-876](tasks/TASK-876-normalized-equality-disequality-solver.md) | Implement normalized equality and conservative constructor-head disequality solving | Typeck/Semantic | 8 | 🟡 Ready |
| [TASK-877](tasks/TASK-877-interface-bound-proposition-solving.md) | Treat interface bounds as proposition evidence without broadening impl search | Typeck/Semantic | 5 | 🟡 Ready |
| [TASK-878](tasks/TASK-878-named-predicate-registration-deferred-solving.md) | Register named predicates and emit deferred unsupported-solving outcomes | Typeck/Diagnostics | 5 | 🟡 Ready |
| [TASK-879](tasks/TASK-879-public-proposition-summary-transport.md) | Export/import public proposition requirements/evidence through V5 semantic summaries | Core/Engine/Typeck | 8 | 🟡 Ready |
| [TASK-880](tasks/TASK-880-checking-point-integration.md) | Integrate proposition discharge at audited checking points without meta/inversion leakage | Typeck/Integration | 7 | 🟡 Ready |
| [TASK-881](tasks/TASK-881-proposition-diagnostics.md) | Add structured diagnostics for unsupported, neutral-blocked, no-inversion, and summary-leakage cases | Diagnostics | 6 | 🟡 Ready |
| [TASK-882](tasks/TASK-882-spec-h-acceptance-non-interference-matrix.md) | Add row-by-row SPEC-H acceptance/non-interference matrix and focused evidence | Tests/Diagnostics | 6 | 🟡 Ready |
| [TASK-883](tasks/TASK-883-spec-h-closeout-docs-and-verification.md) | Reconcile docs/status/changelog and record closeout verification | Docs/Planning | 4 | 🟡 Ready |
| [TASK-884](tasks/TASK-884-phase116-review-remediation.md) | Remediate independent post-closeout review findings | Review/Hardening | 6 | 🟡 Ready |

## Execution Tracks

**Track A (Spec Gate and Audit):** 9h. Promote DESIGN-034 SPEC-H to SPEC-064/PLAN-112, then audit live proposition/constraint/equality/summary seams before Rust changes.

**Track B (Carriers + Surface):** 14h. Add core proposition/evidence/refutation/deferred carriers, V5 summary version gates, parser raw proposition clauses, and named predicate declaration syntax at audited sites.

**Track C (TypeEnv Solver Semantics):** 25h. Build the proposition environment, lower source clauses to canonical propositions, solve normalized equality/disequality, treat interface bounds as evidence, and keep named predicates conservative/deferred.

**Track D (Summaries + Integration):** 15h. Transport public proposition requirements/evidence through V5 summaries, then integrate discharge at audited checking points without type-function inversion or meta-solving leaks.

**Track E (Diagnostics + Closeout):** 22h. Add diagnostics, acceptance matrix, closeout verification, and independent review remediation.

## Key Decisions

1. The proposition solver is conservative and total over outcomes: every proposition is `Satisfied`, `Refuted`, or `Deferred`; deferred is not a panic or stringly fallback.
2. Equality uses SPEC-060 normalize-and-compare evidence. It does not call legacy unification to solve under canonical computation heads.
3. Disequality succeeds only for normalized constructor-head disjointness such as sealed-domain constructor disjointness, even with open constructor arguments; head-open or neutral disequality defers.
4. Interface bounds become proposition facts only when TypeEnv already has concrete impl or where-bound evidence; no broad implicit impl search is added.
5. Named predicates are explicit and registry-backed, but arbitrary predicate proof is deferred unless a compiler-known builtin predicate is registered by TypeEnv.
6. Public proposition transport requires a V5 semantic-summary version. Older summaries carrying proposition facts are malformed.
7. Parser work is raw-surface-only. Semantic proposition identity, solving, and evidence live in `ash-typeck`/`ash-core`, not parser structs.
8. TASK-872 is a hard pre-implementation gate: no TASK-873+ Rust work starts until the audit artifact binds downstream files, callsites, test targets, and zero-test-safe commands.

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
cargo doc --workspace --no-deps 2>&1 | tee /tmp/ash-phase116-doc.log
! grep -i '^warning:' /tmp/ash-phase116-doc.log
```

Focused test commands must either target exact integration-test files created by the task or include a `-- --list`/grep guard proving that matching tests are non-zero before a passing result is accepted. Because TASK-872 is the hard audit gate, TASK-873 through TASK-882 initially contain intentional failing verification guards; TASK-872 must replace those guards with exact focused test commands before implementation starts.

## Completion Checklist

- [x] SPEC-064 promoted in `docs/spec/README.md` as Draft / ready for implementation.
- [x] PLAN-112 and TASK-871 through TASK-884 registered in `docs/plan/PLAN-INDEX.md`.
- [x] TASK-871 created and completed as the planning packet.
- [x] TASK-872 audit artifact created before Rust implementation and downstream TASK-873 through TASK-882 file/test/callsite bindings patched or explicitly confirmed.
- [x] Core proposition/evidence carriers and V5 summary version contract implemented and tested.
- [x] Parser proposition clause and named predicate surface implemented and tested at audited sites.
- [x] TypeEnv proposition environment and canonical lowering implemented and tested.
- [ ] Normalized equality and conservative disequality solver implemented and tested.
- [ ] Interface-bound proposition evidence implemented without broadening impl search.
- [ ] Named predicates produce registered/deferred outcomes with diagnostics.
- [ ] Public proposition summary export/import implemented or explicitly scoped to requirement-only transport with diagnostics.
- [ ] Audited checking points discharge propositions without meta/inversion leakage.
- [ ] Acceptance/non-interference matrix maps every SPEC-064 §12 row to focused tests and all pass.
- [ ] Broad workspace verification recorded.
- [ ] Independent review/remediation complete.
