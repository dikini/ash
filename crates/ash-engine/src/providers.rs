//! Standard capability providers for the Ash engine
//!
//! This module provides built-in capability providers for common I/O operations:
//! - `StdioProvider`: Standard input/output operations (print, println, read_line)
//! - `FsProvider`: Filesystem operations (read_file, write_file, exists)

use ash_core::{Effect, Value};
use async_trait::async_trait;

/// Standard I/O capability provider
///
/// Provides console input/output capabilities:
/// - `print`: Print text without newline
/// - `println`: Print text with newline
/// - `read_line`: Read a line from stdin
#[derive(Debug, Clone)]
pub struct StdioProvider;

/// Filesystem capability provider
///
/// Provides file system operations:
/// - `read_file`: Read file contents
/// - `write_file`: Write contents to file
/// - `exists`: Check if file exists
#[derive(Debug, Clone)]
pub struct FsProvider;

/// Error type for provider operations
#[derive(Debug, Clone)]
pub struct ProviderError {
    message: String,
}

impl std::fmt::Display for ProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ProviderError {}

/// Trait for capability providers
#[async_trait]
pub trait CapabilityProvider: Send + Sync {
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
    /// Create a new stdio provider
    pub const fn new() -> Self {
        Self
    }
}

impl Default for StdioProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CapabilityProvider for StdioProvider {
    fn name(&self) -> &str {
        "stdio"
    }

    fn effect(&self) -> Effect {
        Effect::Operational
    }

    async fn observe(&self, action: &str, args: &[Value]) -> Result<Value, ProviderError> {
        match action {
            "read_line" => {
                // Implementation to be added
                let _ = args;
                Ok(Value::Null)
            }
            _ => Err(ProviderError {
                message: format!("Unknown observe action: {}", action),
            }),
        }
    }

    async fn execute(&self, action: &str, args: &[Value]) -> Result<Value, ProviderError> {
        match action {
            "print" | "println" => {
                // Implementation to be added
                let _ = (action, args);
                Ok(Value::Null)
            }
            _ => Err(ProviderError {
                message: format!("Unknown execute action: {}", action),
            }),
        }
    }
}

impl FsProvider {
    /// Create a new filesystem provider
    pub const fn new() -> Self {
        Self
    }
}

impl Default for FsProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CapabilityProvider for FsProvider {
    fn name(&self) -> &str {
        "fs"
    }

    fn effect(&self) -> Effect {
        Effect::Operational
    }

    async fn observe(&self, action: &str, args: &[Value]) -> Result<Value, ProviderError> {
        match action {
            "exists" | "read_file" => {
                // Implementation to be added
                let _ = (action, args);
                Ok(Value::Null)
            }
            _ => Err(ProviderError {
                message: format!("Unknown observe action: {}", action),
            }),
        }
    }

    async fn execute(&self, action: &str, args: &[Value]) -> Result<Value, ProviderError> {
        match action {
            "write_file" => {
                // Implementation to be added
                let _ = (action, args);
                Ok(Value::Null)
            }
            _ => Err(ProviderError {
                message: format!("Unknown execute action: {}", action),
            }),
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
    fn test_stdio_provider_default() {
        let provider: StdioProvider = Default::default();
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
        let provider = StdioProvider::new();
        let result = provider
            .execute("print", &[Value::String("hello".into())])
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_stdio_println() {
        let provider = StdioProvider::new();
        let result = provider
            .execute("println", &[Value::String("hello".into())])
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_stdio_print_with_empty_string() {
        let provider = StdioProvider::new();
        let result = provider.execute("print", &[Value::String("".into())]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_stdio_print_with_multiple_args() {
        let provider = StdioProvider::new();
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
    }

    #[tokio::test]
    async fn test_stdio_read_line() {
        let provider = StdioProvider::new();
        let result = provider.observe("read_line", &[]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_stdio_unknown_execute_action() {
        let provider = StdioProvider::new();
        let result = provider.execute("unknown_action", &[]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_stdio_unknown_observe_action() {
        let provider = StdioProvider::new();
        let result = provider.observe("unknown_action", &[]).await;
        assert!(result.is_err());
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
    fn test_fs_provider_default() {
        let provider: FsProvider = Default::default();
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
    async fn test_fs_read_file() {
        let provider = FsProvider::new();
        let result = provider
            .observe("read_file", &[Value::String("/tmp/test.txt".into())])
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_fs_write_file() {
        let provider = FsProvider::new();
        let result = provider
            .execute(
                "write_file",
                &[
                    Value::String("/tmp/test.txt".into()),
                    Value::String("content".into()),
                ],
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_fs_exists() {
        let provider = FsProvider::new();
        let result = provider
            .observe("exists", &[Value::String("/tmp".into())])
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_fs_unknown_execute_action() {
        let provider = FsProvider::new();
        let result = provider.execute("delete", &[]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fs_unknown_observe_action() {
        let provider = FsProvider::new();
        let result = provider.observe("list_dir", &[]).await;
        assert!(result.is_err());
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
        let err = ProviderError {
            message: "test error".to_string(),
        };
        assert_eq!(format!("{}", err), "test error");
    }

    #[test]
    fn test_provider_error_debug() {
        let err = ProviderError {
            message: "test error".to_string(),
        };
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("test error"));
    }
}
