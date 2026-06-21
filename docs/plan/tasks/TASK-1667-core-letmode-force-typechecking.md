# TASK-1667: Type LetMode and Force expressions

**Status:** Done
**Phase:** [PLAN-163](../PLAN-163-CORE-LAZY-MEMO-MODES.md)
**Owner:** Phase 163

## Description

Add expression typing for strict/lazy/memo `LetMode` and `Force`.

## Specification Reference

- [SPEC-101 §7](../../spec/SPEC-101-LAZY-AND-MEMO-COMPUTATION-MODES.md#7-row-accounting)
- [SPEC-101 §8](../../spec/SPEC-101-LAZY-AND-MEMO-COMPUTATION-MODES.md#8-explicit-conversion-operations)
- [SPEC-101 §10](../../spec/SPEC-101-LAZY-AND-MEMO-COMPUTATION-MODES.md#10-core-type-checking)

## Dependencies

- [TASK-1666](TASK-1666-core-thunk-value-typing.md)
- [TASK-1644](TASK-1644-core-expression-basics-typecheck.md)

## Requirements

1. Strict `LetMode` behaves like strict binding after mode/type validation.
2. Lazy `LetMode` binds `CoreType::Mode { mode: Lazy, inner: A, latent_row: Some(row) }`,
   construction row `{}`, and records initializer latent row.
3. Memo `LetMode` binds `CoreType::Mode { mode: Memo, inner: A, latent_row: Some(row) }`,
   construction row `{}`, and records initializer latent row.
4. `Force` accepts `Lazy A` or `Memo A`, binds a strict result, and contributes the thunk latent row.
5. Invalid force of non-mode atoms is rejected.
6. `Force` result binding has the inner type `A`; it does not bind another mode wrapper.
7. `LetMode.mode` and `LetMode.ty` must agree exactly before binding the name.
8. Lazy/memo initializer latent rows come from the checker-computed initializer expression row.
9. The annotated `CoreType::Mode.latent_row` must equal the checker-computed initializer row.
   A mismatch is rejected with
   `CoreTypeCheckError::ModeLatentRowMismatch { name, expected, actual }`, where `expected` is the
   annotated latent row and `actual` is the computed initializer row.
10. Do not require lazy/memo `LetMode.expr` to already be a `CoreValue::Thunk`.
11. Add `mode_binding_latent_rows: HashMap<CoreName, CoreRow>` to `CoreTypeCheckFacts`.
12. Add accessor `mode_binding_latent_rows(&self) -> &HashMap<CoreName, CoreRow>`.
13. `merge_facts` extends `mode_binding_latent_rows`; later facts replace earlier rows for the
    same `CoreName`.
14. The checker does not mutate the source AST to insert computed rows; it returns checked types
    and records the computed row in `CoreTypeCheckFacts::mode_binding_latent_rows`.
15. Phase 163 `Force` accepts only `CoreAtom::Var(name)` as the forced thunk atom. Reject literal,
    primitive-name, constructor-name, tuple, record, or any other non-variable atom before lowering.

## TDD Steps

1. Add failing tests in `crates/ash-core/tests/task_1667_core_letmode_force_typecheck.rs`.
2. Cover construction row, force row, mode mismatch, and invalid force.
3. Run the focused test and confirm missing expression cases.
4. Implement expression typing and checked lowering facts needed for force rows.
5. Re-run `task_1667`, `task_1644`, and `task_1645`.

## Completion Checklist

- [x] Force contributes latent row.
- [x] Lazy/memo construction remains pure.
- [x] Strict `LetMode` does not create a thunk boundary.
- [x] Mode mismatch diagnostics are distinct from ordinary type mismatch diagnostics.
- [x] Checked-lowering metadata preserves the same latent row as the mode type.
- [x] `CoreTypeCheckFacts::mode_binding_latent_rows()` exposes lazy/memo binding rows.
- [x] Lazy/memo annotation rows that disagree with computed initializer rows fail with
      `ModeLatentRowMismatch`.
- [x] Non-variable force atoms are rejected by type checking.
