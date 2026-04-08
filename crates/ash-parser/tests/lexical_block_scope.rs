//! Tests for lexical block scope (SPEC-002 Section 4.4)
//!
//! These tests verify that statement lists lower canonically to nested `LET ... in cont`
//! forms that establish lexical scoping for binding statements.

use ash_parser::input::new_input;
use ash_parser::parse_workflow::workflow;
use ash_parser::surface::{Pattern, Workflow};

// ============================================================================
// Helper Functions
// ============================================================================

fn parse_workflow_body(input: &str) -> Workflow {
    let mut input = new_input(input);
    workflow(&mut input).expect("workflow parse failed")
}

// ============================================================================
// Binding Statements Create Nested Let Structures
// ============================================================================

#[test]
fn simple_let_binding_creates_nested_let_with_done_continuation() {
    let parsed = parse_workflow_body("let x = 10");

    // Should produce: LET x = 10 IN Done
    match parsed {
        Workflow::Let {
            pattern: Pattern::Variable(name),
            continuation: Some(cont),
            ..
        } => {
            assert_eq!(name, "x".into());
            assert!(matches!(*cont, Workflow::Done { .. }));
        }
        _ => panic!("Expected Let with continuation, got: {:?}", parsed),
    }
}

#[test]
fn two_let_bindings_create_right_associative_nesting() {
    let parsed = parse_workflow_body("let x = 10; let y = x + 1");

    // Should produce: LET x = 10 IN (LET y = x + 1 IN Done)
    match parsed {
        Workflow::Let {
            pattern: Pattern::Variable(outer_name),
            continuation: Some(outer_cont),
            ..
        } => {
            assert_eq!(outer_name, "x".into());

            match *outer_cont {
                Workflow::Let {
                    pattern: Pattern::Variable(inner_name),
                    continuation: Some(inner_cont),
                    ..
                } => {
                    assert_eq!(inner_name, "y".into());
                    assert!(matches!(*inner_cont, Workflow::Done { .. }));
                }
                _ => panic!(
                    "Expected inner Let with continuation, got: {:?}",
                    outer_cont
                ),
            }
        }
        _ => panic!("Expected outer Let with continuation, got: {:?}", parsed),
    }
}

#[test]
fn let_then_ret_creates_nested_let_with_ret_continuation() {
    let parsed = parse_workflow_body("let x = 10; ret x");

    // Should produce: LET x = 10 IN (ret x)
    // Note: terminal statement optimization returns bare Ret instead of Seq(ret, Done)
    // because Seq would discard the return value (see SPEC-025 SEQ-ADVANCE rule)
    match parsed {
        Workflow::Let {
            pattern: Pattern::Variable(name),
            continuation: Some(cont),
            ..
        } => {
            assert_eq!(name, "x".into());
            assert!(matches!(*cont, Workflow::Ret { .. }));
        }
        _ => panic!("Expected Let with continuation, got: {:?}", parsed),
    }
}

#[test]
fn mixed_binding_and_non_binding_creates_correct_structure() {
    let parsed = parse_workflow_body("let x = 10; act print(x); ret x");

    // Should produce: LET x = 10 IN (SEQ (act print(x)) (ret x))
    // Note: terminal statement optimization returns bare Ret instead of Seq(ret, Done)
    match parsed {
        Workflow::Let {
            pattern: Pattern::Variable(name),
            continuation: Some(cont),
            ..
        } => {
            assert_eq!(name, "x".into());

            match *cont {
                Workflow::Seq {
                    first: first_seq,
                    second: second_seq,
                    ..
                } => {
                    // First should be act
                    assert!(matches!(*first_seq, Workflow::Act { .. }));

                    // Second should be ret (not wrapped in Seq due to terminal statement optimization)
                    assert!(matches!(*second_seq, Workflow::Ret { .. }));
                }
                _ => panic!("Expected Seq, got: {:?}", cont),
            }
        }
        _ => panic!("Expected Let with continuation, got: {:?}", parsed),
    }
}

#[test]
fn observe_with_binding_creates_nested_observe_with_continuation() {
    let parsed = parse_workflow_body("observe cap as x; ret x");

    // Should produce: OBSERVE cap AS x IN (ret x)
    // Note: terminal statement optimization returns bare Ret instead of Seq(ret, Done)
    match parsed {
        Workflow::Observe {
            binding: Some(Pattern::Variable(name)),
            continuation: Some(cont),
            ..
        } => {
            assert_eq!(name, "x".into());
            assert!(matches!(*cont, Workflow::Ret { .. }));
        }
        _ => panic!(
            "Expected Observe with binding and continuation, got: {:?}",
            parsed
        ),
    }
}

#[test]
fn orient_with_binding_creates_nested_orient_with_continuation() {
    let parsed = parse_workflow_body("orient 1 + 1 as x; ret x");

    // Should produce: ORIENT { 1 + 1 } AS x IN (ret x)
    // Note: terminal statement optimization returns bare Ret instead of Seq(ret, Done)
    match parsed {
        Workflow::Orient {
            binding: Some(Pattern::Variable(name)),
            continuation: Some(cont),
            ..
        } => {
            assert_eq!(name, "x".into());
            assert!(matches!(*cont, Workflow::Ret { .. }));
        }
        _ => panic!(
            "Expected Orient with binding and continuation, got: {:?}",
            parsed
        ),
    }
}

#[test]
fn propose_with_binding_creates_nested_propose_with_continuation() {
    let parsed = parse_workflow_body("propose action as x; ret x");

    // Should produce: PROPOSE action AS x IN (ret x)
    // Note: terminal statement optimization returns bare Ret instead of Seq(ret, Done)
    match parsed {
        Workflow::Propose {
            binding: Some(Pattern::Variable(name)),
            continuation: Some(cont),
            ..
        } => {
            assert_eq!(name, "x".into());
            assert!(matches!(*cont, Workflow::Ret { .. }));
        }
        _ => panic!(
            "Expected Propose with binding and continuation, got: {:?}",
            parsed
        ),
    }
}

#[test]
fn observe_without_binding_uses_seq_not_let() {
    let parsed = parse_workflow_body("observe cap; let x = 10");

    // Should produce: SEQ (observe cap) (LET x = 10 IN Done)
    // because observe without binding is not a binding statement
    match parsed {
        Workflow::Seq { first, second, .. } => {
            assert!(matches!(*first, Workflow::Observe { binding: None, .. }));
            assert!(matches!(*second, Workflow::Let { .. }));
        }
        _ => panic!("Expected Seq, got: {:?}", parsed),
    }
}

#[test]
fn three_bindings_create_deeply_nested_structure() {
    let parsed = parse_workflow_body("let x = 1; let y = x + 1; let z = y + 1");

    // Should produce: LET x = 1 IN (LET y = x + 1 IN (LET z = y + 1 IN Done))
    match parsed {
        Workflow::Let {
            pattern: Pattern::Variable(name1),
            continuation: Some(cont1),
            ..
        } => {
            assert_eq!(name1, "x".into());

            match *cont1 {
                Workflow::Let {
                    pattern: Pattern::Variable(name2),
                    continuation: Some(cont2),
                    ..
                } => {
                    assert_eq!(name2, "y".into());

                    match *cont2 {
                        Workflow::Let {
                            pattern: Pattern::Variable(name3),
                            continuation: Some(cont3),
                            ..
                        } => {
                            assert_eq!(name3, "z".into());
                            assert!(matches!(*cont3, Workflow::Done { .. }));
                        }
                        _ => panic!("Expected third Let, got: {:?}", cont2),
                    }
                }
                _ => panic!("Expected second Let, got: {:?}", cont1),
            }
        }
        _ => panic!("Expected first Let, got: {:?}", parsed),
    }
}
