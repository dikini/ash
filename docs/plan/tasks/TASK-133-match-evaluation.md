# TASK-133: Match Expression Evaluation

## Status: 🟡 Ready to Start

## Description

Implement evaluation of match expressions and if-let desugaring.

## Specification Reference

- SPEC-020: ADT Types - Section 7.3

## Requirements

Evaluate match expressions:
```ash
match opt {
    Some { value: x } => x * 2,
    None => 0
}
```

And if-let (desugared to match):
```ash
if let Some { value: x } = opt then { x } else { 0 }
-- Desugars to:
match opt {
    Some { value: x } => x,
    _ => 0
}
```

## TDD Steps

### Step 1: Write Tests (Red)

**File**: `crates/ash-interp/src/eval.rs` (append tests)

```rust
#[cfg(test)]
mod match_tests {
    use super::*;
    use ash_core::ast::{Expr, MatchArm, Pattern};
    use ash_core::value::Value;

    #[test]
    fn test_eval_match_some() {
        let mut ctx = Context::new();
        
        // opt = Some { value: 42 }
        let opt = Value::Variant {
            type_name: "Option".into(),
            variant_name: "Some".into(),
            fields: {
                let mut f = HashMap::new();
                f.insert("value".into(), Value::Int(42));
                f
            },
        };
        ctx.bind("opt".to_string(), opt);
        
        // match opt { Some { value: x } => x * 2, None => 0 }
        let arms = vec![
            MatchArm {
                pattern: Pattern::Variant {
                    name: "Some".to_string(),
                    fields: Some(vec![("value".to_string(), Pattern::Variable("x".to_string()))]),
                },
                body: Expr::Binary {
                    op: BinaryOp::Mul,
                    left: Box::new(Expr::Variable("x".to_string())),
                    right: Box::new(Expr::Literal(Value::Int(2))),
                },
            },
            MatchArm {
                pattern: Pattern::Variant {
                    name: "None".to_string(),
                    fields: None,
                },
                body: Expr::Literal(Value::Int(0)),
            },
        ];
        
        let expr = Expr::Match {
            scrutinee: Box::new(Expr::Variable("opt".to_string())),
            arms,
        };
        
        let result = eval_expr(&mut ctx, &expr);
        assert_eq!(result.unwrap(), Value::Int(84));
    }

    #[test]
    fn test_eval_match_none() {
        let mut ctx = Context::new();
        
        // opt = None
        let opt = Value::Variant {
            type_name: "Option".into(),
            variant_name: "None".into(),
            fields: HashMap::new(),
        };
        ctx.bind("opt".to_string(), opt);
        
        // match opt { Some { value: x } => x, None => 0 }
        let arms = vec![
            MatchArm {
                pattern: Pattern::Variant {
                    name: "Some".to_string(),
                    fields: Some(vec![("value".to_string(), Pattern::Variable("x".to_string()))]),
                },
                body: Expr::Variable("x".to_string()),
            },
            MatchArm {
                pattern: Pattern::Variant {
                    name: "None".to_string(),
                    fields: None,
                },
                body: Expr::Literal(Value::Int(0)),
            },
        ];
        
        let expr = Expr::Match {
            scrutinee: Box::new(Expr::Variable("opt".to_string())),
            arms,
        };
        
        let result = eval_expr(&mut ctx, &expr);
        assert_eq!(result.unwrap(), Value::Int(0));
    }

    #[test]
    fn test_eval_if_let() {
        let mut ctx = Context::new();
        
        // opt = Some { value: 42 }
        let opt = Value::Variant {
            type_name: "Option".into(),
            variant_name: "Some".into(),
            fields: {
                let mut f = HashMap::new();
                f.insert("value".into(), Value::Int(42));
                f
            },
        };
        ctx.bind("opt".to_string(), opt);
        
        // if let Some { value: x } = opt then { x } else { 0 }
        let expr = Expr::IfLet {
            pattern: Pattern::Variant {
                name: "Some".to_string(),
                fields: Some(vec![("value".to_string(), Pattern::Variable("x".to_string()))]),
            },
            expr: Box::new(Expr::Variable("opt".to_string())),
            then_branch: Box::new(Expr::Variable("x".to_string())),
            else_branch: Box::new(Expr::Literal(Value::Int(0))),
        };
        
        let result = eval_expr(&mut ctx, &expr);
        assert_eq!(result.unwrap(), Value::Int(42));
    }
}
```

### Step 2: Implement Match Evaluation (Green)

**File**: `crates/ash-interp/src/eval.rs`

```rust
pub fn eval_expr(ctx: &mut Context, expr: &Expr) -> EvalResult<Value> {
    match expr {
        // Existing cases...
        
        Expr::Match { scrutinee, arms } => {
            eval_match(ctx, scrutinee, arms)
        }
        
        Expr::IfLet { pattern, expr: scrutinee, then_branch, else_branch } => {
            eval_if_let(ctx, pattern, scrutinee, then_branch, else_branch)
        }
        
        // ... rest
    }
}

fn eval_match(
    ctx: &mut Context,
    scrutinee: &Expr,
    arms: &[MatchArm],
) -> EvalResult<Value> {
    // Evaluate scrutinee
    let value = eval_expr(ctx, scrutinee)?;
    
    // Try each arm in order
    for arm in arms {
        match crate::pattern::match_pattern(&arm.pattern, &value) {
            Ok(bindings) => {
                // Arm matched - evaluate body with bindings
                ctx.push_scope();
                for (name, val) in bindings {
                    ctx.bind(name, val);
                }
                let result = eval_expr(ctx, &arm.body);
                ctx.pop_scope();
                return result;
            }
            Err(_) => {
                // Pattern didn't match, try next arm
                continue;
            }
        }
    }
    
    // No arm matched
    Err(EvalError::NonExhaustiveMatch {
        value: value.to_string(),
    })
}

fn eval_if_let(
    ctx: &mut Context,
    pattern: &Pattern,
    scrutinee: &Expr,
    then_branch: &Expr,
    else_branch: &Expr,
) -> EvalResult<Value> {
    let value = eval_expr(ctx, scrutinee)?;
    
    match crate::pattern::match_pattern(pattern, &value) {
        Ok(bindings) => {
            // Pattern matched - evaluate then branch
            ctx.push_scope();
            for (name, val) in bindings {
                ctx.bind(name, val);
            }
            let result = eval_expr(ctx, then_branch);
            ctx.pop_scope();
            result
        }
        Err(_) => {
            // Pattern didn't match - evaluate else branch
            eval_expr(ctx, else_branch)
        }
    }
}
```

### Step 3: Add NonExhaustiveMatch Error

**File**: `crates/ash-interp/src/error.rs`

```rust
#[derive(Debug, Clone, Error)]
pub enum EvalError {
    // Existing...
    
    #[error("Non-exhaustive match: no arm matched value {value}")]
    NonExhaustiveMatch { value: String },
}
```

### Step 4: Add MatchArm to AST

**File**: `crates/ash-core/src/ast.rs`

Ensure `MatchArm` and `Expr::Match` exist:

```rust
/// Match arm: pattern => expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Expr,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    // Existing...
    
    /// Match expression
    Match {
        scrutinee: Box<Expr>,
        arms: Vec<MatchArm>,
    },
    
    /// If-let expression
    IfLet {
        pattern: Pattern,
        expr: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
    },
}
```

### Step 5: Run Tests

```bash
cargo test -p ash-interp match_tests -- --nocapture
```

### Step 6: Add Property Tests

```rust
#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_match_wildcard_always_matches(v in arbitrary_value()) {
            let arms = vec![
                MatchArm {
                    pattern: Pattern::Wildcard,
                    body: Expr::Literal(v.clone()),
                },
            ];
            let expr = Expr::Match {
                scrutinee: Box::new(Expr::Literal(Value::Null)),
                arms,
            };
            
            let mut ctx = Context::new();
            let result = eval_expr(&mut ctx, &expr);
            prop_assert_eq!(result.unwrap(), v);
        }
        
        #[test]
        fn prop_match_first_matching_arm_wins(
            v in arbitrary_value(),
            body1 in arbitrary_value(),
            body2 in arbitrary_value()
        ) {
            let arms = vec![
                MatchArm {
                    pattern: Pattern::Wildcard,
                    body: Expr::Literal(body1.clone()),
                },
                MatchArm {
                    pattern: Pattern::Wildcard,
                    body: Expr::Literal(body2.clone()),
                },
            ];
            let expr = Expr::Match {
                scrutinee: Box::new(Expr::Literal(v)),
                arms,
            };
            
            let mut ctx = Context::new();
            let result = eval_expr(&mut ctx, &expr);
            // First arm should win
            prop_assert_eq!(result.unwrap(), body1);
        }
    }
}
```

### Step 7: Commit

```bash
git add crates/ash-interp/src/eval.rs crates/ash-interp/src/error.rs
git commit -m "feat(interp): match and if-let evaluation (TASK-133)"
```

## Completion Checklist

- [ ] `eval_match` function
- [ ] `eval_if_let` function
- [ ] `NonExhaustiveMatch` error
- [ ] `MatchArm` in AST
- [ ] `Expr::Match` in AST
- [ ] `Expr::IfLet` in AST
- [ ] Try arms in order
- [ ] Bindings scoped to arm body
- [ ] Fallback to else branch for if-let
- [ ] Unit tests for match with Some
- [ ] Unit tests for match with None
- [ ] Unit tests for if-let
- [ ] Property tests for wildcard matching
- [ ] Documentation comments
- [ ] `cargo fmt` and `cargo clippy` pass

## Estimated Effort

5 hours

## Dependencies

- TASK-126 (Parse If-Let)
- TASK-132 (Pattern Matching Engine)

## Blocked By

- TASK-126
- TASK-132

## Blocks

- None (end of interpreter chain)
