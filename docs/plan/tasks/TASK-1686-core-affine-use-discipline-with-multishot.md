# TASK-1686: Add Core LetContCall and Preserve Affine Use Discipline

**Status:** Planned
**Phase:** [PLAN-164](../PLAN-164-CORE-CPS-CONTINUATION-MULTIPLICITY.md)
**Owner:** Phase 164

## Description

Add a Core answer-binding continuation invocation form and update Core handler-use checks so
repeated affine resume use remains rejected while repeated multi-shot-pure resume use is accepted.

## Specification Reference

- [SPEC-102 §8](../../spec/SPEC-102-CPS-CONTINUATION-MULTIPLICITY.md#8-core-type-checking)

## Dependencies

- [TASK-1685](TASK-1685-core-handler-multishot-resume-typecheck.md)

## Files

- Modify: `crates/ash-core/src/core_ash_validate.rs`
- Modify: `crates/ash-core/src/core_ash_typecheck.rs`
- Modify: `crates/ash-core/src/core_ash.rs`
- Modify: `crates/ash-core/src/core_ash_text.rs`
- Test: `crates/ash-core/tests/task_1686_core_affine_use_discipline_with_multishot.rs`

## Requirements

1. Add Core `LetContCall` or an equivalent Core expression that invokes a continuation, binds its
   answer, and continues evaluating a body.
2. Add `.core` text syntax for the new Core form. Suggested spelling:
   `(let-cont-call name cont-ref atom body)`.
3. Type checking requires the callee to have `Cont<A, Ans, row, multiplicity>`, checks the argument
   against `A`, binds `name : Ans`, and contributes `row` plus the body row.
4. Repeated terminal `Jump` or answer-binding `LetContCall` uses of affine resume are still
   rejected.
5. Repeated terminal `Jump` or answer-binding `LetContCall` uses of multi-shot-pure resume are
   accepted.
6. Discarded affine and multi-shot resumes are accepted.
7. Branch-local use accounting remains sound for conditionals and nested handlers.
8. The implementation must key use discipline off the resume type, not variable name conventions.

## TDD Steps

1. Add a failing parser/serializer test for the Core answer-binding continuation form.
2. Add a failing type-check test where `LetContCall` binds a continuation answer.
3. Add a failing positive test for a handler body that invokes a multi-shot resume twice.
4. Add a negative regression proving the same body with affine resume rejects.
5. Add a positive test where a multi-shot resume is discarded.
6. Add a branch-local test if the current affine checker has branch merge logic.
7. Implement the Core form and use-accounting changes.
8. Run focused test plus `task_1626_core_validator_affine_resume` and
   `task_1647_core_handle_affine_resume`.

## Completion Checklist

- [ ] Affine use discipline remains enforced.
- [ ] Multi-shot repeated use is allowed only for legal multi-shot types.
- [ ] Core can express answer-binding continuation invocation without surface syntax changes.
- [ ] CHANGELOG has a task entry.
