use ash_parser::surface::{BinaryOp, DoStmt, DoTarget, Expr, Literal};
use ash_parser::token::Span;
use ash_typeck::check_expr::{WorkflowForm, check_expr, elaborate_typed_do_block};
use ash_typeck::{Kind, QualifiedName, Type, TypeEnv};

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
fn workflow_call(func: &str, args: Vec<Expr>) -> Expr {
    call(Some("workflow"), func, args)
}
fn fn1(name: &str, body: Expr) -> Expr {
    Expr::FnDef {
        params: vec![(name.into(), None)],
        return_type: None,
        body: Box::new(body),
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
fn target(name: &str) -> DoTarget {
    DoTarget {
        name: name.into(),
        args: vec![],
        span: span(),
    }
}
fn do_block(stmts: Vec<DoStmt>) -> Expr {
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
fn computation_type(name: &str, inner: Type) -> Type {
    Type::Constructor {
        name: QualifiedName::root(name),
        args: vec![inner],
        kind: Kind::Type,
    }
}
fn error_text(result: ash_typeck::check_expr::CheckResult) -> String {
    result
        .errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn ordinary_workflow_unit_bind_then_preserve_forms_in_workflow_context() {
    let env = TypeEnv::with_builtin_types();
    let expr = do_block(vec![
        bind_stmt("a", workflow_call("unit", vec![int_lit(1)])),
        bind_stmt(
            "b",
            workflow_call(
                "bind",
                vec![
                    workflow_call("unit", vec![var("a")]),
                    fn1("x", workflow_call("unit", vec![var("x")])),
                ],
            ),
        ),
        bind_stmt(
            "c",
            workflow_call(
                "then",
                vec![
                    workflow_call("unit", vec![var("b")]),
                    workflow_call("unit", vec![int_lit(3)]),
                ],
            ),
        ),
        ret(var("c")),
    ]);

    let checked = check_expr(&env, &expr);
    assert!(
        checked.is_ok(),
        "workflow algebra calls should type-check: {checked:?}"
    );
    assert_eq!(checked.ty, computation_type("Workflow", Type::Int));

    let artifact = elaborate_typed_do_block(&env, &expr)
        .expect("elaborates")
        .workflow_artifact
        .expect("artifact");
    let WorkflowForm::Bind { next, .. } = artifact.form else {
        panic!("expected outer bind: {artifact:?}");
    };
    let WorkflowForm::Bind { source, next, .. } = *next else {
        panic!("expected bind call statement: {next:?}");
    };
    assert!(
        matches!(*source, WorkflowForm::Bind { .. }),
        "workflow::bind must preserve Bind source form"
    );
    let WorkflowForm::Bind { source, .. } = *next else {
        panic!("expected then statement: {next:?}");
    };
    assert!(
        matches!(*source, WorkflowForm::Bind { .. }),
        "workflow::then must elaborate to Bind source form"
    );
}

#[test]
fn ordinary_contract_intrinsic_calls_use_classifier_without_denotable_contract_values() {
    let env = TypeEnv::with_builtin_types();
    let expr = do_block(vec![
        bind_stmt(
            "_",
            workflow_call("requires", vec![call(None, "role", vec![var("admin")])]),
        ),
        bind_stmt(
            "_",
            workflow_call(
                "ensures",
                vec![binary(var("result"), BinaryOp::Gt, int_lit(0))],
            ),
        ),
        ret(int_lit(1)),
    ]);

    let checked = check_expr(&env, &expr);
    assert!(
        checked.is_ok(),
        "contract intrinsic args should be classified raw: {checked:?}"
    );
    let artifact = elaborate_typed_do_block(&env, &expr)
        .expect("elaborates")
        .workflow_artifact
        .expect("artifact");
    assert!(artifact.projection_events.iter().any(|event| matches!(
        event.kind,
        ash_core::workflow_carrier::ProjectionEventKind::Requires { .. }
    )));
    assert!(artifact.projection_events.iter().any(|event| matches!(
        event.kind,
        ash_core::workflow_carrier::ProjectionEventKind::Ensures { .. }
    )));
}

#[test]
fn unqualified_workflow_operations_are_not_available_and_opaque_workflows_still_reject() {
    let mut env = TypeEnv::with_builtin_types();
    env.bind_variable("opaque", computation_type("Workflow", Type::Int));
    let unqualified = do_block(vec![
        bind_stmt("x", call(None, "unit", vec![int_lit(1)])),
        ret(var("x")),
    ]);
    let text = error_text(check_expr(&env, &unqualified));
    assert!(text.contains("unknown function 'unit'"), "{text}");

    let opaque_then = do_block(vec![
        bind_stmt(
            "x",
            workflow_call(
                "then",
                vec![var("opaque"), workflow_call("unit", vec![int_lit(2)])],
            ),
        ),
        ret(var("x")),
    ]);
    let text = error_text(check_expr(&env, &opaque_then));
    assert!(
        text.contains("opaque Workflow") || text.contains("WorkflowTypedArtifact"),
        "{text}"
    );
}

#[test]
fn bind_rejects_continuation_that_does_not_produce_workflow_form() {
    let env = TypeEnv::with_builtin_types();
    let expr = do_block(vec![
        bind_stmt(
            "x",
            workflow_call(
                "bind",
                vec![workflow_call("unit", vec![int_lit(1)]), fn1("n", var("n"))],
            ),
        ),
        ret(var("x")),
    ]);
    let text = error_text(check_expr(&env, &expr));
    assert!(
        text.contains("argument type mismatch") || text.contains("Workflow"),
        "{text}"
    );
}
