# TASK-288: Fix REPL :ast Command Output

## Status: ✅ Complete

## Description

Fix the REPL `:ast` command which currently prints engine/workflow debug output rather than the requested expression AST shape. For expressions it wraps input in a synthetic workflow, so users see implementation scaffolding instead of the AST contract described in SPEC-011.

## Specification Reference

- SPEC-011: REPL Specification
- SPEC-002: Surface Syntax Specification
- SPEC-001: Core IR Specification

## Dependencies

- ✅ TASK-077: REPL crate
- ✅ TASK-080: REPL commands
- ✅ TASK-172: Unify REPL implementation
- ✅ TASK-173: REPL type reporting

## Critical File Locations

- `crates/ash-repl/src/lib.rs:494` - :ast command implementation

## Requirements

### Functional Requirements

1. `:ast` must display the parsed AST for the input expression
2. `:ast` must not wrap expressions in synthetic workflows
3. `:ast` output must match the SPEC-011 contract
4. `:ast` must show surface AST, not internal/core representation
5. `:ast` must handle both expressions and workflow definitions correctly

### Current State (Broken)

**File:** `crates/ash-repl/src/lib.rs:494`

```rust
fn handle_ast_command(&mut self, input: &str) -> ReplResult {
    // Wraps input in synthetic workflow - WRONG!
    let wrapped = format!("workflow __repl_temp__ {{ {} }}", input);
    
    let workflow = parse(&wrapped)?;
    
    // Prints engine debug representation - WRONG!
    println!("{:?}", workflow);
    
    Ok(())
}
```

Problems:
1. Wraps expressions in synthetic workflows
2. Shows debug representation with implementation details
3. Doesn't show the actual expression AST
4. Users see `Workflow { steps: [...] }` instead of expression structure

### Target State (Fixed)

```rust
fn handle_ast_command(&mut self, input: &str) -> ReplResult {
    // Try to parse as expression first
    match parse_expression(input) {
        Ok(expr) => {
            // Print expression AST per SPEC-011
            println!("{}", format_expr_ast(&expr));
            return Ok(());
        }
        Err(ParseError::NotAnExpression) => {
            // Fall through to workflow parsing
        }
        Err(e) => return Err(e.into()),
    }
    
    // Try to parse as workflow
    match parse_workflow(input) {
        Ok(workflow) => {
            // Print workflow AST per SPEC-011
            println!("{}", format_workflow_ast(&workflow));
        }
        Err(e) => return Err(e.into()),
    }
    
    Ok(())
}
```

## TDD Steps

### Step 1: Write Tests (Red)

**File:** `crates/ash-repl/tests/ast_command_test.rs`

```rust
//! Tests for REPL :ast command

use ash_repl::Repl;

#[test]
fn test_ast_simple_expression() {
    let mut repl = Repl::new();
    
    // Input a simple expression
    let output = repl.execute_command(":ast", "1 + 2");
    
    // Should show expression AST, not wrapped workflow
    assert!(output.contains("BinaryExpr"));
    assert!(output.contains("left: IntLiteral(1)"));
    assert!(output.contains("op: Add"));
    assert!(output.contains("right: IntLiteral(2)"));
    
    // Should NOT show workflow wrapper
    assert!(!output.contains("workflow"));
    assert!(!output.contains("__repl_temp__"));
    assert!(!output.contains("Workflow {"));
}

#[test]
fn test_ast_variable() {
    let mut repl = Repl::new();
    
    let output = repl.execute_command(":ast", "x");
    
    assert!(output.contains("Variable"));
    assert!(output.contains("name: \"x\""));
    assert!(!output.contains("workflow"));
}

#[test]
fn test_ast_function_call() {
    let mut repl = Repl::new();
    
    let output = repl.execute_command(":ast", "foo(1, 2)");
    
    assert!(output.contains("CallExpr"));
    assert!(output.contains("function: \"foo\""));
    assert!(output.contains("arguments:"));
    assert!(!output.contains("workflow"));
}

#[test]
fn test_ast_match_expression() {
    let mut repl = Repl::new();
    
    let output = repl.execute_command(":ast", "match x { Some(v) => v, None => 0 }");
    
    assert!(output.contains("MatchExpr"));
    assert!(output.contains("MatchArm"));
    assert!(output.contains("pattern:"));
    assert!(!output.contains("workflow"));
}

#[test]
fn test_ast_workflow_definition() {
    let mut repl = Repl::new();
    
    let output = repl.execute_command(":ast", "workflow test { act log(\"hello\") }");
    
    // Should show workflow AST when input IS a workflow
    assert!(output.contains("WorkflowDef"));
    assert!(output.contains("name: \"test\""));
    assert!(output.contains("Step::Act"));
}

#[test]
fn test_ast_nested_expression() {
    let mut repl = Repl::new();
    
    let output = repl.execute_command(":ast", "(1 + 2) * 3");
    
    assert!(output.contains("BinaryExpr"));
    // Should show proper nesting
    assert!(output.contains("BinaryExpr"));
    assert!(output.contains("op: Mul"));
}

#[test]
fn test_ast_literal_types() {
    let mut repl = Repl::new();
    
    let int_out = repl.execute_command(":ast", "42");
    assert!(int_out.contains("IntLiteral(42)"));
    
    let string_out = repl.execute_command(":ast", "\"hello\"");
    assert!(string_out.contains("StringLiteral(\"hello\")"));
    
    let bool_out = repl.execute_command(":ast", "true");
    assert!(bool_out.contains("BoolLiteral(true)"));
}

#[test]
fn test_ast_no_debug_repr() {
    let mut repl = Repl::new();
    
    let output = repl.execute_command(":ast", "1 + 1");
    
    // Should NOT contain Rust debug representation artifacts
    assert!(!output.contains("Expr {"));
    assert!(!output.contains("span: Span"));
    assert!(!output.contains("id: NodeId"));
}

#[test]
fn test_ast_pretty_printed() {
    let mut repl = Repl::new();
    
    let output = repl.execute_command(":ast", "if true { 1 } else { 2 }");
    
    // Should be pretty-printed with indentation
    assert!(output.contains("IfExpr"));
    // Check for indentation
    let lines: Vec<&str> = output.lines().collect();
    assert!(lines.iter().any(|l| l.starts_with("  ")));
}

proptest! {
    #[test]
    fn ast_output_is_deterministic(expr in expr_strategy()) {
        // Property: same input produces same AST output
        let output1 = format_expr_ast(&expr);
        let output2 = format_expr_ast(&expr);
        assert_eq!(output1, output2);
    }
}
```

### Step 2: Create AST Formatter

**File:** `crates/ash-repl/src/ast_formatter.rs`

```rust
//! Pretty printer for AST nodes per SPEC-011

use ash_parser::surface::{Expr, Workflow, Step, Pattern};
use std::fmt::Write;

pub struct AstFormatter {
    indent: usize,
    output: String,
}

impl AstFormatter {
    pub fn new() -> Self {
        Self {
            indent: 0,
            output: String::new(),
        }
    }
    
    pub fn format_expr(expr: &Expr) -> String {
        let mut formatter = Self::new();
        formatter.write_expr(expr);
        formatter.output
    }
    
    pub fn format_workflow(workflow: &Workflow) -> String {
        let mut formatter = Self::new();
        formatter.write_workflow(workflow);
        formatter.output
    }
    
    fn write_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::IntLiteral(n) => {
                self.write_line(&format!("IntLiteral({})", n));
            }
            Expr::StringLiteral(s) => {
                self.write_line(&format!("StringLiteral(\"{}\")", s));
            }
            Expr::BoolLiteral(b) => {
                self.write_line(&format!("BoolLiteral({})", b));
            }
            Expr::Variable { name, .. } => {
                self.write_line(&format!("Variable {{"));
                self.indent += 2;
                self.write_line(&format!("name: \"{}\"", name));
                self.indent -= 2;
                self.write_line("}");
            }
            Expr::Binary { left, op, right, .. } => {
                self.write_line("BinaryExpr {");
                self.indent += 2;
                self.write_line("left:");
                self.indent += 2;
                self.write_expr(left);
                self.indent -= 2;
                self.write_line(&format!("op: {:?}", op));
                self.write_line("right:");
                self.indent += 2;
                self.write_expr(right);
                self.indent -= 4;
                self.write_line("}");
            }
            Expr::Call { function, arguments, .. } => {
                self.write_line("CallExpr {");
                self.indent += 2;
                self.write_line(&format!("function: \"{}\"", function));
                if !arguments.is_empty() {
                    self.write_line("arguments:");
                    self.indent += 2;
                    for arg in arguments {
                        self.write_expr(arg);
                    }
                    self.indent -= 2;
                }
                self.indent -= 2;
                self.write_line("}");
            }
            Expr::If { condition, then_branch, else_branch, .. } => {
                self.write_line("IfExpr {");
                self.indent += 2;
                self.write_line("condition:");
                self.indent += 2;
                self.write_expr(condition);
                self.indent -= 2;
                self.write_line("then:");
                self.indent += 2;
                self.write_expr(then_branch);
                self.indent -= 2;
                if let Some(else_expr) = else_branch {
                    self.write_line("else:");
                    self.indent += 2;
                    self.write_expr(else_expr);
                    self.indent -= 2;
                }
                self.indent -= 2;
                self.write_line("}");
            }
            Expr::Match { scrutinee, arms, .. } => {
                self.write_line("MatchExpr {");
                self.indent += 2;
                self.write_line("scrutinee:");
                self.indent += 2;
                self.write_expr(scrutinee);
                self.indent -= 2;
                self.write_line("arms:");
                self.indent += 2;
                for arm in arms {
                    self.write_line("MatchArm {");
                    self.indent += 2;
                    self.write_line("pattern:");
                    self.indent += 2;
                    self.write_pattern(&arm.pattern);
                    self.indent -= 2;
                    self.write_line("body:");
                    self.indent += 2;
                    self.write_expr(&arm.body);
                    self.indent -= 4;
                    self.write_line("}");
                }
                self.indent -= 4;
                self.write_line("}");
            }
            // ... other expression types
        }
    }
    
    fn write_pattern(&mut self, pattern: &Pattern) {
        match pattern {
            Pattern::Wildcard => self.write_line("Wildcard"),
            Pattern::Variable(name) => {
                self.write_line(&format!("Variable(\"{}\")", name));
            }
            Pattern::Constructor { name, fields, .. } => {
                self.write_line(&format!("Constructor {{"));
                self.indent += 2;
                self.write_line(&format!("name: \"{}\"", name));
                if !fields.is_empty() {
                    self.write_line("fields:");
                    self.indent += 2;
                    for (field, pat) in fields {
                        self.write_line(&format!("{}: ", field));
                        self.indent += 2;
                        self.write_pattern(pat);
                        self.indent -= 2;
                    }
                    self.indent -= 2;
                }
                self.indent -= 2;
                self.write_line("}");
            }
            // ... other pattern types
        }
    }
    
    fn write_workflow(&mut self, workflow: &Workflow) {
        self.write_line("WorkflowDef {");
        self.indent += 2;
        self.write_line(&format!("name: \"{}\"", workflow.name));
        if !workflow.params.is_empty() {
            self.write_line("params:");
            self.indent += 2;
            for param in &workflow.params {
                self.write_line(&format!("- {}: {:?}", param.name, param.ty));
            }
            self.indent -= 2;
        }
        self.write_line("body:");
        self.indent += 2;
        for step in &workflow.body {
            self.write_step(step);
        }
        self.indent -= 4;
        self.write_line("}");
    }
    
    fn write_step(&mut self, step: &Step) {
        match step {
            Step::Act { action, args, .. } => {
                self.write_line("Step::Act {");
                self.indent += 2;
                self.write_line(&format!("action: \"{}\"", action));
                if !args.is_empty() {
                    self.write_line("args:");
                    self.indent += 2;
                    for arg in args {
                        self.write_expr(arg);
                    }
                    self.indent -= 2;
                }
                self.indent -= 2;
                self.write_line("}");
            }
            Step::Let { name, value, .. } => {
                self.write_line(&format!("Step::Let {{"));
                self.indent += 2;
                self.write_line(&format!("name: \"{}\"", name));
                self.write_line("value:");
                self.indent += 2;
                self.write_expr(value);
                self.indent -= 4;
                self.write_line("}");
            }
            // ... other step types
        }
    }
    
    fn write_line(&mut self, content: &str) {
        for _ in 0..self.indent {
            self.output.push(' ');
        }
        self.output.push_str(content);
        self.output.push('\n');
    }
}
```

### Step 3: Update :ast Command Handler

**File:** `crates/ash-repl/src/lib.rs`

```rust
use crate::ast_formatter::{format_expr_ast, format_workflow_ast};

impl Repl {
    fn handle_ast_command(&mut self, input: &str) -> ReplResult {
        let trimmed = input.trim();
        
        if trimmed.is_empty() {
            return Err(ReplError::EmptyInput);
        }
        
        // Try to parse as expression first
        match self.parser.parse_expression(trimmed) {
            Ok(expr) => {
                let formatted = format_expr_ast(&expr);
                println!("{}", formatted);
                return Ok(());
            }
            Err(ParseError::Incomplete) => {
                return Err(ReplError::Incomplete);
            }
            Err(ParseError::Error(_)) => {
                // Not an expression, try workflow
            }
        }
        
        // Try to parse as workflow definition
        match self.parser.parse_workflow(trimmed) {
            Ok(workflow) => {
                let formatted = format_workflow_ast(&workflow);
                println!("{}", formatted);
                Ok(())
            }
            Err(ParseError::Incomplete) => {
                Err(ReplError::Incomplete)
            }
            Err(e) => {
                Err(ReplError::Parse(e))
            }
        }
    }
}
```

### Step 4: Add Expression Parser Entry Point

**File:** `crates/ash-parser/src/lib.rs`

```rust
/// Parse a standalone expression (for REPL :ast command)
pub fn parse_expression(input: &str) -> Result<Expr, ParseError> {
    let mut parser = Parser::new(input);
    parser.parse_expr()
}

/// Parse a workflow definition (for REPL :ast command)
pub fn parse_workflow(input: &str) -> Result<Workflow, ParseError> {
    let mut parser = Parser::new(input);
    parser.parse_workflow_def()
}
```

## Verification Steps

- [ ] `cargo test -p ash-repl --test ast_command_test` passes
- [ ] `:ast 1 + 2` shows expression AST, not workflow wrapper
- [ ] `:ast workflow test { ... }` shows workflow AST
- [ ] `cargo test -p ash-repl` all tests pass
- [ ] `cargo clippy --all-targets --all-features` clean
- [ ] `cargo fmt --check` clean

## Dependencies for Next Task

This task outputs:
- Correct :ast command behavior per SPEC-011

Required by:
- Full REPL spec compliance

## Notes

**User Experience**: This fix directly improves REPL usability. Users expect to see the AST of what they typed, not internal implementation details.

**SPEC-011 Compliance**: The specification defines `:ast` as showing "the parsed AST for the input". The current implementation shows a debug representation of a synthetic workflow - completely different semantics.
