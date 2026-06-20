# TASK-1645: Type calls and jumps with row accounting

**Status:** Complete
**Phase:** [PLAN-162](../PLAN-162-CORE-ASH-TYPE-CHECKING.md)
**Owner:** Phase 162

## Description

Type `LetCall`, tail `Call`, and `Jump` while preserving SPEC-098b local-vs-total row facts for lowering.

## Specification Reference

- [SPEC-100 §11.5, §11.7, §11.8](../../spec/SPEC-100-CORE-TYPE-CHECKING.md#11-expression-typing)
- [SPEC-098b §2.4](../../spec/SPEC-098b-TARGET-IR.md#24-answer-type-discipline)

## Dependencies

- [TASK-1644](TASK-1644-core-expression-basics-typecheck.md)

## Requirements

### Functional Requirements

1. Type `LetCall` against function parameter/result types.
2. Type tail `Call` against the current continuation context.
3. Type `Jump` by looking up the target continuation in `CoreContEnv`.
4. Record `Jump` Core local row as `{}`.
5. Preserve target continuation row separately for CPS `Jump.row` field synthesis.
6. Reject function arity and argument type mismatches.

### Property Requirements

- Function latent row is charged when called, not when constructed.
- `Jump` continuation effects must not inflate function or handler local body rows.

## TDD Steps

### Step 1: Write failing call/jump tests

**Files:** `crates/ash-core/tests/task_1645_core_call_jump_row_accounting.rs`

Cover:

- `LetCall` binds the function result type in its body;
- function arity mismatch fails;
- tail `Call` reports callee-local row;
- `Jump` has local row `{}` but exposes target continuation row as lowering fact.

Run:

```bash
cargo test -p ash-core --test task_1645_core_call_jump_row_accounting
```

Expected: fail until call/jump typing exists.

### Step 2: Implement call/jump typing

**Files:** `crates/ash-core/src/core_ash_typecheck.rs`

Add typed facts needed by `core_ash_lower`.

### Step 3: Verify

Run:

```bash
cargo test -p ash-core --test task_1645_core_call_jump_row_accounting
cargo test -p ash-core --test task_1644_core_expression_basics_typecheck
cargo fmt --check
```

Expected: focused tests pass.

## Completion Evidence

- Added typed lowering facts to `TypedCoreProgram` for jump target continuation rows.
- Added `LetCall`, tail `Call`, and `Jump` type checking with function arity/type checks and continuation argument checks.
- Preserved local row accounting: function latent rows are charged on calls, while `Jump` local row remains `{}` and target continuation rows are stored separately.
- Added `crates/ash-core/tests/task_1645_core_call_jump_row_accounting.rs` covering non-tail calls, arity mismatch, tail call local rows, jump type mismatch, and jump continuation-row facts.
- Verified with:
  - `cargo test -p ash-core --test task_1645_core_call_jump_row_accounting`
  - `cargo test -p ash-core --test task_1644_core_expression_basics_typecheck`
  - `cargo clippy -p ash-core --all-targets -- -D warnings`
  - `cargo fmt --check`
  - `git diff --check`
