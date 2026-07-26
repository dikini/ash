//! TASK-2008: canonical terminal-observable JSON projection tests.
//!
//! The existing `_variant` object tag is a compatibility serialization for
//! values. Canonical terminal outcomes must use a separate, stable envelope
//! and must not expose trace/session/runtime-artifact internals.

use ash_cli::value_convert::{
    CanonicalTerminalObservable, canonical_terminal_observable_to_json, value_to_json,
};
use ash_core::Value;
use serde_json::json;

fn ok_variant(value: i64) -> Value {
    Value::Variant {
        name: "Ok".to_string(),
        fields: Box::new(vec![("value".to_string(), Value::Int(value))]),
    }
}

#[test]
fn legacy_value_serialization_keeps_variant_tag_for_compatibility() {
    assert_eq!(
        value_to_json(&ok_variant(7)),
        json!({"_variant": "Ok", "value": 7})
    );
}

#[test]
fn canonical_terminal_schema_version_is_explicit_without_versioning_legacy_values() {
    let terminal = canonical_terminal_observable_to_json(&CanonicalTerminalObservable::Return {
        value: Value::Int(7),
    });

    assert_eq!(terminal["schema_version"], json!(1));
    assert!(
        value_to_json(&ok_variant(7))
            .get("schema_version")
            .is_none(),
        "legacy direct-value JSON must retain its existing wire shape"
    );
}

#[test]
fn canonical_return_envelope_does_not_reuse_legacy_variant_tag() {
    let projection = canonical_terminal_observable_to_json(&CanonicalTerminalObservable::Return {
        value: ok_variant(7),
    });

    assert_eq!(
        projection,
        json!({
            "schema_version": 1,
            "kind": "return",
            "value": {
                "constructor": "Ok",
                "fields": {"value": 7}
            }
        })
    );
    assert!(
        projection.pointer("/value/_variant").is_none(),
        "canonical terminal observables must not leak the legacy `_variant` tag"
    );
}

#[test]
fn canonical_trap_and_pre_entry_failure_have_distinct_boundary_kinds() {
    let trap = canonical_terminal_observable_to_json(&CanonicalTerminalObservable::Trap {
        reason: "declared primitive-domain failure".to_string(),
    });
    let pre_entry =
        canonical_terminal_observable_to_json(&CanonicalTerminalObservable::PreEntryFailure {
            class: "verification".to_string(),
            message: "entry contract rejected".to_string(),
        });

    assert_eq!(
        trap,
        json!({
            "schema_version": 1,
            "kind": "trap",
            "reason": "declared primitive-domain failure"
        })
    );
    assert_eq!(
        pre_entry,
        json!({
            "schema_version": 1,
            "kind": "pre_entry_failure",
            "class": "verification",
            "message": "entry contract rejected"
        })
    );
}

#[test]
fn canonical_external_outcome_is_bounded_and_excludes_runtime_telemetry() {
    let projection =
        canonical_terminal_observable_to_json(&CanonicalTerminalObservable::External {
            boundary: "provider".to_string(),
            outcome: "timeout".to_string(),
        });

    assert_eq!(
        projection,
        json!({
            "schema_version": 1,
            "kind": "external",
            "boundary": "provider",
            "outcome": "timeout"
        })
    );
    for forbidden in ["trace", "session", "runtime_artifact", "instance_id"] {
        assert!(
            projection.get(forbidden).is_none(),
            "canonical terminal output must exclude `{forbidden}` telemetry"
        );
    }
}
