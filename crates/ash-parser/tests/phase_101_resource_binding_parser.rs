use ash_parser::input::new_input;
use ash_parser::parse_module::module_file;
use ash_parser::surface::{Definition, Type, Visibility};
use winnow::Parser;

#[test]
fn parses_private_resource_type_definition_with_named_fields() {
    let mut input = new_input("resource type WorkflowKV { map: Map<String, String> }");

    let parsed = module_file
        .parse_next(&mut input)
        .expect("resource type should parse as a top-level module definition");

    assert_eq!(parsed.definitions.len(), 1);
    let Definition::ResourceType(resource) = &parsed.definitions[0] else {
        panic!(
            "expected resource type definition, got {:?}",
            parsed.definitions[0]
        );
    };

    assert_eq!(resource.visibility, Visibility::Inherited);
    assert_eq!(resource.name.as_ref(), "WorkflowKV");
    assert_eq!(resource.fields.len(), 1);
    assert_eq!(resource.fields[0].name.as_ref(), "map");
    assert!(matches!(
        &resource.fields[0].ty,
        Type::Constructor { name, args }
            if name.as_ref() == "Map" && args.len() == 2
    ));
}

#[test]
fn parses_public_resource_type_definition() {
    let mut input = new_input("pub resource type WorkflowKV { map: Map<String, String> }");

    let parsed = module_file
        .parse_next(&mut input)
        .expect("public resource type should parse as a top-level module definition");

    assert_eq!(parsed.definitions.len(), 1);
    let Definition::ResourceType(resource) = &parsed.definitions[0] else {
        panic!(
            "expected resource type definition, got {:?}",
            parsed.definitions[0]
        );
    };

    assert_eq!(resource.visibility, Visibility::Public);
    assert_eq!(resource.name.as_ref(), "WorkflowKV");
}

#[test]
fn rejects_malformed_resource_type_header_missing_name() {
    let mut input = new_input("resource type { map: Map<String, String> }");

    let result = module_file.parse_next(&mut input);

    assert!(
        result.is_err(),
        "resource type declarations require a name before the field block"
    );
}

#[test]
fn rejects_malformed_resource_type_field_missing_colon() {
    let mut input = new_input("resource type WorkflowKV { map Map<String, String> }");

    let result = module_file.parse_next(&mut input);

    assert!(result.is_err(), "resource fields require `name: Type`");
}
