# TASK-097: Schema Validation Logic

## Status: ✅ Complete

## Description

Implement runtime validation that checks if a `Value` matches a `Type` schema.

## Specification Reference

- SPEC-015: Typed Providers - Section 3.3 Validation

## Requirements

### Functional Requirements

1. `Type::matches(&self, value: &Value) -> bool` method
2. Validation for all type variants (Int, String, Record, List, etc.)
3. Recursive validation for nested types
4. Detailed error reporting (which field failed, expected vs actual)

### Property Requirements

```rust
assert!(Type::Int.matches(&Value::Int(42)));
assert!(!Type::Int.matches(&Value::String("hello")));

let schema = Type::Record(vec![("x".into(), Type::Int)]);
assert!(schema.matches(&Value::Record(hashmap! {"x".into() => Value::Int(1)})));
```

## TDD Steps

### Step 1: Write Tests (Red)

```rust
#[test]
fn test_int_matches() {
    assert!(Type::Int.matches(&Value::Int(42)));
    assert!(!Type::Int.matches(&Value::String("42")));
}

#[test]
fn test_string_matches() {
    assert!(Type::String.matches(&Value::String("hello")));
    assert!(!Type::String.matches(&Value::Int(42)));
}

#[test]
fn test_list_matches() {
    let schema = Type::List(Box::new(Type::Int));
    assert!(schema.matches(&Value::List(vec![Value::Int(1), Value::Int(2)])));
    assert!(!schema.matches(&Value::List(vec![Value::String("a")])));
}

#[test]
fn test_record_matches() {
    let schema = Type::Record(vec![
        ("name".into(), Type::String),
        ("age".into(), Type::Int),
    ]);
    
    let valid = Value::Record(hashmap! {
        "name".into() => Value::String("Alice"),
        "age".into() => Value::Int(30),
    });
    assert!(schema.matches(&valid));
    
    let invalid = Value::Record(hashmap! {
        "name".into() => Value::Int(30), // Wrong type
        "age".into() => Value::Int(30),
    });
    assert!(!schema.matches(&invalid));
}

#[test]
fn test_nested_record() {
    let schema = Type::Record(vec![
        ("point".into(), Type::Record(vec![
            ("x".into(), Type::Int),
            ("y".into(), Type::Int),
        ])),
    ]);
    
    let value = Value::Record(hashmap! {
        "point".into() => Value::Record(hashmap! {
            "x".into() => Value::Int(1),
            "y".into() => Value::Int(2),
        }),
    });
    assert!(schema.matches(&value));
}
```

### Step 2: Verify RED

Expected: FAIL - `matches` method not implemented

### Step 3: Implement (Green)

```rust
impl Type {
    /// Check if a value matches this type schema
    pub fn matches(&self, value: &Value) -> bool {
        match (self, value) {
            (Type::Int, Value::Int(_)) => true,
            (Type::String, Value::String(_)) => true,
            (Type::Bool, Value::Bool(_)) => true,
            (Type::Null, Value::Null) => true,
            (Type::Time, Value::Time(_)) => true,
            (Type::Ref, Value::Ref(_)) => true,
            (Type::Cap { .. }, Value::Cap(_)) => true,
            
            (Type::List(elem_type), Value::List(items)) => {
                items.iter().all(|item| elem_type.matches(item))
            }
            
            (Type::Record(fields), Value::Record(record)) => {
                fields.iter().all(|(name, ty)| {
                    record.get(name.as_ref())
                        .map(|val| ty.matches(val))
                        .unwrap_or(false)
                })
            }
            
            _ => false,
        }
    }
    
    /// Detailed validation with error message
    pub fn validate(&self, value: &Value) -> Result<(), TypeError> {
        if self.matches(value) {
            Ok(())
        } else {
            Err(TypeError::Mismatch {
                expected: self.to_string(),
                actual: value.to_string(),
            })
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TypeError {
    #[error("type mismatch: expected {expected}, got {actual}")]
    Mismatch { expected: String, actual: String },
    
    #[error("missing field '{field}' in record")]
    MissingField { field: String },
    
    #[error("field '{field}' type mismatch: expected {expected}, got {actual}")]
    FieldMismatch { field: String, expected: String, actual: String },
}
```

### Step 4: Verify GREEN

Expected: PASS

### Step 5: Commit

```bash
git commit -m "feat: type schema validation for values"
```

## Completion Checklist

- [ ] Type::matches implemented
- [ ] All primitive types covered
- [ ] List validation (recursive)
- [ ] Record validation (field checking)
- [ ] Detailed error types
- [ ] Tests pass
- [ ] `cargo fmt` clean
- [ ] `cargo clippy` clean

## Estimated Effort

4 hours

## Dependencies

None (adds to ash-core Type)

## Blocked By

Nothing

## Blocks

- TASK-099 (Runtime validation in providers)
