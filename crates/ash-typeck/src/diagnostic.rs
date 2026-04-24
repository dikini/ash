//! LSP diagnostic trait implementations for `ash-typeck` error types.

use ash_diagnostic::{AshLspError, DiagnosticCode, Severity, Span};

impl AshLspError for crate::error::ConstructorError {
    fn span(&self) -> Option<Span> {
        Some(
            match self {
                Self::UnknownConstructor(_, span) => span,
                Self::MissingField { span, .. } => span,
                Self::UnknownField { span, .. } => span,
                Self::FieldTypeMismatch { span, .. } => span,
                Self::TupleFieldTypeMismatch { span, .. } => span,
                Self::TupleArityMismatch { span, .. } => span,
                Self::NonExhaustiveMatch { span, .. } => span,
                Self::UnboundVariable { span, .. } => span,
                Self::NotIterable { span, .. } => span,
                Self::MissingRecordField { span, .. } => span,
                Self::NotARecord { span, .. } => span,
                Self::UnsupportedExpression { span, .. } => span,
                Self::UnknownTypeAnnotation { span, .. } => span,
                Self::InvalidInterfaceMethodCall { span, .. } => span,
            }
            .into(),
        )
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn code(&self) -> Option<DiagnosticCode> {
        Some(DiagnosticCode(
            match self {
                Self::UnknownConstructor(..) => "E100",
                Self::MissingField { .. } => "E101",
                Self::UnknownField { .. } => "E102",
                Self::FieldTypeMismatch { .. } => "E103",
                Self::TupleFieldTypeMismatch { .. } => "E104",
                Self::TupleArityMismatch { .. } => "E105",
                Self::NonExhaustiveMatch { .. } => "E106",
                Self::UnboundVariable { .. } => "E107",
                Self::NotIterable { .. } => "E108",
                Self::MissingRecordField { .. } => "E109",
                Self::NotARecord { .. } => "E110",
                Self::UnsupportedExpression { .. } => "E111",
                Self::UnknownTypeAnnotation { .. } => "E112",
                Self::InvalidInterfaceMethodCall { .. } => "E113",
            }
            .into(),
        ))
    }
}

impl AshLspError for crate::error::TypeEnvError {
    fn span(&self) -> Option<Span> {
        Some(
            match self {
                Self::DuplicateType(_, span) => span,
                Self::TypeNotFound(_, span) => span,
                Self::InvalidDefinition(_, span) => span,
                Self::DuplicateInterface(_, span) => span,
                Self::MissingInterface(_, span) => span,
                Self::DuplicateImpl { span, .. } => span,
                Self::MissingImpl { span, .. } => span,
                Self::MissingInterfaceMethod { span, .. } => span,
                Self::OverlappingImpls { span, .. } => span,
                Self::RecursiveBound { span, .. } => span,
                Self::MissingAssociatedType { span, .. } => span,
                Self::MismatchedProjectionInterface { span, .. } => span,
                Self::AmbiguousAssociatedType { span, .. } => span,
            }
            .into(),
        )
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn code(&self) -> Option<DiagnosticCode> {
        Some(DiagnosticCode(
            match self {
                Self::DuplicateType(..) => "E120",
                Self::TypeNotFound(..) => "E121",
                Self::InvalidDefinition(..) => "E122",
                Self::DuplicateInterface(..) => "E123",
                Self::MissingInterface(..) => "E124",
                Self::DuplicateImpl { .. } => "E125",
                Self::MissingImpl { .. } => "E126",
                Self::MissingInterfaceMethod { .. } => "E127",
                Self::OverlappingImpls { .. } => "E128",
                Self::RecursiveBound { .. } => "E129",
                Self::MissingAssociatedType { .. } => "E130",
                Self::MismatchedProjectionInterface { .. } => "E131",
                Self::AmbiguousAssociatedType { .. } => "E132",
            }
            .into(),
        ))
    }
}

impl AshLspError for crate::solver::TypeError {
    fn span(&self) -> Option<Span> {
        match self {
            Self::Mismatch { span, .. } => Some((*span).into()),
            Self::InfiniteType { span, .. } => Some((*span).into()),
            Self::ConstructorNameMismatch { span, .. } => Some((*span).into()),
            Self::ConstructorArityMismatch { span, .. } => Some((*span).into()),
            Self::UnboundVariable(_, span) => Some((*span).into()),
            Self::EffectViolation { span, .. } => Some((*span).into()),
            Self::MissingCapability(_, span) => Some((*span).into()),
            Self::UnsatisfiedObligation(_, span) => Some((*span).into()),
            // Obligation wraps an external error type with no single span.
            Self::Obligation(_) => None,
            Self::UndischargedObligations { span, .. } => Some((*span).into()),
            Self::UnknownObligation { span, .. } => Some((*span).into()),
            Self::ObligationAlreadySatisfied { span, .. } => Some((*span).into()),
            Self::UnsatisfiedObligations { span, .. } => Some((*span).into()),
            Self::PatternMismatch { span, .. } => Some((*span).into()),
            Self::UnknownVariant(_, span) => Some((*span).into()),
            Self::PatternArityMismatch { span, .. } => Some((*span).into()),
            Self::InvalidPattern { span, .. } => Some((*span).into()),
            Self::NotAConstructor(_, span) => Some((*span).into()),
            Self::UnknownCapability { span, .. } => Some((*span).into()),
            Self::InvalidConstraintField { span, .. } => Some((*span).into()),
            Self::ConstraintTypeMismatch { span, .. } => Some((*span).into()),
        }
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn code(&self) -> Option<DiagnosticCode> {
        Some(DiagnosticCode(
            match self {
                Self::Mismatch { .. } => "E140",
                Self::InfiniteType { .. } => "E141",
                Self::ConstructorNameMismatch { .. } => "E142",
                Self::ConstructorArityMismatch { .. } => "E143",
                Self::UnboundVariable(..) => "E144",
                Self::EffectViolation { .. } => "E145",
                Self::MissingCapability(..) => "E146",
                Self::UnsatisfiedObligation(..) => "E147",
                Self::Obligation(_) => "E148",
                Self::UndischargedObligations { .. } => "E149",
                Self::UnknownObligation { .. } => "E150",
                Self::ObligationAlreadySatisfied { .. } => "E151",
                Self::UnsatisfiedObligations { .. } => "E152",
                Self::PatternMismatch { .. } => "E153",
                Self::UnknownVariant(..) => "E154",
                Self::PatternArityMismatch { .. } => "E155",
                Self::InvalidPattern { .. } => "E156",
                Self::NotAConstructor(..) => "E157",
                Self::UnknownCapability { .. } => "E158",
                Self::InvalidConstraintField { .. } => "E159",
                Self::ConstraintTypeMismatch { .. } => "E160",
            }
            .into(),
        ))
    }
}

impl AshLspError for crate::name_binding::NameError {
    fn span(&self) -> Option<Span> {
        Some(
            match self {
                Self::Unresolved { span, .. } => span,
                Self::Private { span, .. } => span,
                Self::WrongTargetCapabilityAsFn { span, .. } => span,
                Self::WrongTargetFnAsCapability { span, .. } => span,
            }
            .into(),
        )
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn code(&self) -> Option<DiagnosticCode> {
        Some(DiagnosticCode(
            match self {
                Self::Unresolved { .. } => "E200",
                Self::Private { .. } => "E201",
                Self::WrongTargetCapabilityAsFn { .. } => "E202",
                Self::WrongTargetFnAsCapability { .. } => "E203",
            }
            .into(),
        ))
    }
}

impl AshLspError for crate::names::ResolutionError {
    fn span(&self) -> Option<Span> {
        Some(
            match self {
                Self::UnboundVariable(_, span) => span,
                Self::DuplicateBinding(_, span) => span,
                Self::UndefinedCapability(_, span) => span,
                Self::UnresolvedSymbolicCapability { span, .. } => span,
                Self::UndefinedPolicy(_, span) => span,
                Self::UndefinedRole(_, span) => span,
            }
            .into(),
        )
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn code(&self) -> Option<DiagnosticCode> {
        Some(DiagnosticCode(
            match self {
                Self::UnboundVariable(..) => "E210",
                Self::DuplicateBinding(..) => "E211",
                Self::UndefinedCapability(..) => "E212",
                Self::UnresolvedSymbolicCapability { .. } => "E213",
                Self::UndefinedPolicy(..) => "E214",
                Self::UndefinedRole(..) => "E215",
            }
            .into(),
        ))
    }
}

impl AshLspError for crate::purity::PurityError {
    fn span(&self) -> Option<Span> {
        Some(self.span.into())
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn code(&self) -> Option<DiagnosticCode> {
        Some(DiagnosticCode(
            match &self.kind {
                crate::purity::PurityViolation::PolicyExpression => "E300",
                crate::purity::PurityViolation::CheckObligation => "E301",
                crate::purity::PurityViolation::UnresolvedCall { .. } => "E302",
                crate::purity::PurityViolation::NonPureCall { .. } => "E303",
                crate::purity::PurityViolation::InvalidInterfaceMethodCall { .. } => "E304",
                crate::purity::PurityViolation::ActBlockInPureContext => "E305",
                crate::purity::PurityViolation::InvokeInPureContext => "E306",
            }
            .into(),
        ))
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
        assert_eq!(err.code(), Some(DiagnosticCode("E120".into())));
    }

    #[test]
    fn test_type_error_diagnostic() {
        use crate::solver::TypeError;
        let err = TypeError::UnboundVariable("x".to_string(), ash_parser::token::Span::default());
        assert!(err.span().is_some());
        assert_eq!(err.severity(), Severity::Error);
        assert_eq!(err.code(), Some(DiagnosticCode("E144".into())));
    }

    #[test]
    fn test_type_error_obligation_no_span() {
        use crate::solver::TypeError;
        use ash_core::workflow_contract::ObligationError;
        let err = TypeError::Obligation(ObligationError::Unknown("foo".into()));
        // Obligation errors wrap an external type without a single span.
        assert!(err.span().is_none());
        assert_eq!(err.code(), Some(DiagnosticCode("E148".into())));
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
        assert_eq!(err.code(), Some(DiagnosticCode("E210".into())));
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

    #[test]
    fn test_purity_error_non_pure_call_code() {
        let err = crate::purity::PurityError {
            kind: crate::purity::PurityViolation::NonPureCall {
                callee: "foo".into(),
                found: "Operational".into(),
            },
            span: ash_parser::token::Span::default(),
        };
        assert_eq!(err.code(), Some(DiagnosticCode("E303".into())));
    }

    #[test]
    fn test_span_roundtrip_via_from() {
        let parser_span = ash_parser::token::Span::new(42, 55, 7, 12);
        let diag_span: ash_diagnostic::Span = parser_span.into();
        assert_eq!(diag_span.start, 42);
        assert_eq!(diag_span.end, 55);
        assert_eq!(diag_span.line, 7);
        assert_eq!(diag_span.column, 12);
    }
}
