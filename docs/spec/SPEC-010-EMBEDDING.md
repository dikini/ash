# SPEC-010: Embedding API

## Status: Draft

## 1. Overview

The Embedding API provides a unified interface for integrating Ash into Rust applications. It encapsulates the entire workflow lifecycle:

```
Parse → Check → Execute
```

## 2. Architecture

### 2.1 Engine Type

The `Engine` is the central type for all Ash operations:

```rust
pub struct Engine {
    // Internal state
}

impl Engine {
    /// Create a new engine with default configuration
    pub fn new() -> EngineBuilder;
    
    /// Parse source code into a Workflow
    pub fn parse(&self, source: &str) -> Result<Workflow, EngineError>;
    
    /// Parse a file into a Workflow
    pub fn parse_file(&self, path: impl AsRef<Path>) -> Result<Workflow, EngineError>;
    
    /// Type check a workflow
    pub fn check(&self, workflow: &Workflow) -> Result<(), EngineError>;
    
    /// Execute a workflow asynchronously
    pub async fn execute(&self, workflow: &Workflow) -> ExecResult<Value>;
    
    /// Parse, check, and execute in one call
    pub async fn run(&self, source: &str) -> ExecResult<Value>;
    
    /// Parse, check, and execute file in one call
    pub async fn run_file(&self, path: impl AsRef<Path>) -> ExecResult<Value>;
}
```

### 2.2 Builder Pattern

Configuration uses the builder pattern:

```rust
let engine = Engine::new()
    .with_stdio_capabilities()    -- Add print/read_line
    .with_fs_capabilities()       -- Add file operations
    .with_http_capabilities()     -- Add HTTP client
    .with_custom_provider(MyProvider)
    .build()?;
```

## 3. Error Handling

### 3.1 Error Types

```rust
pub enum EngineError {
    Parse(String),        -- Syntax errors
    Type(String),         -- Type checking errors
    Execution(String),    -- Runtime errors
    Io(std::io::Error),   -- File/IO errors
    CapabilityNotFound(String),
}
```

### 3.2 Error Context

Errors include context for debugging:
- Source location (line, column)
- Error message
- Suggestions (when available)

## 4. Capability Providers

### 4.1 Provider Trait

```rust
#[async_trait]
pub trait CapabilityProvider: Send + Sync {
    fn name(&self) -> &str;
    fn effect(&self) -> Effect;
    async fn observe(&self, args: &[Value]) -> Result<Value, Error>;
    async fn execute(&self, args: &[Value]) -> Result<Value, Error>;
}
```

### 4.2 Registration

Providers are registered at engine build time:

```rust
engine.with_capability(MyProvider::new())
```

### 4.3 Standard Providers

Built-in provider categories:

| Provider | Capabilities | Effect |
|----------|-------------|--------|
| stdio | print, println, read_line | Operational |
| fs | read_file, write_file | Operational |
| http | get, post, put, delete | Operational |
| env | get_env, set_env | Operational |

## 5. Usage Patterns

### 5.1 Simple Evaluation

```rust
use ash_engine::Engine;

let engine = Engine::new().build()?;
let result = engine.run(r#"
    workflow main {
        action greet {
            effect: operational;
            body: || -> "Hello, World!";
        }
    }
"#).await?;

assert_eq!(result, Value::String("Hello, World!".into()));
```

### 5.2 Separate Phases

```rust
let engine = Engine::new().build()?;

-- Parse once
let workflow = engine.parse_file("workflow.ash")?;

-- Check separately
engine.check(&workflow)?;

-- Execute multiple times
for i in 0..10 {
    let result = engine.execute(&workflow).await?;
    println!("Run {}: {:?}", i, result);
}
```

### 5.3 With Capabilities

```rust
let engine = Engine::new()
    .with_stdio_capabilities()
    .with_fs_capabilities()
    .build()?;

let result = engine.run(r#"
    workflow main {
        action process {
            effect: operational;
            body: || -> {
                let data = file:read("input.txt");
                print("Processing...");
                data
            };
        }
    }
"#).await?;
```

## 6. Async Runtime

### 6.1 Runtime Requirements

The engine requires a Tokio runtime:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let engine = Engine::new().build()?;
    let result = engine.run("...").await?;
    Ok(())
}
```

### 6.2 Runtime Agnostic

Future versions may support other runtimes via feature flags.

## 7. Thread Safety

### 7.1 Send + Sync

`Engine` is `Send + Sync`:
- Can be shared across threads
- Can be moved between threads
- Internal state is synchronized

### 7.2 Workflow Sharing

`Workflow` is `Send + Sync`:
- Parsed workflows can be cached and reused
- Multiple threads can execute the same workflow

## 8. Performance

### 8.1 Workflow Caching

Applications should cache parsed `Workflow` values:

```rust
lazy_static! {
    static ref WORKFLOW: Workflow = {
        let engine = Engine::new().build().unwrap();
        engine.parse_file("workflow.ash").unwrap()
    };
}
```

### 8.2 Capability Reuse

Capability providers are created once at build time and reused.

## 9. Integration Example

### 9.1 Web Server

```rust
use axum::{extract::State, routing::post, Json, Router};
use ash_engine::Engine;
use std::sync::Arc;

struct AppState {
    engine: Engine,
    workflow: Workflow,
}

async fn execute(
    State(state): State<Arc<AppState>>,
    Json(input): Json<Value>,
) -> Json<Value> {
    let result = state.engine.execute(&state.workflow).await.unwrap();
    Json(result)
}

#[tokio::main]
async fn main() {
    let engine = Engine::new()
        .with_http_capabilities()
        .build()
        .unwrap();
    
    let workflow = engine.parse_file("api.ash").unwrap();
    
    let state = Arc::new(AppState { engine, workflow });
    
    let app = Router::new()
        .route("/execute", post(execute))
        .with_state(state);
    
    axum::serve(tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap(), app)
        .await
        .unwrap();
}
```

## 10. Future Extensions

- Module loading with `Engine::load_module()`
- Hot reloading of workflows
- Performance profiling hooks
- Custom effect handlers
