use ash_parser::{Definition, Type, Visibility, new_input, parse_module_decl, parse_surface_file};
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
fn parses_private_capability_interface_operations() {
    let def = parse_inline_definition(
        "capability interface KVStore:
             observe get(key: String) returns Option<String>
           | execute put(key: String, value: String) returns Unit;",
    );

    let Definition::CapabilityInterface(interface) = def else {
        panic!("expected capability interface definition, got {def:?}");
    };

    assert_eq!(interface.visibility, Visibility::Inherited);
    assert_eq!(interface.name.as_ref(), "KVStore");
    assert_eq!(interface.operations.len(), 2);

    let get = &interface.operations[0];
    assert!(get.mode.is_observe());
    assert_eq!(get.name.as_ref(), "get");
    assert_eq!(get.params.len(), 1);
    assert_eq!(get.params[0].name.as_ref(), "key");
    assert!(matches!(&get.params[0].ty, Type::Name(name) if name.as_ref() == "String"));
    assert!(matches!(&get.return_type, Type::Constructor { name, args }
        if name.as_ref() == "Option"
            && matches!(args.as_slice(), [Type::Name(inner)] if inner.as_ref() == "String")));

    let put = &interface.operations[1];
    assert!(put.mode.is_execute());
    assert_eq!(put.name.as_ref(), "put");
    assert_eq!(put.params.len(), 2);
    assert_eq!(put.params[0].name.as_ref(), "key");
    assert_eq!(put.params[1].name.as_ref(), "value");
    assert!(matches!(&put.return_type, Type::Name(name) if name.as_ref() == "Unit"));
}

#[test]
fn parses_public_capability_interface_at_file_scope() {
    let module = parse_surface_file(
        "pub capability interface Fs:
             observe read(path: PathBuf) returns Bytes
           | execute write(path: PathBuf, content: Bytes) returns Unit;",
    )
    .expect("file should parse");

    let Definition::CapabilityInterface(interface) = &module.definitions[0] else {
        panic!(
            "expected capability interface, got {:?}",
            module.definitions[0]
        );
    };

    assert_eq!(interface.visibility, Visibility::Public);
    assert_eq!(interface.name.as_ref(), "Fs");
    assert_eq!(interface.operations.len(), 2);
    assert!(interface.operations[0].mode.is_observe());
    assert!(interface.operations[1].mode.is_execute());
}

#[test]
fn rejects_duplicate_operation_names() {
    let result = parse_surface_file(
        "capability interface Dup:
             observe read(path: PathBuf) returns Bytes
           | execute read(path: PathBuf) returns Unit;",
    );

    assert!(result.is_err(), "duplicate operations should be rejected");
}

#[test]
fn rejects_duplicate_parameter_names() {
    let result = parse_surface_file(
        "capability interface Bad:
             execute copy(path: PathBuf, path: PathBuf) returns Unit;",
    );

    assert!(result.is_err(), "duplicate parameters should be rejected");
}

#[test]
fn rejects_malformed_operation_signature_missing_return_type() {
    let result = parse_surface_file("capability interface Bad: observe get(key: String);");

    assert!(result.is_err(), "operation return types are required");
}

#[test]
fn legacy_pub_capability_surface_still_parses_as_capability() {
    let module = parse_surface_file(
        "pub capability Fs: observe read(path: PathBuf) returns Bytes
                         | execute write(path: PathBuf, content: Bytes);",
    )
    .expect("legacy capability should parse");

    assert_eq!(module.definitions.len(), 1);
    let Definition::Capability(cap) = &module.definitions[0] else {
        panic!("legacy pub capability must remain Definition::Capability");
    };

    assert_eq!(cap.visibility, Visibility::Public);
    assert_eq!(cap.name.as_ref(), "Fs");
}
