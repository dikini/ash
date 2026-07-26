//! TASK-2001 RED fixtures for target effect-row declaration forms.
//!
//! These assertions deliberately avoid naming a surface AST variant until the
//! parser owns one.  The implementation must replace the coarse item-count
//! checks with structural assertions for the alias/group declarations while
//! retaining their source spans and the module source path for Core lowering.

use std::path::Path;

use ash_parser::{
    module::ModuleSource,
    surface::{ComputationRowItem, Definition, Visibility},
};

fn parse_with_origin(source: &str) -> ash_parser::surface::ModuleFile {
    ash_parser::parse_surface_file_with_path(source, Some(Path::new("task-2001.ash")))
        .expect("target effect-row declaration should parse")
}

#[test]
fn task_2001_parses_transparent_effect_alias_declaration() {
    let module = parse_with_origin("effect alias IO = {PosixFs::read, PosixFs::write};");

    assert_eq!(module.definitions.len(), 1);
    assert_eq!(module.path.as_deref(), Some("task-2001.ash"));
    let Definition::EffectAlias(alias) = &module.definitions[0] else {
        panic!("expected transparent effect alias");
    };
    assert_eq!(alias.visibility, Visibility::Inherited);
    assert_eq!(alias.name.as_ref(), "IO");
    assert_eq!(alias.row.items.len(), 2);
    assert!(matches!(
        alias.row.items[0],
        ComputationRowItem::Operation { ref path, .. }
            if path.iter().map(|part| part.as_ref()).collect::<Vec<_>>() == ["PosixFs", "read"]
    ));
    assert!(alias.span.end > alias.span.start);
    assert_eq!(alias.source.as_deref(), Some("task-2001.ash"));
}

#[test]
fn task_2001_parses_diagnostic_effect_group_declaration() {
    let module = parse_with_origin(
        "effect group WorkflowIO = {\n    PosixFs::read,\n    StdoutLog::write,\n    evidence audit_log,\n};",
    );

    assert_eq!(module.definitions.len(), 1);
    assert_eq!(module.path.as_deref(), Some("task-2001.ash"));
    let Definition::EffectGroup(group) = &module.definitions[0] else {
        panic!("expected diagnostic effect group");
    };
    assert_eq!(group.name.as_ref(), "WorkflowIO");
    assert_eq!(group.row.items.len(), 3);
    assert!(matches!(
        group.row.items[2],
        ComputationRowItem::Evidence { ref path, .. }
            if path.iter().map(|part| part.as_ref()).collect::<Vec<_>>() == ["audit_log"]
    ));
    assert!(group.span.end > group.span.start);
    assert_eq!(group.source.as_deref(), Some("task-2001.ash"));
}

#[test]
fn task_2001_parses_effect_declarations_inside_inline_modules() {
    let module = parse_with_origin("mod effects { effect group Audit = {evidence audit_log}; }");

    let ModuleSource::Inline(definitions) = &module.module_decls[0].source else {
        panic!("expected inline module");
    };
    let Definition::EffectGroup(group) = &definitions[0] else {
        panic!("expected inline diagnostic effect group");
    };
    assert_eq!(group.name.as_ref(), "Audit");
    assert_eq!(group.source.as_deref(), Some("task-2001.ash"));
}

#[test]
fn task_2001_rejects_historical_proxy_definition() {
    let error = ash_parser::parse_surface_file("proxy assistant { return 0 }")
        .expect_err("historical proxy definitions must remain rejected");

    assert_eq!(
        error[0].message,
        "`proxy` declarations are removed from target Ash"
    );
}
