//! `RuntimeKernel` verified artifact construction for engine and host callers.

use ash_core::kind::Kind;
use ash_core::module_graph::ModuleId;
use ash_core::runtime_kernel::{
    RuntimeArtifactBuildError, RuntimeArtifactBuildIdentity, RuntimeArtifactBuildInput,
    RuntimeConfigId, RuntimeKernelArtifactBuilder, RuntimeKernelVerifiedArtifact, RuntimeProfileId,
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
use ash_core::{Expr, Span, TowerLevel, Value};

/// Source/check/profile request used by `RuntimeKernel` hosts to build a shared artifact summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeArtifactBuildRequest {
    /// Runtime root set identity.
    pub root_id: String,
    /// Relative module path.
    pub relative_module_path: String,
    /// Exported workflow name.
    pub workflow_name: String,
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
        workflow_name: impl Into<String>,
        profile_id: impl Into<String>,
        config_id: impl Into<String>,
        source: impl Into<String>,
        check_summary: impl Into<String>,
    ) -> Self {
        Self {
            root_id: root_id.into(),
            relative_module_path: relative_module_path.into(),
            workflow_name: workflow_name.into(),
            profile_id: profile_id.into(),
            config_id: config_id.into(),
            source: source.into(),
            check_summary: check_summary.into(),
            runtime_support_identity: None,
        }
    }

    /// Record the selected toolchain runtime-support identity for artifact construction.
    #[must_use]
    pub fn with_runtime_support_identity(mut self, identity: impl Into<String>) -> Self {
        self.runtime_support_identity = Some(identity.into());
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
            request.workflow_name.clone(),
        ),
        request.source.clone(),
        request.check_summary_with_runtime_support(),
        synthetic_tcir(request),
        RuntimeTcirCarrierScope::AlphaCheckedWorkflowBoundary,
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
        format!("workflow:{}", request.workflow_name),
    );
    let return_op = TcirOperation::evidence_intrinsic(
        "RuntimeKernel<Workflow>",
        "return",
        vec![request.workflow_name.clone()],
        "runtime_kernel_verified_artifact",
        Some(source_anchor.clone()),
    );
    let workflow_decl = TypeDeclId::ordinary(
        ModuleIdentity::new(
            None,
            ModuleId(935),
            vec!["runtime_kernel".to_string(), "artifact".to_string()],
            ModuleSourceOrigin::Synthetic {
                reason: "RuntimeKernel verified artifact builder".to_string(),
            },
        ),
        "Workflow",
    );

    TcirComputationExpression {
        source_anchor: source_anchor.clone(),
        target: TcirDoTarget {
            constructor: TypeConstructorExpr::ConstructorHead(TypeConstructorHeadId::nominal(
                workflow_decl.clone(),
                "Workflow",
            )),
            display: "Workflow".to_string(),
            source_anchor: source_anchor.clone(),
        },
        evidence: TcirSelectedEvidence {
            interface: "RuntimeKernel".to_string(),
            evidence_key: "RuntimeKernel<Workflow>".to_string(),
            return_op: return_op.clone(),
            bind_op: return_op.clone(),
        },
        tower_level: TowerLevel::Workflow,
        result_type: CanonicalTypeExpr::NominalApp {
            origin: workflow_decl,
            visible_name: "Workflow".to_string(),
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
        workflow_artifact: None,
    }
}
