# TASK-1683: Validate CPS Multi-Shot Row Legality

**Status:** Planned
**Phase:** [PLAN-164](../PLAN-164-CORE-CPS-CONTINUATION-MULTIPLICITY.md)
**Owner:** Phase 164

## Description

Reject malformed CPS input that marks non-pure continuations as multi-shot-pure.

## Specification Reference

- [SPEC-102 §6.3](../../spec/SPEC-102-CPS-CONTINUATION-MULTIPLICITY.md#63-runtime-validation-boundary)

## Dependencies

- [TASK-1681](TASK-1681-cps-cont-multiplicity-carrier.md)

## Files

- Modify: `crates/ash-interp/src/cps/validate.rs` or the current CPS validation module.
- Modify fallback checks in: `crates/ash-interp/src/cps/mod.rs`
- Test: `crates/ash-core/tests/task_1683_cps_multishot_validation.rs` or
  `crates/ash-interp/tests/task_1683_cps_multishot_validation.rs`, depending on validator ownership.

## Requirements

1. CPS validation rejects `Value::Cont { multiplicity: MultiShotPure, row != {}, ... }`.
2. Runtime fail-closed behavior exists for unchecked invalid values if validation is bypassed.
3. Affine continuations with non-empty rows remain valid.
4. Empty-row multi-shot-pure continuations remain valid.
5. Diagnostics mention multiplicity and row legality.

## TDD Steps

1. Add a failing validator test for a non-empty-row multi-shot-pure continuation.
2. Add a passing validator test for empty-row multi-shot-pure.
3. Add a passing validator test for non-empty-row affine.
4. Add an unchecked runtime fail-closed test if the runtime can observe the invalid value.
5. Implement validator/runtime checks.
6. Run the focused test and existing CPS validation tests.

## Completion Checklist

- [ ] Invalid multi-shot rows are rejected before ordinary execution.
- [ ] Affine non-empty rows are unaffected.
- [ ] CHANGELOG has a task entry.
