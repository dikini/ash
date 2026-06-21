# TASK-1660: Add Core mode AST carriers

**Status:** Planned
**Phase:** [PLAN-163](../PLAN-163-CORE-LAZY-MEMO-MODES.md)
**Owner:** Phase 163

## Description

Add the Core Ash representation required by SPEC-101: mode types, thunk values, `LetMode`, and `Force`.

## Specification Reference

- [SPEC-101 §4](../../spec/SPEC-101-LAZY-AND-MEMO-COMPUTATION-MODES.md#4-core-type-shape)
- [SPEC-101 §5](../../spec/SPEC-101-LAZY-AND-MEMO-COMPUTATION-MODES.md#5-core-values-and-expressions)

## Dependencies

- Phase 162

## Requirements

1. Add mode enums/carriers to `crates/ash-core/src/core_ash.rs`.
2. Add `CoreEvalMode::{Strict, Lazy, Memo}` and `CoreThunkMode::{Lazy, Memo}`.
3. Represent `Strict T`, `Lazy T`, and `Memo T` with
   `CoreType::Mode { mode, inner, latent_row }`.
4. `Strict` mode types require `latent_row == None`; `Lazy` and `Memo` mode types carry
   `Some(CoreRow)` once well-formedness checks are implemented.
5. Represent thunk values exactly as
   `CoreValue::Thunk { mode, result_ty, body, row, captures }`.
6. Add `CoreCaptureSet { values: Vec<CoreName> }` as static metadata; it is not ordinary
   user-visible data.
7. Represent expressions exactly as `CoreExpr::LetMode { name, mode, ty, expr, body }`
   and `CoreExpr::Force { name, thunk, body }`.
8. Preserve serde/debug/clone/equality behavior consistent with existing Core carriers.

## TDD Steps

1. Add failing tests in `crates/ash-core/tests/task_1660_core_mode_ast.rs` that construct all new carriers.
2. Run `cargo test -p ash-core --test task_1660_core_mode_ast` and confirm missing variants/types fail.
3. Add the minimal AST variants and exports.
4. Re-run the focused test and `cargo test -p ash-core --test task_1620_core_ash_ast`.
5. Update CHANGELOG and commit.

## Completion Checklist

- [ ] Mode type wrappers are distinct.
- [ ] Lazy/memo thunk carriers include body and latent row.
- [ ] `LetMode.mode` and `LetMode.ty` can both be represented for later validation.
- [ ] `Force` binds a forced result name from a thunk atom.
- [ ] Task tests construct the exact enum variants and fields named in PLAN-163.
