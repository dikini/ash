# TASK-075: Standard Capability Providers

## Status: ✅ Complete

## Description

Implement standard capability providers for stdio, file system, and HTTP.

## Specification Reference

- SPEC-010: Embedding API - Section 4.3 Standard Providers

## Requirements

### Functional Requirements

1. StdioProvider: print, println, read_line
2. FsProvider: read_file, write_file, exists
3. HttpProvider: get, post, put, delete

### Property Requirements

```rust
// Stdio capability has operational effect
StdioProvider.effect() == Effect::Operational

// Provider registered
engine.with_stdio_capabilities().build().unwrap()
```

## TDD Steps

### Step 1: Write Tests (Red)

```rust
#[test]
fn test_stdio_provider() {
    let provider = StdioProvider::new();
    assert_eq!(provider.name(), "stdio");
    assert_eq!(provider.effect(), Effect::Operational);
}

#[test]
fn test_fs_provider() {
    let provider = FsProvider::new();
    assert_eq!(provider.name(), "fs");
}
```

### Step 2: Implement (Green)

```rust
pub struct StdioProvider;

#[async_trait]
impl CapabilityProvider for StdioProvider {
    fn name(&self) -> &str { "stdio" }
    fn effect(&self) -> Effect { Effect::Operational }
    
    async fn execute(&self, action: &str, args: &[Value]) -> Result<Value, Error> {
        match action {
            "print" => { print!("{}", args[0]); Ok(Value::Null) }
            "println" => { println!("{}", args[0]); Ok(Value::Null) }
            _ => Err(Error::UnknownAction),
        }
    }
}

impl EngineBuilder {
    pub fn with_stdio_capabilities(self) -> Self {
        self.with_capability(StdioProvider)
    }
    
    pub fn with_fs_capabilities(self) -> Self {
        self.with_capability(FsProvider)
    }
}
```

## Completion Checklist

- [ ] StdioProvider implemented
- [ ] FsProvider implemented
- [ ] Builder methods added
- [ ] Tests pass
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Estimated Effort

6 hours

## Dependencies

- TASK-071 (ash-engine crate)

## Blocked By

- TASK-071

## Blocks

None
