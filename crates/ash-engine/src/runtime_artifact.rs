//! `RuntimeKernel` verified artifact construction for engine and host callers.

use ash_core::kind::Kind;
use ash_core::module_graph::ModuleId;
use ash_core::runtime_kernel::{
    AlphaAdmissionProfile, ApplicationAdmissionProfile, ApplicationBoundaryBindings,
    ApplicationEntrypointDiagnostic, ApplicationEntrypointMetadata, RuntimeArtifactBuildError,
    RuntimeArtifactBuildIdentity, RuntimeArtifactBuildInput, RuntimeConfigId,
    RuntimeKernelArtifactBuilder, RuntimeKernelVerifiedArtifact, RuntimeProfileId,
    RuntimeProfileIdentity, RuntimeRootSetId, RuntimeTcirCarrierScope,
};
use ash_core::semantic_summary::{
    ModuleIdentity, ModuleSourceOrigin, SourceAnchor, SourceOrigin, TypeDeclId,
};
use ash_core::type_ir::{
    CanonicalTypeExpr, TcirComputationExpression, TcirDoTarget, TcirOperation,
    TcirSelectedEvidence, TcirStatement, TcirStatementId, TcirStatementKind, TypeConstructorExpr,
    TypeConstructorHeadId,
};
use ash_core::{Expr, FailureBoundary, Span, Value};

/// Source/check/profile request used by `RuntimeKernel` hosts to build a shared artifact summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeArtifactBuildRequest {
    /// Runtime root set identity.
    pub root_id: String,
    /// Relative module path.
    pub relative_module_path: String,
    /// Exported entry name.
    pub entry_name: String,
    /// Application/runtime entrypoint metadata selected for this artifact.
    pub entrypoint: ApplicationEntrypointMetadata,
    /// Admission profile metadata selected at the runtime boundary.
    pub admission_profile: ApplicationAdmissionProfile,
    /// Non-authority boundary bindings selected at the runtime boundary.
    pub boundary_bindings: ApplicationBoundaryBindings,
    /// Runtime profile identity.
    pub profile_id: String,
    /// Runtime config identity.
    pub config_id: String,
    /// Source text used for source hashing.
    pub source: String,
    /// Check summary text used for check-summary hashing.
    pub check_summary: String,
    /// Selected toolchain runtime-support identity used by the host.
    pub runtime_support_identity: Option<String>,
}

impl RuntimeArtifactBuildRequest {
    /// Create a runtime artifact build request.
    #[must_use]
    pub fn new(
        root_id: impl Into<String>,
        relative_module_path: impl Into<String>,
        entry_name: impl Into<String>,
        profile_id: impl Into<String>,
        config_id: impl Into<String>,
        source: impl Into<String>,
        check_summary: impl Into<String>,
    ) -> Self {
        let relative_module_path = relative_module_path.into();
        let entry_name = entry_name.into();
        let entrypoint = ApplicationEntrypointMetadata {
            name: entry_name.clone(),
            kind: ash_core::runtime_kernel::ApplicationEntrypointKind::CheckedCallable,
            callable_identity: Some(format!("callable:{relative_module_path}::{entry_name}")),
            relative_module_path: relative_module_path.clone(),
            runtime_target_identity: format!("runtime-target:application-entry:{entry_name}"),
        };
        let admission_profile = ApplicationAdmissionProfile::alpha(AlphaAdmissionProfile::Empty);
        let boundary_bindings = ApplicationBoundaryBindings::empty("alpha-boundary-bindings");
        Self {
            root_id: root_id.into(),
            relative_module_path,
            entry_name,
            entrypoint,
            admission_profile,
            boundary_bindings,
            profile_id: profile_id.into(),
            config_id: config_id.into(),
            source: source.into(),
            check_summary: check_summary.into(),
            runtime_support_identity: None,
        }
    }

    /// Create a runtime artifact build request for a checked application callable entrypoint.
    ///
    /// # Errors
    ///
    /// Returns [`ApplicationEntrypointDiagnostic`] when the entrypoint name or callable identity is
    /// missing.
    #[allow(clippy::too_many_arguments)]
    pub fn new_application_entrypoint(
        root_id: impl Into<String>,
        relative_module_path: impl Into<String>,
        entrypoint_name: impl Into<String>,
        callable_identity: impl Into<String>,
        runtime_target_identity: impl Into<String>,
        profile_id: impl Into<String>,
        config_id: impl Into<String>,
        source: impl Into<String>,
        check_summary: impl Into<String>,
    ) -> Result<Self, ApplicationEntrypointDiagnostic> {
        let relative_module_path = relative_module_path.into();
        let entrypoint_name = entrypoint_name.into();
        let entrypoint = ApplicationEntrypointMetadata::checked_callable(
            entrypoint_name.clone(),
            callable_identity,
            relative_module_path.clone(),
            runtime_target_identity,
        )?;
        Ok(Self {
            root_id: root_id.into(),
            relative_module_path,
            entry_name: entrypoint_name,
            entrypoint,
            admission_profile: ApplicationAdmissionProfile::alpha(AlphaAdmissionProfile::Empty),
            boundary_bindings: ApplicationBoundaryBindings::empty("alpha-boundary-bindings"),
            profile_id: profile_id.into(),
            config_id: config_id.into(),
            source: source.into(),
            check_summary: check_summary.into(),
            runtime_support_identity: None,
        })
    }

    /// Record the selected toolchain runtime-support identity for artifact construction.
    #[must_use]
    pub fn with_runtime_support_identity(mut self, identity: impl Into<String>) -> Self {
        self.runtime_support_identity = Some(identity.into());
        self
    }

    /// Attach admission profile metadata selected at the runtime boundary.
    #[must_use]
    pub fn with_admission_profile(
        mut self,
        admission_profile: ApplicationAdmissionProfile,
    ) -> Self {
        self.admission_profile = admission_profile;
        self
    }

    /// Attach non-authority boundary binding metadata selected at the runtime boundary.
    #[must_use]
    pub fn with_boundary_bindings(
        mut self,
        boundary_bindings: ApplicationBoundaryBindings,
    ) -> Self {
        self.boundary_bindings = boundary_bindings;
        self
    }

    fn check_summary_with_runtime_support(&self) -> String {
        self.runtime_support_identity.as_ref().map_or_else(
            || self.check_summary.clone(),
            |identity| format!("{};runtime_support_identity={identity}", self.check_summary),
        )
    }
}

/// Build a shared `RuntimeKernel` artifact summary from engine source/check/profile facts.
///
/// # Errors
///
/// Returns [`RuntimeArtifactBuildError`] if verifier-normalized AMIR or bytecode
/// construction rejects the synthesized alpha TCIR carrier.
pub fn build_runtime_kernel_artifact(
    request: &RuntimeArtifactBuildRequest,
) -> Result<RuntimeKernelVerifiedArtifact, RuntimeArtifactBuildError> {
    let profile = RuntimeProfileIdentity::new(
        RuntimeProfileId::new(request.profile_id.clone()),
        RuntimeConfigId::new(request.config_id.clone()),
        runtime_profile_selection_facts(request),
    );
    let input = RuntimeArtifactBuildInput::new(
        RuntimeArtifactBuildIdentity::new(
            RuntimeRootSetId::new(request.root_id.clone()),
            profile,
            request.relative_module_path.clone(),
            request.entry_name.clone(),
        )
        .with_entrypoint(request.entrypoint.clone())
        .with_admission_profile(request.admission_profile.clone())
        .with_boundary_bindings(request.boundary_bindings.clone()),
        request.source.clone(),
        request.check_summary_with_runtime_support(),
        synthetic_tcir(request),
        RuntimeTcirCarrierScope::AlphaCheckedApplicationEntryBoundary,
    );
    RuntimeKernelArtifactBuilder::new().build(input)
}

fn runtime_profile_selection_facts(request: &RuntimeArtifactBuildRequest) -> Vec<String> {
    let mut facts = vec![format!(
        "profile={};config={}",
        request.profile_id, request.config_id
    )];
    if let Some(identity) = &request.runtime_support_identity {
        facts.push(format!("runtime_support_identity={identity}"));
    }
    facts
}

fn synthetic_tcir(request: &RuntimeArtifactBuildRequest) -> TcirComputationExpression {
    let source_anchor = SourceAnchor::new(
        SourceOrigin::File(request.relative_module_path.clone()),
        Some(Span {
            start: 0,
            end: request.source.len(),
        }),
        format!("application-entry:{}", request.entry_name),
    );
    let return_op = TcirOperation::evidence_intrinsic(
        "RuntimeKernel<ApplicationEntry>",
        "return",
        vec![request.entry_name.clone()],
        "runtime_kernel_verified_artifact",
        Some(source_anchor.clone()),
    );
    let entry_decl = TypeDeclId::ordinary(
        ModuleIdentity::new(
            None,
            ModuleId(935),
            vec!["runtime_kernel".to_string(), "artifact".to_string()],
            ModuleSourceOrigin::Synthetic {
                reason: "RuntimeKernel verified artifact builder".to_string(),
            },
        ),
        "ApplicationEntry",
    );

    TcirComputationExpression {
        source_anchor: source_anchor.clone(),
        target: TcirDoTarget {
            constructor: TypeConstructorExpr::ConstructorHead(TypeConstructorHeadId::nominal(
                entry_decl.clone(),
                "ApplicationEntry",
            )),
            display: "ApplicationEntry".to_string(),
            source_anchor: source_anchor.clone(),
        },
        evidence: TcirSelectedEvidence {
            interface: "RuntimeKernel".to_string(),
            evidence_key: "RuntimeKernel<ApplicationEntry>".to_string(),
            return_op: return_op.clone(),
            bind_op: return_op.clone(),
        },
        boundary_level: FailureBoundary::Application,
        result_type: CanonicalTypeExpr::NominalApp {
            origin: entry_decl,
            visible_name: "ApplicationEntry".to_string(),
            args: vec![CanonicalTypeExpr::Primitive("Unit".to_string())],
            kind: Kind::Type,
        },
        statements: vec![TcirStatement {
            id: TcirStatementId::new(0),
            source_anchor,
            kind: TcirStatementKind::Return {
                value: Box::new(Expr::Literal(Value::Null)),
                return_op: Box::new(return_op),
            },
        }],
        explicit_lifts: Vec::new(),
        failure_boundaries: Vec::new(),
        entry_artifact: None,
    }
}
