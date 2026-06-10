# TASK-1393: Add honest Option and Result proof declarations

## Status: ✅ Complete

## Description

Add proof declarations to `std/src/option.ash` and `std/src/result.ash` without overstating proof-checking strength. Use `by test` as the safe baseline; upgrade individual proofs only after the TASK-1388 audit proves the checker validates that form against the law proposition.

## Specification Reference

- [PLAN-138: Stdlib Algebra Laws and Pure-Carrier Proofs](../PLAN-138-STDLIB-ALGEBRA-LAWS-AND-PURE-CARRIER-PROOFS.md)
- [SPEC-078: Standard Algebra Library and Monad Remediation](../../spec/SPEC-078-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md)
- [DESIGN-NOTE-INTERFACE-LAWS](../../design/DESIGN-NOTE-INTERFACE-LAWS.md)

## Dependencies

- TASK-1388 (audit gate) must freeze proof policy.
- TASK-1390–TASK-1392 (Functor/Applicative/Monad law declarations) must exist before proofs can reference them.

## Deferral / Planned-Feature Reconciliation

`by_definition` proofs are only allowed after the audit proves the checker validates them against the law proposition. If validation is not strong enough, `by test "..."` is the honest fallback. No false proof success is acceptable.

## Requirements

### Functional Requirements

- Add parser RED tests for proof declarations inside `impl Functor<Option>`, `Applicative<Option>`, `Monad<Option>`, and the corresponding `Result<_, E>` impls.
- Add `by test "..."` proof bodies for all laws as the safe baseline.
- Only upgrade individual proofs to `by_definition` or `ProofBody::Expr` if audit proves the checker validates that proof form against the law proposition.
- Ensure `Result` proof names preserve fixed error type `E` and do not confuse `Err` domain values with operational `fail`.
- Run focused tests with non-zero proof declaration count.

## Files

- Modify: `std/src/option.ash`
- Modify: `std/src/result.ash`
- Test: parser/typechecker proof tests chosen by TASK-1388

## TDD Steps

### Step 1: Write failing test

Add RED tests for proof declarations inside Option and Result impl blocks.

### Step 2: Add proof declarations

Add `by test "..."` proof bodies for all Functor/Applicative/Monad laws in both carriers.

### Step 3: Selectively upgrade (optional)

If audit proves `by_definition` is semantically validated, upgrade specific proofs.

### Step 4: Verify

```bash
RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ash-parser -- --nocapture
RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ash-typeck -- --nocapture
```

### Step 5: Commit

```bash
git add std/src/option.ash std/src/result.ash
git commit -m "TASK-1393: add honest option and result proof declarations"
```

## Dispatch

```
agent: hermes
reasoning: medium
max_turns: 20
toolsets: [terminal, file]
```

## Verification

- [ ] Focused tests pass with non-zero proof declaration count
- [ ] No fake `by_definition` proofs without audit evidence
- [ ] `Result` proof names preserve fixed error type `E`
- [ ] `Err` domain failure distinct from operational bottom
- [ ] `cargo fmt --check` clean
- [ ] `cargo clippy -p ash-parser -p ash-typeck --all-targets -- -D warnings` clean
- [ ] Codex sub-agent review completed
