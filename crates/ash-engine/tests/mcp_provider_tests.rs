//! Tests for MCP provider

use ash_core::Value;
use ash_engine::providers::{McpConfig, McpProvider};
use serde_json::json;
use std::io::ErrorKind;
use std::net::TcpListener;
use std::sync::OnceLock;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

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
                    "skipping Wiremock MCP integration test: host denied loopback TCP binding \\
                     (127.0.0.1:0, PermissionDenied): {error}"
                );
                return;
            }
            LoopbackTcpBindCapability::UnexpectedFailure(error) => {
                panic!(
                    "Wiremock MCP integration test setup failed while checking loopback TCP \\
                     bind capability (127.0.0.1:0): {error}"
                );
            }
        }
    };
}

#[tokio::test]
async fn test_successful_mcp_call() {
    require_loopback_tcp_bind!();
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
    require_loopback_tcp_bind!();
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
    require_loopback_tcp_bind!();
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
    require_loopback_tcp_bind!();
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
