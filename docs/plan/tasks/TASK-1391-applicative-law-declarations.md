# TASK-1391: Add Applicative identity, homomorphism, interchange, and composition law declarations

## Status: 📝 Planned

## Description

Add the four standard applicative laws to `std/src/algebra/applicative.ash`. This is the riskiest law syntactically because Ash has no implicit currying; nested function-returning closures must parse/check.

## Specification Reference

- [PLAN-138: Stdlib Algebra Laws and Pure-Carrier Proofs](../PLAN-138-STDLIB-ALGEBRA-LAWS-AND-PURE-CARRIER-PROOFS.md)
- [SPEC-078: Standard Algebra Library and Monad Remediation](../../spec/SPEC-078-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md)
- [DESIGN-NOTE-INTERFACE-LAWS](../../design/DESIGN-NOTE-INTERFACE-LAWS.md)

## Dependencies

- TASK-1388 (audit gate) must freeze syntax before this task proceeds.

## Deferral / Planned-Feature Reconciliation

Manual proof validation (Stage 3) is explicitly out of scope. If nested curried lambdas fail to parse/check, the task must add private helper functions or create a follow-up blocker task; the composition law must not silently disappear.

## Requirements

### Functional Requirements

- Add `use algebra::eq::{Eq}`.
- Add `identity`, `homomorphism`, `interchange`, and `composition` laws.
- Treat composition as mandatory unless audit records a concrete syntax/substrate blocker.
- If nested curried lambdas fail, add private helper functions or create a follow-up blocker task; do not silently omit the law.
- Run focused tests and ensure the test target executed at least one law-bearing fixture.

## Files

- Modify: `std/src/algebra/applicative.ash`
- Test: focused parser/typechecker law test chosen by TASK-1388

## TDD Steps

### Step 1: Write failing test

Add tests asserting Applicative law nodes for identity, homomorphism, interchange, and composition exist after parsing.

### Step 2: Add law declarations

Add the four laws with explicit `Eq` evidence per TASK-1388 audit policy. Use private helpers for nested lambdas if the audit shows inline forms are rejected.

### Step 3: Verify

```bash
RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ash-parser -- --nocapture
RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ash-typeck -- --nocapture
```

### Step 4: Commit

```bash
git add std/src/algebra/applicative.ash
git commit -m "TASK-1391: add applicative identity, homomorphism, interchange, composition laws"
```

## Dispatch

```
agent: hermes
reasoning: medium
max_turns: 20
toolsets: [terminal, file]
```

## Verification

- [ ] All four Applicative law declarations present (or explicit blocker for composition)
- [ ] No fake proof claims introduced
- [ ] `cargo fmt --check` clean
- [ ] `cargo clippy -p ash-parser -p ash-typeck --all-targets -- -D warnings` clean
- [ ] Codex sub-agent review completed
