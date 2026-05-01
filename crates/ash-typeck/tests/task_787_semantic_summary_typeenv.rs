use ash_core::ast::{TypeBody, TypeDef, TypeExpr, VariantDef, VariantPayload, Visibility};
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    ModuleIdentity, ModuleSemanticSummary, ModuleSourceOrigin, RepresentationExposure,
    SourceAnchor, SourceOrigin, TypeDeclId, TypeDeclSummary, TypeRepresentationSummary,
};
use ash_core::workflow_carrier::PublicWorkflowSummary;
use ash_typeck::{Kind, QualifiedName, Type, TypeEnv};

fn module_identity(id: u32, path: &[&str]) -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(1)),
        ModuleId(id as usize),
        path.iter().map(|part| (*part).to_string()).collect(),
        ModuleSourceOrigin::Synthetic {
            reason: format!("task-787-{id}"),
        },
    )
}

fn anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "task-787-test".into(),
        },
        None,
        label,
    )
}

fn type_summary(
    module: ModuleIdentity,
    origin_name: &str,
    exported_name: &str,
    params: Vec<&str>,
    exposure: RepresentationExposure,
    representation: TypeRepresentationSummary,
) -> TypeDeclSummary {
    TypeDeclSummary::new(
        TypeDeclId::ordinary(module, origin_name),
        exported_name,
        Visibility::Public,
        exposure,
        representation,
        anchor(exported_name),
    )
    .with_params(params.into_iter().map(str::to_string).collect())
}

#[test]
fn registers_public_summary_with_two_pass_forward_self_and_generic_refs() {
    let module = module_identity(10, &["pkg", "types"]);
    let node = type_summary(
        module.clone(),
        "Node",
        "Node",
        vec!["T"],
        RepresentationExposure::Exposed,
        TypeRepresentationSummary::exposed(TypeBody::Enum(vec![
            VariantDef {
                name: "Empty".into(),
                fields: vec![],
                payload: VariantPayload::Unit,
            },
            VariantDef {
                name: "Link".into(),
                fields: vec![
                    ("value".into(), TypeExpr::Named("T".into())),
                    (
                        "next".into(),
                        TypeExpr::Constructor {
                            name: "Node".into(),
                            args: vec![TypeExpr::Named("T".into())],
                        },
                    ),
                    ("sibling".into(), TypeExpr::Named("Later".into())),
                ],
                payload: VariantPayload::Record(vec![]),
            },
        ])),
    );
    let later = type_summary(
        module.clone(),
        "Later",
        "Later",
        vec![],
        RepresentationExposure::Exposed,
        TypeRepresentationSummary::exposed(TypeBody::Struct(vec![(
            "node".into(),
            TypeExpr::Constructor {
                name: "Node".into(),
                args: vec![TypeExpr::Named("Int".into())],
            },
        )])),
    );

    let summary = ModuleSemanticSummary::new(module)
        .with_exported_type(node)
        .with_exported_type(later);

    let mut env = TypeEnv::with_builtin_types();
    env.register_module_semantic_summary(&summary).unwrap();

    assert!(env.has_type("Node"));
    assert!(env.has_full_type("Node"));
    assert!(env.has_constructor("Empty"));
    assert!(env.has_constructor("Link"));
    assert!(env.has_full_type("Later"));
}

#[test]
fn opaque_summary_registers_identity_without_exposing_constructors_or_full_body() {
    let module = module_identity(11, &["pkg", "opaque"]);
    let secret = type_summary(
        module.clone(),
        "Secret",
        "Secret",
        vec![],
        RepresentationExposure::Opaque,
        TypeRepresentationSummary::opaque(false),
    );
    let summary = ModuleSemanticSummary::new(module).with_exported_type(secret);

    let mut env = TypeEnv::new();
    env.register_module_semantic_summary(&summary).unwrap();

    assert!(env.has_type("Secret"));
    assert!(!env.has_full_type("Secret"));
    assert!(
        env.resolve_type("Secret").unwrap().1.is_none(),
        "opaque identity-only summaries resolve by name but do not expose representation"
    );
    assert!(
        env.unfold_constructor(&ash_typeck::QualifiedName::root("Secret"), &[])
            .is_err(),
        "opaque identity-only summaries must not unfold as empty structs"
    );
    assert!(!env.has_constructor("Reveal"));
    assert!(env.type_identity_for_name("Secret").is_some());
}

#[test]
fn real_empty_struct_is_not_confused_with_placeholder() {
    let mut env = TypeEnv::new();
    env.declare_type_name("Empty");
    assert!(!env.has_full_type("Empty"));

    env.register_type_identity(&TypeDef {
        name: "Empty".into(),
        params: vec![],
        body: TypeBody::Struct(vec![]),
        visibility: Visibility::Public,
        builtin: false,
    })
    .unwrap();

    assert!(env.has_full_type("Empty"));
    assert!(
        env.register_type_identity(&TypeDef {
            name: "Empty".into(),
            params: vec![],
            body: TypeBody::Struct(vec![]),
            visibility: Visibility::Public,
            builtin: false,
        })
        .is_err()
    );
}

#[test]
fn duplicate_exported_names_with_different_identity_are_rejected() {
    let module = module_identity(12, &["pkg", "dups"]);
    let first = type_summary(
        module.clone(),
        "A",
        "Thing",
        vec![],
        RepresentationExposure::Opaque,
        TypeRepresentationSummary::opaque(false),
    );
    let second = type_summary(
        module.clone(),
        "B",
        "Thing",
        vec![],
        RepresentationExposure::Opaque,
        TypeRepresentationSummary::opaque(false),
    );
    let summary = ModuleSemanticSummary::new(module)
        .with_exported_type(first)
        .with_exported_type(second);

    let mut env = TypeEnv::new();
    assert!(env.register_module_semantic_summary(&summary).is_err());
}

#[test]
fn exported_alias_preserves_origin_identity_under_visible_name() {
    let origin_module = module_identity(13, &["pkg", "origin"]);
    let reexport_module = module_identity(14, &["pkg", "facade"]);
    let origin = TypeDeclId::ordinary(origin_module, "Origin");
    let alias = TypeDeclSummary::new(
        origin.clone(),
        "Alias",
        Visibility::Public,
        RepresentationExposure::Opaque,
        TypeRepresentationSummary::opaque(false),
        anchor("Alias"),
    );
    let summary = ModuleSemanticSummary::new(reexport_module).with_exported_type(alias);

    let mut env = TypeEnv::new();
    env.register_module_semantic_summary(&summary).unwrap();

    assert_eq!(env.type_identity_for_name("Alias"), Some(&origin));
    assert_eq!(
        env.canonical_type_name(&origin).map(String::as_str),
        Some("Alias")
    );
}

#[test]
fn malformed_generic_arity_is_rejected_when_summary_metadata_suffices() {
    let module = module_identity(15, &["pkg", "arity"]);
    let box_type = type_summary(
        module.clone(),
        "Box",
        "Box",
        vec!["T"],
        RepresentationExposure::Opaque,
        TypeRepresentationSummary::opaque(false),
    );
    let bad = type_summary(
        module.clone(),
        "Bad",
        "Bad",
        vec![],
        RepresentationExposure::Exposed,
        TypeRepresentationSummary::exposed(TypeBody::Struct(vec![(
            "field".into(),
            TypeExpr::Constructor {
                name: "Box".into(),
                args: vec![],
            },
        )])),
    );
    let summary = ModuleSemanticSummary::new(module)
        .with_exported_type(box_type)
        .with_exported_type(bad);

    let mut env = TypeEnv::new();
    assert!(env.register_module_semantic_summary(&summary).is_err());
}

#[test]
fn ordinary_type_identities_exist_before_workflow_summaries_are_bound() {
    let module = module_identity(16, &["pkg", "workflow"]);
    let payload = type_summary(
        module.clone(),
        "Payload",
        "Payload",
        vec![],
        RepresentationExposure::Opaque,
        TypeRepresentationSummary::opaque(false),
    );
    let summary = ModuleSemanticSummary::new(module).with_exported_type(payload);

    let mut env = TypeEnv::new();
    env.register_module_semantic_summary(&summary).unwrap();
    assert!(env.type_identity_for_name("Payload").is_some());

    env.bind_public_workflow_summary("run", PublicWorkflowSummary::default());
    assert!(env.lookup_public_workflow_summary("run").is_some());
    assert!(env.type_identity_for_name("Payload").is_some());
}

#[test]
fn aliases_for_same_canonical_type_decl_id_unify_without_losing_visible_names() {
    let origin_module = module_identity(17, &["pkg", "origin_aliases"]);
    let reexport_module = module_identity(18, &["pkg", "facade_aliases"]);
    let origin = TypeDeclId::ordinary(origin_module, "Payload");
    let first_alias = TypeDeclSummary::new(
        origin.clone(),
        "PayloadAliasA",
        Visibility::Public,
        RepresentationExposure::Opaque,
        TypeRepresentationSummary::opaque(false),
        anchor("PayloadAliasA"),
    );
    let second_alias = TypeDeclSummary::new(
        origin.clone(),
        "PayloadAliasB",
        Visibility::Public,
        RepresentationExposure::Opaque,
        TypeRepresentationSummary::opaque(false),
        anchor("PayloadAliasB"),
    );
    let summary = ModuleSemanticSummary::new(reexport_module)
        .with_exported_type(first_alias)
        .with_exported_type(second_alias);

    let mut env = TypeEnv::new();
    env.register_module_semantic_summary(&summary).unwrap();

    assert_eq!(
        env.resolve_type("PayloadAliasA").unwrap().0,
        QualifiedName::root("PayloadAliasA")
    );
    assert_eq!(
        env.resolve_type("PayloadAliasB").unwrap().0,
        QualifiedName::root("PayloadAliasB")
    );
    assert_eq!(
        env.canonical_type_name(&origin).map(String::as_str),
        Some("PayloadAliasA")
    );

    let via_a = Type::Constructor {
        name: QualifiedName::root("PayloadAliasA"),
        args: vec![],
        kind: Kind::Type,
    };
    let via_b = Type::Constructor {
        name: QualifiedName::root("PayloadAliasB"),
        args: vec![],
        kind: Kind::Type,
    };

    env.unify_types(&via_a, &via_b)
        .expect("aliases sharing one TypeDeclId must cohere for TypeEnv unification");
}

#[test]
fn distinct_type_decl_ids_remain_nominally_distinct_even_with_same_shape() {
    let module = module_identity(19, &["pkg", "distinct_ids"]);
    let first = type_summary(
        module.clone(),
        "First",
        "First",
        vec![],
        RepresentationExposure::Opaque,
        TypeRepresentationSummary::opaque(false),
    );
    let second = type_summary(
        module.clone(),
        "Second",
        "Second",
        vec![],
        RepresentationExposure::Opaque,
        TypeRepresentationSummary::opaque(false),
    );
    let summary = ModuleSemanticSummary::new(module)
        .with_exported_type(first)
        .with_exported_type(second);

    let mut env = TypeEnv::new();
    env.register_module_semantic_summary(&summary).unwrap();

    let first = Type::Constructor {
        name: QualifiedName::root("First"),
        args: vec![],
        kind: Kind::Type,
    };
    let second = Type::Constructor {
        name: QualifiedName::root("Second"),
        args: vec![],
        kind: Kind::Type,
    };

    assert!(
        env.unify_types(&first, &second).is_err(),
        "different TypeDeclIds must remain distinct despite matching opaque structure"
    );
}
