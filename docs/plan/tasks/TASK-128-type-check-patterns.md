# TASK-128: Type Check Patterns

## Status: 🟡 Ready to Start

## Description

Implement type checking for patterns in match expressions, including binding variables and checking exhaustiveness.

## Specification Reference

- SPEC-020: ADT Types - Section 6.2, 6.3

## Requirements

Type check patterns and report exhaustiveness errors:
```ash
match opt {              -- Error: non-exhaustive
    Some { value: x } => x
}                        -- Missing: None

match opt {              -- OK: exhaustive
    Some { value: x } => x,
    None => 0
}
```

## TDD Steps

### Step 1: Pattern Type Checking

**File**: `crates/ash-typeck/src/check_pattern.rs` (new)

```rust
//! Pattern type checking

use ash_core::ast::Pattern;
use ash_typeck::types::{Type, TypeVar, Substitution};
use std::collections::HashMap;

/// Result of pattern type checking
pub struct PatternResult {
    /// Bindings from pattern variables to their types
    pub bindings: HashMap<String, Type>,
    /// Effect of pattern matching (usually epistemic)
    pub effect: Effect,
}

/// Type check a pattern against an expected type
pub fn check_pattern(
    env: &TypeEnv,
    pattern: &Pattern,
    expected: &Type,
) -> Result<PatternResult, TypeError> {
    let mut bindings = HashMap::new();
    let effect = check_pattern_inner(env, pattern, expected, &mut bindings)?;
    
    Ok(PatternResult { bindings, effect })
}

fn check_pattern_inner(
    env: &TypeEnv,
    pattern: &Pattern,
    expected: &Type,
    bindings: &mut HashMap<String, Type>,
) -> Result<Effect, TypeError> {
    match (pattern, expected) {
        // Wildcard matches anything
        (Pattern::Wildcard, _) => Ok(Effect::Epistemic),
        
        // Variable binds to the expected type
        (Pattern::Variable(name), ty) => {
            bindings.insert(name.clone(), ty.clone());
            Ok(Effect::Epistemic)
        }
        
        // Literal must match type
        (Pattern::Literal(val), ty) => {
            let val_ty = value_to_type(val);
            if types_compatible(ty, &val_ty) {
                Ok(Effect::Epistemic)
            } else {
                Err(TypeError::PatternMismatch {
                    pattern: pattern.to_string(),
                    expected: ty.to_string(),
                    actual: val_ty.to_string(),
                })
            }
        }
        
        // Tuple pattern
        (Pattern::Tuple(patterns), Type::Tuple(types)) => {
            if patterns.len() != types.len() {
                return Err(TypeError::PatternArityMismatch {
                    expected: types.len(),
                    actual: patterns.len(),
                });
            }
            
            let mut total_effect = Effect::Epistemic;
            for (p, t) in patterns.iter().zip(types.iter()) {
                let effect = check_pattern_inner(env, p, t, bindings)?;
                total_effect = total_effect.join(effect);
            }
            Ok(total_effect)
        }
        
        // Record pattern
        (Pattern::Record(field_patterns), Type::Struct { fields, .. }) => {
            check_record_pattern(env, field_patterns, fields, bindings)
        }
        (Pattern::Record(field_patterns), Type::Sum { variants, .. }) => {
            // For sum types, we need to know which variant
            // This is handled by the Variant pattern case
            Err(TypeError::InvalidPattern("record pattern on sum type".to_string()))
        }
        
        // Variant pattern
        (Pattern::Variant { name, fields }, Type::Sum { variants, .. }) => {
            let variant = variants.iter()
                .find(|v| v.name.as_ref() == name)
                .ok_or_else(|| TypeError::UnknownVariant(name.clone()))?;
            
            check_record_pattern(env, fields.as_ref().unwrap_or(&vec![]), &variant.fields, bindings)
        }
        
        // Mismatched pattern and type
        _ => Err(TypeError::PatternMismatch {
            pattern: pattern.to_string(),
            expected: expected.to_string(),
            actual: "unknown".to_string(),
        }),
    }
}

fn check_record_pattern(
    env: &TypeEnv,
    patterns: &[(String, Pattern)],
    fields: &[(Box<str>, Type)],
    bindings: &mut HashMap<String, Type>,
) -> Result<Effect, TypeError> {
    let mut total_effect = Effect::Epistemic;
    
    for (field_name, field_pattern) in patterns {
        let field_type = fields.iter()
            .find(|(n, _)| n.as_ref() == field_name)
            .map(|(_, t)| t)
            .ok_or_else(|| TypeError::UnknownField {
                constructor: "struct".to_string(),
                field: field_name.clone(),
            })?;
        
        let effect = check_pattern_inner(env, field_pattern, field_type, bindings)?;
        total_effect = total_effect.join(effect);
    }
    
    Ok(total_effect)
}
```

### Step 2: Exhaustiveness Checking

**File**: `crates/ash-typeck/src/exhaustiveness.rs` (new)

```rust
//! Exhaustiveness checking for pattern matches

use ash_core::ast::{Pattern, MatchArm};
use ash_typeck::types::{Type, Variant};

/// Coverage result
#[derive(Debug, Clone)]
pub enum Coverage {
    /// All cases covered
    Covered,
    /// Some cases missing
    Missing(Vec<Pattern>),
}

/// Check if patterns are exhaustive for a type
pub fn check_exhaustive(
    patterns: &[Pattern],
    scrutinee_type: &Type,
    env: &TypeEnv,
) -> Coverage {
    let mut matrix = PatternMatrix::new(patterns.len());
    for p in patterns {
        matrix.add_row(p);
    }
    
    match find_uncovered(&matrix, scrutinee_type, env) {
        None => Coverage::Covered,
        Some(witnesses) => Coverage::Missing(witnesses),
    }
}

/// Pattern matrix for exhaustiveness analysis
struct PatternMatrix {
    rows: Vec<Vec<PatternCell>>,
}

#[derive(Debug, Clone)]
enum PatternCell {
    Wildcard,
    Constructor(String, Vec<PatternCell>),
}

impl PatternMatrix {
    fn new(arity: usize) -> Self {
        Self { rows: vec![] }
    }
    
    fn add_row(&mut self, pattern: &Pattern) {
        let cells = pattern_to_cells(pattern);
        self.rows.push(cells);
    }
}

fn pattern_to_cells(pattern: &Pattern) -> Vec<PatternCell> {
    match pattern {
        Pattern::Wildcard | Pattern::Variable(_) => vec![PatternCell::Wildcard],
        Pattern::Literal(_) => vec![PatternCell::Constructor("literal".to_string(), vec![])],
        Pattern::Variant { name, fields } => {
            let field_cells = fields.as_ref()
                .map(|f| f.iter().flat_map(|(_, p)| pattern_to_cells(p)).collect())
                .unwrap_or_default();
            vec![PatternCell::Constructor(name.clone(), field_cells)]
        }
        Pattern::Tuple(patterns) => {
            vec![PatternCell::Constructor(
                "tuple".to_string(),
                patterns.iter().flat_map(pattern_to_cells).collect()
            )]
        }
        Pattern::Record(fields) => {
            vec![PatternCell::Constructor(
                "record".to_string(),
                fields.iter().flat_map(|(_, p)| pattern_to_cells(p)).collect()
            )]
        }
        Pattern::List(_, _) => {
            // Simplified: treat as wildcard for now
            vec![PatternCell::Wildcard]
        }
    }
}

/// Find uncovered patterns using matrix specialization
fn find_uncovered(
    matrix: &PatternMatrix,
    ty: &Type,
    env: &TypeEnv,
) -> Option<Vec<Pattern>> {
    match ty {
        Type::Sum { variants, .. } => {
            let covered_variants: Vec<_> = matrix.rows.iter()
                .filter_map(|row| match &row[0] {
                    PatternCell::Constructor(name, _) => Some(name.clone()),
                    _ => None,
                })
                .collect();
            
            let mut missing = vec![];
            for variant in variants {
                if !covered_variants.contains(&variant.name.to_string()) {
                    // This variant is not covered
                    missing.push(Pattern::Variant {
                        name: variant.name.to_string(),
                        fields: None,  // Simplified
                    });
                }
            }
            
            if missing.is_empty() {
                None
            } else {
                Some(missing)
            }
        }
        
        // For other types, assume covered if any pattern exists
        _ => None,
    }
}
```

### Step 3: Integrate into Match Type Checking

**File**: `crates/ash-typeck/src/check_expr.rs`

```rust
fn check_match(
    env: &TypeEnv,
    var_env: &mut VarEnv,
    scrutinee: &Expr,
    arms: &[MatchArm],
) -> Result<(Type, Effect), TypeError> {
    // Type check scrutinee
    let (scrut_ty, scrut_effect) = check_expr(env, var_env, scrutinee)?;
    
    // Check exhaustiveness
    let patterns: Vec<_> = arms.iter().map(|a| a.pattern.clone()).collect();
    match check_exhaustive(&patterns, &scrut_ty, env) {
        Coverage::Covered => {}
        Coverage::Missing(witnesses) => {
            return Err(TypeError::NonExhaustiveMatch {
                scrutinee_type: scrut_ty.to_string(),
                missing_patterns: witnesses.iter().map(|p| p.to_string()).collect(),
            });
        }
    }
    
    // Type check each arm
    let mut arm_types = vec![];
    let mut total_effect = scrut_effect;
    
    for arm in arms {
        // Create a new scope for this arm
        var_env.push_scope();
        
        // Check pattern and get bindings
        let pattern_result = check_pattern(env, &arm.pattern, &scrut_ty)?;
        
        // Add bindings to environment
        for (name, ty) in pattern_result.bindings {
            var_env.bind(name, ty);
        }
        
        // Type check arm body
        let (arm_ty, arm_effect) = check_expr(env, var_env, &arm.body)?;
        arm_types.push(arm_ty);
        total_effect = total_effect.join(arm_effect);
        
        var_env.pop_scope();
    }
    
    // All arms must have compatible types
    let result_type = arm_types.into_iter().try_fold(None, |acc, ty| {
        match acc {
            None => Ok(Some(ty)),
            Some(acc_ty) => {
                let subst = unify(&acc_ty, &ty)?;
                Ok(Some(subst.apply(&acc_ty)))
            }
        }
    })?.unwrap_or(Type::Null);
    
    Ok((result_type, total_effect))
}
```

### Step 4: Add Error Types

```rust
#[derive(Debug, Clone, Error)]
pub enum TypeError {
    // Existing...
    
    #[error("Non-exhaustive pattern match for type '{scrutinee_type}'")]
    #[diagnostic(help("Add arms for: {}", missing_patterns.join(", ")))]
    NonExhaustiveMatch {
        scrutinee_type: String,
        missing_patterns: Vec<String>,
    },
    
    #[error("Pattern mismatch: expected {expected}, got {actual}")]
    PatternMismatch { pattern: String, expected: String, actual: String },
    
    #[error("Pattern arity mismatch: expected {expected} elements, got {actual}")]
    PatternArityMismatch { expected: usize, actual: usize },
    
    #[error("Unknown variant: {0}")]
    UnknownVariant(String),
    
    #[error("Invalid pattern: {0}")]
    InvalidPattern(String),
}
```

### Step 5: Write Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exhaustive_match() {
        let env = TypeEnv::new();
        
        let patterns = vec![
            Pattern::Variant { name: "Some".into(), fields: None },
            Pattern::Variant { name: "None".into(), fields: None },
        ];
        
        let option_type = Type::Sum {
            name: "Option".into(),
            type_params: vec![TypeVar(0)],
            variants: vec![
                Variant { name: "Some".into(), fields: vec![] },
                Variant { name: "None".into(), fields: vec![] },
            ],
        };
        
        match check_exhaustive(&patterns, &option_type, &env) {
            Coverage::Covered => {}
            _ => panic!("Expected exhaustive"),
        }
    }

    #[test]
    fn test_non_exhaustive_match() {
        let env = TypeEnv::new();
        
        let patterns = vec![
            Pattern::Variant { name: "Some".into(), fields: None },
            // Missing None
        ];
        
        let option_type = Type::Sum {
            name: "Option".into(),
            type_params: vec![],
            variants: vec![
                Variant { name: "Some".into(), fields: vec![] },
                Variant { name: "None".into(), fields: vec![] },
            ],
        };
        
        match check_exhaustive(&patterns, &option_type, &env) {
            Coverage::Missing(missing) => {
                assert_eq!(missing.len(), 1);
            }
            _ => panic!("Expected non-exhaustive"),
        }
    }
}
```

### Step 6: Run Tests

```bash
cargo test -p ash-typeck exhaustiveness -- --nocapture
cargo test -p ash-typeck pattern -- --nocapture
```

## Completion Checklist

- [ ] Pattern type checking for all pattern types
- [ ] Variable binding extraction
- [ ] Type compatibility checking
- [ ] Exhaustiveness checking algorithm
- [ ] Missing pattern reporting
- [ ] Integration with match expression checking
- [ ] Error messages with helpful suggestions
- [ ] Unit tests for exhaustiveness
- [ ] Unit tests for pattern type checking
- [ ] Property tests for pattern soundness
- [ ] `cargo fmt` and `cargo clippy` pass

## Estimated Effort

8 hours

## Dependencies

- TASK-127 (Type Check Constructors)

## Blocked By

- TASK-127

## Blocks

- TASK-129 (Generic Type Instantiation)
- TASK-132 (Pattern Matching Engine)
