use ash_core::ast::{TypeBody, TypeDef, TypeExpr, VariantDef, VariantPayload, Visibility};
use ash_parser::surface::{Expr, Literal, MatchArm, Pattern, VariantPatternPayload};
use ash_parser::token::Span;
use ash_typeck::check_expr::check_expr;
use ash_typeck::check_pattern::check_pattern_with_canonical_type;
use ash_typeck::{
    Kind, PatternCanonicalization, PatternCanonicalizationBlockedReason, QualifiedName, Type,
    TypeEnv, TypeVar,
};

fn span() -> Span {
    Span::default()
}

fn result_type() -> TypeDef {
    TypeDef {
        name: "Result".into(),
        params: vec!["T".into(), "E".into()],
        body: TypeBody::Enum(vec![
            VariantDef {
                name: "Ok".into(),
                fields: vec![("value".into(), TypeExpr::Named("T".into()))],
                payload: VariantPayload::Record(vec![(
                    "value".into(),
                    TypeExpr::Named("T".into()),
                )]),
            },
            VariantDef {
                name: "Err".into(),
                fields: vec![("error".into(), TypeExpr::Named("E".into()))],
                payload: VariantPayload::Record(vec![(
                    "error".into(),
                    TypeExpr::Named("E".into()),
                )]),
            },
        ]),
        visibility: Visibility::Public,
        builtin: false,
    }
}

fn env_with_result() -> TypeEnv {
    let mut env = TypeEnv::new();
    env.register_type(&result_type()).expect("register Result");
    env
}

fn result_ty(ok: Type, err: Type) -> Type {
    Type::Constructor {
        name: QualifiedName::root("Result"),
        args: vec![ok, err],
        kind: Kind::Type,
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

fn record_variant(name: &str, field_name: &str, binding_name: &str) -> Pattern {
    let binding = Pattern::Variable {
        name: binding_name.into(),
        span: span(),
    };
    Pattern::Variant {
        name: name.into(),
        fields: Some(vec![(field_name.into(), binding.clone())]),
        payload: VariantPatternPayload::Record(vec![(field_name.into(), binding)]),
    }
}

fn match_expr(scrutinee_name: &str, arms: Vec<MatchArm>) -> Expr {
    Expr::Match {
        scrutinee: Box::new(variable(scrutinee_name)),
        arms,
        span: span(),
    }
}

fn error_text<T: ToString>(errors: &[T]) -> String {
    errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn unresolved_associated_projection_returns_typed_blocked_reason_for_patterns() {
    let env = env_with_result();
    let source = Type::Associated {
        interface: "Iterable".into(),
        base: Box::new(Type::Var(TypeVar(916))),
        name: "Item".into(),
    };

    match env.canonicalize_type_for_pattern(&source) {
        PatternCanonicalization::Blocked {
            source_type,
            reason:
                PatternCanonicalizationBlockedReason::RigidAssociatedProjection { interface, member },
        } => {
            assert_eq!(source_type, source);
            assert_eq!(interface, "Iterable");
            assert_eq!(member, "Item");
        }
        other => panic!("expected typed blocked projection reason, got {other:?}"),
    }
}

#[test]
fn primitive_scrutinee_with_visible_constructor_does_not_fabricate_missing_witness() {
    let mut env = env_with_result();
    env.bind_variable("subject", Type::Int);
    let expr = match_expr(
        "subject",
        vec![MatchArm {
            pattern: record_variant("Ok", "value", "x"),
            body: Box::new(int(1)),
            span: span(),
        }],
    );

    let checked = check_expr(&env, &expr);
    let errors = error_text(&checked.errors);

    assert!(
        !checked.is_ok(),
        "matching Int with a Result constructor should fail"
    );
    assert!(
        !errors.contains("non-exhaustive"),
        "non-matchable primitive scrutinee must not fabricate an exhaustiveness error:\n{errors}"
    );
    assert!(
        !errors.contains("Err"),
        "non-matchable primitive scrutinee must not invent Result::Err as a missing witness:\n{errors}"
    );
}

#[test]
fn wrong_pattern_constructor_names_offending_constructor_and_canonical_boundary() {
    let env = env_with_result();
    let canonical = match env.canonicalize_type_for_pattern(&result_ty(Type::Int, Type::String)) {
        PatternCanonicalization::Matchable(canonical) => canonical,
        other => panic!("expected Result to canonicalize for pattern checking, got {other:?}"),
    };
    let pattern = record_variant("Ghost", "value", "x");

    let error = check_pattern_with_canonical_type(
        &ash_typeck::check_pattern::TypeEnv::new(),
        &pattern,
        &canonical,
    )
    .expect_err("Ghost must not be accepted for canonical Result");
    let text = error.to_string();

    assert!(
        text.contains("Ghost"),
        "diagnostic must name the offending constructor:\n{text}"
    );
    assert!(
        text.contains("Result") && text.contains("canonical"),
        "diagnostic must name the canonical pattern boundary:\n{text}"
    );
}
