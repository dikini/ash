use ash_core::{Value, WorkflowId};
use ash_interp::{ControlLinkError, ExecError, LinkState, RuntimeOutcomeState};

#[test]
fn yield_suspended_and_requires_approval_classify_as_blocked_or_suspended() {
    let yield_error = ExecError::YieldSuspended {
        role: "proxy".into(),
        request: Box::new(Value::Int(7)),
        expected_response_type: "Int".into(),
        correlation_id: "corr-1".into(),
        proxy_addr: "proxy://workflow".into(),
    };
    let approval_error = ExecError::RequiresApproval {
        role: "admin".into(),
        operation: "set".into(),
        capability: "hvac:target".into(),
    };

    assert_eq!(
        yield_error.runtime_outcome_state(),
        RuntimeOutcomeState::BlockedOrSuspended
    );
    assert_eq!(
        approval_error.runtime_outcome_state(),
        RuntimeOutcomeState::BlockedOrSuspended
    );
}

#[test]
fn terminated_control_surfaces_classify_as_invalid_or_terminated() {
    let instance_id = WorkflowId::new();

    assert_eq!(
        ControlLinkError::Terminated(instance_id).runtime_outcome_state(),
        RuntimeOutcomeState::InvalidOrTerminated
    );
    assert_eq!(
        ControlLinkError::NotFound(instance_id).runtime_outcome_state(),
        RuntimeOutcomeState::InvalidOrTerminated
    );
    assert_eq!(
        LinkState::Terminated.runtime_outcome_state(),
        RuntimeOutcomeState::InvalidOrTerminated
    );
}

#[test]
fn execution_failures_classify_as_execution_failure() {
    assert_eq!(
        ExecError::ExecutionFailed("boom".into()).runtime_outcome_state(),
        RuntimeOutcomeState::ExecutionFailure
    );
    assert_eq!(
        ExecError::ParallelFailed("branch failed".into()).runtime_outcome_state(),
        RuntimeOutcomeState::ExecutionFailure
    );
    assert_eq!(
        ExecError::ForEachFailed("iteration failed".into()).runtime_outcome_state(),
        RuntimeOutcomeState::ExecutionFailure
    );
    assert_eq!(
        ExecError::MailboxFull { limit: 1 }.runtime_outcome_state(),
        RuntimeOutcomeState::ExecutionFailure
    );
}

#[test]
fn blocked_and_invalid_runtime_state_variants_preserve_authoritative_classification() {
    assert_eq!(
        ExecError::Blocked("waiting for stream value".into()).runtime_outcome_state(),
        RuntimeOutcomeState::BlockedOrSuspended
    );
    assert_eq!(
        ExecError::InvalidRuntimeState("control link terminated".into()).runtime_outcome_state(),
        RuntimeOutcomeState::InvalidOrTerminated
    );
}

#[test]
fn successful_execution_results_classify_as_terminal_success() {
    let result: Result<Value, ExecError> = Ok(Value::Int(42));

    assert_eq!(
        RuntimeOutcomeState::from_exec_result(&result),
        RuntimeOutcomeState::TerminalSuccess
    );
}

#[test]
fn running_and_paused_link_states_remain_observable() {
    assert_eq!(
        LinkState::Running.runtime_outcome_state(),
        RuntimeOutcomeState::Active
    );
    assert_eq!(
        LinkState::Paused.runtime_outcome_state(),
        RuntimeOutcomeState::BlockedOrSuspended
    );
}

#[test]
fn terminal_and_live_helpers_match_coarse_runtime_classes() {
    assert!(RuntimeOutcomeState::TerminalSuccess.is_terminal());
    assert!(RuntimeOutcomeState::ExecutionFailure.is_terminal());
    assert!(RuntimeOutcomeState::InvalidOrTerminated.is_terminal());
    assert!(RuntimeOutcomeState::Active.is_live());
    assert!(RuntimeOutcomeState::BlockedOrSuspended.is_live());
    assert!(!RuntimeOutcomeState::BlockedOrSuspended.is_terminal());
}
