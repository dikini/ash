use ash_core::{Observe, Pattern, Value};
use ash_interp::CapabilityOperation;
use ash_interp::CapabilityPolicy;
use ash_interp::CapabilityPolicyEvaluator;
use ash_interp::Direction;
use ash_interp::ExecError;
use ash_interp::PolicyDecision;
use ash_interp::Role;
use ash_interp::Transformation;
use ash_interp::behaviour::{BehaviourContext, MockBehaviourProvider, MockSettableProvider};
use ash_interp::exec_send::execute_send;
use ash_interp::execute_observe::execute_observe;
use ash_interp::execute_set::execute_set;
use ash_interp::stream::{MockSendableProvider, StreamContext, TypedSendableProvider};
use ash_interp::typed_provider::TypedBehaviourProvider;
use ash_typeck::Type;
#[tokio::test]
async fn observe_policy_transform_masks_selected_fields_before_binding() {
    let mut behaviour_ctx = BehaviourContext::new();
    let record_type = Type::Record(vec![
        (Box::from("secret"), Type::Int),
        (Box::from("visible"), Type::Int),
    ]);
    let mut record = std::collections::HashMap::new();
    record.insert("secret".into(), Value::Int(42));
    record.insert("visible".into(), Value::Int(7));
    behaviour_ctx.register(TypedBehaviourProvider::new(
        MockBehaviourProvider::new("sensor", "temp").with_value(Value::Record(Box::new(record))),
        record_type,
    ));

    let mut policy_eval = CapabilityPolicyEvaluator::new();
    policy_eval.add_input_policy(CapabilityPolicy {
        capability_pattern: "sensor:temp".into(),
        condition: Box::new(|ctx| {
            ctx.operation == CapabilityOperation::Observe
                && ctx.direction == Direction::Input
                && ctx.capability == "sensor"
                && ctx.channel == "temp"
                && ctx.value.is_none()
        }),
        decision: Box::new(|_| PolicyDecision::Transform {
            transformation: Transformation::Mask {
                fields: vec!["secret".into()],
            },
        }),
    });

    let observe = Observe {
        capability: "sensor".into(),
        channel: "temp".into(),
        constraints: vec![],
        pattern: Pattern::Variable("reading".into()),
    };

    let result = execute_observe(
        &observe,
        ash_interp::context::Context::new(),
        &behaviour_ctx,
        &policy_eval,
        &Role::new("operator"),
    )
    .await
    .expect("transform should still permit observe");

    let bound = result.get("reading").expect("observe should bind reading");
    let Value::Record(record) = bound else {
        panic!("expected transformed record value, got {bound:?}");
    };
    assert_eq!(record.get("secret"), Some(&Value::Null));
    assert_eq!(record.get("visible"), Some(&Value::Int(7)));
}

#[tokio::test]
async fn set_policy_requires_approval_surfaces_a_distinct_runtime_state() {
    let mut behaviour_ctx = BehaviourContext::new();
    let provider = MockSettableProvider::new("hvac", "target");
    behaviour_ctx.register_settable(ash_interp::behaviour::TypedSettableProvider::new(
        provider,
        Type::Int,
    ));

    let mut policy_eval = CapabilityPolicyEvaluator::new();
    policy_eval.add_output_policy(CapabilityPolicy {
        capability_pattern: "hvac:target".into(),
        condition: Box::new(|_| true),
        decision: Box::new(|_| PolicyDecision::RequireApproval {
            role: Role::new("admin"),
        }),
    });

    let error = execute_set(
        "hvac",
        "target",
        Value::Int(72),
        &behaviour_ctx,
        &policy_eval,
        &Role::new("operator"),
    )
    .await
    .expect_err("approval should be surfaced distinctly");

    assert_eq!(
        error,
        ExecError::RequiresApproval {
            role: "admin".into(),
            operation: "set".into(),
            capability: "hvac:target".into(),
        }
    );
}

#[tokio::test]
async fn send_policy_transform_rewrites_the_value_before_send() {
    let mut stream_ctx = StreamContext::new();
    let provider = MockSendableProvider::new("alert", "critical");
    stream_ctx.register_sendable(TypedSendableProvider::new(provider.clone(), Type::Int));

    let mut policy_eval = CapabilityPolicyEvaluator::new();
    policy_eval.add_output_policy(CapabilityPolicy {
        capability_pattern: "alert:critical".into(),
        condition: Box::new(|_| true),
        decision: Box::new(|_| PolicyDecision::Transform {
            transformation: Transformation::Replace {
                value: Value::Int(0),
            },
        }),
    });

    execute_send(
        "alert",
        "critical",
        Value::Int(5),
        &stream_ctx,
        &policy_eval,
        &Role::new("operator"),
    )
    .await
    .expect("transform should rewrite and send");

    assert_eq!(provider.sent_values(), vec![Value::Int(0)]);
}
