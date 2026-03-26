//! Shared runtime-owned state for interpreter executions.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;

use crate::capability::CapabilityProvider;
use crate::control_link::ControlLinkRegistry;
use crate::proxy_registry::ProxyRegistry;
use crate::yield_routing::YieldRouter;
use crate::yield_state::SuspendedYields;

/// Shared runtime state that must persist across related top-level executions.
///
/// This is the runtime-owned carrier for lifecycle state such as reusable control authority,
/// proxy registrations, suspended yields, and yield routing.
///
/// # Provider Registry
///
/// RuntimeState also maintains a registry of capability providers that can be
/// used during workflow execution. Providers can be registered using
/// [`RuntimeState::with_provider`] or [`RuntimeState::with_providers`].
#[derive(Clone, Default)]
pub struct RuntimeState {
    control_registry: Arc<Mutex<ControlLinkRegistry>>,
    proxy_registry: Arc<Mutex<ProxyRegistry>>,
    suspended_yields: Arc<Mutex<SuspendedYields>>,
    yield_router: Arc<Mutex<YieldRouter>>,
    /// Capability provider registry for execution
    providers: Arc<Mutex<HashMap<String, Arc<dyn CapabilityProvider>>>>,
}

impl std::fmt::Debug for RuntimeState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuntimeState")
            .field("control_registry", &self.control_registry)
            .field("proxy_registry", &self.proxy_registry)
            .field("suspended_yields", &self.suspended_yields)
            .field("yield_router", &self.yield_router)
            .field(
                "providers",
                &"<HashMap<String, Arc<dyn CapabilityProvider>>>",
            )
            .finish()
    }
}

impl RuntimeState {
    /// Create a new empty runtime state.
    pub fn new() -> Self {
        Self {
            control_registry: Arc::new(Mutex::new(ControlLinkRegistry::new())),
            proxy_registry: Arc::new(Mutex::new(ProxyRegistry::new())),
            suspended_yields: Arc::new(Mutex::new(SuspendedYields::new())),
            yield_router: Arc::new(Mutex::new(YieldRouter::new())),
            providers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Add a capability provider to the registry.
    ///
    /// # Arguments
    ///
    /// * `name` - The name to register the provider under
    /// * `provider` - The capability provider to register
    ///
    /// # Example
    ///
    /// ```
    /// use ash_interp::RuntimeState;
    /// use ash_interp::capability::MockProvider;
    /// use ash_core::Effect;
    ///
    /// let state = RuntimeState::new()
    ///     .with_provider("test", std::sync::Arc::new(MockProvider::new("test", Effect::Epistemic)));
    /// ```
    pub fn with_provider(
        self,
        name: impl Into<String>,
        provider: Arc<dyn CapabilityProvider>,
    ) -> Self {
        // This is a bit tricky because we need to modify the Arc<Mutex<_>>
        // We use tokio::sync::Mutex::try_lock in a blocking context
        if let Ok(mut guard) = self.providers.try_lock() {
            guard.insert(name.into(), provider);
        }
        self
    }

    /// Add multiple capability providers to the registry.
    ///
    /// # Arguments
    ///
    /// * `providers` - A HashMap of provider names to providers
    ///
    /// # Example
    ///
    /// ```
    /// use ash_interp::RuntimeState;
    /// use ash_interp::capability::MockProvider;
    /// use ash_core::Effect;
    /// use std::collections::HashMap;
    /// use std::sync::Arc;
    ///
    /// let mut providers = HashMap::new();
    /// providers.insert("test".to_string(), Arc::new(MockProvider::new("test", Effect::Epistemic)));
    ///
    /// let state = RuntimeState::new().with_providers(providers);
    /// ```
    pub fn with_providers(self, providers: HashMap<String, Arc<dyn CapabilityProvider>>) -> Self {
        if let Ok(mut guard) = self.providers.try_lock() {
            guard.extend(providers);
        }
        self
    }

    /// Get a provider by name.
    ///
    /// Returns `Some(Arc<dyn CapabilityProvider>)` if a provider with the given
    /// name is registered, or `None` if not found.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the provider to look up
    ///
    /// # Example
    ///
    /// ```
    /// use ash_interp::RuntimeState;
    /// use ash_interp::capability::MockProvider;
    /// use ash_core::Effect;
    ///
    /// let state = RuntimeState::new()
    ///     .with_provider("test", std::sync::Arc::new(MockProvider::new("test", Effect::Epistemic)));
    ///
    /// let provider = state.get_provider("test");
    /// assert!(provider.is_some());
    /// ```
    pub fn get_provider(&self, name: &str) -> Option<Arc<dyn CapabilityProvider>> {
        // Use blocking_lock for synchronous access
        self.providers.blocking_lock().get(name).cloned()
    }

    /// Get all registered provider names.
    ///
    /// Returns a vector of all provider names currently registered.
    pub fn provider_names(&self) -> Vec<String> {
        self.providers.blocking_lock().keys().cloned().collect()
    }

    /// Check if a provider is registered.
    ///
    /// Returns `true` if a provider with the given name is registered.
    pub fn has_provider(&self, name: &str) -> bool {
        self.providers.blocking_lock().contains_key(name)
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

    /// Get access to the yield router
    pub fn yield_router(&self) -> Arc<Mutex<YieldRouter>> {
        self.yield_router.clone()
    }
}
