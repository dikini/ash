use ash_core::ast::{TypeBody, TypeDef, TypeExpr, Visibility};
use ash_core::kind::Kind;
use ash_core::module_graph::{CrateId, ModuleId};
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

fn registered_nominal(
    env: &TypeEnv,
    name: &str,
    args: Vec<CanonicalTypeExpr>,
) -> CanonicalTypeExpr {
    CanonicalTypeExpr::NominalApp {
        origin: env
            .type_identity_for_name(name)
            .cloned()
            .unwrap_or_else(|| {
                TypeDeclId::ordinary(
                    ModuleIdentity::new(
                        Some(CrateId(usize::MAX)),
                        ModuleId(usize::MAX),
                        vec!["typeenv".to_string(), "defeq_fallback".to_string()],
                        ModuleSourceOrigin::Synthetic {
                            reason: "TASK-826 guarded TypeEnv defeq fallback identity".to_string(),
                        },
                    ),
                    name,
                )
            }),
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
            reason: NormalFormBlockReason::RigidProjection,
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
            reason: NormalFormBlockReason::AbstractScrutinee,
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
        normalize_with(&env, &registered_nominal(&env, "UserId", vec![])),
        NormalTypeExpr::Primitive("String".to_string())
    );
    assert_eq!(
        normalize_with(
            &env,
            &projection(
                ProjectionRigidity::Rigid,
                vec![registered_nominal(&env, "UserId", vec![])],
            )
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
            &registered_nominal(&env, "Id", vec![CanonicalTypeExpr::Var("A".to_string())])
        ),
        NormalTypeExpr::Var("A".to_string())
    );
}

#[test]
fn transparent_alias_does_not_expand_same_visible_name_with_different_origin() {
    let mut env = TypeEnv::new();
    env.register_type(&TypeDef {
        name: "Id".to_string(),
        params: vec!["T".to_string()],
        body: TypeBody::Alias(TypeExpr::Named("T".to_string())),
        visibility: Visibility::Public,
        builtin: false,
    })
    .expect("register local transparent alias");

    let foreign_origin = TypeDeclId::ordinary(
        ModuleIdentity::new(
            None,
            ModuleId(9_826),
            vec!["task_823".to_string(), "foreign_id".to_string()],
            ModuleSourceOrigin::Synthetic {
                reason: "foreign same-visible-name alias identity".to_string(),
            },
        ),
        "Id",
    );
    let foreign_id = CanonicalTypeExpr::NominalApp {
        origin: foreign_origin.clone(),
        visible_name: "Id".to_string(),
        args: vec![CanonicalTypeExpr::Var("A".to_string())],
        kind: Kind::Type,
    };

    assert_eq!(
        normalize_with(&env, &foreign_id),
        NormalTypeExpr::NominalApp {
            origin: foreign_origin,
            visible_name: "Id".to_string(),
            args: vec![NormalTypeExpr::Var("A".to_string())],
            kind: Kind::Type,
        }
    );
}

#[test]
fn transparent_alias_bridge_preserves_unregistered_nominal_origin() {
    let mut env = TypeEnv::new();
    env.register_type(&TypeDef {
        name: "Id".to_string(),
        params: vec!["T".to_string()],
        body: TypeBody::Alias(TypeExpr::Named("T".to_string())),
        visibility: Visibility::Public,
        builtin: false,
    })
    .expect("register generic transparent alias");

    let external_origin = TypeDeclId::ordinary(
        ModuleIdentity::new(
            None,
            ModuleId(9_823),
            vec!["task_823".to_string(), "external".to_string()],
            ModuleSourceOrigin::Synthetic {
                reason: "unregistered nominal origin for alias bridge regression".to_string(),
            },
        ),
        "External",
    );
    let external = CanonicalTypeExpr::NominalApp {
        origin: external_origin.clone(),
        visible_name: "External".to_string(),
        args: vec![],
        kind: Kind::Type,
    };

    assert_eq!(
        normalize_with(
            &env,
            &registered_nominal(&env, "Id", vec![external.clone()])
        ),
        NormalTypeExpr::NominalApp {
            origin: external_origin,
            visible_name: "External".to_string(),
            args: vec![],
            kind: Kind::Type,
        }
    );
    assert_eq!(
        normalize_with(
            &env,
            &registered_nominal(&env, "Id", vec![external.clone()])
        ),
        normalize_with(&env, &external)
    );
}

#[test]
fn transparent_alias_bridge_preserves_same_visible_nominals_with_different_origins() {
    let mut env = TypeEnv::new();
    env.register_type(&TypeDef {
        name: "Pair".to_string(),
        params: vec!["T".to_string(), "U".to_string()],
        body: TypeBody::Struct(vec![
            ("fst".to_string(), TypeExpr::Named("T".to_string())),
            ("snd".to_string(), TypeExpr::Named("U".to_string())),
        ]),
        visibility: Visibility::Public,
        builtin: false,
    })
    .expect("register pair type");
    env.register_type(&TypeDef {
        name: "PairAlias".to_string(),
        params: vec!["T".to_string(), "U".to_string()],
        body: TypeBody::Alias(TypeExpr::Constructor {
            name: "Pair".to_string(),
            args: vec![
                TypeExpr::Named("T".to_string()),
                TypeExpr::Named("U".to_string()),
            ],
        }),
        visibility: Visibility::Public,
        builtin: false,
    })
    .expect("register pair transparent alias");

    let origin_a = TypeDeclId::ordinary(
        ModuleIdentity::new(
            None,
            ModuleId(9_824),
            vec!["task_823".to_string(), "external_a".to_string()],
            ModuleSourceOrigin::Synthetic {
                reason: "first same-name unregistered nominal".to_string(),
            },
        ),
        "External",
    );
    let origin_b = TypeDeclId::ordinary(
        ModuleIdentity::new(
            None,
            ModuleId(9_825),
            vec!["task_823".to_string(), "external_b".to_string()],
            ModuleSourceOrigin::Synthetic {
                reason: "second same-name unregistered nominal".to_string(),
            },
        ),
        "External",
    );
    let external_a = CanonicalTypeExpr::NominalApp {
        origin: origin_a.clone(),
        visible_name: "External".to_string(),
        args: vec![],
        kind: Kind::Type,
    };
    let external_b = CanonicalTypeExpr::NominalApp {
        origin: origin_b.clone(),
        visible_name: "External".to_string(),
        args: vec![],
        kind: Kind::Type,
    };

    match normalize_with(
        &env,
        &registered_nominal(&env, "PairAlias", vec![external_a, external_b]),
    ) {
        NormalTypeExpr::NominalApp {
            origin,
            visible_name,
            args,
            kind,
        } => {
            assert_eq!(origin.module.crate_id, Some(CrateId(usize::MAX)));
            assert_eq!(origin.module.module_id, ModuleId(usize::MAX));
            assert_eq!(visible_name, "Pair");
            assert_eq!(kind, Kind::Type);
            assert_eq!(
                args,
                vec![
                    NormalTypeExpr::NominalApp {
                        origin: origin_a,
                        visible_name: "External".to_string(),
                        args: vec![],
                        kind: Kind::Type,
                    },
                    NormalTypeExpr::NominalApp {
                        origin: origin_b,
                        visible_name: "External".to_string(),
                        args: vec![],
                        kind: Kind::Type,
                    },
                ]
            );
        }
        other => panic!("expected Pair nominal app, got {other:?}"),
    }
}
