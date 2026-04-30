use ash_parser::surface::{BinaryOp, DoStmt, DoTarget, Expr, Literal};
use ash_parser::token::Span;
use ash_typeck::TypeEnv;
use ash_typeck::check_expr::check_expr;

fn span() -> Span {
    Span::default()
}

fn target(name: &str) -> DoTarget {
    DoTarget {
        name: name.into(),
        args: vec![],
        span: span(),
    }
}

fn var(name: &str) -> Expr {
    Expr::Variable {
        name: name.into(),
        span: span(),
    }
}

fn int_lit(value: i64) -> Expr {
    Expr::Literal(Literal::Int(value))
}

fn call(module: Option<&str>, func: &str, args: Vec<Expr>) -> Expr {
    Expr::Call {
        func: func.into(),
        module: module.map(Into::into),
        args,
        span: span(),
    }
}

fn list(items: Vec<Expr>) -> Expr {
    Expr::List {
        items,
        span: span(),
    }
}

fn binary(left: Expr, op: BinaryOp, right: Expr) -> Expr {
    Expr::Binary {
        op,
        left: Box::new(left),
        right: Box::new(right),
        span: span(),
    }
}

fn do_workflow(stmts: Vec<DoStmt>) -> Expr {
    Expr::DoBlock {
        target: target("Workflow"),
        stmts,
        span: span(),
    }
}

fn ret(value: Expr) -> DoStmt {
    DoStmt::Return {
        value: Box::new(value),
        span: span(),
    }
}

fn bind_stmt(name: &str, value: Expr) -> DoStmt {
    DoStmt::Bind {
        name: name.into(),
        value: Box::new(value),
        span: span(),
    }
}

fn requires(expr: Expr) -> DoStmt {
    DoStmt::WorkflowRequires {
        expr: Box::new(expr),
        span: span(),
    }
}

fn ensures(expr: Expr) -> DoStmt {
    DoStmt::WorkflowEnsures {
        expr: Box::new(expr),
        span: span(),
    }
}

fn error_text(expr: &Expr) -> String {
    let result = check_expr(&TypeEnv::with_builtin_types(), expr);
    assert!(!result.is_ok(), "expression should fail: {result:?}");
    result
        .errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

fn assert_contains_all(text: &str, needles: &[&str]) {
    for needle in needles {
        assert!(text.contains(needle), "missing `{needle}` in:\n{text}");
    }
}

#[test]
fn do_workflow_requires_empty_any_role_reports_requirement_policy_failure() {
    let expr = do_workflow(vec![
        requires(call(None, "any_role", vec![list(vec![])])),
        ret(int_lit(1)),
    ]);

    let text = error_text(&expr);
    assert_contains_all(
        &text,
        &[
            "workflow requires",
            "Requirement",
            "any_role",
            "empty",
            "at least one role",
        ],
    );
}

#[test]
fn workflow_requires_intrinsic_empty_any_role_reports_same_classification_failure() {
    let expr = do_workflow(vec![
        bind_stmt(
            "_gate",
            call(
                Some("workflow"),
                "requires",
                vec![call(None, "any_role", vec![list(vec![])])],
            ),
        ),
        ret(int_lit(1)),
    ]);

    let text = error_text(&expr);
    assert_contains_all(
        &text,
        &[
            "workflow requires",
            "Requirement",
            "any_role",
            "empty",
            "at least one role",
        ],
    );
}

#[test]
fn workflow_requires_any_role_rejects_non_role_name_entry_with_policy_context() {
    let expr = do_workflow(vec![
        requires(call(
            None,
            "any_role",
            vec![list(vec![var("Reviewer"), int_lit(1)])],
        )),
        ret(int_lit(1)),
    ]);

    let text = error_text(&expr);
    assert_contains_all(
        &text,
        &[
            "workflow requires",
            "Requirement",
            "any_role",
            "role policy",
            "role name",
        ],
    );
}

#[test]
fn workflow_ensures_non_result_postcondition_mentions_open_postcondition_target() {
    let expr = do_workflow(vec![
        ensures(binary(var("value"), BinaryOp::Gt, int_lit(0))),
        ret(int_lit(1)),
    ]);

    let text = error_text(&expr);
    assert_contains_all(
        &text,
        &[
            "workflow ensures",
            "OpenPostcondition",
            "result",
            "Workflow result",
        ],
    );
}
