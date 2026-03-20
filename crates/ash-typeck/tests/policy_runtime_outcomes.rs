use std::time::Duration;

use ash_typeck::runtime_verification::{
    CapabilityOperation, CapabilitySchema, CapabilitySchemaRegistry, OperationResult,
    OperationVerifier, PolicyDecisionType, RateLimiter, Role, StaticPolicy, StaticPolicyValidator,
    VerificationError, VerificationWarning, WorkflowCapabilities,
};

#[test]
fn static_policy_validator_surfaces_approval_and_transform_as_warnings() {
    let workflow_capabilities = WorkflowCapabilities::new()
        .observe("sensor", "temp")
        .send("alert", "critical");
    let policies = vec![
        StaticPolicy::new("approve-temp")
            .applies_to(|cap, chan| cap == "sensor" && chan == "temp")
            .decision(PolicyDecisionType::RequiresApproval {
                role: Role::new("operator"),
            }),
        StaticPolicy::new("mask-alert")
            .applies_to(|cap, chan| cap == "alert" && chan == "critical")
            .decision(PolicyDecisionType::Transform {
                transformation: "mask(payload)".to_string(),
            }),
    ];

    let result = StaticPolicyValidator::new().validate(&workflow_capabilities, &policies);

    assert!(result.errors.is_empty());
    assert_eq!(
        result.warnings,
        vec![
            VerificationWarning::RequiresApproval {
                role: Role::new("operator"),
                operation: "observe sensor:temp".to_string(),
            },
            VerificationWarning::PolicyTransform {
                operation: "send alert:critical".to_string(),
                transformation: "mask(payload)".to_string(),
            },
        ]
    );
}

#[test]
fn static_policy_validator_still_rejects_denied_operations() {
    let workflow_capabilities = WorkflowCapabilities::new().set("hvac", "target");
    let policies = vec![
        StaticPolicy::new("deny-hvac")
            .applies_to(|cap, chan| cap == "hvac" && chan == "target")
            .decision(PolicyDecisionType::Deny),
    ];

    let result = StaticPolicyValidator::new().validate(&workflow_capabilities, &policies);

    assert_eq!(
        result.errors,
        vec![VerificationError::PolicyConflict {
            policy: "deny-hvac".to_string(),
            reason: "set on hvac:target is denied".to_string(),
        }]
    );
}

#[tokio::test]
async fn operation_verifier_returns_transform_as_a_distinct_runtime_outcome() {
    let mut registry = CapabilitySchemaRegistry::new();
    registry.register(
        "sensor".into(),
        "temp".into(),
        CapabilitySchema::read_only(ash_typeck::Type::Int),
    );
    let policies = vec![
        StaticPolicy::new("mask-temp")
            .applies_to(|cap, chan| cap == "sensor" && chan == "temp")
            .decision(PolicyDecisionType::Transform {
                transformation: "mask(value)".to_string(),
            }),
    ];
    let mut rate_limiter = RateLimiter::new(5, Duration::from_secs(1));

    let result = OperationVerifier::new()
        .verify(
            &CapabilityOperation::observe("sensor", "temp"),
            &registry,
            &policies,
            &mut rate_limiter,
        )
        .await
        .expect("transform should be a supported runtime outcome");

    assert_eq!(
        result,
        OperationResult::Transformed {
            transformation: "mask(value)".to_string(),
        }
    );
}
