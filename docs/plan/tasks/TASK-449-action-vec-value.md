# TASK-449: Update Action to Use Vec<Value>

## Status: 📝 Planned

## Description

Change `ash_core::Action::arguments` from `Vec<Expr>` to `Vec<Value>` to represent ready-to-execute actions with evaluated arguments. This task also includes the parser/lowering and interpreter execution changes required to keep the workspace coherent while that type changes.

## Specification Reference

- [DESIGN-015: Unified Action System](../../design/DESIGN-015-UNIFIED-ACTION-SYSTEM.md) - Decision 1
- [SPEC-001: IR](../../spec/SPEC-001-IR.md) - Core AST types

## Dependencies

None (first task in phase 1)

## Requirements

### Functional Requirements

1. Update `Action` struct in `crates/ash-core/src/ast.rs` to use `Vec<Value>` instead of `Vec<Expr>`
2. Update parser/lowering boundaries so they no longer construct the provider-facing `Action` with unevaluated expressions
3. Update ACT execution in `ash-interp` so expressions are evaluated before provider calls
4. Preserve serialization compatibility where possible
5. Update all direct `Action` constructions in tests
6. No changes to user-visible ACT semantics beyond moving evaluation to an explicit interpreter boundary

### Property Requirements

```rust
// Action should be serializable/deserializable
property_round_trip_serialization(action: Action) == deserialized_action
```

## TDD Steps

### Step 1: Write Tests (Red)

**File:** `crates/ash-core/src/ast.rs`

**Current State:**
```rust
/// An action to execute
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Action {
    pub name: Name,
    pub arguments: Vec<Expr>,  // Unevaluated expressions
}
```

**Target State:**
```rust
/// An action to execute
///
/// Arguments are already evaluated to `Value` at the workflow
/// execution layer before calling providers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Action {
    pub name: Name,
    pub arguments: Vec<Value>,  // Evaluated values
}
```

**Key Changes:**
1. Change `arguments` field type from `Vec<Expr>` to `Vec<Value>` - enables eager evaluation
2. Update docstring to reflect new semantics
3. Land the parser/lowering and interpreter changes in the same implementation slice so the workspace still compiles after the type change

**Test:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_action_with_values() {
        let action = Action {
            name: "write_file".to_string(),
            arguments: vec![
                Value::String("/tmp/test.txt".to_string()),
                Value::String("content".to_string()),
            ],
        };
        assert_eq!(action.name, "write_file");
        assert_eq!(action.arguments.len(), 2);
    }

    proptest! {
        #[test]
        fn action_round_trip_serialization(
            name in "[a-z]+",
            args in proptest::collection::vec(any::<Value>(), 0..10)
        ) {
            let action = Action {
                name,
                arguments: args,
            };
            
            let serialized = bincode::serialize(&action).unwrap();
            let deserialized: Action = bincode::deserialize(&serialized).unwrap();
            
            prop_assert_eq!(action, deserialized);
        }
    }
}
```

### Step 2: Implement (Green)

**File:** `crates/ash-core/src/ast.rs`

Implementation:
1. Change `arguments` field to `Vec<Value>`
2. Update docstring
3. Add or update the parser/lowering substrate so unevaluated surface arguments are not stored in the provider-facing `Action`
4. Update ACT execution to evaluate arguments before `CapabilityContext::execute`
5. Verify all existing tests still compile (they may need updates)

**Code:**
```rust
/// An action to execute
///
/// Arguments are already evaluated to `Value` at the workflow
/// execution layer before calling providers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Action {
    pub name: Name,
    pub arguments: Vec<Value>,
}
```

### Step 3: Integration (Green)

Update any code that directly constructs `Action`:
- Test fixtures
- Helper functions
- Mock implementations

Update the execution boundary in the same task:
- `ash-parser` lowering no longer writes unevaluated `Expr` values into the provider-facing `Action`
- `ash-interp` ACT execution evaluates arguments before dispatch

### Step 4: Property Tests (Verify)

**File:** `tests/task_449_proptest.rs`

```rust
//! Property-based tests for TASK-449

use ash_core::{Action, Value};
use proptest::prelude::*;

proptest! {
    #[test]
    fn action_serialization_preserves_all_values(
        name in "[a-z]+",
        args in proptest::collection::vec(any::<Value>(), 0..20)
    ) {
        let action = Action {
            name,
            arguments: args.clone(),
        };
        
        let serialized = bincode::serialize(&action).unwrap();
        let deserialized: Action = bincode::deserialize(&serialized).unwrap();
        
        prop_assert_eq!(deserialized.name, action.name);
        prop_assert_eq!(deserialized.arguments, args);
    }

    #[test]
    fn action_eq_is_reflexive(
        name in "[a-z]+",
        args in proptest::collection::vec(any::<Value>(), 0..10)
    ) {
        let action = Action { name, arguments: args };
        prop_assert_eq!(action, action);
    }

    #[test]
    fn action_eq_is_symmetric(
        name1 in "[a-z]+",
        name2 in "[a-z]+",
        args in proptest::collection::vec(any::<Value>(), 0..10)
    ) {
        let action1 = Action {
            name: name1.clone(),
            arguments: args.clone(),
        };
        let action2 = Action {
            name: name2.clone(),
            arguments: args.clone(),
        };
        
        prop_assert_eq!(action1 == action2, name1 == name2);
    }
}
```

## Verification Steps

- [ ] `cargo test --package ash-core --lib ast` passes
- [ ] Property tests pass (100+ iterations)
- [ ] `cargo clippy --package ash-core` clean
- [ ] `cargo fmt --check` clean
- [ ] Documentation builds: `cargo doc --package ash-core --no-deps`

## Dependencies for Next Task

This task outputs:
- Updated `Action` type with `Vec<Value>`

Required by:
- [TASK-450](TASK-450-unified-provider-trait.md): Unified provider trait (uses updated Action)

## Notes

**Important considerations:**
- Parser/lowering and ACT execution cannot be deferred to a follow-up task because `ash_core::Action` is a single shared type across parser, interpreter, and providers
- Evaluation happens in this task at the ACT execution layer
- This is a breaking change for any code directly constructing `Action`

**Edge cases to consider:**
- Empty arguments list: `Action { name: "no_args", arguments: vec![] }`
- Single argument: `Action { name: "single_arg", arguments: vec![Value::Int(42)] }`
- Complex values: `Action { name: "complex", arguments: vec![Value::Record(...)] }`

**Ready to Implement Checklist:**
- [x] File paths specified (`crates/ash-core/src/ast.rs`)
- [x] Current code shown
- [x] Target code shown
- [x] Tests defined
- [x] Dependencies listed (none)
- [x] Edge cases mentioned (empty, single, complex)
- [x] Error cases considered (serialization failures)
