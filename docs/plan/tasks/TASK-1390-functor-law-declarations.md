# TASK-1390: Add Functor identity and composition law declarations

## Status: 📝 Planned

## Description

Add `identity` and `composition` laws to `std/src/algebra/functor.ash`.

## Specification Reference

- [PLAN-138: Stdlib Algebra Laws and Pure-Carrier Proofs](../PLAN-138-STDLIB-ALGEBRA-LAWS-AND-PURE-CARRIER-PROOFS.md)
- [SPEC-078: Standard Algebra Library and Monad Remediation](../../spec/SPEC-078-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md)
- [DESIGN-NOTE-INTERFACE-LAWS](../../design/DESIGN-NOTE-INTERFACE-LAWS.md)

## Dependencies

- TASK-1388 (audit gate) must freeze syntax before this task proceeds.

## Deferral / Planned-Feature Reconciliation

Manual proof validation (Stage 3) is explicitly out of scope. Only law declarations are added here.

## Requirements

### Functional Requirements

- Add `use algebra::eq::{Eq}`.
- Add `identity` law: `eq.equiv(map(value, fn(x) => x), value)`.
- Add `composition` law: `eq.equiv(map(map(value, f), g), map(value, fn(x) => g(f(x))))`.
- If inline lambdas do not pass, add private helpers and document why in comments/audit.
- Run focused tests and ensure non-zero coverage.

## Files

- Modify: `std/src/algebra/functor.ash`
- Test: focused parser/typechecker law test chosen by TASK-1388

## TDD Steps

### Step 1: Write failing test

Add a test asserting that Functor `identity` and `composition` law nodes exist after parsing.

### Step 2: Add law declarations

Add the two laws with explicit `Eq<F<A>>` and `Eq<F<C>>` evidence per TASK-1388 audit policy.

### Step 3: Verify

```bash
RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ash-parser -- --nocapture
RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ash-typeck -- --nocapture
```

### Step 4: Commit

```bash
git add std/src/algebra/functor.ash
git commit -m "TASK-1390: add functor identity and composition law declarations"
```

## Dispatch

```
agent: hermes
reasoning: medium
max_turns: 20
toolsets: [terminal, file]
```

## Verification

- [ ] Focused tests pass for Functor law surface
- [ ] No fake proof claims introduced
- [ ] `cargo fmt --check` clean
- [ ] `cargo clippy -p ash-parser -p ash-typeck --all-targets -- -D warnings` clean
- [ ] Codex sub-agent review completed
