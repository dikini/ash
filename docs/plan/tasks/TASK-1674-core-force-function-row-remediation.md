# TASK-1674: Core Force Function Row Remediation

**Status:** Done
**Phase:** [PLAN-163](../PLAN-163-CORE-LAZY-MEMO-MODES.md)
**Owner:** Review remediation

## Description

Fix Phase 163 checked-lowering regressions for forced thunks whose strict inner type is a function.
Checked lowering must keep local `LetMode` bindings in scope during the let-call row prepass and
must preserve the forced function's row when lowering calls of the forced result.

## Requirements

1. `collect_letcall_function_rows` must traverse a `LetMode` body with the newly bound mode value in
   scope.
2. Lowering `CoreExpr::Force` must lower the force body with the forced result binding's inner
   function row in the lowering context when the thunk's inner type is a function.
3. The emitted CPS `Call.row` for calls through a forced function must include the checked function
   row.
4. The fix must not infer rows from CPS terms; it must use checked Core type metadata.
5. Existing Phase 163 mode lowering and typecheck tests must remain green.

## TDD Steps

1. Add failing tests in `crates/ash-core/tests/task_1669_core_mode_lowering.rs`.
2. Confirm the local `LetMode` force/call case fails in the checked-lowering prepass.
3. Confirm the externally supplied forced-function case emits an empty CPS `Call.row`.
4. Implement the minimum checked metadata propagation in `core_ash_typecheck.rs` and
   `core_ash_lower.rs`.
5. Re-run focused and affected crate tests.

## Completion Checklist

- [x] Local `LetMode` bindings are scoped in the let-call row prepass.
- [x] Forced function rows are preserved in lowered CPS call rows.
- [x] Focused regression tests pass.
- [x] `cargo test -p ash-core` passes.
- [x] `cargo fmt --check` passes.
- [x] `cargo clippy -p ash-core --all-targets` passes.
- [x] `CHANGELOG.md` records the remediation.
