use ash_core::ast::TypeExpr;
use ash_core::kind::Kind;
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    AssociatedMemberIdentityId, InterfaceIdentityId, ModuleIdentity, ModuleSourceOrigin,
};
use ash_core::type_ir::{
    AssociatedFamilyHeadId, CanonicalTypeExpr, NormalFormBlockReason, NormalTypeExpr,
    ProjectionRigidity,
};
use ash_parser::surface::{
    AssociatedTypeDecl, AssociatedTypeKind, ImplDef, InterfaceDef, InterfaceTypeParam,
    Type as SurfaceType, Visibility,
};
use ash_parser::token::Span;
use ash_typeck::normalizer::{
    DefinitionalEqualityResult, NormalizationEvidence, Normalizer, NormalizerDiagnosticKind,
};
use ash_typeck::type_env::{
    AssociatedFamilySelection, AssociatedFamilySelectionBlocker, TypeEnv, type_expr_to_type,
};
use ash_typeck::{Type, TypeVar};

fn module(name: &str, id: usize) -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(864)),
        ModuleId(id),
        vec!["task_864".into(), name.into()],
        ModuleSourceOrigin::Synthetic {
            reason: format!("TASK-864 {name}"),
        },
    )
}

fn span() -> Span {
    Span::default()
}

fn param(name: &str) -> InterfaceTypeParam {
    InterfaceTypeParam {
        name: name.into(),
        domain: None,
        kind: None,
        span: span(),
    }
}

fn name_ty(name: &str) -> SurfaceType {
    SurfaceType::Name(name.into())
}

fn list_ty(item: SurfaceType) -> SurfaceType {
    SurfaceType::Constructor {
        name: "List".into(),
        args: vec![item],
    }
}

fn iterator_interface_def() -> InterfaceDef {
    InterfaceDef {
        visibility: Visibility::Inherited,
        name: "Iterator".into(),
        type_params: vec![param("Self")],
        evidence_constraints: vec![],
        associated_types: vec![AssociatedTypeDecl {
            name: "Item".into(),
            kind: AssociatedTypeKind::SealedFamily {
                result_domain: SurfaceType::Name("Type".into()),
                decreases: None,
                span: span(),
            },
            span: span(),
        }],
        methods: vec![],
        laws: Vec::new(),
        span: span(),
    }
}

fn iterator_list_impl(param_name: &str) -> ImplDef {
    ImplDef {
        visibility: Visibility::Inherited,
        interface: "Iterator".into(),
        type_params: vec![param(param_name)],
        type_args: vec![list_ty(name_ty(param_name))],
        where_bounds: vec![],
        associated_type_bindings: vec![ash_parser::surface::AssociatedTypeBinding {
            name: "Item".into(),
            ty: name_ty(param_name),
            span: span(),
        }],
        methods: vec![],
        proofs: Vec::new(),
        span: span(),
    }
}

fn family_head(env: &TypeEnv) -> AssociatedFamilyHeadId {
    env.lookup_associated_family_declaration("Iterator", "Item")
        .expect("Iterator::Item family declaration exists")
        .head
        .clone()
}

fn env_with_iterator_family() -> TypeEnv {
    let owner = module("iterator_owner", 1);
    let mut env = TypeEnv::with_builtin_types();
    env.set_current_module_identity(owner);
    env.register_interface(&iterator_interface_def())
        .expect("Iterator family declaration registers");
    env.register_impl(&iterator_list_impl("A"))
        .expect("Iterator<List<A>>::Item family impl registers");
    env
}

#[test]
fn task_864_in_bounds_type_projection_lowers_to_rigid_canonical_projection() {
    let mut env = env_with_iterator_family();
    let type_var = TypeVar(8640);
    env.bind_type_var_interface_bound(type_var, "Iterator");

    let mut mapping = std::collections::HashMap::new();
    mapping.insert("T".to_string(), type_var);

    let ty = type_expr_to_type(
        &TypeExpr::Associated {
            base: Box::new(TypeExpr::Named("T".into())),
            name: "Item".into(),
        },
        &mapping,
        &env,
    )
    .expect("where-bound T: Iterator should resolve T::Item as an in-bounds projection");
    assert_eq!(
        ty,
        Type::Associated {
            interface: "Iterator".into(),
            base: Box::new(Type::Var(type_var)),
            name: "Item".into(),
        }
    );

    let canonical = env
        .lower_type_to_canonical_for_equality(&ty)
        .expect("in-bounds associated projection should canonicalize at equality boundary");
    match canonical {
        CanonicalTypeExpr::Projection {
            interface,
            member,
            args,
            rigidity,
            ..
        } => {
            assert_eq!(interface.name, "Iterator");
            assert_eq!(member.name, "Item");
            assert_eq!(rigidity, ProjectionRigidity::Rigid);
            assert_eq!(args.len(), 1);
            assert!(matches!(args[0], CanonicalTypeExpr::Var(_)));
        }
        other => panic!("expected rigid canonical projection for T::Item, got {other:?}"),
    }
}

fn interface() -> InterfaceIdentityId {
    InterfaceIdentityId::new(module("manual_projection", 2), "Iterator")
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

#[test]
fn task_864_where_bound_projection_normalizes_as_rigid_projection() {
    let env = TypeEnv::new();
    let normalizer = Normalizer::new(&env);
    let expr = projection(
        ProjectionRigidity::Rigid,
        vec![CanonicalTypeExpr::Var("T".into())],
    );

    let outcome = normalizer
        .normalize(&expr)
        .expect("rigid projection should normalize to preserved projection");

    assert_eq!(
        outcome.evidence,
        NormalizationEvidence::ProjectionPreserved {
            rigidity: ProjectionRigidity::Rigid
        }
    );
    match outcome.normal {
        NormalTypeExpr::Projection {
            rigidity,
            reason,
            args,
            ..
        } => {
            assert_eq!(rigidity, ProjectionRigidity::Rigid);
            assert_eq!(reason, Some(NormalFormBlockReason::RigidProjection));
            assert_eq!(args, vec![NormalTypeExpr::Var("T".into())]);
        }
        other => panic!("expected rigid projection normal form, got {other:?}"),
    }
}

#[test]
fn task_864_where_bound_evidence_does_not_select_family_scheme() {
    let env = env_with_iterator_family();
    let head = family_head(&env);
    let rigid_item = projection(
        ProjectionRigidity::Rigid,
        vec![CanonicalTypeExpr::Var("T".into())],
    );

    let selection = env.select_associated_family_scheme(&head, &[rigid_item]);

    assert!(
        matches!(
            selection,
            AssociatedFamilySelection::Blocked {
                reason: AssociatedFamilySelectionBlocker::RigidProjection,
                ..
            }
        ),
        "where-bound projection evidence must not trigger speculative family impl search: {selection:?}"
    );
}

#[test]
fn task_864_rigid_projection_equality_is_structural_only() {
    let env = TypeEnv::new();
    let normalizer = Normalizer::new(&env);
    let lhs = projection(
        ProjectionRigidity::Rigid,
        vec![CanonicalTypeExpr::Var("T".into())],
    );
    let rhs = projection(
        ProjectionRigidity::Rigid,
        vec![CanonicalTypeExpr::Var("T".into())],
    );

    let result = normalizer
        .definitional_equality(&lhs, &rhs)
        .expect("defeq should compare rigid projections structurally");

    assert_eq!(result, DefinitionalEqualityResult::Equal);
}

#[test]
fn task_864_rigid_projection_equality_does_not_collapse_to_concrete_type() {
    let env = TypeEnv::new();
    let normalizer = Normalizer::new(&env);
    let lhs = projection(
        ProjectionRigidity::Rigid,
        vec![CanonicalTypeExpr::Var("T".into())],
    );
    let rhs = CanonicalTypeExpr::Primitive("Int".into());

    let result = normalizer
        .definitional_equality(&lhs, &rhs)
        .expect("defeq should report blocker rather than solve projection");

    match result {
        DefinitionalEqualityResult::BlockedByNeutrality {
            neutral_subterms,
            no_inversion_note,
            ..
        } => {
            assert!(
                no_inversion_note.contains("does not invert"),
                "expected explicit non-inversion note, got {no_inversion_note}"
            );
            assert!(neutral_subterms.iter().any(|term| matches!(
                term,
                NormalTypeExpr::Projection {
                    rigidity: ProjectionRigidity::Rigid,
                    reason: Some(NormalFormBlockReason::RigidProjection),
                    ..
                }
            )));
        }
        other => panic!("expected rigid projection equality to block, got {other:?}"),
    }
}

#[test]
fn task_864_concrete_family_projection_reduces_only_with_explicit_selection() {
    let env = env_with_iterator_family();
    let head = family_head(&env);
    let list_int = env
        .lower_surface_type_to_canonical(&list_ty(name_ty("Int")))
        .expect("List<Int> lowers canonically");

    let reduced = env
        .reduce_associated_family_projection_once(&head, &[list_int])
        .expect("explicit concrete family projection should reduce");

    match reduced.result {
        ash_core::type_ir::AssociatedFamilyResultExpr::Primitive { name, .. } => {
            assert_eq!(name, "Int")
        }
        other => panic!("expected Iterator<List<Int>>::Item to reduce to Int, got {other:?}"),
    }
}

#[test]
fn task_864_forcing_point_diagnostic_mentions_concrete_reduction_boundary() {
    let env = TypeEnv::new();
    let normalizer = Normalizer::new(&env);
    let expr = projection(
        ProjectionRigidity::Rigid,
        vec![CanonicalTypeExpr::Var("T".into())],
    );

    let diagnostic = normalizer
        .require_concrete_normal_form(&expr)
        .expect_err("forcing a rigid projection should produce a diagnostic");

    assert_eq!(
        diagnostic.kind,
        NormalizerDiagnosticKind::ConcreteNormalFormRequired
    );
    assert!(
        diagnostic.message.contains("concrete")
            && diagnostic.message.contains("family")
            && diagnostic.message.contains("will not invert"),
        "diagnostic should clearly explain concrete family reduction boundary, got: {}",
        diagnostic.message
    );
    assert!(matches!(
        diagnostic.normal_slice,
        Some(NormalTypeExpr::Projection {
            rigidity: ProjectionRigidity::Rigid,
            reason: Some(NormalFormBlockReason::RigidProjection),
            ..
        })
    ));
}

#[test]
fn task_864_normalization_diagnostic_preserves_rigid_projection_note() {
    let env = TypeEnv::new();
    let normalizer = Normalizer::new(&env);
    let expr = projection(
        ProjectionRigidity::Rigid,
        vec![CanonicalTypeExpr::Var("T".into())],
    );

    let diagnostics = normalizer.diagnostics_for_normalization(&expr);

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == NormalizerDiagnosticKind::NeutralAssociatedProjectionNote
            && diagnostic.message.contains("Rigid")
            && diagnostic
                .message
                .contains("without associated-family computation")
            && matches!(
                diagnostic.normal_slice,
                Some(NormalTypeExpr::Projection {
                    rigidity: ProjectionRigidity::Rigid,
                    reason: Some(NormalFormBlockReason::RigidProjection),
                    ..
                })
            )
    }));
}
