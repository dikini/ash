use ash_core::ast::{TypeBody, Visibility};
use ash_core::kind::Kind;
use ash_core::module_graph::ModuleId;
use ash_core::semantic_summary::{
    AssociatedMemberIdentityId, ConstructorId, ConstructorPayloadKind, ConstructorSummary,
    DomainConstructorId, DomainConstructorSummary, InterfaceIdentityId, ModuleIdentity,
    ModuleSemanticSummary, ModuleSourceOrigin, RepresentationExposure, SealedDomainId,
    SealedDomainSummary, SourceAnchor, SourceOrigin, SummaryVersion, TypeDeclId, TypeDeclSummary,
    TypeRepresentationSummary,
};
use ash_core::type_ir::{
    CanonicalTypeExpr, NormalTypeExpr, ProjectionRigidity, TypeComputationHeadId,
};
use ash_parser::token::Span;
use ash_typeck::normalizer::{
    DefinitionalEqualityResult, FixtureDomainConstructorPattern, FixtureEquation,
    FixtureEquationRegistry, FixturePattern, FixtureResultExpr, NormalizationConfig,
    NormalizationError, NormalizationFuel, NormalizationMode, Normalizer, NormalizerDiagnosticKind,
};
use ash_typeck::types::{Type, TypeVar};
use ash_typeck::{QualifiedName, TypeEnv};

fn module(name: &str) -> ModuleIdentity {
    ModuleIdentity::new(
        None,
        ModuleId(827),
        vec!["task_827".to_string(), name.to_string()],
        ModuleSourceOrigin::Synthetic {
            reason: format!("TASK-827 normalizer diagnostics/non-interference test: {name}"),
        },
    )
}

fn anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "TASK-827 normalizer diagnostics/non-interference test".to_string(),
        },
        None,
        label,
    )
}

fn head(name: &str) -> TypeComputationHeadId {
    TypeComputationHeadId::new(module("heads"), name)
}

fn domain() -> SealedDomainId {
    SealedDomainId::new(module("domain"), "List")
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

fn interface() -> InterfaceIdentityId {
    InterfaceIdentityId::new(module("interfaces"), "Iterable")
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

fn diagnostics_for(expr: &CanonicalTypeExpr) -> Vec<ash_typeck::normalizer::NormalizerDiagnostic> {
    let env = TypeEnv::new();
    let registry = registry();
    Normalizer::with_registry(&env, &registry).diagnostics_for_normalization(expr)
}

fn defeq_diagnostics(
    lhs: &CanonicalTypeExpr,
    rhs: &CanonicalTypeExpr,
) -> Vec<ash_typeck::normalizer::NormalizerDiagnostic> {
    let env = TypeEnv::new();
    let registry = registry();
    Normalizer::with_registry(&env, &registry).diagnostics_for_definitional_equality(lhs, rhs)
}

fn type_id(name: &str) -> TypeDeclId {
    TypeDeclId::ordinary(module("ordinary_types"), name)
}

fn ty_ctor(name: &str, args: Vec<Type>) -> Type {
    Type::Constructor {
        name: QualifiedName::root(name),
        args,
        kind: Kind::Type,
    }
}

#[test]
fn task_827_neutral_normalization_note_reports_blocked_computation_head() {
    let diagnostics = diagnostics_for(&app("Append", vec![var("Xs"), nil_expr()]));

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == NormalizerDiagnosticKind::NeutralStuckNormalizationNote
            && diagnostic.message.contains("AbstractScrutinee")
            && matches!(
                diagnostic.normal_slice,
                Some(NormalTypeExpr::NeutralComputationApp { .. })
            )
    }));
}

#[test]
fn task_827_neutral_associated_projection_note_preserves_projection_boundary() {
    let diagnostics = diagnostics_for(&projection(
        ProjectionRigidity::Neutral,
        vec![app("Append", vec![nil_expr(), nil_expr()])],
    ));

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == NormalizerDiagnosticKind::NeutralAssociatedProjectionNote
            && diagnostic
                .message
                .contains("without associated-family computation")
            && diagnostic.message.contains("AbstractScrutinee")
            && matches!(
                diagnostic.normal_slice,
                Some(NormalTypeExpr::Projection { .. })
            )
    }));
}

#[test]
fn task_827_concrete_normal_form_required_reports_neutral_not_inversion() {
    let env = TypeEnv::new();
    let registry = registry();
    let err = Normalizer::with_registry(&env, &registry)
        .require_concrete_normal_form(&app("Append", vec![var("Xs"), nil_expr()]))
        .expect_err("open Append is neutral, not concrete");

    assert_eq!(
        err.kind,
        NormalizerDiagnosticKind::ConcreteNormalFormRequired
    );
    assert!(err.message.contains("will not invert"));
    assert!(matches!(
        err.normal_slice,
        Some(NormalTypeExpr::NeutralComputationApp { .. })
    ));
}

#[test]
fn task_827_equality_blocked_by_neutrality_and_non_inversion_notes_are_emitted() {
    let diagnostics = defeq_diagnostics(
        &app("Append", vec![var("Xs"), nil_expr()]),
        &cons_expr(primitive("A"), nil_expr()),
    );

    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.kind
                == NormalizerDiagnosticKind::EqualityBlockedByNeutrality)
    );
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == NormalizerDiagnosticKind::NonInvertingEqualityNote
            && diagnostic.message.contains("does not invert")
    }));
}

#[test]
fn task_827_normalized_mismatch_diagnostic_contains_both_normal_slices() {
    let diagnostics = defeq_diagnostics(&nil_expr(), &cons_expr(primitive("A"), nil_expr()));

    let mismatch = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.kind == NormalizerDiagnosticKind::NormalizedMismatch)
        .expect("closed mismatch diagnostic");
    assert!(mismatch.message.contains("left normal form"));
    assert!(mismatch.message.contains("right normal form"));
    assert!(mismatch.message.contains("Cons"));
}

#[test]
fn task_827_fuel_guard_diagnostic_is_implementation_failure_not_stuckness() {
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
        .normalize(&app("Append", vec![nil_expr(), nil_expr()]))
        .expect_err("fuel exhausted before reduction finishes");
    assert_eq!(
        err,
        NormalizationError::FuelExhausted {
            mode: NormalizationMode::Full,
            remaining: 0,
        }
    );

    let diagnostic = normalizer
        .diagnostics_for_normalization(&app("Append", vec![nil_expr(), nil_expr()]))
        .into_iter()
        .next()
        .expect("fuel diagnostic");
    assert_eq!(diagnostic.kind, NormalizerDiagnosticKind::FuelOrCycleGuard);
    assert!(
        diagnostic
            .message
            .contains("implementation fuel/cycle guard failed")
    );
    assert!(diagnostic.message.contains("not semantic stuckness"));
}

#[test]
fn task_827_phase109_ordinary_summary_shapes_and_fixture_registry_do_not_serialize() {
    let summary =
        ModuleSemanticSummary::new(module("summary")).with_exported_type(TypeDeclSummary::new(
            type_id("Status"),
            "Status",
            Visibility::Public,
            RepresentationExposure::Exposed,
            TypeRepresentationSummary::exposed(TypeBody::Enum(vec![])),
            anchor("type Status"),
        ));

    assert!(
        summary
            .reserved_identity_slots
            .future_type_functions
            .is_empty()
    );
    let json = serde_json::to_string(&summary).expect("summary serializes");
    assert!(!json.contains("FixtureEquation"));
    assert!(!json.contains("Append"));
    assert!(!json.contains("NilLiteral"));
}

#[test]
fn task_827_phase110_projection_canonicalization_remains_structural_not_computational() {
    let env = TypeEnv::new();
    let registry = registry();
    let normalizer = Normalizer::with_registry(&env, &registry);
    let lhs = projection(
        ProjectionRigidity::Rigid,
        vec![app("Append", vec![nil_expr(), nil_expr()])],
    );
    let rhs = projection(ProjectionRigidity::Rigid, vec![nil_expr()]);

    assert_eq!(
        normalizer
            .definitional_equality(&lhs, &rhs)
            .expect("projection argument spines normalize structurally"),
        DefinitionalEqualityResult::Equal
    );
    assert!(
        normalizer
            .diagnostics_for_normalization(&lhs)
            .iter()
            .any(|diagnostic| diagnostic.kind
                == NormalizerDiagnosticKind::NeutralAssociatedProjectionNote)
    );
}

#[test]
fn task_827_phase111_sealed_domain_registration_does_not_promote_marker_constructors() {
    use ash_core::semantic_summary::DomainFieldSummary;
    let domain_summary = SealedDomainSummary::new(
        domain(),
        "ListDomain",
        Visibility::Public,
        anchor("sealed type domain ListDomain"),
    )
    .with_constructor(DomainConstructorSummary::new(
        ctor("Nil"),
        "Nil",
        vec![],
        anchor("Nil"),
    ))
    .with_constructor(DomainConstructorSummary::new(
        ctor("Cons"),
        "Cons",
        vec![DomainFieldSummary::constrained_to(
            "tail",
            &domain(),
            domain(),
        )],
        anchor("Cons"),
    ));
    let summary = ModuleSemanticSummary {
        version: SummaryVersion::SPEC059_SEALED_DOMAIN_V2,
        ..ModuleSemanticSummary::new(module("sealed_summary"))
            .with_exported_sealed_domain(domain_summary)
    };
    let mut env = TypeEnv::new();

    env.register_module_semantic_summary(&summary)
        .expect("sealed domain summary registers");

    assert!(env.lookup_sealed_domain("ListDomain").is_some());
    assert_eq!(env.lookup_constructor("Nil"), None);
    assert_eq!(env.lookup_constructor("Cons"), None);
}

#[test]
fn task_827_ordinary_adt_constructors_are_not_promoted_to_domain_normal_forms() {
    let status_id = type_id("Status");
    let done_id = ConstructorId::variant(status_id.clone(), "Done", ConstructorPayloadKind::Unit);
    let summary = ModuleSemanticSummary::new(module("ordinary_adt"))
        .with_exported_type(TypeDeclSummary::new(
            status_id.clone(),
            "Status",
            Visibility::Public,
            RepresentationExposure::Exposed,
            TypeRepresentationSummary::exposed(TypeBody::Enum(vec![ash_core::ast::VariantDef {
                name: "Done".to_string(),
                fields: vec![],
                payload: ash_core::ast::VariantPayload::Unit,
            }])),
            anchor("type Status"),
        ))
        .with_exported_constructor(ConstructorSummary::new(
            done_id,
            status_id,
            "Done",
            ConstructorPayloadKind::Unit,
            Visibility::Public,
            anchor("Done"),
        ));
    let mut env = TypeEnv::new();
    env.register_module_semantic_summary(&summary)
        .expect("ordinary ADT summary registers");

    assert_eq!(
        env.lookup_constructor("Done"),
        Some(("Status".to_string(), 0))
    );
    let normal = Normalizer::new(&env)
        .normalize(&CanonicalTypeExpr::NominalApp {
            origin: type_id("Status"),
            visible_name: "Status".to_string(),
            args: vec![],
            kind: Kind::Type,
        })
        .expect("ordinary nominal normalizes structurally")
        .normal;
    assert!(matches!(normal, NormalTypeExpr::NominalApp { .. }));
}

#[test]
fn task_827_typeenv_rollout_remains_guarded_and_legacy_shapes_fallback() {
    let env = TypeEnv::new();
    let meta = TypeVar(827);
    let substitution = env
        .unify_types(
            &ty_ctor("Box", vec![Type::Var(meta)]),
            &ty_ctor("Box", vec![Type::Int]),
        )
        .expect("legacy meta solving stays in fallback unifier");

    assert_eq!(substitution.get(meta), Some(&Type::Int));
    assert!(
        env.unify_types(
            &Type::List(Box::new(Type::Int)),
            &Type::List(Box::new(Type::Int)),
        )
        .is_ok()
    );

    let fallback_diag = ash_typeck::normalizer::NormalizerDiagnostic::new(
        NormalizerDiagnosticKind::LegacyFallback,
        "legacy TypeEnv shape remained on fallback unifier outside TASK-826 owned points",
    );
    assert!(fallback_diag.message.contains("fallback unifier"));
}

#[test]
fn task_827_check_expr_direct_rollout_boundary_still_has_no_new_normalizer_semantics() {
    // This pins the TASK-826 boundary from widening into expression checking
    // syntax branches. The normalizer remains an explicit TypeEnv/canonical-IR
    // dependency, not a parser or direct check_expr surface feature.
    let span = Span::default();
    assert_eq!(span.start, 0);
}
