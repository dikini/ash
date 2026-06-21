# TASK-1666: Type Core thunk values

**Status:** Done
**Phase:** [PLAN-163](../PLAN-163-CORE-LAZY-MEMO-MODES.md)
**Owner:** Phase 163

## Description

Type-check `Thunk` values, preserving latent rows while construction remains pure.

## Specification Reference

- [SPEC-101 §7](../../spec/SPEC-101-LAZY-AND-MEMO-COMPUTATION-MODES.md#7-row-accounting)
- [SPEC-101 §10](../../spec/SPEC-101-LAZY-AND-MEMO-COMPUTATION-MODES.md#10-core-type-checking)

## Dependencies

- [TASK-1665](TASK-1665-core-mode-type-wellformedness.md)
- [TASK-1643](TASK-1643-core-atom-value-typing.md)

## Requirements

1. Thunk body checks against the thunk result type.
2. Thunk annotation row equals the body local row.
3. Thunk construction has local row `{}`.
4. Thunk type is `CoreType::Mode { mode: Lazy, inner: A, latent_row: Some(row) }` or
   `CoreType::Mode { mode: Memo, inner: A, latent_row: Some(row) }` according to the thunk mode.
5. Captures remain metadata and are not ordinary user-visible values.
6. `CoreValue::Thunk.result_ty` is the strict inner result type `A`, not the full mode type.
7. Reject `CoreValue::Thunk.result_ty` when it is already `CoreType::Mode`; nested
   `Strict`/`Lazy`/`Memo` wrappers in the thunk result position are malformed for Phase 163.
8. The checker computes the mode wrapper from `CoreValue::Thunk.mode` and `CoreValue::Thunk.row`;
   it must not read a second latent-row annotation from `result_ty`.

## TDD Steps

1. Add failing tests in `crates/ash-core/tests/task_1666_core_thunk_value_typing.rs`.
2. Cover pure thunk construction, effectful latent row preservation, and body/row mismatch rejection.
3. Run the focused test and confirm typing gaps.
4. Implement thunk value typing.
5. Re-run `task_1666` and `task_1643_core_atom_value_typing`.

## Completion Checklist

- [x] Thunk construction row is empty.
- [x] Latent row is preserved.
- [x] Body type and latent row are checked.
- [x] `result_ty` is validated as the strict inner type.
- [x] A thunk result that is already a mode type is rejected.
- [x] Computed mode type latent row is exactly `CoreValue::Thunk.row`.
