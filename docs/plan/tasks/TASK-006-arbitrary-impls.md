# TASK-006: Property Testing Arbitrary Implementations

## Status: 🔴 Not Started

## Description

Implement proptest Arbitrary trait implementations for all core types to enable property-based testing.

## Specification Reference

- SPEC-001: IR - Section 5 Properties

## Requirements

### Functional Requirements

1. Implement `Arbitrary` for:
   - `Effect` (already exists - verify/enhance)
   - `Value` - Generate valid values of all types
   - `Pattern` - Generate well-formed patterns
   - `Workflow` - Generate well-formed workflow ASTs
   - `Expr` - Generate valid expressions
   - `Capability`, `Action` - Supporting types

2. Smart generation strategies:
   - Control recursion depth for nested types
   - Generate effect-appropriate workflows
   - Ensure generated patterns have unique bindings

3. Helper strategies:
   - `arb_effect()` - Uniform effect distribution
   - `arb_value()` - All value variants
   - `arb_value_of_type()` - Values matching a type
   - `arb_pattern()` - Well-formed patterns
   - `arb_workflow()` - Well-formed workflows
   - `arb_expr()` - Valid expressions

### Property Requirements

```rust
// Generated values are valid
// - No cycles in generated workflows (by construction via Box)
// - Patterns have unique bindings
// - Values are well-formed

// Distribution is reasonable
// - All variants generated
// - No bias toward simple cases
```

## TDD Steps

### Step 1: Value Arbitrary (Green)

```rust
impl Arbitrary for Value {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;
    
    fn arbitrary_with(_: ()) -> Self::Strategy {
        let leaf = prop_oneof![
            any::<i64>().prop_map(Value::Int),
            any::<bool>().prop_map(Value::Bool),
            Just(Value::Null),
            // ... more leaf types
        ];
        
        leaf.prop_recursive(
            4, 64, 10,
            |inner| prop_oneof![
                prop::collection::vec(inner.clone(), 0..10)
                    .prop_map(|v| Value::List(v.into_boxed_slice())),
                // ... record generation
            ]
        ).boxed()
    }
}
```

### Step 2: Pattern Arbitrary (Green)

Generate patterns with controlled complexity and unique bindings.

### Step 3: Workflow Arbitrary (Green)

Generate workflows respecting effect ordering.

### Step 4: Expr Arbitrary (Green)

Generate expressions with type-appropriate values.

### Step 5: Add Tests for Generators (Green)

```rust
proptest! {
    #[test]
    fn prop_arbitrary_value_roundtrips(v in any::<Value>()) {
        // Serialization roundtrip
    }
}
```

## Completion Checklist

- [ ] Arbitrary for Effect (verify existing)
- [ ] Arbitrary for Value
- [ ] Arbitrary for Pattern
- [ ] Arbitrary for Workflow
- [ ] Arbitrary for Expr
- [ ] Recursive depth control
- [ ] Unique binding enforcement for patterns
- [ ] Tests for all generators
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Estimated Effort

6 hours (recursive strategies are complex)

## Dependencies

- TASK-001 (Effect arbitrary)
- TASK-002 (Value)
- TASK-003 (Workflow)
- TASK-005 (Pattern)

## Blocked By

- TASK-002
- TASK-003
- TASK-005

## Blocks

- All tasks requiring property tests
- Fuzzing infrastructure
