//! Session state management for the REPL.
//!
//! This module provides the `Session` type for managing REPL session state,
//! including workflow definitions that can be stored and invoked by name.

use std::collections::HashMap;

use ash_core::Value;
use ash_engine::{Engine, Workflow as EngineWorkflow};

use crate::ReplError;

/// A compiled workflow stored in the session.
///
/// Contains the workflow definition, its name, and the verified type
/// from type checking at definition time.
#[derive(Debug, Clone)]
pub struct CompiledWorkflow {
    /// The name of the workflow
    pub name: String,
    /// The compiled workflow ready for execution
    pub workflow: EngineWorkflow,
    /// The verified type string representation
    pub verified_type: String,
    /// The parameter names for this workflow
    pub params: Vec<String>,
    /// The original source code for the workflow body
    pub body_source: String,
}

/// The result of evaluating input in the REPL.
#[derive(Debug, Clone, PartialEq)]
pub enum EvalResult {
    /// A computed value
    Value(Value),
    /// A workflow was successfully defined
    WorkflowDefined {
        /// The name of the defined workflow
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
/// - Stored workflow definitions
/// - Variable bindings
/// - The execution engine
#[derive(Debug)]
pub struct Session {
    engine: Engine,
    workflows: HashMap<String, CompiledWorkflow>,
    bindings: HashMap<String, Value>,
}

impl Session {
    /// Create a new REPL session with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            engine: Engine::default(),
            workflows: HashMap::new(),
            bindings: HashMap::new(),
        }
    }

    /// Evaluate input in the session context.
    ///
    /// The input can be either:
    /// - A workflow definition (starts with "workflow")
    /// - An expression that may reference stored workflows
    ///
    /// # Errors
    ///
    /// Returns `ReplError` if parsing, type checking, or execution fails.
    pub async fn evaluate(&mut self, input: &str) -> Result<EvalResult, ReplError> {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return Ok(EvalResult::Unit);
        }

        // Try workflow definition first
        if trimmed.starts_with("workflow ") {
            return self.define_workflow(trimmed);
        }

        // Check if this looks like a workflow call: name(...)
        if let Some((call_name, args)) = extract_call_expr(trimmed)
            && self.workflows.contains_key(call_name)
        {
            return self.call_workflow_by_name(call_name, &args).await;
        }

        // Treat as expression - wrap in a workflow and execute
        let wrapped = format!("workflow __repl__ {{ ret {trimmed}; }}");
        let workflow = self.engine.parse(&wrapped)?;
        self.engine.check(&workflow)?;
        let result = self.engine.execute(&workflow).await?;

        Ok(EvalResult::Value(result))
    }

    /// Define a workflow and store it in the session.
    ///
    /// Type checks the workflow at definition time (fail-fast).
    fn define_workflow(&mut self, source: &str) -> Result<EvalResult, ReplError> {
        use ash_parser::parse_workflow::workflow_def;
        use winnow::prelude::*;

        // Parse to extract name and params
        let mut input = ash_parser::new_input(source);
        let def = workflow_def
            .parse_next(&mut input)
            .map_err(|e| ReplError::ParseError(format!("{e}")))?;

        let name = def.name.to_string();
        let params: Vec<String> = def.params.iter().map(|p| p.name.to_string()).collect();

        // Extract body source (approximate - use the original source)
        // Find the opening brace and extract from there
        let body_start = source
            .find('{')
            .ok_or_else(|| ReplError::ParseError("workflow must have a body block".to_string()))?;
        let body_source = source[body_start..].to_string();

        // Now parse with the engine to get the compiled workflow
        let workflow = self.engine.parse(source)?;

        // Type check at definition time
        self.engine.check(&workflow)?;

        // For now, store a simple type representation
        let verified_type = format!("Workflow({})", params.join(", "));

        let compiled = CompiledWorkflow {
            name: name.clone(),
            workflow,
            verified_type,
            params,
            body_source,
        };

        // Store in session (redefinition updates existing)
        self.workflows.insert(name.clone(), compiled);

        Ok(EvalResult::WorkflowDefined { name })
    }

    /// Call a stored workflow by name with arguments.
    async fn call_workflow_by_name(
        &self,
        name: &str,
        args: &[String],
    ) -> Result<EvalResult, ReplError> {
        let compiled = self
            .workflows
            .get(name)
            .ok_or_else(|| ReplError::UnknownWorkflow {
                name: name.to_string(),
            })?;

        // Create a wrapper workflow that binds arguments to parameters
        let wrapper_source = if compiled.params.is_empty() {
            // No parameters - just call the workflow directly
            format!("workflow __call__ {{ {} }}", compiled.body_source)
        } else {
            // Check argument count
            if args.len() != compiled.params.len() {
                return Err(ReplError::Engine(format!(
                    "workflow '{}' expects {} arguments, got {}",
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
                "workflow __call__ {{ {} {} }}",
                bindings.join(" "),
                compiled.body_source
            )
        };

        // Parse, check, and execute the wrapper
        let workflow = self.engine.parse(&wrapper_source)?;
        self.engine.check(&workflow)?;
        let result = self.engine.execute(&workflow).await?;

        Ok(EvalResult::Value(result))
    }

    /// Run a stored workflow with the given input value.
    ///
    /// # Errors
    ///
    /// Returns `ReplError::UnknownWorkflow` if the workflow is not found.
    pub async fn run_workflow(&self, name: &str) -> Result<Value, ReplError> {
        let compiled = self
            .workflows
            .get(name)
            .ok_or_else(|| ReplError::UnknownWorkflow {
                name: name.to_string(),
            })?;

        // Execute without re-type-checking
        let result = self.engine.execute(&compiled.workflow).await?;

        Ok(result)
    }

    /// Get a reference to a stored workflow.
    #[must_use]
    pub fn get_workflow(&self, name: &str) -> Option<&CompiledWorkflow> {
        self.workflows.get(name)
    }

    /// Check if a workflow is defined in this session.
    #[must_use]
    pub fn has_workflow(&self, name: &str) -> bool {
        self.workflows.contains_key(name)
    }

    /// Get the names of all defined workflows.
    pub fn workflow_names(&self) -> impl Iterator<Item = &String> {
        self.workflows.keys()
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
