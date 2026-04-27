use ash_core::module_graph::{ModuleGraph, ModuleNode, ModuleSource};
use ash_parser::capability_export::{
    CapabilityEffect, CapabilityExport, ModuleDefinitionExport, ModuleDefinitionExportKind,
    ModuleDefinitionExports,
};
use ash_parser::import_resolver::{BindingItemKind, ImportError, ImportResolver};
use ash_parser::{Definition, Type, Visibility, parse_surface_file};

fn parse_definition(source: &str) -> Definition {
    parse_surface_file(source)
        .expect("source should parse")
        .definitions
        .into_iter()
        .next()
        .expect("expected one definition")
}

fn graph_with_child() -> (
    ModuleGraph,
    ash_core::module_graph::ModuleId,
    ash_core::module_graph::ModuleId,
) {
    let mut graph = ModuleGraph::new();
    let root = graph.add_node(ModuleNode::new(
        "crate".to_string(),
        ModuleSource::File("main.ash".to_string()),
    ));
    graph.set_root(root);
    let defs = graph.add_node(ModuleNode::new(
        "defs".to_string(),
        ModuleSource::File("defs.ash".to_string()),
    ));
    graph.add_edge(root, defs);
    (graph, root, defs)
}

fn import_use(item: &str) -> ash_parser::use_tree::Use {
    ash_parser::use_tree::Use {
        visibility: Visibility::Inherited,
        path: ash_parser::use_tree::UsePath::Simple(ash_parser::use_tree::SimplePath {
            segments: vec!["crate".into(), "defs".into(), item.into()],
        }),
        alias: None,
        span: ash_parser::Span::new(0, 0, 1, 1),
    }
}

#[test]
fn public_capability_interface_export_metadata_includes_operation_names() {
    let Definition::CapabilityInterface(interface) = parse_definition(
        "pub capability interface KVStore:
             observe get(key: String) returns Option<String>
           | execute put(key: String, value: String) returns Unit;",
    ) else {
        panic!("expected capability interface");
    };

    let export = ModuleDefinitionExport::from_capability_interface(
        &interface,
        ash_parser::capability_export::ModuleId(1),
    );

    assert_eq!(export.visible_name.as_ref(), "KVStore");
    assert_eq!(export.visibility, Visibility::Public);
    let ModuleDefinitionExportKind::CapabilityInterface(metadata) = &export.kind else {
        panic!("expected capability interface metadata");
    };
    let operation_names = metadata
        .operations
        .iter()
        .map(|operation| operation.name.as_ref())
        .collect::<Vec<_>>();
    assert_eq!(operation_names, vec!["get", "put"]);
}

#[test]
fn public_capability_implementation_export_metadata_includes_target_interface() {
    let Definition::CapabilityImplementation(implementation) = parse_definition(
        "pub capability impl MemoryKV for KVStore
             requires resource kv: WorkflowKV
         {
             observe get(key: String) returns Option<String> { key }
         }",
    ) else {
        panic!("expected capability implementation");
    };

    let export = ModuleDefinitionExport::from_capability_implementation(
        &implementation,
        ash_parser::capability_export::ModuleId(1),
    );

    assert_eq!(export.visible_name.as_ref(), "MemoryKV");
    let ModuleDefinitionExportKind::CapabilityImplementation(metadata) = &export.kind else {
        panic!("expected capability implementation metadata");
    };
    assert_eq!(metadata.interface.as_ref(), "KVStore");
    assert_eq!(metadata.dependencies.len(), 1);
    assert_eq!(metadata.operations.len(), 1);
}

#[test]
fn public_resource_type_export_metadata_includes_fields() {
    let Definition::ResourceType(resource) = parse_definition(
        "pub resource type WorkflowKV {
             path: PathBuf,
             namespace: String
         }",
    ) else {
        panic!("expected resource type");
    };

    let export = ModuleDefinitionExport::from_resource_type(
        &resource,
        ash_parser::capability_export::ModuleId(1),
    );

    assert_eq!(export.visible_name.as_ref(), "WorkflowKV");
    let ModuleDefinitionExportKind::ResourceType(metadata) = &export.kind else {
        panic!("expected resource type metadata");
    };
    let fields = metadata
        .fields
        .iter()
        .map(|field| (field.name.as_ref(), &field.ty))
        .collect::<Vec<_>>();
    assert_eq!(fields.len(), 2);
    assert!(matches!(fields[0], ("path", Type::Name(name)) if name.as_ref() == "PathBuf"));
    assert!(matches!(fields[1], ("namespace", Type::Name(name)) if name.as_ref() == "String"));
}

#[test]
fn private_phase_101_definition_exports_are_not_visible_from_another_module() {
    let (graph, root, defs) = graph_with_child();

    let interface =
        match parse_definition("capability interface Hidden: observe get() returns Unit;") {
            Definition::CapabilityInterface(interface) => interface,
            other => panic!("expected capability interface, got {other:?}"),
        };
    let resource = match parse_definition("resource type Secret { id: String }") {
        Definition::ResourceType(resource) => resource,
        other => panic!("expected resource type, got {other:?}"),
    };
    let implementation = match parse_definition(
        "capability impl HiddenMemoryKV for Hidden { observe get() returns Unit { 0 } }",
    ) {
        Definition::CapabilityImplementation(implementation) => implementation,
        other => panic!("expected capability implementation, got {other:?}"),
    };
    let mut exports = ModuleDefinitionExports::new();
    exports.add(ModuleDefinitionExport::from_capability_interface(
        &interface,
        ash_parser::capability_export::ModuleId(defs.0),
    ));
    exports.add(ModuleDefinitionExport::from_resource_type(
        &resource,
        ash_parser::capability_export::ModuleId(defs.0),
    ));
    exports.add(ModuleDefinitionExport::from_capability_implementation(
        &implementation,
        ash_parser::capability_export::ModuleId(defs.0),
    ));

    for (item, message) in [
        (
            "Hidden",
            "private interface must not be importable from another module",
        ),
        (
            "Secret",
            "private resource must not be importable from another module",
        ),
        (
            "HiddenMemoryKV",
            "private capability implementation must not be importable from another module",
        ),
    ] {
        let mut resolver = ImportResolver::new(&graph);
        resolver.add_definition_exports(defs, exports.all().to_vec());
        resolver.add_module_uses(root, vec![import_use(item)]);

        let error = resolver.resolve_all().expect_err(message);
        assert_eq!(
            error,
            ImportError::PrivateItem {
                item: item.to_string(),
                module: "defs".to_string(),
            }
        );
    }
}

#[test]
fn public_phase_101_definition_exports_are_importable_with_item_kind_metadata() {
    let (graph, root, defs) = graph_with_child();
    let mut resolver = ImportResolver::new(&graph);

    let interface =
        match parse_definition("pub capability interface KVStore: observe get() returns Unit;") {
            Definition::CapabilityInterface(interface) => interface,
            other => panic!("expected capability interface, got {other:?}"),
        };
    let resource = match parse_definition("pub resource type WorkflowKV { id: String }") {
        Definition::ResourceType(resource) => resource,
        other => panic!("expected resource type, got {other:?}"),
    };
    let implementation = match parse_definition(
        "pub capability impl MemoryKV for KVStore { observe get() returns Unit { 0 } }",
    ) {
        Definition::CapabilityImplementation(implementation) => implementation,
        other => panic!("expected capability implementation, got {other:?}"),
    };

    let exports = vec![
        ModuleDefinitionExport::from_capability_interface(
            &interface,
            ash_parser::capability_export::ModuleId(defs.0),
        ),
        ModuleDefinitionExport::from_resource_type(
            &resource,
            ash_parser::capability_export::ModuleId(defs.0),
        ),
        ModuleDefinitionExport::from_capability_implementation(
            &implementation,
            ash_parser::capability_export::ModuleId(defs.0),
        ),
    ];
    resolver.add_definition_exports(defs, exports);
    resolver.add_module_uses(
        root,
        vec![
            import_use("KVStore"),
            import_use("WorkflowKV"),
            import_use("MemoryKV"),
        ],
    );

    let bindings = resolver
        .resolve_all()
        .expect("public exports should resolve");
    let root_bindings = bindings.get(&root).expect("root should have bindings");
    assert_eq!(
        root_bindings["KVStore"].item_kind,
        BindingItemKind::CapabilityInterface
    );
    assert!(matches!(
        root_bindings["KVStore"]
            .definition_metadata
            .as_ref()
            .map(|metadata| &metadata.kind),
        Some(ModuleDefinitionExportKind::CapabilityInterface(metadata)) if metadata.operations.len() == 1
    ));
    assert_eq!(
        root_bindings["WorkflowKV"].item_kind,
        BindingItemKind::ResourceType
    );
    assert!(matches!(
        root_bindings["WorkflowKV"]
            .definition_metadata
            .as_ref()
            .map(|metadata| &metadata.kind),
        Some(ModuleDefinitionExportKind::ResourceType(metadata)) if metadata.fields.len() == 1
    ));
    assert_eq!(
        root_bindings["MemoryKV"].item_kind,
        BindingItemKind::CapabilityImplementation
    );
    assert!(matches!(
        root_bindings["MemoryKV"]
            .definition_metadata
            .as_ref()
            .map(|metadata| &metadata.kind),
        Some(ModuleDefinitionExportKind::CapabilityImplementation(metadata))
            if metadata.interface.as_ref() == "KVStore"
    ));
}

#[test]
fn legacy_capability_target_export_still_resolves_provider_action() {
    let (graph, root, defs) = graph_with_child();
    let mut resolver = ImportResolver::new(&graph);

    let legacy = CapabilityExport {
        visible_name: "fs_read".into(),
        declaring_module: ash_parser::capability_export::ModuleId(defs.0),
        target_provider: "io".into(),
        target_action: "fs_read".into(),
        visibility: Visibility::Public,
        effect: CapabilityEffect::Act,
    };

    resolver.add_capability_exports(
        defs,
        vec![(
            legacy.visible_name.to_string(),
            legacy.visibility.clone(),
            legacy.target_provider.to_string(),
            legacy.target_action.to_string(),
        )],
    );
    resolver.add_module_uses(root, vec![import_use("fs_read")]);

    let bindings = resolver
        .resolve_all()
        .expect("legacy export should resolve");
    let binding = &bindings[&root]["fs_read"];
    assert_eq!(binding.item_kind, BindingItemKind::LegacyCapability);
    assert_eq!(
        binding.capability_target,
        Some(("io".to_string(), "fs_read".to_string()))
    );
}
