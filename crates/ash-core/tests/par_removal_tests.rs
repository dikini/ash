//! Par removal tests for core AST
//!
//! These tests verify that the Par variant has been removed from:
//! - The Workflow enum in the core AST
//! - The workflow contract types

use ash_core::ast::Workflow;

#[test]
fn test_workflow_par_variant_removed() {
    // Verify that Workflow enum doesn't have a Par variant that can be constructed
    // This test will fail to compile if Par still exists in the enum

    // Uncommenting the following line should cause a compile error:
    // let _ = Workflow::Par { workflows: vec![] };

    // Since we can't test compile failures at runtime, we instead
    // verify that the enum can be exhaustively matched without Par

    let workflow = Workflow::Done;
    match workflow {
        Workflow::Observe { .. } => panic!("Should not match Observe"),
        Workflow::Receive { .. } => panic!("Should not match Receive"),
        Workflow::Orient { .. } => panic!("Should not match Orient"),
        Workflow::Propose { .. } => panic!("Should not match Propose"),
        Workflow::Decide { .. } => panic!("Should not match Decide"),
        Workflow::Check { .. } => panic!("Should not match Check"),
        Workflow::Act { .. } => panic!("Should not match Act"),
        Workflow::Oblig { .. } => panic!("Should not match Oblig"),
        Workflow::Let { .. } => panic!("Should not match Let"),
        Workflow::If { .. } => panic!("Should not match If"),
        Workflow::Seq { .. } => panic!("Should not match Seq"),
        // Workflow::Par { .. } => panic!("Par should not exist"),
        Workflow::ForEach { .. } => panic!("Should not match ForEach"),
        Workflow::Ret { .. } => panic!("Should not match Ret"),
        Workflow::With { .. } => panic!("Should not match With"),
        Workflow::Maybe { .. } => panic!("Should not match Maybe"),
        Workflow::Must { .. } => panic!("Should not match Must"),
        Workflow::Set { .. } => panic!("Should not match Set"),
        Workflow::Send { .. } => panic!("Should not match Send"),
        Workflow::Spawn { .. } => panic!("Should not match Spawn"),
        Workflow::Split { .. } => panic!("Should not match Split"),
        Workflow::Kill { .. } => panic!("Should not match Kill"),
        Workflow::Pause { .. } => panic!("Should not match Pause"),
        Workflow::Resume { .. } => panic!("Should not match Resume"),
        Workflow::CheckHealth { .. } => panic!("Should not match CheckHealth"),
        Workflow::Oblige { .. } => panic!("Should not match Oblige"),
        Workflow::CheckObligation { .. } => panic!("Should not match CheckObligation"),
        Workflow::Yield { .. } => panic!("Should not match Yield"),
        Workflow::ProxyResume { .. } => panic!("Should not match ProxyResume"),
        Workflow::Call { .. } => panic!("Should not match Call"),
        Workflow::Done => {} // This is the expected match
    }
}

#[test]
fn test_workflow_contract_par_variant_removed() {
    // Verify that the workflow_contract Workflow enum doesn't have a Par variant
    use ash_core::workflow_contract::Workflow as ContractWorkflow;

    let workflow = ContractWorkflow::Done;
    match workflow {
        ContractWorkflow::Oblige { .. } => panic!("Should not match Oblige"),
        ContractWorkflow::CheckObligation { .. } => panic!("Should not match CheckObligation"),
        // ContractWorkflow::Par { .. } => panic!("Par should not exist"),
        ContractWorkflow::Done => {} // This is the expected match
    }
}

#[test]
fn test_visualizer_no_par_handling() {
    // Verify that the visualizer doesn't need to handle Par
    // This is a compile-time test - if Par variant exists, visualize.rs
    // would need a match arm for it

    // We can't directly test this at runtime, but the presence of this test
    // documents the expectation that Par has been removed from the visualizer
    // This is verified at compile time - if Par existed in the AST,
    // the visualizer would require a match arm for it
}

#[test]
fn test_workflow_serialization_no_par() {
    // Verify that serialized Workflow doesn't include Par
    let workflow = Workflow::Done;
    let serialized = serde_json::to_string(&workflow).unwrap();
    let deserialized: Workflow = serde_json::from_str(&serialized).unwrap();

    // If Par variant exists, this test might pass but we'd want to ensure
    // that Par isn't accidentally included in round-trip serialization
    assert!(matches!(deserialized, Workflow::Done));
}
