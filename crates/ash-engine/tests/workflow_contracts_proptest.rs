//! Property-based tests for workflow contract obligation invariants
//!
//! These tests verify that obligation operations maintain expected invariants
//! across a wide range of inputs using proptest.

use ash_core::workflow_contract::{
    ArithConstraint, Contract, Effect, ObligationError, ObligationSet, PostPredicate, Requirement,
    Span, Workflow, WorkflowDef,
};
use proptest::prelude::*;

// ============================================================
// Obligation Set Invariant Properties
// ============================================================

proptest! {
    /// Property: Inserting an obligation makes it available in the set
    #[test]
    fn prop_insert_makes_obligation_available(name in "[a-z_][a-z0-9_]{1,30}") {
        let mut set = ObligationSet::new();
        prop_assume!(set.insert(&name).is_ok());
        prop_assert!(set.contains(&name));
    }

    /// Property: Removing an obligation makes it unavailable
    #[test]
    fn prop_remove_makes_obligation_unavailable(name in "[a-z_][a-z0-9_]{1,30}") {
        let mut set = ObligationSet::new();
        let _ = set.insert(&name);
        let _ = set.remove(&name);
        prop_assert!(!set.contains(&name));
    }

    /// Property: Double insert fails with Duplicate error
    #[test]
    fn prop_double_insert_fails(name in "[a-z_][a-z0-9_]{1,30}") {
        let mut set = ObligationSet::new();
        let _ = set.insert(&name);
        let result = set.insert(&name);
        prop_assert!(
            matches!(result, Err(ObligationError::Duplicate(n)) if n == name)
        );
    }

    /// Property: Double remove fails with Unknown error
    #[test]
    fn prop_double_remove_fails(name in "[a-z_][a-z0-9_]{1,30}") {
        let mut set = ObligationSet::new();
        let _ = set.insert(&name);
        let _ = set.remove(&name);
        let result = set.remove(&name);
        prop_assert!(
            matches!(result, Err(ObligationError::Unknown(n)) if n == name)
        );
    }

    /// Property: Remove on empty set fails
    #[test]
    fn prop_remove_from_empty_fails(name in "[a-z_][a-z0-9_]{1,30}") {
        let mut set = ObligationSet::new();
        let result = set.remove(&name);
        prop_assert!(
            matches!(result, Err(ObligationError::Unknown(n)) if n == name)
        );
    }

    /// Property: After insert+remove cycle, set returns to empty state
    #[test]
    fn prop_insert_remove_cycle_returns_to_empty(name in "[a-z_][a-z0-9_]{1,30}") {
        let mut set = ObligationSet::new();
        let _ = set.insert(&name);
        let _ = set.remove(&name);
        prop_assert!(set.is_empty());
    }

    /// Property: Multiple distinct obligations can coexist
    #[test]
    fn prop_multiple_obligations_coexist(
        names in prop::collection::hash_set("[a-z_][a-z0-9_]{1,20}", 1..10)
    ) {
        let mut set = ObligationSet::new();

        for name in &names {
            let _ = set.insert(name);
        }

        for name in &names {
            prop_assert!(set.contains(name));
        }

        prop_assert_eq!(set.remaining().len(), names.len());
    }

    /// Property: is_empty is true iff no obligations remain
    #[test]
    fn prop_is_empty_iff_no_obligations(
        names in prop::collection::vec("[a-z_][a-z0-9_]{1,20}", 0..10)
    ) {
        let mut set = ObligationSet::new();

        for name in &names {
            let _ = set.insert(name);
        }

        // Remove half of them
        for name in names.iter().take(names.len() / 2) {
            let _ = set.remove(name);
        }

        let remaining_count = set.remaining().len();
        prop_assert_eq!(set.is_empty(), remaining_count == 0);
    }
}

// ============================================================
// Set Operation Properties
// ============================================================

proptest! {
    /// Property: Union contains all elements from both sets
    #[test]
    fn prop_union_contains_all_elements(
        a in prop::collection::hash_set("[a-z]{1,10}", 0..10),
        b in prop::collection::hash_set("[a-z]{1,10}", 0..10)
    ) {
        let mut set_a = ObligationSet::new();
        let mut set_b = ObligationSet::new();
        for item in &a {
            let _ = set_a.insert(item);
        }
        for item in &b {
            let _ = set_b.insert(item);
        }
        let union = set_a.union(&set_b);

        for item in &a {
            prop_assert!(union.contains(item));
        }
        for item in &b {
            prop_assert!(union.contains(item));
        }
    }

    /// Property: Intersection only contains common elements
    #[test]
    fn prop_intersection_only_contains_common(
        a in prop::collection::hash_set("[a-z]{1,10}", 0..10),
        b in prop::collection::hash_set("[a-z]{1,10}", 0..10)
    ) {
        let mut set_a = ObligationSet::new();
        let mut set_b = ObligationSet::new();
        for item in &a {
            let _ = set_a.insert(item);
        }
        for item in &b {
            let _ = set_b.insert(item);
        }
        let intersection = set_a.intersection(&set_b);

        // All elements in intersection should be in both a and b
        for item in intersection.remaining() {
            prop_assert!(a.contains(item.as_str()));
            prop_assert!(b.contains(item.as_str()));
        }
    }

    /// Property: Intersection of set with itself equals the set
    #[test]
    fn prop_intersection_self_equals_self(
        a in prop::collection::hash_set("[a-z]{1,10}", 0..10)
    ) {
        let mut set_a = ObligationSet::new();
        for item in &a {
            let _ = set_a.insert(item);
        }
        let intersection = set_a.intersection(&set_a);

        // Check that intersection contains the same elements
        for item in &a {
            prop_assert!(intersection.contains(item));
        }
        prop_assert_eq!(intersection.remaining().len(), a.len());
    }

    /// Property: Union of set with itself equals the set
    #[test]
    fn prop_union_self_equals_self(
        a in prop::collection::hash_set("[a-z]{1,10}", 0..10)
    ) {
        let mut set_a = ObligationSet::new();
        for item in &a {
            let _ = set_a.insert(item);
        }
        let union = set_a.union(&set_a);

        // Check that union contains the same elements
        for item in &a {
            prop_assert!(union.contains(item));
        }
        prop_assert_eq!(union.remaining().len(), a.len());
    }

    /// Property: Union is commutative
    #[test]
    fn prop_union_commutative(
        a in prop::collection::hash_set("[a-z]{1,10}", 0..10),
        b in prop::collection::hash_set("[a-z]{1,10}", 0..10)
    ) {
        let mut set_a = ObligationSet::new();
        let mut set_b = ObligationSet::new();
        for item in &a {
            let _ = set_a.insert(item);
        }
        for item in &b {
            let _ = set_b.insert(item);
        }

        let union_ab = set_a.union(&set_b);
        let union_ba = set_b.union(&set_a);

        // Compare via remaining() since active is private
        let remaining_ab: std::collections::HashSet<_> = union_ab.remaining()
            .iter().map(|s| s.to_string()).collect();
        let remaining_ba: std::collections::HashSet<_> = union_ba.remaining()
            .iter().map(|s| s.to_string()).collect();
        prop_assert_eq!(remaining_ab, remaining_ba);
    }

    /// Property: Intersection is commutative
    #[test]
    fn prop_intersection_commutative(
        a in prop::collection::hash_set("[a-z]{1,10}", 0..10),
        b in prop::collection::hash_set("[a-z]{1,10}", 0..10)
    ) {
        let mut set_a = ObligationSet::new();
        let mut set_b = ObligationSet::new();
        for item in &a {
            let _ = set_a.insert(item);
        }
        for item in &b {
            let _ = set_b.insert(item);
        }

        let intersection_ab = set_a.intersection(&set_b);
        let intersection_ba = set_b.intersection(&set_a);

        // Compare via remaining() since active is private
        let remaining_ab: std::collections::HashSet<_> = intersection_ab.remaining()
            .iter().map(|s| s.to_string()).collect();
        let remaining_ba: std::collections::HashSet<_> = intersection_ba.remaining()
            .iter().map(|s| s.to_string()).collect();
        prop_assert_eq!(remaining_ab, remaining_ba);
    }

    /// Property: Empty set is identity for union
    #[test]
    fn prop_empty_union_identity(
        a in prop::collection::hash_set("[a-z]{1,10}", 0..10)
    ) {
        let mut set_a = ObligationSet::new();
        for item in &a {
            let _ = set_a.insert(item);
        }
        let empty = ObligationSet::new();

        let union = set_a.union(&empty);

        // Check union contains same elements as a
        for item in &a {
            prop_assert!(union.contains(item));
        }
        prop_assert_eq!(union.remaining().len(), a.len());
    }

    /// Property: Empty set is absorbing for intersection
    #[test]
    fn prop_empty_intersection_absorbing(
        a in prop::collection::hash_set("[a-z]{1,10}", 0..10)
    ) {
        let mut set_a = ObligationSet::new();
        for item in &a {
            let _ = set_a.insert(item);
        }
        let empty = ObligationSet::new();

        let intersection = set_a.intersection(&empty);

        prop_assert!(intersection.is_empty());
    }
}

// ============================================================
// Contract Builder Properties
// ============================================================

proptest! {
    /// Property: Contract builder accumulates requirements
    #[test]
    fn prop_contract_accumulates_requirements(
        count in 0usize..20
    ) {
        let mut contract = Contract::new();

        for i in 0..count {
            contract = contract.with_requirement(Requirement::HasRole(
                format!("role_{}", i)
            ));
        }

        prop_assert_eq!(contract.requires.len(), count);
    }

    /// Property: Contract builder accumulates ensures clauses
    #[test]
    fn prop_contract_accumulates_ensures(
        count in 0usize..20
    ) {
        let mut contract = Contract::new();

        for i in 0..count {
            contract = contract.with_ensures(PostPredicate::StateAssertion(
                format!("state_{}", i)
            ));
        }

        prop_assert_eq!(contract.ensures.len(), count);
    }
}

// ============================================================
// Obligation Lifecycle Roundtrip Properties
// ============================================================

proptest! {
    /// Property: Any sequence of inserts can be fully discharged
    #[test]
    fn prop_any_inserts_can_be_fully_discharged(
        names in prop::collection::vec("[a-z_][a-z0-9_]{1,20}", 0..20)
    ) {
        let mut set = ObligationSet::new();

        // Insert all (skipping duplicates)
        for name in &names {
            let _ = set.insert(name);
        }

        // Remove all
        for name in &names {
            let _ = set.remove(name);
        }

        prop_assert!(set.is_empty());
    }

    /// Property: remaining() returns exactly the undischarged obligations
    #[test]
    fn prop_remaining_returns_exactly_undischarged(
        names in prop::collection::vec("[a-z_][a-z0-9_]{1,20}", 0..15)
    ) {
        let mut set = ObligationSet::new();

        // Insert all
        for name in &names {
            let _ = set.insert(name);
        }

        // Remove only even-indexed items
        for (i, name) in names.iter().enumerate() {
            if i % 2 == 0 {
                let _ = set.remove(name);
            }
        }

        // Check that remaining only contains odd-indexed items
        let remaining: Vec<_> = set.remaining().iter().map(|s| s.to_string()).collect();
        let expected: Vec<_> = names.iter().enumerate()
            .filter(|(i, _)| i % 2 == 1)
            .map(|(_, n)| n.clone())
            .collect();

        for name in &expected {
            prop_assert!(remaining.contains(name));
        }
        prop_assert_eq!(remaining.len(), expected.len());
    }
}

// ============================================================
// Error Preservation Properties
// ============================================================

proptest! {
    /// Property: Duplicate error preserves obligation name
    #[test]
    fn prop_duplicate_error_preserves_name(name in "[a-z_][a-z0-9_]{1,30}") {
        let err = ObligationError::Duplicate(name.clone());
        if let ObligationError::Duplicate(found) = err {
            prop_assert_eq!(found, name);
        } else {
            prop_assert!(false, "Expected Duplicate variant");
        }
    }

    /// Property: Unknown error preserves obligation name
    #[test]
    fn prop_unknown_error_preserves_name(name in "[a-z_][a-z0-9_]{1,30}") {
        let err = ObligationError::Unknown(name.clone());
        if let ObligationError::Unknown(found) = err {
            prop_assert_eq!(found, name);
        } else {
            prop_assert!(false, "Expected Unknown variant");
        }
    }

    /// Property: Undischarged error preserves all obligation names
    #[test]
    fn prop_undischarged_error_preserves_names(
        names in prop::collection::vec("[a-z_][a-z0-9_]{1,20}", 0..10)
    ) {
        let err = ObligationError::Undischarged(names.clone());
        if let ObligationError::Undischarged(found) = err {
            prop_assert_eq!(found, names);
        } else {
            prop_assert!(false, "Expected Undischarged variant");
        }
    }
}

// ============================================================
// Type Safety Properties
// ============================================================

proptest! {
    /// Property: Contract with all requirement variants can be built
    #[test]
    fn prop_contract_with_all_requirement_types(
        cap_name in "[a-z_]{1,20}",
        role_name in "[a-z_]{1,20}",
        var_name in "[a-z_]{1,20}",
        constraint_val in i64::MIN..i64::MAX
    ) {
        let contract = Contract::new()
            .with_requirement(Requirement::HasCapability {
                cap: cap_name.clone(),
                min_effect: Effect::Epistemic,
            })
            .with_requirement(Requirement::HasRole(role_name.clone()))
            .with_requirement(Requirement::Arithmetic {
                var: var_name.clone(),
                constraint: ArithConstraint::Gt(constraint_val),
            });

        prop_assert_eq!(contract.requires.len(), 3);
    }

    /// Property: All effect levels can be used in requirements
    #[test]
    fn prop_all_effect_levels_work(_dummy in any::<u8>()) {
        let effects = vec![
            Effect::Epistemic,
            Effect::Deliberative,
            Effect::Evaluative,
            Effect::Operational,
        ];

        for effect in effects {
            let req = Requirement::HasCapability {
                cap: "test".to_string(),
                min_effect: effect,
            };
            // Just verify it can be constructed
            let _ = req;
        }

        prop_assert!(true);
    }

    /// Property: All arithmetic constraint variants can be constructed
    #[test]
    fn prop_all_arith_constraints_work(val in i64::MIN..i64::MAX) {
        let _gt = ArithConstraint::Gt(val);
        let _lt = ArithConstraint::Lt(val);
        let _gte = ArithConstraint::Gte(val);
        let _lte = ArithConstraint::Lte(val);
        let _eq = ArithConstraint::Eq(val);
        let _range = ArithConstraint::Range { min: val, max: val };

        prop_assert!(true);
    }

    /// Property: All post predicate variants can be constructed
    #[test]
    fn prop_all_post_predicate_work(
        s1 in "[a-z_]{1,20}",
        s2 in "[a-z_]{1,20}"
    ) {
        let _eq = PostPredicate::Eq(s1.clone(), s2.clone());
        let _result = PostPredicate::ResultSatisfies(ArithConstraint::Gt(0));
        let _state = PostPredicate::StateAssertion(s1.clone());

        prop_assert!(true);
    }
}

// ============================================================
// Workflow AST Properties
// ============================================================

proptest! {
    /// Property: Workflow::Oblige preserves name and span
    #[test]
    fn prop_oblige_preserves_fields(
        name in "[a-z_][a-z0-9_]{1,30}",
        start in 0usize..10000,
        end in 0usize..10000
    ) {
        let span = Span { start, end };
        let workflow = Workflow::Oblige {
            name: name.clone(),
            span,
        };

        if let Workflow::Oblige { name: n, span: s } = workflow {
            prop_assert_eq!(n, name);
            prop_assert_eq!(s.start, start);
            prop_assert_eq!(s.end, end);
        } else {
            prop_assert!(false, "Expected Oblige variant");
        }
    }

    /// Property: Workflow::CheckObligation preserves name and span
    #[test]
    fn prop_check_obligation_preserves_fields(
        name in "[a-z_][a-z0-9_]{1,30}",
        start in 0usize..10000,
        end in 0usize..10000
    ) {
        let span = Span { start, end };
        let workflow = Workflow::CheckObligation {
            name: name.clone(),
            span,
        };

        if let Workflow::CheckObligation { name: n, span: s } = workflow {
            prop_assert_eq!(n, name);
            prop_assert_eq!(s.start, start);
            prop_assert_eq!(s.end, end);
        } else {
            prop_assert!(false, "Expected CheckObligation variant");
        }
    }
}

// ============================================================
// WorkflowDef Properties
// ============================================================

proptest! {
    /// Property: WorkflowDef preserves all fields
    #[test]
    fn prop_workflow_def_preserves_fields(
        name in "[a-z_][a-z0-9_]{1,30}",
        export in any::<bool>()
    ) {
        let def = WorkflowDef {
            name: name.clone(),
            params: vec![],
            body: Workflow::Done,
            export,
            contract: None,
            span: Span::default(),
        };

        prop_assert_eq!(def.name, name);
        prop_assert_eq!(def.export, export);
    }

    /// Property: WorkflowDef with contract can be constructed
    #[test]
    fn prop_workflow_def_with_contract(
        name in "[a-z_][a-z0-9_]{1,30}",
        role in "[a-z_]{1,20}"
    ) {
        let def = WorkflowDef {
            name: name.clone(),
            params: vec![],
            body: Workflow::Done,
            export: true,
            contract: Some(Contract::new()
                .with_requirement(Requirement::HasRole(role))),
            span: Span::default(),
        };

        prop_assert!(def.contract.is_some());
        prop_assert_eq!(def.contract.unwrap().requires.len(), 1);
    }
}
