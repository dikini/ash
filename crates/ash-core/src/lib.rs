//! Ash Core - IR and semantics definitions
//!
//! This crate defines the core abstract syntax, effects, and types
//! for the Ash workflow language.

pub mod adt;
pub mod amir;
pub mod ast;
pub mod capabilities;
pub mod capability;
pub mod core_ash;
pub mod core_ash_contract;
pub mod core_ash_lower;
pub mod core_ash_text;
pub mod core_ash_typecheck;
pub mod core_ash_validate;
pub mod cps;
pub mod effect;
pub mod env_frame;
pub mod kind;
pub mod module_graph;
pub mod provenance;
pub mod runtime;
pub mod runtime_kernel;
pub mod semantic_summary;
pub mod sexp;
pub mod small_step;
pub mod stream;
pub mod type_ir;
pub mod value;
pub mod visualize;
pub mod workflow_carrier;
pub mod workflow_contract;

// Property testing helpers available when proptest feature enabled
#[cfg(any(feature = "proptest-helpers", test))]
pub mod proptest_helpers;

// Testing helpers available in test mode
#[cfg(test)]
pub mod test_helpers;

pub use ast::*;
pub use effect::*;
pub use kind::*;
pub use provenance::*;
pub use runtime::*;
pub use semantic_summary::{
    DomainConstructorId, InterfaceIdentityId, ModuleIdentity, ModuleSemanticSummary,
    ModuleSemanticSummaryValidationError, ModuleSourceOrigin, ModuleSummaryRef,
    PromotedConstructorFieldSummary, PromotedConstructorId, PromotedConstructorSummary,
    PromotedDataKindId, PromotedDataKindSummary, PropositionDependencySummaryRef,
    PropositionFactRole, PropositionFactSummary, PropositionPredicateId,
    PropositionPredicateParamSummary, PropositionPredicateSummary, SealedDomainId, SourceAnchor,
    SourceOrigin, SummaryVersion,
};
pub use stream::{
    Mailbox, MailboxEntry, MailboxOverflowError, OverflowStrategy, Receive as StreamReceive,
    ReceiveArm as StreamReceiveArm, ReceiveMode as StreamReceiveMode, StreamRef,
};
pub use type_ir::{
    CanonicalTypeExpr, ConstructorVariableApp, ConstructorVariableRef, DomainConstructorApp,
    InterfaceBoundProposition, KindedTypeBinder, KindedTypeBound, NamedPredicateProposition,
    NormalTypeExpr, PartialTypeArg, PartialTypeConstructorApp, PromotedConstructorApp,
    PropositionBoundary, PropositionDeferredKind, PropositionDeferredReason, PropositionEvidence,
    PropositionEvidenceRule, PropositionOutcome, PropositionRefutation,
    PropositionRefutationReason, PropositionTypeComparisonEvidence, TcirBinder, TcirClosure,
    TcirComputationExpression, TcirDoTarget, TcirExplicitLiftProvenance,
    TcirFailureBoundaryProvenance, TcirOperation, TcirOperationKind, TcirSelectedEvidence,
    TcirStatement, TcirStatementId, TcirStatementKind, TcirWorkflowArtifactProvenance,
    TypeConstructorExpr, TypeConstructorHeadId, TypeDisequalityProposition,
    TypeEqualityProposition, TypeHoleAmbiguity, TypeHoleId, TypeHoleMetadata,
    TypeLevelConstructorApp, TypeProposition, TypePropositionTerm,
};
pub use value::*;
pub use visualize::*;

pub use env_frame::{BindingSlot, EnvFrame};

// Compile-time verification that key types are Send + Sync
const _: () = {
    const fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Value>();
    assert_send_sync::<Expr>();
    assert_send_sync::<EnvFrame>();
    assert_send_sync::<BindingSlot>();
};
