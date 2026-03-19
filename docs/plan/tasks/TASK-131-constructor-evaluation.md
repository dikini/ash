# TASK-131: Constructor Evaluation

## Status: 🟡 Ready

## Description

Implement runtime evaluation of ADT constructors to create variant and struct values.

## Specification Reference

- SPEC-020: ADT Types - Section 7.1

## Requirements

Evaluate constructor expressions:
```ash
let x = Some { value: 42 };        -- Creates Value::Variant
let p = Point { x: 10, y: 20 };    -- Creates Value::Struct
let t = (1, "hello", true);        -- Creates Value::Tuple
```

## TDD Steps

### Step 1: Write Tests (Red)

**File**: `crates/ash-interp/src/constructor.rs` (new)

```rust
//! Constructor evaluation

use ash_core::ast::Expr;
use ash_core::value::Value;
use crate::{EvalError, EvalResult};
use crate::context::Context;

/// Evaluate a constructor expression
pub fn eval_constructor(
    ctx: &mut Context,
    name: &str,
    fields: &[(String, Expr)],
) -> EvalResult<Value> {
    // Implementation
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_core::ast::Expr;
    use ash_core::value::Value;

    #[test]
    fn test_eval_variant_constructor() {
        let mut ctx = Context::new();
        let fields = vec![
            ("value".to_string(), Expr::Literal(Value::Int(42))),
        ];
        
        let result = eval_constructor(&mut ctx, "Some", &fields);
        
        assert!(result.is_ok());
        match result.unwrap() {
            Value::Variant { type_name, variant_name, fields } => {
                assert_eq!(type_name.as_ref(), "Option");
                assert_eq!(variant_name.as_ref(), "Some");
                assert_eq!(fields.get("value"), Some(&Value::Int(42)));
            }
            _ => panic!("Expected Variant"),
        }
    }

    #[test]
    fn test_eval_struct_constructor() {
        let mut ctx = Context::new();
        let fields = vec![
            ("x".to_string(), Expr::Literal(Value::Int(10))),
            ("y".to_string(), Expr::Literal(Value::Int(20))),
        ];
        
        let result = eval_constructor(&mut ctx, "Point", &fields);
        
        assert!(result.is_ok());
        match result.unwrap() {
            Value::Struct { type_name, fields } => {
                assert_eq!(type_name.as_ref(), "Point");
                assert_eq!(fields.get("x"), Some(&Value::Int(10)));
                assert_eq!(fields.get("y"), Some(&Value::Int(20)));
            }
            _ => panic!("Expected Struct"),
        }
    }
}
```

### Step 2: Implement Constructor Evaluation (Green)

```rust
use std::collections::HashMap;

/// Evaluate a constructor expression
pub fn eval_constructor(
    ctx: &mut Context,
    name: &str,
    fields: &[(String, Expr)],
) -> EvalResult<Value> {
    // Evaluate each field expression
    let mut evaluated_fields = HashMap::new();
    for (field_name, field_expr) in fields {
        let value = eval_expr(ctx, field_expr)?;
        evaluated_fields.insert(field_name.clone().into_boxed_str(), value);
    }
    
    // Determine if this is a variant or struct constructor
    // For now, use naming convention: uppercase = variant, check type env
    if is_variant_constructor(ctx, name) {
        let type_name = ctx.lookup_constructor_type(name)
            .ok_or_else(|| EvalError::UnknownConstructor(name.to_string()))?;
        
        Ok(Value::Variant {
            type_name: type_name.into(),
            variant_name: name.into(),
            fields: evaluated_fields,
        })
    } else {
        Ok(Value::Struct {
            type_name: name.into(),
            fields: evaluated_fields,
        })
    }
}

fn is_variant_constructor(ctx: &Context, name: &str) -> bool {
    // Check if this constructor belongs to a sum type
    ctx.lookup_constructor_type(name).is_some()
}

fn eval_expr(ctx: &mut Context, expr: &Expr) -> EvalResult<Value> {
    // Delegate to main expression evaluator
    crate::eval_expr(ctx, expr)
}
```

### Step 3: Add Expr::Constructor to AST

**File**: `crates/ash-core/src/ast.rs`

Ensure `Expr` enum has constructor variant:

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    // Existing variants...
    
    /// Constructor expression: Some { value: 42 }
    Constructor {
        name: Name,
        fields: Vec<(Name, Expr)>,
    },
}
```

### Step 4: Integrate into Expression Evaluator

**File**: `crates/ash-interp/src/eval.rs`

```rust
pub fn eval_expr(ctx: &mut Context, expr: &Expr) -> EvalResult<Value> {
    match expr {
        // Existing cases...
        
        Expr::Constructor { name, fields } => {
            crate::constructor::eval_constructor(ctx, name, fields)
        }
        
        // ... rest
    }
}
```

### Step 5: Run Tests

```bash
cargo test -p ash-interp constructor -- --nocapture
```

### Step 6: Add Context Methods

**File**: `crates/ash-interp/src/context.rs`

```rust
impl Context {
    /// Lookup the type name for a constructor
    pub fn lookup_constructor_type(&self, name: &str) -> Option<&str> {
        // Look up in type environment
        self.type_env.lookup_constructor_type(name)
    }
}
```

### Step 7: Commit

```bash
git add crates/ash-interp/src/constructor.rs crates/ash-interp/src/eval.rs
git commit -m "feat(interp): constructor evaluation (TASK-131)"
```

## Completion Checklist

- [ ] `eval_constructor` function
- [ ] `Expr::Constructor` in AST
- [ ] Evaluate field expressions
- [ ] Create `Value::Variant` for enum constructors
- [ ] Create `Value::Struct` for struct constructors
- [ ] Constructor type lookup in Context
- [ ] Unit tests for variant constructor
- [ ] Unit tests for struct constructor
- [ ] Integration with expression evaluator
- [ ] Error handling for unknown constructors
- [ ] Documentation comments
- [ ] `cargo fmt` and `cargo clippy` pass

## Estimated Effort

4 hours

## Dependencies

- TASK-122 (ADT Runtime Values)
- TASK-127 (Type Check Constructors)

## Blocked By

- TASK-122
- TASK-127

## Blocks

- TASK-132 (Pattern Matching Engine)
- TASK-133 (Match Evaluation)
