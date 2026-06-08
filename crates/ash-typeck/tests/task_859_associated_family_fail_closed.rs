use ash_core::ast::Visibility as CoreVisibility;
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    ModuleIdentity, ModuleSourceOrigin, SealedDomainId, SealedDomainSummary, SourceAnchor,
    SourceOrigin,
};
use ash_parser::surface::{
    AssociatedFamilyDecreases, AssociatedTypeDecl, AssociatedTypeKind, ImplDef, InterfaceDef,
    InterfaceTypeParam, Type as SurfaceType, Visibility,
};
use ash_parser::token::Span;
use ash_typeck::TypeEnv;
use ash_typeck::error::TypeEnvError;

fn span() -> Span {
    Span::default()
}

fn module(name: &str, id: usize) -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(859)),
        ModuleId(id),
        vec![name.to_string()],
        ModuleSourceOrigin::Synthetic {
            reason: format!("TASK-859 transition {name}"),
        },
    )
}

fn anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "TASK-859 transition".to_string(),
        },
        None,
        label,
    )
}

fn domain(module: &ModuleIdentity, name: &str) -> SealedDomainSummary {
    SealedDomainSummary::new(
        SealedDomainId::new(module.clone(), name.to_string()),
        name,
        CoreVisibility::Public,
        anchor(name),
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

#[test]
fn task_859_typeenv_still_rejects_unknown_domain_annotated_interface_params() {
    let mut env = TypeEnv::new();

    let err = env
        .register_interface(&InterfaceDef {
            visibility: Visibility::Inherited,
            name: "Append".into(),
            type_params: vec![param("Xs", Some("TypeList"))],
            evidence_constraints: vec![],
            associated_types: vec![],
            methods: vec![],
            laws: Vec::new(),
            span: span(),
        })
        .expect_err("domain annotations must name a registered sealed domain");

    assert!(
        matches!(&err, TypeEnvError::WrongAssociatedFamilyResultDomain { family, .. } if family == "<declaration>")
    );
    assert!(
        err.to_string()
            .contains("unknown sealed result domain 'TypeList'")
    );
}

#[test]
fn task_859_typeenv_accepts_registered_domain_annotated_interface_params_after_task_861() {
    let owner = module("owner", 1);
    let mut env = TypeEnv::new();
    env.set_current_module_identity(owner.clone());
    env.register_local_sealed_domain_summary(&domain(&owner, "TypeList"))
        .expect("sealed domain precondition");

    env.register_interface(&InterfaceDef {
        visibility: Visibility::Inherited,
        name: "Append".into(),
        type_params: vec![param("Xs", Some("TypeList"))],
        evidence_constraints: vec![],
        associated_types: vec![AssociatedTypeDecl {
            name: "Out".into(),
            kind: AssociatedTypeKind::SealedFamily {
                result_domain: SurfaceType::Name("TypeList".into()),
                decreases: Some(AssociatedFamilyDecreases {
                    param: "Xs".into(),
                    span: span(),
                }),
                span: span(),
            },
            span: span(),
        }],
        methods: vec![],
        laws: Vec::new(),
        span: span(),
    })
    .expect("TASK-861 registers sealed associated families instead of failing closed");

    let decl = env
        .lookup_associated_family_declaration("Append", "Out")
        .expect("sealed associated family declaration should be registered");
    assert_eq!(decl.defining_module, owner);
    assert_eq!(decl.decreases.as_deref(), Some("Xs"));
    assert_eq!(decl.interface_params[0].name, "Xs");
    assert!(decl.interface_params[0].domain_constraint.is_some());
}

#[test]
fn task_859_typeenv_accepts_registered_domain_annotated_impl_params_after_task_861() {
    let owner = module("owner", 2);
    let mut env = TypeEnv::new();
    env.set_current_module_identity(owner.clone());
    env.register_local_sealed_domain_summary(&domain(&owner, "TypeList"))
        .expect("sealed domain precondition");
    env.register_interface(&InterfaceDef {
        visibility: Visibility::Inherited,
        name: "Iterator".into(),
        type_params: vec!["T".into()],
        evidence_constraints: vec![],
        associated_types: vec![],
        methods: vec![],
        laws: Vec::new(),
        span: span(),
    })
    .expect("ordinary interface should register as test precondition");

    env.register_impl(&ImplDef {
        visibility: Visibility::Inherited,
        interface: "Iterator".into(),
        type_params: vec![param("Xs", Some("TypeList"))],
        type_args: vec![SurfaceType::Name("Xs".into())],
        where_bounds: vec![],
        associated_type_bindings: vec![],
        methods: vec![],
        proofs: Vec::new(),
        span: span(),
    })
    .expect("TASK-861 accepts domain-annotated impl params when their sealed domain is registered");
}
