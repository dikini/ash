//! Integration tests for LLM provider using wiremock mock server.
//!
//! These tests verify that `LlmProvider` correctly interacts with an OpenAI-compatible
//! HTTP API, including successful responses and error code mapping to `CapabilityError`
//! variants per SPEC-029 §9.4.

use ash_core::Value;
use ash_core::capability::{CapabilityError, CapabilityProvider};
use ash_engine::providers::{LlmConfig, LlmProvider};
use std::collections::HashMap;
use std::io::ErrorKind;
use std::net::TcpListener;
use std::sync::OnceLock;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// The host's ability to bind the loopback address Wiremock uses for its servers.
///
/// The result is cached because this is a host capability, rather than a behavior
/// exercised independently by each test.
#[derive(Debug)]
enum LoopbackTcpBindCapability {
    Available,
    PermissionDenied(String),
    UnexpectedFailure(String),
}

fn loopback_tcp_bind_capability() -> &'static LoopbackTcpBindCapability {
    static CAPABILITY: OnceLock<LoopbackTcpBindCapability> = OnceLock::new();

    CAPABILITY.get_or_init(|| match TcpListener::bind("127.0.0.1:0") {
        Ok(listener) => {
            drop(listener);
            LoopbackTcpBindCapability::Available
        }
        Err(error) if error.kind() == ErrorKind::PermissionDenied => {
            LoopbackTcpBindCapability::PermissionDenied(error.to_string())
        }
        Err(error) => LoopbackTcpBindCapability::UnexpectedFailure(format!(
            "kind={:?}, error={error}",
            error.kind()
        )),
    })
}

macro_rules! require_loopback_tcp_bind {
    () => {
        match loopback_tcp_bind_capability() {
            LoopbackTcpBindCapability::Available => {}
            LoopbackTcpBindCapability::PermissionDenied(error) => {
                eprintln!(
                    "skipping Wiremock LLM integration test: host denied loopback TCP binding \\
                     (127.0.0.1:0, PermissionDenied): {error}"
                );
                return;
            }
            LoopbackTcpBindCapability::UnexpectedFailure(error) => {
                panic!(
                    "Wiremock LLM integration test setup failed while checking loopback TCP \\
                     bind capability (127.0.0.1:0): {error}"
                );
            }
        }
    };
}

/// Build an `LlmProvider` whose sole "test" provider points at the mock server.
fn make_provider(server: &MockServer) -> LlmProvider {
    let mut configs = HashMap::new();
    configs.insert(
        "test".to_string(),
        LlmConfig::custom(format!("{}/v1", server.uri()), "test-key"),
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

/// Build the `args` slice expected by the `chat` action:
/// `[provider, model, messages, params]`
fn chat_args(messages: Vec<Value>) -> Vec<Value> {
    vec![
        Value::String("test".to_string()),
        Value::String("gpt-4o".to_string()),
        Value::list_from_vec(messages),
        // params = None variant (no optional params)
        Value::unit_variant("None"),
    ]
}

// ---------------------------------------------------------------------------
// a) Successful chat completion
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_chat_success_mock() {
    require_loopback_tcp_bind!();
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "resp_test",
            "object": "chat.completion",
            "created": 1_234_567_890_u64,
            "model": "gpt-4o",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Hello!"
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 5,
                "completion_tokens": 2,
                "total_tokens": 7
            }
        })))
        .mount(&server)
        .await;

    let provider = make_provider(&server);
    let args = chat_args(vec![user_message("Hi")]);

    let result = provider.execute("chat", &args).await;
    assert!(result.is_ok(), "chat should succeed: {:?}", result.err());

    let value = result.unwrap();

    // Response should be a Record with the expected fields
    match &value {
        Value::Record(fields) => {
            // model
            assert_eq!(
                fields.get("model").and_then(|v| v.as_string()),
                Some("gpt-4o")
            );
            // id
            assert_eq!(
                fields.get("id").and_then(|v| v.as_string()),
                Some("resp_test")
            );
            // content should be Some("Hello!")
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
                    assert_eq!(val.as_string(), Some("Hello!"));
                }
                other => panic!("content should be Some variant, got: {other:?}"),
            }
            // finish_reason should be Some("stop")
            match fields.get("finish_reason") {
                Some(Value::Variant { name, .. }) => {
                    assert_eq!(name, "Some");
                }
                other => panic!("finish_reason should be Some variant, got: {other:?}"),
            }
            // usage should be Some(Record)
            match fields.get("usage") {
                Some(Value::Variant { name, .. }) => {
                    assert_eq!(name, "Some");
                }
                other => panic!("usage should be Some variant, got: {other:?}"),
            }
        }
        other => panic!("Expected Record, got: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// b) 401 unauthorized -> PermissionDenied
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_chat_401_unauthorized() {
    require_loopback_tcp_bind!();
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
    let args = chat_args(vec![user_message("Hi")]);

    let result = provider.execute("chat", &args).await;
    assert!(result.is_err());

    match result.unwrap_err() {
        CapabilityError::PermissionDenied(msg) => {
            assert!(
                msg.contains("auth_error"),
                "PermissionDenied should contain 'auth_error', got: {msg}"
            );
        }
        other => panic!("Expected PermissionDenied, got: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// c) 429 rate limited -> ExecutionFailed("rate_limited")
//
// NOTE: async-openai retries 429 responses with exponential backoff by default.
// To avoid a long-running test, we use error type "insufficient_quota" which
// async-openai treats as a Permanent error (no retry). The error still carries
// code "rate_limit_exceeded" so the error mapping in LlmProvider maps it to
// ExecutionFailed("rate_limited") correctly.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_chat_429_rate_limited() {
    require_loopback_tcp_bind!();
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
    let args = chat_args(vec![user_message("Hi")]);

    let result = provider.execute("chat", &args).await;
    assert!(result.is_err());

    match result.unwrap_err() {
        CapabilityError::ExecutionFailed(msg) => {
            assert!(
                msg.contains("rate_limited"),
                "ExecutionFailed should contain 'rate_limited', got: {msg}"
            );
        }
        other => panic!("Expected ExecutionFailed with rate_limited, got: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// d) Successful list_models
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_list_models_success_mock() {
    require_loopback_tcp_bind!();
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
    let items = value
        .list_to_vec()
        .unwrap_or_else(|| panic!("Expected list value, got: {value:?}"));
    // Should contain the two model IDs
    let model_ids: Vec<&str> = items.iter().filter_map(|v| v.as_string()).collect();
    assert!(
        model_ids.contains(&"gpt-4o"),
        "Should contain gpt-4o, got: {model_ids:?}"
    );
    assert!(
        model_ids.contains(&"gpt-4o-mini"),
        "Should contain gpt-4o-mini, got: {model_ids:?}"
    );
    assert_eq!(model_ids.len(), 2);
}
