# TASK-135: Control Link Transfer Semantics

## Status: ✅ Complete

## Description

Implement control link transfer semantics for affine/linear tracking of control links.

> Historical note: the original task framed transfer through affine consumption. The canonical
> control-authority contract was later revised by TASK-211 so transferred `ControlLink` authority
> remains reusable for non-terminal supervision operations and is invalidated by terminal runtime
> state rather than unconditional first use.

## Specification Reference

- SPEC-020: ADT Types - Section 8.2, 8.3
- docs/design/CONTROL_LINK_TRANSFER.md

## Requirements

Track control link consumption:
```ash
spawn worker with {} as w;
let (w_addr, w_ctrl) = split w;

-- w_ctrl: Option<ControlLink> (initially Some)
if let Some { value: link } = w_ctrl then {
    send_control supervisor with link;
    -- After send, w_ctrl is logically consumed
}

-- Cannot use w_ctrl after transfer
```

## TDD Steps

### Step 1: Write Tests (Red)

**File**: `crates/ash-typeck/src/linearity.rs` (new)

```rust
//! Linearity checking for affine types (ControlLink)

use std::collections::HashMap;
use ash_core::ast::{Expr, Pattern};
use crate::types::Type;
use crate::error::TypeError;

/// Variable state for linearity checking
#[derive(Debug, Clone)]
pub enum VarState {
    Available(Type),
    Consumed,
}

/// Linear environment for tracking variable states
#[derive(Debug, Clone, Default)]
pub struct LinearEnv {
    bindings: HashMap<String, VarState>,
}

impl LinearEnv {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }
    
    /// Bind a variable as available
    pub fn bind(&mut self, name: String, ty: Type) {
        self.bindings.insert(name, VarState::Available(ty));
    }
    
    /// Consume a variable (for affine types)
    pub fn consume(&mut self, name: &str) -> Result<Type, TypeError> {
        match self.bindings.get(name) {
            Some(VarState::Available(ty)) => {
                // Only affine types need consumption tracking
                if is_affine(ty) {
                    self.bindings.insert(name.to_string(), VarState::Consumed);
                }
                Ok(ty.clone())
            }
            Some(VarState::Consumed) => {
                Err(TypeError::AlreadyConsumed(name.to_string()))
            }
            None => Err(TypeError::UnboundVariable(name.to_string())),
        }
    }
    
    /// Check if a type is affine (must be consumed exactly once)
    fn is_affine(ty: &Type) -> bool {
        match ty {
            Type::ControlLink { .. } => true,
            Type::Constructor { name, args } if name.as_ref() == "Option" => {
                // Option<ControlLink> is also affine
                args.iter().any(is_affine)
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bind_and_consume() {
        let mut env = LinearEnv::new();
        env.bind("x".to_string(), Type::ControlLink { workflow_type: "W".into() });
        
        // First consume should succeed
        assert!(env.consume("x").is_ok());
        
        // Second consume should fail
        assert!(env.consume("x").is_err());
    }

    #[test]
    fn test_non_affine_not_consumed() {
        let mut env = LinearEnv::new();
        env.bind("x".to_string(), Type::Int);
        
        // Multiple uses of non-affine type should succeed
        assert!(env.consume("x").is_ok());
        assert!(env.consume("x").is_ok());
    }
}
```

### Step 2: Implement Linearity Check (Green)

```rust
/// Check linearity in an expression
pub fn check_linearity(env: &mut LinearEnv, expr: &Expr) -> Result<(), TypeError> {
    match expr {
        Expr::Variable(name) => {
            // Using a variable - consume if affine
            env.consume(name)?;
            Ok(())
        }
        
        Expr::Match { scrutinee, arms } => {
            check_linearity(env, scrutinee)?;
            
            // Check each arm
            for arm in arms {
                let mut arm_env = env.clone();
                
                // Pattern may consume the scrutinee if it's affine
                check_pattern_linearity(&mut arm_env, &arm.pattern)?;
                
                check_linearity(&mut arm_env, &arm.body)?;
            }
            
            // Merge arm environments (conservative: variable must be consumed in all arms)
            // ...
            
            Ok(())
        }
        
        Expr::IfLet { pattern, expr, then_branch, else_branch } => {
            check_linearity(env, expr)?;
            
            let mut then_env = env.clone();
            check_pattern_linearity(&mut then_env, pattern)?;
            check_linearity(&mut then_env, then_branch)?;
            
            check_linearity(env, else_branch)?;
            
            Ok(())
        }
        
        // Other expressions...
        _ => Ok(()),
    }
}

fn check_pattern_linearity(env: &mut LinearEnv, pattern: &Pattern) -> Result<(), TypeError> {
    // Pattern binding doesn't consume, just binds
    Ok(())
}
```

### Step 3: Add TypeError Variants

**File**: `crates/ash-typeck/src/error.rs`

```rust
#[derive(Debug, Clone, Error)]
pub enum TypeError {
    // Existing...
    
    #[error("Variable '{0}' has already been consumed")]
    AlreadyConsumed(String),
    
    #[error("Affine type '{0}' must be consumed exactly once")]
    AffineNotConsumed(String),
}
```

### Step 4: Integrate into Type Checking

**File**: `crates/ash-typeck/src/check_expr.rs`

```rust
pub fn check_expr(
    env: &TypeEnv,
    var_env: &mut VarEnv,
    linear_env: &mut LinearEnv,  // Add linear environment
    expr: &Expr,
) -> Result<(Type, Effect), TypeError> {
    match expr {
        Expr::Variable(name) => {
            // Track linearity
            linear_env.consume(name)?;
            
            // Existing type checking...
            let ty = var_env.lookup(name)
                .ok_or_else(|| TypeError::UnboundVariable(name.clone()))?;
            Ok((ty.clone(), Effect::Epistemic))
        }
        
        // Other cases...
        
        _ => Ok((Type::Null, Effect::Epistemic)),
    }
}
```

### Step 5: Run Tests

```bash
cargo test -p ash-typeck linearity -- --nocapture
```

### Step 6: Add Integration Tests

**File**: `tests/control_link_transfer.ash`

```ash
-- Test control link transfer
workflow worker {
    receive {
        _ => ret Done
    }
}

workflow supervisor {
    spawn worker with {} as w;
    let (w_addr, w_ctrl) = split w;
    
    -- w_ctrl is Option<ControlLink>
    if let Some { value: link } = w_ctrl then {
        -- Use link once
        act log with "Got control link";
        -- After this block, link is consumed
    } else {
        act log with "No control link";
    }
    
    ret Done;
}
```

### Step 7: Commit

```bash
git add crates/ash-typeck/src/linearity.rs crates/ash-typeck/src/error.rs
git commit -m "feat(typeck): control link transfer semantics (TASK-135)"
```

## Completion Checklist

- [ ] `VarState` enum (Available/Consumed)
- [ ] `LinearEnv` struct
- [ ] `LinearEnv::bind` for available variables
- [ ] `LinearEnv::consume` for affine consumption
- [ ] `is_affine` type check
- [ ] `AlreadyConsumed` error
- [ ] `AffineNotConsumed` error
- [ ] Linearity checking in match arms
- [ ] Linearity checking in if-let
- [ ] Integration with expression type checking
- [ ] Unit tests for consumption
- [ ] Unit tests for double-use prevention
- [ ] Integration test for control link workflow
- [ ] Documentation comments
- [ ] `cargo fmt` and `cargo clippy` pass

## Estimated Effort

5 hours

## Dependencies

- TASK-134 (Spawn Option ControlLink)

## Blocked By

- TASK-134

## Blocks

- TASK-136 (Option/Result Library - uses control links)
