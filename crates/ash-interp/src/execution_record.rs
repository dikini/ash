//! Runtime execution-record substrate and semantic terminal projections.

use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

use ash_core::{Decision, Effect, Name, Provenance, TraceEvent, Value, WorkflowId};
use chrono::Utc;

use crate::context::Context;
use crate::{ExecError, ExecResult, RuntimeOutcomeState};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionBlockedReason {
    ReceiveWait,
    CompletionObservationWait,
    ControlWait,
    HelperWait(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionInvalidReason {
    RuntimeState(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionTerminal {
    Return(Value),
    Reject(ExecError),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionPhase {
    Running,
    Blocked(ExecutionBlockedReason),
    Terminal(ExecutionTerminal),
    Invalid(ExecutionInvalidReason),
}

impl ExecutionPhase {
    pub fn from_exec_result(result: &ExecResult<Value>) -> Self {
        match result {
            Ok(value) => Self::Terminal(ExecutionTerminal::Return(value.clone())),
            Err(error) => match error {
                ExecError::YieldSuspended { .. } => Self::Blocked(
                    ExecutionBlockedReason::HelperWait("yield-suspended".to_string()),
                ),
                ExecError::RequiresApproval { .. } => {
                    Self::Blocked(ExecutionBlockedReason::HelperWait("approval".to_string()))
                }
                ExecError::Blocked(message) => Self::Blocked(classify_blocked_message(message)),
                ExecError::InvalidRuntimeState(message) => {
                    Self::Invalid(ExecutionInvalidReason::RuntimeState(message.clone()))
                }
                other => Self::Terminal(ExecutionTerminal::Reject(other.clone())),
            },
        }
    }

    pub fn runtime_outcome_state(&self) -> RuntimeOutcomeState {
        match self {
            Self::Running => RuntimeOutcomeState::Active,
            Self::Blocked(..) => RuntimeOutcomeState::BlockedOrSuspended,
            Self::Terminal(ExecutionTerminal::Return(..)) => RuntimeOutcomeState::TerminalSuccess,
            Self::Terminal(ExecutionTerminal::Reject(..)) => RuntimeOutcomeState::ExecutionFailure,
            Self::Invalid(..) => RuntimeOutcomeState::InvalidOrTerminated,
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Terminal(..) | Self::Invalid(..))
    }
}

fn classify_blocked_message(message: &str) -> ExecutionBlockedReason {
    let lower = message.to_ascii_lowercase();
    if lower.contains("receive") {
        ExecutionBlockedReason::ReceiveWait
    } else if lower.contains("completion") {
        ExecutionBlockedReason::CompletionObservationWait
    } else if lower.contains("control") {
        ExecutionBlockedReason::ControlWait
    } else {
        ExecutionBlockedReason::HelperWait(message.to_string())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExecutionObligationState {
    pending: BTreeSet<Name>,
    active_role: Option<Name>,
    role_pending: BTreeSet<Name>,
    role_discharged: BTreeSet<Name>,
}

impl ExecutionObligationState {
    pub fn from_context(ctx: &Context) -> Self {
        let (active_role, role_pending, role_discharged) = match ctx.role_context() {
            Some(role_ctx) => (
                Some(role_ctx.active_role.name.clone()),
                role_ctx.pending_obligations_set(),
                role_ctx.discharged_obligations_set(),
            ),
            None => (None, BTreeSet::new(), BTreeSet::new()),
        };

        Self {
            pending: ctx.visible_pending_obligations(),
            active_role,
            role_pending,
            role_discharged,
        }
    }

    pub fn pending(&self) -> &BTreeSet<Name> {
        &self.pending
    }

    pub fn active_role(&self) -> Option<&str> {
        self.active_role.as_deref()
    }

    pub fn role_pending(&self) -> &BTreeSet<Name> {
        &self.role_pending
    }

    pub fn role_discharged(&self) -> &BTreeSet<Name> {
        &self.role_discharged
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionEffectSummary {
    terminal: Effect,
    reached: BTreeSet<Effect>,
}

impl Default for ExecutionEffectSummary {
    fn default() -> Self {
        Self {
            terminal: Effect::Epistemic,
            reached: BTreeSet::new(),
        }
    }
}

impl ExecutionEffectSummary {
    pub fn record(&mut self, effect: Effect) {
        self.terminal = self.terminal.max(effect);
        self.reached.insert(effect);
    }

    pub fn terminal(&self) -> Effect {
        self.terminal
    }

    pub fn reached(&self) -> &BTreeSet<Effect> {
        &self.reached
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionRecord {
    phase: ExecutionPhase,
    obligations: ExecutionObligationState,
    provenance: Provenance,
    trace: Vec<TraceEvent>,
    effects: ExecutionEffectSummary,
}

impl ExecutionRecord {
    pub fn new(provenance: Provenance) -> Self {
        Self {
            phase: ExecutionPhase::Running,
            obligations: ExecutionObligationState::default(),
            provenance,
            trace: Vec::new(),
            effects: ExecutionEffectSummary::default(),
        }
    }

    pub fn phase(&self) -> &ExecutionPhase {
        &self.phase
    }

    pub fn obligations(&self) -> &ExecutionObligationState {
        &self.obligations
    }

    pub fn provenance(&self) -> &Provenance {
        &self.provenance
    }

    pub fn trace(&self) -> &[TraceEvent] {
        &self.trace
    }

    pub fn effects(&self) -> &ExecutionEffectSummary {
        &self.effects
    }

    pub fn project_workflow_outcome(&self) -> Option<SemanticWorkflowOutcome> {
        match &self.phase {
            ExecutionPhase::Terminal(ExecutionTerminal::Return(value)) => {
                Some(SemanticWorkflowOutcome::Return {
                    value: value.clone(),
                    effect: self.effects.terminal(),
                    trace: self.trace.clone(),
                    obligations: self.obligations.clone(),
                    provenance: self.provenance.clone(),
                })
            }
            ExecutionPhase::Terminal(ExecutionTerminal::Reject(error)) => {
                Some(SemanticWorkflowOutcome::Reject {
                    error: error.clone(),
                    effect: self.effects.terminal(),
                    trace: self.trace.clone(),
                    obligations: self.obligations.clone(),
                    provenance: self.provenance.clone(),
                })
            }
            ExecutionPhase::Running | ExecutionPhase::Blocked(..) | ExecutionPhase::Invalid(..) => {
                None
            }
        }
    }

    pub fn project_completion(&self) -> Option<SemanticCompletionPayload> {
        match &self.phase {
            ExecutionPhase::Terminal(ExecutionTerminal::Return(value)) => {
                Some(SemanticCompletionPayload {
                    result: Ok(value.clone()),
                    obligations: self.obligations.clone(),
                    provenance: self.provenance.clone(),
                    effects: SemanticEffectTrace {
                        terminal: self.effects.terminal(),
                        reached: self.effects.reached().clone(),
                    },
                })
            }
            ExecutionPhase::Terminal(ExecutionTerminal::Reject(error)) => {
                Some(SemanticCompletionPayload {
                    result: Err(error.clone()),
                    obligations: self.obligations.clone(),
                    provenance: self.provenance.clone(),
                    effects: SemanticEffectTrace {
                        terminal: self.effects.terminal(),
                        reached: self.effects.reached().clone(),
                    },
                })
            }
            ExecutionPhase::Running | ExecutionPhase::Blocked(..) | ExecutionPhase::Invalid(..) => {
                None
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticEffectTrace {
    terminal: Effect,
    reached: BTreeSet<Effect>,
}

impl SemanticEffectTrace {
    pub fn terminal(&self) -> Effect {
        self.terminal
    }

    pub fn reached(&self) -> &BTreeSet<Effect> {
        &self.reached
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SemanticCompletionPayload {
    result: ExecResult<Value>,
    obligations: ExecutionObligationState,
    provenance: Provenance,
    effects: SemanticEffectTrace,
}

impl SemanticCompletionPayload {
    pub fn result(&self) -> &ExecResult<Value> {
        &self.result
    }

    pub fn obligations(&self) -> &ExecutionObligationState {
        &self.obligations
    }

    pub fn provenance(&self) -> &Provenance {
        &self.provenance
    }

    pub fn effects(&self) -> &SemanticEffectTrace {
        &self.effects
    }
}

#[derive(Debug, Clone)]
pub enum SemanticWorkflowOutcome {
    Return {
        value: Value,
        effect: Effect,
        trace: Vec<TraceEvent>,
        obligations: ExecutionObligationState,
        provenance: Provenance,
    },
    Reject {
        error: ExecError,
        effect: Effect,
        trace: Vec<TraceEvent>,
        obligations: ExecutionObligationState,
        provenance: Provenance,
    },
}

impl SemanticWorkflowOutcome {
    pub fn effect(&self) -> Effect {
        match self {
            Self::Return { effect, .. } | Self::Reject { effect, .. } => *effect,
        }
    }

    pub fn trace(&self) -> &[TraceEvent] {
        match self {
            Self::Return { trace, .. } | Self::Reject { trace, .. } => trace,
        }
    }

    pub fn obligations(&self) -> &ExecutionObligationState {
        match self {
            Self::Return { obligations, .. } | Self::Reject { obligations, .. } => obligations,
        }
    }

    pub fn provenance(&self) -> &Provenance {
        match self {
            Self::Return { provenance, .. } | Self::Reject { provenance, .. } => provenance,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ExecutionRecorder {
    inner: Arc<Mutex<ExecutionRecord>>,
}

impl ExecutionRecorder {
    pub(crate) fn new(provenance: Provenance) -> Self {
        Self {
            inner: Arc::new(Mutex::new(ExecutionRecord::new(provenance))),
        }
    }

    pub(crate) fn snapshot(&self) -> ExecutionRecord {
        self.inner
            .lock()
            .expect("execution recorder mutex should not be poisoned")
            .clone()
    }

    pub(crate) fn child_provenance(&self, workflow_id: WorkflowId) -> Provenance {
        let parent = self
            .inner
            .lock()
            .expect("execution recorder mutex should not be poisoned")
            .provenance
            .clone();

        let mut lineage = parent.lineage;
        lineage.push(parent.workflow_id);

        Provenance {
            workflow_id,
            parent: Some(parent.workflow_id),
            lineage,
        }
    }

    pub(crate) fn set_running(&self) {
        self.inner
            .lock()
            .expect("execution recorder mutex should not be poisoned")
            .phase = ExecutionPhase::Running;
    }

    pub(crate) fn sync_context(&self, ctx: &Context) {
        self.inner
            .lock()
            .expect("execution recorder mutex should not be poisoned")
            .obligations = ExecutionObligationState::from_context(ctx);
    }

    pub(crate) fn record_effect(&self, effect: Effect) {
        self.inner
            .lock()
            .expect("execution recorder mutex should not be poisoned")
            .effects
            .record(effect);
    }

    pub(crate) fn push_trace(&self, event: TraceEvent) {
        self.inner
            .lock()
            .expect("execution recorder mutex should not be poisoned")
            .trace
            .push(event);
    }

    pub(crate) fn record_observe(&self, capability: &str, effect: Effect) {
        self.record_effect(effect);
        self.push_trace(TraceEvent::Obs {
            capability: capability.to_string(),
            timestamp: Utc::now(),
        });
    }

    pub(crate) fn record_orient(&self, expr: &str) {
        self.record_effect(Effect::Deliberative);
        self.push_trace(TraceEvent::Orient {
            expr: expr.to_string(),
            timestamp: Utc::now(),
        });
    }

    pub(crate) fn record_decide(&self, policy: &str, decision: Decision) {
        self.record_effect(Effect::Evaluative);
        self.push_trace(TraceEvent::Decide {
            policy: policy.to_string(),
            decision,
            timestamp: Utc::now(),
        });
    }

    pub(crate) fn record_act(&self, action: &str, guard: &str) {
        self.record_effect(Effect::Operational);
        self.push_trace(TraceEvent::Act {
            action: action.to_string(),
            guard: guard.to_string(),
            timestamp: Utc::now(),
        });
    }

    pub(crate) fn record_obligation_check(&self, role: &str, satisfied: bool) {
        self.record_effect(Effect::Evaluative);
        self.push_trace(TraceEvent::Oblig {
            role: role.to_string(),
            satisfied,
            timestamp: Utc::now(),
        });
    }

    pub(crate) fn set_phase_from_result(&self, result: &ExecResult<Value>) {
        self.inner
            .lock()
            .expect("execution recorder mutex should not be poisoned")
            .phase = ExecutionPhase::from_exec_result(result);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocked_message_classification_preserves_semantic_wait_family() {
        assert_eq!(
            classify_blocked_message("receive waiting for input"),
            ExecutionBlockedReason::ReceiveWait
        );
        assert_eq!(
            classify_blocked_message("completion wait for child"),
            ExecutionBlockedReason::CompletionObservationWait
        );
        assert_eq!(
            classify_blocked_message("control operation paused"),
            ExecutionBlockedReason::ControlWait
        );
    }
}
