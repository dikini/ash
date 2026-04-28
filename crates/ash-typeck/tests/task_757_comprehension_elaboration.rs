use ash_core::ast::Expr as CoreExpr;
use ash_parser::surface::{ComprehensionQualifier, DoStmt, DoTarget, Expr, Literal};
use ash_parser::token::Span;
use ash_typeck::check_expr::{check_expr, elaborate_typed_comprehension, elaborate_typed_do_block};
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

fn comprehension(
    target_name: Option<&str>,
    result: Expr,
    qualifiers: Vec<ComprehensionQualifier>,
) -> Expr {
    Expr::Comprehension {
        result: Box::new(result),
        qualifiers,
        target: target_name.map(target),
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

fn let_qual(name: &str, value: Expr) -> ComprehensionQualifier {
    ComprehensionQualifier::Let {
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

fn discard_qual(value: Expr) -> ComprehensionQualifier {
    ComprehensionQualifier::DiscardBind {
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

fn env_with_act_unit() -> TypeEnv {
    let mut env = TypeEnv::with_builtin_types();
    env.bind_variable(
        "act::unit",
        Type::Fn(
            vec![Type::Int],
            Box::new(computation_type("Act", Type::Int)),
        ),
    );
    env
}

fn assert_err_contains(expr: Expr, needles: &[&str]) {
    let env = env_with_act_unit();
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

fn assert_same_elaboration(env: &TypeEnv, comp: Expr, explicit_do: Expr) {
    let comp_result = elaborate_typed_comprehension(env, &comp).expect("comprehension elaborates");
    let do_result = elaborate_typed_do_block(env, &explicit_do).expect("do block elaborates");
    assert_eq!(comp_result.ty, do_result.ty);
    assert_eq!(comp_result.expr, do_result.expr);
}

#[test]
fn comprehension_act_elaborates_like_equivalent_do_act() {
    let env = env_with_act_unit();
    let comp = comprehension(
        Some("Act"),
        var("x"),
        vec![bind_qual("x", unit_call("act", int_lit(1)))],
    );
    let explicit_do = do_block(
        "Act",
        vec![bind_stmt("x", unit_call("act", int_lit(1))), ret(var("x"))],
    );

    let checked = check_expr(&env, &comp);
    assert!(
        checked.is_ok(),
        "comprehension should type-check: {checked:?}"
    );
    assert_eq!(checked.ty, computation_type("Act", Type::Int));
    assert_same_elaboration(&env, comp, explicit_do);
}

#[test]
fn comprehension_proc_elaborates_like_equivalent_do_proc() {
    let env = TypeEnv::with_builtin_types();
    let comp = comprehension(
        Some("Proc"),
        var("x"),
        vec![bind_qual("x", unit_call("proc", int_lit(1)))],
    );
    let explicit_do = do_block(
        "Proc",
        vec![bind_stmt("x", unit_call("proc", int_lit(1))), ret(var("x"))],
    );

    let checked = check_expr(&env, &comp);
    assert!(
        checked.is_ok(),
        "comprehension should type-check: {checked:?}"
    );
    assert_eq!(checked.ty, computation_type("Proc", Type::Int));
    assert_same_elaboration(&env, comp, explicit_do);
}

#[test]
fn comprehension_mixed_let_bind_and_discard_elaborates_like_do_block() {
    let env = env_with_act_unit();
    let comp = comprehension(
        Some("Act"),
        var("x"),
        vec![
            let_qual("seed", int_lit(1)),
            bind_qual("x", unit_call("act", var("seed"))),
            discard_qual(unit_call("act", int_lit(2))),
        ],
    );
    let explicit_do = do_block(
        "Act",
        vec![
            let_stmt("seed", int_lit(1)),
            bind_stmt("x", unit_call("act", var("seed"))),
            bind_stmt("_", unit_call("act", int_lit(2))),
            ret(var("x")),
        ],
    );

    assert_same_elaboration(&env, comp, explicit_do);
}

#[test]
fn comprehension_requires_explicit_target_in_mvp() {
    let expr = comprehension(
        None,
        var("x"),
        vec![bind_qual("x", unit_call("act", int_lit(1)))],
    );
    assert_err_contains(expr, &["comprehension", "explicit target"]);
}

#[test]
fn comprehension_rejects_pure_rhs_with_bind() {
    let expr = comprehension(Some("Act"), var("x"), vec![bind_qual("x", int_lit(1))]);
    assert_err_contains(
        expr,
        &[
            "do:Act",
            "pure expressions cannot be used with <-",
            "use let",
        ],
    );
}

#[test]
fn comprehension_proc_rejects_raw_act_rhs_without_from_act() {
    let expr = comprehension(
        Some("Proc"),
        var("x"),
        vec![bind_qual("x", unit_call("act", int_lit(1)))],
    );
    assert_err_contains(expr, &["do:Proc", "Act<Int>", "proc::from_act"]);
}

#[test]
fn comprehension_proc_accepts_explicit_proc_from_act() {
    let env = env_with_act_unit();
    let lifted = call(Some("proc"), "from_act", vec![unit_call("act", int_lit(1))]);
    let expr = comprehension(Some("Proc"), var("x"), vec![bind_qual("x", lifted)]);

    let result = check_expr(&env, &expr);
    assert!(
        result.is_ok(),
        "explicit proc::from_act should pass: {result:?}"
    );
    assert_eq!(result.ty, computation_type("Proc", Type::Int));
}

#[test]
fn comprehension_rejects_wrong_target_kind() {
    let expr = comprehension(Some("Int"), var("x"), vec![bind_qual("x", int_lit(1))]);
    assert_err_contains(expr, &["do target Int has kind", "expected * -> *"]);
}

#[test]
fn comprehension_rejects_missing_dictionary_target() {
    let expr = comprehension(
        Some("Option"),
        var("x"),
        vec![bind_qual("x", unit_call("act", int_lit(1)))],
    );
    assert_err_contains(expr, &["do target Option", "no MVP dictionary"]);
}

#[test]
fn elaborate_typed_comprehension_rejects_non_comprehension() {
    let env = TypeEnv::with_builtin_types();
    let err =
        elaborate_typed_comprehension(&env, &int_lit(1)).expect_err("non-comprehension rejected");
    let text = err
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    assert!(text.contains("typed comprehension elaboration requires a comprehension expression"));
}

#[test]
fn elaborated_comprehension_core_shape_is_nested_bind_return() {
    let env = TypeEnv::with_builtin_types();
    let expr = comprehension(
        Some("Proc"),
        var("x"),
        vec![bind_qual("x", unit_call("proc", int_lit(1)))],
    );

    let elaborated = elaborate_typed_comprehension(&env, &expr).expect("elaborates");
    let CoreExpr::Call {
        func,
        module,
        arguments,
    } = elaborated.expr
    else {
        panic!("expected outer proc::bind call");
    };
    assert_eq!(module.as_deref(), Some("proc"));
    assert_eq!(func, "bind");
    assert_eq!(arguments.len(), 2);
}
