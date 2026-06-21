# TASK-1685: Type-Check Multi-Shot Handler Resumes

**Status:** Planned
**Phase:** [PLAN-164](../PLAN-164-CORE-CPS-CONTINUATION-MULTIPLICITY.md)
**Owner:** Phase 164

## Description

Allow handler clauses to bind legal multi-shot-pure resume continuations.

## Specification Reference

- [SPEC-102 §7](../../spec/SPEC-102-CPS-CONTINUATION-MULTIPLICITY.md#7-handler-resume-semantics)
- [SPEC-102 §8](../../spec/SPEC-102-CPS-CONTINUATION-MULTIPLICITY.md#8-core-type-checking)

## Dependencies

- [TASK-1684](TASK-1684-core-cont-multiplicity-wellformedness.md)

## Files

- Modify: `crates/ash-core/src/core_ash_typecheck.rs`
- Test: `crates/ash-core/tests/task_1685_core_handler_multishot_resume_typecheck.rs`

## Requirements

1. Replace the current "handler resume with non-affine multiplicity" rejection.
2. Accept `CoreMultiplicity::MultiShotPure` only after the well-formedness rule succeeds.
3. Preserve operation result/input type checking for resumes.
4. Preserve residual-row computation.
5. Keep existing affine handler tests green.

## TDD Steps

1. Add a failing test where a pure handler resume has `(cont Unit Unit {} multi-shot-pure)`.
2. Add a failing test where the resume input type mismatches the handled operation result.
3. Add a failing test where multi-shot-pure resume has an effect row.
4. Update `check_handler_resume`.
5. Run `cargo test -p ash-core --test task_1685_core_handler_multishot_resume_typecheck`.
6. Run `cargo test -p ash-core --test task_1647_core_handle_affine_resume`.

## Completion Checklist

- [ ] Legal multi-shot resumes type check.
- [ ] Illegal multi-shot resumes reject with structured diagnostics.
- [ ] Affine behavior is unchanged.
- [ ] CHANGELOG has a task entry.
