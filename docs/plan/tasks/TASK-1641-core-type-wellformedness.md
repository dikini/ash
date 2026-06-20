# TASK-1641: Check Core type well-formedness

**Status:** Complete
**Phase:** [PLAN-162](../PLAN-162-CORE-ASH-TYPE-CHECKING.md)
**Owner:** Phase 162

## Description

Implement recursive well-formedness checks for all Phase 161 `CoreType` variants.

## Specification Reference

- [SPEC-100 §5](../../spec/SPEC-100-CORE-TYPE-CHECKING.md#5-type-well-formedness)

## Dependencies

- [TASK-1640](TASK-1640-core-typecheck-api-and-environments.md)

## Requirements

### Functional Requirements

1. Validate built-in base type names.
2. Resolve named types and type constructors through the type environment.
3. Validate type variables are in scope.
4. Validate function and continuation rows through the row checker scaffold.
5. Validate tuple, record, refinement, and type application children recursively.
6. Reject malformed type applications by constructor arity/kind.

### Property Requirements

- Type well-formedness must not depend on record field order.
- Textual refinement predicates must still require scoped predicate metadata or a tracked placeholder obligation context.

## TDD Steps

### Step 1: Write failing type well-formedness tests

**Files:** `crates/ash-core/tests/task_1641_core_type_wellformedness.rs`

Cover:

- known base and named types pass;
- unknown named type fails;
- type application arity mismatch fails;
- record equality/well-formedness is field-name based;
- refinement base type is checked recursively.

Run:

```bash
cargo test -p ash-core --test task_1641_core_type_wellformedness
```

Expected: fail until type well-formedness exists.

### Step 2: Implement type well-formedness

**Files:** `crates/ash-core/src/core_ash_typecheck.rs`

Add recursive `check_type_well_formed` helpers and targeted diagnostics.

### Step 3: Verify

Run:

```bash
cargo test -p ash-core --test task_1641_core_type_wellformedness
cargo test -p ash-core --test task_1640_core_typecheck_api
cargo fmt --check
```

Expected: focused tests pass.

## Completion Evidence

- Added recursive Core type well-formedness checking for base, named, variable, function, continuation, tuple, record, refinement, and type-application forms.
- Added row-tail well-formedness scaffolding for function and continuation rows.
- Added field-name-based record type equivalence and scoped textual refinement predicate placeholders.
- Added `crates/ash-core/tests/task_1641_core_type_wellformedness.rs` covering known and unknown type names, type-application arity, record field-order independence, refinement predicate metadata, recursive refinement base checks, and row-tail scope checks.
- Verified with:
  - `cargo test -p ash-core --test task_1641_core_type_wellformedness`
  - `cargo test -p ash-core --test task_1640_core_typecheck_api`
  - `cargo test -p ash-core`
  - `cargo clippy -p ash-core --all-targets -- -D warnings`
  - `cargo fmt --check`
  - `git diff --check`
