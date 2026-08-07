//! Component-wise child process environment projection.
//!
//! `derive_child_env` is the explicit TASK-709 substrate boundary for creating
//! child process contexts. It may reuse existing `Context` storage internally,
//! but callers should use this named projection API rather than treating
//! `Context::clone` as process-environment semantics.

use ash_core::runtime::ProcessId;
use thiserror::Error;

use crate::context::Context;

/// Child process environment identity metadata projected into a child context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcessEnvIdentity {
    /// Process identity for the projected child context.
    pub process_id: ProcessId,
    /// Optional parent process identity for the projected child context.
    pub parent_process_id: Option<ProcessId>,
    /// Child index within the parent admission operation.
    pub child_index: usize,
}

impl ProcessEnvIdentity {
    /// Create projected process identity metadata.
    #[must_use]
    pub fn new(
        process_id: ProcessId,
        parent_process_id: Option<ProcessId>,
        child_index: usize,
    ) -> Self {
        Self {
            process_id,
            parent_process_id,
            child_index,
        }
    }
}

/// Request describing one child process environment projection.
#[derive(Debug, Clone, PartialEq)]
pub struct ChildEnvProjection {
    /// Child process identity receiving projected components.
    pub child_process_id: ProcessId,
    /// Child index within the parent admission operation.
    pub child_index: usize,
    /// Optional parent process identity to store in the projected child context.
    pub parent_process_id: Option<ProcessId>,
}

impl ChildEnvProjection {
    /// Create a projection request for one child process.
    #[must_use]
    pub fn new(child_process_id: ProcessId, child_index: usize) -> Self {
        Self {
            child_process_id,
            child_index,
            parent_process_id: None,
        }
    }

    /// Record the parent process identity in the projected child context.
    #[must_use]
    pub fn with_parent_process_id(mut self, parent_process_id: ProcessId) -> Self {
        self.parent_process_id = Some(parent_process_id);
        self
    }
}

/// Errors returned by child environment projection.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ChildEnvProjectionError {}

/// Derive a child process context through component-wise projection.
///
/// The projection preserves lexical value visibility by snapshotting the visible
/// bindings into the child, allocates a fresh child-local obligation scope, and
/// Hidden Act runtime carriers are shared as read-only `Arc` components by
/// `Context`'s internal projection helper.
pub fn derive_child_env(
    parent: &Context,
    projection: ChildEnvProjection,
) -> Result<Context, ChildEnvProjectionError> {
    Ok(parent.project_process_child(ProcessEnvIdentity::new(
        projection.child_process_id,
        projection.parent_process_id.or_else(|| {
            parent
                .process_identity()
                .map(|identity| identity.process_id)
        }),
        projection.child_index,
    )))
}
