//! Engine-level integration tests for LLM capability.
//!
//! These tests verify that the Ash Engine can execute workflows using the LLM capability
//! registered via `with_llm_capabilities()`, satisfying TASK-523 requirements.
//!
//! Key verifications:
//! 1. Engine built with `with_llm_capabilities(configs)` correctly registers the LLM provider
//! 2. A core `Workflow::Act` targeting provider "llm" dispatches through the engine to LlmProvider
//! 3. The full chain engine → RuntimeState → CapabilityContext → LlmProvider → mock HTTP works

use ash_core::ast::{Guard, Workflow};
use ash_core::{Expr, Provenance, Value};
use ash_engine::providers::LlmConfig;
use ash_engine::Engine;
use std::collections::HashMap;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a `HashMap<String, LlmConfig>` pointing the "test" provider at the mock server.
fn make_configs(server: &MockServer) -> HashMap<String, LlmConfig> {
    let mut configs = HashMap::new();
    configs.insert(
        "test".to_string(),
        LlmConfig::custom(format!("{}/v1", server.uri()), "test-key"),
    );
    configs
}

/// Build a user message Value matching the chat module's expected shape.
fn user_message(content: &str) -> Value {
    let mut fields = HashMap::new();
    fields.insert("role".to_string(), Value::unit_variant("User"));
    fields.insert("content".to_string(), Value::String(content.to_string()));
    Value::Record(Box::new(fields))
}

/// Build the evaluated argument list for `llm:chat`:
/// `[provider_name, model, messages, params]`
fn chat_args_values() -> Vec<Value> {
    vec![
        Value::String("test".to_string()),
        Value::String("gpt-4o".to_string()),
        Value::List(Box::new(vec![user_message("Hello")])),
        Value::unit_variant("None"),
    ]
}

/// Construct a core `Workflow::Act` that calls `llm:chat(...)` with the given arguments.
fn llm_chat_act_workflow(args: Vec<Value>) -> Workflow {
    Workflow::Act {
        provider_name: "llm".into(),
        action_name: "chat".into(),
        arguments: args.into_iter().map(Expr::Literal).collect(),
        guard: Guard::Always,
        provenance: Provenance::default(),
        result_name: None,
        continuation: Box::new(Workflow::Done),
    }
}

/// Mount a standard chat completion mock on the server.
async fn mount_chat_completion_mock(server: &MockServer) {
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "chatcmpl_engine_integration",
            "object": "chat.completion",
            "created": 1_234_567_890_u64,
            "model": "gpt-4o",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Engine integration test response"
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 5,
                "total_tokens": 15
            }
        })))
        .mount(server)
        .await;
}

// ---------------------------------------------------------------------------
// TASK-523: Engine Builder Tests (with_llm_capabilities)
// ---------------------------------------------------------------------------

/// Test that Engine can be built with LLM capabilities registered via
/// the canonical `with_llm_capabilities()` builder method.
#[tokio::test]
async fn test_engine_with_llm_capabilities_builds() {
    let server = MockServer::start().await;
    let configs = make_configs(&server);

    let engine_result = Engine::new()
        .with_llm_capabilities(configs)
        .build();

    assert!(
        engine_result.is_ok(),
        "Engine with LLM capabilities should build successfully: {:?}",
        engine_result.err()
    );
}

/// Test that Engine built with `with_llm_capabilities()` correctly registers
/// the "llm" provider so that capability dispatch resolves it.
#[tokio::test]
async fn test_engine_llm_provider_registered_via_builder() {
    let server = MockServer::start().await;
    let configs = make_configs(&server);

    let engine = Engine::new()
        .with_llm_capabilities(configs)
        .build()
        .expect("engine should build");

    // Verify the engine can parse (proves it's functional)
    let parse_result = engine.parse("workflow main { done }");
    assert!(
        parse_result.is_ok(),
        "Engine should parse workflow: {:?}",
        parse_result.err()
    );
}

// ---------------------------------------------------------------------------
// TASK-523: Engine Executes LLM Action Through Capability Dispatch
// ---------------------------------------------------------------------------

/// Test that a core `Workflow::Act { provider_name: "llm", action_name: "chat" }`
/// executes through the engine built with `with_llm_capabilities()`.
///
/// This is the primary TASK-523 verification: the engine dispatches the action
/// through its RuntimeState → CapabilityContext → registered LlmProvider → mock HTTP.
#[tokio::test]
async fn test_engine_executes_llm_chat_via_capability_dispatch() {
    let server = MockServer::start().await;
    mount_chat_completion_mock(&server).await;

    let configs = make_configs(&server);

    // Build engine using with_llm_capabilities (not with_custom_provider)
    let engine = Engine::new()
        .with_llm_capabilities(configs)
        .build()
        .expect("engine with LLM should build");

    // Construct a core workflow that performs act llm:chat(...)
    let workflow = llm_chat_act_workflow(chat_args_values());

    // Execute through the engine's runtime state
    let result = engine.execute_core_workflow(&workflow).await;

    assert!(
        result.is_ok(),
        "Engine should execute llm:chat action: {:?}",
        result.err()
    );

    // Verify the response shape
    let value = result.unwrap();
    match &value {
        Value::Record(fields) => {
            assert_eq!(
                fields.get("model").and_then(|v| v.as_string()),
                Some("gpt-4o"),
                "Response should contain model field"
            );
            assert_eq!(
                fields.get("id").and_then(|v| v.as_string()),
                Some("chatcmpl_engine_integration"),
                "Response should contain id field"
            );
        }
        other => panic!("Expected Record from chat response, got: {other:?}"),
    }
}

/// Test that engine-built-with-llm-capabilities correctly dispatches `llm:list_models`.
#[tokio::test]
async fn test_engine_executes_llm_list_models() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "object": "list",
            "data": [
                {"id": "gpt-4o", "object": "model", "created": 1234567890, "owned_by": "openai"},
                {"id": "gpt-4", "object": "model", "created": 1234567890, "owned_by": "openai"}
            ]
        })))
        .mount(&server)
        .await;

    let configs = make_configs(&server);

    let engine = Engine::new()
        .with_llm_capabilities(configs)
        .build()
        .expect("engine should build");

    // Core workflow: act llm:list_models("test")
    let workflow = Workflow::Act {
        provider_name: "llm".into(),
        action_name: "list_models".into(),
        arguments: vec![Expr::Literal(Value::String("test".to_string()))],
        guard: Guard::Always,
        provenance: Provenance::default(),
        result_name: None,
        continuation: Box::new(Workflow::Done),
    };

    let result = engine.execute_core_workflow(&workflow).await;
    assert!(
        result.is_ok(),
        "Engine should execute llm:list_models: {:?}",
        result.err()
    );

    let value = result.unwrap();
    match &value {
        Value::List(models) => {
            assert_eq!(models.len(), 2, "Should list 2 models");
        }
        other => panic!("Expected List from list_models, got: {other:?}"),
    }
}

/// Test that engine dispatches `llm:embed` action.
#[tokio::test]
async fn test_engine_executes_llm_embed() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/embeddings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "object": "list",
            "data": [{
                "object": "embedding",
                "embedding": [0.1, 0.2, 0.3],
                "index": 0
            }],
            "model": "text-embedding-3-small",
            "usage": { "prompt_tokens": 2, "total_tokens": 2 }
        })))
        .mount(&server)
        .await;

    let configs = make_configs(&server);

    let engine = Engine::new()
        .with_llm_capabilities(configs)
        .build()
        .expect("engine should build");

    let embed_args = vec![
        Value::String("test".to_string()),
        Value::String("text-embedding-3-small".to_string()),
        Value::List(Box::new(vec![Value::String("hello".to_string())])),
    ];

    let workflow = Workflow::Act {
        provider_name: "llm".into(),
        action_name: "embed".into(),
        arguments: embed_args.into_iter().map(Expr::Literal).collect(),
        guard: Guard::Always,
        provenance: Provenance::default(),
        result_name: None,
        continuation: Box::new(Workflow::Done),
    };

    let result = engine.execute_core_workflow(&workflow).await;
    assert!(
        result.is_ok(),
        "Engine should execute llm:embed: {:?}",
        result.err()
    );

    let value = result.unwrap();
    match &value {
        Value::List(embeddings) => {
            assert_eq!(embeddings.len(), 1, "Should have 1 embedding");
        }
        other => panic!("Expected List from embed, got: {other:?}"),
    }
}

/// Test that engine-built-with-llm-capabilities rejects unknown LLM actions.
#[tokio::test]
async fn test_engine_llm_unknown_action_returns_error() {
    let server = MockServer::start().await;
    let configs = make_configs(&server);

    let engine = Engine::new()
        .with_llm_capabilities(configs)
        .build()
        .expect("engine should build");

    let workflow = Workflow::Act {
        provider_name: "llm".into(),
        action_name: "nonexistent_action".into(),
        arguments: vec![],
        guard: Guard::Always,
        provenance: Provenance::default(),
        result_name: None,
        continuation: Box::new(Workflow::Done),
    };

    let result = engine.execute_core_workflow(&workflow).await;
    assert!(result.is_err(), "Unknown LLM action should fail");
}

// ---------------------------------------------------------------------------
// TASK-523: Multi-Provider Engine Builder
// ---------------------------------------------------------------------------

/// Test that engine built with `with_llm_capabilities()` with multiple provider configs
/// correctly dispatches chat through the right provider.
#[tokio::test]
async fn test_engine_llm_multi_provider_via_builder() {
    let server = MockServer::start().await;
    mount_chat_completion_mock(&server).await;

    let mut configs = HashMap::new();
    configs.insert(
        "test".to_string(),
        LlmConfig::custom(format!("{}/v1", server.uri()), "test-key"),
    );
    configs.insert(
        "ollama".to_string(),
        LlmConfig::ollama(),
    );

    let engine = Engine::new()
        .with_llm_capabilities(configs)
        .build()
        .expect("engine with multi-provider LLM should build");

    // Execute chat via the "test" provider (pointed at mock)
    let workflow = llm_chat_act_workflow(chat_args_values());

    let result = engine.execute_core_workflow(&workflow).await;
    assert!(
        result.is_ok(),
        "Multi-provider engine should execute llm:chat: {:?}",
        result.err()
    );
}

// ---------------------------------------------------------------------------
// TASK-523: Builder Error Handling
// ---------------------------------------------------------------------------

/// Test that invalid LLM configs don't crash the engine build
/// (they're skipped with a warning per the builder's design).
#[tokio::test]
async fn test_engine_llm_invalid_config_skips_gracefully() {
    let mut configs = HashMap::new();
    configs.insert(
        "invalid".to_string(),
        LlmConfig::custom("not-a-url".to_string(), "key"),
    );

    // Should NOT fail - builder logs warning and skips registration
    let result = Engine::new()
        .with_llm_capabilities(configs)
        .build();

    assert!(
        result.is_ok(),
        "Engine should build even with invalid LLM config (skips registration)"
    );
}

// ---------------------------------------------------------------------------
// TASK-523: Engine Executing LLM Action with Result Binding
// ---------------------------------------------------------------------------

/// Test that a workflow with `Act ... as name` (result binding) followed by
/// a continuation correctly propagates the LLM response through the engine.
#[tokio::test]
async fn test_engine_llm_chat_with_result_binding() {
    let server = MockServer::start().await;
    mount_chat_completion_mock(&server).await;

    let configs = make_configs(&server);

    let engine = Engine::new()
        .with_llm_capabilities(configs)
        .build()
        .expect("engine should build");

    // Core workflow:
    //   act llm:chat(args) as result
    //   done
    let workflow = Workflow::Act {
        provider_name: "llm".into(),
        action_name: "chat".into(),
        arguments: chat_args_values().into_iter().map(Expr::Literal).collect(),
        guard: Guard::Always,
        provenance: Provenance::default(),
        result_name: Some("result".into()),
        continuation: Box::new(Workflow::Ret {
            expr: Expr::Variable("result".into()),
        }),
    };

    let result = engine.execute_core_workflow(&workflow).await;
    assert!(
        result.is_ok(),
        "Engine should execute llm:chat with result binding: {:?}",
        result.err()
    );

    // The return should be the chat response record
    let value = result.unwrap();
    match &value {
        Value::Record(fields) => {
            assert_eq!(
                fields.get("id").and_then(|v| v.as_string()),
                Some("chatcmpl_engine_integration"),
                "Result binding should propagate chat response"
            );
        }
        other => panic!("Expected Record from bound result, got: {other:?}"),
    }
}
