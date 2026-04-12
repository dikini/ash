# TASK-529: Create std/src/llm/openai/ module and capability declaration

## Status: Draft

## Description

Create the OpenAI-specific module with the `Llm` capability declaration and five actions: `chat`, `chat_with_tools`, `chat_stream`, `embed`, `list_models`.

## Specification Reference

- [DESIGN-025: LLM Standard Library](../../design/DESIGN-025-LLM-STDLIB.md) (D3: Capability Contract)
- [SPEC-029: LLM Standard Library](../../spec/SPEC-029-LLM-STDLIB.md) (SS5: Capability Contract)
- [PLAN-025: LLM Standard Library](../PLAN-025-LLM-STDLIB.md)

## Dependencies

- [TASK-525](TASK-525-create-llm-types.md)

## Requirements

1. Create `std/src/llm/openai/mod.ash` with `capability Llm` declaration.
2. Actions: `execute chat(...)`, `execute chat_with_tools(...)` (with separate `tools: List<ToolDef>` parameter), `execute chat_stream(...)`, `execute embed(...)`, `execute list_models(...)`.
3. Each action has typed parameters per SPEC-029 SS5.
4. The `chat_with_tools` action carries tools as a first-class parameter, not embedded in CompletionParams.
5. File parses without errors.

## Guidance

Follow the capability declaration pattern from `std/src/io/stdio.ash` or `std/src/io/fs.ash`. The `chat_with_tools` action is a separate action from `chat` because tool definitions are passed as a distinct parameter at the provider boundary.

## Likely Files

- Create: `std/src/llm/openai/mod.ash`

## TDD Steps

### Red

1. Write test: file parses without errors.
2. Write test: capability name is "Llm" with five actions (chat, chat_with_tools, chat_stream, embed, list_models).

### Green

Create the capability declaration.

## Completion Checklist

- [ ] `std/src/llm/openai/mod.ash` created
- [ ] `Llm` capability with five actions declared (chat, chat_with_tools, chat_stream, embed, list_models)
- [ ] `chat_with_tools` has `tools: List<ToolDef>` as separate parameter
- [ ] File parses without errors
