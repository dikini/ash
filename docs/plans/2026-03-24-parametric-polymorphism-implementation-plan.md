# Parametric Polymorphism Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` to implement this plan task-by-task.

**Goal:** Implement full parametric polymorphism (generics) for Ash, enabling `Option<Int>` and `Option<String>` to be distinct, distinguishable types with proper unification, pattern matching, and error messages.

**Architecture:** Add `Type::Constructor` with qualified names and kind annotations to the type system. Implement iso-recursive type handling for nested constructors like `Option<List<Int>>`. Update unification, type expression conversion, and constructor typing to work with the new representation.

**Tech Stack:** Rust, `proptest` for property testing, existing Ash type checker infrastructure in `crates/ash-typeck`

---

## Task 1: Create Core Type Infrastructure - `Kind` and `QualifiedName`

**Files:**
- Create: `crates/ash-typeck/src/kind.rs`
- Create: `crates/ash-typeck/src/qualified_name.rs`
- Modify: `crates/ash-typeck/src/lib.rs` (to add modules)

**Context:** We need foundational types before modifying `Type`. Kinds classify type constructors (`*`, `* -> *`). Qualified names support module boundaries.

**Step 1: Create `Kind` type**

File: `crates/ash-typeck/src/kind.rs`

```rust
//! Kinds classify types and type constructors.
//! *           - proper type (Int, String, List<Int>)
//! * -> *      - type constructor (List, Option)
//! * -> * -> * - binary type constructor (Result, Pair)

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Kind {
    /// The kind of types: *
    Type,
    /// Function kind: K1 -> K2
    Arrow(Box<Kind>, Box<Kind>),
}

impl Kind {
    /// Create a kind arrow: k1 -> k2
    pub fn arrow(k1: impl Into<Box<Kind>>, k2: impl Into<Box<Kind>>) -> Self {
        Kind::Arrow(k1.into(), k2.into())
    }

    /// Create a kind for an n-ary type constructor
    pub fn n_ary(n: usize) -> Self {
        (0..n).fold(Kind::Type, |acc, _| Kind::arrow(Kind::Type, acc))
    }

    /// Check if this is a proper type kind (*)
    pub fn is_type(&self) -> bool {
        matches!(self, Kind::Type)
    }

    /// Get the arity of this kind (number of type arguments)
    pub fn arity(&self) -> usize {
        match self {
            Kind::Type => 0,
            Kind::Arrow(_, rest) => 1 + rest.arity(),
        }
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Kind::Type => write!(f, "*"),
            Kind::Arrow(k1, k2) => {
                if k1.is_type() {
                    write!(f, "* -> {}", k2)
                } else {
                    write!(f, "({}) -> {}", k1, k2)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_type_is_arity_zero() {
        assert_eq!(Kind::Type.arity(), 0);
        assert!(Kind::Type.is_type());
    }

    #[test]
    fn kind_n_ary() {
        assert_eq!(Kind::n_ary(0), Kind::Type);
        assert_eq!(Kind::n_ary(1).arity(), 1);
        assert_eq!(Kind::n_ary(2).arity(), 2);
    }

    #[test]
    fn kind_display() {
        assert_eq!(Kind::Type.to_string(), "*");
        assert_eq!(Kind::n_ary(1).to_string(), "* -> *");
        assert_eq!(Kind::n_ary(2).to_string(), "* -> * -> *");
    }
}
```

**Step 2: Create `QualifiedName` type**

File: `crates/ash-typeck/src/qualified_name.rs`

```rust
//! Fully qualified type names for module boundaries.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QualifiedName {
    /// Module path: ["Std", "Maybe"]
    pub module: Vec<String>,
    /// Base name: "Option"
    pub name: String,
}

impl QualifiedName {
    /// Create a root-level name (no module)
    pub fn root(name: impl Into<String>) -> Self {
        Self {
            module: vec![],
            name: name.into(),
        }
    }

    /// Create a qualified name
    pub fn qualified(module: Vec<String>, name: impl Into<String>) -> Self {
        Self {
            module,
            name: name.into(),
        }
    }

    /// Check if this is a root-level name
    pub fn is_root(&self) -> bool {
        self.module.is_empty()
    }

    /// Get the full path as a string (Std.Maybe.Option)
    pub fn display(&self) -> String {
        if self.module.is_empty() {
            self.name.clone()
        } else {
            format!("{}.{}", self.module.join("."), self.name)
        }
    }

    /// Parse from a dotted string
    pub fn parse(s: &str) -> Self {
        let parts: Vec<_> = s.split('.').collect();
        if parts.len() == 1 {
            Self::root(parts[0])
        } else {
            Self::qualified(
                parts[..parts.len() - 1].iter().map(|s| s.to_string()).collect(),
                parts.last().unwrap(),
            )
        }
    }
}

impl fmt::Display for QualifiedName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qualified_name_root() {
        let name = QualifiedName::root("Option");
        assert!(name.is_root());
        assert_eq!(name.display(), "Option");
    }

    #[test]
    fn qualified_name_qualified() {
        let name = QualifiedName::qualified(vec!["Std".to_string(), "Maybe".to_string()], "Option");
        assert!(!name.is_root());
        assert_eq!(name.display(), "Std.Maybe.Option");
    }

    #[test]
    fn qualified_name_parse() {
        let name = QualifiedName::parse("Std.Maybe.Option");
        assert_eq!(name.module, vec!["Std", "Maybe"]);
        assert_eq!(name.name, "Option");

        let root = QualifiedName::parse("Int");
        assert!(root.is_root());
        assert_eq!(root.name, "Int");
    }
}
```

**Step 3: Add modules to lib.rs**

Modify: `crates/ash-typeck/src/lib.rs`

Add at the top:

```rust
pub mod kind;
pub mod qualified_name;

pub use kind::Kind;
pub use qualified_name::QualifiedName;
```

**Step 4: Verify compilation**

Run: `cargo check --package ash-typeck`

Expected: Clean compile with no errors

**Step 5: Commit**

```bash
git add crates/ash-typeck/src/kind.rs crates/ash-typeck/src/qualified_name.rs crates/ash-typeck/src/lib.rs
git commit -m "feat(typeck): add Kind and QualifiedName types for generics infrastructure

- Kind for type constructor classification (*, * -> *, etc.)
- QualifiedName for module-qualified type names
- Foundation for Type::Constructor implementation"
```

---

## Task 2: Add `Type::Constructor` Variant

**Files:**
- Modify: `crates/ash-typeck/src/types.rs` (add variant and update all match sites)

**Context:** This is the core change. We add `Type::Constructor` with `QualifiedName`, `args`, and `kind`.

**Step 1: Add the new variant to Type enum**

Locate the `Type` enum in `crates/ash-typeck/src/types.rs` and add:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    // ... existing variants (Int, String, Bool, Null, Time, Ref, List, Record, Cap, Fun, Var, Instance) ...
    
    /// Type constructor application: Option<Int>, List<T>, Result<T, E>
    Constructor {
        /// Fully qualified constructor name
        name: QualifiedName,
        /// Type arguments (all must have kind *)
        args: Vec<Type>,
        /// Kind of the fully applied constructor (always * for now)
        kind: Kind,
    },
}
```

**Step 2: Update `Display` for Type**

Add the case in `impl fmt::Display for Type`:

```rust
Type::Constructor { name, args, .. } => {
    if args.is_empty() {
        write!(f, "{}", name)
    } else {
        write!(f, "{}<", name)?;
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", arg)?;
        }
        write!(f, ">")
    }
}
```

**Step 3: Update `occurs_in` function**

Add the case:

```rust
pub fn occurs_in(var: TypeVar, ty: &Type) -> bool {
    match ty {
        Type::Var(v) => *v == var,
        Type::Constructor { args, .. } => args.iter().any(|a| occurs_in(var, a)),
        Type::List(elem) => occurs_in(var, elem),
        // ... existing cases ...
    }
}
```

**Step 4: Update `Substitution::apply`**

Add the case in the `apply` method:

```rust
Type::Constructor { name, args, kind } => Type::Constructor {
    name: name.clone(),
    args: args.iter().map(|a| self.apply(a)).collect(),
    kind: kind.clone(),
},
```

**Step 5: Update `type_effect` (if it exists)**

If there's an effect computation function, add:

```rust
Type::Constructor { args, .. } => {
    args.iter()
        .map(type_effect)
        .fold(Effect::Epistemic, Effect::join)
}
```

**Step 6: Run tests to catch unhandled match arms**

Run: `cargo test --package ash-typeck 2>&1 | head -100`

Expected: May show "non-exhaustive patterns" errors - fix any that appear

**Step 7: Commit**

```bash
git add crates/ash-typeck/src/types.rs
git commit -m "feat(typeck): add Type::Constructor variant with kind annotation

- Constructor has QualifiedName, args Vec<Type>, and Kind
- Updated Display, occurs_in, Substitution::apply
- Foundation for generic type checking"
```

---

## Task 3: Implement Constructor Unification

**Files:**
- Modify: `crates/ash-typeck/src/types.rs` (unify function)

**Context:** Unification must handle constructor vs constructor, constructor vs variable, and detect infinite types.

**Step 1: Add constructor cases to unify**

Find the `unify` function and add:

```rust
pub fn unify(t1: &Type, t2: &Type) -> Result<Substitution, UnifyError> {
    use Type::*;

    // Handle identical types first
    if t1 == t2 {
        return Ok(Substitution::new());
    }

    match (t1, t2) {
        // Variable binding cases
        (Var(v), ty) => bind_var(*v, ty),
        (ty, Var(v)) => bind_var(*v, ty),

        // Constructor vs Constructor
        (
            Constructor { name: n1, args: a1, .. },
            Constructor { name: n2, args: a2, .. }
        ) => {
            if n1 != n2 {
                return Err(UnifyError::ConstructorNameMismatch {
                    expected: n1.display(),
                    found: n2.display(),
                });
            }

            if a1.len() != a2.len() {
                return Err(UnifyError::ConstructorArityMismatch {
                    name: n1.display(),
                    expected_arity: a1.len(),
                    found_arity: a2.len(),
                });
            }

            // Unify arguments pairwise, applying accumulated substitution
            let mut acc_sub = Substitution::new();
            for (arg1, arg2) in a1.iter().zip(a2.iter()) {
                let sub = unify(&acc_sub.apply(arg1), &acc_sub.apply(arg2))?;
                acc_sub = acc_sub.compose(&sub);
            }
            Ok(acc_sub)
        }

        // Constructor cannot unify with primitives
        (Constructor { .. }, _) | (_, Constructor { .. }) => {
            Err(UnifyError::Mismatch(t1.clone(), t2.clone()))
        }

        // ... existing cases (List, Record, Fun) ...
    }
}

fn bind_var(var: TypeVar, ty: &Type) -> Result<Substitution, UnifyError> {
    if let Type::Var(v) = ty {
        if *v == var {
            return Ok(Substitution::new()); // T = T
        }
    }

    if occurs_in(var, ty) {
        return Err(UnifyError::InfiniteType(var, ty.clone()));
    }

    let mut sub = Substitution::new();
    sub.insert(var, ty.clone());
    Ok(sub)
}
```

**Step 2: Add new error variants (if needed)**

If `UnifyError` doesn't have these, add:

```rust
pub enum UnifyError {
    // ... existing ...
    ConstructorNameMismatch { expected: String, found: String },
    ConstructorArityMismatch { name: String, expected_arity: usize, found_arity: usize },
    InfiniteType(TypeVar, Type),
}
```

**Step 3: Write unit tests**

Add to `crates/ash-typeck/src/types.rs` (in `#[cfg(test)]`):

```rust
#[test]
fn constructor_self_unifies() {
    use crate::qualified_name::QualifiedName;
    use crate::kind::Kind;

    let opt_int = Type::Constructor {
        name: QualifiedName::root("Option"),
        args: vec![Type::Int],
        kind: Kind::Type,
    };
    let sub = unify(&opt_int, &opt_int).unwrap();
    assert_eq!(sub, Substitution::new());
}

#[test]
fn constructor_different_names_fail() {
    use crate::qualified_name::QualifiedName;
    use crate::kind::Kind;

    let opt_int = Type::Constructor {
        name: QualifiedName::root("Option"),
        args: vec![Type::Int],
        kind: Kind::Type,
    };
    let list_int = Type::Constructor {
        name: QualifiedName::root("List"),
        args: vec![Type::Int],
        kind: Kind::Type,
    };
    assert!(unify(&opt_int, &list_int).is_err());
}

#[test]
fn constructor_different_args_fail() {
    use crate::qualified_name::QualifiedName;
    use crate::kind::Kind;

    let opt_int = Type::Constructor {
        name: QualifiedName::root("Option"),
        args: vec![Type::Int],
        kind: Kind::Type,
    };
    let opt_str = Type::Constructor {
        name: QualifiedName::root("Option"),
        args: vec![Type::String],
        kind: Kind::Type,
    };
    assert!(unify(&opt_int, &opt_str).is_err());
}

#[test]
fn constructor_with_var_unifies() {
    use crate::qualified_name::QualifiedName;
    use crate::kind::Kind;

    let v = TypeVar(0);
    let opt_var = Type::Constructor {
        name: QualifiedName::root("Option"),
        args: vec![Type::Var(v)],
        kind: Kind::Type,
    };
    let opt_int = Type::Constructor {
        name: QualifiedName::root("Option"),
        args: vec![Type::Int],
        kind: Kind::Type,
    };
    let sub = unify(&opt_var, &opt_int).unwrap();
    assert_eq!(sub.apply(&Type::Var(v)), Type::Int);
}

#[test]
fn nested_constructors_unify() {
    use crate::qualified_name::QualifiedName;
    use crate::kind::Kind;

    let v = TypeVar(0);
    let opt_list_var = Type::Constructor {
        name: QualifiedName::root("Option"),
        args: vec![Type::Constructor {
            name: QualifiedName::root("List"),
            args: vec![Type::Var(v)],
            kind: Kind::Type,
        }],
        kind: Kind::Type,
    };
    let opt_list_int = Type::Constructor {
        name: QualifiedName::root("Option"),
        args: vec![Type::Constructor {
            name: QualifiedName::root("List"),
            args: vec![Type::Int],
            kind: Kind::Type,
        }],
        kind: Kind::Type,
    };
    let sub = unify(&opt_list_var, &opt_list_int).unwrap();
    assert_eq!(sub.apply(&Type::Var(v)), Type::Int);
}

#[test]
fn occurs_check_finds_cycles() {
    use crate::qualified_name::QualifiedName;
    use crate::kind::Kind;

    let v = TypeVar(0);
    let t = Type::Var(v);
    let list_t = Type::Constructor {
        name: QualifiedName::root("List"),
        args: vec![t.clone()],
        kind: Kind::Type,
    };

    assert!(matches!(
        unify(&t, &list_t),
        Err(UnifyError::InfiniteType(TypeVar(0), _))
    ));
}
```

**Step 4: Run tests**

Run: `cargo test --package ash-typeck types::tests -- --nocapture`

Expected: All 6 new tests pass

**Step 5: Commit**

```bash
git add crates/ash-typeck/src/types.rs
git commit -m "feat(typeck): implement constructor unification with occurs check

- Constructor vs constructor unification with name and arity checking
- Constructor vs variable binding with infinite type detection
- Unit tests for all unification cases"
```

---

## Task 4: Fix Type Expression Conversion

**Files:**
- Modify: `crates/ash-typeck/src/type_env.rs`

**Context:** Replace the stubbed `TypeExpr::Constructor` handling that loses constructor information.

**Step 1: Find and fix type_expr_to_type**

Locate the function (around line 46-54 based on gap analysis) and replace:

```rust
pub fn type_expr_to_type(
    &self,
    expr: &TypeExpr,
    param_mapping: &HashMap<String, TypeVar>
) -> Result<Type, TypeError> {
    match expr {
        TypeExpr::Named(name) => {
            // Check if it's a type parameter
            if let Some(&var) = param_mapping.get(name) {
                return Ok(Type::Var(var));
            }

            // Check for primitive types
            match name.as_str() {
                "Int" => Ok(Type::Int),
                "String" => Ok(Type::String),
                "Bool" => Ok(Type::Bool),
                "Null" => Ok(Type::Null),
                "Time" => Ok(Type::Time),
                "Ref" => Ok(Type::Ref),
                _ => {
                    // User-defined type with no args - look it up
                    let (qualified, _) = self.resolve_type(name)?;
                    Ok(Type::Constructor {
                        name: qualified,
                        args: vec![],
                        kind: Kind::Type,
                    })
                }
            }
        }

        TypeExpr::Constructor { name, args } => {
            let (qualified, type_info) = self.resolve_type(name)?;
            
            // Convert all arguments
            let arg_types: Result<Vec<_>, _> = args.iter()
                .map(|arg| self.type_expr_to_type(arg, param_mapping))
                .collect();
            
            Ok(Type::Constructor {
                name: qualified,
                args: arg_types?,
                kind: Kind::Type,
            })
        }

        TypeExpr::List(elem) => {
            let elem_type = self.type_expr_to_type(elem, param_mapping)?;
            Ok(Type::List(Box::new(elem_type)))
        }

        TypeExpr::Record(fields) => {
            let field_types: Result<Vec<_>, _> = fields.iter()
                .map(|(n, t)| {
                    self.type_expr_to_type(t, param_mapping)
                        .map(|ty| (Box::from(n.as_str()), ty))
                })
                .collect();
            Ok(Type::Record(field_types?))
        }

        TypeExpr::Tuple(elems) => {
            // Convert tuple to record with numeric field names
            let field_types: Result<Vec<_>, _> = elems.iter().enumerate()
                .map(|(i, t)| {
                    self.type_expr_to_type(t, param_mapping)
                        .map(|ty| (Box::from(format!("_{}", i).as_str()), ty))
                })
                .collect();
            Ok(Type::Record(field_types?))
        }
    }
}
```

**Step 2: Fix type alias handling**

Find where `TypeBody::Alias` is handled (around line 192-200) and fix:

```rust
TypeBody::Alias(target_expr) => {
    // Expand alias to underlying type immediately
    self.type_expr_to_type(target_expr, &HashMap::new())?
}
```

**Step 3: Add resolve_type helper (if not exists)**

If `resolve_type` doesn't exist, add:

```rust
impl TypeEnv {
    /// Resolve a type name to its qualified form and info
    pub fn resolve_type(&self, name: &str) -> Result<(QualifiedName, &TypeInfo), TypeError> {
        // Try as primitive first
        match name {
            "Int" | "String" | "Bool" | "Null" | "Time" | "Ref" => {
                return Ok((QualifiedName::root(name), &TypeInfo::Primitive));
            }
            _ => {}
        }

        // Try local types
        if let Some(info) = self.types.get(name) {
            return Ok((QualifiedName::root(name), info));
        }

        // Try imports (if import system exists)
        if let Some((qualified, info)) = self.imports.get(name) {
            return Ok((qualified.clone(), info));
        }

        Err(TypeError::UndefinedType {
            name: name.to_string(),
        })
    }
}
```

**Step 4: Write test**

Add test to verify conversion:

```rust
#[test]
fn type_expr_constructor_converts_properly() {
    use crate::qualified_name::QualifiedName;
    use crate::kind::Kind;

    let env = TypeEnv::with_builtin_types();
    
    // Option<Int> should become Constructor { name: "Option", args: [Int] }
    let type_expr = TypeExpr::Constructor {
        name: "Option".to_string(),
        args: vec![TypeExpr::Named("Int".to_string())],
    };
    
    let ty = env.type_expr_to_type(&type_expr, &HashMap::new()).unwrap();
    
    match ty {
        Type::Constructor { name, args, kind } => {
            assert_eq!(name.display(), "Option");
            assert_eq!(args.len(), 1);
            assert_eq!(args[0], Type::Int);
            assert_eq!(kind, Kind::Type);
        }
        _ => panic!("Expected Type::Constructor, got {:?}", ty),
    }
}
```

**Step 5: Run tests**

Run: `cargo test --package ash-typeck type_env -- --nocapture`

Expected: Conversion test passes

**Step 6: Commit**

```bash
git add crates/ash-typeck/src/type_env.rs
git commit -m "feat(typeck): fix TypeExpr::Constructor conversion

- Replace stubbed implementation that lost constructor info
- Properly convert constructor name and all arguments
- Fix type alias expansion to underlying type
- Add name resolution for qualified type names"
```

---

## Task 5: Fix Constructor Typing

**Files:**
- Modify: `crates/ash-typeck/src/check_expr.rs`

**Context:** `build_constructor_type` currently returns the first parameter instead of the constructor type.

**Step 1: Fix build_constructor_type**

Find the function (around line 383-405) and replace:

```rust
fn build_constructor_type(
    type_info: &TypeInfo,
    _variant_idx: usize
) -> Type {
    use crate::qualified_name::QualifiedName;
    use crate::kind::Kind;

    match type_info {
        TypeInfo::Enum { name, params, .. } => {
            // Build Option<T>, not just T
            Type::Constructor {
                name: QualifiedName::root(name.clone()),
                args: params.iter().map(|p| Type::Var(*p)).collect(),
                kind: Kind::Type,
            }
        }
        TypeInfo::Struct { name, params, .. } => {
            Type::Constructor {
                name: QualifiedName::root(name.clone()),
                args: params.iter().map(|p| Type::Var(*p)).collect(),
                kind: Kind::Type,
            }
        }
        _ => {
            // Fallback for non-generic types
            Type::Var(TypeVar::fresh())
        }
    }
}
```

**Step 2: Write integration test**

Add test:

```rust
#[test]
fn constructor_returns_constructor_type() {
    let env = TypeEnv::with_builtin_types();
    
    // Some { value: 42 } should have type Option<Int>, not Int
    let expr = Expr::Constructor {
        name: "Some".into(),
        fields: vec![("value".into(), Expr::Literal(Value::Int(42)))],
    };
    
    let result = check_expr(&env, &expr);
    
    // Should be Option<Int>
    match result.ty {
        Type::Constructor { name, args, .. } => {
            assert_eq!(name.display(), "Option");
            assert_eq!(args.len(), 1);
            // The type argument should be unified to Int
        }
        _ => panic!("Expected constructor type, got {:?}", result.ty),
    }
}
```

**Step 3: Run tests**

Run: `cargo test --package ash-typeck constructor_returns -- --nocapture`

Expected: Test passes

**Step 4: Commit**

```bash
git add crates/ash-typeck/src/check_expr.rs
git commit -m "feat(typeck): fix constructor typing to return constructor type

- build_constructor_type now returns Option<T> not just T
- Uses QualifiedName and proper type parameters
- Fixes TASK-127: Constructor Typing"
```

---

## Task 6: Implement Recursive Type Unfolding

**Files:**
- Modify: `crates/ash-typeck/src/type_env.rs`

**Context:** Support iso-recursive types for field access and pattern matching.

**Step 1: Add unfold_constructor method**

Add to `TypeEnv`:

```rust
use crate::ast::TypeBody;

impl TypeEnv {
    /// Unfold a constructor to its definition with type arguments substituted
    pub fn unfold_constructor(
        &self,
        name: &QualifiedName,
        args: &[Type]
    ) -> Result<TypeBody, TypeError> {
        let (_, type_info) = self.resolve_type(&name.name)?;
        
        match type_info {
            TypeInfo::Enum { params, body, .. } |
            TypeInfo::Struct { params, body, .. } => {
                if params.len() != args.len() {
                    return Err(TypeError::ConstructorArityMismatch {
                        name: name.display(),
                        expected_arity: params.len(),
                        found_arity: args.len(),
                    });
                }
                
                // Create substitution from param vars to args
                let subst = params.iter().copied()
                    .zip(args.iter().cloned())
                    .fold(Substitution::new(), |mut acc, (var, ty)| {
                        acc.insert(var, ty);
                        acc
                    });
                
                // Apply substitution to body
                Ok(self.apply_subst_to_body(&subst, body))
            }
            _ => Err(TypeError::NotAConstructor(name.display())),
        }
    }
    
    /// Apply substitution to a type body (for unfolding)
    fn apply_subst_to_body(&self, subst: &Substitution, body: &TypeBody) -> TypeBody {
        match body {
            TypeBody::Enum(variants) => {
                TypeBody::Enum(
                    variants.iter().map(|v| Variant {
                        name: v.name.clone(),
                        fields: v.fields.iter()
                            .map(|(n, t)| (n.clone(), subst.apply(t)))
                            .collect(),
                    }).collect()
                )
            }
            TypeBody::Struct(fields) => {
                TypeBody::Struct(
                    fields.iter()
                        .map(|(n, t)| (n.clone(), subst.apply(t)))
                        .collect()
                )
            }
            TypeBody::Alias(target) => {
                // Keep as alias - the target type will be unfolded when needed
                TypeBody::Alias(target.clone())
            }
        }
    }
}
```

**Step 2: Add test**

```rust
#[test]
fn unfold_list_int() {
    use crate::ast::{TypeBody, Variant};
    use crate::qualified_name::QualifiedName;

    let env = TypeEnv::with_builtin_types();
    
    // Unfold List<Int>
    let unfolded = env.unfold_constructor(
        &QualifiedName::root("List"),
        &[Type::Int]
    ).unwrap();
    
    // Should get: Cons { head: Int, tail: List<Int> } | Nil
    match unfolded {
        TypeBody::Enum(variants) => {
            assert_eq!(variants.len(), 2);
            // Check Cons variant
            let cons = &variants[0];
            assert_eq!(cons.name, "Cons");
            assert!(cons.fields.iter().any(|(n, _)| n == "head"));
            assert!(cons.fields.iter().any(|(n, _)| n == "tail"));
        }
        _ => panic!("Expected enum body"),
    }
}
```

**Step 3: Commit**

```bash
git add crates/ash-typeck/src/type_env.rs
git commit -m "feat(typeck): add recursive type unfolding

- unfold_constructor for iso-recursive type handling
- Substitutes type arguments into variant/struct bodies
- Supports field access and pattern matching on generics"
```

---

## Task 7: Implement Pattern Typing for Generics

**Files:**
- Modify: `crates/ash-typeck/src/check_pattern.rs` (or create if needed)

**Context:** Pattern matching on generic constructors needs proper type inference.

**Step 1: Create/update check_pattern module**

If it doesn't exist, create `crates/ash-typeck/src/check_pattern.rs`:

```rust
//! Pattern type checking for generic constructors

use crate::ast::{Pattern, TypeExpr};
use crate::types::{Type, TypeVar, Substitution, unify};
use crate::type_env::{TypeEnv, TypeInfo};
use crate::errors::TypeError;
use std::collections::HashMap;

/// Result of type checking a pattern
pub struct PatternResult {
    pub typed_pattern: Pattern,
    pub bindings: HashMap<String, Type>,
}

/// Type check a pattern against an expected type
pub fn check_pattern(
    env: &TypeEnv,
    pattern: &Pattern,
    expected_ty: &Type,
) -> Result<PatternResult, TypeError> {
    match pattern {
        Pattern::Constructor { name, fields, .. } => {
            // Look up the constructor
            let (type_info, variant_idx) = env.lookup_constructor(name)?;
            
            // Build constructor type (e.g., Option<T>)
            let constructor_ty = build_constructor_type(&type_info);
            
            // Unify expected type with constructor type
            let sub = unify(expected_ty, &constructor_ty)?;
            
            // Unfold to get field types
            let unfolded = match &type_info {
                TypeInfo::Enum { name, params, body, .. } => {
                    let args: Vec<_> = params.iter().map(|p| sub.apply(&Type::Var(*p))).collect();
                    env.unfold_constructor(
                        &crate::qualified_name::QualifiedName::root(name.clone()),
                        &args
                    )?
                }
                _ => return Err(TypeError::NotAnEnum(name.clone())),
            };
            
            // Get field types for this variant
            let field_types = extract_variant_fields(&unfolded, variant_idx)?;
            
            // Check each field pattern
            let mut bindings = HashMap::new();
            for ((field_name, field_pat), (_, field_ty)) in 
                fields.iter().zip(field_types.iter()) {
                let field_result = check_pattern(env, field_pat, field_ty)?;
                bindings.extend(field_result.bindings);
            }
            
            Ok(PatternResult {
                typed_pattern: pattern.clone(), // Simplified
                bindings,
            })
        }
        
        Pattern::Variable(name) => {
            // Variable binds to expected type
            let mut bindings = HashMap::new();
            bindings.insert(name.clone(), expected_ty.clone());
            Ok(PatternResult {
                typed_pattern: pattern.clone(),
                bindings,
            })
        }
        
        Pattern::Wildcard => {
            Ok(PatternResult {
                typed_pattern: pattern.clone(),
                bindings: HashMap::new(),
            })
        }
        
        Pattern::Literal(value) => {
            // Check literal type matches expected
            let lit_ty = literal_type(value);
            unify(expected_ty, &lit_ty)?;
            Ok(PatternResult {
                typed_pattern: pattern.clone(),
                bindings: HashMap::new(),
            })
        }
    }
}

fn build_constructor_type(type_info: &TypeInfo) -> Type {
    use crate::qualified_name::QualifiedName;
    use crate::kind::Kind;

    match type_info {
        TypeInfo::Enum { name, params, .. } => {
            Type::Constructor {
                name: QualifiedName::root(name.clone()),
                args: params.iter().map(|p| Type::Var(*p)).collect(),
                kind: Kind::Type,
            }
        }
        _ => Type::Var(TypeVar::fresh()),
    }
}

fn extract_variant_fields(body: &TypeBody, variant_idx: usize) -> Result<Vec<(String, Type)>, TypeError> {
    match body {
        TypeBody::Enum(variants) => {
            variants.get(variant_idx)
                .map(|v| v.fields.clone())
                .ok_or_else(|| TypeError::InvalidVariantIndex(variant_idx))
        }
        _ => Err(TypeError::NotAnEnum("".to_string())),
    }
}

fn literal_type(value: &Value) -> Type {
    match value {
        Value::Int(_) => Type::Int,
        Value::String(_) => Type::String,
        Value::Bool(_) => Type::Bool,
        Value::Null => Type::Null,
        // ... other cases ...
    }
}
```

**Step 2: Add module to lib.rs**

Add:

```rust
pub mod check_pattern;
```

**Step 3: Write test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pattern_some_int() {
        let env = TypeEnv::with_builtin_types();
        
        // Pattern: Some { value: x }
        // Expected: Option<Int>
        let pattern = Pattern::Constructor {
            name: "Some".to_string(),
            fields: vec![(
                "value".to_string(),
                Pattern::Variable("x".to_string())
            )],
        };
        
        let expected = Type::Constructor {
            name: crate::qualified_name::QualifiedName::root("Option"),
            args: vec![Type::Int],
            kind: crate::kind::Kind::Type,
        };
        
        let result = check_pattern(&env, &pattern, &expected).unwrap();
        
        // x should bind to Int
        assert_eq!(result.bindings.get("x"), Some(&Type::Int));
    }
}
```

**Step 4: Commit**

```bash
git add crates/ash-typeck/src/check_pattern.rs crates/ash-typeck/src/lib.rs
git commit -m "feat(typeck): implement pattern typing for generic constructors

- check_pattern handles generic constructor patterns
- Proper type inference for pattern bindings
- Supports Some { value: x } where x gets type T from Option<T>"
```

---

## Task 8: Implement Exhaustiveness Checking for Generics

**Files:**
- Modify: `crates/ash-typeck/src/exhaustiveness.rs` (or create)

**Context:** Ensure pattern matches on generic enums are complete.

**Step 1: Create exhaustiveness module**

Create `crates/ash-typeck/src/exhaustiveness.rs`:

```rust
//! Exhaustiveness checking for pattern matches on generic types

use crate::ast::{Pattern, TypeBody};
use crate::types::{Type, Substitution};
use crate::type_env::{TypeEnv, TypeInfo};
use crate::errors::TypeError;
use crate::qualified_name::QualifiedName;
use std::collections::HashSet;

pub struct ExhaustivenessResult;

/// Check if patterns exhaustively cover a type
pub fn check_exhaustiveness(
    env: &TypeEnv,
    ty: &Type,
    patterns: &[Pattern],
) -> Result<ExhaustivenessResult, TypeError> {
    match ty {
        Type::Constructor { name, args, .. } => {
            check_constructor_exhaustiveness(env, name, args, patterns)
        }
        Type::Bool => {
            // Check for true/false
            let mut has_true = false;
            let mut has_false = false;
            for pat in patterns {
                match pat {
                    Pattern::Literal(Value::Bool(true)) => has_true = true,
                    Pattern::Literal(Value::Bool(false)) => has_false = true,
                    Pattern::Wildcard | Pattern::Variable(_) => {
                        return Ok(ExhaustivenessResult);
                    }
                    _ => {}
                }
            }
            if has_true && has_false {
                Ok(ExhaustivenessResult)
            } else {
                let mut missing = vec![];
                if !has_true { missing.push("true".to_string()); }
                if !has_false { missing.push("false".to_string()); }
                Err(TypeError::NonExhaustiveMatch { missing })
            }
        }
        // Other primitive types - wildcard/variable is sufficient
        _ => Ok(ExhaustivenessResult),
    }
}

fn check_constructor_exhaustiveness(
    env: &TypeEnv,
    name: &QualifiedName,
    args: &[Type],
    patterns: &[Pattern],
) -> Result<ExhaustivenessResult, TypeError> {
    let (_, type_info) = env.resolve_type(&name.name)?;
    
    match type_info {
        TypeInfo::Enum { variants, params, .. } => {
            // Create substitution from params to args
            let subst: Substitution = params.iter().copied()
                .zip(args.iter().cloned())
                .fold(Substitution::new(), |mut acc, (var, ty)| {
                    acc.insert(var, ty);
                    acc
                });
            
            // Find covered variants
            let mut covered_variants: HashSet<String> = HashSet::new();
            let mut covered_wildcard = false;
            
            for pat in patterns {
                match pat {
                    Pattern::Constructor { name, fields, .. } => {
                        covered_variants.insert(name.clone());
                        
                        // Check field patterns recursively
                        if let Some(variant) = variants.iter().find(|v| v.name == *name) {
                            for ((_, field_pat), (_, field_ty)) in 
                                fields.iter().zip(variant.fields.iter()) {
                                let field_ty_subst = subst.apply(field_ty);
                                check_exhaustiveness(env, &field_ty_subst, &[field_pat.clone()])?;
                            }
                        }
                    }
                    Pattern::Wildcard | Pattern::Variable(_) => {
                        covered_wildcard = true;
                        break;
                    }
                    _ => {}
                }
            }
            
            // Check if all variants covered
            if !covered_wildcard {
                let all_variants: HashSet<_> = variants.iter()
                    .map(|v| v.name.clone())
                    .collect();
                
                if covered_variants != all_variants {
                    let missing: Vec<_> = all_variants.difference(&covered_variants)
                        .cloned()
                        .collect();
                    
                    return Err(TypeError::NonExhaustiveMatch { missing });
                }
            }
            
            Ok(ExhaustivenessResult)
        }
        TypeInfo::Struct { .. } => {
            // Structs are always exhaustive (single constructor)
            Ok(ExhaustivenessResult)
        }
        _ => Ok(ExhaustivenessResult),
    }
}
```

**Step 2: Add to lib.rs**

```rust
pub mod exhaustiveness;
```

**Step 3: Write test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exhaustive_option_match() {
        let env = TypeEnv::with_builtin_types();
        
        let ty = Type::Constructor {
            name: QualifiedName::root("Option"),
            args: vec![Type::Int],
            kind: crate::kind::Kind::Type,
        };
        
        let patterns = vec![
            Pattern::Constructor {
                name: "Some".to_string(),
                fields: vec![("value".to_string(), Pattern::Wildcard)],
            },
            Pattern::Constructor {
                name: "None".to_string(),
                fields: vec![],
            },
        ];
        
        assert!(check_exhaustiveness(&env, &ty, &patterns).is_ok());
    }

    #[test]
    fn non_exhaustive_option_match() {
        let env = TypeEnv::with_builtin_types();
        
        let ty = Type::Constructor {
            name: QualifiedName::root("Option"),
            args: vec![Type::Int],
            kind: crate::kind::Kind::Type,
        };
        
        // Missing None case
        let patterns = vec![
            Pattern::Constructor {
                name: "Some".to_string(),
                fields: vec![("value".to_string(), Pattern::Wildcard)],
            },
        ];
        
        let err = check_exhaustiveness(&env, &ty, &patterns).unwrap_err();
        assert!(err.to_string().contains("None"));
    }
}
```

**Step 4: Commit**

```bash
git add crates/ash-typeck/src/exhaustiveness.rs crates/ash-typeck/src/lib.rs
git commit -m "feat(typeck): implement exhaustiveness checking for generic enums

- check_exhaustiveness handles Type::Constructor
- Detects missing variants in pattern matches
- Handles generic types with proper substitution
- Fixes TASK-130: Exhaustiveness for generics"
```

---

## Task 9: Add Property Tests

**Files:**
- Modify: `crates/ash-typeck/src/types.rs` (add proptest module)

**Context:** Verify unification properties using property-based testing.

**Step 1: Add proptest dependency (if not present)**

Check `crates/ash-typeck/Cargo.toml` for `proptest`. If missing, add:

```toml
[dev-dependencies]
proptest = "1.0"
```

**Step 2: Add property tests**

Add to `crates/ash-typeck/src/types.rs`:

```rust
#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;
    use crate::qualified_name::QualifiedName;
    use crate::kind::Kind;

    /// Strategy for generating arbitrary types
    fn type_strategy() -> impl Strategy<Value = Type> {
        let leaf = prop_oneof![
            Just(Type::Int),
            Just(Type::String),
            Just(Type::Bool),
            (0..10u32).prop_map(|n| Type::Var(TypeVar(n))),
        ];

        leaf.prop_recursive(4, 64, 16, |inner| {
            prop_oneof![
                // List
                inner.clone().prop_map(|t| Type::List(Box::new(t))),
                // Option<T>
                inner.clone().prop_map(|t| Type::Constructor {
                    name: QualifiedName::root("Option"),
                    args: vec![t],
                    kind: Kind::Type,
                }),
                // Result<T, E>
                (inner.clone(), inner.clone()).prop_map(|(t, e)| Type::Constructor {
                    name: QualifiedName::root("Result"),
                    args: vec![t, e],
                    kind: Kind::Type,
                }),
                // List<T>
                inner.prop_map(|t| Type::Constructor {
                    name: QualifiedName::root("List"),
                    args: vec![t],
                    kind: Kind::Type,
                }),
            ]
        })
    }

    /// Unification is sound: if unify(a, b) = Ok(s), then s(a) = s(b)
    proptest! {
        #[test]
        fn unification_sound(a in type_strategy(), b in type_strategy()) {
            if let Ok(sub) = unify(&a, &b) {
                let a_applied = sub.apply(&a);
                let b_applied = sub.apply(&b);
                prop_assert_eq!(a_applied, b_applied);
            }
        }
    }

    /// Reflexivity: unify(t, t) always succeeds
    proptest! {
        #[test]
        fn unification_reflexive(t in type_strategy()) {
            let result = unify(&t, &t);
            prop_assert!(result.is_ok());
            prop_assert_eq!(result.unwrap(), Substitution::new());
        }
    }

    /// Symmetry: if unify(a, b) succeeds, unify(b, a) succeeds
    proptest! {
        #[test]
        fn unification_symmetric(a in type_strategy(), b in type_strategy()) {
            let r1 = unify(&a, &b);
            let r2 = unify(&b, &a);

            match (r1, r2) {
                (Ok(s1), Ok(s2)) => {
                    prop_assert_eq!(s1.apply(&a), s2.apply(&a));
                    prop_assert_eq!(s1.apply(&b), s2.apply(&b));
                }
                (Err(_), Err(_)) => (),
                (Ok(_), Err(_)) | (Err(_), Ok(_)) => {
                    prop_assert!(false, "unification not symmetric");
                }
            }
        }
    }

    /// Constructor names must match
    proptest! {
        #[test]
        fn constructor_names_must_match(
            args in prop::collection::vec(type_strategy(), 1..3)
        ) {
            let opt = Type::Constructor {
                name: QualifiedName::root("Option"),
                args: args.clone(),
                kind: Kind::Type,
            };
            let res = Type::Constructor {
                name: QualifiedName::root("Result"),
                args,
                kind: Kind::Type,
            };

            prop_assert!(unify(&opt, &res).is_err());
        }
    }
}
```

**Step 3: Run property tests**

Run: `cargo test --package ash-typeck proptests -- --nocapture`

Expected: 100+ test cases pass for each property

**Step 4: Commit**

```bash
git add crates/ash-typeck/Cargo.toml crates/ash-typeck/src/types.rs
git commit -m "test(typeck): add property tests for generic unification

- Soundness: unification produces equal types after substitution
- Reflexivity: t ~ t always succeeds
- Symmetry: unification order doesn't matter
- Constructor name matching verification"
```

---

## Task 10: Add Integration Tests

**Files:**
- Create: `tests/generic_type_check.rs`

**Context:** End-to-end tests for the full generic type checking pipeline.

**Step 1: Create integration test file**

Create `tests/generic_type_check.rs`:

```rust
//! End-to-end tests for parametric polymorphism

use ash_parser::parse;
use ash_typeck::{type_check, Type};

fn type_check_source(source: &str) -> Result<TypeCheckResult, String> {
    let program = parse(source).map_err(|e| format!("Parse error: {:?}", e))?;
    let result = type_check(&program).map_err(|e| format!("Type error: {:?}", e))?;
    Ok(result)
}

struct TypeCheckResult {
    types: std::collections::HashMap<String, Type>,
}

impl TypeCheckResult {
    fn get(&self, name: &str) -> &Type {
        self.types.get(name).unwrap()
    }
}

#[test]
fn basic_generic_construction() {
    let source = r#"
        type Option<T> = Some { value: T } | None;
        
        let x: Option<Int> = Some { value: 42 };
        let y: Option<String> = Some { value: "hello" };
    "#;

    let result = type_check_source(source).unwrap();
    
    assert_eq!(result.get("x").to_string(), "Option<Int>");
    assert_eq!(result.get("y").to_string(), "Option<String>");
}

#[test]
fn generic_type_mismatch_error() {
    let source = r#"
        type Option<T> = Some { value: T } | None;
        
        fn want_option_int(opt: Option<Int>) {}
        
        let s = Some { value: "hello" };
        want_option_int(s);
    "#;

    let err = type_check_source(source).unwrap_err();
    assert!(err.contains("Option<Int>"), "Error should mention Option<Int>: {}", err);
    assert!(err.contains("Option<String>"), "Error should mention Option<String>: {}", err);
}

#[test]
fn nested_generics() {
    let source = r#"
        type Option<T> = Some { value: T } | None;
        type List<T> = Cons { head: T, tail: List<T> } | Nil;
        
        let nested: Option<List<Int>> = Some { 
            value: Cons { head: 1, tail: Nil }
        };
    "#;

    let result = type_check_source(source);
    assert!(result.is_ok(), "Nested generics failed: {:?}", result);
}

#[test]
fn recursive_list() {
    let source = r#"
        type List<T> = Cons { head: T, tail: List<T> } | Nil;
        
        let nums: List<Int> = Cons { 
            head: 1, 
            tail: Cons { head: 2, tail: Nil } 
        };
    "#;

    let result = type_check_source(source);
    assert!(result.is_ok(), "Recursive list failed: {:?}", result);
}

#[test]
fn type_alias_expansion() {
    let source = r#"
        type List<T> = Cons { head: T, tail: List<T> } | Nil;
        type IntList = List<Int>;
        
        let xs: IntList = Cons { head: 1, tail: Nil };
        let ys: List<Int> = xs;
    "#;

    let result = type_check_source(source);
    assert!(result.is_ok(), "Type alias failed: {:?}", result);
}

#[test]
fn pattern_matching_generics() {
    let source = r#"
        type Option<T> = Some { value: T } | None;
        
        fn map<A, B>(opt: Option<A>, f: A -> B): Option<B> {
            match opt {
                Some { value: x } -> Some { value: f(x) },
                None -> None,
            }
        }
    "#;

    let result = type_check_source(source);
    assert!(result.is_ok(), "Pattern matching generics failed: {:?}", result);
}

#[test]
fn non_exhaustive_match_detected() {
    let source = r#"
        type Option<T> = Some { value: T } | None;
        
        fn unwrap_or_default<T>(opt: Option<T>, default: T): T {
            match opt {
                Some { value: x } -> x,
            }
        }
    "#;

    let err = type_check_source(source).unwrap_err();
    assert!(err.contains("non-exhaustive"), "Should detect non-exhaustive match: {}", err);
    assert!(err.contains("None"), "Should mention missing None variant: {}", err);
}

#[test]
fn different_constructors_dont_unify() {
    let source = r#"
        type Option<T> = Some { value: T } | None;
        type Result<T, E> = Ok { value: T } | Err { error: E };
        
        fn want_result(r: Result<Int, String>) {}
        
        let opt: Option<Int> = Some { value: 42 };
        want_result(opt);
    "#;

    let err = type_check_source(source).unwrap_err();
    assert!(err.contains("Result<Int, String>"), "Error should mention Result: {}", err);
    assert!(err.contains("Option<Int>"), "Error should mention Option: {}", err);
}

#[test]
fn generic_function_instantiation() {
    let source = r#"
        type Option<T> = Some { value: T } | None;
        
        fn identity<T>(x: T): T { x }
        
        let a = identity(42);
        let b = identity("hello");
    "#;

    let result = type_check_source(source).unwrap();
    
    assert_eq!(result.get("a").to_string(), "Int");
    assert_eq!(result.get("b").to_string(), "String");
}
```

**Step 2: Run integration tests**

Run: `cargo test --test generic_type_check -- --nocapture`

Expected: All 8 tests pass

**Step 3: Commit**

```bash
git add tests/generic_type_check.rs
git commit -m "test(typeck): add integration tests for parametric polymorphism

- Basic generic construction
- Type mismatch error messages
- Nested and recursive types
- Type alias expansion
- Pattern matching on generics
- Exhaustiveness checking
- Cross-constructor unification failures
- Generic function instantiation"
```

---

## Task 11: Update Error Messages

**Files:**
- Create: `crates/ash-typeck/src/error_format.rs`
- Modify: `crates/ash-typeck/src/errors.rs` (if needed)

**Context:** Ensure error messages show meaningful type names, not `Var<42>`.

**Step 1: Create error formatting module**

Create `crates/ash-typeck/src/error_format.rs`:

```rust
//! Pretty error message formatting for type errors

use crate::types::{Type, TypeVar};
use std::collections::HashMap;

/// Maps type variables to readable names
pub struct TypeVarNames {
    names: HashMap<TypeVar, String>,
    next_anon: usize,
}

impl TypeVarNames {
    pub fn new() -> Self {
        Self {
            names: HashMap::new(),
            next_anon: 0,
        }
    }

    /// Register a source name for a type variable
    pub fn register(&mut self, var: TypeVar, name: impl Into<String>) {
        self.names.insert(var, name.into());
    }

    /// Get or create a name for a variable
    pub fn get_name(&mut self, var: TypeVar) -> &str {
        self.names.entry(var).or_insert_with(|| {
            let name = format!("t{}", self.next_anon);
            self.next_anon += 1;
            name
        })
    }

    /// Format a type with readable variable names
    pub fn format_type(&mut self, ty: &Type) -> String {
        match ty {
            Type::Var(v) => self.get_name(*v).to_string(),
            Type::Constructor { name, args, .. } => {
                if args.is_empty() {
                    name.display()
                } else {
                    let args_str: Vec<_> = args.iter()
                        .map(|a| self.format_type(a))
                        .collect();
                    format!("{}<{}>", name.display(), args_str.join(", "))
                }
            }
            Type::List(elem) => {
                format!("List<{}>", self.format_type(elem))
            }
            Type::Record(fields) => {
                let fields_str: Vec<_> = fields.iter()
                    .map(|(n, t)| format!("{}: {}", n, self.format_type(t)))
                    .collect();
                format!("{{ {} }}", fields_str.join(", "))
            }
            Type::Fun(params, ret, _) => {
                let params_str: Vec<_> = params.iter()
                    .map(|p| self.format_type(p))
                    .collect();
                format!("({}) -> {}", params_str.join(", "), self.format_type(ret))
            }
            _ => ty.to_string(), // Primitives
        }
    }
}

impl Default for TypeVarNames {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute structural difference between types
#[derive(Debug, Clone)]
pub enum TypeDiff {
    Mismatch,
    ConstructorArgs { position: usize, inner: Box<TypeDiff> },
    FunctionParam { position: usize, inner: Box<TypeDiff> },
    FunctionReturn(Box<TypeDiff>),
    RecordField { field: String, inner: Box<TypeDiff> },
}

pub fn compute_type_diff(t1: &Type, t2: &Type) -> TypeDiff {
    use Type::*;
    
    match (t1, t2) {
        (Constructor { name: n1, args: a1, .. }, Constructor { name: n2, args: a2, .. }) => {
            if n1 != n2 {
                return TypeDiff::Mismatch;
            }
            for (i, (arg1, arg2)) in a1.iter().zip(a2.iter()).enumerate() {
                let inner = compute_type_diff(arg1, arg2);
                if !matches!(inner, TypeDiff::Mismatch) || arg1 != arg2 {
                    return TypeDiff::ConstructorArgs {
                        position: i,
                        inner: Box::new(inner),
                    };
                }
            }
            TypeDiff::Mismatch
        }
        (Fun(p1, r1, _), Fun(p2, r2, _)) => {
            for (i, (p1i, p2i)) in p1.iter().zip(p2.iter()).enumerate() {
                let inner = compute_type_diff(p1i, p2i);
                if !matches!(inner, TypeDiff::Mismatch) || p1i != p2i {
                    return TypeDiff::FunctionParam {
                        position: i,
                        inner: Box::new(inner),
                    };
                }
            }
            let ret_diff = compute_type_diff(r1, r2);
            if !matches!(ret_diff, TypeDiff::Mismatch) || r1 != r2 {
                TypeDiff::FunctionReturn(Box::new(ret_diff))
            } else {
                TypeDiff::Mismatch
            }
        }
        (Record(f1), Record(f2)) => {
            for ((n1, t1), (n2, t2)) in f1.iter().zip(f2.iter()) {
                if n1 != n2 {
                    return TypeDiff::Mismatch;
                }
                let inner = compute_type_diff(t1, t2);
                if !matches!(inner, TypeDiff::Mismatch) || t1 != t2 {
                    return TypeDiff::RecordField {
                        field: n1.to_string(),
                        inner: Box::new(inner),
                    };
                }
            }
            TypeDiff::Mismatch
        }
        _ => TypeDiff::Mismatch,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::qualified_name::QualifiedName;
    use crate::kind::Kind;

    #[test]
    fn format_simple_constructor() {
        let mut names = TypeVarNames::new();
        let ty = Type::Constructor {
            name: QualifiedName::root("Option"),
            args: vec![Type::Int],
            kind: Kind::Type,
        };
        assert_eq!(names.format_type(&ty), "Option<Int>");
    }

    #[test]
    fn format_with_type_var() {
        let mut names = TypeVarNames::new();
        let v = TypeVar(0);
        let ty = Type::Constructor {
            name: QualifiedName::root("Option"),
            args: vec![Type::Var(v)],
            kind: Kind::Type,
        };
        // Variable gets auto-named
        let formatted = names.format_type(&ty);
        assert!(formatted.starts_with("Option<"));
        assert!(formatted.ends_with(">"));
    }

    #[test]
    fn format_with_named_var() {
        let mut names = TypeVarNames::new();
        let v = TypeVar(0);
        names.register(v, "T");
        
        let ty = Type::Constructor {
            name: QualifiedName::root("Option"),
            args: vec![Type::Var(v)],
            kind: Kind::Type,
        };
        assert_eq!(names.format_type(&ty), "Option<T>");
    }
}
```

**Step 2: Add module to lib.rs**

```rust
pub mod error_format;
pub use error_format::{TypeVarNames, TypeDiff, compute_type_diff};
```

**Step 3: Integrate into error reporting**

Update error types to use the formatter (if needed in errors.rs):

```rust
use crate::error_format::TypeVarNames;

impl TypeError {
    pub fn format(&self) -> String {
        let mut names = TypeVarNames::new();
        // ... use names.format_type() for types in error messages
        self.to_string()
    }
}
```

**Step 4: Run tests**

Run: `cargo test --package ash-typeck error_format -- --nocapture`

Expected: All tests pass

**Step 5: Commit**

```bash
git add crates/ash-typeck/src/error_format.rs crates/ash-typeck/src/lib.rs
git commit -m "feat(typeck): add readable error message formatting

- TypeVarNames for mapping vars to readable names (T, t0, etc.)
- format_type for pretty-printing Type::Constructor
- TypeDiff for structural difference detection
- Improves UX for type mismatch errors"
```

---

## Task 12: Final Verification and Cleanup

**Files:**
- All modified files

**Context:** Ensure no stubs remain, all tests pass, update CHANGELOG.

**Step 1: Check for stub implementations**

Run:
```bash
grep -r "TODO\|FIXME\|stub\|placeholder\|unimplemented" crates/ash-typeck/src/ --include="*.rs"
```

Expected: Empty output (or only in comments with issue numbers)

**Step 2: Run full test suite**

Run:
```bash
cargo test --package ash-typeck -- --nocapture 2>&1 | tail -50
```

Expected: All tests pass

**Step 3: Run clippy**

Run:
```bash
cargo clippy --package ash-typeck --all-targets --all-features 2>&1 | grep -E "^error|^warning" | head -20
```

Expected: No errors, minimal warnings

**Step 4: Check formatting**

Run:
```bash
cargo fmt --package ash-typeck -- --check
```

Expected: Clean (no output means formatted)

**Step 5: Update CHANGELOG.md**

Add to top of CHANGELOG.md:

```markdown
## [Unreleased]

### Added
- Full parametric polymorphism (generics) for Ash type system. Type constructors like `Option<Int>` and `Option<String>` are now distinct, distinguishable types. (TASK-127, TASK-128, TASK-129, TASK-130)
- `Type::Constructor` variant with `QualifiedName`, type arguments, and `Kind` annotation for future higher-kinded type support.
- `Kind` system for classifying type constructors (`*`, `* -> *`, etc.).
- `QualifiedName` for module-qualified type names.
- Iso-recursive type unfolding for generic field access and pattern matching.
- Pattern typing and exhaustiveness checking for generic constructors.
- Property-based tests for unification soundness, reflexivity, and symmetry.

### Changed
- `type_expr_to_type` now properly converts `TypeExpr::Constructor` to `Type::Constructor` instead of losing constructor information.
- `build_constructor_type` now returns the constructor type (e.g., `Option<T>`) instead of just the type parameter.
- Type alias expansion now properly unfolds to underlying types.

### Fixed
- `Option<Int>` and `Option<String>` no longer incorrectly unify.
- Error messages now show readable type names (`Option<Int>`) instead of internal variable IDs (`Var<42>`).
```

**Step 6: Final commit**

```bash
git add CHANGELOG.md
git commit -m "docs: update CHANGELOG for parametric polymorphism

- Document all new features and fixes
- Reference related tasks: TASK-127, TASK-128, TASK-129, TASK-130"
```

**Step 7: Verify no gaps**

Run verification commands:

```bash
# Check all Type enum variants are handled
grep -n "Type::" crates/ash-typeck/src/types.rs | grep -v "//" | head -30

# Verify unification completeness
grep -A 30 "pub fn unify" crates/ash-typeck/src/types.rs

# Check for bare string names
grep -n '"[A-Z][a-zA-Z]*"' crates/ash-typeck/src/types.rs | head -10

# Integration test
cargo test --test generic_type_check 2>&1 | tail -20
```

**Step 8: Final summary**

Document completion:

```bash
echo "=== Parametric Polymorphism Implementation Complete ==="
echo "Files modified:"
git diff --name-only HEAD~12

echo ""
echo "Test results:"
cargo test --package ash-typeck 2>&1 | grep -E "^test|test result"
```

---

## Summary

This implementation plan delivers full parametric polymorphism for Ash through 12 tasks:

1. **Foundation** (Tasks 1-2): `Kind`, `QualifiedName`, and `Type::Constructor`
2. **Core Logic** (Tasks 3-6): Unification, type conversion, constructor typing, recursive unfolding
3. **Features** (Tasks 7-8): Pattern typing and exhaustiveness checking
4. **Quality** (Tasks 9-11): Property tests, integration tests, error messages
5. **Verification** (Task 12): No stubs, all tests pass, CHANGELOG updated

**Key Design Decisions:**
- Iso-recursive types (opaque constructors, unfold on demand)
- Kind annotations for future HKT support
- Qualified names for module boundaries
- Readable error messages with `TypeVarNames`

**Verification Checklist:**
- [x] `Type::Constructor` with name, args, kind
- [x] Unification with occurs check
- [x] Type expression conversion
- [x] Constructor typing
- [x] Recursive unfolding
- [x] Pattern typing
- [x] Exhaustiveness checking
- [x] Property tests
- [x] Integration tests
- [x] Error message formatting
- [x] No stub implementations
- [x] CHANGELOG updated


---

## Task 13: Update SPEC-003-Type-System.md

**Files:**
- Modify: `docs/spec/SPEC-003-TYPE-SYSTEM.md`

**Context:** Update the type system specification to document `Type::Constructor`, `Kind`, `QualifiedName`, and constructor unification.

**Correlation Points:**
| Implementation | Spec Section |
|---------------|--------------|
| `Type::Constructor` | Section 3: Value Types |
| `Kind` enum | Section 3: Value Types (new subsection) |
| `QualifiedName` | Section 3: Value Types (new subsection) |
| Constructor unification | Section 7.2: Unification |
| `TypeVarNames` | Section 9: Error Messages |

**Step 1: Update Section 3 - Value Types**

Replace the Type enum definition (lines 58-74):

```markdown
### 3.1 Value Type Representation

```rust
pub enum Type {
    // Primitive types
    Int,
    String,
    Bool,
    Null,
    Time,
    Ref(Box<str>),
    
    // Container types
    List(Box<Type>),
    Record(Vec<(Box<str>, Type)>),
    
    // Type constructor application: Option<Int>, List<T>, Result<T, E>
    Constructor {
        name: QualifiedName,    // Fully qualified constructor name
        args: Vec<Type>,        // Type arguments (all have kind *)
        kind: Kind,             // Kind of the applied constructor
    },
    
    // Capability and function types
    Cap(Box<str>, Effect),
    Fun(Vec<Type>, Box<Type>, Effect),
    
    // Type inference variable
    Var(TypeVar),
}

pub struct TypeVar(pub u32);
```

### 3.2 Kinds

Kinds classify types and type constructors:

```rust
pub enum Kind {
    /// The kind of types: *
    Type,
    /// Function kind: K1 -> K2
    Arrow(Box<Kind>, Box<Kind>),
}
```

- `Int`, `String`, `Option<Int>` have kind `*` (proper types)
- `Option`, `List` have kind `* -> *` (unary type constructors)
- `Result`, `Pair` have kind `* -> * -> *` (binary type constructors)

### 3.3 Qualified Names

Type constructors are referenced by fully qualified names:

```rust
pub struct QualifiedName {
    pub module: Vec<String>,    // ["Std", "Maybe"]
    pub name: String,           // "Option"
}
```

Qualified names support module boundaries and avoid naming conflicts.
```

**Step 2: Update Section 7.2 - Unification**

Replace the unification pseudocode (lines 339-352):

```markdown
### 7.2 Unification

```rust
fn unify(t1: Type, t2: Type) -> Result<Substitution, TypeError> {
    match (t1, t2) {
        // Identical primitives
        (Int, Int) | (String, String) | (Bool, Bool) => Ok(empty_subst()),
        
        // Variable binding
        (Var(v), t) | (t, Var(v)) => bind_var(v, t),
        
        // Container types (structural)
        (List(a), List(b)) => unify(*a, *b),
        (Record(f1), Record(f2)) => unify_records(f1, f2),
        
        // Type constructor unification
        (Constructor { name: n1, args: a1, .. }, 
         Constructor { name: n2, args: a2, .. }) => {
            if n1 != n2 {
                return Err(ConstructorNameMismatch(n1, n2));
            }
            if a1.len() != a2.len() {
                return Err(ConstructorArityMismatch(n1, a1.len(), a2.len()));
            }
            // Unify corresponding arguments
            let mut subst = empty_subst();
            for (arg1, arg2) in a1.iter().zip(a2.iter()) {
                subst = subst.compose(unify(
                    subst.apply(arg1), 
                    subst.apply(arg2)
                )?);
            }
            Ok(subst)
        }
        
        // Constructor cannot unify with primitive
        (Constructor { .. }, _) | (_, Constructor { .. }) => {
            Err(TypeMismatch(t1, t2))
        }
        
        // Function types (contravariant in params, covariant in return)
        (Fun(p1, r1, e1), Fun(p2, r2, e2)) => {
            if e1 != e2 { return Err(EffectMismatch(e1, e2)); }
            if p1.len() != p2.len() { return Err(ArityMismatch); }
            let mut subst = empty_subst();
            // Contravariant: unify p2 with p1 (reversed!)
            for (a, b) in p2.iter().zip(p1.iter()) {
                subst = subst.compose(unify(subst.apply(b), subst.apply(a))?);
            }
            // Covariant: unify returns
            subst = subst.compose(unify(subst.apply(r1), subst.apply(r2))?);
            Ok(subst)
        }
        
        _ => Err(TypeMismatch(t1, t2)),
    }
}

fn bind_var(var: TypeVar, ty: Type) -> Result<Substitution, TypeError> {
    if ty == Type::Var(var) { return Ok(empty_subst()); }
    if occurs_in(var, &ty) { return Err(InfiniteType(var, ty)); }
    Ok(singleton_subst(var, ty))
}

fn occurs_in(var: TypeVar, ty: &Type) -> bool {
    match ty {
        Type::Var(v) => *v == var,
        Type::Constructor { args, .. } => args.iter().any(|a| occurs_in(var, a)),
        Type::List(elem) => occurs_in(var, elem),
        Type::Record(fields) => fields.iter().any(|(_, f)| occurs_in(var, f)),
        Type::Fun(params, ret, _) => {
            params.iter().any(|p| occurs_in(var, p)) || occurs_in(var, ret)
        }
        _ => false,
    }
}
```
```

**Step 3: Update Section 9 - Error Messages**

Add after line 414 (before Related Documents):

```markdown
### 9.1 Type Variable Naming

Error messages display type variables using readable names:

```rust
pub struct TypeVarNames {
    names: HashMap<TypeVar, String>,
    next_anon: usize,
}

impl TypeVarNames {
    /// Register source name for a type variable (from type parameter)
    pub fn register(&mut self, var: TypeVar, name: &str);
    
    /// Get or create readable name
    pub fn get_name(&mut self, var: TypeVar) -> &str;
    
    /// Format type with readable variable names
    pub fn format_type(&mut self, ty: &Type) -> String;
}
```

Examples:
- `Option<Int>` displays as `Option<Int>`
- `Option<Var(0)>` with registered name "T" displays as `Option<T>`
- `Option<Var(42)>` displays as `Option<t0>` (auto-generated)

### 9.2 Type Difference Reporting

Type mismatch errors identify where types differ:

```
error[E001]: Type mismatch
  --> example.ash:10:15
   |
10 |     want_option_int(s)
   |                 ^ expected `Option<Int>`, found `Option<String>`
   |
   = note: Type arguments differ at position 0
   = help: Ensure all type arguments match exactly
```
```

**Step 4: Commit**

```bash
git add docs/spec/SPEC-003-TYPE-SYSTEM.md
git commit -m "docs(spec): update SPEC-003 with Type::Constructor and generics

- Document Type::Constructor with QualifiedName, args, and Kind
- Add Kind enum for type constructor classification
- Add QualifiedName for module-qualified type names
- Update unification algorithm with constructor cases
- Add TypeVarNames for readable error messages"
```

---

## Task 14: Update SPEC-020-ADT-TYPES.md

**Files:**
- Modify: `docs/spec/SPEC-020-ADT-TYPES.md`

**Context:** Update the ADT specification to reflect the actual implementation structure and mark TASK-127 through TASK-130 as complete.

**Correlation Points:**
| Implementation | Spec Section |
|---------------|--------------|
| `Type::Constructor` | Section 4.1 (already has TypeExpr::Constructor) |
| Pattern typing | Section 6.2 |
| Exhaustiveness | Section 6.3 |
| Implementation phases | Section 11 |

**Step 1: Update Section 4.1 - Canonical Source Definition Model**

Add note about internal representation (after line 74):

```markdown
**Internal Representation**: The source `TypeDef` and `TypeExpr` model is canonical for
user-written declarations. The type checker elaborates this into an internal representation
featuring:

- `Type::Constructor { name: QualifiedName, args: Vec<Type>, kind: Kind }` for type constructor
   applications like `Option<Int>`
- `Kind` for classifying type constructors (`*`, `* -> *`, etc.)
- `QualifiedName` for module-qualified type names

This internal representation enables proper unification where `Option<Int>` and `Option<String>`
are distinct, distinguishable types.
```

**Step 2: Update Section 6.4 - Generic Type Instantiation**

Replace lines 251-271 with:

```markdown
### 6.4 Generic Type Instantiation

Generic types are instantiated by substituting type arguments for parameters:

```rust
/// Substitute type arguments for parameters in a type
pub struct Substitution {
    mappings: HashMap<TypeVar, Type>,
}

impl Substitution {
    /// Apply substitution to a type
    pub fn apply(&self, ty: &Type) -> Type {
        match ty {
            Type::Var(v) => self.mappings.get(v).cloned().unwrap_or_else(|| ty.clone()),
            Type::Constructor { name, args, kind } => Type::Constructor {
                name: name.clone(),
                args: args.iter().map(|a| self.apply(a)).collect(),
                kind: kind.clone(),
            },
            Type::List(elem) => Type::List(Box::new(self.apply(elem))),
            Type::Record(fields) => Type::Record(
                fields.iter().map(|(n, t)| (n.clone(), self.apply(t))).collect()
            ),
            Type::Fun(params, ret, eff) => Type::Fun(
                params.iter().map(|p| self.apply(p)).collect(),
                Box::new(self.apply(ret)),
                *eff,
            ),
            // Primitives unchanged
            _ => ty.clone(),
        }
    }
    
    /// Compose two substitutions: self ∘ other
    pub fn compose(&self, other: &Substitution) -> Substitution {
        let mut result = self.clone();
        for (var, ty) in &other.mappings {
            result.mappings.insert(*var, self.apply(ty));
        }
        result
    }
}
```

Instantiation proceeds by creating a substitution from the type definition's parameters
to the provided arguments, then applying that substitution throughout the type body.

Example:
```
instantiate(Option<T>, [Int])
  = Substitution { T => Int }.apply(Option<T>)
  = Option<Int>
```
```

**Step 3: Add Section 6.5 - Recursive Type Unfolding**

Add new subsection after 6.4:

```markdown
### 6.5 Recursive Type Unfolding

Recursive types like `List<T>` use iso-recursive representation. The constructor
application `List<Int>` is stored as an opaque `Type::Constructor`, and unfolded
to its definition only when needed:

```rust
/// Unfold a constructor to its definition with substituted type arguments
pub fn unfold_constructor(
    env: &TypeEnv,
    name: &QualifiedName,
    args: &[Type]
) -> Result<TypeBody, TypeError> {
    let type_info = env.lookup_type(name)?;
    
    match type_info {
        TypeInfo::Enum { params, body, .. } |
        TypeInfo::Struct { params, body, .. } => {
            // Create substitution from params to args
            let subst = params.iter().copied()
                .zip(args.iter().cloned())
                .collect::<Substitution>();
            
            // Apply substitution to body
            Ok(subst.apply_to_body(body))
        }
        _ => Err(TypeError::NotAConstructor(name.display())),
    }
}
```

Unfolding occurs for:
- Field access type checking
- Pattern match exhaustiveness analysis
- Constructor argument checking
```

**Step 4: Update Section 11 - Implementation Phases**

Replace lines 528-557 with:

```markdown
## 11. Implementation Status

| Phase | Tasks | Status | Description |
|-------|-------|--------|-------------|
| Core Types | TASK-121 to TASK-123 | ✅ Complete | `Type::Constructor`, `Kind`, `QualifiedName`, updated unification |
| Parser | TASK-124 to TASK-126 | ✅ Complete | Parse type definitions, variant constructors, match expressions |
| Type Checker | TASK-127 to TASK-130 | ✅ Complete | Constructor typing, pattern typing, exhaustiveness, instantiation |
| Interpreter | TASK-131 to TASK-133 | 🔄 Pending | Evaluate constructors, pattern matching engine |
| Integration | TASK-134 to TASK-135 | 🔄 Pending | Spawn returns `Option<ControlLink>`, control link transfer |
| Standard Library | TASK-136 | 🔄 Pending | Option and Result modules |

### Completed Work

**Type Representation (TASK-121, TASK-122)**
- `Kind` enum for type constructor classification
- `QualifiedName` for module-qualified type names  
- `Type::Constructor { name, args, kind }` variant

**Unification (TASK-123)**
- Constructor vs constructor unification (name and arity checking)
- Constructor vs variable binding with occurs check
- Infinite type detection for recursive types

**Parser (TASK-124 to TASK-126)**
- Generic type definition syntax: `type Option<T> = Some { value: T } | None`
- Constructor expressions: `Some { value: 42 }`
- Pattern matching syntax with variant patterns

**Type Checker (TASK-127 to TASK-130)**
- **TASK-127**: Constructor typing returns `Option<T>` not just `T`
- **TASK-128**: Pattern typing for generic constructors
- **TASK-129**: Generic instantiation with substitution
- **TASK-130**: Exhaustiveness checking for generic enums

**Key Properties**
- `Option<Int>` and `Option<String>` are distinct types that do not unify
- Nested constructors like `Option<List<Int>>` work correctly
- Type aliases expand properly through unfolding
- Error messages show readable type names
```

**Step 5: Commit**

```bash
git add docs/spec/SPEC-020-ADT-TYPES.md
git commit -m "docs(spec): update SPEC-020 with implementation details and status

- Document internal Type::Constructor representation
- Add Substitution with apply and compose operations
- Add iso-recursive type unfolding section
- Update implementation phases with completion status
- Mark TASK-127 through TASK-130 as complete"
```

---

## Task 15: Update SPEC-001-IR.md

**Files:**
- Modify: `docs/spec/SPEC-001-IR.md`

**Context:** Update the IR specification to include `Value::Variant` and `Pattern::Variant` for ADT support.

**Correlation Points:**
| Implementation | Spec Section |
|---------------|--------------|
| `Value::Variant` | Section 2.3 Values |
| `Pattern::Variant` | Section 2.4 Patterns |

**Step 1: Update Section 2.0 - Canonical Core Language**

The core patterns list (line 45) already mentions `Variant`, but the Pattern enum in 2.4 doesn't include it. Update the comment to be explicit:

```markdown
- core patterns: `Variable`, `Tuple`, `Record`, `List`, `Wildcard`, `Literal`, 
  `Variant` (for ADT constructor patterns like `Some { value: x }`)
```

**Step 2: Update Section 2.3 - Values**

Replace the Value enum (lines 240-251):

```markdown
### 2.3 Values

```rust
pub enum Value {
    // Primitive values
    Int(i64),
    String(Box<str>),        // Boxed for smaller enum size
    Bool(bool),
    Null,
    Time(DateTime<Utc>),
    Ref(Box<str>),
    
    // Container values
    List(Box<[Value]>),
    Record(HashMap<Box<str>, Value>),
    
    // ADT variant value
    Variant {
        constructor: String,           // Constructor name: "Some", "None", "Ok"
        fields: Vec<(String, Value)>,  // Named field values
    },
    
    // Capability reference
    Cap(Box<str>),
}
```

**Invariants**:

- `List` and `Record` are immutable after creation
- `String` is valid UTF-8
- `Ref` is a valid URI
- `Variant` constructor names match their type definition
- `Variant` fields are ordered according to the constructor definition
```

**Step 3: Update Section 2.4 - Patterns**

Replace the Pattern enum (lines 262-270):

```markdown
### 2.4 Patterns

```rust
pub enum Pattern {
    // Binding patterns
    Variable(Name),
    Wildcard,
    
    // Structural patterns
    Tuple(Box<[Pattern]>),
    Record(Box<[(Name, Pattern)]>),
    List(Box<[Pattern]>, Option<Name>), // [a, b, ..rest]
    
    // ADT variant pattern
    Variant {
        constructor: Name,
        fields: Box<[(Name, Pattern)]>,
    },
    
    // Literal pattern
    Literal(Value),
}
```

**Matching Rules**:

- `Variable` binds any value to the name
- `Wildcard` matches any, no binding
- `Tuple` matches same-length list value
- `Record` matches superset of fields (extra fields ignored)
- `List` matches exact prefix, binds rest if specified
- `Variant` matches constructor name and recursively matches fields
- `Literal` matches equal value

**ADT Pattern Example**:

```ash
match opt {
    Some { value: x } => x * 2,    -- Variant pattern
    None => 0,
}
```

The `Some { value: x }` pattern matches `Value::Variant` with constructor "Some"
and binds the "value" field to `x`.
```

**Step 4: Commit**

```bash
git add docs/spec/SPEC-001-IR.md
git commit -m "docs(spec): update SPEC-001 with ADT Variant for IR

- Add Value::Variant for ADT constructor values
- Add Pattern::Variant for ADT constructor patterns
- Document matching rules for variant patterns
- Add example showing pattern matching on Option"
```

---

## Task 16: Update SPEC-009-MODULES.md (Optional)

**Files:**
- Modify: `docs/spec/SPEC-009-MODULES.md`

**Context:** Add section on qualified type name resolution if not already present.

**Step 1: Check current content**

Read lines 175-185 of SPEC-009 to see if type name resolution is documented.

If not present, add new section after Section 7:

```markdown
## 8. Type Name Resolution

Type names follow the same resolution rules as other items, with support for
qualified paths:

### 8.1 Qualified Type Names

```rust
pub struct QualifiedName {
    pub module: Vec<String>,    // ["Std", "Maybe"]
    pub name: String,           // "Option"
}
```

### 8.2 Resolution Order

Type names are resolved in this order:

1. **Type parameters**: `T` in `type Option<T> = ...`
2. **Local types**: Defined in current module
3. **Imported types**: From `use` statements
4. **Fully qualified**: `Std.Maybe.Option`

### 8.3 Examples

```ash
-- Using imported type
import Std.Maybe exposing (Option, Some);

let x: Option<Int> = Some { value: 42 };

-- Using fully qualified name
let y: Std.Maybe.Option<String> = ...;

-- Generic type with imported type argument
import Data.Tree as Tree;
let tree: Tree.Node<Int> = ...;
```
```

**Step 2: Commit (if changes made)**

```bash
git add docs/spec/SPEC-009-MODULES.md
git commit -m "docs(spec): add qualified type name resolution to SPEC-009

- Document QualifiedName structure for types
- Add type name resolution order
- Add examples of imported and qualified type usage"
```

---

## Task 17: Verify Spec Updates

**Files:**
- All modified spec files

**Context:** Ensure spec updates are consistent and cross-reference correctly.

**Step 1: Cross-reference check**

Verify all specs reference each other correctly:

```bash
# Check SPEC-003 references to SPEC-020
grep -n "SPEC-020" docs/spec/SPEC-003-TYPE-SYSTEM.md

# Check SPEC-020 references to SPEC-003
grep -n "SPEC-003" docs/spec/SPEC-020-ADT-TYPES.md

# Check SPEC-001 references to SPEC-020
grep -n "SPEC-020\|ADT\|Variant" docs/spec/SPEC-001-IR.md
```

**Step 2: Consistency check**

Ensure type definitions match across specs:

```bash
# Check Type enum consistency
echo "=== SPEC-003 Type enum ==="
grep -A 30 "pub enum Type" docs/spec/SPEC-003-TYPE-SYSTEM.md | head -35

echo ""
echo "=== SPEC-020 TypeExpr enum ==="
grep -A 10 "pub enum TypeExpr" docs/spec/SPEC-020-ADT-TYPES.md
```

**Step 3: Final spec commit**

```bash
git add docs/spec/
git commit -m "docs(spec): complete parametric polymorphism specification updates

- SPEC-003: Type::Constructor, Kind, QualifiedName, unification
- SPEC-020: Implementation status, Substitution, iso-recursive unfolding
- SPEC-001: Value::Variant, Pattern::Variant for ADT IR
- Cross-references verified between all specs"
```

---

## Updated Summary

This implementation plan now includes **17 tasks** across implementation and specification:

### Implementation (Tasks 1-12)
1. **Foundation**: `Kind`, `QualifiedName`, `Type::Constructor`
2. **Core Logic**: Unification, type conversion, constructor typing, recursive unfolding
3. **Features**: Pattern typing, exhaustiveness checking
4. **Quality**: Property tests, integration tests, error messages
5. **Verification**: No stubs, CHANGELOG updated

### Specification (Tasks 13-17)
13. **SPEC-003**: Type system with generics
14. **SPEC-020**: ADT implementation details and status
15. **SPEC-001**: IR Variant values and patterns
16. **SPEC-009**: Qualified type name resolution (optional)
17. **Verification**: Cross-reference and consistency checks

**Documentation Checklist:**
- [x] SPEC-003: Type::Constructor, Kind, QualifiedName documented
- [x] SPEC-003: Unification algorithm with constructor cases
- [x] SPEC-003: Error message formatting with TypeVarNames
- [x] SPEC-020: Substitution and iso-recursive unfolding
- [x] SPEC-020: Implementation phases updated
- [x] SPEC-001: Value::Variant and Pattern::Variant
- [x] Cross-references between specs verified
