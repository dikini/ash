use ash_core::ast::{
    Pattern as CorePattern, TypeBody, TypeDef, TypeExpr, VariantDef, VariantPayload, Visibility,
};
use ash_parser::surface::{Expr, Literal, Pattern as ParserPattern, VariantPatternPayload};
use ash_parser::token::Span;
use ash_typeck::check_expr::check_expr;
use ash_typeck::check_pattern::{TypeEnv, check_pattern};
use ash_typeck::exhaustiveness::{Coverage, check_exhaustive};
use ash_typeck::type_env::TypeEnv as ExprTypeEnv;
use ash_typeck::types::{Type, TypeVar};

fn option_type_def() -> TypeDef {
    TypeDef {
        name: "Option".to_string(),
        params: vec![],
        body: TypeBody::Enum(vec![
            VariantDef {
                name: "Some".to_string(),
                fields: vec![("value".to_string(), TypeExpr::Named("Int".to_string()))],
                payload: VariantPayload::Record(vec![(
                    "value".to_string(),
                    TypeExpr::Named("Int".to_string()),
                )]),
            },
            VariantDef {
                name: "None".to_string(),
                fields: vec![],
                payload: VariantPayload::Unit,
            },
        ]),
        visibility: Visibility::Public,
        builtin: false,
    }
}

fn option_env() -> TypeEnv {
    let mut env = TypeEnv::new();
    env.add_type_def("Option".to_string(), option_type_def());
    env
}

fn runtime_error_type_def() -> TypeDef {
    TypeDef {
        name: "RuntimeError".to_string(),
        params: vec![],
        body: TypeBody::Enum(vec![VariantDef {
            name: "RuntimeError".to_string(),
            fields: vec![
                ("_0".to_string(), TypeExpr::Named("Int".to_string())),
                ("_1".to_string(), TypeExpr::Named("String".to_string())),
            ],
            payload: VariantPayload::Tuple(vec![
                TypeExpr::Named("Int".to_string()),
                TypeExpr::Named("String".to_string()),
            ]),
        }]),
        visibility: Visibility::Public,
        builtin: false,
    }
}

fn runtime_error_expr_env() -> ExprTypeEnv {
    let mut env = ExprTypeEnv::new();
    env.register_type(&runtime_error_type_def())
        .expect("runtime error type should register");
    env
}

fn runtime_error_pattern_env() -> TypeEnv {
    let mut env = TypeEnv::new();
    env.add_type_def("RuntimeError".to_string(), runtime_error_type_def());
    env
}

#[test]
fn variant_patterns_bind_field_types_from_constructor_metadata() {
    let env = option_env();
    let pattern = ParserPattern::Variant {
        name: "Some".into(),
        fields: Some(vec![(
            "value".into(),
            ParserPattern::Variable {
                name: "x".into(),
                span: ash_parser::token::Span::default(),
            },
        )]),
        payload: VariantPatternPayload::Record(vec![(
            "value".into(),
            ParserPattern::Variable {
                name: "x".into(),
                span: ash_parser::token::Span::default(),
            },
        )]),
    };

    let bindings = check_pattern(&env, &pattern, &Type::Var(TypeVar::fresh())).unwrap();

    assert_eq!(bindings.get("x"), Some(&Type::Int));
}

#[test]
fn variant_patterns_reject_unknown_fields_from_constructor_metadata() {
    let env = option_env();
    let pattern = ParserPattern::Variant {
        name: "Some".into(),
        fields: Some(vec![(
            "missing".into(),
            ParserPattern::Variable {
                name: "x".into(),
                span: ash_parser::token::Span::default(),
            },
        )]),
        payload: VariantPatternPayload::Record(vec![(
            "missing".into(),
            ParserPattern::Variable {
                name: "x".into(),
                span: ash_parser::token::Span::default(),
            },
        )]),
    };

    let error = check_pattern(&env, &pattern, &Type::Var(TypeVar::fresh())).unwrap_err();
    let message = error.to_string();

    assert!(message.contains("unknown field"));
}

#[test]
fn variant_patterns_reject_fields_for_unit_variants() {
    let env = option_env();
    let pattern = ParserPattern::Variant {
        name: "None".into(),
        fields: Some(vec![(
            "value".into(),
            ParserPattern::Literal(Literal::Int(42)),
        )]),
        payload: VariantPatternPayload::Record(vec![(
            "value".into(),
            ParserPattern::Literal(Literal::Int(42)),
        )]),
    };

    let error = check_pattern(&env, &pattern, &Type::Var(TypeVar::fresh())).unwrap_err();
    let message = error.to_string();

    assert!(
        message.contains("does not have record payload")
            || message.contains("unknown field")
            || message.contains("payload shape mismatch"),
        "expected unit-variant payload rejection, got: {message}"
    );
}

#[test]
fn tuple_variant_constructor_typing_succeeds_positionally() {
    let env = runtime_error_expr_env();
    let expr = Expr::Constructor {
        name: "RuntimeError".into(),
        fields: vec![
            ("_0".into(), Expr::Literal(Literal::Int(2))),
            (
                "_1".into(),
                Expr::Literal(Literal::String("missing config".into())),
            ),
        ],
        payload: ash_parser::surface::ConstructorPayload::Tuple(vec![
            Expr::Literal(Literal::Int(2)),
            Expr::Literal(Literal::String("missing config".into())),
        ]),
        span: Span::default(),
    };

    let result = check_expr(&env, &expr);

    assert!(
        result.is_ok(),
        "expected success, got errors: {:?}",
        result.errors
    );
    assert!(matches!(
        result.ty,
        Type::Constructor { ref name, .. } if name.to_string() == "RuntimeError"
    ));
}

#[test]
fn tuple_variant_constructor_rejects_arity_mismatch() {
    let env = runtime_error_expr_env();
    let expr = Expr::Constructor {
        name: "RuntimeError".into(),
        fields: vec![("_0".into(), Expr::Literal(Literal::Int(2)))],
        payload: ash_parser::surface::ConstructorPayload::Tuple(vec![Expr::Literal(Literal::Int(
            2,
        ))]),
        span: Span::default(),
    };

    let result = check_expr(&env, &expr);

    assert!(!result.is_ok(), "expected arity mismatch failure");
    assert!(
        result
            .errors
            .iter()
            .any(|error| error.to_string().contains("arity"))
    );
}

#[test]
fn tuple_variant_constructor_rejects_payload_type_mismatch() {
    let env = runtime_error_expr_env();
    let expr = Expr::Constructor {
        name: "RuntimeError".into(),
        fields: vec![
            ("_0".into(), Expr::Literal(Literal::String("wrong".into()))),
            (
                "_1".into(),
                Expr::Literal(Literal::String("missing config".into())),
            ),
        ],
        payload: ash_parser::surface::ConstructorPayload::Tuple(vec![
            Expr::Literal(Literal::String("wrong".into())),
            Expr::Literal(Literal::String("missing config".into())),
        ]),
        span: Span::default(),
    };

    let result = check_expr(&env, &expr);

    assert!(!result.is_ok(), "expected payload type mismatch failure");
    assert!(result.errors.iter().any(|error| matches!(
        error,
        ash_typeck::error::ConstructorError::TupleFieldTypeMismatch { position, .. } if *position == 0
    )));
}

#[test]
fn tuple_variant_patterns_bind_payload_by_position() {
    let env = runtime_error_pattern_env();
    let pattern = ParserPattern::Variant {
        name: "RuntimeError".into(),
        fields: None,
        payload: VariantPatternPayload::Tuple(vec![
            ParserPattern::Variable {
                name: "code".into(),
                span: ash_parser::token::Span::default(),
            },
            ParserPattern::Variable {
                name: "message".into(),
                span: ash_parser::token::Span::default(),
            },
        ]),
    };

    let bindings = check_pattern(&env, &pattern, &Type::Var(TypeVar::fresh())).unwrap();

    assert_eq!(bindings.get("code"), Some(&Type::Int));
    assert_eq!(bindings.get("message"), Some(&Type::String));
}

#[test]
fn exhaustiveness_witnesses_preserve_constructor_field_shape() {
    let patterns = vec![CorePattern::Variant {
        name: "None".into(),
        fields: None,
    }];

    let coverage = check_exhaustive(&patterns, &option_type_def());

    match coverage {
        Coverage::Missing(missing) => {
            assert_eq!(missing.len(), 1);
            assert_eq!(
                missing[0],
                CorePattern::Variant {
                    name: "Some".into(),
                    fields: Some(vec![("value".into(), CorePattern::Wildcard)]),
                }
            );
        }
        Coverage::Covered => panic!("expected missing constructor witness"),
    }
}

#[test]
fn tuple_exhaustiveness_witnesses_preserve_positional_shape() {
    let coverage = check_exhaustive(&[], &runtime_error_type_def());

    match coverage {
        Coverage::Missing(missing) => {
            assert_eq!(missing.len(), 1);
            assert_eq!(
                missing[0],
                CorePattern::Variant {
                    name: "RuntimeError".into(),
                    fields: Some(vec![
                        ("_0".into(), CorePattern::Wildcard),
                        ("_1".into(), CorePattern::Wildcard),
                    ]),
                }
            );
        }
        Coverage::Covered => panic!("expected tuple witness"),
    }
}
