# TASK-533: Create tool_agent workflow

## Status: Done

## Description

Create the `tool_agent` orchestration workflow implementing the orient-decide-act cycle for tool-use agent loops. In Phase 77, tool execution is routed through a statically declared dispatcher helper that matches known tool names to explicit capability actions; there is no generic runtime capability lookup by string.

## Specification Reference

- [DESIGN-025: LLM Standard Library](../../design/DESIGN-025-LLM-STDLIB.md) (D6: Agent Orchestration)
- [SPEC-029: LLM Standard Library](../../spec/SPEC-029-LLM-STDLIB.md) (§8.2: tool_agent)
- [PLAN-025: LLM Standard Library](../PLAN-025-LLM-STDLIB.md)

## Dependencies

- [TASK-530](TASK-530-create-dispatch-workflows.md)
- [TASK-521](TASK-521-implement-tool-dispatch-helpers.md)

## Requirements

1. `workflow tool_agent(provider: String, model: String, messages: List<Message>, tools: List<ToolDef>, max_rounds: Int) -> ChatResponse`.
2. Loop: call complete_with_tools, check if final (is_final), if not extract tool calls, execute tools, append results, repeat.
3. Exit on final response or max_rounds exceeded.
4. Uses `complete_with_tools`, `is_final`, `get_tool_calls`, `append_response`, `append_tool_result`.
5. Tool execution is expressed through a statically declared dispatcher helper/workflow. It matches each tool call's `name` against a fixed set of supported branches and each branch invokes an explicit named `act` target.
6. If no branch matches a tool name, append an error message as the tool result and continue the loop.

## Guidance

Per SPEC-029 §8.2, tool execution is real capability dispatch, not stubbed, but it must fit Ash's current statically named action model. Implement a companion dispatcher helper/workflow that pattern-matches on supported tool names and lowers each supported case to an explicit capability action such as `SomeProvider:some_action(...)`. Do not attempt runtime capability lookup from `ToolCall.name`.

On tool execution failure inside a matched branch, append a tool result with an error message and continue the loop (per SPEC-029 §8.2 error handling). If a tool name has no supported branch, append an unknown-tool error result and continue.

## Likely Files

- Modify: `std/src/llm/tool_agent.ash` (add tool_agent)

## TDD Steps

### Red

1. Write test: tool_agent parses without errors.
2. Write test: tool_agent exits on final response (no tool calls).
3. Write test: tool_agent respects max_rounds.
4. Write test: tool_agent dispatches supported tool calls through the static dispatcher helper.
5. Write test: tool_agent returns error message as tool result when no matching static branch exists.
6. Write test: tool_agent continues loop after tool execution failure.

### Green

Implement the tool_agent workflow.

## Completion Checklist

- [ ] `tool_agent` workflow implemented
- [ ] Orient-decide-act cycle with tool calls
- [ ] Tool execution dispatched through a static helper/workflow with explicit named `act` targets
- [ ] Error message returned as tool result when no matching static branch exists
- [ ] Loop continues after tool execution failure
- [ ] max_rounds termination
- [ ] File parses without errors
