//! Ash Interpreter
//!
//! This crate provides the runtime interpreter for executing Ash workflows.
//!
//! # Example
//!
//! ```
//! use ash_core::{Workflow, Expr, Value};
//! use ash_interp::interpret;
//!
//! # tokio_test::block_on(async {
//! let workflow = Workflow::Ret { expr: Expr::Literal(Value::Int(42)) };
//! let result = interpret(&workflow).await.unwrap();
//! assert_eq!(result, Value::Int(42));
//! # });
//! ```

pub mod behaviour;
pub mod capability;
pub mod capability_policy;
pub mod capability_policy_runtime;
pub mod capability_provenance;
pub mod context;
pub mod control_link;
pub mod error;
pub mod eval;
pub mod exec_send;
pub mod execute;
pub mod execute_observe;
pub mod execute_set;
pub mod execute_stream;
pub mod guard;
pub mod mailbox;
pub mod pattern;
pub mod policy;
pub mod runtime_state;
pub mod stream;
pub mod typed_provider;

pub use behaviour::{
    BehaviourContext, BehaviourProvider, BehaviourRegistry, BidirectionalBehaviour,
    BidirectionalBehaviourProvider, MockBehaviourProvider, MockBidirectionalProvider,
    MockSettableProvider, SettableBehaviourProvider, SettableRegistry, TypedSettableProvider,
};
pub use capability::{CapabilityContext, CapabilityProvider, CapabilityRegistry, MockProvider};
pub use capability_policy::{
    CapabilityContext as CapabilityPolicyContext, CapabilityOperation, CapabilityPolicyEvaluator,
    Direction, Policy as CapabilityPolicy, PolicyDecision, PolicyError, Role, Transformation,
};
pub use context::Context;
pub use control_link::{ControlLinkError, ControlLinkRegistry, LinkState};
pub use error::{
    EvalError, EvalResult, ExecError, ExecResult, PatternError, PatternResult, ValidationError,
    ValidationResult,
};
pub use eval::eval_expr;
pub use exec_send::execute_send;
pub use execute::{
    execute_simple, execute_simple_in_state, execute_workflow, execute_workflow_with_behaviour,
    execute_workflow_with_behaviour_in_state, execute_workflow_with_stream,
    execute_workflow_with_stream_in_state,
};
pub use execute_observe::{execute_changed, execute_observe};
pub use execute_set::execute_set;
pub use guard::eval_guard;
pub use mailbox::{Mailbox, MailboxError, SharedMailbox};
pub use pattern::match_pattern;
pub use policy::{Policy, PolicyEvaluator, PolicyResult, PolicyRule};
pub use runtime_state::RuntimeState;
pub use stream::{
    BidirectionalStream, BidirectionalStreamProvider, MockBidirectionalStream,
    MockSendableProvider, MockStreamProvider, SendableRegistry, SendableStreamProvider,
    StreamContext, StreamProvider, StreamRegistry, TypedSendableProvider,
};
pub use typed_provider::{TypedBehaviourProvider, TypedStreamProvider};

use ash_core::{Value, Workflow};

/// Convenience function to interpret a workflow with default contexts
///
/// This is the simplest way to execute a workflow when you don't need
/// custom capability providers or policies.
///
/// # Example
///
/// ```
/// use ash_core::{Workflow, Expr, Value};
/// use ash_interp::interpret;
///
/// # tokio_test::block_on(async {
/// let workflow = Workflow::Ret { expr: Expr::Literal(Value::String("hello".to_string())) };
/// let result = interpret(&workflow).await.unwrap();
/// assert_eq!(result, Value::String("hello".to_string()));
/// # });
/// ```
pub async fn interpret(workflow: &Workflow) -> ExecResult<Value> {
    execute_simple(workflow).await
}

/// Execute a workflow using explicit runtime-owned state.
pub async fn interpret_in_state(
    workflow: &Workflow,
    runtime_state: &RuntimeState,
) -> ExecResult<Value> {
    execute_simple_in_state(workflow, runtime_state).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_core::{BinaryOp, Expr, Pattern};

    #[tokio::test]
    async fn test_interpret_simple() {
        let workflow = Workflow::Ret {
            expr: Expr::Literal(Value::Int(42)),
        };
        let result = interpret(&workflow).await.unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[tokio::test]
    async fn test_interpret_complex() {
        // let x = 10 in let y = 20 in x + y
        let workflow = Workflow::Let {
            pattern: Pattern::Variable("x".to_string()),
            expr: Expr::Literal(Value::Int(10)),
            continuation: Box::new(Workflow::Let {
                pattern: Pattern::Variable("y".to_string()),
                expr: Expr::Literal(Value::Int(20)),
                continuation: Box::new(Workflow::Ret {
                    expr: Expr::Binary {
                        op: BinaryOp::Add,
                        left: Box::new(Expr::Variable("x".to_string())),
                        right: Box::new(Expr::Variable("y".to_string())),
                    },
                }),
            }),
        };

        let result = interpret(&workflow).await.unwrap();
        assert_eq!(result, Value::Int(30));
    }
}
