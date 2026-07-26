//! TASK-2025: imported effect-row provider-binding summaries fail closed at TypeEnv.

use ash_core::ast::{Span, Visibility};
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    EffectRowBindingExposure, EffectRowClosureMetadata, EffectRowExportClassification,
    EffectRowExportId, EffectRowExportSummary, EffectRowItemSummary, ModuleIdentity,
    ModuleSemanticSummary, ModuleSourceOrigin, SourceAnchor, SourceOrigin, SummaryVersion,
};
use ash_typeck::TypeEnv;

fn module_identity() -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(2025)),
        ModuleId(2025),
        vec!["task2025".into(), "import_boundary".into()],
        ModuleSourceOrigin::Synthetic {
            reason: "TASK-2025 TypeEnv fixture".into(),
        },
    )
}

#[test]
fn opaque_provider_binding_summary_rejects_without_echoing_private_row_data() {
    let secret = "PRIVATE_OPAQUE_ROW_2025";
    let module = module_identity();
    let mut row = EffectRowExportSummary::new(
        EffectRowExportId::new(module.clone(), secret),
        secret,
        Visibility::Public,
        EffectRowExportClassification::TransparentAlias,
        vec![EffectRowItemSummary::new(secret)],
        SourceAnchor::new(
            SourceOrigin::Synthetic {
                reason: secret.into(),
            },
            Some(Span { start: 0, end: 1 }),
            secret,
        ),
    );
    row.closure_metadata = Some(EffectRowClosureMetadata {
        sanitizer_schema_version: 1,
        public_closure_digest: secret.into(),
    });
    row.mark_opaque_inaccessible_dependency();
    let summary = ModuleSemanticSummary::new(module)
        .with_version(SummaryVersion::EFFECT_ROW_PROVIDER_BINDINGS_V7)
        .with_exported_effect_row(row);

    let mut env = TypeEnv::new();
    let error = env
        .register_module_semantic_summary(&summary)
        .expect_err("opaque effect-row boundaries are not usable imported rows");

    assert_eq!(
        error.to_string(),
        "malformed imported-effect-row-summary: provider-binding effect-row closure is inaccessible at public boundary"
    );
    assert!(!error.to_string().contains(secret));
    assert!(env.lookup_effect_row_export(secret).is_none());
}

#[test]
fn incoherent_provider_binding_rejects_transactionally_without_private_detail() {
    let secret = "PRIVATE_INCOHERENT_ROW_2025";
    let module = module_identity();
    let mut row = EffectRowExportSummary::new(
        EffectRowExportId::new(module.clone(), "PublicAudit"),
        "PublicAudit",
        Visibility::Public,
        EffectRowExportClassification::TransparentAlias,
        vec![EffectRowItemSummary::new("TestClock::sleep")],
        SourceAnchor::new(
            SourceOrigin::Synthetic {
                reason: secret.into(),
            },
            Some(Span { start: 0, end: 1 }),
            secret,
        ),
    );
    row.provider.declaration_name = secret.into();
    row.closure_metadata = Some(EffectRowClosureMetadata {
        sanitizer_schema_version: 1,
        public_closure_digest: "sha256:public-audit-closure".into(),
    });
    let summary = ModuleSemanticSummary::new(module)
        .with_version(SummaryVersion::EFFECT_ROW_PROVIDER_BINDINGS_V7)
        .with_exported_effect_row(row);

    let mut env = TypeEnv::new();
    let error = env
        .register_module_semantic_summary(&summary)
        .expect_err("incoherent provider bindings must not partially register");

    assert_eq!(
        error.to_string(),
        "malformed imported-effect-row-summary: provider-binding effect-row identity is incoherent at public boundary"
    );
    assert!(!error.to_string().contains(secret));
    assert!(env.lookup_effect_row_export("PublicAudit").is_none());
}

fn legacy_provider_binding_summary() -> ModuleSemanticSummary {
    let module = module_identity();
    let row = EffectRowExportSummary::new(
        EffectRowExportId::new(module.clone(), "Audit"),
        "Audit",
        Visibility::Public,
        EffectRowExportClassification::TransparentAlias,
        vec![EffectRowItemSummary::new("TestClock::sleep")],
        SourceAnchor::new(
            SourceOrigin::Synthetic {
                reason: "TASK-2025 TypeEnv fixture".into(),
            },
            Some(Span { start: 0, end: 1 }),
            "public Audit row",
        ),
    );

    ModuleSemanticSummary::new(module)
        .with_version(SummaryVersion::SPEC065_PROMOTED_DATA_KIND_V6)
        .with_exported_effect_row(row)
}

fn provider_binding_summary(
    module_number: usize,
    declaration_name: &str,
    visible_name: &str,
    item: &str,
) -> ModuleSemanticSummary {
    let module = ModuleIdentity::new(
        Some(CrateId(module_number)),
        ModuleId(module_number),
        vec![format!("provider_{module_number}")],
        ModuleSourceOrigin::Synthetic {
            reason: "TASK-2025 provider-binding fixture".into(),
        },
    );
    let mut row = EffectRowExportSummary::new(
        EffectRowExportId::new(module.clone(), declaration_name),
        declaration_name,
        Visibility::Public,
        EffectRowExportClassification::TransparentAlias,
        vec![EffectRowItemSummary::new(item)],
        SourceAnchor::new(
            SourceOrigin::Synthetic {
                reason: "TASK-2025 provider-binding fixture".into(),
            },
            Some(Span { start: 0, end: 1 }),
            "public provider binding",
        ),
    );
    row.set_visible_binding(visible_name, EffectRowBindingExposure::NamedImport);
    row.closure_metadata = Some(EffectRowClosureMetadata {
        sanitizer_schema_version: 1,
        public_closure_digest: format!("sha256:provider-{module_number}-{declaration_name}-{item}"),
    });
    ModuleSemanticSummary::new(module)
        .with_version(SummaryVersion::EFFECT_ROW_PROVIDER_BINDINGS_V7)
        .with_exported_effect_row(row)
}

fn facade_provider_binding_summary(
    facade_number: usize,
    provider_summary: &ModuleSemanticSummary,
) -> ModuleSemanticSummary {
    let facade = ModuleIdentity::new(
        Some(CrateId(facade_number)),
        ModuleId(facade_number),
        vec![format!("facade_{facade_number}")],
        ModuleSourceOrigin::Synthetic {
            reason: "TASK-2025 facade-binding fixture".into(),
        },
    );
    let mut row = provider_summary.exported_effect_rows[0].clone();
    row.id.module = facade.clone();
    row.id.name.clone_from(&row.binding.visible_name);
    row.source_anchor = SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: format!("TASK-2025 facade {facade_number}"),
        },
        Some(Span {
            start: facade_number,
            end: facade_number + 1,
        }),
        "facade provider binding",
    );
    ModuleSemanticSummary::new(facade)
        .with_version(SummaryVersion::EFFECT_ROW_PROVIDER_BINDINGS_V7)
        .with_exported_effect_row(row)
}

#[test]
fn second_incompatible_provider_binding_rejects_without_replacing_the_first_visible_binding() {
    let first = provider_binding_summary(20251, "Audit", "Shared", "TestClock::sleep");
    let second = provider_binding_summary(20252, "Audit", "Shared", "TestClock::wake");
    let expected_first = first.exported_effect_rows[0].clone();
    let mut env = TypeEnv::new();

    env.register_module_semantic_summary(&first)
        .expect("first provider binding registers");
    let error = env
        .register_module_semantic_summary(&second)
        .expect_err("an incompatible second provider must not replace an existing visible binding");

    assert!(error.to_string().contains("import-order-conflict"));
    assert_eq!(
        env.lookup_effect_row_export("Shared"),
        Some(&expected_first)
    );
}

#[test]
fn duplicate_same_provider_binding_is_idempotent_at_the_typeenv_boundary() {
    let summary = provider_binding_summary(20253, "Audit", "Shared", "TestClock::sleep");
    let expected = summary.exported_effect_rows[0].clone();
    let mut env = TypeEnv::new();

    env.register_module_semantic_summary(&summary)
        .expect("first provider binding registers");
    env.register_module_semantic_summary(&summary)
        .expect("the same provider binding is idempotent");
    assert_eq!(env.lookup_effect_row_export("Shared"), Some(&expected));
}

#[test]
fn equivalent_provider_binding_through_two_facades_is_idempotent() {
    let provider = provider_binding_summary(20254, "Audit", "Shared", "TestClock::sleep");
    let first_facade = facade_provider_binding_summary(20255, &provider);
    let second_facade = facade_provider_binding_summary(20256, &provider);
    let expected_first = first_facade.exported_effect_rows[0].clone();
    assert_ne!(
        first_facade.exported_effect_rows[0].id.module,
        second_facade.exported_effect_rows[0].id.module,
        "the fixture must differ only at facade-local publication identity"
    );
    assert_ne!(
        first_facade.exported_effect_rows[0].source_anchor,
        second_facade.exported_effect_rows[0].source_anchor,
        "facade-local diagnostic anchors are not semantic binding identity"
    );

    let mut env = TypeEnv::new();
    env.register_module_semantic_summary(&first_facade)
        .expect("the first facade binding registers");
    env.register_module_semantic_summary(&second_facade)
        .expect("equivalent provider bindings through different facades are idempotent");
    assert_eq!(
        env.lookup_effect_row_export("Shared"),
        Some(&expected_first),
        "the second facade must not replace the already registered binding"
    );
}

#[test]
fn same_provider_with_an_incompatible_sanitized_contract_rejects_transactionally_in_either_order() {
    let first = provider_binding_summary(20257, "Audit", "Shared", "TestClock::sleep");
    let mut incompatible = first.clone();
    incompatible.exported_effect_rows[0].row_items =
        vec![EffectRowItemSummary::new("TestClock::wake")];
    incompatible.exported_effect_rows[0].closure_metadata = Some(EffectRowClosureMetadata {
        sanitizer_schema_version: 1,
        public_closure_digest: "sha256:incompatible-sanitized-contract".into(),
    });

    let mut first_then_incompatible = TypeEnv::new();
    let first_error = first_then_incompatible
        .register_module_semantic_summaries(&[first.clone(), incompatible.clone()])
        .expect_err("same-provider closure changes must reject as an import-order conflict");
    assert!(first_error.to_string().contains("import-order-conflict"));
    assert!(
        first_then_incompatible
            .lookup_effect_row_export("Shared")
            .is_none(),
        "a failed batch must not publish the first contract"
    );

    let mut incompatible_then_first = TypeEnv::new();
    let reverse_error = incompatible_then_first
        .register_module_semantic_summaries(&[incompatible, first])
        .expect_err("reverse source order must reject the same incompatible contract");
    assert_eq!(first_error.to_string(), reverse_error.to_string());
    assert!(
        incompatible_then_first
            .lookup_effect_row_export("Shared")
            .is_none(),
        "reverse failed batch must not publish a binding"
    );
}

#[test]
fn legacy_provider_binding_summary_is_rejected_at_a_public_typeenv_boundary_without_private_detail()
{
    let mut env = TypeEnv::new();
    let error = env
        .register_module_semantic_summary(&legacy_provider_binding_summary())
        .expect_err("V1-V6 summaries must not register provider-binding effect rows");

    assert_eq!(
        error.to_string(),
        "malformed imported-effect-row-summary: module semantic summary version 6 cannot carry provider-binding effect-row summaries; expected 7"
    );
    assert!(
        env.lookup_effect_row_export("Audit").is_none(),
        "version rejection must be transactional"
    );
    assert!(
        !error.to_string().contains("TestClock::sleep"),
        "the public boundary must not report row contents"
    );
}
