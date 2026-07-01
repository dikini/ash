//! Process capability provider for the Ash engine
//!
//! Provides process execution as a proper capability:
//! - `run`: Execute a command with arguments (Operational)
//! - `which`: Check if a command exists (Epistemic/observe)
//!
//! Unlike the previous builtin fn implementation, this provider enforces:
//! - Timeout on command execution
//! - Optional command allowlist
//! - Both stdout and stderr capture
//! - Exit code reporting

use ash_core::capability::{CapabilityError, CapabilityProvider};
use ash_core::{Constraint, Effect, Value};
use async_trait::async_trait;
use std::collections::HashMap;
use std::time::Duration;

/// Configuration for the process provider
#[derive(Debug, Clone)]
pub struct ProcessConfig {
    /// Timeout in seconds for command execution (default: 60)
    pub timeout_secs: u64,
    /// If set, only these commands are permitted
    pub allowed_commands: Option<Vec<String>>,
    /// Working directory for commands (default: current directory)
    pub working_dir: Option<String>,
}

impl Default for ProcessConfig {
    fn default() -> Self {
        Self {
            timeout_secs: 60,
            allowed_commands: None,
            working_dir: None,
        }
    }
}

impl ProcessConfig {
    /// Create a new config with default values
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set command timeout in seconds
    #[must_use]
    pub const fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    /// Restrict execution to only these commands
    #[must_use]
    pub fn with_allowed_commands(mut self, commands: Vec<String>) -> Self {
        self.allowed_commands = Some(commands);
        self
    }

    /// Set working directory for commands
    #[must_use]
    pub fn with_working_dir(mut self, dir: &str) -> Self {
        self.working_dir = Some(dir.to_string());
        self
    }
}

/// Process capability provider
///
/// Executes external commands as a proper capability with timeout,
/// allowlisting, and full output capture.
#[derive(Debug)]
pub struct ProcessProvider {
    config: ProcessConfig,
}

impl ProcessProvider {
    /// Create a new process provider with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(ProcessConfig::default())
    }

    /// Create a new process provider with custom configuration
    #[must_use]
    pub const fn with_config(config: ProcessConfig) -> Self {
        Self { config }
    }

    /// Validate that the command is in the allowed list
    fn validate_command(&self, cmd: &str) -> Result<(), CapabilityError> {
        if let Some(ref allowed) = self.config.allowed_commands
            && !allowed.iter().any(|a| a == cmd)
        {
            return Err(CapabilityError::PermissionDenied(format!(
                "Command '{cmd}' not in allowed list"
            )));
        }
        Ok(())
    }

    /// Extract command string from first argument
    fn extract_cmd(args: &[Value]) -> Result<String, CapabilityError> {
        match args.first() {
            Some(Value::String(s)) => Ok(s.clone()),
            Some(_) => Err(CapabilityError::InvalidArgument(
                "Command must be a string".to_string(),
            )),
            None => Err(CapabilityError::InvalidArgument(
                "Missing command argument".to_string(),
            )),
        }
    }

    /// Extract arguments list from second argument
    fn extract_args(args: &[Value]) -> Result<Vec<String>, CapabilityError> {
        match args.get(1) {
            Some(value) if value.is_list() => {
                let mut result = Vec::new();
                let items = value
                    .list_to_vec()
                    .expect("is_list only returns true for convertible lists");
                for item in &items {
                    match item {
                        Value::String(s) => result.push(s.clone()),
                        _ => {
                            return Err(CapabilityError::InvalidArgument(
                                "Command arguments must be strings".to_string(),
                            ));
                        }
                    }
                }
                Ok(result)
            }
            Some(_) => Err(CapabilityError::InvalidArgument(
                "Arguments must be a list of strings".to_string(),
            )),
            None => Ok(vec![]),
        }
    }

    /// Execute a command and capture output
    async fn handle_run(&self, args: &[Value]) -> Result<Value, CapabilityError> {
        let cmd = Self::extract_cmd(args)?;
        self.validate_command(&cmd)?;
        let cmd_args = Self::extract_args(args)?;

        let mut command = tokio::process::Command::new(&cmd);
        command.args(&cmd_args);

        if let Some(ref dir) = self.config.working_dir {
            command.current_dir(dir);
        }

        // Capture both stdout and stderr
        command.stdout(std::process::Stdio::piped());
        command.stderr(std::process::Stdio::piped());

        let output = tokio::time::timeout(
            Duration::from_secs(self.config.timeout_secs),
            command.output(),
        )
        .await
        .map_err(|_| {
            CapabilityError::ExecutionFailed(format!(
                "Command '{cmd}' timed out after {}s",
                self.config.timeout_secs
            ))
        })?
        .map_err(|e| CapabilityError::ExecutionFailed(format!("Command '{cmd}' failed: {e}")))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = i64::from(output.status.code().unwrap_or(-1));

        let mut result = HashMap::new();
        result.insert("stdout".to_string(), Value::String(stdout));
        result.insert("stderr".to_string(), Value::String(stderr));
        result.insert("exit_code".to_string(), Value::Int(exit_code));

        Ok(Value::Record(Box::new(result)))
    }

    /// Check if a command exists (observe)
    fn handle_which(args: &[Value]) -> Result<Value, CapabilityError> {
        let cmd = Self::extract_cmd(args)?;

        // Use `which` to check if command exists
        let output = std::process::Command::new("which").arg(&cmd).output();

        match output {
            Ok(o) if o.status.success() => {
                let path = String::from_utf8_lossy(&o.stdout).trim().to_string();
                Ok(Value::Variant {
                    name: "Some".to_string(),
                    fields: Box::new(vec![("value".to_string(), Value::String(path))]),
                })
            }
            _ => Ok(Value::Variant {
                name: "None".to_string(),
                fields: Box::new(vec![]),
            }),
        }
    }
}

impl Default for ProcessProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CapabilityProvider for ProcessProvider {
    fn name(&self) -> &'static str {
        "process"
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
        match action_name {
            "which" => {
                // Extract cmd from constraint arguments
                // Constraints hold unevaluated Expr, so we can't extract directly
                Err(CapabilityError::NotAvailable(
                    "Process observe requires execute path. Use execute(\"which\", args)."
                        .to_string(),
                ))
            }
            _ => Err(CapabilityError::NotAvailable(format!(
                "Unknown process observe action: {action_name}"
            ))),
        }
    }

    async fn execute(&self, action_name: &str, args: &[Value]) -> Result<Value, CapabilityError> {
        match action_name {
            "run" => self.handle_run(args).await,
            "which" => Self::handle_which(args),
            _ => Err(CapabilityError::NotAvailable(format!(
                "Unknown process action: {action_name}"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_provider_name() {
        let provider = ProcessProvider::new();
        assert_eq!(provider.name(), "process");
    }

    #[test]
    fn test_process_provider_effect() {
        let provider = ProcessProvider::new();
        assert_eq!(provider.effect(), Effect::Operational);
    }

    #[test]
    fn test_config_default() {
        let config = ProcessConfig::default();
        assert_eq!(config.timeout_secs, 60);
        assert!(config.allowed_commands.is_none());
        assert!(config.working_dir.is_none());
    }

    #[test]
    fn test_config_builder() {
        let config = ProcessConfig::new()
            .with_timeout(30)
            .with_allowed_commands(vec!["ls".to_string(), "echo".to_string()])
            .with_working_dir("/tmp");
        assert_eq!(config.timeout_secs, 30);
        assert_eq!(
            config.allowed_commands,
            Some(vec!["ls".to_string(), "echo".to_string()])
        );
        assert_eq!(config.working_dir, Some("/tmp".to_string()));
    }

    #[test]
    fn test_validate_command_allowed() {
        let config = ProcessConfig::new().with_allowed_commands(vec!["echo".to_string()]);
        let provider = ProcessProvider::with_config(config);
        provider.validate_command("echo").unwrap();
    }

    #[test]
    fn test_validate_command_blocked() {
        let config = ProcessConfig::new().with_allowed_commands(vec!["echo".to_string()]);
        let provider = ProcessProvider::with_config(config);
        let err = provider.validate_command("rm").unwrap_err();
        assert!(matches!(err, CapabilityError::PermissionDenied(_)));
    }

    #[test]
    fn test_validate_command_no_restriction() {
        let provider = ProcessProvider::new();
        provider.validate_command("anything").unwrap();
    }

    #[test]
    fn test_extract_cmd_valid() {
        let args = [Value::String("echo".to_string())];
        assert_eq!(ProcessProvider::extract_cmd(&args).unwrap(), "echo");
    }

    #[test]
    fn test_extract_cmd_missing() {
        let args: Vec<Value> = vec![];
        let err = ProcessProvider::extract_cmd(&args).unwrap_err();
        assert!(matches!(err, CapabilityError::InvalidArgument(_)));
    }

    #[test]
    fn test_extract_cmd_wrong_type() {
        let args = [Value::Int(42)];
        let err = ProcessProvider::extract_cmd(&args).unwrap_err();
        assert!(matches!(err, CapabilityError::InvalidArgument(_)));
    }

    #[test]
    fn test_extract_args_valid() {
        let args = [
            Value::String("echo".to_string()),
            Value::list_from_vec(vec![
                Value::String("hello".to_string()),
                Value::String("world".to_string()),
            ]),
        ];
        let result = ProcessProvider::extract_args(&args).unwrap();
        assert_eq!(result, vec!["hello", "world"]);
    }

    #[test]
    fn test_extract_args_missing() {
        let args = [Value::String("echo".to_string())];
        let result = ProcessProvider::extract_args(&args).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_extract_args_wrong_type() {
        let args = [Value::String("echo".to_string()), Value::Int(42)];
        let err = ProcessProvider::extract_args(&args).unwrap_err();
        assert!(matches!(err, CapabilityError::InvalidArgument(_)));
    }

    #[tokio::test]
    async fn test_run_echo() {
        let provider = ProcessProvider::new();
        let result = provider
            .execute(
                "run",
                &[
                    Value::String("echo".to_string()),
                    Value::list_from_vec(vec![Value::String("hello".to_string())]),
                ],
            )
            .await
            .unwrap();

        match result {
            Value::Record(fields) => {
                assert_eq!(fields.get("exit_code"), Some(&Value::Int(0)));
                match fields.get("stdout") {
                    Some(Value::String(s)) => assert!(s.contains("hello")),
                    other => panic!("Expected String stdout, got {other:?}"),
                }
            }
            other => panic!("Expected Record, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_run_blocked_command() {
        let config = ProcessConfig::new().with_allowed_commands(vec!["echo".to_string()]);
        let provider = ProcessProvider::with_config(config);
        let err = provider
            .execute("run", &[Value::String("rm".to_string())])
            .await
            .unwrap_err();
        assert!(matches!(err, CapabilityError::PermissionDenied(_)));
    }

    #[tokio::test]
    async fn test_run_missing_command() {
        let provider = ProcessProvider::new();
        let err = provider.execute("run", &[]).await.unwrap_err();
        assert!(matches!(err, CapabilityError::InvalidArgument(_)));
    }

    #[tokio::test]
    async fn test_run_nonexistent_command() {
        let provider = ProcessProvider::new();
        let err = provider
            .execute(
                "run",
                &[Value::String("nonexistent_command_xyz".to_string())],
            )
            .await
            .unwrap_err();
        assert!(matches!(err, CapabilityError::ExecutionFailed(_)));
    }

    #[tokio::test]
    async fn test_run_captures_stderr() {
        let provider = ProcessProvider::new();
        let result = provider
            .execute(
                "run",
                &[
                    Value::String("ls".to_string()),
                    Value::list_from_vec(vec![Value::String("/nonexistent_path_xyz".to_string())]),
                ],
            )
            .await
            .unwrap();

        match result {
            Value::Record(fields) => {
                // ls on nonexistent path should produce stderr and non-zero exit
                match fields.get("exit_code") {
                    Some(Value::Int(code)) => assert_ne!(*code, 0),
                    other => panic!("Expected Int exit_code, got {other:?}"),
                }
                match fields.get("stderr") {
                    Some(Value::String(s)) => assert!(!s.is_empty()),
                    other => panic!("Expected String stderr, got {other:?}"),
                }
            }
            other => panic!("Expected Record, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_execute_unknown_action() {
        let provider = ProcessProvider::new();
        let err = provider
            .execute("unknown", &[Value::String("echo".to_string())])
            .await
            .unwrap_err();
        assert!(matches!(err, CapabilityError::NotAvailable(_)));
    }

    #[tokio::test]
    async fn test_which_existing_command() {
        let provider = ProcessProvider::new();
        let result = provider
            .execute("which", &[Value::String("ls".to_string())])
            .await
            .unwrap();
        // On Linux, ls should be found
        match result {
            Value::Variant { name, fields } => {
                assert_eq!(name, "Some");
                assert!(matches!(
                    fields.as_slice(),
                    [(field, Value::String(path))] if field == "value" && path.contains("ls")
                ));
            }
            other => panic!("Expected Some(path), got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_which_nonexistent_command() {
        let provider = ProcessProvider::new();
        let result = provider
            .execute(
                "which",
                &[Value::String("nonexistent_command_xyz".to_string())],
            )
            .await
            .unwrap();
        assert_eq!(
            result,
            Value::Variant {
                name: "None".to_string(),
                fields: Box::new(vec![]),
            }
        );
    }
}
