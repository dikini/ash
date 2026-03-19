# TASK-002: Value System Implementation

## Status: ✅ Complete

## Description

Implement the Value enum with full serialization support and property tests.

## Specification Reference

- SPEC-001: IR - Section 2.3 Values

## Requirements

### Functional Requirements

1. `Value` enum with variants:
   - `Int(i64)` - 64-bit integer
   - `String(Box<str>)` - Boxed string for smaller enum size
   - `Bool(bool)` - Boolean
   - `Null` - Null value
   - `Time(DateTime<Utc>)` - Timestamp (from chrono)
   - `Ref(Box<str>)` - Reference to external resource
   - `List(Box<[Value]>)` - Immutable list
   - `Record(HashMap<Box<str>, Value>)` - Record/map
   - `Cap(Box<str>)` - Capability reference

2. Serialization support:
   - `Serialize`/`Deserialize` (serde)
   - JSON for debugging
   - Binary format consideration (bincode)

3. Utility methods:
   - `as_int()`, `as_string()`, `as_bool()` - Type accessors
   - `is_null()` - Null check

4. Display trait for human-readable output

### Property Requirements (proptest)

```rust
// Roundtrip serialization
serialize_deserialize(v) == v

// Display/parse roundtrip (if parser available)
// Value structure invariants
// - List is immutable
// - Record keys are valid identifiers
```

## TDD Steps

### Step 1: Write Property Tests (Red)

Create tests in `crates/ash-core/src/value.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    prop_compose! {
        fn arb_value()(n in 0u8..9) -> Value {
            // Generate various value types
        }
    }

    proptest! {
        #[test]
        fn prop_json_roundtrip(v in arb_value()) {
            let json = serde_json::to_string(&v).unwrap();
            let v2: Value = serde_json::from_str(&json).unwrap();
            assert_eq!(v, v2);
        }
    }
}
```

### Step 2: Implement Value Type (Green)

```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Int(i64),
    String(Box<str>),
    Bool(bool),
    Null,
    Time(DateTime<Utc>),
    Ref(Box<str>),
    List(Box<[Value]>),
    Record(HashMap<Box<str>, Value>),
    Cap(Box<str>),
}
```

### Step 3: Add Custom Serialization (Green)

- Ensure JSON serialization is clean for debugging
- Consider compact binary format for internal use

### Step 4: Add Accessor Methods (Green)

```rust
impl Value {
    pub fn as_int(&self) -> Option<i64> { ... }
    pub fn as_string(&self) -> Option<&str> { ... }
    pub fn as_bool(&self) -> Option<bool> { ... }
    pub fn is_null(&self) -> bool { ... }
}
```

### Step 5: Refactor (Refactor)

- Review enum size optimization
- Ensure Clone is efficient (Arc for large collections?)
- Check for unnecessary allocations

## Completion Checklist

- [ ] Value enum with all 9 variants
- [ ] Serde Serialize/Deserialize implementations
- [ ] JSON roundtrip property tests
- [ ] Accessor methods (as_int, as_string, as_bool, is_null)
- [ ] Display trait implementation
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes with no warnings
- [ ] Documentation complete with examples

## Estimated Effort

4 hours (including serialization testing)

## Dependencies

- TASK-001 (Effect) - for understanding pattern

## Blocked By

Nothing

## Blocks

- TASK-003 (Workflow AST uses Value)
- TASK-006 (Arbitrary implementations need Value)
