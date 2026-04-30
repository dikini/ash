use ash_core::{
    Value,
    workflow_carrier::{ActLowerSummary, ProcLowerSummary, WorkflowNodeId, WorkflowProcProjection},
};
use ash_interp::{
    ExecError, execute_workflow_proc_projection, unsupported_workflow_proc_projection_message,
};

#[test]
fn unit_projection_executes_without_parser_or_typeck_artifacts() {
    let projection = WorkflowProcProjection::Unit {
        node: WorkflowNodeId(1),
        value: Value::Int(42),
    };

    let value = execute_workflow_proc_projection(&projection).expect("unit projection should run");

    assert_eq!(value, Value::Int(42));
}

#[test]
fn ignored_bind_projection_executes_non_dependent_then_sequence() {
    let projection = WorkflowProcProjection::Bind {
        node: WorkflowNodeId(6),
        source: Box::new(WorkflowProcProjection::Unit {
            node: WorkflowNodeId(1),
            value: Value::Int(1),
        }),
        binder: ash_core::workflow_carrier::WorkflowBinder::Ignored,
        next: Box::new(WorkflowProcProjection::Unit {
            node: WorkflowNodeId(2),
            value: Value::Int(2),
        }),
    };

    let value = execute_workflow_proc_projection(&projection)
        .expect("ignored bind / then projection should run");

    assert_eq!(value, Value::Int(2));
}

#[test]
fn named_bind_projection_fails_at_named_phase_108_boundary() {
    let projection = WorkflowProcProjection::Bind {
        node: WorkflowNodeId(3),
        source: Box::new(WorkflowProcProjection::Unit {
            node: WorkflowNodeId(1),
            value: Value::Int(1),
        }),
        binder: ash_core::workflow_carrier::WorkflowBinder::Named("x".to_string()),
        next: Box::new(WorkflowProcProjection::Unit {
            node: WorkflowNodeId(2),
            value: Value::Int(2),
        }),
    };

    let error = execute_workflow_proc_projection(&projection).unwrap_err();

    assert!(matches!(error, ExecError::InvalidRuntimeState(_)));
    let message = error.to_string();
    assert!(message.contains("FirstClassWorkflowProjectionExecutionUnsupported"));
    assert!(message.contains("Phase 108 is still check/lowering-only"));
    assert!(message.contains("bind"));
    assert!(message.contains("node 3"));
}

#[test]
fn unsupported_lift_projections_fail_at_named_phase_108_boundary() {
    let cases = [
        (
            WorkflowProcProjection::FromProc {
                node: WorkflowNodeId(4),
                summary: ProcLowerSummary {
                    coverage_obligation_nodes: vec![WorkflowNodeId(4)],
                    contract_summary: None,
                },
            },
            "from_proc",
            "node 4",
        ),
        (
            WorkflowProcProjection::FromAct {
                node: WorkflowNodeId(5),
                summary: ActLowerSummary {
                    coverage_obligation_nodes: vec![WorkflowNodeId(5)],
                    contract_summary: None,
                },
            },
            "from_act",
            "node 5",
        ),
    ];

    for (projection, shape, node) in cases {
        let error = execute_workflow_proc_projection(&projection).unwrap_err();
        let message = error.to_string();
        assert!(message.contains("FirstClassWorkflowProjectionExecutionUnsupported"));
        assert!(message.contains(shape));
        assert!(message.contains(node));
    }
}

#[test]
fn transparent_scope_projection_executes_body() {
    let projection = WorkflowProcProjection::Scope {
        node: WorkflowNodeId(8),
        scope: ash_core::workflow_carrier::WorkflowScope {
            name: Some("local".to_string()),
            origin: ash_core::workflow_carrier::SourceOrigin::Synthetic {
                parent_span: None,
                reason: "test scope".to_string(),
            },
        },
        body: Box::new(WorkflowProcProjection::Unit {
            node: WorkflowNodeId(7),
            value: Value::String("ok".to_string()),
        }),
    };

    let value = execute_workflow_proc_projection(&projection).expect("scope unit should run");

    assert_eq!(value, Value::String("ok".to_string()));
}

#[test]
fn unsupported_message_names_boundary_and_shape() {
    let projection = WorkflowProcProjection::<Value>::Neutral {
        node: WorkflowNodeId(9),
    };

    let message = unsupported_workflow_proc_projection_message(&projection);

    assert!(message.contains("FirstClassWorkflowProjectionExecutionUnsupported"));
    assert!(message.contains("neutral governance node"));
    assert!(message.contains("node 9"));
}
