//! Standard capability providers for the Ash engine
//!
//! This module provides built-in capability providers for common I/O operations:
//! - `StdioProvider`: Standard input/output operations (print, println, `read_line`)
//! - `FsProvider`: Filesystem operations (`read_file`, `write_file`, `exists`)
//! - `McpProvider`: MCP (Model Context Protocol) for LLM communication

use ash_core::{Effect, Value};
use async_trait::async_trait;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub mod mcp;
pub use mcp::{McpCapabilities, McpConfig, McpProvider};

/// Standard I/O capability provider
///
/// Provides console input/output capabilities:
/// - `print`: Print text without newline
/// - `println`: Print text with newline
/// - `read_line`: Read a line from stdin
///
/// The provider can be configured with custom input/output streams for testing.
#[derive(Debug, Clone)]
pub struct StdioProvider {
    inner: Arc<Mutex<StdioInner>>,
}

#[derive(Debug)]
struct StdioInner {
    /// Custom input buffer for testing (if None, uses stdin)
    input: Option<Vec<String>>,
    /// Custom output buffer for testing (if None, uses stdout)
    output: Option<Vec<String>>,
    /// Current position in input buffer
    input_pos: usize,
}

/// Filesystem capability provider
///
/// Provides file system operations:
/// - `read_file`: Read file contents
/// - `write_file`: Write contents to file
/// - `exists`: Check if file exists
///
/// Supports capability constraints:
/// - `allowed_paths`: List of allowed path prefixes
/// - `read_only`: If true, write operations are blocked
#[derive(Debug, Clone)]
pub struct FsProvider {
    config: FsConfig,
}

/// Configuration for filesystem provider
#[derive(Debug, Clone, Default)]
pub struct FsConfig {
    /// Allowed path prefixes (empty means all paths allowed)
    pub allowed_paths: Vec<PathBuf>,
    /// If true, write operations are blocked
    pub read_only: bool,
    /// Base directory for relative paths (if None, uses current directory)
    pub base_dir: Option<PathBuf>,
}

/// Error type for provider operations
#[derive(Debug, Clone)]
pub struct ProviderError {
    message: String,
}

impl ProviderError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ProviderError {}

/// Trait for capability providers
#[async_trait]
pub trait CapabilityProvider: Send + Sync + std::fmt::Debug {
    /// Get the provider name
    fn name(&self) -> &str;

    /// Get the effect level of this provider
    fn effect(&self) -> Effect;

    /// Observe/read from this capability
    async fn observe(&self, action: &str, args: &[Value]) -> Result<Value, ProviderError>;

    /// Execute an action on this capability
    async fn execute(&self, action: &str, args: &[Value]) -> Result<Value, ProviderError>;
}

impl StdioProvider {
    /// Create a new stdio provider using actual stdin/stdout
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(StdioInner {
                input: None,
                output: None,
                input_pos: 0,
            })),
        }
    }

    /// Create a new stdio provider with buffered I/O for testing
    ///
    /// # Arguments
    /// * `input` - Lines to return from `read_line` operations
    /// * `output` - Buffer to capture print/println output
    #[must_use]
    pub fn with_buffers(input: Vec<String>, output: Vec<String>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(StdioInner {
                input: Some(input),
                output: Some(output),
                input_pos: 0,
            })),
        }
    }

    /// Get the captured output buffer (for testing)
    ///
    /// Returns None if the provider was not created with `with_buffers`.
    #[must_use]
    pub fn get_output(&self) -> Option<Vec<String>> {
        let inner = self.inner.lock().ok()?;
        inner.output.clone()
    }

    /// Get a single concatenated output string (for testing convenience)
    #[must_use]
    pub fn get_output_string(&self) -> Option<String> {
        self.get_output().map(|lines| lines.join(""))
    }

    fn format_args(args: &[Value]) -> String {
        args.iter()
            .map(|v| match v {
                Value::String(s) => s.clone(),
                Value::Int(i) => i.to_string(),
                Value::Bool(b) => b.to_string(),
                Value::Null => "null".to_string(),
                other => format!("{other:?}"),
            })
            .collect::<String>()
    }
}

impl Default for StdioProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CapabilityProvider for StdioProvider {
    fn name(&self) -> &'static str {
        "stdio"
    }

    fn effect(&self) -> Effect {
        Effect::Operational
    }

    #[allow(clippy::significant_drop_tightening)]
    async fn observe(&self, action: &str, _args: &[Value]) -> Result<Value, ProviderError> {
        match action {
            "read_line" => {
                // Check if we have buffered input first
                let buffered_line = {
                    let mut inner = self
                        .inner
                        .lock()
                        .map_err(|_| ProviderError::new("Lock poisoned"))?;

                    if let Some(ref input) = inner.input {
                        // Use buffered input for testing
                        if inner.input_pos < input.len() {
                            let line = input[inner.input_pos].clone();
                            inner.input_pos += 1;
                            Some(line)
                        } else {
                            // Return empty string when buffer exhausted
                            Some(String::new())
                        }
                    } else {
                        None // Need to use actual stdin
                    }
                };

                if let Some(line) = buffered_line {
                    return Ok(Value::String(line));
                }

                // Use actual stdin (lock is already dropped)
                let stdin = io::stdin();
                let mut handle = stdin.lock();
                let mut line = String::new();
                handle
                    .read_line(&mut line)
                    .map_err(|e| ProviderError::new(format!("Read error: {e}")))?;
                // Remove trailing newline
                if line.ends_with('\n') {
                    line.pop();
                    if line.ends_with('\r') {
                        line.pop();
                    }
                }
                Ok(Value::String(line))
            }
            _ => Err(ProviderError::new(format!(
                "Unknown observe action: {action}"
            ))),
        }
    }

    async fn execute(&self, action: &str, args: &[Value]) -> Result<Value, ProviderError> {
        match action {
            "print" => {
                let text = Self::format_args(args);
                let use_buffer = {
                    let mut inner = self
                        .inner
                        .lock()
                        .map_err(|_| ProviderError::new("Lock poisoned"))?;

                    inner.output.as_mut().is_some_and(|output| {
                        output.push(text.clone());
                        true
                    })
                };

                if !use_buffer {
                    print!("{text}");
                    io::stdout()
                        .flush()
                        .map_err(|e| ProviderError::new(format!("Write error: {e}")))?;
                }
                Ok(Value::Null)
            }
            "println" => {
                let text = Self::format_args(args);
                let use_buffer = {
                    let mut inner = self
                        .inner
                        .lock()
                        .map_err(|_| ProviderError::new("Lock poisoned"))?;

                    inner.output.as_mut().is_some_and(|output| {
                        output.push(text.clone());
                        output.push("\n".to_string());
                        true
                    })
                };

                if !use_buffer {
                    println!("{text}");
                }
                Ok(Value::Null)
            }
            _ => Err(ProviderError::new(format!(
                "Unknown execute action: {action}"
            ))),
        }
    }
}

impl FsProvider {
    /// Create a new filesystem provider with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: FsConfig::default(),
        }
    }

    /// Create a new filesystem provider with custom configuration
    #[must_use]
    pub const fn with_config(config: FsConfig) -> Self {
        Self { config }
    }

    /// Validate that a path is allowed based on configuration
    fn validate_path(&self, path: &Path) -> Result<(), ProviderError> {
        // Resolve the path (handle relative paths)
        let resolved = if path.is_absolute() {
            path.to_path_buf()
        } else if let Some(ref base) = self.config.base_dir {
            base.join(path)
        } else {
            std::env::current_dir()
                .map_err(|e| ProviderError::new(format!("Cannot get current dir: {e}")))?
                .join(path)
        };

        let canonical = resolved.canonicalize().unwrap_or(resolved);

        // Check allowed paths if configured
        if !self.config.allowed_paths.is_empty() {
            let allowed = self.config.allowed_paths.iter().any(|allowed_prefix| {
                let allowed_canonical = allowed_prefix
                    .canonicalize()
                    .unwrap_or_else(|_| allowed_prefix.clone());
                canonical.starts_with(&allowed_canonical)
            });
            if !allowed {
                return Err(ProviderError::new(format!(
                    "Path '{}' is not in allowed paths",
                    path.display()
                )));
            }
        }

        Ok(())
    }

    /// Extract a path string from a Value argument
    fn extract_path(arg: &Value) -> Result<PathBuf, ProviderError> {
        match arg {
            Value::String(s) => Ok(PathBuf::from(s)),
            _ => Err(ProviderError::new("Path must be a string")),
        }
    }

    /// Extract string content from a Value argument
    fn extract_content(arg: &Value) -> Result<String, ProviderError> {
        match arg {
            Value::String(s) => Ok(s.clone()),
            _ => Err(ProviderError::new("Content must be a string")),
        }
    }
}

impl Default for FsProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CapabilityProvider for FsProvider {
    fn name(&self) -> &'static str {
        "fs"
    }

    fn effect(&self) -> Effect {
        Effect::Operational
    }

    async fn observe(&self, action: &str, args: &[Value]) -> Result<Value, ProviderError> {
        match action {
            "exists" => {
                if args.is_empty() {
                    return Err(ProviderError::new("exists requires a path argument"));
                }
                let path = Self::extract_path(&args[0])?;
                self.validate_path(&path)?;
                Ok(Value::Bool(path.exists()))
            }
            "read_file" => {
                if args.is_empty() {
                    return Err(ProviderError::new("read_file requires a path argument"));
                }
                let path = Self::extract_path(&args[0])?;
                self.validate_path(&path)?;

                let contents = tokio::fs::read_to_string(&path).await.map_err(|e| {
                    ProviderError::new(format!("Cannot read file '{}': {e}", path.display()))
                })?;
                Ok(Value::String(contents))
            }
            _ => Err(ProviderError::new(format!(
                "Unknown observe action: {action}"
            ))),
        }
    }

    async fn execute(&self, action: &str, args: &[Value]) -> Result<Value, ProviderError> {
        match action {
            "write_file" => {
                if args.len() < 2 {
                    return Err(ProviderError::new(
                        "write_file requires path and content arguments",
                    ));
                }

                // Check read-only mode
                if self.config.read_only {
                    return Err(ProviderError::new("Filesystem is read-only"));
                }

                let path = Self::extract_path(&args[0])?;
                let content = Self::extract_content(&args[1])?;

                self.validate_path(&path)?;

                tokio::fs::write(&path, content).await.map_err(|e| {
                    ProviderError::new(format!("Cannot write file '{}': {e}", path.display()))
                })?;
                Ok(Value::Null)
            }
            _ => Err(ProviderError::new(format!(
                "Unknown execute action: {action}"
            ))),
        }
    }
}

/// Adapter that wraps an `ash_engine` `CapabilityProvider` to work with `ash_interp`
///
/// This allows providers defined using the `ash_engine::providers::CapabilityProvider`
/// trait to be used with the interpreter which expects `ash_interp::capability::CapabilityProvider`.
#[derive(Clone, Debug)]
pub struct InterpProviderAdapter {
    inner: Arc<dyn CapabilityProvider>,
}

impl InterpProviderAdapter {
    /// Create a new adapter wrapping the given provider
    pub fn new(provider: Arc<dyn CapabilityProvider>) -> Self {
        Self { inner: provider }
    }
}

#[async_trait]
impl ash_interp::capability::CapabilityProvider for InterpProviderAdapter {
    fn capability_name(&self) -> &str {
        self.inner.name()
    }

    fn effect(&self) -> Effect {
        self.inner.effect()
    }

    async fn observe(
        &self,
        _constraints: &[ash_core::Constraint],
    ) -> ash_interp::ExecResult<Value> {
        // For now, we delegate to the wrapped provider with empty args
        // The ash_interp trait passes constraints, but the ash_engine trait
        // uses action/args. We use "observe" as the action.
        self.inner
            .observe("observe", &[])
            .await
            .map_err(|e| ash_interp::ExecError::ExecutionFailed(e.to_string()))
    }

    async fn execute(&self, action: &ash_core::Action) -> ash_interp::ExecResult<Value> {
        // Convert Action arguments from Expr to Value by evaluating them
        // For now, we pass empty args - in a full implementation we'd need
        // an evaluation context to evaluate the expressions
        let args: Vec<Value> = action
            .arguments
            .iter()
            .map(|_expr| Value::Null) // Simplified - would need proper eval
            .collect();

        self.inner
            .execute(&action.name, &args)
            .await
            .map_err(|e| ash_interp::ExecError::ExecutionFailed(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // StdioProvider Tests
    // ============================================================

    #[test]
    fn test_stdio_provider_new() {
        let provider = StdioProvider::new();
        let _ = provider; // Just verify it constructs
    }

    #[test]
    #[allow(clippy::default_constructed_unit_structs)]
    fn test_stdio_provider_default() {
        let provider: StdioProvider = StdioProvider::default();
        let _ = provider;
    }

    #[test]
    fn test_stdio_provider_name() {
        let provider = StdioProvider::new();
        assert_eq!(provider.name(), "stdio");
    }

    #[test]
    fn test_stdio_provider_effect() {
        let provider = StdioProvider::new();
        assert_eq!(provider.effect(), Effect::Operational);
    }

    #[test]
    fn test_stdio_provider_effect_is_operational() {
        let provider = StdioProvider::new();
        assert_eq!(provider.effect(), Effect::Operational);
        assert!(provider.effect().at_least(Effect::Operational));
        assert!(provider.effect().at_least(Effect::Evaluative)); // Operational >= Evaluative
        assert!(provider.effect().at_least(Effect::Epistemic)); // Operational >= Epistemic
    }

    #[tokio::test]
    async fn test_stdio_print() {
        let provider = StdioProvider::with_buffers(vec![], vec![]);
        let result = provider
            .execute("print", &[Value::String("hello".into())])
            .await;
        assert!(result.is_ok());
        assert_eq!(provider.get_output_string(), Some("hello".to_string()));
    }

    #[tokio::test]
    async fn test_stdio_println() {
        let provider = StdioProvider::with_buffers(vec![], vec![]);
        let result = provider
            .execute("println", &[Value::String("hello".into())])
            .await;
        assert!(result.is_ok());
        assert_eq!(provider.get_output_string(), Some("hello\n".to_string()));
    }

    #[tokio::test]
    async fn test_stdio_print_with_empty_string() {
        let provider = StdioProvider::with_buffers(vec![], vec![]);
        let result = provider
            .execute("print", &[Value::String(String::new())])
            .await;
        assert!(result.is_ok());
        assert_eq!(provider.get_output_string(), Some(String::new()));
    }

    #[tokio::test]
    async fn test_stdio_print_with_multiple_args() {
        let provider = StdioProvider::with_buffers(vec![], vec![]);
        let result = provider
            .execute(
                "print",
                &[
                    Value::String("hello".into()),
                    Value::String(" ".into()),
                    Value::String("world".into()),
                ],
            )
            .await;
        assert!(result.is_ok());
        assert_eq!(
            provider.get_output_string(),
            Some("hello world".to_string())
        );
    }

    #[tokio::test]
    async fn test_stdio_print_int() {
        let provider = StdioProvider::with_buffers(vec![], vec![]);
        let result = provider.execute("print", &[Value::Int(42)]).await;
        assert!(result.is_ok());
        assert_eq!(provider.get_output_string(), Some("42".to_string()));
    }

    #[tokio::test]
    async fn test_stdio_print_bool() {
        let provider = StdioProvider::with_buffers(vec![], vec![]);
        let result = provider.execute("print", &[Value::Bool(true)]).await;
        assert!(result.is_ok());
        assert_eq!(provider.get_output_string(), Some("true".to_string()));
    }

    #[tokio::test]
    async fn test_stdio_print_null() {
        let provider = StdioProvider::with_buffers(vec![], vec![]);
        let result = provider.execute("print", &[Value::Null]).await;
        assert!(result.is_ok());
        assert_eq!(provider.get_output_string(), Some("null".to_string()));
    }

    #[tokio::test]
    async fn test_stdio_read_line() {
        let provider = StdioProvider::with_buffers(vec!["hello world".to_string()], vec![]);
        let result = provider.observe("read_line", &[]).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::String("hello world".to_string()));
    }

    #[tokio::test]
    async fn test_stdio_read_line_multiple() {
        let provider =
            StdioProvider::with_buffers(vec!["first".to_string(), "second".to_string()], vec![]);
        let first = provider.observe("read_line", &[]).await.unwrap();
        let second = provider.observe("read_line", &[]).await.unwrap();
        assert_eq!(first, Value::String("first".to_string()));
        assert_eq!(second, Value::String("second".to_string()));
    }

    #[tokio::test]
    async fn test_stdio_read_line_empty_when_exhausted() {
        let provider = StdioProvider::with_buffers(vec![], vec![]);
        let result = provider.observe("read_line", &[]).await;
        assert_eq!(result.unwrap(), Value::String(String::new()));
    }

    #[tokio::test]
    async fn test_stdio_unknown_execute_action() {
        let provider = StdioProvider::new();
        let result = provider.execute("unknown_action", &[]).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .message
                .contains("Unknown execute action")
        );
    }

    #[tokio::test]
    async fn test_stdio_unknown_observe_action() {
        let provider = StdioProvider::new();
        let result = provider.observe("unknown_action", &[]).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .message
                .contains("Unknown observe action")
        );
    }

    // ============================================================
    // FsProvider Tests
    // ============================================================

    #[test]
    fn test_fs_provider_new() {
        let provider = FsProvider::new();
        let _ = provider;
    }

    #[test]
    #[allow(clippy::default_constructed_unit_structs)]
    fn test_fs_provider_default() {
        let provider: FsProvider = FsProvider::default();
        let _ = provider;
    }

    #[test]
    fn test_fs_provider_name() {
        let provider = FsProvider::new();
        assert_eq!(provider.name(), "fs");
    }

    #[test]
    fn test_fs_provider_effect() {
        let provider = FsProvider::new();
        assert_eq!(provider.effect(), Effect::Operational);
    }

    #[test]
    fn test_fs_provider_effect_is_operational() {
        let provider = FsProvider::new();
        assert!(provider.effect().at_least(Effect::Operational));
    }

    #[tokio::test]
    async fn test_fs_exists_existing_file() {
        let provider = FsProvider::new();
        // Use the provider source file as test subject
        let result = provider
            .observe("exists", &[Value::String("src/providers/mod.rs".into())])
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(true));
    }

    #[tokio::test]
    async fn test_fs_exists_nonexistent_file() {
        let provider = FsProvider::new();
        let result = provider
            .observe(
                "exists",
                &[Value::String("nonexistent_file_xyz.txt".into())],
            )
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(false));
    }

    #[tokio::test]
    async fn test_fs_read_file() {
        let provider = FsProvider::new();
        let result = provider
            .observe("read_file", &[Value::String("src/providers/mod.rs".into())])
            .await;
        assert!(result.is_ok());
        let content = result.unwrap();
        if let Value::String(s) = content {
            assert!(s.contains("Standard I/O capability provider"));
        } else {
            panic!("Expected string content");
        }
    }

    #[tokio::test]
    async fn test_fs_read_file_not_found() {
        let provider = FsProvider::new();
        let result = provider
            .observe(
                "read_file",
                &[Value::String("nonexistent_file_xyz.txt".into())],
            )
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Cannot read file"));
    }

    #[tokio::test]
    async fn test_fs_read_file_missing_arg() {
        let provider = FsProvider::new();
        let result = provider.observe("read_file", &[]).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .message
                .contains("requires a path argument")
        );
    }

    #[tokio::test]
    async fn test_fs_write_file() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("ash_provider_test_write.txt");

        let config = FsConfig {
            allowed_paths: vec![temp_dir.clone()],
            read_only: false,
            base_dir: None,
        };
        let provider = FsProvider::with_config(config);

        let result = provider
            .execute(
                "write_file",
                &[
                    Value::String(test_file.to_string_lossy().into()),
                    Value::String("test content".into()),
                ],
            )
            .await;
        assert!(result.is_ok());

        // Verify the file was written
        let contents = tokio::fs::read_to_string(&test_file).await.unwrap();
        assert_eq!(contents, "test content");

        // Clean up
        let _ = tokio::fs::remove_file(&test_file).await;
    }

    #[tokio::test]
    async fn test_fs_write_file_missing_args() {
        let provider = FsProvider::new();
        let result = provider.execute("write_file", &[]).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .message
                .contains("requires path and content")
        );
    }

    #[tokio::test]
    async fn test_fs_write_file_read_only() {
        let config = FsConfig {
            allowed_paths: Vec::new(),
            read_only: true,
            base_dir: None,
        };
        let provider = FsProvider::with_config(config);
        let result = provider
            .execute(
                "write_file",
                &[
                    Value::String("/tmp/test.txt".into()),
                    Value::String("content".into()),
                ],
            )
            .await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().message, "Filesystem is read-only");
    }

    #[tokio::test]
    async fn test_fs_exists_missing_arg() {
        let provider = FsProvider::new();
        let result = provider.observe("exists", &[]).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .message
                .contains("requires a path argument")
        );
    }

    #[tokio::test]
    async fn test_fs_unknown_execute_action() {
        let provider = FsProvider::new();
        let result = provider.execute("delete", &[]).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .message
                .contains("Unknown execute action")
        );
    }

    #[tokio::test]
    async fn test_fs_unknown_observe_action() {
        let provider = FsProvider::new();
        let result = provider.observe("list_dir", &[]).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .message
                .contains("Unknown observe action")
        );
    }

    #[tokio::test]
    async fn test_fs_path_constraint_violation() {
        let config = FsConfig {
            allowed_paths: vec![PathBuf::from("/tmp/allowed")],
            read_only: false,
            base_dir: None,
        };
        let provider = FsProvider::with_config(config);
        let result = provider
            .observe("read_file", &[Value::String("/etc/passwd".into())])
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("not in allowed paths"));
    }

    #[tokio::test]
    async fn test_fs_path_non_string_path() {
        let provider = FsProvider::new();
        let result = provider.observe("read_file", &[Value::Int(42)]).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .message
                .contains("Path must be a string")
        );
    }

    #[tokio::test]
    async fn test_fs_content_non_string() {
        let temp_dir = std::env::temp_dir();
        let config = FsConfig {
            allowed_paths: vec![temp_dir],
            read_only: false,
            base_dir: None,
        };
        let provider = FsProvider::with_config(config);
        let result = provider
            .execute(
                "write_file",
                &[Value::String("/tmp/test.txt".into()), Value::Int(42)],
            )
            .await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .message
                .contains("Content must be a string")
        );
    }

    // ============================================================
    // CapabilityProvider Trait Tests
    // ============================================================

    #[test]
    fn test_provider_trait_object_stdio() {
        let provider: Box<dyn CapabilityProvider> = Box::new(StdioProvider::new());
        assert_eq!(provider.name(), "stdio");
        assert_eq!(provider.effect(), Effect::Operational);
    }

    #[test]
    fn test_provider_trait_object_fs() {
        let provider: Box<dyn CapabilityProvider> = Box::new(FsProvider::new());
        assert_eq!(provider.name(), "fs");
        assert_eq!(provider.effect(), Effect::Operational);
    }

    #[test]
    fn test_provider_send_sync_stdio() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<StdioProvider>();
        assert_sync::<StdioProvider>();
    }

    #[test]
    fn test_provider_send_sync_fs() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<FsProvider>();
        assert_sync::<FsProvider>();
    }

    // ============================================================
    // ProviderError Tests
    // ============================================================

    #[test]
    fn test_provider_error_display() {
        let err = ProviderError::new("test error");
        assert_eq!(format!("{err}"), "test error");
    }

    #[test]
    fn test_provider_error_debug() {
        let err = ProviderError::new("test error");
        let debug_str = format!("{err:?}");
        assert!(debug_str.contains("test error"));
    }

    #[test]
    fn test_fs_config_default() {
        let config = FsConfig::default();
        assert!(config.allowed_paths.is_empty());
        assert!(!config.read_only);
        assert!(config.base_dir.is_none());
    }
}
