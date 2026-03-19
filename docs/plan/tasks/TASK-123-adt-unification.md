# TASK-123: ADT Unification

## Status: ✅ ALREADY IMPLEMENTED

> **Note:** Upon review for Phase 17 planning, this functionality was discovered to already be implemented in the codebase. This task file is kept for documentation purposes.

## Description

Extend the unification algorithm to handle ADT type constructors (e.g., `Option<Int>`, `Result<T, E>`).

## Specification Reference

- SPEC-020: ADT Types - Section 4.1, 6.4

## Requirements

### Functional Requirements

1. Unify `Type::Constructor` variants:
   - Same constructor name + compatible args = success
   - Different constructor names = error
   - Different arg counts = error

2. Update substitution to handle constructors:
   - Apply substitution to constructor args
   - Compose substitutions from constructor unification

3. Update occurs check for constructors:
   - Check if type variable occurs in any constructor arg

### Property Requirements

```rust
// Constructor unification is reflexive
prop_constructor_unify_refl(c: TypeConstructor) = {
    let t = Type::Constructor(c);
    assert!(unify(&t, &t.clone()).is_ok());
}

// Different constructors don't unify
prop_constructor_unify_distinct_fails(c1: TypeConstructor, c2: TypeConstructor) = {
    assume!(c1.name != c2.name);
    let t1 = Type::Constructor(c1);
    let t2 = Type::Constructor(c2);
    assert!(unify(&t1, &t2).is_err());
}

// Type variable unification with constructor
prop_var_unify_constructor(v: TypeVar, c: TypeConstructor) = {
    let var = Type::Var(v);
    let ctor = Type::Constructor(c.clone());
    let result = unify(&var, &ctor);
    assert!(result.is_ok());
    
    let subst = result.unwrap();
    assert_eq!(subst.apply(&var), ctor);
}
```

## Verification Steps

Since this is already implemented, verify the existing implementation:

### Step 1: Verify Constructor Unification Exists

**Verify in**: `crates/ash-typeck/src/types.rs`

Check that the `unify` function handles `Type::Constructor`:

```rust
// Constructor unification - names must match, args must unify
(Type::Constructor { name: n1, args: a1 }, Type::Constructor { name: n2, args: a2 }) => {
    if n1 != n2 {
        return Err(UnifyError::Mismatch(t1.clone(), t2.clone()));
    }
    if a1.len() != a2.len() {
        return Err(UnifyError::Mismatch(t1.clone(), t2.clone()));
    }
    let mut acc_sub = Substitution::new();
    for (arg1, arg2) in a1.iter().zip(a2.iter()) {
        let sub = unify(&acc_sub.apply(arg1), &acc_sub.apply(arg2))?;
        acc_sub = acc_sub.compose(&sub);
    }
    Ok(acc_sub)
}
```

### Step 2: Verify Error Types Exist

**File**: `crates/ash-typeck/src/types.rs`

Add to `UnifyError`:

```rust
#[derive(Debug, Clone, Error)]
pub enum UnifyError {
    // Existing variants...
    
    #[error("Cannot unify different type constructors: {0} vs {1}")]
    ConstructorMismatch(String, String),
    
    #[error("Constructor arity mismatch: expected {expected} args, got {actual}")]
    ArityMismatch { expected: usize, actual: usize },
}
```

### Step 3: Verify occurs_in Handles Constructors

**Verify in**: `crates/ash-typeck/src/types.rs`

Check that `occurs_in` handles `Type::Constructor`:

```rust
pub fn occurs_in(var: TypeVar, ty: &Type) -> bool {
    match ty {
        // ... other cases
        
        // Constructor: check all args
        Type::Constructor { args, .. } => args.iter().any(|a| occurs_in(var, a)),
        
        // ... rest
    }
}
```

### Step 4: Run Verification Tests

```bash
# Run existing type tests to verify everything works
cargo test -p ash-typeck types::tests -- --nocapture

# Specifically test constructor unification
cargo test -p ash-typeck -- --nocapture 2>&1 | grep -i constructor || echo "No constructor-specific tests yet - that's OK"
```

### Step 5: Add Property Tests (Optional Enhancement)

If desired, add property tests to `crates/ash-typeck/src/types.rs`:

```rust
#[cfg(test)]
mod adt_unification_tests {
    use super::*;
    use proptest::prelude::*;

    prop_compose! {
        fn arb_constructor()(
            name in "[A-Z][a-zA-Z0-9]*",
            args in prop::collection::vec(
                prop_oneof![Just(Type::Int), Just(Type::String)],
                0..3
            )
        ) -> Type {
            Type::Constructor { name: name.into(), args }
        }
    }

    proptest! {
        #[test]
        fn prop_constructor_unify_self(c in arb_constructor()) {
            let result = unify(&c, &c.clone());
            prop_assert!(result.is_ok());
        }

        #[test]
        fn prop_constructor_unify_different_names_fails(
            name1 in "[A-Z][a-zA-Z0-9]*",
            name2 in "[A-Z][a-zA-Z0-9]*"
        ) {
            prop_assume!(name1 != name2);
            let t1 = Type::Constructor { name: name1.into(), args: vec![] };
            let t2 = Type::Constructor { name: name2.into(), args: vec![] };
            let result = unify(&t1, &t2);
            prop_assert!(result.is_err());
        }
    }
}
```

### Step 6: Document as Complete

This task is already implemented. The following functionality exists:

1. ✅ Constructor unification in `unify()` function
2. ✅ Constructor name matching
3. ✅ Constructor arity checking
4. ✅ Recursive argument unification
5. ✅ `occurs_in` handles `Type::Constructor`
6. ✅ `Substitution::apply` handles `Type::Constructor`

No code changes needed unless:
- You want to add specific error variants (`ConstructorMismatch`, `ArityMismatch`)
- You want to add property tests for constructor unification

### Step 7: Update PLAN-INDEX

Mark TASK-123 as complete in `docs/plan/PLAN-INDEX.md`:

```markdown
| [TASK-123](tasks/TASK-123-adt-unification.md) | Unification with constructors | SPEC-020 | 4 | ✅ Complete (already implemented) |
```

## Completion Checklist (Verification)

- [x] Constructor unification implemented in `unify()`
- [x] Constructor name matching works
- [x] Constructor arity checking works
- [x] `occurs_in` handles `Type::Constructor`
- [x] `Substitution::apply` handles `Type::Constructor`
- [x] All existing tests pass
- [ ] Optional: Add `ConstructorMismatch` and `ArityMismatch` error variants
- [ ] Optional: Add property tests for constructor unification
- [ ] Updated PLAN-INDEX.md to mark complete

## Completion Checklist

This task was discovered to be **already implemented** during Phase 17 planning review.

### Already Done ✅
- [x] Constructor unification in `unify` function
- [x] Constructor name matching (returns `UnifyError::Mismatch`)
- [x] Constructor arity checking (returns `UnifyError::Mismatch`)
- [x] `occurs_in` handles `Type::Constructor`
- [x] `Substitution::apply` handles `Type::Constructor`
- [x] Existing tests pass

### Optional Enhancements
- [ ] Add specific `ConstructorMismatch` error variant (currently uses generic `Mismatch`)
- [ ] Add specific `ArityMismatch` error variant (currently uses generic `Mismatch`)
- [ ] Add property tests for constructor unification

## Estimated Effort

**0 hours** - Already implemented

**Optional: 1 hour** - For error variant improvements and property tests

## Estimated Effort

4 hours

## Dependencies

- TASK-121 (ADT Core Types) - for `Type::Constructor`

## Blocked By

- TASK-121

## Blocks

- TASK-127 (Constructor Typing)
- TASK-128 (Pattern Typing)
- TASK-129 (Generic Instantiation)

## Discovery Notes

This task was created during Phase 17 planning assuming unification needed implementation. Upon code review, we found the constructor unification logic already exists in `crates/ash-typeck/src/types.rs` (lines ~422-438).

The implementation correctly:
1. Matches constructor names
2. Verifies arity matches
3. Recursively unifies argument types
4. Composes substitutions

This task file is kept for documentation and to track any optional enhancements (better error variants, property tests).
