//! Effect lattice for tracking computational power
//!
//! The effect system forms a lattice:
//!     Epistemic < Deliberative < Evaluative < Operational

use std::cmp::Ordering;

/// Effect levels in the Sharo Core language
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Effect {
    /// Read-only operations (OBSERVE)
    Epistemic = 0,
    /// Analysis and planning (ORIENT)
    Deliberative = 1,
    /// Policy evaluation (DECIDE)
    Evaluative = 2,
    /// Side-effecting operations (ACT)
    Operational = 3,
}

impl Effect {
    /// Join (⊔) - least upper bound
    pub fn join(self, other: Effect) -> Effect {
        self.max(other)
    }

    /// Meet (⊓) - greatest lower bound  
    pub fn meet(self, other: Effect) -> Effect {
        self.min(other)
    }

    /// Check if this effect is at least as powerful as other
    pub fn at_least(self, other: Effect) -> bool {
        self >= other
    }
}

impl std::fmt::Display for Effect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Effect::Epistemic => write!(f, "epistemic"),
            Effect::Deliberative => write!(f, "deliberative"),
            Effect::Evaluative => write!(f, "evaluative"),
            Effect::Operational => write!(f, "operational"),
        }
    }
}
