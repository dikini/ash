use ash_core::ast::{TypeBody, TypeDef, TypeExpr, VariantDef, VariantPayload, Visibility};
use ash_parser::surface::{BlockStmt, Expr, Literal, Pattern, VariantPatternPayload};
use ash_parser::token::Span;
use ash_typeck::check_expr::{CheckResult, check_expr};
use ash_typeck::error::ConstructorError;
use ash_typeck::{Kind, QualifiedName, Type, TypeEnv};

fn span() -> Span {
    Span::default()
}

fn option_type() -> TypeDef {
    TypeDef {
        name: "Option".into(),
        params: vec!["T".into()],
        body: TypeBody::Enum(vec![
            VariantDef {
                name: "Some".into(),
                fields: vec![("value".into(), TypeExpr::Named("T".into()))],
                payload: VariantPayload::Record(vec![(
                    "value".into(),
                    TypeExpr::Named("T".into()),
                )]),
            },
            VariantDef {
                name: "None".into(),
                fields: vec![],
                payload: VariantPayload::Unit,
            },
        ]),
        visibility: Visibility::Public,
        builtin: false,
    }
}

fn one_type() -> TypeDef {
    TypeDef {
        name: "One".into(),
        params: vec![],
        body: TypeBody::Enum(vec![VariantDef {
            name: "Only".into(),
            fields: vec![("value".into(), TypeExpr::Named("Int".into()))],
            payload: VariantPayload::Record(vec![("value".into(), TypeExpr::Named("Int".into()))]),
        }]),
        visibility: Visibility::Public,
        builtin: false,
    }
}

fn env() -> TypeEnv {
    let mut env = TypeEnv::new();
    env.register_type(&option_type()).expect("register Option");
    env.register_type(&one_type()).expect("register One");
    env
}

fn constructor_ty(name: &str, args: Vec<Type>) -> Type {
    Type::Constructor {
        name: QualifiedName::root(name),
        args,
        kind: Kind::Type,
    }
}

fn int(value: i64) -> Expr {
    Expr::Literal(Literal::Int(value))
}

fn string(value: &str) -> Expr {
    Expr::Literal(Literal::String(value.into()))
}

fn bool_lit(value: bool) -> Expr {
    Expr::Literal(Literal::Bool(value))
}

fn variable(name: &str) -> Expr {
    Expr::Variable {
        name: name.into(),
        span: span(),
    }
}

fn var_pattern(name: &str) -> Pattern {
    Pattern::Variable {
        name: name.into(),
        span: span(),
    }
}

fn some_pattern(binding: &str) -> Pattern {
    Pattern::Variant {
        name: "Some".into(),
        fields: Some(vec![("value".into(), var_pattern(binding))]),
        payload: VariantPatternPayload::Record(vec![("value".into(), var_pattern(binding))]),
    }
}

fn only_pattern(binding: &str) -> Pattern {
    Pattern::Variant {
        name: "Only".into(),
        fields: Some(vec![("value".into(), var_pattern(binding))]),
        payload: VariantPatternPayload::Record(vec![("value".into(), var_pattern(binding))]),
    }
}

fn if_let(pattern: Pattern, scrutinee: Expr, then_branch: Expr, else_branch: Expr) -> Expr {
    Expr::IfLet {
        pattern,
        expr: Box::new(scrutinee),
        then_branch: Box::new(then_branch),
        else_branch: Box::new(else_branch),
        span: span(),
    }
}

fn errors_text(errors: &[ConstructorError]) -> String {
    errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

fn assert_error_contains(checked: &CheckResult, expected: &[&str]) {
    let text = errors_text(&checked.errors);
    for needle in expected {
        assert!(
            text.contains(needle),
            "expected diagnostic to contain {needle:?}, got:\n{text}"
        );
    }
}

#[test]
fn if_let_check_pattern_errors_are_propagated_not_silent() {
    let mut env = env();
    env.bind_variable("maybe", constructor_ty("Option", vec![Type::Int]));

    let checked = check_expr(
        &env,
        &if_let(
            Pattern::Variant {
                name: "NotOption".into(),
                fields: None,
                payload: VariantPatternPayload::Unit,
            },
            variable("maybe"),
            int(1),
            int(0),
        ),
    );

    assert!(!checked.is_ok(), "pattern type errors must be fatal");
    assert_error_contains(&checked, &["if let pattern", "NotOption"]);
}

#[test]
fn if_let_then_binding_scope_does_not_escape() {
    let mut env = env();
    env.bind_variable("maybe", constructor_ty("Option", vec![Type::Int]));

    let checked = check_expr(
        &env,
        &Expr::Block {
            statements: vec![BlockStmt::Let {
                pattern: Pattern::Wildcard,
                expr: if_let(
                    some_pattern("value"),
                    variable("maybe"),
                    variable("value"),
                    int(0),
                ),
                span: span(),
            }],
            tail_expr: Some(Box::new(variable("value"))),
            span: span(),
        },
    );

    assert!(!checked.is_ok(), "then-only binding must not escape");
    assert_error_contains(&checked, &["unbound variable", "value"]);
}

#[test]
fn if_let_shadowing_then_uses_inner_else_uses_outer() {
    let mut env = env();
    env.bind_variable("maybe", constructor_ty("Option", vec![Type::String]));
    env.bind_variable("value", Type::Int);

    let checked = check_expr(
        &env,
        &if_let(
            some_pattern("value"),
            variable("maybe"),
            variable("value"),
            variable("value"),
        ),
    );

    assert!(
        !checked.is_ok(),
        "inner String then binding and outer Int else binding should force a mismatch"
    );
    assert_error_contains(&checked, &["if let branch type mismatch", "String", "Int"]);
}

#[test]
fn if_let_branch_type_mismatch_is_reported() {
    let mut env = env();
    env.bind_variable("maybe", constructor_ty("Option", vec![Type::Int]));

    let checked = check_expr(
        &env,
        &if_let(
            some_pattern("value"),
            variable("maybe"),
            variable("value"),
            string("none"),
        ),
    );

    assert!(!checked.is_ok(), "branch mismatch must be reported");
    assert_error_contains(&checked, &["if let branch type mismatch", "Int", "String"]);
}

#[test]
fn if_let_duplicate_binders_rejected() {
    let mut env = env();
    env.bind_variable(
        "pair",
        Type::Record(vec![
            (Box::from("0"), Type::Int),
            (Box::from("1"), Type::Int),
        ]),
    );

    let checked = check_expr(
        &env,
        &if_let(
            Pattern::Tuple(vec![var_pattern("x"), var_pattern("x")]),
            variable("pair"),
            int(1),
            int(0),
        ),
    );

    assert!(!checked.is_ok(), "duplicate binders must be rejected");
    assert_error_contains(&checked, &["if let pattern", "duplicate", "x"]);
}

#[test]
fn if_let_irrefutable_pattern_emits_unreachable_else_warning() {
    let mut env = env();
    env.bind_variable("one", constructor_ty("One", vec![]));

    let checked = check_expr(
        &env,
        &if_let(
            only_pattern("value"),
            variable("one"),
            variable("value"),
            int(0),
        ),
    );

    assert!(
        checked.is_ok(),
        "irrefutable if let should remain accepted despite warning: {:?}",
        checked.errors
    );
    assert_eq!(checked.ty, Type::Int);
    assert_error_contains(&checked, &["unreachable if let else", "Only"]);
}

#[test]
fn if_let_irrefutable_warning_survives_parent_expressions_without_type_poisoning() {
    let mut env = env();
    env.bind_variable("one", constructor_ty("One", vec![]));

    let block_tail = check_expr(
        &env,
        &Expr::Block {
            statements: vec![],
            tail_expr: Some(Box::new(if_let(
                only_pattern("value"),
                variable("one"),
                variable("value"),
                int(0),
            ))),
            span: span(),
        },
    );
    assert!(
        block_tail.is_ok(),
        "block-tail warning-only if let should remain successful: {:?}",
        block_tail.errors
    );
    assert_eq!(block_tail.ty, Type::Int);
    assert_error_contains(&block_tail, &["unreachable if let else", "Only"]);

    let parent_if = check_expr(
        &env,
        &Expr::If {
            condition: Box::new(bool_lit(true)),
            then_branch: Box::new(if_let(
                only_pattern("value"),
                variable("one"),
                variable("value"),
                int(0),
            )),
            else_branch: Some(Box::new(int(1))),
            span: span(),
        },
    );
    assert!(
        parent_if.is_ok(),
        "parent if should preserve warning but keep real branch type: {:?}",
        parent_if.errors
    );
    assert_eq!(parent_if.ty, Type::Int);
    assert_error_contains(&parent_if, &["unreachable if let else", "Only"]);

    let fn_apply = check_expr(
        &env,
        &Expr::FnApply {
            func: Box::new(Expr::FnDef {
                params: vec![("x".into(), Some("Int".into()))],
                return_type: Some("Int".into()),
                body: Box::new(variable("x")),
                span: span(),
            }),
            args: vec![if_let(
                only_pattern("value"),
                variable("one"),
                variable("value"),
                int(0),
            )],
            span: span(),
        },
    );
    assert!(
        fn_apply.is_ok(),
        "FnApply should preserve warning but keep real return type: {:?}",
        fn_apply.errors
    );
    assert_eq!(fn_apply.ty, Type::Int);
    assert_error_contains(&fn_apply, &["unreachable if let else", "Only"]);
}

#[test]
fn if_let_impossible_pattern_is_hard_error() {
    let mut env = env();
    env.bind_variable("number", Type::Int);

    let checked = check_expr(
        &env,
        &if_let(some_pattern("value"), variable("number"), int(1), int(0)),
    );

    assert!(!checked.is_ok(), "impossible if let pattern must be fatal");
    assert_error_contains(&checked, &["if let pattern", "impossible", "Some"]);
}
