# TASK-1697: Contract Discharge and Evidence Metadata

**Status:** ✅ Complete
**Phase:** [PLAN-165](../PLAN-165-CONTRACT-SYSTEM-IMPLEMENTATION-HANDOFF.md)
**Owner:** Phase 165

## Description

Record static, survived-testing/evidence, dynamic, and deferred contract discharge metadata in Core summaries and diagnostics.

## Specification Reference

- [NOTE-030](../../notes/NOTE-030-MONADIC-HOARE-LOGIC-FOR-ASH-COMPUTATIONS.md)
- [NOTE-032](../../notes/NOTE-032-CONTRACT-SOUNDNESS-OBLIGATIONS.md)
- [NOTE-033](../../notes/NOTE-033-SURFACE-TO-CORE-CONTRACT-LOWERING.md)
- [SPEC-098b](../../spec/SPEC-098b-TARGET-IR.md)
- [SPEC-100 §9.3](../../spec/SPEC-100-CORE-TYPE-CHECKING.md)

## Dependencies

- ✅ TASK-1696: Dynamic contract traps and predicate faults

## Requirements

1. Materialize `ContractDischarge` records for proven, disproven, unknown/dynamic, survived-testing/evidence, and deferred states.
2. Preserve composed-contract metadata for sequencing/bind obligations.
3. Record enough evidence for later optimizer use without letting optimizers erase diagnostic boundaries.
4. Expose public summaries needed by downstream interface/impl checking.

## TDD Steps

1. Add failing tests in `crates/ash-core/tests/task_1697_contract_discharge_evidence.rs`.
2. Extend Core type-checking/discharge metadata in `crates/ash-core/src/core_ash_typecheck.rs` and related summary types.
3. Add docs consistency tests if public summaries are documented.

## Verification

```text
strictness: clean
commands:
  - cargo fmt --check
  - cargo test -p ash-core --test task_1697_contract_discharge_evidence
  - cargo clippy -p ash-core --all-targets -- -D warnings
checklist:
  - [x] Dynamic discharge records the RuntimeCheckPlan reference.
  - [x] Static/evidence discharge records provenance/evidence references.
  - [x] Composed contract metadata preserves continuation-precondition obligations.
```

## Dependencies for Next Task

Required by TASK-1698.
