# TASK-530: Create dispatch workflows

## Status: Done

## Description

Create the thin dispatch workflows that wrap `act` calls to the `Llm` capability: `complete`, `complete_with_tools`, `complete_tuned`, `ask`, `stream`, `embed`, `list_models`. All are workflows (not fn) because they use `act`.

## Specification Reference

- [DESIGN-025: LLM Standard Library](../../design/DESIGN-025-LLM-STDLIB.md) (D2: Three-Tier Layering, D3: Capability Contract)
- [SPEC-029: LLM Standard Library](../../spec/SPEC-029-LLM-STDLIB.md) (SS6: Dispatch Workflows)
- [PLAN-025: LLM Standard Library](../PLAN-025-LLM-STDLIB.md)

## Dependencies

- [TASK-529](TASK-529-create-openai-capability.md)

## Requirements

1. `workflow complete(provider, model, messages, params) -> ChatResponse` -- single chat completion.
2. `workflow complete_with_tools(provider, model, messages, tools, params) -> ChatResponse` -- with tool definitions. Calls `act llm:chat_with_tools(provider, model, messages, tools, params)`.
3. `workflow complete_tuned(provider, model, messages, params) -> ChatResponse` -- with tuning parameters.
4. `workflow ask(provider, model, question) -> ChatResponse` -- convenience single-turn. Constructs `[user(question)]` only (per SPEC-029 SS6.4: `let messages = [user(question)]`).
5. `workflow stream(provider, model, messages, params) -> Stream<ChatChunk>` -- streaming completion.
6. `workflow embed(provider, model, texts) -> List<Embedding>` -- text embedding.
7. `workflow list_models(provider) -> List<String>` -- list available models.
8. All are workflows using `act` on Llm capability -- NOT fn.

## Guidance

These are thin wrappers. Each dispatches to the corresponding LLM provider action with `act llm:chat(...)`, `act llm:chat_with_tools(...)`, `act llm:chat_stream(...)`, `act llm:embed(...)`, or `act llm:list_models(...)`. The `ask` workflow constructs messages from the question string as `[user(question)]` only -- no system message is injected.

## Likely Files

- Modify: `std/src/llm/openai.ash` (add dispatch workflows)

## TDD Steps

### Red

1. Write test: `complete` dispatches `act llm:chat(...)` with correct args.
2. Write test: `complete_with_tools` dispatches `act llm:chat_with_tools(...)` with correct args including tools.
3. Write test: `ask` constructs `[user(question)]` only before dispatching (no system message).
4. Write test: `stream` returns Stream<ChatChunk>.
5. Write test: `embed` dispatches with correct params.

### Green

Implement all seven dispatch workflows.

## Completion Checklist

- [ ] All seven dispatch workflows implemented as `workflow`
- [ ] None uses `fn` -- all use `act`
- [ ] `complete_with_tools` calls `act llm:chat_with_tools(...)`
- [ ] `ask` constructs `[user(question)]` only per SPEC-029 SS6.4
- [ ] File parses without errors
