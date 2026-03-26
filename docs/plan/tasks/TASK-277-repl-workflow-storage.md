# TASK-277: REPL Workflow Definition Storage

## Status: 📝 Planned

## Description

Fix the REPL issue where workflow definitions entered at the prompt are not stored in REPL session state. Currently, entering a workflow only parses it and returns Null without storing any binding, breaking SPEC-011's workflow-definition session model.

## Specification Reference

- SPEC-011: REPL Specification - Section 3.2 (Workflow Definition)

## Dependencies

- ✅ TASK-077: Create ash-repl crate
- ✅ TASK-078: Expression evaluation in REPL

## Requirements

### Functional Requirements

1. REPL must store workflow definitions in session state
2. Stored workflows must be callable by name later in the session
3. Workflow redefinition must update the stored binding
4. Type checking must occur at definition time (not call time)
5. `ash run` equivalent must work for stored workflows

### Current State (Broken)

**File:** `crates/ash-repl/src/session.rs`

```rust
pub fn evaluate(&mut self, input: &str) -> Result<EvalResult, ReplError> {
    // Try to parse as workflow
    if let Ok(workflow) = parse_workflow(input) {
        // Only parses - never stores!
        return Ok(EvalResult::Value(Value::Null));
    }
    
    // Try to parse as expression
    if let Ok(expr) = parse_expr(input) {
        let value = self.eval_expr(&expr)?;
        return Ok(EvalResult::Value(value));
    }
    
    Err(ReplError::ParseError)
}
```

### Target State (Fixed)

```rust
pub struct SessionState {
    // existing fields...
    workflows: HashMap<String, CompiledWorkflow>,
    type_checker: TypeChecker,
}

pub struct CompiledWorkflow {
    name: String,
    workflow: Workflow,
    verified_type: Type,
}

impl SessionState {
    pub fn evaluate(&mut self, input: &str) -> Result<EvalResult, ReplError> {
        // Try to parse as workflow
        if let Ok(workflow) = parse_workflow(input) {
            return self.define_workflow(workflow);
        }
        
        // Try expression evaluation (may reference stored workflows)
        if let Ok(expr) = parse_expr(input) {
            let value = self.eval_expr(&expr)?;
            return Ok(EvalResult::Value(value));
        }
        
        Err(ReplError::ParseError)
    }
    
    fn define_workflow(&mut self, workflow: Workflow) -> Result<EvalResult, ReplError> {
        // Type check at definition time
        let verified_type = self.type_checker.type_check_workflow(&workflow)
            .map_err(|e| ReplError::TypeError(e))?;
        
        let name = workflow.name.clone();
        let compiled = CompiledWorkflow {
            name: name.clone(),
            workflow,
            verified_type,
        };
        
        // Store in session
        self.workflows.insert(name.clone(), compiled);
        
        Ok(EvalResult::WorkflowDefined { name })
    }
    
    pub fn run_workflow(&self, name: &str, input: Value) -> Result<Value, ReplError> {
        let compiled = self.workflows.get(name)
            .ok_or_else(|| ReplError::UnknownWorkflow { name: name.to_string() })?;
        
        // Execute without re-type-checking
        self.engine.execute(&compiled.workflow, input)
            .map_err(|e| ReplError::ExecutionError(e))
    }
}
```

## TDD Steps

### Step 1: Write Tests (Red)

**File:** `crates/ash-repl/tests/workflow_storage_test.rs`

```rust
//! Tests for REPL workflow storage

use ash_repl::Session;

#[test]
fn test_workflow_definition_stored() {
    let mut session = Session::new();
    
    // Define a workflow
    let result = session.evaluate(r#"
        workflow greet(name) {
            act print("Hello, " + name);
        }
    "#);
    
    assert!(matches!(result, Ok(EvalResult::WorkflowDefined { name })
        if name == "greet"));
    
    // Workflow should be callable
    let result = session.evaluate("greet(\"World\")");
    assert!(result.is_ok());
}

#[test]
fn test_workflow_redefinition_updates() {
    let mut session = Session::new();
    
    // First definition
    session.evaluate(r#"
        workflow test() { act print("v1"); }
    "#).unwrap();
    
    // Redefinition
    session.evaluate(r#"
        workflow test() { act print("v2"); }
    "#).unwrap();
    
    // Should use v2
    // (would need capture mechanism to verify)
}

#[test]
fn test_undefined_workflow_error() {
    let mut session = Session::new();
    
    let result = session.evaluate("undefined_workflow()");
    
    assert!(matches!(result, Err(ReplError::UnknownWorkflow { .. })));
}

#[test]
fn test_workflow_type_checked_at_definition() {
    let mut session = Session::new();
    
    let result = session.evaluate(r#"
        workflow bad(x) {
            act print(x + "string");  // Type error
        }
    "#);
    
    // Should fail at definition time
    assert!(matches!(result, Err(ReplError::TypeError { .. })));
}

#[test]
fn test_stored_workflow_persists_across_inputs() {
    let mut session = Session::new();
    
    // Define workflow
    session.evaluate(r#"
        workflow add(a, b) {
            decide a + b
        }
    "#).unwrap();
    
    // Use in expression
    let result = session.evaluate("add(2, 3)");
    assert_eq!(result.unwrap(), EvalResult::Value(Value::Int(5)));
}
```

### Step 2: Add Workflow Storage to Session

**File:** `crates/ash-repl/src/session.rs`

```rust
use std::collections::HashMap;
use ash_core::{Workflow, Type, Value};
use ash_typeck::TypeChecker;
use ash_engine::Engine;

pub struct CompiledWorkflow {
    pub name: String,
    pub workflow: Workflow,
    pub verified_type: Type,
}

pub struct SessionState {
    engine: Engine,
    type_checker: TypeChecker,
    workflows: HashMap<String, CompiledWorkflow>,
    bindings: HashMap<String, Value>,
}

impl SessionState {
    pub fn new() -> Self {
        Self {
            engine: Engine::new(),
            type_checker: TypeChecker::new(),
            workflows: HashMap::new(),
            bindings: HashMap::new(),
        }
    }
}
```

### Step 3: Implement Workflow Definition Handler

**File:** `crates/ash-repl/src/eval.rs`

```rust
use ash_parser::{parse_workflow, parse_expr};

pub enum EvalResult {
    Value(Value),
    WorkflowDefined { name: String },
    Type { ty: Type },
    Unit,
}

impl SessionState {
    pub fn evaluate(&mut self, input: &str) -> Result<EvalResult, ReplError> {
        // Trim and normalize input
        let input = input.trim();
        
        // Try workflow definition first
        if input.starts_with("workflow ") {
            let workflow = parse_workflow(input)?;
            return self.define_workflow(workflow);
        }
        
        // Try expression
        let expr = parse_expr(input)?;
        self.eval_expr(&expr)
    }
    
    fn define_workflow(&mut self, workflow: Workflow) -> Result<EvalResult, ReplError> {
        // Type check at definition time
        let verified_type = self.type_checker
            .type_check_workflow(&workflow)
            .map_err(ReplError::TypeError)?;
        
        let name = workflow.name.clone();
        
        let compiled = CompiledWorkflow {
            name: name.clone(),
            workflow,
            verified_type,
        };
        
        self.workflows.insert(name.clone(), compiled);
        
        Ok(EvalResult::WorkflowDefined { name })
    }
}
```

### Step 4: Support Workflow Invocation

**File:** `crates/ash-repl/src/eval.rs`

```rust
impl SessionState {
    fn eval_expr(&mut self, expr: &Expr) -> Result<EvalResult, ReplError> {
        match expr {
            Expr::Call { callee, args } => {
                if let Expr::Variable(name) = callee.as_ref() {
                    // Check if it's a stored workflow
                    if let Some(workflow) = self.workflows.get(name) {
                        return self.call_workflow(workflow, args);
                    }
                }
                
                // Regular function call
                self.eval_regular_call(callee, args)
            }
            // ... other expression types
        }
    }
    
    fn call_workflow(
        &self,
        workflow: &CompiledWorkflow,
        args: &[Expr],
    ) -> Result<EvalResult, ReplError> {
        // Evaluate arguments
        let arg_values: Vec<Value> = args.iter()
            .map(|arg| self.eval_to_value(arg))
            .collect::<Result<_, _>>()?;
        
        // Convert to workflow input
        let input = if arg_values.len() == 1 {
            arg_values.into_iter().next().unwrap()
        } else {
            Value::Tuple(arg_values)
        };
        
        // Execute
        let result = self.engine
            .execute(&workflow.workflow, input)
            .map_err(ReplError::ExecutionError)?;
        
        Ok(EvalResult::Value(result.value))
    }
}
```

### Step 5: Update REPL Display

**File:** `crates/ash-repl/src/repl.rs`

```rust
fn display_result(&self, result: &EvalResult) {
    match result {
        EvalResult::Value(v) => println!("{}", v),
        EvalResult::WorkflowDefined { name } => {
            println!("workflow '{}' defined", name);
        }
        EvalResult::Type { ty } => println!("type: {}", ty),
        EvalResult::Unit => {}
    }
}
```

## Verification Steps

- [ ] `cargo test -p ash-repl --test workflow_storage_test` passes
- [ ] `cargo test -p ash-repl` all tests pass
- [ ] Manual test: define workflow in REPL, call it later
- [ ] `cargo clippy --all-targets --all-features` clean
- [ ] `cargo fmt --check` clean

## Dependencies for Next Task

This task outputs:
- Working workflow storage in REPL
- SPEC-011 compliance for workflow definitions

Required by:
- TASK-278: CLI --input (REPL workflow execution model)

## Notes

**Spec Compliance**: SPEC-011 Section 3.2 requires that "workflow definitions entered at the REPL prompt are stored in the session state and can be invoked by name in subsequent expressions."

**Design Decision**: Type check at definition time to fail fast and avoid repeated type checking on each call.

**Edge Cases**:
- Recursive workflows - must handle forward references
- Workflow shadows binding - workflow takes precedence
- Invalid workflow syntax - proper error message
