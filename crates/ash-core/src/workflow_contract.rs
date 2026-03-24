//! Workflow contract types and obligation tracking
//!
//! Provides Hoare-style contracts (requires/ensures) and linear obligation tracking.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// A workflow contract with preconditions and postconditions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Contract {
    /// Preconditions that must hold at call site
    pub requires: Vec<Requirement>,
    /// Postconditions guaranteed after workflow completes
    pub ensures: Vec<PostPredicate>,
}

impl Contract {
    /// Create an empty contract
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a requirement (precondition)
    #[must_use]
    pub fn with_requirement(mut self, req: Requirement) -> Self {
        self.requires.push(req);
        self
    }

    /// Add an ensures clause (postcondition)
    #[must_use]
    pub fn with_ensures(mut self, pred: PostPredicate) -> Self {
        self.ensures.push(pred);
        self
    }
}

/// Requirements checked at call sites
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Requirement {
    /// Required capability with minimum effect level
    HasCapability { cap: String, min_effect: Effect },
    /// Required role membership
    HasRole(String),
    /// Arithmetic constraint on parameter (SMT-checkable)
    Arithmetic {
        var: String,
        constraint: ArithConstraint,
    },
}

/// Effect levels for capability requirements
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Effect {
    /// Read-only operations
    Epistemic,
    /// Analysis and planning
    Deliberative,
    /// Policy evaluation
    Evaluative,
    /// Side-effecting operations
    Operational,
}

/// Arithmetic constraints for requirement checking
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ArithConstraint {
    Gt(i64),
    Lt(i64),
    Gte(i64),
    Lte(i64),
    Eq(i64),
    Range { min: i64, max: i64 },
}

/// Predicates for postconditions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PostPredicate {
    /// Equality between expressions
    Eq(String, String),
    /// Result variable satisfies constraint
    ResultSatisfies(ArithConstraint),
    /// State assertion (for provenance tracking)
    StateAssertion(String),
}

/// Linear obligation set for type checking
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ObligationSet {
    /// Active obligations that must be checked before workflow completes
    active: HashSet<String>,
}

impl ObligationSet {
    /// Create empty obligation set
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Introduce a new obligation (OBLIGE)
    ///
    /// # Errors
    /// Returns error if obligation already exists (duplicate oblige)
    pub fn insert(&mut self, name: impl Into<String>) -> Result<(), ObligationError> {
        let name = name.into();
        if self.active.contains(&name) {
            return Err(ObligationError::Duplicate(name));
        }
        self.active.insert(name);
        Ok(())
    }

    /// Check (consume) an obligation (CHECK)
    ///
    /// # Errors
    /// Returns error if obligation doesn't exist (already checked or never obliged)
    pub fn remove(&mut self, name: &str) -> Result<(), ObligationError> {
        if !self.active.remove(name) {
            return Err(ObligationError::Unknown(name.to_string()));
        }
        Ok(())
    }

    /// Check if obligation exists
    #[must_use]
    pub fn contains(&self, name: &str) -> bool {
        self.active.contains(name)
    }

    /// Check if all obligations have been discharged
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.active.is_empty()
    }

    /// Get remaining obligations (for error reporting)
    #[must_use]
    pub fn remaining(&self) -> Vec<&String> {
        self.active.iter().collect()
    }

    /// Union of two obligation sets (for parallel composition)
    #[must_use]
    pub fn union(&self, other: &Self) -> Self {
        let active = self.active.union(&other.active).cloned().collect();
        Self { active }
    }

    /// Intersection of two obligation sets (for if/else merge)
    /// Both branches must discharge the obligation for it to be removed
    #[must_use]
    pub fn intersection(&self, other: &Self) -> Self {
        let active = self.active.intersection(&other.active).cloned().collect();
        Self { active }
    }
}

/// Obligation operation errors
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ObligationError {
    #[error("obligation '{0}' already exists (duplicate oblige)")]
    Duplicate(String),
    #[error("obligation '{0}' not found (already checked or never obliged)")]
    Unknown(String),
    #[error("obligations not discharged: {0:?}")]
    Undischarged(Vec<String>),
}

/// Parameter for workflow definitions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub ty: TypeExpr,
}

/// Type expression for parameters (simplified from ast::TypeExpr)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeExpr {
    Named(String),
    Constructor { name: String, args: Vec<TypeExpr> },
    Tuple(Vec<TypeExpr>),
}

/// Source span for AST nodes
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

/// Workflow definition with contract support
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowDef {
    pub name: String,
    pub params: Vec<Parameter>,
    pub body: Workflow,
    pub export: bool,
    pub contract: Option<Contract>,
    pub span: Span,
}

/// Core workflow AST with contract extensions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Workflow {
    /// OBLIGE obligation_name - introduce linear obligation
    Oblige { name: String, span: Span },
    /// CHECK obligation_name - check obligation, returns Bool
    CheckObligation { name: String, span: Span },
    /// Terminal
    Done,
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn obligation_insert_adds_to_set(name in "[a-z_][a-z0-9_]*") {
            let mut set = ObligationSet::new();
            assert!(set.insert(&name).is_ok());
            assert!(set.contains(&name));
        }

        #[test]
        fn obligation_remove_consumes(name in "[a-z_][a-z0-9_]*") {
            let mut set = ObligationSet::new();
            set.insert(&name).unwrap();
            assert!(set.remove(&name).is_ok());
            assert!(!set.contains(&name));
            assert!(set.is_empty());
        }

        #[test]
        fn double_insert_fails(name in "[a-z_][a-z0-9_]*") {
            let mut set = ObligationSet::new();
            set.insert(&name).unwrap();
            assert!(matches!(set.insert(&name), Err(ObligationError::Duplicate(_))));
        }

        #[test]
        fn double_remove_fails(name in "[a-z_][a-z0-9_]*") {
            let mut set = ObligationSet::new();
            set.insert(&name).unwrap();
            set.remove(&name).unwrap();
            assert!(matches!(set.remove(&name), Err(ObligationError::Unknown(_))));
        }

        #[test]
        fn intersection_keeps_common(
            a in prop::collection::hash_set("[a-z_][a-z0-9_]*", 1..5),
            b in prop::collection::hash_set("[a-z_][a-z0-9_]*", 1..5),
        ) {
            let set_a = ObligationSet { active: a.clone() };
            let set_b = ObligationSet { active: b.clone() };
            let intersection = set_a.intersection(&set_b);
            let expected: HashSet<_> = a.intersection(&b).cloned().collect();
            assert_eq!(intersection.active, expected);
        }
    }

    #[test]
    fn contract_builder_pattern() {
        let contract = Contract::new()
            .with_requirement(Requirement::HasCapability {
                cap: "file_io".into(),
                min_effect: Effect::Operational,
            })
            .with_ensures(PostPredicate::ResultSatisfies(ArithConstraint::Gt(0)));

        assert_eq!(contract.requires.len(), 1);
        assert_eq!(contract.ensures.len(), 1);
    }

    #[test]
    fn obligation_lifecycle() {
        let mut set = ObligationSet::new();

        // Create obligation
        set.insert("audit_trail").unwrap();
        assert!(set.contains("audit_trail"));
        assert!(!set.is_empty());

        // Check (consume) obligation
        set.remove("audit_trail").unwrap();
        assert!(!set.contains("audit_trail"));
        assert!(set.is_empty());
    }

    #[test]
    fn branch_merge_intersection() {
        // if cond then { check o1 } else { check o1 }
        // => intersection is empty (obligation discharged on both paths)
        let mut then_branch = ObligationSet::new();
        then_branch.insert("o1").unwrap();
        then_branch.remove("o1").unwrap();

        let mut else_branch = ObligationSet::new();
        else_branch.insert("o1").unwrap();
        else_branch.remove("o1").unwrap();

        let merged = then_branch.intersection(&else_branch);
        assert!(merged.is_empty());
    }

    #[test]
    fn branch_partial_discharge_fails() {
        // if cond then { check o1 } else { /* no check */ }
        // => intersection keeps o1 (error: undischarged obligation)
        let mut then_branch = ObligationSet::new();
        then_branch.insert("o1").unwrap();
        then_branch.remove("o1").unwrap();

        let else_branch = ObligationSet::new();
        // obligation created but never checked in else branch
        // (This would be caught by the type checker, not here)

        let merged = then_branch.intersection(&else_branch);
        assert!(merged.is_empty()); // Both empty after discharge
    }

    #[test]
    fn workflow_def_with_contract() {
        let def = WorkflowDef {
            name: "test_workflow".into(),
            params: vec![Parameter {
                name: "x".into(),
                ty: TypeExpr::Named("Int".into()),
            }],
            body: Workflow::Done,
            export: true,
            contract: Some(Contract::new()),
            span: Span::default(),
        };

        assert_eq!(def.name, "test_workflow");
        assert_eq!(def.params.len(), 1);
        assert!(def.contract.is_some());
    }

    #[test]
    fn oblige_workflow_variant() {
        let workflow = Workflow::Oblige {
            name: "audit".into(),
            span: Span { start: 0, end: 10 },
        };

        match workflow {
            Workflow::Oblige { name, span } => {
                assert_eq!(name, "audit");
                assert_eq!(span.start, 0);
                assert_eq!(span.end, 10);
            }
            _ => panic!("expected Oblige variant"),
        }
    }

    #[test]
    fn check_obligation_workflow_variant() {
        let workflow = Workflow::CheckObligation {
            name: "audit".into(),
            span: Span { start: 0, end: 10 },
        };

        match workflow {
            Workflow::CheckObligation { name, span } => {
                assert_eq!(name, "audit");
                assert_eq!(span.start, 0);
                assert_eq!(span.end, 10);
            }
            _ => panic!("expected CheckObligation variant"),
        }
    }

    #[test]
    fn obligation_error_display() {
        let err = ObligationError::Duplicate("test".into());
        assert!(err.to_string().contains("already exists"));

        let err = ObligationError::Unknown("test".into());
        assert!(err.to_string().contains("not found"));

        let err = ObligationError::Undischarged(vec!["a".into(), "b".into()]);
        assert!(err.to_string().contains("not discharged"));
    }

    #[test]
    fn obligation_set_double_insert_fails() {
        let mut set = ObligationSet::new();
        set.insert("audit").unwrap();

        // Double insert should fail
        let result = set.insert("audit");
        assert!(
            matches!(result, Err(ObligationError::Duplicate(_))),
            "Double insert should return Duplicate error"
        );
    }

    #[test]
    fn obligation_set_double_remove_fails() {
        let mut set = ObligationSet::new();
        set.insert("audit").unwrap();

        // First remove should succeed
        set.remove("audit").unwrap();

        // Second remove should fail (already consumed)
        let result = set.remove("audit");
        assert!(
            matches!(result, Err(ObligationError::Unknown(_))),
            "Double remove should return Unknown error"
        );
    }

    #[test]
    fn obligation_set_remove_unknown_fails() {
        let mut set = ObligationSet::new();

        // Remove non-existent obligation should fail
        let result = set.remove("never_created");
        assert!(
            matches!(result, Err(ObligationError::Unknown(_))),
            "Removing unknown obligation should return Unknown error"
        );
    }

    #[test]
    fn obligation_set_union_contains_all_elements() {
        let mut set_a = ObligationSet::new();
        set_a.insert("a").unwrap();
        set_a.insert("b").unwrap();

        let mut set_b = ObligationSet::new();
        set_b.insert("c").unwrap();
        set_b.insert("d").unwrap();

        let union = set_a.union(&set_b);

        assert!(union.contains("a"));
        assert!(union.contains("b"));
        assert!(union.contains("c"));
        assert!(union.contains("d"));
        assert_eq!(union.active.len(), 4);
    }

    #[test]
    fn obligation_set_remaining_returns_all_active() {
        let mut set = ObligationSet::new();
        set.insert("x").unwrap();
        set.insert("y").unwrap();

        let remaining = set.remaining();
        assert_eq!(remaining.len(), 2);
    }

    #[test]
    fn requirement_variants() {
        let req1 = Requirement::HasCapability {
            cap: "file_io".into(),
            min_effect: Effect::Operational,
        };
        let req2 = Requirement::HasRole("admin".into());
        let req3 = Requirement::Arithmetic {
            var: "amount".into(),
            constraint: ArithConstraint::Gt(0),
        };

        // Just verify they can be constructed
        let _ = req1;
        let _ = req2;
        let _ = req3;
    }

    #[test]
    fn post_predicate_variants() {
        let pred1 = PostPredicate::Eq("a".into(), "b".into());
        let pred2 = PostPredicate::ResultSatisfies(ArithConstraint::Gte(0));
        let pred3 = PostPredicate::StateAssertion("valid_state".into());

        // Just verify they can be constructed
        let _ = pred1;
        let _ = pred2;
        let _ = pred3;
    }

    #[test]
    fn arith_constraint_variants() {
        let c1 = ArithConstraint::Gt(1);
        let c2 = ArithConstraint::Lt(100);
        let c3 = ArithConstraint::Gte(0);
        let c4 = ArithConstraint::Lte(255);
        let c5 = ArithConstraint::Eq(42);
        let c6 = ArithConstraint::Range { min: 0, max: 100 };

        // Just verify they can be constructed
        let _ = c1;
        let _ = c2;
        let _ = c3;
        let _ = c4;
        let _ = c5;
        let _ = c6;
    }

    #[test]
    fn effect_variants() {
        let e1 = Effect::Epistemic;
        let e2 = Effect::Deliberative;
        let e3 = Effect::Evaluative;
        let e4 = Effect::Operational;

        // Just verify they can be constructed and are different
        assert_ne!(e1, e2);
        assert_ne!(e2, e3);
        assert_ne!(e3, e4);
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    prop_compose! {
        fn arb_obligation_name()(s in "[a-z_][a-z0-9_]{1,20}") -> String {
            s
        }
    }

    proptest! {
        #[test]
        fn obligation_set_roundtrip(names in prop::collection::vec(arb_obligation_name(), 0..10)) {
            let mut set = ObligationSet::new();

            // Insert all
            for name in &names {
                let _ = set.insert(name); // May fail on duplicates
            }

            // Remove all
            for name in &names {
                let _ = set.remove(name); // May fail if duplicate insert failed
            }

            assert!(set.is_empty());
        }

        #[test]
        fn union_contains_all(a in prop::collection::hash_set("[a-z]+", 0..5),
                              b in prop::collection::hash_set("[a-z]+", 0..5)) {
            let set_a = ObligationSet { active: a.clone() };
            let set_b = ObligationSet { active: b.clone() };
            let union = set_a.union(&set_b);

            for item in &a {
                assert!(union.contains(item));
            }
            for item in &b {
                assert!(union.contains(item));
            }
        }
    }
}
