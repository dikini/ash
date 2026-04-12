# TASK-527: Create std/src/llm/prompt.ash -- Inspectors

## Status: ✅ Complete

## Description

Implement the pure inspector functions that examine `ChatResponse` and `Message` values: `append_response()`, `append_tool_result()`, `has_tool_calls()`, `is_final()`, `get_tool_calls()`. These are Tier 1 pure `fn` definitions.

## Specification Reference

- [DESIGN-025: LLM Standard Library](../../design/DESIGN-025-LLM-STDLIB.md) (D2: Three-Tier Layering)
- [SPEC-029: LLM Standard Library](../../spec/SPEC-029-LLM-STDLIB.md) (SS4.2: Inspectors)
- [PLAN-025: LLM Standard Library](../PLAN-025-LLM-STDLIB.md)

## Dependencies

- [TASK-526](TASK-526-create-prompt-constructors.md)

## Requirements

1. `fn append_response(messages: List<Message>, response: ChatResponse) -> List<Message` -- appends assistant message from response.
2. `fn append_tool_result(messages: List<Message>, call_id: String, content: String) -> List<Message>` -- appends tool result message.
3. `fn has_tool_calls(response: ChatResponse) -> Bool` -- true iff tool_calls is Some.
4. `fn is_final(response: ChatResponse) -> Bool` -- true iff finish_reason is "stop" or "length".
5. `fn get_tool_calls(response: ChatResponse) -> List<ToolCall>` -- extracts calls or returns empty list.
6. All are pure `fn`.

## Guidance

`append_response` constructs an assistant Message from the response's content and tool_calls fields. `is_final` must distinguish "stop"/"length" from "tool_calls" -- the latter means the model wants to call tools, not that it's done.

## Likely Files

- Modify: `std/src/llm/prompt.ash` (add inspectors section)

## TDD Steps

### Red

1. Write test: `has_tool_calls` with tool calls present returns true.
2. Write test: `has_tool_calls` with tool_calls=None returns false.
3. Write test: `is_final` with "stop" and "length" returns true; with "tool_calls" returns false.
4. Write test: `get_tool_calls` extracts calls; empty when None.
5. Write test: `append_response` appends correctly for text-only and tool-call responses.
6. Write test: `append_tool_result` appends a tool message with correct call_id.

### Green

Implement all five inspector functions.

## Completion Checklist

- [ ] All five inspector functions implemented
- [ ] All are pure `fn`
- [ ] Tests pass
