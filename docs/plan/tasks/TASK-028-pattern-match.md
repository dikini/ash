# TASK-028: Pattern Matching Engine

## Status: ✅ Complete

## Description

Implement the pattern matching engine that binds values to patterns and extracts variables.

## Specification Reference

- SPEC-004: Operational Semantics - Section 5.1 Pattern Binding
- SPEC-001: IR - Section 2.4 Patterns

## Requirements

### Pattern Match Result

```rust
/// Result of pattern matching
#[derive(Debug, Clone)]
pub struct MatchResult {
    /// Whether the pattern matched
    pub matched: bool,
    /// Bindings extracted from the match
    pub bindings: Vec<(Box<str>, Value)>,
}

impl MatchResult {
    pub fn success(bindings: Vec<(Box<str>, Value)>) -> Self {
        Self { matched: true, bindings }
    }
    
    pub fn failure() -> Self {
        Self { matched: false, bindings: vec![] }
    }
}
```

### Pattern Matching

```rust
/// Match a value against a pattern
pub fn match_pattern(pat: &Pattern, value: &Value) -> Result<MatchResult, MatchError> {
    match pat {
        Pattern::Wildcard => Ok(MatchResult::success(vec![])),
        
        Pattern::Variable(name) => {
            // Variable binds any value
            Ok(MatchResult::success(vec![(name.clone(), value.clone())]))
        }
        
        Pattern::Literal(lit) => {
            if value == lit {
                Ok(MatchResult::success(vec![]))
            } else {
                Ok(MatchResult::failure())
            }
        }
        
        Pattern::Tuple(pats) => {
            match value {
                Value::List(items) if items.len() == pats.len() => {
                    let mut all_bindings = vec![];
                    for (pat, item) in pats.iter().zip(items.iter()) {
                        let result = match_pattern(pat, item)?;
                        if !result.matched {
                            return Ok(MatchResult::failure());
                        }
                        all_bindings.extend(result.bindings);
                    }
                    Ok(MatchResult::success(all_bindings))
                }
                _ => Ok(MatchResult::failure()),
            }
        }
        
        Pattern::Record(field_pats) => {
            match value {
                Value::Record(fields) => {
                    let mut all_bindings = vec![];
                    for (name, pat) in field_pats {
                        match fields.get(name) {
                            Some(field_val) => {
                                let result = match_pattern(pat, field_val)?;
                                if !result.matched {
                                    return Ok(MatchResult::failure());
                                }
                                all_bindings.extend(result.bindings);
                            }
                            None => return Ok(MatchResult::failure()),
                        }
                    }
                    Ok(MatchResult::success(all_bindings))
                }
                _ => Ok(MatchResult::failure()),
            }
        }
        
        Pattern::List { elements, rest } => {
            match value {
                Value::List(items) => {
                    if items.len() < elements.len() {
                        return Ok(MatchResult::failure());
                    }
                    
                    let mut all_bindings = vec![];
                    
                    // Match fixed elements
                    for (pat, item) in elements.iter().zip(items.iter()) {
                        let result = match_pattern(pat, item)?;
                        if !result.matched {
                            return Ok(MatchResult::failure());
                        }
                        all_bindings.extend(result.bindings);
                    }
                    
                    // Bind rest if present
                    if let Some(rest_name) = rest {
                        let rest_items: Box<_> = items[elements.len()..].to_vec().into();
                        all_bindings.push((rest_name.clone(), Value::List(rest_items)));
                    } else if items.len() > elements.len() {
                        // No rest pattern but extra items
                        return Ok(MatchResult::failure());
                    }
                    
                    Ok(MatchResult::success(all_bindings))
                }
                _ => Ok(MatchResult::failure()),
            }
        }
    }
}

/// Apply bindings to environment
pub fn apply_bindings(env: &Environment, bindings: &[(Box<str>, Value)]) -> Environment {
    env.bind_many(bindings.iter().cloned())
}
```

### Exhaustiveness Checking

```rust
/// Check if a set of patterns is exhaustive for a type
pub fn is_exhaustive(pats: &[Pattern], ty: &Type) -> bool {
    match ty {
        Type::Bool => {
            let has_true = pats.iter().any(|p| matches!(p, Pattern::Literal(Value::Bool(true))));
            let has_false = pats.iter().any(|p| matches!(p, Pattern::Literal(Value::Bool(false))));
            let has_wildcard = pats.iter().any(|p| matches!(p, Pattern::Wildcard | Pattern::Variable(_)));
            (has_true && has_false) || has_wildcard
        }
        
        Type::Null => {
            pats.iter().any(|p| matches!(p, 
                Pattern::Literal(Value::Null) | 
                Pattern::Wildcard | 
                Pattern::Variable(_)
            ))
        }
        
        // For other types, we need a wildcard or variable
        _ => pats.iter().any(|p| matches!(p, Pattern::Wildcard | Pattern::Variable(_))),
    }
}

/// Check for unreachable patterns
pub fn find_unreachable(pats: &[Pattern]) -> Vec<usize> {
    let mut unreachable = vec![];
    let mut covered = vec![];
    
    for (i, pat) in pats.iter().enumerate() {
        // Check if this pattern is already covered by previous patterns
        if is_covered_by_any(pat, &covered) {
            unreachable.push(i);
        } else {
            covered.push(pat.clone());
        }
    }
    
    unreachable
}

fn is_covered_by_any(pat: &Pattern, covered: &[Pattern]) -> bool {
    covered.iter().any(|c| covers(c, pat))
}

/// Check if pattern a covers pattern b (a is more general than b)
fn covers(a: &Pattern, b: &Pattern) -> bool {
    match (a, b) {
        (Pattern::Wildcard, _) => true,
        (Pattern::Variable(_), _) => true,
        (Pattern::Literal(l1), Pattern::Literal(l2)) => l1 == l2,
        (Pattern::Tuple(t1), Pattern::Tuple(t2)) if t1.len() == t2.len() => {
            t1.iter().zip(t2.iter()).all(|(a, b)| covers(a, b))
        }
        _ => false,
    }
}
```

### Match Errors

```rust
#[derive(Debug, Clone, thiserror::Error)]
pub enum MatchError {
    #[error("Pattern match failed: {reason}")]
    MatchFailed { reason: String },
    
    #[error("Type mismatch in pattern: expected {expected}, found {actual}")]
    TypeMismatch { expected: String, actual: String },
    
    #[error("Pattern not exhaustive")]
    NotExhaustive,
    
    #[error("Unreachable pattern at index {0}")]
    UnreachablePattern(usize),
}
```

## TDD Steps

### Step 1: Implement match_pattern

Create `crates/ash-interp/src/pattern.rs` with pattern matching.

### Step 2: Implement Exhaustiveness

Add is_exhaustive and find_unreachable.

### Step 3: Write Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wildcard_matches_anything() {
        let pat = Pattern::Wildcard;
        let value = Value::Int(42);
        
        let result = match_pattern(&pat, &value).unwrap();
        assert!(result.matched);
        assert!(result.bindings.is_empty());
    }

    #[test]
    fn test_variable_binds_value() {
        let pat = Pattern::Variable("x".into());
        let value = Value::Int(42);
        
        let result = match_pattern(&pat, &value).unwrap();
        assert!(result.matched);
        assert_eq!(result.bindings, vec![("x".into(), Value::Int(42))]);
    }

    #[test]
    fn test_literal_matches_equal() {
        let pat = Pattern::Literal(Value::Int(42));
        let value = Value::Int(42);
        
        let result = match_pattern(&pat, &value).unwrap();
        assert!(result.matched);
    }

    #[test]
    fn test_literal_no_match_unequal() {
        let pat = Pattern::Literal(Value::Int(42));
        let value = Value::Int(43);
        
        let result = match_pattern(&pat, &value).unwrap();
        assert!(!result.matched);
    }

    #[test]
    fn test_tuple_matches() {
        let pat = Pattern::Tuple(vec![
            Pattern::Variable("a".into()),
            Pattern::Variable("b".into()),
        ]);
        let value = Value::List(vec![Value::Int(1), Value::Int(2)].into());
        
        let result = match_pattern(&pat, &value).unwrap();
        assert!(result.matched);
        assert_eq!(result.bindings.len(), 2);
    }

    #[test]
    fn test_list_with_rest() {
        let pat = Pattern::List {
            elements: vec![Pattern::Variable("head".into())],
            rest: Some("tail".into()),
        };
        let value = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)].into());
        
        let result = match_pattern(&pat, &value).unwrap();
        assert!(result.matched);
        
        let head = result.bindings.iter().find(|(n, _)| n == "head");
        let tail = result.bindings.iter().find(|(n, _)| n == "tail");
        
        assert_eq!(head.map(|(_, v)| v), Some(&Value::Int(1)));
        assert!(tail.is_some());
    }

    #[test]
    fn test_record_pattern() {
        let pat = Pattern::Record(vec![
            ("name".into(), Pattern::Variable("n".into())),
        ]);
        let mut fields = HashMap::new();
        fields.insert("name".into(), Value::String("test".into()));
        let value = Value::Record(fields);
        
        let result = match_pattern(&pat, &value).unwrap();
        assert!(result.matched);
        assert_eq!(result.bindings, vec![("n".into(), Value::String("test".into()))]);
    }

    #[test]
    fn test_exhaustive_bool() {
        let pats = vec![
            Pattern::Literal(Value::Bool(true)),
            Pattern::Literal(Value::Bool(false)),
        ];
        
        assert!(is_exhaustive(&pats, &Type::Bool));
    }

    #[test]
    fn test_not_exhaustive_bool() {
        let pats = vec![Pattern::Literal(Value::Bool(true))];
        
        assert!(!is_exhaustive(&pats, &Type::Bool));
    }

    #[test]
    fn test_unreachable_pattern() {
        let pats = vec![
            Pattern::Wildcard,
            Pattern::Variable("x".into()),
        ];
        
        let unreachable = find_unreachable(&pats);
        assert_eq!(unreachable, vec![1]);
    }
}
```

## Completion Checklist

- [ ] match_pattern for all pattern types
- [ ] MatchResult with bindings
- [ ] apply_bindings to environment
- [ ] is_exhaustive for exhaustiveness checking
- [ ] find_unreachable for unreachable patterns
- [ ] covers relation
- [ ] MatchError types
- [ ] Unit tests for each pattern type
- [ ] Exhaustiveness tests
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Self-Review Questions

1. **Completeness**: Are all pattern types matchable?
2. **Bindings**: Are all variables extracted correctly?
3. **Exhaustiveness**: Is exhaustiveness checking accurate?

## Estimated Effort

6 hours

## Dependencies

- ash-core: Pattern, Value types
- TASK-026: Runtime context (uses Environment)

## Blocked By

- ash-core: Core types
- TASK-026: Runtime context

## Blocks

- TASK-029: Guards (uses pattern matching)
- All workflow execution tasks
