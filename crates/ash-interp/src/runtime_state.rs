//! Shared runtime-owned state for interpreter executions.

use std::sync::Arc;

use tokio::sync::Mutex;

use crate::control_link::ControlLinkRegistry;

/// Shared runtime state that must persist across related top-level executions.
///
/// This is the runtime-owned carrier for lifecycle state such as reusable control authority.
#[derive(Clone, Debug, Default)]
pub struct RuntimeState {
    control_registry: Arc<Mutex<ControlLinkRegistry>>,
}

impl RuntimeState {
    /// Create a new empty runtime state.
    pub fn new() -> Self {
        Self {
            control_registry: Arc::new(Mutex::new(ControlLinkRegistry::new())),
        }
    }

    pub(crate) fn control_registry(&self) -> Arc<Mutex<ControlLinkRegistry>> {
        self.control_registry.clone()
    }
}
