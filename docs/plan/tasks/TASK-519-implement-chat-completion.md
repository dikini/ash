# TASK-519: Implement chat completion

## Status: Draft

## Description

Implement the `"chat"` and `"chat_with_tools"` actions that convert Ash `Value` arguments into `async-openai` requests and convert responses back to Ash `Value`. This is the core completion logic for the LLM provider.

## Specification Reference

- [DESIGN-025: LLM Standard Library](../../design/DESIGN-025-LLM-STDLIB.md) (D4: Rust Provider Layer)
- [SPEC-029: LLM Standard Library](../../spec/SPEC-029-LLM-STDLIB.md) (SS5.2.1: Chat action, SS9.4: Error mapping)
- [PLAN-025: LLM Standard Library](../PLAN-025-LLM-STDLIB.md)

## Dependencies

- [TASK-518](TASK-518-create-llm-provider-skeleton.md)

## Requirements

1. Handle both `"chat"` and `"chat_with_tools"` action names in the execute dispatch.
2. Message conversion: Ash `Message` values -> `async-openai` `ChatCompletionRequestMessage` types.
3. Request construction: Build `CreateChatCompletionRequest` from action args (provider, model, messages, params).
4. For `"chat_with_tools"`: additionally convert `ToolDef` values to `ChatCompletionTool` and include them in the request.
5. Response conversion: `CreateChatCompletionResponse` -> Ash `Value` with ChatResponse shape.
6. Error mapping: 401/403->auth, 404->model_not_found, 429->rate_limited, connection->network, 400->invalid_request, 5xx->server_error.
7. Provider routing by name (config lookup, client creation).
8. Unit tests for conversion functions; integration tests with mock server.

## Guidance

Use `async-openai` builder pattern (`CreateChatCompletionRequestArgs`) for request construction. For testing, use `wiremock` to mock the `/v1/chat/completions` endpoint.

## Likely Files

- Create: `crates/ash-engine/src/providers/llm/chat.rs`
- Modify: `crates/ash-engine/src/providers/llm/mod.rs` (wire chat action)

## TDD Steps

### Red

1. Write test: `value_to_chat_message` for System/User/Assistant/Tool roles.
2. Write test: response-to-Value conversion produces correct fields.
3. Write test: mock 401 response -> auth error.
4. Write test: mock 429 response -> rate_limited error.
5. Write test: mock successful response -> correct ChatResponse Value.
6. Write test: `"chat_with_tools"` action converts ToolDef values to ChatCompletionTool and includes them in request.

### Green

Implement message conversion, request building, response conversion, and error mapping.

## Completion Checklist

- [ ] Message conversion handles all four roles
- [ ] Request construction with provider routing
- [ ] Handles both "chat" and "chat_with_tools" action names
- [ ] chat_with_tools converts ToolDef to ChatCompletionTool
- [ ] Response conversion to Ash Value
- [ ] Error mapping for all HTTP error codes
- [ ] Tests pass: `cargo test -p ash-engine --lib providers::llm::chat`
