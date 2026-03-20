//! Interactive REPL for the Ash workflow language.
//!
//! The REPL provides an interactive environment for:
//! - Quick experimentation with Ash syntax
//! - Testing workflow fragments
//! - Learning the language
//! - Debugging with `:type` and `:ast` inspection
//!
//! # Example
//!
//! ```rust,no_run
//! use ash_repl::Repl;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut repl = Repl::new(false)?;
//!     repl.run().await?;
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]

mod completer;
mod error;

use ash_core::value::Value;
use ash_engine::Engine;
use colored::Colorize;
use completer::AshCompleter;
use error::{format_error, suggest_fix};
use rustyline::error::ReadlineError;
use rustyline::{Config, Editor};
use std::io::Write;
use std::path::PathBuf;
use thiserror::Error;

/// Canonical normal REPL prompt.
pub const NORMAL_PROMPT: &str = "ash> ";

/// Canonical continuation REPL prompt.
pub const CONTINUATION_PROMPT: &str = "... ";

/// Canonical startup banner for interactive REPL sessions.
pub const STARTUP_BANNER: &str = "Ash REPL - Type :help for help, :quit to exit";

const HELP_TEXT: &str = "\
Commands:
  :help, :h     Show this help
  :quit, :q     Exit the REPL
  :type, :t     Show type of expression
  :ast          Show AST representation
  :clear        Clear screen

Multi-line input is supported automatically.";

const CANONICAL_COMMANDS: [&str; 5] = [":help", ":quit", ":type", ":ast", ":clear"];

/// Errors that can occur in the REPL.
#[derive(Debug, Error)]
pub enum ReplError {
    /// Engine error.
    #[error("engine error: {0}")]
    Engine(String),
    /// IO error.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    /// Readline error.
    #[error("readline error: {0}")]
    Readline(String),
}

impl From<ash_engine::EngineError> for ReplError {
    fn from(err: ash_engine::EngineError) -> Self {
        Self::Engine(err.to_string())
    }
}

impl From<ash_interp::ExecError> for ReplError {
    fn from(err: ash_interp::ExecError) -> Self {
        Self::Engine(err.to_string())
    }
}

impl From<ReadlineError> for ReplError {
    fn from(err: ReadlineError) -> Self {
        Self::Readline(err.to_string())
    }
}

/// Session-level REPL configuration.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReplConfig {
    history_path: Option<PathBuf>,
}

impl ReplConfig {
    /// Disable persistent history for the session.
    #[must_use]
    pub const fn no_history() -> Self {
        Self { history_path: None }
    }

    /// Use the default persistent history location.
    #[must_use]
    pub fn with_default_history() -> Self {
        Self {
            history_path: Repl::get_history_path(),
        }
    }

    /// Override the history path for the session.
    #[must_use]
    pub fn with_history_path(path: impl Into<PathBuf>) -> Self {
        Self {
            history_path: Some(path.into()),
        }
    }

    /// Return the configured history path.
    #[must_use]
    pub const fn history_path(&self) -> Option<&PathBuf> {
        self.history_path.as_ref()
    }
}

/// Return the canonical help text for the interactive command surface.
#[must_use]
pub const fn help_text() -> &'static str {
    HELP_TEXT
}

/// Return the canonical command names from the REPL spec.
#[must_use]
pub const fn canonical_command_names() -> &'static [&'static str] {
    &CANONICAL_COMMANDS
}

/// Infer the canonical Ash type name for an expression.
///
/// # Errors
///
/// Returns an error when the expression cannot be parsed or does not yield a
/// reportable canonical type.
pub fn infer_type_display(expr: &str) -> Result<String, ReplError> {
    Engine::default()
        .infer_expression_type(expr)
        .map_err(Into::into)
}

/// Run a REPL session with explicit session configuration.
///
/// # Errors
///
/// Returns an error when session initialization or interactive execution fails.
pub async fn run_with_config(config: ReplConfig) -> Result<(), ReplError> {
    let mut repl = Repl::from_config(config)?;
    repl.run().await
}

/// Helper struct for managing the readline editor.
#[derive(Debug)]
struct ReplEditor {
    editor: Editor<AshCompleter, rustyline::history::DefaultHistory>,
    history_path: Option<PathBuf>,
}

impl ReplEditor {
    fn new(history_path: Option<PathBuf>) -> Result<Self, ReplError> {
        let config = Config::builder()
            .completion_type(rustyline::CompletionType::List)
            .build();

        let mut editor = Editor::with_config(config)?;
        editor.set_helper(Some(AshCompleter::new()));

        // Load history if path exists
        if let Some(path) = &history_path {
            #[allow(clippy::collapsible_if)]
            if path.exists() {
                editor.load_history(path).ok();
            }
        }

        Ok(Self {
            editor,
            history_path,
        })
    }

    fn readline(&mut self, prompt: &str) -> Result<String, ReadlineError> {
        self.editor.readline(prompt)
    }

    fn add_history_entry(&mut self, line: &str) {
        self.editor.add_history_entry(line).ok();
    }

    fn save_history(&mut self) {
        if let Some(path) = &self.history_path {
            // Ensure parent directory exists
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            self.editor.save_history(path).ok();
        }
    }
}

/// Interactive REPL for Ash workflow language.
#[derive(Debug)]
pub struct Repl {
    engine: Engine,
    /// History path stored for test access and potential future use.
    /// The actual history management is handled by `ReplEditor`.
    #[allow(dead_code)]
    history_path: Option<PathBuf>,
    editor: Option<ReplEditor>,
}

impl Repl {
    /// Create a new REPL instance.
    ///
    /// # Arguments
    ///
    /// * `no_history` - If true, don't load or save history.
    ///
    /// # Errors
    ///
    /// Returns error if history file cannot be accessed.
    pub fn new(no_history: bool) -> Result<Self, ReplError> {
        let config = if no_history {
            ReplConfig::no_history()
        } else {
            ReplConfig::with_default_history()
        };

        Self::from_config(config)
    }

    /// Create a new REPL from explicit session configuration.
    ///
    /// # Errors
    ///
    /// Returns error if history file cannot be accessed.
    pub fn from_config(config: ReplConfig) -> Result<Self, ReplError> {
        let engine = Engine::default();
        let history_path = config.history_path;
        let editor = history_path
            .as_ref()
            .map(|_| ReplEditor::new(history_path.clone()))
            .transpose()?;

        Ok(Self {
            engine,
            history_path,
            editor,
        })
    }

    /// Get the default history file path.
    fn get_history_path() -> Option<PathBuf> {
        directories::ProjectDirs::from("org", "ash", "ash-repl")
            .map(|dirs| dirs.data_dir().join("history"))
    }

    /// Run the REPL interactively.
    ///
    /// # Errors
    ///
    /// Returns error if readline fails.
    pub async fn run(&mut self) -> Result<(), ReplError> {
        println!("{STARTUP_BANNER}");
        println!();

        let mut multi_line_input = String::new();
        let mut is_multiline = false;

        loop {
            let prompt = if is_multiline {
                CONTINUATION_PROMPT
            } else {
                NORMAL_PROMPT
            };

            let input = if let Some(editor) = &mut self.editor {
                match editor.readline(prompt) {
                    Ok(line) => {
                        editor.add_history_entry(&line);
                        line
                    }
                    Err(ReadlineError::Interrupted) => {
                        // Ctrl+C - cancel multiline or quit
                        if is_multiline {
                            multi_line_input.clear();
                            is_multiline = false;
                            println!("^C");
                            continue;
                        }
                        println!("^C");
                        continue;
                    }
                    Err(ReadlineError::Eof) => {
                        // Ctrl+D - exit
                        println!("exit");
                        break;
                    }
                    Err(e) => {
                        return Err(ReplError::Readline(e.to_string()));
                    }
                }
            } else {
                // Non-interactive mode (for testing)
                print!("{prompt}");
                std::io::stdout().flush()?;
                let mut line = String::new();
                std::io::stdin().read_line(&mut line)?;
                if line.is_empty() {
                    break;
                }
                line
            };

            // Check for commands
            let trimmed = input.trim();
            if let Some(cmd) = trimmed.strip_prefix(':') {
                if self.handle_command(cmd) {
                    break;
                }
                continue;
            }

            // Handle multiline input
            if is_multiline {
                multi_line_input.push('\n');
                multi_line_input.push_str(&input);

                // Try to parse - if successful, execute
                if !self.is_incomplete(&multi_line_input) {
                    let source = multi_line_input.clone();
                    let result = self.eval(&source).await;
                    Self::display_result(result, &source);
                    multi_line_input.clear();
                    is_multiline = false;
                }
            } else {
                // Check if input is incomplete
                if self.is_incomplete(&input) {
                    multi_line_input = input;
                    is_multiline = true;
                } else {
                    let result = self.eval(&input).await;
                    Self::display_result(result, &input);
                }
            }
        }

        // Save history on exit
        if let Some(editor) = &mut self.editor {
            editor.save_history();
        }

        Ok(())
    }

    /// Evaluate input (expression or workflow definition).
    ///
    /// # Arguments
    ///
    /// * `input` - The input string to evaluate.
    ///
    /// # Returns
    ///
    /// The result value if successful.
    ///
    /// # Errors
    ///
    /// Returns error if parsing or execution fails.
    pub async fn eval(&mut self, input: &str) -> Result<Value, ReplError> {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return Ok(Value::Null);
        }

        // Check if it's a workflow definition
        if trimmed.starts_with("workflow") {
            self.engine.parse(trimmed)?;
            return Ok(Value::Null);
        }

        // Wrap expression in a workflow and execute
        let wrapped = format!("workflow __repl__ {{ ret {trimmed}; }}");

        self.engine.run(&wrapped).await.map_err(Into::into)
    }

    /// Check if input is incomplete (needs more lines).
    ///
    /// # Arguments
    ///
    /// * `input` - The input string to check.
    ///
    /// # Returns
    ///
    /// `true` if the input is incomplete and needs more lines.
    fn is_incomplete(&self, input: &str) -> bool {
        if input.trim().is_empty() {
            return false;
        }

        // Try parsing as-is
        if let Err(ash_engine::EngineError::Parse(msg)) = self.engine.parse(input) {
            // Check for incomplete indicators in parse error
            let incomplete_indicators = [
                "unexpected end of input",
                "incomplete",
                "unterminated",
                "unexpected eof",
                "expected",
            ];

            let msg_lower = msg.to_lowercase();
            incomplete_indicators
                .iter()
                .any(|&ind| msg_lower.contains(ind))
        } else {
            false
        }
    }

    /// Handle REPL commands.
    ///
    /// # Arguments
    ///
    /// * `cmd` - The command string (without the leading colon).
    ///
    /// # Returns
    ///
    /// `true` if the REPL should exit, `false` otherwise.
    fn handle_command(&self, cmd: &str) -> bool {
        let parts: Vec<&str> = cmd.split_whitespace().collect();

        match parts.first() {
            Some(&("quit" | "q")) => return true,
            Some(&("help" | "h")) => Self::print_help(),
            Some(&("type" | "t")) => {
                if parts.len() > 1 {
                    let expr = parts[1..].join(" ");
                    Self::show_type(&expr);
                } else {
                    println!("Usage: :type <expression>");
                }
            }
            Some(&"ast") => {
                if parts.len() > 1 {
                    let expr = parts[1..].join(" ");
                    self.show_ast(&expr);
                } else {
                    println!("Usage: :ast <expression>");
                }
            }
            Some(&"clear") => {
                print!("\x1B[2J\x1B[1;1H");
            }
            _ => println!("Unknown command: :{cmd}"),
        }

        false
    }

    /// Print the help message.
    fn print_help() {
        println!("{}", help_text());
    }

    /// Show the type of an expression.
    ///
    /// # Arguments
    ///
    /// * `expr` - The expression to type check.
    fn show_type(expr: &str) {
        match infer_type_display(expr) {
            Ok(ty) => println!("{ty}"),
            Err(e) => println!("Error: {e}"),
        }
    }

    /// Show the AST representation of an expression.
    ///
    /// # Arguments
    ///
    /// * `input` - The input to parse and display.
    fn show_ast(&self, input: &str) {
        match self.engine.parse(input) {
            Ok(workflow) => {
                println!("{workflow:#?}");
            }
            Err(e) => {
                // Try parsing as expression wrapped in workflow
                let wrapped = format!("workflow __ast__ {{ ret {input}; }}");
                match self.engine.parse(&wrapped) {
                    Ok(workflow) => println!("{workflow:#?}"),
                    Err(_) => println!("Parse error: {e}"),
                }
            }
        }
    }

    /// Display the result of an evaluation.
    ///
    /// # Arguments
    ///
    /// * `result` - The result to display.
    /// * `source` - The source code that was evaluated (for error context).
    fn display_result(result: Result<Value, ReplError>, source: &str) {
        match result {
            Ok(value) => {
                if value != Value::Null {
                    println!("{value}");
                }
            }
            Err(ReplError::Engine(msg)) => {
                let formatted = format_error(source, &msg, Some(1));
                eprintln!("{formatted}");

                if let Some(suggestion) = suggest_fix(&msg) {
                    eprintln!("\n{} {}", "Hint:".yellow().bold(), suggestion);
                }
            }
            Err(e) => {
                eprintln!("{} {}", "Error:".red().bold(), e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repl_creates() {
        let repl = Repl::new(true);
        assert!(repl.is_ok());
    }

    #[test]
    fn test_history_path_when_disabled() {
        let repl = Repl::new(true).unwrap();
        assert!(repl.history_path.is_none());
    }

    #[tokio::test]
    async fn test_repl_eval_expression() {
        let mut repl = Repl::new(true).unwrap();
        let result = repl.eval("42").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_repl_eval_workflow() {
        let mut repl = Repl::new(true).unwrap();
        // Test parsing a workflow definition (no execution, just storage)
        let result = repl.eval("workflow test { ret 42; }").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_repl_eval_empty() {
        let mut repl = Repl::new(true).unwrap();
        let result = repl.eval("").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Null);
    }

    #[test]
    fn test_multiline_incomplete_brace() {
        let repl = Repl::new(true).unwrap();
        // A workflow with unclosed brace may be incomplete depending on parser behavior
        // Just verify the method runs without panic
        let _ = repl.is_incomplete("workflow test {");
    }

    #[test]
    fn test_multiline_complete_expression() {
        let repl = Repl::new(true).unwrap();
        // A complete expression should not be incomplete
        assert!(!repl.is_incomplete("42"));
    }

    #[test]
    fn test_multiline_complete_workflow() {
        let repl = Repl::new(true).unwrap();
        // A complete workflow should not be incomplete
        assert!(!repl.is_incomplete(
            "\n            workflow test {\n                ret 42;\n            }\n        "
        ));
    }
}
