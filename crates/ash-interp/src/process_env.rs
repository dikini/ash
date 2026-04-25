//! Component-wise child process environment projection.
//!
//! `derive_child_env` is the explicit TASK-709 substrate boundary for creating
//! child process contexts. It may reuse existing `Context` storage internally,
//! but callers should use this named projection API rather than treating
//! `Context::clone` as process-environment semantics.

use ash_core::Capability;
use ash_core::runtime::ProcessId;
use thiserror::Error;

use crate::context::Context;
use crate::role_context::RoleProjectionError;

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
    /// Optional role authority to project. When omitted, the parent role
    /// authority is projected unchanged.
    pub role_authority: Option<Vec<Capability>>,
}

impl ChildEnvProjection {
    /// Create a projection request for one child process.
    #[must_use]
    pub fn new(child_process_id: ProcessId, child_index: usize) -> Self {
        Self {
            child_process_id,
            child_index,
            parent_process_id: None,
            role_authority: None,
        }
    }

    /// Record the parent process identity in the projected child context.
    #[must_use]
    pub fn with_parent_process_id(mut self, parent_process_id: ProcessId) -> Self {
        self.parent_process_id = Some(parent_process_id);
        self
    }

    /// Request a narrower role-authority surface for the child process.
    #[must_use]
    pub fn with_role_authority(mut self, role_authority: Vec<Capability>) -> Self {
        self.role_authority = Some(role_authority);
        self
    }
}

/// Errors returned by child environment projection.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ChildEnvProjectionError {
    /// The child requested authority that the parent does not have.
    #[error("child process projection would widen parent role authority")]
    WiderAuthority,
    /// The child requested role authority but the parent had no role context.
    #[error("child process projection requested role authority without a parent role")]
    MissingParentRole,
}

impl From<RoleProjectionError> for ChildEnvProjectionError {
    fn from(value: RoleProjectionError) -> Self {
        match value {
            RoleProjectionError::WiderAuthority => Self::WiderAuthority,
        }
    }
}

/// Derive a child process context through component-wise projection.
///
/// The projection preserves lexical value visibility by snapshotting the visible
/// bindings into the child, allocates a fresh child-local obligation scope, and
/// projects role authority equal to or narrower than the parent. Hidden policy
/// and Act runtime carriers are shared as read-only `Arc` components by
/// `Context`'s internal projection helper.
pub fn derive_child_env(
    parent: &Context,
    projection: ChildEnvProjection,
) -> Result<Context, ChildEnvProjectionError> {
    let projected_role = match (parent.role_context(), projection.role_authority) {
        (Some(role_context), Some(authority)) => Some(role_context.project_authority(authority)?),
        (Some(role_context), None) => Some(role_context.clone_for_child()),
        (None, Some(_)) => return Err(ChildEnvProjectionError::MissingParentRole),
        (None, None) => None,
    };

    Ok(parent.project_process_child(
        ProcessEnvIdentity::new(
            projection.child_process_id,
            projection.parent_process_id.or_else(|| {
                parent
                    .process_identity()
                    .map(|identity| identity.process_id)
            }),
            projection.child_index,
        ),
        projected_role,
    ))
}
