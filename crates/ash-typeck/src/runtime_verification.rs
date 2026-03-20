//! Runtime verification for capability availability (TASK-114 to TASK-119)
//!
//! Provides verification that runtime provides all capabilities required by workflow.

use crate::types::Type;
use ash_core::{Effect, Name};
use ash_parser::surface::Workflow;
use std::collections::HashMap;
use thiserror::Error;

// =============================================================================
// Capability Schema Types (from capability_typecheck)
// =============================================================================

/// Schema for a capability channel's read/write types
#[derive(Debug, Clone, PartialEq)]
pub struct CapabilitySchema {
    /// Type returned by observe/receive (None if not readable)
    pub read: Option<Type>,
    /// Type accepted by set/send (None if not writable)
    pub write: Option<Type>,
}

impl CapabilitySchema {
    /// Create a read-only schema
    pub fn read_only(read: Type) -> Self {
        Self {
            read: Some(read),
            write: None,
        }
    }

    /// Create a write-only schema
    pub fn write_only(write: Type) -> Self {
        Self {
            read: None,
            write: Some(write),
        }
    }

    /// Create a read-write schema
    pub fn read_write(read: Type, write: Type) -> Self {
        Self {
            read: Some(read),
            write: Some(write),
        }
    }
}

/// Registry of capability schemas
#[derive(Debug, Clone, Default)]
pub struct CapabilitySchemaRegistry {
    /// Maps (capability, channel) -> schema
    schemas: HashMap<(Name, Name), CapabilitySchema>,
}

impl CapabilitySchemaRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            schemas: HashMap::new(),
        }
    }

    /// Register a capability schema
    pub fn register(&mut self, cap: Name, chan: Name, schema: CapabilitySchema) {
        self.schemas.insert((cap, chan), schema);
    }

    /// Get a schema by capability and channel
    pub fn get(&self, cap: &str, chan: &str) -> Option<&CapabilitySchema> {
        self.schemas.get(&(cap.to_string(), chan.to_string()))
    }

    /// Check if a capability is registered
    pub fn has(&self, cap: &str, chan: &str) -> bool {
        self.schemas
            .contains_key(&(cap.to_string(), chan.to_string()))
    }
}

// =============================================================================
// Workflow Capabilities (from obligation_checker)
// =============================================================================

/// Required capabilities for a workflow
#[derive(Debug, Clone, Default, PartialEq)]
pub struct WorkflowCapabilities {
    /// Capabilities that are observed (read)
    pub observes: Vec<(Name, Name)>,
    /// Capabilities that are received from (read stream)
    pub receives: Vec<(Name, Name)>,
    /// Capabilities that are set (write)
    pub sets: Vec<(Name, Name)>,
    /// Capabilities that are sent to (write stream)
    pub sends: Vec<(Name, Name)>,
}

impl WorkflowCapabilities {
    /// Create empty capabilities
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an observe capability
    pub fn observe(mut self, cap: impl Into<Name>, chan: impl Into<Name>) -> Self {
        self.observes.push((cap.into(), chan.into()));
        self
    }

    /// Add a receive capability
    pub fn receive(mut self, cap: impl Into<Name>, chan: impl Into<Name>) -> Self {
        self.receives.push((cap.into(), chan.into()));
        self
    }

    /// Add a set capability
    pub fn set(mut self, cap: impl Into<Name>, chan: impl Into<Name>) -> Self {
        self.sets.push((cap.into(), chan.into()));
        self
    }

    /// Add a send capability
    pub fn send(mut self, cap: impl Into<Name>, chan: impl Into<Name>) -> Self {
        self.sends.push((cap.into(), chan.into()));
        self
    }

    /// Check if can observe a capability
    pub fn can_observe(&self, cap: &str, chan: &str) -> bool {
        self.observes.iter().any(|(c, ch)| c == cap && ch == chan)
    }

    /// Check if can receive from a capability
    pub fn can_receive(&self, cap: &str, chan: &str) -> bool {
        self.receives.iter().any(|(c, ch)| c == cap && ch == chan)
    }

    /// Check if can set a capability
    pub fn can_set(&self, cap: &str, chan: &str) -> bool {
        self.sets.iter().any(|(c, ch)| c == cap && ch == chan)
    }

    /// Check if can send to a capability
    pub fn can_send(&self, cap: &str, chan: &str) -> bool {
        self.sends.iter().any(|(c, ch)| c == cap && ch == chan)
    }
}

// =============================================================================
// Verification Types (TASK-114)
// =============================================================================

/// Verification error
#[derive(Debug, Error, Clone, PartialEq)]
pub enum VerificationError {
    /// Missing capability error
    #[error(
        "missing capability: operation '{operation}' requires '{capability}' but it is not available"
    )]
    MissingCapability {
        /// Operation that requires the capability
        operation: String,
        /// Name of the missing capability
        capability: String,
    },

    /// Capability not observable (no read type)
    #[error("capability '{capability}' is not observable")]
    NotObservable {
        /// Name of the capability
        capability: String,
    },

    /// Capability not receivable (no read type)
    #[error("capability '{capability}' is not receivable")]
    NotReceivable {
        /// Name of the capability
        capability: String,
    },

    /// Capability not settable (no write type)
    #[error("capability '{capability}' is not settable (read-only)")]
    NotSettable {
        /// Name of the capability
        capability: String,
    },

    /// Capability not sendable (no write type)
    #[error("capability '{capability}' is not sendable (receive-only)")]
    NotSendable {
        /// Name of the capability
        capability: String,
    },

    /// Effect too high for runtime bounds
    #[error(
        "effect too high: workflow requires {workflow_effect:?} but runtime only allows {max_allowed:?}"
    )]
    EffectTooHigh {
        /// The effect required by the workflow
        workflow_effect: Effect,
        /// The maximum effect allowed by the runtime
        max_allowed: Effect,
    },

    /// Role mismatch error (TASK-115)
    #[error("role mismatch: required '{required:?}', available {available:?}")]
    RoleMismatch {
        /// Required role
        required: Role,
        /// Available roles
        available: Vec<Role>,
    },

    /// Missing obligation error (TASK-115)
    #[error("missing obligation: '{0}' not found in runtime context")]
    MissingObligation(Name),

    /// Missing required capability (TASK-115 variant)
    #[error("missing capability: required '{required}', not available")]
    MissingRequiredCapability {
        /// Required capability
        required: String,
    },

    /// Policy conflict error (TASK-117)
    #[error("policy conflict in '{policy}': {reason}")]
    PolicyConflict {
        /// Name of the policy that triggered the conflict
        policy: String,
        /// Reason for the conflict
        reason: String,
    },
}

/// Verification warning
#[derive(Debug, Clone, PartialEq)]
pub enum VerificationWarning {
    /// Extra capability available but not required
    ExtraCapability {
        /// Name of the extra capability
        capability: String,
    },
    /// Operation requires approval from a specific role (TASK-117)
    RequiresApproval {
        /// Role that must approve
        role: Role,
        /// Description of the operation requiring approval
        operation: String,
    },
}

/// Verification result
#[derive(Debug, Default, Clone, PartialEq)]
pub struct VerificationResult {
    /// Errors found during verification
    pub errors: Vec<VerificationError>,
    /// Warnings found during verification
    pub warnings: Vec<VerificationWarning>,
    /// Whether the workflow can execute
    pub can_execute: bool,
}

impl VerificationResult {
    /// Create a new empty verification result
    pub fn new() -> Self {
        Self {
            errors: vec![],
            warnings: vec![],
            can_execute: true,
        }
    }

    /// Add an error to the result
    pub fn add_error(&mut self, error: VerificationError) {
        self.errors.push(error);
        self.can_execute = false;
    }

    /// Add a warning to the result
    pub fn add_warning(&mut self, warning: VerificationWarning) {
        self.warnings.push(warning);
    }

    /// Check if verification succeeded (no errors)
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    /// Check if the workflow can execute
    pub fn can_execute(&self) -> bool {
        self.can_execute && self.errors.is_empty()
    }

    /// Merge another verification result into this one
    pub fn merge(&mut self, other: VerificationResult) {
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
        self.can_execute = self.can_execute && other.can_execute;
    }
}

/// Capability availability verifier
#[derive(Debug, Clone)]
pub struct CapabilityVerifier;

impl CapabilityVerifier {
    /// Create a new capability verifier
    pub fn new() -> Self {
        Self
    }

    /// Verify that all required capabilities are available with correct modes
    pub fn verify(
        &self,
        required: &WorkflowCapabilities,
        registry: &CapabilitySchemaRegistry,
    ) -> VerificationResult {
        let mut result = VerificationResult::new();

        // Check observes
        for (cap, chan) in &required.observes {
            let cap_str = format!("{}:{}", cap, chan);
            match registry.get(cap, chan) {
                None => result.add_error(VerificationError::MissingCapability {
                    operation: "observe".into(),
                    capability: cap_str,
                }),
                Some(schema) => {
                    // Check if readable (has read type)
                    if schema.read.is_none() {
                        result.add_error(VerificationError::NotObservable {
                            capability: cap_str,
                        });
                    }
                }
            }
        }

        // Check receives
        for (cap, chan) in &required.receives {
            let cap_str = format!("{}:{}", cap, chan);
            match registry.get(cap, chan) {
                None => result.add_error(VerificationError::MissingCapability {
                    operation: "receive".into(),
                    capability: cap_str,
                }),
                Some(schema) => {
                    if schema.read.is_none() {
                        result.add_error(VerificationError::NotReceivable {
                            capability: cap_str,
                        });
                    }
                }
            }
        }

        // Check sets
        for (cap, chan) in &required.sets {
            let cap_str = format!("{}:{}", cap, chan);
            match registry.get(cap, chan) {
                None => result.add_error(VerificationError::MissingCapability {
                    operation: "set".into(),
                    capability: cap_str,
                }),
                Some(schema) => {
                    if schema.write.is_none() {
                        result.add_error(VerificationError::NotSettable {
                            capability: cap_str,
                        });
                    }
                }
            }
        }

        // Check sends
        for (cap, chan) in &required.sends {
            let cap_str = format!("{}:{}", cap, chan);
            match registry.get(cap, chan) {
                None => result.add_error(VerificationError::MissingCapability {
                    operation: "send".into(),
                    capability: cap_str,
                }),
                Some(schema) => {
                    if schema.write.is_none() {
                        result.add_error(VerificationError::NotSendable {
                            capability: cap_str,
                        });
                    }
                }
            }
        }

        result
    }
}

impl Default for CapabilityVerifier {
    fn default() -> Self {
        Self::new()
    }
}

/// Effect compatibility checker (TASK-116)
///
/// Checks that a workflow's effect is within runtime bounds.
/// This ensures that workflows requiring higher privileges
/// cannot execute in restricted runtime environments.
///
/// # Example
///
/// ```
/// use ash_typeck::runtime_verification::EffectChecker;
/// use ash_core::Effect;
/// use ash_parser::surface::Workflow;
/// use ash_parser::token::Span;
///
/// let checker = EffectChecker::new();
/// let workflow = Workflow::Done { span: Span::default() };
/// let result = checker.check(&workflow, Effect::Operational);
/// assert!(result.is_ok());
/// ```
#[derive(Debug, Clone)]
pub struct EffectChecker;

impl EffectChecker {
    /// Create a new effect checker
    pub fn new() -> Self {
        Self
    }

    /// Check that workflow effect is within runtime bounds
    ///
    /// # Arguments
    /// * `workflow` - The workflow to check
    /// * `max_allowed` - The maximum effect level allowed by the runtime
    ///
    /// # Returns
    /// A `VerificationResult` containing any errors found
    ///
    /// # Effect Comparison
    /// If `workflow_effect > max_allowed`, the workflow requires higher
    /// privileges than the runtime allows. For example, a workflow that
    /// is `Operational` cannot execute in a runtime that only allows
    /// `Epistemic` effects.
    pub fn check(&self, workflow: &Workflow, max_allowed: Effect) -> VerificationResult {
        let mut result = VerificationResult::new();

        let workflow_effect = crate::effect::infer_effect(workflow);

        // Effect comparison: workflow_effect > max_allowed means
        // workflow requires higher privileges than runtime allows
        // For example: workflow is Operational but runtime only allows Epistemic
        if workflow_effect > max_allowed {
            result.add_error(VerificationError::EffectTooHigh {
                workflow_effect,
                max_allowed,
            });
        }

        result
    }
}

impl Default for EffectChecker {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Runtime Obligation Checker (TASK-115)
// =============================================================================

/// Role type for authorization
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Role(pub String);

impl Role {
    /// Create a new role
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// Get the role name as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for Role {
    fn from(s: &str) -> Self {
        Self(s.into())
    }
}

/// An obligation that must be satisfied at runtime
#[derive(Debug, Clone, PartialEq)]
pub struct Obligation {
    /// Name of the obligation
    pub name: Name,
    /// Description of what the obligation requires
    pub description: String,
    /// Required capabilities to satisfy this obligation
    pub required_capabilities: ObligationRequirements,
}

impl Obligation {
    /// Create a new obligation
    pub fn new(name: impl Into<Name>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            required_capabilities: ObligationRequirements::default(),
        }
    }

    /// Add a required observe capability.
    pub fn with_observe_capability(
        mut self,
        capability: impl Into<Name>,
        channel: impl Into<Name>,
    ) -> Self {
        self.required_capabilities = self
            .required_capabilities
            .require_observe(capability, channel);
        self
    }

    /// Add a required receive capability.
    pub fn with_receive_capability(
        mut self,
        capability: impl Into<Name>,
        channel: impl Into<Name>,
    ) -> Self {
        self.required_capabilities = self
            .required_capabilities
            .require_receive(capability, channel);
        self
    }

    /// Add a required set capability.
    pub fn with_set_capability(
        mut self,
        capability: impl Into<Name>,
        channel: impl Into<Name>,
    ) -> Self {
        self.required_capabilities = self.required_capabilities.require_set(capability, channel);
        self
    }

    /// Add a required send capability.
    pub fn with_send_capability(
        mut self,
        capability: impl Into<Name>,
        channel: impl Into<Name>,
    ) -> Self {
        self.required_capabilities = self.required_capabilities.require_send(capability, channel);
        self
    }
}

/// Explicit obligation-backed runtime requirements consumed by aggregate verification.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ObligationRequirements {
    /// Required observe capabilities.
    pub observes: Vec<(Name, Name)>,
    /// Required receive capabilities.
    pub receives: Vec<(Name, Name)>,
    /// Required set capabilities.
    pub sets: Vec<(Name, Name)>,
    /// Required send capabilities.
    pub sends: Vec<(Name, Name)>,
    /// Required runtime roles.
    pub roles: Vec<Role>,
    /// Required obligation names.
    pub obligations: Vec<Name>,
}

impl ObligationRequirements {
    /// Create empty obligation-backed requirements.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add and return a required observe capability.
    pub fn require_observe(
        mut self,
        capability: impl Into<Name>,
        channel: impl Into<Name>,
    ) -> Self {
        self.observes.push((capability.into(), channel.into()));
        self
    }

    /// Add and return a required receive capability.
    pub fn require_receive(
        mut self,
        capability: impl Into<Name>,
        channel: impl Into<Name>,
    ) -> Self {
        self.receives.push((capability.into(), channel.into()));
        self
    }

    /// Add and return a required set capability.
    pub fn require_set(mut self, capability: impl Into<Name>, channel: impl Into<Name>) -> Self {
        self.sets.push((capability.into(), channel.into()));
        self
    }

    /// Add and return a required send capability.
    pub fn require_send(mut self, capability: impl Into<Name>, channel: impl Into<Name>) -> Self {
        self.sends.push((capability.into(), channel.into()));
        self
    }

    /// Add and return a required runtime role.
    pub fn require_role(mut self, role: Role) -> Self {
        self.roles.push(role);
        self
    }

    /// Add and return a required obligation name.
    pub fn require_obligation(mut self, obligation: impl Into<Name>) -> Self {
        self.obligations.push(obligation.into());
        self
    }

    /// Check if no requirements are present.
    pub fn is_empty(&self) -> bool {
        self.observes.is_empty()
            && self.receives.is_empty()
            && self.sets.is_empty()
            && self.sends.is_empty()
            && self.roles.is_empty()
            && self.obligations.is_empty()
    }

    /// Build runtime obligation requirements from structured declaration requirements.
    pub fn from_declared_requirements(
        requirements: &crate::obligation_checker::RequiredCapabilities,
    ) -> Self {
        Self {
            observes: requirements
                .observes
                .iter()
                .map(|(cap, channel)| (cap.to_string(), channel.to_string()))
                .collect(),
            receives: requirements
                .receives
                .iter()
                .map(|(cap, channel)| (cap.to_string(), channel.to_string()))
                .collect(),
            sets: requirements
                .sets
                .iter()
                .map(|(cap, channel)| (cap.to_string(), channel.to_string()))
                .collect(),
            sends: requirements
                .sends
                .iter()
                .map(|(cap, channel)| (cap.to_string(), channel.to_string()))
                .collect(),
            roles: Vec::new(),
            obligations: Vec::new(),
        }
    }
}

/// Runtime context for obligations
#[derive(Debug, Clone)]
pub struct RuntimeObligations {
    /// Available roles
    pub roles: Vec<Role>,
    /// Active obligations
    pub obligations: Vec<Obligation>,
}

impl RuntimeObligations {
    /// Create a new empty runtime obligations context
    pub fn new() -> Self {
        Self {
            roles: vec![],
            obligations: vec![],
        }
    }

    /// Add a role (builder-style)
    pub fn with_role(mut self, role: Role) -> Self {
        self.roles.push(role);
        self
    }

    /// Add an obligation (builder-style)
    pub fn with_obligation(mut self, obligation: Obligation) -> Self {
        self.obligations.push(obligation);
        self
    }

    /// Check if a specific role is present
    pub fn has_role(&self, role: &Role) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    /// Check if a specific obligation is present by name
    pub fn has_obligation(&self, name: &Name) -> bool {
        self.obligations.iter().any(|o| &o.name == name)
    }
}

impl Default for RuntimeObligations {
    fn default() -> Self {
        Self::new()
    }
}

/// Obligation satisfaction checker
#[derive(Debug)]
pub struct RuntimeObligationChecker;

impl RuntimeObligationChecker {
    /// Create a new obligation checker
    pub fn new() -> Self {
        Self
    }

    /// Check that runtime obligations satisfy workflow requirements
    pub fn check(
        &self,
        required: &ObligationRequirements,
        runtime: &RuntimeContext,
    ) -> VerificationResult {
        let mut result = VerificationResult::new();

        for (capability, channel) in &required.observes {
            if !runtime.obligations.obligations.iter().any(|obligation| {
                obligation
                    .required_capabilities
                    .observes
                    .contains(&(capability.clone(), channel.clone()))
            }) {
                result.add_error(VerificationError::MissingRequiredCapability {
                    required: format!("{capability}:{channel}"),
                });
            }
        }

        for (capability, channel) in &required.receives {
            if !runtime.obligations.obligations.iter().any(|obligation| {
                obligation
                    .required_capabilities
                    .receives
                    .contains(&(capability.clone(), channel.clone()))
            }) {
                result.add_error(VerificationError::MissingRequiredCapability {
                    required: format!("{capability}:{channel}"),
                });
            }
        }

        for (capability, channel) in &required.sets {
            if !runtime.obligations.obligations.iter().any(|obligation| {
                obligation
                    .required_capabilities
                    .sets
                    .contains(&(capability.clone(), channel.clone()))
            }) {
                result.add_error(VerificationError::MissingRequiredCapability {
                    required: format!("{capability}:{channel}"),
                });
            }
        }

        for (capability, channel) in &required.sends {
            if !runtime.obligations.obligations.iter().any(|obligation| {
                obligation
                    .required_capabilities
                    .sends
                    .contains(&(capability.clone(), channel.clone()))
            }) {
                result.add_error(VerificationError::MissingRequiredCapability {
                    required: format!("{capability}:{channel}"),
                });
            }
        }

        for role in &required.roles {
            let has_role = runtime.role.as_ref().is_some_and(|active| active == role)
                || runtime.obligations.has_role(role);

            if !has_role {
                result.add_error(VerificationError::RoleMismatch {
                    required: role.clone(),
                    available: runtime
                        .role
                        .iter()
                        .cloned()
                        .chain(runtime.obligations.roles.iter().cloned())
                        .collect(),
                });
            }
        }

        for obligation in &required.obligations {
            if !runtime.obligations.has_obligation(obligation) {
                result.add_error(VerificationError::MissingObligation(obligation.clone()));
            }
        }

        result
    }

    /// Check role requirement
    pub fn check_role(
        &self,
        required_role: &Role,
        runtime: &RuntimeObligations,
    ) -> VerificationResult {
        let mut result = VerificationResult::new();

        if !runtime.has_role(required_role) {
            result.add_error(VerificationError::RoleMismatch {
                required: required_role.clone(),
                available: runtime.roles.clone(),
            });
        }

        result
    }

    /// Check if a specific obligation is present
    pub fn check_obligation_present(
        &self,
        name: &Name,
        runtime: &RuntimeObligations,
    ) -> VerificationResult {
        let mut result = VerificationResult::new();

        if !runtime.has_obligation(name) {
            result.add_error(VerificationError::MissingObligation(name.clone()));
        }

        result
    }
}

impl Default for RuntimeObligationChecker {
    fn default() -> Self {
        Self::new()
    }
}

// Extend VerificationError with TASK-115 variants
impl VerificationError {
    /// Create a role mismatch error
    pub fn role_mismatch(required: Role, available: Vec<Role>) -> Self {
        Self::RoleMismatch {
            required,
            available,
        }
    }

    /// Create a missing obligation error
    pub fn missing_obligation(name: impl Into<Name>) -> Self {
        Self::MissingObligation(name.into())
    }

    /// Create a missing capability error
    pub fn missing_capability(required: impl Into<String>) -> Self {
        Self::MissingRequiredCapability {
            required: required.into(),
        }
    }
}

// =============================================================================
// Static Policy Validator (TASK-117)
// =============================================================================

/// Policy decision types for static validation
#[derive(Debug, Clone, PartialEq)]
pub enum PolicyDecisionType {
    /// Allow the operation
    Permit,
    /// Deny the operation
    Deny,
    /// Require approval from a specific role
    RequiresApproval { role: Role },
}

/// Policy for static validation
#[derive(Debug, Clone)]
pub struct StaticPolicy {
    /// Name of the policy
    pub name: String,
    /// Function to determine if this policy applies to a capability/channel
    pub applies_to: fn(&str, &str) -> bool,
    /// Decision to apply when this policy matches
    pub decision: PolicyDecisionType,
}

impl StaticPolicy {
    /// Create a new static policy with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            applies_to: |_, _| true,
            decision: PolicyDecisionType::Permit,
        }
    }

    /// Set the applies_to function for selective policy application
    pub fn applies_to(mut self, f: fn(&str, &str) -> bool) -> Self {
        self.applies_to = f;
        self
    }

    /// Set the decision for this policy
    pub fn decision(mut self, d: PolicyDecisionType) -> Self {
        self.decision = d;
        self
    }
}

/// Static policy validator
///
/// Validates workflows against static policies before execution.
/// Policies can permit, deny, or require approval for specific capabilities.
#[derive(Debug, Clone)]
pub struct StaticPolicyValidator;

impl StaticPolicyValidator {
    /// Create a new static policy validator
    pub fn new() -> Self {
        Self
    }

    /// Validate workflow against static policies
    ///
    /// Checks each required capability against the provided policies.
    /// Returns a VerificationResult containing any errors or warnings.
    pub fn validate(
        &self,
        workflow_capabilities: &WorkflowCapabilities,
        policies: &[StaticPolicy],
    ) -> VerificationResult {
        let mut result = VerificationResult::new();

        // Check each required capability against policies
        for (cap, chan) in &workflow_capabilities.observes {
            self.check_capability(cap, chan, "observe", policies, &mut result);
        }

        for (cap, chan) in &workflow_capabilities.receives {
            self.check_capability(cap, chan, "receive", policies, &mut result);
        }

        for (cap, chan) in &workflow_capabilities.sets {
            self.check_capability(cap, chan, "set", policies, &mut result);
        }

        for (cap, chan) in &workflow_capabilities.sends {
            self.check_capability(cap, chan, "send", policies, &mut result);
        }

        result
    }

    fn check_capability(
        &self,
        cap: &Name,
        chan: &Name,
        operation: &str,
        policies: &[StaticPolicy],
        result: &mut VerificationResult,
    ) {
        let cap_str = cap.as_str();
        let chan_str = chan.as_str();

        for policy in policies {
            if (policy.applies_to)(cap_str, chan_str) {
                match &policy.decision {
                    PolicyDecisionType::Deny => {
                        result.add_error(VerificationError::PolicyConflict {
                            policy: policy.name.clone(),
                            reason: format!("{} on {}:{} is denied", operation, cap_str, chan_str),
                        });
                    }
                    PolicyDecisionType::RequiresApproval { role } => {
                        result.add_warning(VerificationWarning::RequiresApproval {
                            role: role.clone(),
                            operation: format!("{} {}:{}", operation, cap_str, chan_str),
                        });
                    }
                    PolicyDecisionType::Permit => {}
                }
            }
        }
    }
}

impl Default for StaticPolicyValidator {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Per-Operation Runtime Verifier (TASK-118)
// =============================================================================

use std::time::Duration;

/// Direction of a capability operation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    /// Input operation (observe, receive)
    Input,
    /// Output operation (set, send)
    Output,
}

/// Operation type for per-operation verification
#[derive(Debug, Clone)]
pub struct CapabilityOperation {
    /// Direction of the operation
    pub direction: Direction,
    /// Capability name
    pub capability: Name,
    /// Channel name
    pub channel: Name,
    /// Value for output operations
    pub value: Option<ash_core::Value>,
}

impl CapabilityOperation {
    /// Create an observe operation (input)
    pub fn observe(cap: impl Into<Name>, chan: impl Into<Name>) -> Self {
        Self {
            direction: Direction::Input,
            capability: cap.into(),
            channel: chan.into(),
            value: None,
        }
    }

    /// Create a set operation (output)
    pub fn set(cap: impl Into<Name>, chan: impl Into<Name>, value: ash_core::Value) -> Self {
        Self {
            direction: Direction::Output,
            capability: cap.into(),
            channel: chan.into(),
            value: Some(value),
        }
    }
}

/// Operation verification result
#[derive(Debug, Clone, PartialEq)]
pub enum OperationResult {
    /// Operation is permitted to proceed
    Proceed,
    /// Operation is denied by policy
    Denied { policy: String },
    /// Operation requires approval from a specific role
    RequiresApproval { role: Role },
    /// Operation was transformed
    Transformed { transformation: String },
}

/// Operation error
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
pub enum OperationError {
    /// Capability is not available
    #[error("capability unavailable: {0}")]
    CapabilityUnavailable(String),

    /// Operation mode is not supported
    #[error("operation mode not supported: {0}")]
    ModeNotSupported(String),

    /// Rate limit exceeded
    #[error("rate limit exceeded for {0}")]
    RateLimitExceeded(String),

    /// Policy denied the operation
    #[error("policy denied: {0}")]
    PolicyDenied(String),
}

/// Simple rate limiter for capability operations
#[derive(Debug)]
pub struct RateLimiter {
    /// Maximum requests allowed in the window
    max_requests: u32,
    /// Time window for rate limiting
    window: Duration,
    /// Request timestamps within the window
    requests: Vec<std::time::Instant>,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(max_requests: u32, window: Duration) -> Self {
        Self {
            max_requests,
            window,
            requests: vec![],
        }
    }

    /// Check if adding a new request would exceed the rate limit
    pub fn would_exceed(&mut self, _op: &CapabilityOperation) -> bool {
        let now = std::time::Instant::now();
        // Remove old requests outside the window
        self.requests
            .retain(|t| now.duration_since(*t) < self.window);

        // Check if adding a new request would exceed the limit
        if self.requests.len() as u32 >= self.max_requests {
            true
        } else {
            self.requests.push(now);
            false
        }
    }

    /// Record a request (for manual tracking)
    pub fn record(&mut self, _op: &CapabilityOperation) {
        self.requests.push(std::time::Instant::now());
    }
}

/// Per-operation verifier
///
/// Verifies individual capability operations at runtime against
/// registered schemas, policies, and rate limits.
#[derive(Debug, Clone)]
pub struct OperationVerifier;

impl OperationVerifier {
    /// Create a new operation verifier
    pub fn new() -> Self {
        Self
    }

    /// Verify a single operation at runtime
    ///
    /// # Arguments
    /// * `op` - The capability operation to verify
    /// * `registry` - Registry of capability schemas
    /// * `policies` - Static policies to evaluate
    /// * `rate_limiter` - Rate limiter for the operation
    ///
    /// # Returns
    /// `OperationResult` if verification succeeds, or `OperationError` if it fails
    pub async fn verify(
        &self,
        op: &CapabilityOperation,
        registry: &CapabilitySchemaRegistry,
        policies: &[StaticPolicy],
        rate_limiter: &mut RateLimiter,
    ) -> Result<OperationResult, OperationError> {
        // 1. Check capability is available
        let schema = registry.get(&op.capability, &op.channel).ok_or_else(|| {
            OperationError::CapabilityUnavailable(format!("{}:{}", op.capability, op.channel))
        })?;

        // 2. Check mode is supported
        match op.direction {
            Direction::Input => {
                if schema.read.is_none() {
                    return Err(OperationError::ModeNotSupported(format!(
                        "{}:{} is not readable",
                        op.capability, op.channel
                    )));
                }
            }
            Direction::Output => {
                if schema.write.is_none() {
                    return Err(OperationError::ModeNotSupported(format!(
                        "{}:{} is not writable",
                        op.capability, op.channel
                    )));
                }
            }
        }

        // 3. Evaluate dynamic policies
        let cap_str = op.capability.as_str();
        let chan_str = op.channel.as_str();

        for policy in policies {
            if (policy.applies_to)(cap_str, chan_str) {
                match &policy.decision {
                    PolicyDecisionType::Deny => {
                        return Ok(OperationResult::Denied {
                            policy: policy.name.clone(),
                        });
                    }
                    PolicyDecisionType::RequiresApproval { role } => {
                        return Ok(OperationResult::RequiresApproval { role: role.clone() });
                    }
                    PolicyDecisionType::Permit => {}
                }
            }
        }

        // 4. Check rate limits
        if rate_limiter.would_exceed(op) {
            return Err(OperationError::RateLimitExceeded(format!(
                "{}:{}",
                op.capability, op.channel
            )));
        }

        Ok(OperationResult::Proceed)
    }
}

impl Default for OperationVerifier {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Verification Aggregator (TASK-119)
// =============================================================================

/// Runtime context for verification
#[derive(Debug)]
pub struct RuntimeContext {
    /// Available capability schemas
    pub capabilities: CapabilitySchemaRegistry,
    /// Runtime obligations
    pub obligations: RuntimeObligations,
    /// Runtime mailbox registry used by receive verification.
    pub mailboxes: MailboxRegistry,
    /// Source scheduler used by runtime receive execution.
    pub scheduler: SourceScheduler,
    /// Approval queue for approval-gated policy outcomes.
    pub approval_queue: ApprovalQueue,
    /// Provenance sink for verification-time audit metadata.
    pub provenance: ProvenanceSink,
    /// Active runtime role, when known.
    pub role: Option<Role>,
    /// Maximum allowed effect
    pub max_effect: Effect,
    /// Static policies to validate against. This is the type-layer stand-in for the
    /// canonical runtime policy registry.
    pub policies: Vec<StaticPolicy>,
}

impl RuntimeContext {
    /// Create a new runtime context with the given max effect
    pub fn new(max_effect: Effect) -> Self {
        Self {
            capabilities: CapabilitySchemaRegistry::new(),
            obligations: RuntimeObligations::new(),
            mailboxes: MailboxRegistry::new(),
            scheduler: SourceScheduler::new(),
            approval_queue: ApprovalQueue::new(),
            provenance: ProvenanceSink::new(),
            role: None,
            max_effect,
            policies: vec![],
        }
    }

    /// Set the capability registry (builder-style)
    pub fn with_capabilities(mut self, registry: CapabilitySchemaRegistry) -> Self {
        self.capabilities = registry;
        self
    }

    /// Set the obligations (builder-style)
    pub fn with_obligations(mut self, obligations: RuntimeObligations) -> Self {
        self.obligations = obligations;
        self
    }

    /// Set the policies (builder-style)
    pub fn with_policies(mut self, policies: Vec<StaticPolicy>) -> Self {
        self.policies = policies;
        self
    }

    /// Set the runtime role (builder-style)
    pub fn with_role(mut self, role: Role) -> Self {
        self.role = Some(role);
        self
    }

    /// Add a single policy (builder-style)
    pub fn add_policy(mut self, policy: StaticPolicy) -> Self {
        self.policies.push(policy);
        self
    }
}

/// Verification-time mailbox registry placeholder.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MailboxRegistry {
    entries: Vec<String>,
}

impl MailboxRegistry {
    /// Create an empty mailbox registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Check whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Verification-time source scheduler placeholder.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SourceScheduler;

impl SourceScheduler {
    /// Create a new source scheduler placeholder.
    pub fn new() -> Self {
        Self
    }
}

/// Verification-time approval queue placeholder.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ApprovalQueue {
    pending: Vec<String>,
}

impl ApprovalQueue {
    /// Create an empty approval queue.
    pub fn new() -> Self {
        Self::default()
    }

    /// Check whether the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }
}

/// Verification-time provenance sink placeholder.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ProvenanceSink {
    events: Vec<String>,
}

impl ProvenanceSink {
    /// Create an empty provenance sink.
    pub fn new() -> Self {
        Self::default()
    }

    /// Check whether the sink is empty.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

/// Comprehensive verification aggregator
///
/// Combines results from all verification components:
/// - Capability verification (TASK-114)
/// - Obligation checking (TASK-115)
/// - Effect bounds checking (TASK-116)
/// - Static policy validation (TASK-117)
#[derive(Debug)]
pub struct VerificationAggregator {
    capability_verifier: CapabilityVerifier,
    obligation_checker: RuntimeObligationChecker,
    effect_checker: EffectChecker,
    policy_validator: StaticPolicyValidator,
}

/// Inputs to aggregate runtime verification.
///
/// Workflow capability declarations and obligation-backed runtime requirements are distinct
/// inputs and must be provided separately.
#[derive(Debug, Clone, PartialEq)]
pub struct AggregateVerificationInputs {
    /// Workflow-declared capability use.
    pub workflow_capabilities: WorkflowCapabilities,
    /// Explicit obligation-backed runtime requirements.
    pub obligation_requirements: ObligationRequirements,
}

impl AggregateVerificationInputs {
    /// Create aggregate verification inputs from separate workflow and obligation requirements.
    pub fn new(
        workflow_capabilities: WorkflowCapabilities,
        obligation_requirements: ObligationRequirements,
    ) -> Self {
        Self {
            workflow_capabilities,
            obligation_requirements,
        }
    }
}

impl VerificationAggregator {
    /// Create a new verification aggregator
    pub fn new() -> Self {
        Self {
            capability_verifier: CapabilityVerifier::new(),
            obligation_checker: RuntimeObligationChecker::new(),
            effect_checker: EffectChecker::new(),
            policy_validator: StaticPolicyValidator::new(),
        }
    }

    /// Run all verifications and aggregate results
    ///
    /// # Arguments
    /// * `workflow` - The workflow to verify
    /// * `inputs` - The separated workflow and obligation requirements
    /// * `runtime` - The runtime context to verify against
    ///
    /// # Returns
    /// A `VerificationResult` containing all errors and warnings from all checks
    pub fn aggregate(
        &self,
        workflow: &Workflow,
        inputs: &AggregateVerificationInputs,
        runtime: &RuntimeContext,
    ) -> VerificationResult {
        let mut result = VerificationResult::new();

        // 1. Verify capabilities are available
        let cap_result = self
            .capability_verifier
            .verify(&inputs.workflow_capabilities, &runtime.capabilities);
        result.merge(cap_result);

        // 2. Check obligations are satisfied
        let obl_result = self
            .obligation_checker
            .check(&inputs.obligation_requirements, runtime);
        result.merge(obl_result);

        // 3. Check effect bounds
        let effect_result = self.effect_checker.check(workflow, runtime.max_effect);
        result.merge(effect_result);

        // 4. Validate against static policies
        let policy_result = self
            .policy_validator
            .validate(&inputs.workflow_capabilities, &runtime.policies);
        result.merge(policy_result);

        result
    }

    /// Quick check: can the workflow execute?
    ///
    /// # Returns
    /// `true` if the workflow can execute (no blocking errors)
    pub fn can_execute(
        &self,
        workflow: &Workflow,
        inputs: &AggregateVerificationInputs,
        runtime: &RuntimeContext,
    ) -> bool {
        self.aggregate(workflow, inputs, runtime).can_execute()
    }
}

impl Default for VerificationAggregator {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Aggregator Tests (TASK-119)
    // =========================================================================

    mod aggregator_tests {
        use super::*;
        use ash_core::Effect;

        fn valid_workflow() -> Workflow {
            Workflow::Done {
                span: ash_parser::token::Span::default(),
            }
        }

        #[test]
        fn test_all_checks_pass() {
            let aggregator = VerificationAggregator::new();

            let workflow = valid_workflow();

            let workflow_capabilities = WorkflowCapabilities {
                observes: vec![(Name::from("sensor"), Name::from("temp"))],
                ..Default::default()
            };
            let inputs = AggregateVerificationInputs::new(
                workflow_capabilities,
                ObligationRequirements::new().require_observe("sensor", "temp"),
            );

            let runtime = RuntimeContext::new(Effect::Operational)
                .with_capabilities({
                    let mut r = CapabilitySchemaRegistry::new();
                    r.register(
                        Name::from("sensor"),
                        Name::from("temp"),
                        CapabilitySchema::read_only(Type::Int),
                    );
                    r
                })
                .with_obligations(
                    RuntimeObligations::new().with_obligation(
                        Obligation::new("monitor", "read temp")
                            .with_observe_capability("sensor", "temp"),
                    ),
                );

            let result = aggregator.aggregate(&workflow, &inputs, &runtime);
            assert!(result.can_execute());
            assert!(result.errors.is_empty());
        }

        #[test]
        fn test_with_errors() {
            let aggregator = VerificationAggregator::new();

            let workflow = valid_workflow();

            let workflow_capabilities = WorkflowCapabilities {
                observes: vec![(Name::from("missing"), Name::from("cap"))],
                ..Default::default()
            };
            let inputs = AggregateVerificationInputs::new(
                workflow_capabilities,
                ObligationRequirements::new().require_observe("missing", "cap"),
            );

            let runtime = RuntimeContext::new(Effect::Operational);

            let result = aggregator.aggregate(&workflow, &inputs, &runtime);
            assert!(!result.can_execute());
            assert!(!result.errors.is_empty());
        }

        #[test]
        fn test_effect_too_high() {
            let aggregator = VerificationAggregator::new();

            // Act workflow has Operational effect
            let workflow = Workflow::Act {
                action: ash_parser::surface::ActionRef {
                    name: "test".into(),
                    args: vec![],
                },
                guard: None,
                span: ash_parser::token::Span::default(),
            };

            let inputs = AggregateVerificationInputs::new(
                WorkflowCapabilities::new(),
                ObligationRequirements::new(),
            );
            let runtime = RuntimeContext::new(Effect::Epistemic);

            let result = aggregator.aggregate(&workflow, &inputs, &runtime);
            assert!(!result.can_execute());
            assert!(
                result
                    .errors
                    .iter()
                    .any(|e| matches!(e, VerificationError::EffectTooHigh { .. }))
            );
        }

        #[test]
        fn test_with_warnings_only() {
            let aggregator = VerificationAggregator::new();

            let workflow = valid_workflow();

            let workflow_capabilities = WorkflowCapabilities {
                sets: vec![(Name::from("hvac"), Name::from("target"))],
                ..Default::default()
            };
            let inputs = AggregateVerificationInputs::new(
                workflow_capabilities,
                ObligationRequirements::new().require_set("hvac", "target"),
            );

            let runtime = RuntimeContext::new(Effect::Operational)
                .with_capabilities({
                    let mut r = CapabilitySchemaRegistry::new();
                    r.register(
                        Name::from("hvac"),
                        Name::from("target"),
                        CapabilitySchema::write_only(Type::Int),
                    );
                    r
                })
                .with_obligations(RuntimeObligations::new().with_obligation(
                    Obligation::new("control", "set hvac").with_set_capability("hvac", "target"),
                ))
                .with_policies(vec![StaticPolicy::new("approval_required").decision(
                    PolicyDecisionType::RequiresApproval {
                        role: Role::new("admin"),
                    },
                )]);

            let result = aggregator.aggregate(&workflow, &inputs, &runtime);
            assert!(result.can_execute()); // Warnings don't block
            assert!(!result.warnings.is_empty());
        }

        #[test]
        fn test_can_execute_convenience_method() {
            let aggregator = VerificationAggregator::new();

            let workflow = valid_workflow();
            let inputs = AggregateVerificationInputs::new(
                WorkflowCapabilities::new(),
                ObligationRequirements::new(),
            );
            let runtime = RuntimeContext::new(Effect::Operational);

            assert!(aggregator.can_execute(&workflow, &inputs, &runtime));
        }

        #[test]
        fn test_multiple_errors() {
            let aggregator = VerificationAggregator::new();

            // Act workflow has Operational effect - too high for Epistemic runtime
            let workflow = Workflow::Act {
                action: ash_parser::surface::ActionRef {
                    name: "test".into(),
                    args: vec![],
                },
                guard: None,
                span: ash_parser::token::Span::default(),
            };

            let workflow_capabilities = WorkflowCapabilities {
                observes: vec![(Name::from("missing"), Name::from("cap"))],
                sets: vec![(Name::from("also"), Name::from("missing"))],
                ..Default::default()
            };
            let inputs = AggregateVerificationInputs::new(
                workflow_capabilities,
                ObligationRequirements::new()
                    .require_observe("missing", "cap")
                    .require_set("also", "missing"),
            );

            let runtime = RuntimeContext::new(Effect::Epistemic);

            let result = aggregator.aggregate(&workflow, &inputs, &runtime);
            assert!(!result.can_execute());
            // Should have: 2 missing runtime capabilities, 2 missing required runtime obligations,
            // and 1 effect-ceiling failure.
            assert_eq!(result.errors.len(), 5);
        }

        #[test]
        fn test_runtime_context_builder() {
            let ctx = RuntimeContext::new(Effect::Operational)
                .with_capabilities(CapabilitySchemaRegistry::new())
                .with_obligations(RuntimeObligations::new())
                .with_policies(vec![])
                .add_policy(StaticPolicy::new("test"));

            assert_eq!(ctx.max_effect, Effect::Operational);
            assert_eq!(ctx.policies.len(), 1);
        }

        #[test]
        fn test_aggregator_default() {
            let aggregator: VerificationAggregator = Default::default();
            let workflow = valid_workflow();
            let inputs = AggregateVerificationInputs::new(
                WorkflowCapabilities::new(),
                ObligationRequirements::new(),
            );
            let runtime = RuntimeContext::new(Effect::Operational);

            let result = aggregator.aggregate(&workflow, &inputs, &runtime);
            assert!(result.can_execute());
        }
    }

    // =========================================================================
    // Operation Verifier Tests (TASK-118)
    // =========================================================================

    mod operation_tests {
        use super::*;

        #[test]
        fn test_rate_limiter_window_expiration() {
            let op = CapabilityOperation::observe("sensor", "temp");

            // Create a rate limiter with a very short window
            let mut rate_limiter = RateLimiter::new(1, Duration::from_millis(50));

            // First request should succeed
            assert!(!rate_limiter.would_exceed(&op));

            // Second request should fail (limit reached)
            assert!(rate_limiter.would_exceed(&op));

            // Wait for the window to expire
            std::thread::sleep(Duration::from_millis(60));

            // After window expiration, old requests should be cleaned up
            // and new requests should succeed
            let op2 = CapabilityOperation::observe("sensor", "temp");
            assert!(!rate_limiter.would_exceed(&op2));
        }

        #[test]
        fn test_operation_verifier_default() {
            let verifier: OperationVerifier = Default::default();
            let _ = verifier;
        }

        #[test]
        fn test_direction_enum() {
            assert_ne!(Direction::Input, Direction::Output);
            assert_eq!(Direction::Input, Direction::Input);
            assert_eq!(Direction::Output, Direction::Output);
        }

        #[test]
        fn test_operation_result_variants() {
            let proceed = OperationResult::Proceed;
            assert_eq!(proceed, OperationResult::Proceed);

            let denied = OperationResult::Denied {
                policy: "test".to_string(),
            };
            assert!(matches!(denied, OperationResult::Denied { .. }));

            let requires_approval = OperationResult::RequiresApproval {
                role: Role::new("admin"),
            };
            assert!(matches!(
                requires_approval,
                OperationResult::RequiresApproval { .. }
            ));

            let transformed = OperationResult::Transformed {
                transformation: "encrypt".to_string(),
            };
            assert!(matches!(transformed, OperationResult::Transformed { .. }));
        }

        #[test]
        fn test_operation_error_display() {
            let err = OperationError::CapabilityUnavailable("sensor:temp".to_string());
            assert!(format!("{err}").contains("capability unavailable"));

            let err = OperationError::ModeNotSupported("write".to_string());
            assert!(format!("{err}").contains("operation mode not supported"));

            let err = OperationError::RateLimitExceeded("sensor:temp".to_string());
            assert!(format!("{err}").contains("rate limit exceeded"));

            let err = OperationError::PolicyDenied("test_policy".to_string());
            assert!(format!("{err}").contains("policy denied"));
        }
    }
}
