//! Process registry substrate for Phase 98 process runtime semantics.
//!
//! This module stores process identity, parent/child relations, lifecycle state,
//! and write-once terminal outcomes. It is intentionally separate from the
//! existing workflow [`ControlLinkRegistry`](crate::control_link::ControlLinkRegistry):
//! `ControlLink` remains workflow supervision/control authority, while
//! `ProcessId` identifies Proc-layer execution entities.

use std::collections::HashMap;

use ash_core::runtime::{ProcessId, ProcessLifecycleState, ProcessTerminalState};
use thiserror::Error;

/// Runtime-owned record for one process identity.
#[derive(Debug, Clone, PartialEq)]
pub struct ProcessRecord {
    /// Process identity for this record.
    pub process_id: ProcessId,
    /// Optional parent process identity.
    pub parent_process_id: Option<ProcessId>,
    /// Child index in the parent admission operation, when this is a child.
    pub child_index: Option<usize>,
    /// Current lifecycle state.
    pub lifecycle_state: ProcessLifecycleState,
    /// Terminal state, recorded exactly once.
    pub terminal_state: Option<ProcessTerminalState>,
}

impl ProcessRecord {
    fn root(process_id: ProcessId) -> Self {
        Self {
            process_id,
            parent_process_id: None,
            child_index: None,
            lifecycle_state: ProcessLifecycleState::Admitting,
            terminal_state: None,
        }
    }

    fn child(parent_process_id: ProcessId, process_id: ProcessId, child_index: usize) -> Self {
        Self {
            process_id,
            parent_process_id: Some(parent_process_id),
            child_index: Some(child_index),
            lifecycle_state: ProcessLifecycleState::Admitting,
            terminal_state: None,
        }
    }
}

/// Errors returned by the process registry substrate.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ProcessRegistryError {
    /// A process identity was registered more than once.
    #[error("process {0:?} is already registered")]
    AlreadyRegistered(ProcessId),
    /// A parent process identity was required but not present.
    #[error("parent process {0:?} is not registered")]
    ParentNotFound(ProcessId),
    /// A parent process is already terminal and cannot admit new children.
    #[error("parent process {0:?} is terminal and cannot admit children")]
    ParentTerminal(ProcessId),
    /// The parent already has a child registered at the requested child index.
    #[error("parent process {parent_process_id:?} already has a child at index {child_index}")]
    DuplicateChildIndex {
        /// Parent process identity.
        parent_process_id: ProcessId,
        /// Duplicate child index.
        child_index: usize,
    },
    /// A process identity was not present in the registry.
    #[error("process {0:?} is not registered")]
    NotFound(ProcessId),
    /// A lifecycle transition was requested for a process that is already running.
    #[error("process {0:?} is already running")]
    AlreadyRunning(ProcessId),
    /// A terminal state already exists and cannot be overwritten.
    #[error("process {0:?} already has a terminal state")]
    AlreadyTerminal(ProcessId),
    /// Terminal state identity did not match the process being recorded.
    #[error("terminal state identity does not match process {0:?}")]
    TerminalIdentityMismatch(ProcessId),
}

/// Registry keyed by [`ProcessId`] for process-runtime lifecycle state.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ProcessRegistry {
    records: HashMap<ProcessId, ProcessRecord>,
    children: HashMap<ProcessId, Vec<ProcessId>>,
}

impl ProcessRegistry {
    /// Create an empty process registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a root process.
    pub fn register_root(&mut self, process_id: ProcessId) -> Result<(), ProcessRegistryError> {
        self.insert_record(ProcessRecord::root(process_id))
    }

    /// Register a child process below an already-registered parent.
    pub fn register_child(
        &mut self,
        parent_process_id: ProcessId,
        child_process_id: ProcessId,
        child_index: usize,
    ) -> Result<(), ProcessRegistryError> {
        self.register_children_batch(parent_process_id, vec![(child_process_id, child_index)])
    }

    /// Register multiple children atomically below one already-registered parent.
    pub fn register_children_batch(
        &mut self,
        parent_process_id: ProcessId,
        children: Vec<(ProcessId, usize)>,
    ) -> Result<(), ProcessRegistryError> {
        let parent = self
            .records
            .get(&parent_process_id)
            .ok_or(ProcessRegistryError::ParentNotFound(parent_process_id))?;
        if is_terminal_lifecycle(&parent.lifecycle_state) {
            return Err(ProcessRegistryError::ParentTerminal(parent_process_id));
        }

        let existing_children = self
            .children
            .get(&parent_process_id)
            .cloned()
            .unwrap_or_default();
        for (_child_process_id, child_index) in &children {
            if existing_children
                .iter()
                .filter_map(|child_id| self.records.get(child_id))
                .any(|child| child.child_index == Some(*child_index))
            {
                return Err(ProcessRegistryError::DuplicateChildIndex {
                    parent_process_id,
                    child_index: *child_index,
                });
            }
        }
        for (idx, (child_process_id, child_index)) in children.iter().enumerate() {
            if children[..idx]
                .iter()
                .any(|(_, seen_child_index)| seen_child_index == child_index)
            {
                return Err(ProcessRegistryError::DuplicateChildIndex {
                    parent_process_id,
                    child_index: *child_index,
                });
            }
            if self.records.contains_key(child_process_id) {
                return Err(ProcessRegistryError::AlreadyRegistered(*child_process_id));
            }
        }

        let child_ids = children
            .iter()
            .map(|(child_process_id, _)| *child_process_id)
            .collect::<Vec<_>>();
        for (child_process_id, child_index) in children {
            self.records.insert(
                child_process_id,
                ProcessRecord::child(parent_process_id, child_process_id, child_index),
            );
        }
        let registered_children = self.children.entry(parent_process_id).or_default();
        registered_children.extend(child_ids);
        registered_children.sort_by_key(|child_id| {
            self.records
                .get(child_id)
                .and_then(|record| record.child_index)
                .unwrap_or(usize::MAX)
        });
        Ok(())
    }

    /// Transition an admitted process to running.
    pub fn mark_running(&mut self, process_id: ProcessId) -> Result<(), ProcessRegistryError> {
        let record = self
            .records
            .get_mut(&process_id)
            .ok_or(ProcessRegistryError::NotFound(process_id))?;
        match record.lifecycle_state {
            ProcessLifecycleState::Admitting => {
                record.lifecycle_state = ProcessLifecycleState::Running;
                Ok(())
            }
            ProcessLifecycleState::Running => Err(ProcessRegistryError::AlreadyRunning(process_id)),
            _ if is_terminal_lifecycle(&record.lifecycle_state) => {
                Err(ProcessRegistryError::AlreadyTerminal(process_id))
            }
            _ => {
                record.lifecycle_state = ProcessLifecycleState::Running;
                Ok(())
            }
        }
    }

    /// Return a process record by identity.
    #[must_use]
    pub fn record(&self, process_id: ProcessId) -> Option<&ProcessRecord> {
        self.records.get(&process_id)
    }

    /// Return child process identities in parent child-index order.
    #[must_use]
    pub fn children_of(&self, parent_process_id: ProcessId) -> Vec<ProcessId> {
        self.children
            .get(&parent_process_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Record a terminal state exactly once.
    pub fn record_terminal(
        &mut self,
        process_id: ProcessId,
        terminal_state: ProcessTerminalState,
    ) -> Result<(), ProcessRegistryError> {
        ensure_terminal_identity(process_id, &terminal_state)?;
        let record = self
            .records
            .get_mut(&process_id)
            .ok_or(ProcessRegistryError::NotFound(process_id))?;
        if record.terminal_state.is_some() {
            return Err(ProcessRegistryError::AlreadyTerminal(process_id));
        }
        record.lifecycle_state = lifecycle_from_terminal(&terminal_state);
        record.terminal_state = Some(terminal_state);
        Ok(())
    }

    fn insert_record(&mut self, record: ProcessRecord) -> Result<(), ProcessRegistryError> {
        if self.records.contains_key(&record.process_id) {
            return Err(ProcessRegistryError::AlreadyRegistered(record.process_id));
        }
        self.records.insert(record.process_id, record);
        Ok(())
    }
}

fn lifecycle_from_terminal(terminal_state: &ProcessTerminalState) -> ProcessLifecycleState {
    match terminal_state {
        ProcessTerminalState::Succeeded { value } => ProcessLifecycleState::Succeeded {
            value: value.clone(),
        },
        ProcessTerminalState::Failed {
            process_id,
            failure,
        } => ProcessLifecycleState::Failed {
            process_id: *process_id,
            failure: failure.clone(),
        },
        ProcessTerminalState::Cancelled {
            process_id,
            failure,
        } => ProcessLifecycleState::Cancelled {
            process_id: *process_id,
            failure: failure.clone(),
        },
    }
}

fn is_terminal_lifecycle(lifecycle_state: &ProcessLifecycleState) -> bool {
    matches!(
        lifecycle_state,
        ProcessLifecycleState::Succeeded { .. }
            | ProcessLifecycleState::Failed { .. }
            | ProcessLifecycleState::Cancelled { .. }
    )
}

fn ensure_terminal_identity(
    process_id: ProcessId,
    terminal_state: &ProcessTerminalState,
) -> Result<(), ProcessRegistryError> {
    match terminal_state {
        ProcessTerminalState::Succeeded { .. } => Ok(()),
        ProcessTerminalState::Failed {
            process_id: failed_id,
            ..
        }
        | ProcessTerminalState::Cancelled {
            process_id: failed_id,
            ..
        } => {
            if *failed_id == process_id {
                Ok(())
            } else {
                Err(ProcessRegistryError::TerminalIdentityMismatch(process_id))
            }
        }
    }
}
