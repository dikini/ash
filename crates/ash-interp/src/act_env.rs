//! Runtime-only carrier for expression-level Act computations.
//!
//! `ActEnv` stays in the Rust runtime layer. It is intentionally not part of the Ash `Value`
//! space; later Act primitive work can thread it through closures and runtime call boundaries
//! without exposing it to user code.

use ash_core::{Effect, Provenance};

use crate::capability::CapabilityContext;
use crate::policy::PolicyEvaluator;
use crate::runtime_state::RuntimeState;

/// Runtime environment threaded by expression-level `Act` computations.
///
/// The carrier reuses existing runtime/provenance types instead of inventing new ones:
/// - `CapabilityContext` for provider dispatch
/// - `PolicyEvaluator` for the runtime policy collection
/// - `Provenance` for execution lineage
/// - `Vec<Effect>` for the append-only effect log
pub struct ActEnv {
    pub capability_ctx: CapabilityContext,
    pub policies: PolicyEvaluator,
    pub provenance: Provenance,
    pub effects: Vec<Effect>,
    runtime_state_ambient_authority: bool,
}

impl std::fmt::Debug for ActEnv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActEnv")
            .field("capability_ctx", &"<CapabilityContext>")
            .field("policies", &"<PolicyEvaluator>")
            .field("provenance", &self.provenance)
            .field("effects", &self.effects)
            .finish()
    }
}

impl ActEnv {
    /// Construct a new runtime Act environment from explicit runtime pieces.
    pub fn new(
        capability_ctx: CapabilityContext,
        policies: PolicyEvaluator,
        provenance: Provenance,
    ) -> Self {
        Self {
            capability_ctx,
            policies,
            provenance,
            effects: Vec::new(),
            runtime_state_ambient_authority: false,
        }
    }

    /// Construct an Act environment from a runtime state snapshot.
    ///
    /// This keeps the boundary explicit for later Act primitive integration: runtime state owns
    /// provider registration, while `ActEnv` receives the capability dispatch context produced from
    /// that state.
    pub async fn from_runtime_state(
        runtime_state: &RuntimeState,
        policies: PolicyEvaluator,
        provenance: Provenance,
    ) -> Self {
        let capability_ctx = runtime_state.create_capability_context().await;
        Self::new(capability_ctx, policies, provenance).with_runtime_state_ambient_authority()
    }

    /// Construct an Act environment from explicitly admitted capability bindings.
    pub async fn from_runtime_state_with_admitted_bindings(
        runtime_state: &RuntimeState,
        binding_ids: &[ash_core::CapabilityBindingId],
        policies: PolicyEvaluator,
        provenance: Provenance,
    ) -> crate::ExecResult<Self> {
        let capability_ctx = runtime_state
            .create_capability_context_for_bindings(binding_ids)
            .await?;
        Ok(Self::new(capability_ctx, policies, provenance))
    }

    /// Replace the accumulated effect log.
    pub fn with_effects(mut self, effects: Vec<Effect>) -> Self {
        self.effects = effects;
        self
    }

    fn with_runtime_state_ambient_authority(mut self) -> Self {
        self.runtime_state_ambient_authority = true;
        self
    }

    pub(crate) fn has_runtime_state_ambient_authority(&self) -> bool {
        self.runtime_state_ambient_authority
    }
}

impl Default for ActEnv {
    fn default() -> Self {
        Self::new(
            CapabilityContext::new(),
            PolicyEvaluator::new(),
            Provenance::default(),
        )
    }
}
