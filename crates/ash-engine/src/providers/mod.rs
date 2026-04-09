//! Standard capability providers for the Ash engine
//!
//! This module provides built-in capability providers for common I/O operations:
//! - `StdioProvider`: Standard input/output operations (print, println, `read_line`)
//! - `FsProvider`: Filesystem operations (`read_file`, `write_file`, `exists`)
//! - `McpProvider`: MCP (Model Context Protocol) for LLM communication
//!
//! All providers implement the unified `ash_core::capability::CapabilityProvider` trait.

use ash_core::capability::{CapabilityError, CapabilityProvider};
use ash_core::{Action, Constraint, Effect, Value};
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

    /// Handle `read_line` observe operation
    #[allow(clippy::significant_drop_tightening)]
    fn handle_read_line(&self) -> Result<Value, CapabilityError> {
        let buffered_line = {
            let mut inner = self
                .inner
                .lock()
                .map_err(|_| CapabilityError::ExecutionFailed("Lock poisoned".to_string()))?;

            if let Some(ref input) = inner.input {
                if inner.input_pos < input.len() {
                    let line = input[inner.input_pos].clone();
                    inner.input_pos += 1;
                    Some(line)
                } else {
                    Some(String::new())
                }
            } else {
                None
            }
        };

        if let Some(line) = buffered_line {
            return Ok(Value::String(line));
        }

        // Use actual stdin
        let stdin = io::stdin();
        let mut handle = stdin.lock();
        let mut line = String::new();
        handle
            .read_line(&mut line)
            .map_err(|e| CapabilityError::ExecutionFailed(format!("Read error: {e}")))?;
        if line.ends_with('\n') {
            line.pop();
            if line.ends_with('\r') {
                line.pop();
            }
        }
        Ok(Value::String(line))
    }

    /// Handle print execute operation
    fn handle_print(&self, text: &str) -> Result<Value, CapabilityError> {
        let use_buffer = {
            let mut inner = self
                .inner
                .lock()
                .map_err(|_| CapabilityError::ExecutionFailed("Lock poisoned".to_string()))?;

            inner.output.as_mut().is_some_and(|output| {
                output.push(text.to_string());
                true
            })
        };

        if !use_buffer {
            print!("{text}");
            io::stdout()
                .flush()
                .map_err(|e| CapabilityError::ExecutionFailed(format!("Write error: {e}")))?;
        }
        Ok(Value::Null)
    }

    /// Handle println execute operation
    fn handle_println(&self, text: &str) -> Result<Value, CapabilityError> {
        let use_buffer = {
            let mut inner = self
                .inner
                .lock()
                .map_err(|_| CapabilityError::ExecutionFailed("Lock poisoned".to_string()))?;

            inner.output.as_mut().is_some_and(|output| {
                output.push(text.to_string());
                output.push("\n".to_string());
                true
            })
        };

        if !use_buffer {
            println!("{text}");
        }
        Ok(Value::Null)
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

    async fn observe(&self, constraints: &[Constraint]) -> Result<Value, CapabilityError> {
        // For stdio, we dispatch based on constraint predicate name
        if constraints.is_empty() {
            return Err(CapabilityError::InvalidArgument(
                "No observe constraints provided".to_string(),
            ));
        }
        // Use the first constraint's predicate name as the action
        let action_name = constraints[0].predicate.name.as_str();
        match action_name {
            "read_line" => self.handle_read_line(),
            _ => Err(CapabilityError::NotAvailable(format!(
                "Unknown observe action: {action_name}"
            ))),
        }
    }

    async fn execute(&self, action: &Action) -> Result<Value, CapabilityError> {
        let text = Self::format_args(&action.arguments);
        match action.name.as_str() {
            "print" => self.handle_print(&text),
            "println" => self.handle_println(&text),
            _ => Err(CapabilityError::NotAvailable(format!(
                "Unknown execute action: {}",
                action.name
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
    fn validate_path(&self, path: &Path) -> Result<(), CapabilityError> {
        let resolved = if path.is_absolute() {
            path.to_path_buf()
        } else if let Some(ref base) = self.config.base_dir {
            base.join(path)
        } else {
            std::env::current_dir()
                .map_err(|e| {
                    CapabilityError::ExecutionFailed(format!("Cannot get current dir: {e}"))
                })?
                .join(path)
        };

        let canonical = resolved.canonicalize().unwrap_or(resolved);

        if !self.config.allowed_paths.is_empty() {
            let allowed = self.config.allowed_paths.iter().any(|allowed_prefix| {
                let allowed_canonical = allowed_prefix
                    .canonicalize()
                    .unwrap_or_else(|_| allowed_prefix.clone());
                canonical.starts_with(&allowed_canonical)
            });
            if !allowed {
                return Err(CapabilityError::PermissionDenied(format!(
                    "Path '{}' is not in allowed paths",
                    path.display()
                )));
            }
        }

        Ok(())
    }

    /// Extract a path string from a Value argument
    fn extract_path(arg: &Value) -> Result<PathBuf, CapabilityError> {
        match arg {
            Value::String(s) => Ok(PathBuf::from(s)),
            _ => Err(CapabilityError::InvalidArgument(
                "Path must be a string".to_string(),
            )),
        }
    }

    /// Extract string content from a Value argument
    fn extract_content(arg: &Value) -> Result<String, CapabilityError> {
        match arg {
            Value::String(s) => Ok(s.clone()),
            _ => Err(CapabilityError::InvalidArgument(
                "Content must be a string".to_string(),
            )),
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

    async fn observe(&self, constraints: &[Constraint]) -> Result<Value, CapabilityError> {
        if constraints.is_empty() {
            return Err(CapabilityError::InvalidArgument(
                "No observe constraints provided".to_string(),
            ));
        }
        let action_name = constraints[0].predicate.name.as_str();
        // For observe, arguments come from predicate arguments
        let args: Vec<Value> = constraints[0]
            .predicate
            .arguments
            .iter()
            .map(|expr| match expr {
                ash_core::Expr::Literal(v) => v.clone(),
                _ => Value::Null, // Simplified - in practice would need evaluation
            })
            .collect();

        match action_name {
            "exists" => {
                if args.is_empty() {
                    return Err(CapabilityError::InvalidArgument(
                        "exists requires a path argument".to_string(),
                    ));
                }
                let path = Self::extract_path(&args[0])?;
                self.validate_path(&path)?;
                Ok(Value::Bool(path.exists()))
            }
            "read_file" => {
                if args.is_empty() {
                    return Err(CapabilityError::InvalidArgument(
                        "read_file requires a path argument".to_string(),
                    ));
                }
                let path = Self::extract_path(&args[0])?;
                self.validate_path(&path)?;

                let contents = tokio::fs::read_to_string(&path).await.map_err(|e| {
                    CapabilityError::ExecutionFailed(format!(
                        "Cannot read file '{}': {e}",
                        path.display()
                    ))
                })?;
                Ok(Value::String(contents))
            }
            _ => Err(CapabilityError::NotAvailable(format!(
                "Unknown observe action: {action_name}"
            ))),
        }
    }

    async fn execute(&self, action: &Action) -> Result<Value, CapabilityError> {
        match action.name.as_str() {
            "write_file" => {
                if action.arguments.len() < 2 {
                    return Err(CapabilityError::InvalidArgument(
                        "write_file requires path and content arguments".to_string(),
                    ));
                }

                if self.config.read_only {
                    return Err(CapabilityError::PermissionDenied(
                        "Filesystem is read-only".to_string(),
                    ));
                }

                let path = Self::extract_path(&action.arguments[0])?;
                let content = Self::extract_content(&action.arguments[1])?;

                self.validate_path(&path)?;

                tokio::fs::write(&path, content).await.map_err(|e| {
                    CapabilityError::ExecutionFailed(format!(
                        "Cannot write file '{}': {e}",
                        path.display()
                    ))
                })?;
                Ok(Value::Null)
            }
            _ => Err(CapabilityError::NotAvailable(format!(
                "Unknown execute action: {}",
                action.name
            ))),
        }
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
    fn test_stdio_provider_buffers() {
        let provider =
            StdioProvider::with_buffers(vec!["hello".to_string(), "world".to_string()], Vec::new());
        assert!(provider.get_output().is_some());
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
    fn test_fs_provider_with_config() {
        let config = FsConfig {
            allowed_paths: vec![PathBuf::from("/tmp")],
            read_only: true,
            base_dir: None,
        };
        let provider = FsProvider::with_config(config);
        assert_eq!(provider.name(), "fs");
    }

    #[test]
    fn test_fs_extract_path() {
        let value = Value::String("/tmp/test.txt".to_string());
        let path = FsProvider::extract_path(&value).unwrap();
        assert_eq!(path, PathBuf::from("/tmp/test.txt"));
    }

    #[test]
    fn test_fs_extract_path_invalid() {
        let value = Value::Int(42);
        assert!(FsProvider::extract_path(&value).is_err());
    }

    #[test]
    fn test_fs_extract_content() {
        let value = Value::String("hello world".to_string());
        let content = FsProvider::extract_content(&value).unwrap();
        assert_eq!(content, "hello world");
    }

    #[test]
    fn test_fs_extract_content_invalid() {
        let value = Value::Int(42);
        assert!(FsProvider::extract_content(&value).is_err());
    }
}
