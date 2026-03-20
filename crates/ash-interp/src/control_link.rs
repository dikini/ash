//! Control link registry for runtime control authority.
//!
//! The current implementation still uses a consumption-style registry, but the canonical
//! documentation now treats `ControlLink` as reusable supervision authority with terminal
//! invalidation semantics. This module will be aligned to that contract by later runtime tasks.

use ash_core::{ControlLink, WorkflowId};
use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur when using control links
#[derive(Debug, Error, Clone, PartialEq)]
pub enum ControlLinkError {
    /// The control link has already been consumed
    #[error("control link for instance {0:?} has already been consumed")]
    AlreadyConsumed(WorkflowId),

    /// The control link was not found in the registry
    #[error("control link for instance {0:?} not found")]
    NotFound(WorkflowId),

    /// The instance is not valid or has been terminated
    #[error("instance {0:?} is not valid")]
    InvalidInstance(WorkflowId),

    /// Attempted to use a control link that was never registered
    #[error("control link was never registered")]
    NeverRegistered,
}

/// The state of a control link in the registry
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkState {
    /// The link is available for use
    Available,
    /// The link has been consumed and cannot be reused
    Consumed,
}

/// Registry for tracking control link usage
///
/// Implements affine semantics: each control link can be used exactly once.
/// After a link is used for a supervision operation (kill, pause, resume, check_health),
/// it is marked as consumed and cannot be used again.
#[derive(Debug, Clone, Default)]
pub struct ControlLinkRegistry {
    /// Maps instance IDs to their control link state
    links: HashMap<WorkflowId, LinkState>,
}

impl ControlLinkRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            links: HashMap::new(),
        }
    }

    /// Register a new control link as available
    ///
    /// This should be called when a workflow is spawned and provides a control link.
    pub fn register(&mut self, instance_id: WorkflowId) {
        self.links.insert(instance_id, LinkState::Available);
    }

    /// Acquire a control link for use, marking it as consumed
    ///
    /// Returns the control link if available, or an error if:
    /// - The link was never registered
    /// - The link has already been consumed
    pub fn acquire(&mut self, link: &ControlLink) -> Result<ControlLink, ControlLinkError> {
        match self.links.get_mut(&link.instance_id) {
            Some(LinkState::Available) => {
                self.links.insert(link.instance_id, LinkState::Consumed);
                Ok(link.clone())
            }
            Some(LinkState::Consumed) => Err(ControlLinkError::AlreadyConsumed(link.instance_id)),
            None => Err(ControlLinkError::NotFound(link.instance_id)),
        }
    }

    /// Verify that a control link is still available (not consumed)
    ///
    /// Returns true if the link exists and is available, false otherwise.
    /// This does NOT consume the link - use `acquire` for that.
    pub fn verify_unused(&self, link: &ControlLink) -> bool {
        matches!(
            self.links.get(&link.instance_id),
            Some(LinkState::Available)
        )
    }

    /// Check if a control link has been consumed
    ///
    /// Returns true if the link exists and has been consumed.
    pub fn is_consumed(&self, instance_id: &WorkflowId) -> bool {
        matches!(self.links.get(instance_id), Some(LinkState::Consumed))
    }

    /// Mark a control link as consumed without returning it
    ///
    /// This is useful when transferring ownership through other means.
    pub fn consume(&mut self, instance_id: &WorkflowId) -> Result<(), ControlLinkError> {
        match self.links.get_mut(instance_id) {
            Some(LinkState::Available) => {
                self.links.insert(*instance_id, LinkState::Consumed);
                Ok(())
            }
            Some(LinkState::Consumed) => Err(ControlLinkError::AlreadyConsumed(*instance_id)),
            None => Err(ControlLinkError::NotFound(*instance_id)),
        }
    }

    /// Remove a control link from the registry
    ///
    /// This should be called when the instance is terminated or cleaned up.
    pub fn remove(&mut self, instance_id: &WorkflowId) -> Option<LinkState> {
        self.links.remove(instance_id)
    }

    /// Get the current state of a control link
    pub fn get_state(&self, instance_id: &WorkflowId) -> Option<LinkState> {
        self.links.get(instance_id).copied()
    }

    /// Returns the number of registered control links
    pub fn len(&self) -> usize {
        self.links.len()
    }

    /// Returns true if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.links.is_empty()
    }

    /// Returns the number of available (unused) links
    pub fn available_count(&self) -> usize {
        self.links
            .values()
            .filter(|&&state| state == LinkState::Available)
            .count()
    }

    /// Returns the number of consumed links
    pub fn consumed_count(&self) -> usize {
        self.links
            .values()
            .filter(|&&state| state == LinkState::Consumed)
            .count()
    }
}

/// Extension trait for Result types involving control links
pub trait ControlLinkResultExt<T> {
    /// Ensure the control link is consumed on success
    fn consume_link(self, registry: &mut ControlLinkRegistry, link: &ControlLink) -> Self;
}

impl<T> ControlLinkResultExt<T> for Result<T, ControlLinkError> {
    fn consume_link(self, registry: &mut ControlLinkRegistry, link: &ControlLink) -> Self {
        if self.is_ok() {
            // If operation succeeded, ensure link is marked consumed
            let _ = registry.consume(&link.instance_id);
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_instance_id() -> WorkflowId {
        WorkflowId::new()
    }

    fn test_control_link(instance_id: WorkflowId) -> ControlLink {
        ControlLink { instance_id }
    }

    #[test]
    fn test_register_and_acquire() {
        let mut registry = ControlLinkRegistry::new();
        let id = test_instance_id();
        let link = test_control_link(id);

        // Register the link
        registry.register(id);
        assert!(registry.verify_unused(&link));

        // Acquire should succeed
        let acquired = registry.acquire(&link);
        assert!(acquired.is_ok());

        // Now it's consumed
        assert!(!registry.verify_unused(&link));
        assert!(registry.is_consumed(&id));
    }

    #[test]
    fn test_double_acquire_fails() {
        let mut registry = ControlLinkRegistry::new();
        let id = test_instance_id();
        let link = test_control_link(id);

        registry.register(id);

        // First acquire succeeds
        assert!(registry.acquire(&link).is_ok());

        // Second acquire fails
        let result = registry.acquire(&link);
        assert!(matches!(result, Err(ControlLinkError::AlreadyConsumed(_))));
    }

    #[test]
    fn test_acquire_unregistered_fails() {
        let mut registry = ControlLinkRegistry::new();
        let id = test_instance_id();
        let link = test_control_link(id);

        // Trying to acquire without registering
        let result = registry.acquire(&link);
        assert!(matches!(result, Err(ControlLinkError::NotFound(_))));
    }

    #[test]
    fn test_verify_unused_does_not_consume() {
        let mut registry = ControlLinkRegistry::new();
        let id = test_instance_id();
        let link = test_control_link(id);

        registry.register(id);

        // Multiple verifies should all return true
        assert!(registry.verify_unused(&link));
        assert!(registry.verify_unused(&link));
        assert!(registry.verify_unused(&link));

        // Link is still available
        assert_eq!(registry.get_state(&id), Some(LinkState::Available));
    }

    #[test]
    fn test_consume_explicitly() {
        let mut registry = ControlLinkRegistry::new();
        let id = test_instance_id();

        registry.register(id);
        assert_eq!(registry.get_state(&id), Some(LinkState::Available));

        // Explicitly consume
        assert!(registry.consume(&id).is_ok());
        assert_eq!(registry.get_state(&id), Some(LinkState::Consumed));

        // Double consume fails
        assert!(matches!(
            registry.consume(&id),
            Err(ControlLinkError::AlreadyConsumed(_))
        ));
    }

    #[test]
    fn test_remove_link() {
        let mut registry = ControlLinkRegistry::new();
        let id = test_instance_id();

        registry.register(id);
        assert_eq!(registry.len(), 1);

        registry.remove(&id);
        assert_eq!(registry.len(), 0);
        assert_eq!(registry.get_state(&id), None);
    }

    #[test]
    fn test_counts() {
        let mut registry = ControlLinkRegistry::new();

        let id1 = test_instance_id();
        let id2 = test_instance_id();
        let id3 = test_instance_id();

        registry.register(id1);
        registry.register(id2);
        registry.register(id3);

        assert_eq!(registry.len(), 3);
        assert_eq!(registry.available_count(), 3);
        assert_eq!(registry.consumed_count(), 0);

        registry.consume(&id1).unwrap();

        assert_eq!(registry.len(), 3);
        assert_eq!(registry.available_count(), 2);
        assert_eq!(registry.consumed_count(), 1);
    }

    // Property-based tests for affine semantics
    #[test]
    fn test_link_used_exactly_once() {
        let mut registry = ControlLinkRegistry::new();
        let id = test_instance_id();
        let link = test_control_link(id);

        registry.register(id);

        // Use the link once
        assert!(registry.acquire(&link).is_ok());

        // Cannot use again
        assert!(registry.acquire(&link).is_err());
        assert!(registry.consume(&id).is_err());
        assert!(!registry.verify_unused(&link));
    }

    #[test]
    fn test_split_creates_link_that_must_be_used() {
        // This test simulates the pattern from the spec:
        // spawn worker with {} as w;
        // let (w_addr, w_ctrl) = split w;
        // -- w_ctrl is Option<ControlLink>, initially Some

        let mut registry = ControlLinkRegistry::new();
        let instance_id = test_instance_id();

        // After spawn, the control link is registered
        registry.register(instance_id);

        // Create a control link (this would come from split)
        let control_link = ControlLink { instance_id };

        // Initially available
        assert!(registry.verify_unused(&control_link));

        // Use it for supervision (e.g., kill)
        assert!(registry.acquire(&control_link).is_ok());

        // Now consumed
        assert!(registry.is_consumed(&instance_id));
    }
}
