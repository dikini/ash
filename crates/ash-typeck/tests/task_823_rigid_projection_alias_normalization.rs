use ash_core::ast::{TypeBody, TypeDef, TypeExpr, Visibility};
use ash_core::kind::Kind;
use ash_core::module_graph::ModuleId;
use ash_core::semantic_summary::{
    AssociatedMemberIdentityId, DomainConstructorId, InterfaceIdentityId, ModuleIdentity,
    ModuleSourceOrigin, SealedDomainId, TypeDeclId,
};
use ash_core::type_ir::{
    CanonicalTypeExpr, NormalFormBlockReason, NormalTypeExpr, ProjectionRigidity,
    TypeComputationHeadId,
};
use ash_typeck::TypeEnv;
use ash_typeck::normalizer::{
    FixtureDomainConstructorPattern, FixtureEquation, FixtureEquationRegistry, FixturePattern,
    FixtureResultExpr, Normalizer,
};

fn module() -> ModuleIdentity {
    ModuleIdentity::new(
        None,
        ModuleId(823),
        vec!["task_823".to_string(), "rigid_projection".to_string()],
        ModuleSourceOrigin::Synthetic {
            reason: "TASK-823 rigid projection alias normalization tests".to_string(),
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

fn app(name: &str, args: Vec<CanonicalTypeExpr>) -> CanonicalTypeExpr {
    CanonicalTypeExpr::ComputationHeadApp {
        head: head(name),
        args,
        kind: Kind::Type,
    }
}

fn nil_expr() -> CanonicalTypeExpr {
    app("NilLiteral", vec![])
}

fn nil_normal() -> NormalTypeExpr {
    NormalTypeExpr::DomainConstructorApp {
        constructor: ctor("Nil"),
        domain: domain(),
        args: vec![],
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

fn normalized_projection(
    rigidity: ProjectionRigidity,
    args: Vec<NormalTypeExpr>,
) -> NormalTypeExpr {
    NormalTypeExpr::Projection {
        interface: interface(),
        member: member(),
        args,
        kind: Kind::Type,
        rigidity,
        reason: Some(match rigidity {
            ProjectionRigidity::Rigid => NormalFormBlockReason::RigidProjection,
            ProjectionRigidity::Neutral => NormalFormBlockReason::AbstractScrutinee,
        }),
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
                head("Append"),
                vec![ctor_pattern("Nil", vec![]), var_pattern("ys")],
                var_result("ys"),
            )
            .expect("append nil fixture"),
        )
        .expect("register append nil")
}

fn normalize_with(env: &TypeEnv, expr: &CanonicalTypeExpr) -> NormalTypeExpr {
    let registry = registry();
    Normalizer::with_registry(env, &registry)
        .normalize(expr)
        .expect("normalization succeeds")
        .normal
}

fn normalize(expr: &CanonicalTypeExpr) -> NormalTypeExpr {
    normalize_with(&TypeEnv::new(), expr)
}

fn nominal(name: &str, args: Vec<CanonicalTypeExpr>) -> CanonicalTypeExpr {
    CanonicalTypeExpr::NominalApp {
        origin: TypeDeclId::ordinary(module(), name),
        visible_name: name.to_string(),
        args,
        kind: Kind::Type,
    }
}

#[test]
fn rigid_projection_argument_spine_normalizes_nested_reducible_computations() {
    let expr = projection(
        ProjectionRigidity::Rigid,
        vec![app("Append", vec![nil_expr(), nil_expr()])],
    );

    assert_eq!(
        normalize(&expr),
        normalized_projection(ProjectionRigidity::Rigid, vec![nil_normal()])
    );
}

#[test]
fn neutral_projection_argument_spine_normalizes_nested_reducible_computations_without_rigidifying()
{
    let expr = projection(
        ProjectionRigidity::Neutral,
        vec![app("Append", vec![nil_expr(), nil_expr()])],
    );

    assert_eq!(
        normalize(&expr),
        normalized_projection(ProjectionRigidity::Neutral, vec![nil_normal()])
    );
}

#[test]
fn projection_blocker_reason_is_preserved_when_projection_blocks_outer_equation() {
    let rigid = projection(ProjectionRigidity::Rigid, vec![nil_expr()]);
    let neutral = projection(ProjectionRigidity::Neutral, vec![nil_expr()]);

    assert_eq!(
        normalize(&app("Append", vec![rigid, nil_expr()])),
        NormalTypeExpr::NeutralComputationApp {
            head: head("Append"),
            args: vec![
                normalized_projection(ProjectionRigidity::Rigid, vec![nil_normal()]),
                nil_normal()
            ],
            kind: Kind::Type,
            reason: Some(NormalFormBlockReason::RigidProjection),
        }
    );
    assert_eq!(
        normalize(&app("Append", vec![neutral, nil_expr()])),
        NormalTypeExpr::NeutralComputationApp {
            head: head("Append"),
            args: vec![
                normalized_projection(ProjectionRigidity::Neutral, vec![nil_normal()]),
                nil_normal()
            ],
            kind: Kind::Type,
            reason: Some(NormalFormBlockReason::AbstractScrutinee),
        }
    );
}

#[test]
fn transparent_aliases_are_expanded_inside_normalizer_inputs_before_comparison_forms() {
    let mut env = TypeEnv::new();
    env.register_type(&TypeDef {
        name: "UserId".to_string(),
        params: vec![],
        body: TypeBody::Alias(TypeExpr::Named("String".to_string())),
        visibility: Visibility::Public,
        builtin: false,
    })
    .expect("register transparent alias");

    assert_eq!(
        normalize_with(&env, &nominal("UserId", vec![])),
        NormalTypeExpr::Primitive("String".to_string())
    );
    assert_eq!(
        normalize_with(
            &env,
            &projection(ProjectionRigidity::Rigid, vec![nominal("UserId", vec![])])
        ),
        normalized_projection(
            ProjectionRigidity::Rigid,
            vec![NormalTypeExpr::Primitive("String".to_string())],
        )
    );
}

#[test]
fn projections_do_not_compute_associated_family_outputs() {
    let expr = projection(
        ProjectionRigidity::Rigid,
        vec![CanonicalTypeExpr::Primitive("String".to_string())],
    );

    assert_eq!(
        normalize(&expr),
        normalized_projection(
            ProjectionRigidity::Rigid,
            vec![NormalTypeExpr::Primitive("String".to_string())],
        )
    );
}

#[test]
fn generic_transparent_alias_preserves_canonical_variable_spelling() {
    let mut env = TypeEnv::new();
    env.register_type(&TypeDef {
        name: "Id".to_string(),
        params: vec!["T".to_string()],
        body: TypeBody::Alias(TypeExpr::Named("T".to_string())),
        visibility: Visibility::Public,
        builtin: false,
    })
    .expect("register generic transparent alias");

    assert_eq!(
        normalize_with(
            &env,
            &nominal("Id", vec![CanonicalTypeExpr::Var("A".to_string())])
        ),
        NormalTypeExpr::Var("A".to_string())
    );
}
