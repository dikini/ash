use ash_core::ast::{Span, Visibility};
use ash_core::kind::Kind;
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    ConstructorId, ConstructorPayloadKind, ModuleIdentity, ModuleSemanticSummary,
    ModuleSemanticSummaryValidationError, ModuleSourceOrigin, PromotedConstructorFieldSummary,
    PromotedConstructorId, PromotedConstructorSummary, PromotedDataKindId, PromotedDataKindSummary,
    SourceAnchor, SourceOrigin, SummaryVersion, TypeDeclId,
};
use ash_core::type_ir::{CanonicalTypeExpr, PromotedConstructorApp, TypeLevelConstructorApp};

fn module_identity() -> ModuleIdentity {
    ModuleIdentity::new(
        Some(CrateId(7)),
        ModuleId(42),
        vec!["crate".into(), "nat".into()],
        ModuleSourceOrigin::File("/repo/src/nat.ash".into()),
    )
}

fn anchor(label: &str) -> SourceAnchor {
    SourceAnchor::new(
        SourceOrigin::File("/repo/src/nat.ash".into()),
        Some(Span { start: 0, end: 10 }),
        label,
    )
}

#[test]
fn task_894_promoted_identities_are_distinct_from_runtime_and_sealed_domain_ids() {
    let module = module_identity();
    let source_type = TypeDeclId::ordinary(module.clone(), "Nat");
    let runtime_ctor =
        ConstructorId::variant(source_type.clone(), "Z", ConstructorPayloadKind::Unit);
    let kind = PromotedDataKindId::new(module.clone(), source_type.clone(), "NatKind");
    let promoted_ctor = PromotedConstructorId::new(kind.clone(), runtime_ctor.clone(), "Z");

    assert_eq!(kind.module, module);
    assert_eq!(kind.source_type, source_type);
    assert_eq!(kind.name.as_str(), "NatKind");
    assert_eq!(promoted_ctor.kind, kind);
    assert_eq!(promoted_ctor.source_constructor, runtime_ctor);
    assert_eq!(promoted_ctor.name.as_str(), "Z");
}

#[test]
fn task_894_summary_version_contract_rejects_promoted_data_kinds_before_v6() {
    let module = module_identity();
    let source_type = TypeDeclId::ordinary(module.clone(), "Nat");
    let kind = PromotedDataKindSummary::new(
        PromotedDataKindId::new(module.clone(), source_type.clone(), "NatKind"),
        "NatKind",
        Visibility::Public,
        source_type,
        anchor("data kind NatKind"),
    );

    let old_summary =
        ModuleSemanticSummary::new(module.clone()).with_exported_promoted_data_kind(kind.clone());
    assert_eq!(
        old_summary.validate_summary_version_contract(),
        Err(
            ModuleSemanticSummaryValidationError::PromotedDataKindsRequireV6 {
                version: SummaryVersion::SPEC057_ORDINARY_TYPE_V1,
            }
        )
    );

    let v6_summary = ModuleSemanticSummary::new(module)
        .with_version(SummaryVersion::SPEC065_PROMOTED_DATA_KIND_V6)
        .with_exported_promoted_data_kind(kind);
    assert!(v6_summary.validate_summary_version_contract().is_ok());
}

#[test]
fn task_894_promoted_summary_carries_constructor_field_metadata_and_cache_key() {
    let module = module_identity();
    let source_type = TypeDeclId::ordinary(module.clone(), "Nat");
    let kind_id = PromotedDataKindId::new(module.clone(), source_type.clone(), "NatKind");
    let source_ctor =
        ConstructorId::variant(source_type.clone(), "S", ConstructorPayloadKind::Tuple);
    let ctor_id = PromotedConstructorId::new(kind_id.clone(), source_ctor.clone(), "S");
    let field = PromotedConstructorFieldSummary::new(
        "pred",
        Kind::Type,
        Some(kind_id.clone()),
        anchor("pred"),
    );
    let ctor_summary = PromotedConstructorSummary::new(
        ctor_id.clone(),
        "S",
        source_ctor,
        vec![field.clone()],
        Visibility::Public,
        anchor("S"),
    );
    let kind_summary = PromotedDataKindSummary::new(
        kind_id.clone(),
        "NatKind",
        Visibility::Public,
        source_type,
        anchor("NatKind"),
    )
    .with_constructor(ctor_summary);

    assert_eq!(kind_summary.constructors.len(), 1);
    assert_eq!(kind_summary.constructors[0].fields, vec![field]);

    let summary = ModuleSemanticSummary::new(module)
        .with_version(SummaryVersion::SPEC065_PROMOTED_DATA_KIND_V6)
        .with_exported_promoted_data_kind(kind_summary);
    assert!(
        summary
            .semantic_cache_key()
            .iter()
            .any(|entry| entry.starts_with("promoted_data_kind::NatKind::"))
    );
}

#[test]
fn task_894_type_level_app_carrier_preserves_promoted_constructor_family() {
    let module = module_identity();
    let source_type = TypeDeclId::ordinary(module.clone(), "Nat");
    let kind_id = PromotedDataKindId::new(module, source_type.clone(), "NatKind");
    let source_ctor = ConstructorId::variant(source_type, "Z", ConstructorPayloadKind::Unit);
    let ctor_id = PromotedConstructorId::new(kind_id.clone(), source_ctor, "Z");

    let app = PromotedConstructorApp {
        constructor: ctor_id.clone(),
        data_kind: kind_id.clone(),
        args: Vec::new(),
        kind: Kind::Type,
    };
    let type_level = TypeLevelConstructorApp::PromotedDataConstructor(Box::new(app.clone()));
    let canonical = CanonicalTypeExpr::PromotedDataConstructorApp(Box::new(app));

    assert!(matches!(
        type_level,
        TypeLevelConstructorApp::PromotedDataConstructor(app)
            if app.constructor == ctor_id && app.data_kind == kind_id
    ));
    assert!(matches!(
        canonical,
        CanonicalTypeExpr::PromotedDataConstructorApp(_)
    ));
}
