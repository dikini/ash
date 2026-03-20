//! Stream execution with pattern matching and guards
//!
//! This module provides execution of receive constructs with pattern matching
//! and guard evaluation for stream message handling.

use std::time::Duration;

use tokio::time::{sleep, timeout};

use ash_core::ast::{ReceiveArm as CoreReceiveArm, ReceiveMode as CoreReceiveMode, ReceivePattern};
use ash_core::stream::{MailboxEntry, Receive, ReceiveArm, ReceiveMode};
use ash_core::{Expr, Pattern, Value};

use crate::behaviour::BehaviourContext;
use crate::capability::CapabilityContext;
use crate::context::Context;
use crate::control_link::ControlLinkRegistry;
use crate::error::{EvalError, ExecError, ExecResult};
use crate::eval::eval_expr;
use crate::execute::execute_workflow_inner;
use crate::mailbox::{Mailbox, SharedMailbox};
use crate::pattern::match_pattern;
use crate::policy::PolicyEvaluator;
use crate::runtime_state::RuntimeState;
use crate::stream::StreamContext;

const CONTROL_CAPABILITY: &str = "__control__";
const CONTROL_CHANNEL: &str = "__mailbox__";

pub struct CoreReceiveRuntime<'a> {
    pub mailbox: SharedMailbox,
    pub control_registry: std::sync::Arc<tokio::sync::Mutex<ControlLinkRegistry>>,
    pub stream_ctx: &'a StreamContext,
    pub cap_ctx: &'a CapabilityContext,
    pub policy_eval: &'a PolicyEvaluator,
    pub behaviour_ctx: &'a BehaviourContext,
}

/// Execute a canonical core receive using the shared stream-aware execution path.
pub async fn execute_core_receive(
    mode: &CoreReceiveMode,
    arms: &[CoreReceiveArm],
    control: bool,
    ctx: Context,
    runtime: CoreReceiveRuntime<'_>,
) -> ExecResult<Value> {
    if !control {
        verify_stream_bindings_available(arms, runtime.stream_ctx)?;
    }
    let stream_sources = collect_core_stream_sources(arms);

    loop {
        {
            let mut mb = runtime.mailbox.lock().await;
            for arm in arms {
                if let Some(entry) = find_matching_core_entry(&mb, &arm.pattern, control) {
                    if let Some(ref guard_expr) = arm.guard {
                        let guard_ctx = build_core_guard_context(&ctx, entry, &arm.pattern)?;
                        if !eval_guard_expr(guard_expr, &guard_ctx)? {
                            continue;
                        }
                    }

                    let entry_value = entry.value.clone();
                    let entry_source = entry.source().to_string();
                    let entry_channel = entry.channel().to_string();

                    mb.remove_matching(|e| {
                        e.source() == entry_source
                            && e.channel() == entry_channel
                            && e.value == entry_value
                    });

                    let arm_ctx = build_core_arm_context(ctx, &entry_value, &arm.pattern)?;
                    return execute_workflow_inner(
                        &arm.body,
                        arm_ctx,
                        runtime.cap_ctx,
                        runtime.policy_eval,
                        runtime.behaviour_ctx,
                        Some(runtime.stream_ctx),
                        runtime.mailbox.clone(),
                        runtime.control_registry.clone(),
                    )
                    .await;
                }
            }
        }

        if pump_available_core_message(
            runtime.mailbox.clone(),
            runtime.stream_ctx,
            control,
            &stream_sources,
        )
        .await?
        {
            continue;
        }

        match mode {
            CoreReceiveMode::NonBlocking => {
                if let Some(wildcard_arm) = arms.iter().find(|arm| is_core_wildcard(&arm.pattern)) {
                    return execute_workflow_inner(
                        &wildcard_arm.body,
                        ctx,
                        runtime.cap_ctx,
                        runtime.policy_eval,
                        runtime.behaviour_ctx,
                        Some(runtime.stream_ctx),
                        runtime.mailbox.clone(),
                        runtime.control_registry.clone(),
                    )
                    .await;
                }
                return Ok(Value::Null);
            }
            CoreReceiveMode::Blocking(None) => {
                wait_for_core_message(
                    runtime.mailbox.clone(),
                    runtime.stream_ctx,
                    control,
                    &stream_sources,
                )
                .await?;
            }
            CoreReceiveMode::Blocking(Some(duration)) => {
                match timeout(
                    *duration,
                    wait_for_core_message(
                        runtime.mailbox.clone(),
                        runtime.stream_ctx,
                        control,
                        &stream_sources,
                    ),
                )
                .await
                {
                    Ok(Ok(())) => {}
                    Ok(Err(error)) => return Err(error),
                    Err(_) => {
                        if let Some(wildcard_arm) =
                            arms.iter().find(|arm| is_core_wildcard(&arm.pattern))
                        {
                            return execute_workflow_inner(
                                &wildcard_arm.body,
                                ctx,
                                runtime.cap_ctx,
                                runtime.policy_eval,
                                runtime.behaviour_ctx,
                                Some(runtime.stream_ctx),
                                runtime.mailbox.clone(),
                                runtime.control_registry.clone(),
                            )
                            .await;
                        }
                        return Ok(Value::Null);
                    }
                }
            }
        }
    }
}

/// Execute a receive construct with pattern matching and guard evaluation
///
/// This function handles the main receive logic:
/// - For non-blocking mode: returns immediately if no match found
/// - For blocking mode: waits indefinitely for a matching message
/// - For timeout mode: waits up to the specified duration
///
/// # Arguments
/// * `receive` - The receive construct to execute
/// * `ctx` - The runtime context with variable bindings
/// * `mailbox` - The shared mailbox for message buffering
/// * `stream_ctx` - The stream context for receiving from providers
/// * `cap_ctx` - The capability context for workflow execution
/// * `policy_eval` - The policy evaluator for workflow execution
///
/// # Returns
/// The result of executing the matching arm's body, or `Value::Null` if no match
pub async fn execute_receive(
    receive: &Receive,
    ctx: Context,
    mailbox: SharedMailbox,
    stream_ctx: &StreamContext,
    cap_ctx: &CapabilityContext,
    policy_eval: &PolicyEvaluator,
) -> ExecResult<Value> {
    let runtime_state = RuntimeState::new();
    execute_receive_in_state(
        receive,
        ctx,
        mailbox,
        stream_ctx,
        cap_ctx,
        policy_eval,
        &runtime_state,
    )
    .await
}

pub async fn execute_receive_in_state(
    receive: &Receive,
    ctx: Context,
    mailbox: SharedMailbox,
    stream_ctx: &StreamContext,
    cap_ctx: &CapabilityContext,
    policy_eval: &PolicyEvaluator,
    runtime_state: &RuntimeState,
) -> ExecResult<Value> {
    let control_registry = runtime_state.control_registry();
    // Check control arms first if present (non-blocking check)
    if let Some(ref control_arms) = receive.control_arms {
        let control_result = execute_receive_control(
            control_arms,
            ctx.clone(),
            mailbox.clone(),
            control_registry.clone(),
            stream_ctx,
            cap_ctx,
            policy_eval,
        )
        .await?;
        // If a control arm matched and returned a value other than Null, return it
        // (e.g., a "break" or "shutdown" command)
        if control_result != Value::Null {
            return Ok(control_result);
        }
    }

    loop {
        // 1. Try to match existing mailbox entries
        {
            let mut mb = mailbox.lock().await;
            for arm in &receive.arms {
                if let Some(entry) = find_matching_entry(&mb, &arm.pattern) {
                    // Check guard if present
                    if let Some(ref guard_expr) = arm.guard {
                        // Build context with pattern bindings for guard evaluation
                        let guard_ctx = build_guard_context(&ctx, entry, &arm.pattern)?;
                        if !eval_guard_expr(guard_expr, &guard_ctx)? {
                            continue; // Guard failed, try next arm
                        }
                    }

                    // Match found - remove from mailbox and execute
                    let entry_value = entry.value.clone();
                    let entry_source = entry.source().to_string();
                    let entry_channel = entry.channel().to_string();

                    // Remove the matching entry
                    mb.remove_matching(|e| {
                        e.source() == entry_source
                            && e.channel() == entry_channel
                            && e.value == entry_value
                    });

                    // Build arm context with bindings
                    let arm_ctx = build_arm_context(ctx, &entry_value, &arm.pattern)?;
                    let behaviour_ctx = BehaviourContext::new();
                    return execute_workflow_inner(
                        &arm.body,
                        arm_ctx,
                        cap_ctx,
                        policy_eval,
                        &behaviour_ctx,
                        Some(stream_ctx),
                        mailbox.clone(),
                        control_registry.clone(),
                    )
                    .await;
                }
            }

            // Check for wildcard pattern (always matches) - only in non-blocking mode
            // In blocking modes, we need to wait for a real message or timeout
            if receive.mode.is_non_blocking()
                && let Some(wildcard_arm) = receive.arms.iter().find(|a| is_wildcard(&a.pattern))
            {
                let behaviour_ctx = BehaviourContext::new();
                return execute_workflow_inner(
                    &wildcard_arm.body,
                    ctx,
                    cap_ctx,
                    policy_eval,
                    &behaviour_ctx,
                    Some(stream_ctx),
                    mailbox.clone(),
                    control_registry.clone(),
                )
                .await;
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
                        if let Some(wildcard) =
                            receive.arms.iter().find(|a| is_wildcard(&a.pattern))
                        {
                            let behaviour_ctx = BehaviourContext::new();
                            return execute_workflow_inner(
                                &wildcard.body,
                                ctx,
                                cap_ctx,
                                policy_eval,
                                &behaviour_ctx,
                                Some(stream_ctx),
                                mailbox.clone(),
                                control_registry.clone(),
                            )
                            .await;
                        }
                        return Ok(Value::Null);
                    }
                }
            }
        }
    }
}

/// Execute control receive (non-blocking)
///
/// Control receive checks the mailbox for control messages without blocking.
/// It's used to handle system-level messages like shutdown commands.
///
/// # Arguments
/// * `control_arms` - The control arms to match against
/// * `ctx` - The runtime context with variable bindings
/// * `mailbox` - The shared mailbox for message buffering
/// * `cap_ctx` - The capability context for workflow execution
/// * `policy_eval` - The policy evaluator for workflow execution
async fn execute_receive_control(
    control_arms: &[ReceiveArm],
    ctx: Context,
    mailbox: SharedMailbox,
    control_registry: std::sync::Arc<tokio::sync::Mutex<ControlLinkRegistry>>,
    stream_ctx: &StreamContext,
    cap_ctx: &CapabilityContext,
    policy_eval: &PolicyEvaluator,
) -> ExecResult<Value> {
    let mb = mailbox.lock().await;

    for arm in control_arms {
        if let Some(entry) = find_matching_entry(&mb, &arm.pattern) {
            // Clone all needed data before releasing the lock
            let entry_value = entry.value.clone();
            let entry_source = entry.source().to_string();
            let entry_channel = entry.channel().to_string();
            let arm_pattern = arm.pattern.clone();
            let arm_body = arm.body.clone();
            drop(mb);

            // Remove the entry from mailbox
            let mut mb = mailbox.lock().await;
            mb.remove_matching(|e| {
                e.source() == entry_source && e.channel() == entry_channel && e.value == entry_value
            });
            drop(mb);

            let arm_ctx = build_arm_context(ctx, &entry_value, &arm_pattern)?;
            let behaviour_ctx = BehaviourContext::new();
            return execute_workflow_inner(
                &arm_body,
                arm_ctx,
                cap_ctx,
                policy_eval,
                &behaviour_ctx,
                Some(stream_ctx),
                mailbox.clone(),
                control_registry.clone(),
            )
            .await;
        }
    }

    // No control message matched - return null (continue)
    Ok(Value::Null)
}

/// Wait for a new message to arrive from any registered stream
///
/// Polls all registered streams until one has a message available,
/// then adds it to the mailbox.
///
/// # Arguments
/// * `mailbox` - The shared mailbox to add the message to
/// * `stream_ctx` - The stream context containing registered providers
///
/// # Returns
/// Ok(()) when a message has been added to the mailbox
async fn wait_for_message(mailbox: SharedMailbox, stream_ctx: &StreamContext) -> ExecResult<()> {
    // Poll all registered streams until one has a message
    loop {
        // Try to receive from any available stream
        if let Some((cap, chan, result)) = stream_ctx.try_recv_any() {
            let mut mb = mailbox.lock().await;
            match result {
                Ok(value) => {
                    mb.push(MailboxEntry::new(cap.clone(), chan.clone(), value))?;
                }
                Err(e) => {
                    // Log error but continue polling - don't crash on stream errors
                    eprintln!("Stream error from {cap}:{chan}: {e}");
                }
            }
            return Ok(());
        }

        // Yield to avoid busy-waiting
        sleep(Duration::from_millis(1)).await;

        // Also check if any message was already in mailbox
        let mb = mailbox.lock().await;
        if !mb.is_empty() {
            return Ok(());
        }
    }
}

/// Find the first mailbox entry matching the given pattern
///
/// # Arguments
/// * `mailbox` - The mailbox to search
/// * `pattern` - The pattern to match against
///
/// # Returns
/// Some(&MailboxEntry) if a match is found, None otherwise
fn find_matching_entry<'a>(mailbox: &'a Mailbox, pattern: &Pattern) -> Option<&'a MailboxEntry> {
    mailbox.find_matching(|entry| matches_pattern_entry(entry, pattern))
}

/// Check if a mailbox entry matches the given pattern
///
/// # Arguments
/// * `entry` - The mailbox entry to check
/// * `pattern` - The pattern to match against
///
/// # Returns
/// true if the entry matches the pattern
fn matches_pattern_entry(entry: &MailboxEntry, pattern: &Pattern) -> bool {
    match pattern {
        Pattern::Wildcard => true,
        Pattern::Literal(val) => entry.value == *val,
        Pattern::Variable(_) => true, // Variable matches any value
        Pattern::Tuple(patterns) => match &entry.value {
            Value::List(values) => {
                if values.len() != patterns.len() {
                    return false;
                }
                patterns.iter().zip(values.iter()).all(|(p, v)| {
                    // For tuple elements, check if they match
                    match p {
                        Pattern::Literal(lit) => lit == v,
                        Pattern::Variable(_) => true,
                        Pattern::Wildcard => true,
                        _ => false,
                    }
                })
            }
            _ => false,
        },
        Pattern::Record(field_patterns) => match &entry.value {
            Value::Record(fields) => field_patterns.iter().all(|(field_name, field_pattern)| {
                match fields.get(field_name) {
                    Some(field_value) => match field_pattern {
                        Pattern::Literal(lit) => lit == field_value,
                        Pattern::Variable(_) => true,
                        Pattern::Wildcard => true,
                        _ => false,
                    },
                    None => false,
                }
            }),
            _ => false,
        },
        Pattern::List(prefix_patterns, _rest) => match &entry.value {
            Value::List(values) => {
                if values.len() < prefix_patterns.len() {
                    return false;
                }
                prefix_patterns
                    .iter()
                    .zip(values.iter())
                    .all(|(p, v)| match p {
                        Pattern::Literal(lit) => lit == v,
                        Pattern::Variable(_) => true,
                        Pattern::Wildcard => true,
                        _ => false,
                    })
            }
            _ => false,
        },
        Pattern::Variant { .. } => {
            // TODO: Implement variant pattern matching in TASK-132
            false
        }
    }
}

/// Check if a pattern is a wildcard
fn is_wildcard(pattern: &Pattern) -> bool {
    matches!(pattern, Pattern::Wildcard)
}

/// Evaluate a guard expression in the given context
///
/// # Arguments
/// * `guard` - The guard expression to evaluate
/// * `ctx` - The context with variable bindings
///
/// # Returns
/// Ok(true) if the guard evaluates to true, Ok(false) otherwise
fn eval_guard_expr(guard: &Expr, ctx: &Context) -> Result<bool, EvalError> {
    let value = eval_expr(guard, ctx)?;
    match value {
        Value::Bool(b) => Ok(b),
        _ => Err(EvalError::TypeMismatch {
            expected: "bool".to_string(),
            actual: format!("{:?}", value),
        }),
    }
}

fn verify_stream_bindings_available(
    arms: &[CoreReceiveArm],
    stream_ctx: &StreamContext,
) -> ExecResult<()> {
    for arm in arms {
        if let ReceivePattern::Stream {
            capability,
            channel,
            ..
        } = &arm.pattern
            && stream_ctx.get(capability, channel).is_none()
        {
            return Err(ExecError::CapabilityNotAvailable(format!(
                "{capability}:{channel}"
            )));
        }
    }

    Ok(())
}

fn find_matching_core_entry<'a>(
    mailbox: &'a Mailbox,
    pattern: &ReceivePattern,
    control: bool,
) -> Option<&'a MailboxEntry> {
    mailbox.find_matching(|entry| matches_core_pattern_entry(entry, pattern, control))
}

fn matches_core_pattern_entry(
    entry: &MailboxEntry,
    pattern: &ReceivePattern,
    control: bool,
) -> bool {
    if control {
        if entry.source() != CONTROL_CAPABILITY || entry.channel() != CONTROL_CHANNEL {
            return false;
        }

        return match pattern {
            ReceivePattern::Literal(value) => &entry.value == value,
            ReceivePattern::Wildcard => true,
            ReceivePattern::Stream { .. } => false,
        };
    }

    match pattern {
        ReceivePattern::Stream {
            capability,
            channel,
            pattern,
        } => {
            entry.source() == capability
                && entry.channel() == channel
                && match_pattern(pattern, &entry.value).is_ok()
        }
        ReceivePattern::Literal(value) => &entry.value == value,
        ReceivePattern::Wildcard => true,
    }
}

fn is_core_wildcard(pattern: &ReceivePattern) -> bool {
    matches!(pattern, ReceivePattern::Wildcard)
}

fn build_core_guard_context(
    base_ctx: &Context,
    entry: &MailboxEntry,
    pattern: &ReceivePattern,
) -> ExecResult<Context> {
    let mut ctx = base_ctx.clone();
    if let Some(bindings) = try_bind_receive_pattern(pattern, &entry.value)? {
        ctx.set_many(bindings);
    }
    Ok(ctx)
}

fn build_core_arm_context(
    base_ctx: Context,
    value: &Value,
    pattern: &ReceivePattern,
) -> ExecResult<Context> {
    let mut ctx = base_ctx.extend();
    if let Some(bindings) = try_bind_receive_pattern(pattern, value)? {
        ctx.set_many(bindings);
    }
    Ok(ctx)
}

fn try_bind_receive_pattern(
    pattern: &ReceivePattern,
    value: &Value,
) -> ExecResult<Option<std::collections::HashMap<String, Value>>> {
    match pattern {
        ReceivePattern::Stream { pattern, .. } => {
            let bindings =
                match_pattern(pattern, value).map_err(|_| ExecError::PatternMatchFailed {
                    pattern: format!("{:?}", pattern),
                    value: value.clone(),
                })?;
            Ok(Some(bindings))
        }
        ReceivePattern::Literal(_) | ReceivePattern::Wildcard => Ok(None),
    }
}

async fn wait_for_core_message(
    mailbox: SharedMailbox,
    stream_ctx: &StreamContext,
    control: bool,
    stream_sources: &[(String, String)],
) -> ExecResult<()> {
    loop {
        if pump_available_core_message(mailbox.clone(), stream_ctx, control, stream_sources).await?
        {
            return Ok(());
        }

        sleep(Duration::from_millis(1)).await;

        let mb = mailbox.lock().await;
        if mailbox_has_relevant_core_entry(&mb, control, stream_sources) {
            return Ok(());
        }
    }
}

async fn pump_available_core_message(
    mailbox: SharedMailbox,
    stream_ctx: &StreamContext,
    control: bool,
    stream_sources: &[(String, String)],
) -> ExecResult<bool> {
    if control {
        if let Some(value) = stream_ctx.try_recv_control() {
            let mut mb = mailbox.lock().await;
            mb.push(MailboxEntry::new(
                CONTROL_CAPABILITY,
                CONTROL_CHANNEL,
                value,
            ))?;
            return Ok(true);
        }
        return Ok(false);
    }

    for (capability, channel) in stream_sources {
        if let Some(result) = stream_ctx.try_recv(capability, channel) {
            let mut mb = mailbox.lock().await;
            match result {
                Ok(value) => {
                    mb.push(MailboxEntry::new(
                        capability.clone(),
                        channel.clone(),
                        value,
                    ))?;
                }
                Err(error) => {
                    eprintln!("Stream error from receive source {capability}:{channel}: {error}");
                }
            }
            return Ok(true);
        }
    }

    Ok(false)
}

fn collect_core_stream_sources(arms: &[CoreReceiveArm]) -> Vec<(String, String)> {
    let mut sources = Vec::new();

    for arm in arms {
        if let ReceivePattern::Stream {
            capability,
            channel,
            ..
        } = &arm.pattern
            && !sources
                .iter()
                .any(|(existing_capability, existing_channel)| {
                    existing_capability == capability && existing_channel == channel
                })
        {
            sources.push((capability.clone(), channel.clone()));
        }
    }

    sources
}

fn mailbox_has_relevant_core_entry(
    mailbox: &Mailbox,
    control: bool,
    stream_sources: &[(String, String)],
) -> bool {
    mailbox.iter().any(|entry| {
        if control {
            return entry.source() == CONTROL_CAPABILITY && entry.channel() == CONTROL_CHANNEL;
        }

        stream_sources.iter().any(|(capability, channel)| {
            entry.source() == capability.as_str() && entry.channel() == channel.as_str()
        })
    })
}

/// Build a context for guard evaluation with pattern bindings
///
/// # Arguments
/// * `base_ctx` - The base context to extend
/// * `entry` - The mailbox entry containing the value
/// * `pattern` - The pattern that matched
///
/// # Returns
/// A new context with pattern bindings
fn build_guard_context(
    base_ctx: &Context,
    entry: &MailboxEntry,
    pattern: &Pattern,
) -> ExecResult<Context> {
    let mut ctx = base_ctx.clone();

    // Try to extract bindings from the pattern match
    if let Ok(bindings) = match_pattern(pattern, &entry.value) {
        ctx.set_many(bindings);
    }

    Ok(ctx)
}

/// Build a context for arm execution with pattern bindings
///
/// # Arguments
/// * `base_ctx` - The base context to extend
/// * `value` - The matched value
/// * `pattern` - The pattern that matched
///
/// # Returns
/// A new context with pattern bindings
fn build_arm_context(base_ctx: Context, value: &Value, pattern: &Pattern) -> ExecResult<Context> {
    let mut ctx = base_ctx.extend();

    // Extract bindings from the pattern match
    let bindings = match_pattern(pattern, value).map_err(|_| ExecError::PatternMatchFailed {
        pattern: format!("{:?}", pattern),
        value: value.clone(),
    })?;

    ctx.set_many(bindings);
    Ok(ctx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_core::{BinaryOp, Expr, Pattern, Workflow};
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    /// Helper to create a test mailbox with entries
    fn create_test_mailbox(entries: Vec<MailboxEntry>) -> SharedMailbox {
        let mut mailbox = Mailbox::new();
        for entry in entries {
            mailbox.push(entry).unwrap();
        }
        Arc::new(Mutex::new(mailbox))
    }

    /// Helper to create a simple receive arm
    fn create_arm(pattern: Pattern, body: Workflow) -> ReceiveArm {
        ReceiveArm {
            pattern,
            guard: None,
            body,
        }
    }

    /// Helper to create a receive arm with guard
    fn create_arm_with_guard(pattern: Pattern, guard: Expr, body: Workflow) -> ReceiveArm {
        ReceiveArm {
            pattern,
            guard: Some(guard),
            body,
        }
    }

    #[tokio::test]
    async fn test_receive_non_blocking_no_match() {
        let ctx = Context::new();
        let mailbox: SharedMailbox = Arc::new(Mutex::new(Mailbox::new()));
        let stream_ctx = StreamContext::new();
        let cap_ctx = CapabilityContext::new();
        let policy_eval = PolicyEvaluator::new();

        let receive = Receive {
            mode: ReceiveMode::NonBlocking,
            arms: vec![create_arm(
                Pattern::Literal(Value::Int(42)),
                Workflow::Ret {
                    expr: Expr::Literal(Value::String("matched".to_string())),
                },
            )],
            control_arms: None,
        };

        // No messages in mailbox, no wildcard
        let result =
            execute_receive(&receive, ctx, mailbox, &stream_ctx, &cap_ctx, &policy_eval).await;

        // Should return null (no match)
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Null);
    }

    #[tokio::test]
    async fn test_receive_non_blocking_with_wildcard() {
        let ctx = Context::new();
        let mailbox: SharedMailbox = Arc::new(Mutex::new(Mailbox::new()));
        let stream_ctx = StreamContext::new();
        let cap_ctx = CapabilityContext::new();
        let policy_eval = PolicyEvaluator::new();

        let receive = Receive {
            mode: ReceiveMode::NonBlocking,
            arms: vec![
                create_arm(
                    Pattern::Literal(Value::Int(42)),
                    Workflow::Ret {
                        expr: Expr::Literal(Value::String("specific".to_string())),
                    },
                ),
                create_arm(
                    Pattern::Wildcard,
                    Workflow::Ret {
                        expr: Expr::Literal(Value::String("wildcard".to_string())),
                    },
                ),
            ],
            control_arms: None,
        };

        // No messages in mailbox, but has wildcard
        let result =
            execute_receive(&receive, ctx, mailbox, &stream_ctx, &cap_ctx, &policy_eval).await;

        // Should execute wildcard arm
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::String("wildcard".to_string()));
    }

    #[tokio::test]
    async fn test_receive_blocking_waits() {
        let ctx = Context::new();
        let mailbox = create_test_mailbox(vec![]);
        let mailbox_clone = mailbox.clone();
        let stream_ctx = StreamContext::new();
        let cap_ctx = CapabilityContext::new();
        let policy_eval = PolicyEvaluator::new();

        // Spawn a task to add a message after delay
        tokio::spawn(async move {
            sleep(Duration::from_millis(50)).await;
            let entry = MailboxEntry::new("test", "channel", Value::Int(42));
            mailbox_clone.lock().await.push(entry).unwrap();
        });

        let receive = Receive {
            mode: ReceiveMode::Blocking(None),
            arms: vec![create_arm(
                Pattern::Literal(Value::Int(42)),
                Workflow::Ret {
                    expr: Expr::Literal(Value::String("found".to_string())),
                },
            )],
            control_arms: None,
        };

        let start = std::time::Instant::now();
        let result =
            execute_receive(&receive, ctx, mailbox, &stream_ctx, &cap_ctx, &policy_eval).await;

        // Should have waited approximately 50ms (with some tolerance for scheduler variance)
        assert!(
            start.elapsed() >= Duration::from_millis(40),
            "Expected at least 40ms elapsed, got {:?}",
            start.elapsed()
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::String("found".to_string()));
    }

    #[tokio::test]
    async fn test_receive_with_timeout() {
        let ctx = Context::new();
        let mailbox: SharedMailbox = Arc::new(Mutex::new(Mailbox::new()));
        let stream_ctx = StreamContext::new();
        let cap_ctx = CapabilityContext::new();
        let policy_eval = PolicyEvaluator::new();

        let receive = Receive {
            mode: ReceiveMode::Blocking(Some(Duration::from_millis(100))),
            arms: vec![
                create_arm(
                    Pattern::Literal(Value::Int(42)),
                    Workflow::Ret {
                        expr: Expr::Literal(Value::String("found".to_string())),
                    },
                ),
                create_arm(
                    Pattern::Wildcard,
                    Workflow::Ret {
                        expr: Expr::Literal(Value::String("timeout".to_string())),
                    },
                ),
            ],
            control_arms: None,
        };

        let start = std::time::Instant::now();
        let result =
            execute_receive(&receive, ctx, mailbox, &stream_ctx, &cap_ctx, &policy_eval).await;

        // Should have waited approximately 100ms (with some tolerance for scheduler variance)
        assert!(
            start.elapsed() >= Duration::from_millis(90),
            "Expected at least 90ms elapsed, got {:?}",
            start.elapsed()
        );
        // Should execute wildcard arm on timeout
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::String("timeout".to_string()));
    }

    #[tokio::test]
    async fn test_receive_pattern_matching() {
        let ctx = Context::new();

        // Create a record value
        let mut fields = HashMap::new();
        fields.insert("priority".to_string(), Value::String("urgent".to_string()));
        let record_value = Value::Record(Box::new(fields));

        let mailbox = create_test_mailbox(vec![MailboxEntry::new(
            "kafka",
            "orders",
            record_value.clone(),
        )]);
        let stream_ctx = StreamContext::new();
        let cap_ctx = CapabilityContext::new();
        let policy_eval = PolicyEvaluator::new();

        // Create record pattern matching priority: "urgent"
        let urgent_pattern = Pattern::Record(vec![(
            "priority".to_string(),
            Pattern::Literal(Value::String("urgent".to_string())),
        )]);

        let normal_pattern = Pattern::Record(vec![(
            "priority".to_string(),
            Pattern::Literal(Value::String("normal".to_string())),
        )]);

        let receive = Receive {
            mode: ReceiveMode::NonBlocking,
            arms: vec![
                create_arm(
                    urgent_pattern,
                    Workflow::Ret {
                        expr: Expr::Literal(Value::String("urgent_workflow".to_string())),
                    },
                ),
                create_arm(
                    normal_pattern,
                    Workflow::Ret {
                        expr: Expr::Literal(Value::String("normal_workflow".to_string())),
                    },
                ),
            ],
            control_arms: None,
        };

        let result =
            execute_receive(&receive, ctx, mailbox, &stream_ctx, &cap_ctx, &policy_eval).await;

        // Should select urgent_workflow based on pattern match
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Value::String("urgent_workflow".to_string())
        );
    }

    #[tokio::test]
    async fn test_receive_guard_evaluation() {
        let mut ctx = Context::new();
        ctx.set("threshold".to_string(), Value::Int(100));

        // Create a record value with value: 150
        let mut fields = HashMap::new();
        fields.insert("value".to_string(), Value::Int(150));
        let record_value = Value::Record(Box::new(fields));

        let mailbox = create_test_mailbox(vec![MailboxEntry::new("sensor", "temp", record_value)]);
        let stream_ctx = StreamContext::new();
        let cap_ctx = CapabilityContext::new();
        let policy_eval = PolicyEvaluator::new();

        // Create pattern that binds to the message
        let pattern = Pattern::Variable("msg".to_string());

        // Guard: msg.value > threshold
        let guard = Expr::Binary {
            op: BinaryOp::Gt,
            left: Box::new(Expr::FieldAccess {
                expr: Box::new(Expr::Variable("msg".to_string())),
                field: "value".to_string(),
            }),
            right: Box::new(Expr::Variable("threshold".to_string())),
        };

        let receive = Receive {
            mode: ReceiveMode::NonBlocking,
            arms: vec![
                create_arm_with_guard(
                    pattern.clone(),
                    guard,
                    Workflow::Ret {
                        expr: Expr::Literal(Value::String("high_temp".to_string())),
                    },
                ),
                create_arm(
                    pattern,
                    Workflow::Ret {
                        expr: Expr::Literal(Value::String("normal_temp".to_string())),
                    },
                ),
            ],
            control_arms: None,
        };

        let result =
            execute_receive(&receive, ctx, mailbox, &stream_ctx, &cap_ctx, &policy_eval).await;

        // Guard should pass (150 > 100), execute high_temp_workflow
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::String("high_temp".to_string()));
    }

    #[tokio::test]
    async fn test_receive_control() {
        let ctx = Context::new();
        let control_mailbox = create_test_mailbox(vec![MailboxEntry::new(
            "control",
            "command",
            Value::String("shutdown".to_string()),
        )]);
        let stream_ctx = StreamContext::new();
        let cap_ctx = CapabilityContext::new();
        let policy_eval = PolicyEvaluator::new();

        let receive = Receive {
            mode: ReceiveMode::NonBlocking,
            arms: vec![],
            control_arms: Some(vec![create_arm(
                Pattern::Literal(Value::String("shutdown".to_string())),
                Workflow::Ret {
                    expr: Expr::Literal(Value::String("shutting_down".to_string())),
                },
            )]),
        };

        let result = execute_receive(
            &receive,
            ctx,
            control_mailbox,
            &stream_ctx,
            &cap_ctx,
            &policy_eval,
        )
        .await;

        // Should match shutdown command
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::String("shutting_down".to_string()));
    }

    #[tokio::test]
    async fn test_guard_falls_through_to_next_arm() {
        let ctx = Context::new();

        // Create a record value with value: 50
        let mut fields = HashMap::new();
        fields.insert("value".to_string(), Value::Int(50));
        let record_value = Value::Record(Box::new(fields));

        let mailbox = create_test_mailbox(vec![MailboxEntry::new("sensor", "temp", record_value)]);
        let stream_ctx = StreamContext::new();
        let cap_ctx = CapabilityContext::new();
        let policy_eval = PolicyEvaluator::new();

        let pattern = Pattern::Variable("msg".to_string());

        // Guard: msg.value > 100 (will fail)
        let guard_high = Expr::Binary {
            op: BinaryOp::Gt,
            left: Box::new(Expr::FieldAccess {
                expr: Box::new(Expr::Variable("msg".to_string())),
                field: "value".to_string(),
            }),
            right: Box::new(Expr::Literal(Value::Int(100))),
        };

        let receive = Receive {
            mode: ReceiveMode::NonBlocking,
            arms: vec![
                create_arm_with_guard(
                    pattern.clone(),
                    guard_high,
                    Workflow::Ret {
                        expr: Expr::Literal(Value::String("high".to_string())),
                    },
                ),
                create_arm(
                    pattern,
                    Workflow::Ret {
                        expr: Expr::Literal(Value::String("normal".to_string())),
                    },
                ),
            ],
            control_arms: None,
        };

        let result =
            execute_receive(&receive, ctx, mailbox, &stream_ctx, &cap_ctx, &policy_eval).await;

        // Guard fails, should fall through to second arm
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::String("normal".to_string()));
    }

    #[tokio::test]
    async fn test_variable_pattern_binding() {
        let ctx = Context::new();
        let mailbox =
            create_test_mailbox(vec![MailboxEntry::new("test", "channel", Value::Int(42))]);
        let stream_ctx = StreamContext::new();
        let cap_ctx = CapabilityContext::new();
        let policy_eval = PolicyEvaluator::new();

        let receive = Receive {
            mode: ReceiveMode::NonBlocking,
            arms: vec![create_arm(
                Pattern::Variable("x".to_string()),
                Workflow::Ret {
                    expr: Expr::Variable("x".to_string()),
                },
            )],
            control_arms: None,
        };

        let result =
            execute_receive(&receive, ctx, mailbox, &stream_ctx, &cap_ctx, &policy_eval).await;

        // Should bind the value and return it
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Int(42));
    }

    #[tokio::test]
    async fn test_tuple_pattern_matching() {
        let ctx = Context::new();
        let mailbox = create_test_mailbox(vec![MailboxEntry::new(
            "test",
            "channel",
            Value::List(Box::new(vec![Value::Int(1), Value::Int(2)])),
        )]);
        let stream_ctx = StreamContext::new();
        let cap_ctx = CapabilityContext::new();
        let policy_eval = PolicyEvaluator::new();

        let receive = Receive {
            mode: ReceiveMode::NonBlocking,
            arms: vec![create_arm(
                Pattern::Tuple(vec![
                    Pattern::Variable("a".to_string()),
                    Pattern::Variable("b".to_string()),
                ]),
                Workflow::Ret {
                    expr: Expr::Binary {
                        op: BinaryOp::Add,
                        left: Box::new(Expr::Variable("a".to_string())),
                        right: Box::new(Expr::Variable("b".to_string())),
                    },
                },
            )],
            control_arms: None,
        };

        let result =
            execute_receive(&receive, ctx, mailbox, &stream_ctx, &cap_ctx, &policy_eval).await;

        // Should match tuple and sum values: 1 + 2 = 3
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Int(3));
    }
}
