//! TASK-743 engine configuration-surface regression tests.

use ash_engine::{Engine, EngineError};

const MODULE_SOURCE: &str = r#"
pub capability interface KeyValue:
    observe get(key: String) returns String;

pub resource type WorkflowKV {
    namespace: String
}

pub capability impl MockInternalKV for KeyValue
    requires resource store: WorkflowKV
    requires config fixture: String
{
    observe get(key: String) returns String { "mock" }
    execute put(key: String, value: String) returns Unit { () }
}
"#;

#[test]
fn builder_records_capability_implementation_selection() {
    let engine = Engine::new()
        .with_capability_implementation("kv", "MockInternalKV")
        .build()
        .expect("engine builds");

    assert_eq!(
        engine.capability_implementation_selection("kv"),
        Some("MockInternalKV")
    );
    assert_eq!(engine.capability_implementation_selection_count(), 1);
}

#[test]
fn builder_records_resource_initializer_selection() {
    let engine = Engine::new()
        .with_resource_initializer("WorkflowKV", "memory")
        .build()
        .expect("engine builds");

    assert_eq!(
        engine.resource_initializer_selection("WorkflowKV"),
        Some("memory")
    );
    assert_eq!(engine.resource_initializer_selection_count(), 1);
}

#[test]
fn builder_rejects_duplicate_capability_implementation_selection() {
    let error = Engine::new()
        .with_capability_implementation("kv", "MockInternalKV")
        .with_capability_implementation("kv", "OtherKV")
        .build()
        .expect_err("duplicate selection should be rejected");

    assert!(matches!(error, EngineError::Configuration(_)));
    assert!(format!("{error}").contains("duplicate capability implementation selection"));
}

#[test]
fn builder_rejects_duplicate_resource_initializer_selection() {
    let error = Engine::new()
        .with_resource_initializer("WorkflowKV", "memory")
        .with_resource_initializer("WorkflowKV", "disk")
        .build()
        .expect_err("duplicate selection should be rejected");

    assert!(matches!(error, EngineError::Configuration(_)));
    assert!(format!("{error}").contains("duplicate resource initializer selection"));
}

#[test]
fn configuration_preserves_default_provider_wiring() {
    let engine = Engine::new()
        .with_stdio_capabilities()
        .with_fs_capabilities()
        .with_capability_implementation("kv", "MockInternalKV")
        .with_resource_initializer("WorkflowKV", "memory")
        .build()
        .expect("engine builds");

    assert!(engine.has_provider("stdio"));
    assert!(engine.has_provider("fs"));
    assert!(engine.has_provider("dir"));
    assert!(engine.has_provider("meta"));
}

#[test]
fn known_configuration_validates_against_module_source() {
    let engine = Engine::new()
        .with_capability_implementation("kv", "MockInternalKV")
        .with_resource_initializer("WorkflowKV", "memory")
        .build()
        .expect("engine builds");

    engine
        .validate_configuration_for_source(MODULE_SOURCE)
        .expect("known names validate");
}

#[test]
fn unknown_implementation_selection_reports_name() {
    let engine = Engine::new()
        .with_capability_implementation("kv", "MissingKV")
        .build()
        .expect("engine builds");

    let error = engine
        .validate_configuration_for_source(MODULE_SOURCE)
        .expect_err("unknown implementation should fail");

    assert!(matches!(error, EngineError::Configuration(_)));
    assert!(format!("{error}").contains("unknown capability implementation 'MissingKV'"));
}

#[test]
fn unknown_resource_initializer_reports_name() {
    let engine = Engine::new()
        .with_resource_initializer("MissingResource", "memory")
        .build()
        .expect("engine builds");

    let error = engine
        .validate_configuration_for_source(MODULE_SOURCE)
        .expect_err("unknown resource should fail");

    assert!(matches!(error, EngineError::Configuration(_)));
    assert!(format!("{error}").contains("unknown resource initializer target 'MissingResource'"));
}
