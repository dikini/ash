//! Runtime implementation-binding data models.

use ash_core::runtime::{CapabilityImplementationId, CapabilityInterfaceId, ResourceTypeId};
use ash_core::{Expr, Value};

/// Runtime-registered executable body for one Ash-defined capability implementation operation.
#[derive(Debug, Clone, PartialEq)]
pub struct ImplementationOperationBody {
    /// Operation parameter names, in positional invocation order.
    pub params: Vec<String>,
    /// Core expression body to evaluate in an effectful runtime context.
    pub body: Expr,
    /// Whether a returned closure should be forced as an `Act` body with the hidden `ActEnv` token.
    pub returns_act: bool,
}

impl ImplementationOperationBody {
    /// Create a runtime operation body from positional parameter names and a core expression.
    #[must_use]
    pub fn new<I, S>(params: I, body: Expr) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            params: params.into_iter().map(Into::into).collect(),
            body,
            returns_act: false,
        }
    }

    /// Mark this operation body as returning an effectful `Act` closure.
    #[must_use]
    pub fn returns_act(mut self) -> Self {
        self.returns_act = true;
        self
    }
}

/// Explicit source-name dependency metadata for implementation binding admission.
#[derive(Debug, Clone, PartialEq)]
pub enum ImplementationBindingDependencySource {
    /// Dependency on an entry-owned resource source by explicit source name.
    Resource {
        name: String,
        type_id: ResourceTypeId,
    },
    /// Dependency on a previously admitted capability binding by explicit runtime binding name.
    Capability {
        name: String,
        source_binding_name: String,
        interface: CapabilityInterfaceId,
    },
    /// Inert configuration value dependency.
    Config { name: String, value: Value },
}

impl ImplementationBindingDependencySource {
    /// Create a resource dependency source.
    #[must_use]
    pub fn resource(name: impl Into<String>, type_id: ResourceTypeId) -> Self {
        Self::Resource {
            name: name.into(),
            type_id,
        }
    }

    /// Create a capability dependency source.
    #[must_use]
    pub fn capability(
        name: impl Into<String>,
        source_binding_name: impl Into<String>,
        interface: CapabilityInterfaceId,
    ) -> Self {
        Self::Capability {
            name: name.into(),
            source_binding_name: source_binding_name.into(),
            interface,
        }
    }

    /// Create a config dependency source.
    #[must_use]
    pub fn config(name: impl Into<String>, value: Value) -> Self {
        Self::Config {
            name: name.into(),
            value,
        }
    }
}

/// Explicit metadata for admitting an Ash-defined implementation capability binding.
#[derive(Debug, Clone, PartialEq)]
pub struct ImplementationBindingAdmission {
    /// Runtime binding name.
    pub name: String,
    /// Static interface identifier this binding satisfies.
    pub interface: CapabilityInterfaceId,
    /// Static implementation identifier.
    pub implementation: CapabilityImplementationId,
    /// Explicit source-name dependencies.
    pub dependencies: Vec<ImplementationBindingDependencySource>,
    /// Metadata-only operation names the implementation asks to expose.
    pub requested_operations: Vec<String>,
}

impl ImplementationBindingAdmission {
    /// Create implementation binding admission metadata with no dependencies.
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        interface: CapabilityInterfaceId,
        implementation: CapabilityImplementationId,
    ) -> Self {
        Self {
            name: name.into(),
            interface,
            implementation,
            dependencies: Vec::new(),
            requested_operations: Vec::new(),
        }
    }

    /// Append one dependency source.
    #[must_use]
    pub fn with_dependency(mut self, dependency: ImplementationBindingDependencySource) -> Self {
        self.dependencies.push(dependency);
        self
    }

    /// Attach the metadata-only operation names this implementation requests to expose.
    #[must_use]
    pub fn with_requested_operations<I, S>(mut self, operations: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.requested_operations = operations.into_iter().map(Into::into).collect();
        self
    }
}
