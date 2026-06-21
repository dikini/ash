# TASK-1650: Add Core type-check integration fixtures

**Status:** Complete
**Phase:** [PLAN-162](../PLAN-162-CORE-ASH-TYPE-CHECKING.md)
**Owner:** Phase 162

## Description

Add end-to-end fixtures proving `.core` text can parse, validate, type-check, and lower to CPS with typed facts preserved.

## Specification Reference

- [SPEC-100](../../spec/SPEC-100-CORE-TYPE-CHECKING.md)
- [SPEC-099 §12](../../spec/SPEC-099-CORE-LANGUAGE.md#12-lowering-to-cps-ir)

## Dependencies

- [TASK-1649](TASK-1649-core-public-summary-scaffold.md)

## Requirements

### Functional Requirements

1. Add valid `.core` fixtures that type-check and lower.
2. Add invalid `.core` fixtures for type mismatch, row mismatch, operation arity mismatch, and affine resume misuse.
3. Add a test pipeline: parse -> validate -> type-check -> lower.
4. Compare representative typed facts, not just successful lowering.
5. Keep fixture text canonical through the existing serializer when possible.

### Property Requirements

- Type-checking must run before lowering in the integration path.
- Lowering must not recompute or contradict checked row facts.

## TDD Steps

### Step 1: Write failing integration tests

**Files:** `crates/ash-core/tests/task_1650_core_typecheck_integration.rs`

Run:

```bash
cargo test -p ash-core --test task_1650_core_typecheck_integration
```

Expected: fail until type-check integration fixtures and APIs exist.

### Step 2: Add fixtures and integration pipeline

**Files:**

- `crates/ash-core/tests/fixtures/core/*.core`
- `crates/ash-core/tests/task_1650_core_typecheck_integration.rs`
- `crates/ash-core/src/core_ash_typecheck.rs`

### Step 3: Verify

Run:

```bash
cargo test -p ash-core --test task_1650_core_typecheck_integration
cargo test -p ash-core --test task_1629_core_end_to_end
cargo test -p ash-core
cargo fmt --check
```

Expected: integration and affected crate tests pass.

## Completion Evidence

Implemented in Phase 162 worktree:

- Added `crates/ash-core/tests/task_1650_core_typecheck_integration.rs`.
- Added invalid Core fixture coverage for type mismatch, row mismatch, operation arity mismatch, and affine resume misuse.
- Added `type_check_and_lower_core_program` so lowering runs after type checking and consumes checked continuation/function row facts.
- Added regression coverage proving checked lowering preserves external function rows from the type-check environment even when the lowering context does not pre-register them.

Verified:

```bash
cargo test -p ash-core --test task_1650_core_typecheck_integration checked_lowering_uses_typechecked_external_function_rows
cargo test -p ash-core --test task_1650_core_typecheck_integration
cargo test -p ash-core --test task_1629_core_end_to_end
cargo test -p ash-core
cargo clippy -p ash-core --all-targets -- -D warnings
cargo fmt --check
git diff --check
cargo test -p spec_processor spec_links
```
