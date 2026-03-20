use ash_core::Effect;
use ash_parser::surface::Workflow;
use ash_parser::token::Span;
use ash_typeck::obligation_checker::RequiredCapabilities as DeclaredObligationRequirements;
use ash_typeck::runtime_verification::{
    AggregateVerificationInputs, CapabilitySchema, CapabilitySchemaRegistry, Obligation,
    ObligationRequirements, Role, RuntimeContext, RuntimeObligations, VerificationAggregator,
    VerificationError, WorkflowCapabilities,
};

fn valid_workflow() -> Workflow {
    Workflow::Done {
        span: Span::default(),
    }
}

fn runtime_with_read_capability(capability: &str, channel: &str) -> RuntimeContext {
    let mut capabilities = CapabilitySchemaRegistry::new();
    capabilities.register(
        capability.into(),
        channel.into(),
        CapabilitySchema::read_only(ash_typeck::Type::Int),
    );

    RuntimeContext::new(Effect::Operational).with_capabilities(capabilities)
}

#[test]
fn capability_availability_can_succeed_while_obligation_requirements_fail() {
    let workflow = valid_workflow();
    let inputs = AggregateVerificationInputs::new(
        WorkflowCapabilities::new().observe("sensor", "temp"),
        ObligationRequirements::new().require_observe("audit", "log"),
    );
    let runtime = runtime_with_read_capability("sensor", "temp");

    let result = VerificationAggregator::new().aggregate(&workflow, &inputs, &runtime);

    assert!(
        result.errors.iter().any(|error| matches!(
            error,
            VerificationError::MissingRequiredCapability { required } if required == "audit:log"
        )),
        "explicit obligation requirements should be enforced independently"
    );
    assert!(
        !result.errors.iter().any(|error| matches!(
            error,
            VerificationError::MissingCapability { capability, .. } if capability == "sensor:temp"
        )),
        "workflow capability availability should still succeed"
    );
}

#[test]
fn obligation_requirements_can_be_satisfied_without_workflow_capability_changes() {
    let workflow = valid_workflow();
    let declared = DeclaredObligationRequirements::new().require_observe("sensor", "temp");
    let inputs = AggregateVerificationInputs::new(
        WorkflowCapabilities::new(),
        ObligationRequirements::from_declared_requirements(&declared),
    );
    let runtime = RuntimeContext::new(Effect::Operational).with_obligations(
        RuntimeObligations::new().with_obligation(
            Obligation::new("monitor", "read temp").with_observe_capability("sensor", "temp"),
        ),
    );

    let result = VerificationAggregator::new().aggregate(&workflow, &inputs, &runtime);

    assert!(
        result.errors.is_empty(),
        "obligation-backed requirements should be satisfied independently of workflow declarations"
    );
}

#[test]
fn aggregate_verification_does_not_derive_obligation_requirements_from_workflow_capabilities() {
    let workflow = valid_workflow();
    let inputs = AggregateVerificationInputs::new(
        WorkflowCapabilities::new().observe("sensor", "temp"),
        ObligationRequirements::new(),
    );
    let runtime = runtime_with_read_capability("sensor", "temp");

    let result = VerificationAggregator::new().aggregate(&workflow, &inputs, &runtime);

    assert!(
        result.errors.is_empty(),
        "empty explicit obligation requirements should not be backfilled from workflow capabilities"
    );
    assert!(result.can_execute());
}

#[test]
fn role_and_obligation_requirements_are_checked_independently_of_workflow_capabilities() {
    let workflow = valid_workflow();
    let inputs = AggregateVerificationInputs::new(
        WorkflowCapabilities::new(),
        ObligationRequirements::new()
            .require_role(Role::new("operator"))
            .require_obligation("monitor"),
    );
    let runtime = RuntimeContext::new(Effect::Operational);

    let result = VerificationAggregator::new().aggregate(&workflow, &inputs, &runtime);

    assert!(
        result
            .errors
            .iter()
            .any(|error| matches!(error, VerificationError::RoleMismatch { .. }))
    );
    assert!(result.errors.iter().any(
        |error| matches!(error, VerificationError::MissingObligation(name) if name == "monitor")
    ));
}

#[test]
fn role_and_obligation_requirements_can_be_satisfied_without_workflow_capability_changes() {
    let workflow = valid_workflow();
    let inputs = AggregateVerificationInputs::new(
        WorkflowCapabilities::new(),
        ObligationRequirements::new()
            .require_role(Role::new("operator"))
            .require_obligation("monitor"),
    );
    let runtime = RuntimeContext::new(Effect::Operational)
        .with_role(Role::new("operator"))
        .with_obligations(
            RuntimeObligations::new()
                .with_obligation(Obligation::new("monitor", "explicit monitor obligation")),
        );

    let result = VerificationAggregator::new().aggregate(&workflow, &inputs, &runtime);

    assert!(result.errors.is_empty());
}
