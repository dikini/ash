# TASK-1694: Core Contract Predicate Artifacts

**Status:** 📝 Planned
**Phase:** [PLAN-165](../PLAN-165-CONTRACT-SYSTEM-IMPLEMENTATION-HANDOFF.md)
**Owner:** Phase 165

## Description

Add Core-side carriers for lowered contract predicates, predicate binders, predicate environments, boundary-local snapshots, and runtime check plans.

## Specification Reference

- [NOTE-031](../../notes/NOTE-031-CONTRACT-PREDICATE-WELL-FORMEDNESS-AND-SNAPSHOTS.md)
- [NOTE-033](../../notes/NOTE-033-SURFACE-TO-CORE-CONTRACT-LOWERING.md)
- [SPEC-098b §4](../../spec/SPEC-098b-TARGET-IR.md)
- [SPEC-100 §9](../../spec/SPEC-100-CORE-TYPE-CHECKING.md)

## Dependencies

- ✅ TASK-1693: Handoff packet

## Requirements

1. Add Rust carriers equivalent to `LoweredPredicate`, `PredicateNode`, `PredicateBinder`, `SnapshotRef`, `PredicateEnvironment`, and `RuntimeCheckPlan`.
2. Preserve source text only as diagnostic metadata, not executable semantics.
3. Make stable predicate identity depend on boundary id, lowered tree, binders, snapshots, predicate-function identities, and type encodings.
4. Expose constructors or builders that make invalid missing-boundary/missing-type states difficult to build.

## TDD Steps

1. Add failing tests in `crates/ash-core/tests/task_1694_core_contract_predicate_artifacts.rs`.
2. Implement carriers in `crates/ash-core/src/core_ash.rs` or a narrowly named adjacent module, then export them from `crates/ash-core/src/lib.rs` if public tests need them.
3. Add text/debug serialization only if needed for focused fixtures; otherwise keep textual support for TASK-1695.

## Verification

```text
strictness: clean
commands:
  - cargo fmt --check
  - cargo test -p ash-core --test task_1694_core_contract_predicate_artifacts
  - cargo clippy -p ash-core --all-targets -- -D warnings
checklist:
  - [ ] Stable identity test covers snapshots and binders.
  - [ ] Source text is diagnostic-only in API shape.
  - [ ] SnapshotRef is boundary-local.
```

## Dependencies for Next Task

Required by TASK-1695, TASK-1696, TASK-1700.
