# PLAN-117: Pattern and Exhaustiveness Canonicalization

> **For Hermes:** This is an implementation plan for [SPEC-068](../spec/SPEC-068-PATTERN-EXHAUSTIVENESS-CANONICALIZATION.md). Use subagent-driven-development task-by-task only after the audit gate replaces downstream failing verification guards with exact focused commands. Do not implement beyond the owning SPEC.

**Goal:** Implement [SPEC-068](../spec/SPEC-068-PATTERN-EXHAUSTIVENESS-CANONICALIZATION.md) so the deferred DESIGN-034 gap has a concrete, tested, and non-overclaiming language/type-system slice.

**Architecture:** Pattern canonicalization is audit-first and must not solve under neutral computation heads. Parser work is raw surface; shared identities/carriers live in `ash-core`; TypeEnv owns semantic checking; engine transports summaries without owning type semantics.

**Tech Stack:** Rust 2024, ash-parser, ash-core, ash-typeck, ash-engine, cargo fmt/check/clippy/test/doc.

---

**Status:** ✅ Complete (Implemented MVP)
**Spec:** [SPEC-068](../spec/SPEC-068-PATTERN-EXHAUSTIVENESS-CANONICALIZATION.md)
**Design:** [DESIGN-039](../design/DESIGN-039-PATTERN-EXHAUSTIVENESS-CANONICALIZATION.md)
**Depends on:** SPEC-057 through SPEC-064 implemented MVPs
**Task range:** TASK-912 through TASK-917

## Task Breakdown

| Task | Description | Type | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-912](tasks/TASK-912-pattern-canonicalization-audit-gate.md) | Audit pattern/exhaustiveness constructor resolution and decide equality API vs pattern-specific API | Docs/Substrate | 5 | ✅ Complete |
| [TASK-913](tasks/TASK-913-pattern-canonicalization-api.md) | Add or select the canonicalization API consumed by pattern typing | Typeck/Substrate | 6 | ✅ Complete |
| [TASK-914](tasks/TASK-914-alias-aware-constructor-resolution.md) | Use canonical ADT identities for alias-equivalent constructor lookup without name leakage | Typeck/Pattern | 7 | ✅ Complete |
| [TASK-915](tasks/TASK-915-exhaustiveness-canonical-constructor-universe.md) | Run exhaustiveness over the same canonical constructor universe as pattern typing | Typeck/Exhaustiveness | 7 | ✅ Complete |
| [TASK-916](tasks/TASK-916-pattern-canonicalization-diagnostics-and-negative-leakage.md) | Add blocked-neutral, wrong-identity, and unrelated-name leakage diagnostics/tests | Diagnostics/Tests | 6 | ✅ Complete |
| [TASK-917](tasks/TASK-917-pattern-canonicalization-closeout.md) | Reconcile SPEC-068/PLAN-117 docs, acceptance matrix, broad gates, and review remediation | Docs/Closeout | 5 | ✅ Complete |

## Execution Tracks

- TASK-912: Audit pattern/exhaustiveness constructor resolution and decide equality API vs pattern-specific API
- TASK-913: Add or select the canonicalization API consumed by pattern typing
- TASK-914: Use canonical ADT identities for alias-equivalent constructor lookup without name leakage
- TASK-915: Run exhaustiveness over the same canonical constructor universe as pattern typing
- TASK-916: Add blocked-neutral, wrong-identity, and unrelated-name leakage diagnostics/tests
- TASK-917: Reconcile SPEC-068/PLAN-117 docs, acceptance matrix, broad gates, and review remediation

Total estimate: 36h before review remediation.

## Decision Gates

- D1: The audit gate is mandatory and must bind exact files, callsites, tests, and zero-test-safe commands before Rust changes.
- D2: Do not reuse ordinary `Type::Constructor` for computation-grade or partially applied type-level terms.
- D3: Do not broaden `do`, interface resolution, pattern checking, or summary transport outside this SPEC's explicit scope.
- D4: Every downstream task starts with a fail-closed verification guard until the audit gate patches it.
- D5: Closeout must update the spec index, PLAN-INDEX, task files, CHANGELOG, and acceptance/non-interference evidence.

## Verification Strategy

Each implementation task must run focused non-zero tests for its layer plus:

```bash
cargo fmt --check
git diff --check
cargo check --workspace
```

Closeout additionally runs:

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo doc --workspace --no-deps 2>&1 | tee /tmp/ash-plan-117-doc.log
! grep -i '^warning:' /tmp/ash-plan-117-doc.log
```

## Completion Checklist

- [x] Audit gate artifact exists and downstream guards are patched.
- [x] Parser/core/typeck/engine ownership matches [SPEC-068](../spec/SPEC-068-PATTERN-EXHAUSTIVENESS-CANONICALIZATION.md): pattern semantics remain in `ash-typeck`, parser pattern names remain raw surface, `ash-core`/`ash-engine` do not gain runtime pattern semantics, and ADT runtime layout is unchanged.
- [x] Acceptance/non-interference matrix maps every SPEC row to focused evidence in [SPEC-068](../spec/SPEC-068-PATTERN-EXHAUSTIVENESS-CANONICALIZATION.md) and [TASK-917](tasks/TASK-917-pattern-canonicalization-closeout.md), with PC-4 recorded as partial MVP boundary coverage rather than source-level type-function runtime matching.
- [x] Broad workspace gates are recorded in [TASK-917](tasks/TASK-917-pattern-canonicalization-closeout.md).
- [x] Review remediation surface reconciled for closeout; no TASK-917 review findings were present before the controller's unstaged verification/review pass.

## Phase 121 Closeout

PLAN-117 is complete as an implemented MVP of SPEC-068. The accepted slice canonicalizes transparent aliases and selected reducible associated projections to ordinary ADT constructor universes for pattern typing and exhaustiveness, rejects or blocks neutral/stuck/non-matchable forms without guessing, and preserves constructor identity hygiene for unrelated same-visible-name constructors.

The closeout preserves these explicit boundaries: no GADT/refinement patterns, no runtime matching on type-level sealed-domain or promoted constructors, no broad search-and-replace adoption of equality canonicalization, no type-function or associated-family inversion, and no ADT runtime layout change.
