//! Ash Core - IR and semantics definitions
//!
//! This crate defines the core abstract syntax, effects, and types
//! for the Ash workflow language.

pub mod adt;
pub mod ast;
pub mod capabilities;
pub mod capability;
pub mod effect;
pub mod env_frame;
pub mod kind;
pub mod module_graph;
pub mod provenance;
pub mod runtime;
pub mod semantic_summary;
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
    PropositionDependencySummaryRef, PropositionFactRole, PropositionFactSummary,
    PropositionPredicateId, PropositionPredicateParamSummary, PropositionPredicateSummary,
    SealedDomainId, SourceAnchor, SourceOrigin, SummaryVersion,
};
pub use stream::{
    Mailbox, MailboxEntry, MailboxOverflowError, OverflowStrategy, Receive as StreamReceive,
    ReceiveArm as StreamReceiveArm, ReceiveMode as StreamReceiveMode, StreamRef,
};
pub use type_ir::{
    CanonicalTypeExpr, InterfaceBoundProposition, NamedPredicateProposition, NormalTypeExpr,
    PropositionBoundary, PropositionDeferredKind, PropositionDeferredReason, PropositionEvidence,
    PropositionEvidenceRule, PropositionOutcome, PropositionRefutation,
    PropositionRefutationReason, PropositionTypeComparisonEvidence, TypeDisequalityProposition,
    TypeEqualityProposition, TypeProposition, TypePropositionTerm,
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
