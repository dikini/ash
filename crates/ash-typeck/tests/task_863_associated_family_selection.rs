use ash_core::kind::Kind;
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    AssociatedMemberIdentityId, InterfaceIdentityId, ModuleIdentity, ModuleSourceOrigin,
    SourceAnchor, SourceOrigin,
};
use ash_core::type_ir::{
    AssociatedFamilyEquation, AssociatedFamilyHeadId, AssociatedFamilyPattern,
    AssociatedFamilyResultConstraint, AssociatedFamilyResultExpr, AssociatedFamilyScheme,
    CanonicalTypeExpr, ProjectionRigidity, TypeComputationHeadId,
};
use ash_parser::surface::{
    AssociatedTypeBinding, AssociatedTypeDecl, AssociatedTypeKind, ImplDef, InterfaceDef,
    InterfaceTypeParam, Type as SurfaceType, Visibility,
};
use ash_parser::token::Span;
use ash_typeck::type_env::{AssociatedFamilySelection, AssociatedFamilySelectionBlocker, TypeEnv};

fn span() -> Span {
    Span::default()
}

fn anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "TASK-863 test".to_string(),
        },
        None,
        label,
    )
}

fn module(name: &str, id: usize) -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(863)),
        ModuleId(id),
        vec![name.to_string()],
        ModuleSourceOrigin::Synthetic {
            reason: format!("TASK-863 {name}"),
        },
    )
}

fn param(name: &str, domain: Option<&str>) -> InterfaceTypeParam {
    InterfaceTypeParam {
        name: name.into(),
        domain: domain.map(|name| SurfaceType::Name(name.into())),
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
        type_params: vec![param("I", None)],
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

fn identity_family_interface_def() -> InterfaceDef {
    InterfaceDef {
        visibility: Visibility::Inherited,
        name: "IdentityFamily".into(),
        type_params: vec![param("T", None)],
        evidence_constraints: vec![],
        associated_types: vec![AssociatedTypeDecl {
            name: "Out".into(),
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
        type_params: vec![param(param_name, None)],
        type_args: vec![list_ty(name_ty(param_name))],
        where_bounds: vec![],
        associated_type_bindings: vec![AssociatedTypeBinding {
            name: "Item".into(),
            ty: name_ty(param_name),
            span: span(),
        }],
        methods: vec![],
        span: span(),
    }
}

fn iterator_surface_list_impl(param_name: &str) -> ImplDef {
    ImplDef {
        visibility: Visibility::Inherited,
        interface: "Iterator".into(),
        type_params: vec![param(param_name, None)],
        type_args: vec![SurfaceType::List(Box::new(name_ty(param_name)))],
        where_bounds: vec![],
        associated_type_bindings: vec![AssociatedTypeBinding {
            name: "Item".into(),
            ty: name_ty(param_name),
            span: span(),
        }],
        methods: vec![],
        span: span(),
    }
}

fn identity_impl(param_name: &str) -> ImplDef {
    ImplDef {
        visibility: Visibility::Inherited,
        interface: "IdentityFamily".into(),
        type_params: vec![param(param_name, None)],
        type_args: vec![name_ty(param_name)],
        where_bounds: vec![],
        associated_type_bindings: vec![AssociatedTypeBinding {
            name: "Out".into(),
            ty: name_ty(param_name),
            span: span(),
        }],
        methods: vec![],
        span: span(),
    }
}

fn env_with_iterator_item_family() -> TypeEnv {
    let owner = module("iterator_owner", 1);
    let mut env = TypeEnv::with_builtin_types();
    env.set_current_module_identity(owner);
    env.register_interface(&iterator_interface_def())
        .expect("Iterator family declaration registers");
    env.register_impl(&iterator_list_impl("T"))
        .expect("Iterator<List<T>> family impl registers");
    env
}

fn env_with_identity_family() -> TypeEnv {
    let owner = module("identity_owner", 2);
    let mut env = TypeEnv::with_builtin_types();
    env.set_current_module_identity(owner);
    env.register_interface(&identity_family_interface_def())
        .expect("IdentityFamily declaration registers");
    env.register_impl(&identity_impl("T"))
        .expect("IdentityFamily<T>::Out = T impl registers");
    env
}

fn family_head(env: &TypeEnv, interface: &str, member: &str) -> AssociatedFamilyHeadId {
    env.lookup_associated_family_declaration(interface, member)
        .expect("family declaration exists")
        .head
        .clone()
}

fn canonical(env: &TypeEnv, ty: SurfaceType) -> CanonicalTypeExpr {
    env.lower_surface_type_to_canonical(&ty)
        .expect("surface type lowers to canonical")
}

fn result_var_name(result: &AssociatedFamilyResultExpr) -> &str {
    match result {
        AssociatedFamilyResultExpr::Var { name, .. } => name,
        other => panic!("expected associated-family result variable, got {other:?}"),
    }
}

#[test]
fn task_863_iterator_list_item_reduces_concrete_list_spine_to_element() {
    let env = env_with_iterator_item_family();
    let head = family_head(&env, "Iterator", "Item");
    let arg = canonical(&env, list_ty(name_ty("A")));

    let reduction = env
        .reduce_associated_family_projection_once(&head, &[arg])
        .expect("Iterator<List<A>>::Item should reduce through unique generic scheme");

    assert_eq!(result_var_name(&reduction.result), "A");
    assert_eq!(
        reduction.selected.scheme_param_bindings.get("T"),
        Some(&CanonicalTypeExpr::Var("A".to_string()))
    );
    assert!(
        !reduction.selected.scheme_param_bindings.contains_key("A"),
        "selection must bind only scheme-owned T, not caller variable A"
    );
}

#[test]
fn task_863_iterator_list_item_reduces_abstract_element_without_solving_query_var() {
    let env = env_with_iterator_item_family();
    let head = family_head(&env, "Iterator", "Item");
    let arg = canonical(&env, list_ty(name_ty("X")));

    let reduction = env
        .reduce_associated_family_projection_once(&head, &[arg])
        .expect("Iterator<List<X>>::Item should reduce by binding scheme T := X");

    assert_eq!(result_var_name(&reduction.result), "X");
    assert_eq!(
        reduction.selected.scheme_param_bindings.get("T"),
        Some(&CanonicalTypeExpr::Var("X".to_string()))
    );
    assert!(
        !reduction.selected.scheme_param_bindings.contains_key("X"),
        "query variable X must remain opaque and unsolved"
    );
}

#[test]
fn task_863_open_query_variable_does_not_select_by_inversion() {
    let env = env_with_iterator_item_family();
    let head = family_head(&env, "Iterator", "Item");

    let selection =
        env.select_associated_family_scheme(&head, &[CanonicalTypeExpr::Var("I".to_string())]);

    assert!(
        matches!(
            selection,
            AssociatedFamilySelection::Blocked {
                reason: AssociatedFamilySelectionBlocker::AbstractScrutinee,
                ..
            }
        ),
        "Iterator<I>::Item must not solve I := List<T>: {selection:?}"
    );
}

#[test]
fn task_863_expected_output_shape_is_not_used_to_select_open_input() {
    let env = env_with_iterator_item_family();
    let head = family_head(&env, "Iterator", "Item");
    let expected_output = AssociatedFamilyResultExpr::Primitive {
        name: "String".to_string(),
        kind: Kind::Type,
        constraint: AssociatedFamilyResultConstraint::Kind(Kind::Type),
        source_anchor: anchor("expected output ignored"),
    };

    let selection =
        env.select_associated_family_scheme(&head, &[CanonicalTypeExpr::Var("I".to_string())]);

    assert!(matches!(
        expected_output,
        AssociatedFamilyResultExpr::Primitive { .. }
    ));
    assert!(
        matches!(
            selection,
            AssociatedFamilySelection::Blocked {
                reason: AssociatedFamilySelectionBlocker::AbstractScrutinee,
                ..
            }
        ),
        "expected output must not make Iterator<I>::Item select Iterator<List<T>>: {selection:?}"
    );
}

#[test]
fn task_863_neutral_query_head_is_not_captured_by_scheme_variable() {
    let env = env_with_identity_family();
    let head = family_head(&env, "IdentityFamily", "Out");
    let neutral = CanonicalTypeExpr::ComputationHeadApp {
        head: TypeComputationHeadId::new(module("neutral", 3), "Unknown"),
        args: vec![CanonicalTypeExpr::Var("X".to_string())],
        kind: Kind::Type,
    };

    let selection = env.select_associated_family_scheme(&head, &[neutral]);

    assert!(
        matches!(
            selection,
            AssociatedFamilySelection::Blocked {
                reason: AssociatedFamilySelectionBlocker::NeutralScrutinee,
                ..
            }
        ),
        "scheme variable must not capture neutral computation query heads: {selection:?}"
    );
}

#[test]
fn task_863_rigid_projection_query_arg_is_not_captured_by_scheme_variable() {
    let env = env_with_identity_family();
    let head = family_head(&env, "IdentityFamily", "Out");
    let interface = InterfaceIdentityId::new(module("rigid", 4), "Bound");
    let member = AssociatedMemberIdentityId::associated_type(
        interface.clone(),
        "Assoc",
        vec!["Bound".to_string(), "Assoc".to_string()],
    );
    let rigid_projection = CanonicalTypeExpr::Projection {
        interface,
        member,
        args: vec![CanonicalTypeExpr::Var("T".to_string())],
        kind: Kind::Type,
        rigidity: ProjectionRigidity::Rigid,
    };

    let selection = env.select_associated_family_scheme(&head, &[rigid_projection]);

    assert!(
        matches!(
            selection,
            AssociatedFamilySelection::Blocked {
                reason: AssociatedFamilySelectionBlocker::RigidProjection,
                ..
            }
        ),
        "scheme variable must not capture rigid projection query args: {selection:?}"
    );
}

#[test]
fn task_863_unique_selection_returns_scheme_evidence_and_bindings() {
    let env = env_with_iterator_item_family();
    let head = family_head(&env, "Iterator", "Item");
    let arg = canonical(&env, list_ty(name_ty("X")));

    let selection = env.select_associated_family_scheme(&head, &[arg]);

    match selection {
        AssociatedFamilySelection::Selected(selected) => {
            assert_eq!(selected.family_head, head);
            assert_eq!(selected.equation.ordinal, 0);
            assert_eq!(
                selected.scheme_param_bindings.get("T"),
                Some(&CanonicalTypeExpr::Var("X".to_string()))
            );
        }
        other => panic!("expected unique selected Iterator<List<X>> scheme, got {other:?}"),
    }
}

#[test]
fn task_863_missing_scheme_reports_no_match_without_guessing() {
    let owner = module("empty", 5);
    let mut env = TypeEnv::with_builtin_types();
    env.set_current_module_identity(owner);
    env.register_interface(&iterator_interface_def())
        .expect("Iterator family declaration registers");
    let head = family_head(&env, "Iterator", "Item");
    let arg = canonical(&env, list_ty(name_ty("A")));

    let selection = env.select_associated_family_scheme(&head, &[arg]);

    assert!(
        matches!(selection, AssociatedFamilySelection::NoMatch { .. }),
        "missing scheme must not invent a reduction: {selection:?}"
    );
}

#[test]
fn task_863_surface_list_syntax_participates_in_family_selection() {
    let owner = module("surface_list_owner", 6);
    let mut env = TypeEnv::with_builtin_types();
    env.set_current_module_identity(owner);
    env.register_interface(&iterator_interface_def())
        .expect("Iterator family declaration registers");
    env.register_impl(&iterator_surface_list_impl("T"))
        .expect("Iterator<[T]> family impl registers");
    let head = family_head(&env, "Iterator", "Item");
    let arg = canonical(&env, list_ty(name_ty("X")));

    let reduction = env
        .reduce_associated_family_projection_once(&head, &[arg])
        .expect("Iterator<[X]>::Item should reduce through list syntax");

    assert_eq!(result_var_name(&reduction.result), "X");
}

#[test]
fn task_863_concrete_primitive_pattern_does_not_capture_other_primitives() {
    let owner = module("primitive_owner", 7);
    let mut env = TypeEnv::with_builtin_types();
    env.set_current_module_identity(owner.clone());
    env.register_interface(&iterator_interface_def())
        .expect("Iterator family declaration registers");
    let head = family_head(&env, "Iterator", "Item");
    let list_origin = env
        .type_identity_for_name("List")
        .expect("List builtin has a canonical identity")
        .clone();
    let scheme = AssociatedFamilyScheme {
        head: head.clone(),
        params: vec![],
        result_domain: CanonicalTypeExpr::Primitive("Type".to_string()),
        result_kind: Kind::Type,
        equations: vec![AssociatedFamilyEquation {
            head: head.clone(),
            ordinal: 0,
            interface_arg_patterns: vec![AssociatedFamilyPattern::NominalApp {
                origin: list_origin,
                visible_name: "List".to_string(),
                args: vec![AssociatedFamilyPattern::Primitive {
                    name: "Int".to_string(),
                    constraint: AssociatedFamilyResultConstraint::Kind(Kind::Type),
                    source_anchor: anchor("List<Int> primitive pattern"),
                }],
                constraint: AssociatedFamilyResultConstraint::Kind(Kind::Type),
                source_anchor: anchor("List<Int> pattern"),
            }],
            result: AssociatedFamilyResultExpr::Primitive {
                name: "Bool".to_string(),
                kind: Kind::Type,
                constraint: AssociatedFamilyResultConstraint::Kind(Kind::Type),
                source_anchor: anchor("Bool result"),
            },
            decreases: None,
            source_anchor: anchor("Iterator<List<Int>>::Item scheme"),
            case_head_anchor: anchor("Iterator<List<Int>>::Item case"),
        }],
        source_anchor: anchor("Iterator<List<Int>>::Item scheme"),
    };
    env.register_associated_family_scheme(scheme, owner)
        .expect("manual Iterator<List<Int>> family scheme registers");

    let int_arg = canonical(&env, list_ty(name_ty("Int")));
    let int_reduction = env
        .reduce_associated_family_projection_once(&head, &[int_arg])
        .expect("Iterator<List<Int>>::Item should select the concrete primitive scheme");
    match int_reduction.result {
        AssociatedFamilyResultExpr::Primitive { name, .. } => assert_eq!(name, "Bool"),
        other => panic!("expected Bool primitive result, got {other:?}"),
    }

    let string_arg = canonical(&env, list_ty(name_ty("String")));
    let selection = env.select_associated_family_scheme(&head, &[string_arg]);
    assert!(
        matches!(selection, AssociatedFamilySelection::NoMatch { .. }),
        "List<Int> pattern must not capture List<String>: {selection:?}"
    );
}
