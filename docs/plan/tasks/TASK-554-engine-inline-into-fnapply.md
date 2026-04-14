# TASK-554: Engine -- Inline Imported Callables into FnApply

**Phase:** 80
**Spec:** SPEC-031 §5.4, §9.3
**Depends on:** TASK-552
**Estimate:** 3 hours

## Description

Update the engine's `inline_imported_calls_in_workflow_def` to produce `FnApply` nodes instead of `Call` nodes for imported user functions. This is the bridge between module-level function imports and the new `FnApply` IR.

## Requirements

### 1. Inline into FnApply

Currently `inline_imported_calls_in_workflow_def` rewrites call sites by inlining the imported function body. After this task, imported function calls should produce `Expr::FnApply { func: Expr::Variable(qualified_name), args }` rather than `Expr::Call`.

### 2. Keep Inlining for Pure Runtime (Temporary)

During the transition (before TASK-555 deletes pure_runtime), the old inlining path must remain functional. Both paths coexist until Phase C cleanup.

## TDD Steps

1. Test: importing a `pub fn` and calling it produces `FnApply` in lowered IR
2. Test: existing import tests still pass (backward compatibility)
3. Verify `cargo test --all` passes

## Completion Checklist

- [ ] `inline_imported_calls_in_workflow_def` produces `FnApply`
- [ ] Existing import tests pass
- [ ] `cargo test --all` passes
- [ ] `cargo clippy` clean
- [ ] CHANGELOG.md updated
