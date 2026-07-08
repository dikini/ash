use ash_parser::input::new_input;
use ash_parser::parse_expr::expr;
use ash_parser::surface::{DoStmt, Expr};

fn parse_expr(src: &str) -> Expr {
    let mut input = new_input(src);
    let parsed = expr(&mut input).expect("expression should parse");
    assert_eq!(*input.input.as_ref(), "", "parser left trailing input");
    parsed
}

#[test]
fn parses_workflow_requires_do_statement_as_raw_contract_surface() {
    let parsed = parse_expr("do:Workflow { requires: role(admin); return x }");
    let Expr::DoBlock { target, stmts, .. } = parsed else {
        panic!("expected do block");
    };
    assert_eq!(target.name.as_ref(), "Workflow");
    assert_eq!(stmts.len(), 2);
    match &stmts[0] {
        DoStmt::WorkflowRequires { expr, span } => {
            assert!(span.start < span.end);
            assert!(
                matches!(expr.as_ref(), Expr::Call { func, args, .. } if func.as_ref() == "role" && args.len() == 1)
            );
        }
        other => panic!("expected workflow requires statement, got {other:?}"),
    }
}

#[test]
fn parses_workflow_ensures_do_statement_as_raw_contract_surface() {
    let parsed = parse_expr("do:Workflow { ensures: result > 0; return x }");
    let Expr::DoBlock { stmts, .. } = parsed else {
        panic!("expected do block");
    };
    match &stmts[0] {
        DoStmt::WorkflowEnsures { expr, span } => {
            assert!(span.start < span.end);
            assert!(matches!(expr.as_ref(), Expr::Binary { .. }));
        }
        other => panic!("expected workflow ensures statement, got {other:?}"),
    }
}

#[test]
fn existing_act_and_proc_do_syntax_still_parse_unchanged() {
    for src in [
        "do:Act { x <- read(); return x }",
        "do:Proc { let x = 1; return x }",
    ] {
        let Expr::DoBlock { stmts, .. } = parse_expr(src) else {
            panic!("expected do block for {src}");
        };
        assert!(matches!(stmts.last(), Some(DoStmt::Return { .. })));
        assert!(!stmts.iter().any(|stmt| matches!(
            stmt,
            DoStmt::WorkflowRequires { .. } | DoStmt::WorkflowEnsures { .. }
        )));
    }
}

#[test]
fn classifies_role_any_role_arithmetic_and_result_postcondition_contracts() {
    let role_expr = parse_expr("role(admin)");
    assert!(
        matches!(ash_parser::workflow_contract_classifier::classify_requirement(&role_expr), Ok(ash_core::workflow_contract::Requirement::HasRole(role)) if role == "admin")
    );

    let any_role_expr = parse_expr("any_role([admin, reviewer])");
    assert!(
        matches!(ash_parser::workflow_contract_classifier::classify_requirement(&any_role_expr), Ok(ash_core::workflow_contract::Requirement::AnyRole(policy)) if policy.roles == ["admin", "reviewer"])
    );

    let arithmetic_expr = parse_expr("amount > 0");
    assert!(
        matches!(ash_parser::workflow_contract_classifier::classify_requirement(&arithmetic_expr), Ok(ash_core::workflow_contract::Requirement::Arithmetic { var, .. }) if var == "amount")
    );

    let post_expr = parse_expr("result > 0");
    assert!(matches!(
        ash_parser::workflow_contract_classifier::classify_postcondition(&post_expr),
        Ok(ash_core::workflow_contract::PostPredicate::ResultSatisfies(
            _
        ))
    ));
}

#[test]
fn rejects_empty_any_role_and_unclassified_contracts() {
    let empty = parse_expr("any_role([])");
    assert!(ash_parser::workflow_contract_classifier::classify_requirement(&empty).is_err());

    let unclassified = parse_expr("foo(bar)");
    assert!(ash_parser::workflow_contract_classifier::classify_requirement(&unclassified).is_err());
}
