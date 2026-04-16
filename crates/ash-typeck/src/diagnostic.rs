//! LSP diagnostic trait implementations for `ash-typeck` error types.

use ash_diagnostic::{AshLspError, DiagnosticCode, Severity, Span};

fn to_diag_span(span: ash_parser::token::Span) -> Span {
    Span::new(span.start, span.end, span.line, span.column)
}

impl AshLspError for crate::error::ConstructorError {
    fn span(&self) -> Option<Span> {
        match self {
            Self::UnknownConstructor(_, span) => Some(to_diag_span(*span)),
            Self::MissingField { span, .. } => Some(to_diag_span(*span)),
            Self::UnknownField { span, .. } => Some(to_diag_span(*span)),
            Self::FieldTypeMismatch { span, .. } => Some(to_diag_span(*span)),
            Self::TupleFieldTypeMismatch { span, .. } => Some(to_diag_span(*span)),
            Self::TupleArityMismatch { span, .. } => Some(to_diag_span(*span)),
            Self::NonExhaustiveMatch { span, .. } => Some(to_diag_span(*span)),
            Self::UnboundVariable { span, .. } => Some(to_diag_span(*span)),
            Self::NotIterable { span, .. } => Some(to_diag_span(*span)),
            Self::UnsupportedExpression { span, .. } => Some(to_diag_span(*span)),
            Self::UnknownTypeAnnotation { span, .. } => Some(to_diag_span(*span)),
            Self::InvalidInterfaceMethodCall { span, .. } => Some(to_diag_span(*span)),
        }
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn code(&self) -> Option<DiagnosticCode> {
        Some(DiagnosticCode("E100".into()))
    }
}

impl AshLspError for crate::error::TypeEnvError {
    fn span(&self) -> Option<Span> {
        match self {
            Self::DuplicateType(_, span) => Some(to_diag_span(*span)),
            Self::TypeNotFound(_, span) => Some(to_diag_span(*span)),
            Self::InvalidDefinition(_, span) => Some(to_diag_span(*span)),
            Self::DuplicateInterface(_, span) => Some(to_diag_span(*span)),
            Self::MissingInterface(_, span) => Some(to_diag_span(*span)),
            Self::DuplicateImpl { span, .. } => Some(to_diag_span(*span)),
            Self::MissingImpl { span, .. } => Some(to_diag_span(*span)),
            Self::MissingInterfaceMethod { span, .. } => Some(to_diag_span(*span)),
            Self::OverlappingImpls { span, .. } => Some(to_diag_span(*span)),
            Self::RecursiveBound { span, .. } => Some(to_diag_span(*span)),
            Self::MissingAssociatedType { span, .. } => Some(to_diag_span(*span)),
            Self::MismatchedProjectionInterface { span, .. } => Some(to_diag_span(*span)),
            Self::AmbiguousAssociatedType { span, .. } => Some(to_diag_span(*span)),
        }
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn code(&self) -> Option<DiagnosticCode> {
        Some(DiagnosticCode("E101".into()))
    }
}

impl AshLspError for crate::solver::TypeError {
    fn span(&self) -> Option<Span> {
        match self {
            Self::Mismatch { span, .. } => Some(to_diag_span(*span)),
            Self::InfiniteType { span, .. } => Some(to_diag_span(*span)),
            Self::ConstructorNameMismatch { span, .. } => Some(to_diag_span(*span)),
            Self::ConstructorArityMismatch { span, .. } => Some(to_diag_span(*span)),
            Self::UnboundVariable(_, span) => Some(to_diag_span(*span)),
            Self::EffectViolation { span, .. } => Some(to_diag_span(*span)),
            Self::MissingCapability(_, span) => Some(to_diag_span(*span)),
            Self::UnsatisfiedObligation(_, span) => Some(to_diag_span(*span)),
            Self::Obligation(_) => Some(Span::default()),
            Self::UndischargedObligations { span, .. } => Some(to_diag_span(*span)),
            Self::UnknownObligation { span, .. } => Some(to_diag_span(*span)),
            Self::ObligationAlreadySatisfied { span, .. } => Some(to_diag_span(*span)),
            Self::UnsatisfiedObligations { span, .. } => Some(to_diag_span(*span)),
            Self::PatternMismatch { span, .. } => Some(to_diag_span(*span)),
            Self::UnknownVariant(_, span) => Some(to_diag_span(*span)),
            Self::PatternArityMismatch { span, .. } => Some(to_diag_span(*span)),
            Self::InvalidPattern { span, .. } => Some(to_diag_span(*span)),
            Self::NotAConstructor(_, span) => Some(to_diag_span(*span)),
            Self::UnknownCapability { span, .. } => Some(to_diag_span(*span)),
            Self::InvalidConstraintField { span, .. } => Some(to_diag_span(*span)),
            Self::ConstraintTypeMismatch { span, .. } => Some(to_diag_span(*span)),
        }
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn code(&self) -> Option<DiagnosticCode> {
        Some(DiagnosticCode("E102".into()))
    }
}

impl AshLspError for crate::name_binding::NameError {
    fn span(&self) -> Option<Span> {
        match self {
            Self::Unresolved { span, .. } => Some(to_diag_span(*span)),
            Self::Private { span, .. } => Some(to_diag_span(*span)),
            Self::WrongTargetCapabilityAsFn { span, .. } => Some(to_diag_span(*span)),
            Self::WrongTargetFnAsCapability { span, .. } => Some(to_diag_span(*span)),
        }
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn code(&self) -> Option<DiagnosticCode> {
        Some(DiagnosticCode("E200".into()))
    }
}

impl AshLspError for crate::names::ResolutionError {
    fn span(&self) -> Option<Span> {
        match self {
            Self::UnboundVariable(_, span) => Some(to_diag_span(*span)),
            Self::DuplicateBinding(_, span) => Some(to_diag_span(*span)),
            Self::UndefinedCapability(_, span) => Some(to_diag_span(*span)),
            Self::UnresolvedSymbolicCapability { span, .. } => Some(to_diag_span(*span)),
            Self::UndefinedPolicy(_, span) => Some(to_diag_span(*span)),
            Self::UndefinedRole(_, span) => Some(to_diag_span(*span)),
        }
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn code(&self) -> Option<DiagnosticCode> {
        Some(DiagnosticCode("E201".into()))
    }
}

impl AshLspError for crate::purity::PurityError {
    fn span(&self) -> Option<Span> {
        Some(to_diag_span(self.span))
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn code(&self) -> Option<DiagnosticCode> {
        Some(DiagnosticCode("E300".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_diagnostic::AshLspError;

    #[test]
    fn test_constructor_error_diagnostic() {
        let err = crate::error::ConstructorError::UnknownConstructor(
            "Foo".to_string(),
            ash_parser::token::Span::default(),
        );
        assert!(err.span().is_some());
        assert_eq!(err.severity(), Severity::Error);
        assert_eq!(err.code(), Some(DiagnosticCode("E100".into())));
    }

    #[test]
    fn test_type_env_error_diagnostic() {
        let err = crate::error::TypeEnvError::DuplicateType(
            "Option".to_string(),
            ash_parser::token::Span::default(),
        );
        assert!(err.span().is_some());
        assert_eq!(err.severity(), Severity::Error);
        assert_eq!(err.code(), Some(DiagnosticCode("E101".into())));
    }

    #[test]
    fn test_type_error_diagnostic() {
        use crate::solver::TypeError;
        let err = TypeError::UnboundVariable("x".to_string(), ash_parser::token::Span::default());
        assert!(err.span().is_some());
        assert_eq!(err.severity(), Severity::Error);
        assert_eq!(err.code(), Some(DiagnosticCode("E102".into())));
    }

    #[test]
    fn test_name_error_diagnostic() {
        let err = crate::name_binding::NameError::Unresolved {
            name: "foo".to_string(),
            span: ash_parser::token::Span::default(),
        };
        assert!(err.span().is_some());
        assert_eq!(err.severity(), Severity::Error);
        assert_eq!(err.code(), Some(DiagnosticCode("E200".into())));
    }

    #[test]
    fn test_resolution_error_diagnostic() {
        let err = crate::names::ResolutionError::UnboundVariable(
            "x".to_string(),
            ash_parser::token::Span::default(),
        );
        assert!(err.span().is_some());
        assert_eq!(err.severity(), Severity::Error);
        assert_eq!(err.code(), Some(DiagnosticCode("E201".into())));
    }

    #[test]
    fn test_purity_error_diagnostic() {
        let err = crate::purity::PurityError {
            kind: crate::purity::PurityViolation::PolicyExpression,
            span: ash_parser::token::Span::default(),
        };
        assert!(err.span().is_some());
        assert_eq!(err.severity(), Severity::Error);
        assert_eq!(err.code(), Some(DiagnosticCode("E300".into())));
    }
}
