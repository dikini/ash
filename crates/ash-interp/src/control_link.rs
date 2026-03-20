//! Control link registry for reusable runtime supervision authority.

use ash_core::{ControlLink, WorkflowId};
use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur when using control links.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum ControlLinkError {
    /// The control link was not found in the registry.
    #[error("control link for instance {0:?} not registered")]
    NotFound(WorkflowId),

    /// The instance has been terminated and can no longer be controlled.
    #[error("instance {0:?} has been terminated")]
    Terminated(WorkflowId),
}

/// The supervision state of a control link target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkState {
    /// The instance is alive and currently running.
    Running,
    /// The instance is alive but currently paused.
    Paused,
    /// The instance has been killed; future control operations are invalid.
    Terminated,
}

/// Registry for runtime control-link lifecycle.
///
/// A control link is reusable while the target instance remains valid. Non-terminal operations
/// such as pause, resume, and health checks do not consume the link. Kill is terminal and
/// invalidates future control operations.
#[derive(Debug, Clone, Default)]
pub struct ControlLinkRegistry {
    links: HashMap<WorkflowId, LinkState>,
}

impl ControlLinkRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            links: HashMap::new(),
        }
    }

    /// Register a new control link as running.
    pub fn register(&mut self, instance_id: WorkflowId) {
        self.links.entry(instance_id).or_insert(LinkState::Running);
    }

    fn get_live_state(&self, link: &ControlLink) -> Result<LinkState, ControlLinkError> {
        match self.links.get(&link.instance_id).copied() {
            Some(LinkState::Running) => Ok(LinkState::Running),
            Some(LinkState::Paused) => Ok(LinkState::Paused),
            Some(LinkState::Terminated) => Err(ControlLinkError::Terminated(link.instance_id)),
            None => Err(ControlLinkError::NotFound(link.instance_id)),
        }
    }

    /// Check that the control link still points to a valid supervised instance.
    pub fn check_health(&self, link: &ControlLink) -> Result<LinkState, ControlLinkError> {
        self.get_live_state(link)
    }

    /// Pause the supervised instance if it is still valid.
    pub fn pause(&mut self, link: &ControlLink) -> Result<(), ControlLinkError> {
        match self.get_live_state(link)? {
            LinkState::Running | LinkState::Paused => {
                self.links.insert(link.instance_id, LinkState::Paused);
                Ok(())
            }
            LinkState::Terminated => Err(ControlLinkError::Terminated(link.instance_id)),
        }
    }

    /// Resume the supervised instance if it is still valid.
    pub fn resume(&mut self, link: &ControlLink) -> Result<(), ControlLinkError> {
        match self.get_live_state(link)? {
            LinkState::Running | LinkState::Paused => {
                self.links.insert(link.instance_id, LinkState::Running);
                Ok(())
            }
            LinkState::Terminated => Err(ControlLinkError::Terminated(link.instance_id)),
        }
    }

    /// Kill the supervised instance, invalidating future control operations.
    pub fn kill(&mut self, link: &ControlLink) -> Result<(), ControlLinkError> {
        self.get_live_state(link)?;
        self.links.insert(link.instance_id, LinkState::Terminated);
        Ok(())
    }

    /// Remove a control link from the registry.
    pub fn remove(&mut self, instance_id: &WorkflowId) -> Option<LinkState> {
        self.links.remove(instance_id)
    }

    /// Get the current state of a control link.
    pub fn get_state(&self, instance_id: &WorkflowId) -> Option<LinkState> {
        self.links.get(instance_id).copied()
    }

    /// Returns the number of registered control links.
    pub fn len(&self) -> usize {
        self.links.len()
    }

    /// Returns true if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.links.is_empty()
    }

    /// Returns the number of live control links.
    pub fn live_count(&self) -> usize {
        self.links
            .values()
            .filter(|&&state| state != LinkState::Terminated)
            .count()
    }

    /// Returns the number of terminated control links.
    pub fn terminated_count(&self) -> usize {
        self.links
            .values()
            .filter(|&&state| state == LinkState::Terminated)
            .count()
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
    fn register_defaults_to_running() {
        let mut registry = ControlLinkRegistry::new();
        let id = test_instance_id();

        registry.register(id);

        assert_eq!(registry.get_state(&id), Some(LinkState::Running));
    }

    #[test]
    fn pause_and_resume_are_reusable() {
        let mut registry = ControlLinkRegistry::new();
        let id = test_instance_id();
        let link = test_control_link(id);
        registry.register(id);

        assert!(registry.pause(&link).is_ok());
        assert_eq!(registry.get_state(&id), Some(LinkState::Paused));
        assert_eq!(registry.check_health(&link), Ok(LinkState::Paused));

        assert!(registry.resume(&link).is_ok());
        assert_eq!(registry.get_state(&id), Some(LinkState::Running));
        assert_eq!(registry.check_health(&link), Ok(LinkState::Running));
    }

    #[test]
    fn kill_is_terminal() {
        let mut registry = ControlLinkRegistry::new();
        let id = test_instance_id();
        let link = test_control_link(id);
        registry.register(id);

        assert!(registry.kill(&link).is_ok());
        assert_eq!(registry.get_state(&id), Some(LinkState::Terminated));
        assert_eq!(
            registry.check_health(&link),
            Err(ControlLinkError::Terminated(id))
        );
        assert_eq!(registry.pause(&link), Err(ControlLinkError::Terminated(id)));
        assert_eq!(
            registry.resume(&link),
            Err(ControlLinkError::Terminated(id))
        );
        assert_eq!(registry.kill(&link), Err(ControlLinkError::Terminated(id)));
    }

    #[test]
    fn missing_link_fails() {
        let mut registry = ControlLinkRegistry::new();
        let id = test_instance_id();
        let link = test_control_link(id);

        assert_eq!(
            registry.check_health(&link),
            Err(ControlLinkError::NotFound(id))
        );
        assert_eq!(registry.pause(&link), Err(ControlLinkError::NotFound(id)));
        assert_eq!(registry.resume(&link), Err(ControlLinkError::NotFound(id)));
        assert_eq!(registry.kill(&link), Err(ControlLinkError::NotFound(id)));
    }

    #[test]
    fn remove_forgets_link_state() {
        let mut registry = ControlLinkRegistry::new();
        let id = test_instance_id();
        registry.register(id);
        registry.kill(&test_control_link(id)).unwrap();

        assert_eq!(registry.remove(&id), Some(LinkState::Terminated));
        assert!(registry.get_state(&id).is_none());
        assert!(registry.is_empty());
    }
}
