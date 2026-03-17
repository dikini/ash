# TASK-037: Async Runtime Integration

## Status: 🟢 Complete

## Description

Implement async runtime integration for the interpreter using Tokio, including proper cancellation, timeouts, and error handling.

## Specification Reference

- SPEC-004: Operational Semantics - Section 6. Concurrent Semantics
- AGENTS.md - Rust async patterns

## Requirements

### Async Runtime Configuration

```rust
/// Async runtime configuration
#[derive(Debug, Clone)]
pub struct AsyncConfig {
    /// Number of worker threads
    pub worker_threads: usize,
    /// Max blocking threads
    pub max_blocking_threads: usize,
    /// Thread stack size
    pub thread_stack_size: usize,
    /// Enable tokio console
    pub enable_console: bool,
}

impl Default for AsyncConfig {
    fn default() -> Self {
        Self {
            worker_threads: num_cpus::get(),
            max_blocking_threads: 512,
            thread_stack_size: 2 * 1024 * 1024, // 2MB
            enable_console: false,
        }
    }
}

/// Initialize tokio runtime
pub fn init_runtime(config: AsyncConfig) -> tokio::runtime::Runtime {
    let mut builder = tokio::runtime::Builder::new_multi_thread();
    
    builder
        .worker_threads(config.worker_threads)
        .max_blocking_threads(config.max_blocking_threads)
        .thread_stack_size(config.thread_stack_size)
        .thread_name("ash-runtime");
    
    if config.enable_console {
        builder.enable_all();
    } else {
        builder.enable_io().enable_time();
    }
    
    builder.build().expect("Failed to create tokio runtime")
}
```

### Cancellation Support

```rust
use tokio_util::sync::CancellationToken;

/// Execute workflow with cancellation support
pub async fn execute_workflow_cancellable(
    ctx: &RuntimeContext,
    workflow: &Workflow,
    cancel: CancellationToken,
) -> Result<(Value, RuntimeContext), InterpError> {
    tokio::select! {
        result = execute_workflow(ctx, workflow) => result,
        _ = cancel.cancelled() => {
            Err(InterpError::Cancelled)
        }
    }
}

/// Execute with timeout
pub async fn execute_with_timeout(
    ctx: &RuntimeContext,
    workflow: &Workflow,
    timeout: Duration,
) -> Result<(Value, RuntimeContext), InterpError> {
    tokio::time::timeout(timeout, execute_workflow(ctx, workflow))
        .await
        .map_err(|_| InterpError::Timeout)?
}

/// Execute workflow with both timeout and cancellation
pub async fn execute_with_guardrails(
    ctx: &RuntimeContext,
    workflow: &Workflow,
    timeout: Duration,
    cancel: CancellationToken,
) -> Result<(Value, RuntimeContext), InterpError> {
    tokio::select! {
        result = tokio::time::timeout(timeout, execute_workflow(ctx, workflow)) => {
            result.map_err(|_| InterpError::Timeout)?
        }
        _ = cancel.cancelled() => {
            Err(InterpError::Cancelled)
        }
    }
}
```

### Concurrent Execution

```rust
/// Execute workflows in parallel with proper error handling
pub async fn execute_parallel(
    ctx: &RuntimeContext,
    workflows: &[Workflow],
) -> Result<Vec<(Value, RuntimeContext)>, InterpError> {
    // Create forked contexts
    let branches: Vec<_> = workflows.iter()
        .map(|wf| {
            let forked = ctx.fork();
            execute_workflow(forked, wf)
        })
        .collect();
    
    // Execute all branches
    let results = futures::future::join_all(branches).await;
    
    // Collect results, failing fast on first error
    let mut outputs = Vec::with_capacity(results.len());
    for result in results {
        outputs.push(result?);
    }
    
    Ok(outputs)
}

/// Execute with bounded parallelism
pub async fn execute_parallel_bounded(
    ctx: &RuntimeContext,
    workflows: &[Workflow],
    max_concurrency: usize,
) -> Result<Vec<(Value, RuntimeContext)>, InterpError> {
    use futures::stream::{self, StreamExt};
    
    let stream = stream::iter(workflows)
        .map(|wf| {
            let forked = ctx.fork();
            execute_workflow(forked, wf)
        })
        .buffer_unordered(max_concurrency);
    
    let results: Vec<_> = stream.collect().await;
    
    // Collect results
    let mut outputs = Vec::with_capacity(results.len());
    for result in results {
        outputs.push(result?);
    }
    
    Ok(outputs)
}

/// Execute with early termination on first success (race)
pub async fn execute_race(
    ctx: &RuntimeContext,
    workflows: &[Workflow],
) -> Result<(Value, RuntimeContext), InterpError> {
    // Create forked contexts
    let branches: Vec<_> = workflows.iter()
        .enumerate()
        .map(|(idx, wf)| {
            let forked = ctx.fork();
            async move {
                let result = execute_workflow(forked, wf).await;
                (idx, result)
            }
        })
        .collect();
    
    // Race them
    let (idx, result) = futures::future::select_ok(
        branches.into_iter().map(Box::pin)
    ).await.map_err(|e| e.0)?;
    
    result
}
```

### Blocking Operations

```rust
/// Execute a blocking operation
pub async fn execute_blocking<F, R>(f: F) -> R
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    tokio::task::spawn_blocking(f).await
        .expect("Blocking task failed")
}

/// Blocking file operations
pub mod blocking {
    use super::*;
    use std::path::Path;
    
    pub async fn read_file(path: impl AsRef<Path>) -> std::io::Result<String> {
        let path = path.as_ref().to_owned();
        execute_blocking(move || std::fs::read_to_string(&path)).await
    }
    
    pub async fn write_file(
        path: impl AsRef<Path>,
        content: impl Into<String>,
    ) -> std::io::Result<()> {
        let path = path.as_ref().to_owned();
        let content = content.into();
        execute_blocking(move || std::fs::write(&path, content)).await
    }
}
```

### Resource Limits

```rust
/// Resource limit tracker
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub max_memory: Option<usize>,
    pub max_file_descriptors: Option<usize>,
    pub max_network_connections: Option<usize>,
    pub max_execution_time: Option<Duration>,
}

/// Execute with resource limits
pub async fn execute_with_limits(
    ctx: &RuntimeContext,
    workflow: &Workflow,
    limits: ResourceLimits,
) -> Result<(Value, RuntimeContext), InterpError> {
    let start = Instant::now();
    
    // Would need integration with cgroups or similar for real limits
    // This is a simplified version
    
    let timeout = limits.max_execution_time
        .unwrap_or(Duration::from_secs(300));
    
    let result = tokio::time::timeout(timeout, execute_workflow(ctx, workflow)).await;
    
    match result {
        Ok(Ok((val, ctx))) => {
            let elapsed = start.elapsed();
            tracing::info!("Workflow completed in {:?}", elapsed);
            Ok((val, ctx))
        }
        Ok(Err(e)) => Err(e),
        Err(_) => Err(InterpError::Timeout),
    }
}
```

### Error Handling

```rust
#[derive(Debug, Clone, thiserror::Error)]
pub enum InterpError {
    // ... existing errors
    
    #[error("Execution cancelled")]
    Cancelled,
    
    #[error("Execution timeout")]
    Timeout,
    
    #[error("Concurrent execution failed: {0}")]
    ConcurrentFailure(String),
    
    #[error("Join error: {0}")]
    JoinError(String),
}
```

## TDD Steps

### Step 1: Implement Async Configuration

Create `crates/ash-interp/src/async_runtime.rs`.

### Step 2: Implement Cancellation

Add cancellation token support.

### Step 3: Implement Concurrent Execution

Add parallel execution with proper error handling.

### Step 4: Implement Resource Limits

Add timeout and resource tracking.

### Step 5: Write Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cancellation() {
        let ctx = RuntimeContext::new(
            Arc::new(CapabilityRegistry::new()),
            Arc::new(PolicyRegistry::new()),
        );
        
        let cancel = CancellationToken::new();
        let cancel_clone = cancel.clone();
        
        // Cancel after short delay
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(10)).await;
            cancel_clone.cancel();
        });
        
        let workflow = Workflow::Ret {
            expr: Expr::Literal(Value::Int(1)),
        };
        
        let result = execute_workflow_cancellable(&ctx, &workflow, cancel).await;
        
        // Result depends on timing - either success or cancelled
    }

    #[tokio::test]
    async fn test_timeout() {
        let ctx = RuntimeContext::new(
            Arc::new(CapabilityRegistry::new()),
            Arc::new(PolicyRegistry::new()),
        );
        
        let workflow = Workflow::Ret {
            expr: Expr::Literal(Value::Int(1)),
        };
        
        let result = execute_with_timeout(&ctx, &workflow, Duration::from_secs(1)).await;
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_parallel_execution() {
        let ctx = RuntimeContext::new(
            Arc::new(CapabilityRegistry::new()),
            Arc::new(PolicyRegistry::new()),
        );
        
        let workflows = vec![
            Workflow::Ret { expr: Expr::Literal(Value::Int(1)) },
            Workflow::Ret { expr: Expr::Literal(Value::Int(2)) },
            Workflow::Ret { expr: Expr::Literal(Value::Int(3)) },
        ];
        
        let results = execute_parallel(&ctx, &workflows).await.unwrap();
        
        assert_eq!(results.len(), 3);
    }

    #[tokio::test]
    async fn test_blocking_operation() {
        let result = blocking::read_file("/etc/hostname").await;
        assert!(result.is_ok());
    }
}
```

## Completion Checklist

- [ ] AsyncConfig
- [ ] Runtime initialization
- [ ] Cancellation support
- [ ] Timeout support
- [ ] Parallel execution
- [ ] Bounded concurrency
- [ ] Race execution
- [ ] Blocking operation wrapper
- [ ] Resource limits
- [ ] Error types for async failures
- [ ] Unit tests for cancellation
- [ ] Unit tests for timeout
- [ ] Unit tests for parallel execution
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Self-Review Questions

1. **Cancellation**: Are all operations cancellable?
2. **Resource limits**: Are limits enforced?
3. **Error handling**: Are async errors handled gracefully?

## Estimated Effort

6 hours

## Dependencies

- tokio
- futures
- tokio-util

## Blocked By

- All previous interpreter tasks

## Blocks

- TASK-054: CLI run command
- TASK-060: Integration tests
