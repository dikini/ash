//! Session state management for the REPL.
//!
//! This module provides the `Session` type for managing REPL session state,
//! including entry computations that can be stored and invoked by name.

use std::collections::HashMap;

use ash_core::Value;
use ash_engine::{Engine, Workflow as EngineEntry};

use crate::ReplError;

/// A compiled entry computation stored in the session.
///
/// Contains the entry computation, its name, and the verified type
/// from type checking at definition time.
#[derive(Debug, Clone)]
pub struct CompiledEntry {
    /// The name of the entry computation
    pub name: String,
    /// The compiled entry computation ready for execution
    pub entry: EngineEntry,
    /// The verified type string representation
    pub verified_type: String,
    /// The parameter names for this entry.
    pub params: Vec<String>,
    /// The original source code for the computation body
    pub body_source: String,
}

/// The result of evaluating input in the REPL.
#[derive(Debug, Clone, PartialEq)]
pub enum EvalResult {
    /// A computed value
    Value(Value),
    /// An entry computation was successfully defined
    EntryDefined {
        /// The name of the defined entry.
        name: String,
    },
    /// A type was inferred
    Type {
        /// The type that was inferred
        ty: String,
    },
    /// Unit result (no value to display)
    Unit,
}

impl From<Value> for EvalResult {
    fn from(value: Value) -> Self {
        Self::Value(value)
    }
}

/// REPL session state.
///
/// Maintains the state for a REPL session, including:
/// - Stored entry computations
/// - Variable bindings
/// - The execution engine
#[derive(Debug)]
pub struct Session {
    engine: Engine,
    entries: HashMap<String, CompiledEntry>,
    bindings: HashMap<String, Value>,
}

impl Session {
    /// Create a new REPL session with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            engine: Engine::default(),
            entries: HashMap::new(),
            bindings: HashMap::new(),
        }
    }

    /// Evaluate input in the session context.
    ///
    /// The input is an expression that may reference stored entry computations.
    ///
    /// # Errors
    ///
    /// Returns `ReplError` if parsing, type checking, or execution fails.
    pub async fn evaluate(&mut self, input: &str) -> Result<EvalResult, ReplError> {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return Ok(EvalResult::Unit);
        }

        // Check if this looks like a stored computation call: name(...)
        if let Some((call_name, args)) = extract_call_expr(trimmed)
            && self.entries.contains_key(call_name)
        {
            return self.call_entry_by_name(call_name, &args).await;
        }

        // Treat as expression - wrap in a target Ash entry function and execute.
        let wrapped = format!("fn main() {{ {trimmed} }}");
        let mut entry = self.engine.parse(&wrapped)?;
        self.engine.check(&mut entry)?;
        let result = self.engine.execute(&entry).await?;

        Ok(EvalResult::Value(result))
    }

    /// Call a stored entry computation by name with arguments.
    async fn call_entry_by_name(
        &self,
        name: &str,
        args: &[String],
    ) -> Result<EvalResult, ReplError> {
        let compiled = self
            .entries
            .get(name)
            .ok_or_else(|| ReplError::UnknownEntry {
                name: name.to_string(),
            })?;

        // Create a wrapper entry function that binds arguments to parameters.
        let wrapper_source = if compiled.params.is_empty() {
            // No parameters - just call the stored computation directly.
            format!("fn main() {{ {} }}", compiled.body_source)
        } else {
            // Check argument count
            if args.len() != compiled.params.len() {
                return Err(ReplError::Engine(format!(
                    "entry '{}' expects {} arguments, got {}",
                    name,
                    compiled.params.len(),
                    args.len()
                )));
            }

            // Create let bindings for each parameter
            let bindings: Vec<String> = compiled
                .params
                .iter()
                .zip(args.iter())
                .map(|(param, arg)| format!("let {param} = {arg};"))
                .collect();

            format!(
                "fn main() {{ {} {} }}",
                bindings.join(" "),
                compiled.body_source
            )
        };

        // Parse, check, and execute the wrapper
        let mut entry = self.engine.parse(&wrapper_source)?;
        self.engine.check(&mut entry)?;
        let result = self.engine.execute(&entry).await?;

        Ok(EvalResult::Value(result))
    }

    /// Run a stored entry computation with the given input value.
    ///
    /// # Errors
    ///
    /// Returns `ReplError::UnknownEntry` if the computation is not found.
    pub async fn run_entry(&self, name: &str) -> Result<Value, ReplError> {
        let compiled = self
            .entries
            .get(name)
            .ok_or_else(|| ReplError::UnknownEntry {
                name: name.to_string(),
            })?;

        // Execute without re-type-checking
        let result = self.engine.execute(&compiled.entry).await?;

        Ok(result)
    }

    /// Get a reference to a stored entry computation.
    #[must_use]
    pub fn get_entry(&self, name: &str) -> Option<&CompiledEntry> {
        self.entries.get(name)
    }

    /// Check if an entry computation is defined in this session.
    #[must_use]
    pub fn has_entry(&self, name: &str) -> bool {
        self.entries.contains_key(name)
    }

    /// Get the names of all defined entries.
    pub fn entry_names(&self) -> impl Iterator<Item = &String> {
        self.entries.keys()
    }

    /// Insert a binding into the session.
    pub fn bind(&mut self, name: impl Into<String>, value: Value) {
        self.bindings.insert(name.into(), value);
    }

    /// Get a binding from the session.
    #[must_use]
    pub fn get_binding(&self, name: &str) -> Option<&Value> {
        self.bindings.get(name)
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract the function name and arguments from a call expression.
/// Returns `Some((name, args))` if the input looks like `name(arg1, arg2, ...)`.
fn extract_call_expr(input: &str) -> Option<(&str, Vec<String>)> {
    let input = input.trim();

    // Find the opening paren
    let paren_idx = input.find('(')?;

    // Extract the part before the paren
    let name = &input[..paren_idx].trim();

    // It should be a simple identifier (no spaces, no dots)
    if name.contains(' ') || name.contains('.') {
        return None;
    }

    // Verify there's a closing paren
    let close_idx = input.rfind(')')?;

    // Extract arguments
    let args_str = &input[paren_idx + 1..close_idx];
    let args = parse_args(args_str);

    Some((name, args))
}

/// Parse comma-separated arguments, handling nested parentheses and strings.
fn parse_args(input: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut depth = 0;
    let mut in_string = false;
    let mut escape = false;

    for ch in input.chars() {
        if escape {
            current.push(ch);
            escape = false;
            continue;
        }

        if ch == '\\' && in_string {
            escape = true;
            current.push(ch);
            continue;
        }

        if ch == '"' {
            in_string = !in_string;
            current.push(ch);
            continue;
        }

        if in_string {
            current.push(ch);
            continue;
        }

        match ch {
            '(' | '[' | '{' => {
                depth += 1;
                current.push(ch);
            }
            ')' | ']' | '}' => {
                depth -= 1;
                current.push(ch);
            }
            ',' if depth == 0 => {
                let arg = current.trim().to_string();
                if !arg.is_empty() {
                    args.push(arg);
                }
                current.clear();
            }
            _ => current.push(ch),
        }
    }

    // Don't forget the last argument
    let arg = current.trim().to_string();
    if !arg.is_empty() {
        args.push(arg);
    }

    args
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_call_expr_simple() {
        let (name, args) = extract_call_expr("foo()").unwrap();
        assert_eq!(name, "foo");
        assert!(args.is_empty());
    }

    #[test]
    fn test_extract_call_expr_with_args() {
        let (name, args) = extract_call_expr("foo(1, 2, 3)").unwrap();
        assert_eq!(name, "foo");
        assert_eq!(args, vec!["1", "2", "3"]);
    }

    #[test]
    fn test_extract_call_expr_with_string() {
        let (name, args) = extract_call_expr(r#"greet("World")"#).unwrap();
        assert_eq!(name, "greet");
        assert_eq!(args, vec![r#""World""#]);
    }

    #[test]
    fn test_extract_call_expr_with_whitespace() {
        let (name, args) = extract_call_expr("  foo(  1  ,  2  )  ").unwrap();
        assert_eq!(name, "foo");
        assert_eq!(args, vec!["1", "2"]);
    }

    #[test]
    fn test_extract_call_expr_not_call() {
        assert!(extract_call_expr("foo").is_none());
        assert!(extract_call_expr("1 + 2").is_none());
        assert!(extract_call_expr("obj.method()").is_none()); // contains dot
    }

    #[test]
    fn test_parse_args_nested() {
        let args = parse_args("1, foo(2, 3), 4");
        assert_eq!(args, vec!["1", "foo(2, 3)", "4"]);
    }
}
