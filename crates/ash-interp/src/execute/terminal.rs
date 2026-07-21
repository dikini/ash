//! Terminal execution observation helpers.

use std::collections::BTreeSet;
use std::sync::Arc;

use ash_core::runtime::{
    FailureBoundary, FailureEntity, LexicalFrameId, OperationalFailure, ProcessId,
    ProcessTerminalState,
};
use ash_core::{Value, ApplicationId};

use crate::ExecResult;
use crate::context::Context;
use crate::control_link::ConservativeRetainedObligationsSummary;
use crate::error::{EvalError, ExecError};
use crate::runtime_outcome_state::RuntimeOutcomeState;

#[derive(Debug, Clone)]
pub(super) struct TerminalObservationRecorder {
    obligations: Arc<std::sync::Mutex<Option<ConservativeRetainedObligationsSummary>>>,
}

impl TerminalObservationRecorder {
    pub(super) fn new() -> Self {
        Self {
            obligations: Arc::new(std::sync::Mutex::new(None)),
        }
    }

    fn record_terminal_result(&self, ctx: &Context, result: &ExecResult<Value>) {
        if RuntimeOutcomeState::from_exec_result(result).is_terminal() {
            let mut slot = self
                .obligations
                .lock()
                .expect("terminal observation recorder mutex should not be poisoned");
            if slot.is_none() {
                *slot = Some(conservative_obligations_summary_from_context(ctx));
            }
        }
    }

    pub(super) fn observed_obligations(&self) -> Option<ConservativeRetainedObligationsSummary> {
        self.obligations
            .lock()
            .expect("terminal observation recorder mutex should not be poisoned")
            .clone()
    }
}

fn conservative_obligations_summary_from_context(
    ctx: &Context,
) -> ConservativeRetainedObligationsSummary {
    let (active_role, role_pending, role_discharged) = match ctx.role_context() {
        Some(role_ctx) => (
            Some(role_ctx.active_role.name.clone()),
            role_ctx.pending_obligations_set(),
            role_ctx.discharged_obligations_set(),
        ),
        None => (None, BTreeSet::new(), BTreeSet::new()),
    };

    ConservativeRetainedObligationsSummary::new(
        ctx.local_pending_obligations(),
        active_role,
        role_pending,
        role_discharged,
    )
}

pub(super) fn record_terminal_result_if_observed(
    terminal_observer: Option<&TerminalObservationRecorder>,
    ctx: &Context,
    result: &ExecResult<Value>,
) {
    if let Some(observer) = terminal_observer {
        observer.record_terminal_result(ctx, result);
    }
}

pub(super) fn finish_with_terminal_observation(
    terminal_observer: Option<&TerminalObservationRecorder>,
    ctx: &Context,
    result: ExecResult<Value>,
) -> ExecResult<Value> {
    record_terminal_result_if_observed(terminal_observer, ctx, &result);
    result
}

pub(super) fn process_terminal_state_from_exec_result(
    process_id: ProcessId,
    result: &ExecResult<Value>,
) -> Option<ProcessTerminalState> {
    match RuntimeOutcomeState::from_exec_result(result) {
        RuntimeOutcomeState::TerminalSuccess => match result {
            Ok(value) => Some(ProcessTerminalState::Succeeded {
                value: value.clone(),
            }),
            Err(_) => None,
        },
        RuntimeOutcomeState::ExecutionFailure => match result {
            Err(error) => Some(ProcessTerminalState::Failed {
                process_id,
                failure: Box::new(operational_failure_from_exec_error(process_id, error)),
            }),
            Ok(_) => None,
        },
        RuntimeOutcomeState::InvalidOrTerminated => match result {
            Err(error) => Some(ProcessTerminalState::Cancelled {
                process_id,
                failure: Box::new(operational_failure_from_exec_error(process_id, error)),
            }),
            Ok(_) => None,
        },
        RuntimeOutcomeState::BlockedOrSuspended | RuntimeOutcomeState::Active => None,
    }
}

fn operational_failure_from_exec_error(
    process_id: ProcessId,
    error: &ExecError,
) -> OperationalFailure {
    match error {
        ExecError::Eval(EvalError::OperationalFailure(failure)) => failure.as_ref().clone(),
        ExecError::Eval(eval_error) => OperationalFailure::new(
            FailureBoundary::Process,
            FailureEntity::Process(process_id),
            Value::String(eval_error.to_string()),
            "String",
        )
        .with_cause(operational_failure_from_eval_error(eval_error)),
        _ => OperationalFailure::new(
            FailureBoundary::Process,
            FailureEntity::Process(process_id),
            Value::String(error.to_string()),
            "String",
        )
        .with_cause(OperationalFailure::new(
            FailureBoundary::Application,
            FailureEntity::Application(ApplicationId::new()),
            Value::String(error.to_string()),
            "String",
        )),
    }
}

fn operational_failure_from_eval_error(error: &EvalError) -> OperationalFailure {
    match error {
        EvalError::OperationalFailure(failure) => failure.as_ref().clone(),
        _ => OperationalFailure::new(
            FailureBoundary::Pure,
            FailureEntity::LexicalFrame(LexicalFrameId::new()),
            Value::String(error.to_string()),
            "String",
        ),
    }
}
