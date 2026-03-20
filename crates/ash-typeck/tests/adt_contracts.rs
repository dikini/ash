use ash_core::ast::{Pattern as CorePattern, TypeBody, TypeDef, TypeExpr, VariantDef, Visibility};
use ash_parser::surface::{Literal, Pattern as ParserPattern};
use ash_typeck::check_pattern::{TypeEnv, check_pattern};
use ash_typeck::exhaustiveness::{Coverage, check_exhaustive};
use ash_typeck::types::{Type, TypeVar};

fn option_type_def() -> TypeDef {
    TypeDef {
        name: "Option".to_string(),
        params: vec![],
        body: TypeBody::Enum(vec![
            VariantDef {
                name: "Some".to_string(),
                fields: vec![("value".to_string(), TypeExpr::Named("Int".to_string()))],
            },
            VariantDef {
                name: "None".to_string(),
                fields: vec![],
            },
        ]),
        visibility: Visibility::Public,
    }
}

fn option_env() -> TypeEnv {
    let mut env = TypeEnv::new();
    env.add_type_def("Option".to_string(), option_type_def());
    env
}

#[test]
fn variant_patterns_bind_field_types_from_constructor_metadata() {
    let env = option_env();
    let pattern = ParserPattern::Variant {
        name: "Some".into(),
        fields: Some(vec![("value".into(), ParserPattern::Variable("x".into()))]),
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
            ParserPattern::Variable("x".into()),
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
    };

    let error = check_pattern(&env, &pattern, &Type::Var(TypeVar::fresh())).unwrap_err();
    let message = error.to_string();

    assert!(message.contains("unknown field"));
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
