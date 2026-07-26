//! TASK-2001 RED contracts for imported effect-row and handler summary metadata.
//!
//! A module summary already transports effect-row and handler exports. These
//! tests require the import boundary to register that metadata for later
//! checking without turning a row name into authority.

use ash_core::{
    ast::Visibility,
    module_graph::ModuleId,
    semantic_summary::{
        EffectRowAuthority, EffectRowClosureMetadata, EffectRowExportClassification,
        EffectRowExportId, EffectRowExportSummary, EffectRowItemSummary, ModuleIdentity,
        ModuleSemanticSummary, ModuleSourceOrigin, SourceAnchor, SourceOrigin, SummaryVersion,
        ValueExportKind, ValueExportSummary,
    },
};
use ash_typeck::{CallableDeclarationKind, TypeEnv};

fn module_identity(id: usize, path: &[&str]) -> ModuleIdentity {
    ModuleIdentity::new(
        None,
        ModuleId(id),
        path.iter().map(|segment| (*segment).to_string()).collect(),
        ModuleSourceOrigin::Synthetic {
            reason: format!("task-2001-import-{id}"),
        },
    )
}

fn anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "task-2001-import-test".to_string(),
        },
        None,
        label,
    )
}

fn row_export(
    module: &ModuleIdentity,
    name: &str,
    classification: EffectRowExportClassification,
    items: &[&str],
) -> EffectRowExportSummary {
    let mut row = EffectRowExportSummary::new(
        EffectRowExportId::new(module.clone(), name),
        name,
        Visibility::Public,
        classification,
        items
            .iter()
            .map(|item| EffectRowItemSummary::new(*item))
            .collect(),
        anchor(&format!("effect row {name}")),
    );
    row.closure_metadata = Some(EffectRowClosureMetadata {
        sanitizer_schema_version: 1,
        public_closure_digest: format!("task-2001::{name}"),
    });
    row
}

fn imported_summary() -> ModuleSemanticSummary {
    let module = module_identity(2001, &["fixtures", "effects"]);
    ModuleSemanticSummary::new(module.clone())
        .with_version(SummaryVersion::EFFECT_ROW_PROVIDER_BINDINGS_V7)
        .with_exported_effect_row(row_export(
            &module,
            "IO",
            EffectRowExportClassification::TransparentAlias,
            &["PosixFs::read", "evidence audit_log"],
        ))
        .with_exported_effect_row(row_export(
            &module,
            "WorkflowIO",
            EffectRowExportClassification::DiagnosticGroup,
            &["PosixFs::write"],
        ))
        .with_exported_value(ValueExportSummary::new(
            "canonical_handler",
            Visibility::Public,
            ValueExportKind::Handler,
            anchor("handler canonical_handler"),
        ))
}

#[test]
fn imported_effect_rows_are_lookupable_expandable_and_non_granting() {
    let summary = imported_summary();
    let mut env = TypeEnv::with_builtin_types();

    env.register_module_semantic_summary(&summary)
        .expect("public imported row metadata should register");

    let alias = env
        .lookup_effect_row_export("IO")
        .expect("imported transparent aliases must be available by visible name");
    assert_eq!(alias.id.module, summary.module);
    assert_eq!(
        alias.classification,
        EffectRowExportClassification::TransparentAlias
    );
    assert_eq!(alias.authority, EffectRowAuthority::NonGranting);

    let group = env
        .lookup_effect_row_export("WorkflowIO")
        .expect("imported diagnostic groups must be available by visible name");
    assert_eq!(
        group.classification,
        EffectRowExportClassification::DiagnosticGroup
    );
    assert_eq!(group.authority, EffectRowAuthority::NonGranting);

    let expanded = env
        .expand_effect_row_export("IO")
        .expect("registered transparent alias metadata must expand source-order row items");
    assert_eq!(
        expanded,
        vec![
            EffectRowItemSummary::new("PosixFs::read"),
            EffectRowItemSummary::new("evidence audit_log"),
        ]
    );
}

#[test]
fn imported_handler_value_export_satisfies_handler_only_admission() {
    let mut env = TypeEnv::with_builtin_types();
    env.register_module_semantic_summary(&imported_summary())
        .expect("public imported handler export should register");

    assert_eq!(
        env.callable_declaration_kind("canonical_handler"),
        Some(CallableDeclarationKind::Handler),
        "imported ValueExportKind::Handler must retain the marker used by handler-only checking"
    );
    env.require_handler_callable("canonical_handler")
        .expect("an imported handler export must satisfy handler-only admission");
}

#[test]
fn imported_effect_row_lookup_rejects_unknown_names() {
    let mut env = TypeEnv::with_builtin_types();
    env.register_module_semantic_summary(&imported_summary())
        .expect("test precondition: imported row metadata registers");

    assert!(
        env.expand_effect_row_export("Missing").is_err(),
        "unknown imported effect-row names must not silently become empty or authority-bearing rows"
    );
}

#[test]
fn imported_effect_row_registration_rejects_conflicting_duplicate_names() {
    let first = imported_summary();
    let conflicting_module = module_identity(2002, &["fixtures", "other_effects"]);
    let conflicting = ModuleSemanticSummary::new(conflicting_module.clone())
        .with_version(SummaryVersion::EFFECT_ROW_PROVIDER_BINDINGS_V7)
        .with_exported_effect_row(row_export(
            &conflicting_module,
            "IO",
            EffectRowExportClassification::DiagnosticGroup,
            &["PosixFs::write"],
        ));
    let mut env = TypeEnv::with_builtin_types();

    assert!(
        env.register_module_semantic_summaries(&[first, conflicting])
            .is_err(),
        "conflicting imported effect-row names must fail atomically rather than make import order observable"
    );
}
