# TASK-016: Surface to Core Lowering

## Status: 🟢 Complete

## Description

Implement the lowering pass that transforms Surface AST into Core IR. This includes validation, desugaring, and semantic analysis.

## Specification Reference

- SPEC-001: IR
- SPEC-002: Surface Language

## Requirements

### Lowering Pipeline

```
Surface AST → Validation → Desugaring → Core IR
```

### Lowering Context

```rust
pub struct LoweringContext {
    /// Errors collected during lowering
    pub errors: Vec<LoweringError>,
    /// Warnings collected during lowering
    pub warnings: Vec<LoweringWarning>,
    /// Symbol table for name resolution
    pub symbols: SymbolTable,
    /// Current scope depth
    pub scope_depth: usize,
}

impl LoweringContext {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
            symbols: SymbolTable::new(),
            scope_depth: 0,
        }
    }
    
    pub fn error(&mut self, span: Span, message: impl Into<String>) {
        self.errors.push(LoweringError {
            span,
            message: message.into(),
        });
    }
    
    pub fn warning(&mut self, span: Span, message: impl Into<String>) {
        self.warnings.push(LoweringWarning {
            span,
            message: message.into(),
        });
    }
}
```

### Main Lowering Function

```rust
pub fn lower_program(program: surface::Program) -> Result<core::Program, Vec<LoweringError>> {
    let mut ctx = LoweringContext::new();
    
    // First pass: collect definitions
    for def in &program.definitions {
        collect_definition(&mut ctx, def);
    }
    
    // Second pass: lower definitions
    let definitions: Vec<_> = program.definitions
        .into_iter()
        .filter_map(|d| lower_definition(&mut ctx, d))
        .collect();
    
    // Lower main workflow
    let workflow = lower_workflow_def(&mut ctx, program.workflow);
    
    if ctx.errors.is_empty() {
        Ok(core::Program {
            definitions,
            workflow,
        })
    } else {
        Err(ctx.errors)
    }
}
```

### Workflow Lowering

```rust
pub fn lower_workflow(ctx: &mut LoweringContext, workflow: surface::Workflow) -> core::Workflow {
    match workflow {
        surface::Workflow::Observe { capability, binding, continuation, .. } => {
            let cap = lower_capability_ref(ctx, capability);
            let pat = binding.map(|b| lower_pattern(ctx, b));
            let cont = continuation.map(|c| lower_workflow(ctx, *c))
                .unwrap_or(core::Workflow::Done);
            
            core::Workflow::Observe {
                capability: cap,
                pattern: pat.unwrap_or(core::Pattern::Wildcard),
                continuation: Box::new(cont),
            }
        }
        
        surface::Workflow::Orient { expr, binding, continuation, .. } => {
            let expr = lower_expr(ctx, expr);
            let pat = binding.map(|b| lower_pattern(ctx, b));
            let cont = continuation.map(|c| lower_workflow(ctx, *c))
                .unwrap_or(core::Workflow::Done);
            
            core::Workflow::Orient {
                expr,
                pattern: pat.unwrap_or(core::Pattern::Wildcard),
                continuation: Box::new(cont),
            }
        }
        
        surface::Workflow::Decide { expr, policy, then_branch, else_branch, .. } => {
            let expr = lower_expr(ctx, expr);
            let policy_name = policy.unwrap_or_else(|| "default".into());
            let then_wf = lower_workflow(ctx, *then_branch);
            
            // Decide desugars to: DECIDE + conditional SEQ
            let else_wf = else_branch
                .map(|e| lower_workflow(ctx, *e))
                .unwrap_or(core::Workflow::Done);
            
            core::Workflow::Decide {
                expr,
                policy: policy_name,
                continuation: Box::new(core::Workflow::If {
                    condition: core::Expr::Literal(core::Value::Bool(true)), // Placeholder
                    then_branch: Box::new(then_wf),
                    else_branch: Box::new(else_wf),
                }),
            }
        }
        
        surface::Workflow::Act { action, guard, .. } => {
            let act = lower_action_ref(ctx, action);
            let guard = guard.map(|g| lower_guard(ctx, g))
                .unwrap_or(core::Guard::Always);
            
            core::Workflow::Act {
                action: act,
                guard,
                provenance: core::Provenance::default(),
            }
        }
        
        surface::Workflow::Let { pattern, expr, continuation, .. } => {
            let pat = lower_pattern(ctx, pattern);
            let expr = lower_expr(ctx, expr);
            let cont = continuation.map(|c| lower_workflow(ctx, *c))
                .unwrap_or(core::Workflow::Done);
            
            core::Workflow::Let {
                pattern: pat,
                expr,
                continuation: Box::new(cont),
            }
        }
        
        surface::Workflow::If { condition, then_branch, else_branch, .. } => {
            let cond = lower_expr(ctx, condition);
            let then_wf = lower_workflow(ctx, *then_branch);
            let else_wf = else_branch
                .map(|e| lower_workflow(ctx, *e))
                .unwrap_or(core::Workflow::Done);
            
            core::Workflow::If {
                condition: cond,
                then_branch: Box::new(then_wf),
                else_branch: Box::new(else_wf),
            }
        }
        
        surface::Workflow::Par { branches, .. } => {
            let lowered: Vec<_> = branches
                .into_iter()
                .map(|b| lower_workflow(ctx, b))
                .collect();
            
            core::Workflow::Par { workflows: lowered }
        }
        
        surface::Workflow::Seq { first, second, .. } => {
            core::Workflow::Seq {
                first: Box::new(lower_workflow(ctx, *first)),
                second: Box::new(lower_workflow(ctx, *second)),
            }
        }
        
        surface::Workflow::Done { .. } => core::Workflow::Done,
        
        // Desugar other constructs
        surface::Workflow::While { condition, body, .. } => {
            lower_while_loop(ctx, condition, body)
        }
        
        surface::Workflow::For { pattern, collection, body, .. } => {
            lower_for_loop(ctx, pattern, collection, body)
        }
        
        surface::Workflow::With { capability, body, .. } => {
            let cap = lower_capability_ref(ctx, capability);
            let wf = lower_workflow(ctx, *body);
            
            core::Workflow::With {
                capability: cap,
                workflow: Box::new(wf),
            }
        }
        
        _ => {
            ctx.error(workflow.span(), format!("unsupported workflow construct"));
            core::Workflow::Done
        }
    }
}
```

### Desugaring

```rust
/// Desugar while loop to recursive structure
fn lower_while_loop(
    ctx: &mut LoweringContext,
    condition: surface::Expr,
    body: Box<surface::Workflow>,
) -> core::Workflow {
    // while cond do body
    // =>
    // if cond then { body; while cond do body } else done
    
    let cond = lower_expr(ctx, condition);
    let body_wf = lower_workflow(ctx, *body);
    
    // We need to create a recursive reference here
    // For now, we use a placeholder that will be resolved later
    core::Workflow::If {
        condition: cond,
        then_branch: Box::new(core::Workflow::Seq {
            first: Box::new(body_wf),
            second: Box::new(core::Workflow::Done), // Placeholder for recursion
        }),
        else_branch: Box::new(core::Workflow::Done),
    }
}

/// Desugar for loop to iteration
fn lower_for_loop(
    ctx: &mut LoweringContext,
    pattern: surface::Pattern,
    collection: surface::Expr,
    body: Box<surface::Workflow>,
) -> core::Workflow {
    // for pat in coll do body
    // =>
    // FOREACH pat coll body
    
    core::Workflow::ForEach {
        pattern: lower_pattern(ctx, pattern),
        collection: lower_expr(ctx, collection),
        body: Box::new(lower_workflow(ctx, *body)),
    }
}
```

### Expression Lowering

```rust
pub fn lower_expr(ctx: &mut LoweringContext, expr: surface::Expr) -> core::Expr {
    match expr {
        surface::Expr::Literal(lit) => core::Expr::Literal(lower_literal(lit)),
        surface::Expr::Variable(name) => core::Expr::Var(name),
        surface::Expr::InputRef(name) => core::Expr::Input(name),
        
        surface::Expr::FieldAccess { base, field, .. } => {
            let base = lower_expr(ctx, *base);
            core::Expr::Field {
                base: Box::new(base),
                field,
            }
        }
        
        surface::Expr::Binary { op, left, right, .. } => {
            let left = lower_expr(ctx, *left);
            let right = lower_expr(ctx, *right);
            let core_op = lower_binary_op(op);
            
            core::Expr::BinOp {
                op: core_op,
                left: Box::new(left),
                right: Box::new(right),
            }
        }
        
        surface::Expr::Unary { op, operand, .. } => {
            let operand = lower_expr(ctx, *operand);
            let core_op = lower_unary_op(op);
            
            core::Expr::UnOp {
                op: core_op,
                operand: Box::new(operand),
            }
        }
        
        _ => {
            ctx.error(expr.span(), format!("unsupported expression"));
            core::Expr::Literal(core::Value::Null)
        }
    }
}
```

## TDD Steps

### Step 1: Define Lowering Context

Create context struct with error/warning collection.

### Step 2: Implement Basic Lowering

Start with simplest workflows (Done, Act, Observe).

### Step 3: Implement Expression Lowering

Lower all expression types.

### Step 4: Implement Desugaring

Add while/for loop desugaring.

### Step 5: Add Validation

Add semantic validation during lowering.

### Step 6: Write Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lower_observe() {
        let surface_wf = surface::Workflow::Observe {
            capability: surface::CapabilityRef { name: "read".into(), args: vec![] },
            binding: Some(surface::Pattern::Variable("x".into())),
            continuation: Some(Box::new(surface::Workflow::Done { span: Span::default() })),
            span: Span::default(),
        };
        
        let mut ctx = LoweringContext::new();
        let core_wf = lower_workflow(&mut ctx, surface_wf);
        
        assert!(matches!(core_wf, core::Workflow::Observe { .. }));
        assert!(ctx.errors.is_empty());
    }

    #[test]
    fn test_lower_seq() {
        let surface_wf = surface::Workflow::Seq {
            first: Box::new(surface::Workflow::Done { span: Span::default() }),
            second: Box::new(surface::Workflow::Done { span: Span::default() }),
            span: Span::default(),
        };
        
        let mut ctx = LoweringContext::new();
        let core_wf = lower_workflow(&mut ctx, surface_wf);
        
        assert!(matches!(core_wf, core::Workflow::Seq { .. }));
    }
}
```

## Completion Checklist

- [ ] LoweringContext with error/warning collection
- [ ] All surface workflow types lowered to core
- [ ] All expression types lowered
- [ ] Desugaring of while/for loops
- [ ] Pattern lowering
- [ ] Guard lowering
- [ ] Validation during lowering
- [ ] Comprehensive tests
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Self-Review Questions

1. **Completeness**: Can every valid surface program be lowered?
2. **Error handling**: Are all lowering errors reported clearly?
3. **Performance**: Is lowering linear in program size?

## Estimated Effort

8 hours

## Dependencies

- ash-core: Core IR types (Workflow, Expr, etc.)
- TASK-011: Surface AST (lowers from these types)

## Blocked By

- TASK-011: Surface AST
- TASK-013: Workflow parser (produces surface AST)

## Blocks

- TASK-018: Type representation (works on core IR)
- TASK-022: Name resolution (works on core IR)
