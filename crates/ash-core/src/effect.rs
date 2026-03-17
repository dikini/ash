//! Effect lattice for tracking computational power
//!
//! The effect system forms a lattice:
//!     Epistemic < Deliberative < Evaluative < Operational

use serde::{Deserialize, Serialize};

/// Effect levels in the Sharo Core language
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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

#[cfg(test)]
mod tests {
    use super::Effect;
    use proptest::prelude::*;

    // Generate arbitrary Effect values uniformly
    prop_compose! {
        fn arb_effect()(
            n in 0u8..4
        ) -> Effect {
            match n {
                0 => Effect::Epistemic,
                1 => Effect::Deliberative,
                2 => Effect::Evaluative,
                _ => Effect::Operational,
            }
        }
    }

    proptest! {
        // ============================================================
        // Associativity Properties
        // ============================================================

        /// Join is associative: join(a, join(b, c)) == join(join(a, b), c)
        #[test]
        fn prop_join_associative(a in arb_effect(), b in arb_effect(), c in arb_effect()) {
            let left = a.join(b).join(c);
            let right = a.join(b.join(c));
            prop_assert_eq!(left, right);
        }

        /// Meet is associative: meet(a, meet(b, c)) == meet(meet(a, b), c)
        #[test]
        fn prop_meet_associative(a in arb_effect(), b in arb_effect(), c in arb_effect()) {
            let left = a.meet(b).meet(c);
            let right = a.meet(b.meet(c));
            prop_assert_eq!(left, right);
        }

        // ============================================================
        // Commutativity Properties
        // ============================================================

        /// Join is commutative: join(a, b) == join(b, a)
        #[test]
        fn prop_join_commutative(a in arb_effect(), b in arb_effect()) {
            prop_assert_eq!(a.join(b), b.join(a));
        }

        /// Meet is commutative: meet(a, b) == meet(b, a)
        #[test]
        fn prop_meet_commutative(a in arb_effect(), b in arb_effect()) {
            prop_assert_eq!(a.meet(b), b.meet(a));
        }

        // ============================================================
        // Idempotence Properties
        // ============================================================

        /// Join is idempotent: join(a, a) == a
        #[test]
        fn prop_join_idempotent(a in arb_effect()) {
            prop_assert_eq!(a.join(a), a);
        }

        /// Meet is idempotent: meet(a, a) == a
        #[test]
        fn prop_meet_idempotent(a in arb_effect()) {
            prop_assert_eq!(a.meet(a), a);
        }

        // ============================================================
        // Absorption Properties
        // ============================================================

        /// Meet absorbs join: meet(a, join(a, b)) == a
        #[test]
        fn prop_meet_absorbs_join(a in arb_effect(), b in arb_effect()) {
            prop_assert_eq!(a.meet(a.join(b)), a);
        }

        /// Join absorbs meet: join(a, meet(a, b)) == a
        #[test]
        fn prop_join_absorbs_meet(a in arb_effect(), b in arb_effect()) {
            prop_assert_eq!(a.join(a.meet(b)), a);
        }

        // ============================================================
        // Identity Element Properties
        // ============================================================

        /// Epistemic is the bottom element (identity for join):
        /// join(Epistemic, a) == a
        #[test]
        fn prop_epistemic_join_identity(a in arb_effect()) {
            prop_assert_eq!(Effect::Epistemic.join(a), a);
        }

        /// Operational is the top element (identity for meet):
        /// meet(Operational, a) == a
        #[test]
        fn prop_operational_meet_identity(a in arb_effect()) {
            prop_assert_eq!(Effect::Operational.meet(a), a);
        }

        /// Epistemic is the absorbing element for meet:
        /// meet(Epistemic, a) == Epistemic
        #[test]
        fn prop_epistemic_meet_absorbing(a in arb_effect()) {
            prop_assert_eq!(Effect::Epistemic.meet(a), Effect::Epistemic);
        }

        /// Operational is the absorbing element for join:
        /// join(Operational, a) == Operational
        #[test]
        fn prop_operational_join_absorbing(a in arb_effect()) {
            prop_assert_eq!(Effect::Operational.join(a), Effect::Operational);
        }

        // ============================================================
        // Partial Order Consistency
        // ============================================================

        /// Order consistency via join: (a <= b) == (join(a, b) == b)
        /// The join equals b if and only if a is less than or equal to b
        #[test]
        fn prop_order_consistency_via_join(a in arb_effect(), b in arb_effect()) {
            let join_equals_b = a.join(b) == b;
            let a_leq_b = a <= b;
            prop_assert_eq!(join_equals_b, a_leq_b);
        }

        /// Order consistency via meet: (a <= b) == (meet(a, b) == a)
        /// The meet equals a if and only if a is less than or equal to b
        #[test]
        fn prop_order_consistency_via_meet(a in arb_effect(), b in arb_effect()) {
            let meet_equals_a = a.meet(b) == a;
            let a_leq_b = a <= b;
            prop_assert_eq!(meet_equals_a, a_leq_b);
        }

        // ============================================================
        // At Least Consistency
        // ============================================================

        /// at_least is consistent with partial order: a.at_least(b) == (a >= b)
        #[test]
        fn prop_at_least_consistency(a in arb_effect(), b in arb_effect()) {
            prop_assert_eq!(a.at_least(b), a >= b);
        }

        // ============================================================
        // Lattice Ordering Properties
        // ============================================================

        /// The lattice ordering is consistent with the enum discriminant ordering
        /// For all effects, Epistemic <= a <= Operational
        #[test]
        fn prop_effect_bounds(a in arb_effect()) {
            prop_assert!(Effect::Epistemic <= a);
            prop_assert!(a <= Effect::Operational);
        }

        /// Join produces an upper bound: a <= join(a, b) and b <= join(a, b)
        #[test]
        fn prop_join_is_upper_bound(a in arb_effect(), b in arb_effect()) {
            let join = a.join(b);
            prop_assert!(a <= join);
            prop_assert!(b <= join);
        }

        /// Meet produces a lower bound: meet(a, b) <= a and meet(a, b) <= b
        #[test]
        fn prop_meet_is_lower_bound(a in arb_effect(), b in arb_effect()) {
            let meet = a.meet(b);
            prop_assert!(meet <= a);
            prop_assert!(meet <= b);
        }
    }
}
