//! TASK-2025: provider-binding summary version and cache-key contract.

use ash_core::ast::{Span, Visibility};
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    EffectRowBindingExposure, EffectRowClosureMetadata, EffectRowExportClassification,
    EffectRowExportId, EffectRowExportSummary, EffectRowItemSummary, ModuleIdentity,
    ModuleSemanticSummary, ModuleSemanticSummaryValidationError, ModuleSourceOrigin,
    ModuleSummaryRef, ReExportSummary, SourceAnchor, SourceOrigin, SummaryVersion, TypeDeclId,
};

fn module_identity() -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(2025)),
        ModuleId(2025),
        vec!["task2025".into(), "provider".into()],
        ModuleSourceOrigin::Synthetic {
            reason: "TASK-2025 summary version/cache fixture".into(),
        },
    )
}

fn anchor() -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: "TASK-2025 summary version/cache fixture".into(),
        },
        Some(Span { start: 0, end: 1 }),
        "public effect row",
    )
}

fn provider_row(
    provider_name: &str,
    visible_name: &str,
    exposure: EffectRowBindingExposure,
    sanitizer_schema_version: u16,
    public_closure_digest: &str,
) -> EffectRowExportSummary {
    let module = module_identity();
    let mut row = EffectRowExportSummary::new(
        EffectRowExportId::new(module, provider_name),
        provider_name,
        Visibility::Public,
        EffectRowExportClassification::TransparentAlias,
        vec![EffectRowItemSummary::new("TestClock::sleep")],
        anchor(),
    );
    row.set_visible_binding(visible_name, exposure);
    row.closure_metadata = Some(EffectRowClosureMetadata {
        sanitizer_schema_version,
        public_closure_digest: public_closure_digest.into(),
    });
    row
}

fn v7_summary(row: EffectRowExportSummary) -> ModuleSemanticSummary {
    ModuleSemanticSummary::new(module_identity())
        .with_version(SummaryVersion::EFFECT_ROW_PROVIDER_BINDINGS_V7)
        .with_exported_effect_row(row)
}

#[test]
fn provider_binding_payloads_under_v1_through_v6_are_rejected_fail_closed() {
    let versions = [
        SummaryVersion::SPEC057_ORDINARY_TYPE_V1,
        SummaryVersion::SPEC059_SEALED_DOMAIN_V2,
        SummaryVersion::SPEC062_TYPE_COMPUTATION_V3,
        SummaryVersion::SPEC063_ASSOCIATED_FAMILY_V4,
        SummaryVersion::SPEC064_PROPOSITIONS_V5,
        SummaryVersion::SPEC065_PROMOTED_DATA_KIND_V6,
    ];

    for version in versions {
        let summary = ModuleSemanticSummary::new(module_identity())
            .with_version(version)
            .with_exported_effect_row(provider_row(
                "Audit",
                "Audit",
                EffectRowBindingExposure::Declaration,
                1,
                "sha256:public-audit-closure",
            ));

        assert_eq!(
            summary.validate_summary_version_contract(),
            Err(
                ModuleSemanticSummaryValidationError::EffectRowProviderBindingsRequireV7 {
                    version,
                }
            ),
            "V{} provider-binding data must not be reinterpreted as V7 data",
            version.0
        );
    }
}

#[test]
fn v7_requires_complete_provider_binding_closure_metadata() {
    let module = module_identity();
    let incomplete = EffectRowExportSummary::new(
        EffectRowExportId::new(module, "Audit"),
        "Audit",
        Visibility::Public,
        EffectRowExportClassification::TransparentAlias,
        vec![EffectRowItemSummary::new("TestClock::sleep")],
        anchor(),
    );

    assert_eq!(
        v7_summary(incomplete).validate_summary_version_contract(),
        Err(
            ModuleSemanticSummaryValidationError::EffectRowProviderBindingClosureIncomplete {
                version: SummaryVersion::EFFECT_ROW_PROVIDER_BINDINGS_V7,
            }
        )
    );

    assert!(
        v7_summary(provider_row(
            "Audit",
            "PublicAudit",
            EffectRowBindingExposure::PublicReExport,
            1,
            "sha256:public-audit-closure",
        ))
        .validate_summary_version_contract()
        .is_ok()
    );
}

#[test]
fn v7_rejects_incoherent_provider_binding_identity_and_visible_name() {
    let mut provider_mismatch = provider_row(
        "Audit",
        "PublicAudit",
        EffectRowBindingExposure::PublicReExport,
        1,
        "sha256:public-audit-closure",
    );
    provider_mismatch.binding.provider.declaration_name = "OtherAudit".into();
    assert_eq!(
        v7_summary(provider_mismatch).validate_summary_version_contract(),
        Err(
            ModuleSemanticSummaryValidationError::EffectRowProviderBindingIncoherent {
                version: SummaryVersion::EFFECT_ROW_PROVIDER_BINDINGS_V7,
            }
        )
    );

    let mut binding_mismatch = provider_row(
        "Audit",
        "PublicAudit",
        EffectRowBindingExposure::PublicReExport,
        1,
        "sha256:public-audit-closure",
    );
    binding_mismatch.binding.visible_name = "OtherPublicAudit".into();
    assert_eq!(
        v7_summary(binding_mismatch).validate_summary_version_contract(),
        Err(
            ModuleSemanticSummaryValidationError::EffectRowProviderBindingIncoherent {
                version: SummaryVersion::EFFECT_ROW_PROVIDER_BINDINGS_V7,
            }
        )
    );
}

#[test]
fn complete_v7_wire_payload_rejects_unknown_fields() {
    let row = provider_row(
        "Audit",
        "PublicAudit",
        EffectRowBindingExposure::PublicReExport,
        1,
        "sha256:public-audit-closure",
    );
    let mut wire = serde_json::to_value(row).expect("complete V7 row should serialize");
    wire.as_object_mut()
        .expect("effect-row summary should serialize as an object")
        .insert(
            "unexpected_private_payload".into(),
            serde_json::json!("forbidden"),
        );

    assert!(
        serde_json::from_value::<EffectRowExportSummary>(wire).is_err(),
        "complete V7 payloads must be closed rather than dropping unknown fields"
    );
}

#[test]
fn unknown_future_provider_binding_summary_versions_are_rejected() {
    let future = SummaryVersion(SummaryVersion::EFFECT_ROW_PROVIDER_BINDINGS_V7.0 + 1);
    let summary = ModuleSemanticSummary::new(module_identity())
        .with_version(future)
        .with_exported_effect_row(provider_row(
            "Audit",
            "Audit",
            EffectRowBindingExposure::Declaration,
            1,
            "sha256:public-audit-closure",
        ));

    assert_eq!(
        summary.validate_summary_version_contract(),
        Err(ModuleSemanticSummaryValidationError::UnsupportedSummaryVersion { version: future })
    );
}

#[test]
fn semantic_cache_key_covers_provider_binding_and_sanitized_closure_but_never_opaque_secrets() {
    let base = v7_summary(provider_row(
        "Audit",
        "PublicAudit",
        EffectRowBindingExposure::PublicReExport,
        1,
        "sha256:public-audit-closure-a",
    ));
    let changed_provider = v7_summary(provider_row(
        "OtherAudit",
        "PublicAudit",
        EffectRowBindingExposure::PublicReExport,
        1,
        "sha256:public-audit-closure-a",
    ));
    let changed_binding = v7_summary(provider_row(
        "Audit",
        "AuditFacade",
        EffectRowBindingExposure::PublicReExport,
        1,
        "sha256:public-audit-closure-a",
    ));
    let changed_exposure = v7_summary(provider_row(
        "Audit",
        "PublicAudit",
        EffectRowBindingExposure::NamedImport,
        1,
        "sha256:public-audit-closure-a",
    ));
    let changed_closure_digest = v7_summary(provider_row(
        "Audit",
        "PublicAudit",
        EffectRowBindingExposure::PublicReExport,
        1,
        "sha256:public-audit-closure-b",
    ));
    let changed_sanitizer_schema = v7_summary(provider_row(
        "Audit",
        "PublicAudit",
        EffectRowBindingExposure::PublicReExport,
        2,
        "sha256:public-audit-closure-a",
    ));

    for changed in [
        changed_provider,
        changed_binding,
        changed_exposure,
        changed_closure_digest,
        changed_sanitizer_schema,
    ] {
        assert_ne!(base.semantic_cache_key(), changed.semantic_cache_key());
    }

    let mut first_opaque = provider_row(
        "PrivateDependencyOne",
        "PublicAudit",
        EffectRowBindingExposure::PublicReExport,
        1,
        "sha256:private-dependency-one",
    );
    first_opaque.mark_opaque_inaccessible_dependency();
    let mut second_opaque = provider_row(
        "PrivateDependencyTwo",
        "PublicAudit",
        EffectRowBindingExposure::PublicReExport,
        1,
        "sha256:private-dependency-two",
    );
    second_opaque.mark_opaque_inaccessible_dependency();

    let first_opaque_key = v7_summary(first_opaque).semantic_cache_key();
    let second_opaque_key = v7_summary(second_opaque).semantic_cache_key();
    assert_eq!(first_opaque_key, second_opaque_key);
    assert!(
        first_opaque_key
            .iter()
            .all(|part| !part.contains("PrivateDependency"))
    );
    assert!(
        second_opaque_key
            .iter()
            .all(|part| !part.contains("private-dependency"))
    );
}

#[test]
fn opaque_cache_key_discards_all_surrounding_summary_detail() {
    let secret = "PRIVATE_SUMMARY_CONTEXT_2025";
    let mut first_row = provider_row(
        secret,
        "PublicAudit",
        EffectRowBindingExposure::PublicReExport,
        1,
        secret,
    );
    first_row.mark_opaque_inaccessible_dependency();
    let mut first = v7_summary(first_row);
    first.module.path = vec![secret.into()];
    first.re_exports.push(ReExportSummary::new(
        vec![secret.into()],
        TypeDeclId::ordinary(module_identity(), secret),
    ));
    first.imported_summary_refs.push(ModuleSummaryRef {
        module: ModuleIdentity::new(
            None,
            ModuleId(2026),
            vec![secret.into()],
            ModuleSourceOrigin::Synthetic {
                reason: secret.into(),
            },
        ),
        version: SummaryVersion::EFFECT_ROW_PROVIDER_BINDINGS_V7,
    });
    first.diagnostic_anchors.push(SourceAnchor::new(
        SourceOrigin::Synthetic {
            reason: secret.into(),
        },
        None,
        secret,
    ));

    let mut second_row = provider_row(
        "OtherPrivateDependency",
        "PublicAudit",
        EffectRowBindingExposure::PublicReExport,
        2,
        "sha256:another-private-closure",
    );
    second_row.mark_opaque_inaccessible_dependency();
    let second = v7_summary(second_row);

    let first_key = first.semantic_cache_key();
    let second_key = second.semantic_cache_key();
    assert_eq!(first_key, second_key);
    assert!(first_key.iter().all(|part| !part.contains(secret)));
}

#[test]
fn v7_rejects_an_unknown_nonzero_sanitizer_schema_version_deterministically() {
    let unknown_sanitizer_schema = u16::MAX;
    let summary = v7_summary(provider_row(
        "Audit",
        "PublicAudit",
        EffectRowBindingExposure::PublicReExport,
        unknown_sanitizer_schema,
        "sha256:public-audit-closure",
    ));

    let first = summary
        .validate_summary_version_contract()
        .expect_err("an unknown sanitizer schema must fail closed");
    let second = summary
        .validate_summary_version_contract()
        .expect_err("the unknown-schema rejection must be deterministic");

    assert_eq!(first, second);
}
