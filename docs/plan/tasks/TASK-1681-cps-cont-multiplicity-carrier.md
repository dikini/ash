# TASK-1681: Add CPS Continuation and Invocation Carriers

**Status:** Planned
**Phase:** [PLAN-164](../PLAN-164-CORE-CPS-CONTINUATION-MULTIPLICITY.md)
**Owner:** Phase 164

## Description

Add CPS IR row/multiplicity carriers to continuation values, ordinary `LetCont` binders, handler
resume metadata, and the answer-binding `LetContCall` term while preserving current affine behavior
for all existing programs.

## Specification Reference

- [SPEC-102 §5](../../spec/SPEC-102-CPS-CONTINUATION-MULTIPLICITY.md#5-cps-ir-amendment)

## Dependencies

- [TASK-1680](TASK-1680-continuation-multiplicity-spec-plan-packet.md)

## Files

- Modify: `crates/ash-core/src/cps.rs`
- Modify: CPS serde/text/S-expression helpers used for `.cps` fixtures, if separate from `cps.rs`.
- Modify: CPS validation compile surfaces as needed so the new carriers are traversable.
- Test: `crates/ash-core/tests/task_1681_cps_cont_multiplicity_carrier.rs`

## Requirements

1. Add CPS `ContMultiplicity` with `Affine` and `MultiShotPure`.
2. Add `multiplicity: ContMultiplicity` to `Value::Cont`.
3. Add `row: EffectRow` and `multiplicity: ContMultiplicity` to CPS `Term::LetCont`.
4. Add CPS `Term::LetContCall { name, cont, arg, row, body }`.
5. Add resume row metadata to CPS `HandlerClause`, for example
   `resume_row: ResumeRowMetadata`, where the metadata distinguishes `Known(EffectRow)` from a
   legacy omitted/inherit-from-target state.
6. Add resume multiplicity metadata to CPS `HandlerClause`, for example
   `resume_multiplicity: ContMultiplicity`.
7. Preserve serde compatibility with old continuation and `LetCont` fixtures by defaulting missing
   continuation/`LetCont` rows to `{}` and missing multiplicity to `Affine`.
8. Preserve serde compatibility with old handler fixtures by defaulting omitted handler
   `resume_row` to the explicit legacy inherit-from-target state, not to a real `{}` row. Omitted
   handler `resume_multiplicity` still defaults to `Affine`.
9. Keep `ConsumedFlag` on continuation values for affine behavior.
10. Do not change runtime invocation behavior in this task beyond compile fixes.
11. Update existing tests that construct `Value::Cont`, `Term::LetCont`, or `HandlerClause`
    explicitly.

## TDD Steps

1. Add a failing test that constructs affine and multi-shot `Value::Cont` values.
2. Add a failing test that constructs affine and multi-shot `Term::LetCont` values with explicit
   rows.
3. Add a failing test that constructs `Term::LetContCall` with explicit row accounting.
4. Add a failing test that constructs `HandlerClause` with known resume row metadata.
5. Add a failing test that constructs affine and multi-shot `HandlerClause` resume multiplicity
   metadata.
6. Add a failing serde/defaulting test proving omitted continuation/`LetCont` rows default to `{}`.
7. Add a failing serde/defaulting test proving omitted handler `resume_row` deserializes to the
   legacy inherit-from-target state, not to `Known({})`.
8. Add a failing serde/defaulting test proving omitted continuation/`LetCont`/handler
   multiplicity deserializes as `Affine`.
9. Implement the enum, term fields, new term variant, defaulting, traversal, and compile fixes.
10. Run `cargo test -p ash-core --test task_1681_cps_cont_multiplicity_carrier`.
11. Run focused existing CPS carrier tests touched by compile fixes.

## Completion Checklist

- [ ] CPS continuations carry multiplicity.
- [ ] CPS `LetCont` carries row and multiplicity for runtime-created continuations.
- [ ] CPS `LetContCall` exists as an IR carrier with row accounting.
- [ ] CPS handler clauses carry resume row and multiplicity.
- [ ] Legacy omitted handler rows deserialize to an inherit-from-target state.
- [ ] Existing fixtures remain affine by default.
- [ ] No runtime multi-shot behavior is claimed before TASK-1682.
- [ ] CHANGELOG has a task entry.
