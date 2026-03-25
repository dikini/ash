//! Shared runtime-owned state for interpreter executions.

use std::sync::Arc;

use tokio::sync::Mutex;

use crate::control_link::ControlLinkRegistry;
use crate::proxy_registry::ProxyRegistry;
use crate::yield_state::SuspendedYields;

/// Shared runtime state that must persist across related top-level executions.
///
/// This is the runtime-owned carrier for lifecycle state such as reusable control authority,
/// proxy registrations, and suspended yields.
#[derive(Clone, Debug, Default)]
pub struct RuntimeState {
    control_registry: Arc<Mutex<ControlLinkRegistry>>,
    proxy_registry: Arc<Mutex<ProxyRegistry>>,
    suspended_yields: Arc<Mutex<SuspendedYields>>,
}

impl RuntimeState {
    /// Create a new empty runtime state.
    pub fn new() -> Self {
        Self {
            control_registry: Arc::new(Mutex::new(ControlLinkRegistry::new())),
            proxy_registry: Arc::new(Mutex::new(ProxyRegistry::new())),
            suspended_yields: Arc::new(Mutex::new(SuspendedYields::new())),
        }
    }

    pub(crate) fn control_registry(&self) -> Arc<Mutex<ControlLinkRegistry>> {
        self.control_registry.clone()
    }

    /// Get access to the proxy registry
    pub fn proxy_registry(&self) -> Arc<Mutex<ProxyRegistry>> {
        self.proxy_registry.clone()
    }

    /// Get access to the suspended yields registry
    pub fn suspended_yields(&self) -> Arc<Mutex<SuspendedYields>> {
        self.suspended_yields.clone()
    }
}
