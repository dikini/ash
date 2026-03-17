# TASK-020: Constraint Solving (Unification Algorithm)

## Status: 🟢 Complete

## Description

Implement the constraint solving phase that takes generated constraints and solves them using unification.

## Specification Reference

- SPEC-003: Type System - Section 7.2 Unification

## Requirements

### Solver State

```rust
/// Solver state for constraint solving
#[derive(Debug, Default)]
pub struct Solver {
    /// Current substitution
    substitution: Substitution,
    /// Collected errors
    errors: Vec<TypeError>,
    /// Effect constraints to check
    effect_constraints: Vec<EffectConstraint>,
}

#[derive(Debug, Clone)]
pub struct EffectConstraint {
    pub effect: Effect,
    pub bound: Effect,
    pub span: Span,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum TypeError {
    #[error("Type mismatch: expected {expected}, found {actual}")]
    Mismatch { expected: String, actual: String, span: Span },
    
    #[error("Undefined variable: {0}")]
    UndefinedVariable(String, Span),
    
    #[error("Undefined capability: {0}")]
    UndefinedCapability(String, Span),
    
    #[error("Effect mismatch: {effect} > {bound}")]
    EffectMismatch { effect: Effect, bound: Effect, span: Span },
    
    #[error("Occurs check failed: {0} appears in its own type")]
    OccursCheck(String, Span),
    
    #[error("Cannot unify: {0}")]
    UnificationFailure(String, Span),
}
```

### Constraint Solving

```rust
impl Solver {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Solve a list of constraints
    pub fn solve(&mut self, constraints: Vec<Constraint>) -> Result<Substitution, Vec<TypeError>> {
        for constraint in constraints {
            if let Err(e) = self.solve_constraint(constraint) {
                self.errors.push(e);
            }
        }
        
        // Check effect constraints
        self.check_effects();
        
        if self.errors.is_empty() {
            Ok(self.substitution.clone())
        } else {
            Err(self.errors.clone())
        }
    }
    
    fn solve_constraint(&mut self, constraint: Constraint) -> Result<(), TypeError> {
        match constraint {
            Constraint::TypeEqual { expected, actual, span } => {
                let expected = self.substitution.apply(&expected);
                let actual = self.substitution.apply(&actual);
                
                match unify(&expected, &actual) {
                    Ok(subst) => {
                        self.substitution = subst.compose(self.substitution.clone());
                        Ok(())
                    }
                    Err(UnifyError::OccursCheck(var, ty)) => {
                        Err(TypeError::OccursCheck(
                            format!("{:?}", var),
                            span,
                        ))
                    }
                    Err(_) => {
                        Err(TypeError::Mismatch {
                            expected: format!("{:?}", expected),
                            actual: format!("{:?}", actual),
                            span,
                        })
                    }
                }
            }
            
            Constraint::TypeSubtype { sub, super_, span } => {
                // For now, subtyping is just equality
                // Later: add actual subtyping rules
                let sub = self.substitution.apply(&sub);
                let super_ = self.substitution.apply(&super_);
                
                match unify(&sub, &super_) {
                    Ok(subst) => {
                        self.substitution = subst.compose(self.substitution.clone());
                        Ok(())
                    }
                    Err(_) => {
                        Err(TypeError::Mismatch {
                            expected: format!("{:?}", super_),
                            actual: format!("{:?}", sub),
                            span,
                        })
                    }
                }
            }
            
            Constraint::EffectLeq { effect, bound, span } => {
                // Defer effect checking until all type constraints are solved
                self.effect_constraints.push(EffectConstraint {
                    effect,
                    bound,
                    span,
                });
                Ok(())
            }
            
            Constraint::HasCapability { name, sig, span } => {
                // Look up capability in environment
                // For now, always succeed
                // Later: check against capability declarations
                Ok(())
            }
            
            Constraint::PatternMatch { pat, ty, span } => {
                // Check pattern against type
                self.check_pattern(pat, &ty, span)
            }
            
            _ => Ok(()),
        }
    }
    
    fn check_effects(&mut self) {
        for eff_constraint in &self.effect_constraints {
            // Apply substitution to resolve any type variables in effects
            // (though effects shouldn't have type variables)
            
            if eff_constraint.effect > eff_constraint.bound {
                self.errors.push(TypeError::EffectMismatch {
                    effect: eff_constraint.effect,
                    bound: eff_constraint.bound,
                    span: eff_constraint.span,
                });
            }
        }
    }
    
    fn check_pattern(&mut self, pat: &Pattern, ty: &Type, span: Span) -> Result<(), TypeError> {
        match pat {
            Pattern::Wildcard => Ok(()),
            Pattern::Variable(_) => Ok(()),
            Pattern::Literal(lit) => {
                let lit_ty = type_of_literal(lit);
                match unify(&lit_ty, ty) {
                    Ok(subst) => {
                        self.substitution = subst.compose(self.substitution.clone());
                        Ok(())
                    }
                    Err(_) => Err(TypeError::Mismatch {
                        expected: format!("{:?}", ty),
                        actual: format!("{:?}", lit_ty),
                        span,
                    }),
                }
            }
            Pattern::Tuple(pats) => {
                if let Type::Record(fields) = ty {
                    if pats.len() != fields.len() {
                        return Err(TypeError::UnificationFailure(
                            format!("tuple pattern has {} elements, type has {}", 
                                pats.len(), fields.len()),
                            span,
                        ));
                    }
                    for (pat, (_, field_ty)) in pats.iter().zip(fields.iter()) {
                        self.check_pattern(pat, field_ty, span)?;
                    }
                    Ok(())
                } else {
                    Err(TypeError::Mismatch {
                        expected: "tuple".to_string(),
                        actual: format!("{:?}", ty),
                        span,
                    })
                }
            }
            Pattern::Record(field_pats) => {
                if let Type::Record(fields) = ty {
                    let field_map: HashMap<_, _> = fields.iter().cloned().collect();
                    for (name, pat) in field_pats {
                        match field_map.get(name) {
                            Some(field_ty) => {
                                self.check_pattern(pat, field_ty, span)?;
                            }
                            None => {
                                return Err(TypeError::UnificationFailure(
                                    format!("field {} not found in type", name),
                                    span,
                                ));
                            }
                        }
                    }
                    Ok(())
                } else {
                    Err(TypeError::Mismatch {
                        expected: "record".to_string(),
                        actual: format!("{:?}", ty),
                        span,
                    })
                }
            }
            Pattern::List(pats, rest) => {
                if let Type::List(elem_ty) = ty {
                    for pat in pats {
                        self.check_pattern(pat, elem_ty, span)?;
                    }
                    // Rest binding is also a list
                    if rest.is_some() {
                        // Already valid by type
                    }
                    Ok(())
                } else {
                    Err(TypeError::Mismatch {
                        expected: "list".to_string(),
                        actual: format!("{:?}", ty),
                        span,
                    })
                }
            }
        }
    }
}
```

### Type Inference Entry Point

```rust
/// Infer types for a workflow, returning the substitution and any errors
pub fn infer_types(workflow: &Workflow) -> (Substitution, Vec<TypeError>) {
    // Generate constraints
    let mut ctx = ConstraintContext::new();
    let (_ty, _eff) = generate_workflow_constraints(&mut ctx, workflow);
    
    // Solve constraints
    let mut solver = Solver::new();
    match solver.solve(ctx.constraints) {
        Ok(subst) => (subst, vec![]),
        Err(errors) => (solver.substitution, errors),
    }
}

/// Type check a program, returning all type errors
pub fn type_check(program: &Program) -> Vec<TypeError> {
    let mut all_errors = Vec::new();
    
    // Type check each definition
    for def in &program.definitions {
        // Check definition
    }
    
    // Type check main workflow
    let (_, errors) = infer_types(&program.workflow);
    all_errors.extend(errors);
    
    all_errors
}
```

## TDD Steps

### Step 1: Implement Solver State

Create `crates/ash-typeck/src/solver.rs` with Solver struct.

### Step 2: Implement Constraint Solving

Add solve_constraint for each constraint type.

### Step 3: Implement Pattern Checking

Add check_pattern for pattern validation.

### Step 4: Implement Effect Checking

Add check_effects for effect constraint validation.

### Step 5: Write Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solve_simple_equality() {
        let constraints = vec![
            Constraint::TypeEqual {
                expected: Type::Int,
                actual: Type::Int,
                span: Span::default(),
            },
        ];
        
        let mut solver = Solver::new();
        let result = solver.solve(constraints);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_solve_type_mismatch() {
        let constraints = vec![
            Constraint::TypeEqual {
                expected: Type::Int,
                actual: Type::String,
                span: Span::default(),
            },
        ];
        
        let mut solver = Solver::new();
        let result = solver.solve(constraints);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_solve_with_substitution() {
        let v = Type::Var(TypeVar(0));
        let constraints = vec![
            Constraint::TypeEqual {
                expected: v.clone(),
                actual: Type::Int,
                span: Span::default(),
            },
            Constraint::TypeEqual {
                expected: v.clone(),
                actual: Type::Int,
                span: Span::default(),
            },
        ];
        
        let mut solver = Solver::new();
        let result = solver.solve(constraints);
        
        assert!(result.is_ok());
        assert_eq!(solver.substitution.get(TypeVar(0)), Some(&Type::Int));
    }

    #[test]
    fn test_effect_constraint_violation() {
        let constraints = vec![
            Constraint::EffectLeq {
                effect: Effect::Operational,
                bound: Effect::Epistemic,
                span: Span::default(),
            },
        ];
        
        let mut solver = Solver::new();
        let result = solver.solve(constraints);
        
        assert!(result.is_err());
    }
}
```

## Completion Checklist

- [ ] Solver struct with substitution and error collection
- [ ] solve_constraint for all constraint types
- [ ] Effect constraint checking
- [ ] Pattern validation
- [ ] Type inference entry point
- [ ] Type checking for programs
- [ ] Unit tests for solver
- [ ] Integration tests with real workflows
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Self-Review Questions

1. **Correctness**: Does solver correctly handle all constraints?
2. **Error accumulation**: Are all errors reported, not just first?
3. **Performance**: Is solving efficient for large constraint sets?

## Estimated Effort

6 hours

## Dependencies

- TASK-018: Type representation (uses Substitution, unify)
- TASK-019: Type constraints (solves these constraints)

## Blocked By

- TASK-018: Type representation
- TASK-019: Type constraints

## Blocks

- TASK-021: Effect inference (builds on solver)
- TASK-025: Type errors (error formatting)
