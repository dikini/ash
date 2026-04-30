//! Runtime-facing boundary for first-class Workflow Proc projections.
//!
//! Phase 108 introduced shared `ash-core` lowering carriers before the full
//! runtime executor exists. This module is the named seam consumed by runtime
//! crates: shapes that are already sound (`unit`, non-dependent ignored
//! `bind`/`then`, transparent `scope`) execute, and all remaining first-class
//! Workflow projection shapes fail at an explicit diagnostic boundary instead
//! of silently becoming inert values.

use ash_core::{
    Value,
    workflow_carrier::{WorkflowBinder, WorkflowNodeId, WorkflowProcProjection},
};

use crate::{ExecError, ExecResult};

const UNSUPPORTED_BOUNDARY: &str = "FirstClassWorkflowProjectionExecutionUnsupported";

/// Execute the runtime-facing Proc projection of a first-class Workflow value.
///
/// This intentionally accepts only the public `ash-core` projection carrier. It
/// does not depend on parser AST nodes or typechecker-private typed artifacts.
/// Full dependent `bind`, `from_proc`, and `from_act` scheduling is still outside
/// this slice; those shapes return a named Phase 108 unsupported diagnostic.
pub fn execute_workflow_proc_projection(
    projection: &WorkflowProcProjection<Value>,
) -> ExecResult<Value> {
    match projection {
        WorkflowProcProjection::Unit { value, .. } => Ok(value.clone()),
        WorkflowProcProjection::Scope { body, .. } => execute_workflow_proc_projection(body),
        WorkflowProcProjection::Bind {
            source,
            binder: WorkflowBinder::Ignored,
            next,
            ..
        } => {
            let _ = execute_workflow_proc_projection(source)?;
            execute_workflow_proc_projection(next)
        }
        WorkflowProcProjection::Bind { .. }
        | WorkflowProcProjection::FromProc { .. }
        | WorkflowProcProjection::FromAct { .. }
        | WorkflowProcProjection::Neutral { .. } => Err(ExecError::InvalidRuntimeState(
            unsupported_workflow_proc_projection_message(projection),
        )),
    }
}

/// Build the stable diagnostic for unsupported first-class Workflow projection
/// execution cases.
#[must_use]
pub fn unsupported_workflow_proc_projection_message(
    projection: &WorkflowProcProjection<Value>,
) -> String {
    let (shape, node) = projection_shape_and_node(projection);
    format!(
        "{UNSUPPORTED_BOUNDARY}: Phase 108 is still check/lowering-only for first-class Workflow projection execution of {shape} at node {}",
        node.0
    )
}

fn projection_shape_and_node(
    projection: &WorkflowProcProjection<Value>,
) -> (&'static str, WorkflowNodeId) {
    match projection {
        WorkflowProcProjection::Unit { node, .. } => ("unit", *node),
        WorkflowProcProjection::Bind { node, .. } => ("bind", *node),
        WorkflowProcProjection::FromProc { node, .. } => ("from_proc", *node),
        WorkflowProcProjection::FromAct { node, .. } => ("from_act", *node),
        WorkflowProcProjection::Scope { node, .. } => ("scope", *node),
        WorkflowProcProjection::Neutral { node } => ("neutral governance node", *node),
    }
}
