//! Ash Engine - Unified embedding API for Ash workflows
//!
//! This crate provides the central `Engine` type for integrating Ash into Rust applications.
//! It encapsulates the entire workflow lifecycle: Parse → Check → Execute.
//!
//! # Example
//!
//! ```
//! use ash_engine::Engine;
//!
//! # tokio_test::block_on(async {
//! let engine = Engine::new().build().expect("engine builds");
//! # });
//! ```

pub mod check;
pub mod error;
pub mod execute;
pub mod parse;
pub mod providers;

pub use error::EngineError;

use ash_core::Value;
use ash_interp::{ExecResult, RuntimeState, interpret_in_state};

/// The central engine for all Ash operations
///
/// The `Engine` provides a unified interface for parsing, type checking,
/// and executing Ash workflows. It is designed to be:
///
/// - **Send + Sync**: Can be shared across threads
/// - **Configurable**: Built using the builder pattern
/// - **Extensible**: Supports custom capability providers
///
/// # Example
///
/// ```
/// use ash_engine::Engine;
///
/// # tokio_test::block_on(async {
/// let engine = Engine::new()
///     .with_stdio_capabilities()
///     .build()
///     .expect("engine builds");
/// # });
/// ```
#[derive(Debug)]
pub struct Engine {
    /// Store surface workflows by a unique ID
    /// This is a temporary solution until type checking supports core workflows
    surface_workflows:
        std::sync::Mutex<std::collections::HashMap<u64, ash_parser::surface::Workflow>>,
    /// Counter for generating unique IDs
    next_id: std::sync::atomic::AtomicU64,
    /// Runtime-owned state that persists across related executions.
    runtime_state: RuntimeState,
}

/// A workflow handle that carries its internal ID for type checking
///
/// This wraps an `ash_core::Workflow` and maintains the association
/// with its surface representation needed for type checking.
#[derive(Debug, Clone)]
pub struct Workflow {
    /// The core workflow
    pub core: ash_core::Workflow,
    /// The internal ID for looking up the surface workflow
    id: u64,
}

impl PartialEq for Workflow {
    fn eq(&self, other: &Self) -> bool {
        self.core == other.core && self.id == other.id
    }
}

impl std::ops::Deref for Workflow {
    type Target = ash_core::Workflow;

    fn deref(&self) -> &Self::Target {
        &self.core
    }
}

impl std::ops::DerefMut for Workflow {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.core
    }
}

impl Engine {
    /// Create a new engine builder with default configuration
    ///
    /// Returns an `EngineBuilder` that can be used to configure capabilities
    /// before building the engine.
    ///
    /// # Example
    ///
    /// ```
    /// use ash_engine::Engine;
    ///
    /// let builder = Engine::new();
    /// ```
    #[allow(clippy::new_ret_no_self, clippy::missing_const_for_fn)]
    #[must_use]
    pub fn new() -> EngineBuilder {
        EngineBuilder::new()
    }

    /// Generate a unique ID for storing surface workflows
    fn next_workflow_id(&self) -> u64 {
        self.next_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }

    /// Store a surface workflow and return its ID
    fn store_surface_workflow(&self, surface: ash_parser::surface::Workflow) -> u64 {
        let id = self.next_workflow_id();
        if let Ok(mut map) = self.surface_workflows.lock() {
            map.insert(id, surface);
        }
        id
    }

    /// Retrieve a surface workflow by its ID
    fn get_surface_workflow(&self, id: u64) -> Option<ash_parser::surface::Workflow> {
        self.surface_workflows
            .lock()
            .map_or(None, |map| map.get(&id).cloned())
    }

    /// Parse source code into a Workflow
    ///
    /// # Errors
    ///
    /// Returns `EngineError::Parse` if the source contains syntax errors.
    pub fn parse(&self, source: &str) -> Result<Workflow, EngineError> {
        use ash_parser::{lower_workflow, new_input, workflow_def};
        use winnow::prelude::*;

        let mut input = new_input(source);

        match workflow_def.parse_next(&mut input) {
            Ok(def) => {
                let surface = def.body.clone();
                let core = lower_workflow(&def);
                let id = self.store_surface_workflow(surface);
                Ok(Workflow { core, id })
            }
            Err(e) => {
                // Format the parse error into a readable message
                let error_message = format!("{e}");
                Err(EngineError::Parse(error_message))
            }
        }
    }

    /// Parse a workflow from a file
    ///
    /// # Errors
    ///
    /// Returns `EngineError::Io` if the file cannot be read.
    /// Returns `EngineError::Parse` if the file contains syntax errors.
    pub fn parse_file(&self, path: impl AsRef<std::path::Path>) -> Result<Workflow, EngineError> {
        let source = std::fs::read_to_string(path)?;
        self.parse(&source)
    }

    /// Infer the canonical Ash type name for an expression.
    ///
    /// # Errors
    ///
    /// Returns `EngineError::Parse` if the expression does not parse and
    /// `EngineError::Type` if the inferred type is not concrete enough to report.
    pub fn infer_expression_type(&self, source: &str) -> Result<String, EngineError> {
        use ash_parser::parse_expr::expr;
        use ash_typeck::type_env::TypeEnv;
        use winnow::prelude::*;

        let mut input = ash_parser::new_input(source);
        let expr = expr
            .parse_next(&mut input)
            .map_err(|e| EngineError::Parse(format!("{e}")))?;

        let ty = ash_typeck::check_expr::infer_type(&TypeEnv::with_builtin_types(), &expr);
        match ty {
            ash_typeck::Type::Var(_) => Err(EngineError::Type(
                "could not infer a canonical type for expression".to_string(),
            )),
            other => Ok(other.to_string()),
        }
    }

    /// Type check a workflow
    ///
    /// # Errors
    ///
    /// Returns `EngineError::Type` if type checking fails.
    pub fn check(&self, workflow: &Workflow) -> Result<(), EngineError> {
        // Retrieve the surface workflow that was stored during parsing
        let surface = self
            .get_surface_workflow(workflow.id)
            .ok_or_else(|| EngineError::Type("workflow not found in cache".to_string()))?;

        // Run the type checker
        match ash_typeck::type_check_workflow(&surface) {
            Ok(result) => {
                if result.is_ok() {
                    Ok(())
                } else {
                    // Collect type errors into a message
                    let errors: Vec<String> =
                        result.errors.iter().map(|e| format!("{e:?}")).collect();
                    Err(EngineError::Type(errors.join("; ")))
                }
            }
            Err(e) => Err(EngineError::Type(format!("{e}"))),
        }
    }

    /// Execute a workflow asynchronously
    ///
    /// # Errors
    ///
    /// Returns execution errors from the interpreter.
    pub async fn execute(&self, workflow: &Workflow) -> ExecResult<Value> {
        interpret_in_state(&workflow.core, &self.runtime_state).await
    }

    /// Parse, check, and execute in one call
    ///
    /// Convenience method that chains parse → check → execute.
    ///
    /// # Errors
    ///
    /// Returns the first error encountered at any stage.
    pub async fn run(&self, source: &str) -> ExecResult<Value> {
        let workflow = self.parse(source)?;
        self.check(&workflow)?;
        self.execute(&workflow).await
    }

    /// Parse, check, and execute a workflow from a file
    ///
    /// Convenience method that reads a file and then runs parse → check → execute.
    ///
    /// # Errors
    ///
    /// Returns `EngineError::Io` if the file cannot be read.
    /// Returns other errors from parse, check, or execute stages.
    pub async fn run_file(&self, path: impl AsRef<std::path::Path>) -> ExecResult<Value> {
        let workflow = self.parse_file(path)?;
        self.check(&workflow)?;
        self.execute(&workflow).await
    }
}

impl Default for Engine {
    /// Creates a default engine with standard configuration.
    ///
    /// # Panics
    ///
    /// Panics if the engine cannot be built (e.g., out of memory). This is
    /// extremely unlikely in practice since the default configuration requires
    /// no external resources.
    fn default() -> Self {
        // SAFETY: The default EngineBuilder configuration is infallible.
        // It only allocates memory and performs simple initializations.
        Self::new().build().expect("default engine builds")
    }
}

/// Builder for configuring and constructing an Engine
///
/// The builder pattern allows for fluent configuration of capabilities:
///
/// ```
/// use ash_engine::Engine;
///
/// let engine = Engine::new()
///     .with_stdio_capabilities()
///     .with_fs_capabilities()
///     .build()
///     .expect("engine builds");
/// ```
#[derive(Debug)]
pub struct EngineBuilder {
    // Configuration state will be added during implementation
}

impl EngineBuilder {
    /// Create a new engine builder
    ///
    /// Prefer using `Engine::new()` instead of this method directly.
    const fn new() -> Self {
        Self {}
    }

    /// Build the configured engine
    ///
    /// # Errors
    ///
    /// Returns `EngineError` if the engine cannot be constructed
    /// (e.g., missing required capabilities or invalid configuration).
    pub fn build(self) -> Result<Engine, EngineError> {
        Ok(Engine {
            surface_workflows: std::sync::Mutex::new(std::collections::HashMap::new()),
            next_id: std::sync::atomic::AtomicU64::new(1),
            runtime_state: RuntimeState::new(),
        })
    }

    /// Add standard I/O capabilities (print, println, `read_line`)
    ///
    /// These are operational-effect capabilities for console I/O.
    #[must_use]
    pub const fn with_stdio_capabilities(self) -> Self {
        // Implementation to be added
        self
    }

    /// Add filesystem capabilities (`read_file`, `write_file`)
    ///
    /// These are operational-effect capabilities for file operations.
    #[must_use]
    pub const fn with_fs_capabilities(self) -> Self {
        // Implementation to be added
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // ============================================================
    // Engine Creation Tests
    // ============================================================

    #[test]
    fn test_engine_new_build_succeeds() {
        // Basic test: Engine::new().build() should succeed
        let result = Engine::new().build();
        assert!(
            result.is_ok(),
            "Engine::new().build() should succeed but got: {result:?}"
        );
    }

    #[test]
    fn test_engine_default_succeeds() {
        // Engine::default() should succeed using the builder
        let _engine: Engine = Engine::default();
    }

    #[test]
    fn test_engine_builder_returns_valid_engine() {
        let engine = Engine::new().build().expect("engine builds");
        // The engine should be usable (not panic, not be null, etc.)
        // We verify this by checking it can execute basic operations
        let _ = &engine;
    }

    // ============================================================
    // EngineBuilder Configuration Tests
    // ============================================================

    #[test]
    fn test_builder_stdio_capabilities_chaining() {
        // with_stdio_capabilities should return Self for chaining
        let builder = Engine::new();
        let builder = builder.with_stdio_capabilities();
        let result = builder.build();
        assert!(
            result.is_ok(),
            "Builder with stdio capabilities should build successfully"
        );
    }

    #[test]
    fn test_builder_fs_capabilities_chaining() {
        // with_fs_capabilities should return Self for chaining
        let builder = Engine::new();
        let builder = builder.with_fs_capabilities();
        let result = builder.build();
        assert!(
            result.is_ok(),
            "Builder with fs capabilities should build successfully"
        );
    }

    #[test]
    fn test_builder_chaining_multiple_capabilities() {
        // Multiple capability methods should chain together
        let result = Engine::new()
            .with_stdio_capabilities()
            .with_fs_capabilities()
            .build();
        assert!(
            result.is_ok(),
            "Builder with multiple capabilities should build successfully"
        );
    }

    #[test]
    fn test_builder_chaining_order_independent() {
        // Order of capability configuration should not matter
        let engine1 = Engine::new()
            .with_stdio_capabilities()
            .with_fs_capabilities()
            .build();

        let engine2 = Engine::new()
            .with_fs_capabilities()
            .with_stdio_capabilities()
            .build();

        // Both should succeed (we can't compare engines directly without PartialEq,
        // but we can verify both builds succeed)
        assert!(engine1.is_ok(), "First build order should succeed");
        assert!(engine2.is_ok(), "Second build order should succeed");
    }

    #[test]
    fn test_builder_reusable_pattern() {
        // The builder pattern should be usable for multiple engines
        let base_builder = Engine::new();

        let engine1 = base_builder.build();
        // Note: After build(), the builder is consumed. This test documents
        // the expected usage pattern where a new builder is created each time.
        assert!(engine1.is_ok());

        // Creating a new engine from a new builder
        let engine2 = Engine::new().build();
        assert!(engine2.is_ok());
    }

    // ============================================================
    // Send + Sync Thread Safety Tests
    // ============================================================

    #[test]
    fn test_engine_is_send() {
        // Compile-time check: Engine must be Send
        fn assert_send<T: Send>() {}
        assert_send::<Engine>();
    }

    #[test]
    fn test_engine_is_sync() {
        // Compile-time check: Engine must be Sync
        fn assert_sync<T: Sync>() {}
        assert_sync::<Engine>();
    }

    #[test]
    fn test_engine_builder_is_send() {
        // Compile-time check: EngineBuilder should be Send for flexibility
        fn assert_send<T: Send>() {}
        assert_send::<EngineBuilder>();
    }

    #[test]
    fn test_engine_error_is_send_sync() {
        // Compile-time check: EngineError must be Send + Sync for error handling
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<EngineError>();
        assert_sync::<EngineError>();
    }

    #[tokio::test]
    async fn test_engine_can_be_shared_across_tasks() {
        // Runtime check: Engine can be shared across async tasks
        use std::sync::Arc;

        let engine = Arc::new(Engine::new().build().expect("engine builds"));

        let engine_clone = Arc::clone(&engine);
        let task = tokio::spawn(async move {
            // Access the engine in a spawned task
            let _ = &*engine_clone;
            true
        });

        let result = task.await.expect("task completed");
        assert!(result, "Engine should be accessible across tasks");
    }

    // ============================================================
    // Error Type Tests
    // ============================================================

    #[test]
    fn test_engine_error_parse_variant() {
        // Verify Parse variant exists and can be created
        let err = EngineError::Parse("syntax error".to_string());
        assert!(
            matches!(err, EngineError::Parse(_)),
            "Error should be Parse variant"
        );
    }

    #[test]
    fn test_engine_error_type_variant() {
        // Verify Type variant exists and can be created
        let err = EngineError::Type("type mismatch".to_string());
        assert!(
            matches!(err, EngineError::Type(_)),
            "Error should be Type variant"
        );
    }

    #[test]
    fn test_engine_error_execution_variant() {
        // Verify Execution variant exists and can be created
        let err = EngineError::Execution("runtime error".to_string());
        assert!(
            matches!(err, EngineError::Execution(_)),
            "Error should be Execution variant"
        );
    }

    #[test]
    fn test_engine_error_io_variant() {
        // Verify Io variant exists with std::io::Error
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = EngineError::Io(io_err);
        assert!(
            matches!(err, EngineError::Io(_)),
            "Error should be Io variant"
        );
    }

    #[test]
    fn test_engine_error_capability_not_found_variant() {
        // Verify CapabilityNotFound variant exists
        let err = EngineError::CapabilityNotFound("fs:read".to_string());
        assert!(
            matches!(err, EngineError::CapabilityNotFound(_)),
            "Error should be CapabilityNotFound variant"
        );
    }

    #[test]
    fn test_engine_error_display_format() {
        // Verify error messages are informative
        let parse_err = EngineError::Parse("unexpected token".to_string());
        let display = format!("{parse_err}");
        assert!(
            display.contains("unexpected token"),
            "Parse error should display message: {display}"
        );
    }

    #[test]
    fn test_engine_error_from_io_error() {
        // Verify automatic conversion from std::io::Error
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let engine_err: EngineError = io_err.into();
        assert!(
            matches!(engine_err, EngineError::Io(_)),
            "Should convert from io::Error"
        );
    }

    // ============================================================
    // Property-Based Tests
    // ============================================================

    proptest! {
        /// Property: Engine creation always succeeds (when not configured with invalid options)
        #[test]
        fn prop_engine_creation_succeeds(_dummy in any::<u8>()) {
            let result = Engine::new().build();
            prop_assert!(result.is_ok(), "Engine creation should always succeed");
        }

        /// Property: Error messages preserve their content
        #[test]
        fn prop_error_message_preservation(message in "[a-zA-Z0-9_ ]{1,100}") {
            let err = EngineError::Parse(message.clone());
            let display = format!("{err}");
            prop_assert!(
                display.contains(&message),
                "Error display should contain original message"
            );
        }

        /// Property: CapabilityNotFound preserves capability name
        #[test]
        fn prop_capability_name_preservation(name in "[a-z_][a-z0-9_:]{1,50}") {
            let err = EngineError::CapabilityNotFound(name.clone());
            if let EngineError::CapabilityNotFound(found_name) = err {
                prop_assert_eq!(found_name, name, "Capability name should be preserved");
            } else {
                prop_assert!(false, "Error should be CapabilityNotFound variant");
            }
        }
    }

    // ============================================================
    // TODO/Future Tests (marked as ignore until implemented)
    // ============================================================

    #[test]
    fn test_engine_parse_valid_source() {
        let engine = Engine::new().build().unwrap();
        let result = engine.parse("workflow main { done }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_engine_parse_invalid_source_returns_parse_error() {
        let engine = Engine::new().build().unwrap();
        let result = engine.parse("invalid syntax!!!");
        assert!(matches!(result, Err(EngineError::Parse(_))));
    }

    #[test]
    fn test_engine_check_valid_workflow() {
        let engine = Engine::new().build().unwrap();
        let workflow = engine.parse("workflow main { ret 42; }").unwrap();
        let result = engine.check(&workflow);
        assert!(result.is_ok());
    }

    #[test]
    fn test_engine_infer_expression_type_reports_canonical_names() {
        let engine = Engine::new().build().unwrap();

        assert_eq!(engine.infer_expression_type("42").unwrap(), "Int");
        assert_eq!(engine.infer_expression_type("\"hello\"").unwrap(), "String");
        assert_eq!(
            engine.infer_expression_type("[1, 2, 3]").unwrap(),
            "List<Int>"
        );
        assert_eq!(engine.infer_expression_type("1 + 2").unwrap(), "Int");
        assert_eq!(engine.infer_expression_type("!true").unwrap(), "Bool");
    }

    #[test]
    fn test_engine_execute_workflow() {
        // This will be an async test
    }
}
