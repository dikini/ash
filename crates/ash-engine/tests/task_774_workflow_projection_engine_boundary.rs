//! Engine-facing first-class Workflow projection boundary tests.

use ash_core::{
    Value,
    workflow_carrier::{WorkflowBinder, WorkflowNodeId, WorkflowProcProjection},
};
use ash_engine::Engine;

#[test]
fn engine_executes_first_class_workflow_projection_through_interp_boundary() {
    let engine = Engine::new().build().expect("engine builds");
    let projection = WorkflowProcProjection::Bind {
        node: WorkflowNodeId(10),
        source: Box::new(WorkflowProcProjection::Unit {
            node: WorkflowNodeId(11),
            value: Value::Int(1),
        }),
        binder: WorkflowBinder::Ignored,
        next: Box::new(WorkflowProcProjection::Unit {
            node: WorkflowNodeId(12),
            value: Value::String("engine-boundary".to_string()),
        }),
    };

    let value = engine
        .execute_workflow_proc_projection(&projection)
        .expect("engine projection boundary should forward supported ash-core carrier");

    assert_eq!(value, Value::String("engine-boundary".to_string()));
}

#[test]
fn engine_reports_named_unsupported_projection_diagnostic_from_interp_boundary() {
    let engine = Engine::new().build().expect("engine builds");
    let projection = WorkflowProcProjection::<Value>::Neutral {
        node: WorkflowNodeId(44),
    };

    let error = engine
        .execute_workflow_proc_projection(&projection)
        .expect_err("neutral projection remains unsupported in this slice");
    let message = error.to_string();

    assert!(
        message.contains("FirstClassWorkflowProjectionExecutionUnsupported"),
        "expected named unsupported diagnostic, got {message}"
    );
    assert!(
        message.contains("neutral governance node at node 44"),
        "expected interp projection shape/node diagnostic, got {message}"
    );
}
