# TASK-072: Implement Engine::parse

## Status: 🟢 Complete

## Description

Implement the parse methods for the Engine type.

## Specification Reference

- SPEC-010: Embedding API - Section 2.1 Engine Type

## Requirements

### Functional Requirements

1. `Engine::parse(source: &str) -> Result<Workflow, EngineError>`
2. `Engine::parse_file(path) -> Result<Workflow, EngineError>`

### Property Requirements

```rust
// Valid workflow parses
engine.parse("workflow w { action a { effect: operational; body: || -> 1; } }").is_ok()

// Invalid syntax returns error
engine.parse("invalid").is_err()

// File parsing reads and parses
engine.parse_file("test.ash").is_ok() // if file exists and is valid
```

## TDD Steps

### Step 1: Write Tests (Red)

```rust
#[test]
fn test_engine_parse_workflow() {
    let engine = Engine::new().build().unwrap();
    let result = engine.parse(r#"
        workflow test {
            action a {
                effect: operational;
                body: || -> 42;
            }
        }
    "#);
    assert!(result.is_ok());
}

#[test]
fn test_engine_parse_error() {
    let engine = Engine::new().build().unwrap();
    let result = engine.parse("invalid syntax");
    assert!(result.is_err());
}
```

### Step 2: Implement (Green)

```rust
use ash_parser::new_input;
use ash_parser::parse_workflow::workflow_def;

impl Engine {
    pub fn parse(&self, source: &str) -> Result<Workflow, EngineError> {
        let mut input = new_input(source);
        workflow_def(&mut input)
            .map_err(|e| EngineError::Parse(format!("{:?}", e)))
    }

    pub fn parse_file(&self, path: impl AsRef<Path>) -> Result<Workflow, EngineError> {
        let source = std::fs::read_to_string(path)?;
        self.parse(&source)
    }
}
```

## Completion Checklist

- [ ] Engine::parse implemented
- [ ] Engine::parse_file implemented
- [ ] Tests pass
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Estimated Effort

2 hours

## Dependencies

- TASK-071 (ash-engine crate)

## Blocked By

- TASK-071

## Blocks

- TASK-073 (Engine::check)
