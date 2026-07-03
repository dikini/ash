# TASK-1858: Fix CPS frame-ordered dispatch

## Description

Make CPS raise dispatch search handler and provider frames in one innermost-to-outermost pass.

## Requirements

- Add tests first where an inner provider shadows an outer handler for the same operation.
- Preserve handler-over-handler innermost shadowing.
- Preserve provider dispatch for matching provider frames.

## Completion criteria

- [x] Tests fail before implementation and pass after.
- [x] `HandlerChain` exposes or uses a frame-ordered lookup.
- [x] `eval_raise` no longer checks all handlers before providers.

## Evidence

- RED: `cargo test -p ash-interp --test task_1858_1859_handler_provider_semantics` failed because an outer handler shadowed an inner provider. GREEN: `HandlerChain::find_operation_frame` now searches handler/provider frames in one innermost-to-outermost pass and `eval_raise` dispatches through that result.

## Depends on

- TASK-1856.
