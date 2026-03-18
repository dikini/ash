# TASK-076: Update CLI to Use ash-engine

## Status: 🟢 Complete

## Description

Update ash-cli to use ash-engine instead of direct crate dependencies.

## Specification Reference

- SPEC-010: Embedding API

## Requirements

### Functional Requirements

1. Replace direct parser/typeck/interp calls with Engine API
2. Update `ash run` to use Engine::run_file
3. Update `ash check` to use Engine::parse + Engine::check

### Property Requirements

```rust
// CLI uses Engine
cli::run("file.ash").await == engine.run_file("file.ash").await
```

## TDD Steps

### Step 1: Write Tests (Red)

Existing integration tests should continue to work after migration.

### Step 2: Refactor CLI (Green)

Update `crates/ash-cli/src/commands/run.rs`:

```rust
use ash_engine::Engine;

pub async fn run(path: &Path) -> Result<()> {
    let engine = Engine::new()
        .with_stdio_capabilities()
        .with_fs_capabilities()
        .build()?;
    
    let result = engine.run_file(path).await?;
    println!("{:?}", result);
    Ok(())
}
```

## Completion Checklist

- [ ] ash-cli depends on ash-engine
- [ ] `ash run` uses Engine
- [ ] `ash check` uses Engine
- [ ] All existing tests pass
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Estimated Effort

4 hours

## Dependencies

- TASK-071 through TASK-075 (Engine complete)

## Blocked By

- TASK-075

## Blocks

None (completes Phase 11)
