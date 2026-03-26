# TASK-246: Make EngineBuilder Methods Real

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Objective:** Implement actual functionality for `EngineBuilder` methods that are currently no-ops.

**Spec Reference:** SPEC-010 (Embedding API), Engine builder contract

**File Locations:**
- Modify: `crates/ash-engine/src/lib.rs:306,318,327`
- Test: `crates/ash-engine/tests/engine_builder_tests.rs` (create)

---

## Background

The audit found `EngineBuilder` methods are documented but not implemented:

```rust
// Current no-ops (lines 306, 318, 327)
pub fn with_http_capabilities(self, _config: HttpConfig) -> Self {
    self  // No actual HTTP setup
}

pub fn with_custom_provider(self, _provider: Box<dyn Provider>) -> Self {
    self  // No provider registration
}
```

This violates SPEC-010's embedding API contract.

---

## Step 1: Audit Current Builder

Find all builder methods:

```bash
grep -n "pub fn with_" crates/ash-engine/src/lib.rs
grep -n "pub fn build" crates/ash-engine/src/lib.rs
```

Review Engine struct to understand what needs configuration:

```bash
grep -n "struct Engine" crates/ash-engine/src/lib.rs
```

---

## Step 2: Design Builder State

Determine what state EngineBuilder needs:

```rust
pub struct EngineBuilder {
    providers: HashMap<String, Box<dyn Provider>>,
    http_config: Option<HttpConfig>,
    capability_grants: Vec<CapabilityGrant>,
    policy_config: Option<PolicyConfig>,
    // ... other config ...
}
```

---

## Step 3: Write Failing Tests

```rust
// crates/ash-engine/tests/engine_builder_tests.rs
use ash_engine::*;

#[test]
fn test_builder_with_http_capabilities() {
    let engine = Engine::builder()
        .with_http_capabilities(HttpConfig {
            timeout_ms: 5000,
            max_connections: 10,
        })
        .build()
        .unwrap();
    
    // Verify HTTP provider registered
    assert!(engine.has_provider("http"));
}

#[test]
fn test_builder_with_custom_provider() {
    let my_provider = TestProvider::new();
    
    let engine = Engine::builder()
        .with_custom_provider("test", Box::new(my_provider))
        .build()
        .unwrap();
    
    assert!(engine.has_provider("test"));
}

#[test]
fn test_builder_multiple_providers() {
    let engine = Engine::builder()
        .with_http_capabilities(HttpConfig::default())
        .with_filesystem_capabilities(FilesystemConfig::default())
        .build()
        .unwrap();
    
    assert!(engine.has_provider("http"));
    assert!(engine.has_provider("fs"));
}
```

---

## Step 4: Implement Builder State

Modify `crates/ash-engine/src/lib.rs`:

```rust
pub struct EngineBuilder {
    providers: HashMap<String, Box<dyn Provider>>,
    http_config: Option<HttpConfig>,
    fs_config: Option<FilesystemConfig>,
    custom_providers: Vec<(String, Box<dyn Provider>)>,
}

impl EngineBuilder {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            http_config: None,
            fs_config: None,
            custom_providers: Vec::new(),
        }
    }
    
    pub fn with_http_capabilities(mut self, config: HttpConfig) -> Self {
        self.http_config = Some(config);
        self
    }
    
    pub fn with_filesystem_capabilities(mut self, config: FilesystemConfig) -> Self {
        self.fs_config = Some(config);
        self
    }
    
    pub fn with_custom_provider(
        mut self,
        name: impl Into<String>,
        provider: Box<dyn Provider>,
    ) -> Self {
        self.custom_providers.push((name.into(), provider));
        self
    }
    
    pub fn build(self) -> Result<Engine, EngineError> {
        let mut engine = Engine::new();
        
        // Register HTTP provider if configured
        if let Some(config) = self.http_config {
            let http_provider = HttpProvider::new(config);
            engine.register_provider("http", Box::new(http_provider))?;
        }
        
        // Register filesystem provider if configured
        if let Some(config) = self.fs_config {
            let fs_provider = FilesystemProvider::new(config);
            engine.register_provider("fs", Box::new(fs_provider))?;
        }
        
        // Register custom providers
        for (name, provider) in self.custom_providers {
            engine.register_provider(&name, provider)?;
        }
        
        Ok(engine)
    }
}
```

---

## Step 5: Create Standard Providers

Implement `HttpProvider` and `FilesystemProvider` if they don't exist:

```rust
// crates/ash-engine/src/providers/http.rs
pub struct HttpProvider {
    config: HttpConfig,
    client: reqwest::Client,  // or similar
}

impl Provider for HttpProvider {
    fn capability_name(&self) -> &str { "http" }
    
    fn invoke(&self, operation: &str, args: Value) -> Result<Value, ProviderError> {
        match operation {
            "get" => self.http_get(args),
            "post" => self.http_post(args),
            _ => Err(ProviderError::UnknownOperation),
        }
    }
}
```

---

## Step 6: Run Tests

```bash
cargo test --package ash-engine builder -v
```

---

## Step 7: Commit

```bash
git add crates/ash-engine/src/lib.rs
git add crates/ash-engine/src/providers/
git add crates/ash-engine/tests/engine_builder_tests.rs
git commit -m "feat: implement EngineBuilder configuration methods (TASK-246)

- Add builder state for providers and config
- Implement with_http_capabilities with HttpProvider
- Implement with_filesystem_capabilities with FilesystemProvider
- Implement with_custom_provider for user providers
- Build method registers all configured providers
- Tests for builder configuration and provider registration
- Aligns implementation with SPEC-010 API contract"
```

---

## Step 8: Codex Verification (REQUIRED)

```
delegate_task to codex:
  goal: "Verify TASK-246 implementation"
  context: |
    Files to verify:
    - crates/ash-engine/src/lib.rs (EngineBuilder)
    - crates/ash-engine/src/providers/ (new providers)
    - crates/ash-engine/tests/engine_builder_tests.rs
    
    Spec reference: SPEC-010 embedding API
    Requirements:
    1. with_http_capabilities registers HTTP provider
    2. with_filesystem_capabilities registers FS provider
    3. with_custom_provider registers user providers
    4. Build creates Engine with all configured providers
    5. No more no-op methods
    6. Matches SPEC-010 documented API
    
    Run and report:
    1. cargo test --package ash-engine builder
    2. cargo clippy --package ash-engine --all-targets --all-features -- -D warnings
    3. cargo fmt --check --package ash-engine
    4. Verify SPEC-010 alignment
    5. Check all builder methods have real implementations
    
    Expected: "VERIFIED" or "BLOCKED: [issues]"
```

---

## Completion Checklist

- [ ] Builder methods audited
- [ ] Builder state designed
- [ ] Failing tests written
- [ ] Builder methods implemented
- [ ] Standard providers created
- [ ] Tests pass
- [ ] Clippy clean
- [ ] Format clean
- [ ] **Codex verification passed**
- [ ] CHANGELOG.md entry added

---

**Estimated Hours:** 10
**Blocked by:** None
**Blocks:** TASK-247 (stub providers - some overlap)
