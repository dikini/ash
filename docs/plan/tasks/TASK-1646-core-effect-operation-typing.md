# TASK-1646: Type Core raised operations

**Status:** Complete
**Phase:** [PLAN-162](../PLAN-162-CORE-ASH-TYPE-CHECKING.md)
**Owner:** Phase 162

## Description

Type `Raise` expressions and operation signatures for representable SPEC-096b/SPEC-098b operation kinds.

## Specification Reference

- [SPEC-100 §11.9](../../spec/SPEC-100-CORE-TYPE-CHECKING.md#119-raise)
- [SPEC-099 §8](../../spec/SPEC-099-CORE-LANGUAGE.md#8-effect-operations)

## Dependencies

- [TASK-1645](TASK-1645-core-call-jump-row-accounting.md)

## Requirements

### Functional Requirements

1. Check capability operation path, operation name, argument types, and result type.
2. Check channel operation path, mode, payload type, and result type.
3. Check process operation name, argument types, and result type.
4. Check failure operation optional type.
5. Reject malformed or unrepresentable operation kinds.
6. Type `Raise` local row as operation row only.

### Property Requirements

- Resume/continuation effects are not part of Core `Raise` local row.
- `ContractViolation` must not be accepted as a raised operation.

## TDD Steps

### Step 1: Write failing operation tests

**Files:** `crates/ash-core/tests/task_1646_core_effect_operation_typing.rs`

Cover:

- capability raise checks argument and result types;
- arity/type mismatch fails;
- failure raise with payload type checks;
- contract violation as operation is impossible/rejected;
- raise local row is operation-only.

Run:

```bash
cargo test -p ash-core --test task_1646_core_effect_operation_typing
```

Expected: fail until raised operation typing exists.

### Step 2: Implement operation typing

**Files:** `crates/ash-core/src/core_ash_typecheck.rs`

Add operation signature lookup and row synthesis.

### Step 3: Verify

Run:

```bash
cargo test -p ash-core --test task_1646_core_effect_operation_typing
cargo test -p ash-core --test task_1645_core_call_jump_row_accounting
cargo fmt --check
```

Expected: focused tests pass.

## Completion Evidence

- Added `Raise` type checking for capability, channel, process, and failure operations.
- Added operation signature lookup through `CoreOpEnv`, with arity/type mismatch diagnostics.
- Synthesized operation-local rows only, leaving resume/continuation effects to CPS lowering.
- Added `crates/ash-core/tests/task_1646_core_effect_operation_typing.rs` covering capability, channel, process, failure, unknown operations, argument mismatches, and ContractViolation trap separation.
- Verified with:
  - `cargo test -p ash-core --test task_1646_core_effect_operation_typing`
  - `cargo test -p ash-core --test task_1645_core_call_jump_row_accounting`
  - `cargo clippy -p ash-core --all-targets -- -D warnings`
  - `cargo fmt --check`
  - `git diff --check`
