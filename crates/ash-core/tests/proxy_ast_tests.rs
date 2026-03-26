//! Tests for ProxyDef AST structures (TASK-239)

use ash_core::ast::{
    Capability, CorrelationId, Expr, InputCapability, ModuleItem, Pattern, ProxyDef, ReceiveMode,
    Role, Span, Workflow, WorkflowDef,
};
use ash_core::effect::Effect;
use ash_core::workflow_contract::TypeExpr;

/// Create a test span
fn test_span() -> Span {
    Span { start: 0, end: 10 }
}

/// Create a test role
fn test_role(name: &str) -> Role {
    Role {
        name: name.to_string(),
        authority: vec![],
        obligations: vec![],
    }
}

/// Create a test capability
fn test_capability(name: &str) -> Capability {
    Capability {
        name: name.to_string(),
        effect: Effect::Epistemic,
        constraints: vec![],
    }
}

#[test]
fn test_input_capability_creation() {
    // Test observes capability
    let observes = InputCapability::Observes {
        capability: test_capability("sensor"),
        channel: Some("temperature".to_string()),
    };
    match observes {
        InputCapability::Observes {
            capability,
            channel,
        } => {
            assert_eq!(capability.name, "sensor");
            assert_eq!(channel, Some("temperature".to_string()));
        }
        _ => panic!("expected Observes variant"),
    }

    // Test receives capability
    let receives = InputCapability::Receives {
        capability: test_capability("mailbox"),
        channel: Some("requests".to_string()),
    };
    match receives {
        InputCapability::Receives {
            capability,
            channel,
        } => {
            assert_eq!(capability.name, "mailbox");
            assert_eq!(channel, Some("requests".to_string()));
        }
        _ => panic!("expected Receives variant"),
    }
}

#[test]
fn test_proxy_def_construction() {
    let proxy_def = ProxyDef {
        name: "manager_proxy".to_string(),
        handled_role: test_role("manager"),
        inputs: vec![InputCapability::Receives {
            capability: test_capability("requests"),
            channel: Some("approval_request".to_string()),
        }],
        body: Workflow::Done,
        span: test_span(),
    };

    assert_eq!(proxy_def.name, "manager_proxy");
    assert_eq!(proxy_def.handled_role.name, "manager");
    assert_eq!(proxy_def.inputs.len(), 1);
    assert!(matches!(proxy_def.body, Workflow::Done));
}

#[test]
fn test_proxy_def_with_multiple_inputs() {
    let proxy_def = ProxyDef {
        name: "board_proxy".to_string(),
        handled_role: test_role("board_members"),
        inputs: vec![
            InputCapability::Receives {
                capability: test_capability("requests"),
                channel: Some("board_decision".to_string()),
            },
            InputCapability::Observes {
                capability: test_capability("status"),
                channel: Some("meeting_status".to_string()),
            },
        ],
        body: Workflow::Receive {
            mode: ReceiveMode::Blocking(None),
            arms: vec![],
            control: false,
        },
        span: test_span(),
    };

    assert_eq!(proxy_def.inputs.len(), 2);
}

#[test]
fn test_module_item_proxy_variant() {
    let proxy_def = ProxyDef {
        name: "test_proxy".to_string(),
        handled_role: test_role("test_role"),
        inputs: vec![],
        body: Workflow::Done,
        span: test_span(),
    };

    let module_item = ModuleItem::Proxy(proxy_def);

    match module_item {
        ModuleItem::Proxy(proxy) => {
            assert_eq!(proxy.name, "test_proxy");
        }
        _ => panic!("expected Proxy variant"),
    }
}

#[test]
fn test_module_item_workflow_variant() {
    let workflow_def = WorkflowDef {
        name: "test_workflow".to_string(),
        params: vec![],
        body: Workflow::Done,
        export: false,
        contract: None,
        span: test_span(),
    };

    let module_item = ModuleItem::Workflow(workflow_def);

    match module_item {
        ModuleItem::Workflow(wf) => {
            assert_eq!(wf.name, "test_workflow");
        }
        _ => panic!("expected Workflow variant"),
    }
}

#[test]
fn test_workflow_yield_construction() {
    // Create request expression
    let request = Box::new(Expr::Literal(ash_core::Value::String(
        "test_request".to_string(),
    )));

    // Create continuation workflow
    let continuation = Box::new(Workflow::Done);

    // Create expected response type
    let expected_response_type = TypeExpr::Named("TransferResponse".to_string());

    // Construct Yield workflow
    let yield_workflow = Workflow::Yield {
        role: "manager".to_string(),
        request,
        expected_response_type,
        continuation,
        span: test_span(),
        resume_var: "response".to_string(),
    };

    match yield_workflow {
        Workflow::Yield {
            role,
            request: _,
            expected_response_type,
            continuation,
            span,
            resume_var,
        } => {
            assert_eq!(resume_var, "response");
            assert_eq!(role, "manager");
            assert!(matches!(*continuation, Workflow::Done));
            assert_eq!(span, test_span());
            match expected_response_type {
                TypeExpr::Named(name) => assert_eq!(name, "TransferResponse"),
                _ => panic!("expected Named type"),
            }
        }
        _ => panic!("expected Yield variant"),
    }
}

#[test]
fn test_workflow_proxy_resume_construction() {
    // Create response value expression
    let value = Box::new(Expr::Literal(ash_core::Value::String(
        "test_response".to_string(),
    )));

    // Create value type
    let value_type = TypeExpr::Named("TransferResponse".to_string());

    // Create correlation ID
    let correlation_id = CorrelationId::new(12345);

    // Construct ProxyResume workflow
    let resume_workflow = Workflow::ProxyResume {
        value,
        value_type,
        correlation_id,
        span: test_span(),
    };

    match resume_workflow {
        Workflow::ProxyResume {
            value,
            value_type,
            correlation_id: _,
            span,
        } => {
            assert!(matches!(*value, Expr::Literal(_)));
            match value_type {
                TypeExpr::Named(name) => assert_eq!(name, "TransferResponse"),
                _ => panic!("expected Named type"),
            }
            assert_eq!(span, test_span());
        }
        _ => panic!("expected ProxyResume variant"),
    }
}

#[test]
fn test_correlation_id_creation() {
    let cid1 = CorrelationId::new(12345);
    let cid2 = CorrelationId::new(12345);
    let cid3 = CorrelationId::new(99999);

    assert_eq!(cid1, cid2);
    assert_ne!(cid1, cid3);
}

#[test]
fn test_proxy_def_clone() {
    let proxy_def = ProxyDef {
        name: "manager_proxy".to_string(),
        handled_role: test_role("manager"),
        inputs: vec![InputCapability::Receives {
            capability: test_capability("requests"),
            channel: Some("approval".to_string()),
        }],
        body: Workflow::Done,
        span: test_span(),
    };

    let cloned = proxy_def.clone();
    assert_eq!(cloned.name, proxy_def.name);
    assert_eq!(cloned.handled_role.name, proxy_def.handled_role.name);
}

#[test]
fn test_proxy_def_debug() {
    let proxy_def = ProxyDef {
        name: "test_proxy".to_string(),
        handled_role: test_role("test_role"),
        inputs: vec![],
        body: Workflow::Done,
        span: test_span(),
    };

    let debug_str = format!("{:?}", proxy_def);
    assert!(debug_str.contains("test_proxy"));
    assert!(debug_str.contains("test_role"));
}

#[test]
fn test_input_capability_equality() {
    let cap1 = InputCapability::Observes {
        capability: test_capability("sensor"),
        channel: Some("temp".to_string()),
    };
    let cap2 = InputCapability::Observes {
        capability: test_capability("sensor"),
        channel: Some("temp".to_string()),
    };
    let cap3 = InputCapability::Observes {
        capability: test_capability("sensor"),
        channel: Some("humidity".to_string()),
    };

    assert_eq!(cap1, cap2);
    assert_ne!(cap1, cap3);
}

#[test]
fn test_yield_workflow_with_complex_continuation() {
    // Create a more complex continuation
    let continuation = Box::new(Workflow::Let {
        pattern: Pattern::Variable("response".to_string()),
        expr: Expr::Literal(ash_core::Value::Int(42)),
        continuation: Box::new(Workflow::Done),
    });

    let yield_workflow = Workflow::Yield {
        role: "manager".to_string(),
        request: Box::new(Expr::Literal(ash_core::Value::Null)),
        expected_response_type: TypeExpr::Named("Response".to_string()),
        continuation,
        span: test_span(),
        resume_var: "response".to_string(),
    };

    match yield_workflow {
        Workflow::Yield { continuation, .. } => {
            assert!(matches!(*continuation, Workflow::Let { .. }));
        }
        _ => panic!("expected Yield variant"),
    }
}

#[test]
fn test_module_item_roundtrip_serde() {
    use ash_core::ast::{ModuleItem, ProxyDef};

    let proxy_def = ProxyDef {
        name: "test_proxy".to_string(),
        handled_role: test_role("test_role"),
        inputs: vec![],
        body: Workflow::Done,
        span: test_span(),
    };

    let module_item = ModuleItem::Proxy(proxy_def);

    // Serialize and deserialize
    let serialized = serde_json::to_string(&module_item).expect("should serialize");
    let deserialized: ModuleItem = serde_json::from_str(&serialized).expect("should deserialize");

    match (module_item, deserialized) {
        (ModuleItem::Proxy(orig), ModuleItem::Proxy(loaded)) => {
            assert_eq!(orig.name, loaded.name);
            assert_eq!(orig.handled_role.name, loaded.handled_role.name);
        }
        _ => panic!("roundtrip failed"),
    }
}
