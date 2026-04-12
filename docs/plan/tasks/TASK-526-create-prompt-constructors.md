# TASK-526: Create std/src/llm/prompt.ash -- Constructors

## Status: ✅ Complete

## Description

Implement the pure constructor functions that build `Message` values: `system()`, `user()`, `assistant()`, `tool_result()`. These are Tier 1 pure `fn` definitions -- no `act`, no `ret`, no workflow constructs.

## Specification Reference

- [DESIGN-025: LLM Standard Library](../../design/DESIGN-025-LLM-STDLIB.md) (D2: Three-Tier Layering)
- [SPEC-029: LLM Standard Library](../../spec/SPEC-029-LLM-STDLIB.md) (SS4.1: Constructors)
- [PLAN-025: LLM Standard Library](../PLAN-025-LLM-STDLIB.md)

## Dependencies

- [TASK-525](TASK-525-create-llm-types.md)

## Requirements

1. `fn system(content: String) -> Message` -- role=System, tool_calls=None, tool_call_id=None.
2. `fn user(content: String) -> Message` -- role=User, tool_calls=None, tool_call_id=None.
3. `fn assistant(content: String) -> Message` -- role=Assistant, tool_calls=None, tool_call_id=None.
4. `fn tool_result(call_id: String, content: String) -> Message` -- role=Tool, tool_call_id=Some(call_id), tool_calls=None.
5. All are pure `fn` -- no workflow constructs.

## Guidance

These are simple record constructors. Each returns a `Message` with the appropriate role and content, with optional fields set to None.

## Likely Files

- Create: `std/src/llm/prompt.ash` (initial version with constructors)

## TDD Steps

### Red

1. Write test: `system("hello")` produces `Message { role: System, content: "hello", tool_calls: None, tool_call_id: None }`.
2. Write test: `user("question")` produces correct User message.
3. Write test: `assistant("reply")` produces correct Assistant message.
4. Write test: `tool_result("call_123", "result")` produces Tool message with call_id.

### Green

Implement all four constructors.

## Completion Checklist

- [ ] `system()` constructor implemented
- [ ] `user()` constructor implemented
- [ ] `assistant()` constructor implemented
- [ ] `tool_result()` constructor implemented
- [ ] All are pure `fn`
- [ ] Tests pass
