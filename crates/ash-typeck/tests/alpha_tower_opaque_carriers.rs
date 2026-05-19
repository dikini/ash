use ash_parser::surface::{DoStmt, DoTarget, Expr, Literal};
use ash_parser::token::Span;
use ash_typeck::check_expr::check_expr;
use ash_typeck::{Kind, PublicTowerManifestKind, QualifiedName, Type, TypeEnv};

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

fn error_text(result: ash_typeck::check_expr::CheckResult) -> String {
    result
        .errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn proc_requires_explicit_from_act_lift() {
    let env = TypeEnv::with_builtin_types();
    let direct_act = do_block(
        "Proc",
        vec![
            bind_stmt("value", do_block("Act", vec![ret(int_lit(1))])),
            ret(var("value")),
        ],
    );

    let direct_result = check_expr(&env, &direct_act);
    assert!(
        !direct_result.is_ok(),
        "direct Act RHS in do:Proc must not be implicitly lifted"
    );
    let direct_text = error_text(direct_result);
    assert!(direct_text.contains("do:Proc"), "{direct_text}");
    assert!(direct_text.contains("Act<Int>"), "{direct_text}");
    assert!(direct_text.contains("proc::from_act"), "{direct_text}");

    let explicit = do_block(
        "Proc",
        vec![
            bind_stmt(
                "value",
                call(
                    Some("proc"),
                    "from_act",
                    vec![do_block("Act", vec![ret(int_lit(1))])],
                ),
            ),
            ret(var("value")),
        ],
    );
    let explicit_result = check_expr(&env, &explicit);
    assert!(
        explicit_result.is_ok(),
        "explicit proc::from_act lift must remain accepted: {explicit_result:?}"
    );
    assert_eq!(
        explicit_result.substitution.apply(&explicit_result.ty),
        computation_type("Proc", Type::Int)
    );
}

#[test]
fn workflow_requires_explicit_from_proc_or_from_act_lift() {
    let env = TypeEnv::with_builtin_types();

    let direct_proc = do_block(
        "Workflow",
        vec![
            bind_stmt("value", call(Some("proc"), "unit", vec![int_lit(1)])),
            ret(var("value")),
        ],
    );
    let direct_proc_result = check_expr(&env, &direct_proc);
    assert!(
        !direct_proc_result.is_ok(),
        "direct Proc RHS in do:Workflow must not be implicitly lifted"
    );
    let direct_proc_text = error_text(direct_proc_result);
    assert!(
        direct_proc_text.contains("do:Workflow"),
        "{direct_proc_text}"
    );
    assert!(
        direct_proc_text.contains("workflow::from_proc"),
        "{direct_proc_text}"
    );

    let direct_act = do_block(
        "Workflow",
        vec![
            bind_stmt("value", do_block("Act", vec![ret(int_lit(1))])),
            ret(var("value")),
        ],
    );
    let direct_act_result = check_expr(&env, &direct_act);
    assert!(
        !direct_act_result.is_ok(),
        "direct Act RHS in do:Workflow must not be implicitly lifted"
    );
    let direct_act_text = error_text(direct_act_result);
    assert!(direct_act_text.contains("do:Workflow"), "{direct_act_text}");
    assert!(
        direct_act_text.contains("workflow::from_act"),
        "{direct_act_text}"
    );

    for (label, lifted) in [
        (
            "workflow::from_proc",
            call(
                Some("workflow"),
                "from_proc",
                vec![call(Some("proc"), "unit", vec![int_lit(1)])],
            ),
        ),
        (
            "workflow::from_act",
            call(
                Some("workflow"),
                "from_act",
                vec![do_block("Act", vec![ret(int_lit(1))])],
            ),
        ),
    ] {
        let explicit = do_block(
            "Workflow",
            vec![bind_stmt("value", lifted), ret(var("value"))],
        );
        let explicit_result = check_expr(&env, &explicit);
        assert!(
            explicit_result.is_ok(),
            "{label} explicit lift must remain accepted: {explicit_result:?}"
        );
        assert_eq!(
            explicit_result.substitution.apply(&explicit_result.ty),
            computation_type("Workflow", Type::Int)
        );
    }
}

#[test]
fn act_env_and_process_identity_remain_non_denotable() {
    let env = TypeEnv::with_builtin_types();

    assert!(
        !env.has_type("ActEnv"),
        "ActEnv is hidden runtime environment state, not an ordinary Ash type"
    );
    assert!(
        env.lookup_type("ActEnv").is_none(),
        "ActEnv must not be exposed through ordinary type lookup"
    );
    assert!(
        env.lookup_type_info("ActEnv").is_none(),
        "ActEnv must not have a source-denotable TypeInfo"
    );
    assert!(
        env.lookup_constructor("ActEnv").is_none(),
        "ActEnv must not have a user data constructor"
    );

    assert!(
        !env.has_type("ProcessId"),
        "process identity is runtime-owned and not ordinary user data"
    );
    assert!(
        !env.has_type("ProcessHandle"),
        "runtime process handles are represented only by public opaque P<T>"
    );
    assert!(env.lookup_type("ProcessId").is_none());
    assert!(env.lookup_type("ProcessHandle").is_none());
    assert!(env.lookup_constructor("ProcessId").is_none());
    assert!(env.lookup_constructor("ProcessHandle").is_none());
    assert!(env.lookup_constructor("P").is_none());

    let manifest = env.public_tower_manifest();
    let p = manifest
        .algebra("P")
        .expect("public tower manifest records P<T>");
    assert_eq!(p.kind, PublicTowerManifestKind::ProcessHandle);
    assert!(p.nameable);
    assert!(p.typeable);
    assert!(!p.user_constructible);
    assert!(
        env.has_type("P"),
        "P<T> remains the public typeable process-handle surface"
    );
}
