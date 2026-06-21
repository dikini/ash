# TASK-1686: Preserve Affine Use Discipline With Multi-Shot

**Status:** Planned
**Phase:** [PLAN-164](../PLAN-164-CORE-CPS-CONTINUATION-MULTIPLICITY.md)
**Owner:** Phase 164

## Description

Update Core handler-use checks so repeated affine resume use remains rejected while repeated
multi-shot-pure resume use is accepted.

## Specification Reference

- [SPEC-102 §8](../../spec/SPEC-102-CPS-CONTINUATION-MULTIPLICITY.md#8-core-type-checking)

## Dependencies

- [TASK-1685](TASK-1685-core-handler-multishot-resume-typecheck.md)

## Files

- Modify: `crates/ash-core/src/core_ash_validate.rs`
- Modify: `crates/ash-core/src/core_ash_typecheck.rs`
- Test: `crates/ash-core/tests/task_1686_core_affine_use_discipline_with_multishot.rs`

## Requirements

1. Repeated jumps to affine resume are still rejected.
2. Repeated jumps to multi-shot-pure resume are accepted.
3. Discarded affine and multi-shot resumes are accepted.
4. Branch-local use accounting remains sound for conditionals and nested handlers.
5. The implementation must key use discipline off the resume type, not variable name conventions.

## TDD Steps

1. Add a failing positive test for a handler body that jumps to a multi-shot resume twice.
2. Add a negative regression proving the same body with affine resume rejects.
3. Add a positive test where a multi-shot resume is discarded.
4. Add a branch-local test if the current affine checker has branch merge logic.
5. Implement use-accounting changes.
6. Run focused test plus `task_1626_core_validator_affine_resume` and
   `task_1647_core_handle_affine_resume`.

## Completion Checklist

- [ ] Affine use discipline remains enforced.
- [ ] Multi-shot repeated use is allowed only for legal multi-shot types.
- [ ] CHANGELOG has a task entry.
