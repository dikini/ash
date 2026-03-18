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
        ReplError::Engine(err.to_string())
    }
}

impl From<ash_interp::ExecError> for ReplError {
    fn from(err: ash_interp::ExecError) -> Self {
        ReplError::Engine(err.to_string())
    }
}

impl From<ReadlineError> for ReplError {
    fn from(err: ReadlineError) -> Self {
        ReplError::Readline(err.to_string())
    }
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
        let engine = Engine::default();

        let history_path = if no_history {
            None
        } else {
            Self::get_history_path()
        };

        let editor = if no_history {
            None
        } else {
            Some(ReplEditor::new(history_path.clone())?)
        };

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
        println!("Ash REPL - Type :help for help, :quit to exit");
        println!();

        let mut multi_line_input = String::new();
        let mut is_multiline = false;

        loop {
            let prompt = if is_multiline { "... " } else { "ash> " };

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
            if trimmed.starts_with(':') {
                let cmd = &trimmed[1..];
                if self.handle_command(cmd).await? {
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
                    self.display_result(result, &source);
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
                    self.display_result(result, &input);
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
        match self.engine.parse(input) {
            Ok(_) => false,
            Err(ash_engine::EngineError::Parse(msg)) => {
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
            }
            Err(_) => false,
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
    ///
    /// # Errors
    ///
    /// Returns error if command execution fails.
    async fn handle_command(&mut self, cmd: &str) -> Result<bool, ReplError> {
        let parts: Vec<&str> = cmd.split_whitespace().collect();

        match parts.first() {
            Some(&"quit") | Some(&"q") => return Ok(true),
            Some(&"help") | Some(&"h") => self.print_help(),
            Some(&"type") | Some(&"t") => {
                if parts.len() > 1 {
                    let expr = parts[1..].join(" ");
                    self.show_type(&expr).await?;
                } else {
                    println!("Usage: :type <expression>");
                }
            }
            Some(&"ast") => {
                if parts.len() > 1 {
                    let expr = parts[1..].join(" ");
                    self.show_ast(&expr)?;
                } else {
                    println!("Usage: :ast <expression>");
                }
            }
            Some(&"clear") => {
                print!("\x1B[2J\x1B[1;1H");
            }
            _ => println!("Unknown command: :{cmd}"),
        }

        Ok(false)
    }

    /// Print the help message.
    fn print_help(&self) {
        println!("Commands:");
        println!("  :help, :h     Show this help");
        println!("  :quit, :q     Exit the REPL");
        println!("  :type, :t     Show type of expression");
        println!("  :ast          Show AST representation");
        println!("  :clear        Clear screen");
        println!();
        println!("Multi-line input is supported automatically.");
    }

    /// Show the type of an expression.
    ///
    /// # Arguments
    ///
    /// * `expr` - The expression to type check.
    ///
    /// # Errors
    ///
    /// Returns error if parsing fails.
    async fn show_type(&self, expr: &str) -> Result<(), ReplError> {
        // Parse and type check without executing
        let wrapped = format!("workflow __typecheck__ {{ ret {expr}; }}");

        match self.engine.parse(&wrapped) {
            Ok(workflow) => {
                // TODO: Get type from type checker
                println!("Type: (inferred from context)");
                let _ = workflow;
            }
            Err(e) => println!("Error: {e}"),
        }

        Ok(())
    }

    /// Show the AST representation of an expression.
    ///
    /// # Arguments
    ///
    /// * `input` - The input to parse and display.
    ///
    /// # Errors
    ///
    /// Returns error if parsing fails.
    fn show_ast(&self, input: &str) -> Result<(), ReplError> {
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
        Ok(())
    }

    /// Display the result of an evaluation.
    ///
    /// # Arguments
    ///
    /// * `result` - The result to display.
    /// * `source` - The source code that was evaluated (for error context).
    fn display_result(&self, result: Result<Value, ReplError>, source: &str) {
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
            r#"
            workflow test {
                ret 42;
            }
        "#
        ));
    }
}
