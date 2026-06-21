# TASK-1648: Record refinement obligations and discharge metadata

**Status:** Complete
**Phase:** [PLAN-162](../PLAN-162-CORE-ASH-TYPE-CHECKING.md)
**Owner:** Phase 162

## Description

Implement the initial refinement obligation and discharge metadata checks from SPEC-100 without proof search.

## Specification Reference

- [SPEC-100 §9](../../spec/SPEC-100-CORE-TYPE-CHECKING.md#9-refinements-and-contracts)

## Dependencies

- [TASK-1647](TASK-1647-core-handle-affine-resume-typecheck.md)

## Requirements

### Functional Requirements

1. Generate obligations when checking plain `T` as `T | P`.
2. Allow values already typed as `T | P` to be used as `T` without a new obligation.
3. Track predicate text with scoped metadata.
4. Validate static/evidence/dynamic discharge record shape.
5. Reject disproved or malformed evidence records when represented.
6. Keep obligations and evidence as compiler metadata, not ordinary Core values.

### Property Requirements

- Statistical evidence must not satisfy hard refinements.
- `ContractViolation` remains trap metadata and never becomes a row item.

## TDD Steps

### Step 1: Write failing refinement/discharge tests

**Files:** `crates/ash-core/tests/task_1648_core_refinement_discharge.rs`

Cover:

- base-to-refinement emits an obligation;
- refinement-to-base emits no obligation;
- unknown/disproved evidence behavior is classified;
- dynamic discharge shape checks;
- contract violation does not add a row item.

Run:

```bash
cargo test -p ash-core --test task_1648_core_refinement_discharge
```

Expected: fail until obligation/discharge support exists.

### Step 2: Implement obligation/discharge scaffolding

**Files:** `crates/ash-core/src/core_ash_typecheck.rs`

Add obligation and discharge metadata outputs to `TypedCoreProgram`.

### Step 3: Verify

Run:

```bash
cargo test -p ash-core --test task_1648_core_refinement_discharge
cargo test -p ash-core --test task_1647_core_handle_affine_resume
cargo fmt --check
```

Expected: focused tests pass.

## Completion Evidence

- Added focused tests in `crates/ash-core/tests/task_1648_core_refinement_discharge.rs`.
- Implemented refinement obligation metadata and discharge validation in `crates/ash-core/src/core_ash_typecheck.rs`.
- Verified:
  - `cargo test -p ash-core --test task_1648_core_refinement_discharge`
  - `cargo test -p ash-core --test task_1647_core_handle_affine_resume`
  - `cargo fmt --check`
