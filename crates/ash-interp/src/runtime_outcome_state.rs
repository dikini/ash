//! Authoritative coarse-grained runtime outcome/state classification.

use crate::error::ExecError;

/// Conservative authoritative runtime outcome/state classes for interpreter-facing callers.
///
/// This type intentionally stays coarse-grained. It does not claim to solve cumulative
/// semantic-carrier packaging, retained completion payload observation, or full `Par`
/// aggregation. Its role is to provide one public runtime-side classification surface that can
/// consistently distinguish active, blocked/suspended, invalid/terminated, generic execution
/// failure, and terminal success outcomes across the current interpreter/runtime boundaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuntimeOutcomeState {
    /// Execution completed successfully with a terminal value.
    TerminalSuccess,
    /// The runtime target is live and currently able to make progress.
    Active,
    /// Execution is currently blocked, paused, or explicitly suspended awaiting external input.
    BlockedOrSuspended,
    /// The runtime target is terminally unusable or no longer valid for further control.
    InvalidOrTerminated,
    /// Execution failed without being classified as a blocked/suspended or invalid/terminated
    /// condition.
    ExecutionFailure,
}

impl RuntimeOutcomeState {
    /// Classify an interpreter execution result into the authoritative runtime outcome/state.
    pub fn from_exec_result<T>(result: &Result<T, ExecError>) -> Self {
        match result {
            Ok(_) => Self::TerminalSuccess,
            Err(error) => error.runtime_outcome_state(),
        }
    }

    /// Returns `true` when the classification denotes a terminal condition.
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::TerminalSuccess | Self::InvalidOrTerminated | Self::ExecutionFailure
        )
    }

    /// Returns `true` when the classification denotes a live but non-terminal condition.
    pub fn is_live(self) -> bool {
        matches!(self, Self::Active | Self::BlockedOrSuspended)
    }
}
