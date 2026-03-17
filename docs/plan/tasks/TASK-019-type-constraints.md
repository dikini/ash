# TASK-019: Type Constraint Generation

## Status: 🔴 Not Started

## Description

Implement the constraint generation phase that walks the AST and produces type constraints for later solving.

## Specification Reference

- SPEC-003: Type System - Section 7. Constraint Generation

## Requirements

### Constraint Types

```rust
/// Type constraints generated during type checking
#[derive(Debug, Clone, PartialEq)]
pub enum Constraint {
    /// Two types must be equal
    TypeEqual { expected: Type, actual: Type, span: Span },
    
    /// One type must be a subtype of another
    TypeSubtype { sub: Type, super_: Type, span: Span },
    
    /// Effect must be less than or equal to bound
    EffectLeq { effect: Effect, bound: Effect, span: Span },
    
    /// Expression must have a specific type
    HasType { expr: ExprId, ty: Type, span: Span },
    
    /// Variable binding constraint
    VarBinding { name: Box<str>, ty: Type, span: Span },
    
    /// Pattern must match type
    PatternMatch { pat: Pattern, ty: Type, span: Span },
    
    /// Capability must exist with given signature
    HasCapability { name: Box<str>, sig: CapabilitySig, span: Span },
    
    /// Obligation must be satisfiable
    SatisfiesObligation { obligation: Obligation, span: Span },
}

/// Constraint context for collecting constraints
#[derive(Debug, Default)]
pub struct ConstraintContext {
    pub constraints: Vec<Constraint>,
    pub type_env: TypeEnv,
    pub effect_env: EffectEnv,
}

impl ConstraintContext {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn add(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }
    
    pub fn fresh_var(&self) -> Type {
        Type::Var(TypeVar::fresh())
    }
}
```

### Workflow Constraint Generation

```rust
/// Generate constraints for a workflow, returning its type and effect
pub fn generate_workflow_constraints(
    ctx: &mut ConstraintContext,
    workflow: &Workflow,
) -> (Type, Effect) {
    match workflow {
        Workflow::Observe { capability, pattern, continuation } => {
            // Capability must have signature τ -> epistemic
            let cap_ty = ctx.fresh_var();
            ctx.add(Constraint::HasCapability {
                name: capability.name.clone(),
                sig: CapabilitySig {
                    param_ty: cap_ty.clone(),
                    return_ty: ctx.fresh_var(),
                    effect: Effect::Epistemic,
                },
                span: Span::default(),
            });
            
            // Pattern must match capability return type
            let return_ty = ctx.fresh_var();
            ctx.add(Constraint::PatternMatch {
                pat: pattern.clone(),
                ty: return_ty.clone(),
                span: Span::default(),
            });
            
            // Bind pattern in environment
            bind_pattern(ctx, pattern, return_ty);
            
            // Continuation type and effect
            let (cont_ty, cont_eff) = generate_workflow_constraints(ctx, continuation);
            
            // Overall effect is join of epistemic and continuation effect
            let overall_effect = Effect::Epistemic.join(cont_eff);
            
            (cont_ty, overall_effect)
        }
        
        Workflow::Orient { expr, pattern, continuation } => {
            // Expression must be evaluable
            let expr_ty = generate_expr_constraints(ctx, expr);
            
            // Pattern matches expression type
            ctx.add(Constraint::PatternMatch {
                pat: pattern.clone(),
                ty: expr_ty,
                span: Span::default(),
            });
            
            bind_pattern(ctx, pattern, expr_ty);
            
            let (cont_ty, cont_eff) = generate_workflow_constraints(ctx, continuation);
            let overall_effect = Effect::Deliberative.join(cont_eff);
            
            (cont_ty, overall_effect)
        }
        
        Workflow::Act { action, guard, .. } => {
            // Guard must be boolean
            if let Some(g) = guard {
                let guard_ty = generate_guard_constraints(ctx, g);
                ctx.add(Constraint::TypeEqual {
                    expected: Type::Bool,
                    actual: guard_ty,
                    span: Span::default(),
                });
            }
            
            // Action must have operational effect
            ctx.add(Constraint::HasCapability {
                name: action.name.clone(),
                sig: CapabilitySig {
                    param_ty: Type::Record(vec![]), // Simplified
                    return_ty: ctx.fresh_var(),
                    effect: Effect::Operational,
                },
                span: Span::default(),
            });
            
            (ctx.fresh_var(), Effect::Operational)
        }
        
        Workflow::Let { pattern, expr, continuation } => {
            let expr_ty = generate_expr_constraints(ctx, expr);
            
            ctx.add(Constraint::PatternMatch {
                pat: pattern.clone(),
                ty: expr_ty.clone(),
                span: Span::default(),
            });
            
            bind_pattern(ctx, pattern, expr_ty);
            
            generate_workflow_constraints(ctx, continuation)
        }
        
        Workflow::Seq { first, second } => {
            let (_, eff1) = generate_workflow_constraints(ctx, first);
            let (ty2, eff2) = generate_workflow_constraints(ctx, second);
            
            // Sequential effect is join
            let overall_effect = eff1.join(eff2);
            
            (ty2, overall_effect)
        }
        
        Workflow::Par { workflows } => {
            let mut types = Vec::new();
            let mut effects = Vec::new();
            
            for wf in workflows {
                let (ty, eff) = generate_workflow_constraints(ctx, wf);
                types.push(ty);
                effects.push(eff);
            }
            
            // Parallel effect is join of all
            let overall_effect = effects.into_iter()
                .fold(Effect::Epistemic, Effect::join);
            
            // Return type is tuple of all
            (Type::Record(vec![]), overall_effect) // Simplified
        }
        
        Workflow::If { condition, then_branch, else_branch } => {
            // Condition must be boolean
            let cond_ty = generate_expr_constraints(ctx, condition);
            ctx.add(Constraint::TypeEqual {
                expected: Type::Bool,
                actual: cond_ty,
                span: Span::default(),
            });
            
            let (then_ty, then_eff) = generate_workflow_constraints(ctx, then_branch);
            let (else_ty, else_eff) = generate_workflow_constraints(ctx, else_branch);
            
            // Both branches must return same type
            ctx.add(Constraint::TypeEqual {
                expected: then_ty.clone(),
                actual: else_ty,
                span: Span::default(),
            });
            
            // Effect is join of branches
            let overall_effect = then_eff.join(else_eff);
            
            (then_ty, overall_effect)
        }
        
        Workflow::Done => {
            (Type::Null, Effect::Epistemic)
        }
        
        _ => {
            // Simplified for other cases
            (ctx.fresh_var(), Effect::Epistemic)
        }
    }
}
```

### Expression Constraint Generation

```rust
/// Generate constraints for an expression, returning its inferred type
pub fn generate_expr_constraints(ctx: &mut ConstraintContext, expr: &Expr) -> Type {
    match expr {
        Expr::Literal(val) => type_of_literal(val),
        
        Expr::Var(name) => {
            match ctx.type_env.get(name) {
                Some(ty) => ty.clone(),
                None => {
                    // Unknown variable - will error later
                    ctx.fresh_var()
                }
            }
        }
        
        Expr::Input(name) => {
            // Input types come from context
            ctx.fresh_var()
        }
        
        Expr::Field { base, field } => {
            let base_ty = generate_expr_constraints(ctx, base);
            let result_ty = ctx.fresh_var();
            
            // Base must have field with appropriate type
            ctx.add(Constraint::TypeEqual {
                expected: Type::Record(vec![(field.clone(), result_ty.clone())]),
                actual: base_ty,
                span: Span::default(),
            });
            
            result_ty
        }
        
        Expr::BinOp { op, left, right } => {
            let left_ty = generate_expr_constraints(ctx, left);
            let right_ty = generate_expr_constraints(ctx, right);
            
            match op {
                BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => {
                    // Arithmetic: both operands and result are numeric
                    ctx.add(Constraint::TypeEqual {
                        expected: Type::Int,
                        actual: left_ty,
                        span: Span::default(),
                    });
                    ctx.add(Constraint::TypeEqual {
                        expected: Type::Int,
                        actual: right_ty,
                        span: Span::default(),
                    });
                    Type::Int
                }
                
                BinOp::Eq | BinOp::Neq | BinOp::Lt | BinOp::Gt | BinOp::Leq | BinOp::Geq => {
                    // Comparison: operands same type, result bool
                    ctx.add(Constraint::TypeEqual {
                        expected: left_ty.clone(),
                        actual: right_ty,
                        span: Span::default(),
                    });
                    Type::Bool
                }
                
                BinOp::And | BinOp::Or => {
                    // Logical: both operands bool, result bool
                    ctx.add(Constraint::TypeEqual {
                        expected: Type::Bool,
                        actual: left_ty,
                        span: Span::default(),
                    });
                    ctx.add(Constraint::TypeEqual {
                        expected: Type::Bool,
                        actual: right_ty,
                        span: Span::default(),
                    });
                    Type::Bool
                }
                
                _ => ctx.fresh_var(),
            }
        }
        
        Expr::UnOp { op, operand } => {
            let operand_ty = generate_expr_constraints(ctx, operand);
            
            match op {
                UnOp::Not => {
                    ctx.add(Constraint::TypeEqual {
                        expected: Type::Bool,
                        actual: operand_ty,
                        span: Span::default(),
                    });
                    Type::Bool
                }
                UnOp::Neg => {
                    ctx.add(Constraint::TypeEqual {
                        expected: Type::Int,
                        actual: operand_ty,
                        span: Span::default(),
                    });
                    Type::Int
                }
                _ => ctx.fresh_var(),
            }
        }
        
        _ => ctx.fresh_var(),
    }
}

fn type_of_literal(val: &Value) -> Type {
    match val {
        Value::Int(_) => Type::Int,
        Value::String(_) => Type::String,
        Value::Bool(_) => Type::Bool,
        Value::Null => Type::Null,
        Value::List(_) => Type::List(Box::new(Type::Var(TypeVar(0)))), // Generic list
        Value::Record(fields) => {
            let field_types: Vec<_> = fields.iter()
                .map(|(k, v)| (k.clone(), type_of_literal(v)))
                .collect();
            Type::Record(field_types)
        }
        _ => Type::Var(TypeVar(0)),
    }
}
```

### Pattern Binding

```rust
/// Bind pattern variables in the type environment
fn bind_pattern(ctx: &mut ConstraintContext, pat: &Pattern, ty: Type) {
    match pat {
        Pattern::Variable(name) => {
            ctx.type_env.insert(name.clone(), ty);
        }
        Pattern::Tuple(pats) => {
            // Type must be a tuple/record
            if let Type::Record(fields) = &ty {
                for (i, pat) in pats.iter().enumerate() {
                    if let Some((_, field_ty)) = fields.get(i) {
                        bind_pattern(ctx, pat, field_ty.clone());
                    }
                }
            }
        }
        Pattern::Record(field_pats) => {
            if let Type::Record(fields) = &ty {
                let field_map: HashMap<_, _> = fields.iter().cloned().collect();
                for (name, pat) in field_pats {
                    if let Some(field_ty) = field_map.get(name) {
                        bind_pattern(ctx, pat, field_ty.clone());
                    }
                }
            }
        }
        Pattern::Wildcard => {
            // No binding
        }
        _ => {}
    }
}
```

## TDD Steps

### Step 1: Define Constraint Types

Create `crates/ash-typeck/src/constraints.rs` with Constraint enum.

### Step 2: Implement Workflow Constraint Gen

Add generate_workflow_constraints for all workflow types.

### Step 3: Implement Expression Constraint Gen

Add generate_expr_constraints for all expression types.

### Step 4: Implement Pattern Binding

Add bind_pattern for variable extraction.

### Step 5: Write Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_observe_constraints() {
        let wf = Workflow::Observe {
            capability: Capability { name: "read".into() },
            pattern: Pattern::Variable("x".into()),
            continuation: Box::new(Workflow::Done),
        };
        
        let mut ctx = ConstraintContext::new();
        let (ty, eff) = generate_workflow_constraints(&mut ctx, &wf);
        
        // Should have capability constraint
        assert!(ctx.constraints.iter().any(|c| matches!(c, 
            Constraint::HasCapability { .. }
        )));
        
        // Effect should include epistemic
        assert!(eff >= Effect::Epistemic);
    }

    #[test]
    fn test_if_type_consistency() {
        let wf = Workflow::If {
            condition: Expr::Literal(Value::Bool(true)),
            then_branch: Box::new(Workflow::Ret { expr: Expr::Literal(Value::Int(1)) }),
            else_branch: Box::new(Workflow::Ret { expr: Expr::Literal(Value::Int(2)) }),
        };
        
        let mut ctx = ConstraintContext::new();
        generate_workflow_constraints(&mut ctx, &wf);
        
        // Should have type equality constraint between branches
        assert!(ctx.constraints.iter().any(|c| matches!(c,
            Constraint::TypeEqual { .. }
        )));
    }
}
```

## Completion Checklist

- [ ] Constraint enum with all variants
- [ ] ConstraintContext for collection
- [ ] Workflow constraint generation for all constructs
- [ ] Expression constraint generation for all operators
- [ ] Pattern binding in type environment
- [ ] Effect tracking during constraint generation
- [ ] Unit tests for each constraint type
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Self-Review Questions

1. **Completeness**: Are constraints generated for all AST nodes?
2. **Accuracy**: Do constraints match SPEC-003 typing rules?
3. **Environment**: Is the type environment managed correctly?

## Estimated Effort

6 hours

## Dependencies

- TASK-018: Type representation (uses Type, TypeVar)

## Blocked By

- TASK-018: Type representation

## Blocks

- TASK-020: Unification (solves these constraints)
- TASK-021: Effect inference (uses effect constraints)
