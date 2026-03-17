# TASK-092: Stream Execution

## Status: 🔴 Not Started

## Description

Implement execution of receive construct with pattern matching and guard evaluation.

## Specification Reference

- SPEC-013: Streams and Event Processing - Section 6 Execution Semantics

## Requirements

### Functional Requirements

1. Execute `receive` in non-blocking mode
2. Execute `receive wait` in blocking mode
3. Execute `receive wait DURATION` with timeout
4. Pattern matching with destructuring
5. Guard clause evaluation
6. Control stream handling
7. Mailbox management during execution

### Property Requirements

```rust
// Non-blocking returns immediately if no match
execute_receive_nonblocking(empty_mailbox, patterns).is_ok_and(|r| r.is_none())

// Blocking waits for message
execute_receive_blocking(mailbox_with_match, patterns).await.is_ok()

// Timeout returns after duration
let start = Instant::now();
execute_receive_with_timeout(empty_mailbox, patterns, 100ms).await;
assert!(start.elapsed() >= 100ms);

// Pattern matching selects correct arm
let result = execute_receive(mailbox, arms).await;
assert!(selected_correct_arm(result, mailbox_contents));
```

## TDD Steps

### Step 1: Write Tests (Red)

Create tests in `crates/ash-interp/src/execute_stream.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_receive_non_blocking_no_match() {
        let ctx = Context::new();
        let mailbox = Arc::new(Mutex::new(Mailbox::new()));
        
        let receive = Receive {
            mode: ReceiveMode::NonBlocking,
            arms: vec![ReceiveArm {
                pattern: Pattern::Stream(StreamPattern::Binding { ... }),
                guard: None,
                body: Workflow::Ret { ... },
            }],
            is_control: false,
        };
        
        // No messages in mailbox, no wildcard
        let result = execute_receive(&receive, ctx, mailbox, ...).await;
        // Should return without executing any arm
    }

    #[tokio::test]
    async fn test_receive_non_blocking_with_wildcard() {
        let ctx = Context::new();
        let mailbox = Arc::new(Mutex::new(Mailbox::new()));
        
        let receive = Receive {
            mode: ReceiveMode::NonBlocking,
            arms: vec![
                ReceiveArm { pattern: Pattern::Stream(...), body: arm1 },
                ReceiveArm { pattern: Pattern::Wildcard, body: arm2 },
            ],
            is_control: false,
        };
        
        let result = execute_receive(&receive, ctx, mailbox, ...).await.unwrap();
        // Should execute wildcard arm when no match
    }

    #[tokio::test]
    async fn test_receive_blocking_waits() {
        let ctx = Context::new();
        let mailbox = Arc::new(Mutex::new(Mailbox::new()));
        let mailbox_clone = mailbox.clone();
        
        // Spawn a task to add a message after delay
        tokio::spawn(async move {
            sleep(Duration::from_millis(50)).await;
            mailbox_clone.lock().await.push(MailboxEntry::new(...));
        });
        
        let receive = Receive {
            mode: ReceiveMode::Blocking(None),
            arms: vec![ReceiveArm { ... }],
            is_control: false,
        };
        
        let start = Instant::now();
        let result = execute_receive(&receive, ctx, mailbox, ...).await;
        assert!(start.elapsed() >= Duration::from_millis(50));
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_receive_with_timeout() {
        let ctx = Context::new();
        let mailbox = Arc::new(Mutex::new(Mailbox::new()));
        
        let receive = Receive {
            mode: ReceiveMode::Blocking(Some(Duration::from_millis(100))),
            arms: vec![
                ReceiveArm { pattern: Pattern::Stream(...), body: arm1 },
                ReceiveArm { pattern: Pattern::Wildcard, body: timeout_arm },
            ],
            is_control: false,
        };
        
        let start = Instant::now();
        let result = execute_receive(&receive, ctx, mailbox, ...).await;
        assert!(start.elapsed() >= Duration::from_millis(100));
        // Should execute wildcard arm on timeout
    }

    #[tokio::test]
    async fn test_receive_pattern_matching() {
        let ctx = Context::new();
        let mailbox = Arc::new(Mutex::new(Mailbox::new()));
        mailbox.lock().await.push(MailboxEntry::new(
            "kafka", "orders", 
            Value::Record(hashmap! { "priority".into() => Value::String("urgent".into()) })
        ));
        
        let receive = Receive {
            arms: vec![
                ReceiveArm {
                    pattern: Pattern::Stream(StreamPattern::Binding {
                        source: "kafka",
                        channel: "orders",
                        bind_pattern: Pattern::Record(...), // match priority: "urgent"
                    }),
                    guard: None,
                    body: urgent_workflow,
                },
                ReceiveArm {
                    pattern: Pattern::Stream(StreamPattern::Binding { ... }),
                    guard: None,
                    body: normal_workflow,
                },
            ],
            ...
        };
        
        let result = execute_receive(&receive, ctx, mailbox, ...).await;
        // Should select urgent_workflow based on pattern match
    }

    #[tokio::test]
    async fn test_receive_guard_evaluation() {
        let ctx = Context::new();
        let mut ctx_with_temp = ctx.extend();
        ctx_with_temp.set("threshold".to_string(), Value::Int(100));
        
        let mailbox = Arc::new(Mutex::new(Mailbox::new()));
        mailbox.lock().await.push(MailboxEntry::new(
            "sensor", "temp", 
            Value::Record(hashmap! { "value".into() => Value::Int(150) })
        ));
        
        let receive = Receive {
            arms: vec![
                ReceiveArm {
                    pattern: Pattern::Stream(...),
                    guard: Some(Expr::Binary { // t.value > 100
                        op: BinaryOp::Gt,
                        left: Box::new(Expr::FieldAccess { ... }),
                        right: Box::new(Expr::Literal(Value::Int(100))),
                    }),
                    body: high_temp_workflow,
                },
                ReceiveArm {
                    pattern: Pattern::Stream(...),
                    guard: None,
                    body: normal_workflow,
                },
            ],
            ...
        };
        
        let result = execute_receive(&receive, ctx_with_temp, mailbox, ...).await;
        // Guard should pass, execute high_temp_workflow
    }

    #[tokio::test]
    async fn test_receive_control() {
        let ctx = Context::new();
        let control_mailbox = Arc::new(Mutex::new(Mailbox::new()));
        control_mailbox.lock().await.push(MailboxEntry::new(
            "control", "command",
            Value::String("shutdown".into())
        ));
        
        let receive = Receive {
            is_control: true,
            arms: vec![
                ReceiveArm {
                    pattern: Pattern::Literal(Value::String("shutdown".into())),
                    guard: None,
                    body: Workflow::Done, // Break equivalent
                },
            ],
            ...
        };
        
        let result = execute_receive_control(&receive, ctx, control_mailbox, ...).await;
        // Should match shutdown command
    }
}
```

### Step 2: Verify RED

Run: `cargo test -p ash-interp stream_execution -- --nocapture`
Expected: FAIL - executor not implemented

### Step 3: Implement (Green)

Create `crates/ash-interp/src/execute_stream.rs`:

```rust
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::{sleep, timeout};

use ash_core::{Receive, ReceiveMode, ReceiveArm, Workflow, Value};
use crate::context::Context;
use crate::mailbox::{Mailbox, SharedMailbox};
use crate::stream::StreamContext;
use crate::pattern::match_pattern;
use crate::eval::eval_expr;
use crate::execute::execute_workflow;
use crate::ExecResult;

/// Execute a receive construct
pub async fn execute_receive(
    receive: &Receive,
    ctx: Context,
    mailbox: SharedMailbox,
    stream_ctx: &StreamContext,
    cap_ctx: &CapabilityContext,
    policy_eval: &PolicyEvaluator,
) -> ExecResult<Value> {
    if receive.is_control {
        return execute_receive_control(receive, ctx, mailbox, cap_ctx, policy_eval).await;
    }
    
    loop {
        // 1. Try to match existing mailbox entries
        {
            let mut mb = mailbox.lock().await;
            for arm in &receive.arms {
                if let Some(entry) = find_matching_entry(&*mb, &arm.pattern) {
                    // Check guard if present
                    if let Some(guard) = &arm.guard {
                        let guard_ctx = build_guard_ctx(&ctx, &entry.value, &arm.pattern);
                        if !eval_guard(guard, &guard_ctx)? {
                            continue; // Guard failed, try next arm
                        }
                    }
                    
                    // Match found - remove from mailbox and execute
                    let entry = mb.remove_matching(|e| std::ptr::eq(e, entry)).unwrap();
                    let arm_ctx = build_arm_context(ctx, entry.value, &arm.pattern)?;
                    return execute_workflow(&arm.body, arm_ctx, cap_ctx, policy_eval).await;
                }
            }
            
            // Check for wildcard pattern (always matches)
            if let Some(wildcard_arm) = receive.arms.iter().find(|a| is_wildcard(&a.pattern)) {
                return execute_workflow(&wildcard_arm.body, ctx, cap_ctx, policy_eval).await;
            }
        }
        
        // 2. Handle based on mode
        match receive.mode {
            ReceiveMode::NonBlocking => {
                // No match, no wildcard - return null
                return Ok(Value::Null);
            }
            ReceiveMode::Blocking(None) => {
                // Block forever until message arrives
                wait_for_message(mailbox.clone(), stream_ctx).await?;
                // Loop back to retry matching
            }
            ReceiveMode::Blocking(Some(duration)) => {
                // Block with timeout
                match timeout(duration, wait_for_message(mailbox.clone(), stream_ctx)).await {
                    Ok(Ok(())) => {
                        // Message arrived, retry matching
                    }
                    Ok(Err(e)) => return Err(e),
                    Err(_) => {
                        // Timeout - check for timeout arm (wildcard)
                        if let Some(wildcard) = receive.arms.iter().find(|a| is_wildcard(&a.pattern)) {
                            return execute_workflow(&wildcard.body, ctx, cap_ctx, policy_eval).await;
                        }
                        return Ok(Value::Null);
                    }
                }
            }
        }
    }
}

async fn execute_receive_control(
    receive: &Receive,
    ctx: Context,
    control_mailbox: SharedMailbox,
    cap_ctx: &CapabilityContext,
    policy_eval: &PolicyEvaluator,
) -> ExecResult<Value> {
    // Control receive is always non-blocking
    let mb = control_mailbox.lock().await;
    
    for arm in &receive.arms {
        if let Some(entry) = find_matching_entry(&*mb, &arm.pattern) {
            let entry = mb.remove_matching(|e| std::ptr::eq(e, entry)).unwrap();
            let arm_ctx = build_arm_context(ctx, entry.value, &arm.pattern)?;
            return execute_workflow(&arm.body, arm_ctx, cap_ctx, policy_eval).await;
        }
    }
    
    // No control message matched - return null (continue)
    Ok(Value::Null)
}

async fn wait_for_message(
    mailbox: SharedMailbox,
    stream_ctx: &StreamContext,
) -> ExecResult<()> {
    // Poll all registered streams until one has a message
    loop {
        for (cap, channel) in stream_ctx.registered_streams() {
            if let Some(provider) = stream_ctx.get(cap, channel) {
                if let Some(result) = provider.try_recv() {
                    let value = result?;
                    let entry = MailboxEntry::new(cap, channel, value);
                    mailbox.lock().await.push(entry);
                    return Ok(());
                }
            }
        }
        
        // Yield to avoid busy-waiting
        sleep(Duration::from_millis(1)).await;
    }
}

fn find_matching_entry(mailbox: &Mailbox, pattern: &Pattern) -> Option<&MailboxEntry> {
    mailbox.find_matching(|entry| matches_pattern(entry, pattern))
}

fn matches_pattern(entry: &MailboxEntry, pattern: &Pattern) -> bool {
    // Implementation of pattern matching
    match pattern {
        Pattern::Stream(stream_pat) => matches_stream_pattern(entry, stream_pat),
        Pattern::Wildcard => true,
        Pattern::Literal(val) => entry.value == *val,
        _ => false,
    }
}

fn is_wildcard(pattern: &Pattern) -> bool {
    matches!(pattern, Pattern::Wildcard)
}

fn eval_guard(guard: &Expr, ctx: &Context) -> Result<bool, EvalError> {
    let value = eval_expr(guard, ctx)?;
    match value {
        Value::Bool(b) => Ok(b),
        _ => Err(EvalError::TypeMismatch { ... }),
    }
}
```

### Step 4: Verify GREEN

Run: `cargo test -p ash-interp stream_execution -- --nocapture`
Expected: PASS

### Step 5: Commit

```bash
git add crates/ash-interp/src/execute_stream.rs
git commit -m "feat: stream execution with pattern matching and guards"
```

## Completion Checklist

- [ ] Non-blocking receive execution
- [ ] Blocking receive with wait
- [ ] Timeout handling
- [ ] Pattern matching in mailbox
- [ ] Guard clause evaluation
- [ ] Control receive handling
- [ ] Tests pass
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Estimated Effort

8 hours

## Dependencies

- TASK-089 (Stream provider)
- TASK-090 (Parse receive)
- TASK-091 (Mailbox implementation)

## Blocked By

- TASK-091

## Blocks

None (completes streams)
