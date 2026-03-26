# TASK-291: Fix Unsound Expression Typing

## Status: 📝 Planned

## Description

Fix the critical issue where expression typing is unsound for variables and many expression forms. Variables are assigned fresh type variables instead of using the environment, and many unimplemented forms silently return fresh type variables instead of errors. This violates SPEC-003 and can let invalid programs appear well-typed.

## Specification Reference

- SPEC-003: Type System Specification

## Dependencies

- ✅ TASK-018: Type representation and unification
- ✅ TASK-019: Type constraint generation
- ✅ TASK-020: Unification algorithm

## Critical File Locations

- `crates/ash-typeck/src/check_expr.rs:54` - variables get fresh type variables
- `crates/ash-typeck/src/check_expr.rs:57` - variables don't use environment
- `crates/ash-typeck/src/check_expr.rs:78` - unimplemented forms return fresh variables

## Requirements

### Functional Requirements

1. Variables must use types from the environment
2. Unbound variables must produce errors, not fresh type variables
3. Unimplemented expression forms must produce errors
4. Type soundness must be preserved

### Current State (Broken)

**File:** `crates/ash-typeck/src/check_expr.rs:54-78`

```rust
fn check_expr(&mut self, expr: &Expr) -> Result<Type, TypeError> {
    match expr {
        Expr::Variable(name) => {
            // WRONG: Creates fresh type variable instead of looking up
            Ok(self.fresh_type_var())  // Line 54
            // MISSING: self.env.lookup(name)
        }
        Expr::Binary { op, left, right } => {
            let left_ty = self.check_expr(left)?;
            let right_ty = self.check_expr(right)?;
            // ...
        }
        Expr::Call { func, args } => {
            // Partially implemented
        }
        // ... many other forms
        _ => {
            // WRONG: Silent fallback to fresh type variable
            Ok(self.fresh_type_var())  // Line 78
        }
    }
}
```

Problems:
1. Variables ignore the type environment
2. Unbound variables appear well-typed
3. Unimplemented forms silently succeed
4. Type errors are missed
5. Invalid programs pass type checking

### Target State (Fixed)

```rust
fn check_expr(&mut self, expr: &Expr, ctx: &TypeContext) -> Result<Type, TypeError> {
    match expr {
        Expr::Variable(name) => {
            // FIX: Look up variable in environment
            ctx.lookup(name)
                .ok_or_else(|| TypeError::UnboundVariable {
                    name: name.clone(),
                    span: expr.span(),
                })
                .cloned()
        }
        Expr::Binary { op, left, right } => {
            let left_ty = self.check_expr(left, ctx)?;
            let right_ty = self.check_expr(right, ctx)?;
            self.check_binary_op(*op, left_ty, right_ty, expr.span())
        }
        Expr::Literal(Literal::Int(_)) => Ok(Type::Int),
        Expr::Literal(Literal::String(_)) => Ok(Type::String),
        Expr::Literal(Literal::Bool(_)) => Ok(Type::Bool),
        Expr::Call { func, args } => {
            self.check_call(func, args, ctx)
        }
        Expr::If { condition, then_branch, else_branch } => {
            self.check_if(condition, then_branch, else_branch.as_deref(), ctx)
        }
        Expr::Match { scrutinee, arms } => {
            self.check_match(scrutinee, arms, ctx)
        }
        Expr::Block(stmts) => {
            self.check_block(stmts, ctx)
        }
        // ... explicitly handle all expression forms
        _ => {
            // FIX: Error on truly unimplemented forms
            Err(TypeError::UnsupportedExpression {
                expr_type: expr.kind_name(),
                span: expr.span(),
            })
        }
    }
}
```

## TDD Steps

### Step 1: Write Tests (Red)

**File:** `crates/ash-typeck/tests/sound_expression_typing_test.rs`

```rust
//! Tests for sound expression typing

use ash_typeck::TypeChecker;
use ash_parser::parse_expression;

#[test]
fn test_bound_variable_uses_environment_type() {
    let mut checker = TypeChecker::new();
    let mut ctx = TypeContext::new();
    ctx.bind("x".to_string(), Type::Int);
    
    let expr = parse_expression("x").unwrap();
    let ty = checker.check_expr(&expr, &ctx).unwrap();
    
    assert_eq!(ty, Type::Int);
}

#[test]
fn test_unbound_variable_produces_error() {
    let mut checker = TypeChecker::new();
    let ctx = TypeContext::new();  // Empty - x not bound
    
    let expr = parse_expression("unbound_var").unwrap();
    let result = checker.check_expr(&expr, &ctx);
    
    // Should fail: unbound variable
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("unbound variable") || err.contains("not found"));
    assert!(err.contains("unbound_var"));
}

#[test]
fn test_variable_type_matches_environment() {
    let mut checker = TypeChecker::new();
    let mut ctx = TypeContext::new();
    ctx.bind("s".to_string(), Type::String);
    
    let expr = parse_expression("s").unwrap();
    let ty = checker.check_expr(&expr, &ctx).unwrap();
    
    assert_eq!(ty, Type::String);
}

#[test]
fn test_literal_types() {
    let mut checker = TypeChecker::new();
    let ctx = TypeContext::new();
    
    let int_expr = parse_expression("42").unwrap();
    assert_eq!(checker.check_expr(&int_expr, &ctx).unwrap(), Type::Int);
    
    let string_expr = parse_expression("\"hello\"").unwrap();
    assert_eq!(checker.check_expr(&string_expr, &ctx).unwrap(), Type::String);
    
    let bool_expr = parse_expression("true").unwrap();
    assert_eq!(checker.check_expr(&bool_expr, &ctx).unwrap(), Type::Bool);
}

#[test]
fn test_binary_op_type_inference() {
    let mut checker = TypeChecker::new();
    let ctx = TypeContext::new();
    
    let expr = parse_expression("1 + 2").unwrap();
    let ty = checker.check_expr(&expr, &ctx).unwrap();
    
    assert_eq!(ty, Type::Int);
}

#[test]
fn test_binary_op_type_error() {
    let mut checker = TypeChecker::new();
    let ctx = TypeContext::new();
    
    let expr = parse_expression("\"hello\" + 1").unwrap();
    let result = checker.check_expr(&expr, &ctx);
    
    // Should fail: can't add string and int
    assert!(result.is_err());
}

#[test]
fn test_if_expression_type() {
    let mut checker = TypeChecker::new();
    let ctx = TypeContext::new();
    
    let expr = parse_expression("if true { 1 } else { 2 }").unwrap();
    let ty = checker.check_expr(&expr, &ctx).unwrap();
    
    assert_eq!(ty, Type::Int);
}

#[test]
fn test_if_branches_must_match() {
    let mut checker = TypeChecker::new();
    let ctx = TypeContext::new();
    
    let expr = parse_expression("if true { 1 } else { \"hello\" }").unwrap();
    let result = checker.check_expr(&expr, &ctx);
    
    // Should fail: branches have different types
    assert!(result.is_err());
}

#[test]
fn test_match_expression_type() {
    let mut checker = TypeChecker::new();
    let mut ctx = TypeContext::new();
    
    // Setup Option<Int> type context
    let option_type = Type::Adt("Option".to_string(), vec![Type::Int]);
    let some_variant = Type::Constructor("Some".to_string(), vec![Type::Int]);
    let none_variant = Type::Constructor("None".to_string(), vec![]);
    
    ctx.bind("x".to_string(), option_type);
    
    let expr = parse_expression("match x { Some(v) => v, None => 0 }").unwrap();
    let ty = checker.check_expr(&expr, &ctx).unwrap();
    
    assert_eq!(ty, Type::Int);
}

#[test]
fn test_no_silent_fresh_type_variables() {
    let mut checker = TypeChecker::new();
    let ctx = TypeContext::new();
    
    // This should NOT silently produce a type variable
    // It should either produce a proper type or an error
    let expr = parse_expression("some_unimplemented_feature()").unwrap();
    let result = checker.check_expr(&expr, &ctx);
    
    // Must not silently succeed with unknown type
    match result {
        Ok(ty) => {
            // If it succeeds, the type must be concrete
            assert!(!matches!(ty, Type::Var(_)), 
                "Got fresh type variable - soundness violation!");
        }
        Err(_) => {
            // Error is acceptable
        }
    }
}

#[test]
fn test_block_expression_type() {
    let mut checker = TypeChecker::new();
    let mut ctx = TypeContext::new();
    ctx.bind("x".to_string(), Type::Int);
    
    let expr = parse_expression("{ let y = x; y + 1 }").unwrap();
    let ty = checker.check_expr(&expr, &ctx).unwrap();
    
    assert_eq!(ty, Type::Int);
}

proptest! {
    #[test]
    fn variable_lookup_is_deterministic(var_name in "[a-z]{1,10}") {
        let mut checker = TypeChecker::new();
        let mut ctx = TypeContext::new();
        ctx.bind(var_name.clone(), Type::Int);
        
        let expr = Expr::Variable(var_name.clone());
        let ty1 = checker.check_expr(&expr, &ctx).unwrap();
        let ty2 = checker.check_expr(&expr, &ctx).unwrap();
        
        assert_eq!(ty1, ty2);
    }
    
    #[test]
    fn unbound_variable_always_fails(var_name in "[a-z]{1,10}") {
        let mut checker = TypeChecker::new();
        let ctx = TypeContext::new();  // Empty
        
        let expr = Expr::Variable(var_name);
        let result = checker.check_expr(&expr, &ctx);
        
        assert!(result.is_err());
    }
}
```

### Step 2: Implement TypeContext

**File:** `crates/ash-typeck/src/context.rs`

```rust
//! Type checking context with variable bindings

use std::collections::HashMap;
use ash_core::Type;

#[derive(Debug, Clone)]
pub struct TypeContext {
    bindings: HashMap<String, Type>,
    parent: Option<Box<TypeContext>>,
}

impl TypeContext {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            parent: None,
        }
    }
    
    pub fn with_parent(parent: &TypeContext) -> Self {
        Self {
            bindings: HashMap::new(),
            parent: Some(Box::new(parent.clone())),
        }
    }
    
    pub fn bind(&mut self, name: String, ty: Type) {
        self.bindings.insert(name, ty);
    }
    
    pub fn lookup(&self, name: &str) -> Option<&Type> {
        self.bindings.get(name)
            .or_else(|| self.parent.as_ref()?.lookup(name))
    }
    
    pub fn contains(&self, name: &str) -> bool {
        self.lookup(name).is_some()
    }
}

impl Default for TypeContext {
    fn default() -> Self {
        Self::new()
    }
}
```

### Step 3: Fix Expression Type Checking

**File:** `crates/ash-typeck/src/check_expr.rs`

```rust
use crate::context::TypeContext;

impl TypeChecker {
    pub fn check_expr(&mut self, expr: &Expr, ctx: &TypeContext) -> Result<Type, TypeError> {
        match expr {
            Expr::Variable(name) => {
                // FIX: Look up in environment
                ctx.lookup(name)
                    .cloned()
                    .ok_or_else(|| TypeError::UnboundVariable {
                        name: name.clone(),
                        span: expr.span(),
                    })
            }
            
            Expr::Literal(lit) => match lit {
                Literal::Int(_) => Ok(Type::Int),
                Literal::String(_) => Ok(Type::String),
                Literal::Bool(_) => Ok(Type::Bool),
                Literal::Float(_) => Ok(Type::Float),
                Literal::Unit => Ok(Type::Unit),
            },
            
            Expr::Binary { op, left, right } => {
                let left_ty = self.check_expr(left, ctx)?;
                let right_ty = self.check_expr(right, ctx)?;
                self.check_binary_op(*op, left_ty, right_ty, expr.span())
            }
            
            Expr::Unary { op, expr } => {
                let ty = self.check_expr(expr, ctx)?;
                self.check_unary_op(*op, ty, expr.span())
            }
            
            Expr::Call { func, args } => {
                self.check_call(func, args, ctx)
            }
            
            Expr::If { condition, then_branch, else_branch } => {
                let cond_ty = self.check_expr(condition, ctx)?;
                self.unify(&cond_ty, &Type::Bool)
                    .map_err(|_| TypeError::ExpectedBool {
                        actual: cond_ty,
                        span: condition.span(),
                    })?;
                
                let then_ty = self.check_expr(then_branch, ctx)?;
                
                match else_branch {
                    Some(else_expr) => {
                        let else_ty = self.check_expr(else_expr, ctx)?;
                        self.unify(&then_ty, &else_ty)
                            .map_err(|_| TypeError::BranchTypeMismatch {
                                then_type: then_ty,
                                else_type: else_ty,
                                span: expr.span(),
                            })?;
                        Ok(then_ty)
                    }
                    None => Ok(Type::Unit),
                }
            }
            
            Expr::Match { scrutinee, arms } => {
                self.check_match(scrutinee, arms, ctx)
            }
            
            Expr::Block(stmts) => {
                self.check_block(stmts, ctx)
            }
            
            Expr::FieldAccess { expr, field } => {
                self.check_field_access(expr, field, ctx)
            }
            
            Expr::Index { expr, index } => {
                self.check_index(expr, index, ctx)
            }
            
            Expr::Tuple(elems) => {
                let elem_types: Result<Vec<_>, _> = elems
                    .iter()
                    .map(|e| self.check_expr(e, ctx))
                    .collect();
                Ok(Type::Tuple(elem_types?))
            }
            
            Expr::Constructor { name, fields } => {
                self.check_constructor(name, fields, ctx)
            }
            
            // FIX: Explicitly list all expression types
            // If a type is genuinely not yet implemented, return an error
            _ => Err(TypeError::UnsupportedExpression {
                expr_type: expr.kind_name(),
                span: expr.span(),
            }),
        }
    }
    
    fn check_binary_op(&mut self, op: BinOp, left: Type, right: Type, span: Span) 
        -> Result<Type, TypeError> {
        match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => {
                // Arithmetic: both operands must be numeric
                if !left.is_numeric() {
                    return Err(TypeError::ExpectedNumeric { actual: left, span });
                }
                if !right.is_numeric() {
                    return Err(TypeError::ExpectedNumeric { actual: right, span });
                }
                // Result is the more general type
                Ok(Type::unify_numeric(left, right))
            }
            BinOp::Eq | BinOp::Neq | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                // Comparison: types must match, result is Bool
                self.unify(&left, &right)
                    .map_err(|_| TypeError::TypeMismatch { expected: left, actual: right, span })?;
                Ok(Type::Bool)
            }
            BinOp::And | BinOp::Or => {
                // Logical: both must be Bool
                self.unify(&left, &Type::Bool)
                    .map_err(|_| TypeError::ExpectedBool { actual: left, span })?;
                self.unify(&right, &Type::Bool)
                    .map_err(|_| TypeError::ExpectedBool { actual: right, span })?;
                Ok(Type::Bool)
            }
        }
    }
}
```

## Verification Steps

- [ ] `cargo test -p ash-typeck --test sound_expression_typing_test` passes
- [ ] Unbound variables produce errors
- [ ] All expression forms properly handled
- [ ] No silent fresh type variable fallbacks
- [ ] `cargo clippy --all-targets --all-features` clean
- [ ] `cargo fmt --check` clean

## Dependencies for Next Task

This task outputs:
- Sound expression type checking
- SPEC-003 compliance for variable resolution

Required by:
- All workflow type checking
- User-defined workflow validation

## Notes

**Critical Issue**: This is a fundamental type soundness violation. The type checker currently accepts invalid programs.

**Risk Assessment**: Critical - type system is unsound.

**Implementation Strategy**:
1. First: Implement TypeContext for environment tracking
2. Second: Remove fresh type variable fallback
3. Third: Implement proper expression checking
4. Fourth: Add comprehensive soundness tests
