//! TASK-2001 RED contract for effect-row and handler module-summary exports.
//!
//! The existing ordinary-type summary path is intentionally insufficient for
//! these declarations.  This test names the dedicated, checked export surface
//! required before aliases, groups, or handler markers can cross a module
//! boundary.  In particular, an effect-row name is diagnostic/typechecking
//! metadata only: it must not become an authority grant merely by being
//! exported.

use std::path::Path;

use ash_core::{
    module_graph::ModuleId,
    semantic_summary::{
        EffectRowAuthority, EffectRowExportClassification, ModuleIdentity, ModuleSourceOrigin,
        SourceOrigin, ValueExportKind,
    },
};
use ash_parser::lower::lower_module_type_metadata;

fn module_identity() -> ModuleIdentity {
    ModuleIdentity::new(
        None,
        ModuleId(2001),
        vec!["task_2001".to_string()],
        ModuleSourceOrigin::File("task-2001-effects.ash".to_string()),
    )
}

fn lower(source: &str) -> ash_parser::lower::LoweredTypeMetadata {
    let module =
        ash_parser::parse_surface_file_with_path(source, Some(Path::new("task-2001-effects.ash")))
            .expect("canonical TASK-2001 declarations parse before summary lowering");

    lower_module_type_metadata(&module, module_identity())
}

#[test]
fn task_2001_exports_effect_alias_row_with_non_granting_transparent_classification_and_origin() {
    let lowered = lower("pub effect alias IO = {PosixFs::read, evidence audit_log};");

    let alias = lowered
        .summary
        .exported_effect_rows
        .iter()
        .find(|row| row.exported_name.as_str() == "IO")
        .expect("effect aliases must cross the dedicated module-summary handoff");

    assert_eq!(alias.id.module, module_identity());
    assert_eq!(
        alias.classification,
        EffectRowExportClassification::TransparentAlias
    );
    assert_eq!(alias.authority, EffectRowAuthority::NonGranting);
    assert_eq!(alias.row_items.len(), 2);
    assert_eq!(alias.source_anchor.label, "effect alias IO");
    assert_eq!(
        alias.source_anchor.origin,
        SourceOrigin::File("task-2001-effects.ash".to_string())
    );
}

#[test]
fn task_2001_exports_effect_group_row_with_diagnostic_non_granting_classification_and_origin() {
    let lowered = lower("pub effect group WorkflowIO = {PosixFs::write, evidence audit_log};");

    let group = lowered
        .summary
        .exported_effect_rows
        .iter()
        .find(|row| row.exported_name.as_str() == "WorkflowIO")
        .expect("effect groups must cross the dedicated module-summary handoff");

    assert_eq!(group.id.module, module_identity());
    assert_eq!(
        group.classification,
        EffectRowExportClassification::DiagnosticGroup
    );
    assert_eq!(group.authority, EffectRowAuthority::NonGranting);
    assert_eq!(group.row_items.len(), 2);
    assert_eq!(group.source_anchor.label, "effect group WorkflowIO");
    assert_eq!(
        group.source_anchor.origin,
        SourceOrigin::File("task-2001-effects.ash".to_string())
    );
}

#[test]
fn task_2001_exports_handler_as_value_with_distinct_handler_marker_and_origin() {
    let lowered = lower("pub handler canonical_handler(comp: Unit) -> Unit { comp }");

    let handler = lowered
        .summary
        .exported_values
        .iter()
        .find(|value| value.exported_name.as_str() == "canonical_handler")
        .expect("handlers must be exported in the value namespace");

    assert_eq!(handler.kind, ValueExportKind::Handler);
    assert_eq!(handler.source_anchor.label, "handler canonical_handler");
    assert_eq!(
        handler.source_anchor.origin,
        SourceOrigin::File("task-2001-effects.ash".to_string())
    );
}
