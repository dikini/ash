# TASK-750: Act-Block Compatibility and Migration

## Status: 📝 Planned

## Description

Route `act { ... }` compatibility through generalized do-notation while preserving or warning on legacy SPEC-047 syntax. This task makes `act { ... }` an Act-target spelling rather than a separate semantic system.

## Specification Reference

- [SPEC-054](../../spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md) §5.3
- [SPEC-047](../../spec/SPEC-047-ACT-MONAD.md)

## Dependencies

- 📝 TASK-749: typed do elaboration.

## Requirements

1. Accept new-form `act { x <- expr; return x }` as sugar for `do:Act { ... }`.
2. Preserve legacy `act { x = expr; ret x; }` temporarily or explicitly gate its removal.
3. Emit migration diagnostics for legacy `ret` and ambiguous `x = expr;` bind/inlining.
4. Ensure examples/std tests using current Act blocks are either migrated or covered by compatibility tests.
5. Keep workflow-level `act provider:action ...` disambiguation intact.
6. Update SPEC-047 with a narrow pointer that SPEC-054 owns generalized/new Act-block grammar.

## TDD Steps

### Step 1: Add compatibility tests

**Files:**

- Modify: `crates/ash-parser/src/parse_expr.rs`
- Modify: `crates/ash-typeck/src/check_expr.rs`
- Modify: `docs/spec/SPEC-047-ACT-MONAD.md`

Tests:

- `act { x <- act::unit(1); return x }` checks as `Act<Int>`.
- `act { x = act::unit(1); ret x; }` still checks or emits expected migration warning.
- `act Fs:read ...` workflow syntax still parses on the workflow path.
- new `do:Act` and new `act` produce equivalent checked types.

### Step 2: Implement parser compatibility

Choose one representation:

- parse new `act { ... }` into `Expr::DoBlock` with target `Act`; or
- keep `Expr::ActBlock` as a legacy carrier and normalize to `DoBlock` before type checking.

The result must have one semantic path for new Act sequencing.

### Step 3: Add migration warnings/errors

Add diagnostics for:

- deprecated `ret`;
- legacy `x = expr;` inside Act block;
- final semicolon after new-form `return`.

### Step 4: Verify

Run:

```bash
cargo test -p ash-parser act_block -- --nocapture
cargo test -p ash-typeck act_block -- --nocapture
cargo test --workspace act -- --nocapture
cargo fmt --check
```

## Verification Steps

- [ ] New Act sugar works.
- [ ] Legacy Act behavior is either supported with warnings or intentionally migrated with all examples updated.
- [ ] Workflow-level Act syntax is not regressed.
- [ ] SPEC-047 cross-reference updated.
- [ ] Independent review confirms compatibility decision is explicit.

## Dependencies for Next Task

Required by:

- TASK-751: Act/Proc tower integration tests.
- TASK-752: diagnostics hardening.
