# TASK-121: ADT Core Types

## Status: 🟡 Ready to Start

## Description

Add core type representations for Algebraic Data Types to `ash-core` and `ash-typeck`. This includes `Type::Sum`, `Type::Struct`, `Type::Constructor`, and related structures.

## Specification Reference

- SPEC-020: ADT Types - Section 4.1, 4.2

## Requirements

### Functional Requirements

1. Extend `Type` enum with:
   - `Sum` - Sum type with name, type params, variants
   - `Struct` - Struct type with name, type params, fields
   - `Constructor` - Type constructor application (e.g., `Option<Int>`)

2. Add `Variant` struct representing enum variants

3. Add `TypeDef` AST node for type definitions

4. Update unification to handle new types:
   - Constructor unification with substitution
   - Occurs check for recursive types

### Property Requirements

```rust
// Type equality is reflexive
prop_type_eq_refl(t: Type) = assert_eq!(t, t);

// Constructor unification
prop_constructor_unify(c: TypeConstructor) = {
    let t1 = Type::Constructor(c.clone());
    let t2 = Type::Constructor(c.clone());
    assert!(unify(&t1, &t2).is_ok());
}

// Generic substitution
prop_subst_idempotent(subst: Substitution, t: Type) = {
    let applied1 = subst.apply(&t);
    let applied2 = subst.apply(&applied1);
    assert_eq!(applied1, applied2);
}
```

## TDD Steps

### Step 1: Write Property Tests (Red)

**File**: `crates/ash-typeck/src/types.rs` (append to tests)

```rust
#[cfg(test)]
mod adt_tests {
    use super::*;
    use proptest::prelude::*;

    prop_compose! {
        fn arb_variant()(name in "[A-Z][a-zA-Z0-9]*") -> Variant {
            Variant {
                name: name.into(),
                fields: vec![],
            }
        }
    }

    prop_compose! {
        fn arb_sum_type()(
            name in "[A-Z][a-zA-Z0-9]*",
            variants in prop::collection::vec(arb_variant(), 1..5)
        ) -> Type {
            Type::Sum {
                name: name.into(),
                type_params: vec![],
                variants,
            }
        }
    }

    proptest! {
        #[test]
        fn prop_sum_type_equality(t in arb_sum_type()) {
            // Same type equals itself
            assert_eq!(t, t.clone());
        }

        #[test]
        fn prop_constructor_unify_self(c in arb_constructor()) {
            let t = Type::Constructor(c);
            let result = unify(&t, &t.clone());
            assert!(result.is_ok());
        }
    }
}
```

### Step 2: Add Type Variants (Green)

**File**: `crates/ash-typeck/src/types.rs`

Add to `Type` enum:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    // Existing variants...
    Int, String, Bool, Null, Time, Ref, List, Record, Cap, Fun, Var,

    /// Sum type: enum with variants
    Sum {
        name: Box<str>,
        type_params: Vec<TypeVar>,
        variants: Vec<Variant>,
    },

    /// Struct type: product with named fields
    Struct {
        name: Box<str>,
        type_params: Vec<TypeVar>,
        fields: Vec<(Box<str>, Type)>,
    },

    /// Type constructor application (e.g., Option<Int>)
    Constructor {
        name: Box<str>,
        args: Vec<Type>,
    },
}

/// Enum variant definition
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Variant {
    pub name: Box<str>,
    pub fields: Vec<(Box<str>, Type)>,
}
```

Update `Display` for `Type`:

```rust
impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Existing cases...
            
            Type::Sum { name, type_params, variants } => {
                write!(f, "{}", name)?;
                if !type_params.is_empty() {
                    write!(f, "<")?;
                    for (i, p) in type_params.iter().enumerate() {
                        if i > 0 { write!(f, ", ")?; }
                        write!(f, "{}", p.0)?;
                    }
                    write!(f, ">")?;
                }
                write!(f, " = ")?;
                for (i, v) in variants.iter().enumerate() {
                    if i > 0 { write!(f, " | ")?; }
                    write!(f, "{}", v.name)?;
                    if !v.fields.is_empty() {
                        write!(f, " {{ ")?;
                        for (j, (name, ty)) in v.fields.iter().enumerate() {
                            if j > 0 { write!(f, ", ")?; }
                            write!(f, "{}: {}", name, ty)?;
                        }
                        write!(f, " }}")?;
                    }
                }
                Ok(())
            }
            
            Type::Struct { name, type_params, fields } => {
                write!(f, "{}", name)?;
                if !type_params.is_empty() {
                    write!(f, "<")?;
                    for (i, p) in type_params.iter().enumerate() {
                        if i > 0 { write!(f, ", ")?; }
                        write!(f, "{}", p.0)?;
                    }
                    write!(f, ">")?;
                }
                write!(f, " {{ ")?;
                for (i, (name, ty)) in fields.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}: {}", name, ty)?;
                }
                write!(f, " }}")
            }
            
            Type::Constructor { name, args } => {
                write!(f, "{}", name)?;
                if !args.is_empty() {
                    write!(f, "<")?;
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 { write!(f, ", ")?; }
                        write!(f, "{}", arg)?;
                    }
                    write!(f, ">")?;
                }
                Ok(())
            }
        }
    }
}
```

### Step 3: Update Substitution::apply (Green)

**File**: `crates/ash-typeck/src/types.rs`

Add cases to `Substitution::apply`:

```rust
pub fn apply(&self, ty: &Type) -> Type {
    match ty {
        // Existing cases...
        
        Type::Sum { name, type_params, variants } => Type::Sum {
            name: name.clone(),
            type_params: type_params.clone(),
            variants: variants.iter()
                .map(|v| Variant {
                    name: v.name.clone(),
                    fields: v.fields.iter()
                        .map(|(n, t)| (n.clone(), self.apply(t)))
                        .collect(),
                })
                .collect(),
        },
        
        Type::Struct { name, type_params, fields } => Type::Struct {
            name: name.clone(),
            type_params: type_params.clone(),
            fields: fields.iter()
                .map(|(n, t)| (n.clone(), self.apply(t)))
                .collect(),
        },
        
        Type::Constructor { name, args } => Type::Constructor {
            name: name.clone(),
            args: args.iter().map(|a| self.apply(a)).collect(),
        },
        
        // ... rest
    }
}
```

### Step 4: Update occurs_in (Green)

**File**: `crates/ash-typeck/src/types.rs`

Add cases to `occurs_in`:

```rust
pub fn occurs_in(var: TypeVar, ty: &Type) -> bool {
    match ty {
        // Existing cases...
        
        Type::Sum { variants, .. } => variants.iter()
            .any(|v| v.fields.iter().any(|(_, t)| occurs_in(var, t))),
        
        Type::Struct { fields, .. } => fields.iter()
            .any(|(_, t)| occurs_in(var, t)),
        
        Type::Constructor { args, .. } => args.iter()
            .any(|a| occurs_in(var, a)),
        
        // ... rest
    }
}
```

### Step 5: Add Unification Cases (Green)

**File**: `crates/ash-typeck/src/types.rs`

Add to `unify` function:

```rust
pub fn unify(t1: &Type, t2: &Type) -> Result<Substitution, UnifyError> {
    match (t1, t2) {
        // Existing cases...
        
        // Constructor unification
        (
            Type::Constructor { name: n1, args: a1 },
            Type::Constructor { name: n2, args: a2 },
        ) => {
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
        
        // Cannot unify different type constructors
        _ => Err(UnifyError::Mismatch(t1.clone(), t2.clone())),
    }
}
```

### Step 6: Add TypeDef to AST (Green)

**File**: `crates/ash-core/src/ast.rs`

Add type definition structures:

```rust
/// Type definition in source code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeDef {
    pub name: Name,
    pub params: Vec<TypeVar>,
    pub body: TypeBody,
    pub visibility: Visibility,
}

/// Body of a type definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeBody {
    /// type Point = { x: Int, y: Int }
    Struct(Vec<(Name, TypeExpr)>),
    
    /// type Status = Pending | Processing { ... }
    Enum(Vec<VariantDef>),
    
    /// type Name = String
    Alias(TypeExpr),
}

/// Variant definition for enums
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VariantDef {
    pub name: Name,
    pub fields: Vec<(Name, TypeExpr)>,
}

/// Visibility modifier
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Crate,
    Private,
}

/// Surface syntax type expression (to be resolved)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeExpr {
    Named(Name),
    Constructor { name: Name, args: Vec<TypeExpr> },
    Tuple(Vec<TypeExpr>),
    Record(Vec<(Name, TypeExpr)>),
}
```

Update `Definition` enum:

```rust
pub enum Definition {
    // Existing...
    Workflow(Workflow),
    
    /// Type definition
    TypeDef(TypeDef),
}
```

### Step 7: Add Tests and Verify (Green)

Add unit tests:

```rust
#[test]
fn test_sum_type_display() {
    let t = Type::Sum {
        name: "Option".into(),
        type_params: vec![TypeVar(0)],
        variants: vec![
            Variant { name: "Some".into(), fields: vec![("value".into(), Type::Var(TypeVar(0)))] },
            Variant { name: "None".into(), fields: vec![] },
        ],
    };
    let s = t.to_string();
    assert!(s.contains("Option"));
    assert!(s.contains("Some"));
    assert!(s.contains("None"));
}

#[test]
fn test_constructor_unify() {
    let t1 = Type::Constructor {
        name: "Option".into(),
        args: vec![Type::Var(TypeVar(0))],
    };
    let t2 = Type::Constructor {
        name: "Option".into(),
        args: vec![Type::Int],
    };
    
    let result = unify(&t1, &t2);
    assert!(result.is_ok());
    
    let subst = result.unwrap();
    assert_eq!(subst.apply(&Type::Var(TypeVar(0))), Type::Int);
}

#[test]
fn test_constructor_unify_different_names_fails() {
    let t1 = Type::Constructor { name: "Option".into(), args: vec![] };
    let t2 = Type::Constructor { name: "Result".into(), args: vec![] };
    
    let result = unify(&t1, &t2);
    assert!(result.is_err());
}
```

Run tests:
```bash
cargo test -p ash-typeck types::adt_tests -- --nocapture
cargo test -p ash-core ast::tests -- --nocapture
```

### Step 8: Refactor

Review for:
- Can any code be shared between Sum and Struct handling?
- Are all public items documented?
- Follow rust-skills guidelines:
  - [api-doc-errors] Document error cases
  - [mem-box-large-variant] Consider boxing large variant fields if needed
  - [api-common-traits] Ensure Debug, Clone, PartialEq on public types

## Completion Checklist

- [ ] `Type::Sum`, `Type::Struct`, `Type::Constructor` added
- [ ] `Variant` struct defined
- [ ] `TypeDef`, `TypeBody`, `VariantDef` added to AST
- [ ] `Substitution::apply` handles new types
- [ ] `occurs_in` handles new types
- [ ] `unify` handles constructor unification
- [ ] Display impl for new types
- [ ] Property tests for type equality and unification
- [ ] Unit tests for display and basic operations
- [ ] Documentation comments on all public items
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes
- [ ] Self-review completed

## Self-Review Questions

1. **Simplicity**: Is the type representation minimal?
   - Yes, follows standard ADT representations

2. **Spec drift**: Does it match SPEC-020?
   - Verify Type enum variants match spec
   - Verify Variant structure matches spec

3. **Error handling**: Are errors descriptive?
   - UnifyError::Mismatch is used appropriately

## Estimated Effort

6 hours (including property tests)

## Dependencies

None - foundational task

## Blocked By

Nothing

## Blocks

- TASK-122 (ADT Values)
- TASK-127 (Constructor Typing)
- TASK-128 (Pattern Typing)
