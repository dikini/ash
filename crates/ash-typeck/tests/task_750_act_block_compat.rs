use ash_parser::surface::{DoStmt, DoTarget, Expr, Literal};
use ash_parser::token::Span;
use ash_typeck::check_expr::{check_expr, legacy_act_migration_diagnostics};
use ash_typeck::{Kind, QualifiedName, Type, TypeEnv};

fn span() -> Span {
    Span::default()
}

fn computation_type(name: &str, inner: Type) -> Type {
    Type::Constructor {
        name: QualifiedName::root(name),
        args: vec![inner],
        kind: Kind::Type,
    }
}

fn bind_act_unit(env: &mut TypeEnv) {
    env.bind_variable(
        "act::unit",
        Type::Fn(
            vec![Type::Int],
            Box::new(computation_type("Act", Type::Int)),
        ),
    );
}

fn parse_expr(source: &str) -> Expr {
    let mut input = ash_parser::new_input(source);
    ash_parser::parse_expr::expr(&mut input).expect(source)
}

#[test]
fn new_act_block_sugar_parses_as_do_act() {
    let expr = parse_expr("act { x <- act::unit(1); return x }");

    match expr {
        Expr::DoBlock { target, stmts, .. } => {
            assert_eq!(target.name.as_ref(), "Act");
            assert!(target.args.is_empty());
            assert_eq!(stmts.len(), 2);
            assert!(matches!(&stmts[0], DoStmt::Bind { name, .. } if name.as_ref() == "x"));
            assert!(
                matches!(&stmts[1], DoStmt::Return { value, .. } if matches!(value.as_ref(), Expr::Variable { name, .. } if name.as_ref() == "x"))
            );
        }
        other => panic!("new-form act block should parse as DoBlock sugar, got {other:?}"),
    }
}

#[test]
fn new_act_block_sugar_typechecks_like_do_act() {
    let mut env = TypeEnv::with_builtin_types();
    bind_act_unit(&mut env);

    let act_sugar = parse_expr("act { x <- act::unit(1); return x }");
    let do_act = Expr::DoBlock {
        target: DoTarget {
            name: "Act".into(),
            args: vec![],
            span: span(),
        },
        stmts: vec![
            DoStmt::Bind {
                name: "x".into(),
                value: Box::new(Expr::Call {
                    func: "unit".into(),
                    module: Some("act".into()),
                    args: vec![Expr::Literal(Literal::Int(1))],
                    span: span(),
                }),
                span: span(),
            },
            DoStmt::Return {
                value: Box::new(Expr::Variable {
                    name: "x".into(),
                    span: span(),
                }),
                span: span(),
            },
        ],
        span: span(),
    };

    let sugar_result = check_expr(&env, &act_sugar);
    let do_result = check_expr(&env, &do_act);

    assert!(sugar_result.is_ok(), "act sugar failed: {sugar_result:?}");
    assert!(do_result.is_ok(), "do:Act failed: {do_result:?}");
    assert_eq!(sugar_result.ty, computation_type("Act", Type::Int));
    assert_eq!(sugar_result.ty, do_result.ty);
}

#[test]
fn legacy_act_block_still_typechecks_and_carries_migration_diagnostics() {
    let mut env = TypeEnv::with_builtin_types();
    bind_act_unit(&mut env);

    let legacy = parse_expr("act { x = act::unit(1); ret x; }");
    assert!(
        matches!(legacy, Expr::ActBlock { .. }),
        "legacy act should keep compatibility carrier"
    );

    let result = check_expr(&env, &legacy);
    assert!(
        result.is_ok(),
        "legacy act block should remain compatible: {result:?}"
    );
    assert_eq!(result.ty, computation_type("Act", Type::Int));

    let diagnostics = legacy_act_migration_diagnostics(&legacy);
    let text = diagnostics.join("\n");
    assert!(text.contains("legacy act bind"), "{text}");
    assert!(text.contains("x <-"), "{text}");
    assert!(text.contains("ret"), "{text}");
    assert!(text.contains("return"), "{text}");
}
