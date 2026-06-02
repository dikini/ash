use ash_core::ast::{TypeBody, TypeDef, TypeExpr, VariantDef, VariantPayload, Visibility};
use ash_parser::surface::{Expr, Literal, MatchArm, Pattern, VariantPatternPayload};
use ash_parser::token::Span;
use ash_typeck::check_expr::check_expr;
use ash_typeck::error::ConstructorError;
use ash_typeck::{Kind, QualifiedName, Type, TypeEnv};

fn span() -> Span {
    Span::default()
}

fn failure_type() -> TypeDef {
    TypeDef {
        name: "Failure".into(),
        params: vec![],
        body: TypeBody::Enum(vec![
            VariantDef {
                name: "NetworkFailure".into(),
                fields: vec![("message".into(), TypeExpr::Named("String".into()))],
                payload: VariantPayload::Record(vec![(
                    "message".into(),
                    TypeExpr::Named("String".into()),
                )]),
            },
            VariantDef {
                name: "TimeoutFailure".into(),
                fields: vec![],
                payload: VariantPayload::Unit,
            },
        ]),
        visibility: Visibility::Public,
        builtin: false,
    }
}

fn env_with_failure() -> TypeEnv {
    let mut env = TypeEnv::new();
    env.register_type(&failure_type())
        .expect("register Failure");
    env
}

fn failure_ty() -> Type {
    Type::Constructor {
        name: QualifiedName::root("Failure"),
        args: vec![],
        kind: Kind::Type,
    }
}

fn int(value: i64) -> Expr {
    Expr::Literal(Literal::Int(value))
}

fn string(value: &str) -> Expr {
    Expr::Literal(Literal::String(value.into()))
}

fn failure_payload_expr() -> Expr {
    Expr::Constructor {
        name: "NetworkFailure".into(),
        fields: vec![(Box::<str>::from("message"), string("offline"))],
        payload: ash_parser::surface::ConstructorPayload::Record(vec![(
            Box::<str>::from("message"),
            string("offline"),
        )]),
        span: span(),
    }
}

fn fail_failure_payload() -> Expr {
    Expr::Fail {
        payload: Box::new(failure_payload_expr()),
        span: span(),
    }
}

fn variable(name: &str) -> Expr {
    Expr::Variable {
        name: name.into(),
        span: span(),
    }
}

fn wildcard() -> Pattern {
    Pattern::Wildcard
}

fn binding(name: &str) -> Pattern {
    Pattern::Variable {
        name: name.into(),
        span: span(),
    }
}

fn variant_unit(name: &str) -> Pattern {
    Pattern::Variant {
        name: name.into(),
        fields: None,
        payload: VariantPatternPayload::Unit,
    }
}

fn variant_record(name: &str, field_name: &str, field_pattern: Pattern) -> Pattern {
    Pattern::Variant {
        name: name.into(),
        fields: Some(vec![(field_name.into(), field_pattern.clone())]),
        payload: VariantPatternPayload::Record(vec![(field_name.into(), field_pattern)]),
    }
}

fn arm(pattern: Pattern, body: Expr) -> MatchArm {
    MatchArm {
        pattern,
        body: Box::new(body),
        span: span(),
    }
}

fn with_error(body: Expr, arms: Vec<MatchArm>) -> Expr {
    Expr::WithError {
        body: Box::new(body),
        arms,
        span: span(),
    }
}

fn error_text(errors: &[ConstructorError]) -> String {
    errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn with_error_total_handler_reports_or_defers_closed_payload_missing_case() {
    let env = env_with_failure();

    let checked = check_expr(
        &env,
        &with_error(
            fail_failure_payload(),
            vec![arm(
                variant_record("NetworkFailure", "message", binding("message")),
                int(1),
            )],
        ),
    );
    assert!(
        !checked.is_ok(),
        "constructor-specific with_error handler coverage must not be silently accepted"
    );
    let Some(ConstructorError::NonExhaustiveWithErrorHandler {
        payload_type,
        missing,
        ..
    }) = checked.errors.iter().find(|error| {
        matches!(
            error,
            ConstructorError::NonExhaustiveWithErrorHandler { .. }
        )
    })
    else {
        panic!(
            "expected structured with_error handler coverage diagnostic, got:\n{}",
            error_text(&checked.errors)
        );
    };
    assert_eq!(payload_type, "Failure");
    assert!(
        missing.contains("TimeoutFailure"),
        "missing witness should name TimeoutFailure, got {missing}"
    );

    let deferred = check_expr(
        &env,
        &with_error(
            int(1),
            vec![arm(
                variant_record("NetworkFailure", "message", binding("message")),
                int(1),
            )],
        ),
    );

    assert!(
        deferred.errors.iter().any(|error| {
            matches!(
                error,
                ConstructorError::WithErrorHandlerCoverageDeferred { payload_type, reason, .. }
                    if payload_type == "<unavailable>"
                        && reason.contains("failure payload type is not tracked")
            )
        }),
        "expected deferred handler coverage diagnostic for unavailable payload universe, got:\n{}",
        error_text(&deferred.errors)
    );
}

#[test]
fn with_error_handler_pattern_type_error_is_structured() {
    let env = env_with_failure();

    let checked = check_expr(
        &env,
        &with_error(int(1), vec![arm(variant_unit("NotFailure"), int(1))]),
    );

    assert!(
        checked.errors.iter().any(|error| {
            matches!(
                error,
                ConstructorError::UnsupportedExpression { kind, .. }
                    if kind.contains("with_error handler pattern type error")
                        && kind.contains("NotFailure")
            )
        }),
        "expected structured handler pattern type error, got:\n{}",
        error_text(&checked.errors)
    );
}

#[test]
fn with_error_wildcard_accepts_open_payload() {
    let env = TypeEnv::new();

    let checked = check_expr(&env, &with_error(int(1), vec![arm(wildcard(), int(2))]));

    assert!(
        checked.is_ok(),
        "wildcard handler must accept currently open with_error payload: {:?}",
        checked.errors
    );
    assert_eq!(checked.substitution.apply(&checked.ty), Type::Int);
}

#[test]
fn with_error_wildcard_accepts_unavailable_payload_with_constructor_arm() {
    let env = env_with_failure();

    let checked = check_expr(
        &env,
        &with_error(
            int(1),
            vec![
                arm(
                    variant_record("NetworkFailure", "message", binding("message")),
                    int(2),
                ),
                arm(wildcard(), int(3)),
            ],
        ),
    );

    assert!(
        !checked.errors.iter().any(|error| {
            matches!(
                error,
                ConstructorError::WithErrorHandlerCoverageDeferred { .. }
            )
        }),
        "wildcard/default handler must prove unavailable payload coverage, got:\n{}",
        error_text(&checked.errors)
    );
}

#[test]
fn with_error_empty_direct_fail_handler_reports_non_exhaustive_known_payload() {
    let env = env_with_failure();

    let checked = check_expr(&env, &with_error(fail_failure_payload(), vec![]));

    assert!(
        !checked.errors.iter().any(|error| {
            matches!(
                error,
                ConstructorError::WithErrorHandlerCoverageDeferred { .. }
            )
        }),
        "known closed payload with empty handlers must not defer coverage, got:\n{}",
        error_text(&checked.errors)
    );
    let Some(ConstructorError::NonExhaustiveWithErrorHandler {
        payload_type,
        missing,
        ..
    }) = checked.errors.iter().find(|error| {
        matches!(
            error,
            ConstructorError::NonExhaustiveWithErrorHandler { .. }
        )
    })
    else {
        panic!(
            "expected non-exhaustive with_error handler diagnostic, got:\n{}",
            error_text(&checked.errors)
        );
    };
    assert_eq!(payload_type, "Failure");
    assert!(
        missing.contains("NetworkFailure") && missing.contains("TimeoutFailure"),
        "empty handler missing witnesses should name both constructors, got {missing}"
    );
}

#[test]
fn with_error_branch_type_mismatch_reports_handler_context() {
    let mut env = env_with_failure();
    env.bind_variable("failure", failure_ty());

    let checked = check_expr(
        &env,
        &with_error(
            int(1),
            vec![arm(
                variant_record("NetworkFailure", "message", binding("message")),
                variable("message"),
            )],
        ),
    );

    assert!(
        checked.errors.iter().any(|error| {
            matches!(
                error,
                ConstructorError::UnsupportedExpression { kind, .. }
                    if kind.contains("with_error handler type mismatch")
                        && kind.contains("expected Int")
                        && kind.contains("got String")
            )
        }),
        "expected handler-context branch type mismatch, got:\n{}",
        error_text(&checked.errors)
    );
}
