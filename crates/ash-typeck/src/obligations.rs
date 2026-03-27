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

// ============================================================================
// Linear Obligation Tracking for Workflow Contracts (TASK-227)
// ============================================================================

use crate::solver::TypeError;
use ash_core::workflow_contract::ObligationSet;
use ash_parser::surface::{CheckTarget, Workflow};

/// Variable bindings type
pub type VarBindings = HashMap<String, Type>;

/// Extended type check context with linear obligation tracking
#[derive(Debug, Clone, Default)]
pub struct LinearObligationContext {
    /// Variable type bindings
    pub var_types: VarBindings,
    /// Linear obligation set (must be discharged before workflow completes)
    pub obligations: ObligationSet,
}

// ============================================================================
// Obligation Collector for Workflow AST (TASK-275)
// ============================================================================

/// Collector that walks the workflow AST to collect and verify obligations
///
/// This implements linear obligation tracking where:
/// - `oblige obligation_name` introduces a new obligation
/// - `check obligation_name` consumes/discharge the obligation
/// - Obligations must be discharged exactly once
/// - All obligations must be discharged before workflow completion
#[derive(Debug, Clone, Default)]
pub struct ObligationCollector;

impl ObligationCollector {
    /// Create a new obligation collector
    pub fn new() -> Self {
        Self
    }

    /// Collect and verify obligations from a workflow
    ///
    /// Walks the workflow AST, tracking obligations and verifying that:
    /// 1. All obligations are properly discharged
    /// 2. No obligation is discharged twice
    /// 3. No unknown obligation is discharged
    ///
    /// # Arguments
    /// * `workflow` - The workflow AST to analyze
    /// * `ctx` - The obligation context to update
    ///
    /// # Returns
    /// * `Ok(())` if all obligations are satisfied
    /// * `Err(TypeError)` if there are unsatisfied or invalid obligations
    pub fn collect(
        &mut self,
        workflow: &Workflow,
        ctx: &mut LinearObligationContext,
    ) -> Result<(), TypeError> {
        self.collect_from_workflow(workflow, ctx)
    }

    fn collect_from_workflow(
        &mut self,
        workflow: &Workflow,
        ctx: &mut LinearObligationContext,
    ) -> Result<(), TypeError> {
        match workflow {
            // OBLIGE: Introduce a new obligation
            Workflow::Oblige {
                obligation,
                span: _,
            } => {
                ctx.obligations
                    .insert(obligation.to_string())
                    .map_err(TypeError::from)?;
            }

            // CHECK: Verify an obligation exists and discharge it
            Workflow::Check {
                target,
                continuation,
                span,
            } => {
                if let CheckTarget::Obligation(obl_ref) = target {
                    // Try to discharge the obligation
                    // Use the role field as the obligation identifier
                    let obl_name = obl_ref.role.to_string();
                    ctx.obligations.remove(&obl_name).map_err(|e| match e {
                        ash_core::workflow_contract::ObligationError::Unknown(_) => {
                            TypeError::UnknownObligation {
                                name: obl_name,
                                span: *span,
                            }
                        }
                        _ => TypeError::from(e),
                    })?;
                }

                // Continue with the rest of the workflow
                if let Some(cont) = continuation {
                    self.collect_from_workflow(cont, ctx)?;
                }
            }

            // IF: Both branches must discharge obligations for them to be considered satisfied
            // Per SPEC-003 Section 4.5: Ω_out = Ω_then ∩ Ω_else (where Ω is discharged set)
            // For remaining-obligation tracking: union of remaining obligations
            // If an obligation remains in ANY branch, it wasn't discharged on all paths
            Workflow::If {
                condition: _,
                then_branch,
                else_branch,
                span: _,
            } => {
                // Save parent context before branching
                let parent_ctx = ctx.clone();

                let mut then_ctx = parent_ctx.branch();
                self.collect_from_workflow(then_branch, &mut then_ctx)?;

                if let Some(else_branch) = else_branch {
                    let mut else_ctx = parent_ctx.branch();
                    self.collect_from_workflow(else_branch, &mut else_ctx)?;
                    // Union: obligation remains if pending in EITHER branch
                    ctx.obligations = then_ctx.obligations.union(&else_ctx.obligations);
                } else {
                    // No else branch: obligation must be discharged in then branch
                    ctx.obligations = then_ctx.obligations;
                }
            }

            // PAR: Union of remaining obligations from all branches
            // Per SPEC-003 Section 4.5: discharged = ∩ discharged_i
            // For remaining-obligation tracking: union of remaining obligations
            Workflow::Par { branches, span: _ } => {
                let parent_ctx = ctx.clone();

                let mut branch_contexts = Vec::new();
                for branch in branches {
                    let mut branch_ctx = parent_ctx.branch();
                    self.collect_from_workflow(branch, &mut branch_ctx)?;
                    branch_contexts.push(branch_ctx);
                }

                // Union: obligation remains if pending in ANY branch
                if let Some(first) = branch_contexts.first() {
                    ctx.obligations = first.obligations.clone();
                    for branch_ctx in &branch_contexts[1..] {
                        ctx.obligations = ctx.obligations.union(&branch_ctx.obligations);
                    }
                }
            }

            // MAYBE: Try primary, fallback on failure
            // Per SPEC-003 Section 4.6: Obligations must be discharged in both branches
            // For remaining-obligation tracking: union of remaining obligations
            Workflow::Maybe {
                primary,
                fallback,
                span: _,
            } => {
                let parent_ctx = ctx.clone();

                let mut primary_ctx = parent_ctx.branch();
                self.collect_from_workflow(primary, &mut primary_ctx)?;

                let mut fallback_ctx = parent_ctx.branch();
                self.collect_from_workflow(fallback, &mut fallback_ctx)?;

                // Union: obligation remains if pending in EITHER branch
                ctx.obligations = primary_ctx.obligations.union(&fallback_ctx.obligations);
            }

            // MUST: Ensure workflow succeeds
            // Same as regular body
            Workflow::Must { body, span: _ } => {
                self.collect_from_workflow(body, ctx)?;
            }

            // WITH: Scoped capability
            Workflow::With {
                capability: _,
                body,
                span: _,
            } => {
                self.collect_from_workflow(body, ctx)?;
            }

            // FOR: Loop body
            Workflow::For {
                pattern: _,
                collection: _,
                body,
                span: _,
            } => {
                self.collect_from_workflow(body, ctx)?;
            }

            // LET: Binding with continuation
            Workflow::Let {
                pattern: _,
                expr: _,
                continuation,
                span: _,
            } => {
                if let Some(cont) = continuation {
                    self.collect_from_workflow(cont, ctx)?;
                }
            }

            // OBSERVE: Capability observation with continuation
            Workflow::Observe {
                capability: _,
                binding: _,
                continuation,
                span: _,
            } => {
                if let Some(cont) = continuation {
                    self.collect_from_workflow(cont, ctx)?;
                }
            }

            // ORIENT: Expression evaluation with continuation
            Workflow::Orient {
                expr: _,
                binding: _,
                continuation,
                span: _,
            } => {
                if let Some(cont) = continuation {
                    self.collect_from_workflow(cont, ctx)?;
                }
            }

            // PROPOSE: Action proposal with continuation
            Workflow::Propose {
                action: _,
                binding: _,
                continuation,
                span: _,
            } => {
                if let Some(cont) = continuation {
                    self.collect_from_workflow(cont, ctx)?;
                }
            }

            // DECIDE: Conditional with branches
            // Per SPEC-003 Section 4.3: Same semantics as IF
            Workflow::Decide {
                expr: _,
                policy: _,
                then_branch,
                else_branch,
                span: _,
            } => {
                let parent_ctx = ctx.clone();

                let mut then_ctx = parent_ctx.branch();
                self.collect_from_workflow(then_branch, &mut then_ctx)?;

                if let Some(else_branch) = else_branch {
                    let mut else_ctx = parent_ctx.branch();
                    self.collect_from_workflow(else_branch, &mut else_ctx)?;
                    // Union: obligation remains if pending in EITHER branch
                    ctx.obligations = then_ctx.obligations.union(&else_ctx.obligations);
                } else {
                    ctx.obligations = then_ctx.obligations;
                }
            }

            // ACT: Action (terminal - no continuation field)
            Workflow::Act {
                action: _,
                guard: _,
                span: _,
            } => {
                // Terminal action - no obligations to track
            }

            // SET: Set value with optional continuation
            Workflow::Set {
                capability: _,
                channel: _,
                value: _,
                continuation,
                span: _,
            } => {
                if let Some(cont) = continuation {
                    self.collect_from_workflow(cont, ctx)?;
                }
            }

            // SEND: Send value with optional continuation
            Workflow::Send {
                capability: _,
                channel: _,
                value: _,
                continuation,
                span: _,
            } => {
                if let Some(cont) = continuation {
                    self.collect_from_workflow(cont, ctx)?;
                }
            }

            // SEQ: Sequential composition
            Workflow::Seq {
                first,
                second,
                span: _,
            } => {
                self.collect_from_workflow(first, ctx)?;
                self.collect_from_workflow(second, ctx)?;
            }

            // RECEIVE: Pattern matching on messages
            // Per SPEC-003 Section 4.1: All arms must discharge obligations
            Workflow::Receive {
                mode: _,
                arms,
                is_control: _,
                span: _,
            } => {
                let parent_ctx = ctx.clone();

                let mut arm_contexts = Vec::new();
                for arm in arms {
                    let mut arm_ctx = parent_ctx.branch();
                    self.collect_from_workflow(&arm.body, &mut arm_ctx)?;
                    arm_contexts.push(arm_ctx);
                }

                // Union: obligation remains if pending in ANY arm
                if let Some(first) = arm_contexts.first() {
                    ctx.obligations = first.obligations.clone();
                    for arm_ctx in &arm_contexts[1..] {
                        ctx.obligations = ctx.obligations.union(&arm_ctx.obligations);
                    }
                }
            }

            // YIELD: Role delegation with resumption
            // Same semantics as RECEIVE: all arms must discharge obligations
            Workflow::Yield {
                role: _,
                expr: _,
                resume_var: _,
                resume_type: _,
                arms,
                span: _,
            } => {
                let parent_ctx = ctx.clone();

                let mut arm_contexts = Vec::new();
                for arm in arms {
                    let mut arm_ctx = parent_ctx.branch();
                    self.collect_from_workflow(&arm.body, &mut arm_ctx)?;
                    arm_contexts.push(arm_ctx);
                }

                // Union: obligation remains if pending in ANY arm
                if let Some(first) = arm_contexts.first() {
                    ctx.obligations = first.obligations.clone();
                    for arm_ctx in &arm_contexts[1..] {
                        ctx.obligations = ctx.obligations.union(&arm_ctx.obligations);
                    }
                }
            }

            // RET: Return expression
            Workflow::Ret { expr: _, span: _ } => {}

            // RESUME: Resume from yield with a value
            Workflow::Resume {
                expr: _,
                ty: _,
                span: _,
            } => {}

            // DONE: Terminal workflow
            Workflow::Done { span: _ } => {}
        }

        Ok(())
    }

    /// Finalize obligation checking and report any unsatisfied obligations
    ///
    /// # Arguments
    /// * `ctx` - The obligation context to finalize
    ///
    /// # Returns
    /// * `Ok(())` if all obligations are satisfied
    /// * `Err(TypeError::UnsatisfiedObligations)` if there are pending obligations
    pub fn finalize(&self, ctx: &LinearObligationContext) -> Result<(), TypeError> {
        if ctx.is_clean() {
            Ok(())
        } else {
            let remaining: Vec<String> = ctx.obligations.remaining().into_iter().cloned().collect();
            Err(TypeError::UnsatisfiedObligations {
                obligations: remaining,
            })
        }
    }
}

impl LinearObligationContext {
    /// Create a new context with empty obligations
    #[must_use]
    pub fn new() -> Self {
        Self {
            var_types: VarBindings::new(),
            obligations: ObligationSet::new(),
        }
    }

    /// Create a context with existing variable bindings
    #[must_use]
    pub fn with_bindings(var_types: VarBindings) -> Self {
        Self {
            var_types,
            obligations: ObligationSet::new(),
        }
    }

    /// Branch the context for if/else and parallel composition
    #[must_use]
    pub fn branch(&self) -> Self {
        Self {
            var_types: self.var_types.clone(),
            obligations: self.obligations.clone(),
        }
    }

    /// Merge a branched context back into self
    ///
    /// Per SPEC-003 Section 4.5: An obligation must be discharged on ALL execution paths.
    /// Since we track REMAINING obligations, we use union semantics:
    /// If an obligation remains pending in ANY branch, it remains pending after merge.
    ///
    /// # Arguments
    /// * `branch` - The branch context to merge
    /// * `_parent` - The parent context before branching (used to determine what was discharged)
    pub fn merge(&mut self, branch: Self, _parent: &Self) {
        // Union: obligation survives if pending in EITHER context
        self.obligations = self.obligations.union(&branch.obligations);
    }

    /// Check if all obligations have been discharged
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.obligations.is_empty()
    }
}

#[cfg(test)]
mod linear_tests {
    use super::*;
    use ash_core::workflow_contract::ObligationError;

    #[test]
    fn test_linear_obligation_lifecycle() {
        let mut ctx = LinearObligationContext::new();

        // Create obligation
        ctx.obligations.insert("audit".to_string()).unwrap();
        assert!(ctx.obligations.contains("audit"));
        assert!(!ctx.is_clean());

        // Check (consume) obligation
        ctx.obligations.remove("audit").unwrap();
        assert!(!ctx.obligations.contains("audit"));
        assert!(ctx.is_clean());
    }

    #[test]
    fn test_double_insert_fails() {
        let mut ctx = LinearObligationContext::new();

        ctx.obligations.insert("duplicate".to_string()).unwrap();
        let result = ctx.obligations.insert("duplicate".to_string());

        assert!(matches!(result, Err(ObligationError::Duplicate(_))));
    }

    #[test]
    fn test_double_remove_fails() {
        let mut ctx = LinearObligationContext::new();

        ctx.obligations.insert("once".to_string()).unwrap();
        ctx.obligations.remove("once").unwrap();

        let result = ctx.obligations.remove("once");
        assert!(matches!(result, Err(ObligationError::Unknown(_))));
    }

    #[test]
    fn test_branch_and_merge_both_discharge() {
        // Start with a context containing an obligation
        let mut base_ctx = LinearObligationContext::new();
        base_ctx.obligations.insert("o1".to_string()).unwrap();

        // Branch from base
        let mut then_ctx = base_ctx.branch();
        let mut else_ctx = base_ctx.branch();

        // Both branches discharge
        then_ctx.obligations.remove("o1").unwrap();
        else_ctx.obligations.remove("o1").unwrap();

        // Merge: union of remaining obligations (empty ∪ empty = empty)
        let merged = then_ctx.obligations.union(&else_ctx.obligations);
        assert!(merged.is_empty());
    }

    #[test]
    fn test_branch_and_merge_partial_discharge() {
        // This test verifies that if only one branch discharges an obligation,
        // the merged result still has the obligation
        let mut then_ctx = LinearObligationContext::new();
        let mut else_ctx = LinearObligationContext::new();

        // Both branches start with the obligation
        then_ctx.obligations.insert("o1".to_string()).unwrap();
        else_ctx.obligations.insert("o1".to_string()).unwrap();

        // Only then branch discharges
        then_ctx.obligations.remove("o1").unwrap();
        // else_ctx does NOT discharge - still has "o1"

        // Union: then_ctx has {}, else_ctx has {o1}
        // union = {} ∪ {o1} = {o1} (obligation discharged in one branch only)
        then_ctx.obligations = then_ctx.obligations.union(&else_ctx.obligations);
        // The obligation is still pending because not all paths discharged it
        assert!(then_ctx.obligations.contains("o1"));
    }

    #[test]
    fn test_context_preserves_var_types() {
        let mut bindings = VarBindings::new();
        bindings.insert("x".to_string(), Type::Int);

        let ctx = LinearObligationContext::with_bindings(bindings);
        let branched = ctx.branch();

        assert_eq!(branched.var_types.get("x"), Some(&Type::Int));
    }
}

// ============================================================================
// Property Tests for Obligation Branch/Merge Semantics (TASK-301)
// ============================================================================

#[cfg(test)]
mod prop_tests {
    use super::*;
    use proptest::prelude::*;

    /// Generate a valid obligation name (alphanumeric with underscores)
    fn arb_obligation_name() -> impl Strategy<Value = String> {
        "[a-z_][a-z0-9_]*"
    }

    /// Generate a set of obligation names
    fn arb_obligation_set() -> impl Strategy<Value = Vec<String>> {
        prop::collection::vec(arb_obligation_name(), 0..10)
    }

    proptest! {
        /// Property: If both branches discharge all obligations, merge is clean
        #[test]
        fn prop_both_branches_discharge_all_clean(
            obligations in arb_obligation_set()
        ) {
            // Start with a base context containing obligations
            let mut base_ctx = LinearObligationContext::new();
            for obl in &obligations {
                let _ = base_ctx.obligations.insert(obl.clone());
            }

            // Branch and discharge all in both
            let mut then_ctx = base_ctx.branch();
            let mut else_ctx = base_ctx.branch();

            for obl in &obligations {
                let _ = then_ctx.obligations.remove(obl);
                let _ = else_ctx.obligations.remove(obl);
            }

            // Merge branches with union (not parent with branches)
            let merged = then_ctx.obligations.union(&else_ctx.obligations);

            prop_assert!(merged.is_empty(),
                "Both branches discharged all obligations, merge should be clean");
        }

        /// Property: If any branch leaves an obligation pending, it remains after merge
        #[test]
        fn prop_partial_discharge_preserves_obligation(
            obligation in arb_obligation_name()
        ) {
            // Start with a base context containing the obligation
            let mut base_ctx = LinearObligationContext::new();
            let _ = base_ctx.obligations.insert(obligation.clone());

            // Branch: then discharges, else does not
            let mut then_ctx = base_ctx.branch();
            let else_ctx = base_ctx.branch();

            let _ = then_ctx.obligations.remove(&obligation);
            // else_ctx does NOT discharge

            // Merge branches (not parent with branches)
            let merged = then_ctx.obligations.union(&else_ctx.obligations);

            prop_assert!(merged.contains(&obligation),
                "Obligation should remain when not discharged in all branches");
        }

        /// Property: Union of remaining obligations is idempotent for clean branches
        #[test]
        fn prop_union_idempotent_for_clean(
            obligations in arb_obligation_set()
        ) {
            let mut ctx1 = LinearObligationContext::new();
            let mut ctx2 = LinearObligationContext::new();

            for obl in &obligations {
                let _ = ctx1.obligations.insert(obl.clone());
                let _ = ctx2.obligations.insert(obl.clone());
            }

            // Both discharge all
            for obl in &obligations {
                let _ = ctx1.obligations.remove(obl);
                let _ = ctx2.obligations.remove(obl);
            }

            let union1 = ctx1.obligations.union(&ctx2.obligations);

            prop_assert!(union1.is_empty(),
                "Union of clean branches should be empty");

            // Test idempotence: union with empty set should be empty
            let union2 = union1.clone().union(&ctx1.obligations);
            prop_assert!(union2.is_empty(),
                "Union should be idempotent when both sets are empty");
            prop_assert_eq!(union1, union2, "Union should be equal");
        }

        /// Property: Obligations created inside only one branch don't affect parent
        #[test]
        fn prop_branch_local_obligations(
            parent_obl in arb_obligation_name(),
            branch_obl in arb_obligation_name()
        ) {
            prop_assume!(parent_obl != branch_obl);

            let mut ctx = LinearObligationContext::new();
            let _ = ctx.obligations.insert(parent_obl.clone());

            // Branch creates a new obligation
            let mut branch_ctx = ctx.branch();
            let _ = branch_ctx.obligations.insert(branch_obl.clone());

            // Parent should not see branch-local obligation
            prop_assert!(!ctx.obligations.contains(&branch_obl),
                "Parent should not see branch-local obligations");
            prop_assert!(ctx.obligations.contains(&parent_obl),
                "Parent should retain its obligations");
        }

        /// Property: Nested branches maintain correct discharge tracking
        #[test]
        fn prop_nested_branch_discharge(
            o1 in arb_obligation_name(),
            o2 in arb_obligation_name()
        ) {
            prop_assume!(o1 != o2);

            // Base context with both obligations
            let mut base_ctx = LinearObligationContext::new();
            let _ = base_ctx.obligations.insert(o1.clone());
            let _ = base_ctx.obligations.insert(o2.clone());

            // Outer then branch - discharges o1 only
            let mut then_ctx = base_ctx.branch();
            let _ = then_ctx.obligations.remove(&o1);

            // Outer else branch - has nested if
            let else_base = base_ctx.branch();
            // Nested branches
            let mut nested_then = else_base.branch();
            let mut nested_else = else_base.branch();

            // Nested then discharges both
            let _ = nested_then.obligations.remove(&o1);
            let _ = nested_then.obligations.remove(&o2);

            // Nested else discharges only o2
            let _ = nested_else.obligations.remove(&o2);

            // Merge nested branches: union of remaining obligations
            // nested_then: {} (both discharged)
            // nested_else: {o1} (o2 discharged, o1 remains)
            // union = {o1}
            let else_merged = nested_then.obligations.union(&nested_else.obligations);

            // o1 should remain (not discharged in nested_else)
            prop_assert!(else_merged.contains(&o1),
                "o1 should remain when not discharged in all nested branches");
            // o2 discharged in both nested branches
            prop_assert!(!else_merged.contains(&o2),
                "o2 should be discharged when removed from all nested branches");

            // Merge outer branches
            // then_ctx: {o2} (o1 discharged, o2 remains)
            // else_merged: {o1} (from nested)
            // union = {o1, o2}
            let final_merged = then_ctx.obligations.union(&else_merged);

            // Both should remain (each not discharged in one branch)
            prop_assert!(final_merged.contains(&o1), "o1 not discharged in else branch");
            prop_assert!(final_merged.contains(&o2), "o2 not discharged in then branch");
        }

        /// Property: Empty branches don't create obligations
        #[test]
        fn prop_empty_branch_no_obligations(
            obligation in arb_obligation_name()
        ) {
            // Base context with obligation
            let mut base_ctx = LinearObligationContext::new();
            let _ = base_ctx.obligations.insert(obligation.clone());

            // Branch with no discharge
            let branch_ctx = base_ctx.branch();

            // Merge base with branch (both have the obligation)
            let merged = base_ctx.obligations.union(&branch_ctx.obligations);

            prop_assert!(merged.contains(&obligation),
                "Obligation should remain when branch doesn't discharge it");
        }

        /// Property: Multiple parallel branches with partial discharge
        #[test]
        fn prop_parallel_branches_partial_discharge(
            o1 in arb_obligation_name(),
            o2 in arb_obligation_name()
        ) {
            prop_assume!(o1 != o2);

            let mut ctx = LinearObligationContext::new();
            let _ = ctx.obligations.insert(o1.clone());
            let _ = ctx.obligations.insert(o2.clone());

            // Branch 1: discharges o1 only
            let mut branch1 = ctx.branch();
            let _ = branch1.obligations.remove(&o1);

            // Branch 2: discharges o2 only
            let mut branch2 = ctx.branch();
            let _ = branch2.obligations.remove(&o2);

            // Branch 3: discharges both
            let mut branch3 = ctx.branch();
            let _ = branch3.obligations.remove(&o1);
            let _ = branch3.obligations.remove(&o2);

            // Merge all with union
            let merged = branch1.obligations
                .union(&branch2.obligations)
                .union(&branch3.obligations);

            // Both should remain (not discharged in ALL branches)
            prop_assert!(merged.contains(&o1), "o1 not discharged in branch2");
            prop_assert!(merged.contains(&o2), "o2 not discharged in branch1");
        }
    }
}
