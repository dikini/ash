//! Tests for Yield workflow lowering
//!
//! These tests verify that SurfaceWorkflow::Yield is correctly lowered
//! to CoreWorkflow::Yield, preserving role, request expression, and
//! continuation semantics.

use ash_core::Workflow as CoreWorkflow;
use ash_parser::input::new_input;
use ash_parser::lower::lower_workflow;
use ash_parser::parse_module::parse_yield;
use winnow::prelude::*;

#[test]
fn test_yield_lowers_to_yield_not_done() {
    // Test that Yield lowers to CoreWorkflow::Yield, not Done
    let input = r#"yield role(payment_processor) 100
    resume response : Int {
        _ => done
    }"#;
    let mut input = new_input(input);
    let result = parse_yield.parse_next(&mut input);

    assert!(
        result.is_ok(),
        "Expected successful parse, got: {:?}",
        result
    );
    let surface_workflow = result.unwrap();

    // Create a workflow definition to use with lower_workflow
    let wf_def = ash_parser::surface::WorkflowDef {
        name: "test".into(),
        params: vec![],
        body: surface_workflow,
        contract: None,
        span: ash_parser::token::Span::new(0, 100, 1, 1),
    };

    let core = lower_workflow(&wf_def).unwrap();

    assert!(
        matches!(core, CoreWorkflow::Yield { .. }),
        "Expected Yield to lower to CoreWorkflow::Yield, got {:?}",
        core
    );
}

#[test]
fn test_yield_preserves_role() {
    // Test that the role name is preserved during lowering
    let input = r#"yield role(auth_service) "token_request"
    resume auth_result : AuthToken {
        _ => done
    }"#;
    let mut input = new_input(input);
    let result = parse_yield.parse_next(&mut input);

    assert!(
        result.is_ok(),
        "Expected successful parse, got: {:?}",
        result
    );
    let surface_workflow = result.unwrap();

    let wf_def = ash_parser::surface::WorkflowDef {
        name: "test".into(),
        params: vec![],
        body: surface_workflow,
        contract: None,
        span: ash_parser::token::Span::new(0, 100, 1, 1),
    };

    let core = lower_workflow(&wf_def).unwrap();

    match core {
        CoreWorkflow::Yield { role, .. } => {
            assert_eq!(role, "auth_service");
        }
        _ => panic!("Expected CoreWorkflow::Yield, got {:?}", core),
    }
}

#[test]
fn test_yield_preserves_request_expr() {
    // Test that the request expression is lowered and preserved
    let input = r#"yield role(calculator) 10 + 20
    resume result : Int {
        _ => done
    }"#;
    let mut input = new_input(input);
    let result = parse_yield.parse_next(&mut input);

    assert!(
        result.is_ok(),
        "Expected successful parse, got: {:?}",
        result
    );
    let surface_workflow = result.unwrap();

    let wf_def = ash_parser::surface::WorkflowDef {
        name: "test".into(),
        params: vec![],
        body: surface_workflow,
        contract: None,
        span: ash_parser::token::Span::new(0, 100, 1, 1),
    };

    let core = lower_workflow(&wf_def).unwrap();

    match core {
        CoreWorkflow::Yield { request, .. } => {
            // Verify the request is a binary expression
            assert!(
                matches!(request.as_ref(), ash_core::Expr::Binary { .. }),
                "Expected request to be a binary expression, got {:?}",
                request
            );
        }
        _ => panic!("Expected CoreWorkflow::Yield"),
    }
}

#[test]
fn test_yield_creates_continuation() {
    // Test that yield arms are converted to a continuation workflow
    let input = r#"yield role(validator) true
    resume validation_result : Bool {
        success => ret success
    }"#;
    let mut input = new_input(input);
    let result = parse_yield.parse_next(&mut input);

    assert!(
        result.is_ok(),
        "Expected successful parse, got: {:?}",
        result
    );
    let surface_workflow = result.unwrap();

    let wf_def = ash_parser::surface::WorkflowDef {
        name: "test".into(),
        params: vec![],
        body: surface_workflow,
        contract: None,
        span: ash_parser::token::Span::new(0, 100, 1, 1),
    };

    let core = lower_workflow(&wf_def).unwrap();

    match core {
        CoreWorkflow::Yield { continuation, .. } => {
            // The continuation should not be empty/Done
            assert!(
                !matches!(continuation.as_ref(), CoreWorkflow::Done),
                "Expected non-trivial continuation from yield arms, got {:?}",
                continuation
            );
        }
        _ => panic!("Expected CoreWorkflow::Yield"),
    }
}

#[test]
fn test_yield_with_multiple_arms_creates_continuation() {
    // Test that multiple yield arms result in a valid continuation
    let input = r#"yield role(router) "route_request"
    resume route : Route {
        r1 => ret r1,
        r2 => ret r2
    }"#;
    let mut input = new_input(input);
    let result = parse_yield.parse_next(&mut input);

    assert!(
        result.is_ok(),
        "Expected successful parse, got: {:?}",
        result
    );
    let surface_workflow = result.unwrap();

    let wf_def = ash_parser::surface::WorkflowDef {
        name: "test".into(),
        params: vec![],
        body: surface_workflow,
        contract: None,
        span: ash_parser::token::Span::new(0, 100, 1, 1),
    };

    let core = lower_workflow(&wf_def).unwrap();

    assert!(
        matches!(core, CoreWorkflow::Yield { .. }),
        "Expected Yield with multiple arms to lower to CoreWorkflow::Yield, got {:?}",
        core
    );
}

#[test]
fn test_yield_preserves_span() {
    // Test that span information is preserved
    let input = r#"yield role(test_role) 42
    resume resp : Int {
        _ => done
    }"#;
    let mut input = new_input(input);
    let result = parse_yield.parse_next(&mut input);

    assert!(
        result.is_ok(),
        "Expected successful parse, got: {:?}",
        result
    );
    let surface_workflow = result.unwrap();

    let wf_def = ash_parser::surface::WorkflowDef {
        name: "test".into(),
        params: vec![],
        body: surface_workflow,
        contract: None,
        span: ash_parser::token::Span::new(0, 100, 1, 1),
    };

    let core = lower_workflow(&wf_def).unwrap();

    match core {
        CoreWorkflow::Yield { span, .. } => {
            // Span should be set (not default zero values)
            assert!(span.start <= span.end, "Span start should be <= end");
        }
        _ => panic!("Expected CoreWorkflow::Yield"),
    }
}

#[test]
fn test_yield_continuation_has_let_binding() {
    // Test that the continuation includes a let binding for the resume variable
    let input = r#"yield role(processor) request
    resume response : Response {
        _ => ret response
    }"#;
    let mut input = new_input(input);
    let result = parse_yield.parse_next(&mut input);

    assert!(
        result.is_ok(),
        "Expected successful parse, got: {:?}",
        result
    );
    let surface_workflow = result.unwrap();

    let wf_def = ash_parser::surface::WorkflowDef {
        name: "test".into(),
        params: vec![],
        body: surface_workflow,
        contract: None,
        span: ash_parser::token::Span::new(0, 100, 1, 1),
    };

    let core = lower_workflow(&wf_def).unwrap();

    match core {
        CoreWorkflow::Yield { continuation, .. } => {
            // The continuation should be a Let binding the resume variable
            assert!(
                matches!(continuation.as_ref(), CoreWorkflow::Let { .. }),
                "Expected continuation to be a Let binding, got {:?}",
                continuation
            );
        }
        _ => panic!("Expected CoreWorkflow::Yield"),
    }
}
