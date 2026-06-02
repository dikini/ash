use ash_core::ast::{TypeBody, TypeDef, TypeExpr, VariantDef, VariantPayload, Visibility};
use ash_parser::surface::{Literal, Pattern, VariantPatternPayload};
use ash_parser::token::Span;
use ash_typeck::check_pattern::{
    IrrefutabilityImpossibleReason, IrrefutabilityOutcome, IrrefutabilityWitness,
    TypeEnv as PatternTypeEnv, check_irrefutable_pattern,
    check_irrefutable_pattern_with_canonical_type, check_irrefutable_pattern_with_canonicalization,
};
use ash_typeck::{
    Kind, PatternCanonicalization, PatternCanonicalizationBlockedReason, QualifiedName, Type,
    TypeEnv, TypeVar,
};

fn span() -> Span {
    Span::default()
}

fn var(name: &str) -> Pattern {
    Pattern::Variable {
        name: name.into(),
        span: span(),
    }
}

fn record_variant(name: &str, field_name: &str, pattern: Pattern) -> Pattern {
    Pattern::Variant {
        name: name.into(),
        fields: Some(vec![(field_name.into(), pattern.clone())]),
        payload: VariantPatternPayload::Record(vec![(field_name.into(), pattern)]),
    }
}

fn tuple_variant(name: &str, items: Vec<Pattern>) -> Pattern {
    Pattern::Variant {
        name: name.into(),
        fields: Some(
            items
                .iter()
                .enumerate()
                .map(|(index, pattern)| (index.to_string().into_boxed_str(), pattern.clone()))
                .collect(),
        ),
        payload: VariantPatternPayload::Tuple(items),
    }
}

fn constructor_ty(name: &str) -> Type {
    Type::Constructor {
        name: QualifiedName::root(name),
        args: vec![],
        kind: Kind::Type,
    }
}

fn constructor_ty_with_args(name: &str, args: Vec<Type>) -> Type {
    Type::Constructor {
        name: QualifiedName::root(name),
        args,
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

fn nested_one_type() -> TypeDef {
    TypeDef {
        name: "NestedOne".into(),
        params: vec![],
        body: TypeBody::Enum(vec![VariantDef {
            name: "NestedOnly".into(),
            fields: vec![(
                "maybe".into(),
                TypeExpr::Constructor {
                    name: "Option".into(),
                    args: vec![TypeExpr::Named("Int".into())],
                },
            )],
            payload: VariantPayload::Record(vec![(
                "maybe".into(),
                TypeExpr::Constructor {
                    name: "Option".into(),
                    args: vec![TypeExpr::Named("Int".into())],
                },
            )]),
        }]),
        visibility: Visibility::Public,
        builtin: false,
    }
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

fn pair_type() -> TypeDef {
    TypeDef {
        name: "Pair".into(),
        params: vec![],
        body: TypeBody::Enum(vec![VariantDef {
            name: "PairOnly".into(),
            fields: vec![
                ("0".into(), TypeExpr::Named("Int".into())),
                ("1".into(), TypeExpr::Named("Bool".into())),
            ],
            payload: VariantPayload::Tuple(vec![
                TypeExpr::Named("Int".into()),
                TypeExpr::Named("Bool".into()),
            ]),
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
                name: "SomeInt".into(),
                fields: vec![("value".into(), TypeExpr::Named("Int".into()))],
                payload: VariantPayload::Record(vec![(
                    "value".into(),
                    TypeExpr::Named("Int".into()),
                )]),
            },
            VariantDef {
                name: "NoInt".into(),
                fields: vec![],
                payload: VariantPayload::Unit,
            },
        ]),
        visibility: Visibility::Public,
        builtin: false,
    }
}

fn wrapper_with_user_defined_field_type() -> TypeDef {
    TypeDef {
        name: "Wrapper".into(),
        params: vec![],
        body: TypeBody::Enum(vec![VariantDef {
            name: "Wrap".into(),
            fields: vec![("inner".into(), TypeExpr::Named("MaybeInt".into()))],
            payload: VariantPayload::Record(vec![(
                "inner".into(),
                TypeExpr::Named("MaybeInt".into()),
            )]),
        }]),
        visibility: Visibility::Public,
        builtin: false,
    }
}

fn canonical_env(types: &[TypeDef]) -> TypeEnv {
    let mut env = TypeEnv::new();
    for ty in types {
        env.register_type(ty).expect("register type");
    }
    env
}

fn pattern_env(types: &[TypeDef]) -> PatternTypeEnv {
    let mut env = PatternTypeEnv::new();
    for ty in types {
        env.add_type_def(ty.name.clone(), ty.clone());
    }
    env
}

#[test]
fn irrefutable_variable_and_wildcard_accept_open_non_adt_scrutinees() {
    let env = PatternTypeEnv::new();
    let open_ty = Type::Var(TypeVar(1002));
    let variable = check_irrefutable_pattern(&env, &var("x"), &open_ty);
    assert_eq!(variable.outcome, IrrefutabilityOutcome::Irrefutable);
    assert_eq!(variable.bindings.get("x"), Some(&open_ty));

    let wildcard = check_irrefutable_pattern(&env, &Pattern::Wildcard, &Type::Int);
    assert_eq!(wildcard.outcome, IrrefutabilityOutcome::Irrefutable);
    assert!(wildcard.bindings.is_empty());
}

#[test]
fn irrefutable_single_variant_adt_accepts_nested_irrefutable_fields() {
    let one = one_type();
    let canonical_env = canonical_env(std::slice::from_ref(&one));
    let pattern_env = pattern_env(std::slice::from_ref(&one));
    let canonical = match canonical_env.canonicalize_type_for_pattern(&constructor_ty("One")) {
        PatternCanonicalization::Matchable(canonical) => canonical,
        other => panic!("expected canonical One, got {other:?}"),
    };
    let pattern = record_variant("Only", "value", var("value"));

    let result = check_irrefutable_pattern_with_canonical_type(&pattern_env, &pattern, &canonical);

    assert_eq!(result.outcome, IrrefutabilityOutcome::Irrefutable);
    assert_eq!(result.bindings.get("value"), Some(&Type::Int));
}

#[test]
fn irrefutable_nested_refutable_binder_reports_missing_witness() {
    let option = option_type();
    let nested = nested_one_type();
    let canonical_env = canonical_env(&[option.clone(), nested.clone()]);
    let pattern_env = pattern_env(&[option, nested]);
    let canonical = match canonical_env.canonicalize_type_for_pattern(&constructor_ty("NestedOne"))
    {
        PatternCanonicalization::Matchable(canonical) => canonical,
        other => panic!("expected canonical NestedOne, got {other:?}"),
    };
    let pattern = record_variant(
        "NestedOnly",
        "maybe",
        record_variant("Some", "value", var("x")),
    );

    let result = check_irrefutable_pattern_with_canonical_type(&pattern_env, &pattern, &canonical);

    match result.outcome {
        IrrefutabilityOutcome::Refutable { witness } => {
            let IrrefutabilityWitness::Pattern(pattern) = witness else {
                panic!("expected lifted pattern witness, got {witness:?}");
            };
            assert_eq!(
                pattern.as_ref(),
                &record_variant(
                    "NestedOnly",
                    "maybe",
                    Pattern::Variant {
                        name: "None".into(),
                        fields: None,
                        payload: VariantPatternPayload::Unit,
                    }
                )
            );
        }
        other => panic!("expected nested refutable outcome, got {other:?}"),
    }
}

#[test]
fn irrefutable_list_pattern_without_rest_is_refutable() {
    let env = PatternTypeEnv::new();
    let pattern = Pattern::List {
        elements: vec![var("head")],
        rest: None,
    };

    let result = check_irrefutable_pattern(&env, &pattern, &Type::List(Box::new(Type::Int)));

    match result.outcome {
        IrrefutabilityOutcome::Refutable { witness } => {
            assert_eq!(witness, IrrefutabilityWitness::ShortList { minimum_len: 1 });
        }
        other => panic!("expected refutable list pattern, got {other:?}"),
    }
}

#[test]
fn irrefutable_list_rest_only_is_irrefutable_and_binds_rest_to_list_type() {
    let env = PatternTypeEnv::new();
    let list_ty = Type::List(Box::new(Type::Int));
    let pattern = Pattern::List {
        elements: vec![],
        rest: Some("tail".into()),
    };

    let result = check_irrefutable_pattern(&env, &pattern, &list_ty);

    assert_eq!(result.outcome, IrrefutabilityOutcome::Irrefutable);
    assert_eq!(result.bindings.get("tail"), Some(&list_ty));
}

#[test]
fn irrefutable_empty_list_without_rest_uses_non_empty_list_witness() {
    let env = PatternTypeEnv::new();
    let pattern = Pattern::List {
        elements: vec![],
        rest: None,
    };

    let result = check_irrefutable_pattern(&env, &pattern, &Type::List(Box::new(Type::Int)));

    match result.outcome {
        IrrefutabilityOutcome::Refutable { witness } => {
            assert_eq!(
                witness,
                IrrefutabilityWitness::Description("non-empty list".to_string())
            );
        }
        other => panic!("expected empty list pattern to be refutable, got {other:?}"),
    }
}

#[test]
fn irrefutable_literal_pattern_is_refutable_without_singleton() {
    let env = PatternTypeEnv::new();
    let pattern = Pattern::Literal(Literal::Int(1));

    let result = check_irrefutable_pattern(&env, &pattern, &Type::Int);

    assert!(matches!(
        result.outcome,
        IrrefutabilityOutcome::Refutable { .. }
    ));
}

#[test]
fn irrefutable_top_level_multi_variant_constructor_is_refutable_with_other_constructor_witness() {
    let option = option_type();
    let canonical_env = canonical_env(std::slice::from_ref(&option));
    let pattern_env = pattern_env(std::slice::from_ref(&option));
    let canonical = match canonical_env
        .canonicalize_type_for_pattern(&constructor_ty_with_args("Option", vec![Type::Int]))
    {
        PatternCanonicalization::Matchable(canonical) => canonical,
        other => panic!("expected canonical Option, got {other:?}"),
    };
    let pattern = record_variant("Some", "value", var("x"));

    let result = check_irrefutable_pattern_with_canonical_type(&pattern_env, &pattern, &canonical);

    match result.outcome {
        IrrefutabilityOutcome::Refutable {
            witness: IrrefutabilityWitness::Pattern(pattern),
        } => {
            assert!(matches!(
                pattern.as_ref(),
                Pattern::Variant {
                    name,
                    payload: VariantPatternPayload::Unit,
                    ..
                } if name.as_ref() == "None"
            ));
        }
        other => panic!("expected other constructor witness, got {other:?}"),
    }
}

#[test]
fn irrefutable_direct_tuple_and_record_cases_and_nested_refutable_product_witnesses() {
    let env = PatternTypeEnv::new();
    let tuple_ty = Type::Record(vec![("0".into(), Type::Int), ("1".into(), Type::Bool)]);
    let tuple = Pattern::Tuple(vec![var("left"), Pattern::Wildcard]);

    let tuple_result = check_irrefutable_pattern(&env, &tuple, &tuple_ty);

    assert_eq!(tuple_result.outcome, IrrefutabilityOutcome::Irrefutable);
    assert_eq!(tuple_result.bindings.get("left"), Some(&Type::Int));

    let record_ty = Type::Record(vec![
        ("tag".into(), Type::Bool),
        ("count".into(), Type::Int),
    ]);
    let record = Pattern::Record(vec![
        ("tag".into(), Pattern::Literal(Literal::Bool(true))),
        ("count".into(), var("count")),
    ]);

    let record_result = check_irrefutable_pattern(&env, &record, &record_ty);

    match record_result.outcome {
        IrrefutabilityOutcome::Refutable {
            witness: IrrefutabilityWitness::Pattern(pattern),
        } => assert_eq!(
            pattern.as_ref(),
            &Pattern::Record(vec![
                ("tag".into(), Pattern::Literal(Literal::Bool(false))),
                ("count".into(), Pattern::Wildcard),
            ])
        ),
        other => panic!("expected lifted record witness, got {other:?}"),
    }

    let nested_tuple = Pattern::Tuple(vec![Pattern::Literal(Literal::Int(1)), Pattern::Wildcard]);
    let nested_tuple_result = check_irrefutable_pattern(&env, &nested_tuple, &tuple_ty);

    match nested_tuple_result.outcome {
        IrrefutabilityOutcome::Refutable {
            witness: IrrefutabilityWitness::Pattern(pattern),
        } => assert_eq!(
            pattern.as_ref(),
            &Pattern::Tuple(vec![Pattern::Literal(Literal::Int(2)), Pattern::Wildcard,])
        ),
        other => panic!("expected lifted tuple witness, got {other:?}"),
    }
}

#[test]
fn irrefutable_tuple_and_record_over_type_var_block_product_shape() {
    let env = PatternTypeEnv::new();
    let open_ty = Type::Var(TypeVar(1004));

    let tuple_result = check_irrefutable_pattern(&env, &Pattern::Tuple(vec![var("x")]), &open_ty);
    assert!(matches!(
        tuple_result.outcome,
        IrrefutabilityOutcome::Blocked { .. }
    ));

    let record_result = check_irrefutable_pattern(
        &env,
        &Pattern::Record(vec![("x".into(), var("x"))]),
        &open_ty,
    );
    assert!(matches!(
        record_result.outcome,
        IrrefutabilityOutcome::Blocked { .. }
    ));
}

#[test]
fn irrefutable_variant_payload_shape_and_arity_mismatch_are_impossible() {
    let pair = pair_type();
    let canonical_env = canonical_env(std::slice::from_ref(&pair));
    let pattern_env = pattern_env(std::slice::from_ref(&pair));
    let canonical = match canonical_env.canonicalize_type_for_pattern(&constructor_ty("Pair")) {
        PatternCanonicalization::Matchable(canonical) => canonical,
        other => panic!("expected canonical Pair, got {other:?}"),
    };

    let record_payload = Pattern::Variant {
        name: "PairOnly".into(),
        fields: Some(vec![
            ("0".into(), var("x")),
            ("1".into(), Pattern::Wildcard),
        ]),
        payload: VariantPatternPayload::Record(vec![
            ("0".into(), var("x")),
            ("1".into(), Pattern::Wildcard),
        ]),
    };
    let record_result =
        check_irrefutable_pattern_with_canonical_type(&pattern_env, &record_payload, &canonical);
    assert!(matches!(
        record_result.outcome,
        IrrefutabilityOutcome::Impossible { .. }
    ));

    let arity_result = check_irrefutable_pattern_with_canonical_type(
        &pattern_env,
        &tuple_variant("PairOnly", vec![var("x")]),
        &canonical,
    );
    assert!(matches!(
        arity_result.outcome,
        IrrefutabilityOutcome::Impossible { .. }
    ));

    let unit_result = check_irrefutable_pattern_with_canonical_type(
        &pattern_env,
        &Pattern::Variant {
            name: "PairOnly".into(),
            fields: None,
            payload: VariantPatternPayload::Unit,
        },
        &canonical,
    );
    assert!(matches!(
        unit_result.outcome,
        IrrefutabilityOutcome::Impossible { .. }
    ));
}

#[test]
fn irrefutable_nested_user_defined_named_field_type_is_lowered_to_adt() {
    let maybe_int = maybe_int_type();
    let wrapper = wrapper_with_user_defined_field_type();
    let canonical_env = canonical_env(&[maybe_int.clone(), wrapper.clone()]);
    let pattern_env = pattern_env(&[maybe_int, wrapper]);
    let canonical = match canonical_env.canonicalize_type_for_pattern(&constructor_ty("Wrapper")) {
        PatternCanonicalization::Matchable(canonical) => canonical,
        other => panic!("expected canonical Wrapper, got {other:?}"),
    };
    let pattern = record_variant(
        "Wrap",
        "inner",
        record_variant("SomeInt", "value", var("x")),
    );

    let result = check_irrefutable_pattern_with_canonical_type(&pattern_env, &pattern, &canonical);

    match result.outcome {
        IrrefutabilityOutcome::Refutable {
            witness: IrrefutabilityWitness::Pattern(pattern),
        } => assert_eq!(
            pattern.as_ref(),
            &record_variant(
                "Wrap",
                "inner",
                Pattern::Variant {
                    name: "NoInt".into(),
                    fields: None,
                    payload: VariantPatternPayload::Unit,
                }
            )
        ),
        other => panic!("expected nested ADT field to produce lifted witness, got {other:?}"),
    }
}

#[test]
fn irrefutable_duplicate_binders_rejected() {
    let env = PatternTypeEnv::new();
    let pattern = Pattern::Tuple(vec![var("x"), var("x")]);
    let tuple_ty = Type::Record(vec![("0".into(), Type::Int), ("1".into(), Type::Int)]);

    let result = check_irrefutable_pattern(&env, &pattern, &tuple_ty);

    match result.outcome {
        IrrefutabilityOutcome::Impossible {
            reason: IrrefutabilityImpossibleReason::DuplicateBinder { name },
        } => {
            assert_eq!(name, "x");
        }
        other => panic!("expected duplicate binder rejection, got {other:?}"),
    }
}

#[test]
fn irrefutable_impossible_pattern_reports_type_mismatch() {
    let env = PatternTypeEnv::new();
    let pattern = Pattern::Tuple(vec![var("x")]);

    let result = check_irrefutable_pattern(&env, &pattern, &Type::Int);

    match result.outcome {
        IrrefutabilityOutcome::Impossible { reason } => {
            assert!(format!("{reason:?}").contains("PatternMismatch"));
        }
        other => panic!("expected impossible type mismatch, got {other:?}"),
    }
}

#[test]
fn irrefutable_blocked_constructor_coverage_reports_blocked_reason() {
    let env = PatternTypeEnv::new();
    let open_ty = Type::Var(TypeVar(1003));
    let blocked = PatternCanonicalization::Blocked {
        source_type: open_ty.clone(),
        reason: PatternCanonicalizationBlockedReason::TypeVariable,
    };
    let pattern = record_variant("Some", "value", var("x"));

    let result =
        check_irrefutable_pattern_with_canonicalization(&env, &pattern, &open_ty, &blocked);

    match result.outcome {
        IrrefutabilityOutcome::Blocked { reason } => {
            assert!(format!("{reason:?}").contains("TypeVariable"));
        }
        other => panic!("expected blocked constructor coverage, got {other:?}"),
    }
}
