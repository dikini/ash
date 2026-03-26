# TASK-294: Implement REPL Workflow Definition Storage

## Status: 📝 Planned

## Description

Fix the critical issue where the REPL cannot support the spec's workflow-definition session model. Entering `workflow ...` only parses it and returns Null; no session binding is stored. The SPEC-011 flow of defining `workflow test { ... }` and later invoking `test` is not implemented.

## Specification Reference

- SPEC-011: REPL Specification
- SPEC-001: Core IR Specification

## Dependencies

- ✅ TASK-077: REPL crate
- ✅ TASK-080: REPL commands
- ✅ TASK-172: Unify REPL implementation

## Critical File Locations

- `crates/ash-repl/src/lib.rs:386` - workflow parsed but not stored
- `crates/ash-repl/src/lib.rs:395` - returns Null, no binding created

## Requirements

### Functional Requirements

1. Workflow definitions must be stored in REPL session
2. Stored workflows must be invocable by name
3. Workflow parameters must be supported at invocation
4. Workflows must persist for the REPL session
5. SPEC-011 session model must be supported

### Current State (Broken)

**File:** `crates/ash-repl/src/lib.rs:386-395`

```rust
fn handle_input(&mut self, input: &str) -> ReplResult {
    // Try to parse as workflow definition
    if let Ok(workflow) = self.parser.parse_workflow(input) {
        // Parse succeeds but workflow is discarded!
        println!("Defined workflow: {}", workflow.name);
        return Ok(Value::Null);  // Line 395: No storage!
    }
    
    // ... other input handling
}
```

Problems:
1. Workflow is parsed but not stored
2. Cannot invoke workflow by name
3. Session model not supported
4. SPEC-011 workflow definition flow broken

### Target State (Fixed)

```rust
pub struct Repl {
    // ... existing fields ...
    
    // FIX: Store defined workflows in session
    defined_workflows: HashMap<String, Workflow>,
    
    // Type checker for workflow validation
    type_checker: TypeChecker,
}

impl Repl {
    fn handle_input(&mut self, input: &str) -> ReplResult {
        // Try to parse as workflow definition first
        match self.parser.parse_workflow(input) {
            Ok(workflow) => {
                // FIX: Type check the workflow
                if let Err(e) = self.type_checker.type_check_workflow(&workflow) {
                    return Err(ReplError::TypeError(e));
                }
                
                // FIX: Store the workflow
                let name = workflow.name.clone();
                self.defined_workflows.insert(name.clone(), workflow);
                
                println!("Defined workflow: {}", name);
                Ok(Value::Null)
            }
            Err(ParseError::NotAWorkflow) => {
                // Not a workflow definition, try other forms
                self.handle_expression_or_invocation(input)
            }
            Err(e) => Err(e.into()),
        }
    }
    
    fn handle_expression_or_invocation(&mut self, input: &str) -> ReplResult {
        // Check if input is a workflow invocation
        if let Some(invocation) = self.try_parse_invocation(input) {
            return self.execute_invocation(invocation);
        }
        
        // Otherwise handle as expression
        self.evaluate_expression(input)
    }
    
    fn try_parse_invocation(&self, input: &str) -> Option<WorkflowInvocation> {
        // Parse patterns like:
        // - "test"
        // - "test(1, 2)"
        // - "test arg1 arg2"
        
        let trimmed = input.trim();
        
        // Try "name(args)" syntax
        if let Some(caps) = self.invocation_regex.captures(trimmed) {
            let name = caps.get(1)?.as_str().to_string();
            if self.defined_workflows.contains_key(&name) {
                let args_str = caps.get(2)?.as_str();
                let args = self.parse_args(args_str).ok()?;
                return Some(WorkflowInvocation { name, args });
            }
        }
        
        // Try bare name syntax
        if self.defined_workflows.contains_key(trimmed) {
            return Some(WorkflowInvocation {
                name: trimmed.to_string(),
                args: vec![],
            });
        }
        
        None
    }
    
    fn execute_invocation(&mut self, invocation: WorkflowInvocation) -> ReplResult {
        let workflow = self.defined_workflows.get(&invocation.name)
            .ok_or_else(|| ReplError::UndefinedWorkflow(invocation.name.clone()))?;
        
        // Bind arguments to parameters
        let mut ctx = Context::new();
        for (param, arg) in workflow.params.iter().zip(invocation.args.iter()) {
            let value = self.evaluate_expression(arg)?;
            ctx.bind(param.name.clone(), value);
        }
        
        // Execute the workflow
        let result = self.engine.execute_workflow(workflow, ctx)?;
        
        Ok(result)
    }
}

/// Represents a workflow invocation parsed from input
struct WorkflowInvocation {
    name: String,
    args: Vec<String>,
}
```

## TDD Steps

### Step 1: Write Tests (Red)

**File:** `crates/ash-repl/tests/workflow_definition_test.rs`

```rust
//! Tests for REPL workflow definition and invocation

use ash_repl::Repl;

#[test]
fn test_define_workflow_stores_in_session() {
    let mut repl = Repl::new();
    
    // Define a workflow
    let result = repl.execute(r#"
        workflow greet(name: String) {
            act print("Hello, " + name);
        }
    "#);
    
    assert!(result.is_ok());
    
    // Invoke the stored workflow
    let result = repl.execute(r#"greet("World")"#);
    assert!(result.is_ok());
    
    // Check output
    assert!(repl.output_contains("Hello, World"));
}

#[test]
fn test_workflow_persists_across_inputs() {
    let mut repl = Repl::new();
    
    // Define workflow
    repl.execute(r#"
        workflow add(x: Int, y: Int) -> Int {
            x + y
        }
    "#).unwrap();
    
    // Invoke multiple times
    let r1 = repl.execute("add(1, 2)").unwrap();
    assert_eq!(r1, Value::Int(3));
    
    let r2 = repl.execute("add(5, 3)").unwrap();
    assert_eq!(r2, Value::Int(8));
}

#[test]
fn test_multiple_workflows_in_session() {
    let mut repl = Repl::new();
    
    repl.execute(r#"
        workflow greet(name: String) {
            act print("Hello, " + name);
        }
    "#).unwrap();
    
    repl.execute(r#"
        workflow farewell(name: String) {
            act print("Goodbye, " + name);
        }
    "#).unwrap();
    
    // Both should be invocable
    repl.execute(r#"greet("Alice")"#).unwrap();
    assert!(repl.output_contains("Hello, Alice"));
    
    repl.execute(r#"farewell("Bob")"#).unwrap();
    assert!(repl.output_contains("Goodbye, Bob"));
}

#[test]
fn test_undefined_workflow_error() {
    let mut repl = Repl::new();
    
    // Try to invoke undefined workflow
    let result = repl.execute("undefined_workflow()");
    
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("undefined workflow"));
    assert!(err.contains("undefined_workflow"));
}

#[test]
fn test_workflow_type_checking_on_definition() {
    let mut repl = Repl::new();
    
    // Define workflow with type error
    let result = repl.execute(r#"
        workflow bad(x: Int) {
            act print(x + "string");  // Type error: Int + String
        }
    "#);
    
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("type error"));
    
    // Workflow should NOT be stored if type check fails
    let invoke_result = repl.execute("bad(42)");
    assert!(invoke_result.is_err());  // Undefined
}

#[test]
fn test_workflow_redefinition_allowed() {
    let mut repl = Repl::new();
    
    // Define workflow
    repl.execute(r#"
        workflow compute(x: Int) -> Int {
            x * 2
        }
    "#).unwrap();
    
    let r1 = repl.execute("compute(5)").unwrap();
    assert_eq!(r1, Value::Int(10));
    
    // Redefine with different body
    repl.execute(r#"
        workflow compute(x: Int) -> Int {
            x + 100  // Changed from * 2
        }
    "#).unwrap();
    
    let r2 = repl.execute("compute(5)").unwrap();
    assert_eq!(r2, Value::Int(105));
}

#[test]
fn test_workflow_parameter_count_validation() {
    let mut repl = Repl::new();
    
    repl.execute(r#"
        workflow add(x: Int, y: Int) -> Int {
            x + y
        }
    "#).unwrap();
    
    // Too few arguments
    let result = repl.execute("add(1)");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("argument"));
    
    // Too many arguments
    let result = repl.execute("add(1, 2, 3)");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("argument"));
}

#[test]
fn test_bare_workflow_name_invocation() {
    let mut repl = Repl::new();
    
    repl.execute(r#"
        workflow hello {
            act print("Hello!");
        }
    "#).unwrap();
    
    // Invoke without parentheses
    let result = repl.execute("hello");
    assert!(result.is_ok());
    assert!(repl.output_contains("Hello!"));
}

#[test]
fn test_workflow_listing() {
    let mut repl = Repl::new();
    
    repl.execute(r#"
        workflow one { act print("one"); }
    "#).unwrap();
    
    repl.execute(r#"
        workflow two { act print("two"); }
    "#).unwrap();
    
    // List defined workflows
    repl.execute(":workflows").unwrap();
    
    assert!(repl.output_contains("one"));
    assert!(repl.output_contains("two"));
}
```

### Step 2: Add Workflow Storage to Repl

**File:** `crates/ash-repl/src/lib.rs`

```rust
use std::collections::HashMap;
use ash_engine::{Workflow, Engine};
use ash_typeck::TypeChecker;
use regex::Regex;

pub struct Repl {
    engine: Engine,
    type_checker: TypeChecker,
    parser: Parser,
    history: Vec<String>,
    
    // FIX: Store defined workflows
    defined_workflows: HashMap<String, Workflow>,
    
    // For parsing invocations
    invocation_regex: Regex,
}

impl Repl {
    pub fn new() -> Self {
        Self {
            engine: Engine::new(),
            type_checker: TypeChecker::new(),
            parser: Parser::new(),
            history: vec![],
            defined_workflows: HashMap::new(),
            invocation_regex: Regex::new(r"^(\w+)\s*\((.*)\)$").unwrap(),
        }
    }
    
    pub fn execute(&mut self, input: &str) -> ReplResult {
        self.history.push(input.to_string());
        
        // Try workflow definition first
        if let Some(workflow) = self.try_define_workflow(input)? {
            return Ok(workflow);
        }
        
        // Try workflow invocation
        if let Some(result) = self.try_invoke_workflow(input)? {
            return Ok(result);
        }
        
        // Handle as expression
        self.evaluate_expression(input)
    }
    
    fn try_define_workflow(&mut self, input: &str) -> Option<ReplResult> {
        match self.parser.parse_workflow(input) {
            Ok(workflow) => {
                // Type check before storing
                if let Err(e) = self.type_checker.type_check_workflow(&workflow) {
                    return Some(Err(ReplError::TypeError(e)));
                }
                
                let name = workflow.name.clone();
                self.defined_workflows.insert(name.clone(), workflow);
                
                println!("Defined workflow: {}", name);
                Some(Ok(Value::Null))
            }
            Err(_) => None,  // Not a workflow definition
        }
    }
    
    fn try_invoke_workflow(&mut self, input: &str) -> Option<ReplResult> {
        let trimmed = input.trim();
        
        // Try "name(args)" syntax
        if let Some(caps) = self.invocation_regex.captures(trimmed) {
            let name = caps.get(1)?.as_str();
            if let Some(workflow) = self.defined_workflows.get(name) {
                let args_str = caps.get(2)?.as_str();
                return Some(self.execute_workflow(workflow, args_str));
            }
        }
        
        // Try bare name syntax
        if let Some(workflow) = self.defined_workflows.get(trimmed) {
            return Some(self.execute_workflow(workflow, ""));
        }
        
        None
    }
    
    fn execute_workflow(&self, workflow: &Workflow, args_str: &str) -> ReplResult {
        // Parse and evaluate arguments
        let args: Vec<Value> = if args_str.trim().is_empty() {
            vec![]
        } else {
            args_str.split(',')
                .map(|s| self.evaluate_expression(s.trim()))
                .collect::<Result<Vec<_>, _>>()?
        };
        
        // Validate argument count
        if args.len() != workflow.params.len() {
            return Err(ReplError::ArgumentCountMismatch {
                workflow: workflow.name.clone(),
                expected: workflow.params.len(),
                actual: args.len(),
            });
        }
        
        // Bind arguments to parameters
        let mut ctx = Context::new();
        for (param, arg) in workflow.params.iter().zip(args.iter()) {
            ctx.bind(param.name.clone(), arg.clone());
        }
        
        // Execute
        self.engine.execute_with_context(workflow, ctx)
            .map_err(|e| ReplError::Execution(e))
    }
}
```

### Step 3: Add :workflows Command

**File:** `crates/ash-repl/src/commands.rs`

```rust
impl Repl {
    fn handle_command(&mut self, cmd: &str, args: &str) -> ReplResult {
        match cmd {
            // ... existing commands ...
            
            "workflows" => {
                // List all defined workflows
                if self.defined_workflows.is_empty() {
                    println!("No workflows defined.");
                } else {
                    println!("Defined workflows:");
                    for (name, workflow) in &self.defined_workflows {
                        let params = if workflow.params.is_empty() {
                            String::new()
                        } else {
                            format!("({})", 
                                workflow.params.iter()
                                    .map(|p| format!("{}: {:?}", p.name, p.ty))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            )
                        };
                        println!("  {}{}", name, params);
                    }
                }
                Ok(Value::Null)
            }
            
            _ => Err(ReplError::UnknownCommand(cmd.to_string())),
        }
    }
}
```

## Verification Steps

- [ ] `cargo test -p ash-repl --test workflow_definition_test` passes
- [ ] Workflows can be defined and invoked
- [ ] Multiple workflows persist in session
- [ ] Type errors prevent workflow storage
- [ ] `:workflows` command lists defined workflows
- [ ] `cargo clippy --all-targets --all-features` clean
- [ ] `cargo fmt --check` clean

## Dependencies for Next Task

This task outputs:
- Working REPL workflow session model
- SPEC-011 compliance for workflow definition

Required by:
- Interactive workflow development
- REPL as primary development tool

## Notes

**Critical Issue**: REPL workflow definition is non-functional despite being in SPEC-011.

**Risk Assessment**: High - core REPL feature broken.

**Implementation Strategy**:
1. First: Add workflow storage to Repl struct
2. Second: Implement definition parsing
3. Third: Implement invocation
4. Fourth: Add :workflows command
