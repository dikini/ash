//! Tests for MCP provider

use ash_core::Value;
use ash_engine::providers::{McpConfig, McpProvider};
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_successful_mcp_call() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/jsonrpc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "result": {"status": "ok", "data": "test"},
            "id": 1
        })))
        .mount(&mock_server)
        .await;

    let config = McpConfig {
        base_url: mock_server.uri(),
        timeout_ms: 5000,
    };

    let provider = McpProvider::new(config).unwrap();
    let result = provider.call("test/method", json!({})).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_jsonrpc_error_response() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/jsonrpc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "error": {"code": -32600, "message": "Invalid Request"},
            "id": 1
        })))
        .mount(&mock_server)
        .await;

    let config = McpConfig {
        base_url: mock_server.uri(),
        timeout_ms: 5000,
    };

    let provider = McpProvider::new(config).unwrap();
    let result = provider.call("test/method", json!({})).await;

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("JSON-RPC error"));
}

#[tokio::test]
async fn test_tool_call_format() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/jsonrpc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "result": {"output": "success"},
            "id": 1
        })))
        .mount(&mock_server)
        .await;

    let config = McpConfig {
        base_url: mock_server.uri(),
        timeout_ms: 5000,
    };

    let provider = McpProvider::new(config).unwrap();
    let mut args = std::collections::HashMap::new();
    args.insert("key".to_string(), Value::String("value".to_string()));
    let result = provider.call_tool("test_tool", args).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_timeout_handling() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/jsonrpc"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(std::time::Duration::from_secs(2))
                .set_body_json(json!({"jsonrpc": "2.0", "result": {}, "id": 1})),
        )
        .mount(&mock_server)
        .await;

    let config = McpConfig {
        base_url: mock_server.uri(),
        timeout_ms: 100, // Very short timeout
    };

    let provider = McpProvider::new(config).unwrap();
    let result = provider.call("test/method", json!({})).await;

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("timeout") || err.contains("HTTP request failed"));
}
