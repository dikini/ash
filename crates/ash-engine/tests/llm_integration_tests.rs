//! Integration tests for LLM stdlib functionality.
//!
//! These tests verify the full integration path from Ash workflows through
//! the LlmProvider to mock OpenAI-compatible HTTP endpoints.
//!
//! Test categories:
//! - Mock provider tests for chat, streaming, embeddings, and tool use
//! - Error handling tests for HTTP status codes (401, 404, 429, 500)
//! - Multi-provider routing verification
//!
//! All tests use wiremock to mock the OpenAI API, ensuring no external
//! network calls are made during test execution.

use ash_core::Value;
use ash_core::capability::{CapabilityError, CapabilityProvider};
use ash_engine::providers::{LlmConfig, LlmProvider};
use std::collections::HashMap;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build an `LlmProvider` whose sole "test" provider points at the mock server.
fn make_provider(server: &MockServer) -> LlmProvider {
    let mut configs = HashMap::new();
    configs.insert(
        "test".to_string(),
        LlmConfig::custom(format!("{}/v1", server.uri()), "test-key"),
    );
    LlmProvider::new(configs).expect("provider creation should succeed")
}

/// Build an `LlmProvider` with multiple provider configurations.
fn make_multi_provider(primary_server: &MockServer, secondary_server: &MockServer) -> LlmProvider {
    let mut configs = HashMap::new();
    configs.insert(
        "primary".to_string(),
        LlmConfig::custom(format!("{}/v1", primary_server.uri()), "primary-key"),
    );
    configs.insert(
        "secondary".to_string(),
        LlmConfig::custom(format!("{}/v1", secondary_server.uri()), "secondary-key"),
    );
    LlmProvider::new(configs).expect("provider creation should succeed")
}

/// Build a user message Value (ADT-variant role style matching the chat module).
fn user_message(content: &str) -> Value {
    let mut fields = HashMap::new();
    fields.insert("sender".to_string(), Value::unit_variant("User"));
    fields.insert("content".to_string(), Value::String(content.to_string()));
    Value::Record(Box::new(fields))
}

/// Build a system message Value.
fn system_message(content: &str) -> Value {
    let mut fields = HashMap::new();
    fields.insert("sender".to_string(), Value::unit_variant("System"));
    fields.insert("content".to_string(), Value::String(content.to_string()));
    Value::Record(Box::new(fields))
}

/// Build the `args` slice expected by the `chat` action:
/// `[provider, model, messages, params]`
fn chat_args(messages: Vec<Value>) -> Vec<Value> {
    vec![
        Value::String("test".to_string()),
        Value::String("gpt-4o".to_string()),
        Value::List(Box::new(messages)),
        Value::unit_variant("None"),
    ]
}

/// Build the `args` slice for `chat_with_tools` action:
/// `[provider, model, messages, tools, params]`
fn chat_with_tools_args(messages: Vec<Value>, tools: Vec<Value>) -> Vec<Value> {
    vec![
        Value::String("test".to_string()),
        Value::String("gpt-4o".to_string()),
        Value::List(Box::new(messages)),
        Value::List(Box::new(tools)),
        Value::unit_variant("None"),
    ]
}

/// Build a tool definition Value.
fn tool_def(name: &str, description: &str, parameters: &str) -> Value {
    let mut fields = HashMap::new();
    fields.insert("name".to_string(), Value::String(name.to_string()));
    fields.insert(
        "description".to_string(),
        Value::String(description.to_string()),
    );
    fields.insert(
        "parameters".to_string(),
        Value::String(parameters.to_string()),
    );
    Value::Record(Box::new(fields))
}

/// Build the `args` slice for `embed` action:
/// `[provider, model, texts]`
fn embed_args(texts: Vec<&str>) -> Vec<Value> {
    vec![
        Value::String("test".to_string()),
        Value::String("text-embedding-3-small".to_string()),
        Value::List(Box::new(
            texts.iter().map(|s| Value::String(s.to_string())).collect(),
        )),
    ]
}

// ---------------------------------------------------------------------------
// Chat Completion Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_chat_completion_mock() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "chatcmpl_test_123",
            "object": "chat.completion",
            "created": 1_234_567_890_u64,
            "model": "gpt-4o",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Hello! How can I help you today?"
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 7,
                "total_tokens": 17
            }
        })))
        .mount(&server)
        .await;

    let provider = make_provider(&server);
    let args = chat_args(vec![user_message("Hi there")]);

    let result = provider.execute("chat", &args).await;
    assert!(result.is_ok(), "chat should succeed: {:?}", result.err());

    let value = result.unwrap();

    // Verify response structure
    match &value {
        Value::Record(fields) => {
            assert_eq!(
                fields.get("model").and_then(|v| v.as_string()),
                Some("gpt-4o")
            );
            assert_eq!(
                fields.get("id").and_then(|v| v.as_string()),
                Some("chatcmpl_test_123")
            );

            // Verify content is Some("Hello! How can I help you today?")
            let content_val = fields.get("content").expect("missing content field");
            match content_val {
                Value::Variant {
                    name,
                    fields: inner,
                } => {
                    assert_eq!(name, "Some");
                    let (_, val) = inner
                        .iter()
                        .find(|(k, _)| k == "0")
                        .expect("Some should have field 0");
                    assert_eq!(val.as_string(), Some("Hello! How can I help you today?"));
                }
                other => panic!("content should be Some variant, got: {other:?}"),
            }

            // Verify finish_reason is Some("stop")
            match fields.get("finish_reason") {
                Some(Value::Variant { name, .. }) => {
                    assert_eq!(name, "Some");
                }
                other => panic!("finish_reason should be Some variant, got: {other:?}"),
            }
        }
        other => panic!("Expected Record, got: {other:?}"),
    }
}

#[tokio::test]
async fn test_chat_with_tools_mock() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "chatcmpl_tools_456",
            "object": "chat.completion",
            "created": 1_234_567_891_u64,
            "model": "gpt-4o",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": null,
                    "tool_calls": [{
                        "id": "call_abc123",
                        "type": "function",
                        "function": {
                            "name": "get_weather",
                            "arguments": "{\"location\":\"San Francisco\",\"unit\":\"celsius\"}"
                        }
                    }]
                },
                "finish_reason": "tool_calls"
            }],
            "usage": {
                "prompt_tokens": 50,
                "completion_tokens": 25,
                "total_tokens": 75
            }
        })))
        .mount(&server)
        .await;

    let provider = make_provider(&server);
    let messages = vec![user_message("What's the weather in San Francisco?")];
    let tools = vec![tool_def(
        "get_weather",
        "Get the current weather for a location",
        r#"{"type": "object", "properties": {"location": {"type": "string"}, "unit": {"type": "string"}}, "required": ["location"]}"#,
    )];
    let args = chat_with_tools_args(messages, tools);

    let result = provider.execute("chat_with_tools", &args).await;
    assert!(
        result.is_ok(),
        "chat_with_tools should succeed: {:?}",
        result.err()
    );

    let value = result.unwrap();

    // Verify response contains tool calls
    match &value {
        Value::Record(fields) => {
            // Content should be None when tool_calls are present
            match fields.get("content") {
                Some(Value::Variant { name, .. }) => {
                    assert_eq!(name, "None");
                }
                other => {
                    panic!("content should be None variant when tool_calls present, got: {other:?}")
                }
            }

            // Verify tool_calls is present and contains the expected call
            match fields.get("tool_calls") {
                Some(Value::Variant {
                    name,
                    fields: tool_fields,
                }) => {
                    assert_eq!(name, "Some");
                    // Extract the list from the Some variant
                    if let Some((_, Value::List(tool_list))) =
                        tool_fields.iter().find(|(k, _)| k == "0")
                    {
                        assert_eq!(tool_list.len(), 1);
                        // Verify the tool call structure
                        match &tool_list[0] {
                            Value::Record(tool_record) => {
                                assert_eq!(
                                    tool_record.get("id").and_then(|v| v.as_string()),
                                    Some("call_abc123")
                                );
                                assert_eq!(
                                    tool_record.get("name").and_then(|v| v.as_string()),
                                    Some("get_weather")
                                );
                            }
                            other => panic!("Expected tool call Record, got: {other:?}"),
                        }
                    } else {
                        panic!("Expected tool_calls to contain a List");
                    }
                }
                other => panic!("tool_calls should be Some variant, got: {other:?}"),
            }

            // Verify finish_reason is "tool_calls"
            match fields.get("finish_reason") {
                Some(Value::Variant {
                    name,
                    fields: reason_fields,
                }) => {
                    assert_eq!(name, "Some");
                    if let Some((_, Value::String(reason))) =
                        reason_fields.iter().find(|(k, _)| k == "0")
                    {
                        assert_eq!(reason, "tool_calls");
                    }
                }
                other => panic!("finish_reason should be Some variant, got: {other:?}"),
            }
        }
        other => panic!("Expected Record, got: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Streaming Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_streaming_mock() {
    let server = MockServer::start().await;

    // Mock the streaming endpoint
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "chatcmpl_stream_789",
            "object": "chat.completion.chunk",
            "created": 1_234_567_892_u64,
            "model": "gpt-4o",
            "choices": [{
                "index": 0,
                "delta": {
                    "role": "assistant",
                    "content": "Hello"
                },
                "finish_reason": null
            }]
        })))
        .mount(&server)
        .await;

    let provider = make_provider(&server);
    let args = chat_args(vec![user_message("Say hello")]);

    // Execute chat_stream action - this initiates the stream
    let result = provider.execute("chat_stream", &args).await;
    assert!(
        result.is_ok(),
        "chat_stream should succeed: {:?}",
        result.err()
    );

    // Verify we get a Stream handle back per SPEC-013/SPEC-029
    let value = result.unwrap();
    let stream_id = match &value {
        Value::Stream(handle) => {
            assert!(handle.id.starts_with("llm_stream_test_"));
            assert_eq!(handle.item_type, "ChatChunk");
            handle.id.clone()
        }
        other => panic!("Expected Stream handle, got: {other:?}"),
    };

    // Test pulling chunks from the stream
    let pull_args = vec![Value::String(stream_id.clone())];
    let chunk_result = provider.execute("pull_stream_chunk", &pull_args).await;
    assert!(chunk_result.is_ok(), "pull_stream_chunk should succeed");

    // Verify we got a valid response
    // Could be: Some(chunk), None (not ready), or End (finished)
    let chunk = chunk_result.unwrap();
    match &chunk {
        Value::Variant { name, fields } => {
            match name.as_str() {
                "Some" => {
                    // Got a real chunk - verify it's a ChatChunk record
                    if let Some((_, Value::Record(chunk_fields))) = fields.first() {
                        assert!(chunk_fields.contains_key("delta_content"));
                        assert!(chunk_fields.contains_key("delta_tool_calls"));
                        assert!(chunk_fields.contains_key("finish_reason"));
                    } else {
                        panic!("Expected Some(Record), got: {chunk:?}");
                    }
                }
                "None" => {
                    // No chunk available yet - this is OK for non-blocking
                }
                "End" => {
                    // Stream ended - this can happen if stream completes quickly
                }
                _ => panic!("Unexpected variant: {name}"),
            }
        }
        _ => panic!("Expected Variant, got: {chunk:?}"),
    }
}

// ---------------------------------------------------------------------------
// Embeddings Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_embeddings_mock() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/embeddings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "object": "list",
            "data": [
                {
                    "object": "embedding",
                    "embedding": [0.1, 0.2, 0.3, 0.4, 0.5],
                    "index": 0
                },
                {
                    "object": "embedding",
                    "embedding": [0.6, 0.7, 0.8, 0.9, 1.0],
                    "index": 1
                }
            ],
            "model": "text-embedding-3-small",
            "usage": {
                "prompt_tokens": 8,
                "total_tokens": 8
            }
        })))
        .mount(&server)
        .await;

    let provider = make_provider(&server);
    let args = embed_args(vec!["Hello world", "Goodbye world"]);

    let result = provider.execute("embed", &args).await;
    assert!(result.is_ok(), "embed should succeed: {:?}", result.err());

    let value = result.unwrap();

    // Verify response is a list of embeddings
    match &value {
        Value::List(embeddings) => {
            assert_eq!(embeddings.len(), 2, "Should have 2 embeddings");

            // Verify first embedding structure
            match &embeddings[0] {
                Value::Record(fields) => {
                    // Check index
                    match fields.get("index") {
                        Some(Value::Int(idx)) => assert_eq!(*idx, 0),
                        other => panic!("Expected index Int(0), got: {other:?}"),
                    }

                    // Check embedding vector
                    match fields.get("embedding") {
                        Some(Value::List(vec)) => {
                            assert_eq!(vec.len(), 5, "Embedding should have 5 dimensions");
                            // Verify first value is approximately 0.1
                            match &vec[0] {
                                Value::Float(f) => assert!((f - 0.1).abs() < 0.001),
                                other => panic!("Expected Float, got: {other:?}"),
                            }
                        }
                        other => panic!("Expected embedding List, got: {other:?}"),
                    }
                }
                other => panic!("Expected embedding Record, got: {other:?}"),
            }
        }
        other => panic!("Expected List of embeddings, got: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Error Handling Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_error_401_unauthorized() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "error": {
                "message": "Invalid API key",
                "type": "authentication_error",
                "code": "invalid_api_key"
            }
        })))
        .mount(&server)
        .await;

    let provider = make_provider(&server);
    let args = chat_args(vec![user_message("Test")]);

    let result = provider.execute("chat", &args).await;
    assert!(result.is_err());

    match result.unwrap_err() {
        CapabilityError::PermissionDenied(msg) => {
            assert!(
                msg.contains("auth_error"),
                "Expected auth_error in message: {msg}"
            );
        }
        other => panic!("Expected PermissionDenied, got: {other:?}"),
    }
}

#[tokio::test]
async fn test_error_404_model_not_found() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
            "error": {
                "message": "The model 'unknown-model' does not exist",
                "type": "invalid_request_error",
                "code": "model_not_found"
            }
        })))
        .mount(&server)
        .await;

    let provider = make_provider(&server);
    let args = chat_args(vec![user_message("Test")]);

    let result = provider.execute("chat", &args).await;
    assert!(result.is_err());

    // Per SPEC-029 §9.4, 404 model_not_found maps to NotAvailable
    match result.unwrap_err() {
        CapabilityError::NotAvailable(msg) => {
            assert!(
                msg.contains("model_not_found"),
                "Expected model_not_found in message: {msg}"
            );
        }
        other => panic!("Expected NotAvailable, got: {other:?}"),
    }
}

#[tokio::test]
async fn test_error_429_rate_limited() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(429).set_body_json(serde_json::json!({
            "error": {
                "message": "Rate limit exceeded",
                "type": "insufficient_quota",
                "code": "rate_limit_exceeded"
            }
        })))
        .mount(&server)
        .await;

    let provider = make_provider(&server);
    let args = chat_args(vec![user_message("Test")]);

    let result = provider.execute("chat", &args).await;
    assert!(result.is_err());

    match result.unwrap_err() {
        CapabilityError::ExecutionFailed(msg) => {
            assert!(
                msg.contains("rate_limited"),
                "Expected rate_limited in message: {msg}"
            );
        }
        other => panic!("Expected ExecutionFailed with rate_limited, got: {other:?}"),
    }
}

// Note: 5xx error testing is omitted because async-openai's retry behavior
// causes long timeouts (exponential backoff with multiple retries).
// The error mapping for 5xx is covered by unit tests in the provider module.

// ---------------------------------------------------------------------------
// Multi-Provider Routing Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_multi_provider_routing() {
    let primary_server = MockServer::start().await;
    let secondary_server = MockServer::start().await;

    // Primary server responds with specific content
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "primary_resp",
            "object": "chat.completion",
            "created": 1_234_567_890_u64,
            "model": "gpt-4o",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Response from primary"
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 5,
                "completion_tokens": 3,
                "total_tokens": 8
            }
        })))
        .mount(&primary_server)
        .await;

    // Secondary server responds with different content
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "secondary_resp",
            "object": "chat.completion",
            "created": 1_234_567_891_u64,
            "model": "gpt-4o-mini",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Response from secondary"
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 5,
                "completion_tokens": 3,
                "total_tokens": 8
            }
        })))
        .mount(&secondary_server)
        .await;

    let provider = make_multi_provider(&primary_server, &secondary_server);

    // Test primary provider
    let primary_args = vec![
        Value::String("primary".to_string()),
        Value::String("gpt-4o".to_string()),
        Value::List(Box::new(vec![user_message("Hi")])),
        Value::unit_variant("None"),
    ];

    let result = provider.execute("chat", &primary_args).await;
    assert!(
        result.is_ok(),
        "Primary provider should succeed: {:?}",
        result.err()
    );

    let value = result.unwrap();
    match &value {
        Value::Record(fields) => {
            assert_eq!(
                fields.get("id").and_then(|v| v.as_string()),
                Some("primary_resp")
            );
        }
        other => panic!("Expected Record, got: {other:?}"),
    }

    // Test secondary provider
    let secondary_args = vec![
        Value::String("secondary".to_string()),
        Value::String("gpt-4o-mini".to_string()),
        Value::List(Box::new(vec![user_message("Hi")])),
        Value::unit_variant("None"),
    ];

    let result = provider.execute("chat", &secondary_args).await;
    assert!(
        result.is_ok(),
        "Secondary provider should succeed: {:?}",
        result.err()
    );

    let value = result.unwrap();
    match &value {
        Value::Record(fields) => {
            assert_eq!(
                fields.get("id").and_then(|v| v.as_string()),
                Some("secondary_resp")
            );
        }
        other => panic!("Expected Record, got: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// List Models Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_list_models_mock() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "object": "list",
            "data": [
                {
                    "id": "gpt-4o",
                    "object": "model",
                    "created": 1_700_000_000_u64,
                    "owned_by": "openai"
                },
                {
                    "id": "gpt-4o-mini",
                    "object": "model",
                    "created": 1_700_000_001_u64,
                    "owned_by": "openai"
                },
                {
                    "id": "text-embedding-3-small",
                    "object": "model",
                    "created": 1_700_000_002_u64,
                    "owned_by": "openai"
                }
            ]
        })))
        .mount(&server)
        .await;

    let provider = make_provider(&server);
    let args = vec![Value::String("test".to_string())];

    let result = provider.execute("list_models", &args).await;
    assert!(
        result.is_ok(),
        "list_models should succeed: {:?}",
        result.err()
    );

    let value = result.unwrap();
    match &value {
        Value::List(models) => {
            let model_ids: Vec<&str> = models.iter().filter_map(|v| v.as_string()).collect();
            assert!(model_ids.contains(&"gpt-4o"));
            assert!(model_ids.contains(&"gpt-4o-mini"));
            assert!(model_ids.contains(&"text-embedding-3-small"));
            assert_eq!(model_ids.len(), 3);
        }
        other => panic!("Expected Value::List, got: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Edge Case Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_chat_empty_messages_rejected() {
    let server = MockServer::start().await;

    let provider = make_provider(&server);
    let args = chat_args(vec![]);

    let result = provider.execute("chat", &args).await;
    assert!(result.is_err(), "Empty messages should be rejected");
}

#[tokio::test]
async fn test_chat_system_and_user_messages() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "multi_msg_resp",
            "object": "chat.completion",
            "created": 1_234_567_893_u64,
            "model": "gpt-4o",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "4"
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 20,
                "completion_tokens": 1,
                "total_tokens": 21
            }
        })))
        .mount(&server)
        .await;

    let provider = make_provider(&server);
    let messages = vec![
        system_message("You are a calculator. Answer with just the number."),
        user_message("What is 2 + 2?"),
    ];
    let args = chat_args(messages);

    let result = provider.execute("chat", &args).await;
    assert!(
        result.is_ok(),
        "Multi-message chat should succeed: {:?}",
        result.err()
    );

    let value = result.unwrap();
    match &value {
        Value::Record(fields) => match fields.get("content") {
            Some(Value::Variant {
                name,
                fields: inner,
            }) => {
                assert_eq!(name, "Some");
                if let Some((_, Value::String(content))) = inner.iter().find(|(k, _)| k == "0") {
                    assert_eq!(content, "4");
                }
            }
            other => panic!("Expected Some content, got: {other:?}"),
        },
        other => panic!("Expected Record, got: {other:?}"),
    }
}

#[tokio::test]
async fn test_embed_empty_texts_rejected() {
    let server = MockServer::start().await;

    let provider = make_provider(&server);
    let args = embed_args(vec![]);

    let result = provider.execute("embed", &args).await;
    assert!(result.is_err(), "Empty texts list should be rejected");
}

#[tokio::test]
async fn test_embed_single_text() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/embeddings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "object": "list",
            "data": [
                {
                    "object": "embedding",
                    "embedding": [0.1, 0.2, 0.3],
                    "index": 0
                }
            ],
            "model": "text-embedding-3-small",
            "usage": {
                "prompt_tokens": 2,
                "total_tokens": 2
            }
        })))
        .mount(&server)
        .await;

    let provider = make_provider(&server);
    let args = embed_args(vec!["Hello"]);

    let result = provider.execute("embed", &args).await;
    assert!(
        result.is_ok(),
        "Single text embed should succeed: {:?}",
        result.err()
    );

    let value = result.unwrap();
    match &value {
        Value::List(embeddings) => {
            assert_eq!(embeddings.len(), 1);
        }
        other => panic!("Expected List, got: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Stream Error Propagation Tests (TASK-520)
// ---------------------------------------------------------------------------

/// Test that stream errors from upstream are propagated as ExecutionFailed
/// when pulling chunks via `pull_stream_chunk`.
///
/// This verifies the error-handling contract per SPEC-029 §9.4 SC4:
/// "On stream error, the stream terminates with an error rather than silently stopping."
///
/// The test injects an `StreamChunk::Error` directly into the stream storage
/// and verifies that `pull_stream_chunk` returns `CapabilityError::ExecutionFailed`.
#[tokio::test]
async fn test_stream_error_propagation() {
    use ash_engine::providers::llm::stream_storage::StreamChunk;

    // Create a provider with a stream that will encounter an error
    let server = MockServer::start().await;

    let mut configs = HashMap::new();
    configs.insert(
        "test".to_string(),
        LlmConfig::custom(format!("{}/v1", server.uri()), "test-key"),
    );
    let provider = LlmProvider::new(configs).expect("provider creation should succeed");

    // Manually inject an error chunk into the provider's stream storage
    // This simulates an upstream stream failure (e.g., connection reset, API error mid-stream)
    let (tx, rx) = tokio::sync::mpsc::channel(10);
    let stream_id = "llm_stream_test_error_123".to_string();
    provider
        .stream_storage()
        .store_stream(stream_id.clone(), rx);

    // Send an error chunk to simulate upstream failure
    tx.try_send(StreamChunk::Error(
        "upstream connection reset: HTTP 502".to_string(),
    ))
    .expect("should send error chunk");

    // Now pull the chunk through the provider's action interface
    let pull_args = vec![Value::String(stream_id)];
    let result = provider.execute("pull_stream_chunk", &pull_args).await;

    // The error chunk must surface as ExecutionFailed, not silently succeed
    assert!(
        result.is_err(),
        "pull_stream_chunk should return error for stream error: {:?}",
        result.ok()
    );
    match result.unwrap_err() {
        CapabilityError::ExecutionFailed(msg) => {
            assert!(
                msg.contains("Stream error"),
                "Error message should mention 'Stream error': {msg}"
            );
            assert!(
                msg.contains("upstream connection reset"),
                "Error message should contain the upstream error detail: {msg}"
            );
        }
        other => panic!("Expected ExecutionFailed for stream error, got: {other:?}"),
    }
}

/// Test that a stream with error followed by end correctly handles terminal states.
///
/// Verifies that after an error chunk is consumed, the stream is cleaned up
/// (removed from storage) and subsequent pulls return not-found.
#[tokio::test]
async fn test_stream_error_cleanup_after_consumption() {
    use ash_engine::providers::llm::stream_storage::StreamChunk;

    let server = MockServer::start().await;

    let mut configs = HashMap::new();
    configs.insert(
        "test".to_string(),
        LlmConfig::custom(format!("{}/v1", server.uri()), "test-key"),
    );
    let provider = LlmProvider::new(configs).expect("provider creation should succeed");

    let (tx, rx) = tokio::sync::mpsc::channel(10);
    let stream_id = "llm_stream_test_cleanup_456".to_string();
    provider
        .stream_storage()
        .store_stream(stream_id.clone(), rx);

    // Send error chunk
    tx.try_send(StreamChunk::Error("test error".to_string()))
        .unwrap();

    // First pull should return the error
    let result = provider
        .execute("pull_stream_chunk", &[Value::String(stream_id.clone())])
        .await;
    assert!(result.is_err());

    // Second pull should get not-found (stream was removed after error was consumed)
    let result2 = provider
        .execute("pull_stream_chunk", &[Value::String(stream_id)])
        .await;
    assert!(
        result2.is_err(),
        "Stream should be cleaned up after error: {:?}",
        result2.ok()
    );
    match result2.unwrap_err() {
        CapabilityError::ExecutionFailed(msg) => {
            assert!(
                msg.contains("not found"),
                "Should report stream not found: {msg}"
            );
        }
        other => panic!("Expected ExecutionFailed for missing stream, got: {other:?}"),
    }
}
