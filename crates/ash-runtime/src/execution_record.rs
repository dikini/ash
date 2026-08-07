//! Runtime execution-record substrate and semantic terminal projections.

use ash_core::{ApplicationId, Effect, Name, Provenance, TraceEvent, Value};
use chrono::{DateTime, Utc};
use std::collections::BTreeSet;

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
/// Terminal outcome of an execution attempt.
#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum ExecutionTerminal {
    Return(Value),
    Reject(ExecError),
}
/// Execution phase for a single application attempt.
#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)]
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
}

impl ExecutionObligationState {
    pub fn from_context(ctx: &Context) -> Self {
        Self {
            pending: ctx.visible_pending_obligations(),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn merge_parallel(branches: &[Self]) -> Self {
        let mut pending = BTreeSet::new();

        for branch in branches {
            pending.extend(branch.pending.iter().cloned());
        }

        Self { pending }
    }

    pub fn pending(&self) -> &BTreeSet<Name> {
        &self.pending
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

    #[allow(dead_code)]
    pub(crate) fn merge_parallel(branches: &[Self]) -> Self {
        let mut merged = Self::default();
        for branch in branches {
            merged.terminal = merged.terminal.max(branch.terminal);
            merged.reached.extend(branch.reached.iter().copied());
        }
        merged
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
    admission: ExecutionAdmissionFacts,
    obligations: ExecutionObligationState,
    provenance: Provenance,
    trace: Vec<TraceEvent>,
    effects: ExecutionEffectSummary,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExecutionAdmissionFacts {
    capability_binding_grants: Vec<String>,
    resource_grants: Vec<String>,
    action_grants: Vec<String>,
}

impl ExecutionAdmissionFacts {
    pub fn new(
        capability_binding_grants: Vec<String>,
        resource_grants: Vec<String>,
        action_grants: Vec<String>,
    ) -> Self {
        Self {
            capability_binding_grants,
            resource_grants,
            action_grants,
        }
    }

    pub fn capability_binding_grants(&self) -> &[String] {
        &self.capability_binding_grants
    }

    pub fn resource_grants(&self) -> &[String] {
        &self.resource_grants
    }

    pub fn action_grants(&self) -> &[String] {
        &self.action_grants
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct ParallelTraceEvent {
    branch_index: usize,
    event: TraceEvent,
    timestamp: DateTime<Utc>,
}

impl ParallelTraceEvent {
    #[allow(dead_code)]
    fn new(branch_index: usize, event: TraceEvent) -> Self {
        let timestamp = trace_event_timestamp(&event);
        Self {
            branch_index,
            event,
            timestamp,
        }
    }
}

#[allow(dead_code)]
fn trace_event_timestamp(event: &TraceEvent) -> DateTime<Utc> {
    match event {
        TraceEvent::Obs { timestamp, .. }
        | TraceEvent::Orient { timestamp, .. }
        | TraceEvent::Decide { timestamp, .. }
        | TraceEvent::Act { timestamp, .. }
        | TraceEvent::Oblig { timestamp, .. }
        | TraceEvent::ThunkConstructed { timestamp, .. }
        | TraceEvent::ThunkForceStarted { timestamp, .. }
        | TraceEvent::ThunkBodyEvaluationStarted { timestamp, .. }
        | TraceEvent::ThunkBodyEvaluationCompleted { timestamp, .. }
        | TraceEvent::ThunkForceCompleted { timestamp, .. }
        | TraceEvent::MemoCacheFilled { timestamp, .. }
        | TraceEvent::MemoCacheHit { timestamp, .. }
        | TraceEvent::MemoReplayFailure { timestamp, .. }
        | TraceEvent::MemoReentrantRejected { timestamp } => *timestamp,
    }
}

#[allow(dead_code)]
fn join_parallel_provenance(branches: &[ExecutionRecord]) -> Provenance {
    if branches.is_empty() {
        return Provenance::new();
    }

    let parent = branches.iter().find_map(|branch| branch.provenance.parent);
    let mut lineage = Vec::<ApplicationId>::new();
    if let Some(parent_id) = parent {
        lineage.push(parent_id);
    }
    for branch in branches {
        for ancestor in &branch.provenance.lineage {
            if !lineage.contains(ancestor) {
                lineage.push(*ancestor);
            }
        }
        if !lineage.contains(&branch.provenance.application_id) {
            lineage.push(branch.provenance.application_id);
        }
    }

    Provenance {
        application_id: parent.unwrap_or(branches[0].provenance.application_id),
        parent,
        lineage,
    }
}

#[allow(dead_code)]
fn merge_parallel_traces(branches: &[ExecutionRecord]) -> Vec<TraceEvent> {
    let mut events = branches
        .iter()
        .enumerate()
        .flat_map(|(branch_index, branch)| {
            branch
                .trace
                .iter()
                .cloned()
                .map(move |event| ParallelTraceEvent::new(branch_index, event))
        })
        .collect::<Vec<_>>();

    events.sort_by_key(|event| (event.timestamp, event.branch_index));
    events.into_iter().map(|event| event.event).collect()
}

impl ExecutionRecord {
    pub fn new(provenance: Provenance) -> Self {
        Self {
            phase: ExecutionPhase::Running,
            admission: ExecutionAdmissionFacts::default(),
            obligations: ExecutionObligationState::default(),
            provenance,
            trace: Vec::new(),
            effects: ExecutionEffectSummary::default(),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn merge_parallel_success(branches: &[Self]) -> Self {
        let values = branches
            .iter()
            .map(|branch| match branch.phase() {
                ExecutionPhase::Terminal(ExecutionTerminal::Return(value)) => value.clone(),
                other => panic!("parallel success merge requires terminal returns, got {other:?}"),
            })
            .collect();

        Self::merge_parallel_terminal(
            branches,
            ExecutionTerminal::Return(Value::list_from_vec(values)),
        )
    }

    #[allow(dead_code)]
    pub(crate) fn merge_parallel_rejection(branches: &[Self], error: ExecError) -> Self {
        Self::merge_parallel_terminal(branches, ExecutionTerminal::Reject(error))
    }

    #[allow(dead_code)]
    fn merge_parallel_terminal(branches: &[Self], terminal: ExecutionTerminal) -> Self {
        let obligations = ExecutionObligationState::merge_parallel(
            &branches
                .iter()
                .map(|branch| branch.obligations.clone())
                .collect::<Vec<_>>(),
        );
        let effects = ExecutionEffectSummary::merge_parallel(
            &branches
                .iter()
                .map(|branch| branch.effects.clone())
                .collect::<Vec<_>>(),
        );
        let provenance = join_parallel_provenance(branches);
        let trace = merge_parallel_traces(branches);

        Self {
            phase: ExecutionPhase::Terminal(terminal),
            admission: ExecutionAdmissionFacts::default(),
            obligations,
            provenance,
            trace,
            effects,
        }
    }

    pub fn phase(&self) -> &ExecutionPhase {
        &self.phase
    }

    pub fn admission(&self) -> &ExecutionAdmissionFacts {
        &self.admission
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

    pub fn project_application_outcome(&self) -> Option<SemanticApplicationOutcome> {
        match &self.phase {
            ExecutionPhase::Terminal(ExecutionTerminal::Return(value)) => {
                Some(SemanticApplicationOutcome::Return {
                    value: value.clone(),
                    effect: self.effects.terminal(),
                    trace: self.trace.clone(),
                    obligations: self.obligations.clone(),
                    provenance: self.provenance.clone(),
                })
            }
            ExecutionPhase::Terminal(ExecutionTerminal::Reject(error)) => {
                Some(SemanticApplicationOutcome::Reject {
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
#[allow(clippy::large_enum_variant)]
pub enum SemanticApplicationOutcome {
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

impl SemanticApplicationOutcome {
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
