//! Interactive REPL for Ash.
//!
//! The REPL provides an interactive environment for:
//! - Quick experimentation with Ash syntax
//! - Testing target-Ash expressions and entry functions
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

mod ast;
mod completer;
mod display;
mod editor;
mod error;
pub mod input;
pub mod session;

pub use input::{InputDetector, InputStatus};

pub use ash_core::Value;
use ash_engine::{CanonicalTerminalEnvelopeV1, Engine, EngineError};
use colored::Colorize;
use error::{format_error, suggest_fix};
use rustyline::error::ReadlineError;
pub use session::{EvalResult, Session};
use std::io::Write;
use std::path::Path;
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
const REPL_SUBMISSION_PATH: &str = "repl-submission.ash";

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
    /// Type error during type checking.
    #[error("type error: {0}")]
    TypeError(String),
    /// Unknown stored entry computation referenced.
    #[error("unknown entry: {name}")]
    UnknownEntry {
        /// The name of the unknown entry.
        name: String,
    },
    /// Parse error.
    #[error("parse error: {0}")]
    ParseError(String),
}

impl From<ash_engine::EngineError> for ReplError {
    fn from(err: ash_engine::EngineError) -> Self {
        match err {
            ash_engine::EngineError::Type(msg) => Self::TypeError(msg),
            ash_engine::EngineError::Parse(msg) | ash_engine::EngineError::Execution(msg) => {
                Self::Engine(msg)
            }
            ash_engine::EngineError::Io(e) => Self::Io(e),
            ash_engine::EngineError::CapabilityNotFound(cap) => {
                Self::Engine(format!("capability not found: {cap}"))
            }
            ash_engine::EngineError::Configuration(msg) => {
                Self::Engine(format!("configuration error: {msg}"))
            }
            ash_engine::EngineError::ProductionTerminal {
                classification,
                message,
            } => Self::Engine(format!("production terminal {classification:?}: {message}")),
        }
    }
}

impl From<ash_runtime::ExecError> for ReplError {
    fn from(err: ash_runtime::ExecError) -> Self {
        Self::Engine(err.to_string())
    }
}

impl From<ReadlineError> for ReplError {
    fn from(err: ReadlineError) -> Self {
        Self::Readline(err.to_string())
    }
}

/// Render the normalized result of an Engine-issued admitted request.
///
/// The REPL owns only this presentation mapping; request admission and
/// execution authority remain in [`Engine`].
pub(crate) fn render_canonical_terminal(
    terminal: CanonicalTerminalEnvelopeV1,
) -> Result<Value, ReplError> {
    match terminal {
        CanonicalTerminalEnvelopeV1::Returned(value) => Ok(value),
        CanonicalTerminalEnvelopeV1::Trapped(reason) => Err(ReplError::Engine(format!(
            "admitted program terminal trap: {reason}"
        ))),
        CanonicalTerminalEnvelopeV1::AdmissionRejected => {
            Err(ReplError::Engine("admission rejected".to_string()))
        }
        CanonicalTerminalEnvelopeV1::InvalidCheckedArtifact => Err(ReplError::Engine(
            "checked Core/CPS artifact is invalid".to_string(),
        )),
        CanonicalTerminalEnvelopeV1::TimedOut => {
            Err(ReplError::Engine("admitted program timed out".to_string()))
        }
        CanonicalTerminalEnvelopeV1::Cancelled => {
            Err(ReplError::Engine("admitted program cancelled".to_string()))
        }
    }
}

pub(crate) fn map_admission_error(error: &EngineError) -> ReplError {
    if let Some(terminal) = error.canonical_terminal_envelope() {
        return match render_canonical_terminal(terminal) {
            Ok(value) => ReplError::Engine(format!(
                "admission produced an unexpected returned terminal: {value}"
            )),
            Err(error) => error,
        };
    }

    ReplError::Engine(format!(
        "application execution failed: checked Core/CPS admission rejected: {error}"
    ))
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

pub use display::{ast_display, infer_type_display};

/// Run a REPL session with explicit session configuration.
///
/// # Errors
///
/// Returns an error when session initialization or interactive execution fails.
pub async fn run_with_config(config: ReplConfig) -> Result<(), ReplError> {
    let mut repl = Repl::from_config(config)?;
    repl.run().await
}

use editor::ReplEditor;

/// Interactive REPL for Ash.
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
                if Self::handle_command(cmd) {
                    break;
                }
                continue;
            }

            // Handle multiline input
            if is_multiline {
                multi_line_input.push('\n');
                multi_line_input.push_str(&input);

                // Try to parse - if successful, execute
                if !Self::is_incomplete(&multi_line_input) {
                    let source = multi_line_input.clone();
                    let result = self.eval(&source).await;
                    Self::display_result(result, &source);
                    multi_line_input.clear();
                    is_multiline = false;
                }
            } else {
                // Check if input is incomplete
                if Self::is_incomplete(&input) {
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

    /// Evaluate input as an expression or target entry function.
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
        if input.trim().is_empty() {
            return Ok(Value::Null);
        }

        let mut entry = self
            .engine
            .parse_file_source(Path::new(REPL_SUBMISSION_PATH), input)?;
        let execution = {
            let admitted = self
                .engine
                .admit_program(&mut entry)
                .map_err(|error| map_admission_error(&error))?;
            let (request, _cancellation) =
                self.engine.new_admitted_program_request(&admitted, None)?;
            self.engine.execute_admitted_program(&request)
        };
        let terminal = execution.await?;

        render_canonical_terminal(terminal)
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
    fn is_incomplete(input: &str) -> bool {
        let mut detector = InputDetector::new();
        matches!(detector.check(input), InputStatus::Incomplete(_))
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
    fn handle_command(cmd: &str) -> bool {
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
                    Self::show_ast(&expr);
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
    fn show_ast(input: &str) {
        match ast_display(input) {
            Ok(ast) => println!("{ast}"),
            Err(e) => println!("Error: {e}"),
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
mod tests;
