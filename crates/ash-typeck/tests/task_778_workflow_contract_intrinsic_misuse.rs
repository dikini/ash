use ash_parser::surface::{BinaryOp, Expr, Literal};
use ash_parser::token::Span;
use ash_typeck::TypeEnv;
use ash_typeck::check_expr::check_expr;

fn span() -> Span {
    Span::default()
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

fn binary(left: Expr, op: BinaryOp, right: Expr) -> Expr {
    Expr::Binary {
        op,
        left: Box::new(left),
        right: Box::new(right),
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
fn standalone_workflow_requires_reports_requirement_is_non_denotable_contract_only() {
    let expr = call(Some("workflow"), "requires", vec![int_lit(1)]);

    let text = error_text(&expr);
    assert_contains_all(
        &text,
        &[
            "workflow::requires",
            "Requirement",
            "non-denotable",
            "contract-only",
            "do:Workflow",
        ],
    );
}

#[test]
fn standalone_valid_workflow_requires_is_still_contract_only_misuse() {
    let expr = call(
        Some("workflow"),
        "requires",
        vec![call(None, "role", vec![var("admin")])],
    );

    let text = error_text(&expr);
    assert_contains_all(
        &text,
        &[
            "workflow::requires",
            "Requirement",
            "non-denotable",
            "contract-only",
            "outside do:Workflow",
        ],
    );
}

#[test]
fn standalone_workflow_ensures_open_result_reports_open_postcondition_boundary() {
    let expr = call(
        Some("workflow"),
        "ensures",
        vec![binary(var("result"), BinaryOp::Gt, int_lit(0))],
    );

    let text = error_text(&expr);
    assert_contains_all(
        &text,
        &[
            "workflow::ensures",
            "OpenPostcondition",
            "result",
            "Workflow result boundary",
            "do:Workflow",
        ],
    );
}

#[test]
fn standalone_valid_workflow_ensures_is_still_contract_only_misuse() {
    let expr = call(
        Some("workflow"),
        "ensures",
        vec![binary(var("result"), BinaryOp::Geq, int_lit(0))],
    );

    let text = error_text(&expr);
    assert_contains_all(
        &text,
        &[
            "workflow::ensures",
            "OpenPostcondition",
            "non-denotable",
            "contract-only",
            "Workflow result boundary",
        ],
    );
}

#[test]
fn workflow_requires_wrong_arity_reports_requirement_intrinsic_parameter_class() {
    let expr = call(Some("workflow"), "requires", vec![]);

    let text = error_text(&expr);
    assert_contains_all(
        &text,
        &[
            "workflow::requires",
            "Requirement",
            "expects 1",
            "non-denotable",
        ],
    );
}
