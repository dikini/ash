use ash_core::ast::Expr as CoreExpr;
use ash_parser::surface::{DoStmt, DoTarget, Expr, Literal};
use ash_parser::token::Span;
use ash_typeck::check_expr::{check_expr, elaborate_typed_do_block};
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

fn unit_call(module: &str, value: Expr) -> Expr {
    Expr::Call {
        func: "unit".into(),
        module: Some(module.into()),
        args: vec![value],
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

fn let_stmt(name: &str, value: Expr) -> DoStmt {
    DoStmt::Let {
        name: name.into(),
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

fn assert_err_contains(expr: Expr, needles: &[&str]) {
    let env = TypeEnv::with_builtin_types();
    let result = check_expr(&env, &expr);
    assert!(!result.is_ok(), "expected type error, got {result:?}");
    let text = result
        .errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    for needle in needles {
        assert!(
            text.contains(needle),
            "expected error text to contain {needle:?}; got:\n{text}"
        );
    }
}

#[test]
fn do_act_pure_return_has_act_int_type() {
    let env = TypeEnv::with_builtin_types();
    let expr = do_block("Act", vec![ret(int_lit(1))]);

    let result = check_expr(&env, &expr);

    assert!(
        result.is_ok(),
        "do:Act return should type-check: {result:?}"
    );
    assert_eq!(result.ty, computation_type("Act", Type::Int));
}

#[test]
fn do_act_let_then_return_uses_pure_lexical_binding() {
    let env = TypeEnv::with_builtin_types();
    let expr = do_block("Act", vec![let_stmt("x", int_lit(1)), ret(var("x"))]);

    let result = check_expr(&env, &expr);

    assert!(
        result.is_ok(),
        "do:Act let/return should type-check: {result:?}"
    );
    assert_eq!(result.ty, computation_type("Act", Type::Int));
}

#[test]
fn do_act_monadic_bind_unwraps_act_inner_type() {
    let mut env = TypeEnv::with_builtin_types();
    env.bind_variable(
        "act::unit",
        Type::Fn(
            vec![Type::Int],
            Box::new(computation_type("Act", Type::Int)),
        ),
    );
    let expr = do_block(
        "Act",
        vec![bind_stmt("x", unit_call("act", int_lit(1))), ret(var("x"))],
    );

    let result = check_expr(&env, &expr);

    assert!(result.is_ok(), "do:Act bind should type-check: {result:?}");
    assert_eq!(result.ty, computation_type("Act", Type::Int));
}

#[test]
fn do_proc_monadic_bind_unwraps_proc_inner_type() {
    let env = TypeEnv::with_builtin_types();
    let expr = do_block(
        "Proc",
        vec![bind_stmt("x", unit_call("proc", int_lit(1))), ret(var("x"))],
    );

    let result = check_expr(&env, &expr);

    assert!(result.is_ok(), "do:Proc bind should type-check: {result:?}");
    assert_eq!(result.ty, computation_type("Proc", Type::Int));
}

#[test]
fn do_proc_rejects_act_bind_rhs_as_constructor_mismatch() {
    let mut env = TypeEnv::with_builtin_types();
    env.bind_variable(
        "act::unit",
        Type::Fn(
            vec![Type::Int],
            Box::new(computation_type("Act", Type::Int)),
        ),
    );
    let expr = do_block(
        "Proc",
        vec![bind_stmt("x", unit_call("act", int_lit(1))), ret(var("x"))],
    );

    let result = check_expr(&env, &expr);

    assert!(!result.is_ok(), "Act RHS in do:Proc should be rejected");
    let text = result
        .errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    assert!(text.contains("do:Proc"), "{text}");
    assert!(text.contains("Act<Int>"), "{text}");
    assert!(text.contains("proc::from_act"), "{text}");
}

#[test]
fn do_act_rejects_pure_rhs_used_as_bind_with_let_hint() {
    let expr = do_block("Act", vec![bind_stmt("x", int_lit(1)), ret(var("x"))]);

    assert_err_contains(expr, &["<-", "Act", "Int", "use let"]);
}

#[test]
fn do_act_rejects_return_before_last() {
    let expr = do_block("Act", vec![ret(int_lit(1)), let_stmt("x", int_lit(2))]);

    assert_err_contains(expr, &["return", "last", "do block"]);
}

#[test]
fn do_act_rejects_missing_final_return() {
    let expr = do_block("Act", vec![let_stmt("x", int_lit(1))]);

    assert_err_contains(expr, &["do block", "end", "return"]);
}

#[test]
fn do_act_rejects_empty_block() {
    let expr = do_block("Act", vec![]);

    assert_err_contains(expr, &["empty", "do block"]);
}

#[test]
fn typed_elaboration_lowers_act_return_through_hidden_act_dictionary() {
    let env = TypeEnv::with_builtin_types();
    let expr = do_block("Act", vec![ret(int_lit(1))]);

    let elaborated = elaborate_typed_do_block(&env, &expr).expect("typed do:Act elaborates");

    assert_eq!(elaborated.ty, computation_type("Act", Type::Int));
    assert!(matches!(
        elaborated.expr,
        CoreExpr::Call { ref func, module: None, ref arguments }
            if func == "unit" && arguments.len() == 1
    ));
}

#[test]
fn typed_elaboration_lowers_proc_bind_through_resolved_proc_dictionary() {
    let env = TypeEnv::with_builtin_types();
    let expr = do_block(
        "Proc",
        vec![bind_stmt("x", unit_call("proc", int_lit(1))), ret(var("x"))],
    );

    let elaborated = elaborate_typed_do_block(&env, &expr).expect("typed do:Proc elaborates");

    assert_eq!(elaborated.ty, computation_type("Proc", Type::Int));
    assert!(matches!(
        elaborated.expr,
        CoreExpr::Call { ref func, module: Some(ref module), ref arguments }
            if func == "bind" && module == "proc" && arguments.len() == 2
    ));
}

#[test]
fn do_bind_requires_full_qualified_constructor_identity() {
    let mut env = TypeEnv::with_builtin_types();
    env.bind_variable(
        "other_proc",
        Type::Constructor {
            name: QualifiedName::qualified(vec!["other".to_string()], "Proc"),
            args: vec![Type::Int],
            kind: Kind::Type,
        },
    );
    let expr = do_block(
        "Proc",
        vec![bind_stmt("x", var("other_proc")), ret(var("x"))],
    );

    let result = check_expr(&env, &expr);
    assert!(
        !result.is_ok(),
        "qualified constructor mismatch should be rejected"
    );
    let text = result
        .errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    assert!(text.contains("do:Proc"), "{text}");
    assert!(text.contains("other::Proc<Int>"), "{text}");
    assert!(text.contains("explicit lift"), "{text}");
}
