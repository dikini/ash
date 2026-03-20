use ash_core::Effect;
use ash_parser::surface::Workflow;
use ash_parser::token::Span;
use ash_typeck::runtime_verification::{
    AggregateVerificationInputs, CapabilitySchema, CapabilitySchemaRegistry,
    ObligationRequirements, Role, RuntimeContext, RuntimeObligations, StaticPolicy,
    VerificationAggregator, VerificationError, WorkflowCapabilities,
};

fn valid_workflow() -> Workflow {
    Workflow::Done {
        span: Span::default(),
    }
}

#[test]
fn aggregate_reports_missing_required_capabilities_from_runtime_obligations() {
    let workflow = valid_workflow();
    let inputs = AggregateVerificationInputs::new(
        WorkflowCapabilities::new().observe("sensor", "temp"),
        ObligationRequirements::new().require_observe("sensor", "temp"),
    );

    let mut capabilities = CapabilitySchemaRegistry::new();
    capabilities.register(
        "sensor".into(),
        "temp".into(),
        CapabilitySchema::read_only(ash_typeck::Type::Int),
    );

    let runtime = RuntimeContext::new(Effect::Operational).with_capabilities(capabilities);

    let result = VerificationAggregator::new().aggregate(&workflow, &inputs, &runtime);

    assert!(
        result.errors.iter().any(|error| matches!(
            error,
            VerificationError::MissingRequiredCapability { required }
            if required == "sensor:temp"
        )),
        "aggregate verification should surface missing obligation-backed capabilities"
    );
}

#[test]
fn aggregate_accepts_required_capabilities_provided_by_runtime_obligations() {
    let workflow = valid_workflow();
    let inputs = AggregateVerificationInputs::new(
        WorkflowCapabilities::new().observe("sensor", "temp"),
        ObligationRequirements::new().require_observe("sensor", "temp"),
    );

    let mut capabilities = CapabilitySchemaRegistry::new();
    capabilities.register(
        "sensor".into(),
        "temp".into(),
        CapabilitySchema::read_only(ash_typeck::Type::Int),
    );

    let runtime = RuntimeContext::new(Effect::Operational)
        .with_capabilities(capabilities)
        .with_obligations(
            RuntimeObligations::new().with_obligation(
                ash_typeck::runtime_verification::Obligation::new("monitor", "read temp")
                    .with_observe_capability("sensor", "temp"),
            ),
        );

    let result = VerificationAggregator::new().aggregate(&workflow, &inputs, &runtime);

    assert!(
        result.errors.is_empty(),
        "required capabilities satisfied by runtime obligations should pass"
    );
}

#[test]
fn runtime_context_tracks_canonical_runtime_verification_fields() {
    let context = RuntimeContext::new(Effect::Operational)
        .with_policies(vec![StaticPolicy::new("gate")])
        .with_role(Role::new("operator"));

    assert_eq!(context.max_effect, Effect::Operational);
    assert_eq!(context.policies.len(), 1);
    assert_eq!(context.role.as_ref().map(Role::as_str), Some("operator"));
    assert!(context.mailboxes.is_empty());
    assert!(context.approval_queue.is_empty());
}
