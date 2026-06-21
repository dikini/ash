# TASK-1687: Preserve Multiplicity Through Core-to-CPS Lowering

**Status:** Planned
**Phase:** [PLAN-164](../PLAN-164-CORE-CPS-CONTINUATION-MULTIPLICITY.md)
**Owner:** Phase 164

## Description

Lower Core continuation multiplicity into CPS `Value::Cont` values.

## Specification Reference

- [SPEC-102 §9](../../spec/SPEC-102-CPS-CONTINUATION-MULTIPLICITY.md#9-core-to-cps-lowering)

## Dependencies

- [TASK-1682](TASK-1682-cps-multishot-runtime.md)
- [TASK-1685](TASK-1685-core-handler-multishot-resume-typecheck.md)

## Files

- Modify: `crates/ash-core/src/core_ash_lower.rs`
- Test: `crates/ash-core/tests/task_1687_core_to_cps_multiplicity_lowering.rs`

## Requirements

1. Checked lowering preserves handler resume multiplicity.
2. Affine Core resumes lower to affine CPS continuations.
3. Multi-shot-pure Core resumes lower to multi-shot-pure CPS continuations.
4. Lowering does not infer multi-shot-pure merely because a row is empty.
5. Untyped fallback lowering remains conservative and affine where facts are unavailable.
6. Lowered multi-shot CPS can be run by `ash-interp` without second-use trap.

## TDD Steps

1. Add a failing lowering test that inspects emitted CPS continuation multiplicity.
2. Add a failing integration test that lowers and runs a pure double-resume program.
3. Add an affine-empty-row regression proving empty row alone still lowers affine.
4. Implement lowering facts/plumbing.
5. Run `cargo test -p ash-core --test task_1687_core_to_cps_multiplicity_lowering`.
6. Run relevant `ash-interp` focused runtime test from TASK-1682.

## Completion Checklist

- [ ] Lowering preserves explicit multiplicity.
- [ ] Runtime integration proves lowered multi-shot works.
- [ ] CHANGELOG has a task entry.
