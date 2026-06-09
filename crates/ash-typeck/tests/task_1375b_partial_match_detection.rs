use ash_parser::surface::VariantPatternPayload;
use ash_parser::surface::{
    BlockStmt, Expr, Literal, MatchArm, Name, Param, Pattern, ProofBody, ProofDef, Type,
};
use ash_parser::token::Span;

fn name(value: &str) -> Name {
    value.into()
}

fn option_int_type() -> Type {
    option_type(Type::Name(name("Int")))
}

fn option_type(item: Type) -> Type {
    Type::Constructor {
        name: name("Option"),
        args: vec![item],
    }
}

fn proof_with_match(arms: Vec<MatchArm>) -> ProofDef {
    proof_with_match_type(option_int_type(), arms)
}

fn proof_with_match_type(param_ty: Type, arms: Vec<MatchArm>) -> ProofDef {
    ProofDef {
        name: name("option_total"),
        params: vec![Param {
            name: name("x"),
            ty: param_ty,
        }],
        constraints: vec![],
        body: ProofBody::Expr(Expr::Match {
            scrutinee: Box::new(Expr::Variable {
                name: name("x"),
                span: Span::default(),
            }),
            arms,
            span: Span::default(),
        }),
        span: Span::default(),
    }
}

fn some_arm() -> MatchArm {
    some_arm_with_body("value", Box::new(Expr::Literal(Literal::Bool(true))))
}

fn some_arm_with_body(binding: &str, body: Box<Expr>) -> MatchArm {
    some_arm_with_pattern_and_body(
        Pattern::Variable {
            name: name(binding),
            span: Span::default(),
        },
        body,
    )
}

fn some_arm_with_pattern(pattern: Pattern) -> MatchArm {
    some_arm_with_pattern_and_body(pattern, Box::new(Expr::Literal(Literal::Bool(true))))
}

fn some_arm_with_pattern_and_body(pattern: Pattern, body: Box<Expr>) -> MatchArm {
    MatchArm {
        pattern: Pattern::Variant {
            name: name("Some"),
            fields: Some(vec![(name("value"), pattern.clone())]),
            payload: VariantPatternPayload::Record(vec![(name("value"), pattern)]),
        },
        body,
        span: Span::default(),
    }
}

fn none_arm() -> MatchArm {
    MatchArm {
        pattern: Pattern::Variant {
            name: name("None"),
            fields: None,
            payload: VariantPatternPayload::Unit,
        },
        body: Box::new(Expr::Literal(Literal::Bool(true))),
        span: Span::default(),
    }
}

fn wildcard_arm() -> MatchArm {
    MatchArm {
        pattern: Pattern::Wildcard,
        body: Box::new(Expr::Literal(Literal::Bool(true))),
        span: Span::default(),
    }
}

#[test]
fn proof_match_missing_option_constructor_is_rejected() {
    let proof = proof_with_match(vec![some_arm()]);

    let err = ash_typeck::TypeEnv::with_builtin_types()
        .check_proof_totality(&proof)
        .expect_err("proof match missing None should fail totality checking");
    let errors = err.to_string();

    assert!(
        errors.contains("non-exhaustive") && errors.contains("None"),
        "expected non-exhaustive proof match diagnostic naming missing None, got:\n{errors}"
    );
}

#[test]
fn proof_match_with_wildcard_catchall_passes() {
    let proof = proof_with_match(vec![some_arm(), wildcard_arm()]);

    let result = ash_typeck::TypeEnv::with_builtin_types()
        .check_proof_totality(&proof)
        .expect("wildcard catch-all should make proof match exhaustive");

    assert_eq!(result.status, ash_typeck::ProofTotalityStatus::Checked);
}

#[test]
fn proof_match_with_complete_constructor_coverage_passes() {
    let proof = proof_with_match(vec![some_arm(), none_arm()]);

    let result = ash_typeck::TypeEnv::with_builtin_types()
        .check_proof_totality(&proof)
        .expect("complete Option constructor coverage should pass");

    assert_eq!(result.status, ash_typeck::ProofTotalityStatus::Checked);
}

#[test]
fn nested_proof_match_can_use_outer_arm_pattern_binding() {
    let nested_match = Expr::Match {
        scrutinee: Box::new(Expr::Variable {
            name: name("inner"),
            span: Span::default(),
        }),
        arms: vec![wildcard_arm()],
        span: Span::default(),
    };
    let proof = proof_with_match_type(
        option_type(option_int_type()),
        vec![
            some_arm_with_body("inner", Box::new(nested_match)),
            wildcard_arm(),
        ],
    );

    let result = ash_typeck::TypeEnv::with_builtin_types()
        .check_proof_totality(&proof)
        .expect("outer match arm bindings should be in scope for nested proof matches");

    assert_eq!(result.status, ash_typeck::ProofTotalityStatus::Checked);
}

#[test]
fn nested_proof_match_can_use_block_let_binding() {
    let proof = ProofDef {
        name: name("block_match"),
        params: vec![Param {
            name: name("x"),
            ty: option_int_type(),
        }],
        constraints: vec![],
        body: ProofBody::Expr(Expr::Block {
            statements: vec![BlockStmt::Let {
                pattern: Pattern::Variable {
                    name: name("y"),
                    span: Span::default(),
                },
                expr: Expr::Variable {
                    name: name("x"),
                    span: Span::default(),
                },
                span: Span::default(),
            }],
            tail_expr: Some(Box::new(Expr::Match {
                scrutinee: Box::new(Expr::Variable {
                    name: name("y"),
                    span: Span::default(),
                }),
                arms: vec![wildcard_arm()],
                span: Span::default(),
            })),
            span: Span::default(),
        }),
        span: Span::default(),
    };

    let result = ash_typeck::TypeEnv::with_builtin_types()
        .check_proof_totality(&proof)
        .expect("block let bindings should be in scope for nested proof matches");

    assert_eq!(result.status, ash_typeck::ProofTotalityStatus::Checked);
}

#[test]
fn nested_proof_match_can_use_fn_parameter_binding() {
    let proof = ProofDef {
        name: name("fn_match"),
        params: vec![],
        constraints: vec![],
        body: ProofBody::Expr(Expr::FnDef {
            params: vec![(name("arg"), None)],
            return_type: None,
            body: Box::new(Expr::Match {
                scrutinee: Box::new(Expr::Variable {
                    name: name("arg"),
                    span: Span::default(),
                }),
                arms: vec![wildcard_arm()],
                span: Span::default(),
            }),
            span: Span::default(),
        }),
        span: Span::default(),
    };

    let result = ash_typeck::TypeEnv::with_builtin_types()
        .check_proof_totality(&proof)
        .expect("fn parameters should be in scope for nested proof matches");

    assert_eq!(result.status, ash_typeck::ProofTotalityStatus::Checked);
}

#[test]
fn wildcard_proof_match_does_not_require_scrutinee_type_resolution() {
    let proof = ProofDef {
        name: name("wildcard_unknown"),
        params: vec![],
        constraints: vec![],
        body: ProofBody::Expr(Expr::Match {
            scrutinee: Box::new(Expr::Call {
                func: name("unknown_proof_helper"),
                module: None,
                args: vec![],
                span: Span::default(),
            }),
            arms: vec![wildcard_arm()],
            span: Span::default(),
        }),
        span: Span::default(),
    };

    let result = ash_typeck::TypeEnv::with_builtin_types()
        .check_proof_totality(&proof)
        .expect("wildcard proof matches should be exhaustive without scrutinee type resolution");

    assert_eq!(result.status, ash_typeck::ProofTotalityStatus::Checked);
}

#[test]
fn generic_by_definition_proof_params_do_not_require_concrete_type_resolution() {
    let proof = ProofDef {
        name: name("generic_reflexive"),
        params: vec![Param {
            name: name("x"),
            ty: Type::Name(name("T")),
        }],
        constraints: vec![],
        body: ProofBody::ByDefinition,
        span: Span::default(),
    };

    let result = ash_typeck::TypeEnv::with_builtin_types()
        .check_proof_totality(&proof)
        .expect("generic proof params should be accepted by totality checking");

    assert_eq!(result.status, ash_typeck::ProofTotalityStatus::Checked);
}

#[test]
fn generic_adt_proof_param_preserves_constructor_coverage() {
    let proof = proof_with_match_type(
        option_type(Type::Name(name("T"))),
        vec![some_arm(), none_arm()],
    );

    let result = ash_typeck::TypeEnv::with_builtin_types()
        .check_proof_totality(&proof)
        .expect("generic ADT params should preserve constructor coverage");

    assert_eq!(result.status, ash_typeck::ProofTotalityStatus::Checked);
}

#[test]
fn generic_adt_constructor_coverage_rejects_refutable_payload_patterns() {
    let nested_some_pattern = Pattern::Variant {
        name: name("Some"),
        fields: Some(vec![(name("value"), Pattern::Wildcard)]),
        payload: VariantPatternPayload::Record(vec![(name("value"), Pattern::Wildcard)]),
    };
    let proof = proof_with_match_type(
        option_type(Type::Name(name("T"))),
        vec![some_arm_with_pattern(nested_some_pattern), none_arm()],
    );

    let result = ash_typeck::TypeEnv::with_builtin_types().check_proof_totality(&proof);

    assert!(
        result.is_err(),
        "nested payload patterns over generic ADTs must not be accepted by constructor-name fallback"
    );
}

#[test]
fn low_fuel_match_returns_untested_before_scrutinee_resolution_error() {
    let proof = ProofDef {
        name: name("low_fuel"),
        params: vec![],
        constraints: vec![],
        body: ProofBody::Expr(Expr::Match {
            scrutinee: Box::new(Expr::Call {
                func: name("unknown_proof_helper"),
                module: None,
                args: vec![],
                span: Span::default(),
            }),
            arms: vec![some_arm(), none_arm()],
            span: Span::default(),
        }),
        span: Span::default(),
    };

    let result = ash_typeck::TypeEnv::with_builtin_types()
        .check_proof_totality_with_fuel(&proof, 1)
        .expect("fuel exhaustion should be untested, not a scrutinee resolution error");

    assert_eq!(
        result.status,
        ash_typeck::ProofTotalityStatus::Untested(
            ash_typeck::ProofTotalityUntestedReason::FuelExhausted
        )
    );
}

#[test]
fn low_fuel_catchall_match_returns_untested_before_pattern_type_error() {
    let invalid_nested_some_pattern = Pattern::Variant {
        name: name("Some"),
        fields: Some(vec![(name("value"), Pattern::Wildcard)]),
        payload: VariantPatternPayload::Record(vec![(name("value"), Pattern::Wildcard)]),
    };
    let proof = proof_with_match(vec![
        some_arm_with_pattern(invalid_nested_some_pattern),
        wildcard_arm(),
    ]);

    let result = ash_typeck::TypeEnv::with_builtin_types()
        .check_proof_totality_with_fuel(&proof, 1)
        .expect("fuel exhaustion should win before catch-all arm pattern typing");

    assert_eq!(
        result.status,
        ash_typeck::ProofTotalityStatus::Untested(
            ash_typeck::ProofTotalityUntestedReason::FuelExhausted
        )
    );
}

#[test]
fn concrete_match_scrutinee_is_counted_once_for_fuel() {
    let proof = proof_with_match(vec![some_arm(), none_arm()]);

    let result = ash_typeck::TypeEnv::with_builtin_types()
        .check_proof_totality_with_fuel(&proof, 4)
        .expect("exact AST traversal fuel should be enough for concrete complete matches");

    assert_eq!(result.status, ash_typeck::ProofTotalityStatus::Checked);
}

#[test]
fn wildcard_covered_match_still_binds_typed_constructor_arm_variables() {
    let nested_match = Expr::Match {
        scrutinee: Box::new(Expr::Variable {
            name: name("inner"),
            span: Span::default(),
        }),
        arms: vec![some_arm(), none_arm()],
        span: Span::default(),
    };
    let proof = proof_with_match_type(
        option_type(option_int_type()),
        vec![
            some_arm_with_body("inner", Box::new(nested_match)),
            wildcard_arm(),
        ],
    );

    let result = ash_typeck::TypeEnv::with_builtin_types()
        .check_proof_totality(&proof)
        .expect("typed constructor arms before a catch-all should bind nested proof variables");

    assert_eq!(result.status, ash_typeck::ProofTotalityStatus::Checked);
}

#[test]
fn block_let_unknown_initializer_does_not_block_unrelated_totality_checking() {
    let proof = ProofDef {
        name: name("opaque_let"),
        params: vec![],
        constraints: vec![],
        body: ProofBody::Expr(Expr::Block {
            statements: vec![BlockStmt::Let {
                pattern: Pattern::Variable {
                    name: name("opaque"),
                    span: Span::default(),
                },
                expr: Expr::Call {
                    func: name("unknown_proof_helper"),
                    module: None,
                    args: vec![],
                    span: Span::default(),
                },
                span: Span::default(),
            }],
            tail_expr: Some(Box::new(Expr::Literal(Literal::Bool(true)))),
            span: Span::default(),
        }),
        span: Span::default(),
    };

    let result = ash_typeck::TypeEnv::with_builtin_types()
        .check_proof_totality(&proof)
        .expect("opaque let initializers should not block unrelated proof totality traversal");

    assert_eq!(result.status, ash_typeck::ProofTotalityStatus::Checked);
}
