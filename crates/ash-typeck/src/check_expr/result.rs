//! Shared result and carrier types for expression checking.

use crate::error::ConstructorError;
use crate::types::{Substitution, Type, TypeVar};
use ash_core::ast::Expr as CoreExpr;
use ash_core::type_ir::TcirComputationExpression;
use ash_core::workflow_carrier::{ContractPlan, ProjectionEvent, SourceOrigin, WorkflowObligation};

/// Result of type checking an expression
#[derive(Debug, Clone, PartialEq)]
pub struct CheckResult {
    /// The inferred type of the expression
    pub ty: Type,
    /// Any substitutions generated during checking
    pub substitution: Substitution,
    /// Any errors encountered
    pub errors: Vec<ConstructorError>,
}

pub type WorkflowForm = ash_core::workflow_carrier::WorkflowForm<CoreExpr>;

/// Typechecked entry artifact preserved by `do:Workflow` elaboration.
#[derive(Debug, Clone, PartialEq)]
pub struct EntryTypedArtifact {
    pub form: WorkflowForm,
    pub projection_events: Vec<ProjectionEvent>,
    pub contract_plan: ContractPlan<CoreExpr>,
    pub obligations: Vec<WorkflowObligation>,
    pub source_origin: SourceOrigin,
}

/// Result of type-directed generalized do-block elaboration.
#[derive(Debug, Clone, PartialEq)]
pub struct DoElaborationResult {
    /// Lowered core expression produced from resolved dictionary evidence.
    pub expr: CoreExpr,
    /// The checked computation type, e.g. `Act<T>`, `Proc<T>`, or `Workflow<T>`.
    pub ty: Type,
    /// Preserved workflow semantic artifact for `do:Workflow`.
    pub entry_artifact: Option<EntryTypedArtifact>,
    /// Selected evidence captured at the current do elaboration boundary.
    pub selected_evidence: Option<crate::do_target::SelectedDoEvidence>,
    /// Typed computation-expression carrier retained for later execution lowering.
    pub tcir: Option<TcirComputationExpression>,
}

impl CheckResult {
    /// Create a successful check result
    pub fn success(ty: Type) -> Self {
        Self {
            ty,
            substitution: Substitution::new(),
            errors: Vec::new(),
        }
    }

    /// Create a check result with an error
    pub fn error(err: ConstructorError) -> Self {
        Self {
            ty: Type::Var(TypeVar::fresh()),
            substitution: Substitution::new(),
            errors: vec![err],
        }
    }

    /// Check if the result has no fatal errors.
    pub fn is_ok(&self) -> bool {
        self.errors.iter().all(ConstructorError::is_non_fatal)
    }

    /// Check if the result contains diagnostics that should stop type propagation.
    pub fn has_fatal_errors(&self) -> bool {
        !self.is_ok()
    }
}

pub(super) fn has_fatal_diagnostics(errors: &[ConstructorError]) -> bool {
    !errors.iter().all(ConstructorError::is_non_fatal)
}
