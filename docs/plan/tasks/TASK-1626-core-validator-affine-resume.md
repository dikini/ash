# TASK-1626: Validate affine handler resume restrictions

**Status:** Complete
**Phase:** [PLAN-161](../PLAN-161-CORE-ASH-IR-FOUNDATION.md)
**Owner:** Phase 161

## Description

Extend Core validation to enforce the Phase 161 subset of SPEC-099 handler resume discipline.

## Specification Reference

- [SPEC-099: Core Language](../../spec/SPEC-099-CORE-LANGUAGE.md)
- [SPEC-098b: Target IR](../../spec/SPEC-098b-TARGET-IR.md)

## Dependencies

- [TASK-1625](TASK-1625-core-validator-basic-invariants.md)

## Requirements

### Functional Requirements

1. Track handler resume names within handler bodies.
2. Reject more than one dynamic `Jump` use of the same affine resume in a single straight-line path for the Phase 161 approximation.
3. Reject storing resume continuations in records/tuples or passing them as ordinary function arguments.
4. Allow zero-use non-resumptive handlers.
5. Allow one direct `Jump` to the resume.

### Property Requirements

- The check may be conservative.
- False rejection is acceptable for complex control flow in Phase 161; false acceptance of duplicate resume use is not.

## TDD Steps

### Step 1: Write failing affine validator tests

**Files:** `crates/ash-core/tests/task_1626_core_validator_affine_resume.rs`

Cover:

- one direct resume jump accepted;
- no resume jump accepted;
- two resume jumps in sequence rejected;
- resume passed to `Call` as ordinary argument rejected;
- resume stored in a record rejected.

Run:

```bash
cargo test -p ash-core --test task_1626_core_validator_affine_resume
```

Expected: fail because affine checks are not implemented.

### Step 2: Implement affine checks

**Files:** `crates/ash-core/src/core_ash_validate.rs`

Keep the implementation conservative and documented.

### Step 3: Verify

Run:

```bash
cargo test -p ash-core --test task_1626_core_validator_affine_resume
cargo test -p ash-core --test task_1625_core_validator_basic
cargo fmt --check
```

Expected: affine validator tests pass.

## Completion Evidence

- Added conservative affine handler resume validation to `core_ash_validate`.
- Validates handler resume parameters are affine continuation types.
- Rejects duplicate resume jumps by conservatively counting uses across handler-body branches.
- Rejects resume escape through ordinary calls, primitive/raise arguments, lambda capture, record storage, tuple storage, and ordinary data positions.
- Allows zero-use non-resumptive handlers and one direct `Jump` to the resume.
- Verified:
  - `cargo test -p ash-core --test task_1626_core_validator_affine_resume`
  - `cargo test -p ash-core --test task_1625_core_validator_basic`
  - `cargo test -p ash-core`
  - `cargo clippy -p ash-core --all-targets -- -D warnings`
  - `cargo fmt --check`
  - `git diff --check`
