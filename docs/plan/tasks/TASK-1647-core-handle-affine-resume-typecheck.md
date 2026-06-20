# TASK-1647: Type Handle clauses and affine resume

**Status:** Planned
**Phase:** [PLAN-162](../PLAN-162-CORE-ASH-TYPE-CHECKING.md)
**Owner:** Phase 162

## Description

Type `Handle` clauses, enforce conservative affine resume usage, and preserve captured resume effects in residual rows.

## Specification Reference

- [SPEC-100 §11.10 and §12](../../spec/SPEC-100-CORE-TYPE-CHECKING.md#1110-handle)
- [SPEC-098b §5.5](../../spec/SPEC-098b-TARGET-IR.md#55-handler-row-transformation)

## Dependencies

- [TASK-1646](TASK-1646-core-effect-operation-typing.md)

## Requirements

### Functional Requirements

1. Check handled operation identity and parameter types.
2. Require resume type `Cont<op_result, Ans, resume_row, Affine>`.
3. Reject `MultiShotPure` handler resumes in the initial profile.
4. Check handler body under operation params and resume binding.
5. Reuse or integrate Phase 161 affine resume validation.
6. Check `clause.row` excludes resume and outer continuation rows.
7. Compute residual local row as `(handled_segment.local - handled_op) union resume_row union clause.row` for user-installed resumptive handlers.

### Property Requirements

- Effects reachable after `resume` must remain in residual rows.
- `Handle` must not discharge ambient role, policy, contract, resource, or evidence items.

## TDD Steps

### Step 1: Write failing handle tests

**Files:** `crates/ash-core/tests/task_1647_core_handle_affine_resume.rs`

Cover:

- handler params match operation args;
- resume type mismatch fails;
- duplicate resume jump fails;
- storing/passing resume as ordinary data fails;
- residual row preserves `resume_row`;
- handled operation is removed only from the delimited pre-resume segment.

Run:

```bash
cargo test -p ash-core --test task_1647_core_handle_affine_resume
```

Expected: fail until handle typing exists.

### Step 2: Implement handle typing

**Files:** `crates/ash-core/src/core_ash_typecheck.rs`

Integrate conservative affine resume checks and row transformation.

### Step 3: Verify

Run:

```bash
cargo test -p ash-core --test task_1647_core_handle_affine_resume
cargo test -p ash-core --test task_1646_core_effect_operation_typing
cargo test -p ash-core --test task_1626_core_validator_affine_resume
cargo fmt --check
```

Expected: focused tests pass.
