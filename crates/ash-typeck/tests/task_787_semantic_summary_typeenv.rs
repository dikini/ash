use ash_core::ast::{TypeBody, TypeDef, TypeExpr, VariantDef, VariantPayload, Visibility};
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    ConstructorId, ConstructorPayloadKind, ConstructorSummary, ModuleIdentity,
    ModuleSemanticSummary, ModuleSourceOrigin, RepresentationExposure, SourceAnchor, SourceOrigin,
    SummaryVersion, TypeDeclId, TypeDeclSummary, TypeRepresentationSummary,
};
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

fn constructor_summary(
    parent: TypeDeclId,
    name: &str,
    payload_kind: ConstructorPayloadKind,
) -> ConstructorSummary {
    ConstructorSummary::new(
        ConstructorId::variant(parent.clone(), name, payload_kind),
        parent,
        name,
        payload_kind,
        Visibility::Public,
        anchor(name),
    )
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

    let node_id = node.id.clone();
    let summary = ModuleSemanticSummary::new(module)
        .with_exported_type(node)
        .with_exported_type(later)
        .with_exported_constructor(constructor_summary(
            node_id.clone(),
            "Empty",
            ConstructorPayloadKind::Unit,
        ))
        .with_exported_constructor(constructor_summary(
            node_id,
            "Link",
            ConstructorPayloadKind::Record,
        ));

    let mut env = TypeEnv::with_builtin_types();
    env.register_module_semantic_summary(&summary).unwrap();

    assert!(env.has_type("Node"));
    assert!(env.has_full_type("Node"));
    assert!(env.has_constructor("Empty"));
    assert!(env.has_constructor("Link"));
    assert!(env.has_full_type("Later"));
}

#[test]
fn std_result_summary_binds_existing_prelude_result_identity_without_duplicate_error() {
    let module = module_identity(101, &["std", "result"]);
    let result = type_summary(
        module.clone(),
        "Result",
        "Result",
        vec!["T", "E"],
        RepresentationExposure::Exposed,
        TypeRepresentationSummary::exposed(TypeBody::Enum(vec![
            VariantDef {
                name: "Ok".into(),
                fields: vec![("value".into(), TypeExpr::Named("T".into()))],
                payload: VariantPayload::Record(vec![(
                    "value".into(),
                    TypeExpr::Named("T".into()),
                )]),
            },
            VariantDef {
                name: "Err".into(),
                fields: vec![("error".into(), TypeExpr::Named("E".into()))],
                payload: VariantPayload::Record(vec![(
                    "error".into(),
                    TypeExpr::Named("E".into()),
                )]),
            },
        ])),
    );
    let result_id = result.id.clone();
    let summary = ModuleSemanticSummary::new(module)
        .with_exported_type(result)
        .with_exported_constructor(constructor_summary(
            result_id.clone(),
            "Ok",
            ConstructorPayloadKind::Record,
        ))
        .with_exported_constructor(constructor_summary(
            result_id.clone(),
            "Err",
            ConstructorPayloadKind::Record,
        ));

    let mut env = TypeEnv::with_builtin_types();
    env.register_module_semantic_summary(&summary)
        .expect("std::result summary should refine the prelude Result identity");

    assert_eq!(env.type_identity_for_name("Result"), Some(&result_id));
    assert!(env.has_constructor("Ok"));
    assert!(env.has_constructor("Err"));
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
    let err = env
        .register_module_semantic_summary(&summary)
        .expect_err("duplicate exported names with distinct identities must be rejected");
    let msg = err.to_string();
    assert!(
        msg.contains("duplicate ordinary type summary identity"),
        "{msg}"
    );
    assert!(msg.contains("Thing"), "{msg}");
    assert!(msg.contains("A") && msg.contains("B"), "{msg}");
    assert!(msg.contains("pkg::dups"), "{msg}");
    assert!(
        msg.contains("task-787-test"),
        "diagnostic should include source-anchor context: {msg}"
    );
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
    assert!(
        env.resolve_type("Box").is_err(),
        "summary registration must be transactional after second-pass failure"
    );
    assert!(
        env.resolve_type("Bad").is_err(),
        "malformed summary registration must not leave partial identities behind"
    );
}

#[test]
fn summary_constructor_metadata_controls_constructor_exposure() {
    let module = module_identity(115, &["pkg", "constructors"]);
    let status = type_summary(
        module.clone(),
        "Status",
        "Status",
        vec![],
        RepresentationExposure::Exposed,
        TypeRepresentationSummary::exposed(TypeBody::Enum(vec![
            VariantDef {
                name: "Hidden".into(),
                fields: vec![],
                payload: VariantPayload::Unit,
            },
            VariantDef {
                name: "Ready".into(),
                fields: vec![],
                payload: VariantPayload::Unit,
            },
        ])),
    );
    let status_id = status.id.clone();
    let summary = ModuleSemanticSummary::new(module)
        .with_exported_type(status)
        .with_exported_constructor(constructor_summary(
            status_id,
            "Ready",
            ConstructorPayloadKind::Unit,
        ));

    let mut env = TypeEnv::new();
    env.register_module_semantic_summary(&summary).unwrap();

    assert!(env.has_constructor("Ready"));
    assert!(
        !env.has_constructor("Hidden"),
        "TypeEnv must not reconstruct constructors that the summary did not export"
    );
}

#[test]
fn non_public_exposed_summary_is_rejected_without_leaking_representation() {
    let module = module_identity(116, &["pkg", "private_summary"]);
    let mut private = type_summary(
        module.clone(),
        "Secret",
        "Secret",
        vec![],
        RepresentationExposure::Exposed,
        TypeRepresentationSummary::exposed(TypeBody::Struct(vec![])),
    );
    private.visibility = Visibility::Private;
    let summary = ModuleSemanticSummary::new(module).with_exported_type(private);

    let mut env = TypeEnv::new();
    let err = env
        .register_module_semantic_summary(&summary)
        .expect_err("private exposed summaries must be rejected by TypeEnv");
    let msg = err.to_string();
    assert!(
        msg.contains("Secret"),
        "diagnostic should name Secret: {msg}"
    );
    assert!(
        env.resolve_type("Secret").is_err(),
        "rejected private summary must not leave a type identity behind"
    );
}

#[test]
fn non_public_opaque_summary_is_rejected_without_leaking_identity() {
    let module = module_identity(118, &["pkg", "private_opaque_summary"]);
    let mut private = type_summary(
        module.clone(),
        "Secret",
        "Secret",
        vec![],
        RepresentationExposure::Opaque,
        TypeRepresentationSummary::opaque(false),
    );
    private.visibility = Visibility::Private;
    let summary = ModuleSemanticSummary::new(module).with_exported_type(private);

    let mut env = TypeEnv::new();
    let err = env
        .register_module_semantic_summary(&summary)
        .expect_err("private opaque summaries must be rejected by public summary import");
    let msg = err.to_string();
    assert!(
        msg.contains("Secret"),
        "diagnostic should name Secret: {msg}"
    );
    assert!(
        env.resolve_type("Secret").is_err(),
        "rejected private opaque summary must not leave a type identity behind"
    );
}

#[test]
fn opaque_exposure_with_exposed_body_is_rejected_transactionally() {
    let module = module_identity(123, &["pkg", "mismatched_opaque_exposed"]);
    let secret = type_summary(
        module.clone(),
        "Secret",
        "Secret",
        vec![],
        RepresentationExposure::Opaque,
        TypeRepresentationSummary::exposed(TypeBody::Struct(vec![(
            "value".into(),
            TypeExpr::Named("Int".into()),
        )])),
    );
    let summary = ModuleSemanticSummary::new(module).with_exported_type(secret);

    let mut env = TypeEnv::with_builtin_types();
    let err = env
        .register_module_semantic_summary(&summary)
        .expect_err("opaque exposure paired with an exposed body must be rejected");
    let msg = err.to_string();
    assert!(
        msg.contains("Secret"),
        "diagnostic should name Secret: {msg}"
    );
    assert!(
        msg.contains("opaque") && msg.contains("exposed"),
        "diagnostic should explain the exposure/body mismatch: {msg}"
    );
    assert!(
        env.resolve_type("Secret").is_err(),
        "mismatched summary registration must not leave a partial identity behind"
    );
}

#[test]
fn malformed_fn_constructor_summary_body_returns_error_without_partial_identity() {
    let module = module_identity(124, &["pkg", "malformed_fn"]);
    let bad = type_summary(
        module.clone(),
        "BadFnCarrier",
        "BadFnCarrier",
        vec![],
        RepresentationExposure::Exposed,
        TypeRepresentationSummary::exposed(TypeBody::Struct(vec![(
            "callback".into(),
            TypeExpr::Constructor {
                name: "Fn".into(),
                args: vec![],
            },
        )])),
    );
    let summary = ModuleSemanticSummary::new(module).with_exported_type(bad);

    let mut env = TypeEnv::with_builtin_types();
    let err = env
        .register_module_semantic_summary(&summary)
        .expect_err("malformed Fn constructor in imported summary must return an error");
    let msg = err.to_string();
    assert!(msg.contains("BadFnCarrier"), "{msg}");
    assert!(msg.contains("Fn"), "{msg}");
    assert!(
        env.resolve_type("BadFnCarrier").is_err(),
        "malformed summary registration must not leave a partial identity behind"
    );
}

#[test]
fn duplicate_exported_constructor_names_are_rejected_transactionally() {
    let module = module_identity(119, &["pkg", "constructor_dups"]);
    let status = type_summary(
        module.clone(),
        "Status",
        "Status",
        vec![],
        RepresentationExposure::Exposed,
        TypeRepresentationSummary::exposed(TypeBody::Enum(vec![
            VariantDef {
                name: "First".into(),
                fields: vec![],
                payload: VariantPayload::Unit,
            },
            VariantDef {
                name: "Second".into(),
                fields: vec![],
                payload: VariantPayload::Unit,
            },
        ])),
    );
    let status_id = status.id.clone();
    let mut second = constructor_summary(status_id.clone(), "Second", ConstructorPayloadKind::Unit);
    second.exported_name = "First".into();
    let summary = ModuleSemanticSummary::new(module)
        .with_exported_type(status)
        .with_exported_constructor(constructor_summary(
            status_id,
            "First",
            ConstructorPayloadKind::Unit,
        ))
        .with_exported_constructor(second);

    let mut env = TypeEnv::new();
    let err = env
        .register_module_semantic_summary(&summary)
        .expect_err("duplicate constructor export names must be rejected");
    let msg = err.to_string();
    assert!(msg.contains("constructor"), "{msg}");
    assert!(
        !env.has_constructor("First"),
        "failed constructor summary registration must be transactional"
    );
}

#[test]
fn sequential_constructor_name_conflicts_are_rejected_without_overwriting_existing_binding() {
    let first_module = module_identity(120, &["pkg", "first_constructors"]);
    let status = type_summary(
        first_module.clone(),
        "Status",
        "Status",
        vec![],
        RepresentationExposure::Exposed,
        TypeRepresentationSummary::exposed(TypeBody::Enum(vec![VariantDef {
            name: "Ready".into(),
            fields: vec![],
            payload: VariantPayload::Unit,
        }])),
    );
    let status_id = status.id.clone();
    let first_summary = ModuleSemanticSummary::new(first_module)
        .with_exported_type(status)
        .with_exported_constructor(constructor_summary(
            status_id,
            "Ready",
            ConstructorPayloadKind::Unit,
        ));

    let second_module = module_identity(121, &["pkg", "second_constructors"]);
    let other = type_summary(
        second_module.clone(),
        "Other",
        "Other",
        vec![],
        RepresentationExposure::Exposed,
        TypeRepresentationSummary::exposed(TypeBody::Enum(vec![VariantDef {
            name: "Ready".into(),
            fields: vec![],
            payload: VariantPayload::Unit,
        }])),
    );
    let other_id = other.id.clone();
    let second_summary = ModuleSemanticSummary::new(second_module)
        .with_exported_type(other)
        .with_exported_constructor(constructor_summary(
            other_id,
            "Ready",
            ConstructorPayloadKind::Unit,
        ));

    let mut env = TypeEnv::new();
    env.register_module_semantic_summary(&first_summary)
        .expect("first constructor summary registers");
    let err = env
        .register_module_semantic_summary(&second_summary)
        .expect_err("second summary must not overwrite an existing constructor name");
    let msg = err.to_string();
    assert!(msg.contains("Ready"), "{msg}");
    assert_eq!(env.lookup_constructor("Ready").unwrap().0, "Status");
}

#[test]
fn same_type_constructor_name_remap_to_different_variant_is_rejected() {
    let module = module_identity(122, &["pkg", "same_type_constructor_remap"]);
    let status = type_summary(
        module.clone(),
        "Status",
        "Status",
        vec![],
        RepresentationExposure::Exposed,
        TypeRepresentationSummary::exposed(TypeBody::Enum(vec![
            VariantDef {
                name: "First".into(),
                fields: vec![],
                payload: VariantPayload::Unit,
            },
            VariantDef {
                name: "Second".into(),
                fields: vec![],
                payload: VariantPayload::Unit,
            },
        ])),
    );
    let status_id = status.id.clone();
    let mut first_ready =
        constructor_summary(status_id.clone(), "First", ConstructorPayloadKind::Unit);
    first_ready.exported_name = "Ready".into();
    let mut second_ready = constructor_summary(status_id, "Second", ConstructorPayloadKind::Unit);
    second_ready.exported_name = "Ready".into();
    let first_summary = ModuleSemanticSummary::new(module.clone())
        .with_exported_type(status.clone())
        .with_exported_constructor(first_ready);
    let second_summary = ModuleSemanticSummary::new(module)
        .with_exported_type(status)
        .with_exported_constructor(second_ready);

    let mut env = TypeEnv::new();
    env.register_module_semantic_summary(&first_summary)
        .expect("first constructor alias registers");
    let err = env
        .register_module_semantic_summary(&second_summary)
        .expect_err("same type constructor name must not remap to another variant");
    let msg = err.to_string();
    assert!(msg.contains("Ready"), "{msg}");
    assert_eq!(env.lookup_constructor("Ready").unwrap().1, 0);
}

#[test]
fn conflicting_duplicate_same_identity_summary_is_rejected_without_stale_constructors() {
    let module = module_identity(117, &["pkg", "conflict"]);
    let first = type_summary(
        module.clone(),
        "Status",
        "Status",
        vec![],
        RepresentationExposure::Exposed,
        TypeRepresentationSummary::exposed(TypeBody::Enum(vec![VariantDef {
            name: "First".into(),
            fields: vec![],
            payload: VariantPayload::Unit,
        }])),
    );
    let second = type_summary(
        module.clone(),
        "Status",
        "Status",
        vec![],
        RepresentationExposure::Exposed,
        TypeRepresentationSummary::exposed(TypeBody::Enum(vec![VariantDef {
            name: "Second".into(),
            fields: vec![],
            payload: VariantPayload::Unit,
        }])),
    );
    let first_id = first.id.clone();
    let second_id = second.id.clone();
    let summary = ModuleSemanticSummary::new(module)
        .with_exported_type(first)
        .with_exported_type(second)
        .with_exported_constructor(constructor_summary(
            first_id,
            "First",
            ConstructorPayloadKind::Unit,
        ))
        .with_exported_constructor(constructor_summary(
            second_id,
            "Second",
            ConstructorPayloadKind::Unit,
        ));

    let mut env = TypeEnv::new();
    let err = env
        .register_module_semantic_summary(&summary)
        .expect_err("same-identity conflicting duplicate summary must be rejected");
    let msg = err.to_string();
    assert!(msg.contains("conflicting"), "{msg}");
    assert!(
        !env.has_constructor("First") && !env.has_constructor("Second"),
        "failed registration must not leave stale constructors"
    );
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

#[test]
fn rejects_unsupported_module_semantic_summary_version_transactionally() {
    let module = module_identity(20, &["pkg", "future_summary"]);
    let token = type_summary(
        module.clone(),
        "Token",
        "Token",
        vec![],
        RepresentationExposure::Opaque,
        TypeRepresentationSummary::opaque(false),
    );
    let mut summary = ModuleSemanticSummary::new(module).with_exported_type(token);
    summary.version = SummaryVersion(999);

    let mut env = TypeEnv::new();
    let err = env
        .register_module_semantic_summary(&summary)
        .expect_err("unsupported summary versions must be rejected");
    assert!(
        err.to_string()
            .contains("unsupported module semantic summary version")
    );
    assert!(env.type_identity_for_name("Token").is_none());
}

#[test]
fn rejects_conflicting_same_identity_summaries_under_different_visible_names() {
    let module = module_identity(21, &["pkg", "conflicting_aliases"]);
    let first = type_summary(
        module.clone(),
        "Payload",
        "PayloadAliasA",
        vec![],
        RepresentationExposure::Exposed,
        TypeRepresentationSummary::exposed(TypeBody::Struct(vec![])),
    );
    let second = type_summary(
        module.clone(),
        "Payload",
        "PayloadAliasB",
        vec!["T"],
        RepresentationExposure::Exposed,
        TypeRepresentationSummary::exposed(TypeBody::Struct(vec![])),
    );
    let summary = ModuleSemanticSummary::new(module)
        .with_exported_type(first)
        .with_exported_type(second);

    let mut env = TypeEnv::new();
    let err = env
        .register_module_semantic_summary(&summary)
        .expect_err("same TypeDeclId aliases with conflicting contracts must be rejected");
    assert!(
        err.to_string()
            .contains("conflicting ordinary type summary metadata")
    );
    assert!(env.type_identity_for_name("PayloadAliasA").is_none());
    assert!(env.type_identity_for_name("PayloadAliasB").is_none());
}

#[test]
fn rejects_sequential_conflicting_same_identity_alias_registration() {
    let module = module_identity(22, &["pkg", "sequential_aliases"]);
    let first = type_summary(
        module.clone(),
        "Payload",
        "PayloadAliasA",
        vec![],
        RepresentationExposure::Exposed,
        TypeRepresentationSummary::exposed(TypeBody::Struct(vec![])),
    );
    let second = type_summary(
        module.clone(),
        "Payload",
        "PayloadAliasB",
        vec!["T"],
        RepresentationExposure::Exposed,
        TypeRepresentationSummary::exposed(TypeBody::Struct(vec![])),
    );

    let mut env = TypeEnv::new();
    env.register_module_semantic_summary(
        &ModuleSemanticSummary::new(module.clone()).with_exported_type(first),
    )
    .expect("first alias registers");
    let err = env
        .register_module_semantic_summary(
            &ModuleSemanticSummary::new(module).with_exported_type(second),
        )
        .expect_err("second conflicting alias should fail");
    assert!(
        err.to_string()
            .contains("conflicting ordinary type summary metadata")
    );
    assert!(env.type_identity_for_name("PayloadAliasA").is_some());
    assert!(env.type_identity_for_name("PayloadAliasB").is_none());
}

#[test]
fn compatible_exposed_summary_upgrades_existing_identity_only_summary() {
    let module = module_identity(23, &["pkg", "identity_upgrade"]);
    let identity_only = type_summary(
        module.clone(),
        "Token",
        "Token",
        vec![],
        RepresentationExposure::Opaque,
        TypeRepresentationSummary::opaque(false),
    );
    let exposed = type_summary(
        module.clone(),
        "Token",
        "Token",
        vec![],
        RepresentationExposure::Exposed,
        TypeRepresentationSummary::exposed(TypeBody::Struct(vec![(
            "value".into(),
            TypeExpr::Named("Int".into()),
        )])),
    );

    let mut env = TypeEnv::with_builtin_types();
    env.register_module_semantic_summary(
        &ModuleSemanticSummary::new(module.clone()).with_exported_type(identity_only),
    )
    .expect("identity-only summary registers");
    assert!(env.has_type("Token"));
    assert!(!env.has_full_type("Token"));

    env.register_module_semantic_summary(
        &ModuleSemanticSummary::new(module).with_exported_type(exposed),
    )
    .expect("compatible exposed summary upgrades identity-only binding");
    assert!(env.has_full_type("Token"));
}

#[test]
fn sequential_partial_constructor_summaries_accumulate_for_same_type() {
    let module = module_identity(24, &["pkg", "partial_constructors"]);
    let status = type_summary(
        module.clone(),
        "Status",
        "Status",
        vec![],
        RepresentationExposure::Exposed,
        TypeRepresentationSummary::exposed(TypeBody::Enum(vec![
            VariantDef {
                name: "Pending".into(),
                fields: vec![],
                payload: VariantPayload::Unit,
            },
            VariantDef {
                name: "Ready".into(),
                fields: vec![("value".into(), TypeExpr::Named("Int".into()))],
                payload: VariantPayload::Tuple(vec![TypeExpr::Named("Int".into())]),
            },
        ])),
    );
    let status_id = status.id.clone();
    let pending = constructor_summary(status_id.clone(), "Pending", ConstructorPayloadKind::Unit);
    let ready = constructor_summary(status_id, "Ready", ConstructorPayloadKind::Tuple);

    let mut env = TypeEnv::with_builtin_types();
    env.register_module_semantic_summary(
        &ModuleSemanticSummary::new(module.clone())
            .with_exported_type(status.clone())
            .with_exported_constructor(pending),
    )
    .expect("first partial constructor summary registers");
    assert!(env.has_constructor("Pending"));
    assert!(!env.has_constructor("Ready"));

    env.register_module_semantic_summary(
        &ModuleSemanticSummary::new(module)
            .with_exported_type(status)
            .with_exported_constructor(ready),
    )
    .expect("second partial constructor summary registers without erasing the first");
    assert!(env.has_constructor("Pending"));
    assert!(env.has_constructor("Ready"));
}
