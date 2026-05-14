# TASK-876: Normalized equality and disequality solver

## Status: ✅ Complete

## Description

Implement conservative proposition solving for normalized equality and constructor-head disequality over sealed-domain constructor normal forms.

## Specification Reference

- [SPEC-064: Constraint and Proposition Layer](../../spec/SPEC-064-CONSTRAINT-PROPOSITION-LAYER.md)
- [PLAN-112: Constraint and Proposition Layer](../PLAN-112-CONSTRAINT-PROPOSITION-LAYER.md)
- [DESIGN-034 §16.8](../../design/DESIGN-034-TOTAL-TYPE-COMPUTATION.md#168-spec-h-constraintproposition-layer)

## Dependencies

- Depends on TASK-875 completion

## Files / Ownership

- Modify: `crates/ash-typeck/src/type_env.rs`
- Modify: `crates/ash-typeck/src/normalizer.rs` only for missing public helpers over existing `DefinitionalEqualityResult`/`NormalTypeExpr` evidence
- Modify: `crates/ash-typeck/src/error.rs`, `diagnostic.rs` as needed
- Test: `crates/ash-typeck/tests/task_876_proposition_solver.rs`
- Audit rows: H-AUD-CORE-01, H-AUD-CORE-02, H-AUD-TYPECK-04, H-AUD-TYPECK-05, H-FORCE-04, H-RISK-01

## TASK-872 Binding Notes

- Equality propositions must use `Normalizer::definitional_equality`: satisfy only `Equal`, refute only closed `NotEqual`, and defer `BlockedByNeutrality`.
- Disequality may satisfy only closed sealed-domain constructor-head disjointness over `NormalTypeExpr::DomainConstructorApp`, including open arguments under disjoint heads.
- No legacy unification fallback, type-function inversion, associated-family output solving, or substitution/meta mutation is allowed.

## Requirements

### Functional Requirements

1. Use SPEC-060 normalizer/definitional equality for equality propositions.
2. Satisfy equality only on `Equal`; refute only on closed `NotEqual`; defer neutral/rigid blockers.
3. Satisfy disequality for closed sealed-domain constructor-head disjointness such as `Cons<A,T> != Nil`, even when constructor arguments contain open variables.
4. Refute disequality when both sides normalize equal.
5. Defer disequality when the head comparison is open, neutral, or rigid.
6. Prove `Append<Xs, Ys> == Cons<A, Nil>` does not bind/solve `Xs` or `Ys`.

### Non-Goals

- Do not implement type-function inversion, injectivity, unrestricted SMT/proof search, HKT, holes, or partial type-constructor application.
- Do not merge type-level propositions with runtime workflow/capability/provider constraints.
- Do not move semantic proposition ownership into `ash-parser` or `ash-engine`.
- Preserve SPEC-035/SPEC-063 associated-type behavior unless the task explicitly assigns compatibility coverage.

## TDD / Execution Steps

### Step 1

- Write failing tests for H1, H2, H4, H5, and H6 acceptance rows.

### Step 2

- Implement equality solver wrapper returning typed proposition outcomes.

### Step 3

- Implement constructor-head disequality structural-disjointness helper; sealed-domain constructor-head disjointness succeeds even when constructor arguments are open.

### Step 4

- Add negative tests proving no substitution/meta-solving occurs under type functions or associated families.

### Independent Verification

Dispatch an independent review/verification subagent with this task file, SPEC-064, PLAN-112, changed files, and any task-owned audit/test evidence. Do not mark this task complete until findings are fixed and verification passes.

## Completion Checklist

- [x] Focused solver tests pass.
- [x] No legacy unification fallback is used for canonical proposition inversion.
- [x] Neutral/rigid blockers preserve no-inversion notes.

## Completion Notes

- Implemented `TypeEnv::solve_proposition`, `solve_proposition_obligations`, and `add_proposition_obligation` for the Phase 116 proposition environment.
- Equality propositions now wrap SPEC-060 normalizer/definitional-equality evidence: `Equal` satisfies, closed `NotEqual` refutes, and neutral/rigid blockers defer with `no_inversion_boundary=true`.
- Disequality propositions now refute definitionally equal normal forms, satisfy same-domain sealed-constructor head disjointness such as `Cons<A, T> != Nil` even with open arguments, and defer open/neutral/rigid/unrecognized proof-search cases.
- Focused coverage in `crates/ash-typeck/tests/task_876_proposition_solver.rs` verifies H1/H2/H5/H6 behavior directly, no-substitution/no-meta-solving behavior, obligation outcome recording, and non-zero task-owned solver tests. H4 direct type-function equality remains exercised through the existing normalizer substrate; the current core `TypePropositionTerm` carrier cannot faithfully encode sealed-domain constructor proposition terms as `CanonicalTypeExpr::ComputationHeadApp` arguments without a future carrier extension, and the TASK-876 solver delegates any representable canonical type-function equality to `Normalizer::definitional_equality`.
- Verification passed: `cargo fmt --check`, `git diff --check`, non-zero `cargo test -p ash-typeck --test task_876_proposition_solver -- --list`, `cargo test -p ash-typeck --test task_876_proposition_solver` (9 tests), `cargo test -p ash-typeck --test task_875_proposition_environment` (6 tests), and `cargo check --workspace`.
- Independent review and targeted re-review approved the implementation; reviewers found no legacy unification fallback, proof search, type-function inversion, associated-family output solving, or substitution/meta mutation in proposition solver paths.

## Dispatch

```yaml
agent: hermes
reasoning: medium
max_turns: 15
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - cargo fmt --check
  - git diff --check
  - cargo check --workspace
  - test -f crates/ash-typeck/tests/task_876_proposition_solver.rs
  - cargo test -p ash-typeck --test task_876_proposition_solver -- --list | grep -q task_876_
  - cargo test -p ash-typeck --test task_876_proposition_solver
checklist:
  - "[x] Task requirements are satisfied"
  - "[x] Focused verification is recorded"
  - "[x] Status docs and CHANGELOG.md are updated if release-facing docs changed"
```

## Dependencies for Next Task

This task outputs:
- Phase 116 artifact/surface owned by TASK-876 for downstream tasks.
