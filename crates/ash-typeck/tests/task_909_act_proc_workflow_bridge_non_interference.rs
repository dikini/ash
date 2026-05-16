use ash_core::ast::Expr as CoreExpr;
use ash_parser::surface::{DoStmt, DoTarget, Expr, Literal};
use ash_parser::token::Span;
use ash_typeck::check_expr::{WorkflowForm, check_expr, elaborate_typed_do_block};
use ash_typeck::{Kind, QualifiedName, Type, TypeEnv, resolve_do_target_for_test};

fn span() -> Span {
    Span::default()
}

fn target(name: &str) -> DoTarget {
    DoTarget {
        name: name.into(),
        args: Vec::new(),
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

fn computation_type(name: &str, inner: Type) -> Type {
    Type::Constructor {
        name: QualifiedName::root(name),
        args: vec![inner],
        kind: Kind::Type,
    }
}

#[test]
fn bridge_targets_resolve_without_source_declared_monad_interface() {
    let env = TypeEnv::with_builtin_types();

    for target_name in ["Act", "Proc", "Workflow"] {
        resolve_do_target_for_test(&env, &target(target_name))
            .unwrap_or_else(|err| panic!("do:{target_name} bridge should resolve: {err}"));
    }
}

#[test]
fn do_act_still_elaborates_to_hidden_unqualified_dictionary_calls() {
    let env = TypeEnv::with_builtin_types();
    let expr = do_block("Act", vec![ret(int_lit(1))]);

    let result = check_expr(&env, &expr);
    assert!(result.is_ok(), "do:Act should type-check: {result:?}");
    assert_eq!(result.ty, computation_type("Act", Type::Int));

    let elaborated = elaborate_typed_do_block(&env, &expr).expect("do:Act elaborates");
    assert_eq!(elaborated.ty, computation_type("Act", Type::Int));
    assert!(
        matches!(elaborated.expr, CoreExpr::Call { ref func, module: None, .. } if func == "unit"),
        "do:Act should keep the hidden Act unit bridge call: {:?}",
        elaborated.expr
    );
}

#[test]
fn do_proc_still_elaborates_to_proc_dictionary_calls() {
    let env = TypeEnv::with_builtin_types();
    let expr = do_block(
        "Proc",
        vec![
            bind_stmt("x", call(Some("proc"), "unit", vec![int_lit(1)])),
            ret(var("x")),
        ],
    );

    let result = check_expr(&env, &expr);
    assert!(result.is_ok(), "do:Proc should type-check: {result:?}");
    assert_eq!(result.ty, computation_type("Proc", Type::Int));

    let elaborated = elaborate_typed_do_block(&env, &expr).expect("do:Proc elaborates");
    assert!(
        matches!(elaborated.expr, CoreExpr::Call { ref func, ref module, .. } if func == "bind" && module.as_deref() == Some("proc")),
        "do:Proc should keep the proc::bind bridge call: {:?}",
        elaborated.expr
    );
}

#[test]
fn do_workflow_still_preserves_workflow_artifact_bridge() {
    let env = TypeEnv::with_builtin_types();
    let expr = do_block(
        "Workflow",
        vec![
            bind_stmt("x", call(Some("workflow"), "unit", vec![int_lit(1)])),
            ret(var("x")),
        ],
    );

    let result = check_expr(&env, &expr);
    assert!(result.is_ok(), "do:Workflow should type-check: {result:?}");
    assert_eq!(result.ty, computation_type("Workflow", Type::Int));

    let elaborated = elaborate_typed_do_block(&env, &expr).expect("do:Workflow elaborates");
    let artifact = elaborated
        .workflow_artifact
        .expect("do:Workflow bridge must keep a live WorkflowTypedArtifact");
    assert!(
        matches!(artifact.form, WorkflowForm::Bind { .. }),
        "{artifact:?}"
    );
}
