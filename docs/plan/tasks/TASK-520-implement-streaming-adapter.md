# TASK-520: Implement streaming adapter

## Status: ✅ Complete

## Description

Adapt `async-openai`'s `Stream<CreateChatCompletionStreamResponse>` into Ash's stream model, with chunk parsing for delta content, tool call deltas, and finish reason.

## Specification Reference

- [DESIGN-025: LLM Standard Library](../../design/DESIGN-025-LLM-STDLIB.md) (D4: Rust Provider Layer)
- [SPEC-029: LLM Standard Library](../../spec/SPEC-029-LLM-STDLIB.md) (SS5.2.2: chat_stream action, SS9.5: Streaming contract)
- [PLAN-025: LLM Standard Library](../PLAN-025-LLM-STDLIB.md)

## Dependencies

- [TASK-518](TASK-518-create-llm-provider-skeleton.md)

## Requirements

1. Convert each SSE chunk to Ash `Value` with ChatChunk fields.
2. Filter empty keep-alive chunks (no delta content and no delta tool calls).
3. Final chunk with finish_reason produces ChatChunk with finish_reason set.
4. Stream errors terminate the stream with an error, not silent stop.
5. No buffering -- chunks yielded as they arrive.
6. Tests for chunk conversion, filtering, and error handling.

## Guidance

Use `futures::StreamExt` to iterate over `async-openai`'s stream. Each chunk converts to a `Value::Record` with `delta_content`, `delta_tool_calls`, `finish_reason`.

## Likely Files

- Create: `crates/ash-engine/src/providers/llm/stream_adapter.rs`
- Modify: `crates/ash-engine/src/providers/llm/mod.rs` (wire chat_stream action)

## TDD Steps

### Red

1. Write test: single chunk with delta content converts correctly.
2. Write test: chunk with tool call delta converts correctly.
3. Write test: final chunk with finish_reason converts correctly.
4. Write test: empty keep-alive chunk is filtered out.

### Green

Implement the stream adapter with chunk conversion and filtering.

## Completion Checklist

- [ ] Chunk conversion produces correct Value shape
- [ ] Keep-alive chunks filtered
- [ ] Final chunk with finish_reason handled
- [ ] Stream errors propagated, not swallowed
- [ ] Tests pass: `cargo test -p ash-engine --lib providers::llm::stream_adapter`
