use ash_parser::{
    CapabilityImplementationDependencyKind, Definition, Expr, Type, Visibility, new_input,
    parse_module_decl, parse_surface_file,
};
use winnow::prelude::*;

fn parse_inline_definition(source: &str) -> Definition {
    let wrapped = format!("mod test {{ {source} }}");
    let mut input = new_input(&wrapped);
    let decl = parse_module_decl
        .parse_next(&mut input)
        .expect("inline module should parse");
    decl.definitions()
        .expect("expected inline module")
        .first()
        .expect("expected one definition")
        .clone()
}

#[test]
fn parses_public_capability_implementation_with_dependencies_and_bodies() {
    let def = parse_inline_definition(
        "pub capability impl MemoryKV for KVStore
             requires resource kv: WorkflowKV
             requires capability inner: KVStore
             requires config retries: Int
         {
             observe get(key: String) returns Option<String> { key }
             execute put(key: String, value: String) returns Unit { value }
         }",
    );

    let Definition::CapabilityImplementation(implementation) = def else {
        panic!("expected capability implementation definition, got {def:?}");
    };

    assert_eq!(implementation.visibility, Visibility::Public);
    assert_eq!(implementation.name.as_ref(), "MemoryKV");
    assert_eq!(implementation.interface.as_ref(), "KVStore");
    assert_eq!(implementation.dependencies.len(), 3);

    let resource = &implementation.dependencies[0];
    assert!(matches!(
        resource.kind,
        CapabilityImplementationDependencyKind::Resource
    ));
    assert_eq!(resource.name.as_ref(), "kv");
    assert!(matches!(&resource.ty, Type::Name(name) if name.as_ref() == "WorkflowKV"));

    let capability = &implementation.dependencies[1];
    assert!(matches!(
        capability.kind,
        CapabilityImplementationDependencyKind::Capability
    ));
    assert_eq!(capability.name.as_ref(), "inner");
    assert!(matches!(&capability.ty, Type::Name(name) if name.as_ref() == "KVStore"));

    let config = &implementation.dependencies[2];
    assert!(matches!(
        config.kind,
        CapabilityImplementationDependencyKind::Config
    ));
    assert_eq!(config.name.as_ref(), "retries");
    assert!(matches!(&config.ty, Type::Name(name) if name.as_ref() == "Int"));

    assert_eq!(implementation.operations.len(), 2);
    let get = &implementation.operations[0];
    assert!(get.mode.is_observe());
    assert_eq!(get.name.as_ref(), "get");
    assert_eq!(get.params.len(), 1);
    assert_eq!(get.params[0].name.as_ref(), "key");
    assert!(matches!(&get.return_type, Type::Constructor { name, args }
        if name.as_ref() == "Option"
            && matches!(args.as_slice(), [Type::Name(inner)] if inner.as_ref() == "String")));
    assert!(
        matches!(&get.body, Expr::Block { tail_expr: Some(tail), .. }
        if matches!(tail.as_ref(), Expr::Variable { name, .. } if name.as_ref() == "key"))
    );

    let put = &implementation.operations[1];
    assert!(put.mode.is_execute());
    assert_eq!(put.name.as_ref(), "put");
    assert_eq!(put.params.len(), 2);
    assert_eq!(put.params[0].name.as_ref(), "key");
    assert_eq!(put.params[1].name.as_ref(), "value");
    assert!(matches!(&put.return_type, Type::Name(name) if name.as_ref() == "Unit"));
    assert!(
        matches!(&put.body, Expr::Block { tail_expr: Some(tail), .. }
        if matches!(tail.as_ref(), Expr::Variable { name, .. } if name.as_ref() == "value"))
    );
}

#[test]
fn parses_private_capability_implementation_without_dependencies_at_file_scope() {
    let module = parse_surface_file(
        "capability impl NoopKV for KVStore {
             observe get(key: String) returns Option<String> { key }
         }",
    )
    .expect("file should parse");

    let Definition::CapabilityImplementation(implementation) = &module.definitions[0] else {
        panic!(
            "expected capability implementation, got {:?}",
            module.definitions[0]
        );
    };

    assert_eq!(implementation.visibility, Visibility::Inherited);
    assert_eq!(implementation.name.as_ref(), "NoopKV");
    assert_eq!(implementation.interface.as_ref(), "KVStore");
    assert!(implementation.dependencies.is_empty());
    assert_eq!(implementation.operations.len(), 1);
}

#[test]
fn rejects_capability_implementation_header_missing_for() {
    let result = parse_surface_file(
        "capability impl Broken KVStore {
             observe get(key: String) returns Option<String> { key }
         }",
    );

    assert!(result.is_err(), "capability impl headers require `for`");
}

#[test]
fn rejects_duplicate_capability_implementation_operation_names() {
    let result = parse_surface_file(
        "capability impl DupKV for KVStore {
             observe get(key: String) returns Option<String> { key }
             execute get(key: String) returns Unit { key }
         }",
    );

    assert!(
        result.is_err(),
        "duplicate operation bodies should be rejected"
    );
}

#[test]
fn legacy_pub_capability_surface_still_parses_as_capability() {
    let module = parse_surface_file(
        "pub capability Fs: observe read(path: PathBuf) returns Bytes
                         | execute write(path: PathBuf, content: Bytes);",
    )
    .expect("legacy capability should parse");

    assert_eq!(module.definitions.len(), 1);
    assert!(matches!(module.definitions[0], Definition::Capability(_)));
}
