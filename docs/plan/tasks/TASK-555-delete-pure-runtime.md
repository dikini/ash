# TASK-555: Delete pure_runtime.rs

**Phase:** 80
**Spec:** SPEC-031 §11
**Depends on:** TASK-554, TASK-553
**Estimate:** 3 hours

## Description

Delete the duplicate interpreter (`pure_runtime.rs`, 476 lines) and all associated dispatch/inlining code. All execution goes through `ash-interp`.

## Requirements

### 1. Delete Files

- Delete `crates/ash-engine/src/pure_runtime.rs`
- Remove `mod pure_runtime;` from `crates/ash-engine/src/lib.rs`

### 2. Remove Dispatch Logic

From `crates/ash-engine/src/lib.rs`:
- Remove `should_execute_via_pure_runtime` function (~line 771)
- Remove `workflow_is_supported_by_pure_runtime` function
- Remove pure_runtime dispatch at lines ~577-582, ~649-654
- Remove `parse_program_with_functions` usage at ~349

### 3. Remove Inlining Code

From `crates/ash-engine/src/lib.rs`:
- Remove `inline_imported_calls_in_workflow_def`
- Remove `collect_local_inline_callables`

### 4. Remove Expr::Call Closure Fallback

Remove the temporary `Expr::Call` closure-lookup fallback added in TASK-552. After this task, `Expr::Call` handles built-ins only.

### 5. Verify Single Interpreter Path

All programs previously handled by `pure_runtime` must now execute correctly through `ash-interp`. Run the full test suite.

## TDD Steps

1. Verify all existing tests pass without `pure_runtime`
2. Run `cargo test --all` -- 0 failures
3. Run `cargo clippy --all` -- 0 warnings
4. Verify `pure_runtime` is not referenced anywhere

## Completion Checklist

- [ ] `pure_runtime.rs` deleted
- [ ] `mod pure_runtime` removed from `lib.rs`
- [ ] `should_execute_via_pure_runtime` removed
- [ ] `inline_imported_calls_in_workflow_def` removed
- [ ] `collect_local_inline_callables` removed
- [ ] `Expr::Call` closure fallback removed
- [ ] All existing tests pass through single interpreter path
- [ ] `grep -r pure_runtime crates/` returns nothing
- [ ] `cargo test --all` passes
- [ ] `cargo clippy` clean
- [ ] CHANGELOG.md updated
