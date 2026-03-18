//! Obligation checking with capabilities (TASK-109)
//!
//! This module provides capability-based obligation checking for workflows,
//! ensuring that workflows have the required capabilities before they can
//! satisfy obligations.

use ash_core::Effect;
use thiserror::Error;

/// A name/identifier for capabilities and channels.
pub type Name = Box<str>;

/// Required capabilities for an obligation
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RequiredCapabilities {
    /// (capability, channel) pairs for observe operations
    pub observes: Vec<(Name, Name)>,
    /// (capability, channel) pairs for receive operations
    pub receives: Vec<(Name, Name)>,
    /// (capability, channel) pairs for set operations
    pub sets: Vec<(Name, Name)>,
    /// (capability, channel) pairs for send operations
    pub sends: Vec<(Name, Name)>,
}

impl RequiredCapabilities {
    /// Create a new empty set of required capabilities
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an observe capability requirement
    pub fn require_observe(mut self, cap: impl Into<Name>, channel: impl Into<Name>) -> Self {
        self.observes.push((cap.into(), channel.into()));
        self
    }

    /// Add a receive capability requirement
    pub fn require_receive(mut self, cap: impl Into<Name>, channel: impl Into<Name>) -> Self {
        self.receives.push((cap.into(), channel.into()));
        self
    }

    /// Add a set capability requirement
    pub fn require_set(mut self, cap: impl Into<Name>, channel: impl Into<Name>) -> Self {
        self.sets.push((cap.into(), channel.into()));
        self
    }

    /// Add a send capability requirement
    pub fn require_send(mut self, cap: impl Into<Name>, channel: impl Into<Name>) -> Self {
        self.sends.push((cap.into(), channel.into()));
        self
    }
}

/// Obligation definition with required capabilities and minimum effect
#[derive(Debug, Clone, PartialEq)]
pub struct Obligation {
    /// Name of the obligation
    pub name: Name,
    /// Required capabilities for this obligation
    pub required: RequiredCapabilities,
    /// Minimum effect level required
    pub min_effect: Effect,
}

impl Obligation {
    /// Create a new obligation with the given name
    pub fn new(name: impl Into<Name>) -> Self {
        Self {
            name: name.into(),
            required: RequiredCapabilities::new(),
            min_effect: Effect::Epistemic,
        }
    }

    /// Set the required capabilities
    pub fn with_required(mut self, required: RequiredCapabilities) -> Self {
        self.required = required;
        self
    }

    /// Set the minimum required effect
    pub fn with_min_effect(mut self, effect: Effect) -> Self {
        self.min_effect = effect;
        self
    }
}

/// Workflow capabilities (declared)
#[derive(Debug, Clone, Default, PartialEq)]
pub struct WorkflowCapabilities {
    /// (capability, channel) pairs for observe operations
    pub observes: Vec<(Name, Name)>,
    /// (capability, channel) pairs for receive operations
    pub receives: Vec<(Name, Name)>,
    /// (capability, channel) pairs for set operations
    pub sets: Vec<(Name, Name)>,
    /// (capability, channel) pairs for send operations
    pub sends: Vec<(Name, Name)>,
}

impl WorkflowCapabilities {
    /// Create a new empty set of workflow capabilities
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if the workflow can observe a capability on a channel
    pub fn can_observe(&self, cap: &Name, channel: &Name) -> bool {
        self.observes
            .iter()
            .any(|(c, ch)| c == cap && ch == channel)
    }

    /// Check if the workflow can receive from a capability on a channel
    pub fn can_receive(&self, cap: &Name, channel: &Name) -> bool {
        self.receives
            .iter()
            .any(|(c, ch)| c == cap && ch == channel)
    }

    /// Check if the workflow can set a capability on a channel
    pub fn can_set(&self, cap: &Name, channel: &Name) -> bool {
        self.sets.iter().any(|(c, ch)| c == cap && ch == channel)
    }

    /// Check if the workflow can send to a capability on a channel
    pub fn can_send(&self, cap: &Name, channel: &Name) -> bool {
        self.sends.iter().any(|(c, ch)| c == cap && ch == channel)
    }
}

/// Obligation check error
#[derive(Debug, Error, Clone, PartialEq)]
pub enum ObligationCheckError {
    /// Missing capability for an operation
    #[error(
        "missing capability for obligation '{obligation}': operation '{operation}' on '{capability}' not available"
    )]
    MissingCapability {
        /// Name of the obligation
        obligation: String,
        /// Operation type (observe, receive, set, send)
        operation: String,
        /// Capability:channel that is missing
        capability: String,
    },

    /// Insufficient effect level
    #[error(
        "insufficient effect for obligation '{obligation}': required {required:?}, actual {actual:?}"
    )]
    InsufficientEffect {
        /// Name of the obligation
        obligation: String,
        /// Required effect level
        required: Effect,
        /// Actual effect level
        actual: Effect,
    },
}

/// Obligation checker for verifying workflow capabilities
pub struct ObligationChecker;

impl ObligationChecker {
    /// Create a new obligation checker
    pub fn new() -> Self {
        Self
    }

    /// Verify that a workflow satisfies an obligation
    ///
    /// Checks that:
    /// 1. All required observe capabilities are available
    /// 2. All required receive capabilities are available
    /// 3. All required set capabilities are available
    /// 4. All required send capabilities are available
    /// 5. The workflow's effect level meets the minimum requirement
    ///
    /// # Arguments
    /// * `obligation` - The obligation to verify against
    /// * `capabilities` - The workflow's declared capabilities
    /// * `workflow_effect` - The computed effect of the workflow
    ///
    /// # Returns
    /// * `Ok(())` if all requirements are satisfied
    /// * `Err(ObligationCheckError)` if any requirement is not met
    pub fn verify(
        &self,
        obligation: &Obligation,
        capabilities: &WorkflowCapabilities,
        workflow_effect: Effect,
    ) -> Result<(), ObligationCheckError> {
        // Check input capabilities
        for (cap, channel) in &obligation.required.observes {
            if !capabilities.can_observe(cap, channel) {
                return Err(ObligationCheckError::MissingCapability {
                    obligation: obligation.name.to_string(),
                    operation: "observe".into(),
                    capability: format!("{}:{}", cap, channel),
                });
            }
        }

        for (cap, channel) in &obligation.required.receives {
            if !capabilities.can_receive(cap, channel) {
                return Err(ObligationCheckError::MissingCapability {
                    obligation: obligation.name.to_string(),
                    operation: "receive".into(),
                    capability: format!("{}:{}", cap, channel),
                });
            }
        }

        // Check output capabilities
        for (cap, channel) in &obligation.required.sets {
            if !capabilities.can_set(cap, channel) {
                return Err(ObligationCheckError::MissingCapability {
                    obligation: obligation.name.to_string(),
                    operation: "set".into(),
                    capability: format!("{}:{}", cap, channel),
                });
            }
        }

        for (cap, channel) in &obligation.required.sends {
            if !capabilities.can_send(cap, channel) {
                return Err(ObligationCheckError::MissingCapability {
                    obligation: obligation.name.to_string(),
                    operation: "send".into(),
                    capability: format!("{}:{}", cap, channel),
                });
            }
        }

        // Check effect level (using Effect's partial ordering)
        if workflow_effect < obligation.min_effect {
            return Err(ObligationCheckError::InsufficientEffect {
                obligation: obligation.name.to_string(),
                required: obligation.min_effect,
                actual: workflow_effect,
            });
        }

        Ok(())
    }
}

impl Default for ObligationChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sufficient_capabilities() {
        let checker = ObligationChecker::new();

        let obligation = Obligation::new("control_temp").with_required(
            RequiredCapabilities::new()
                .require_observe("sensor", "temp")
                .require_set("hvac", "target"),
        );

        let capabilities = WorkflowCapabilities {
            observes: vec![("sensor".into(), "temp".into())],
            receives: vec![],
            sets: vec![("hvac".into(), "target".into())],
            sends: vec![],
        };

        let result = checker.verify(&obligation, &capabilities, Effect::Operational);
        assert!(result.is_ok());
    }

    #[test]
    fn test_missing_observe_capability() {
        let checker = ObligationChecker::new();

        let obligation = Obligation::new("check_temp")
            .with_required(RequiredCapabilities::new().require_observe("sensor", "temp"));

        let capabilities = WorkflowCapabilities::new(); // Empty

        let result = checker.verify(&obligation, &capabilities, Effect::Epistemic);
        assert!(result.is_err());
        assert!(
            matches!(result.unwrap_err(), ObligationCheckError::MissingCapability { operation, .. } if operation == "observe")
        );
    }

    #[test]
    fn test_missing_set_capability() {
        let checker = ObligationChecker::new();

        let obligation = Obligation::new("control")
            .with_required(RequiredCapabilities::new().require_set("hvac", "target"));

        let capabilities = WorkflowCapabilities::new();

        let result = checker.verify(&obligation, &capabilities, Effect::Epistemic);
        assert!(result.is_err());
    }

    #[test]
    fn test_insufficient_effect() {
        let checker = ObligationChecker::new();

        let obligation = Obligation::new("control").with_min_effect(Effect::Operational);

        let capabilities = WorkflowCapabilities::new();

        // Workflow has Epistemic effect but Operational required
        let result = checker.verify(&obligation, &capabilities, Effect::Epistemic);
        assert!(result.is_err());
        assert!(
            matches!(result.unwrap_err(), ObligationCheckError::InsufficientEffect { required, actual, .. }
            if required == Effect::Operational && actual == Effect::Epistemic)
        );
    }

    #[test]
    fn test_capabilities_check_methods() {
        let caps = WorkflowCapabilities {
            observes: vec![("s1".into(), "c1".into())],
            receives: vec![("s2".into(), "c2".into())],
            sets: vec![("s3".into(), "c3".into())],
            sends: vec![("s4".into(), "c4".into())],
        };

        assert!(caps.can_observe(&"s1".into(), &"c1".into()));
        assert!(!caps.can_observe(&"s1".into(), &"wrong".into()));

        assert!(caps.can_receive(&"s2".into(), &"c2".into()));
        assert!(caps.can_set(&"s3".into(), &"c3".into()));
        assert!(caps.can_send(&"s4".into(), &"c4".into()));
    }

    #[test]
    fn test_required_capabilities_builder() {
        let req = RequiredCapabilities::new()
            .require_observe("cap1", "chan1")
            .require_receive("cap2", "chan2")
            .require_set("cap3", "chan3")
            .require_send("cap4", "chan4");

        assert_eq!(req.observes.len(), 1);
        assert_eq!(req.receives.len(), 1);
        assert_eq!(req.sets.len(), 1);
        assert_eq!(req.sends.len(), 1);
        assert_eq!(req.observes[0], ("cap1".into(), "chan1".into()));
        assert_eq!(req.receives[0], ("cap2".into(), "chan2".into()));
        assert_eq!(req.sets[0], ("cap3".into(), "chan3".into()));
        assert_eq!(req.sends[0], ("cap4".into(), "chan4".into()));
    }

    #[test]
    fn test_obligation_builder() {
        let obl = Obligation::new("test_obligation")
            .with_required(RequiredCapabilities::new().require_observe("s", "c"))
            .with_min_effect(Effect::Deliberative);

        assert_eq!(obl.name, "test_obligation".into());
        assert_eq!(obl.min_effect, Effect::Deliberative);
        assert_eq!(obl.required.observes.len(), 1);
    }

    #[test]
    fn test_missing_receive_capability() {
        let checker = ObligationChecker::new();

        let obligation = Obligation::new("recv_test")
            .with_required(RequiredCapabilities::new().require_receive("queue", "events"));

        let capabilities = WorkflowCapabilities::new();

        let result = checker.verify(&obligation, &capabilities, Effect::Epistemic);
        assert!(result.is_err());
        assert!(
            matches!(result.unwrap_err(), ObligationCheckError::MissingCapability { operation, .. } if operation == "receive")
        );
    }

    #[test]
    fn test_missing_send_capability() {
        let checker = ObligationChecker::new();

        let obligation = Obligation::new("send_test")
            .with_required(RequiredCapabilities::new().require_send("api", "webhook"));

        let capabilities = WorkflowCapabilities::new();

        let result = checker.verify(&obligation, &capabilities, Effect::Epistemic);
        assert!(result.is_err());
        assert!(
            matches!(result.unwrap_err(), ObligationCheckError::MissingCapability { operation, .. } if operation == "send")
        );
    }

    #[test]
    fn test_exact_effect_match() {
        let checker = ObligationChecker::new();

        let obligation = Obligation::new("exact_effect").with_min_effect(Effect::Deliberative);

        let capabilities = WorkflowCapabilities::new();

        // Exact match should succeed
        let result = checker.verify(&obligation, &capabilities, Effect::Deliberative);
        assert!(result.is_ok());
    }

    #[test]
    fn test_higher_effect_satisfies() {
        let checker = ObligationChecker::new();

        let obligation = Obligation::new("higher_effect").with_min_effect(Effect::Epistemic);

        let capabilities = WorkflowCapabilities::new();

        // Higher effect should satisfy lower requirement
        let result = checker.verify(&obligation, &capabilities, Effect::Operational);
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_display() {
        let err = ObligationCheckError::MissingCapability {
            obligation: "test".to_string(),
            operation: "observe".to_string(),
            capability: "sensor:temp".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("missing capability"));
        assert!(msg.contains("test"));
        assert!(msg.contains("observe"));
        assert!(msg.contains("sensor:temp"));

        let err = ObligationCheckError::InsufficientEffect {
            obligation: "test".to_string(),
            required: Effect::Operational,
            actual: Effect::Epistemic,
        };
        let msg = format!("{}", err);
        assert!(msg.contains("insufficient effect"));
        assert!(msg.contains("test"));
    }
}
