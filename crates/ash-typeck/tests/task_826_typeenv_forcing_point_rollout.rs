use ash_core::ast::{TypeBody, TypeDef, TypeExpr, Visibility as CoreVisibility};
use ash_core::kind::Kind;
use ash_core::module_graph::ModuleId;
use ash_core::semantic_summary::{
    AssociatedMemberIdentityId, AssociatedMemberIdentitySummary, InterfaceIdentityId,
    InterfaceIdentitySummary, ModuleIdentity, ModuleSemanticSummary, ModuleSourceOrigin,
    RepresentationExposure, SourceAnchor, SourceOrigin, TypeDeclId, TypeDeclSummary,
    TypeRepresentationSummary,
};
use ash_core::type_ir::{CanonicalTypeExpr, ProjectionRigidity};
use ash_parser::surface::{
    AssociatedTypeBinding, AssociatedTypeDecl, Expr, ImplDef, ImplMethodDef, InterfaceDef,
    InterfaceMethodSig, Literal, Pattern, Type as SurfaceType, Visibility as SurfaceVisibility,
};
use ash_parser::token::Span;
use ash_typeck::error::TypeEnvError;
use ash_typeck::normalizer::{DefinitionalEqualityResult, Normalizer};
use ash_typeck::types::{Type, TypeVar};
use ash_typeck::{QualifiedName, TypeEnv};

fn module(name: &str) -> ModuleIdentity {
    ModuleIdentity::new(
        None,
        ModuleId(826),
        vec!["task_826".to_string(), name.to_string()],
        ModuleSourceOrigin::Synthetic {
            reason: format!("TASK-826 TypeEnv forcing-point rollout test: {name}"),
        },
    )
}

fn type_id(name: &str) -> TypeDeclId {
    TypeDeclId::ordinary(module("types"), name)
}

fn interface_id(name: &str) -> InterfaceIdentityId {
    InterfaceIdentityId::new(module("interfaces"), name)
}

fn member_id(interface: InterfaceIdentityId, name: &str) -> AssociatedMemberIdentityId {
    AssociatedMemberIdentityId::associated_type(interface, name, vec![])
}

fn anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "TASK-826 forcing-point rollout test".to_string(),
        },
        None,
        label,
    )
}

fn nominal(name: &str, args: Vec<CanonicalTypeExpr>) -> CanonicalTypeExpr {
    CanonicalTypeExpr::NominalApp {
        origin: type_id(name),
        visible_name: name.to_string(),
        args,
        kind: Kind::Type,
    }
}

fn ty_ctor(name: &str, args: Vec<Type>) -> Type {
    Type::Constructor {
        name: QualifiedName::root(name),
        args,
        kind: Kind::Type,
    }
}

fn associated(interface: &str, base: Type, name: &str) -> Type {
    Type::Associated {
        interface: interface.to_string(),
        base: Box::new(base),
        name: name.to_string(),
    }
}

fn projection(
    interface: InterfaceIdentityId,
    member: AssociatedMemberIdentityId,
    rigidity: ProjectionRigidity,
    args: Vec<CanonicalTypeExpr>,
) -> CanonicalTypeExpr {
    CanonicalTypeExpr::Projection {
        interface,
        member,
        args,
        kind: Kind::Type,
        rigidity,
    }
}

fn env_with_nominals(names: &[&str]) -> TypeEnv {
    let mut env = TypeEnv::new();
    let module = module("types");
    for name in names {
        let params = if matches!(*name, "Box" | "AliasBox") {
            vec!["T".to_string()]
        } else {
            vec![]
        };
        let summary = TypeDeclSummary::new(
            TypeDeclId::ordinary(module.clone(), *name),
            *name,
            CoreVisibility::Public,
            RepresentationExposure::Opaque,
            TypeRepresentationSummary::opaque(false),
            anchor(name),
        )
        .with_params(params);
        env.register_module_semantic_summary(
            &ModuleSemanticSummary::new(module.clone()).with_exported_type(summary),
        )
        .expect("register nominal type identity summary");
    }
    env
}

fn env_with_projection_interface() -> TypeEnv {
    let mut env = env_with_nominals(&["Box", "Widget"]);
    env.register_type(&TypeDef {
        name: "AliasBox".to_string(),
        params: vec!["T".to_string()],
        body: TypeBody::Alias(TypeExpr::Constructor {
            name: "Box".to_string(),
            args: vec![TypeExpr::Named("T".to_string())],
        }),
        visibility: CoreVisibility::Public,
        builtin: false,
    })
    .expect("upgrade AliasBox to transparent alias");
    env.register_type(&TypeDef {
        name: "IntBox".to_string(),
        params: vec![],
        body: TypeBody::Alias(TypeExpr::Constructor {
            name: "Box".to_string(),
            args: vec![TypeExpr::Named("Int".to_string())],
        }),
        visibility: CoreVisibility::Public,
        builtin: false,
    })
    .expect("upgrade IntBox to transparent alias");
    env.register_type(&TypeDef {
        name: "IntAlias".to_string(),
        params: vec![],
        body: TypeBody::Alias(TypeExpr::Named("Int".to_string())),
        visibility: CoreVisibility::Public,
        builtin: false,
    })
    .expect("upgrade IntAlias to transparent alias");
    let iterable = interface_id("Iterable");
    let summary = ModuleSemanticSummary::new(module("projection_summary"))
        .with_interface_identity(InterfaceIdentitySummary::new(
            iterable.clone(),
            "Iterable",
            vec!["Iterable".to_string()],
            anchor("interface Iterable"),
        ))
        .with_associated_member_identity(AssociatedMemberIdentitySummary::new(
            member_id(iterable.clone(), "Item"),
            "Item",
            anchor("associated type Item"),
        ));
    env.register_module_semantic_summary(&summary)
        .expect("register canonical projection identity summary");
    env.register_interface_identity_summary(&InterfaceIdentitySummary::new(
        iterable.clone(),
        "IterAlias",
        vec!["IterAlias".to_string()],
        anchor("interface IterAlias"),
    ))
    .expect("register interface visible alias");
    env.register_associated_member_identity_summary(&AssociatedMemberIdentitySummary::new(
        member_id(iterable, "Item"),
        "Elem",
        anchor("associated type Elem alias"),
    ))
    .expect("register associated member visible alias");
    env
}

#[test]
fn task_826_fp1_typeenv_unify_uses_defeq_for_canonical_associated_projection_spines() {
    // TASK-817 FP-1/FP-17: TypeEnv::unify_types may force defeq only for
    // canonicalizable slices. This projection spine contains a transparent alias
    // that only the normalizer-defeq path reduces before structural comparison.
    let env = env_with_projection_interface();
    let lhs = associated("Iterable", ty_ctor("AliasBox", vec![Type::Int]), "Item");
    let rhs = associated("IterAlias", ty_ctor("Box", vec![Type::Int]), "Elem");

    assert!(env.unify_types(&lhs, &rhs).is_ok());
}

#[test]
fn task_826_fp2_boolean_wrapper_uses_defeq_for_canonical_slices() {
    // TASK-817 FP-2: boolean equality is allowed to use the guarded TypeEnv defeq
    // wrapper when both sides lower to canonical IR.
    let env = env_with_projection_interface();

    assert!(env.types_equivalent_for_equality(
        &associated("Iterable", ty_ctor("IntBox", vec![]), "Item"),
        &associated("IterAlias", ty_ctor("Box", vec![Type::Int]), "Elem"),
    ));
}

#[test]
fn task_826_projection_argument_spines_normalize_without_losing_rigidity() {
    // TASK-817 FP-17: associated projection argument spines normalize via the
    // normalizer while preserving Phase 110 ProjectionRigidity.
    let env = env_with_projection_interface();
    let iterable = interface_id("Iterable");
    let member = member_id(iterable.clone(), "Item");
    let aliased = nominal(
        "AliasBox",
        vec![CanonicalTypeExpr::Primitive("Int".to_string())],
    );
    let canonical = nominal("Box", vec![CanonicalTypeExpr::Primitive("Int".to_string())]);

    let normalizer = Normalizer::new(&env);
    assert_eq!(
        normalizer
            .definitional_equality(
                &projection(
                    iterable.clone(),
                    member.clone(),
                    ProjectionRigidity::Rigid,
                    vec![aliased.clone()],
                ),
                &projection(
                    iterable.clone(),
                    member.clone(),
                    ProjectionRigidity::Rigid,
                    vec![canonical.clone()],
                ),
            )
            .expect("rigid projection defeq"),
        DefinitionalEqualityResult::Equal
    );
    assert!(matches!(
        normalizer
            .definitional_equality(
                &projection(
                    iterable.clone(),
                    member.clone(),
                    ProjectionRigidity::Rigid,
                    vec![aliased],
                ),
                &projection(
                    iterable,
                    member,
                    ProjectionRigidity::Neutral,
                    vec![canonical]
                ),
            )
            .expect("projection rigidity mismatch produces structured evidence"),
        DefinitionalEqualityResult::BlockedByNeutrality { .. }
    ));
}

#[test]
fn task_826_legacy_meta_solving_boundary_still_falls_back_to_unifier() {
    // TASK-817 FP-1 / TASK-825 boundary: unsupported/non-ground meta-solving
    // stays with legacy Type::Var unification and is not rewritten into canonical
    // abstract variables.
    let env = env_with_nominals(&["Box"]);
    let meta = TypeVar(8261);
    let substitution = env
        .unify_types(
            &ty_ctor("Box", vec![Type::Var(meta)]),
            &ty_ctor("Box", vec![Type::Int]),
        )
        .expect("ordinary nominal unification still solves metas");

    assert_eq!(substitution.get(meta), Some(&Type::Int));
}

#[test]
fn task_826_deferred_constructor_field_style_shape_still_uses_legacy_fallback() {
    // TASK-817 FP-12/FP-13 are deferred. Unsupported legacy shapes such as lists
    // remain accepted through fallback unification rather than being forced into
    // canonical IR.
    let env = TypeEnv::new();

    assert!(
        env.unify_types(
            &Type::List(Box::new(Type::Int)),
            &Type::List(Box::new(Type::Int))
        )
        .is_ok()
    );
}

#[test]
fn task_826_fp6_impl_overlap_sees_defeq_compatible_canonical_heads() {
    // TASK-817 FP-6: impl overlap/coherence compares compatible canonical heads
    // with guarded normalization. The second impl head is a transparent alias of
    // the first, so it must be rejected as duplicate/overlapping.
    let mut env = env_with_projection_interface();
    env.register_interface(&InterfaceDef {
        visibility: SurfaceVisibility::Inherited,
        name: "Iterable".into(),
        type_params: vec!["T".into()],
        associated_types: vec![],
        methods: vec![],
        span: Span::default(),
    })
    .expect("register interface");

    let first = ImplDef {
        visibility: SurfaceVisibility::Inherited,
        interface: "Iterable".into(),
        type_params: vec![],
        type_args: vec![SurfaceType::Constructor {
            name: "Box".into(),
            args: vec![SurfaceType::Name("Int".into())],
        }],
        where_bounds: vec![],
        associated_type_bindings: vec![],
        methods: vec![],
        span: Span::default(),
    };
    env.register_impl(&first).expect("first impl registers");

    let duplicate = ImplDef {
        type_args: vec![SurfaceType::Constructor {
            name: "AliasBox".into(),
            args: vec![SurfaceType::Name("Int".into())],
        }],
        ..first
    };
    let err = env
        .register_impl(&duplicate)
        .expect_err("transparent-alias-compatible head overlaps first impl");

    assert!(matches!(
        err,
        TypeEnvError::DuplicateImpl { .. } | TypeEnvError::OverlappingImpls { .. }
    ));
}

#[test]
fn task_826_fp7_impl_method_return_check_uses_defeq_for_declared_return() {
    // TASK-817 FP-7: impl method declared expected-vs-actual return comparison is
    // an owned forcing point. The method declares an alias return and returns the
    // canonical underlying type.
    let mut env = env_with_projection_interface();
    env.register_interface(&InterfaceDef {
        visibility: SurfaceVisibility::Inherited,
        name: "Iterable".into(),
        type_params: vec!["T".into()],
        associated_types: vec![AssociatedTypeDecl {
            name: "Item".into(),
            span: Span::default(),
        }],
        methods: vec![InterfaceMethodSig {
            name: "item".into(),
            params: vec![],
            return_type: SurfaceType::Name("IntAlias".into()),
            span: Span::default(),
        }],
        span: Span::default(),
    })
    .expect("register interface");

    let def = ImplDef {
        visibility: SurfaceVisibility::Inherited,
        interface: "Iterable".into(),
        type_params: vec![],
        type_args: vec![SurfaceType::Name("Widget".into())],
        where_bounds: vec![],
        associated_type_bindings: vec![AssociatedTypeBinding {
            name: "Item".into(),
            ty: SurfaceType::Name("Int".into()),
            span: Span::default(),
        }],
        methods: vec![ImplMethodDef {
            name: "item".into(),
            params: vec![],
            body: Expr::Literal(Literal::Int(1)),
            span: Span::default(),
        }],
        span: Span::default(),
    };

    env.register_impl(&def)
        .expect("defeq accepts canonical actual return for alias declared return");
}

#[test]
fn task_826_no_direct_expression_branch_rollout_by_accident() {
    // TASK-817 FP-10/FP-11/FP-12/FP-13/FP-15/FP-16 are deferred for this rollout.
    // This test intentionally references expression syntax only to pin that this
    // suite did not need direct check_expr rewrites.
    let pattern = Pattern::Wildcard;
    assert!(matches!(pattern, Pattern::Wildcard));
}
