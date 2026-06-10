# TASK-1388: Audit law/proof stdlib readiness

## Status: 📝 Planned

## Description

Freeze exact accepted law and proof syntax for stdlib algebra files before any edits. Verify parser/typechecker behavior for lambda expressions inside law bodies, `by_definition` semantic strength, and `ProofBody::Expr` totality capability.

## Specification Reference

- [PLAN-138: Stdlib Algebra Laws and Pure-Carrier Proofs](../PLAN-138-STDLIB-ALGEBRA-LAWS-AND-PURE-CARRIER-PROOFS.md)
- [SPEC-078: Standard Algebra Library and Monad Remediation](../../spec/SPEC-078-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md)
- [DESIGN-NOTE-INTERFACE-LAWS](../../design/DESIGN-NOTE-INTERFACE-LAWS.md)

## Dependencies

- Phase 136 law/proof syntax (TASK-1360–TASK-1367) completed.

## Deferral / Planned-Feature Reconciliation

This is an audit gate. No stdlib files are changed. Results gate downstream tasks.

## Requirements

### Functional Requirements

- Parse/check current stdlib law-bearing files (`std/src/algebra/semigroup.ash`, `std/src/algebra/monoid.ash`).
- Verify which law-body forms parse and typecheck: `fn(x) => x`, `fn(x) => g(f(x))`, `fn(f) => f(y)`, `fn(f) => fn(g) => fn(x) => f(g(x))`.
- Verify whether `by_definition` is semantically checked or only accepted as proof syntax.
- Verify whether `ProofBody::Expr` can express a total case proof that typechecks as a proposition.
- Record results in `docs/plan/audits/TASK-1388-stdlib-law-proof-readiness.md` with exact commands, failing snippets (if any), and the chosen proof policy.
- Patch downstream task files (TASK-1389–TASK-1394) if the audit changes any planned code shape.

## Files

- Inspect: `std/src/algebra/*.ash`
- Inspect: `std/src/option.ash`
- Inspect: `std/src/result.ash`
- Inspect: `crates/ash-parser/src/parse_module.rs`
- Inspect: `crates/ash-parser/src/surface.rs`
- Inspect: `crates/ash-typeck/src/type_env/`
- Create: `docs/plan/audits/TASK-1388-stdlib-law-proof-readiness.md`

## TDD Steps

### Step 1: Run existing law tests

```bash
RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ash-parser task_1360 task_1362 task_1363 -- --nocapture
```

### Step 2: Test throwaway law-body forms

Create scratch test snippets (outside repo or in a temporary test) to verify:
- `fn(x) => x`
- `fn(x) => g(f(x))`
- `fn(f) => f(y)`
- `fn(f) => fn(g) => fn(x) => f(g(x))`

### Step 3: Verify proof semantics

Check whether `by_definition` is semantically validated or only syntactically accepted.

### Step 4: Record audit

Write `docs/plan/audits/TASK-1388-stdlib-law-proof-readiness.md` with exact commands, results, and the chosen proof policy.

### Step 5: Commit

```bash
git add docs/plan/audits/TASK-1388-stdlib-law-proof-readiness.md
git commit -m "TASK-1388: audit law/proof stdlib readiness"
```

## Dispatch

```
agent: hermes
reasoning: medium
max_turns: 20
toolsets: [terminal, file]
```

## Verification

- [ ] Audit artifact exists with concrete parse/check results
- [ ] Every planned law-body form is classified as parse-accepted or rejected
- [ ] `by_definition` strength is honestly recorded
- [ ] Downstream task files patched if audit changes planned code shape
- [ ] `cargo fmt --check` clean
- [ ] Codex sub-agent review completed
