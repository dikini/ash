# TASK-1662: Validate Core mode forms

**Status:** Planned
**Phase:** [PLAN-163](../PLAN-163-CORE-LAZY-MEMO-MODES.md)
**Owner:** Phase 163

## Description

Add representation validation for SPEC-101 mode forms before type checking or lowering.

## Specification Reference

- [SPEC-101 §5](../../spec/SPEC-101-LAZY-AND-MEMO-COMPUTATION-MODES.md#5-core-values-and-expressions)
- [SPEC-101 §10](../../spec/SPEC-101-LAZY-AND-MEMO-COMPUTATION-MODES.md#10-core-type-checking)

## Dependencies

- [TASK-1661](TASK-1661-core-mode-text-format.md)

## Requirements

1. Reject `LetMode` when `mode` and `ty` disagree.
2. Reject strict `LetMode` paired with non-`Strict` mode type and lazy/memo modes paired with the wrong wrapper.
3. Validate thunk body and force body recursively.
4. Thunk and lambda bodies may reference captured outer bindings.
5. Binders introduced inside thunk/lambda bodies must not leak outward into the enclosing
   expression scope.
6. Add structured validation errors where possible.
7. Phase 163 `Force` requires the forced atom to be `CoreAtom::Var(name)`. Reject non-variable
   forced atoms during validation so lowering never needs to invent a latent-row source for
   literals, primitive names, constructor names, or other atom forms.

## TDD Steps

1. Add failing tests in `crates/ash-core/tests/task_1662_core_mode_validation.rs`.
2. Include valid `if` branch examples with independent mode binders and invalid mode/type mismatch examples.
3. Run `cargo test -p ash-core --test task_1662_core_mode_validation`; expect validation gaps.
4. Implement validation in `core_ash_validate.rs`.
5. Re-run `task_1662`, `task_1625_core_validator_basic`, and `task_1626_core_validator_affine_resume`.

## Completion Checklist

- [ ] Mode/type agreement is validated before type checking.
- [ ] Thunk/force subexpressions are validated.
- [ ] `Force` rejects non-variable forced atoms.
- [ ] Thunk/lambda bodies can reference captured outer bindings.
- [ ] Binders introduced inside nested thunk/lambda values do not leak outward.
- [ ] Existing validation behavior is preserved.
