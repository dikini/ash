use ash_parser::surface::{BinaryOp, DoStmt, DoTarget, Expr, Literal};
use ash_parser::token::Span;
use ash_typeck::check_expr::{WorkflowForm, check_expr, elaborate_typed_do_block};
use ash_typeck::{Kind, QualifiedName, Type, TypeEnv};

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
fn int_lit(value: i64) -> Expr {
    Expr::Literal(Literal::Int(value))
}
fn var(name: &str) -> Expr {
    Expr::Variable {
        name: name.into(),
        span: span(),
    }
}
fn call(module: Option<&str>, func: &str, args: Vec<Expr>) -> Expr {
    Expr::Call {
        func: func.into(),
        module: module.map(Into::into),
        args,
        span: span(),
    }
}
fn workflow_unit(value: Expr) -> Expr {
    call(Some("workflow"), "unit", vec![value])
}
fn binary(left: Expr, op: BinaryOp, right: Expr) -> Expr {
    Expr::Binary {
        op,
        left: Box::new(left),
        right: Box::new(right),
        span: span(),
    }
}
fn do_block(target_name: &str, stmts: Vec<DoStmt>) -> Expr {
    Expr::DoBlock {
        target: target(target_name),
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
fn let_stmt(name: &str, value: Expr) -> DoStmt {
    DoStmt::Let {
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
fn do_workflow_return_has_workflow_type_and_unit_form_artifact() {
    let env = TypeEnv::with_builtin_types();
    let expr = do_block("Workflow", vec![ret(int_lit(1))]);
    let result = check_expr(&env, &expr);
    assert!(result.is_ok(), "do:Workflow should type-check: {result:?}");
    assert_eq!(result.ty, computation_type("Workflow", Type::Int));

    let elaborated = elaborate_typed_do_block(&env, &expr).expect("workflow elaborates");
    assert_eq!(elaborated.ty, computation_type("Workflow", Type::Int));
    let artifact = elaborated
        .workflow_artifact
        .expect("workflow artifact is preserved");
    assert!(matches!(artifact.form, WorkflowForm::Unit { .. }));
    assert!(!artifact.projection_events.is_empty());
    assert!(matches!(
        artifact.contract_plan,
        ash_core::workflow_carrier::ContractPlan::EmptyContract { .. }
    ));
    assert!(matches!(
        artifact.source_origin,
        ash_core::workflow_carrier::SourceOrigin::Synthetic { .. }
    ));
}

#[test]
fn do_workflow_bind_preserves_bind_form() {
    let env = TypeEnv::with_builtin_types();
    let expr = do_block(
        "Workflow",
        vec![bind_stmt("x", workflow_unit(int_lit(1))), ret(var("x"))],
    );
    let elaborated = elaborate_typed_do_block(&env, &expr).expect("workflow bind elaborates");
    let artifact = elaborated.workflow_artifact.expect("workflow artifact");
    assert!(
        matches!(artifact.form, WorkflowForm::Bind { .. }),
        "{artifact:?}"
    );
    assert!(matches!(
        artifact.contract_plan,
        ash_core::workflow_carrier::ContractPlan::BindContract { .. }
    ));
}

#[test]
fn do_workflow_contract_statements_are_preserved() {
    let env = TypeEnv::with_builtin_types();
    let expr = do_block(
        "Workflow",
        vec![
            requires(call(None, "role", vec![var("admin")])),
            ensures(binary(var("result"), BinaryOp::Gt, int_lit(0))),
            ret(int_lit(1)),
        ],
    );
    let elaborated = elaborate_typed_do_block(&env, &expr).expect("workflow contracts elaborate");
    let artifact = elaborated.workflow_artifact.expect("workflow artifact");
    assert!(matches!(
        artifact.contract_plan,
        ash_core::workflow_carrier::ContractPlan::BindContract { .. }
    ));
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
fn do_workflow_rejects_proc_and_act_rhs_with_lift_hints() {
    let env = TypeEnv::with_builtin_types();
    let proc_rhs = do_block(
        "Workflow",
        vec![
            bind_stmt("x", call(Some("proc"), "unit", vec![int_lit(1)])),
            ret(var("x")),
        ],
    );
    let proc_result = check_expr(&env, &proc_rhs);
    assert!(!proc_result.is_ok());
    let proc_text = error_text(proc_result);
    assert!(proc_text.contains("do:Workflow"), "{proc_text}");
    assert!(proc_text.contains("workflow::from_proc"), "{proc_text}");

    let act_rhs = do_block(
        "Workflow",
        vec![
            bind_stmt("x", do_block("Act", vec![ret(int_lit(1))])),
            ret(var("x")),
        ],
    );
    let act_result = check_expr(&env, &act_rhs);
    assert!(!act_result.is_ok());
    let act_text = error_text(act_result);
    assert!(act_text.contains("do:Workflow"), "{act_text}");
    assert!(act_text.contains("workflow::from_act"), "{act_text}");
}

#[test]
fn do_workflow_rejects_opaque_workflow_rhs_but_accepts_live_local_artifact() {
    let mut env = TypeEnv::with_builtin_types();
    env.bind_variable("opaque", computation_type("Workflow", Type::Int));
    let opaque = do_block(
        "Workflow",
        vec![bind_stmt("x", var("opaque")), ret(var("x"))],
    );
    let opaque_result = check_expr(&env, &opaque);
    assert!(!opaque_result.is_ok());
    let opaque_text = error_text(opaque_result);
    assert!(opaque_text.contains("opaque Workflow"), "{opaque_text}");
    assert!(
        opaque_text.contains("WorkflowTypedArtifact"),
        "{opaque_text}"
    );

    let live = do_block(
        "Workflow",
        vec![
            let_stmt("w", workflow_unit(int_lit(1))),
            bind_stmt("x", var("w")),
            ret(var("x")),
        ],
    );
    let live_result = check_expr(&env, &live);
    assert!(
        live_result.is_ok(),
        "live local workflow artifact should bind: {live_result:?}"
    );

    let elaborated = elaborate_typed_do_block(&env, &live).expect("live local workflow elaborates");
    let artifact = elaborated.workflow_artifact.expect("workflow artifact");
    let WorkflowForm::Bind { source, .. } = artifact.form else {
        panic!("expected bind form for live local workflow artifact: {artifact:?}");
    };
    assert!(
        matches!(*source, WorkflowForm::Unit { .. }),
        "local workflow binding should recover the original unit artifact, not wrap opaque variable: {source:?}"
    );
}

#[test]
fn workflow_contract_statements_use_classifier_and_resolve_ordinary_requirement_names() {
    let env = TypeEnv::with_builtin_types();
    let missing = do_block(
        "Workflow",
        vec![
            requires(binary(var("threshold"), BinaryOp::Gt, int_lit(0))),
            ret(int_lit(1)),
        ],
    );
    let missing_result = check_expr(&env, &missing);
    assert!(!missing_result.is_ok());
    let missing_text = error_text(missing_result);
    assert!(missing_text.contains("threshold"), "{missing_text}");

    let role_symbol = do_block(
        "Workflow",
        vec![
            requires(call(None, "role", vec![var("admin")])),
            ret(int_lit(1)),
        ],
    );
    let role_result = check_expr(&env, &role_symbol);
    assert!(
        role_result.is_ok(),
        "role symbols are contract names, not ordinary value variables: {role_result:?}"
    );
}

#[test]
fn workflow_live_artifact_tracking_clears_shadowed_bindings() {
    let mut env = TypeEnv::with_builtin_types();
    env.bind_variable("opaque", computation_type("Workflow", Type::Int));
    let shadowed = do_block(
        "Workflow",
        vec![
            let_stmt("w", workflow_unit(int_lit(1))),
            let_stmt("w", var("opaque")),
            bind_stmt("x", var("w")),
            ret(var("x")),
        ],
    );

    let result = check_expr(&env, &shadowed);
    assert!(!result.is_ok());
    let text = error_text(result);
    assert!(text.contains("opaque Workflow"), "{text}");
}

#[test]
fn contract_statements_remain_workflow_only() {
    let env = TypeEnv::with_builtin_types();
    let expr = do_block("Proc", vec![requires(var("p")), ret(int_lit(1))]);
    let result = check_expr(&env, &expr);
    assert!(!result.is_ok());
    let text = error_text(result);
    assert!(text.contains("do:Workflow"), "{text}");
    assert!(text.contains("workflow contract statement"), "{text}");
}
