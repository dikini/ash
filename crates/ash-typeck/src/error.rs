//! Error types for type checking
//!
//! Defines errors that can occur during type checking of expressions,
//! including constructor checking errors.

use ash_core::semantic_summary::{ModuleIdentity, SummaryVersion};
use ash_parser::token::Span;
use std::fmt;
use thiserror::Error;

/// Error type for constructor checking
#[derive(Debug, Clone, PartialEq, Error)]
pub enum ConstructorError {
    /// Unknown constructor name
    #[error("Unknown constructor: {0}")]
    UnknownConstructor(String, Span),

    /// Missing required field in constructor
    #[error("Missing field '{field}' in constructor '{constructor}'")]
    MissingField {
        /// Name of the constructor
        constructor: String,
        /// Name of the missing field
        field: String,
        /// Source span
        span: Span,
    },

    /// Unknown field provided to constructor
    #[error("Unknown field '{field}' in constructor '{constructor}'")]
    UnknownField {
        /// Name of the constructor
        constructor: String,
        /// Name of the unknown field
        field: String,
        /// Source span
        span: Span,
    },

    /// Type mismatch in field
    #[error(
        "Type mismatch in field '{field}' of constructor '{constructor}': expected {expected}, got {actual}"
    )]
    FieldTypeMismatch {
        /// Name of the constructor
        constructor: String,
        /// Source-facing field label or tuple slot label
        field: String,
        /// Expected type
        expected: String,
        /// Actual type
        actual: String,
        /// Source span
        span: Span,
    },

    /// Type mismatch in positional tuple payload item
    #[error(
        "Type mismatch in positional item {position} of constructor '{constructor}': expected {expected}, got {actual}"
    )]
    TupleFieldTypeMismatch {
        /// Name of the constructor
        constructor: String,
        /// Zero-based tuple position
        position: usize,
        /// Expected type
        expected: String,
        /// Actual type
        actual: String,
        /// Source span
        span: Span,
    },

    /// Wrong number of positional tuple payload items for a constructor.
    #[error(
        "Tuple constructor arity mismatch for '{constructor}': expected {expected}, got {actual}"
    )]
    TupleArityMismatch {
        /// Name of the constructor
        constructor: String,
        /// Expected number of positional items
        expected: usize,
        /// Actual number of positional items
        actual: usize,
        /// Source span
        span: Span,
    },

    /// Match expression does not cover all variants of the scrutinee enum
    #[error("non-exhaustive match on type '{scrutinee_type}': missing {missing}")]
    NonExhaustiveMatch {
        /// Enum (or ADT) type being matched
        scrutinee_type: String,
        /// Human-readable list of missing cases
        missing: String,
        /// Source span
        span: Span,
    },

    /// `with_error` handler arms do not cover all known failure payload cases.
    #[error(
        "non-exhaustive with_error handler on failure payload type '{payload_type}': missing {missing}"
    )]
    NonExhaustiveWithErrorHandler {
        /// Failure payload type being handled.
        payload_type: String,
        /// Human-readable list of missing cases.
        missing: String,
        /// Source span.
        span: Span,
    },

    /// `with_error` handler coverage cannot be proven because payload type information is unavailable.
    #[error(
        "with_error handler coverage deferred for failure payload type '{payload_type}': {reason}"
    )]
    WithErrorHandlerCoverageDeferred {
        /// Failure payload type boundary, or `<unavailable>` when no static payload channel exists.
        payload_type: String,
        /// Human-facing reason and guidance.
        reason: String,
        /// Source span.
        span: Span,
    },

    /// Non-fatal diagnostic for an accepted `if let` whose else branch cannot be reached.
    #[error("unreachable if let else branch: {reason}")]
    UnreachableIfLetElse {
        /// Human-readable reason and rewrite guidance.
        reason: String,
        /// Source span.
        span: Span,
    },

    /// Unbound variable - variable not found in environment
    #[error("unbound variable: {name}")]
    UnboundVariable {
        /// Name of the variable
        name: String,
        /// Source span
        span: Span,
    },

    /// Type is not iterable (used in for loops)
    #[error("type {ty} is not iterable")]
    NotIterable {
        /// The type that cannot be iterated
        ty: crate::types::Type,
        /// Source span
        span: Span,
    },

    /// Field access requested a field missing from a record-typed base.
    #[error("missing field '{field}' in record")]
    MissingRecordField {
        /// Missing field name.
        field: String,
        /// Source span.
        span: Span,
    },

    /// Field access was applied to a non-record value.
    #[error("cannot access field '{field}' on non-record type {actual}")]
    NotARecord {
        /// Field being requested.
        field: String,
        /// Actual base type encountered.
        actual: crate::types::Type,
        /// Source span.
        span: Span,
    },

    /// Unsupported expression type
    #[error("unsupported expression: {kind}")]
    UnsupportedExpression {
        /// Kind of expression that is unsupported
        kind: String,
        /// Source span
        span: Span,
    },

    /// Unknown type annotation in FnDef parameter or return type
    #[error("unknown type annotation `{name}` in {context}")]
    UnknownTypeAnnotation {
        /// The unresolvable type name
        name: String,
        /// Where the annotation appeared, e.g. "parameter `x`" or "return type"
        context: String,
        /// Source span
        span: Span,
    },

    /// Invalid canonical interface method call
    #[error("invalid interface method call {interface}::{method}: {reason}")]
    InvalidInterfaceMethodCall {
        /// Interface name
        interface: String,
        /// Method name
        method: String,
        /// Human-readable failure reason
        reason: String,
        /// Source span
        span: Span,
    },
}

impl ConstructorError {
    /// Returns true for diagnostics that should not make a check result fail.
    pub fn is_non_fatal(&self) -> bool {
        matches!(self, Self::UnreachableIfLetElse { .. })
    }
}

/// Structured proposition diagnostic families from SPEC-064 §11.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PropositionDiagnosticKind {
    /// Unsupported proposition syntax at a surface not enabled by this phase.
    UnsupportedSurfaceSyntax,
    /// Unknown named predicate.
    UnknownNamedPredicate,
    /// Named predicate exists, but this solver slice cannot prove it.
    UnsupportedNamedPredicateSolving,
    /// Equality blocked by a neutral computation head.
    EqualityBlockedByNeutralHead,
    /// Equality blocked by a rigid associated projection.
    EqualityBlockedByRigidProjection,
    /// Disequality unsupported because one side is open or neutral.
    DisequalityOpenOrNeutral,
    /// Disequality refuted because both sides are equal.
    DisequalityRefutedByEquality,
    /// Required interface-bound evidence was not found.
    InterfaceBoundNotFound,
    /// Public proposition summary was malformed or used an unsupported version.
    MalformedPropositionSummary,
    /// Public proposition summary would leak a private proposition dependency.
    PrivatePropositionDependencyLeak,
    /// Proposition would require solving inputs from outputs.
    NoInversionBoundary,
}

impl PropositionDiagnosticKind {
    /// Return the stable diagnostic code for this proposition diagnostic family.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::UnsupportedSurfaceSyntax => "E168",
            Self::UnknownNamedPredicate => "E166",
            Self::UnsupportedNamedPredicateSolving => "E169",
            Self::EqualityBlockedByNeutralHead => "E170",
            Self::EqualityBlockedByRigidProjection => "E171",
            Self::DisequalityOpenOrNeutral => "E172",
            Self::DisequalityRefutedByEquality => "E173",
            Self::InterfaceBoundNotFound => "E174",
            Self::MalformedPropositionSummary => "E175",
            Self::PrivatePropositionDependencyLeak => "E176",
            Self::NoInversionBoundary => "E177",
        }
    }
}

impl fmt::Display for PropositionDiagnosticKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSurfaceSyntax => write!(f, "unsupported proposition syntax"),
            Self::UnknownNamedPredicate => write!(f, "unknown named proposition predicate"),
            Self::UnsupportedNamedPredicateSolving => {
                write!(f, "unsupported named predicate solving")
            }
            Self::EqualityBlockedByNeutralHead => {
                write!(f, "equality blocked by neutral computation head")
            }
            Self::EqualityBlockedByRigidProjection => {
                write!(f, "equality blocked by rigid associated projection")
            }
            Self::DisequalityOpenOrNeutral => {
                write!(f, "disequality blocked by open or neutral side")
            }
            Self::DisequalityRefutedByEquality => {
                write!(f, "disequality refuted because both sides are equal")
            }
            Self::InterfaceBoundNotFound => write!(f, "interface bound not found"),
            Self::MalformedPropositionSummary => write!(f, "malformed proposition summary"),
            Self::PrivatePropositionDependencyLeak => {
                write!(f, "private proposition dependency leak")
            }
            Self::NoInversionBoundary => write!(
                f,
                "no-inversion boundary: Ash normalized both sides but did not solve under type functions or associated families"
            ),
        }
    }
}

/// Error type for type environment operations
#[derive(Debug, Clone, PartialEq, Error)]
pub enum TypeEnvError {
    /// Type already defined
    #[error("Type '{0}' is already defined")]
    DuplicateType(String, Span),

    /// Type not found
    #[error("Type '{0}' not found")]
    TypeNotFound(String, Span),

    /// Invalid type definition
    #[error("Invalid type definition: {0}")]
    InvalidDefinition(String, Span),

    /// Unsupported imported module semantic summary version.
    #[error(
        "unsupported-version: unsupported module semantic summary version {}; expected {}",
        version.0, expected
    )]
    UnsupportedSummaryVersion {
        /// Unsupported version found in the imported summary.
        version: SummaryVersion,
        /// Human-readable expected version set.
        expected: String,
        /// Source span.
        span: Span,
    },

    /// Imported public computation data is malformed for the summary version/content contract.
    #[error("malformed imported-computation-summary: {message}")]
    MalformedImportedComputationSummary {
        /// Human-readable reason.
        message: String,
        /// Summary version that carried malformed computation fields.
        version: SummaryVersion,
        /// Source span.
        span: Span,
    },

    /// A public export would leak a private dependency.
    #[error(
        "private-dependency-export-failure: public type function '{public_item}' depends on private {dependency_kind} '{dependency}'"
    )]
    PrivateDependencyExportFailure {
        /// Public item being exported.
        public_item: String,
        /// Private dependency name.
        dependency: String,
        /// Private dependency family.
        dependency_kind: String,
        /// Source span.
        span: Span,
    },

    /// Imported summaries conflict in a way that would make registration order observable.
    #[error("import-order-conflict: {family} '{name}' has conflicting metadata")]
    ImportOrderConflict {
        /// Summary family that conflicted.
        family: String,
        /// Exported name.
        name: String,
        /// Source span.
        span: Span,
    },

    /// Named proposition predicate was referenced without a registered identity.
    #[error(
        "proposition diagnostic (unknown named proposition predicate): proposition shape `{name}<...>`; expected shape: registered named predicate identity; found shape: unregistered predicate `{name}`; solver rule: proposition predicate registry lookup; next step: declare or import proposition predicate `{name}` before use"
    )]
    UnknownPropositionPredicate {
        /// Source predicate name.
        name: String,
        /// Source span covering the predicate name.
        span: Span,
    },

    /// Named proposition predicate was applied to the wrong number of arguments.
    #[error(
        "named proposition predicate '{name}' arity mismatch: expected {expected}, got {actual}"
    )]
    PropositionPredicateArityMismatch {
        /// Source predicate name.
        name: String,
        /// Expected parameter count.
        expected: usize,
        /// Actual argument count.
        actual: usize,
        /// Source span covering the predicate use.
        span: Span,
    },

    /// Structured proposition diagnostic with stable SPEC-064 diagnostic family.
    #[error(
        "proposition diagnostic ({kind}): proposition shape `{proposition}`; expected shape: {expected}; found shape: {found}; solver rule: {solver_rule}; next step: {help}"
    )]
    PropositionDiagnostic {
        /// Diagnostic family.
        kind: PropositionDiagnosticKind,
        /// Source-facing proposition shape.
        proposition: String,
        /// Expected proposition shape.
        expected: String,
        /// Found proposition shape.
        found: String,
        /// Solver rule or conservative deferred reason.
        solver_rule: String,
        /// Likely next step/help text.
        help: String,
        /// Source span covering the proposition.
        span: Span,
    },

    /// Callable declaration supplied the same row both inline and in `where row`.
    #[error(
        "row specified twice for callable '{callable}': inline row appears in return type and expanded row appears in where clause"
    )]
    DuplicateCallableRow {
        /// Callable name.
        callable: String,
        /// Span covering the inline row.
        inline_span: Span,
        /// Span covering the expanded `where row` block.
        expanded_span: Span,
        /// Primary diagnostic span.
        span: Span,
    },

    /// Row tail appeared before the final row entry.
    #[error("row tail '| {tail}' must be the final row entry")]
    RowTailNotFinal {
        /// Tail variable name.
        tail: String,
        /// Source span covering the tail.
        span: Span,
    },

    /// A row contains more than one tail entry.
    #[error("duplicate row tail '| {tail}'")]
    DuplicateRowTail {
        /// Tail variable name.
        tail: String,
        /// Source span covering the duplicate tail.
        span: Span,
    },

    /// Unsupported row item family was found before Core lowering.
    #[error(
        "unsupported row item family '{family}' in `{item}`: rows contain requirements, not grants; use an evidence row item for predicate, law, contract, or proof requirements"
    )]
    UnsupportedRowItemFamily {
        /// Unsupported family spelling.
        family: String,
        /// Full item spelling.
        item: String,
        /// Source span covering the row item.
        span: Span,
    },

    /// Interface-qualified operation identity was used where an impl-qualified
    /// operation identity is required.
    #[error(
        "interface-qualified operation row identity `{item}` is ambiguous; use an impl-qualified identity such as `{suggestion}`"
    )]
    InterfaceQualifiedOperationRowIdentity {
        /// Full row item spelling.
        item: String,
        /// Suggested impl-qualified form when available.
        suggestion: String,
        /// Source span covering the row item.
        span: Span,
    },

    /// Operation row identity names an unknown concrete impl target.
    #[error("unknown impl type `{impl_type}` in operation row identity `{item}`")]
    UnknownOperationRowImplType {
        /// Unknown impl target spelling.
        impl_type: String,
        /// Full row item spelling.
        item: String,
        /// Source span covering the row item.
        span: Span,
    },

    /// Operation row identity names a visible impl target but no matching operation.
    #[error(
        "unknown operation in operation row identity `{item}`; no matching method in {candidates}"
    )]
    UnknownOperationRowMethod {
        /// Full row item spelling.
        item: String,
        /// Candidate impl heads for the target type.
        candidates: String,
        /// Source span covering the row item.
        span: Span,
    },

    /// Interface already defined
    #[error("Interface '{0}' is already defined")]
    DuplicateInterface(String, Span),

    /// Interface not found
    #[error("Interface '{0}' not found")]
    MissingInterface(String, Span),

    /// Duplicate impl for the same interface and full interface application
    #[error("Impl for interface '{interface}' and type '{ty}' is already defined")]
    DuplicateImpl {
        /// Interface name
        interface: String,
        /// Full interface application
        ty: String,
        /// Source span
        span: Span,
    },

    /// Impl not found for a canonical interface method call
    #[error("No impl found for interface '{interface}' and type '{ty}'")]
    MissingImpl {
        /// Interface name
        interface: String,
        /// Full interface application
        ty: String,
        /// Source span
        span: Span,
    },

    /// Interface method not found
    #[error("Interface '{interface}' does not define method '{method}'")]
    MissingInterfaceMethod {
        /// Interface name
        interface: String,
        /// Method name
        method: String,
        /// Source span
        span: Span,
    },

    /// Overlapping impls for an interface
    #[error("overlapping impls for interface '{interface}'")]
    OverlappingImpls {
        /// Interface name
        interface: String,
        /// Source span
        span: Span,
    },

    /// Recursive interface bound exceeded depth limit
    #[error("recursive interface bound exceeded depth limit")]
    RecursiveBound {
        /// Human-readable failure reason
        message: String,
        /// Source span
        span: Span,
    },

    /// Missing associated type in impl
    #[error("missing associated type '{name}' in impl for interface '{interface}'")]
    MissingAssociatedType {
        /// Interface name
        interface: String,
        /// Associated type name
        name: String,
        /// Source span
        span: Span,
    },

    /// Mismatched projection interface
    #[error("mismatched projection interface: expected '{expected}', found '{found}'")]
    MismatchedProjectionInterface {
        /// Expected interface name
        expected: String,
        /// Found interface name
        found: String,
        /// Source span
        span: Span,
    },

    /// Ambiguous associated type
    #[error("ambiguous associated type '{name}'")]
    AmbiguousAssociatedType {
        /// Associated type name
        name: String,
        /// Source span
        span: Span,
    },

    /// Missing sealed associated-family binding in an impl.
    #[error("missing associated family '{family}' in impl for interface '{interface}'")]
    MissingAssociatedFamilyBinding {
        /// Interface name.
        interface: String,
        /// Family member name.
        family: String,
        /// Source span.
        span: Span,
    },

    /// Extra associated-family binding in an impl.
    #[error("extra associated family binding '{family}' in impl for interface '{interface}'")]
    ExtraAssociatedFamilyBinding {
        /// Interface name.
        interface: String,
        /// Family member name.
        family: String,
        /// Source span.
        span: Span,
    },

    /// Duplicate sealed associated-family head declaration.
    #[error("duplicate associated family head '{family}' in interface '{interface}'")]
    DuplicateAssociatedFamilyHead {
        /// Interface name.
        interface: String,
        /// Family member name.
        family: String,
        /// Source span.
        span: Span,
    },

    /// Sealed associated-family equations were supplied outside the owner module.
    #[error(
        "unauthorized extension of sealed associated family '{family}' from module '{attempted_module:?}'; owner is '{owner_module:?}'"
    )]
    UnauthorizedAssociatedFamilyExtension {
        /// Family member name.
        family: String,
        /// Module that owns the sealed equation set.
        owner_module: ModuleIdentity,
        /// Module that attempted to add an equation.
        attempted_module: ModuleIdentity,
        /// Source span.
        span: Span,
    },

    /// Associated-family declaration/scheme lacked or violated module-owner context.
    #[error("associated family '{family}' requires defining module owner context: {reason}")]
    AssociatedFamilyModuleOwnerViolation {
        /// Family member name.
        family: String,
        /// Human-readable reason.
        reason: String,
        /// Source span.
        span: Span,
    },

    /// Overlapping associated-family schemes for the same head/patterns.
    #[error("overlapping associated family scheme for '{family}'")]
    OverlappingAssociatedFamilyScheme {
        /// Family member name.
        family: String,
        /// Source span.
        span: Span,
    },

    /// Associated-family declaration/scheme has the wrong result kind.
    #[error(
        "wrong result kind for associated family '{family}': expected {expected}, found {found}"
    )]
    WrongAssociatedFamilyResultKind {
        /// Family member name.
        family: String,
        /// Expected kind.
        expected: String,
        /// Found kind.
        found: String,
        /// Source span.
        span: Span,
    },

    /// Associated-family declaration/scheme has the wrong result domain.
    #[error("wrong result domain for associated family '{family}': {reason}")]
    WrongAssociatedFamilyResultDomain {
        /// Family member name.
        family: String,
        /// Human-readable reason.
        reason: String,
        /// Source span.
        span: Span,
    },
}

impl TypeEnvError {
    /// Return the best source span associated with this type-environment diagnostic.
    #[must_use]
    pub const fn span(&self) -> Span {
        match self {
            Self::DuplicateType(_, span)
            | Self::TypeNotFound(_, span)
            | Self::InvalidDefinition(_, span)
            | Self::UnsupportedSummaryVersion { span, .. }
            | Self::MalformedImportedComputationSummary { span, .. }
            | Self::PrivateDependencyExportFailure { span, .. }
            | Self::ImportOrderConflict { span, .. }
            | Self::UnknownPropositionPredicate { span, .. }
            | Self::PropositionPredicateArityMismatch { span, .. }
            | Self::PropositionDiagnostic { span, .. }
            | Self::DuplicateCallableRow { span, .. }
            | Self::RowTailNotFinal { span, .. }
            | Self::DuplicateRowTail { span, .. }
            | Self::UnsupportedRowItemFamily { span, .. }
            | Self::InterfaceQualifiedOperationRowIdentity { span, .. }
            | Self::UnknownOperationRowImplType { span, .. }
            | Self::UnknownOperationRowMethod { span, .. }
            | Self::DuplicateInterface(_, span)
            | Self::MissingInterface(_, span)
            | Self::DuplicateImpl { span, .. }
            | Self::MissingImpl { span, .. }
            | Self::MissingInterfaceMethod { span, .. }
            | Self::OverlappingImpls { span, .. }
            | Self::RecursiveBound { span, .. }
            | Self::MissingAssociatedType { span, .. }
            | Self::MismatchedProjectionInterface { span, .. }
            | Self::AmbiguousAssociatedType { span, .. }
            | Self::MissingAssociatedFamilyBinding { span, .. }
            | Self::ExtraAssociatedFamilyBinding { span, .. }
            | Self::DuplicateAssociatedFamilyHead { span, .. }
            | Self::UnauthorizedAssociatedFamilyExtension { span, .. }
            | Self::AssociatedFamilyModuleOwnerViolation { span, .. }
            | Self::OverlappingAssociatedFamilyScheme { span, .. }
            | Self::WrongAssociatedFamilyResultKind { span, .. }
            | Self::WrongAssociatedFamilyResultDomain { span, .. } => *span,
        }
    }
}

/// Error type for exhaustiveness checking
#[derive(Debug, Clone, PartialEq, Error)]
pub enum ExhaustivenessError {
    /// Non-exhaustive pattern match
    #[error("non-exhaustive pattern match for type '{scrutinee_type}'")]
    NonExhaustiveMatch {
        /// Type being matched
        scrutinee_type: String,
        /// Missing patterns
        missing_patterns: Vec<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unknown_constructor_error() {
        let err = ConstructorError::UnknownConstructor("Foo".to_string(), Span::default());
        let msg = format!("{err}");
        assert!(msg.contains("Unknown constructor"));
        assert!(msg.contains("Foo"));
    }

    #[test]
    fn test_missing_field_error() {
        let err = ConstructorError::MissingField {
            constructor: "Some".to_string(),
            field: "value".to_string(),
            span: Span::default(),
        };
        let msg = format!("{err}");
        assert!(msg.contains("Missing field"));
        assert!(msg.contains("Some"));
        assert!(msg.contains("value"));
    }

    #[test]
    fn test_unknown_field_error() {
        let err = ConstructorError::UnknownField {
            constructor: "Point".to_string(),
            field: "z".to_string(),
            span: Span::default(),
        };
        let msg = format!("{err}");
        assert!(msg.contains("Unknown field"));
        assert!(msg.contains("Point"));
        assert!(msg.contains("z"));
    }

    #[test]
    fn test_field_type_mismatch_error() {
        let err = ConstructorError::FieldTypeMismatch {
            constructor: "Some".to_string(),
            field: "value".to_string(),
            expected: "Int".to_string(),
            actual: "String".to_string(),
            span: Span::default(),
        };
        let msg = format!("{err}");
        assert!(msg.contains("Type mismatch"));
        assert!(msg.contains("Some"));
        assert!(msg.contains("value"));
        assert!(msg.contains("Int"));
        assert!(msg.contains("String"));
    }

    #[test]
    fn test_tuple_field_type_mismatch_error() {
        let err = ConstructorError::TupleFieldTypeMismatch {
            constructor: "RuntimeError".to_string(),
            position: 0,
            expected: "Int".to_string(),
            actual: "String".to_string(),
            span: Span::default(),
        };
        let msg = format!("{err}");
        assert!(msg.contains("positional item 0"));
        assert!(msg.contains("RuntimeError"));
        assert!(msg.contains("Int"));
        assert!(msg.contains("String"));
    }

    #[test]
    fn test_duplicate_type_error() {
        let err = TypeEnvError::DuplicateType("Option".to_string(), Span::default());
        let msg = format!("{err}");
        assert!(msg.contains("already defined"));
        assert!(msg.contains("Option"));
    }

    #[test]
    fn test_type_not_found_error() {
        let err = TypeEnvError::TypeNotFound("Unknown".to_string(), Span::default());
        let msg = format!("{err}");
        assert!(msg.contains("not found"));
        assert!(msg.contains("Unknown"));
    }
}
