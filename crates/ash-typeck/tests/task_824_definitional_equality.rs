use ash_core::kind::Kind;
use ash_core::module_graph::ModuleId;
use ash_core::semantic_summary::{
    AssociatedMemberIdentityId, DomainConstructorId, InterfaceIdentityId, ModuleIdentity,
    ModuleSourceOrigin, SealedDomainId,
};
use ash_core::type_ir::{
    CanonicalTypeExpr, NormalFormBlockReason, NormalTypeExpr, ProjectionRigidity,
    TypeComputationHeadId,
};
use ash_typeck::TypeEnv;
use ash_typeck::normalizer::{
    DefinitionalEqualityResult, FixtureDomainConstructorPattern, FixtureEquation,
    FixtureEquationRegistry, FixturePattern, FixtureResultExpr, NormalizationConfig,
    NormalizationError, NormalizationFuel, NormalizationMode, Normalizer,
};

fn module() -> ModuleIdentity {
    ModuleIdentity::new(
        None,
        ModuleId(824),
        vec!["task_824".to_string(), "definitional_equality".to_string()],
        ModuleSourceOrigin::Synthetic {
            reason: "TASK-824 definitional equality tests".to_string(),
        },
    )
}

fn head(name: &str) -> TypeComputationHeadId {
    TypeComputationHeadId::new(module(), name)
}

fn domain() -> SealedDomainId {
    SealedDomainId::new(module(), "List")
}

fn ctor(name: &str) -> DomainConstructorId {
    DomainConstructorId::new(domain(), name)
}

fn var_pattern(name: &str) -> FixturePattern {
    FixturePattern::Var(name.to_string())
}

fn ctor_pattern(name: &str, args: Vec<FixturePattern>) -> FixturePattern {
    FixturePattern::DomainConstructor(Box::new(FixtureDomainConstructorPattern {
        constructor: ctor(name),
        domain: domain(),
        args,
    }))
}

fn var_result(name: &str) -> FixtureResultExpr {
    FixtureResultExpr::BoundVar(name.to_string())
}

fn ctor_result(name: &str, args: Vec<FixtureResultExpr>) -> FixtureResultExpr {
    FixtureResultExpr::DomainConstructor {
        constructor: ctor(name),
        domain: domain(),
        args,
        kind: Kind::Type,
    }
}

fn app_result(name: &str, args: Vec<FixtureResultExpr>) -> FixtureResultExpr {
    FixtureResultExpr::ComputationHeadApp {
        head: head(name),
        args,
        kind: Kind::Type,
    }
}

fn app(name: &str, args: Vec<CanonicalTypeExpr>) -> CanonicalTypeExpr {
    CanonicalTypeExpr::ComputationHeadApp {
        head: head(name),
        args,
        kind: Kind::Type,
    }
}

fn var(name: &str) -> CanonicalTypeExpr {
    CanonicalTypeExpr::Var(name.to_string())
}

fn primitive(name: &str) -> CanonicalTypeExpr {
    CanonicalTypeExpr::Primitive(name.to_string())
}

fn nil_expr() -> CanonicalTypeExpr {
    app("NilLiteral", vec![])
}

fn cons_expr(head_expr: CanonicalTypeExpr, tail: CanonicalTypeExpr) -> CanonicalTypeExpr {
    app("ConsLiteral", vec![head_expr, tail])
}

fn nil_normal() -> NormalTypeExpr {
    NormalTypeExpr::DomainConstructorApp {
        constructor: ctor("Nil"),
        domain: domain(),
        args: vec![],
        kind: Kind::Type,
    }
}

fn cons_normal(head_expr: NormalTypeExpr, tail: NormalTypeExpr) -> NormalTypeExpr {
    NormalTypeExpr::DomainConstructorApp {
        constructor: ctor("Cons"),
        domain: domain(),
        args: vec![head_expr, tail],
        kind: Kind::Type,
    }
}

fn interface() -> InterfaceIdentityId {
    InterfaceIdentityId::new(module(), "Iterable")
}

fn member() -> AssociatedMemberIdentityId {
    AssociatedMemberIdentityId::associated_type(interface(), "Item", vec![])
}

fn projection(rigidity: ProjectionRigidity, args: Vec<CanonicalTypeExpr>) -> CanonicalTypeExpr {
    CanonicalTypeExpr::Projection {
        interface: interface(),
        member: member(),
        args,
        kind: Kind::Type,
        rigidity,
    }
}

fn registry() -> FixtureEquationRegistry {
    FixtureEquationRegistry::empty()
        .with_equation(
            FixtureEquation::new(head("NilLiteral"), vec![], ctor_result("Nil", vec![]))
                .expect("nil literal fixture"),
        )
        .expect("register nil literal")
        .with_equation(
            FixtureEquation::new(
                head("ConsLiteral"),
                vec![var_pattern("h"), var_pattern("t")],
                ctor_result("Cons", vec![var_result("h"), var_result("t")]),
            )
            .expect("cons literal fixture"),
        )
        .expect("register cons literal")
        .with_equation(
            FixtureEquation::new(
                head("Append"),
                vec![ctor_pattern("Nil", vec![]), var_pattern("ys")],
                var_result("ys"),
            )
            .expect("append nil fixture"),
        )
        .expect("register append nil")
        .with_equation(
            FixtureEquation::new(
                head("Append"),
                vec![
                    ctor_pattern("Cons", vec![var_pattern("h"), var_pattern("t")]),
                    var_pattern("ys"),
                ],
                ctor_result(
                    "Cons",
                    vec![
                        var_result("h"),
                        app_result("Append", vec![var_result("t"), var_result("ys")]),
                    ],
                ),
            )
            .expect("append cons fixture"),
        )
        .expect("register append cons")
}

fn defeq(lhs: &CanonicalTypeExpr, rhs: &CanonicalTypeExpr) -> DefinitionalEqualityResult {
    let env = TypeEnv::new();
    let registry = registry();
    Normalizer::with_registry(&env, &registry)
        .definitional_equality(lhs, rhs)
        .expect("definitional equality normalizes")
}

#[test]
fn task_824_closed_reductions_compare_equal_by_normal_form() {
    let lhs = app(
        "Append",
        vec![cons_expr(primitive("A"), nil_expr()), nil_expr()],
    );
    let rhs = cons_expr(primitive("A"), nil_expr());

    assert_eq!(defeq(&lhs, &rhs), DefinitionalEqualityResult::Equal);

    let env = TypeEnv::new();
    let registry = registry();
    assert!(
        Normalizer::with_registry(&env, &registry)
            .definitionally_equal(&lhs, &rhs)
            .expect("boolean wrapper uses structured equality")
    );
}

#[test]
fn task_824_open_neutral_computation_apps_compare_structurally() {
    let lhs = app("Append", vec![var("Xs"), nil_expr()]);
    let rhs = app("Append", vec![var("Xs"), nil_expr()]);

    assert_eq!(defeq(&lhs, &rhs), DefinitionalEqualityResult::Equal);
}

#[test]
fn task_824_neutral_and_rigid_projections_compare_by_identity_rigidity_and_normalized_args() {
    let reducible_arg = app("Append", vec![nil_expr(), nil_expr()]);
    let rigid_lhs = projection(ProjectionRigidity::Rigid, vec![reducible_arg.clone()]);
    let rigid_rhs = projection(ProjectionRigidity::Rigid, vec![nil_expr()]);
    assert_eq!(
        defeq(&rigid_lhs, &rigid_rhs),
        DefinitionalEqualityResult::Equal
    );

    let neutral_lhs = projection(ProjectionRigidity::Neutral, vec![reducible_arg]);
    let neutral_rhs = projection(ProjectionRigidity::Neutral, vec![nil_expr()]);
    assert_eq!(
        defeq(&neutral_lhs, &neutral_rhs),
        DefinitionalEqualityResult::Equal
    );

    let mismatch = defeq(&rigid_lhs, &neutral_rhs);
    assert!(matches!(
        mismatch,
        DefinitionalEqualityResult::BlockedByNeutrality { .. }
    ));
}

#[test]
fn task_824_closed_normalized_mismatch_reports_normal_slices() {
    let lhs = nil_expr();
    let rhs = cons_expr(primitive("A"), nil_expr());

    let result = defeq(&lhs, &rhs);

    assert_eq!(
        result,
        DefinitionalEqualityResult::NotEqual {
            lhs_norm: nil_normal(),
            rhs_norm: cons_normal(NormalTypeExpr::Primitive("A".to_string()), nil_normal()),
            mismatch: "root".to_string(),
        }
    );
}

#[test]
fn task_824_inequality_with_neutral_blockers_reports_non_inverting_evidence() {
    let lhs = app("Append", vec![var("Xs"), nil_expr()]);
    let rhs = nil_expr();

    let result = defeq(&lhs, &rhs);

    match result {
        DefinitionalEqualityResult::BlockedByNeutrality {
            lhs_norm,
            rhs_norm,
            neutral_subterms,
            no_inversion_note,
        } => {
            assert_eq!(
                lhs_norm,
                NormalTypeExpr::NeutralComputationApp {
                    head: head("Append"),
                    args: vec![NormalTypeExpr::Var("Xs".to_string()), nil_normal()],
                    kind: Kind::Type,
                    reason: Some(NormalFormBlockReason::AbstractScrutinee),
                }
            );
            assert_eq!(rhs_norm, nil_normal());
            assert_eq!(neutral_subterms, vec![lhs_norm]);
            assert!(no_inversion_note.contains("does not invert"));
        }
        other => panic!("expected neutrality-blocked evidence, got {other:?}"),
    }
}

#[test]
fn task_824_normalization_errors_propagate_from_equality() {
    let env = TypeEnv::new();
    let registry = registry();
    let normalizer = Normalizer::with_config_and_registry(
        &env,
        NormalizationConfig {
            mode: NormalizationMode::Full,
            fuel: NormalizationFuel::new(1),
            trace: false,
        },
        &registry,
    );

    let err = normalizer
        .definitional_equality(&app("Append", vec![nil_expr(), nil_expr()]), &nil_expr())
        .expect_err("normalization fuel failures propagate through equality");

    assert_eq!(
        err,
        NormalizationError::FuelExhausted {
            mode: NormalizationMode::Full,
            remaining: 0,
        }
    );
}
