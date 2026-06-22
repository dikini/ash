# TASK-1683: Validate CPS Multi-Shot Row Legality

**Status:** ✅ Complete
**Phase:** [PLAN-164](../PLAN-164-CORE-CPS-CONTINUATION-MULTIPLICITY.md)
**Owner:** Phase 164

## Description

Reject malformed CPS input that marks non-pure continuations as multi-shot-pure.

## Specification Reference

- [SPEC-102 §6.4](../../spec/SPEC-102-CPS-CONTINUATION-MULTIPLICITY.md#64-runtime-validation-boundary)

## Dependencies

- [TASK-1681](TASK-1681-cps-cont-multiplicity-carrier.md)

## Files

- Modify: `crates/ash-interp/src/cps/validate.rs` or the current CPS validation module.
- Modify fallback checks in: `crates/ash-interp/src/cps/mod.rs`
- Test: `crates/ash-core/tests/task_1683_cps_multishot_validation.rs` or
  `crates/ash-interp/tests/task_1683_cps_multishot_validation.rs`, depending on validator ownership.

## Requirements

1. CPS validation rejects `Value::Cont { multiplicity: MultiShotPure, row != {}, ... }`.
2. CPS validation rejects `Value::Cont { multiplicity: MultiShotPure, row = {}, body, ... }` when
   the effective row of `body` is non-empty or cannot be proven to match the declared empty row.
3. Runtime fail-closed behavior exists for unchecked invalid values if validation is bypassed.
4. CPS validation rejects `HandlerClause { resume_multiplicity: MultiShotPure, resume_row = Known(non_empty), ... }`.
5. CPS validation rejects `HandlerClause { resume_multiplicity: MultiShotPure, resume_row = legacy/unknown, ... }`.
6. CPS validation rejects `Term::LetCont { multiplicity: MultiShotPure, row != {}, ... }`.
7. CPS validation rejects `Term::LetCont { multiplicity: MultiShotPure, row = {}, cont_body, ... }`
   when the effective row of `cont_body` is non-empty or cannot be proven to match the declared
   empty row.
8. CPS validation rejects `LetContCall` when `LetContCall.row` does not include the resolved
   continuation row.
9. CPS validation rejects statically resolvable known `HandlerClause.resume_row` mismatches with the
   `Raise.resume` target row.
10. Legacy omitted/inherit-from-target `HandlerClause.resume_row` remains valid for affine resumes
   and is not treated as `Known({})`.
11. Affine continuations, affine `LetCont`, and affine handler resumes with non-empty rows remain
   valid.
12. Empty-row multi-shot-pure continuations, `LetCont`, and handler resumes remain valid only when
   their effective body/target rows also validate as empty.
13. Diagnostics mention multiplicity and row legality.

## TDD Steps

1. Add a failing validator test for a non-empty-row multi-shot-pure continuation.
2. Add a failing validator test for an empty-declared-row multi-shot-pure `Value::Cont` whose body
   has a non-empty effective row.
3. Add a passing validator test for empty-row multi-shot-pure with an empty-row body.
4. Add a passing validator test for non-empty-row affine.
5. Add a failing validator test for non-empty `Term::LetCont.row` with multi-shot-pure
   multiplicity.
6. Add a failing validator test for empty-declared-row multi-shot-pure `Term::LetCont` whose
   `cont_body` has a non-empty effective row.
7. Add a failing validator test for known non-empty `HandlerClause.resume_row` with multi-shot-pure
   resume multiplicity.
8. Add a failing validator test for legacy/unknown `HandlerClause.resume_row` with multi-shot-pure
   resume multiplicity.
9. Add a failing validator test for statically resolvable known `HandlerClause.resume_row` mismatch
   with the target resume continuation row.
10. Add a passing validator test for legacy/unknown `HandlerClause.resume_row` with affine
   multiplicity and a non-empty resolved target row.
11. Add a failing validator test for `LetContCall.row` under-reporting the resolved continuation
   row.
12. Add an unchecked runtime fail-closed test if the runtime can observe the invalid value.
13. Implement validator/runtime checks.
14. Run the focused test and existing CPS validation tests.

## Completion Checklist

- [x] Invalid multi-shot rows are rejected before ordinary execution.
- [x] Declared-empty but effectful multi-shot continuation bodies are rejected.
- [x] Invalid multi-shot `LetCont` rows are rejected before ordinary execution.
- [x] Declared-empty but effectful multi-shot `LetCont` bodies are rejected.
- [x] Invalid multi-shot handler resume rows are rejected before resume construction.
- [x] Legacy omitted handler rows are affine-only and are not treated as `Known({})`.
- [x] Statically resolvable handler resume-row mismatches are rejected.
- [ ] `LetContCall` row under-reporting is rejected.
- [x] Affine non-empty rows are unaffected.
- [x] CHANGELOG has a task entry.
