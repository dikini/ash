# TASK-525: Create std/src/llm/types.ash

## Status: ✅ Complete

## Description

Define all LLM data types as pure type definitions in `std/src/llm/types.ash`, per SPEC-029 SS3. These are provider-agnostic types shared across all LLM implementations.

## Specification Reference

- [DESIGN-025: LLM Standard Library](../../design/DESIGN-025-LLM-STDLIB.md) (D2: Three-Tier Layering)
- [SPEC-029: LLM Standard Library](../../spec/SPEC-029-LLM-STDLIB.md) (SS3: Types)
- [PLAN-025: LLM Standard Library](../PLAN-025-LLM-STDLIB.md)

## Dependencies

- [TASK-524](TASK-524-create-llm-module-structure.md)

## Requirements

1. Define `Role` enum: System | User | Assistant | Tool.
2. Define `Message` record: role, content, tool_calls, tool_call_id.
3. Define `ChatResponse` record: content, tool_calls, finish_reason, usage, model, id.
4. Define `ToolCall` record: id, name, arguments.
5. Define `ToolDef` record: name, description, parameters.
6. Define `Usage` record: prompt_tokens, completion_tokens, total_tokens.
7. Define `ChatChunk` record: delta_content, delta_tool_calls, finish_reason.
8. Define `ToolCallDelta` record: index, id, name, arguments.
9. Define `Embedding` record: index, embedding.
10. Define `ProviderConfig` record: name, api_base, api_key, default_model.
11. Define `CompletionParams` record: temperature, top_p, max_tokens, stop, seed.
12. All types are pure definitions with no effectful constructs.

## Guidance

Follow SPEC-029 SS3 invariants. Types use Ash record syntax. All are `pub` for cross-module use.

## Likely Files

- Create: `std/src/llm/types.ash`

## TDD Steps

### Red

1. Write test: file parses without errors.
2. Write test: each type is importable from `llm::types`.

### Green

Define all types per SPEC-029.

## Completion Checklist

- [ ] All 11 types defined per SPEC-029 SS3
- [ ] File parses without errors
- [ ] Types importable from `llm::types`
