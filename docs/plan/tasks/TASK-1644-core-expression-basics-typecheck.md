# TASK-1644: Type basic Core expressions

**Status:** Complete
**Phase:** [PLAN-162](../PLAN-162-CORE-ASH-TYPE-CHECKING.md)
**Owner:** Phase 162

## Description

Type-check Core expression forms that do not require function-call, continuation, or raised-operation machinery.

## Specification Reference

- [SPEC-100 §11.1-§11.6 and §11.12](../../spec/SPEC-100-CORE-TYPE-CHECKING.md#11-expression-typing)

## Dependencies

- [TASK-1643](TASK-1643-core-atom-value-typing.md)

## Requirements

### Functional Requirements

1. Type `Atom` expressions with row `{}`.
2. Type `LetVal` with declared type checking.
3. Type `LetRec` by pre-binding the declared name.
4. Type `LetPrim` using compiler-known pure primitive signatures.
5. Type `If` with `Bool` condition and compatible branch result types.
6. Type `Trap` at any expected type with row `{}`.

### Property Requirements

- Pure primitives must not introduce effect rows.
- A trapping branch must not add row requirements.

## TDD Steps

### Step 1: Write failing basic expression tests

**Files:** `crates/ash-core/tests/task_1644_core_expression_basics_typecheck.rs`

Cover:

- let-bound literal type-checks;
- declared type mismatch fails;
- pure `LetPrim::Add` checks `Int` arguments/result;
- `If` rejects non-`Bool` condition;
- `Trap` checks against an expected result type with empty row.

Run:

```bash
cargo test -p ash-core --test task_1644_core_expression_basics_typecheck
```

Expected: fail until basic expression typing exists.

### Step 2: Implement basic expression typing

**Files:** `crates/ash-core/src/core_ash_typecheck.rs`

Add expression synthesis/checking for the covered forms.

### Step 3: Verify

Run:

```bash
cargo test -p ash-core --test task_1644_core_expression_basics_typecheck
cargo test -p ash-core --test task_1643_core_atom_value_typing
cargo fmt --check
```

Expected: focused tests pass.

## Completion Evidence

- Added basic expression typing for `LetVal`, `LetRec`, pure `LetPrim`, `If`, and `Trap` in expected-type contexts.
- Added structured type mismatch and argument-count mismatch diagnostics for annotation and primitive-application checks.
- Added `crates/ash-core/tests/task_1644_core_expression_basics_typecheck.rs` covering let-bound literals, declared type mismatch, pure `Add` success/failure, non-`Bool` `If` conditions, and trap branches with empty rows.
- Verified with:
  - `cargo test -p ash-core --test task_1644_core_expression_basics_typecheck`
  - `cargo test -p ash-core --test task_1643_core_atom_value_typing`
  - `cargo clippy -p ash-core --all-targets -- -D warnings`
  - `cargo fmt --check`
  - `git diff --check`
