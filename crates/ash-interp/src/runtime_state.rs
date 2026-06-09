//! Shared runtime-owned state for interpreter executions.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex as StdMutex};

use async_trait::async_trait;
use tokio::sync::Mutex as AsyncMutex;

use ash_core::capability::CapabilityError;
use ash_core::runtime::{
    CapabilityBinding, CapabilityBindingDependency, CapabilityBindingId, CapabilityBindingKind,
    CapabilityImplementationId, CapabilityInterfaceId, ProcessId, ProcessTerminalState, ResourceId,
    ResourceInstance, ResourceLifecycle, ResourceOwner, ResourceProvenance,
    ResourceSplitJoinPolicy, ResourceTypeId,
};
use ash_core::{ControlLink, Effect, Expr, Value, Workflow, WorkflowId};

use crate::capability::CapabilityProvider;
use crate::control_link::{
    ConservativeRetainedEffectSummary, ConservativeRetainedObligationsSummary,
    ConservativeRetainedProvenanceSummary, ControlLinkRegistry, LinkState,
    RetainedCompletionRecord, RetainedCompletionWaiter,
};
use crate::{ExecError, ExecResult};

use crate::execution_record::{ExecutionAdmissionFacts, ExecutionRecord};
use crate::process_registry::{ProcessRecord, ProcessRegistry, ProcessRegistryError};
use crate::proxy_registry::ProxyRegistry;
use crate::runtime_outcome_state::RuntimeOutcomeState;
use crate::yield_routing::YieldRouter;
use crate::yield_state::SuspendedYields;
use std::time::Duration;

pub(crate) const SPAWNED_CHILD_CONTROL_BINDING: &str = "__ash_spawn_control_link";

mod implementation;
pub use implementation::{
    ImplementationBindingAdmission, ImplementationBindingDependencySource,
    ImplementationOperationBody,
};

/// Standard internal resource pilot that Phase 104 can admit without host authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StandardPilotResource {
    /// Workflow-local opaque key/value resource pilot.
    WorkflowKv,
    /// Deterministic frozen/test clock resource pilot.
    FrozenClock,
}

impl StandardPilotResource {
    /// Static runtime resource type identifier used by this pilot.
    #[must_use]
    pub fn resource_type_id(self) -> ResourceTypeId {
        match self {
            Self::WorkflowKv => ResourceTypeId::new("WorkflowKV"),
            Self::FrozenClock => ResourceTypeId::new("FrozenClock"),
        }
    }

    fn provenance_note(self) -> &'static str {
        match self {
            Self::WorkflowKv => "standard WorkflowKV pilot admitted by runtime",
            Self::FrozenClock => "standard FrozenClock pilot admitted by runtime",
        }
    }
}

/// Host-facing request to admit one standard internal capability/resource pilot.
#[derive(Debug, Clone, PartialEq)]
pub struct StandardInternalPilot {
    binding_name: String,
    resource_name: String,
    resource: StandardPilotResource,
    interface: CapabilityInterfaceId,
    implementation: CapabilityImplementationId,
    operation: String,
    fixture: Value,
}

impl StandardInternalPilot {
    /// Create the standard WorkflowKV pilot.
    #[must_use]
    pub fn workflow_kv(
        binding_name: impl Into<String>,
        resource_name: impl Into<String>,
        fixture: Value,
    ) -> Self {
        Self {
            binding_name: binding_name.into(),
            resource_name: resource_name.into(),
            resource: StandardPilotResource::WorkflowKv,
            interface: CapabilityInterfaceId::new("KeyValue"),
            implementation: CapabilityImplementationId::new("__ash_standard_pilot.WorkflowKV"),
            operation: "get".to_string(),
            fixture,
        }
    }

    /// Create the standard FrozenClock/TestClock pilot.
    #[must_use]
    pub fn frozen_clock(
        binding_name: impl Into<String>,
        resource_name: impl Into<String>,
        frozen_epoch_millis: i64,
    ) -> Self {
        Self {
            binding_name: binding_name.into(),
            resource_name: resource_name.into(),
            resource: StandardPilotResource::FrozenClock,
            interface: CapabilityInterfaceId::new("Clock"),
            implementation: CapabilityImplementationId::new("__ash_standard_pilot.FrozenClock"),
            operation: "epoch_millis".to_string(),
            fixture: Value::Int(frozen_epoch_millis),
        }
    }
}

/// Identity pair returned by standard internal pilot admission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StandardPilotBinding {
    /// Admitted implementation-backed binding id.
    pub binding_id: CapabilityBindingId,
    /// Admitted internal resource id that backs the binding authority.
    pub resource_id: ResourceId,
}

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
            .field("name", &self.inner.name())
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ProviderAdmissionSurface {
    Provider,
    Actions(HashSet<String>),
}

impl ProviderAdmissionSurface {
    fn from_capabilities(provider_name: &str, admitted_capabilities: &[String]) -> Option<Self> {
        let mut admitted_actions = HashSet::new();

        for capability in admitted_capabilities {
            if capability == provider_name || capability == &format!("{provider_name}.*") {
                return Some(Self::Provider);
            }

            for separator in ['.', ':'] {
                if let Some((candidate_provider, action_name)) = capability.split_once(separator)
                    && candidate_provider == provider_name
                    && !action_name.is_empty()
                {
                    admitted_actions.insert(action_name.to_string());
                }
            }
        }

        if admitted_actions.is_empty() {
            None
        } else {
            Some(Self::Actions(admitted_actions))
        }
    }

    fn allows_observe(&self) -> bool {
        matches!(self, Self::Provider)
    }

    fn allows_action(&self, action_name: &str) -> bool {
        match self {
            Self::Provider => true,
            Self::Actions(actions) => actions.contains(action_name),
        }
    }
}

fn normalized_action_grant(provider_name: &str, grant: &str) -> String {
    if grant == provider_name {
        return format!("{provider_name}.*");
    }
    if let Some((candidate_provider, action_name)) = grant.split_once('.')
        && candidate_provider == provider_name
        && !action_name.is_empty()
    {
        return format!("{candidate_provider}.{action_name}");
    }
    if let Some((candidate_provider, action_name)) = grant.split_once(':')
        && candidate_provider == provider_name
        && !action_name.is_empty()
    {
        return format!("{candidate_provider}.{action_name}");
    }
    grant.to_string()
}

#[derive(Debug, Clone)]
struct ProjectedProviderWrapper {
    inner: Arc<dyn CapabilityProvider>,
    provider_name: String,
    projected_name: String,
    surface: ProviderAdmissionSurface,
}

impl ProjectedProviderWrapper {
    fn new(
        inner: Arc<dyn CapabilityProvider>,
        provider_name: String,
        projected_name: String,
        surface: ProviderAdmissionSurface,
    ) -> Self {
        Self {
            inner,
            provider_name,
            projected_name,
            surface,
        }
    }

    fn with_projected_name(mut self, projected_name: String) -> Self {
        self.projected_name = projected_name;
        self
    }
}

#[async_trait]
impl ash_core::capability::CapabilityProvider for ArcProviderWrapper {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn effect(&self) -> Effect {
        self.inner.effect()
    }

    async fn observe(
        &self,
        constraints: &[ash_core::Constraint],
    ) -> Result<Value, ash_core::capability::CapabilityError> {
        self.inner.observe(constraints).await
    }

    async fn execute(
        &self,
        action_name: &str,
        args: &[Value],
    ) -> Result<Value, ash_core::capability::CapabilityError> {
        self.inner.execute(action_name, args).await
    }
}

#[async_trait]
impl ash_core::capability::CapabilityProvider for ProjectedProviderWrapper {
    fn name(&self) -> &str {
        &self.projected_name
    }

    fn effect(&self) -> Effect {
        self.inner.effect()
    }

    async fn observe(
        &self,
        constraints: &[ash_core::Constraint],
    ) -> Result<Value, ash_core::capability::CapabilityError> {
        if self.surface.allows_observe() {
            self.inner.observe(constraints).await
        } else {
            Err(CapabilityError::NotAvailable(self.provider_name.clone()))
        }
    }

    async fn execute(
        &self,
        action_name: &str,
        args: &[Value],
    ) -> Result<Value, ash_core::capability::CapabilityError> {
        if self.surface.allows_action(action_name) {
            self.inner.execute(action_name, args).await
        } else {
            Err(CapabilityError::NotAvailable(format!(
                "{}.{}",
                self.provider_name, action_name
            )))
        }
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
#[derive(Debug, Clone, PartialEq)]
pub struct RegisteredCallableWorkflow {
    /// Workflow body executed when this callable target is invoked.
    pub workflow: Workflow,
    /// Expected argument count for the currently registered runtime-call path.
    pub arity: usize,
    /// Parameter names in declaration order, used to bind call-site arguments.
    pub params: Vec<String>,
}

/// Explicit metadata for admitting one resource owned by a workflow `owns` clause.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowOwnedResourceAdmission {
    /// Workflow-local owned resource name.
    pub name: String,
    /// Static resource type identifier.
    pub type_id: ResourceTypeId,
}

impl WorkflowOwnedResourceAdmission {
    /// Create owned resource admission metadata from a workflow-local name and resource type.
    #[must_use]
    pub fn new(name: impl Into<String>, type_id: ResourceTypeId) -> Self {
        Self {
            name: name.into(),
            type_id,
        }
    }
}

mod resource_admission;
pub use resource_admission::ResourceSplitJoinViolation;

#[derive(Clone, Default)]
pub struct RuntimeState {
    control_registry: Arc<AsyncMutex<ControlLinkRegistry>>,
    proxy_registry: Arc<AsyncMutex<ProxyRegistry>>,
    suspended_yields: Arc<AsyncMutex<SuspendedYields>>,
    yield_router: Arc<AsyncMutex<YieldRouter>>,
    child_workflows: Arc<AsyncMutex<HashMap<String, Workflow>>>,
    callable_workflows: Arc<AsyncMutex<HashMap<String, RegisteredCallableWorkflow>>>,
    process_registry: Arc<AsyncMutex<ProcessRegistry>>,
    resource_instances: Arc<AsyncMutex<HashMap<ResourceId, ResourceInstance>>>,
    capability_bindings: Arc<AsyncMutex<HashMap<CapabilityBindingId, CapabilityBinding>>>,
    capability_interface_operations:
        Arc<AsyncMutex<HashMap<CapabilityInterfaceId, HashSet<String>>>>,
    implementation_operation_bodies:
        Arc<AsyncMutex<HashMap<(CapabilityImplementationId, String), ImplementationOperationBody>>>,
    last_execution_record: Arc<AsyncMutex<Option<ExecutionRecord>>>,
    /// Capability provider registry for execution
    providers: Arc<StdMutex<HashMap<String, Arc<dyn CapabilityProvider>>>>,
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
                "callable_workflows",
                &"<HashMap<String, RegisteredCallableWorkflow>>",
            )
            .field("process_registry", &self.process_registry)
            .field(
                "resource_instances",
                &"<HashMap<ResourceId, ResourceInstance>>",
            )
            .field(
                "capability_bindings",
                &"<HashMap<CapabilityBindingId, CapabilityBinding>>",
            )
            .field(
                "capability_interface_operations",
                &"<HashMap<CapabilityInterfaceId, HashSet<String>>>",
            )
            .field("last_execution_record", &self.last_execution_record)
            .field(
                "providers",
                &"<HashMap<String, Arc<dyn CapabilityProvider>>>",
            )
            .finish()
    }
}

impl RuntimeState {
    pub(crate) async fn implementation_binding_dependency_context(
        &self,
        binding: &CapabilityBinding,
    ) -> ExecResult<(
        crate::capability::CapabilityContext,
        HashMap<String, Value>,
        Vec<CapabilityBindingId>,
    )> {
        use crate::capability::{CapabilityContext, CapabilityRegistry};

        let mut values = HashMap::new();
        let mut registry = CapabilityRegistry::new();
        let mut admitted_capability_dependencies = Vec::new();

        for dependency in &binding.dependencies {
            match dependency {
                CapabilityBindingDependency::Capability {
                    name, binding_id, ..
                } => {
                    let source = self.capability_binding(*binding_id).await.ok_or_else(|| {
                        ExecError::InvalidRuntimeState(format!(
                            "capability dependency '{name}' references unknown binding {binding_id:?}"
                        ))
                    })?;
                    values.insert(name.clone(), Value::String(name.clone()));
                    admitted_capability_dependencies.push(*binding_id);

                    let CapabilityBindingKind::HostProvider {
                        provider_name,
                        admitted_capabilities,
                    } = &source.kind
                    else {
                        values.insert(
                            format!("__ash_capability_dependency_alias:{name}"),
                            Value::String(source.name.clone()),
                        );
                        continue;
                    };

                    let providers = self
                        .providers
                        .lock()
                        .expect("provider registry mutex poisoned");
                    let provider = providers
                        .get(provider_name)
                        .ok_or_else(|| ExecError::CapabilityNotAvailable(provider_name.clone()))?;
                    let mut dependency_capabilities = admitted_capabilities.clone();
                    if source.name != *provider_name {
                        dependency_capabilities.extend(admitted_capabilities.iter().filter_map(
                            |grant| {
                                grant
                                    .strip_prefix(&format!("{}.", source.name))
                                    .map(|action| format!("{provider_name}.{action}"))
                                    .or_else(|| {
                                        grant
                                            .strip_prefix(&format!("{}:", source.name))
                                            .map(|action| format!("{provider_name}:{action}"))
                                    })
                            },
                        ));
                    }
                    let surface = ProviderAdmissionSurface::from_capabilities(
                        provider_name,
                        &dependency_capabilities,
                    )
                    .ok_or_else(|| {
                        ExecError::InvalidRuntimeState(format!(
                            "capability dependency '{name}' exposes no admitted provider surface"
                        ))
                    })?;
                    let wrapper = ProjectedProviderWrapper::new(
                        provider.clone(),
                        provider_name.clone(),
                        source.name.clone(),
                        surface,
                    );
                    registry.register(Box::new(wrapper.clone()));
                    if source.name != *provider_name {
                        registry.register(Box::new(
                            wrapper.clone().with_projected_name(provider_name.clone()),
                        ));
                    }
                    if name != &source.name && name != provider_name {
                        registry.register(Box::new(wrapper.with_projected_name(name.clone())));
                    }
                    values.insert(
                        format!("__ash_capability_dependency_alias:{name}"),
                        Value::String(name.clone()),
                    );
                    values.insert(
                        format!("__ash_capability_dependency_local:{name}"),
                        Value::Bool(true),
                    );
                }
                CapabilityBindingDependency::Resource { .. } => {}
                CapabilityBindingDependency::Config { name, value } => {
                    values.insert(name.clone(), value.clone());
                }
            }
        }

        Ok((
            CapabilityContext::with_registry(registry),
            values,
            admitted_capability_dependencies,
        ))
    }

    /// Create a new empty runtime state.
    pub fn new() -> Self {
        Self {
            control_registry: Arc::new(AsyncMutex::new(ControlLinkRegistry::new())),
            proxy_registry: Arc::new(AsyncMutex::new(ProxyRegistry::new())),
            suspended_yields: Arc::new(AsyncMutex::new(SuspendedYields::new())),
            yield_router: Arc::new(AsyncMutex::new(YieldRouter::new())),
            child_workflows: Arc::new(AsyncMutex::new(HashMap::new())),
            callable_workflows: Arc::new(AsyncMutex::new(HashMap::new())),
            process_registry: Arc::new(AsyncMutex::new(ProcessRegistry::new())),
            resource_instances: Arc::new(AsyncMutex::new(HashMap::new())),
            capability_bindings: Arc::new(AsyncMutex::new(HashMap::new())),
            capability_interface_operations: Arc::new(AsyncMutex::new(HashMap::new())),
            implementation_operation_bodies: Arc::new(AsyncMutex::new(HashMap::new())),
            last_execution_record: Arc::new(AsyncMutex::new(None)),
            providers: Arc::new(StdMutex::new(HashMap::new())),
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

    /// Register a runtime-owned callable workflow entry keyed by name.
    ///
    /// This registry is used by `Workflow::Call` / `Stmt::Call` execution.
    pub async fn register_callable_workflow(
        &self,
        workflow_name: impl Into<String>,
        workflow: Workflow,
        arity: usize,
        params: Vec<String>,
    ) {
        self.callable_workflows.lock().await.insert(
            workflow_name.into(),
            RegisteredCallableWorkflow {
                workflow,
                arity,
                params,
            },
        );
    }

    /// Blocking version of [`Self::register_callable_workflow`] for use from
    /// synchronous call sites (e.g., the engine's `parse` method).
    ///
    /// Uses `std::sync::Mutex` internally to avoid tokio runtime conflicts.
    pub fn blocking_register_callable_workflow(
        &self,
        workflow_name: impl Into<String>,
        workflow: Workflow,
        arity: usize,
        params: Vec<String>,
    ) {
        // tokio::sync::Mutex::try_lock works outside of async context.
        // Inside a tokio runtime, we must avoid blocking_lock().
        // Use try_lock which is non-blocking.
        if let Ok(mut guard) = self.callable_workflows.try_lock() {
            guard.insert(
                workflow_name.into(),
                RegisteredCallableWorkflow {
                    workflow,
                    arity,
                    params,
                },
            );
        } else {
            // Fallback: acquire the async mutex from a plain thread so this
            // remains safe on current-thread runtimes where `block_in_place`
            // would panic.
            let map = self.callable_workflows.clone();
            let name = workflow_name.into();
            let entry = RegisteredCallableWorkflow {
                workflow,
                arity,
                params,
            };
            std::thread::spawn(move || {
                futures::executor::block_on(async move {
                    map.lock().await.insert(name, entry);
                });
            })
            .join()
            .expect("blocking callable workflow registration thread panicked");
        }
    }

    /// Look up a runtime-owned callable workflow entry by name.
    pub async fn callable_workflow(
        &self,
        workflow_name: &str,
    ) -> Option<RegisteredCallableWorkflow> {
        self.callable_workflows
            .lock()
            .await
            .get(workflow_name)
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
        self.providers
            .lock()
            .expect("provider registry mutex poisoned")
            .insert(name.into(), provider);
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
        self.providers
            .lock()
            .expect("provider registry mutex poisoned")
            .extend(providers);
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
        self.providers
            .lock()
            .expect("provider registry mutex poisoned")
            .get(name)
            .cloned()
    }

    /// Get all registered provider names.
    ///
    /// Returns a vector of all provider names currently registered.
    pub fn provider_names(&self) -> Vec<String> {
        self.providers
            .lock()
            .expect("provider registry mutex poisoned")
            .keys()
            .cloned()
            .collect()
    }

    /// Check if a provider is registered.
    ///
    /// Returns `true` if a provider with the given name is registered.
    pub fn has_provider(&self, name: &str) -> bool {
        self.providers
            .lock()
            .expect("provider registry mutex poisoned")
            .contains_key(name)
    }

    /// Get the number of registered providers.
    ///
    /// Returns the count of providers currently registered.
    pub fn provider_count(&self) -> usize {
        self.providers
            .lock()
            .expect("provider registry mutex poisoned")
            .len()
    }

    /// Create a CapabilityContext from the registered providers.
    ///
    /// This allows the interpreter to access capability providers
    /// during workflow execution.
    pub async fn create_capability_context(&self) -> crate::capability::CapabilityContext {
        use crate::capability::{CapabilityContext, CapabilityRegistry};

        let mut registry = CapabilityRegistry::new();
        let providers = self
            .providers
            .lock()
            .expect("provider registry mutex poisoned");

        for provider in providers.values() {
            let wrapper = Box::new(ArcProviderWrapper::new(provider.clone()));
            registry.register(wrapper);
        }

        CapabilityContext::with_registry(registry)
    }

    /// Create a BehaviourContext backed by runtime capability providers.
    pub async fn create_behaviour_context(&self) -> crate::behaviour::BehaviourContext {
        use crate::behaviour::{BehaviourContext, BehaviourProvider, BehaviourRegistry};
        use crate::typed_provider::TypedBehaviourProvider;
        use ash_typeck::{Type, TypeVar};

        struct RuntimeProviderBehaviour {
            provider: Arc<dyn CapabilityProvider>,
            capability: String,
            channel: String,
        }

        #[async_trait::async_trait]
        impl BehaviourProvider for RuntimeProviderBehaviour {
            fn capability_name(&self) -> &str {
                &self.capability
            }

            fn channel_name(&self) -> &str {
                &self.channel
            }

            async fn sample(&self, constraints: &[ash_core::Constraint]) -> ExecResult<Value> {
                self.provider
                    .observe(constraints)
                    .await
                    .map_err(|error| ExecError::ExecutionFailed(error.to_string()))
            }
        }

        let mut registry = BehaviourRegistry::new();
        let providers = self
            .providers
            .lock()
            .expect("provider registry mutex poisoned");

        for (name, provider) in providers.iter() {
            if let Some((capability, channel)) = name.split_once(':')
                && !capability.is_empty()
                && !channel.is_empty()
            {
                registry.register(TypedBehaviourProvider::new(
                    RuntimeProviderBehaviour {
                        provider: provider.clone(),
                        capability: capability.to_string(),
                        channel: channel.to_string(),
                    },
                    Type::Var(TypeVar::fresh()),
                ));
            }
        }

        BehaviourContext::with_registry(registry)
    }

    /// Create a CapabilityContext projected to the admitted provider/action surface.
    pub async fn create_projected_capability_context(
        &self,
        admitted_capabilities: &[String],
    ) -> crate::capability::CapabilityContext {
        use crate::capability::{CapabilityContext, CapabilityRegistry};

        let mut registry = CapabilityRegistry::new();

        let providers = self
            .providers
            .lock()
            .expect("provider registry mutex poisoned");

        for (name, provider) in providers.iter() {
            let Some(surface) =
                ProviderAdmissionSurface::from_capabilities(name, admitted_capabilities)
            else {
                continue;
            };

            let wrapper = Box::new(ProjectedProviderWrapper::new(
                provider.clone(),
                name.clone(),
                name.clone(),
                surface,
            ));
            registry.register(wrapper);
        }

        CapabilityContext::with_registry(registry)
    }

    /// Admit workflow-owned resources from explicit `owns` metadata.
    ///
    /// Returned resource ids are keyed only by the explicit workflow-local resource names supplied
    /// by the caller; this API does not perform ambient resource lookup.
    pub async fn admit_workflow_owned_resources(
        &self,
        workflow_id: WorkflowId,
        resources: Vec<WorkflowOwnedResourceAdmission>,
    ) -> ExecResult<HashMap<String, ResourceId>> {
        let mut seen_names = HashSet::new();
        for resource in &resources {
            if !seen_names.insert(resource.name.clone()) {
                return Err(ExecError::InvalidRuntimeState(format!(
                    "duplicate owned resource name '{}'",
                    resource.name
                )));
            }
        }

        let mut admitted = HashMap::new();
        let mut instances = Vec::with_capacity(resources.len());
        for resource in resources {
            let id = ResourceId::new();
            let type_name = resource.type_id.as_str().to_string();
            let instance =
                ResourceInstance::new(id, resource.type_id, ResourceOwner::Workflow(workflow_id))
                    .with_lifecycle(ResourceLifecycle::Admitted)
                    .with_provenance(ResourceProvenance::internal(format!(
                        "workflow owns {}: {type_name}",
                        resource.name
                    )));
            admitted.insert(resource.name, id);
            instances.push(instance);
        }

        let mut registry = self.resource_instances.lock().await;
        for instance in instances {
            registry.insert(instance.id, instance);
        }
        Ok(admitted)
    }

    /// Register the canonical runtime operation surface for one capability interface.
    ///
    /// This metadata is normally sourced from already typechecked capability-interface declarations.
    /// Runtime admission uses it as the authority for metadata-only non-widening checks instead of
    /// trusting caller-supplied operation lists on individual implementation binding admissions.
    pub async fn register_capability_interface_operations<I, S>(
        &self,
        interface: CapabilityInterfaceId,
        operations: I,
    ) -> ExecResult<()>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let operations = operations
            .into_iter()
            .map(Into::into)
            .collect::<HashSet<_>>();
        if operations.is_empty() {
            return Err(ExecError::InvalidRuntimeState(format!(
                "capability interface {} must declare at least one operation",
                interface.as_str()
            )));
        }

        let mut interfaces = self.capability_interface_operations.lock().await;
        if interfaces.contains_key(&interface) {
            return Err(ExecError::InvalidRuntimeState(format!(
                "capability interface {} operation surface already registered",
                interface.as_str()
            )));
        }
        interfaces.insert(interface, operations);
        Ok(())
    }

    /// Return the registered operation surface for one capability interface.
    pub async fn capability_interface_operations(
        &self,
        interface: &CapabilityInterfaceId,
    ) -> Option<HashSet<String>> {
        self.capability_interface_operations
            .lock()
            .await
            .get(interface)
            .cloned()
    }

    /// Register one callable Ash-defined implementation operation body.
    pub async fn register_implementation_operation_body(
        &self,
        implementation: CapabilityImplementationId,
        operation: impl Into<String>,
        body: ImplementationOperationBody,
    ) -> ExecResult<()> {
        let key = (implementation, operation.into());
        let mut bodies = self.implementation_operation_bodies.lock().await;
        if bodies.contains_key(&key) {
            return Err(ExecError::InvalidRuntimeState(format!(
                "implementation operation body for {}.{} already registered",
                key.0.as_str(),
                key.1
            )));
        }
        bodies.insert(key, body);
        Ok(())
    }

    /// Look up one registered Ash-defined implementation operation body.
    pub async fn implementation_operation_body(
        &self,
        implementation: &CapabilityImplementationId,
        operation: &str,
    ) -> Option<ImplementationOperationBody> {
        self.implementation_operation_bodies
            .lock()
            .await
            .get(&(implementation.clone(), operation.to_string()))
            .cloned()
    }

    /// Admit one standard internal pilot resource and its implementation-backed binding.
    ///
    /// The pilot creates an internal runtime-owned resource, registers a deterministic
    /// Ash-defined implementation operation body backed by an inert config value, and then
    /// admits an implementation binding whose authority derives only from the explicit resource
    /// dependency. It does not expose the resource as a first-class Ash value and does not perform
    /// ambient capability/resource lookup.
    ///
    /// # Errors
    ///
    /// Returns an error if interface operation validation, implementation body registration, or
    /// implementation binding admission fails.
    pub async fn admit_standard_internal_pilot(
        &self,
        pilot: StandardInternalPilot,
    ) -> ExecResult<StandardPilotBinding> {
        if self
            .capability_interface_operations(&pilot.interface)
            .await
            .is_none()
        {
            self.register_capability_interface_operations(
                pilot.interface.clone(),
                [pilot.operation.as_str()],
            )
            .await?;
        }

        if self
            .implementation_operation_body(&pilot.implementation, &pilot.operation)
            .await
            .is_some()
        {
            return Err(ExecError::InvalidRuntimeState(format!(
                "standard internal pilot body {}.{} is already registered",
                pilot.implementation.as_str(),
                pilot.operation
            )));
        }

        let params = match pilot.resource {
            StandardPilotResource::WorkflowKv => vec!["__ash_standard_pilot_arg"],
            StandardPilotResource::FrozenClock => Vec::new(),
        };
        self.register_implementation_operation_body(
            pilot.implementation.clone(),
            pilot.operation.clone(),
            ImplementationOperationBody::new(
                params,
                Expr::Variable {
                    name: "__ash_standard_pilot_fixture".to_string(),
                    span: Default::default(),
                },
            ),
        )
        .await?;

        let resource_id = ResourceId::new();
        let resource = ResourceInstance::new(
            resource_id,
            pilot.resource.resource_type_id(),
            ResourceOwner::Workflow(WorkflowId::new()),
        )
        .with_lifecycle(ResourceLifecycle::Admitted)
        .with_provenance(ResourceProvenance::internal(
            pilot.resource.provenance_note(),
        ));
        self.register_resource_instance(resource).await;

        let mut resources = HashMap::new();
        resources.insert(pilot.resource_name.clone(), resource_id);
        let binding_id = self
            .admit_implementation_binding(
                ImplementationBindingAdmission::new(
                    pilot.binding_name,
                    pilot.interface,
                    pilot.implementation,
                )
                .with_dependency(ImplementationBindingDependencySource::resource(
                    pilot.resource_name,
                    pilot.resource.resource_type_id(),
                ))
                .with_dependency(ImplementationBindingDependencySource::config(
                    "__ash_standard_pilot_fixture",
                    pilot.fixture,
                ))
                .with_requested_operations([pilot.operation]),
                &resources,
            )
            .await?;

        Ok(StandardPilotBinding {
            binding_id,
            resource_id,
        })
    }

    /// Admit one implementation-backed capability binding from explicit dependency source names.
    ///
    /// Resource dependencies are resolved only from `resource_sources`, and capability dependencies
    /// are resolved only by previously admitted binding names. The resulting binding is admitted via
    /// [`Self::admit_capability_binding`] so existing dependency validation remains authoritative.
    pub async fn admit_implementation_binding(
        &self,
        admission: ImplementationBindingAdmission,
        resource_sources: &HashMap<String, ResourceId>,
    ) -> ExecResult<CapabilityBindingId> {
        if !admission.requested_operations.is_empty() {
            let Some(registered_operations) = self
                .capability_interface_operations(&admission.interface)
                .await
            else {
                return Err(ExecError::InvalidRuntimeState(format!(
                    "cannot validate requested operations for unregistered interface {}",
                    admission.interface.as_str()
                )));
            };
            let allowed_operations: HashSet<&str> =
                registered_operations.iter().map(String::as_str).collect();
            for operation in &admission.requested_operations {
                if !allowed_operations.contains(operation.as_str()) {
                    return Err(ExecError::InvalidRuntimeState(format!(
                        "requested operation '{operation}' is outside registered interface {}",
                        admission.interface.as_str()
                    )));
                }
            }
        }

        let mut dependencies = Vec::with_capacity(admission.dependencies.len());
        for dependency in admission.dependencies {
            match dependency {
                ImplementationBindingDependencySource::Resource { name, type_id } => {
                    let Some(resource_id) = resource_sources.get(&name).copied() else {
                        return Err(ExecError::InvalidRuntimeState(format!(
                            "missing resource dependency source '{name}'"
                        )));
                    };
                    dependencies.push(CapabilityBindingDependency::Resource {
                        name,
                        resource_id,
                        type_id,
                    });
                }
                ImplementationBindingDependencySource::Capability {
                    name,
                    source_binding_name,
                    interface,
                } => {
                    let Some(binding) = self.capability_binding_by_name(&source_binding_name).await
                    else {
                        return Err(ExecError::InvalidRuntimeState(format!(
                            "missing capability dependency source '{source_binding_name}'"
                        )));
                    };
                    dependencies.push(CapabilityBindingDependency::Capability {
                        name,
                        binding_id: binding.id,
                        interface,
                    });
                }
                ImplementationBindingDependencySource::Config { name, value } => {
                    dependencies.push(CapabilityBindingDependency::Config { name, value });
                }
            }
        }

        let binding = CapabilityBinding::implementation(
            CapabilityBindingId::new(),
            admission.name,
            admission.interface,
            admission.implementation,
            dependencies,
        );
        let binding = admission
            .requested_operations
            .iter()
            .fold(binding, |binding, operation| {
                let source_names = binding
                    .dependencies
                    .iter()
                    .filter(|dependency| dependency.carries_authority())
                    .map(|dependency| dependency.name().to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                binding.with_authority_note(format!(
                    "operation {operation} derives from {source_names}"
                ))
            });
        self.admit_capability_binding(binding).await
    }

    /// Admit one host- or implementation-backed capability binding into runtime state.
    ///
    /// Host bindings require an already-registered provider. Implementation bindings store
    /// dependency metadata only and require resource/capability dependencies to have been admitted
    /// explicitly beforehand.
    pub async fn admit_capability_binding(
        &self,
        binding: CapabilityBinding,
    ) -> ExecResult<CapabilityBindingId> {
        Self::validate_capability_binding_authority(&binding)?;

        match &binding.kind {
            CapabilityBindingKind::HostProvider { provider_name, .. } => {
                if !self.has_provider(provider_name) {
                    return Err(ExecError::CapabilityNotAvailable(provider_name.clone()));
                }
            }
            CapabilityBindingKind::Implementation { .. } => {
                if !binding
                    .dependencies
                    .iter()
                    .any(CapabilityBindingDependency::carries_authority)
                {
                    return Err(ExecError::InvalidRuntimeState(
                        "implementation capability binding must derive authority from at least one resource or capability dependency"
                            .to_string(),
                    ));
                }
                self.validate_capability_binding_dependencies(&binding.dependencies)
                    .await?;
            }
        }

        let id = binding.id;
        let mut bindings = self.capability_bindings.lock().await;
        if bindings.contains_key(&id) {
            return Err(ExecError::InvalidRuntimeState(format!(
                "duplicate capability binding id {id:?}"
            )));
        }
        if bindings
            .values()
            .any(|existing| existing.name == binding.name)
        {
            return Err(ExecError::InvalidRuntimeState(format!(
                "duplicate capability binding name '{}'",
                binding.name
            )));
        }
        bindings.insert(id, binding);
        Ok(id)
    }

    /// Validate that a binding kind and authority provenance agree.
    fn validate_capability_binding_authority(binding: &CapabilityBinding) -> ExecResult<()> {
        match (&binding.kind, &binding.authority) {
            (
                CapabilityBindingKind::HostProvider { .. },
                ash_core::CapabilityAuthorityProvenance::HostAuthority { .. },
            ) => Ok(()),
            (
                CapabilityBindingKind::Implementation { .. },
                ash_core::CapabilityAuthorityProvenance::DerivedAuthority { .. },
            ) => Ok(()),
            (CapabilityBindingKind::HostProvider { .. }, _) => Err(ExecError::InvalidRuntimeState(
                "host capability binding must carry host authority provenance".to_string(),
            )),
            (CapabilityBindingKind::Implementation { .. }, _) => {
                Err(ExecError::InvalidRuntimeState(
                    "implementation capability binding must carry derived authority provenance"
                        .to_string(),
                ))
            }
        }
    }

    async fn validate_capability_binding_dependencies(
        &self,
        dependencies: &[CapabilityBindingDependency],
    ) -> ExecResult<()> {
        let bindings = self.capability_bindings.lock().await;
        let resources = self.resource_instances.lock().await;

        for dependency in dependencies {
            match dependency {
                CapabilityBindingDependency::Resource {
                    name,
                    resource_id,
                    type_id,
                } => match resources.get(resource_id) {
                    Some(resource) if &resource.type_id == type_id => {}
                    Some(_) => {
                        return Err(ExecError::InvalidRuntimeState(format!(
                            "resource dependency '{name}' has mismatched type"
                        )));
                    }
                    None => {
                        return Err(ExecError::InvalidRuntimeState(format!(
                            "missing resource dependency '{name}'"
                        )));
                    }
                },
                CapabilityBindingDependency::Capability {
                    name,
                    binding_id,
                    interface,
                } => match bindings.get(binding_id) {
                    Some(binding) if &binding.interface == interface => {}
                    Some(_) => {
                        return Err(ExecError::InvalidRuntimeState(format!(
                            "capability dependency '{name}' has mismatched interface"
                        )));
                    }
                    None => {
                        return Err(ExecError::InvalidRuntimeState(format!(
                            "missing capability dependency '{name}'"
                        )));
                    }
                },
                CapabilityBindingDependency::Config { .. } => {}
            }
        }

        Ok(())
    }

    /// Look up one admitted capability binding by identity.
    pub async fn capability_binding(
        &self,
        binding_id: CapabilityBindingId,
    ) -> Option<CapabilityBinding> {
        self.capability_bindings
            .lock()
            .await
            .get(&binding_id)
            .cloned()
    }

    /// Look up one admitted capability binding by runtime binding name.
    pub async fn capability_binding_by_name(&self, name: &str) -> Option<CapabilityBinding> {
        self.capability_bindings
            .lock()
            .await
            .values()
            .find(|binding| binding.name == name)
            .cloned()
    }

    /// Return true if a capability binding has been explicitly admitted.
    pub async fn has_capability_binding(&self, binding_id: CapabilityBindingId) -> bool {
        self.capability_bindings
            .lock()
            .await
            .contains_key(&binding_id)
    }

    /// Return the number of admitted capability bindings.
    pub async fn capability_binding_count(&self) -> usize {
        self.capability_bindings.lock().await.len()
    }

    /// Create a capability context from explicit admitted binding identities.
    ///
    /// Host-provider bindings project their admitted provider/action surface into the existing
    /// `CapabilityContext` compatibility layer. Implementation bindings remain metadata-only and do
    /// not register executable providers.
    pub async fn create_capability_context_for_bindings(
        &self,
        binding_ids: &[CapabilityBindingId],
    ) -> ExecResult<crate::capability::CapabilityContext> {
        use crate::capability::{CapabilityContext, CapabilityRegistry};

        let bindings = self.capability_bindings.lock().await;
        let mut projected_surfaces = Vec::new();

        for binding_id in binding_ids {
            let Some(binding) = bindings.get(binding_id) else {
                return Err(ExecError::InvalidRuntimeState(format!(
                    "unadmitted capability binding {binding_id:?}"
                )));
            };

            if let CapabilityBindingKind::HostProvider {
                provider_name,
                admitted_capabilities,
            } = &binding.kind
            {
                projected_surfaces.push((
                    provider_name.clone(),
                    binding.name.clone(),
                    admitted_capabilities.clone(),
                ));
            }
        }
        drop(bindings);

        let providers = self
            .providers
            .lock()
            .expect("provider registry mutex poisoned");
        let mut registry = CapabilityRegistry::new();

        for (provider_name, projected_name, mut admitted_capabilities) in projected_surfaces {
            let Some(provider) = providers.get(&provider_name) else {
                return Err(ExecError::CapabilityNotAvailable(provider_name));
            };
            admitted_capabilities.sort();
            admitted_capabilities.dedup();
            let projected_capabilities = admitted_capabilities
                .iter()
                .map(|grant| {
                    if grant == &format!("{provider_name}.*")
                        || grant == &format!("{projected_name}.*")
                    {
                        return projected_name.clone();
                    }
                    if grant == &provider_name || grant == &projected_name {
                        return projected_name.clone();
                    }
                    if let Some((candidate_provider, action_name)) = grant.split_once('.')
                        && (candidate_provider == provider_name
                            || candidate_provider == projected_name)
                    {
                        return format!("{projected_name}.{action_name}");
                    }
                    if let Some((candidate_provider, action_name)) = grant.split_once(':')
                        && (candidate_provider == provider_name
                            || candidate_provider == projected_name)
                    {
                        return format!("{projected_name}:{action_name}");
                    }
                    grant.clone()
                })
                .collect::<Vec<_>>();
            let Some(surface) = ProviderAdmissionSurface::from_capabilities(
                &projected_name,
                &projected_capabilities,
            ) else {
                continue;
            };
            let wrapper = ProjectedProviderWrapper::new(
                provider.clone(),
                provider_name.clone(),
                projected_name.clone(),
                surface,
            );
            registry.register(Box::new(wrapper));
        }

        Ok(CapabilityContext::with_registry(registry))
    }

    /// Return all currently admitted capability binding identities.
    pub async fn admitted_capability_binding_ids(&self) -> Vec<CapabilityBindingId> {
        let mut binding_ids = self
            .capability_bindings
            .lock()
            .await
            .keys()
            .copied()
            .collect::<Vec<_>>();
        binding_ids.sort_unstable_by_key(|binding_id| binding_id.0);
        binding_ids
    }

    /// Resolve requested provider/action grant strings to explicit admitted binding identities.
    pub async fn resolve_admitted_capability_bindings(
        &self,
        admitted_capabilities: &[String],
    ) -> Vec<CapabilityBindingId> {
        let bindings = self.capability_bindings.lock().await;
        let mut binding_ids = Vec::new();
        for binding in bindings.values() {
            let CapabilityBindingKind::HostProvider {
                provider_name,
                admitted_capabilities: binding_capabilities,
            } = &binding.kind
            else {
                continue;
            };
            let grants_requested_surface = admitted_capabilities.iter().any(|requested| {
                let normalized_request = normalized_action_grant(provider_name, requested);
                binding.name == *requested
                    || provider_name == requested
                    || binding_capabilities.iter().any(|grant| {
                        let normalized_grant = normalized_action_grant(provider_name, grant);
                        normalized_grant == normalized_request
                            || (normalized_grant == format!("{provider_name}.*")
                                && normalized_request.starts_with(&format!("{provider_name}.")))
                    })
            });
            if grants_requested_surface {
                binding_ids.push(binding.id);
            }
        }
        binding_ids.sort_unstable_by_key(|binding_id| binding_id.0);
        binding_ids.dedup();
        binding_ids
    }

    /// Build audit facts for the projected alpha admission grant set.
    pub async fn execution_admission_facts(
        &self,
        binding_ids: &[CapabilityBindingId],
    ) -> ExecutionAdmissionFacts {
        let bindings = self.capability_bindings.lock().await;
        let mut capability_binding_grants = Vec::new();
        let mut resource_grants = Vec::new();
        let mut action_grants = Vec::new();

        for binding_id in binding_ids {
            let Some(binding) = bindings.get(binding_id) else {
                continue;
            };
            capability_binding_grants.push(format!("{binding_id:?}"));
            match &binding.kind {
                CapabilityBindingKind::HostProvider {
                    provider_name,
                    admitted_capabilities,
                } => {
                    for grant in admitted_capabilities {
                        action_grants.push(normalized_action_grant(provider_name, grant));
                    }
                }
                CapabilityBindingKind::Implementation { .. } => {
                    for dependency in &binding.dependencies {
                        if let CapabilityBindingDependency::Resource { resource_id, .. } =
                            dependency
                        {
                            resource_grants.push(format!("{resource_id:?}"));
                        }
                    }
                }
            }
        }

        capability_binding_grants.sort();
        capability_binding_grants.dedup();
        resource_grants.sort();
        resource_grants.dedup();
        action_grants.sort();
        action_grants.dedup();

        ExecutionAdmissionFacts::new(capability_binding_grants, resource_grants, action_grants)
    }

    pub(crate) fn control_registry(&self) -> Arc<AsyncMutex<ControlLinkRegistry>> {
        self.control_registry.clone()
    }

    /// Register one root process identity in the runtime process registry.
    pub async fn register_root_process(
        &self,
        process_id: ProcessId,
    ) -> Result<(), ProcessRegistryError> {
        self.process_registry.lock().await.register_root(process_id)
    }

    /// Register one child process identity in the runtime process registry.
    pub async fn register_child_process(
        &self,
        parent_process_id: ProcessId,
        child_process_id: ProcessId,
        child_index: usize,
    ) -> Result<(), ProcessRegistryError> {
        self.process_registry.lock().await.register_child(
            parent_process_id,
            child_process_id,
            child_index,
        )
    }

    /// Register multiple child process identities atomically in the runtime process registry.
    pub async fn register_child_processes_batch(
        &self,
        parent_process_id: ProcessId,
        children: Vec<(ProcessId, usize)>,
    ) -> Result<(), ProcessRegistryError> {
        self.process_registry
            .lock()
            .await
            .register_children_batch(parent_process_id, children)
    }

    /// Transition one registered process identity to running.
    pub async fn mark_process_running(
        &self,
        process_id: ProcessId,
    ) -> Result<(), ProcessRegistryError> {
        self.process_registry.lock().await.mark_running(process_id)
    }

    /// Record a write-once terminal process state.
    pub async fn record_process_terminal(
        &self,
        process_id: ProcessId,
        terminal_state: ProcessTerminalState,
    ) -> Result<(), ProcessRegistryError> {
        self.process_registry
            .lock()
            .await
            .record_terminal(process_id, terminal_state)
    }

    /// Look up one process record by identity.
    pub async fn process_record(&self, process_id: ProcessId) -> Option<ProcessRecord> {
        self.process_registry
            .lock()
            .await
            .record(process_id)
            .cloned()
    }

    /// Return one retained terminal process state by identity, if present.
    pub async fn process_terminal_state(
        &self,
        process_id: ProcessId,
    ) -> Option<ProcessTerminalState> {
        self.process_registry
            .lock()
            .await
            .record(process_id)
            .and_then(|record| record.terminal_state.clone())
    }

    /// Wait until a registered process reaches a retained terminal state.
    pub async fn wait_for_process_terminal_state(
        &self,
        process_id: ProcessId,
    ) -> Option<ProcessTerminalState> {
        loop {
            let maybe_state = {
                let registry = self.process_registry.lock().await;
                match registry.record(process_id) {
                    Some(record) => record.terminal_state.clone(),
                    None => return None,
                }
            };
            if maybe_state.is_some() {
                return maybe_state;
            }
            tokio::task::yield_now().await;
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    }

    /// Blocking version of [`Self::process_terminal_state`] for synchronous Proc observation.
    pub fn blocking_process_terminal_state(
        &self,
        process_id: ProcessId,
    ) -> Option<ProcessTerminalState> {
        if let Ok(guard) = self.process_registry.try_lock() {
            return guard
                .record(process_id)
                .and_then(|record| record.terminal_state.clone());
        }

        let registry = self.process_registry.clone();
        std::thread::spawn(move || {
            futures::executor::block_on(async move {
                registry
                    .lock()
                    .await
                    .record(process_id)
                    .and_then(|record| record.terminal_state.clone())
            })
        })
        .join()
        .expect("blocking process terminal-state lookup thread panicked")
    }

    /// Return child process identities for a parent in child-index order.
    pub async fn process_children(&self, parent_process_id: ProcessId) -> Vec<ProcessId> {
        self.process_registry
            .lock()
            .await
            .children_of(parent_process_id)
    }

    /// Register or replace one runtime-owned resource instance by stable identity.
    ///
    /// This stores environment-owned resource metadata only; it does not expose resources as
    /// first-class [`Value`] handles or execute resource-backed capability operations.
    pub async fn register_resource_instance(&self, instance: ResourceInstance) {
        self.resource_instances
            .lock()
            .await
            .insert(instance.id, instance);
    }

    /// Look up one runtime-owned resource instance by identity.
    pub async fn resource_instance(&self, resource_id: ResourceId) -> Option<ResourceInstance> {
        self.resource_instances
            .lock()
            .await
            .get(&resource_id)
            .cloned()
    }

    /// Return true if a resource instance with `resource_id` is registered.
    pub async fn has_resource_instance(&self, resource_id: ResourceId) -> bool {
        self.resource_instances
            .lock()
            .await
            .contains_key(&resource_id)
    }

    /// Return resource instances matching one owner scope and resource type identifier.
    ///
    /// Runtime resource lookup remains scoped by owner to avoid ambient type-only discovery across
    /// unrelated runs, workflows, processes, effect scopes, or test scopes.
    pub async fn resource_instances_for_owner_by_type(
        &self,
        owner: ResourceOwner,
        type_id: ResourceTypeId,
    ) -> Vec<ResourceInstance> {
        self.resource_instances
            .lock()
            .await
            .values()
            .filter(|instance| instance.owner == owner && instance.type_id == type_id)
            .cloned()
            .collect()
    }

    /// Return all resource instances owned by one runtime owner scope.
    pub async fn resource_instances_for_owner(
        &self,
        owner: ResourceOwner,
    ) -> Vec<ResourceInstance> {
        self.resource_instances
            .lock()
            .await
            .values()
            .filter(|instance| instance.owner == owner)
            .cloned()
            .collect()
    }

    /// Check and record process resource policy before a Proc split admits children.
    pub async fn apply_process_resource_split(
        &self,
        parent_process_id: ProcessId,
        child_count: usize,
        operation: &'static str,
    ) -> Result<(), ResourceSplitJoinViolation> {
        if child_count == 0 {
            return Ok(());
        }

        let mut instances = self.resource_instances.lock().await;
        let mut parent_resource_ids = instances
            .iter()
            .filter_map(|(id, instance)| {
                (instance.owner == ResourceOwner::Process(parent_process_id)).then_some(*id)
            })
            .collect::<Vec<_>>();
        parent_resource_ids.sort_by_key(|id| id.0);

        for resource_id in &parent_resource_ids {
            let instance = instances
                .get(resource_id)
                .expect("resource id collected from map must still exist");
            match instance.split_join_policy {
                ResourceSplitJoinPolicy::ReadOnlyShare
                | ResourceSplitJoinPolicy::CommunicationOnly
                | ResourceSplitJoinPolicy::Mergeable => {}
                ResourceSplitJoinPolicy::NonShareable => {
                    return Err(ResourceSplitJoinViolation::new(
                        parent_process_id,
                        operation,
                        instance.clone(),
                        "non-shareable resource cannot cross a process split",
                    ));
                }
                ResourceSplitJoinPolicy::BranchLocalClone => {
                    return Err(ResourceSplitJoinViolation::new(
                        parent_process_id,
                        operation,
                        instance.clone(),
                        "branch-local clone resource has no runtime clone implementation in the MVP",
                    ));
                }
                ResourceSplitJoinPolicy::LinearMove => {
                    return Err(ResourceSplitJoinViolation::new(
                        parent_process_id,
                        operation,
                        instance.clone(),
                        "linear-move resource requires an explicit destination child in the MVP",
                    ));
                }
            }
        }

        for resource_id in parent_resource_ids {
            if let Some(instance) = instances.get_mut(&resource_id)
                && matches!(
                    instance.split_join_policy,
                    ResourceSplitJoinPolicy::ReadOnlyShare
                        | ResourceSplitJoinPolicy::CommunicationOnly
                        | ResourceSplitJoinPolicy::Mergeable
                )
            {
                instance.lifecycle = ResourceLifecycle::Splitting;
            }
        }

        Ok(())
    }

    /// Apply parent-owned resource merge policy after `join`/`gather` observes child success.
    pub async fn apply_process_resource_join(
        &self,
        parent_process_id: ProcessId,
        child_process_ids: &[ProcessId],
        operation: &'static str,
    ) -> Result<(), ResourceSplitJoinViolation> {
        if child_process_ids.is_empty() {
            return Ok(());
        }

        let mut instances = self.resource_instances.lock().await;
        let mut parent_resource_ids = instances
            .iter()
            .filter_map(|(id, instance)| {
                (instance.owner == ResourceOwner::Process(parent_process_id)
                    && instance.lifecycle == ResourceLifecycle::Splitting)
                    .then_some(*id)
            })
            .collect::<Vec<_>>();
        parent_resource_ids.sort_by_key(|id| id.0);

        for resource_id in &parent_resource_ids {
            let instance = instances
                .get(resource_id)
                .expect("resource id collected from map must still exist");
            match instance.split_join_policy {
                ResourceSplitJoinPolicy::Mergeable => {}
                ResourceSplitJoinPolicy::ReadOnlyShare
                | ResourceSplitJoinPolicy::CommunicationOnly => {}
                ResourceSplitJoinPolicy::NonShareable
                | ResourceSplitJoinPolicy::BranchLocalClone
                | ResourceSplitJoinPolicy::LinearMove => {
                    return Err(ResourceSplitJoinViolation::new(
                        parent_process_id,
                        operation,
                        instance.clone(),
                        "split resource cannot be joined by the MVP merge policy",
                    ));
                }
            }
        }

        for resource_id in parent_resource_ids {
            if let Some(instance) = instances.get_mut(&resource_id) {
                match instance.split_join_policy {
                    ResourceSplitJoinPolicy::Mergeable => {
                        instance.lifecycle = ResourceLifecycle::Joined;
                    }
                    ResourceSplitJoinPolicy::ReadOnlyShare
                    | ResourceSplitJoinPolicy::CommunicationOnly => {
                        instance.lifecycle = ResourceLifecycle::Active;
                    }
                    ResourceSplitJoinPolicy::NonShareable
                    | ResourceSplitJoinPolicy::BranchLocalClone
                    | ResourceSplitJoinPolicy::LinearMove => {}
                }
            }
        }

        Ok(())
    }

    /// Return the number of registered resource instances.
    pub async fn resource_instance_count(&self) -> usize {
        self.resource_instances.lock().await.len()
    }

    /// Return the live control-link state for a workflow identity, if registered.
    pub async fn control_link_state(&self, instance_id: ash_core::WorkflowId) -> Option<LinkState> {
        let link = ControlLink { instance_id };
        self.control_registry.lock().await.check_health(&link).ok()
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
    pub fn proxy_registry(&self) -> Arc<AsyncMutex<ProxyRegistry>> {
        self.proxy_registry.clone()
    }

    /// Get access to the suspended yields registry
    pub fn suspended_yields(&self) -> Arc<AsyncMutex<SuspendedYields>> {
        self.suspended_yields.clone()
    }

    /// Get access to the yield router
    pub fn yield_router(&self) -> Arc<AsyncMutex<YieldRouter>> {
        self.yield_router.clone()
    }

    /// Store the most recent authoritative top-level execution record observed for this runtime state.
    ///
    /// Spawned-child and other auxiliary execution instances must not overwrite this slot; they own
    /// distinct execution instances and should surface their terminal state through retained-completion
    /// or other instance-scoped carriers instead.
    pub async fn set_last_execution_record(&self, record: ExecutionRecord) {
        *self.last_execution_record.lock().await = Some(record);
    }

    /// Read the most recent authoritative top-level execution record observed for this runtime state.
    pub async fn last_execution_record(&self) -> Option<ExecutionRecord> {
        (*self.last_execution_record.lock().await).clone()
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

    #[test]
    fn provider_builder_registers_provider() {
        let runtime_state = RuntimeState::new().with_provider(
            "test",
            Arc::new(crate::capability::MockProvider::new(
                "test",
                Effect::Epistemic,
            )),
        );

        assert!(runtime_state.has_provider("test"));
        assert_eq!(runtime_state.provider_count(), 1);
        assert_eq!(runtime_state.provider_names(), vec!["test".to_string()]);
        assert!(runtime_state.get_provider("test").is_some());
    }

    #[test]
    fn providers_builder_registers_all_providers() {
        let mut providers: HashMap<String, Arc<dyn CapabilityProvider>> = HashMap::new();
        providers.insert(
            "one".to_string(),
            Arc::new(crate::capability::MockProvider::new(
                "one",
                Effect::Epistemic,
            )),
        );
        providers.insert(
            "two".to_string(),
            Arc::new(crate::capability::MockProvider::new(
                "two",
                Effect::Epistemic,
            )),
        );

        let runtime_state = RuntimeState::new().with_providers(providers);

        assert!(runtime_state.has_provider("one"));
        assert!(runtime_state.has_provider("two"));
        assert_eq!(runtime_state.provider_count(), 2);
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
    async fn callable_workflow_registry_round_trips() {
        let runtime_state = RuntimeState::new();
        let workflow = Workflow::Ret {
            expr: Expr::Literal(Value::Int(1)),
        };

        runtime_state
            .register_callable_workflow("worker", workflow.clone(), 0, vec![])
            .await;

        assert_eq!(
            runtime_state.callable_workflow("worker").await,
            Some(RegisteredCallableWorkflow {
                workflow,
                arity: 0,
                params: vec![]
            })
        );
        assert!(runtime_state.callable_workflow("missing").await.is_none());
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
