use ash_core::{Observe, Pattern, Value};
use ash_runtime::CapabilityOperation;
use ash_runtime::CapabilityPolicy;
use ash_runtime::CapabilityPolicyEvaluator;
use ash_runtime::Direction;
use ash_runtime::ExecError;
use ash_runtime::PolicyDecision;
use ash_runtime::Role;
use ash_runtime::Transformation;
use ash_runtime::behaviour::{BehaviourContext, MockBehaviourProvider, MockSettableProvider};
use ash_runtime::exec_send::execute_send;
use ash_runtime::execute_observe::execute_observe;
use ash_runtime::execute_set::execute_set;
use ash_runtime::stream::{MockSendableProvider, StreamContext, TypedSendableProvider};
use ash_runtime::typed_provider::TypedBehaviourProvider;
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
        pattern: Pattern::Variable {
            name: "reading".into(),
            span: ash_core::ast::Span::default(),
        },
    };

    let result = execute_observe(
        &observe,
        ash_runtime::context::Context::new(),
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
    behaviour_ctx.register_settable(ash_runtime::behaviour::TypedSettableProvider::new(
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
async fn approval_role_remains_an_explicit_flat_name() {
    let mut behaviour_ctx = BehaviourContext::new();
    let provider = MockSettableProvider::new("repo", "merge");
    behaviour_ctx.register_settable(ash_runtime::behaviour::TypedSettableProvider::new(
        provider,
        Type::Bool,
    ));

    let mut policy_eval = CapabilityPolicyEvaluator::new();
    policy_eval.add_output_policy(CapabilityPolicy {
        capability_pattern: "repo:merge".into(),
        condition: Box::new(|_| true),
        decision: Box::new(|_| PolicyDecision::RequireApproval {
            role: Role::new("senior_reviewer"),
        }),
    });

    let actor = Role::new("reviewer_delegate");
    let error = execute_set(
        "repo",
        "merge",
        Value::Bool(true),
        &behaviour_ctx,
        &policy_eval,
        &actor,
    )
    .await
    .expect_err("approval should remain distinct from the acting role");

    assert_ne!(actor.as_ref(), "senior_reviewer");
    assert_eq!(
        error,
        ExecError::RequiresApproval {
            role: "senior_reviewer".into(),
            operation: "set".into(),
            capability: "repo:merge".into(),
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
