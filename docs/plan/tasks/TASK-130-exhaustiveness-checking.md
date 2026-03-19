# TASK-130: Exhaustiveness Checking

## Status: 🟡 Ready

## Description

Implement exhaustiveness checking for pattern matches to ensure all cases are covered.

## Specification Reference

- SPEC-020: ADT Types - Section 6.3

## Requirements

Check if match arms cover all possible values:
```ash
match opt {
    Some { value: x } => x  -- ERROR: Missing None
}

match opt {
    Some { value: x } => x,
    None => 0  -- OK: All cases covered
}
```

## TDD Steps

### Step 1: Write Tests (Red)

**File**: `crates/ash-typeck/src/exhaustiveness.rs` (new)

```rust
//! Exhaustiveness checking for pattern matches

use ash_core::ast::Pattern;
use crate::types::{Type, Variant};

/// Coverage result
#[derive(Debug, Clone, PartialEq)]
pub enum Coverage {
    /// All cases covered
    Covered,
    /// Some cases missing
    Missing(Vec<Pattern>),
}

/// Check if patterns cover all cases for a type
pub fn check_exhaustive(patterns: &[Pattern], scrutinee_type: &Type) -> Coverage {
    let matrix = PatternMatrix::new(patterns);
    
    match find_uncovered(&matrix, scrutinee_type) {
        None => Coverage::Covered,
        Some(witnesses) => Coverage::Missing(witnesses),
    }
}

// Implementation details...
struct PatternMatrix { /* ... */ }

#[cfg(test)]
mod tests {
    use super::*;
    use ash_core::ast::Pattern;
    use crate::types::{Type, TypeVar, Variant};

    fn make_option_type() -> Type {
        Type::Sum {
            name: "Option".into(),
            type_params: vec![TypeVar(0)],
            variants: vec![
                Variant {
                    name: "Some".into(),
                    fields: vec![("value".into(), Type::Var(TypeVar(0)))],
                },
                Variant { name: "None".into(), fields: vec![] },
            ],
        }
    }

    #[test]
    fn test_exhaustive_full_coverage() {
        let option_type = make_option_type();
        let patterns = vec![
            Pattern::Variant { name: "Some".into(), fields: None },
            Pattern::Variant { name: "None".into(), fields: None },
        ];
        
        assert_eq!(
            check_exhaustive(&patterns, &option_type),
            Coverage::Covered
        );
    }

    #[test]
    fn test_non_exhaustive_missing_variant() {
        let option_type = make_option_type();
        let patterns = vec![
            Pattern::Variant { name: "Some".into(), fields: None },
            // Missing None
        ];
        
        match check_exhaustive(&patterns, &option_type) {
            Coverage::Missing(missing) => {
                assert_eq!(missing.len(), 1);
            }
            _ => panic!("Expected Missing coverage"),
        }
    }

    #[test]
    fn test_exhaustive_with_wildcard() {
        let option_type = make_option_type();
        let patterns = vec![
            Pattern::Variant { name: "Some".into(), fields: None },
            Pattern::Wildcard,
        ];
        
        assert_eq!(
            check_exhaustive(&patterns, &option_type),
            Coverage::Covered
        );
    }
}
```

### Step 2: Implement Pattern Matrix (Green)

```rust
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
    fn new(patterns: &[Pattern]) -> Self {
        let rows = patterns.iter()
            .map(|p| vec![pattern_to_cell(p)])
            .collect();
        Self { rows }
    }
}

fn pattern_to_cell(pattern: &Pattern) -> PatternCell {
    match pattern {
        Pattern::Wildcard | Pattern::Variable(_) => PatternCell::Wildcard,
        Pattern::Variant { name, fields } => {
            let field_cells = fields.as_ref()
                .map(|f| f.iter().map(|(_, p)| pattern_to_cell(p)).collect())
                .unwrap_or_default();
            PatternCell::Constructor(name.clone(), field_cells)
        }
        Pattern::Tuple(patterns) => {
            PatternCell::Constructor(
                "tuple".to_string(),
                patterns.iter().map(pattern_to_cell).collect()
            )
        }
        Pattern::Literal(_) => {
            PatternCell::Constructor("literal".to_string(), vec![])
        }
        _ => PatternCell::Wildcard,
    }
}
```

### Step 3: Implement find_uncovered (Green)

```rust
fn find_uncovered(matrix: &PatternMatrix, ty: &Type) -> Option<Vec<Pattern>> {
    match ty {
        Type::Sum { variants, .. } => {
            let covered: Vec<_> = matrix.rows.iter()
                .filter_map(|row| match &row[0] {
                    PatternCell::Constructor(name, _) => Some(name.clone()),
                    _ => None,
                })
                .collect();
            
            let mut missing = vec![];
            for variant in variants {
                if !covered.contains(&variant.name.to_string()) {
                    missing.push(Pattern::Variant {
                        name: variant.name.to_string(),
                        fields: None,
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
        _ => {
            if matrix.rows.is_empty() {
                Some(vec![Pattern::Wildcard])
            } else {
                None
            }
        }
    }
}
```

### Step 4: Add TypeError for Non-Exhaustive Match

**File**: `crates/ash-typeck/src/error.rs`

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
}
```

### Step 5: Integrate into Match Type Checking

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
    match check_exhaustive(&patterns, &scrut_ty) {
        Coverage::Covered => {}
        Coverage::Missing(witnesses) => {
            return Err(TypeError::NonExhaustiveMatch {
                scrutinee_type: scrut_ty.to_string(),
                missing_patterns: witnesses.iter().map(|p| p.to_string()).collect(),
            });
        }
    }
    
    // Check each arm...
    // ... rest of implementation
}
```

### Step 6: Run Tests

```bash
cargo test -p ash-typeck exhaustiveness -- --nocapture
```

### Step 7: Commit

```bash
git add crates/ash-typeck/src/exhaustiveness.rs crates/ash-typeck/src/error.rs
git commit -m "feat(typeck): exhaustiveness checking for patterns (TASK-130)"
```

## Completion Checklist

- [ ] `Coverage` enum (Covered/Missing)
- [ ] `PatternMatrix` data structure
- [ ] `pattern_to_cell` conversion
- [ ] `find_uncovered` algorithm
- [ ] `NonExhaustiveMatch` error type
- [ ] Integration into match type checking
- [ ] Unit tests for exhaustive match
- [ ] Unit tests for non-exhaustive match
- [ ] Unit tests for wildcard coverage
- [ ] Property tests for coverage soundness
- [ ] Documentation comments
- [ ] `cargo fmt` and `cargo clippy` pass

## Estimated Effort

8 hours

## Dependencies

- TASK-128 (Type Check Patterns)
- TASK-129 (Generic Instantiation)

## Blocked By

- TASK-128
- TASK-129

## Blocks

- None (end of type checker chain)
