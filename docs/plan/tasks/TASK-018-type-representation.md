# TASK-018: Type Representation and Unification

## Status: ✅ Complete

## Description

Implement the type enum, type variables, and unification algorithm for Hindley-Milner style type inference.

## Specification Reference

- SPEC-003: Type System - Section 3. Value Types

## Requirements

### Type Enum

```rust
/// Types in the Ash type system
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    // Primitive types
    Int,
    String,
    Bool,
    Null,
    Time,
    Ref,
    
    // Composite types
    List(Box<Type>),
    Record(Vec<(Box<str>, Type)>),
    
    // Capability type
    Cap { name: Box<str>, effect: Effect },
    
    // Function type (arguments, return, effect)
    Fun(Vec<Type>, Box<Type>, Effect),
    
    // Type variable for inference
    Var(TypeVar),
}

/// Type variable identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeVar(pub u32);

impl TypeVar {
    pub fn fresh() -> Self {
        use std::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        TypeVar(COUNTER.fetch_add(1, Ordering::SeqCst))
    }
}
```

### Substitution

```rust
/// Type substitution: maps type variables to types
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Substitution {
    mappings: HashMap<TypeVar, Type>,
}

impl Substitution {
    pub fn new() -> Self {
        Self {
            mappings: HashMap::new(),
        }
    }
    
    /// Add a mapping from var to type
    pub fn insert(&mut self, var: TypeVar, ty: Type) {
        self.mappings.insert(var, ty);
    }
    
    /// Look up a variable in the substitution
    pub fn get(&self, var: TypeVar) -> Option<&Type> {
        self.mappings.get(&var)
    }
    
    /// Apply substitution to a type
    pub fn apply(&self, ty: &Type) -> Type {
        match ty {
            Type::Var(v) => {
                match self.get(*v) {
                    Some(t) => self.apply(t),  // Recursive application
                    None => Type::Var(*v),
                }
            }
            Type::List(elem) => Type::List(Box::new(self.apply(elem))),
            Type::Record(fields) => {
                let new_fields: Vec<_> = fields
                    .iter()
                    .map(|(name, ty)| (name.clone(), self.apply(ty)))
                    .collect();
                Type::Record(new_fields)
            }
            Type::Fun(args, ret, eff) => {
                let new_args: Vec<_> = args.iter().map(|a| self.apply(a)).collect();
                Type::Fun(new_args, Box::new(self.apply(ret)), *eff)
            }
            other => other.clone(),
        }
    }
    
    /// Compose two substitutions: self ∘ other
    pub fn compose(self, other: Substitution) -> Substitution {
        let mut result = Substitution::new();
        
        // Apply self to other's mappings
        for (var, ty) in other.mappings {
            result.insert(var, self.apply(&ty));
        }
        
        // Add self's mappings
        for (var, ty) in self.mappings {
            result.insert(var, ty);
        }
        
        result
    }
}
```

### Unification

```rust
#[derive(Debug, Clone, thiserror::Error)]
pub enum UnifyError {
    #[error("Cannot unify {0} with {1}")]
    Mismatch(Type, Type),
    
    #[error("Occurs check failed: {0} occurs in {1}")]
    OccursCheck(TypeVar, Type),
    
    #[error("Record fields mismatch")]
    FieldMismatch,
    
    #[error("Different number of function arguments")]
    ArityMismatch,
}

/// Unify two types, returning a substitution
pub fn unify(t1: &Type, t2: &Type) -> Result<Substitution, UnifyError> {
    use Type::*;
    
    match (t1, t2) {
        // Identical types
        (Int, Int) | (String, String) | (Bool, Bool) | (Null, Null) | (Time, Time) | (Ref, Ref) => {
            Ok(Substitution::new())
        }
        
        // Type variable handling
        (Var(v), t) | (t, Var(v)) => bind(*v, t),
        
        // List types
        (List(e1), List(e2)) => unify(e1, e2),
        
        // Record types (structural)
        (Record(f1), Record(f2)) => {
            if f1.len() != f2.len() {
                return Err(UnifyError::FieldMismatch);
            }
            
            let mut subst = Substitution::new();
            let mut f2_map: HashMap<_, _> = f2.iter().cloned().collect();
            
            for (name1, ty1) in f1 {
                match f2_map.remove(name1) {
                    Some(ty2) => {
                        let s = unify(&subst.apply(ty1), &subst.apply(&ty2))?;
                        subst = s.compose(subst);
                    }
                    None => return Err(UnifyError::FieldMismatch),
                }
            }
            
            Ok(subst)
        }
        
        // Function types
        (Fun(args1, ret1, eff1), Fun(args2, ret2, eff2)) => {
            if args1.len() != args2.len() {
                return Err(UnifyError::ArityMismatch);
            }
            if eff1 != eff2 {
                // Effects must match exactly
                return Err(UnifyError::Mismatch(t1.clone(), t2.clone()));
            }
            
            let mut subst = Substitution::new();
            
            // Unify argument types
            for (a1, a2) in args1.iter().zip(args2.iter()) {
                let s = unify(&subst.apply(a1), &subst.apply(a2))?;
                subst = s.compose(subst);
            }
            
            // Unify return types
            let s = unify(&subst.apply(ret1), &subst.apply(ret2))?;
            subst = s.compose(subst);
            
            Ok(subst)
        }
        
        // Capability types
        (Cap { name: n1, effect: e1 }, Cap { name: n2, effect: e2 }) => {
            if n1 == n2 && e1 == e2 {
                Ok(Substitution::new())
            } else {
                Err(UnifyError::Mismatch(t1.clone(), t2.clone()))
            }
        }
        
        // Mismatch
        _ => Err(UnifyError::Mismatch(t1.clone(), t2.clone())),
    }
}

/// Bind a type variable to a type, checking occurs
fn bind(var: TypeVar, ty: &Type) -> Result<Substitution, UnifyError> {
    if let Type::Var(v) = ty {
        if *v == var {
            // Trivial binding
            return Ok(Substitution::new());
        }
    }
    
    if occurs_in(var, ty) {
        return Err(UnifyError::OccursCheck(var, ty.clone()));
    }
    
    let mut subst = Substitution::new();
    subst.insert(var, ty.clone());
    Ok(subst)
}

/// Check if a variable occurs in a type
fn occurs_in(var: TypeVar, ty: &Type) -> bool {
    match ty {
        Type::Var(v) => *v == var,
        Type::List(elem) => occurs_in(var, elem),
        Type::Record(fields) => fields.iter().any(|(_, t)| occurs_in(var, t)),
        Type::Fun(args, ret, _) => {
            args.iter().any(|a| occurs_in(var, a)) || occurs_in(var, ret)
        }
        _ => false,
    }
}
```

### Type Environment

```rust
/// Type environment: maps names to type schemes
#[derive(Debug, Clone, Default)]
pub struct TypeEnv {
    bindings: HashMap<Box<str>, Type>,
    parent: Option<Box<TypeEnv>>,
}

impl TypeEnv {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            parent: None,
        }
    }
    
    pub fn with_parent(parent: Box<TypeEnv>) -> Self {
        Self {
            bindings: HashMap::new(),
            parent: Some(parent),
        }
    }
    
    pub fn insert(&mut self, name: impl Into<Box<str>>, ty: Type) {
        self.bindings.insert(name.into(), ty);
    }
    
    pub fn get(&self, name: &str) -> Option<&Type> {
        self.bindings.get(name).or_else(|| {
            self.parent.as_ref().and_then(|p| p.get(name))
        })
    }
    
    pub fn apply_subst(&mut self, subst: &Substitution) {
        for (_, ty) in self.bindings.iter_mut() {
            *ty = subst.apply(ty);
        }
    }
}
```

## TDD Steps

### Step 1: Implement Type Enum

Create `crates/ash-typeck/src/types.rs` with Type and TypeVar.

### Step 2: Implement Substitution

Add Substitution struct with apply and compose.

### Step 3: Implement Unification

Add unify function with all type cases.

### Step 4: Implement Type Environment

Add TypeEnv for name resolution.

### Step 5: Write Property Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_unify_same_type() {
        let t = Type::Int;
        let subst = unify(&t, &t).unwrap();
        assert!(subst.mappings.is_empty());
    }

    #[test]
    fn test_unify_var_with_type() {
        let v = Type::Var(TypeVar(0));
        let t = Type::Int;
        let subst = unify(&v, &t).unwrap();
        assert_eq!(subst.get(TypeVar(0)), Some(&Type::Int));
    }

    #[test]
    fn test_unify_list_types() {
        let t1 = Type::List(Box::new(Type::Var(TypeVar(0))));
        let t2 = Type::List(Box::new(Type::Int));
        let subst = unify(&t1, &t2).unwrap();
        assert_eq!(subst.get(TypeVar(0)), Some(&Type::Int));
    }

    #[test]
    fn test_occurs_check_fails() {
        let v = Type::Var(TypeVar(0));
        let t = Type::List(Box::new(v.clone()));
        let result = unify(&v, &t);
        assert!(matches!(result, Err(UnifyError::OccursCheck(_, _))));
    }

    proptest! {
        #[test]
        fn prop_unify_symmetric(t1 in arb_type(), t2 in arb_type()) {
            // Unification should be symmetric
            let r1 = unify(&t1, &t2);
            let r2 = unify(&t2, &t1);
            
            match (r1, r2) {
                (Ok(_), Ok(_)) => {}, // Both succeed
                (Err(_), Err(_)) => {}, // Both fail
                _ => panic!("Unify not symmetric"),
            }
        }
    }
}
```

## Completion Checklist

- [ ] Type enum with all variants
- [ ] TypeVar with fresh generation
- [ ] Substitution with apply and compose
- [ ] Unification algorithm
- [ ] Occurs check
- [ ] Type environment
- [ ] Unit tests for each type combination
- [ ] Property tests for unification properties
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes

## Self-Review Questions

1. **Completeness**: Does unification handle all type combinations?
2. **Termination**: Is occurs check implemented correctly?
3. **Performance**: Is substitution application efficient?

## Estimated Effort

4 hours

## Dependencies

- ash-core: Effect enum

## Blocked By

- TASK-001: Effect lattice (uses Effect)

## Blocks

- TASK-019: Type constraints (uses unification)
- TASK-020: Constraint solving (uses unification)
