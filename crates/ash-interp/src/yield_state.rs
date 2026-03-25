//! Yield state management for proxy workflow request/response tracking
//!
//! Provides correlation ID generation and suspended yield tracking per SPEC-023 Section 6.
//!
//! # Example
//!
//! ```
//! use ash_interp::yield_state::{CorrelationId, SuspendedYields, YieldState};
//! use ash_core::Workflow;
//! use ash_typeck::types::Type;
//! use std::time::Duration;
//!
//! let mut suspended = SuspendedYields::new();
//!
//! // Create a yield state
//! let state = YieldState {
//!     correlation_id: CorrelationId::new(),
//!     expected_response_type: Type::Int,
//!     continuation: Workflow::Ret { expr: ash_core::Expr::Literal(ash_core::Value::Int(42)) },
//!     origin_workflow: "instance-1".to_string(),
//!     target_role: "admin".to_string(),
//!     request_sent_at: std::time::Instant::now(),
//! };
//!
//! // Suspend the workflow
//! let id = suspended.suspend(state);
//!
//! // Check if correlation ID exists
//! assert!(suspended.contains(id));
//!
//! // Get state without removing
//! let _state = suspended.get(id).unwrap();
//!
//! // Resume the workflow
//! let resumed = suspended.resume(id);
//! assert!(resumed.is_some());
//! assert!(!suspended.contains(id));
//! ```

use ash_core::Workflow;
use ash_typeck::types::Type;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Unique identifier for yield/resume correlation
///
/// This is a re-export from ash_core::ast::CorrelationId for convenience.
/// Each yield expression generates a unique correlation ID that must be
/// included in the corresponding resume.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CorrelationId(pub u64);

impl CorrelationId {
    /// Generate a new unique correlation ID
    ///
    /// Uses a global atomic counter to ensure uniqueness across the process.
    #[must_use]
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::SeqCst))
    }

    /// Generate next ID from a counter
    ///
    /// Useful for testing or when managing your own counter.
    #[must_use]
    pub fn next(counter: &mut u64) -> Self {
        let id = *counter;
        *counter += 1;
        Self(id)
    }
}

impl Default for CorrelationId {
    fn default() -> Self {
        Self::new()
    }
}

/// State of a suspended yield expression
///
/// Tracks all information needed to resume a workflow after receiving
/// a response from a proxy.
#[derive(Debug, Clone)]
pub struct YieldState {
    /// Unique correlation ID for matching yield/resume pairs
    pub correlation_id: CorrelationId,
    /// Expected type of the response value
    pub expected_response_type: Type,
    /// Continuation workflow to execute after resume
    pub continuation: Workflow,
    /// Address of the originating workflow instance
    pub origin_workflow: String,
    /// Target role that should handle the yield
    pub target_role: String,
    /// Timestamp when the request was sent
    pub request_sent_at: Instant,
}

/// Registry for tracking suspended yields
///
/// Manages the lifecycle of yield expressions that are awaiting responses.
/// Provides timeout handling and efficient lookup by correlation ID.
#[derive(Debug, Clone)]
pub struct SuspendedYields {
    yields: HashMap<CorrelationId, YieldState>,
    next_id: u64,
}

impl SuspendedYields {
    /// Create a new empty suspended yields registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            yields: HashMap::new(),
            next_id: 1,
        }
    }

    /// Suspend a workflow waiting for response
    ///
    /// Registers the yield state and returns the correlation ID.
    /// If the state already has a correlation ID, it is used; otherwise,
    /// a new one is generated from the internal counter.
    pub fn suspend(&mut self, mut state: YieldState) -> CorrelationId {
        // Generate new ID if not set (ID 0 is reserved/invalid)
        if state.correlation_id.0 == 0 {
            state.correlation_id = CorrelationId::next(&mut self.next_id);
        }
        let id = state.correlation_id;
        self.yields.insert(id, state);
        id
    }

    /// Resume a workflow by correlation ID
    ///
    /// Removes and returns the yield state if found.
    /// Returns None if the correlation ID does not exist.
    pub fn resume(&mut self, id: CorrelationId) -> Option<YieldState> {
        self.yields.remove(&id)
    }

    /// Get state without removing
    ///
    /// Returns a reference to the yield state if found.
    pub fn get(&self, id: CorrelationId) -> Option<&YieldState> {
        self.yields.get(&id)
    }

    /// Check if correlation ID exists
    #[must_use]
    pub fn contains(&self, id: CorrelationId) -> bool {
        self.yields.contains_key(&id)
    }

    /// Remove expired yields (timeout handling)
    ///
    /// Removes and returns all yield states that have been suspended
    /// for longer than the specified timeout duration.
    pub fn remove_expired(&mut self, timeout: Duration) -> Vec<YieldState> {
        let now = Instant::now();
        let expired_ids: Vec<CorrelationId> = self
            .yields
            .iter()
            .filter(|(_, state)| now.duration_since(state.request_sent_at) > timeout)
            .map(|(id, _)| *id)
            .collect();

        expired_ids
            .into_iter()
            .filter_map(|id| self.yields.remove(&id))
            .collect()
    }

    /// Count of suspended yields
    #[must_use]
    pub fn len(&self) -> usize {
        self.yields.len()
    }

    /// Check if no yields are suspended
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.yields.is_empty()
    }

    /// Clear all suspended yields
    pub fn clear(&mut self) {
        self.yields.clear();
    }
}

impl Default for SuspendedYields {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_core::{Expr, Value};

    // ============================================================
    // CorrelationId Tests
    // ============================================================

    #[test]
    fn correlation_id_new_generates_unique_ids() {
        let id1 = CorrelationId::new();
        let id2 = CorrelationId::new();
        let id3 = CorrelationId::new();

        assert_ne!(id1, id2, "Correlation IDs should be unique");
        assert_ne!(id2, id3, "Correlation IDs should be unique");
        assert_ne!(id1, id3, "Correlation IDs should be unique");
    }

    #[test]
    fn correlation_id_new_increments_monotonically() {
        let id1 = CorrelationId::new();
        let id2 = CorrelationId::new();

        assert!(id2.0 > id1.0, "IDs should increment monotonically");
    }

    #[test]
    fn correlation_id_next_increments_counter() {
        let mut counter = 1u64;

        let id1 = CorrelationId::next(&mut counter);
        let id2 = CorrelationId::next(&mut counter);
        let id3 = CorrelationId::next(&mut counter);

        assert_eq!(id1.0, 1);
        assert_eq!(id2.0, 2);
        assert_eq!(id3.0, 3);
        assert_eq!(counter, 4);
    }

    #[test]
    fn correlation_id_default_creates_new_id() {
        let id1 = CorrelationId::default();
        let id2 = CorrelationId::default();

        assert_ne!(id1, id2, "Default should generate unique IDs");
        assert!(id1.0 > 0, "Default ID should be non-zero");
    }

    // ============================================================
    // YieldState Creation Tests
    // ============================================================

    fn create_test_workflow() -> Workflow {
        Workflow::Ret {
            expr: Expr::Literal(Value::Int(42)),
        }
    }

    fn create_test_yield_state() -> YieldState {
        YieldState {
            correlation_id: CorrelationId::new(),
            expected_response_type: Type::Int,
            continuation: create_test_workflow(),
            origin_workflow: "instance-1".to_string(),
            target_role: "admin".to_string(),
            request_sent_at: Instant::now(),
        }
    }

    #[test]
    fn yield_state_creation() {
        let state = create_test_yield_state();

        assert_eq!(state.expected_response_type, Type::Int);
        assert_eq!(state.origin_workflow, "instance-1");
        assert_eq!(state.target_role, "admin");
        assert!(state.correlation_id.0 > 0);
    }

    #[test]
    fn yield_state_clone() {
        let state = create_test_yield_state();
        let cloned = state.clone();

        assert_eq!(state.correlation_id, cloned.correlation_id);
        assert_eq!(state.expected_response_type, cloned.expected_response_type);
        assert_eq!(state.origin_workflow, cloned.origin_workflow);
        assert_eq!(state.target_role, cloned.target_role);
    }

    // ============================================================
    // SuspendedYields Tests
    // ============================================================

    #[test]
    fn suspended_yields_new_is_empty() {
        let suspended = SuspendedYields::new();

        assert!(suspended.is_empty());
        assert_eq!(suspended.len(), 0);
    }

    #[test]
    fn suspended_yields_default_is_empty() {
        let suspended: SuspendedYields = Default::default();

        assert!(suspended.is_empty());
        assert_eq!(suspended.len(), 0);
    }

    #[test]
    fn suspend_and_resume() {
        let mut suspended = SuspendedYields::new();
        let state = create_test_yield_state();
        let correlation_id = state.correlation_id;

        // Suspend the workflow
        let id = suspended.suspend(state);

        assert_eq!(id, correlation_id);
        assert_eq!(suspended.len(), 1);
        assert!(suspended.contains(id));

        // Resume the workflow
        let resumed = suspended.resume(id);

        assert!(resumed.is_some());
        assert_eq!(resumed.unwrap().correlation_id, correlation_id);
        assert_eq!(suspended.len(), 0);
        assert!(!suspended.contains(id));
    }

    #[test]
    fn suspend_generates_id_if_zero() {
        let mut suspended = SuspendedYields::new();
        let mut state = create_test_yield_state();
        state.correlation_id = CorrelationId(0); // Set to invalid ID

        let id = suspended.suspend(state);

        assert!(id.0 > 0, "Should generate a non-zero ID");
        assert!(suspended.contains(id));
    }

    #[test]
    fn get_without_removing() {
        let mut suspended = SuspendedYields::new();
        let state = create_test_yield_state();
        let correlation_id = state.correlation_id;

        suspended.suspend(state);

        // Get should not remove
        let got = suspended.get(correlation_id);
        assert!(got.is_some());
        assert_eq!(got.unwrap().correlation_id, correlation_id);
        assert_eq!(suspended.len(), 1);
        assert!(suspended.contains(correlation_id));
    }

    #[test]
    fn contains_check() {
        let mut suspended = SuspendedYields::new();
        let state = create_test_yield_state();
        let id = suspended.suspend(state);

        assert!(suspended.contains(id));
        assert!(!suspended.contains(CorrelationId(999999)));
    }

    #[test]
    fn resume_non_existent_id() {
        let mut suspended = SuspendedYields::new();

        let result = suspended.resume(CorrelationId(999999));

        assert!(result.is_none());
    }

    #[test]
    fn get_non_existent_id() {
        let suspended = SuspendedYields::new();

        let result = suspended.get(CorrelationId(999999));

        assert!(result.is_none());
    }

    #[test]
    fn multiple_concurrent_yields() {
        let mut suspended = SuspendedYields::new();

        // Create multiple yield states
        let state1 = create_test_yield_state();
        let id1 = state1.correlation_id;

        let mut state2 = create_test_yield_state();
        state2.target_role = "user".to_string();
        let id2 = state2.correlation_id;

        let mut state3 = create_test_yield_state();
        state3.target_role = "moderator".to_string();
        let id3 = state3.correlation_id;

        // Suspend all
        suspended.suspend(state1);
        suspended.suspend(state2);
        suspended.suspend(state3);

        assert_eq!(suspended.len(), 3);
        assert!(suspended.contains(id1));
        assert!(suspended.contains(id2));
        assert!(suspended.contains(id3));

        // Resume in different order
        let resumed2 = suspended.resume(id2).unwrap();
        assert_eq!(resumed2.target_role, "user");
        assert_eq!(suspended.len(), 2);

        let resumed1 = suspended.resume(id1).unwrap();
        assert_eq!(resumed1.target_role, "admin");
        assert_eq!(suspended.len(), 1);

        let resumed3 = suspended.resume(id3).unwrap();
        assert_eq!(resumed3.target_role, "moderator");
        assert_eq!(suspended.len(), 0);
    }

    #[test]
    fn remove_expired_yields() {
        let mut suspended = SuspendedYields::new();

        // Create a state with an old timestamp
        let old_state = YieldState {
            correlation_id: CorrelationId::new(),
            expected_response_type: Type::Int,
            continuation: create_test_workflow(),
            origin_workflow: "instance-old".to_string(),
            target_role: "admin".to_string(),
            request_sent_at: Instant::now() - Duration::from_secs(100),
        };
        let old_id = old_state.correlation_id;

        // Create a state with a recent timestamp
        let recent_state = YieldState {
            correlation_id: CorrelationId::new(),
            expected_response_type: Type::String,
            continuation: create_test_workflow(),
            origin_workflow: "instance-recent".to_string(),
            target_role: "user".to_string(),
            request_sent_at: Instant::now() - Duration::from_secs(5),
        };
        let recent_id = recent_state.correlation_id;

        suspended.suspend(old_state);
        suspended.suspend(recent_state);

        assert_eq!(suspended.len(), 2);

        // Remove yields older than 60 seconds
        let expired = suspended.remove_expired(Duration::from_secs(60));

        assert_eq!(expired.len(), 1);
        assert_eq!(expired[0].origin_workflow, "instance-old");
        assert_eq!(expired[0].correlation_id, old_id);

        // Only recent yield should remain
        assert_eq!(suspended.len(), 1);
        assert!(!suspended.contains(old_id));
        assert!(suspended.contains(recent_id));
    }

    #[test]
    fn remove_expired_with_no_expired_yields() {
        let mut suspended = SuspendedYields::new();
        let state = create_test_yield_state();
        let id = state.correlation_id;

        suspended.suspend(state);

        // Remove yields older than 1 hour (none should be expired)
        let expired = suspended.remove_expired(Duration::from_secs(3600));

        assert!(expired.is_empty());
        assert_eq!(suspended.len(), 1);
        assert!(suspended.contains(id));
    }

    #[test]
    fn remove_expired_empty_registry() {
        let mut suspended = SuspendedYields::new();

        let expired = suspended.remove_expired(Duration::from_secs(60));

        assert!(expired.is_empty());
        assert!(suspended.is_empty());
    }

    #[test]
    fn clear_removes_all() {
        let mut suspended = SuspendedYields::new();

        suspended.suspend(create_test_yield_state());
        suspended.suspend(create_test_yield_state());
        suspended.suspend(create_test_yield_state());

        assert_eq!(suspended.len(), 3);

        suspended.clear();

        assert!(suspended.is_empty());
        assert_eq!(suspended.len(), 0);
    }

    #[test]
    fn len_and_is_empty_consistency() {
        let mut suspended = SuspendedYields::new();

        assert!(suspended.is_empty());
        assert_eq!(suspended.len(), 0);

        let id1 = suspended.suspend(create_test_yield_state());
        assert!(!suspended.is_empty());
        assert_eq!(suspended.len(), 1);

        let id2 = suspended.suspend(create_test_yield_state());
        assert!(!suspended.is_empty());
        assert_eq!(suspended.len(), 2);

        suspended.resume(id1);
        assert!(!suspended.is_empty());
        assert_eq!(suspended.len(), 1);

        suspended.resume(id2);
        assert!(suspended.is_empty());
        assert_eq!(suspended.len(), 0);
    }

    #[test]
    fn suspend_with_existing_id_preserves_id() {
        let mut suspended = SuspendedYields::new();
        let custom_id = CorrelationId(12345);

        let state = YieldState {
            correlation_id: custom_id,
            expected_response_type: Type::Int,
            continuation: create_test_workflow(),
            origin_workflow: "test".to_string(),
            target_role: "test-role".to_string(),
            request_sent_at: Instant::now(),
        };

        let returned_id = suspended.suspend(state);

        assert_eq!(returned_id, custom_id);
        assert!(suspended.contains(custom_id));
    }

    #[test]
    fn yield_state_ownership_transfer() {
        let mut suspended = SuspendedYields::new();
        let state = create_test_yield_state();
        let id = state.correlation_id;

        // suspend takes ownership
        suspended.suspend(state);

        // resume transfers ownership back
        let resumed = suspended.resume(id).unwrap();

        // We can access the resumed state
        assert_eq!(resumed.correlation_id, id);
        assert_eq!(resumed.target_role, "admin");
    }

    #[test]
    fn debug_formatting() {
        let id = CorrelationId(42);
        let debug_str = format!("{:?}", id);
        assert!(debug_str.contains("42"), "Debug should contain the ID value");

        let suspended = SuspendedYields::new();
        let debug_str = format!("{:?}", suspended);
        assert!(
            debug_str.contains("SuspendedYields"),
            "Debug should show struct name"
        );
    }

    #[test]
    fn clone_suspended_yields() {
        let mut original = SuspendedYields::new();
        original.suspend(create_test_yield_state());
        original.suspend(create_test_yield_state());

        let cloned = original.clone();

        assert_eq!(original.len(), cloned.len());

        // Modifying original should not affect clone
        original.clear();
        assert!(original.is_empty());
        assert_eq!(cloned.len(), 2);
    }
}
