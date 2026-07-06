//! MCP (Model Context Protocol) Capability Provider
//!
//! Provides JSON-RPC 2.0 communication with MCP-compatible LLM servers.

use ash_core::capability::{
    CapabilityError, CapabilityProvider, ProviderAuthoringMetadata, ProviderOperationMetadata,
};
use ash_core::{Constraint, Effect, Value};
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
    /// Returns `CapabilityError` if HTTP client creation fails
    pub fn new(config: McpConfig) -> Result<Self, CapabilityError> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_millis(config.timeout_ms))
            .build()
            .map_err(|e| {
                CapabilityError::ExecutionFailed(format!("failed to create HTTP client: {e}"))
            })?;

        Ok(Self {
            client,
            base_url: config.base_url,
            capabilities: McpCapabilities::default(),
        })
    }

    /// Create with default configuration
    ///
    /// # Errors
    /// Returns `CapabilityError` if HTTP client creation fails
    pub fn default_config() -> Result<Self, CapabilityError> {
        Self::new(McpConfig::default())
    }

    /// Call a JSON-RPC method
    ///
    /// # Errors
    /// Returns `CapabilityError` if HTTP request fails or JSON parsing fails
    pub async fn call(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<Value, CapabilityError> {
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
            .map_err(|e| CapabilityError::ExecutionFailed(format!("HTTP request failed: {e}")))?;

        let rpc_response: JsonRpcResponse = response
            .json()
            .await
            .map_err(|e| CapabilityError::ExecutionFailed(format!("JSON parse failed: {e}")))?;

        if let Some(error) = rpc_response.error {
            return Err(CapabilityError::ExecutionFailed(format!(
                "JSON-RPC error {}: {}",
                error.code, error.message
            )));
        }

        rpc_response
            .result
            .map(|v| serde_json::from_value(v).unwrap_or(Value::Null))
            .ok_or_else(|| CapabilityError::ExecutionFailed("empty JSON-RPC result".to_string()))
    }

    /// Call an MCP tool
    ///
    /// # Errors
    /// Returns `CapabilityError` if tool call fails
    pub async fn call_tool(
        &self,
        name: &str,
        args: HashMap<String, Value>,
    ) -> Result<Value, CapabilityError> {
        let params = json!({
            "name": name,
            "arguments": args,
        });
        self.call("tools/call", params).await
    }

    /// Get an MCP prompt
    ///
    /// # Errors
    /// Returns `CapabilityError` if prompt retrieval fails
    pub async fn get_prompt(
        &self,
        name: &str,
        args: HashMap<String, String>,
    ) -> Result<Value, CapabilityError> {
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

    fn provider_metadata(&self) -> ProviderAuthoringMetadata {
        ProviderAuthoringMetadata::new("mcp")
            .with_operation(
                ProviderOperationMetadata::new("capabilities", Effect::Epistemic)
                    .with_required_row("mcp.capabilities")
                    .with_resource("mcp")
                    .with_sandbox_policy("host.mcp.capabilities")
                    .with_provenance_policy("host.mcp.capabilities.redacted"),
            )
            .with_operation(
                ProviderOperationMetadata::new("call", Effect::Deliberative)
                    .with_required_row("mcp.call")
                    .with_constraint("tools")
                    .with_resource("mcp")
                    .with_sandbox_policy("host.mcp.call")
                    .with_provenance_policy("host.mcp.call.redacted"),
            )
    }

    async fn observe(&self, constraints: &[Constraint]) -> Result<Value, CapabilityError> {
        if constraints.is_empty() {
            return Err(CapabilityError::InvalidArgument(
                "No observe constraints provided".to_string(),
            ));
        }
        let action_name = constraints[0].predicate.name.as_str();
        match action_name {
            "capabilities" => {
                let caps = serde_json::json!({
                    "tools": self.capabilities.tools,
                    "prompts": self.capabilities.prompts,
                });
                Ok(serde_json::from_value(caps).unwrap_or(Value::Null))
            }
            _ => Err(CapabilityError::NotAvailable(format!(
                "unknown observe action: {action_name}"
            ))),
        }
    }

    async fn execute(&self, action_name: &str, args: &[Value]) -> Result<Value, CapabilityError> {
        match action_name {
            "call" => {
                if args.len() < 2 {
                    return Err(CapabilityError::InvalidArgument(
                        "call requires method and params".to_string(),
                    ));
                }
                let method = args[0].as_string().unwrap_or("");
                let params = serde_json::to_value(&args[1]).unwrap_or_else(|_| json!({}));
                self.call(method, params).await
            }
            _ => Err(CapabilityError::NotAvailable(format!(
                "unknown execute action: {action_name}"
            ))),
        }
    }
}
