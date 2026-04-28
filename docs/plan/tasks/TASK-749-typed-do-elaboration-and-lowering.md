# TASK-749: Typed Do Elaboration and Lowering

## Status: 📝 Planned

## Description

Type-check generalized do-block statements and lower them only after typed elaboration has resolved the do target and dictionary. This replaces parser/lowerer-driven Act heuristics for new `DoBlock` nodes.

## Specification Reference

- [SPEC-054](../../spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md) §§8-9
- [SPEC-031](../../spec/SPEC-031-FIRST-CLASS-FUNCTIONS.md)
- [SPEC-047](../../spec/SPEC-047-ACT-MONAD.md)
- [SPEC-048](../../spec/SPEC-048-PROC-LIBRARY.md)

## Dependencies

- 📝 TASK-747: surface AST/parser substrate.
- 📝 TASK-748: do-target dictionary resolution.

## Requirements

1. Type `let x = expr;` as pure lexical binding.
2. Type `x <- expr;` only when `expr : K<A>` for the current target `K`.
3. Type final `return expr` as producing block type `K<A>`.
4. Reject missing final return and return-before-end.
5. Reject pure RHS in `<-` with a `use let` hint.
6. Reject mismatched constructors, e.g. `Act<A>` inside `do:Proc` without `proc::from_act`.
7. Lower to target-specific `return`/`bind` calls or an equivalent internal expression only after typed checking.
8. Preserve source spans in error paths.

## TDD Steps

### Step 1: Add semantic tests

**Files:**

- Modify: `crates/ash-typeck/src/check_expr.rs`
- Modify: `crates/ash-parser/src/lower.rs` or a typed elaboration module chosen during implementation.
- Test: typechecker/lowering tests in affected crates.

Tests:

- `do:Act { let x = 1; return x }` has type `Act<Int>`.
- `do:Act { x <- act::unit(1); return x }` has type `Act<Int>`.
- `do:Proc { x <- proc::unit(1); return x }` has type `Proc<Int>`.
- `do:Proc { x <- act::unit(1); return x }` fails with constructor mismatch.
- `do:Act { x <- 1; return x }` fails with pure RHS used as bind.
- `do:Act { let x = act::unit(1); return x }` warns or records lint for unsequenced monadic value.

### Step 2: Implement statement checker

Create helper(s) that thread a block-local environment and substitutions through the statement list. Do not reuse the legacy Act behavior where `=` means either pure or monadic bind.

### Step 3: Implement lowering/elaboration

Lower checked blocks to nested target operations:

- Act -> Act dictionary return/bind.
- Proc -> Proc dictionary return/bind.

Avoid unqualified accidental `unit`/`bind` capture if the existing lowering path allows qualified or internal operations.

### Step 4: Verify

Run:

```bash
cargo test -p ash-typeck do_block -- --nocapture
cargo test -p ash-parser do_block -- --nocapture
cargo fmt --check
```

## Verification Steps

- [ ] Positive Act/Proc type tests pass.
- [ ] Negative wrong-constructor and pure-RHS tests pass.
- [ ] Lowering is demonstrably type-directed for `DoBlock`.
- [ ] Legacy `ActBlock` behavior remains unchanged until TASK-750.
- [ ] Independent review confirms no parser-only lowering regression.

## Dependencies for Next Task

Required by:

- TASK-750: Act compatibility migration.
- TASK-751: Proc integration.
