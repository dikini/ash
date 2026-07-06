use std::collections::HashMap;
use std::sync::Arc;

use ash_core::{EnvFrame, Expr, ProcessHandle, SendabilityRejection, Value};

#[test]
fn owned_primitive_records_and_variants_are_sendable() {
    let mut record = HashMap::new();
    record.insert("count".to_string(), Value::Int(3));
    record.insert("label".to_string(), Value::String("ready".to_string()));

    let value = Value::variant(
        "Envelope",
        vec![
            ("payload", Value::Record(Box::new(record))),
            ("done", Value::Bool(false)),
        ],
    );

    assert!(value.validate_sendable_for_process_boundary().is_ok());
    assert!(value.is_sendable_across_process_boundary());
}

#[test]
fn closures_are_rejected_as_non_sendable_process_boundary_values() {
    let closure = Value::Closure {
        params: vec![],
        body: Box::new(Expr::Literal(Value::Null)),
        env: Arc::new(EnvFrame::new()),
    };

    assert_eq!(
        closure.validate_sendable_for_process_boundary(),
        Err(SendabilityRejection::Closure)
    );
    assert!(!closure.is_sendable_across_process_boundary());
}

#[test]
fn borrowed_resources_and_authority_carriers_report_nested_paths() {
    let mut inner = HashMap::new();
    inner.insert("borrowed".to_string(), Value::Ref("session".to_string()));

    let value = Value::variant(
        "Work",
        vec![
            ("meta", Value::String("x".to_string())),
            ("resources", Value::Record(Box::new(inner))),
        ],
    );

    assert_eq!(
        value.validate_sendable_for_process_boundary(),
        Err(SendabilityRejection::at_path(
            "resources.borrowed",
            SendabilityRejection::BorrowedResource
        ))
    );
}

#[test]
fn runtime_tokens_and_live_process_captures_are_rejected() {
    assert_eq!(
        Value::ActEnvToken.validate_sendable_for_process_boundary(),
        Err(SendabilityRejection::RuntimeToken("act-env"))
    );
    assert_eq!(
        Value::ProcYieldCapture.validate_sendable_for_process_boundary(),
        Err(SendabilityRejection::RuntimeToken("proc-yield"))
    );
}

#[test]
fn consumed_process_handles_are_rejected_but_unconsumed_handles_are_sendable() {
    let handle = ProcessHandle::new(ash_core::ProcessId::new(), Some("Int".to_string()));
    let process_id = handle.process_id;

    assert!(
        Value::ProcessHandle(handle.clone())
            .validate_sendable_for_process_boundary()
            .is_ok()
    );

    assert!(handle.try_consume());

    assert_eq!(
        Value::ProcessHandle(handle).validate_sendable_for_process_boundary(),
        Err(SendabilityRejection::ConsumedProcessHandle { process_id })
    );
}
