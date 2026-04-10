# TASK-489: Interpreter Act Continuation Execution

## Status: Done

## Description

Update the interpreter's `Workflow::Act` execution branch in `ash-interp/src/execute.rs` to bind
`result_name` into the execution context and execute the `continuation` after the action completes.

## Specification Reference

- [DESIGN-019](../../design/DESIGN-019-ACTION-RESULT-BINDING.md)
- [PLAN-019](../PLAN-019-ACTION-RESULT-BINDING.md)
- [SPEC-004](../../spec/SPEC-004-SEMANTICS.md)

## Dependencies

- [TASK-486](TASK-486-core-act-continuation-shape.md) — core Act must have new fields
- [TASK-488](TASK-488-parser-act-then-as.md) — new forms must parse and lower

## Requirements

1. In the `Workflow::Act` match arm of `execute_workflow_inner_observed`:
   - After `cap_ctx.execute(provider_name, action_name, &evaluated_args)` returns a `Value`:
   - If `result_name` is `Some(name)`, bind the result in `ctx` (same as `Let` binding).
   - Execute the `continuation` with the (possibly extended) context.
   - Return the continuation's result.
   - If `continuation` is `Done`, return the action result directly (existing behavior).
2. The guard evaluation and argument evaluation logic is unchanged.
3. The provenance recording is unchanged.
4. Update any execution-recorder calls if needed for the continuation.
5. `cargo test -p ash-interp` passes.

## TDD Steps

### Red

- Write tests that construct `Workflow::Act` with `result_name` and `continuation` manually,
  execute them, and assert the result. These fail before implementation.

### Green

- Implement the continuation execution logic.
- Verify new tests pass and all existing tests still green.

### Refactor

- Extract the bind-and-continue logic into a helper if it parallels `Let` execution closely.

## Completion Checklist

- [ ] Interpreter binds `result_name` after action execution
- [ ] Interpreter executes continuation after binding
- [ ] Returns continuation result (or action result if Done)
- [ ] Existing bare-act tests pass unchanged
- [ ] New interpreter tests for continuation
- [ ] `cargo test -p ash-interp` passes
- [ ] `cargo clippy` clean
- [ ] CHANGELOG.md entry added
