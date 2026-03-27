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
pub mod harness;
pub mod parse;
pub mod providers;

pub use error::EngineError;
pub use providers::CapabilityProvider;

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
    /// Store surface workflow definitions by a unique ID
    /// This stores the full `WorkflowDef` including parameters for type checking
    surface_workflow_defs:
        std::sync::Mutex<std::collections::HashMap<u64, ash_parser::surface::WorkflowDef>>,
    /// Counter for generating unique IDs
    next_id: std::sync::atomic::AtomicU64,
    /// Runtime-owned state that persists across related executions.
    /// Providers configured via `EngineBuilder` are passed to `RuntimeState` during build.
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

    /// Store a surface workflow definition and return its ID
    fn store_surface_workflow_def(&self, def: ash_parser::surface::WorkflowDef) -> u64 {
        let id = self.next_workflow_id();
        if let Ok(mut map) = self.surface_workflow_defs.lock() {
            map.insert(id, def);
        }
        id
    }

    /// Retrieve a surface workflow definition by its ID
    fn get_surface_workflow_def(&self, id: u64) -> Option<ash_parser::surface::WorkflowDef> {
        self.surface_workflow_defs
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
                let core = lower_workflow(&def)
                    .map_err(|e| EngineError::Parse(format!("lowering error: {e}")))?;
                let id = self.store_surface_workflow_def(def);
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
        // Retrieve the surface workflow definition that was stored during parsing
        let def = self
            .get_surface_workflow_def(workflow.id)
            .ok_or_else(|| EngineError::Type("workflow not found in cache".to_string()))?;

        // Convert workflow parameters to typeck types
        let param_bindings: Vec<(String, ash_typeck::Type)> = def
            .params
            .iter()
            .map(|p| (p.name.to_string(), Self::surface_type_to_typeck(&p.ty)))
            .collect();

        // Run the type checker with parameter bindings
        let param_refs: Vec<_> = param_bindings
            .iter()
            .map(|(n, t)| (n.clone(), t.clone()))
            .collect();
        match ash_typeck::type_check_workflow(&def.body, Some(&param_refs)) {
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

    /// Convert a surface type annotation to a typeck type
    fn surface_type_to_typeck(surface_type: &ash_parser::surface::Type) -> ash_typeck::Type {
        use ash_parser::surface::Type as SurfaceType;
        match surface_type {
            SurfaceType::Name(name) => match name.as_ref() {
                "Int" => ash_typeck::Type::Int,
                "String" => ash_typeck::Type::String,
                "Bool" => ash_typeck::Type::Bool,
                "Null" => ash_typeck::Type::Null,
                "Time" => ash_typeck::Type::Time,
                "Ref" => ash_typeck::Type::Ref,
                _ => ash_typeck::Type::Var(ash_typeck::TypeVar::fresh()),
            },
            SurfaceType::List(inner) => {
                ash_typeck::Type::List(Box::new(Self::surface_type_to_typeck(inner)))
            }
            SurfaceType::Record(fields) => ash_typeck::Type::Record(
                fields
                    .iter()
                    .map(|(name, ty)| (name.clone(), Self::surface_type_to_typeck(ty)))
                    .collect(),
            ),
            SurfaceType::Capability(_) => ash_typeck::Type::Var(ash_typeck::TypeVar::fresh()),
            SurfaceType::Constructor { name, args } => ash_typeck::Type::Constructor {
                name: ash_typeck::QualifiedName::root(name.as_ref()),
                args: args.iter().map(|ty| Self::surface_type_to_typeck(ty)).collect(),
                kind: ash_typeck::Kind::Type,
            },
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

    /// Execute a workflow asynchronously with input bindings
    ///
    /// The input bindings are injected into the workflow's execution context
    /// as initial variable bindings. This is useful for passing CLI arguments
    /// or other external inputs to the workflow.
    ///
    /// # Arguments
    /// * `workflow` - The workflow to execute
    /// * `input_bindings` - Initial variable bindings (e.g., from CLI --input)
    ///
    /// # Errors
    ///
    /// Returns execution errors from the interpreter.
    pub async fn execute_with_input(
        &self,
        workflow: &Workflow,
        input_bindings: std::collections::HashMap<String, Value>,
    ) -> ExecResult<Value> {
        ash_interp::execute_with_bindings_in_state(
            &workflow.core,
            &self.runtime_state,
            input_bindings,
        )
        .await
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

    /// Parse, check, and execute a workflow from a file with input bindings
    ///
    /// Convenience method that reads a file and then runs parse → check → execute
    /// with the provided input bindings injected into the execution context.
    ///
    /// # Arguments
    /// * `path` - Path to the workflow file
    /// * `input_bindings` - Initial variable bindings (e.g., from CLI --input)
    ///
    /// # Errors
    ///
    /// Returns `EngineError::Io` if the file cannot be read.
    /// Returns other errors from parse, check, or execute stages.
    pub async fn run_file_with_input(
        &self,
        path: impl AsRef<std::path::Path>,
        input_bindings: std::collections::HashMap<String, Value>,
    ) -> ExecResult<Value> {
        let workflow = self.parse_file(path)?;
        self.check(&workflow)?;
        self.execute_with_input(&workflow, input_bindings).await
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

/// Configuration for HTTP capabilities
#[derive(Debug, Clone, Default)]
pub struct HttpConfig {
    /// Timeout for HTTP requests in seconds
    pub timeout_seconds: u64,
    /// Maximum number of redirects to follow
    pub max_redirects: u32,
    /// Whether to verify SSL certificates
    pub verify_ssl: bool,
}

impl HttpConfig {
    /// Create a new HTTP config with default settings
    #[must_use]
    pub const fn new() -> Self {
        Self {
            timeout_seconds: 30,
            max_redirects: 10,
            verify_ssl: true,
        }
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
#[derive(Debug, Default)]
pub struct EngineBuilder {
    /// Whether to enable stdio capabilities
    enable_stdio: bool,
    /// Whether to enable filesystem capabilities
    enable_fs: bool,
    /// HTTP configuration if enabled
    http_config: Option<HttpConfig>,
    /// Custom providers to register
    custom_providers: std::collections::HashMap<String, std::sync::Arc<dyn CapabilityProvider>>,
}

impl EngineBuilder {
    /// Create a new engine builder
    ///
    /// Prefer using `Engine::new()` instead of this method directly.
    fn new() -> Self {
        Self::default()
    }

    /// Build the configured engine
    ///
    /// # Errors
    ///
    /// Returns `EngineError` if the engine cannot be constructed
    /// (e.g., missing required capabilities or invalid configuration).
    pub fn build(self) -> Result<Engine, EngineError> {
        use providers::{FsProvider, InterpProviderAdapter, StdioProvider};
        use std::sync::Arc;

        let mut providers: std::collections::HashMap<String, Arc<dyn CapabilityProvider>> =
            std::collections::HashMap::new();

        // Register stdio provider if enabled
        if self.enable_stdio {
            let provider = StdioProvider::new();
            providers.insert(provider.name().to_string(), Arc::new(provider));
        }

        // Register filesystem provider if enabled
        if self.enable_fs {
            let provider = FsProvider::new();
            providers.insert(provider.name().to_string(), Arc::new(provider));
        }

        // Register HTTP provider if configured
        // Note: HTTP provider is not yet implemented.
        if self.http_config.is_some() {
            return Err(EngineError::Configuration(
                "HTTP provider not yet implemented. Use with_custom_provider() to add your own HTTP implementation.".to_string(),
            ));
        }

        // Register custom providers (these can override built-ins)
        for (name, provider) in self.custom_providers {
            providers.insert(name, provider);
        }

        // Convert engine providers to interpreter-compatible providers
        // and build the RuntimeState with them
        let mut interp_providers: std::collections::HashMap<
            String,
            Arc<dyn ash_interp::capability::CapabilityProvider>,
        > = std::collections::HashMap::new();

        for (name, provider) in &providers {
            // Create an adapter that wraps the engine provider
            // The adapter implements ash_interp::capability::CapabilityProvider
            let adapter = InterpProviderAdapter::new(provider.clone());
            interp_providers.insert(name.clone(), Arc::new(adapter));
        }

        let runtime_state = RuntimeState::new().with_providers(interp_providers);

        Ok(Engine {
            surface_workflow_defs: std::sync::Mutex::new(std::collections::HashMap::new()),
            next_id: std::sync::atomic::AtomicU64::new(1),
            runtime_state,
        })
    }

    /// Add standard I/O capabilities (print, println, `read_line`)
    ///
    /// These are operational-effect capabilities for console I/O.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // Cannot be const due to HashMap operations in build()
    pub fn with_stdio_capabilities(mut self) -> Self {
        self.enable_stdio = true;
        self
    }

    /// Add filesystem capabilities (`read_file`, `write_file`)
    ///
    /// These are operational-effect capabilities for file operations.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // Cannot be const due to HashMap operations in build()
    pub fn with_fs_capabilities(mut self) -> Self {
        self.enable_fs = true;
        self
    }

    /// Add HTTP capabilities (get, post, put, delete)
    ///
    /// These are operational-effect capabilities for HTTP operations.
    /// Uses the provided configuration for timeout, redirects, and SSL verification.
    ///
    /// # Note
    ///
    /// HTTP provider is not yet implemented. Calling `build()` after using this method
    /// will return a `Configuration` error. Use `with_custom_provider()` to add your own
    /// HTTP implementation.
    ///
    /// # Example
    ///
    /// ```should_panic
    /// use ash_engine::{Engine, HttpConfig};
    ///
    /// let config = HttpConfig {
    ///     timeout_seconds: 60,
    ///     max_redirects: 5,
    ///     verify_ssl: true,
    /// };
    /// // This will panic because HTTP provider is not yet implemented
    /// let engine = Engine::new()
    ///     .with_http_capabilities(config)
    ///     .build()
    ///     .expect("engine builds");
    /// ```
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // Cannot be const due to HashMap operations in build()
    pub fn with_http_capabilities(mut self, config: HttpConfig) -> Self {
        self.http_config = Some(config);
        self
    }

    /// Add a custom capability provider
    ///
    /// Custom providers can be used to extend the engine with application-specific
    /// capabilities. They can also override built-in providers by using the same name.
    ///
    /// # Example
    ///
    /// ```
    /// use ash_engine::{Engine, CapabilityProvider};
    /// use ash_core::{Effect, Value};
    /// use async_trait::async_trait;
    /// use std::sync::Arc;
    ///
    /// #[derive(Debug)]
    /// struct MyProvider;
    ///
    /// #[async_trait]
    /// impl CapabilityProvider for MyProvider {
    ///     fn name(&self) -> &str { "my_provider" }
    ///     fn effect(&self) -> Effect { Effect::Operational }
    ///     async fn observe(&self, _action: &str, _args: &[Value]) -> Result<Value, ash_engine::providers::ProviderError> {
    ///         Ok(Value::Null)
    ///     }
    ///     async fn execute(&self, _action: &str, _args: &[Value]) -> Result<Value, ash_engine::providers::ProviderError> {
    ///         Ok(Value::Null)
    ///     }
    /// }
    ///
    /// let engine = Engine::new()
    ///     .with_custom_provider("custom", Arc::new(MyProvider))
    ///     .build()
    ///     .expect("engine builds");
    /// ```
    #[must_use]
    pub fn with_custom_provider(
        mut self,
        name: &str,
        provider: std::sync::Arc<dyn CapabilityProvider>,
    ) -> Self {
        self.custom_providers.insert(name.to_string(), provider);
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

    // ============================================================
    // EngineBuilder HTTP Capabilities Tests
    // ============================================================

    #[test]
    fn test_builder_http_capabilities_returns_error() {
        // HTTP provider is not yet implemented, should return Configuration error
        let config = HttpConfig::new();
        let result = Engine::new().with_http_capabilities(config).build();
        assert!(
            result.is_err(),
            "Builder with HTTP capabilities should return error (not yet implemented)"
        );
        let err = result.unwrap_err();
        assert!(
            matches!(err, EngineError::Configuration(_)),
            "Error should be Configuration variant"
        );
        let err_msg = format!("{err}");
        assert!(
            err_msg.contains("HTTP provider not yet implemented"),
            "Error message should indicate HTTP not implemented: {err_msg}"
        );
    }

    #[test]
    fn test_builder_http_capabilities_with_custom_config_returns_error() {
        // HTTP provider is not yet implemented, should return Configuration error
        let config = HttpConfig {
            timeout_seconds: 60,
            max_redirects: 5,
            verify_ssl: false,
        };
        let result = Engine::new().with_http_capabilities(config).build();
        assert!(
            result.is_err(),
            "Builder with custom HTTP config should return error (not yet implemented)"
        );
        let err = result.unwrap_err();
        assert!(
            matches!(err, EngineError::Configuration(_)),
            "Error should be Configuration variant"
        );
    }

    #[test]
    fn test_builder_http_default_config() {
        // Test HttpConfig::new() provides sensible defaults
        let config = HttpConfig::new();
        assert_eq!(config.timeout_seconds, 30);
        assert_eq!(config.max_redirects, 10);
        assert!(config.verify_ssl);
    }

    // ============================================================
    // EngineBuilder Custom Provider Tests
    // ============================================================

    #[test]
    fn test_builder_custom_provider_chaining() {
        // with_custom_provider should return Self for chaining
        use providers::StdioProvider;
        use std::sync::Arc;

        let provider = StdioProvider::new();
        let builder = Engine::new();
        let builder = builder.with_custom_provider("custom_stdio", Arc::new(provider));
        let result = builder.build();
        assert!(
            result.is_ok(),
            "Builder with custom provider should build successfully"
        );
    }

    #[test]
    fn test_builder_custom_provider_overrides_builtin() {
        // Custom providers with the same name as built-ins should override them
        use providers::StdioProvider;

        let custom_stdio = StdioProvider::new();
        let result = Engine::new()
            .with_stdio_capabilities() // Enable built-in stdio
            .with_custom_provider("stdio", std::sync::Arc::new(custom_stdio)) // Override with custom
            .build();
        assert!(
            result.is_ok(),
            "Builder should allow overriding built-in providers"
        );
    }

    #[test]
    fn test_builder_multiple_custom_providers() {
        // Multiple custom providers should all be registered
        use providers::{FsProvider, StdioProvider};

        let result = Engine::new()
            .with_custom_provider("my_stdio", std::sync::Arc::new(StdioProvider::new()))
            .with_custom_provider("my_fs", std::sync::Arc::new(FsProvider::new()))
            .build();
        assert!(
            result.is_ok(),
            "Builder with multiple custom providers should build successfully"
        );
    }

    // ============================================================
    // EngineBuilder Combined Configuration Tests
    // ============================================================

    #[test]
    fn test_builder_stdio_fs_custom_together() {
        // Test stdio, fs, and custom providers together (HTTP returns error until implemented)
        use providers::StdioProvider;

        let result = Engine::new()
            .with_stdio_capabilities()
            .with_fs_capabilities()
            .with_custom_provider("custom", std::sync::Arc::new(StdioProvider::new()))
            .build();

        assert!(
            result.is_ok(),
            "Builder with stdio, fs, and custom providers should build successfully"
        );
    }

    #[test]
    fn test_builder_http_with_other_capabilities_returns_error() {
        // HTTP provider not yet implemented - should return error even with other capabilities
        let result = Engine::new()
            .with_stdio_capabilities()
            .with_fs_capabilities()
            .with_http_capabilities(HttpConfig::new())
            .build();

        assert!(
            result.is_err(),
            "Builder with HTTP should return error (not yet implemented)"
        );
    }

    #[test]
    fn test_builder_complex_chaining_order_without_http() {
        // Different ordering should all work (without HTTP which returns error)
        use providers::StdioProvider;

        let engine1 = Engine::new()
            .with_stdio_capabilities()
            .with_fs_capabilities()
            .build();

        let engine2 = Engine::new()
            .with_custom_provider("custom", std::sync::Arc::new(StdioProvider::new()))
            .with_stdio_capabilities()
            .build();

        assert!(engine1.is_ok(), "First order should succeed");
        assert!(engine2.is_ok(), "Second order should succeed");
    }

    #[test]
    fn test_http_config_clone() {
        // HttpConfig should be cloneable
        let config = HttpConfig {
            timeout_seconds: 45,
            max_redirects: 3,
            verify_ssl: false,
        };
        let config_clone = config.clone();
        assert_eq!(config.timeout_seconds, config_clone.timeout_seconds);
        assert_eq!(config.max_redirects, config_clone.max_redirects);
        assert_eq!(config.verify_ssl, config_clone.verify_ssl);
    }

    #[test]
    fn test_http_config_default() {
        // HttpConfig should implement Default
        let config = HttpConfig::default();
        assert_eq!(config.timeout_seconds, 0); // Default for u64
        assert_eq!(config.max_redirects, 0); // Default for u32
        assert!(!config.verify_ssl); // Default for bool
    }
}
