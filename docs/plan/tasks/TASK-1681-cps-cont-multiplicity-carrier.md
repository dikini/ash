# TASK-1681: Add CPS Continuation Multiplicity Carrier

**Status:** Planned
**Phase:** [PLAN-164](../PLAN-164-CORE-CPS-CONTINUATION-MULTIPLICITY.md)
**Owner:** Phase 164

## Description

Add a CPS IR multiplicity carrier to continuation values while preserving current affine behavior
for all existing programs.

## Specification Reference

- [SPEC-102 §5](../../spec/SPEC-102-CPS-CONTINUATION-MULTIPLICITY.md#5-cps-ir-amendment)

## Dependencies

- [TASK-1680](TASK-1680-continuation-multiplicity-spec-plan-packet.md)

## Files

- Modify: `crates/ash-core/src/cps.rs`
- Modify as needed: CPS text/serde helpers if continuation fields are serialized outside serde.
- Test: `crates/ash-core/tests/task_1681_cps_cont_multiplicity_carrier.rs`

## Requirements

1. Add CPS `ContMultiplicity` with `Affine` and `MultiShotPure`.
2. Add `multiplicity: ContMultiplicity` to `Value::Cont`.
3. Preserve serde compatibility with old fixtures by defaulting missing multiplicity to `Affine`.
4. Keep `ConsumedFlag` on continuation values for affine behavior.
5. Do not change runtime invocation behavior in this task beyond compile fixes.
6. Update existing tests that construct `Value::Cont` explicitly.

## TDD Steps

1. Add a failing test that constructs affine and multi-shot `Value::Cont` values.
2. Add a failing serde/defaulting test proving omitted multiplicity deserializes as `Affine`.
3. Implement the enum, default, field, and compile fixes.
4. Run `cargo test -p ash-core --test task_1681_cps_cont_multiplicity_carrier`.
5. Run focused existing CPS carrier tests touched by compile fixes.

## Completion Checklist

- [ ] CPS continuations carry multiplicity.
- [ ] Existing fixtures remain affine by default.
- [ ] No runtime multi-shot behavior is claimed before TASK-1682.
- [ ] CHANGELOG has a task entry.
