# TASK-074: Implement Engine::execute

## Status: ✅ Complete

## Description

Implement the execute method and convenience run methods for the Engine type.

## Specification Reference

- SPEC-010: Embedding API - Section 2.1 Engine Type

## Requirements

### Functional Requirements

1. `Engine::execute(workflow: &Workflow) -> ExecResult<Value>`
2. `Engine::run(source: &str) -> ExecResult<Value>` - parse + check + execute
3. `Engine::run_file(path) -> ExecResult<Value>` - parse file + check + execute

### Property Requirements

```rust
// Execute parsed workflow
engine.execute(&workflow).await.is_ok()

// Run does full pipeline
engine.run("workflow w { ... }").await.is_ok()
```

## TDD Steps

### Step 1: Write Tests (Red)

```rust
#[tokio::test]
async fn test_engine_execute() {
    let engine = Engine::new().build().unwrap();
    let workflow = engine.parse(r#"
        workflow test {
            action a {
                effect: operational;
                body: || -> 42;
            }
        }
    "#).unwrap();
    
    let result = engine.execute(&workflow).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_engine_run() {
    let engine = Engine::new().build().unwrap();
    let result = engine.run(r#"
        workflow test {
            action a {
                effect: operational;
                body: || -> 42;
            }
        }
    "#).await;
    
    assert!(result.is_ok());
}
```

### Step 2: Implement (Green)

```rust
use ash_interp::interpret;

impl Engine {
    pub async fn execute(&self, workflow: &Workflow) -> ExecResult<Value> {
        interpret(workflow).await
    }

    pub async fn run(&self, source: &str) -> ExecResult<Value> {
        let workflow = self.parse(source)?;
        self.check(&workflow)?;
        self.execute(&workflow).await
    }

    pub async fn run_file(&self, path: impl AsRef<Path>) -> ExecResult<Value> {
        let workflow = self.parse_file(path)?;
        self.check(&workflow)?;
        self.execute(&workflow).await
    }
}
```

## Completion Checklist

- [ ] Engine::execute implemented
- [ ] Engine::run implemented
- [ ] Engine::run_file implemented
- [ ] Tests pass
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Estimated Effort

3 hours

## Dependencies

- TASK-072 (Engine::parse)
- TASK-073 (Engine::check)

## Blocked By

- TASK-073

## Blocks

- TASK-077 (REPL uses Engine)
