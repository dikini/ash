# TASK-536: Integration tests

## Status: ✅ Complete

## Description

End-to-end integration tests for the LLM stdlib: Rust provider + Ash workflow integration with mock servers.

## Specification Reference

- [DESIGN-025: LLM Standard Library](../../design/DESIGN-025-LLM-STDLIB.md)
- [SPEC-029: LLM Standard Library](../../spec/SPEC-029-LLM-STDLIB.md) (SS10: Conformance)
- [PLAN-025: LLM Standard Library](../PLAN-025-LLM-STDLIB.md)

## Dependencies

- [TASK-523](TASK-523-wire-engine-builder.md)
- [TASK-530](TASK-530-create-dispatch-workflows.md)
- [TASK-533](TASK-533-create-tool-agent-workflow.md)

## Requirements

1. End-to-end test: Rust LlmProvider + Ash workflow execution with mock server.
2. Multi-provider routing test: two providers configured, requests route correctly.
3. Streaming test: stream adapter produces correct chunks.
4. Tool-use loop test: tool_agent cycle with mock tool execution.
5. All integration tests use wiremock to mock the OpenAI API.

## Guidance

Use `wiremock` to mock `/v1/chat/completions` and `/v1/embeddings` endpoints. Tests verify the full path from Ash workflow -> capability dispatch -> Rust provider -> mock HTTP -> response -> Ash Value.

## Likely Files

- Create: `crates/ash-engine/tests/llm_integration.rs`

## TDD Steps

### Red

1. Write test: end-to-end chat completion with mock returns correct response.
2. Write test: multi-provider routes to correct api_base.
3. Write test: streaming produces ordered chunks.
4. Write test: tool-use loop completes in 2 rounds.

### Green

Implement all integration tests with mock servers.

## Completion Checklist

- [ ] End-to-end chat completion test passes
- [ ] Multi-provider routing test passes
- [ ] Streaming test passes
- [ ] Tool-use loop test passes
- [ ] All tests: `cargo test -p ash-engine --test llm_integration`
