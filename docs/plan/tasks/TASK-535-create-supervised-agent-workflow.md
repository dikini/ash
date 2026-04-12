# TASK-535: Create supervised_agent workflow

## Status: Done

## Description

Create the `supervised_agent` orchestration workflow implementing the spawn/kill/restart supervision pattern. Returns `Result<ChatResponse, AgentError>` — Ok with the child's response on normal completion, Err with restart count and optional last response on failure.

## Specification Reference

- [DESIGN-025: LLM Standard Library](../../design/DESIGN-025-LLM-STDLIB.md) (D6: Agent Orchestration)
- [SPEC-029: LLM Standard Library](../../spec/SPEC-029-LLM-STDLIB.md) (SS8.4: supervised_agent)
- [PLAN-025: LLM Standard Library](../PLAN-025-LLM-STDLIB.md)

## Dependencies

- [TASK-533](TASK-533-create-tool-agent-workflow.md)

## Requirements

1. `workflow supervised_agent(config: AgentConfig) -> Result<ChatResponse, AgentError>`.
2. Spawns a tool_agent as a child workflow.
3. The child `tool_agent` sends its terminal `ChatResponse` to the supervisor mailbox before exit.
4. Monitor loop: `check_health`, `receive` child results, or handle failures.
5. On failure: kill child, spawn replacement (up to max_restarts).
6. On normal child completion: `receive` the mailbox payload and return `Ok(received_response)`.
7. On max_restarts exceeded: return `Err(AgentError { max_restarts_exceeded, last_response })`.
8. Uses `spawn`, `kill`, `check_health`, `receive`.
9. `AgentError` type: `{ max_restarts_exceeded: Int, last_response: Option<ChatResponse> }`.

## Guidance

This follows Erlang/OTP supervisor patterns. The supervisor monitors the child and restarts it on failure. The child is a `tool_agent` workflow. On successful completion, the child sends its final `ChatResponse` to the supervisor mailbox; the supervisor then `receive`s that payload and returns it as `Ok(...)`. The return type is `Result<ChatResponse, AgentError>` so callers can pattern-match on success or supervised failure.

## Likely Files

- Modify: `std/src/llm/supervised.ash` (add supervised_agent)

## TDD Steps

### Red

1. Write test: supervised_agent parses without errors.
2. Write test: supervisor returns Ok(ChatResponse) when child completes normally.
3. Write test: supervisor returns Err(AgentError) when max_restarts exceeded.
4. Write test: supervisor respawns child after failure when restarts < max_restarts.

### Green

Implement the supervised_agent workflow.

## Completion Checklist

- [ ] `supervised_agent` workflow implemented
- [ ] Return type is `Result<ChatResponse, AgentError>`
- [ ] spawn/kill/restart pattern with max_restarts
- [ ] Ok(child response) on normal completion
- [ ] Err(AgentError) on max_restarts exceeded
- [ ] File parses without errors
