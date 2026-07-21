# TASK-611: Local Helper Workflow Surface and Registration

## Status: Superseded by TASK-1971

## Description

Historical task record. This helper-workflow entry model was removed by
[TASK-1971](TASK-1971-residual-workflow-form-carriers.md) when target entries moved to ordinary
function metadata instead of workflow-form program carriers.

Extend the parser surface, program model, and engine registration so that ordinary source files can declare local helper workflows as real `Workflow::Call` targets. Currently the parser limits files to a single workflow definition, and the module loader can only import workflows whose body is a single `Ret` expression. This task adds multi-workflow file support with automatic callable registration.

## Requirements

1. Historically extended `Program` in `surface.rs` to carry helper workflow definitions alongside
   the entry workflow. This carrier is no longer present after TASK-1971.
2. Extend `parse_program_with_functions()` to parse `fn* workflow* workflow` — multiple named workflow definitions where the last is the entry point and preceding ones are helper workflows.
3. In the engine's `parse_workflow_source_with_imports()`, lower helper workflows and register them as callable targets via `RuntimeState::register_callable_workflow()` before execution begins.
4. Helper workflows bypass the `InlineCallable` single-`Ret` restriction — their full body is registered as a `Workflow`.
5. Add end-to-end engine tests demonstrating:
   - A source file with a helper workflow that performs multiple steps (let/act/ret) called from the main workflow.
   - Arity mismatch detection for helper workflow calls.
   - Unknown target detection when calling an unregistered workflow name.

## TDD Steps

1. Write a failing engine integration test: parse a multi-workflow source, register helpers, execute main workflow that calls a helper.
2. Historically extended `Program` with a helper workflow list.
3. Extend `parse_program_with_functions()` to parse multiple workflows.
4. Wire helper workflow registration in the engine.
5. Add arity-mismatch and unknown-target tests.
6. Verify `cargo test --workspace`, `cargo clippy`, `cargo fmt --check` all pass.

## Completion Checklist

- [x] Historical implementation carried helper workflows in `Program`
- [x] `parse_program_with_functions()` parses multiple named workflows
- [x] Engine registers helpers as callable targets before execution
- [x] Engine integration test: helper workflow with multi-step body
- [x] Engine integration test: arity mismatch
- [x] Engine integration test: unknown target
- [x] `cargo test --workspace` green
- [x] `cargo clippy --all-targets` clean
- [x] `cargo fmt --check` clean
- [x] CHANGELOG.md updated

## Related Documents

- `docs/plan/PLAN-091-SMALL-STEP-LIFTING-PRODUCTIONIZATION.md`
- `docs/design/DESIGN-027-SMALL-STEP-IR-COMPRESSION.md`
- `docs/spec/SPEC-001-IR.md`
- `docs/spec/SPEC-002-SURFACE.md`
