use ash_parser::surface::{
    CapabilityImplementationDef, CapabilityImplementationDependency,
    CapabilityImplementationDependencyKind, CapabilityImplementationOperation,
    CapabilityInterfaceDef, CapabilityOperationMode, CapabilityOperationSig, Expr, Literal,
    ResourceField, ResourceTypeDef, Type as SurfaceType, Visibility, Workflow, WorkflowDef,
    WorkflowOwnedResource, WorkflowUsedBinding,
};
use ash_parser::token::Span;
use ash_typeck::type_env::TypeEnv;
use ash_typeck::{AuthorityProvenanceKind, ProvenanceSourceKind};
use proptest::prelude::*;
use proptest::test_runner::Config;

fn span() -> Span {
    Span::default()
}

fn ty(name: &str) -> SurfaceType {
    SurfaceType::Name(name.into())
}

fn var(name: &str) -> Expr {
    Expr::Variable {
        name: name.into(),
        span: span(),
    }
}

fn lit_str(value: &str) -> Expr {
    Expr::Literal(Literal::String(value.into()))
}

fn call(name: &str, args: Vec<Expr>) -> Expr {
    Expr::Call {
        module: None,
        func: name.into(),
        args,
        span: span(),
    }
}

fn cap_iface(name: &str) -> CapabilityInterfaceDef {
    CapabilityInterfaceDef {
        visibility: Visibility::Public,
        name: name.into(),
        operations: vec![CapabilityOperationSig {
            mode: CapabilityOperationMode::Observe,
            name: "read".into(),
            params: vec![],
            return_type: ty("String"),
            span: span(),
        }],
        span: span(),
    }
}

fn impl_op() -> CapabilityImplementationOperation {
    CapabilityImplementationOperation {
        mode: CapabilityOperationMode::Observe,
        name: "read".into(),
        params: vec![],
        return_type: ty("String"),
        body: lit_str("ok"),
        span: span(),
    }
}

fn dep(
    kind: CapabilityImplementationDependencyKind,
    name: &str,
    ty: SurfaceType,
) -> CapabilityImplementationDependency {
    CapabilityImplementationDependency {
        kind,
        name: name.into(),
        ty,
        span: span(),
    }
}

fn cap_impl(
    name: &str,
    interface: &str,
    dependencies: Vec<CapabilityImplementationDependency>,
) -> CapabilityImplementationDef {
    CapabilityImplementationDef {
        visibility: Visibility::Public,
        name: name.into(),
        interface: interface.into(),
        dependencies,
        operations: vec![impl_op()],
        span: span(),
    }
}

fn resource_type(name: &str, fields: Vec<(&str, SurfaceType)>) -> ResourceTypeDef {
    ResourceTypeDef {
        visibility: Visibility::Public,
        name: name.into(),
        fields: fields
            .into_iter()
            .map(|(name, ty)| ResourceField {
                name: name.into(),
                ty,
                span: span(),
            })
            .collect(),
        span: span(),
    }
}

fn own(name: &str, ty: &str) -> WorkflowOwnedResource {
    WorkflowOwnedResource {
        name: name.into(),
        ty: SurfaceType::Name(ty.into()),
        span: span(),
    }
}

fn use_binding(name: &str, interface: &str, implementation: Expr) -> WorkflowUsedBinding {
    WorkflowUsedBinding {
        name: name.into(),
        interface: SurfaceType::Name(interface.into()),
        implementation,
        span: span(),
    }
}

fn workflow() -> WorkflowDef {
    WorkflowDef {
        name: "main".into(),
        type_params: vec![],
        params: vec![],
        declared_return_type: None,
        plays_roles: vec![],
        capabilities: vec![],
        owned_resources: vec![],
        used_bindings: vec![],
        body: Workflow::Done { span: span() },
        contract: None,
        span: span(),
    }
}

fn env_with_storage_and_file() -> TypeEnv {
    let mut env = TypeEnv::with_builtin_types();
    env.register_resource_type(&resource_type("File", vec![("path", ty("String"))]))
        .unwrap();
    env.register_capability_interface(&cap_iface("Storage"))
        .unwrap();
    env
}

#[test]
fn resource_only_implementation_is_internal_authority() {
    let mut env = env_with_storage_and_file();
    env.register_capability_implementation(&cap_impl(
        "FileStorage",
        "Storage",
        vec![dep(
            CapabilityImplementationDependencyKind::Resource,
            "file",
            ty("File"),
        )],
    ))
    .unwrap();

    let implementation = env.lookup_capability_implementation("FileStorage").unwrap();
    assert_eq!(
        implementation.authority_provenance,
        AuthorityProvenanceKind::Internal
    );
    assert_eq!(implementation.authority_sources.len(), 1);
    assert_eq!(
        implementation.authority_sources[0].kind,
        ProvenanceSourceKind::Resource
    );
    assert_eq!(implementation.authority_sources[0].dependency_name, "file");
    assert_eq!(implementation.authority_sources[0].target_name, "File");
}

#[test]
fn capability_dependency_implementation_is_derived_authority() {
    let mut env = env_with_storage_and_file();
    env.register_capability_interface(&cap_iface("Cache"))
        .unwrap();
    env.register_capability_implementation(&cap_impl(
        "CachedStorage",
        "Storage",
        vec![dep(
            CapabilityImplementationDependencyKind::Capability,
            "inner",
            ty("Cache"),
        )],
    ))
    .unwrap();

    let implementation = env
        .lookup_capability_implementation("CachedStorage")
        .unwrap();
    assert_eq!(
        implementation.authority_provenance,
        AuthorityProvenanceKind::Derived
    );
    assert_eq!(implementation.authority_sources.len(), 1);
    assert_eq!(
        implementation.authority_sources[0].kind,
        ProvenanceSourceKind::Capability
    );
    assert_eq!(implementation.authority_sources[0].dependency_name, "inner");
    assert_eq!(implementation.authority_sources[0].target_name, "Cache");
}

#[test]
fn resource_plus_capability_implementation_is_derived_with_all_sources() {
    let mut env = env_with_storage_and_file();
    env.register_capability_interface(&cap_iface("Cache"))
        .unwrap();
    env.register_capability_implementation(&cap_impl(
        "CachingStorage",
        "Storage",
        vec![
            dep(
                CapabilityImplementationDependencyKind::Capability,
                "inner",
                ty("Cache"),
            ),
            dep(
                CapabilityImplementationDependencyKind::Resource,
                "file",
                ty("File"),
            ),
        ],
    ))
    .unwrap();

    let implementation = env
        .lookup_capability_implementation("CachingStorage")
        .unwrap();
    assert_eq!(
        implementation.authority_provenance,
        AuthorityProvenanceKind::Derived
    );
    assert_eq!(implementation.authority_sources.len(), 2);
    assert_eq!(
        implementation.authority_sources[0].kind,
        ProvenanceSourceKind::Capability
    );
    assert_eq!(
        implementation.authority_sources[1].kind,
        ProvenanceSourceKind::Resource
    );
}

#[test]
fn config_only_and_zero_dependency_implementations_do_not_manufacture_host_authority() {
    let mut env = env_with_storage_and_file();
    env.register_capability_implementation(&cap_impl("ConstantStorage", "Storage", vec![]))
        .unwrap();
    env.register_capability_implementation(&cap_impl(
        "ConfiguredStorage",
        "Storage",
        vec![dep(
            CapabilityImplementationDependencyKind::Config,
            "prefix",
            ty("String"),
        )],
    ))
    .unwrap();

    assert_eq!(
        env.lookup_capability_implementation("ConstantStorage")
            .unwrap()
            .authority_provenance,
        AuthorityProvenanceKind::NoAuthority
    );
    assert_eq!(
        env.lookup_capability_implementation("ConfiguredStorage")
            .unwrap()
            .authority_provenance,
        AuthorityProvenanceKind::NoAuthority
    );
    assert_ne!(
        env.lookup_capability_implementation("ConstantStorage")
            .unwrap()
            .authority_provenance,
        AuthorityProvenanceKind::Host
    );
}

#[test]
fn workflow_records_internal_binding_provenance_from_owned_resource() {
    let mut env = env_with_storage_and_file();
    env.register_capability_implementation(&cap_impl(
        "FileStorage",
        "Storage",
        vec![dep(
            CapabilityImplementationDependencyKind::Resource,
            "file",
            ty("File"),
        )],
    ))
    .unwrap();

    let mut wf = workflow();
    wf.owned_resources = vec![own("input", "File")];
    wf.used_bindings = vec![use_binding(
        "store",
        "Storage",
        call("FileStorage", vec![var("input")]),
    )];

    let result = ash_typeck::type_check_workflow_def_in_env(&env, &wf).unwrap();
    assert_eq!(result.authority_provenance.resource_bindings.len(), 1);
    assert_eq!(
        result.authority_provenance.resource_bindings[0].authority,
        AuthorityProvenanceKind::Internal
    );
    assert_eq!(
        result.authority_provenance.resource_bindings[0].name,
        "input"
    );
    assert_eq!(
        result.authority_provenance.resource_bindings[0].resource_type,
        "File"
    );

    assert_eq!(result.authority_provenance.capability_bindings.len(), 1);
    let binding = &result.authority_provenance.capability_bindings[0];
    assert_eq!(binding.name, "store");
    assert_eq!(binding.interface, "Storage");
    assert_eq!(binding.implementation, "FileStorage");
    assert_eq!(binding.authority, AuthorityProvenanceKind::Internal);
    assert_eq!(binding.sources.len(), 1);
    assert_eq!(binding.sources[0].kind, ProvenanceSourceKind::Resource);
    assert_eq!(binding.sources[0].dependency_name, "file");
    assert_eq!(binding.sources[0].binding_name, "input");
    assert_eq!(binding.sources[0].target_name, "File");
}

#[test]
fn workflow_records_derived_binding_provenance_links_to_inner_capability() {
    let mut env = env_with_storage_and_file();
    env.register_capability_implementation(&cap_impl(
        "FileStorage",
        "Storage",
        vec![dep(
            CapabilityImplementationDependencyKind::Resource,
            "file",
            ty("File"),
        )],
    ))
    .unwrap();
    env.register_capability_implementation(&cap_impl(
        "LoggingStorage",
        "Storage",
        vec![dep(
            CapabilityImplementationDependencyKind::Capability,
            "inner",
            ty("Storage"),
        )],
    ))
    .unwrap();

    let mut wf = workflow();
    wf.owned_resources = vec![own("input", "File")];
    wf.used_bindings = vec![
        use_binding("base", "Storage", call("FileStorage", vec![var("input")])),
        use_binding(
            "logged",
            "Storage",
            call("LoggingStorage", vec![var("base")]),
        ),
    ];

    let result = ash_typeck::type_check_workflow_def_in_env(&env, &wf).unwrap();
    let logged = result
        .authority_provenance
        .capability_bindings
        .iter()
        .find(|binding| binding.name == "logged")
        .expect("logged binding provenance should be recorded");
    assert_eq!(logged.authority, AuthorityProvenanceKind::Derived);
    assert_eq!(logged.sources.len(), 1);
    assert_eq!(logged.sources[0].kind, ProvenanceSourceKind::Capability);
    assert_eq!(logged.sources[0].dependency_name, "inner");
    assert_eq!(logged.sources[0].binding_name, "base");
    assert_eq!(logged.sources[0].target_name, "Storage");
}

#[test]
fn workflow_records_no_authority_and_config_source_metadata() {
    let mut env = env_with_storage_and_file();
    env.register_capability_implementation(&cap_impl(
        "ConfiguredStorage",
        "Storage",
        vec![dep(
            CapabilityImplementationDependencyKind::Config,
            "prefix",
            ty("String"),
        )],
    ))
    .unwrap();

    let mut wf = workflow();
    wf.used_bindings = vec![use_binding(
        "store",
        "Storage",
        call("ConfiguredStorage", vec![lit_str("cache")]),
    )];

    let result = ash_typeck::type_check_workflow_def_in_env(&env, &wf).unwrap();
    let binding = &result.authority_provenance.capability_bindings[0];
    assert_eq!(binding.name, "store");
    assert_eq!(binding.authority, AuthorityProvenanceKind::NoAuthority);
    assert_eq!(binding.sources.len(), 1);
    assert_eq!(binding.sources[0].kind, ProvenanceSourceKind::Config);
    assert_eq!(binding.sources[0].dependency_name, "prefix");
    assert_eq!(binding.sources[0].binding_name, "<config-expression>");
    assert_eq!(binding.sources[0].target_name, "String");
}

proptest! {
    #![proptest_config(Config { failure_persistence: None, ..Config::default() })]

    #[test]
    fn ash_defined_implementation_classification_never_manufactures_host(
        has_resource in any::<bool>(),
        has_capability in any::<bool>(),
        has_config in any::<bool>(),
    ) {
        let mut env = env_with_storage_and_file();
        env.register_capability_interface(&cap_iface("Inner"))
            .unwrap();
        let mut deps = Vec::new();
        if has_resource {
            deps.push(dep(CapabilityImplementationDependencyKind::Resource, "file", ty("File")));
        }
        if has_capability {
            deps.push(dep(CapabilityImplementationDependencyKind::Capability, "inner", ty("Inner")));
        }
        if has_config {
            deps.push(dep(CapabilityImplementationDependencyKind::Config, "prefix", ty("String")));
        }

        env.register_capability_implementation(&cap_impl("Impl", "Storage", deps))
            .unwrap();
        let implementation = env.lookup_capability_implementation("Impl").unwrap();
        let expected = if has_capability {
            AuthorityProvenanceKind::Derived
        } else if has_resource {
            AuthorityProvenanceKind::Internal
        } else {
            AuthorityProvenanceKind::NoAuthority
        };
        prop_assert_eq!(implementation.authority_provenance, expected);
        prop_assert_ne!(implementation.authority_provenance, AuthorityProvenanceKind::Host);
    }
}
