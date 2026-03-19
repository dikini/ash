# TASK-005: Pattern Matching System

## Status: ✅ Complete

## Description

Implement the Pattern enum for destructuring values in workflows.

## Specification Reference

- SPEC-001: IR - Section 2.4 Patterns

## Requirements

### Functional Requirements

1. `Pattern` enum with variants:
   - `Variable(Name)` - Bind any value to name
   - `Tuple(Box<[Pattern]>)` - Match tuple/list of patterns
   - `Record(Box<[(Name, Pattern)]>)` - Match record with field patterns
   - `List(Box<[Pattern]>, Option<Name>)` - Match list prefix, bind rest
   - `Wildcard` - Match any, no binding
   - `Literal(Value)` - Match exact value

2. Matching rules (documented, tested):
   - Variable: binds any value
   - Tuple: matches same-length list
   - Record: matches superset of fields
   - List: matches exact prefix, binds rest if specified
   - Wildcard: matches any, no binding
   - Literal: matches equal value

3. Utility methods:
   - `bindings()` - List of variable names bound by pattern
   - `is_refutable()` - Can this pattern fail to match?

### Property Requirements (proptest)

```rust
// Pattern binding uniqueness
// - No duplicate bindings in a pattern

// Pattern exhaustiveness (simple cases)
// - Variable and Wildcard are exhaustive
// - Literal is not exhaustive alone

// Pattern overlap
// - Variable overlaps with everything
// - Wildcard overlaps with everything
// - Specific literals don't overlap with different literals
```

## TDD Steps

### Step 1: Define Pattern Enum (Green)

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Pattern {
    Variable(Name),
    Tuple(Box<[Pattern]>),
    Record(Box<[(Name, Pattern)]>),
    List(Box<[Pattern]>, Option<Name>),
    Wildcard,
    Literal(Value),
}
```

### Step 2: Implement binding analysis (Green)

```rust
impl Pattern {
    /// Returns all variable names bound by this pattern
    pub fn bindings(&self) -> Vec<Name> { ... }
    
    /// Returns true if pattern can fail to match some value
    pub fn is_refutable(&self) -> bool { ... }
}
```

### Step 3: Write Property Tests (Red→Green)

```rust
proptest! {
    #[test]
    fn prop_bindings_unique(pat in arb_pattern()) {
        let bindings = pat.bindings();
        let unique: HashSet<_> = bindings.iter().collect();
        prop_assert_eq!(bindings.len(), unique.len());
    }
    
    #[test]
    fn prop_variable_is_irrefutable() {
        prop_assert!(!Pattern::Variable("x".into()).is_refutable());
    }
    
    #[test]
    fn prop_wildcard_is_irrefutable() {
        prop_assert!(!Pattern::Wildcard.is_refutable());
    }
}
```

### Step 4: Refactor (Refactor)

- Ensure efficient binding collection
- Consider Box usage for nested patterns

## Completion Checklist

- [ ] Pattern enum with all 6 variants
- [ ] `bindings()` method
- [ ] `is_refutable()` method
- [ ] Binding uniqueness property tests
- [ ] Irrefutability tests
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes
- [ ] Documentation with matching rules

## Estimated Effort

6 hours (matching logic can be complex)

## Dependencies

- TASK-002 (Value used in Literal pattern)
- TASK-003 (Name type)

## Blocked By

- TASK-002
- TASK-003

## Blocks

- TASK-003 (Workflow uses Pattern - circular but Pattern can be done after)
- TASK-016 (Lowering uses patterns)
- TASK-028 (Pattern matching engine)
