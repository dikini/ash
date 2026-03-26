//! Role runtime context for authority and obligation tracking
//!
//! Provides runtime enforcement of role authority and obligations per SPEC-019.
//!
//! # Example
//!
//! ```
//! use ash_core::{Role, Capability, Effect, RoleObligationRef};
//! use ash_interp::role_context::RoleContext;
//!
//! let role = Role {
//!     name: "admin".to_string(),
//!     authority: vec![Capability {
//!         name: "sensor".to_string(),
//!         effect: Effect::Epistemic,
//!         constraints: vec![],
//!     }],
//!     obligations: vec![RoleObligationRef { name: "audit".to_string() }],
//! };
//!
//! let ctx = RoleContext::new(role);
//!
//! // Check authority
//! let cap = Capability {
//!     name: "sensor".to_string(),
//!     effect: Effect::Epistemic,
//!     constraints: vec![],
//! };
//! assert!(ctx.can_access(&cap));
//!
//! // Manage obligations
//! assert!(!ctx.is_discharged("audit"));
//! assert!(ctx.discharge("audit").is_ok());
//! assert!(ctx.is_discharged("audit"));
//! assert!(ctx.discharge("audit").is_err()); // Already discharged
//! ```

use ash_core::{Capability, Name, Role};
use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt;

/// Errors that can occur when discharging an obligation
#[derive(Debug, Clone, PartialEq)]
pub enum DischargeError {
    /// The obligation was not declared on the role
    UndeclaredObligation,
    /// The obligation was already discharged
    AlreadyDischarged,
}

impl fmt::Display for DischargeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DischargeError::UndeclaredObligation => {
                write!(f, "obligation not declared on role")
            }
            DischargeError::AlreadyDischarged => write!(f, "obligation already discharged"),
        }
    }
}

impl std::error::Error for DischargeError {}

/// Runtime context for role-based authority and obligation enforcement
///
/// Tracks the active role and discharged obligations using interior mutability
/// for linear discharge semantics.
#[derive(Debug, Clone)]
pub struct RoleContext {
    /// The active role with its authority and obligations
    pub active_role: Role,
    /// Set of obligations that have been discharged (linear semantics)
    discharged_obligations: RefCell<HashSet<Name>>,
}

impl RoleContext {
    /// Create a new RoleContext with the given active role
    pub fn new(active_role: Role) -> Self {
        Self {
            active_role,
            discharged_obligations: RefCell::new(HashSet::new()),
        }
    }

    /// Check if the active role has authority to access a capability
    ///
    /// Returns true if the capability is in the role's authority list.
    /// Authority matching is based on capability name equality.
    pub fn can_access(&self, capability: &Capability) -> bool {
        self.active_role
            .authority
            .iter()
            .any(|auth| auth.name == capability.name)
    }

    /// Check if an obligation has been discharged
    pub fn is_discharged(&self, obligation: &str) -> bool {
        self.discharged_obligations.borrow().contains(obligation)
    }

    /// Discharge an obligation (mark it as fulfilled)
    ///
    /// Returns `Ok(())` if the obligation was successfully discharged.
    /// Returns `Err(DischargeError)` if:
    /// - The obligation was not declared on the role
    /// - The obligation was already discharged
    ///
    /// This follows linear semantics: first discharge succeeds,
    /// subsequent discharges return `AlreadyDischarged` error.
    pub fn discharge(&self, obligation: &str) -> Result<(), DischargeError> {
        // Check if the obligation is declared on the role
        let is_declared = self
            .active_role
            .obligations
            .iter()
            .any(|obl| obl.name == obligation);

        if !is_declared {
            return Err(DischargeError::UndeclaredObligation);
        }

        let mut discharged = self.discharged_obligations.borrow_mut();
        if discharged.contains(obligation) {
            // Already discharged
            Err(DischargeError::AlreadyDischarged)
        } else {
            discharged.insert(obligation.to_string());
            Ok(())
        }
    }

    /// Check if all role obligations have been discharged
    pub fn all_discharged(&self) -> bool {
        let discharged = self.discharged_obligations.borrow();
        self.active_role
            .obligations
            .iter()
            .all(|obl| discharged.contains(&obl.name))
    }

    /// Get list of pending (non-discharged) obligations
    pub fn pending_obligations(&self) -> Vec<Name> {
        let discharged = self.discharged_obligations.borrow();
        self.active_role
            .obligations
            .iter()
            .filter(|obl| !discharged.contains(&obl.name))
            .map(|obl| obl.name.clone())
            .collect()
    }

    /// Get the set of discharged obligations (for testing/inspection)
    pub fn discharged_set(&self) -> HashSet<Name> {
        self.discharged_obligations.borrow().clone()
    }

    /// Reset all discharged obligations (for testing)
    pub fn reset_discharged(&self) {
        self.discharged_obligations.borrow_mut().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_core::{Capability, Effect, RoleObligationRef};

    fn create_test_role() -> Role {
        Role {
            name: "test_role".to_string(),
            authority: vec![
                Capability {
                    name: "sensor".to_string(),
                    effect: Effect::Epistemic,
                    constraints: vec![],
                },
                Capability {
                    name: "actuator".to_string(),
                    effect: Effect::Operational,
                    constraints: vec![],
                },
            ],
            obligations: vec![
                RoleObligationRef {
                    name: "audit".to_string(),
                },
                RoleObligationRef {
                    name: "log".to_string(),
                },
            ],
        }
    }

    #[test]
    fn test_can_access_with_authority() {
        let role = create_test_role();
        let ctx = RoleContext::new(role);

        let sensor_cap = Capability {
            name: "sensor".to_string(),
            effect: Effect::Epistemic,
            constraints: vec![],
        };
        assert!(ctx.can_access(&sensor_cap));

        let actuator_cap = Capability {
            name: "actuator".to_string(),
            effect: Effect::Operational,
            constraints: vec![],
        };
        assert!(ctx.can_access(&actuator_cap));
    }

    #[test]
    fn test_can_access_without_authority() {
        let role = create_test_role();
        let ctx = RoleContext::new(role);

        let unknown_cap = Capability {
            name: "unknown".to_string(),
            effect: Effect::Epistemic,
            constraints: vec![],
        };
        assert!(!ctx.can_access(&unknown_cap));
    }

    #[test]
    fn test_discharge_obligation() {
        let role = create_test_role();
        let ctx = RoleContext::new(role);

        // Initially not discharged
        assert!(!ctx.is_discharged("audit"));

        // First discharge succeeds
        assert!(ctx.discharge("audit").is_ok());
        assert!(ctx.is_discharged("audit"));

        // Second discharge returns AlreadyDischarged error
        assert_eq!(
            ctx.discharge("audit"),
            Err(DischargeError::AlreadyDischarged)
        );
    }

    #[test]
    fn test_all_discharged() {
        let role = create_test_role();
        let ctx = RoleContext::new(role);

        // Initially not all discharged
        assert!(!ctx.all_discharged());

        // Discharge first obligation
        ctx.discharge("audit").unwrap();
        assert!(!ctx.all_discharged());

        // Discharge second obligation
        ctx.discharge("log").unwrap();
        assert!(ctx.all_discharged());
    }

    #[test]
    fn test_pending_obligations() {
        let role = create_test_role();
        let ctx = RoleContext::new(role);

        // Both obligations pending initially
        let pending = ctx.pending_obligations();
        assert_eq!(pending.len(), 2);
        assert!(pending.contains(&"audit".to_string()));
        assert!(pending.contains(&"log".to_string()));

        // Discharge one
        ctx.discharge("audit").unwrap();
        let pending = ctx.pending_obligations();
        assert_eq!(pending.len(), 1);
        assert!(!pending.contains(&"audit".to_string()));
        assert!(pending.contains(&"log".to_string()));

        // Discharge all
        ctx.discharge("log").unwrap();
        let pending = ctx.pending_obligations();
        assert!(pending.is_empty());
    }

    #[test]
    fn test_discharge_unknown_obligation_fails() {
        let role = create_test_role();
        let ctx = RoleContext::new(role);

        // Discharging an obligation not declared on the role fails
        assert_eq!(
            ctx.discharge("unknown"),
            Err(DischargeError::UndeclaredObligation)
        );
        // Unknown obligation is not tracked
        assert!(!ctx.is_discharged("unknown"));
    }

    #[test]
    fn test_role_without_obligations() {
        let role = Role {
            name: "simple_role".to_string(),
            authority: vec![],
            obligations: vec![],
        };
        let ctx = RoleContext::new(role);

        assert!(ctx.all_discharged());
        assert!(ctx.pending_obligations().is_empty());
    }

    #[test]
    fn test_role_without_authority() {
        let role = Role {
            name: "powerless_role".to_string(),
            authority: vec![],
            obligations: vec![],
        };
        let ctx = RoleContext::new(role);

        let cap = Capability {
            name: "anything".to_string(),
            effect: Effect::Operational,
            constraints: vec![],
        };
        assert!(!ctx.can_access(&cap));
    }
}
