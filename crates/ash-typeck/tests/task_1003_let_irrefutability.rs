use ash_core::ast::{
    Expr as CoreExpr, Pattern as CorePattern, Span as CoreSpan, TypeBody, TypeDef, TypeExpr,
    VariantDef, VariantPayload, Visibility,
};
use ash_parser::surface::{BlockStmt, Expr, Literal, Pattern, VariantPatternPayload};
use ash_parser::token::Span;
use ash_typeck::check_expr::{check_core_expr, check_expr};
use ash_typeck::error::ConstructorError;
use ash_typeck::{Kind, QualifiedName, Type, TypeEnv, TypeVar};

fn span() -> Span {
    Span::default()
}

fn custom_span(start: usize) -> Span {
    Span::new(start, start + 1, 1, start + 1)
}

fn core_span() -> CoreSpan {
    CoreSpan::default()
}

fn custom_core_span(start: usize) -> CoreSpan {
    CoreSpan {
        start,
        end: start + 1,
    }
}

fn var_pattern(name: &str) -> Pattern {
    Pattern::Variable {
        name: name.into(),
        span: span(),
    }
}

fn some_pattern(inner: Pattern) -> Pattern {
    Pattern::Variant {
        name: "Some".into(),
        fields: Some(vec![("value".into(), inner.clone())]),
        payload: VariantPatternPayload::Record(vec![("value".into(), inner)]),
    }
}

fn just_pattern(inner: Pattern) -> Pattern {
    Pattern::Variant {
        name: "Just".into(),
        fields: Some(vec![("value".into(), inner.clone())]),
        payload: VariantPatternPayload::Record(vec![("value".into(), inner)]),
    }
}

fn record_variant_pattern(name: &str, field: &str, pattern: Pattern) -> Pattern {
    Pattern::Variant {
        name: name.into(),
        fields: Some(vec![(field.into(), pattern.clone())]),
        payload: VariantPatternPayload::Record(vec![(field.into(), pattern)]),
    }
}

fn variable(name: &str) -> Expr {
    Expr::Variable {
        name: name.into(),
        span: span(),
    }
}

fn int(value: i64) -> Expr {
    Expr::Literal(Literal::Int(value))
}

fn block_let(pattern: Pattern, expr: Expr, tail: Expr) -> Expr {
    Expr::Block {
        statements: vec![BlockStmt::Let {
            pattern,
            expr,
            span: span(),
        }],
        tail_expr: Some(Box::new(tail)),
        span: span(),
    }
}

fn option_int() -> Type {
    Type::Constructor {
        name: QualifiedName::root("Option"),
        args: vec![Type::Int],
        kind: Kind::Type,
    }
}

fn constructor_ty(name: &str) -> Type {
    Type::Constructor {
        name: QualifiedName::root(name),
        args: vec![],
        kind: Kind::Type,
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

fn tuple_one_type() -> TypeDef {
    TypeDef {
        name: "TupleOne".into(),
        params: vec![],
        body: TypeBody::Enum(vec![VariantDef {
            name: "TupleOnly".into(),
            fields: vec![("_0".into(), TypeExpr::Named("Int".into()))],
            payload: VariantPayload::Tuple(vec![TypeExpr::Named("Int".into())]),
        }]),
        visibility: Visibility::Public,
        builtin: false,
    }
}

fn maybe_int_type() -> TypeDef {
    TypeDef {
        name: "MaybeInt".into(),
        params: vec![],
        body: TypeBody::Enum(vec![
            VariantDef {
                name: "Just".into(),
                fields: vec![("value".into(), TypeExpr::Named("Int".into()))],
                payload: VariantPayload::Record(vec![(
                    "value".into(),
                    TypeExpr::Named("Int".into()),
                )]),
            },
            VariantDef {
                name: "Empty".into(),
                fields: vec![],
                payload: VariantPayload::Unit,
            },
        ]),
        visibility: Visibility::Public,
        builtin: false,
    }
}

fn nested_one_type() -> TypeDef {
    TypeDef {
        name: "NestedOne".into(),
        params: vec![],
        body: TypeBody::Enum(vec![VariantDef {
            name: "NestedOnly".into(),
            fields: vec![("maybe".into(), TypeExpr::Named("MaybeInt".into()))],
            payload: VariantPayload::Record(vec![(
                "maybe".into(),
                TypeExpr::Named("MaybeInt".into()),
            )]),
        }]),
        visibility: Visibility::Public,
        builtin: false,
    }
}

fn env() -> TypeEnv {
    let mut env = TypeEnv::with_builtin_types();
    env.register_type(&one_type()).expect("register One");
    env.register_type(&maybe_int_type())
        .expect("register MaybeInt");
    env.register_type(&nested_one_type())
        .expect("register NestedOne");
    env.register_type(&tuple_one_type())
        .expect("register TupleOne");
    env
}

fn errors_text(errors: &[ConstructorError]) -> String {
    errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

fn assert_non_irrefutable_let(message: &str) {
    assert!(message.contains("let"), "{message}");
    assert!(message.contains("irrefutable"), "{message}");
    assert!(message.contains("use match or if let"), "{message}");
}

fn first_error_span(errors: &[ConstructorError]) -> Span {
    match errors.first().expect("expected an error") {
        ConstructorError::UnsupportedExpression { span, .. } => *span,
        other => panic!("expected unsupported expression error, got {other:?}"),
    }
}

#[test]
fn pure_block_let_rejects_some_over_option() {
    let mut env = env();
    env.bind_variable("maybe", option_int());

    let checked = check_expr(
        &env,
        &block_let(
            some_pattern(var_pattern("value")),
            variable("maybe"),
            variable("value"),
        ),
    );

    assert!(!checked.is_ok(), "refutable let should fail");
    let message = errors_text(&checked.errors);
    assert_non_irrefutable_let(&message);
    assert!(message.contains("Some"), "{message}");
    assert!(message.contains("None"), "{message}");
}

#[test]
fn pure_block_let_diagnostic_uses_pattern_span() {
    let mut env = env();
    env.bind_variable("maybe", option_int());

    let pattern = Pattern::Variant {
        name: "Some".into(),
        fields: Some(vec![(
            "value".into(),
            Pattern::Variable {
                name: "value".into(),
                span: custom_span(41),
            },
        )]),
        payload: VariantPatternPayload::Record(vec![(
            "value".into(),
            Pattern::Variable {
                name: "value".into(),
                span: custom_span(41),
            },
        )]),
    };
    let checked = check_expr(
        &env,
        &block_let(pattern, variable("maybe"), variable("value")),
    );

    assert!(!checked.is_ok(), "refutable let should fail");
    assert_eq!(first_error_span(&checked.errors), custom_span(41));
}

#[test]
fn pure_block_let_rejects_nested_refutable_binders() {
    let mut env = env();
    env.bind_variable("nested", constructor_ty("NestedOne"));

    let checked = check_expr(
        &env,
        &block_let(
            record_variant_pattern("NestedOnly", "maybe", just_pattern(var_pattern("value"))),
            variable("nested"),
            variable("value"),
        ),
    );

    assert!(!checked.is_ok(), "nested refutable let should fail");
    let message = errors_text(&checked.errors);
    assert_non_irrefutable_let(&message);
    assert!(message.contains("NestedOnly"), "{message}");
    assert!(message.contains("Empty"), "{message}");
}

#[test]
fn pure_block_let_rejects_list_patterns() {
    let mut env = env();
    env.bind_variable("items", Type::List(Box::new(Type::Int)));

    let checked = check_expr(
        &env,
        &block_let(
            Pattern::List {
                elements: vec![var_pattern("head")],
                rest: None,
            },
            variable("items"),
            variable("head"),
        ),
    );

    assert!(!checked.is_ok(), "fixed-prefix list let should fail");
    let message = errors_text(&checked.errors);
    assert_non_irrefutable_let(&message);
    assert!(
        message.contains("ShortList") || message.contains("short list"),
        "{message}"
    );
}

#[test]
fn pure_block_let_reports_blocked_product_and_variant_cases() {
    let mut env = env();
    env.bind_variable("opaque_tuple", Type::Var(TypeVar(9001)));
    env.bind_variable("opaque_sum", Type::Var(TypeVar(9002)));

    let tuple_checked = check_expr(
        &env,
        &block_let(
            Pattern::Tuple(vec![var_pattern("x")]),
            variable("opaque_tuple"),
            variable("x"),
        ),
    );
    assert!(!tuple_checked.is_ok(), "opaque tuple shape should block");
    let tuple_message = errors_text(&tuple_checked.errors);
    assert_non_irrefutable_let(&tuple_message);
    assert!(tuple_message.contains("blocked"), "{tuple_message}");
    assert!(
        tuple_message.contains("product shape")
            || tuple_message.contains("ProductShapeUnavailable"),
        "{tuple_message}"
    );

    let variant_checked = check_expr(
        &env,
        &block_let(
            record_variant_pattern("Only", "value", var_pattern("value")),
            variable("opaque_sum"),
            variable("value"),
        ),
    );
    assert!(
        !variant_checked.is_ok(),
        "opaque variant universe should block"
    );
    let variant_message = errors_text(&variant_checked.errors);
    assert_non_irrefutable_let(&variant_message);
    assert!(variant_message.contains("blocked"), "{variant_message}");
    assert!(
        variant_message.contains("constructor universe")
            || variant_message.contains("ConstructorUniverseUnavailable")
            || variant_message.contains("canonicalization"),
        "{variant_message}"
    );
}

#[test]
fn core_expr_let_rejects_refutable_host_ir() {
    let mut env = env();
    env.bind_variable("maybe", option_int());

    let checked = check_core_expr(
        &env,
        &CoreExpr::Let {
            pattern: CorePattern::Variant {
                name: "Some".into(),
                fields: Some(vec![(
                    "value".into(),
                    CorePattern::Variable {
                        name: "value".into(),
                        span: core_span(),
                    },
                )]),
            },
            expr: Box::new(CoreExpr::Variable {
                name: "maybe".into(),
                span: core_span(),
            }),
            body: Box::new(CoreExpr::Variable {
                name: "value".into(),
                span: core_span(),
            }),
            span: core_span(),
        },
    );

    assert!(!checked.is_ok(), "host-created core let should fail");
    let message = errors_text(&checked.errors);
    assert!(message.contains("core let"), "{message}");
    assert!(message.contains("irrefutable"), "{message}");
    assert!(message.contains("Some"), "{message}");
    assert!(message.contains("None"), "{message}");
    assert!(message.contains("use match or if let"), "{message}");
}

#[test]
fn core_expr_let_diagnostic_uses_core_let_span() {
    let mut env = env();
    env.bind_variable("maybe", option_int());

    let checked = check_core_expr(
        &env,
        &CoreExpr::Let {
            pattern: CorePattern::Variant {
                name: "Some".into(),
                fields: Some(vec![(
                    "value".into(),
                    CorePattern::Variable {
                        name: "value".into(),
                        span: custom_core_span(11),
                    },
                )]),
            },
            expr: Box::new(CoreExpr::Variable {
                name: "maybe".into(),
                span: custom_core_span(13),
            }),
            body: Box::new(CoreExpr::Variable {
                name: "value".into(),
                span: custom_core_span(17),
            }),
            span: custom_core_span(19),
        },
    );

    assert!(!checked.is_ok(), "host-created core let should fail");
    assert_eq!(first_error_span(&checked.errors).start, 19);
}

#[test]
fn core_expr_constructor_rejects_malformed_payload_before_binding() {
    let checked = check_core_expr(
        &env(),
        &CoreExpr::Let {
            pattern: CorePattern::Variant {
                name: "Only".into(),
                fields: Some(vec![(
                    "value".into(),
                    CorePattern::Variable {
                        name: "value".into(),
                        span: core_span(),
                    },
                )]),
            },
            expr: Box::new(CoreExpr::Constructor {
                name: "Only".into(),
                fields: vec![(
                    "value".into(),
                    CoreExpr::Literal(ash_core::Value::Bool(true)),
                )],
            }),
            body: Box::new(CoreExpr::Variable {
                name: "value".into(),
                span: core_span(),
            }),
            span: core_span(),
        },
    );

    assert!(!checked.is_ok(), "malformed core constructor should fail");
    let message = errors_text(&checked.errors);
    assert!(message.contains("value"), "{message}");
    assert!(message.contains("Int"), "{message}");
    assert!(message.contains("Bool"), "{message}");
}

#[test]
fn core_expr_variant_literal_rejects_malformed_payload_before_binding() {
    let checked = check_core_expr(
        &env(),
        &CoreExpr::Let {
            pattern: CorePattern::Variant {
                name: "Only".into(),
                fields: Some(vec![(
                    "value".into(),
                    CorePattern::Variable {
                        name: "value".into(),
                        span: core_span(),
                    },
                )]),
            },
            expr: Box::new(CoreExpr::Literal(ash_core::Value::Variant {
                name: "Only".into(),
                fields: Box::new(vec![("value".into(), ash_core::Value::Bool(true))]),
            })),
            body: Box::new(CoreExpr::Variable {
                name: "value".into(),
                span: core_span(),
            }),
            span: core_span(),
        },
    );

    assert!(!checked.is_ok(), "malformed variant literal should fail");
    let message = errors_text(&checked.errors);
    assert!(message.contains("value"), "{message}");
    assert!(message.contains("Int"), "{message}");
    assert!(message.contains("Bool"), "{message}");
}

#[test]
fn core_expr_variant_literal_rejects_unknown_constructor() {
    let checked = check_core_expr(
        &env(),
        &CoreExpr::Literal(ash_core::Value::Variant {
            name: "Missing".into(),
            fields: Box::new(vec![]),
        }),
    );

    assert!(!checked.is_ok(), "unknown variant literal should fail");
    let message = errors_text(&checked.errors);
    assert!(message.contains("Missing"), "{message}");
}

#[test]
fn core_expr_constructor_rejects_duplicate_record_fields() {
    let checked = check_core_expr(
        &env(),
        &CoreExpr::Constructor {
            name: "Only".into(),
            fields: vec![
                ("value".into(), CoreExpr::Literal(ash_core::Value::Int(1))),
                ("value".into(), CoreExpr::Literal(ash_core::Value::Int(2))),
            ],
        },
    );

    assert!(
        !checked.is_ok(),
        "duplicate core constructor fields should fail"
    );
    let message = errors_text(&checked.errors);
    assert!(message.contains("duplicate"), "{message}");
    assert!(message.contains("value"), "{message}");
}

#[test]
fn core_expr_let_accepts_tuple_variant_payload_shape() {
    let mut env = env();
    env.bind_variable("tuple_one", constructor_ty("TupleOne"));

    let checked = check_core_expr(
        &env,
        &CoreExpr::Let {
            pattern: CorePattern::Variant {
                name: "TupleOnly".into(),
                fields: Some(vec![(
                    "_0".into(),
                    CorePattern::Variable {
                        name: "value".into(),
                        span: core_span(),
                    },
                )]),
            },
            expr: Box::new(CoreExpr::Variable {
                name: "tuple_one".into(),
                span: core_span(),
            }),
            body: Box::new(CoreExpr::Variable {
                name: "value".into(),
                span: core_span(),
            }),
            span: core_span(),
        },
    );

    assert!(checked.is_ok(), "{checked:?}");
    assert_eq!(checked.ty, Type::Int);
}

#[test]
fn pure_block_let_accepts_variable_wildcard_and_single_variant() {
    let mut env = env();
    env.bind_variable("one", constructor_ty("One"));

    let variable_checked = check_expr(&env, &block_let(var_pattern("x"), int(1), variable("x")));
    assert!(variable_checked.is_ok(), "{variable_checked:?}");

    let wildcard_checked = check_expr(
        &env,
        &block_let(Pattern::Wildcard, int(1), Expr::Literal(Literal::Int(2))),
    );
    assert!(wildcard_checked.is_ok(), "{wildcard_checked:?}");

    let single_variant_checked = check_expr(
        &env,
        &block_let(
            record_variant_pattern("Only", "value", var_pattern("value")),
            variable("one"),
            variable("value"),
        ),
    );
    assert!(single_variant_checked.is_ok(), "{single_variant_checked:?}");
    assert_eq!(single_variant_checked.ty, Type::Int);
}

#[test]
fn pure_block_let_duplicate_binders_rejected() {
    let mut env = env();
    env.bind_variable(
        "pair",
        Type::Record(vec![("_0".into(), Type::Int), ("_1".into(), Type::Int)]),
    );

    let checked = check_expr(
        &env,
        &block_let(
            Pattern::Tuple(vec![var_pattern("x"), var_pattern("x")]),
            variable("pair"),
            variable("x"),
        ),
    );

    assert!(!checked.is_ok(), "duplicate binders should fail");
    let message = errors_text(&checked.errors);
    assert_non_irrefutable_let(&message);
    assert!(
        message.contains("duplicate") || message.contains("Duplicate"),
        "{message}"
    );
    assert!(message.contains("x"), "{message}");
}
