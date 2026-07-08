use ash_core::module_graph::{ModuleGraph, ModuleNode, ModuleSource};
use ash_parser::capability_export::{ModuleDefinitionExport, ModuleDefinitionExportKind};
use ash_parser::import_resolver::{BindingItemKind, ImportResolver};
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
    let ModuleDefinitionExportKind::ResourceType(metadata) = &export.kind;
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
fn public_resource_definition_exports_are_importable_with_item_kind_metadata() {
    let (graph, root, defs) = graph_with_child();
    let mut resolver = ImportResolver::new(&graph);

    let resource = match parse_definition("pub resource type WorkflowKV { id: String }") {
        Definition::ResourceType(resource) => resource,
        other => panic!("expected resource type, got {other:?}"),
    };

    let exports = vec![ModuleDefinitionExport::from_resource_type(
        &resource,
        ash_parser::capability_export::ModuleId(defs.0),
    )];
    resolver.add_definition_exports(defs, exports);
    resolver.add_module_uses(root, vec![import_use("WorkflowKV")]);

    let bindings = resolver
        .resolve_all()
        .expect("public exports should resolve");
    let root_bindings = bindings.get(&root).expect("root should have bindings");
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
}
