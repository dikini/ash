//! TASK-2001 RED contract for V8 structural imported effect-row summaries.
//!
//! The V7 provider/binding transport retained formatted row text.  V8 must
//! preserve the same non-authorizing provider/binding envelope while carrying
//! each row requirement structurally, so typed-handler normalization never
//! reparses summary text.

use ash_core::ast::{Span, Visibility};
use ash_core::module_graph::{CrateId, ModuleId};
use ash_core::semantic_summary::{
    EffectRowClosureMetadata, EffectRowExportClassification, EffectRowExportId,
    EffectRowExportSummary, EffectRowItemSummary, ModuleIdentity, ModuleSemanticSummary,
    ModuleSourceOrigin, SourceAnchor, SourceOrigin, SummaryVersion,
};

fn v7_summary() -> ModuleSemanticSummary {
    let module = ModuleIdentity::new(
        Some(CrateId(2001)),
        ModuleId(2001),
        vec!["task2001".into(), "v8_structural_row".into()],
        ModuleSourceOrigin::Synthetic {
            reason: "TASK-2001 V8 structural summary fixture".into(),
        },
    );
    let mut row = EffectRowExportSummary::new(
        EffectRowExportId::new(module.clone(), "ClockAudit"),
        "ClockAudit",
        Visibility::Public,
        EffectRowExportClassification::TransparentAlias,
        vec![EffectRowItemSummary::new("TestClock::sleep")],
        SourceAnchor::new(
            SourceOrigin::Synthetic {
                reason: "TASK-2001 V8 structural summary fixture".into(),
            },
            Some(Span { start: 0, end: 1 }),
            "public ClockAudit row",
        ),
    );
    row.closure_metadata = Some(EffectRowClosureMetadata {
        sanitizer_schema_version: 1,
        public_closure_digest: "sha256:task-2001-clock-audit".into(),
    });

    ModuleSemanticSummary::new(module)
        .with_version(SummaryVersion::EFFECT_ROW_PROVIDER_BINDINGS_V7)
        .with_exported_effect_row(row)
}

fn v8_structural_wire() -> serde_json::Value {
    let mut v8_wire = serde_json::to_value(v7_summary()).expect("serialize V7 fixture");
    v8_wire["version"] = serde_json::json!(8);
    v8_wire["exported_effect_rows"][0]["row_items"] = serde_json::json!([
        {
            "kind": "operation",
            "impl_type": "TestClock",
            "interface": "Clock",
            "operation": "sleep"
        },
        { "kind": "evidence", "path": ["response", "proved"] },
        { "kind": "tail", "variable": "rest" }
    ]);
    v8_wire
}

#[test]
fn v8_round_trips_structural_operation_evidence_and_open_tail_with_provider_binding() {
    let v8_wire = v8_structural_wire();

    let summary: ModuleSemanticSummary = serde_json::from_value(v8_wire.clone())
        .expect("V8 must decode structural provider/binding row content");
    summary
        .validate_summary_version_contract()
        .expect("V8 structural provider/binding rows must be eligible semantic summaries");

    assert_eq!(
        serde_json::to_value(&summary).expect("re-serialize V8 summary"),
        v8_wire,
        "V8 must preserve operation identity, evidence path, open tail, and the provider/binding envelope exactly"
    );
}

#[test]
fn v8_rejects_unknown_structural_item_fields_at_the_summary_schema_boundary() {
    let mut v8_wire = v8_structural_wire();
    v8_wire["exported_effect_rows"][0]["row_items"][0]["unexpected"] =
        serde_json::json!("must-not-be-ignored");

    assert!(
        serde_json::from_value::<ModuleSemanticSummary>(v8_wire).is_err(),
        "a same-version V8 structural item with an unknown field must reject instead of silently dropping wire content"
    );
}

#[test]
fn v7_rejects_structural_item_payloads_at_the_summary_schema_boundary() {
    let mut v7_wire = v8_structural_wire();
    v7_wire["version"] = serde_json::json!(7);
    let summary: ModuleSemanticSummary = serde_json::from_value(v7_wire)
        .expect("the version boundary, not deserialization, owns the V7 payload rejection");

    assert!(
        summary.validate_summary_version_contract().is_err(),
        "V7 must reject a structural item payload; only V8 may carry typed-handler row content"
    );
}

#[test]
fn v8_round_trips_unresolved_qualified_operation_as_non_dependency_metadata() {
    let mut wire = v8_structural_wire();
    wire["exported_effect_rows"][0]["row_items"] = serde_json::json!([
        {
            "kind": "symbolic_operation",
            "impl_type": "PosixFs",
            "operation": "read"
        }
    ]);

    let summary: ModuleSemanticSummary =
        serde_json::from_value(wire.clone()).expect("V8 symbolic operation must decode");
    summary
        .validate_summary_version_contract()
        .expect("symbolic operation metadata remains a valid non-granting V8 row item");
    assert_eq!(
        summary.exported_effect_rows[0].row_items[0].text,
        "PosixFs::read"
    );
    assert_eq!(
        serde_json::to_value(summary).expect("re-serialize symbolic V8 summary"),
        wire
    );
}
