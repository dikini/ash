# TASK-1625: Validate basic Core Ash invariants

**Status:** Complete
**Phase:** [PLAN-161](../PLAN-161-CORE-ASH-IR-FOUNDATION.md)
**Owner:** Phase 161

## Description

Add a Core validator boundary that rejects malformed raw Core AST before lowering.

## Specification Reference

- [SPEC-099: Core Language](../../spec/SPEC-099-CORE-LANGUAGE.md)
- [SPEC-098b: Target IR](../../spec/SPEC-098b-TARGET-IR.md)

## Dependencies

- [TASK-1623](TASK-1623-core-text-parser-expressions.md)

## Requirements

### Functional Requirements

1. Create `crates/ash-core/src/core_ash_validate.rs`.
2. Add `RawCoreProgram` and `ValidCoreProgram` wrappers, or an equivalent clearly named validator API.
3. Validate atom-only argument positions.
4. Validate supported effect operation kinds.
5. Validate row duplicate detection.
6. Validate that labels are not used as ordinary data atoms.

### Property Requirements

- Invalid Core fixtures must fail before lowering.
- Validator errors must identify the violated invariant.

## TDD Steps

### Step 1: Write failing validator tests

**Files:** `crates/ash-core/tests/task_1625_core_validator_basic.rs`

Cover positive and negative cases:

- valid let/prim/call fixture;
- duplicate row item rejected;
- unsupported effect operation rejected;
- label in data position rejected.

Run:

```bash
cargo test -p ash-core --test task_1625_core_validator_basic
```

Expected: fail because validator APIs do not exist.

### Step 2: Implement validator slice

**Files:** `crates/ash-core/src/core_ash_validate.rs`, `crates/ash-core/src/lib.rs`

Do not implement affine resume validation in this task.

### Step 3: Verify

Run:

```bash
cargo test -p ash-core --test task_1625_core_validator_basic
cargo test -p ash-core --test task_1623_core_text_parser_expressions
cargo fmt --check
```

Expected: validator tests pass and parser tests remain green.

## Completion Evidence

- Added `core_ash_validate` with `RawCoreProgram`, `ValidCoreProgram`, `CoreValidationError`, and `validate_core_program`.
- Added recursive validation for rows, duplicate row items, effect operation shape, operation signatures, handler clauses, values, and nested types.
- Confirmed attempted label data atoms fail before lowering at the Core text boundary.
- Verified:
  - `cargo test -p ash-core --test task_1625_core_validator_basic`
  - `cargo test -p ash-core --test task_1623_core_text_parser_expressions`
  - `cargo test -p ash-core`
  - `cargo clippy -p ash-core --all-targets -- -D warnings`
  - `cargo fmt --check`
  - `git diff --check`
