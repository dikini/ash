//! Runtime support for Ash Engine clients.
//!
//! This crate supplies provider, state, and terminal-support carriers.
//! It does not expose a source or AST execution route; checked programs execute
//! through the shared Engine implementation.

pub mod act_env;
pub mod behaviour;
pub mod builtin_catalog;
pub mod capability;
pub mod channel;
pub mod context;
pub mod control_link;
pub mod error;
pub mod exec_send;
pub mod execute_observe;
pub mod execute_set;
pub mod execution_record;
pub mod list_helpers;
pub mod mailbox;
pub mod pattern;
pub mod predicate_evaluator;
pub mod process_env;
pub mod process_registry;
pub mod runtime_outcome_state;
pub mod runtime_state;
pub mod stream;
pub mod typed_provider;

pub use act_env::ActEnv;
pub use behaviour::{
    BehaviourContext, BehaviourProvider, BehaviourRegistry, BidirectionalBehaviour,
    BidirectionalBehaviourProvider, MockBehaviourProvider, MockBidirectionalProvider,
    MockSettableProvider, SettableBehaviourProvider, SettableRegistry, TypedSettableProvider,
};
pub use builtin_catalog::{
    BuiltinEntry, BuiltinHostHookMetadata, BuiltinHostHookMetadataError, builtin_dispatch_table,
    builtin_host_hook_metadata, builtin_requires_host_hook_metadata,
    validate_builtin_host_hook_metadata,
};
pub use capability::{CapabilityContext, CapabilityProvider, CapabilityRegistry, MockProvider};
pub use channel::{ChannelError, ChannelId, ChannelRegistry};
pub use context::Context;
pub use control_link::{
    ConservativeRetainedEffectSummary, ConservativeRetainedObligationsSummary,
    ConservativeRetainedProvenanceSummary, ControlLinkError, ControlLinkRegistry, LinkState,
    RetainedCompletionKind, RetainedCompletionRecord,
};
pub use error::{
    EvalError, EvalResult, ExecError, ExecResult, PatternError, PatternResult, ValidationError,
    ValidationResult,
};
pub use exec_send::execute_send;
pub use execute_observe::{execute_changed, execute_observe};
pub use execute_set::execute_set;
pub use execution_record::{
    ExecutionAdmissionFacts, ExecutionBlockedReason, ExecutionEffectSummary,
    ExecutionInvalidReason, ExecutionObligationState, ExecutionPhase, ExecutionRecord,
    ExecutionTerminal, SemanticApplicationOutcome, SemanticCompletionPayload, SemanticEffectTrace,
};
pub use mailbox::{Mailbox, MailboxError, SharedMailbox};
pub use pattern::match_pattern;
pub use process_env::{
    ChildEnvProjection, ChildEnvProjectionError, ProcessEnvIdentity, derive_child_env,
};
pub use process_registry::{ProcessRecord, ProcessRegistry, ProcessRegistryError};
pub use runtime_outcome_state::RuntimeOutcomeState;
pub use runtime_state::{
    EntryOwnedResourceAdmission, ImplementationBindingAdmission,
    ImplementationBindingDependencySource, ImplementationOperationBody, RuntimeState,
    StandardInternalPilot, StandardPilotBinding, StandardPilotResource,
};
pub use stream::{
    BidirectionalStream, BidirectionalStreamProvider, MockBidirectionalStream,
    MockSendableProvider, MockStreamProvider, SendableRegistry, SendableStreamProvider,
    StreamContext, StreamProvider, StreamRegistry, TypedSendableProvider,
};
pub use typed_provider::{TypedBehaviourProvider, TypedStreamProvider};

use ash_core::{
    ApplicationBoundaryOutcome, ApplicationFailure, ApplicationFailureKind, ApplicationId,
    ApplicationReport, FailureBoundary, FailureEntity, OperationalFailure, RunId, Value,
};

/// Project an existing runtime result into the outer application-boundary carrier.
#[must_use]
pub fn application_boundary_outcome_from_exec_result(
    application_id: ApplicationId,
    run_id: RunId,
    result: ExecResult<Value>,
) -> ApplicationBoundaryOutcome {
    match result {
        Ok(value) => {
            let report =
                ApplicationReport::succeeded(application_id, run_id).with_result(value.clone());
            ApplicationBoundaryOutcome::succeeded(value, report)
        }
        Err(error) => {
            let cause = OperationalFailure::new(
                FailureBoundary::Application,
                FailureEntity::Run(run_id),
                Value::String(error.to_string()),
                "ExecError",
            );
            let failure = ApplicationFailure::new(
                application_id,
                run_id,
                ApplicationFailureKind::BodyFailureEscaped,
                Some(cause),
            );
            let report = ApplicationReport::failed(application_id, run_id, failure.clone());
            ApplicationBoundaryOutcome::failed(failure, report)
        }
    }
}
