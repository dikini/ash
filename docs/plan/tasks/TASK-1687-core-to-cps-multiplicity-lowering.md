# TASK-1687: Preserve Multiplicity and LetContCall Through Core-to-CPS Lowering

**Status:** Planned
**Phase:** [PLAN-164](../PLAN-164-CORE-CPS-CONTINUATION-MULTIPLICITY.md)
**Owner:** Phase 164

## Description

Lower Core continuation multiplicity and answer-binding continuation invocation into CPS.

## Specification Reference

- [SPEC-102 §9](../../spec/SPEC-102-CPS-CONTINUATION-MULTIPLICITY.md#9-core-to-cps-lowering)

## Dependencies

- [TASK-1682](TASK-1682-cps-multishot-runtime.md)
- [TASK-1685](TASK-1685-core-handler-multishot-resume-typecheck.md)
- [TASK-1686](TASK-1686-core-affine-use-discipline-with-multishot.md)

## Files

- Modify: `crates/ash-core/src/core_ash_lower.rs`
- Test: `crates/ash-core/tests/task_1687_core_to_cps_multiplicity_lowering.rs`

## Requirements

1. Checked lowering preserves handler resume row and multiplicity by writing a known row plus
   multiplicity to CPS `HandlerClause` metadata.
2. Core continuation binders lower to CPS `Term::LetCont.row` and `Term::LetCont.multiplicity`.
3. Affine Core resumes lower to affine CPS handler metadata and continuation values.
4. Multi-shot-pure Core resumes lower to multi-shot-pure CPS handler metadata and continuation
   values.
5. Core answer-binding continuation invocation lowers to CPS `Term::LetContCall` with the checked
   continuation row.
6. Lowering does not infer multi-shot-pure merely because a row is empty.
7. Untyped fallback lowering remains conservative and affine where facts are unavailable.
8. Checked lowering never emits the legacy omitted/inherit-from-target handler row state.
9. Lowered multi-shot CPS can be run by `ash-interp` without second-use trap.

## TDD Steps

1. Add a failing lowering test that inspects emitted CPS `HandlerClause.resume_row` and proves it
   is known metadata, not the legacy omitted/inherit-from-target state.
2. Add a failing lowering test that inspects emitted CPS `HandlerClause.resume_multiplicity`.
3. Add a failing lowering test that inspects emitted CPS `Term::LetCont.row` and
   `Term::LetCont.multiplicity`.
4. Add a failing lowering test that inspects emitted CPS continuation value multiplicity where
   Core lowering constructs an ordinary continuation value.
5. Add a failing lowering test for Core answer-binding continuation invocation to CPS
   `Term::LetContCall` with the checked row.
6. Add a failing integration test that lowers and runs a pure double-resume program.
7. Add an affine-empty-row regression proving empty row alone still lowers affine.
8. Implement lowering facts/plumbing.
9. Run `cargo test -p ash-core --test task_1687_core_to_cps_multiplicity_lowering`.
10. Run relevant `ash-interp` focused runtime test from TASK-1682.

## Completion Checklist

- [ ] Lowering preserves explicit multiplicity.
- [ ] Core continuation binders lower through CPS `Term::LetCont` row and multiplicity fields.
- [ ] Handler resume row and multiplicity survive through known CPS handler metadata.
- [ ] Checked lowering never emits the legacy handler row compatibility state.
- [ ] Core answer-binding continuation invocation lowers to CPS with row accounting.
- [ ] Runtime integration proves lowered multi-shot works.
- [ ] CHANGELOG has a task entry.
