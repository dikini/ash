# TASK-1640: Add Core type-checker API and environments

**Status:** Complete
**Phase:** [PLAN-162](../PLAN-162-CORE-ASH-TYPE-CHECKING.md)
**Owner:** Phase 162

## Description

Create the `ash-core::core_ash_typecheck` module with the public checker boundary, typed-program wrapper, environment carriers, and structured diagnostics.

## Specification Reference

- [SPEC-100: Core Type Checking](../../spec/SPEC-100-CORE-TYPE-CHECKING.md)
- [SPEC-099: Core Language](../../spec/SPEC-099-CORE-LANGUAGE.md)

## Dependencies

- [TASK-1632](TASK-1632-core-text-roundtrip-review-fixes.md)

## Requirements

### Functional Requirements

1. Add `crates/ash-core/src/core_ash_typecheck.rs`.
2. Export the module from `crates/ash-core/src/lib.rs`.
3. Define `CoreTypeEnv`, `CoreValueEnv`, `CoreContEnv`, `CoreRowEnv`, `CoreOpEnv`, and `CoreDischargeEnv` or equivalent scoped carriers.
4. Define `TypedCoreProgram` and `CoreTypeCheckError`.
5. Add a checker entrypoint that accepts `ValidCoreProgram` and returns `TypedCoreProgram`.
6. Keep the initial implementation fail-closed for unsupported checks.

### Property Requirements

- The checker API must not accept unvalidated Core without validating first.
- Diagnostics must be structured enough to classify unknown value/type/continuation/operation errors.

## TDD Steps

### Step 1: Write failing API tests

**Files:** `crates/ash-core/tests/task_1640_core_typecheck_api.rs`

Cover:

- empty/default environment construction;
- a minimal valid Core atom program type-checks through the public API;
- unknown variables produce a structured type-check error.

Run:

```bash
cargo test -p ash-core --test task_1640_core_typecheck_api
```

Expected: fail because the type-check module does not exist.

### Step 2: Implement minimal API

**Files:** `crates/ash-core/src/core_ash_typecheck.rs`, `crates/ash-core/src/lib.rs`

Add the smallest environment and error carriers needed by the tests.

### Step 3: Verify

Run:

```bash
cargo test -p ash-core --test task_1640_core_typecheck_api
cargo fmt --check
git diff --check
```

Expected: focused tests pass and formatting is clean.

## Completion Evidence

- Added `ash-core::core_ash_typecheck` with the initial validated-program checker API, typed program wrapper, scoped environment carriers, and structured diagnostics.
- Added `crates/ash-core/tests/task_1640_core_typecheck_api.rs` covering default environments, populated value environments, minimal literal-program type checking, and structured unknown-value errors.
- Verified with:
  - `cargo test -p ash-core --test task_1640_core_typecheck_api`
  - `cargo test -p ash-core`
  - `cargo clippy -p ash-core --all-targets -- -D warnings`
  - `cargo fmt --check`
  - `git diff --check`
