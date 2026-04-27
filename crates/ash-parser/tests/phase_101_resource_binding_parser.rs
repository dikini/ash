use ash_parser::input::new_input;
use ash_parser::parse_module::module_file;
use ash_parser::parse_workflow::workflow_def;
use ash_parser::surface::{Definition, Expr, Type, Visibility, Workflow};
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
fn parses_workflow_owns_and_uses_header_clauses_as_dedicated_carriers() {
    let mut input = new_input(
        "workflow example owns kv: WorkflowKV uses store: KVStore = MemoryKV(kv) { done }",
    );

    let parsed = workflow_def(&mut input)
        .expect("workflow owns/uses header clauses should parse before body");

    assert_eq!(parsed.name.as_ref(), "example");
    assert!(parsed.params.is_empty());
    assert_eq!(parsed.owned_resources.len(), 1);
    assert_eq!(parsed.owned_resources[0].name.as_ref(), "kv");
    assert!(matches!(
        &parsed.owned_resources[0].ty,
        Type::Name(name) if name.as_ref() == "WorkflowKV"
    ));

    assert_eq!(parsed.used_bindings.len(), 1);
    assert_eq!(parsed.used_bindings[0].name.as_ref(), "store");
    assert!(matches!(
        &parsed.used_bindings[0].interface,
        Type::Name(name) if name.as_ref() == "KVStore"
    ));
    assert!(matches!(
        &parsed.used_bindings[0].implementation,
        Expr::Constructor { name, payload, .. }
            if name.as_ref() == "MemoryKV"
                && matches!(payload, ash_parser::surface::ConstructorPayload::Tuple(items) if items.len() == 1)
    ));
    assert!(matches!(parsed.body, Workflow::Done { .. }));
}

#[test]
fn parses_workflow_header_clauses_with_existing_capabilities_clause() {
    let mut input = new_input(
        "workflow example owns kv: WorkflowKV uses store: KVStore = MemoryKV(kv) capabilities: [network] { done }",
    );

    let parsed = workflow_def(&mut input)
        .expect("owns/uses clauses should compose with legacy capabilities clause");

    assert_eq!(parsed.owned_resources.len(), 1);
    assert_eq!(parsed.used_bindings.len(), 1);
    assert_eq!(parsed.capabilities.len(), 1);
    assert_eq!(parsed.capabilities[0].capability.as_ref(), "network");
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

#[test]
fn rejects_malformed_uses_clause_missing_selected_implementation() {
    let mut input = new_input("workflow example uses store: KVStore { done }");

    let result = workflow_def(&mut input);

    assert!(
        result.is_err(),
        "uses clauses require `uses name: Interface = Implementation(...)`"
    );
}

#[test]
fn rejects_malformed_uses_clause_missing_binding_name() {
    let mut input = new_input("workflow example uses KVStore = MemoryKV(kv) { done }");

    let result = workflow_def(&mut input);

    assert!(
        result.is_err(),
        "uses clauses require an explicit binding name before the interface type"
    );
}
