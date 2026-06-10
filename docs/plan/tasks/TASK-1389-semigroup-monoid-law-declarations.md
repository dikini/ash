# TASK-1389: Normalize Semigroup and Monoid law declarations

## Status: 📝 Planned

## Description

Keep Semigroup and Monoid laws in source with explicit `Eq` evidence, normalize imports and formatting, and add regression coverage that the existing laws survive stdlib parsing/import.

## Specification Reference

- [PLAN-138: Stdlib Algebra Laws and Pure-Carrier Proofs](../PLAN-138-STDLIB-ALGEBRA-LAWS-AND-PURE-CARRIER-PROOFS.md)
- [SPEC-078: Standard Algebra Library and Monad Remediation](../../spec/SPEC-078-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md)
- [DESIGN-NOTE-INTERFACE-LAWS](../../design/DESIGN-NOTE-INTERFACE-LAWS.md)

## Dependencies

- TASK-1388 (audit gate) must freeze syntax before this task proceeds.

## Deferral / Planned-Feature Reconciliation

Manual proof validation (Stage 3) is explicitly out of scope. Only law declarations are normalized here.

## Requirements

### Functional Requirements

- Write failing regression tests if Semigroup/Monoid laws are not currently asserted.
- Preserve `associativity`, `left_identity`, `right_identity` with explicit `Eq<A>` evidence.
- Do not duplicate Semigroup associativity inside Monoid unless audit proves required-law reporting cannot traverse `where A: Semigroup`.
- Run focused parser/typechecker tests from TASK-1388 audit.

## Files

- Modify: `std/src/algebra/semigroup.ash`
- Modify: `std/src/algebra/monoid.ash`
- Test: parser/typechecker stdlib law fixture chosen by TASK-1388

## TDD Steps

### Step 1: Write failing regression

Add a test asserting that Semigroup `associativity` and Monoid `left_identity`/`right_identity` law nodes exist after stdlib parsing.

### Step 2: Normalize declarations

Ensure imports, formatting, and `Eq` evidence are consistent with the TASK-1388 audit policy.

### Step 3: Verify

```bash
RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ash-parser -- --nocapture
RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ash-typeck -- --nocapture
```

### Step 4: Commit

```bash
git add std/src/algebra/semigroup.ash std/src/algebra/monoid.ash
git commit -m "TASK-1389: normalize semigroup and monoid law declarations"
```

## Dispatch

```
agent: hermes
reasoning: medium
max_turns: 20
toolsets: [terminal, file]
```

## Verification

- [ ] Focused tests pass for Semigroup/Monoid law surface
- [ ] No fake proof claims introduced
- [ ] `cargo fmt --check` clean
- [ ] `cargo clippy -p ash-parser -p ash-typeck --all-targets -- -D warnings` clean
- [ ] Codex sub-agent review completed
