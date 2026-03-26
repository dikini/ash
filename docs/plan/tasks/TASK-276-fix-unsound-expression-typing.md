# TASK-276: Fix Unsound Expression Typing

## Status: 📝 Planned

## Description

Fix the critical type system unsoundness where variables receive fresh type variables instead of being looked up in the environment, and many unhandled forms silently return a fresh type variable rather than an error. This causes the type checker to accept invalid programs.

## Specification Reference

- SPEC-003: Type System Specification
- SPEC-022: Workflow Typing Specification

## Dependencies

- ✅ TASK-018: Type representation and unification
- ✅ TASK-019: Type constraint generation
- ✅ TASK-275: Enable obligation checking (related)

## Requirements

### Functional Requirements

1. Variable expressions must look up type from environment, not create fresh type vars
2. Unhandled expression forms must return explicit errors, not silent fresh type vars
3. Type environment must be properly threaded through all expression types
4. All expression variants must have explicit type rules implemented

### Current State (Broken)

**File:** `crates/ash-typeck/src/infer.rs`

```rust
impl TypeInference {
    pub fn infer_expr(&mut self, expr: &Expr, env: &TypeEnv) -> Result<Type, TypeError> {
        match expr {
            // WRONG: Variables get fresh type vars instead of env lookup
            Expr::Variable(name) => {
                Ok(self.fresh_type_var()) // BUG: Should be env.lookup(name)
            }
            
            // WRONG: Many forms silently return fresh vars
            Expr::Block(_) => Ok(self.fresh_type_var()), // Not implemented
            Expr::Loop(_) => Ok(self.fresh_type_var()),  // Not implemented
            Expr::For(_) => Ok(self.fresh_type_var()),   // Not implemented
            // ... many more
            
            Expr::Literal(lit) => Ok(self.infer_literal(lit)),
            Expr::Binary { op, left, right } => {
                // Some forms implemented correctly
                self.infer_binary(op, left, right, env)
            }
            // ...
        }
    }
}
```

### Target State (Fixed)

```rust
impl TypeInference {
    pub fn infer_expr(&mut self, expr: &Expr, env: &TypeEnv) -> Result<Type, TypeError> {
        match expr {
            // FIXED: Variables looked up in environment
            Expr::Variable(name) => {
                env.lookup(name)
                    .ok_or_else(|| TypeError::UnboundVariable {
                        name: name.clone(),
                        span: expr.span(),
                    })
            }
            
            // FIXED: Block expressions type their statements and return value
            Expr::Block(stmts) => {
                let mut block_env = env.extend();
                let mut last_type = Type::Unit;
                for stmt in stmts {
                    match stmt {
                        Stmt::Let { name, value, .. } => {
                            let ty = self.infer_expr(value, &block_env)?;
                            block_env.bind(name.clone(), ty);
                        }
                        Stmt::Expr(expr) => {
                            last_type = self.infer_expr(expr, &block_env)?;
                        }
                    }
                }
                Ok(last_type)
            }
            
            // FIXED: Loop expressions
            Expr::Loop { body, .. } => {
                self.infer_expr(body, env)?;
                Ok(Type::Never) // loop never returns normally
            }
            
            // FIXED: For expressions
            Expr::For { var, iter, body } => {
                let iter_type = self.infer_expr(iter, env)?;
                let elem_type = self.iter_element_type(iter_type)?;
                
                let mut body_env = env.extend();
                body_env.bind(var.clone(), elem_type);
                self.infer_expr(body, &body_env)?;
                Ok(Type::Unit)
            }
            
            // ... all forms implemented
            
            _ => Err(TypeError::UnsupportedExpression {
                kind: expr.kind_name(),
                span: expr.span(),
            }),
        }
    }
}
```

## TDD Steps

### Step 1: Write Tests (Red)

**File:** `crates/ash-typeck/tests/expression_typing_soundness_test.rs`

```rust
//! Tests for expression typing soundness

use ash_typeck::{TypeChecker, TypeError};
use ash_parser::parse_expr;
use ash_core::Type;

#[test]
fn test_variable_lookup_from_env() {
    let mut checker = TypeChecker::new();
    
    // Set up environment with x: Int
    checker.bind("x", Type::Int);
    
    let expr = parse_expr("x").unwrap();
    let ty = checker.infer_expr(&expr).unwrap();
    
    assert_eq!(ty, Type::Int);
}

#[test]
fn test_unbound_variable_error() {
    let mut checker = TypeChecker::new();
    
    let expr = parse_expr("undefined_var").unwrap();
    let result = checker.infer_expr(&expr);
    
    assert!(matches!(result, Err(TypeError::UnboundVariable { .. })));
}

#[test]
fn test_block_expression_scoping() {
    let mut checker = TypeChecker::new();
    
    let expr = parse_expr(r#"
        {
            let x = 42;
            let y = x + 1;
            y
        }
    "#).unwrap();
    
    let ty = checker.infer_expr(&expr).unwrap();
    assert_eq!(ty, Type::Int);
}

#[test]
fn test_block_binding_not_leaked() {
    let mut checker = TypeChecker::new();
    
    let expr = parse_expr(r#"
        {
            let x = 42;
            x
        };
        x  // ERROR: x not in scope
    "#).unwrap();
    
    let result = checker.infer_expr(&expr);
    assert!(matches!(result, Err(TypeError::UnboundVariable { name })
        if name == "x"));
}

#[test]
fn test_loop_type_is_never() {
    let mut checker = TypeChecker::new();
    
    let expr = parse_expr("loop { break }").unwrap();
    let ty = checker.infer_expr(&expr).unwrap();
    
    assert_eq!(ty, Type::Never);
}

#[test]
fn test_for_loop_type() {
    let mut checker = TypeChecker::new();
    
    let expr = parse_expr("for x in [1, 2, 3] { x }").unwrap();
    let ty = checker.infer_expr(&expr).unwrap();
    
    assert_eq!(ty, Type::Unit);
}

proptest! {
    #[test]
    fn type_soundness_preservation(expr in arb_expression()) {
        // Property: If type checking succeeds, the type is sound
        // (no runtime type errors possible for that expression)
        let mut checker = TypeChecker::new();
        if let Ok(ty) = checker.infer_expr(&expr) {
            prop_assert!(ty.is_well_formed());
        }
    }
}
```

### Step 2: Fix Variable Lookup

**File:** `crates/ash-typeck/src/infer.rs`

```rust
impl TypeInference {
    fn infer_expr(&mut self, expr: &Expr, env: &TypeEnv) -> Result<Type, TypeError> {
        match expr {
            Expr::Variable(name) => {
                env.lookup(name)
                    .ok_or_else(|| TypeError::UnboundVariable {
                        name: name.clone(),
                        span: expr.span(),
                    })
            }
            // ... rest
        }
    }
}
```

### Step 3: Implement Missing Expression Types

**File:** `crates/ash-typeck/src/infer.rs`

```rust
impl TypeInference {
    fn infer_block(&mut self, stmts: &[Stmt], env: &TypeEnv) -> Result<Type, TypeError> {
        let mut block_env = env.extend();
        let mut last_type = Type::Unit;
        
        for stmt in stmts {
            match stmt {
                Stmt::Let { name, type_hint, value } => {
                    let value_type = self.infer_expr(value, &block_env)?;
                    let var_type = if let Some(hint) = type_hint {
                        self.unify(&value_type, hint)?;
                        hint.clone()
                    } else {
                        value_type
                    };
                    block_env.bind(name.clone(), var_type);
                }
                Stmt::Expr(expr) => {
                    last_type = self.infer_expr(expr, &block_env)?;
                }
                Stmt::Return(expr) => {
                    let ret_type = self.infer_expr(expr, &block_env)?;
                    // Check against expected return type
                    return Ok(ret_type);
                }
            }
        }
        
        Ok(last_type)
    }
    
    fn infer_loop(&mut self, body: &Expr, env: &TypeEnv) -> Result<Type, TypeError> {
        // Loop body is checked but loop itself has type ! (never)
        self.infer_expr(body, env)?;
        Ok(Type::Never)
    }
    
    fn infer_for(
        &mut self,
        var: &str,
        iter: &Expr,
        body: &Expr,
        env: &TypeEnv,
    ) -> Result<Type, TypeError> {
        let iter_type = self.infer_expr(iter, env)?;
        let elem_type = match &iter_type {
            Type::List(t) => *t.clone(),
            Type::Array(t, _) => *t.clone(),
            Type::Range => Type::Int,
            _ => return Err(TypeError::NotIterable {
                ty: iter_type,
                span: iter.span(),
            }),
        };
        
        let mut body_env = env.extend();
        body_env.bind(var.to_string(), elem_type);
        self.infer_expr(body, &body_env)?;
        
        Ok(Type::Unit)
    }
}
```

### Step 4: Add Error Types

**File:** `crates/ash-typeck/src/error.rs`

```rust
#[derive(Debug, Clone, Error, PartialEq)]
pub enum TypeError {
    // ... existing errors ...
    
    #[error("unbound variable: {name}")]
    UnboundVariable {
        name: String,
        span: Span,
    },
    
    #[error("type {ty} is not iterable")]
    NotIterable {
        ty: Type,
        span: Span,
    },
    
    #[error("unsupported expression: {kind}")]
    UnsupportedExpression {
        kind: String,
        span: Span,
    },
}
```

## Verification Steps

- [ ] `cargo test -p ash-typeck --test expression_typing_soundness_test` passes
- [ ] `cargo test -p ash-typeck --test type_inference_test` passes
- [ ] `cargo test -p ash-engine` passes (integration tests)
- [ ] `cargo test -p ash-cli` passes
- [ ] `cargo clippy --all-targets --all-features` clean
- [ ] `cargo fmt --check` clean

## Dependencies for Next Task

This task outputs:
- Sound expression type checking
- Proper environment threading
- Complete expression coverage

Required by:
- All downstream type-dependent tasks

## Notes

**Critical Issue**: This is a fundamental type system soundness bug. The type checker currently accepts programs that will fail at runtime.

**Audit Finding**: "Variables are assigned a fresh type variable instead of being looked up in the environment, and many unhandled forms silently return a fresh type variable rather than an error."

**Edge Cases**:
- Shadowing - inner binding should hide outer
- Mutable variables - type must remain consistent
- Early return - must unify with expected return type
- Recursion - requires fixpoint computation
