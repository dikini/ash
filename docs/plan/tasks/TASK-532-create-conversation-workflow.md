# TASK-532: Create conversation workflow

## Status: Draft

## Description

Create the `conversation` orchestration workflow in `std/src/llm/conversation.ash` that manages a multi-turn conversation loop and returns the accumulated message history.

## Specification Reference

- [DESIGN-025: LLM Standard Library](../../design/DESIGN-025-LLM-STDLIB.md) (D6: Agent Orchestration)
- [SPEC-029: LLM Standard Library](../../spec/SPEC-029-LLM-STDLIB.md) (SS8.1: conversation)
- [PLAN-025: LLM Standard Library](../PLAN-025-LLM-STDLIB.md)

## Dependencies

- [TASK-530](TASK-530-create-dispatch-workflows.md)

## Requirements

1. `workflow conversation(provider: String, model: String, system_prompt: String, max_turns: Int) -> List<Message>`.
2. Initializes history with system message.
3. Loop: call `complete` with accumulated messages, append response to history, repeat.
4. Terminates on: final response, `max_turns` reached, or termination signal.
5. On `act` failure: returns accumulated messages collected so far.
6. Uses `complete` (dispatch workflow).

## Guidance

This is an Ash workflow using the orchestration vocabulary. Per SPEC-029 SS8.1, the return type is `List<Message>` (not a handle). Termination is controlled by `max_turns` or a termination signal -- not by matching a literal "exit" string.

## Likely Files

- Create: `std/src/llm/conversation.ash` (initial version)

## TDD Steps

### Red

1. Write test: conversation workflow parses without errors.
2. Write test: conversation terminates on max_turns reached.
3. Write test: conversation returns `List<Message>`.
4. Write test: conversation returns accumulated messages on act failure.

### Green

Implement the conversation workflow.

## Completion Checklist

- [ ] `conversation` workflow implemented with correct signature
- [ ] Returns `List<Message>` (not a handle)
- [ ] Terminates on final response, max_turns, or termination signal
- [ ] Returns accumulated messages on act failure
- [ ] Uses `complete` dispatch workflow
- [ ] File parses without errors
