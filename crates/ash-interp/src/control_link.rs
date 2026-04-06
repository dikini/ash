//! Control link registry for reusable runtime supervision authority.

use ash_core::{ControlLink, Effect, Name, Value, WorkflowId};
use std::collections::{BTreeSet, HashMap};
use thiserror::Error;
use tokio::sync::watch;

use crate::error::ExecResult;
use crate::runtime_outcome_state::RuntimeOutcomeState;

/// Errors that can occur when using control links.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum ControlLinkError {
    /// The control link was not found in the registry.
    #[error("control link for instance {0:?} not registered")]
    NotFound(WorkflowId),

    /// The instance has been terminated and can no longer be controlled.
    #[error("instance {0:?} has been terminated")]
    Terminated(WorkflowId),

    /// A non-terminal runtime outcome was incorrectly supplied for retained completion
    /// observation.
    #[error(
        "retained completion observation for instance {0:?} requires a terminal outcome, got {1:?}"
    )]
    NonTerminalObservation(WorkflowId, RuntimeOutcomeState),

    /// A retained terminal observation was already sealed for this instance and cannot be
    /// overwritten.
    #[error("retained completion observation for instance {0:?} is already sealed as {1:?}")]
    CompletionAlreadySealed(WorkflowId, Box<RetainedCompletionRecord>),
}

/// The supervision state of a control link target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkState {
    /// The instance is alive and currently running.
    Running,
    /// The instance is alive but currently paused.
    Paused,
    /// The instance has been killed; future control operations are invalid.
    Terminated,
}

/// The coarse-grained reason why a retained terminal observation exists for a control target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RetainedCompletionKind {
    /// The child reached an explicit terminal completion recorded by runtime/control-facing code.
    Completed,
    /// The supervisor imposed a terminal control outcome (for example via `kill`).
    ControlTerminated,
}

/// Minimal retained terminal observation for a control target.
///
/// This is intentionally conservative. It does not claim to carry the full `SPEC-004`
/// `CompletionPayload`; it only retains one explicit runtime-visible terminal observation record
/// so completion-style state remains queryable after the target reaches a terminal condition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConservativeRetainedEffectSummary {
    /// Conservative terminal effect-summary field comparable to `CompletionPayload.effects.terminal`.
    ///
    /// The current runtime does not yet transport the full authoritative completion carrier, so
    /// this field is derived from the runtime-visible workflow effect layers the interpreter can
    /// honestly summarize today. It therefore remains a conservative upper bound rather than a
    /// proof that the exact big-step terminal `eff` carrier has been preserved.
    terminal: Effect,
    /// Conservative reached-effect summary comparable to `CompletionPayload.effects.reached`.
    ///
    /// This is intentionally not a full trace transport. It retains only the surfaced effect
    /// layers the runtime can currently summarize honestly for the completion record.
    reached: BTreeSet<Effect>,
}

impl ConservativeRetainedEffectSummary {
    /// Create one conservative retained effect summary.
    pub fn new(terminal_upper_bound: Effect, reached_upper_bound: BTreeSet<Effect>) -> Self {
        assert!(
            reached_upper_bound.contains(&terminal_upper_bound),
            "retained effect summaries must include the terminal upper bound in the reached upper-bound set"
        );
        Self {
            terminal: terminal_upper_bound,
            reached: reached_upper_bound,
        }
    }

    /// Return a baseline summary for payload slices that do not yet have retained effect details.
    pub fn baseline() -> Self {
        Self::new(Effect::Epistemic, BTreeSet::from([Effect::Epistemic]))
    }

    /// Return the conservative terminal effect upper bound retained for this completion.
    pub fn terminal_upper_bound(&self) -> Effect {
        self.terminal
    }

    /// Borrow the conservative reached-effect upper-bound set retained for this completion.
    pub fn reached_upper_bound(&self) -> &BTreeSet<Effect> {
        &self.reached
    }
}

/// Honest retained obligations summary for what the runtime can actually observe at child
/// terminal time.
///
/// This is intentionally not full `CompletionPayload.obligations` parity. It only preserves the
/// obligation carriers the current runtime can truthfully snapshot from the terminal observation
/// path: local pending obligations visible through the observed child execution context and the
/// active role context's pending/discharged state, if any.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConservativeRetainedObligationsSummary {
    local_pending: BTreeSet<Name>,
    active_role: Option<Name>,
    role_pending: BTreeSet<Name>,
    role_discharged: BTreeSet<Name>,
}

impl ConservativeRetainedObligationsSummary {
    /// Create one conservative retained obligations summary.
    pub fn new(
        local_pending_visible_at_terminal: BTreeSet<Name>,
        active_role_visible_at_terminal: Option<Name>,
        role_pending_visible_at_terminal: BTreeSet<Name>,
        role_discharged_visible_at_terminal: BTreeSet<Name>,
    ) -> Self {
        Self {
            local_pending: local_pending_visible_at_terminal,
            active_role: active_role_visible_at_terminal,
            role_pending: role_pending_visible_at_terminal,
            role_discharged: role_discharged_visible_at_terminal,
        }
    }

    /// Borrow the local pending obligations visible in the terminal observed child context.
    pub fn local_pending_visible_at_terminal(&self) -> &BTreeSet<Name> {
        &self.local_pending
    }

    /// Return the active role name visible at terminal observation time, if any.
    pub fn active_role_visible_at_terminal(&self) -> Option<&str> {
        self.active_role.as_deref()
    }

    /// Borrow the role pending obligations visible at terminal observation time.
    pub fn role_pending_visible_at_terminal(&self) -> &BTreeSet<Name> {
        &self.role_pending
    }

    /// Borrow the role discharged obligations visible at terminal observation time.
    pub fn role_discharged_visible_at_terminal(&self) -> &BTreeSet<Name> {
        &self.role_discharged
    }
}

/// Honest retained provenance summary for the runtime-owned child identity and spawn ancestry the
/// current runtime can actually snapshot.
///
/// This is intentionally not full `CompletionPayload.provenance` parity. It preserves only the
/// runtime-owned child workflow identity plus immediate spawn ancestry derived from the spawned
/// child lifecycle path. In particular, this summary reflects the runtime's own control-link /
/// child-registration lineage rather than claiming exact cumulative terminal `π'` transport.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConservativeRetainedProvenanceSummary {
    workflow_id: WorkflowId,
    parent_workflow_id: Option<WorkflowId>,
    lineage: Vec<WorkflowId>,
}

impl ConservativeRetainedProvenanceSummary {
    /// Create one conservative retained provenance summary.
    pub fn new(
        workflow_id: WorkflowId,
        parent_workflow_id: Option<WorkflowId>,
        lineage: Vec<WorkflowId>,
    ) -> Self {
        Self {
            workflow_id,
            parent_workflow_id,
            lineage,
        }
    }

    /// Return the runtime-owned child workflow identity retained for this completion.
    pub fn workflow_id(&self) -> WorkflowId {
        self.workflow_id
    }

    /// Return the immediate runtime-owned parent workflow identity, if any.
    pub fn parent_workflow_id(&self) -> Option<WorkflowId> {
        self.parent_workflow_id
    }

    /// Borrow the runtime-owned spawn lineage retained for this completion.
    pub fn lineage(&self) -> &[WorkflowId] {
        &self.lineage
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetainedCompletionRecord {
    /// The supervised workflow instance whose terminal observation is being retained.
    instance_id: WorkflowId,
    /// The authoritative coarse-grained terminal outcome/state currently retained.
    outcome_state: RuntimeOutcomeState,
    /// The direct terminal child result payload when the retained observation came from child
    /// completion rather than from a control tombstone.
    ///
    /// This is the first honest `CompletionPayload.result`-like slice the runtime can currently
    /// preserve. Control tombstones keep this as `None` so they remain distinguishable from
    /// child-owned completion payloads.
    result: Option<Box<ExecResult<Value>>>,
    /// Conservative retained `CompletionPayload.effects`-like summary contents for child-owned
    /// retained completions.
    ///
    /// The current runtime does not transport the full authoritative trace/carrier state, so this
    /// stays as the narrowest honest effect-summary slice it can retain today. Control tombstones
    /// keep this as `None` so they remain distinct from child-owned retained payloads.
    effects: Option<ConservativeRetainedEffectSummary>,
    /// Honest retained obligations summary observed at child terminal time for child-owned
    /// completions.
    ///
    /// This intentionally reflects only the obligation state the runtime can actually snapshot from
    /// the observed terminal execution context. It does not claim full `CompletionPayload`
    /// obligations parity across all cumulative carriers or hidden/earlier frames. Control
    /// tombstones keep this as `None` so they remain distinct from child-owned retained payloads.
    obligations: Option<ConservativeRetainedObligationsSummary>,
    /// Honest retained provenance summary for runtime-owned child identity and spawn ancestry.
    ///
    /// This preserves only the identity/lineage slice the current runtime can snapshot from its own
    /// spawned-child lifecycle path. It deliberately does not claim exact full
    /// `CompletionPayload.provenance` parity. Control tombstones keep this as `None` so they remain
    /// distinct from child-owned retained payloads.
    provenance: Option<ConservativeRetainedProvenanceSummary>,
    /// Whether this retained observation came from explicit completion recording or terminal
    /// control invalidation.
    kind: RetainedCompletionKind,
}

impl RetainedCompletionRecord {
    /// Create a retained record for explicit terminal completion observation.
    pub(crate) fn completed(
        instance_id: WorkflowId,
        result: ExecResult<Value>,
        effects: ConservativeRetainedEffectSummary,
        obligations: ConservativeRetainedObligationsSummary,
        provenance: Option<ConservativeRetainedProvenanceSummary>,
    ) -> Self {
        let outcome_state = RuntimeOutcomeState::from_exec_result(&result);
        debug_assert!(
            outcome_state.is_terminal(),
            "retained completion observations must be terminal"
        );
        Self {
            instance_id,
            outcome_state,
            result: Some(Box::new(result)),
            effects: Some(effects),
            obligations: Some(obligations),
            provenance,
            kind: RetainedCompletionKind::Completed,
        }
    }

    /// Return the supervised workflow instance whose terminal observation is being retained.
    pub fn instance_id(&self) -> WorkflowId {
        self.instance_id
    }

    /// Return the authoritative coarse-grained terminal outcome/state retained for this control
    /// target.
    pub fn outcome_state(&self) -> RuntimeOutcomeState {
        self.outcome_state
    }

    /// Return why this retained terminal observation exists.
    pub fn kind(&self) -> RetainedCompletionKind {
        self.kind
    }

    /// Borrow the retained direct terminal child result payload, if this record came from child
    /// completion rather than a control tombstone.
    pub fn terminal_result(&self) -> Option<&ExecResult<Value>> {
        self.result.as_deref()
    }

    /// Borrow the retained effect-summary slice, if this record came from child completion rather
    /// than a control tombstone.
    pub fn conservative_effect_summary(&self) -> Option<&ConservativeRetainedEffectSummary> {
        self.effects.as_ref()
    }

    /// Borrow the honest retained obligations summary slice, if this record came from child
    /// completion rather than a control tombstone.
    pub fn conservative_obligations_summary(
        &self,
    ) -> Option<&ConservativeRetainedObligationsSummary> {
        self.obligations.as_ref()
    }

    /// Borrow the honest retained provenance summary slice, if this record came from child
    /// completion rather than a control tombstone.
    pub fn conservative_provenance_summary(
        &self,
    ) -> Option<&ConservativeRetainedProvenanceSummary> {
        self.provenance.as_ref()
    }

    /// Create a retained record for terminal supervisor control invalidation.
    pub(crate) fn control_terminated(instance_id: WorkflowId) -> Self {
        Self {
            instance_id,
            outcome_state: RuntimeOutcomeState::InvalidOrTerminated,
            result: None,
            effects: None,
            obligations: None,
            provenance: None,
            kind: RetainedCompletionKind::ControlTerminated,
        }
    }
}

impl ControlLinkError {
    /// Classify this control-link error into the authoritative runtime outcome/state surface.
    pub fn runtime_outcome_state(&self) -> RuntimeOutcomeState {
        match self {
            Self::NotFound(..) | Self::Terminated(..) => RuntimeOutcomeState::InvalidOrTerminated,
            Self::NonTerminalObservation(..) => RuntimeOutcomeState::ExecutionFailure,
            Self::CompletionAlreadySealed(..) => RuntimeOutcomeState::InvalidOrTerminated,
        }
    }
}

impl LinkState {
    /// Classify this control-link lifecycle state into the authoritative runtime outcome/state
    /// surface.
    pub fn runtime_outcome_state(self) -> RuntimeOutcomeState {
        match self {
            Self::Running => RuntimeOutcomeState::Active,
            Self::Paused => RuntimeOutcomeState::BlockedOrSuspended,
            Self::Terminated => RuntimeOutcomeState::InvalidOrTerminated,
        }
    }
}

/// Registry for runtime control-link lifecycle.
///
/// A control link is reusable while the target instance remains valid. Non-terminal operations
/// such as pause, resume, and health checks do not consume the link. Kill is terminal and
/// invalidates future control operations.
#[derive(Debug, Clone, Default)]
pub struct ControlLinkRegistry {
    links: HashMap<WorkflowId, LinkState>,
    state_signals: HashMap<WorkflowId, watch::Sender<LinkState>>,
    completion_signals: HashMap<WorkflowId, watch::Sender<Option<RetainedCompletionRecord>>>,
    retained_completions: HashMap<WorkflowId, RetainedCompletionRecord>,
    live_spawn_provenance: HashMap<WorkflowId, ConservativeRetainedProvenanceSummary>,
}

#[derive(Debug, Clone)]
enum TerminalTransition {
    Completion {
        result: Box<ExecResult<Value>>,
        effects: Box<ConservativeRetainedEffectSummary>,
        obligations: Box<ConservativeRetainedObligationsSummary>,
        provenance: Option<Box<ConservativeRetainedProvenanceSummary>>,
    },
    ControlKill,
}

#[derive(Debug)]
pub(crate) enum RetainedCompletionWaiter {
    Ready(Box<RetainedCompletionRecord>),
    Pending(watch::Receiver<Option<RetainedCompletionRecord>>),
}

impl ControlLinkRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            links: HashMap::new(),
            state_signals: HashMap::new(),
            completion_signals: HashMap::new(),
            retained_completions: HashMap::new(),
            live_spawn_provenance: HashMap::new(),
        }
    }

    fn set_state(&mut self, instance_id: WorkflowId, state: LinkState) {
        self.links.insert(instance_id, state);
        if let Some(signal) = self.state_signals.get(&instance_id) {
            let _ = signal.send(state);
        }
    }

    fn seal_retained_completion(&mut self, record: RetainedCompletionRecord) {
        let instance_id = record.instance_id();
        self.set_state(instance_id, LinkState::Terminated);
        self.retained_completions
            .insert(instance_id, record.clone());
        if let Some(signal) = self.completion_signals.get(&instance_id) {
            let _ = signal.send(Some(record));
        }
    }

    fn seal_terminal_transition(
        &mut self,
        link: &ControlLink,
        transition: TerminalTransition,
    ) -> Result<RetainedCompletionRecord, ControlLinkError> {
        let instance_id = link.instance_id;
        match self.links.get(&instance_id).copied() {
            Some(LinkState::Running) | Some(LinkState::Paused) => {}
            Some(LinkState::Terminated) => {
                return match self.retained_completions.get(&instance_id).cloned() {
                    Some(record) => Err(ControlLinkError::CompletionAlreadySealed(
                        instance_id,
                        Box::new(record),
                    )),
                    None => Err(ControlLinkError::Terminated(instance_id)),
                };
            }
            None => return Err(ControlLinkError::NotFound(instance_id)),
        }

        let record = match transition {
            TerminalTransition::Completion {
                result,
                effects,
                obligations,
                provenance,
            } => {
                let outcome_state = RuntimeOutcomeState::from_exec_result(&result);
                if !outcome_state.is_terminal() {
                    return Err(ControlLinkError::NonTerminalObservation(
                        instance_id,
                        outcome_state,
                    ));
                }
                let retained_provenance = provenance
                    .map(|summary| *summary)
                    .or_else(|| self.live_spawn_provenance.remove(&instance_id));
                RetainedCompletionRecord::completed(
                    instance_id,
                    *result,
                    *effects,
                    *obligations,
                    retained_provenance,
                )
            }
            TerminalTransition::ControlKill => {
                self.live_spawn_provenance.remove(&instance_id);
                RetainedCompletionRecord::control_terminated(instance_id)
            }
        };

        if let Some(existing) = self.retained_completions.get(&instance_id).cloned() {
            return Err(ControlLinkError::CompletionAlreadySealed(
                instance_id,
                Box::new(existing),
            ));
        }

        self.seal_retained_completion(record.clone());
        Ok(record)
    }

    /// Register a new control link as running.
    pub fn register(&mut self, instance_id: WorkflowId) {
        let (signal, _) = watch::channel(LinkState::Running);
        let (completion_signal, _) = watch::channel(None);
        self.links.insert(instance_id, LinkState::Running);
        self.state_signals.insert(instance_id, signal);
        self.completion_signals
            .insert(instance_id, completion_signal);
        self.retained_completions.remove(&instance_id);
        self.live_spawn_provenance.remove(&instance_id);
    }

    /// Register a new runtime-owned spawned control link with conservative spawn provenance.
    pub fn register_with_spawn_provenance(
        &mut self,
        provenance: ConservativeRetainedProvenanceSummary,
    ) {
        let instance_id = provenance.workflow_id();
        self.register(instance_id);
        self.live_spawn_provenance.insert(instance_id, provenance);
    }

    fn get_live_state(&self, link: &ControlLink) -> Result<LinkState, ControlLinkError> {
        match self.links.get(&link.instance_id).copied() {
            Some(LinkState::Running) => Ok(LinkState::Running),
            Some(LinkState::Paused) => Ok(LinkState::Paused),
            Some(LinkState::Terminated) => Err(ControlLinkError::Terminated(link.instance_id)),
            None => Err(ControlLinkError::NotFound(link.instance_id)),
        }
    }

    /// Check that the control link still points to a valid supervised instance.
    pub fn check_health(&self, link: &ControlLink) -> Result<LinkState, ControlLinkError> {
        self.get_live_state(link)
    }

    /// Subscribe to live state transitions for one supervised instance.
    pub fn subscribe(
        &self,
        link: &ControlLink,
    ) -> Result<watch::Receiver<LinkState>, ControlLinkError> {
        self.get_live_state(link)?;
        self.state_signals
            .get(&link.instance_id)
            .map(watch::Sender::subscribe)
            .ok_or(ControlLinkError::NotFound(link.instance_id))
    }

    /// Pause the supervised instance if it is still valid.
    pub fn pause(&mut self, link: &ControlLink) -> Result<(), ControlLinkError> {
        match self.get_live_state(link)? {
            LinkState::Running | LinkState::Paused => {
                self.set_state(link.instance_id, LinkState::Paused);
                Ok(())
            }
            LinkState::Terminated => Err(ControlLinkError::Terminated(link.instance_id)),
        }
    }

    /// Resume the supervised instance if it is still valid.
    pub fn resume(&mut self, link: &ControlLink) -> Result<(), ControlLinkError> {
        match self.get_live_state(link)? {
            LinkState::Running | LinkState::Paused => {
                self.set_state(link.instance_id, LinkState::Running);
                Ok(())
            }
            LinkState::Terminated => Err(ControlLinkError::Terminated(link.instance_id)),
        }
    }

    /// Kill the supervised instance, invalidating future control operations.
    pub fn kill(&mut self, link: &ControlLink) -> Result<(), ControlLinkError> {
        match self.seal_terminal_transition(link, TerminalTransition::ControlKill) {
            Ok(_) => Ok(()),
            Err(ControlLinkError::CompletionAlreadySealed(..))
            | Err(ControlLinkError::Terminated(..)) => {
                Err(ControlLinkError::Terminated(link.instance_id))
            }
            Err(error) => Err(error),
        }
    }

    /// Retain one explicit terminal completion observation for a supervised instance.
    ///
    /// Terminal observations are write-once: after the first retained record is sealed for a
    /// control target, later attempts to rewrite it fail with
    /// [`ControlLinkError::CompletionAlreadySealed`].
    pub fn record_completion(
        &mut self,
        link: &ControlLink,
        result: ExecResult<Value>,
        effects: ConservativeRetainedEffectSummary,
        obligations: ConservativeRetainedObligationsSummary,
        provenance: Option<ConservativeRetainedProvenanceSummary>,
    ) -> Result<RetainedCompletionRecord, ControlLinkError> {
        self.seal_terminal_transition(
            link,
            TerminalTransition::Completion {
                result: Box::new(result),
                effects: Box::new(effects),
                obligations: Box::new(obligations),
                provenance: provenance.map(Box::new),
            },
        )
    }

    /// Remove a control link from the registry.
    pub fn remove(&mut self, instance_id: &WorkflowId) -> Option<LinkState> {
        self.retained_completions.remove(instance_id);
        self.live_spawn_provenance.remove(instance_id);
        self.completion_signals.remove(instance_id);
        self.state_signals.remove(instance_id);
        self.links.remove(instance_id)
    }

    /// Get the current state of a control link.
    pub fn get_state(&self, instance_id: &WorkflowId) -> Option<LinkState> {
        self.links.get(instance_id).copied()
    }

    /// Get the retained terminal completion-style observation for a control link target.
    pub fn retained_completion(
        &self,
        instance_id: &WorkflowId,
    ) -> Option<RetainedCompletionRecord> {
        self.retained_completions.get(instance_id).cloned()
    }

    /// Prepare a dedicated wait handle for the first sealed retained completion record.
    ///
    /// This reuses the authoritative retained completion carrier. If the target already has a
    /// sealed retained record, the waiter resolves immediately to that record. If the target is
    /// registered but not yet sealed, the waiter subscribes to the future first sealed record. If
    /// the target is invalid or unregistered, the error remains distinguishable from a real
    /// retained completion observation.
    pub(crate) fn retained_completion_waiter(
        &self,
        link: &ControlLink,
    ) -> Result<RetainedCompletionWaiter, ControlLinkError> {
        if let Some(record) = self.retained_completion(&link.instance_id) {
            return Ok(RetainedCompletionWaiter::Ready(Box::new(record)));
        }

        match self.links.get(&link.instance_id).copied() {
            Some(LinkState::Running) | Some(LinkState::Paused) => self
                .completion_signals
                .get(&link.instance_id)
                .map(watch::Sender::subscribe)
                .map(RetainedCompletionWaiter::Pending)
                .ok_or(ControlLinkError::NotFound(link.instance_id)),
            Some(LinkState::Terminated) => {
                unreachable!(
                    "terminated control targets should either keep a sealed retained record or be removed"
                )
            }
            None => Err(ControlLinkError::NotFound(link.instance_id)),
        }
    }

    /// Read the conservative live spawn provenance tracked for a running spawned control target.
    pub fn live_spawn_provenance(
        &self,
        instance_id: &WorkflowId,
    ) -> Option<ConservativeRetainedProvenanceSummary> {
        self.live_spawn_provenance.get(instance_id).cloned()
    }

    /// Returns the number of registered control links.
    pub fn len(&self) -> usize {
        self.links.len()
    }

    /// Returns true if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.links.is_empty()
    }

    /// Returns the number of live control links.
    pub fn live_count(&self) -> usize {
        self.links
            .values()
            .filter(|&&state| state != LinkState::Terminated)
            .count()
    }

    /// Returns the number of terminated control links.
    pub fn terminated_count(&self) -> usize {
        self.links
            .values()
            .filter(|&&state| state == LinkState::Terminated)
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn retained_effect_summary(
        terminal: Effect,
        reached: &[Effect],
    ) -> ConservativeRetainedEffectSummary {
        ConservativeRetainedEffectSummary::new(terminal, reached.iter().copied().collect())
    }

    fn retained_obligations_summary() -> ConservativeRetainedObligationsSummary {
        ConservativeRetainedObligationsSummary::new(
            BTreeSet::new(),
            None,
            BTreeSet::new(),
            BTreeSet::new(),
        )
    }

    fn retained_provenance_summary(
        workflow_id: WorkflowId,
        parent_workflow_id: Option<WorkflowId>,
        lineage: Vec<WorkflowId>,
    ) -> ConservativeRetainedProvenanceSummary {
        ConservativeRetainedProvenanceSummary::new(workflow_id, parent_workflow_id, lineage)
    }

    fn test_instance_id() -> WorkflowId {
        WorkflowId::new()
    }

    fn test_control_link(instance_id: WorkflowId) -> ControlLink {
        ControlLink { instance_id }
    }

    #[test]
    fn register_defaults_to_running() {
        let mut registry = ControlLinkRegistry::new();
        let id = test_instance_id();

        registry.register(id);

        assert_eq!(registry.get_state(&id), Some(LinkState::Running));
    }

    #[test]
    fn pause_and_resume_are_reusable() {
        let mut registry = ControlLinkRegistry::new();
        let id = test_instance_id();
        let link = test_control_link(id);
        registry.register(id);

        assert!(registry.pause(&link).is_ok());
        assert_eq!(registry.get_state(&id), Some(LinkState::Paused));
        assert_eq!(registry.check_health(&link), Ok(LinkState::Paused));

        assert!(registry.resume(&link).is_ok());
        assert_eq!(registry.get_state(&id), Some(LinkState::Running));
        assert_eq!(registry.check_health(&link), Ok(LinkState::Running));
    }

    #[test]
    fn kill_is_terminal() {
        let mut registry = ControlLinkRegistry::new();
        let id = test_instance_id();
        let link = test_control_link(id);
        registry.register(id);

        assert!(registry.kill(&link).is_ok());
        assert_eq!(registry.get_state(&id), Some(LinkState::Terminated));
        assert_eq!(
            registry.check_health(&link),
            Err(ControlLinkError::Terminated(id))
        );
        assert_eq!(registry.pause(&link), Err(ControlLinkError::Terminated(id)));
        assert_eq!(
            registry.resume(&link),
            Err(ControlLinkError::Terminated(id))
        );
        assert_eq!(registry.kill(&link), Err(ControlLinkError::Terminated(id)));
    }

    #[test]
    fn missing_link_fails() {
        let mut registry = ControlLinkRegistry::new();
        let id = test_instance_id();
        let link = test_control_link(id);

        assert_eq!(
            registry.check_health(&link),
            Err(ControlLinkError::NotFound(id))
        );
        assert_eq!(registry.pause(&link), Err(ControlLinkError::NotFound(id)));
        assert_eq!(registry.resume(&link), Err(ControlLinkError::NotFound(id)));
        assert_eq!(registry.kill(&link), Err(ControlLinkError::NotFound(id)));
    }

    #[test]
    fn remove_forgets_link_state() {
        let mut registry = ControlLinkRegistry::new();
        let id = test_instance_id();
        registry.register(id);
        registry.kill(&test_control_link(id)).unwrap();

        assert_eq!(registry.remove(&id), Some(LinkState::Terminated));
        assert!(registry.get_state(&id).is_none());
        assert!(registry.retained_completion(&id).is_none());
        assert!(registry.is_empty());
    }

    #[test]
    fn record_completion_retains_explicit_terminal_observation() {
        let mut registry = ControlLinkRegistry::new();
        let id = test_instance_id();
        let link = test_control_link(id);
        registry.register(id);

        let obligations = retained_obligations_summary();
        let provenance = retained_provenance_summary(id, None, vec![]);
        let record = registry
            .record_completion(
                &link,
                Ok(Value::Int(7)),
                retained_effect_summary(Effect::Operational, &[Effect::Operational]),
                obligations.clone(),
                Some(provenance.clone()),
            )
            .unwrap();

        assert_eq!(
            record,
            RetainedCompletionRecord::completed(
                id,
                Ok(Value::Int(7)),
                retained_effect_summary(Effect::Operational, &[Effect::Operational]),
                obligations,
                Some(provenance),
            )
        );
        assert_eq!(registry.get_state(&id), Some(LinkState::Terminated));
        assert_eq!(registry.retained_completion(&id), Some(record));
    }

    #[test]
    fn record_completion_is_write_once_after_seal() {
        let mut registry = ControlLinkRegistry::new();
        let id = test_instance_id();
        let link = test_control_link(id);
        registry.register(id);

        let first = registry
            .record_completion(
                &link,
                Ok(Value::Int(1)),
                retained_effect_summary(Effect::Operational, &[Effect::Operational]),
                retained_obligations_summary(),
                Some(retained_provenance_summary(id, None, vec![])),
            )
            .unwrap();
        let error = registry
            .record_completion(
                &link,
                Ok(Value::Int(2)),
                retained_effect_summary(Effect::Epistemic, &[Effect::Epistemic]),
                retained_obligations_summary(),
                Some(retained_provenance_summary(id, None, vec![])),
            )
            .expect_err("sealed retained completion must not be overwritten");

        assert_eq!(
            error,
            ControlLinkError::CompletionAlreadySealed(id, Box::new(first.clone()))
        );
        assert_eq!(registry.retained_completion(&id), Some(first));
    }

    #[test]
    fn kill_retains_terminal_control_tombstone() {
        let mut registry = ControlLinkRegistry::new();
        let id = test_instance_id();
        let link = test_control_link(id);
        registry.register(id);

        registry.kill(&link).unwrap();

        assert_eq!(
            registry.retained_completion(&id),
            Some(RetainedCompletionRecord::control_terminated(id))
        );
    }

    #[test]
    fn kill_seals_control_tombstone_against_later_completion_rewrite() {
        let mut registry = ControlLinkRegistry::new();
        let id = test_instance_id();
        let link = test_control_link(id);
        registry.register(id);

        registry.kill(&link).unwrap();

        let error = registry
            .record_completion(
                &link,
                Ok(Value::Int(1)),
                retained_effect_summary(Effect::Operational, &[Effect::Operational]),
                retained_obligations_summary(),
                Some(retained_provenance_summary(id, None, vec![])),
            )
            .expect_err("killed links must keep their original retained tombstone");

        assert_eq!(
            error,
            ControlLinkError::CompletionAlreadySealed(
                id,
                Box::new(RetainedCompletionRecord::control_terminated(id))
            )
        );
        assert_eq!(
            registry.retained_completion(&id),
            Some(RetainedCompletionRecord::control_terminated(id))
        );
    }

    #[test]
    fn completion_seals_before_later_kill_can_win() {
        let mut registry = ControlLinkRegistry::new();
        let id = test_instance_id();
        let link = test_control_link(id);
        registry.register(id);

        let record = registry
            .record_completion(
                &link,
                Ok(Value::Int(1)),
                retained_effect_summary(Effect::Operational, &[Effect::Operational]),
                retained_obligations_summary(),
                Some(retained_provenance_summary(id, None, vec![])),
            )
            .expect("completion should seal the first terminal observation");
        let error = registry
            .kill(&link)
            .expect_err("later kill must not replace an already-sealed completion");

        assert_eq!(error, ControlLinkError::Terminated(id));
        assert_eq!(registry.retained_completion(&id), Some(record));
    }

    #[test]
    fn kill_seals_before_later_completion_can_win() {
        let mut registry = ControlLinkRegistry::new();
        let id = test_instance_id();
        let link = test_control_link(id);
        registry.register(id);

        registry
            .kill(&link)
            .expect("kill should seal the first terminal observation");
        let error = registry
            .record_completion(
                &link,
                Ok(Value::Int(1)),
                retained_effect_summary(Effect::Operational, &[Effect::Operational]),
                retained_obligations_summary(),
                Some(retained_provenance_summary(id, None, vec![])),
            )
            .expect_err("later completion must not replace an already-sealed kill tombstone");

        assert_eq!(
            error,
            ControlLinkError::CompletionAlreadySealed(
                id,
                Box::new(RetainedCompletionRecord::control_terminated(id))
            )
        );
        assert_eq!(
            registry.retained_completion(&id),
            Some(RetainedCompletionRecord::control_terminated(id))
        );
    }

    #[test]
    fn record_completion_rejects_non_terminal_outcomes() {
        let mut registry = ControlLinkRegistry::new();
        let id = test_instance_id();
        let link = test_control_link(id);
        registry.register(id);

        let error = registry
            .record_completion(
                &link,
                Err(crate::error::ExecError::Blocked("waiting".to_string())),
                retained_effect_summary(Effect::Epistemic, &[Effect::Epistemic]),
                retained_obligations_summary(),
                Some(retained_provenance_summary(id, None, vec![])),
            )
            .expect_err("non-terminal states must not be retained as completion observations");

        assert_eq!(
            error,
            ControlLinkError::NonTerminalObservation(id, RuntimeOutcomeState::BlockedOrSuspended)
        );
        assert!(registry.retained_completion(&id).is_none());
        assert_eq!(registry.get_state(&id), Some(LinkState::Running));
    }

    #[test]
    fn record_completion_retains_effect_summary_contents() {
        let mut registry = ControlLinkRegistry::new();
        let id = test_instance_id();
        let link = test_control_link(id);
        registry.register(id);
        let effects = retained_effect_summary(
            Effect::Operational,
            &[Effect::Epistemic, Effect::Deliberative, Effect::Operational],
        );

        let obligations = retained_obligations_summary();
        let provenance = retained_provenance_summary(id, None, vec![]);
        let record = registry
            .record_completion(
                &link,
                Ok(Value::Int(7)),
                effects.clone(),
                obligations.clone(),
                Some(provenance.clone()),
            )
            .unwrap();

        assert_eq!(record.conservative_effect_summary(), Some(&effects));
        assert_eq!(
            record.conservative_obligations_summary(),
            Some(&obligations)
        );
        assert_eq!(record.conservative_provenance_summary(), Some(&provenance));
    }

    #[test]
    fn control_tombstones_keep_effect_summary_absent() {
        let mut registry = ControlLinkRegistry::new();
        let id = test_instance_id();
        let link = test_control_link(id);
        registry.register(id);
        registry.kill(&link).unwrap();

        let record = registry.retained_completion(&id).unwrap();
        assert_eq!(record.conservative_effect_summary(), None);
        assert_eq!(record.terminal_result(), None);
        assert_eq!(record.conservative_provenance_summary(), None);
    }

    #[test]
    fn retained_completion_waiter_returns_immediately_for_already_sealed_record() {
        let mut registry = ControlLinkRegistry::new();
        let id = test_instance_id();
        let link = test_control_link(id);
        registry.register(id);

        let sealed = registry
            .record_completion(
                &link,
                Ok(Value::Int(7)),
                retained_effect_summary(Effect::Operational, &[Effect::Operational]),
                retained_obligations_summary(),
                Some(retained_provenance_summary(id, None, vec![])),
            )
            .expect("completion should seal the retained record");

        match registry
            .retained_completion_waiter(&link)
            .expect("already-sealed targets should resolve immediately")
        {
            RetainedCompletionWaiter::Ready(record) => assert_eq!(record, Box::new(sealed)),
            RetainedCompletionWaiter::Pending(..) => {
                panic!("already-sealed targets must not require waiting")
            }
        }
    }

    #[test]
    fn retained_completion_waiter_rejects_unregistered_targets() {
        let registry = ControlLinkRegistry::new();
        let id = test_instance_id();
        let link = test_control_link(id);

        assert!(matches!(
            registry.retained_completion_waiter(&link),
            Err(ControlLinkError::NotFound(not_found_id)) if not_found_id == id
        ));
    }

    #[test]
    fn register_with_spawn_provenance_promotes_live_summary_to_retained_completion() {
        let mut registry = ControlLinkRegistry::new();
        let parent_id = test_instance_id();
        let id = test_instance_id();
        let link = test_control_link(id);
        let provenance = retained_provenance_summary(id, Some(parent_id), vec![parent_id]);

        registry.register_with_spawn_provenance(provenance.clone());

        assert_eq!(
            registry.live_spawn_provenance(&id),
            Some(provenance.clone())
        );

        let record = registry
            .record_completion(
                &link,
                Err(crate::error::ExecError::ExecutionFailed("boom".to_string())),
                retained_effect_summary(Effect::Operational, &[Effect::Operational]),
                retained_obligations_summary(),
                None,
            )
            .expect("runtime-owned live spawn provenance should be retained on completion");

        assert_eq!(record.conservative_provenance_summary(), Some(&provenance));
        assert_eq!(registry.live_spawn_provenance(&id), None);
    }

    #[test]
    fn link_state_runtime_outcome_mapping_is_authoritative() {
        assert_eq!(
            LinkState::Running.runtime_outcome_state(),
            RuntimeOutcomeState::Active
        );
        assert_eq!(
            LinkState::Paused.runtime_outcome_state(),
            RuntimeOutcomeState::BlockedOrSuspended
        );
        assert_eq!(
            LinkState::Terminated.runtime_outcome_state(),
            RuntimeOutcomeState::InvalidOrTerminated
        );
    }
}
