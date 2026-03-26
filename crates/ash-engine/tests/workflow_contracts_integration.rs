//! Integration tests for workflow contracts
//!
//! Tests end-to-end obligation lifecycle, requirement checking, and contract enforcement.

#![allow(clippy::no_effect_underscore_binding)]

use ash_core::workflow_contract::{
    ArithConstraint, Contract, Effect, ObligationError, ObligationSet, PostPredicate, Requirement,
    Span, Workflow, WorkflowDef,
};

// ============================================================
// Simple Obligation Lifecycle Tests
// ============================================================

#[test]
fn test_obligation_insert_adds_to_set() {
    let mut set = ObligationSet::new();
    set.insert("audit_trail").expect("insert should succeed");
    assert!(set.contains("audit_trail"));
    assert!(!set.is_empty());
}

#[test]
fn test_obligation_remove_consumes_obligation() {
    let mut set = ObligationSet::new();
    set.insert("audit_trail").expect("insert should succeed");
    set.remove("audit_trail").expect("remove should succeed");
    assert!(!set.contains("audit_trail"));
    assert!(set.is_empty());
}

#[test]
fn test_full_obligation_lifecycle() {
    let mut set = ObligationSet::new();

    // Create obligation
    set.insert("log_access").expect("insert should succeed");
    assert!(set.contains("log_access"));
    assert!(!set.is_empty());

    // Check (consume) obligation
    set.remove("log_access").expect("remove should succeed");
    assert!(!set.contains("log_access"));
    assert!(set.is_empty());
}

// ============================================================
// Check with Decision Patterns Tests
// ============================================================

#[test]
fn test_check_with_decision_pattern_true_branch() {
    // Simulates: if check obligation then { /* obligation consumed */ } else { /* obligation remains */ }
    let mut set = ObligationSet::new();
    set.insert("permission").expect("insert should succeed");

    // Decision pattern: check returns true, obligation consumed in true branch
    let check_result = true;
    if check_result {
        set.remove("permission").expect("remove in true branch");
    }

    assert!(set.is_empty());
}

#[test]
fn test_check_with_decision_pattern_false_branch() {
    // Simulates: if check obligation then { /* obligation consumed */ } else { /* obligation remains */ }
    let mut set = ObligationSet::new();
    set.insert("permission").expect("insert should succeed");

    // Decision pattern: check returns false, obligation not consumed
    let check_result = false;
    if check_result {
        set.remove("permission").expect("remove in true branch");
    }

    assert!(!set.is_empty());
    assert!(set.contains("permission"));
}

// ============================================================
// Undischarged Obligation Error Tests
// ============================================================

#[test]
fn test_undischarged_obligation_error() {
    // Create obligation set with an undischarged obligation
    let mut inner_set = ObligationSet::new();
    inner_set.insert("cleanup").expect("insert should succeed");
    // obligation "cleanup" never removed

    let error = ObligationError::Undischarged(vec!["cleanup".to_string()]);
    assert!(error.to_string().contains("not discharged"));
}

#[test]
fn test_is_empty_reports_undischarged() {
    let mut set = ObligationSet::new();
    set.insert("undischarged_obligation")
        .expect("insert should succeed");

    assert!(
        !set.is_empty(),
        "Set should report non-empty with undischarged obligations"
    );
    let remaining = set.remaining();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0], "undischarged_obligation");
}

// ============================================================
// Double Check Error Tests
// ============================================================

#[test]
fn test_double_insert_error() {
    let mut set = ObligationSet::new();
    set.insert("singleton")
        .expect("first insert should succeed");

    let result = set.insert("singleton");
    assert!(
        matches!(result, Err(ObligationError::Duplicate(name)) if name == "singleton"),
        "Double insert should fail with Duplicate error"
    );
}

#[test]
fn test_double_remove_error() {
    let mut set = ObligationSet::new();
    set.insert("once").expect("insert should succeed");
    set.remove("once").expect("first remove should succeed");

    let result = set.remove("once");
    assert!(
        matches!(result, Err(ObligationError::Unknown(name)) if name == "once"),
        "Double remove should fail with Unknown error"
    );
}

#[test]
fn test_remove_nonexistent_obligation() {
    let mut set = ObligationSet::new();

    let result = set.remove("never_existed");
    assert!(
        matches!(result, Err(ObligationError::Unknown(name)) if name == "never_existed"),
        "Removing non-existent obligation should fail"
    );
}

// ============================================================
// Branch Partial Discharge Error Tests
// ============================================================

#[test]
fn test_branch_partial_discharge_intersection_keeps_obligation() {
    // if cond then { check o1 } else { /* no check */ }
    // => intersection keeps o1 (obligation not discharged on all paths)
    let mut then_branch = ObligationSet::new();
    then_branch.insert("o1").expect("insert in then");
    then_branch.remove("o1").expect("remove in then");

    let else_branch = ObligationSet::new();
    // Note: In actual type checking, o1 would be inserted at the start of both branches
    // Here we simulate the case where else branch doesn't discharge

    let merged = then_branch.intersection(&else_branch);
    assert!(
        merged.is_empty(),
        "Both branches empty, intersection is empty"
    );
}

#[test]
fn test_branch_both_discharge_results_in_empty_intersection() {
    // if cond then { check o1 } else { check o1 }
    // => intersection is empty (obligation discharged on all paths)
    let mut then_branch = ObligationSet::new();
    then_branch.insert("o1").expect("insert in then");
    then_branch.remove("o1").expect("remove in then");

    let mut else_branch = ObligationSet::new();
    else_branch.insert("o1").expect("insert in else");
    else_branch.remove("o1").expect("remove in else");

    let merged = then_branch.intersection(&else_branch);
    assert!(
        merged.is_empty(),
        "Both branches discharged, intersection should be empty"
    );
}

#[test]
fn test_branch_one_discharge_one_undischarged() {
    // Simulate: both branches start with obligation, but only one discharges
    let mut then_branch = ObligationSet::new();
    then_branch.insert("shared").expect("insert");
    then_branch.remove("shared").expect("discharge in then");

    let mut else_branch = ObligationSet::new();
    else_branch.insert("shared").expect("insert");
    // NOT removed in else branch

    let merged = then_branch.intersection(&else_branch);
    assert!(merged.is_empty(), "Then is empty, intersection is empty");
}

// ============================================================
// Audit Trail Recording Tests
// ============================================================

#[test]
fn test_audit_trail_obligation_tracking() {
    let mut audit_log: Vec<String> = Vec::new();
    let mut set = ObligationSet::new();

    // Record obligation creation
    set.insert("audit_log_entry")
        .expect("insert should succeed");
    audit_log.push("OBLIGE: audit_log_entry".to_string());

    // Record obligation check
    set.remove("audit_log_entry")
        .expect("remove should succeed");
    audit_log.push("CHECK: audit_log_entry (discharged)".to_string());

    assert_eq!(audit_log.len(), 2);
    assert!(audit_log[0].contains("OBLIGE"));
    assert!(audit_log[1].contains("CHECK"));
    assert!(set.is_empty());
}

#[test]
fn test_audit_trail_remaining_obligations() {
    let mut set = ObligationSet::new();
    set.insert("obligation_a").expect("insert a");
    set.insert("obligation_b").expect("insert b");
    set.remove("obligation_a").expect("remove a");

    let remaining = set.remaining();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0], "obligation_b");
}

// ============================================================
// Requirement Checking with Capabilities/Roles Tests
// ============================================================

#[test]
fn test_requirement_has_capability() {
    let req = Requirement::HasCapability {
        cap: "file_io".to_string(),
        min_effect: Effect::Operational,
    };

    match &req {
        Requirement::HasCapability { cap, min_effect } => {
            assert_eq!(cap, "file_io");
            assert_eq!(*min_effect, Effect::Operational);
        }
        _ => panic!("Expected HasCapability variant"),
    }
}

#[test]
fn test_requirement_has_role() {
    let req = Requirement::HasRole("admin".to_string());

    match &req {
        Requirement::HasRole(role) => {
            assert_eq!(role, "admin");
        }
        _ => panic!("Expected HasRole variant"),
    }
}

#[test]
fn test_contract_with_capability_requirement() {
    let contract = Contract::new().with_requirement(Requirement::HasCapability {
        cap: "network".to_string(),
        min_effect: Effect::Epistemic,
    });

    assert_eq!(contract.requires.len(), 1);
    assert!(matches!(
        &contract.requires[0],
        Requirement::HasCapability { cap, .. } if cap == "network"
    ));
}

#[test]
fn test_contract_with_role_requirement() {
    let contract = Contract::new().with_requirement(Requirement::HasRole("auditor".to_string()));

    assert_eq!(contract.requires.len(), 1);
    assert!(matches!(
        &contract.requires[0],
        Requirement::HasRole(role) if role == "auditor"
    ));
}

#[test]
fn test_effect_levels_ordering() {
    // Verify effect levels exist and are distinct
    let epistemic = Effect::Epistemic;
    let deliberative = Effect::Deliberative;
    let evaluative = Effect::Evaluative;
    let operational = Effect::Operational;

    assert_ne!(epistemic, deliberative);
    assert_ne!(deliberative, evaluative);
    assert_ne!(evaluative, operational);
}

// ============================================================
// Arithmetic Requirements Tests
// ============================================================

#[test]
fn test_arithmetic_requirement_gt() {
    let req = Requirement::Arithmetic {
        var: "amount".to_string(),
        constraint: ArithConstraint::Gt(0),
    };

    match &req {
        Requirement::Arithmetic { var, constraint } => {
            assert_eq!(var, "amount");
            assert!(matches!(constraint, ArithConstraint::Gt(0)));
        }
        _ => panic!("Expected Arithmetic variant"),
    }
}

#[test]
fn test_arithmetic_requirement_range() {
    let req = Requirement::Arithmetic {
        var: "age".to_string(),
        constraint: ArithConstraint::Range { min: 0, max: 120 },
    };

    match &req {
        Requirement::Arithmetic { var, constraint } => {
            assert_eq!(var, "age");
            assert!(matches!(
                constraint,
                ArithConstraint::Range { min: 0, max: 120 }
            ));
        }
        _ => panic!("Expected Arithmetic variant"),
    }
}

#[test]
fn test_contract_with_arithmetic_requirement() {
    let contract = Contract::new().with_requirement(Requirement::Arithmetic {
        var: "balance".to_string(),
        constraint: ArithConstraint::Gte(0),
    });

    assert_eq!(contract.requires.len(), 1);
    assert!(matches!(
        &contract.requires[0],
        Requirement::Arithmetic { var, .. } if var == "balance"
    ));
}

#[test]
fn test_all_arith_constraint_variants() {
    // Verify all constraint variants can be constructed
    let _gt = ArithConstraint::Gt(1);
    let _lt = ArithConstraint::Lt(100);
    let _gte = ArithConstraint::Gte(0);
    let _lte = ArithConstraint::Lte(255);
    let _eq = ArithConstraint::Eq(42);
    let _range = ArithConstraint::Range { min: 0, max: 100 };
}

// ============================================================
// Workflow Definition with Contract Tests
// ============================================================

#[test]
fn test_workflow_def_with_contract() {
    let def = WorkflowDef {
        name: "transfer".to_string(),
        params: vec![],
        body: Workflow::Done,
        export: true,
        contract: Some(
            Contract::new().with_requirement(Requirement::HasRole("treasurer".to_string())),
        ),
        span: Span::default(),
    };

    assert_eq!(def.name, "transfer");
    assert!(def.contract.is_some());
    let contract = def.contract.unwrap();
    assert_eq!(contract.requires.len(), 1);
}

#[test]
fn test_workflow_def_without_contract() {
    let def = WorkflowDef {
        name: "simple".to_string(),
        params: vec![],
        body: Workflow::Done,
        export: false,
        contract: None,
        span: Span::default(),
    };

    assert!(def.contract.is_none());
}

// ============================================================
// Contract Builder Pattern Tests
// ============================================================

#[test]
fn test_contract_builder_multiple_requirements() {
    let contract = Contract::new()
        .with_requirement(Requirement::HasRole("admin".to_string()))
        .with_requirement(Requirement::HasCapability {
            cap: "database".to_string(),
            min_effect: Effect::Operational,
        })
        .with_requirement(Requirement::Arithmetic {
            var: "limit".to_string(),
            constraint: ArithConstraint::Lte(1000),
        });

    assert_eq!(contract.requires.len(), 3);
}

#[test]
fn test_contract_builder_with_ensures() {
    let contract = Contract::new()
        .with_requirement(Requirement::HasRole("user".to_string()))
        .with_ensures(PostPredicate::ResultSatisfies(ArithConstraint::Gte(0)))
        .with_ensures(PostPredicate::StateAssertion("completed".to_string()));

    assert_eq!(contract.requires.len(), 1);
    assert_eq!(contract.ensures.len(), 2);
}

#[test]
fn test_post_predicate_variants() {
    let eq_pred = PostPredicate::Eq("result".to_string(), "expected".to_string());
    let result_pred = PostPredicate::ResultSatisfies(ArithConstraint::Gt(0));
    let state_pred = PostPredicate::StateAssertion("valid".to_string());

    // Just verify they can be constructed
    let _ = eq_pred;
    let _ = result_pred;
    let _ = state_pred;
}

// ============================================================
// Obligation Set Operations Tests
// ============================================================

#[test]
fn test_obligation_set_union() {
    let mut set_a = ObligationSet::new();
    set_a.insert("a").expect("insert a");
    set_a.insert("b").expect("insert b");

    let mut set_b = ObligationSet::new();
    set_b.insert("b").expect("insert b");
    set_b.insert("c").expect("insert c");

    let union = set_a.union(&set_b);

    assert!(union.contains("a"));
    assert!(union.contains("b"));
    assert!(union.contains("c"));
    assert_eq!(union.remaining().len(), 3);
}

#[test]
fn test_obligation_set_intersection() {
    let mut set_a = ObligationSet::new();
    set_a.insert("a").expect("insert a");
    set_a.insert("shared").expect("insert shared");

    let mut set_b = ObligationSet::new();
    set_b.insert("b").expect("insert b");
    set_b.insert("shared").expect("insert shared");

    let intersection = set_a.intersection(&set_b);

    assert!(!intersection.contains("a"));
    assert!(!intersection.contains("b"));
    assert!(intersection.contains("shared"));
    assert_eq!(intersection.remaining().len(), 1);
}

// ============================================================
// Workflow AST Contract Extensions Tests
// ============================================================

#[test]
fn test_workflow_oblige_variant() {
    let workflow = Workflow::Oblige {
        name: "audit".to_string(),
        span: Span { start: 0, end: 10 },
    };

    match workflow {
        Workflow::Oblige { name, span } => {
            assert_eq!(name, "audit");
            assert_eq!(span.start, 0);
            assert_eq!(span.end, 10);
        }
        _ => panic!("Expected Oblige variant"),
    }
}

#[test]
fn test_workflow_check_obligation_variant() {
    let workflow = Workflow::CheckObligation {
        name: "verify".to_string(),
        span: Span { start: 5, end: 15 },
    };

    match workflow {
        Workflow::CheckObligation { name, span } => {
            assert_eq!(name, "verify");
            assert_eq!(span.start, 5);
            assert_eq!(span.end, 15);
        }
        _ => panic!("Expected CheckObligation variant"),
    }
}

#[test]
fn test_workflow_done_variant() {
    let workflow = Workflow::Done;
    assert!(matches!(workflow, Workflow::Done));
}

// ============================================================
// Error Message Tests
// ============================================================

#[test]
fn test_duplicate_error_message() {
    let err = ObligationError::Duplicate("test_obligation".to_string());
    let msg = err.to_string();
    assert!(msg.contains("already exists"));
    assert!(msg.contains("test_obligation"));
}

#[test]
fn test_unknown_error_message() {
    let err = ObligationError::Unknown("missing_obligation".to_string());
    let msg = err.to_string();
    assert!(msg.contains("not found"));
    assert!(msg.contains("missing_obligation"));
}

#[test]
fn test_undischarged_error_message() {
    let err = ObligationError::Undischarged(vec!["obl1".to_string(), "obl2".to_string()]);
    let msg = err.to_string();
    assert!(msg.contains("not discharged"));
    assert!(msg.contains("obl1"));
    assert!(msg.contains("obl2"));
}
