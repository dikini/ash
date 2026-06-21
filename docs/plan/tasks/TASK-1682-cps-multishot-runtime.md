# TASK-1682: Implement CPS Multi-Shot Runtime Behavior

**Status:** Planned
**Phase:** [PLAN-164](../PLAN-164-CORE-CPS-CONTINUATION-MULTIPLICITY.md)
**Owner:** Phase 164

## Description

Implement runtime behavior for affine versus multi-shot-pure continuation invocation over the
carriers from TASK-1681, including answer-binding `LetContCall` and dynamic handler resume
construction.

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
5. Evaluating `Term::LetCont` copies `Term::LetCont.row` and `Term::LetCont.multiplicity` into the
   created `Value::Cont`.
6. `LetContCall` invokes the continuation, binds its answer to `name`, then evaluates `body`.
7. `LetContCall.row` records the continuation invocation row, equivalent to `Jump.row`.
8. `LetContCall` consumes affine continuations and does not consume multi-shot-pure continuations.
9. Handler dispatch resolves the `Raise.resume` target row before constructing the dynamic resume.
10. For known `HandlerClause.resume_row` metadata, dispatch compares it with the resolved
    `Raise.resume` target row. Mismatch or inability to resolve the target row traps/fails closed.
11. For legacy omitted/inherit-from-target `HandlerClause.resume_row` metadata, dispatch derives
    the affine dynamic resume row from the resolved target row instead of comparing against `{}`.
    This compatibility path must trap/fail closed if the target row cannot be resolved and must not
    be allowed for `resume_multiplicity = MultiShotPure`.
12. Handler dispatch copies the resolved/known row and `HandlerClause.resume_multiplicity` into the
    dynamic resume `Value::Cont`.
13. Existing handler/provider resume tests must keep passing.
14. Do not add Core type-checker behavior in this task.

## TDD Steps

1. Add a failing test where two jumps to an affine continuation trap on the second jump.
2. Add a failing test where two jumps to the same multi-shot-pure continuation both succeed.
3. Add a failing test proving runtime-created continuations from `LetCont` preserve term row and
   multiplicity.
4. Add a failing test proving captured env is used on each invocation.
5. Add a failing test proving captured handler chain behavior matches existing resume semantics.
6. Add a failing `LetContCall` test for affine answer binding and consumption.
7. Add a failing `LetContCall` test for repeated multi-shot answer binding.
8. Add a failing `LetContCall` row-accounting test for an affine non-empty continuation row.
9. Add a failing handler-dispatch test proving known resume-row mismatch traps/fails closed.
10. Add a failing handler-dispatch test proving a legacy omitted affine handler row inherits a
    non-empty resolved target row and does not compare as `{}`.
11. Add a failing handler-dispatch test proving legacy omitted handler row plus multi-shot-pure
    multiplicity traps/fails closed.
12. Add a failing handler-dispatch test proving dynamic resume row and multiplicity come from the
    resolved/known metadata path.
13. Implement runtime branch on `ContMultiplicity`, `LetContCall`, and handler resume construction.
14. Run `cargo test -p ash-interp --test task_1682_cps_multishot_runtime`.
15. Run `cargo test -p ash-interp --test task_1595_cps_ir`.

## Completion Checklist

- [ ] Affine second use still traps.
- [ ] Multi-shot repeated use succeeds.
- [ ] Runtime-created `LetCont` continuations preserve term row and multiplicity.
- [ ] `LetContCall` can bind continuation answers.
- [ ] `LetContCall` carries continuation invocation row accounting.
- [ ] Handler dispatch traps/fails closed on known resume-row mismatch or unresolved target row.
- [ ] Legacy omitted affine handler rows inherit the resolved target row.
- [ ] Legacy omitted handler rows cannot construct multi-shot-pure resumes.
- [ ] Handler-created resumes preserve resolved/known row metadata and handler-clause multiplicity.
- [ ] Captured env/chain behavior is covered.
- [ ] CHANGELOG has a task entry.
