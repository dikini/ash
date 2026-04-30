use ash_core::workflow_carrier::{ContractPlan, ProjectionEventKind, SourceOrigin, WorkflowBinder};
use ash_parser::surface::{ComprehensionQualifier, DoStmt, DoTarget, Expr, Literal};
use ash_parser::token::Span;
use ash_typeck::check_expr::{
    WorkflowForm, check_expr, elaborate_typed_comprehension, elaborate_typed_do_block,
};
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

fn proc_unit(value: Expr) -> Expr {
    call(Some("proc"), "unit", vec![value])
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

fn bind_qual(name: &str, value: Expr) -> ComprehensionQualifier {
    ComprehensionQualifier::Bind {
        name: name.into(),
        value: Box::new(value),
        span: span(),
    }
}

fn comprehension(result: Expr, qualifiers: Vec<ComprehensionQualifier>) -> Expr {
    Expr::Comprehension {
        result: Box::new(result),
        qualifiers,
        target: Some(target("Workflow")),
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
fn workflow_comprehension_elaborates_like_equivalent_do_workflow() {
    let env = TypeEnv::with_builtin_types();
    let comp = comprehension(var("x"), vec![bind_qual("x", workflow_unit(int_lit(1)))]);
    let explicit_do = do_block(
        "Workflow",
        vec![bind_stmt("x", workflow_unit(int_lit(1))), ret(var("x"))],
    );

    let checked = check_expr(&env, &comp);
    assert!(
        checked.is_ok(),
        "Workflow comprehension should type-check through do:Workflow: {checked:?}"
    );
    assert_eq!(checked.ty, computation_type("Workflow", Type::Int));

    let comp_elaborated =
        elaborate_typed_comprehension(&env, &comp).expect("comprehension elaborates");
    let do_elaborated =
        elaborate_typed_do_block(&env, &explicit_do).expect("do:Workflow elaborates");
    assert_eq!(comp_elaborated.ty, do_elaborated.ty);
    assert_eq!(comp_elaborated.expr, do_elaborated.expr);

    let comp_artifact = comp_elaborated
        .workflow_artifact
        .expect("Workflow comprehension keeps a live WorkflowTypedArtifact");
    let do_artifact = do_elaborated
        .workflow_artifact
        .expect("do:Workflow keeps a live WorkflowTypedArtifact");
    assert_eq!(comp_artifact.form, do_artifact.form);
    assert_eq!(
        comp_artifact.projection_events,
        do_artifact.projection_events
    );
    assert_eq!(comp_artifact.contract_plan, do_artifact.contract_plan);
    assert_eq!(comp_artifact.obligations, do_artifact.obligations);
    assert_eq!(comp_artifact.source_origin, do_artifact.source_origin);

    let WorkflowForm::Bind {
        binder,
        source,
        next,
        ..
    } = comp_artifact.form
    else {
        panic!("expected comprehension to normalize to a Workflow bind form");
    };
    assert_eq!(binder, WorkflowBinder::Named("x".to_string()));
    assert!(matches!(*source, WorkflowForm::Unit { .. }));
    assert!(matches!(*next, WorkflowForm::Unit { .. }));
    assert!(matches!(
        comp_artifact.contract_plan,
        ContractPlan::BindContract { .. }
    ));
    assert!(matches!(
        comp_artifact.source_origin,
        SourceOrigin::Synthetic { ref reason, .. } if reason == "do:Workflow"
    ));
    assert!(comp_artifact.projection_events.iter().any(|event| {
        matches!(event.kind, ProjectionEventKind::Bind { binder: WorkflowBinder::Named(ref name) } if name == "x")
    }));
}

#[test]
fn workflow_comprehension_rejects_raw_proc_rhs_without_from_proc() {
    let env = TypeEnv::with_builtin_types();
    let expr = comprehension(var("x"), vec![bind_qual("x", proc_unit(int_lit(1)))]);

    let result = check_expr(&env, &expr);
    assert!(!result.is_ok(), "raw Proc RHS must not bind as Workflow");
    let text = error_text(result);
    assert!(text.contains("do:Workflow"), "{text}");
    assert!(text.contains("Proc<Int>"), "{text}");
    assert!(text.contains("workflow::from_proc"), "{text}");
}

#[test]
fn workflow_comprehension_rejects_raw_act_rhs_without_from_act() {
    let env = TypeEnv::with_builtin_types();
    let raw_act = do_block("Act", vec![ret(int_lit(1))]);
    let expr = comprehension(var("x"), vec![bind_qual("x", raw_act)]);

    let result = check_expr(&env, &expr);
    assert!(!result.is_ok(), "raw Act RHS must not bind as Workflow");
    let text = error_text(result);
    assert!(text.contains("do:Workflow"), "{text}");
    assert!(text.contains("Act<Int>"), "{text}");
    assert!(text.contains("workflow::from_act"), "{text}");
}

#[test]
fn workflow_comprehension_accepts_explicit_proc_and_act_lifts() {
    let env = TypeEnv::with_builtin_types();
    let from_proc = comprehension(
        var("x"),
        vec![bind_qual(
            "x",
            call(Some("workflow"), "from_proc", vec![proc_unit(int_lit(1))]),
        )],
    );
    let from_act = comprehension(
        var("x"),
        vec![bind_qual(
            "x",
            call(
                Some("workflow"),
                "from_act",
                vec![do_block("Act", vec![ret(int_lit(1))])],
            ),
        )],
    );

    for expr in [&from_proc, &from_act] {
        let result = check_expr(&env, expr);
        assert!(
            result.is_ok(),
            "explicit workflow lift should pass: {result:?}"
        );
        assert_eq!(result.ty, computation_type("Workflow", Type::Int));
    }

    let proc_artifact = elaborate_typed_comprehension(&env, &from_proc)
        .expect("from_proc comprehension elaborates")
        .workflow_artifact
        .expect("from_proc artifact");
    let WorkflowForm::Bind { source, .. } = proc_artifact.form else {
        panic!("expected from_proc comprehension bind artifact");
    };
    assert!(matches!(*source, WorkflowForm::FromProc { .. }));

    let act_artifact = elaborate_typed_comprehension(&env, &from_act)
        .expect("from_act comprehension elaborates")
        .workflow_artifact
        .expect("from_act artifact");
    let WorkflowForm::Bind { source, .. } = act_artifact.form else {
        panic!("expected from_act comprehension bind artifact");
    };
    assert!(matches!(*source, WorkflowForm::FromAct { .. }));
}
