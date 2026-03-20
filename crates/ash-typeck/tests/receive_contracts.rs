use ash_parser::surface::{Pattern, ReceiveArm, ReceiveMode, StreamPattern, Workflow};
use ash_parser::token::Span;
use ash_typeck::capability_check::{CapabilityCheckError, CapabilityChecker};

fn span() -> Span {
    Span::default()
}

fn stream_arm(capability: &str, channel: &str) -> ReceiveArm {
    ReceiveArm {
        pattern: StreamPattern::Binding {
            capability: capability.into(),
            channel: channel.into(),
            pattern: Pattern::Variable("msg".into()),
        },
        guard: None,
        body: Workflow::Done { span: span() },
        span: span(),
    }
}

#[test]
fn receive_requires_declared_stream_binding() {
    let workflow = Workflow::Receive {
        mode: ReceiveMode::NonBlocking,
        arms: vec![stream_arm("sensor", "temp")],
        is_control: false,
        span: span(),
    };

    let error = CapabilityChecker::new()
        .verify(&workflow)
        .expect_err("receive should require a declared stream binding");

    assert!(matches!(
        error,
        CapabilityCheckError::NotDeclared {
            operation,
            capability,
            channel,
        } if operation == "receive" && capability == "sensor" && channel == "temp"
    ));
}

#[test]
fn receive_accepts_declared_stream_binding() {
    let workflow = Workflow::Receive {
        mode: ReceiveMode::NonBlocking,
        arms: vec![stream_arm("sensor", "temp")],
        is_control: false,
        span: span(),
    };

    assert!(
        CapabilityChecker::new()
            .receive("sensor", "temp")
            .verify(&workflow)
            .is_ok()
    );
}

#[test]
fn control_receive_literals_do_not_require_stream_declarations() {
    let workflow = Workflow::Receive {
        mode: ReceiveMode::NonBlocking,
        arms: vec![ReceiveArm {
            pattern: StreamPattern::Literal("shutdown".to_string()),
            guard: None,
            body: Workflow::Done { span: span() },
            span: span(),
        }],
        is_control: true,
        span: span(),
    };

    assert!(CapabilityChecker::new().verify(&workflow).is_ok());
}

#[test]
fn receive_requires_all_stream_bindings_to_be_declared() {
    let workflow = Workflow::Receive {
        mode: ReceiveMode::NonBlocking,
        arms: vec![stream_arm("sensor", "temp"), stream_arm("queue", "events")],
        is_control: false,
        span: span(),
    };

    let error = CapabilityChecker::new()
        .receive("sensor", "temp")
        .verify(&workflow)
        .expect_err("all receive stream bindings must be declared");

    assert!(matches!(
        error,
        CapabilityCheckError::NotDeclared {
            operation,
            capability,
            channel,
        } if operation == "receive" && capability == "queue" && channel == "events"
    ));
}
