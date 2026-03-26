//! Yield routing by role for proxy workflow request/response
//!
//! Provides runtime routing of `yield role(R)` statements to appropriate role handlers
//! per SPEC-023 Section 6.
//!
//! # Example
//!
//! ```
//! use ash_interp::yield_routing::{YieldRouter, YieldId, YieldError};
//! use ash_core::{WorkflowId, Value, Workflow};
//! use ash_interp::context::Context;
//! use std::collections::HashMap;
//!
//! let mut router = YieldRouter::new();
//!
//! // Register a handler for a role
//! let handler_id = WorkflowId::new();
//! router.register_handler("ai_assistant", handler_id);
//!
//! // Route a yield to the handler
//! let caller_id = WorkflowId::new();
//! let mut record_data = HashMap::new();
//! record_data.insert("data".to_string(), Value::Int(42));
//! let request = Value::Record(Box::new(record_data));
//! let continuation = Workflow::Done;
//! let ctx = Context::new();
//!
//! let yield_id = router.route_yield(
//!     caller_id,
//!     "ai_assistant",
//!     request,
//!     continuation,
//!     ctx,
//! ).unwrap();
//!
//! // Check if yield is pending
//! assert!(router.is_pending(&yield_id));
//! ```

use ash_core::{Value, Workflow, WorkflowId};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::context::Context;
use crate::error::ExecError;

/// Unique identifier for a yield operation
///
/// Each yield expression generates a unique YieldId that is used to
/// track the pending yield and match it with the corresponding resume.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct YieldId(pub u64);

impl YieldId {
    /// Generate a new unique yield ID
    #[must_use]
    pub fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::SeqCst))
    }
}

impl Default for YieldId {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur during yield routing operations
#[derive(Debug, Clone, PartialEq)]
pub enum YieldError {
    /// No handler registered for the requested role
    NoHandlerForRole(String),
    /// Unknown yield ID - no pending yield found
    UnknownYield(YieldId),
    /// Handler is currently busy and cannot accept new requests
    HandlerBusy,
    /// A yield with this ID is already pending
    YieldAlreadyPending,
}

impl std::fmt::Display for YieldError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoHandlerForRole(role) => write!(f, "no handler registered for role: {role}"),
            Self::UnknownYield(id) => write!(f, "unknown yield ID: {}", id.0),
            Self::HandlerBusy => write!(f, "handler is busy"),
            Self::YieldAlreadyPending => write!(f, "yield already pending"),
        }
    }
}

impl std::error::Error for YieldError {}

/// Tracks suspended yield state awaiting response
///
/// Contains all information needed to resume a workflow after receiving
/// a response from a proxy handler.
#[derive(Debug, Clone)]
pub struct PendingYield {
    /// Unique yield identifier
    pub yield_id: YieldId,
    /// ID of the workflow that yielded (caller)
    pub caller: WorkflowId,
    /// Target role that should handle the yield
    pub role: String,
    /// Request value sent to the handler
    pub request: Value,
    /// Continuation workflow to execute after resume
    pub continuation: Workflow,
    /// Correlation ID for matching yield/resume pairs
    pub correlation_id: crate::yield_state::CorrelationId,
    /// Captured variable bindings from the execution context
    pub saved_bindings: HashMap<String, Value>,
}

/// Result of resuming a workflow
#[derive(Debug)]
pub struct ResumeResult {
    /// ID of the workflow that was resumed (caller)
    pub caller: WorkflowId,
    /// Result of executing the continuation
    pub result: Result<Value, ExecError>,
}

/// Routes yields to role handlers
///
/// Manages the mapping from roles to handler workflows and tracks
/// pending yields awaiting responses.
#[derive(Debug, Default)]
pub struct YieldRouter {
    /// Role name -> current handler workflow ID
    handlers: HashMap<String, WorkflowId>,
    /// Pending yields awaiting response (by YieldId)
    pending: HashMap<YieldId, PendingYield>,
    /// Pending yields by correlation ID for lookup during resume
    pending_by_correlation: HashMap<crate::yield_state::CorrelationId, YieldId>,
}

impl YieldRouter {
    /// Create a new empty yield router
    #[must_use]
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            pending: HashMap::new(),
            pending_by_correlation: HashMap::new(),
        }
    }

    /// Register a workflow as handler for a role
    ///
    /// If a handler was already registered for this role, it will be
    /// replaced with the new handler.
    pub fn register_handler(&mut self, role: impl Into<String>, workflow: WorkflowId) {
        self.handlers.insert(role.into(), workflow);
    }

    /// Unregister a handler for a role
    ///
    /// Returns the workflow ID of the removed handler if one was registered.
    pub fn unregister_handler(&mut self, role: &str) -> Option<WorkflowId> {
        self.handlers.remove(role)
    }

    /// Get the current handler for a role
    #[must_use]
    pub fn get_handler(&self, role: &str) -> Option<WorkflowId> {
        self.handlers.get(role).copied()
    }

    /// Route a yield to the appropriate handler
    ///
    /// # Arguments
    /// * `caller` - ID of the workflow that is yielding
    /// * `role` - Target role to yield to
    /// * `request` - Request value to send to the handler
    /// * `continuation` - Workflow to execute after resume
    /// * `context` - Execution context with variable bindings to capture
    ///
    /// # Returns
    /// The YieldId for tracking this yield, or an error if routing fails
    ///
    /// # Errors
    /// * `NoHandlerForRole` - If no handler is registered for the role
    /// * `YieldAlreadyPending` - If a yield with a duplicate ID somehow exists
    pub fn route_yield(
        &mut self,
        caller: WorkflowId,
        role: &str,
        request: Value,
        continuation: Workflow,
        context: Context,
    ) -> Result<YieldId, YieldError> {
        // Look up handler for the role
        let _handler = self
            .handlers
            .get(role)
            .ok_or_else(|| YieldError::NoHandlerForRole(role.to_string()))?;

        // Generate unique yield ID
        let yield_id = YieldId::new();

        // Generate correlation ID for yield/resume matching
        let correlation_id = crate::yield_state::CorrelationId::new();

        // Capture bindings from context
        let saved_bindings = context.local_bindings().clone();

        // Create pending yield entry
        let pending = PendingYield {
            yield_id,
            caller,
            role: role.to_string(),
            request: request.clone(),
            continuation,
            correlation_id,
            saved_bindings,
        };

        // Check for duplicate yield ID (shouldn't happen with UUIDs)
        if self.pending.contains_key(&yield_id) {
            return Err(YieldError::YieldAlreadyPending);
        }

        // Store the pending yield
        self.pending.insert(yield_id, pending);
        self.pending_by_correlation.insert(correlation_id, yield_id);

        // In a full implementation, we would send the request to the handler here
        // For now, the handler is expected to poll for pending yields

        Ok(yield_id)
    }

    /// Resume a workflow with a response value
    ///
    /// # Arguments
    /// * `yield_id` - ID of the yield to resume
    /// * `response` - Response value to bind to the resume variable
    ///
    /// # Returns
    /// ResumeResult containing the caller workflow ID and execution result
    ///
    /// # Errors
    /// * `UnknownYield` - If the yield ID is not found in pending yields
    pub fn resume_with_response(
        &mut self,
        yield_id: YieldId,
        response: Value,
    ) -> Result<ResumeResult, YieldError> {
        // Remove the pending yield
        let pending = self
            .pending
            .remove(&yield_id)
            .ok_or(YieldError::UnknownYield(yield_id))?;

        // Remove from correlation index
        self.pending_by_correlation.remove(&pending.correlation_id);

        // Note: In the full implementation, the continuation workflow would be
        // executed with the response bound to the resume variable.
        // For now, we return a success result indicating the resume was processed.
        // The actual continuation execution happens in execute.rs's ProxyResume handling.

        // The continuation is stored but not executed here - execute.rs handles that
        // when ProxyResume is processed with the correlation_id
        // The saved_bindings from pending.saved_bindings can be restored via Context::with_bindings
        // when needed for continuation execution.

        Ok(ResumeResult {
            caller: pending.caller,
            result: Ok(response),
        })
    }

    /// Resume a workflow by correlation ID
    ///
    /// This is an alternative to resume_with_response that uses the correlation
    /// ID from the ProxyResume workflow variant.
    ///
    /// # Arguments
    /// * `correlation_id` - Correlation ID from the original yield
    /// * `response` - Response value to bind to the resume variable
    ///
    /// # Returns
    /// ResumeResult containing the caller workflow ID and execution result
    pub fn resume_by_correlation(
        &mut self,
        correlation_id: crate::yield_state::CorrelationId,
        response: Value,
    ) -> Result<ResumeResult, YieldError> {
        // Look up the yield ID by correlation ID
        let yield_id = self
            .pending_by_correlation
            .get(&correlation_id)
            .copied()
            .ok_or(YieldError::UnknownYield(YieldId(0)))?;

        self.resume_with_response(yield_id, response)
    }

    /// Check if a yield is pending
    #[must_use]
    pub fn is_pending(&self, yield_id: &YieldId) -> bool {
        self.pending.contains_key(yield_id)
    }

    /// Check if a yield with the given correlation ID is pending
    #[must_use]
    pub fn is_pending_by_correlation(
        &self,
        correlation_id: crate::yield_state::CorrelationId,
    ) -> bool {
        self.pending_by_correlation.contains_key(&correlation_id)
    }

    /// Get pending yield details by yield ID
    #[must_use]
    pub fn get_pending(&self, yield_id: &YieldId) -> Option<&PendingYield> {
        self.pending.get(yield_id)
    }

    /// Get pending yield details by correlation ID
    #[must_use]
    pub fn get_pending_by_correlation(
        &self,
        correlation_id: crate::yield_state::CorrelationId,
    ) -> Option<&PendingYield> {
        self.pending_by_correlation
            .get(&correlation_id)
            .and_then(|id| self.pending.get(id))
    }

    /// Cancel a pending yield
    ///
    /// Returns the pending yield if it was found and removed, None otherwise.
    pub fn cancel_yield(&mut self, yield_id: YieldId) -> Option<PendingYield> {
        let pending = self.pending.remove(&yield_id)?;
        self.pending_by_correlation.remove(&pending.correlation_id);
        Some(pending)
    }

    /// Get all pending yields for a specific caller workflow
    pub fn get_pending_for_caller(&self, caller: WorkflowId) -> Vec<&PendingYield> {
        self.pending
            .values()
            .filter(|p| p.caller == caller)
            .collect()
    }

    /// Get all pending yields for a specific role
    pub fn get_pending_for_role(&self, role: &str) -> Vec<&PendingYield> {
        self.pending.values().filter(|p| p.role == role).collect()
    }

    /// Get count of pending yields
    #[must_use]
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Get all registered handlers
    pub fn handlers(&self) -> &HashMap<String, WorkflowId> {
        &self.handlers
    }

    /// Clear all pending yields
    pub fn clear_pending(&mut self) {
        self.pending.clear();
        self.pending_by_correlation.clear();
    }

    /// Clear all handlers
    pub fn clear_handlers(&mut self) {
        self.handlers.clear();
    }

    /// Clear all state (handlers and pending yields)
    pub fn clear(&mut self) {
        self.clear_handlers();
        self.clear_pending();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_core::Expr;

    fn test_workflow() -> Workflow {
        Workflow::Ret {
            expr: Expr::Literal(Value::Int(42)),
        }
    }

    #[test]
    fn yield_id_is_unique() {
        let id1 = YieldId::new();
        let id2 = YieldId::new();
        let id3 = YieldId::new();

        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_ne!(id1, id3);
    }

    #[test]
    fn router_new_is_empty() {
        let router = YieldRouter::new();

        assert_eq!(router.pending_count(), 0);
        assert!(router.handlers().is_empty());
    }

    #[test]
    fn register_and_get_handler() {
        let mut router = YieldRouter::new();
        let workflow_id = WorkflowId::new();

        router.register_handler("test_role", workflow_id);

        assert_eq!(router.get_handler("test_role"), Some(workflow_id));
    }

    #[test]
    fn get_handler_unknown_role() {
        let router = YieldRouter::new();

        assert_eq!(router.get_handler("unknown"), None);
    }

    #[test]
    fn unregister_handler() {
        let mut router = YieldRouter::new();
        let workflow_id = WorkflowId::new();

        router.register_handler("test_role", workflow_id);
        let removed = router.unregister_handler("test_role");

        assert_eq!(removed, Some(workflow_id));
        assert_eq!(router.get_handler("test_role"), None);
    }

    #[test]
    fn route_yield_succeeds_with_handler() {
        let mut router = YieldRouter::new();
        let handler_id = WorkflowId::new();
        let caller_id = WorkflowId::new();

        router.register_handler("test_role", handler_id);

        let request = Value::Int(42);
        let continuation = test_workflow();
        let ctx = Context::new();

        let yield_id = router
            .route_yield(caller_id, "test_role", request, continuation, ctx)
            .unwrap();

        assert!(router.is_pending(&yield_id));
        assert_eq!(router.pending_count(), 1);
    }

    #[test]
    fn route_yield_fails_without_handler() {
        let mut router = YieldRouter::new();
        let caller_id = WorkflowId::new();

        let request = Value::Int(42);
        let continuation = test_workflow();
        let ctx = Context::new();

        let result = router.route_yield(caller_id, "unknown_role", request, continuation, ctx);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            YieldError::NoHandlerForRole(_)
        ));
    }

    #[test]
    fn resume_with_response_succeeds() {
        let mut router = YieldRouter::new();
        let handler_id = WorkflowId::new();
        let caller_id = WorkflowId::new();

        router.register_handler("test_role", handler_id);

        let request = Value::Int(42);
        let continuation = test_workflow();
        let ctx = Context::new();

        let yield_id = router
            .route_yield(caller_id, "test_role", request, continuation, ctx)
            .unwrap();

        let response = Value::String("success".to_string());
        let result = router.resume_with_response(yield_id, response);

        assert!(result.is_ok());
        let resume_result = result.unwrap();
        assert_eq!(resume_result.caller, caller_id);

        // Yield should no longer be pending
        assert!(!router.is_pending(&yield_id));
        assert_eq!(router.pending_count(), 0);
    }

    #[test]
    fn resume_unknown_yield_fails() {
        let mut router = YieldRouter::new();

        let result = router.resume_with_response(YieldId::new(), Value::Null);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), YieldError::UnknownYield(_)));
    }

    #[test]
    fn cancel_yield_removes_pending() {
        let mut router = YieldRouter::new();
        let handler_id = WorkflowId::new();
        let caller_id = WorkflowId::new();

        router.register_handler("test_role", handler_id);

        let yield_id = router
            .route_yield(
                caller_id,
                "test_role",
                Value::Null,
                test_workflow(),
                Context::new(),
            )
            .unwrap();

        assert!(router.is_pending(&yield_id));

        let cancelled = router.cancel_yield(yield_id);

        assert!(cancelled.is_some());
        assert!(!router.is_pending(&yield_id));
    }

    #[test]
    fn get_pending_details() {
        let mut router = YieldRouter::new();
        let handler_id = WorkflowId::new();
        let caller_id = WorkflowId::new();

        router.register_handler("test_role", handler_id);

        let request = Value::Int(42);
        let yield_id = router
            .route_yield(
                caller_id,
                "test_role",
                request.clone(),
                test_workflow(),
                Context::new(),
            )
            .unwrap();

        let pending = router.get_pending(&yield_id).unwrap();

        assert_eq!(pending.yield_id, yield_id);
        assert_eq!(pending.caller, caller_id);
        assert_eq!(pending.role, "test_role");
        assert_eq!(pending.request, request);
    }

    #[test]
    fn multiple_concurrent_yields() {
        let mut router = YieldRouter::new();
        let handler_id = WorkflowId::new();

        router.register_handler("test_role", handler_id);

        let caller1 = WorkflowId::new();
        let caller2 = WorkflowId::new();
        let caller3 = WorkflowId::new();

        let yield1 = router
            .route_yield(
                caller1,
                "test_role",
                Value::Int(1),
                test_workflow(),
                Context::new(),
            )
            .unwrap();
        let yield2 = router
            .route_yield(
                caller2,
                "test_role",
                Value::Int(2),
                test_workflow(),
                Context::new(),
            )
            .unwrap();
        let yield3 = router
            .route_yield(
                caller3,
                "test_role",
                Value::Int(3),
                test_workflow(),
                Context::new(),
            )
            .unwrap();

        assert_eq!(router.pending_count(), 3);
        assert!(router.is_pending(&yield1));
        assert!(router.is_pending(&yield2));
        assert!(router.is_pending(&yield3));

        // All yield IDs should be unique
        assert_ne!(yield1, yield2);
        assert_ne!(yield2, yield3);
        assert_ne!(yield1, yield3);
    }

    #[test]
    fn get_pending_for_caller() {
        let mut router = YieldRouter::new();
        let handler_id = WorkflowId::new();

        router.register_handler("role_a", handler_id);
        router.register_handler("role_b", handler_id);

        let caller1 = WorkflowId::new();
        let caller2 = WorkflowId::new();

        let _yield1 = router
            .route_yield(
                caller1,
                "role_a",
                Value::Int(1),
                test_workflow(),
                Context::new(),
            )
            .unwrap();
        let _yield2 = router
            .route_yield(
                caller1,
                "role_b",
                Value::Int(2),
                test_workflow(),
                Context::new(),
            )
            .unwrap();
        let _yield3 = router
            .route_yield(
                caller2,
                "role_a",
                Value::Int(3),
                test_workflow(),
                Context::new(),
            )
            .unwrap();

        let caller1_yields = router.get_pending_for_caller(caller1);
        let caller2_yields = router.get_pending_for_caller(caller2);

        assert_eq!(caller1_yields.len(), 2);
        assert_eq!(caller2_yields.len(), 1);
    }

    #[test]
    fn clear_pending_removes_all() {
        let mut router = YieldRouter::new();
        let handler_id = WorkflowId::new();

        router.register_handler("test_role", handler_id);

        for _ in 0..5 {
            let _ = router
                .route_yield(
                    WorkflowId::new(),
                    "test_role",
                    Value::Null,
                    test_workflow(),
                    Context::new(),
                )
                .unwrap();
        }

        assert_eq!(router.pending_count(), 5);

        router.clear_pending();

        assert_eq!(router.pending_count(), 0);
    }

    #[test]
    fn clear_handlers_removes_all() {
        let mut router = YieldRouter::new();

        router.register_handler("role_a", WorkflowId::new());
        router.register_handler("role_b", WorkflowId::new());

        assert_eq!(router.handlers().len(), 2);

        router.clear_handlers();

        assert!(router.handlers().is_empty());
    }

    #[test]
    fn resume_by_correlation_works() {
        let mut router = YieldRouter::new();
        let handler_id = WorkflowId::new();
        let caller_id = WorkflowId::new();

        router.register_handler("test_role", handler_id);

        let yield_id = router
            .route_yield(
                caller_id,
                "test_role",
                Value::Int(42),
                test_workflow(),
                Context::new(),
            )
            .unwrap();

        let pending = router.get_pending(&yield_id).unwrap();
        let correlation_id = pending.correlation_id;

        let result =
            router.resume_by_correlation(correlation_id, Value::String("done".to_string()));

        assert!(result.is_ok());
        assert!(!router.is_pending(&yield_id));
    }
}
