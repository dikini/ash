# TASK-270: MCP Capability Provider

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Objective:** Implement MCP (Model Context Protocol) capability provider for LLM communication.

**Spec Reference:** Agent harness design document, MCP spec

**File Locations:**
- Create: `crates/ash-engine/src/providers/mcp.rs`
- Test: `crates/ash-engine/tests/mcp_provider_tests.rs`

---

## Background

MCP provider enables communication with LLM agents:
```ash
capability mcp = http
    where constraints {
        jsonrpc_version: "2.0",
        capabilities: ["tools", "prompts"]
    }
```

---

## Step 1: Create MCP Provider

Create `crates/ash-engine/src/providers/mcp.rs`:

```rust
use crate::providers::*;
use reqwest;
use serde_json::json;

/// MCP (Model Context Protocol) provider
pub struct McpProvider {
    client: reqwest::Client,
    base_url: String,
    config: McpConfig,
}

#[derive(Debug, Clone)]
pub struct McpConfig {
    pub base_url: String,
    pub timeout_ms: u64,
    pub jsonrpc_version: String,
    pub capabilities: Vec<String>,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8080".to_string(),
            timeout_ms: 30000,
            jsonrpc_version: "2.0".to_string(),
            capabilities: vec!["tools".to_string(), "prompts".to_string()],
        }
    }
}

impl McpProvider {
    pub fn new(config: McpConfig) -> Result<Self, ProviderError> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(config.timeout_ms))
            .build()
            .map_err(|e| ProviderError::Configuration(e.to_string()))?;
        
        Ok(Self {
            client,
            base_url: config.base_url.clone(),
            config,
        })
    }
    
    /// Call MCP method
    pub async fn call(
        &self,
        method: &str,
        params: Value,
    ) -> Result<Value, ProviderError> {
        let request = json!({
            "jsonrpc": self.config.jsonrpc_version,
            "method": method,
            "params": params,
            "id": generate_request_id(),
        });
        
        let response = self.client
            .post(&format!("{}/mcp", self.base_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?;
        
        let status = response.status();
        if !status.is_success() {
            return Err(ProviderError::HttpError {
                status: status.as_u16(),
                message: response.text().await.unwrap_or_default(),
            });
        }
        
        let json: Value = response.json().await
            .map_err(|e| ProviderError::ParseError(e.to_string()))?;
        
        // Check for JSON-RPC error
        if let Some(error) = json.get("error") {
            return Err(ProviderError::McpError {
                code: error.get("code").and_then(|v| v.as_i64()).unwrap_or(-1),
                message: error.get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown error")
                    .to_string(),
            });
        }
        
        // Extract result
        json.get("result")
            .cloned()
            .ok_or(ProviderError::MissingField("result"))
    }
    
    /// Initialize MCP session
    pub async fn initialize(&self) -> Result<McpSession, ProviderError> {
        let result = self.call("initialize", json!({
            "capabilities": self.config.capabilities,
        })).await?;
        
        Ok(McpSession {
            server_capabilities: result.get("capabilities").cloned(),
        })
    }
    
    /// Call a tool
    pub async fn call_tool(
        &self,
        name: &str,
        arguments: Value,
    ) -> Result<Value, ProviderError> {
        self.call("tools/call", json!({
            "name": name,
            "arguments": arguments,
        })).await
    }
    
    /// Get a prompt
    pub async fn get_prompt(
        &self,
        name: &str,
        arguments: Option<Value>,
    ) -> Result<Value, ProviderError> {
        let mut params = json!({ "name": name });
        if let Some(args) = arguments {
            params["arguments"] = args;
        }
        
        self.call("prompts/get", params).await
    }
}

pub struct McpSession {
    pub server_capabilities: Option<Value>,
}

impl Provider for McpProvider {
    fn capability_name(&self) -> &str { "mcp" }
    
    fn invoke(&self, operation: &str, args: Value) -> Result<Value, ProviderError> {
        // For sync contexts, block on async
        let rt = tokio::runtime::Handle::try_current()
            .or_else(|_| {
                tokio::runtime::Runtime::new()
                    .map(|rt| rt.handle().clone())
            })
            .map_err(|e| ProviderError::Runtime(e.to_string()))?;
        
        rt.block_on(async {
            match operation {
                "call" => self.call("delegate", args).await,
                "tool" => {
                    let name = args.get("name")
                        .and_then(|v| v.as_str())
                        .ok_or(ProviderError::MissingArgument("name"))?;
                    let arguments = args.get("arguments").cloned().unwrap_or(Value::Null);
                    self.call_tool(name, arguments).await
                }
                "prompt" => {
                    let name = args.get("name")
                        .and_then(|v| v.as_str())
                        .ok_or(ProviderError::MissingArgument("name"))?;
                    let arguments = args.get("arguments").cloned();
                    self.get_prompt(name, arguments).await
                }
                _ => Err(ProviderError::UnknownOperation(operation.to_string())),
            }
        })
    }
}

fn generate_request_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::SeqCst)
}
```

---

## Step 2: Write Tests

```rust
// crates/ash-engine/tests/mcp_provider_tests.rs
use ash_engine::providers::*;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path, body_json};

#[tokio::test]
async fn test_mcp_call_success() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("POST"))
        .and(path("/mcp"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(serde_json::json!({
                "jsonrpc": "2.0",
                "result": { "answer": 42 },
                "id": 1
            })))
        .mount(&mock_server)
        .await;
    
    let config = McpConfig {
        base_url: mock_server.uri(),
        ..Default::default()
    };
    
    let provider = McpProvider::new(config).unwrap();
    let result = provider.call("test", serde_json::json!({})).await;
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap()["answer"], 42);
}

#[tokio::test]
async fn test_mcp_error_response() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("POST"))
        .and(path("/mcp"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(serde_json::json!({
                "jsonrpc": "2.0",
                "error": {
                    "code": -32600,
                    "message": "Invalid request"
                },
                "id": 1
            })))
        .mount(&mock_server)
        .await;
    
    let config = McpConfig {
        base_url: mock_server.uri(),
        ..Default::default()
    };
    
    let provider = McpProvider::new(config).unwrap();
    let result = provider.call("test", serde_json::json!({})).await;
    
    assert!(result.is_err());
}

#[tokio::test]
async fn test_call_tool() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("POST"))
        .and(path("/mcp"))
        .and(body_json(serde_json::json!({
            "jsonrpc": "2.0",
            "method": "tools/call",
            "params": {
                "name": "analyze",
                "arguments": { "data": "test" }
            }
        })))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(serde_json::json!({
                "jsonrpc": "2.0",
                "result": { "analysis": "complete" },
                "id": 1
            })))
        .mount(&mock_server)
        .await;
    
    let config = McpConfig {
        base_url: mock_server.uri(),
        ..Default::default()
    };
    
    let provider = McpProvider::new(config).unwrap();
    let result = provider.call_tool("analyze", json!({ "data": "test" })).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_timeout() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200)
            .set_delay(std::time::Duration::from_secs(5)))
        .mount(&mock_server)
        .await;
    
    let config = McpConfig {
        base_url: mock_server.uri(),
        timeout_ms: 100,  // Very short timeout
        ..Default::default()
    };
    
    let provider = McpProvider::new(config).unwrap();
    let result = provider.call("test", json!({})).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("timeout"));
}
```

---

## Step 3: Integrate into Provider Registry

```rust
// crates/ash-engine/src/providers/mod.rs
pub mod mcp;
pub use mcp::*;
```

---

## Step 4: Run Tests

```bash
cargo test --package ash-engine mcp -v
```

---

## Step 5: Commit

```bash
git add crates/ash-engine/src/providers/mcp.rs
git add crates/ash-engine/tests/mcp_provider_tests.rs
git add crates/ash-engine/src/providers/mod.rs
git commit -m "feat: MCP capability provider (TASK-270)

- McpProvider with JSON-RPC 2.0 over HTTP
- Configurable base URL, timeout, capabilities
- Async call method with error handling
- Initialize, call_tool, get_prompt helpers
- Provider trait implementation
- Proper error handling for HTTP, JSON-RPC errors
- Tests with wiremock for HTTP mocking
- Timeout handling"
```

---

## Step 6: Codex Verification (REQUIRED)

```
delegate_task to codex:
  goal: "Verify TASK-270 implementation"
  context: |
    Files to verify:
    - crates/ash-engine/src/providers/mcp.rs
    - crates/ash-engine/tests/mcp_provider_tests.rs
    
    Requirements:
    1. JSON-RPC 2.0 format correct
    2. HTTP client configurable
    3. Timeout handling works
    4. Error handling complete
    5. Provider trait implemented
    6. Tests cover success/failure cases
    7. Async/await correct
    
    Run and report:
    1. cargo test --package ash-engine mcp
    2. cargo clippy --package ash-engine --all-targets --all-features -- -D warnings
    3. cargo fmt --check --package ash-engine
    4. Review JSON-RPC format
    5. Check error handling completeness
    6. Verify wiremock tests
    
    Expected: "VERIFIED" or "BLOCKED: [issues]"
```

---

## Completion Checklist

- [ ] McpProvider created
- [ ] Failing tests written
- [ ] JSON-RPC implementation
- [ ] HTTP client
- [ ] Error handling
- [ ] Provider trait
- [ ] Tests pass
- [ ] Clippy clean
- [ ] Format clean
- [ ] **Codex verification passed**
- [ ] CHANGELOG.md entry added

---

**Estimated Hours:** 10
**Blocked by:** TASK-269
**Blocks:** Phase 46 closeout

**Note:** This completes the optional Agent Harness sub-phase.
