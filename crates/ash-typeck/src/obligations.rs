//! Obligation tracking and proof obligations (TASK-023, TASK-024)
//!
//! Provides tracking of obligations and proof obligations for the type system,
//! ensuring that workflows satisfy their declared obligations.

use crate::types::Type;
use std::collections::HashMap;

/// A proof obligation that must be satisfied
#[derive(Debug, Clone, PartialEq)]
pub enum ProofObligation {
    /// Obligation to check a condition
    CheckCondition { condition: Box<str>, at: EffectTime },
    /// Obligation to maintain an invariant
    MaintainInvariant { invariant: Box<str> },
    /// Obligation to satisfy a policy
    SatisfyPolicy { policy: Box<str> },
    /// Obligation to fulfill a role requirement
    FulfillRole { role: Box<str> },
    /// Obligation to audit an action
    AuditAction { action: Box<str> },
    /// Custom obligation with a name and type
    Custom { name: Box<str>, proof_type: Type },
}

impl ProofObligation {
    /// Get the name of this obligation
    pub fn name(&self) -> &str {
        match self {
            ProofObligation::CheckCondition { condition, .. } => condition,
            ProofObligation::MaintainInvariant { invariant } => invariant,
            ProofObligation::SatisfyPolicy { policy } => policy,
            ProofObligation::FulfillRole { role } => role,
            ProofObligation::AuditAction { action } => action,
            ProofObligation::Custom { name, .. } => name,
        }
    }

    /// Create a check condition obligation
    pub fn check_condition(condition: impl Into<Box<str>>, at: EffectTime) -> Self {
        Self::CheckCondition {
            condition: condition.into(),
            at,
        }
    }

    /// Create a maintain invariant obligation
    pub fn maintain_invariant(invariant: impl Into<Box<str>>) -> Self {
        Self::MaintainInvariant {
            invariant: invariant.into(),
        }
    }

    /// Create a satisfy policy obligation
    pub fn satisfy_policy(policy: impl Into<Box<str>>) -> Self {
        Self::SatisfyPolicy {
            policy: policy.into(),
        }
    }

    /// Create a fulfill role obligation
    pub fn fulfill_role(role: impl Into<Box<str>>) -> Self {
        Self::FulfillRole { role: role.into() }
    }

    /// Create an audit action obligation
    pub fn audit_action(action: impl Into<Box<str>>) -> Self {
        Self::AuditAction {
            action: action.into(),
        }
    }

    /// Create a custom obligation
    pub fn custom(name: impl Into<Box<str>>, proof_type: Type) -> Self {
        Self::Custom {
            name: name.into(),
            proof_type,
        }
    }
}

/// When an obligation must be satisfied
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectTime {
    /// Before the action
    Before,
    /// After the action
    After,
    /// During the action
    During,
    /// At workflow completion
    Completion,
}

impl std::fmt::Display for EffectTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EffectTime::Before => write!(f, "before"),
            EffectTime::After => write!(f, "after"),
            EffectTime::During => write!(f, "during"),
            EffectTime::Completion => write!(f, "completion"),
        }
    }
}

/// Status of an obligation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObligationStatus {
    /// Obligation is pending (not yet checked)
    Pending,
    /// Obligation is satisfied
    Satisfied,
    /// Obligation failed
    Failed,
    /// Obligation is waived
    Waived,
}

impl std::fmt::Display for ObligationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObligationStatus::Pending => write!(f, "pending"),
            ObligationStatus::Satisfied => write!(f, "satisfied"),
            ObligationStatus::Failed => write!(f, "failed"),
            ObligationStatus::Waived => write!(f, "waived"),
        }
    }
}

/// A tracked obligation with its status
#[derive(Debug, Clone)]
pub struct TrackedObligation {
    /// The obligation
    pub obligation: ProofObligation,
    /// Current status
    pub status: ObligationStatus,
    /// Optional proof witness
    pub witness: Option<ProofWitness>,
}

impl TrackedObligation {
    /// Create a new tracked obligation
    pub fn new(obligation: ProofObligation) -> Self {
        Self {
            obligation,
            status: ObligationStatus::Pending,
            witness: None,
        }
    }

    /// Mark as satisfied
    pub fn satisfy(&mut self, witness: Option<ProofWitness>) {
        self.status = ObligationStatus::Satisfied;
        self.witness = witness;
    }

    /// Mark as failed
    pub fn fail(&mut self) {
        self.status = ObligationStatus::Failed;
    }

    /// Mark as waived
    pub fn waive(&mut self) {
        self.status = ObligationStatus::Waived;
    }

    /// Check if satisfied
    pub fn is_satisfied(&self) -> bool {
        self.status == ObligationStatus::Satisfied
    }

    /// Check if failed
    pub fn is_failed(&self) -> bool {
        self.status == ObligationStatus::Failed
    }
}

/// A proof witness for an obligation
#[derive(Debug, Clone, PartialEq)]
pub enum ProofWitness {
    /// Proof by direct verification
    Direct,
    /// Proof by logical derivation
    Derivation(Vec<Box<str>>),
    /// Proof by external verification
    External(Box<str>),
    /// Proof by assumption
    Assumption(Box<str>),
}

/// Tracker for obligations during type checking (TASK-023)
#[derive(Debug, Clone, Default)]
pub struct ObligationTracker {
    /// Tracked obligations
    obligations: Vec<TrackedObligation>,
    /// Named obligation mappings
    named: HashMap<Box<str>, usize>,
}

impl ObligationTracker {
    /// Create a new empty obligation tracker
    pub fn new() -> Self {
        Self {
            obligations: Vec::new(),
            named: HashMap::new(),
        }
    }

    /// Add an obligation to track
    pub fn add(&mut self, obligation: ProofObligation) -> usize {
        let id = self.obligations.len();
        let name = obligation.name().to_string().into_boxed_str();
        self.obligations.push(TrackedObligation::new(obligation));
        self.named.insert(name, id);
        id
    }

    /// Get an obligation by ID
    pub fn get(&self, id: usize) -> Option<&TrackedObligation> {
        self.obligations.get(id)
    }

    /// Get a mutable obligation by ID
    pub fn get_mut(&mut self, id: usize) -> Option<&mut TrackedObligation> {
        self.obligations.get_mut(id)
    }

    /// Lookup an obligation by name
    pub fn lookup(&self, name: &str) -> Option<&TrackedObligation> {
        self.named.get(name).and_then(|&id| self.get(id))
    }

    /// Mark an obligation as satisfied
    pub fn satisfy(&mut self, id: usize, witness: Option<ProofWitness>) -> bool {
        if let Some(obl) = self.get_mut(id) {
            obl.satisfy(witness);
            true
        } else {
            false
        }
    }

    /// Mark a named obligation as satisfied
    pub fn satisfy_named(&mut self, name: &str, witness: Option<ProofWitness>) -> bool {
        if let Some(&id) = self.named.get(name) {
            self.satisfy(id, witness)
        } else {
            false
        }
    }

    /// Mark an obligation as failed
    pub fn fail(&mut self, id: usize) -> bool {
        if let Some(obl) = self.get_mut(id) {
            obl.fail();
            true
        } else {
            false
        }
    }

    /// Mark an obligation as waived
    pub fn waive(&mut self, id: usize) -> bool {
        if let Some(obl) = self.get_mut(id) {
            obl.waive();
            true
        } else {
            false
        }
    }

    /// Get all obligations
    pub fn all(&self) -> &[TrackedObligation] {
        &self.obligations
    }

    /// Get all pending obligations
    pub fn pending(&self) -> impl Iterator<Item = (usize, &TrackedObligation)> {
        self.obligations
            .iter()
            .enumerate()
            .filter(|(_, o)| o.status == ObligationStatus::Pending)
    }

    /// Get all satisfied obligations
    pub fn satisfied(&self) -> impl Iterator<Item = (usize, &TrackedObligation)> {
        self.obligations
            .iter()
            .enumerate()
            .filter(|(_, o)| o.status == ObligationStatus::Satisfied)
    }

    /// Get all failed obligations
    pub fn failed(&self) -> impl Iterator<Item = (usize, &TrackedObligation)> {
        self.obligations
            .iter()
            .enumerate()
            .filter(|(_, o)| o.status == ObligationStatus::Failed)
    }

    /// Check if all obligations are satisfied or waived
    pub fn all_satisfied(&self) -> bool {
        self.obligations.iter().all(|o| {
            o.status == ObligationStatus::Satisfied || o.status == ObligationStatus::Waived
        })
    }

    /// Check if there are any failed obligations
    pub fn has_failures(&self) -> bool {
        self.obligations
            .iter()
            .any(|o| o.status == ObligationStatus::Failed)
    }

    /// Get count of obligations by status
    pub fn count_by_status(&self, status: ObligationStatus) -> usize {
        self.obligations
            .iter()
            .filter(|o| o.status == status)
            .count()
    }

    /// Clear all obligations
    pub fn clear(&mut self) {
        self.obligations.clear();
        self.named.clear();
    }

    /// Check obligations and return result (TASK-024)
    pub fn check_obligations(&self) -> ObligationCheckResult {
        let pending: Vec<_> = self.pending().map(|(id, _)| id).collect();
        let failed: Vec<_> = self.failed().map(|(id, _)| id).collect();

        if failed.is_empty() && pending.is_empty() {
            ObligationCheckResult::Success
        } else if !failed.is_empty() {
            ObligationCheckResult::Failed(failed)
        } else {
            ObligationCheckResult::Pending(pending)
        }
    }
}

/// Result of checking obligations
#[derive(Debug, Clone, PartialEq)]
pub enum ObligationCheckResult {
    /// All obligations satisfied
    Success,
    /// Some obligations still pending
    Pending(Vec<usize>),
    /// Some obligations failed
    Failed(Vec<usize>),
}

impl ObligationCheckResult {
    /// Check if all obligations are satisfied
    pub fn is_success(&self) -> bool {
        matches!(self, ObligationCheckResult::Success)
    }

    /// Check if there are pending obligations
    pub fn is_pending(&self) -> bool {
        matches!(self, ObligationCheckResult::Pending(_))
    }

    /// Check if there are failed obligations
    pub fn is_failed(&self) -> bool {
        matches!(self, ObligationCheckResult::Failed(_))
    }
}

/// Builder for creating obligation contexts
#[derive(Debug, Clone, Default)]
pub struct ObligationContextBuilder {
    obligations: Vec<ProofObligation>,
}

impl ObligationContextBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            obligations: Vec::new(),
        }
    }

    /// Add an obligation
    #[allow(clippy::should_implement_trait)]
    pub fn add(mut self, obligation: ProofObligation) -> Self {
        self.obligations.push(obligation);
        self
    }

    /// Add a check condition obligation
    pub fn check_condition(mut self, condition: impl Into<Box<str>>, at: EffectTime) -> Self {
        self.obligations
            .push(ProofObligation::check_condition(condition, at));
        self
    }

    /// Add a maintain invariant obligation
    pub fn maintain_invariant(mut self, invariant: impl Into<Box<str>>) -> Self {
        self.obligations
            .push(ProofObligation::maintain_invariant(invariant));
        self
    }

    /// Add a satisfy policy obligation
    pub fn satisfy_policy(mut self, policy: impl Into<Box<str>>) -> Self {
        self.obligations
            .push(ProofObligation::satisfy_policy(policy));
        self
    }

    /// Build the tracker
    pub fn build(self) -> ObligationTracker {
        let mut tracker = ObligationTracker::new();
        for obl in self.obligations {
            tracker.add(obl);
        }
        tracker
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proof_obligation_creation() {
        let obl = ProofObligation::check_condition("x > 0", EffectTime::Before);
        assert_eq!(obl.name(), "x > 0");

        let obl = ProofObligation::maintain_invariant("valid");
        assert_eq!(obl.name(), "valid");

        let obl = ProofObligation::satisfy_policy("access_control");
        assert_eq!(obl.name(), "access_control");

        let obl = ProofObligation::fulfill_role("admin");
        assert_eq!(obl.name(), "admin");

        let obl = ProofObligation::audit_action("delete");
        assert_eq!(obl.name(), "delete");

        let obl = ProofObligation::custom("special", Type::Bool);
        assert_eq!(obl.name(), "special");
    }

    #[test]
    fn test_effect_time_display() {
        assert_eq!(format!("{}", EffectTime::Before), "before");
        assert_eq!(format!("{}", EffectTime::After), "after");
        assert_eq!(format!("{}", EffectTime::During), "during");
        assert_eq!(format!("{}", EffectTime::Completion), "completion");
    }

    #[test]
    fn test_obligation_status_display() {
        assert_eq!(format!("{}", ObligationStatus::Pending), "pending");
        assert_eq!(format!("{}", ObligationStatus::Satisfied), "satisfied");
        assert_eq!(format!("{}", ObligationStatus::Failed), "failed");
        assert_eq!(format!("{}", ObligationStatus::Waived), "waived");
    }

    #[test]
    fn test_tracked_obligation_creation() {
        let obl = ProofObligation::check_condition("x > 0", EffectTime::Before);
        let tracked = TrackedObligation::new(obl.clone());

        assert_eq!(tracked.obligation, obl);
        assert_eq!(tracked.status, ObligationStatus::Pending);
        assert!(tracked.witness.is_none());
    }

    #[test]
    fn test_tracked_obligation_satisfy() {
        let obl = ProofObligation::check_condition("x > 0", EffectTime::Before);
        let mut tracked = TrackedObligation::new(obl);

        tracked.satisfy(Some(ProofWitness::Direct));

        assert!(tracked.is_satisfied());
        assert_eq!(tracked.witness, Some(ProofWitness::Direct));
    }

    #[test]
    fn test_tracked_obligation_fail() {
        let obl = ProofObligation::check_condition("x > 0", EffectTime::Before);
        let mut tracked = TrackedObligation::new(obl);

        tracked.fail();

        assert!(tracked.is_failed());
        assert!(!tracked.is_satisfied());
    }

    #[test]
    fn test_obligation_tracker_creation() {
        let tracker = ObligationTracker::new();
        assert!(tracker.all().is_empty());
        assert!(tracker.all_satisfied());
        assert!(!tracker.has_failures());
    }

    #[test]
    fn test_obligation_tracker_add() {
        let mut tracker = ObligationTracker::new();
        let id = tracker.add(ProofObligation::check_condition(
            "x > 0",
            EffectTime::Before,
        ));

        assert_eq!(id, 0);
        assert_eq!(tracker.all().len(), 1);
    }

    #[test]
    fn test_obligation_tracker_get() {
        let mut tracker = ObligationTracker::new();
        let obl = ProofObligation::check_condition("x > 0", EffectTime::Before);
        let id = tracker.add(obl.clone());

        let retrieved = tracker.get(id).unwrap();
        assert_eq!(retrieved.obligation.name(), "x > 0");
    }

    #[test]
    fn test_obligation_tracker_lookup() {
        let mut tracker = ObligationTracker::new();
        tracker.add(ProofObligation::check_condition(
            "x > 0",
            EffectTime::Before,
        ));

        let found = tracker.lookup("x > 0").unwrap();
        assert_eq!(found.obligation.name(), "x > 0");

        assert!(tracker.lookup("not_found").is_none());
    }

    #[test]
    fn test_obligation_tracker_satisfy() {
        let mut tracker = ObligationTracker::new();
        let id = tracker.add(ProofObligation::check_condition(
            "x > 0",
            EffectTime::Before,
        ));

        assert!(tracker.satisfy(id, Some(ProofWitness::Direct)));
        assert!(tracker.get(id).unwrap().is_satisfied());
    }

    #[test]
    fn test_obligation_tracker_satisfy_named() {
        let mut tracker = ObligationTracker::new();
        tracker.add(ProofObligation::check_condition(
            "x > 0",
            EffectTime::Before,
        ));

        assert!(tracker.satisfy_named("x > 0", Some(ProofWitness::Direct)));
        assert!(!tracker.satisfy_named("not_found", None));
    }

    #[test]
    fn test_obligation_tracker_pending() {
        let mut tracker = ObligationTracker::new();
        let id1 = tracker.add(ProofObligation::check_condition(
            "x > 0",
            EffectTime::Before,
        ));
        let id2 = tracker.add(ProofObligation::maintain_invariant("valid"));

        tracker.satisfy(id1, None);

        let pending: Vec<_> = tracker.pending().collect();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].0, id2);
    }

    #[test]
    fn test_obligation_tracker_satisfied() {
        let mut tracker = ObligationTracker::new();
        let id = tracker.add(ProofObligation::check_condition(
            "x > 0",
            EffectTime::Before,
        ));

        tracker.satisfy(id, None);

        let satisfied: Vec<_> = tracker.satisfied().collect();
        assert_eq!(satisfied.len(), 1);
        assert_eq!(satisfied[0].0, id);
    }

    #[test]
    fn test_obligation_tracker_failed() {
        let mut tracker = ObligationTracker::new();
        let id = tracker.add(ProofObligation::check_condition(
            "x > 0",
            EffectTime::Before,
        ));

        tracker.fail(id);

        let failed: Vec<_> = tracker.failed().collect();
        assert_eq!(failed.len(), 1);
    }

    #[test]
    fn test_obligation_tracker_all_satisfied() {
        let mut tracker = ObligationTracker::new();
        let id = tracker.add(ProofObligation::check_condition(
            "x > 0",
            EffectTime::Before,
        ));

        assert!(!tracker.all_satisfied());

        tracker.satisfy(id, None);
        assert!(tracker.all_satisfied());
    }

    #[test]
    fn test_obligation_tracker_has_failures() {
        let mut tracker = ObligationTracker::new();
        let id = tracker.add(ProofObligation::check_condition(
            "x > 0",
            EffectTime::Before,
        ));

        assert!(!tracker.has_failures());

        tracker.fail(id);
        assert!(tracker.has_failures());
    }

    #[test]
    fn test_obligation_tracker_count_by_status() {
        let mut tracker = ObligationTracker::new();
        let id1 = tracker.add(ProofObligation::check_condition(
            "x > 0",
            EffectTime::Before,
        ));
        let id2 = tracker.add(ProofObligation::maintain_invariant("valid"));
        let _id3 = tracker.add(ProofObligation::satisfy_policy("policy1"));

        tracker.satisfy(id1, None);
        tracker.fail(id2);

        assert_eq!(tracker.count_by_status(ObligationStatus::Satisfied), 1);
        assert_eq!(tracker.count_by_status(ObligationStatus::Failed), 1);
        assert_eq!(tracker.count_by_status(ObligationStatus::Pending), 1);
    }

    #[test]
    fn test_obligation_tracker_clear() {
        let mut tracker = ObligationTracker::new();
        tracker.add(ProofObligation::check_condition(
            "x > 0",
            EffectTime::Before,
        ));

        assert_eq!(tracker.all().len(), 1);

        tracker.clear();
        assert!(tracker.all().is_empty());
        assert!(tracker.lookup("x > 0").is_none());
    }

    #[test]
    fn test_check_obligations_success() {
        let mut tracker = ObligationTracker::new();
        let id = tracker.add(ProofObligation::check_condition(
            "x > 0",
            EffectTime::Before,
        ));
        tracker.satisfy(id, None);

        assert_eq!(tracker.check_obligations(), ObligationCheckResult::Success);
    }

    #[test]
    fn test_check_obligations_pending() {
        let mut tracker = ObligationTracker::new();
        let id = tracker.add(ProofObligation::check_condition(
            "x > 0",
            EffectTime::Before,
        ));

        assert_eq!(
            tracker.check_obligations(),
            ObligationCheckResult::Pending(vec![id])
        );
    }

    #[test]
    fn test_check_obligations_failed() {
        let mut tracker = ObligationTracker::new();
        let id = tracker.add(ProofObligation::check_condition(
            "x > 0",
            EffectTime::Before,
        ));
        tracker.fail(id);

        assert_eq!(
            tracker.check_obligations(),
            ObligationCheckResult::Failed(vec![id])
        );
    }

    #[test]
    fn test_obligation_check_result_is_success() {
        assert!(ObligationCheckResult::Success.is_success());
        assert!(!ObligationCheckResult::Pending(vec![]).is_success());
        assert!(!ObligationCheckResult::Failed(vec![]).is_success());
    }

    #[test]
    fn test_obligation_check_result_is_pending() {
        assert!(!ObligationCheckResult::Success.is_pending());
        assert!(ObligationCheckResult::Pending(vec![]).is_pending());
        assert!(!ObligationCheckResult::Failed(vec![]).is_pending());
    }

    #[test]
    fn test_obligation_check_result_is_failed() {
        assert!(!ObligationCheckResult::Success.is_failed());
        assert!(!ObligationCheckResult::Pending(vec![]).is_failed());
        assert!(ObligationCheckResult::Failed(vec![]).is_failed());
    }

    #[test]
    fn test_obligation_context_builder() {
        let tracker = ObligationContextBuilder::new()
            .check_condition("x > 0", EffectTime::Before)
            .maintain_invariant("valid")
            .satisfy_policy("policy1")
            .build();

        assert_eq!(tracker.all().len(), 3);
    }

    #[test]
    fn test_witness_variants() {
        let w1 = ProofWitness::Direct;
        assert_eq!(w1, ProofWitness::Direct);

        let w2 = ProofWitness::Derivation(vec!["step1".into(), "step2".into()]);
        assert!(matches!(w2, ProofWitness::Derivation(_)));

        let w3 = ProofWitness::External("verifier".into());
        assert!(matches!(w3, ProofWitness::External(_)));

        let w4 = ProofWitness::Assumption("assume".into());
        assert!(matches!(w4, ProofWitness::Assumption(_)));
    }

    #[test]
    fn test_tracked_obligation_waive() {
        let obl = ProofObligation::check_condition("x > 0", EffectTime::Before);
        let mut tracked = TrackedObligation::new(obl);

        tracked.waive();
        assert_eq!(tracked.status, ObligationStatus::Waived);
    }

    #[test]
    fn test_all_satisfied_with_waived() {
        let mut tracker = ObligationTracker::new();
        let id = tracker.add(ProofObligation::check_condition(
            "x > 0",
            EffectTime::Before,
        ));
        tracker.waive(id);

        assert!(tracker.all_satisfied());
    }
}
