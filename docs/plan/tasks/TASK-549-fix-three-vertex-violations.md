# TASK-549: Fix three-vertex violations in orchestration modules

**Plan Reference:** PLAN-027 (LLM Stdlb Usability Remediation)
**Spec Reference:** SPEC-029 §8.3, §8.4; three-vertex model from SPEC-027, DESIGN-020
**Status:** Done
**Depends on:** TASK-546

## Description

Restructure `router.ash` and `supervised.ash` so pure functions (`fn`) don't call workflows.
The three-vertex model forbids `fn -> workflow` edges: fns are pure and can only call other fns;
workflows are effectful and can call both fns and workflows.

## Violations Found

### router.ash
- `fn classify_route` called `complete()` (a dispatch workflow) at line 58

### supervised.ash
- `fn request_approval` called `complete()` (a dispatch workflow) at line 56

## Fixes Applied

### router.ash
- **Removed**: `fn classify_route` (mixed pure + effectful)
- **Added**: `fn build_classify_message(user_message: String) -> Message` -- pure prompt construction
- **Added**: `fn parse_route(response: ChatResponse) -> RouteTarget` -- pure response parsing
- **Modified**: `workflow router` now calls `build_classify_message()`, invokes `complete()` directly (workflow -> workflow), then calls `parse_route()`

### supervised.ash
- **Removed**: `fn request_approval` (mixed pure + effectful)
- **Added**: `fn build_approval_message(messages: List<Message>, tool_calls: List<ToolCall>) -> Message` -- pure prompt construction
- **Added**: `fn parse_supervisor_response(response: ChatResponse) -> SupervisorDecision` -- pure response parsing
- **Modified**: `workflow supervised_agent` now calls `build_approval_message()`, invokes `complete()` directly (workflow -> workflow), then calls `parse_supervisor_response()`

## Unchanged

- `type RouteTarget`, `type SupervisorDecision` declarations
- `fn select_model` (router.ash) -- already pure
- `fn format_tool_calls_for_review`, `fn execute_tool_call`, `fn execute_tool_calls` (supervised.ash) -- already pure
- `workflow router` and `workflow supervised_agent` signatures and behavior

## TDD Steps

1. Red: Write test asserting no fn in router.ash or supervised.ash calls `complete`, `stream`, `embed`, or `act`.
2. Green: Restructure both files by splitting effectful fns into pure prompt construction + pure response parsing.
3. Verify: All tests pass. Three-vertex compliance tests pass.

## Files

- Modify: `std/src/llm/router.ash`
- Modify: `std/src/llm/supervised.ash`
- Modify: `crates/ash-engine/tests/llm_stdlib_tests.rs` (three-vertex compliance tests)

## Completion Checklist

- [x] `fn classify_route` split into `build_classify_message` + `parse_route`
- [x] `fn request_approval` split into `build_approval_message` + `parse_supervisor_response`
- [x] `complete()` calls moved into workflow bodies (workflow -> workflow allowed)
- [x] Three-vertex compliance tests added for router.ash and supervised.ash
- [x] All existing tests pass (0 failures)
- [x] Workflow signatures unchanged
- [x] Pure helper fns unchanged
