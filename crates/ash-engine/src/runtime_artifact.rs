//! `RuntimeKernel` verified artifact construction for engine and host callers.

use ash_core::FailureBoundary;
use ash_core::core_ash::{CoreRow, CoreRowItem, CoreType};
use ash_core::runtime_kernel::{
    AlphaAdmissionProfile, ApplicationAdmissionProfile, ApplicationBoundaryBindings,
    ApplicationEntrypointDiagnostic, ApplicationEntrypointMetadata, CheckedFunctionArtifact,
    RuntimeArtifactBuildError, RuntimeArtifactBuildIdentity, RuntimeArtifactBuildInput,
    RuntimeConfigId, RuntimeKernelArtifactBuilder, RuntimeKernelVerifiedArtifact, RuntimeProfileId,
    RuntimeProfileIdentity, RuntimeRootSetId, RuntimeTcirCarrierScope,
};
use ash_core::type_ir::{
    CanonicalTypeExpr, TcirComputationExpression, TcirDoTarget, TcirFunctionArtifactProvenance,
    TcirOperation, TcirSelectedEvidence, TcirStatement, TcirStatementId, TcirStatementKind,
    TypeConstructorExpr,
};

/// Source/check/profile request used by `RuntimeKernel` hosts to build a shared artifact summary.
#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeArtifactBuildRequest {
    /// Runtime root set identity.
    pub root_id: String,
    /// Relative module path.
    pub relative_module_path: String,
    /// Exported entry name.
    pub entry_name: String,
    /// Application/runtime entrypoint metadata selected for this artifact.
    pub entrypoint: ApplicationEntrypointMetadata,
    /// Checked/lowered function artifact supplied by the engine pipeline.
    pub checked_function: CheckedFunctionArtifact,
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
    /// Honest scope of the non-authorizing TCIR metadata carrier.
    pub tcir_carrier_scope: RuntimeTcirCarrierScope,
}

impl RuntimeArtifactBuildRequest {
    /// Create a runtime artifact build request for a checked application callable entrypoint.
    ///
    /// # Errors
    ///
    /// Returns [`ApplicationEntrypointDiagnostic`] when the entrypoint name or callable identity is
    /// missing, or when its callable identity differs from the supplied checked function.
    #[allow(clippy::too_many_arguments)]
    pub fn new_application_entrypoint(
        root_id: impl Into<String>,
        relative_module_path: impl Into<String>,
        entrypoint_name: impl Into<String>,
        callable_identity: impl Into<String>,
        runtime_target_identity: impl Into<String>,
        profile_id: impl Into<String>,
        config_id: impl Into<String>,
        checked_function: CheckedFunctionArtifact,
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
        let callable_identity = entrypoint.callable_identity.as_deref().ok_or_else(|| {
            ApplicationEntrypointDiagnostic::MissingCallableIdentity {
                entrypoint_name: entrypoint_name.clone(),
            }
        })?;
        if callable_identity != checked_function.function_identity {
            return Err(ApplicationEntrypointDiagnostic::incompatible(
                entrypoint_name.clone(),
                callable_identity,
                &checked_function.function_identity,
            ));
        }
        Ok(Self {
            root_id: root_id.into(),
            relative_module_path,
            entry_name: entrypoint_name,
            entrypoint,
            checked_function,
            admission_profile: ApplicationAdmissionProfile::alpha(AlphaAdmissionProfile::Empty),
            boundary_bindings: ApplicationBoundaryBindings::empty("alpha-boundary-bindings"),
            profile_id: profile_id.into(),
            config_id: config_id.into(),
            source: source.into(),
            check_summary: check_summary.into(),
            runtime_support_identity: None,
            tcir_carrier_scope: RuntimeTcirCarrierScope::CheckedFunctionArtifact,
        })
    }

    /// Record the selected toolchain runtime-support identity for artifact construction.
    #[must_use]
    pub fn with_runtime_support_identity(mut self, identity: impl Into<String>) -> Self {
        self.runtime_support_identity = Some(identity.into());
        self
    }

    /// Set the honest scope of the non-authorizing TCIR metadata carrier.
    #[must_use]
    pub const fn with_tcir_carrier_scope(mut self, scope: RuntimeTcirCarrierScope) -> Self {
        self.tcir_carrier_scope = scope;
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
/// construction rejects the supplied checked-function TCIR carrier.
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
        checked_function_tcir(&request.checked_function),
        request.tcir_carrier_scope,
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

fn checked_function_tcir(artifact: &CheckedFunctionArtifact) -> TcirComputationExpression {
    let effect_row = render_core_row(&artifact.effect_row);
    let return_op = TcirOperation::evidence_intrinsic(
        "RuntimeKernelFunction",
        "return",
        vec![artifact.function_identity.clone(), effect_row.clone()],
        "runtime_kernel_verified_artifact",
        Some(artifact.source_anchor.clone()),
    );

    TcirComputationExpression {
        source_anchor: artifact.source_anchor.clone(),
        target: TcirDoTarget {
            constructor: TypeConstructorExpr::ProperType(CanonicalTypeExpr::Primitive(
                "Function".to_string(),
            )),
            display: format!("function:{}", artifact.function_identity),
            source_anchor: artifact.source_anchor.clone(),
        },
        evidence: TcirSelectedEvidence {
            interface: "RuntimeKernelFunction".to_string(),
            evidence_key: format!(
                "RuntimeKernelFunction<{};{effect_row}>",
                artifact.function_identity
            ),
            return_op: return_op.clone(),
            bind_op: return_op.clone(),
        },
        boundary_level: FailureBoundary::Application,
        result_type: canonical_result_type(&artifact.result_type),
        function_artifact: Some(TcirFunctionArtifactProvenance {
            function_identity: artifact.function_identity.clone(),
            effect_row,
            canonical_effect_row: artifact.effect_row.clone(),
            result_type: artifact.result_type.clone(),
        }),
        statements: vec![TcirStatement {
            id: TcirStatementId::new(0),
            source_anchor: artifact.source_anchor.clone(),
            kind: TcirStatementKind::Return {
                value: Box::new(artifact.body.clone()),
                return_op: Box::new(return_op),
            },
        }],
        explicit_lifts: Vec::new(),
        failure_boundaries: Vec::new(),
    }
}

fn canonical_result_type(ty: &CoreType) -> CanonicalTypeExpr {
    match ty {
        CoreType::Base(name) | CoreType::Named(name) | CoreType::Var(name) => {
            CanonicalTypeExpr::Primitive(name.clone())
        }
        _ => CanonicalTypeExpr::Primitive(format!("{ty:?}")),
    }
}

fn render_core_row(row: &CoreRow) -> String {
    let mut items = row
        .items
        .iter()
        .map(|item| match item {
            CoreRowItem::Process { operation } => format!("process {operation}"),
            CoreRowItem::Operation { path, operation } => {
                let path = path
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("::");
                format!("{path}::{operation}")
            }
            other => format!("{other:?}"),
        })
        .collect::<Vec<_>>();
    if let Some(tail) = &row.tail {
        items.push(tail.clone());
    }
    if items.is_empty() {
        "pure".to_string()
    } else {
        items.join(" ")
    }
}
