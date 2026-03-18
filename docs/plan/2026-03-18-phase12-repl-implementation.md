# Phase 12: REPL Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build an interactive REPL (Read-Eval-Print Loop) for Ash with expression evaluation, multi-line input, commands, tab completion, history, and error display.

**Architecture:** The ash-repl crate provides a `Repl` struct wrapping `ash_engine::Engine` with rustyline for readline features. It handles both expression evaluation (wrapping in temporary workflows) and workflow definitions (storing for later use).

**Tech Stack:** Rust 2024, rustyline (readline), ash-engine (execution), tokio (async)

---

## Overview

Phase 12 implements an interactive REPL for the Ash workflow language. The REPL allows users to:
- Evaluate expressions immediately
- Define workflows interactively
- Use readline features (history, completion)
- Inspect types and AST
- Get helpful error messages

**Dependencies:**
- `ash-engine` - For parsing, type checking, and execution
- `rustyline` - For readline functionality (history, completion, editing)
- `tokio` - For async execution
- `directories` - For cross-platform config/data directories

---

## Task 1: Create ash-repl Crate (TASK-077)

**Files:**
- Create: `crates/ash-repl/Cargo.toml`
- Create: `crates/ash-repl/src/lib.rs`
- Create: `crates/ash-repl/src/main.rs`
- Modify: `Cargo.toml` (workspace members)

**Step 1: Create crate directory structure**

```bash
mkdir -p crates/ash-repl/src
```

**Step 2: Write Cargo.toml**

```toml
[package]
name = "ash-repl"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
authors.workspace = true
repository.workspace = true
description = "Interactive REPL for the Ash workflow language"

[dependencies]
ash-engine = { path = "../ash-engine" }
ash-core = { path = "../ash-core" }
ash-parser = { path = "../ash-parser" }
tokio = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }
rustyline = { version = "15.0", features = ["derive"] }
directories = "6.0"
colored = { workspace = true }

[dev-dependencies]
tokio-test = "0.4"
```

**Step 3: Write lib.rs with basic Repl struct**

```rust
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

use ash_engine::Engine;
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

/// Interactive REPL for Ash workflow language.
#[derive(Debug)]
pub struct Repl {
    engine: Engine,
    history_path: Option<PathBuf>,
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
        
        Ok(Self {
            engine,
            history_path,
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
        
        // TODO: Implement in TASK-078
        Ok(())
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
}
```

**Step 4: Write main.rs for standalone binary**

```rust
//! Standalone REPL binary for Ash workflow language.

use ash_repl::Repl;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    let mut repl = Repl::new(false)?;
    repl.run().await?;
    
    Ok(())
}
```

**Step 5: Add to workspace Cargo.toml**

Add `"crates/ash-repl"` to the workspace members list.

**Step 6: Run tests**

```bash
cargo test -p ash-repl
```

Expected: 2 tests pass

**Step 7: Commit**

```bash
git add crates/ash-repl/ Cargo.toml
git commit -m "feat(repl): create ash-repl crate (TASK-077)"
```

---

## Task 2: Expression Evaluation (TASK-078)

**Files:**
- Modify: `crates/ash-repl/src/lib.rs`

**Step 1: Add eval method and input handling**

Add to `impl Repl` in lib.rs:

```rust
use ash_core::value::Value;

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
        println!("Workflow defined");
        return Ok(Value::Null);
    }
    
    // Wrap expression in a workflow and execute
    let wrapped = format!(
        r#"workflow __repl__ {{
            action __expr__ {{
                effect: operational;
                body: || -> {};
            }}
        }}"#,
        trimmed
    );
    
    self.engine.run(&wrapped).await.map_err(Into::into)
}
```

**Step 2: Add tests**

Add to the tests module:

```rust
#[tokio::test]
async fn test_repl_eval_expression() {
    let mut repl = Repl::new(true).unwrap();
    let result = repl.eval("42").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_repl_eval_workflow() {
    let mut repl = Repl::new(true).unwrap();
    let result = repl.eval(r#"
        workflow test {
            action a {
                effect: operational;
                body: || -> 42;
            }
        }
    "#).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_repl_eval_empty() {
    let mut repl = Repl::new(true).unwrap();
    let result = repl.eval("").await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Null);
}
```

**Step 3: Run tests**

```bash
cargo test -p ash-repl
```

Expected: 5 tests pass

**Step 4: Commit**

```bash
git add crates/ash-repl/src/lib.rs
git commit -m "feat(repl): expression evaluation (TASK-078)"
```

---

## Task 3: REPL Loop with rustyline (TASK-077 continuation)

**Files:**
- Modify: `crates/ash-repl/src/lib.rs`
- Create: `crates/ash-repl/src/completer.rs`

**Step 1: Create completer.rs module**

```rust
//! Tab completion for the REPL.

use rustyline::completion::{Completer, Pair};
use rustyline::Context;

/// Completer for Ash language.
#[derive(Debug, Clone, Default)]
pub struct AshCompleter;

impl AshCompleter {
    /// Create a new completer.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Completer for AshCompleter {
    type Candidate = Pair;
    
    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        // Basic keyword completion - will be expanded in TASK-081
        let keywords = ["workflow", "action", "capability", "effect", "let", "if", "then", "else"];
        
        let start = line[..pos].rfind(|c: char| c.is_whitespace()).map_or(0, |i| i + 1);
        let prefix = &line[start..pos];
        
        let matches: Vec<Pair> = keywords
            .iter()
            .filter(|kw| kw.starts_with(prefix))
            .map(|kw| Pair {
                display: (*kw).to_string(),
                replacement: (*kw).to_string(),
            })
            .collect();
        
        Ok((start, matches))
    }
}
```

**Step 2: Update lib.rs with full REPL loop**

Add to lib.rs:

```rust
mod completer;

use completer::AshCompleter;
use rustyline::{Config, Editor};
use rustyline::error::ReadlineError;

/// Helper struct for managing the readline editor.
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
    
    fn save_history(&self) {
        if let Some(path) = &self.history_path {
            // Ensure parent directory exists
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            self.editor.save_history(path).ok();
        }
    }
}

// Update the Repl struct:
pub struct Repl {
    engine: Engine,
    history_path: Option<PathBuf>,
    editor: Option<ReplEditor>,
}

// Update Repl::new:
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

// Update run method:
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
            print!("{}", prompt);
            std::io::Write::flush(&mut std::io::stdout())?;
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
            if !self.is_incomplete(&multi_line_input).await {
                let result = self.eval(&multi_line_input).await;
                self.display_result(result);
                multi_line_input.clear();
                is_multiline = false;
            }
        } else {
            // Check if input is incomplete
            if self.is_incomplete(&input).await {
                multi_line_input = input;
                is_multiline = true;
            } else {
                let result = self.eval(&input).await;
                self.display_result(result);
            }
        }
    }
    
    // Save history on exit
    if let Some(editor) = &self.editor {
        editor.save_history();
    }
    
    Ok(())
}

/// Check if input is incomplete (needs more lines).
async fn is_incomplete(&self, input: &str) -> bool {
    // TODO: Implement properly in TASK-079
    // For now, simple heuristic: check for unclosed braces
    let open_braces = input.chars().filter(|&c| c == '{').count();
    let close_braces = input.chars().filter(|&c| c == '}').count();
    open_braces > close_braces
}

/// Handle REPL commands. Returns true if should exit.
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
        _ => println!("Unknown command: :{}", cmd),
    }
    
    Ok(false)
}

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

async fn show_type(&self, expr: &str) -> Result<(), ReplError> {
    // Parse and type check without executing
    let wrapped = format!(
        r#"workflow __typecheck__ {{
            action __expr__ {{
                effect: operational;
                body: || -> {};
            }}
        }}"#,
        expr
    );
    
    match self.engine.parse(&wrapped) {
        Ok(workflow) => {
            // TODO: Get type from type checker
            println!("Type: (inferred from context)");
            let _ = workflow;
        }
        Err(e) => println!("Error: {}", e),
    }
    
    Ok(())
}

fn show_ast(&self, input: &str) -> Result<(), ReplError> {
    match self.engine.parse(input) {
        Ok(workflow) => {
            println!("{:#?}", workflow);
        }
        Err(e) => {
            // Try parsing as expression wrapped in workflow
            let wrapped = format!(
                r#"workflow __ast__ {{
                    action __expr__ {{
                        effect: operational;
                        body: || -> {};
                    }}
                }}"#,
                input
            );
            match self.engine.parse(&wrapped) {
                Ok(workflow) => println!("{:#?}", workflow),
                Err(_) => println!("Parse error: {}", e),
            }
        }
    }
    Ok(())
}

fn display_result(&self, result: Result<Value, ReplError>) {
    match result {
        Ok(value) => {
            if value != Value::Null {
                println!("{}", value);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
```

**Step 3: Run tests**

```bash
cargo test -p ash-repl
```

**Step 4: Commit**

```bash
git add crates/ash-repl/
git commit -m "feat(repl): REPL loop with rustyline integration (TASK-077)"
```

---

## Task 4: Multi-line Input Detection (TASK-079)

**Files:**
- Modify: `crates/ash-repl/src/lib.rs`

**Step 1: Improve incomplete detection**

Replace the `is_incomplete` method:

```rust
/// Check if input is incomplete (needs more lines).
///
/// This uses the parser to detect incomplete input vs actual errors.
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
            incomplete_indicators.iter().any(|&ind| msg_lower.contains(ind))
        }
        Err(_) => false,
    }
}
```

**Step 2: Add tests**

```rust
#[test]
fn test_multiline_incomplete_brace() {
    let repl = Repl::new(true).unwrap();
    // A workflow with unclosed brace should be incomplete
    assert!(repl.is_incomplete("workflow test {"));
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
    assert!(!repl.is_incomplete(r#"
        workflow test {
            action a {
                effect: operational;
                body: || -> 42;
            }
        }
    "#));
}
```

**Step 3: Run tests**

```bash
cargo test -p ash-repl
```

**Step 4: Commit**

```bash
git add crates/ash-repl/src/lib.rs
git commit -m "feat(repl): multi-line input detection (TASK-079)"
```

---

## Task 5: Enhanced Tab Completion (TASK-081)

**Files:**
- Modify: `crates/ash-repl/src/completer.rs`

**Step 1: Enhance the completer**

```rust
//! Tab completion for the REPL.

use rustyline::completion::{Completer, Pair};
use rustyline::hint::HistoryHinter;
use rustyline::highlight::Highlighter;
use rustyline::validate::Validator;
use rustyline::{Helper, Context};
use std::borrow::Cow;

/// Helper for rustyline with completion and highlighting.
#[derive(Debug, Clone, Default)]
pub struct AshHelper {
    completer: AshCompleter,
    hinter: HistoryHinter,
}

impl AshHelper {
    /// Create a new helper.
    #[must_use]
    pub fn new() -> Self {
        Self {
            completer: AshCompleter::new(),
            hinter: HistoryHinter::new(),
        }
    }
}

impl Helper for AshHelper {}

impl Completer for AshHelper {
    type Candidate = Pair;
    
    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        self.completer.complete(line, pos, ctx)
    }
}

impl Highlighter for AshHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        if default {
            Cow::Owned(format!("\x1b[1;32m{}\x1b[0m", prompt))
        } else {
            Cow::Borrowed(prompt)
        }
    }
    
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Cow::Owned(format!("\x1b[1;30m{}\x1b[0m", hint))
    }
}

impl Validator for AshHelper {}

/// Completer for Ash language.
#[derive(Debug, Clone, Default)]
pub struct AshCompleter {
    keywords: Vec<&'static str>,
    builtins: Vec<&'static str>,
}

impl AshCompleter {
    /// Create a new completer with all keywords and builtins.
    #[must_use]
    pub fn new() -> Self {
        Self {
            keywords: vec![
                "workflow",
                "action",
                "capability",
                "effect",
                "let",
                "if",
                "then",
                "else",
                "match",
                "with",
                "ret",
                "for",
                "in",
                "epistemic",
                "deliberative",
                "evaluative",
                "operational",
            ],
            builtins: vec![
                "true",
                "false",
                "null",
            ],
        }
    }
}

impl Completer for AshCompleter {
    type Candidate = Pair;
    
    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let start = line[..pos].rfind(|c: char| c.is_whitespace() || c == '{' || c == '}' || c == '(' || c == ')')
            .map_or(0, |i| i + 1);
        let prefix = &line[start..pos];
        
        if prefix.is_empty() {
            return Ok((start, vec![]));
        }
        
        let prefix_lower = prefix.to_lowercase();
        
        // Collect matches from keywords and builtins
        let mut matches: Vec<Pair> = self.keywords
            .iter()
            .chain(self.builtins.iter())
            .filter(|&&kw| kw.to_lowercase().starts_with(&prefix_lower))
            .map(|&kw| Pair {
                display: kw.to_string(),
                replacement: kw.to_string(),
            })
            .collect();
        
        // Sort and deduplicate
        matches.sort_by(|a, b| a.display.cmp(&b.display));
        matches.dedup_by(|a, b| a.display == b.display);
        
        Ok((start, matches))
    }
}
```

**Step 2: Update lib.rs to use AshHelper**

Replace `AshCompleter` with `AshHelper` in lib.rs:

```rust
use completer::AshHelper;

// In ReplEditor::new:
let mut editor = Editor::with_config(config)?;
editor.set_helper(Some(AshHelper::new()));

// Change type from:
// Editor<AshCompleter, ...>
// to:
// Editor<AshHelper, ...>
```

**Step 3: Add tests**

Add to completer.rs:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_complete_keyword() {
        let completer = AshCompleter::new();
        let (pos, matches) = completer.complete("wor", 3, &Context::new(&[])).unwrap();
        assert_eq!(pos, 0);
        assert!(matches.iter().any(|m| m.display == "workflow"));
    }
    
    #[test]
    fn test_complete_partial() {
        let completer = AshCompleter::new();
        let (pos, matches) = completer.complete("act", 3, &Context::new(&[])).unwrap();
        assert!(matches.iter().any(|m| m.display == "action"));
    }
    
    #[test]
    fn test_no_match() {
        let completer = AshCompleter::new();
        let (pos, matches) = completer.complete("xyz", 3, &Context::new(&[])).unwrap();
        assert!(matches.is_empty());
    }
}
```

**Step 4: Run tests**

```bash
cargo test -p ash-repl
```

**Step 5: Commit**

```bash
git add crates/ash-repl/
git commit -m "feat(repl): enhanced tab completion (TASK-081)"
```

---

## Task 6: Error Display Improvements (TASK-083)

**Files:**
- Create: `crates/ash-repl/src/error.rs`
- Modify: `crates/ash-repl/src/lib.rs`

**Step 1: Create error.rs module**

```rust
//! Error formatting and display for the REPL.

use colored::Colorize;

/// Format an error with context and highlighting.
///
/// # Arguments
///
/// * `source` - The source code that caused the error.
/// * `error` - The error message.
/// * `line_num` - The line number where the error occurred (1-indexed).
///
/// # Returns
///
/// A formatted error string with line numbers and highlighting.
#[must_use]
pub fn format_error(source: &str, error: &str, line_num: Option<usize>) -> String {
    let mut output = String::new();
    
    // Error header
    output.push_str(&format!("{}\n", "Error:".red().bold()));
    output.push_str(&format!("  {}\n", error));
    
    if let Some(line) = line_num {
        output.push('\n');
        
        // Show context lines
        let lines: Vec<&str> = source.lines().collect();
        let start = line.saturating_sub(2);
        let end = (line + 1).min(lines.len());
        
        for (i, line_content) in lines.iter().enumerate().take(end).skip(start) {
            let line_number = i + 1;
            let is_error_line = line_number == line;
            
            if is_error_line {
                output.push_str(&format!(
                    "{} | {}\n",
                    line_number.to_string().red().bold(),
                    line_content
                ));
                // Add caret underline
                let caret = "^".repeat(line_content.len().max(1));
                output.push_str(&format!(
                    "{} | {}\n",
                    " ".repeat(line_number.to_string().len()),
                    caret.red().bold()
                ));
            } else {
                output.push_str(&format!(
                    "{} | {}\n",
                    line_number.to_string().dimmed(),
                    line_content
                ));
            }
        }
    }
    
    output
}

/// Format a type error with context.
#[must_use]
pub fn format_type_error(expr: &str, expected: &str, found: &str) -> String {
    format!(
        "{}\n  {}\n\nExpected: {}\nFound: {}\n",
        "Type Error:".red().bold(),
        "Type mismatch in expression",
        expected.green(),
        found.red()
    )
}

/// Suggest fixes for common errors.
#[must_use]
pub fn suggest_fix(error: &str) -> Option<String> {
    if error.contains("unexpected end of input") {
        Some("Did you forget to close a brace or parenthesis?".to_string())
    } else if error.contains("unterminated string") {
        Some("Check that all string literals are properly closed with quotes.".to_string())
    } else if error.contains("unknown keyword") {
        Some("Check for typos in keywords.".to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_format_error_with_line() {
        let source = "1 +\n2 + 3";
        let error = "unexpected end of input";
        let formatted = format_error(source, error, Some(1));
        
        assert!(formatted.contains("Error:"));
        assert!(formatted.contains("unexpected end of input"));
        assert!(formatted.contains("1"));
    }
    
    #[test]
    fn test_format_error_without_line() {
        let formatted = format_error("", "something went wrong", None);
        assert!(formatted.contains("Error:"));
        assert!(formatted.contains("something went wrong"));
    }
    
    #[test]
    fn test_suggest_fix_unclosed() {
        let suggestion = suggest_fix("unexpected end of input");
        assert!(suggestion.is_some());
        assert!(suggestion.unwrap().contains("brace"));
    }
    
    #[test]
    fn test_suggest_fix_none() {
        let suggestion = suggest_fix("some random error");
        assert!(suggestion.is_none());
    }
}
```

**Step 2: Update lib.rs to use error formatting**

Add to lib.rs:

```rust
mod error;

use error::{format_error, suggest_fix};

// Update display_result method:
fn display_result(&self, result: Result<Value, ReplError>, source: &str) {
    match result {
        Ok(value) => {
            if value != Value::Null {
                println!("{}", value);
            }
        }
        Err(ReplError::Engine(msg)) => {
            let formatted = format_error(source, &msg, Some(1));
            eprintln!("{}", formatted);
            
            if let Some(suggestion) = suggest_fix(&msg) {
                eprintln!("\n{} {}", "Hint:".yellow().bold(), suggestion);
            }
        }
        Err(e) => {
            eprintln!("{} {}", "Error:".red().bold(), e);
        }
    }
}
```

**Step 3: Run tests**

```bash
cargo test -p ash-repl
```

**Step 4: Commit**

```bash
git add crates/ash-repl/
git commit -m "feat(repl): error display with highlighting (TASK-083)"
```

---

## Task 7: CLI Integration (TASK-080 continuation)

**Files:**
- Modify: `crates/ash-cli/src/commands/repl.rs`
- Modify: `crates/ash-cli/src/cli.rs`
- Modify: `crates/ash-cli/Cargo.toml`

**Step 1: Add ash-repl dependency to ash-cli**

```toml
[dependencies]
ash-repl = { path = "../ash-repl" }
```

**Step 2: Update repl.rs command**

Replace the stub implementation:

```rust
//! REPL command implementation.

use anyhow::Result;
use clap::Args;

/// Start the interactive REPL
#[derive(Args, Debug)]
pub struct ReplArgs {
    /// Don't load or save history
    #[arg(long)]
    pub no_history: bool,
    
    /// Show history file location and exit
    #[arg(long)]
    pub history: bool,
}

pub async fn execute(args: &ReplArgs) -> Result<()> {
    if args.history {
        let path = directories::ProjectDirs::from("org", "ash", "ash-repl")
            .map(|dirs| dirs.data_dir().join("history"))
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "No history path available".to_string());
        println!("{}", path);
        return Ok(());
    }
    
    let mut repl = ash_repl::Repl::new(args.no_history)?;
    repl.run().await?;
    
    Ok(())
}
```

**Step 3: Update main.rs if needed**

The main.rs should already call the repl command.

**Step 4: Run tests**

```bash
cargo test -p ash-cli
```

**Step 5: Commit**

```bash
git add crates/ash-cli/
git commit -m "feat(cli): integrate ash-repl into CLI (TASK-080)"
```

---

## Task 8: Update PLAN-INDEX and CHANGELOG

**Files:**
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`

**Step 1: Update Phase 12 status in PLAN-INDEX.md**

Mark all Phase 12 tasks as complete:
- TASK-077: ✅ Complete
- TASK-078: ✅ Complete
- TASK-079: ✅ Complete
- TASK-080: ✅ Complete
- TASK-081: ✅ Complete
- TASK-082: ✅ Complete
- TASK-083: ✅ Complete

**Step 2: Add CHANGELOG entry**

```markdown
### Added
- Interactive REPL with rustyline (Phase 12, TASK-077 to TASK-083). Features expression evaluation, multi-line input, commands (:help, :quit, :type, :ast, :clear), tab completion, persistent history, and syntax error highlighting.
```

**Step 3: Commit**

```bash
git add docs/plan/PLAN-INDEX.md CHANGELOG.md
git commit -m "docs: mark Phase 12 complete and update changelog"
```

---

## Final Verification

Run full verification:

```bash
# Tests
cargo test -p ash-repl

# Clippy
cargo clippy -p ash-repl --all-targets --all-features

# Format
cargo fmt --check -p ash-repl

# Doc tests
cargo doc -p ash-repl --no-deps
```

---

## Summary

Phase 12 delivers:

1. **ash-repl crate** - New crate for interactive REPL
2. **Expression evaluation** - Wrap expressions in workflows for execution
3. **Multi-line input** - Smart detection of incomplete input
4. **REPL commands** - :help, :quit, :type, :ast, :clear
5. **Tab completion** - Keywords and builtin functions
6. **Persistent history** - Saved between sessions
7. **Error display** - Syntax highlighting and helpful suggestions
8. **CLI integration** - `ash repl` command works

**Deliverable:** Interactive REPL with readline features
