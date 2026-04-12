# TASK-534: Create router workflow

## Status: Done

## Description

Create the `router` orchestration workflow that classifies task complexity and routes to an appropriate model.

## Specification Reference

- [DESIGN-025: LLM Standard Library](../../design/DESIGN-025-LLM-STDLIB.md) (D6: Agent Orchestration)
- [SPEC-029: LLM Standard Library](../../spec/SPEC-029-LLM-STDLIB.md) (SS8.3: router)
- [PLAN-025: LLM Standard Library](../PLAN-025-LLM-STDLIB.md)

## Dependencies

- [TASK-530](TASK-530-create-dispatch-workflows.md)

## Requirements

1. `workflow router(provider: String, messages: List<Message>) -> ChatResponse`.
2. Phase 77 uses the fixed classifier model `"gpt-4o-mini"` for the classification step.
3. First calls that classifier via `ask(provider, "gpt-4o-mini", classification_prompt)`.
4. Routes to appropriate model based on classification.
5. Calls `complete` with the selected model.

## Guidance

The classification step uses `ask` with a classification prompt so the router stays consistent with
the single-turn workflow surface and does not pass a bare prompt to `complete`, which expects
`List<Message>`. For Phase 77 the classifier itself is fixed to `gpt-4o-mini`. The model mapping is configurable but defaults to simple=gpt-4o-mini,
moderate=gpt-4o, complex=o1.

## Likely Files

- Modify: `std/src/llm/router.ash` (add router)

## TDD Steps

### Red

1. Write test: router parses without errors.
2. Write test: router selects model based on classification.

### Green

Implement the router workflow.

## Completion Checklist

- [ ] `router` workflow implemented
- [ ] Classification + model selection logic
- [ ] File parses without errors
