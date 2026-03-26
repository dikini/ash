//! MCP (Model Context Protocol) Capability Provider
//!
//! Provides JSON-RPC 2.0 communication with MCP-compatible LLM servers.

use super::{CapabilityProvider, ProviderError};
use ash_core::{Effect, Value};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

/// MCP Provider for LLM communication
#[derive(Debug, Clone)]
pub struct McpProvider {
    /// HTTP client
    client: Client,
    /// Base URL for MCP server
    base_url: String,
    /// Server capabilities (populated during initialization)
    capabilities: McpCapabilities,
}

/// MCP configuration
#[derive(Debug, Clone)]
pub struct McpConfig {
    /// Base URL for MCP server
    pub base_url: String,
    /// Request timeout in milliseconds
    pub timeout_ms: u64,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8080".to_string(),
            timeout_ms: 30000,
        }
    }
}

/// MCP server capabilities
#[derive(Debug, Clone, Default)]
pub struct McpCapabilities {
    /// Available tools
    pub tools: Vec<String>,
    /// Available prompts
    pub prompts: Vec<String>,
}

/// JSON-RPC 2.0 request
#[derive(Serialize, Debug)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: serde_json::Value,
    id: u64,
}

/// JSON-RPC 2.0 response
#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct JsonRpcResponse {
    jsonrpc: String,
    result: Option<serde_json::Value>,
    error: Option<JsonRpcError>,
    id: u64,
}

/// JSON-RPC 2.0 error
#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct JsonRpcError {
    code: i32,
    message: String,
    data: Option<serde_json::Value>,
}

impl McpProvider {
    /// Create a new MCP provider with the given config
    ///
    /// # Errors
    /// Returns `ProviderError` if HTTP client creation fails
    pub fn new(config: McpConfig) -> Result<Self, ProviderError> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_millis(config.timeout_ms))
            .build()
            .map_err(|e| ProviderError::new(format!("failed to create HTTP client: {e}")))?;

        Ok(Self {
            client,
            base_url: config.base_url,
            capabilities: McpCapabilities::default(),
        })
    }

    /// Create with default configuration
    ///
    /// # Errors
    /// Returns `ProviderError` if HTTP client creation fails
    pub fn default_config() -> Result<Self, ProviderError> {
        Self::new(McpConfig::default())
    }

    /// Call a JSON-RPC method
    ///
    /// # Errors
    /// Returns `ProviderError` if HTTP request fails or JSON parsing fails
    pub async fn call(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<Value, ProviderError> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id: 1,
        };

        let response = self
            .client
            .post(format!("{}/jsonrpc", self.base_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| ProviderError::new(format!("HTTP request failed: {e}")))?;

        let rpc_response: JsonRpcResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::new(format!("JSON parse failed: {e}")))?;

        if let Some(error) = rpc_response.error {
            return Err(ProviderError::new(format!(
                "JSON-RPC error {}: {}",
                error.code, error.message
            )));
        }

        rpc_response
            .result
            .map(|v| serde_json::from_value(v).unwrap_or(Value::Null))
            .ok_or_else(|| ProviderError::new("empty JSON-RPC result".to_string()))
    }

    /// Call an MCP tool
    ///
    /// # Errors
    /// Returns `ProviderError` if tool call fails
    pub async fn call_tool(
        &self,
        name: &str,
        args: HashMap<String, Value>,
    ) -> Result<Value, ProviderError> {
        let params = json!({
            "name": name,
            "arguments": args,
        });
        self.call("tools/call", params).await
    }

    /// Get an MCP prompt
    ///
    /// # Errors
    /// Returns `ProviderError` if prompt retrieval fails
    pub async fn get_prompt(
        &self,
        name: &str,
        args: HashMap<String, String>,
    ) -> Result<Value, ProviderError> {
        let params = json!({
            "name": name,
            "arguments": args,
        });
        self.call("prompts/get", params).await
    }

    /// Get capabilities (for testing)
    #[must_use]
    pub const fn capabilities(&self) -> &McpCapabilities {
        &self.capabilities
    }
}

#[async_trait]
impl CapabilityProvider for McpProvider {
    fn name(&self) -> &'static str {
        "mcp"
    }

    fn effect(&self) -> Effect {
        Effect::Deliberative
    }

    async fn observe(&self, action: &str, _args: &[Value]) -> Result<Value, ProviderError> {
        match action {
            "capabilities" => {
                let caps = serde_json::json!({
                    "tools": self.capabilities.tools,
                    "prompts": self.capabilities.prompts,
                });
                Ok(serde_json::from_value(caps).unwrap_or(Value::Null))
            }
            _ => Err(ProviderError::new(format!(
                "unknown observe action: {action}"
            ))),
        }
    }

    async fn execute(&self, action: &str, args: &[Value]) -> Result<Value, ProviderError> {
        match action {
            "call" => {
                if args.len() < 2 {
                    return Err(ProviderError::new(
                        "call requires method and params".to_string(),
                    ));
                }
                let method = args[0].as_string().unwrap_or("");
                let params = serde_json::to_value(&args[1]).unwrap_or_else(|_| json!({}));
                self.call(method, params).await
            }
            _ => Err(ProviderError::new(format!(
                "unknown execute action: {action}"
            ))),
        }
    }
}
