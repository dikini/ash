# TASK-1682: Implement CPS Multi-Shot Runtime Behavior

**Status:** Planned
**Phase:** [PLAN-164](../PLAN-164-CORE-CPS-CONTINUATION-MULTIPLICITY.md)
**Owner:** Phase 164

## Description

Implement runtime behavior for affine versus multi-shot-pure continuation invocation.

## Specification Reference

- [SPEC-102 §6](../../spec/SPEC-102-CPS-CONTINUATION-MULTIPLICITY.md#6-runtime-semantics)

## Dependencies

- [TASK-1681](TASK-1681-cps-cont-multiplicity-carrier.md)

## Files

- Modify: `crates/ash-interp/src/cps/mod.rs`
- Test: `crates/ash-interp/tests/task_1682_cps_multishot_runtime.rs`

## Requirements

1. Affine continuations keep current consumed-flag semantics.
2. Multi-shot-pure continuations may be jumped to repeatedly.
3. Multi-shot-pure jumps must not set consumed state.
4. Each multi-shot invocation uses the continuation captured environment and handler chain.
5. Existing handler/provider resume tests must keep passing.
6. Do not add Core type-checker behavior in this task.

## TDD Steps

1. Add a failing test where two jumps to an affine continuation trap on the second jump.
2. Add a failing test where two jumps to the same multi-shot-pure continuation both succeed.
3. Add a failing test proving captured env is used on each invocation.
4. Add a failing test proving captured handler chain behavior matches existing resume semantics.
5. Implement runtime branch on `ContMultiplicity`.
6. Run `cargo test -p ash-interp --test task_1682_cps_multishot_runtime`.
7. Run `cargo test -p ash-interp --test task_1595_cps_ir`.

## Completion Checklist

- [ ] Affine second use still traps.
- [ ] Multi-shot repeated use succeeds.
- [ ] Captured env/chain behavior is covered.
- [ ] CHANGELOG has a task entry.
