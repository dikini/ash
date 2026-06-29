# TASK-1695: Contract Predicate Validation and Lowering

**Status:** ✅ Complete
**Phase:** [PLAN-165](../PLAN-165-CONTRACT-SYSTEM-IMPLEMENTATION-HANDOFF.md)
**Owner:** Phase 165

## Description

Validate contract-position predicates and lower accepted predicates into the Core artifact model from TASK-1694.

## Specification Reference

- [NOTE-031](../../notes/NOTE-031-CONTRACT-PREDICATE-WELL-FORMEDNESS-AND-SNAPSHOTS.md)
- [NOTE-033](../../notes/NOTE-033-SURFACE-TO-CORE-CONTRACT-LOWERING.md)
- [SPEC-097b](../../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md)
- [SPEC-100 §9.1](../../spec/SPEC-100-CORE-TYPE-CHECKING.md)

## Dependencies

- ✅ TASK-1694: Core predicate artifacts

## Requirements

1. Implement the static/dynamic/rejected classification boundary.
2. Lower `old(path)` to `SnapshotRef`; reject arbitrary computation in snapshot paths.
3. Reject capability calls, process/workflow operations, handler dispatch, time/randomness/environment observation, and implicit lazy/memo forcing inside predicates.
4. Produce `RuntimeCheckPlan` only for accepted dynamic predicates.
5. Ensure rejected predicates do not reach prover/runtime-check artifacts.

## TDD Steps

1. Add failing tests in `crates/ash-core/tests/task_1695_contract_predicate_validation_lowering.rs`.
2. Implement validation/lowering in `crates/ash-core/src/core_ash_typecheck.rs` or a dedicated helper module called from the type checker.
3. Add `.core` fixtures only if existing Core text format can honestly represent the boundary.

## Verification

```text
strictness: clean
commands:
  - cargo fmt --check
  - cargo test -p ash-core --test task_1695_contract_predicate_validation_lowering
  - cargo clippy -p ash-core --all-targets -- -D warnings
checklist:
  - [x] Accepted static predicates produce proof obligations.
  - [x] Accepted dynamic predicates produce RuntimeCheckPlan.
  - [x] Rejected predicates fail before runtime lowering.
```

## Dependencies for Next Task

Required by TASK-1696 and TASK-1699.
