//! Shared runtime-owned state for interpreter executions.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

use ash_core::{ControlLink, Value, Workflow};

use crate::capability::CapabilityProvider;
use crate::control_link::{
    ConservativeRetainedEffectSummary, ConservativeRetainedObligationsSummary,
    ConservativeRetainedProvenanceSummary, ControlLinkRegistry, LinkState,
    RetainedCompletionRecord, RetainedCompletionWaiter,
};
use crate::{ExecError, ExecResult};

use crate::proxy_registry::ProxyRegistry;
use crate::runtime_outcome_state::RuntimeOutcomeState;
use crate::yield_routing::YieldRouter;
use crate::yield_state::SuspendedYields;
use std::time::Duration;

pub(crate) const SPAWNED_CHILD_CONTROL_BINDING: &str = "__ash_spawn_control_link";

/// Wrapper that adapts an `Arc<dyn CapabilityProvider>` to work as a `Box<dyn CapabilityProvider>`.
///
/// This is used internally by `RuntimeState` to create a `CapabilityContext` from
/// its stored providers. The wrapper delegates all trait methods to the inner
/// Arc-wrapped provider.
#[derive(Clone)]
struct ArcProviderWrapper {
    inner: Arc<dyn CapabilityProvider>,
}

impl ArcProviderWrapper {
    /// Create a new wrapper around the given Arc-wrapped provider.
    fn new(inner: Arc<dyn CapabilityProvider>) -> Self {
        Self { inner }
    }
}

impl std::fmt::Debug for ArcProviderWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArcProviderWrapper")
            .field("capability_name", &self.inner.capability_name())
            .finish()
    }
}

#[async_trait]
impl CapabilityProvider for ArcProviderWrapper {
    fn capability_name(&self) -> &str {
        self.inner.capability_name()
    }

    fn effect(&self) -> ash_core::Effect {
        self.inner.effect()
    }

    async fn observe(
        &self,
        constraints: &[ash_core::Constraint],
    ) -> crate::ExecResult<ash_core::Value> {
        self.inner.observe(constraints).await
    }

    async fn execute(&self, action: &ash_core::Action) -> crate::ExecResult<ash_core::Value> {
        self.inner.execute(action).await
    }
}

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
    child_workflows: Arc<Mutex<HashMap<String, Workflow>>>,
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
            .field("child_workflows", &"<HashMap<String, Workflow>>")
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
            child_workflows: Arc::new(Mutex::new(HashMap::new())),
            providers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register one runtime-owned child workflow entry keyed by `workflow_type`.
    ///
    /// The current spawned-child substrate uses a narrow runtime-owned entry contract:
    /// when a spawned child is executed, the evaluated spawn `init` expression is bound into the
    /// child context as the variable `init` before this workflow body is run.
    pub async fn register_child_workflow(
        &self,
        workflow_type: impl Into<String>,
        workflow: Workflow,
    ) {
        self.child_workflows
            .lock()
            .await
            .insert(workflow_type.into(), workflow);
    }

    /// Look up one runtime-owned child workflow entry by `workflow_type`.
    pub async fn child_workflow(&self, workflow_type: &str) -> Option<Workflow> {
        self.child_workflows
            .lock()
            .await
            .get(workflow_type)
            .cloned()
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
    /// use ash_interp::capability::{CapabilityProvider, MockProvider};
    /// use ash_core::Effect;
    /// use std::collections::HashMap;
    /// use std::sync::Arc;
    ///
    /// let mut providers: HashMap<String, Arc<dyn CapabilityProvider>> = HashMap::new();
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

    /// Get the number of registered providers.
    ///
    /// Returns the count of providers currently registered.
    pub fn provider_count(&self) -> usize {
        self.providers.blocking_lock().len()
    }

    /// Create a CapabilityContext from the registered providers.
    ///
    /// This allows the interpreter to access capability providers
    /// during workflow execution.
    pub async fn create_capability_context(&self) -> crate::capability::CapabilityContext {
        use crate::capability::{CapabilityContext, CapabilityRegistry};

        let mut registry = CapabilityRegistry::new();

        // Convert Arc<dyn CapabilityProvider> from RuntimeState to Box<dyn CapabilityProvider>
        // for the CapabilityContext. We clone the Arc to get a reference, then create
        // a wrapper that delegates to the original provider.
        let providers = self.providers.lock().await;
        for (_name, provider) in providers.iter() {
            // Create a wrapper that implements the CapabilityProvider trait
            // by delegating to the Arc-wrapped provider
            let wrapper = Box::new(ArcProviderWrapper::new(provider.clone()));
            registry.register(wrapper);
        }

        CapabilityContext::with_registry(registry)
    }

    pub(crate) fn control_registry(&self) -> Arc<Mutex<ControlLinkRegistry>> {
        self.control_registry.clone()
    }

    /// Register one spawned control target in the shared runtime state.
    pub async fn register_spawned_control_link(&self, instance_id: ash_core::WorkflowId) {
        self.control_registry.lock().await.register(instance_id);
    }

    /// Register one spawned control target together with the conservative runtime-owned spawn
    /// provenance the runtime can snapshot today.
    pub async fn register_spawned_control_link_with_provenance(
        &self,
        provenance: ConservativeRetainedProvenanceSummary,
    ) {
        self.control_registry
            .lock()
            .await
            .register_with_spawn_provenance(provenance);
    }

    /// Pause one controlled runtime target.
    pub async fn pause_control_link(
        &self,
        link: &ControlLink,
    ) -> Result<(), crate::control_link::ControlLinkError> {
        self.control_registry.lock().await.pause(link)
    }

    /// Resume one controlled runtime target.
    pub async fn resume_control_link(
        &self,
        link: &ControlLink,
    ) -> Result<(), crate::control_link::ControlLinkError> {
        self.control_registry.lock().await.resume(link)
    }

    /// Kill one controlled runtime target.
    pub async fn kill_control_link(
        &self,
        link: &ControlLink,
    ) -> Result<(), crate::control_link::ControlLinkError> {
        self.control_registry.lock().await.kill(link)
    }

    /// Build the initial input bindings for a runtime-owned child entry execution.
    ///
    /// In addition to the user-visible `init` binding, the runtime injects one internal control
    /// binding so spawned child execution can cooperatively observe pause/resume/kill authority.
    pub fn spawned_child_init_bindings(
        init_value: Value,
        control_link: ControlLink,
    ) -> HashMap<String, Value> {
        HashMap::from([
            ("init".to_string(), init_value),
            (
                SPAWNED_CHILD_CONTROL_BINDING.to_string(),
                Value::ControlLink(control_link),
            ),
        ])
    }

    /// Wait until a controlled spawned child is allowed to make progress.
    ///
    /// This is intentionally cooperative rather than preemptive: it checks the control state at
    /// workflow entry boundaries, blocks while paused, and stops further progress after kill.
    pub async fn wait_for_control_authority(&self, link: &ControlLink) -> crate::ExecResult<()> {
        loop {
            let state = self.control_registry.lock().await.check_health(link);
            match state {
                Ok(LinkState::Running) => return Ok(()),
                Ok(LinkState::Paused) => tokio::time::sleep(Duration::from_millis(1)).await,
                Ok(LinkState::Terminated) => unreachable!(
                    "terminated links are reported as errors by ControlLinkRegistry::check_health"
                ),
                Err(error) => {
                    return Err(ExecError::InvalidRuntimeState(format!(
                        "spawned child control wait failed for instance {:?}: {error}",
                        link.instance_id
                    )));
                }
            }
        }
    }

    /// Record one retained terminal completion-style observation for a control target.
    pub async fn record_control_completion(
        &self,
        link: &ControlLink,
        result: ExecResult<Value>,
        effects: ConservativeRetainedEffectSummary,
        obligations: ConservativeRetainedObligationsSummary,
        provenance: Option<ConservativeRetainedProvenanceSummary>,
    ) -> Result<RetainedCompletionRecord, crate::control_link::ControlLinkError> {
        self.control_registry.lock().await.record_completion(
            link,
            result,
            effects,
            obligations,
            provenance,
        )
    }

    /// Read the retained terminal completion-style observation for a control target, if one
    /// exists.
    ///
    /// This surface answers a different question from
    /// [`Self::control_link_runtime_outcome_state`]: once a control link is no longer live, that
    /// coarse runtime state reports terminal control-liveness (`InvalidOrTerminated`), while this
    /// retained record preserves the sealed terminal completion subtype/payload captured for that
    /// target.
    pub async fn retained_completion(
        &self,
        link: &ControlLink,
    ) -> Option<RetainedCompletionRecord> {
        self.control_registry
            .lock()
            .await
            .retained_completion(&link.instance_id)
    }

    /// Wait for the first sealed retained completion-style observation for a control target.
    ///
    /// This reuses the same retained completion carrier returned by [`Self::retained_completion`].
    /// If the target is already sealed when waiting begins, this returns immediately. If the
    /// target is registered but not yet sealed, this waits until the first authoritative retained
    /// record is sealed. Invalid or unregistered targets remain distinguishable as
    /// [`crate::control_link::ControlLinkError`] values rather than being synthesized into a fake
    /// completion record.
    pub async fn wait_for_retained_completion(
        &self,
        link: &ControlLink,
    ) -> Result<RetainedCompletionRecord, crate::control_link::ControlLinkError> {
        let waiter = {
            self.control_registry
                .lock()
                .await
                .retained_completion_waiter(link)?
        };

        match waiter {
            RetainedCompletionWaiter::Ready(record) => Ok(*record),
            RetainedCompletionWaiter::Pending(mut receiver) => loop {
                if let Some(record) = receiver.borrow().clone() {
                    return Ok(record);
                }

                if receiver.changed().await.is_err() {
                    return Err(crate::control_link::ControlLinkError::NotFound(
                        link.instance_id,
                    ));
                }
            },
        }
    }

    /// Classify the current runtime-visible control-liveness state of a control link using the
    /// authoritative runtime outcome/state surface.
    ///
    /// This method reports whether the control authority is still live/usable. After a child has
    /// sealed a retained completion and the link is terminal, this surface intentionally reports
    /// `InvalidOrTerminated`; callers that need the sealed terminal completion subtype/payload must
    /// consult [`Self::retained_completion`].
    pub async fn control_link_runtime_outcome_state(
        &self,
        link: &ControlLink,
    ) -> RuntimeOutcomeState {
        let registry = self.control_registry.lock().await;
        match registry.check_health(link) {
            Ok(state) => state.runtime_outcome_state(),
            Err(error) => error.runtime_outcome_state(),
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use ash_core::{Effect, Expr, Workflow};
    use tokio::time::{Duration, timeout};

    fn retained_effect_summary(
        terminal: Effect,
        reached: &[Effect],
    ) -> crate::control_link::ConservativeRetainedEffectSummary {
        crate::control_link::ConservativeRetainedEffectSummary::new(
            terminal,
            reached.iter().copied().collect(),
        )
    }

    fn retained_obligations_summary() -> crate::control_link::ConservativeRetainedObligationsSummary
    {
        crate::control_link::ConservativeRetainedObligationsSummary::new(
            std::collections::BTreeSet::new(),
            None,
            std::collections::BTreeSet::new(),
            std::collections::BTreeSet::new(),
        )
    }

    #[tokio::test]
    async fn child_workflow_registry_round_trips() {
        let runtime_state = RuntimeState::new();
        let workflow = Workflow::Ret {
            expr: Expr::Literal(Value::Int(1)),
        };

        runtime_state
            .register_child_workflow("worker", workflow.clone())
            .await;

        assert_eq!(runtime_state.child_workflow("worker").await, Some(workflow));
        assert!(runtime_state.child_workflow("missing").await.is_none());
    }

    #[tokio::test]
    async fn control_link_runtime_outcome_state_reports_active_then_invalid() {
        let runtime_state = RuntimeState::new();
        let instance_id = ash_core::WorkflowId::new();
        let link = ControlLink { instance_id };

        {
            let registry = runtime_state.control_registry();
            registry.lock().await.register(instance_id);
        }

        assert_eq!(
            runtime_state
                .control_link_runtime_outcome_state(&link)
                .await,
            RuntimeOutcomeState::Active
        );

        {
            let registry = runtime_state.control_registry();
            registry.lock().await.kill(&link).unwrap();
        }

        assert_eq!(
            runtime_state
                .control_link_runtime_outcome_state(&link)
                .await,
            RuntimeOutcomeState::InvalidOrTerminated
        );
    }

    #[tokio::test]
    async fn retained_completion_round_trips_through_runtime_state() {
        let runtime_state = RuntimeState::new();
        let instance_id = ash_core::WorkflowId::new();
        let link = ControlLink { instance_id };

        runtime_state
            .register_spawned_control_link(instance_id)
            .await;
        let effects = retained_effect_summary(Effect::Operational, &[Effect::Operational]);
        let obligations = retained_obligations_summary();
        let record = runtime_state
            .record_control_completion(
                &link,
                Ok(Value::Int(7)),
                effects.clone(),
                obligations.clone(),
                None,
            )
            .await
            .unwrap();

        assert_eq!(runtime_state.retained_completion(&link).await, Some(record));
        assert_eq!(
            runtime_state
                .retained_completion(&link)
                .await
                .unwrap()
                .conservative_effect_summary(),
            Some(&effects)
        );
        assert_eq!(
            runtime_state
                .control_link_runtime_outcome_state(&link)
                .await,
            RuntimeOutcomeState::InvalidOrTerminated
        );
    }

    #[tokio::test]
    async fn retained_completion_is_write_once_through_runtime_state() {
        let runtime_state = RuntimeState::new();
        let instance_id = ash_core::WorkflowId::new();
        let link = ControlLink { instance_id };

        runtime_state
            .register_spawned_control_link(instance_id)
            .await;
        let effects = retained_effect_summary(Effect::Operational, &[Effect::Operational]);
        let obligations = retained_obligations_summary();
        let record = runtime_state
            .record_control_completion(
                &link,
                Ok(Value::Int(1)),
                effects.clone(),
                obligations.clone(),
                None,
            )
            .await
            .unwrap();
        let error = runtime_state
            .record_control_completion(
                &link,
                Ok(Value::Int(2)),
                retained_effect_summary(Effect::Epistemic, &[Effect::Epistemic]),
                retained_obligations_summary(),
                None,
            )
            .await
            .expect_err("retained completion should be sealed after first record");

        assert_eq!(
            error,
            crate::control_link::ControlLinkError::CompletionAlreadySealed(
                instance_id,
                Box::new(record.clone())
            )
        );
        assert_eq!(runtime_state.retained_completion(&link).await, Some(record));
    }

    #[tokio::test]
    async fn wait_for_retained_completion_returns_immediately_for_already_sealed_record() {
        let runtime_state = RuntimeState::new();
        let instance_id = ash_core::WorkflowId::new();
        let link = ControlLink { instance_id };

        runtime_state
            .register_spawned_control_link(instance_id)
            .await;
        let record = runtime_state
            .record_control_completion(
                &link,
                Ok(Value::Int(7)),
                retained_effect_summary(Effect::Operational, &[Effect::Operational]),
                retained_obligations_summary(),
                None,
            )
            .await
            .expect("completion should already be sealed");

        let waited = timeout(
            Duration::from_millis(50),
            runtime_state.wait_for_retained_completion(&link),
        )
        .await
        .expect("already-sealed record should return immediately")
        .expect("already-sealed record should still be readable");

        assert_eq!(waited, record);
    }

    #[tokio::test]
    async fn wait_for_retained_completion_rejects_unregistered_targets() {
        let runtime_state = RuntimeState::new();
        let link = ControlLink {
            instance_id: ash_core::WorkflowId::new(),
        };

        let error = timeout(
            Duration::from_millis(50),
            runtime_state.wait_for_retained_completion(&link),
        )
        .await
        .expect("unregistered completion wait should not hang")
        .expect_err("unregistered completion wait should not synthesize a retained record");

        assert!(matches!(
            error,
            crate::control_link::ControlLinkError::NotFound(not_found_id)
                if not_found_id == link.instance_id
        ));
    }
}
