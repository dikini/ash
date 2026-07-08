use ash_parser::module::ModuleSource;
use ash_parser::surface::{Definition, TypeBody, VariantPayload, Visibility};

fn parse(source: &str) -> ash_parser::surface::ModuleFile {
    ash_parser::parse_surface_file(source).expect("module file should parse")
}

fn type_names(module: &ash_parser::surface::ModuleFile) -> Vec<&str> {
    module
        .definitions
        .iter()
        .filter_map(|definition| match definition {
            Definition::Type(type_def) => Some(type_def.name.as_ref()),
            _ => None,
        })
        .collect()
}

#[test]
fn top_level_module_file_keeps_ordinary_type_definition() {
    let source =
        "pub type Result<T, E> = Ok { value: T } | Err { error: E };\nfn id(x: Int) -> Int { x }";
    let module =
        ash_parser::parse_surface_file_with_path(source, Some(std::path::Path::new("task782.ash")))
            .expect("module file should parse");

    assert_eq!(module.definitions.len(), 2);
    let Definition::Type(type_def) = &module.definitions[0] else {
        panic!(
            "expected ordinary type definition, got {:?}",
            module.definitions[0]
        );
    };

    assert_eq!(type_def.visibility, Visibility::Public);
    assert_eq!(type_def.name.as_ref(), "Result");
    assert_eq!(type_def.params, vec!["T".into(), "E".into()]);
    assert!(!type_def.builtin);
    assert!(type_def.source.is_some());
    assert_eq!(type_def.span.start, 0);
    assert!(type_def.span.end > type_def.span.start);

    let TypeBody::Enum(variants) = &type_def.body else {
        panic!("expected enum body, got {:?}", type_def.body);
    };
    assert_eq!(variants.len(), 2);
    assert_eq!(variants[0].name.as_ref(), "Ok");
    assert_eq!(variants[0].span, type_def.span);
    assert_eq!(variants[0].fields[0].name.as_ref(), "value");
    assert_eq!(variants[0].fields[0].span, type_def.span);
    assert!(matches!(variants[0].payload, VariantPayload::Record(_)));
}

#[test]
fn file_containing_only_ordinary_types_is_valid_module_file() {
    let module = parse("type UserId = String;\npub(crate) builtin type HostHandle;");

    assert_eq!(module.definitions.len(), 2);
    let Definition::Type(alias) = &module.definitions[0] else {
        panic!("expected alias type");
    };
    assert_eq!(alias.visibility, Visibility::Inherited);
    assert_eq!(alias.name.as_ref(), "UserId");
    assert!(matches!(alias.body, TypeBody::Alias(_)));

    let Definition::Type(builtin) = &module.definitions[1] else {
        panic!("expected builtin type");
    };
    assert_eq!(builtin.visibility, Visibility::Crate);
    assert_eq!(builtin.name.as_ref(), "HostHandle");
    assert!(builtin.builtin);
}

#[test]
fn function_entry_file_can_include_file_local_type_declarations() {
    let module = parse("type Request = { id: String };\nfn main() -> Int { 0 }");

    assert_eq!(module.definitions.len(), 2);
    assert!(matches!(module.definitions[0], Definition::Type(_)));
    assert!(matches!(module.definitions[1], Definition::Function(_)));
    assert!(module.workflow.is_none());
}

#[test]
fn inline_module_keeps_ordinary_type_definitions() {
    let module = parse("mod model { type Status = Pending | Done; }");

    assert_eq!(module.module_decls.len(), 1);
    let ModuleSource::Inline(definitions) = &module.module_decls[0].source else {
        panic!("expected inline module");
    };
    assert_eq!(definitions.len(), 1);
    let Definition::Type(type_def) = &definitions[0] else {
        panic!("expected inline ordinary type definition");
    };
    assert_eq!(type_def.name.as_ref(), "Status");
}

#[test]
fn inline_type_definitions_keep_source_origin_when_path_is_known() {
    let module = ash_parser::parse_surface_file_with_path(
        "mod model { type Status = Pending | Done; }",
        Some(std::path::Path::new("models/status.ash")),
    )
    .expect("module file should parse");

    let ModuleSource::Inline(definitions) = &module.module_decls[0].source else {
        panic!("expected inline module");
    };
    let Definition::Type(type_def) = &definitions[0] else {
        panic!("expected inline ordinary type definition");
    };
    assert_eq!(type_def.source.as_deref(), Some("models/status.ash"));
}

#[test]
fn unknown_item_recovery_no_longer_preserves_visible_type_definition_forms() {
    let source = concat!(
        "extension custom { enabled: true } ",
        "pub type Status = Pending | Done; ",
        "extension second { enabled: false } ",
        "pub(crate) builtin type HostHandle;",
    );
    assert!(
        ash_parser::parse_surface_file(source).is_err(),
        "unknown top-level items must fail closed instead of recovering past stale syntax"
    );
}

#[test]
fn unknown_item_recovery_no_longer_preserves_following_visible_function() {
    let source = "extension custom { enabled: true } pub fn id(x: Int) -> Int { x }";
    assert!(
        ash_parser::parse_surface_file(source).is_err(),
        "unknown top-level items must fail closed instead of recovering past stale syntax"
    );
}

#[test]
fn unknown_item_recovery_no_longer_skips_to_following_valid_type_definition() {
    let source = "extension custom { enabled: true } type Status = Pending | Done;";
    assert!(
        ash_parser::parse_surface_file(source).is_err(),
        "unknown top-level items must fail closed instead of recovering past stale syntax"
    );
}

#[test]
fn deferred_type_computation_syntax_does_not_parse_as_ordinary_type_definition() {
    for source in [
        "type fn Append<A, B> = A;\npub type Keep = Int;",
        "sealed type domain Nat { Zero, Succ };\npub type Keep = Int;",
    ] {
        if let Ok(module) = ash_parser::parse_surface_file(source) {
            assert_eq!(
                type_names(&module),
                vec!["Keep"],
                "deferred DESIGN-034 syntax must remain unrecognized, not ordinary type metadata: {source}"
            );
        }
    }
}
