use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{ModuleIdentity, ModuleSourceOrigin, SourceAnchor, SourceOrigin};
use ash_core::type_ir::{
    CanonicalTypeExpr, InterfaceBoundProposition, PropositionBoundary, PropositionDeferredKind,
    PropositionEvidenceRule, PropositionOutcome, TypeProposition, TypePropositionTerm,
};
use ash_parser::surface::{
    AssociatedTypeBinding, AssociatedTypeDecl, AssociatedTypeKind, ImplDef, InterfaceDef,
    InterfaceTypeParam, Type as SurfaceType, Visibility, WhereBound,
};
use ash_parser::token::Span;
use ash_typeck::type_env::PropositionCheckingSiteKind;
use ash_typeck::{TypeEnv, TypeVar};

fn module_identity(id: usize, path: &[&str]) -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(1)),
        ModuleId(id),
        path.iter().map(|part| (*part).to_string()).collect(),
        ModuleSourceOrigin::Synthetic {
            reason: format!("task-877-{id}"),
        },
    )
}

fn origin() -> SourceOrigin {
    SourceOrigin::Synthetic {
        reason: "task-877-test".into(),
    }
}

fn anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(origin(), None, label)
}

fn span() -> Span {
    Span::default()
}

fn interface_param(name: &str) -> InterfaceTypeParam {
    InterfaceTypeParam {
        name: name.into(),
        domain: None,
        kind: None,
        span: span(),
    }
}

fn surface_name(name: &str) -> SurfaceType {
    SurfaceType::Name(name.into())
}

fn surface_list(item: SurfaceType) -> SurfaceType {
    SurfaceType::Constructor {
        name: "List".into(),
        args: vec![item],
    }
}

fn interface_def(name: &str, params: &[&str]) -> InterfaceDef {
    InterfaceDef {
        visibility: Visibility::Inherited,
        name: name.into(),
        type_params: params.iter().map(|param| interface_param(param)).collect(),
        evidence_constraints: vec![],
        associated_types: vec![],
        methods: vec![],
        laws: Vec::new(),
        span: span(),
    }
}

fn proposition_bound(
    subject: TypePropositionTerm,
    interface: ash_core::semantic_summary::InterfaceIdentityId,
    interface_args: Vec<TypePropositionTerm>,
) -> TypeProposition {
    TypeProposition::InterfaceBound(InterfaceBoundProposition {
        subject,
        interface,
        interface_args,
    })
}

fn primitive(name: &str) -> TypePropositionTerm {
    TypePropositionTerm::Canonical(CanonicalTypeExpr::Primitive(name.to_string()))
}

fn type_var_bound(var: TypeVar) -> TypePropositionTerm {
    TypePropositionTerm::Canonical(CanonicalTypeExpr::Var(format!("type_var_{}", var.0)))
}

fn register_interface(env: &mut TypeEnv, name: &str, params: &[&str]) {
    env.register_interface(&interface_def(name, params))
        .expect("interface fixture should register");
}

fn env_with_displayable() -> TypeEnv {
    let mut env = TypeEnv::with_builtin_types();
    env.set_current_module_identity(module_identity(877_100, &["pkg", "display"]));
    register_interface(&mut env, "Displayable", &["Self"]);
    env
}

#[test]
fn task_877_satisfies_interface_bound_from_exact_concrete_impl_assumption() {
    let mut env = TypeEnv::with_builtin_types();
    env.set_current_module_identity(module_identity(877_101, &["pkg", "format"]));
    register_interface(&mut env, "FormatsAs", &["Self", "Format"]);
    let interface_id = env
        .interface_identity_for_name("FormatsAs")
        .expect("interface identity registered")
        .clone();

    env.register_impl(&ImplDef {
        visibility: Visibility::Inherited,
        interface: "FormatsAs".into(),
        type_params: vec![],
        type_args: vec![surface_name("Int"), surface_name("String")],
        where_bounds: vec![],
        associated_type_bindings: vec![],
        methods: vec![],
        span: span(),
    })
    .expect("concrete impl evidence should register");

    let proposition = proposition_bound(primitive("Int"), interface_id, vec![primitive("String")]);

    let outcome = env
        .solve_proposition(&proposition, Some(anchor("Int: FormatsAs<String>")))
        .expect("exact concrete impl assumption should solve interface proposition");

    match outcome {
        PropositionOutcome::Satisfied(evidence) => {
            assert_eq!(evidence.proposition, proposition);
            assert_eq!(evidence.rule, PropositionEvidenceRule::ConcreteImplEvidence);
            assert_eq!(evidence.boundary, PropositionBoundary::Local);
            assert!(evidence.normalized_terms.is_none());
            assert!(evidence.source_anchor.is_some());
        }
        other => panic!("expected concrete impl evidence to satisfy proposition, got {other:?}"),
    }
}

#[test]
fn task_877_satisfies_interface_bound_from_exact_type_variable_where_bound_assumption() {
    let mut env = env_with_displayable();
    let interface_id = env
        .interface_identity_for_name("Displayable")
        .expect("interface identity registered")
        .clone();
    env.bind_type_var_interface_bound(TypeVar(7), "Displayable");

    let proposition = proposition_bound(type_var_bound(TypeVar(7)), interface_id, vec![]);
    let outcome = env
        .solve_proposition(&proposition, Some(anchor("type_var_7: Displayable")))
        .expect("exact type-var bound assumption should solve interface proposition");

    match outcome {
        PropositionOutcome::Satisfied(evidence) => {
            assert_eq!(evidence.proposition, proposition);
            assert_eq!(
                evidence.rule,
                PropositionEvidenceRule::InScopeInterfaceBound
            );
            assert_eq!(evidence.boundary, PropositionBoundary::Local);
            assert!(evidence.normalized_terms.is_none());
        }
        other => {
            panic!("expected in-scope where-bound evidence to satisfy proposition, got {other:?}")
        }
    }
}

#[test]
fn task_877_satisfies_interface_bound_from_exact_impl_where_bound_assumption() {
    let mut env = TypeEnv::with_builtin_types();
    env.set_current_module_identity(module_identity(877_104, &["pkg", "impl_where"]));
    register_interface(&mut env, "Displayable", &["Self"]);
    register_interface(&mut env, "Container", &["Self"]);

    env.register_impl(&ImplDef {
        visibility: Visibility::Inherited,
        interface: "Container".into(),
        type_params: vec![interface_param("T")],
        type_args: vec![surface_name("T")],
        where_bounds: vec![WhereBound {
            param: "T".into(),
            bound: "Displayable".into(),
            span: span(),
        }],
        associated_type_bindings: vec![],
        methods: vec![],
        span: span(),
    })
    .expect("generic impl where-bound evidence should register");

    let proposition = env
        .proposition_assumptions()
        .iter()
        .find(|record| record.owner_site.kind == PropositionCheckingSiteKind::ImplWhereBound)
        .expect("impl where-bound assumption should be recorded")
        .proposition
        .clone();

    let outcome = env
        .solve_proposition(&proposition, Some(anchor("impl where T: Displayable")))
        .expect("exact impl where-bound assumption should solve interface proposition");

    match outcome {
        PropositionOutcome::Satisfied(evidence) => {
            assert_eq!(evidence.proposition, proposition);
            assert_eq!(
                evidence.rule,
                PropositionEvidenceRule::InScopeInterfaceBound
            );
            assert_eq!(evidence.boundary, PropositionBoundary::Local);
        }
        other => panic!("expected impl where-bound evidence to satisfy proposition, got {other:?}"),
    }
}

#[test]
fn task_877_missing_interface_bound_evidence_defers_with_no_inversion_boundary() {
    let env = env_with_displayable();
    let interface_id = env
        .interface_identity_for_name("Displayable")
        .expect("interface identity registered")
        .clone();
    let proposition = proposition_bound(primitive("Int"), interface_id, vec![]);

    let outcome = env
        .solve_proposition(
            &proposition,
            Some(anchor("Int: Displayable without evidence")),
        )
        .expect("missing interface evidence should be a conservative deferred outcome");

    match outcome {
        PropositionOutcome::Deferred(reason) => {
            assert_eq!(reason.proposition, proposition);
            assert_eq!(
                reason.kind,
                PropositionDeferredKind::MissingInterfaceEvidence
            );
            assert!(reason.no_inversion_boundary);
            assert!(reason.source_anchor.is_some());
        }
        other => panic!("expected missing evidence to defer, got {other:?}"),
    }
}

#[test]
fn task_877_requires_exact_subject_and_interface_arguments_for_existing_evidence() {
    let mut env = TypeEnv::with_builtin_types();
    env.set_current_module_identity(module_identity(877_102, &["pkg", "format_exact"]));
    register_interface(&mut env, "FormatsAs", &["Self", "Format"]);
    let interface_id = env
        .interface_identity_for_name("FormatsAs")
        .expect("interface identity registered")
        .clone();
    env.register_impl(&ImplDef {
        visibility: Visibility::Inherited,
        interface: "FormatsAs".into(),
        type_params: vec![],
        type_args: vec![surface_name("Int"), surface_name("String")],
        where_bounds: vec![],
        associated_type_bindings: vec![],
        methods: vec![],
        span: span(),
    })
    .expect("concrete impl evidence should register");

    let mismatched_arg = proposition_bound(
        primitive("Int"),
        interface_id.clone(),
        vec![primitive("Bool")],
    );
    let mismatched_subject =
        proposition_bound(primitive("String"), interface_id, vec![primitive("String")]);

    for proposition in [mismatched_arg, mismatched_subject] {
        let outcome = env
            .solve_proposition(&proposition, Some(anchor("non-exact interface evidence")))
            .expect("non-exact interface proposition should not be solved by nearby evidence");
        match outcome {
            PropositionOutcome::Deferred(reason) => {
                assert_eq!(reason.proposition, proposition);
                assert_eq!(
                    reason.kind,
                    PropositionDeferredKind::MissingInterfaceEvidence
                );
                assert!(reason.no_inversion_boundary);
            }
            other => panic!("expected non-exact interface evidence to defer, got {other:?}"),
        }
    }
}

fn iterator_interface_def() -> InterfaceDef {
    InterfaceDef {
        visibility: Visibility::Inherited,
        name: "Iterator".into(),
        type_params: vec![interface_param("Self")],
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
        type_params: vec![interface_param(param_name)],
        type_args: vec![surface_list(surface_name(param_name))],
        where_bounds: vec![],
        associated_type_bindings: vec![AssociatedTypeBinding {
            name: "Item".into(),
            ty: surface_name(param_name),
            span: span(),
        }],
        methods: vec![],
        span: span(),
    }
}

#[test]
fn task_877_does_not_solve_interface_bound_by_searching_generic_impls_or_family_equations() {
    let mut env = TypeEnv::with_builtin_types();
    env.set_current_module_identity(module_identity(877_103, &["pkg", "iterator"]));
    env.register_interface(&iterator_interface_def())
        .expect("Iterator associated family registers");
    env.register_impl(&iterator_list_impl("A"))
        .expect("generic Iterator<List<A>>::Item family scheme registers");
    let interface_id = env
        .interface_identity_for_name("Iterator")
        .expect("interface identity registered")
        .clone();
    let list_string = env
        .lower_surface_type_to_canonical(&surface_list(surface_name("String")))
        .expect("List<String> lowers to canonical term");
    let proposition = proposition_bound(
        TypePropositionTerm::Canonical(list_string),
        interface_id,
        vec![],
    );

    assert!(
        !env.proposition_assumptions().iter().any(|record| {
            record.owner_site.kind == PropositionCheckingSiteKind::ConcreteImpl
                && record.proposition == proposition
        }),
        "fixture must not accidentally create exact selected concrete evidence"
    );

    let outcome = env
        .solve_proposition(
            &proposition,
            Some(anchor(
                "List<String>: Iterator generic impl is not selected evidence",
            )),
        )
        .expect("interface solver must return conservative outcome without impl search");

    match outcome {
        PropositionOutcome::Deferred(reason) => {
            assert_eq!(reason.proposition, proposition);
            assert_eq!(
                reason.kind,
                PropositionDeferredKind::MissingInterfaceEvidence
            );
            assert!(reason.no_inversion_boundary);
        }
        other => panic!(
            "expected generic impl/family-equation search boundary to defer interface proposition, got {other:?}"
        ),
    }
}
