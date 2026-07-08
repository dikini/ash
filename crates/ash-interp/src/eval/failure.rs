//! Runtime failure attribution helpers for expression evaluation.

use ash_core::Value;
use ash_core::runtime::{FailureEvidence, OperationalFailure};

use crate::context::Context;
use crate::error::EvalError;

pub(super) fn operational_failure_for_payload(payload: Value, ctx: &Context) -> OperationalFailure {
    let payload_type = value_type_name(&payload);
    let (boundary, entity) = ctx.current_failure_attribution();
    OperationalFailure::new(boundary, entity, payload, payload_type)
}

pub(super) fn operational_failure_with_attribution(
    payload: Value,
    boundary: ash_core::runtime::FailureBoundary,
    entity: ash_core::runtime::FailureEntity,
) -> OperationalFailure {
    let payload_type = value_type_name(&payload);
    OperationalFailure::new(boundary, entity, payload, payload_type)
}

pub(super) fn operational_eval_error_for_message(message: String, ctx: &Context) -> EvalError {
    EvalError::OperationalFailure(Box::new(operational_failure_for_payload(
        Value::String(message),
        ctx,
    )))
}

pub(super) fn operational_eval_error_for_message_with_attribution(
    message: String,
    boundary: ash_core::runtime::FailureBoundary,
    entity: ash_core::runtime::FailureEntity,
) -> EvalError {
    EvalError::OperationalFailure(Box::new(operational_failure_with_attribution(
        Value::String(message),
        boundary,
        entity,
    )))
}

pub(super) fn operational_eval_error_for_resource_policy(
    violation: crate::runtime_state::ResourceSplitJoinViolation,
    ctx: &Context,
) -> EvalError {
    let mut failure = operational_failure_for_payload(Value::String(violation.to_string()), ctx);
    failure.evidence = FailureEvidence {
        notes: violation.evidence_notes(),
        provenance: violation.evidence_provenance(),
    };
    EvalError::OperationalFailure(Box::new(failure))
}

pub(super) fn value_type_name(value: &Value) -> &'static str {
    if value.is_list() {
        return "List";
    }
    match value {
        Value::Int(_) => "Int",
        Value::Float(_) => "Float",
        Value::String(_) => "String",
        Value::Bool(_) => "Bool",
        Value::Null => "Null",
        Value::Time(_) => "Time",
        Value::Ref(_) => "Ref",
        Value::Record(_) => "Record",
        Value::Cap(_) => "Cap",
        Value::Variant { .. } => "Variant",
        Value::Instance(_) => "Instance",
        Value::InstanceAddr(_) => "InstanceAddr",
        Value::ControlLink(_) => "ControlLink",
        Value::Stream(_) => "Stream",
        Value::ProcessHandle(_) => "P",
        Value::ProcAwaitCapture(_) => "<proc-await>",
        Value::ProcYieldCapture => "<proc-yield>",
        Value::ProcParCapture { .. } => "<proc-par>",
        Value::ProcScatterCapture { .. } => "<proc-scatter>",
        Value::ProcJoinCapture { .. } => "<proc-join>",
        Value::ProcGatherCapture { .. } => "<proc-gather>",
        Value::Closure { .. } => "Closure",
        Value::ActEnvToken => "ActEnvToken",
    }
}
