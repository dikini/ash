# TASK-129: Generic Type Instantiation

## Status: ✅ Complete

## Description

Implement generic type instantiation for substituting type parameters with concrete types.

## Specification Reference

- SPEC-020: ADT Types - Section 6.4

## Requirements

Implement instantiation:
```rust
// Given: type Option<T> = Some { value: T } | None
// instantiate(Option<T>, [Int]) => Option<Int>
//   where T is substituted with Int throughout
```

## TDD Steps

### Step 1: Write Tests (Red)

**File**: `crates/ash-typeck/src/instantiate.rs` (new)

```rust
//! Generic type instantiation

use ash_core::ast::{TypeDef, TypeBody, VariantDef};
use crate::types::{Type, TypeVar, Variant, Substitution};

/// Instantiate a generic type definition with concrete arguments
pub fn instantiate(def: &TypeDef, args: &[Type]) -> Result<Type, InstantiateError> {
    if def.params.len() != args.len() {
        return Err(InstantiateError::ArityMismatch {
            expected: def.params.len(),
            actual: args.len(),
        });
    }
    
    let subst = Substitution::from_pairs(
        def.params.iter().zip(args.iter())
    );
    
    Ok(apply_to_type_def(&subst, def))
}

/// Apply substitution to a type definition
fn apply_to_type_def(subst: &Substitution, def: &TypeDef) -> Type {
    match &def.body {
        TypeBody::Enum(variants) => Type::Sum {
            name: def.name.clone(),
            type_params: def.params.clone(),
            variants: variants.iter().map(|v| Variant {
                name: v.name.clone(),
                fields: v.fields.iter()
                    .map(|(n, t)| (n.clone(), subst.apply(t)))
                    .collect(),
            }).collect(),
        },
        TypeBody::Struct(fields) => Type::Struct {
            name: def.name.clone(),
            type_params: def.params.clone(),
            fields: fields.iter()
                .map(|(n, t)| (n.clone(), subst.apply(t)))
                .collect(),
        },
        TypeBody::Alias(ty) => subst.apply(ty),
    }
}

#[derive(Debug, Clone, Error)]
pub enum InstantiateError {
    #[error("Arity mismatch: expected {expected} type arguments, got {actual}")]
    ArityMismatch { expected: usize, actual: usize },
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_core::ast::{Visibility, TypeExpr};

    fn make_option_def() -> TypeDef {
        TypeDef {
            name: "Option".into(),
            params: vec![TypeVar(0)],
            body: TypeBody::Enum(vec![
                VariantDef {
                    name: "Some".into(),
                    fields: vec![("value".into(), TypeExpr::Var(TypeVar(0)))],
                },
                VariantDef {
                    name: "None".into(),
                    fields: vec![],
                },
            ]),
            visibility: Visibility::Public,
        }
    }

    #[test]
    fn test_instantiate_option_int() {
        let def = make_option_def();
        let result = instantiate(&def, &[Type::Int]);
        
        assert!(result.is_ok());
        match result.unwrap() {
            Type::Sum { name, variants, .. } => {
                assert_eq!(name.as_ref(), "Option");
                assert_eq!(variants.len(), 2);
                // Some variant should have Int field
                assert_eq!(variants[0].fields[0].1, Type::Int);
            }
            _ => panic!("Expected Sum type"),
        }
    }

    #[test]
    fn test_instantiate_arity_mismatch() {
        let def = make_option_def();
        let result = instantiate(&def, &[Type::Int, Type::String]);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), InstantiateError::ArityMismatch { .. }));
    }
}
```

### Step 2: Implement Substitution::from_pairs (Green)

**File**: `crates/ash-typeck/src/types.rs`

Add to `Substitution`:

```rust
impl Substitution {
    /// Create substitution from pairs of (var, type)
    pub fn from_pairs(pairs: impl Iterator<Item = (TypeVar, &Type)>) -> Self {
        let mut mappings = HashMap::new();
        for (var, ty) in pairs {
            mappings.insert(var, ty.clone());
        }
        Self { mappings }
    }
}
```

### Step 3: Run Tests

```bash
cargo test -p ash-typeck instantiate -- --nocapture
```

### Step 4: Add Property Tests

```rust
#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_instantiate_no_free_vars(
            def in arbitrary_type_def(),
            args in prop::collection::vec(arbitrary_type(), 0..3)
        ) {
            prop_assume!(def.params.len() == args.len());
            
            let instantiated = instantiate(&def, &args).unwrap();
            
            // No free variables from params should remain
            for (param, _) in def.params.iter().zip(args.iter()) {
                prop_assert!(!contains_var(&instantiated, *param));
            }
        }
    }
    
    fn contains_var(ty: &Type, var: TypeVar) -> bool {
        match ty {
            Type::Var(v) => *v == var,
            Type::Constructor { args, .. } => args.iter().any(|a| contains_var(a, var)),
            Type::Sum { variants, .. } => variants.iter()
                .any(|v| v.fields.iter().any(|(_, t)| contains_var(t, var))),
            Type::Struct { fields, .. } => fields.iter()
                .any(|(_, t)| contains_var(t, var)),
            _ => false,
        }
    }
}
```

### Step 5: Commit

```bash
git add crates/ash-typeck/src/instantiate.rs crates/ash-typeck/src/types.rs
git commit -m "feat(typeck): generic type instantiation (TASK-129)"
```

## Completion Checklist

- [ ] `instantiate` function
- [ ] `InstantiateError::ArityMismatch`
- [ ] `Substitution::from_pairs`
- [ ] `apply_to_type_def` for Sum types
- [ ] `apply_to_type_def` for Struct types
- [ ] `apply_to_type_def` for Alias types
- [ ] Unit tests for Option<T> instantiation
- [ ] Unit tests for arity mismatch
- [ ] Property tests for no free vars after instantiation
- [ ] Documentation comments
- [ ] `cargo fmt` and `cargo clippy` pass

## Estimated Effort

6 hours

## Dependencies

- TASK-121 (ADT Core Types)
- TASK-123 (ADT Unification)

## Blocked By

- TASK-121
- TASK-123

## Blocks

- TASK-127 (Constructor Typing)
- TASK-128 (Pattern Typing)
- TASK-130 (Exhaustiveness Checking)
