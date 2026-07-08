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

pub mod act_env;
pub mod behaviour;
pub mod capability;
pub mod capability_policy;
pub mod capability_policy_runtime;
pub mod capability_provenance;
pub mod channel;
pub mod constraint_enforcement;
pub mod context;
pub mod control_link;
pub mod cps;
pub mod error;
pub mod eval;
pub mod exec_send;
pub mod execute;
pub mod execute_observe;
pub mod execute_set;
pub mod execute_stream;
pub mod execution_record;
pub mod guard;
pub mod list_helpers;
pub mod mailbox;
pub mod pattern;
pub mod policy;
pub mod predicate_evaluator;
pub mod process_env;
pub mod process_registry;
pub mod proxy_registry;
pub mod role_context;
pub mod role_runtime;
pub mod runtime_outcome_state;
pub mod runtime_state;
pub mod small_step;
pub mod stream;
pub mod typed_provider;
pub mod yield_routing;
pub mod yield_state;

pub use act_env::ActEnv;
pub use behaviour::{
    BehaviourContext, BehaviourProvider, BehaviourRegistry, BidirectionalBehaviour,
    BidirectionalBehaviourProvider, MockBehaviourProvider, MockBidirectionalProvider,
    MockSettableProvider, SettableBehaviourProvider, SettableRegistry, TypedSettableProvider,
};
pub use capability::{CapabilityContext, CapabilityProvider, CapabilityRegistry, MockProvider};
pub use capability_policy::{
    CapabilityContext as CapabilityPolicyContext, CapabilityOperation, CapabilityPolicyEvaluator,
    Direction, Policy as CapabilityPolicy, PolicyDecision, PolicyError, Reason, Role,
    Transformation,
};
pub use channel::{ChannelError, ChannelId, ChannelRegistry};
pub use constraint_enforcement::{ConstraintEnforcer, ConstraintViolation};
pub use context::Context;
pub use control_link::{
    ConservativeRetainedEffectSummary, ConservativeRetainedObligationsSummary,
    ConservativeRetainedProvenanceSummary, ControlLinkError, ControlLinkRegistry, LinkState,
    RetainedCompletionKind, RetainedCompletionRecord,
};
pub use error::{
    EvalError, EvalResult, ExecError, ExecResult, PatternError, PatternResult, ValidationError,
    ValidationResult,
};
pub use eval::{eval_expr, eval_expr_async};
pub use exec_send::execute_send;
pub use execute::{
    execute_simple, execute_simple_in_state, execute_with_bindings_in_state,
    execute_workflow_with_behaviour, execute_workflow_with_behaviour_in_state,
    execute_workflow_with_stream, execute_workflow_with_stream_in_state,
};
pub use execute_observe::{execute_changed, execute_observe};
pub use execute_set::execute_set;
pub use execution_record::{
    ExecutionAdmissionFacts, ExecutionBlockedReason, ExecutionEffectSummary,
    ExecutionInvalidReason, ExecutionObligationState, ExecutionPhase, ExecutionRecord,
    ExecutionTerminal, SemanticCompletionPayload, SemanticEffectTrace, SemanticWorkflowOutcome,
};
pub use guard::eval_guard;
pub use mailbox::{Mailbox, MailboxError, SharedMailbox};
pub use pattern::match_pattern;
pub use policy::{Policy, PolicyEvaluator, PolicyRule};
pub use process_env::{
    ChildEnvProjection, ChildEnvProjectionError, ProcessEnvIdentity, derive_child_env,
};
pub use process_registry::{ProcessRecord, ProcessRegistry, ProcessRegistryError};
pub use proxy_registry::{InstanceAddr, ProxyRegistry, RoleName};
pub use role_context::RoleContext;
pub use role_runtime::{
    CapabilityError, CapabilityGrant, RoleError, RoleRegistry, RuntimeCapabilitySet,
};
pub use runtime_outcome_state::RuntimeOutcomeState;
pub use runtime_state::{
    EntryOwnedResourceAdmission, ImplementationBindingAdmission,
    ImplementationBindingDependencySource, ImplementationOperationBody, RuntimeState,
    StandardInternalPilot, StandardPilotBinding, StandardPilotResource,
};
pub use stream::{
    BidirectionalStream, BidirectionalStreamProvider, MockBidirectionalStream,
    MockSendableProvider, MockStreamProvider, SendableRegistry, SendableStreamProvider,
    StreamContext, StreamProvider, StreamRegistry, TypedSendableProvider,
};
pub use typed_provider::{TypedBehaviourProvider, TypedStreamProvider};
pub use yield_routing::{PendingYield, ResumeResult, YieldError, YieldId, YieldRouter};
pub use yield_state::{CorrelationId, SuspendedYields, YieldState};

use ash_core::{
    ApplicationBoundaryOutcome, ApplicationFailure, ApplicationFailureKind, ApplicationReport,
    FailureBoundary, FailureEntity, OperationalFailure, RunId, Value, Workflow, WorkflowId,
};

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

/// Project an existing `ExecResult<Value>` into the outer application-boundary carrier.
#[must_use]
pub fn application_boundary_outcome_from_exec_result(
    workflow_id: WorkflowId,
    run_id: RunId,
    result: ExecResult<Value>,
) -> ApplicationBoundaryOutcome {
    match result {
        Ok(value) => {
            let report =
                ApplicationReport::succeeded(workflow_id, run_id).with_result(value.clone());
            ApplicationBoundaryOutcome::succeeded(value, report)
        }
        Err(error) => {
            let cause = OperationalFailure::new(
                FailureBoundary::Application,
                FailureEntity::Run(run_id),
                Value::String(error.to_string()),
                "ExecError",
            );
            let failure = ApplicationFailure::new(
                workflow_id,
                run_id,
                ApplicationFailureKind::BodyFailureEscaped,
                Some(cause),
            );
            let report = ApplicationReport::failed(workflow_id, run_id, failure.clone());
            ApplicationBoundaryOutcome::failed(failure, report)
        }
    }
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
            pattern: Pattern::Variable {
                name: "x".to_string(),
                span: ash_core::ast::Span::default(),
            },
            expr: Expr::Literal(Value::Int(10)),
            continuation: Box::new(Workflow::Let {
                pattern: Pattern::Variable {
                    name: "y".to_string(),
                    span: ash_core::ast::Span::default(),
                },
                expr: Expr::Literal(Value::Int(20)),
                continuation: Box::new(Workflow::Ret {
                    expr: Expr::Binary {
                        op: BinaryOp::Add,
                        left: Box::new(Expr::Variable {
                            name: "x".to_string(),
                            span: ash_core::ast::Span::default(),
                        }),
                        right: Box::new(Expr::Variable {
                            name: "y".to_string(),
                            span: ash_core::ast::Span::default(),
                        }),
                    },
                }),
            }),
        };

        let result = interpret(&workflow).await.unwrap();
        assert_eq!(result, Value::Int(30));
    }
}
