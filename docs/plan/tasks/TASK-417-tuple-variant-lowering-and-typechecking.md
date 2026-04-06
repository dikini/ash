# TASK-417: Tuple Variant Lowering, Typechecking, and Exhaustiveness

## Status: 🟡 Ready

## Description

Implement the second code follow-on after TASK-416 by teaching lowering and typechecking to handle
unit, record, and tuple variant payloads coherently.

This task should make the tuple-variant source contract type-safe:
- constructor expressions must check payload arity and types by position
- tuple-variant patterns must bind payload positions by order
- ADT metadata and exhaustiveness logic must preserve payload shape

## Specification Reference

- [TASK-413: Canonical Tuple Variant Syntax and ADT Contract Alignment](TASK-413-canonical-tuple-variant-syntax.md)
- [TASK-416: Tuple Variant Parser and Surface AST Substrate](TASK-416-tuple-variant-parser-and-surface-ast.md)
- [SPEC-020: Algebraic Data Types](../../spec/SPEC-020-ADT-TYPES.md)
- [SPEC-003: Type System](../../spec/SPEC-003-TYPE-SYSTEM.md)
- [Type-to-Runtime Contract](../../reference/type-to-runtime-contract.md)

## Dependencies

- ✅ TASK-413 complete
- 🟡 TASK-416 ready/completes parser substrate

## Requirements

### Functional Requirements

1. Lower tuple-variant declarations/expressions/patterns into canonical internal payload metadata while preserving source positional order.
2. Extend ADT/type environment metadata so variants track payload shape explicitly.
3. Typecheck tuple-variant constructor expressions by positional arity and type.
4. Typecheck tuple-variant patterns by positional payload binding.
5. Update exhaustiveness/missing-pattern witnesses so tuple-variant constructors preserve payload shape.
6. Preserve unit-variant and record-variant behavior.
7. Add tests covering:
   - correct constructor typing
   - arity mismatch rejection
   - payload-type mismatch rejection
   - tuple-pattern binding correctness
   - exhaustiveness witness shape preservation

### Non-Functional Requirements

1. Internal elaboration may use synthetic field names for tuple payloads if needed, but source positional semantics must remain the contract.
2. Do not add positional projection syntax.
3. Keep runtime/interpreter execution changes out of this task unless strictly required for test fixtures.
4. Update `CHANGELOG.md`.

## Files

- Modify: `crates/ash-parser/src/lower.rs`
- Modify: `crates/ash-core/src/adt.rs`
- Modify: `crates/ash-typeck/src/type_env.rs`
- Modify: `crates/ash-typeck/src/check_expr.rs`
- Modify: `crates/ash-typeck/src/check_pattern.rs`
- Modify: `crates/ash-typeck/src/exhaustiveness.rs`
- Modify/Add tests under: `crates/ash-typeck/tests/`
- Modify: `CHANGELOG.md`

## TDD Steps

### Step 1: Write failing typeck/exhaustiveness tests

Add tests that prove tuple constructors and tuple patterns are checked positionally, not as named-field record variants.

### Step 2: Implement lowering and type metadata changes

Teach lowering/type environment code to preserve tuple payload shape and constructor metadata.

### Step 3: Implement constructor/pattern/exhaustiveness checking

Add the minimum code to make the new tests pass while preserving old ADT behavior.

### Step 4: Verify affected crate quality

Run at least:
- `cargo test -p ash-typeck`
- `cargo clippy -p ash-typeck --all-targets -- -D warnings`
- `cargo fmt --check`

## Completion Checklist

- [ ] lowering preserves tuple payload order
- [ ] type environment tracks tuple payload shape
- [ ] tuple constructors typecheck correctly
- [ ] tuple patterns typecheck correctly
- [ ] exhaustiveness logic updated
- [ ] tests added/updated
- [ ] `CHANGELOG.md` updated

## Notes

This task intentionally stops at lowering/typechecking. Runtime evaluation and observable behavior
for tuple variants should land in the next task.
