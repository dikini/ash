# TASK-521: Implement tool dispatch helpers

## Status: Draft

## Description

Parse tool calls from chat responses and format tool results for follow-up requests, supporting the tool-use agent loop.

## Specification Reference

- [DESIGN-025: LLM Standard Library](../../design/DESIGN-025-LLM-STDLIB.md) (D6: Agent Orchestration)
- [SPEC-029: LLM Standard Library](../../spec/SPEC-029-LLM-STDLIB.md) (SS3: Types, SS8: Agent Workflows)
- [PLAN-025: LLM Standard Library](../PLAN-025-LLM-STDLIB.md)

## Dependencies

- [TASK-518](TASK-518-create-llm-provider-skeleton.md)

## Requirements

1. Extract tool calls from ChatResponse Value: `extract_tool_calls(response: &Value) -> Vec<ToolCallValue>`.
2. Format tool result messages: `format_tool_result_message(call_id: &str, content: &str) -> Value`.
3. Convert Ash ToolDef values to async-openai ChatCompletionTool format.
4. Tests for all conversion functions.

## Guidance

Tool calls are in `response.tool_calls` as a list of records with `id`, `name`, `arguments`. Tool result messages need `role: Tool`, `tool_call_id`, and `content`.

## Likely Files

- Create: `crates/ash-engine/src/providers/llm/tool_dispatch.rs`
- Modify: `crates/ash-engine/src/providers/llm/mod.rs` (re-export)

## TDD Steps

### Red

1. Write test: extract tool calls from response with tool_calls present.
2. Write test: extract tool calls from response with tool_calls = None returns empty.
3. Write test: format tool result message produces correct Value shape.
4. Write test: convert ToolDef values to OpenAI tool format.

### Green

Implement all helper functions.

## Completion Checklist

- [ ] Tool call extraction from response Value
- [ ] Tool result message formatting
- [ ] ToolDef to OpenAI tool conversion
- [ ] Tests pass: `cargo test -p ash-engine --lib providers::llm::tool_dispatch`
