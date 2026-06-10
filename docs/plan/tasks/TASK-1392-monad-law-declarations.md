# TASK-1392: Add Monad left identity, right identity, and associativity law declarations

## Status: ✅ Complete

## Description

Add `left_identity`, `right_identity`, and `associativity` laws to `std/src/algebra/monad.ash`.

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
- Add three laws using `unit` and `bind`.
- Prefer `fn(x) => unit(x)` over bare `unit` as a function value unless audit proves bare method references are accepted.
- Run focused tests and ensure non-zero coverage.

## Files

- Modify: `std/src/algebra/monad.ash`
- Test: focused parser/typechecker law test chosen by TASK-1388

## TDD Steps

### Step 1: Write failing test

Add tests asserting Monad law nodes for left identity, right identity, and associativity exist after parsing.

### Step 2: Add law declarations

Add the three laws with explicit `Eq<M<B>>`, `Eq<M<A>>`, `Eq<M<C>>` evidence per TASK-1388 audit policy.

### Step 3: Verify

```bash
RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ash-parser -- --nocapture
RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ash-typeck -- --nocapture
```

### Step 4: Commit

```bash
git add std/src/algebra/monad.ash
git commit -m "TASK-1392: add monad left identity, right identity, associativity laws"
```

## Dispatch

```
agent: hermes
reasoning: medium
max_turns: 20
toolsets: [terminal, file]
```

## Verification

- [ ] All three Monad law declarations present
- [ ] No fake proof claims introduced
- [ ] `cargo fmt --check` clean
- [ ] `cargo clippy -p ash-parser -p ash-typeck --all-targets -- -D warnings` clean
- [ ] Codex sub-agent review completed
