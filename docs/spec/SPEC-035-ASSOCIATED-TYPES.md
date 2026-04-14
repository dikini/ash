# SPEC-035: Associated Types on Interfaces

**Status:** Draft  
**Date:** 2026-04-14  
**Version:** 0.2  

## 1. Overview

Allow interfaces to declare **associated types** — type-level outputs that are determined by each `impl` block. This enables interfaces to act as type families: a single interface maps a set of input types to a specific output type.

Associated types are the final extension needed for ergonomic generic libraries. They eliminate the "type parameter explosion" seen when output types must be threaded explicitly through every generic signature. Use cases include serialization (`Serializer::Ok`, `Serializer::Error`), collection traits (`Map::Key`, `Map::Value`), and besedarium's query builder (`QueryBuilder::Result`, `QueryBuilder::Error`).

## 2. Motivation

Without associated types, a generic serializer interface forces four type parameters into every function signature:

```ash
-- Verbose and leaky
interface Serialize<T, S, Out, Err> {
    serialize(T, S) -> Result<Out, Err>
}

fn to_json<T, Out, Err>(value: T) -> Result<Out, Err>
    where T: Serialize
{
    Serialize::serialize(value, JsonWriter)
}
```

With associated types, the caller only needs to know the input types:

```ash
interface Serializer<S> {
    type Ok
    type Error
    serialize_bool(S, Bool) -> Result<S::Ok, S::Error>
    serialize_string(S, String) -> Result<S::Ok, S::Error>
}

fn to_json<T>(value: T) -> Result<String, SerializeError>
    where T: Serialize
{
    Serialize::serialize(value, JsonWriter)
}
```

The output and error types are **projected** from the `impl` block for `JsonWriter`.

## 3. Semantics

### 3.1 Interface Declaration

Associated types are declared inside an `interface` block with the `type` keyword:

```ash
interface Iterator<I> {
    type Item
    next(I) -> Option<I::Item>
}

interface Serializer<S> {
    type Ok
    type Error
    serialize_bool(S, Bool) -> Result<S::Ok, S::Error>
}
```

Associated type names are in scope for all method signatures inside the interface. They are referenced via the interface's type parameter using `Param::AssocName` syntax (e.g., `S::Ok`).

### 3.2 Concrete Implementation

Each `impl` block must provide a concrete type for every associated type:

```ash
impl Serializer<JsonWriter> {
    type Ok = String
    type Error = SerializeError
    serialize_bool(writer, value) = ...
}

impl Iterator<ListIter<Int>> {
    type Item = Int
    next(iter) = list_iter::next(iter)
}
```

Generic `impl` blocks may use their own type parameters on the right-hand side:

```ash
impl<T> Iterator<ListIter<T>> {
    type Item = T
    next(iter) = ...
}
```

### 3.3 Projection Syntax

Associated types appear in user code and method signatures as:

```ash
S::Ok
S::Error
I::Item
Map<K, V>::Entry
```

At the type-system level, this is a **type projection** that must be **normalized** (resolved to a concrete type) using the selected `impl` block.

**Ambiguity rule:** If a type variable `T` has multiple interface bounds and two or more of those interfaces declare an associated type with the same name (e.g., both `A` and `B` define `Ok`), then writing `T::Ok` is **ambiguous** and must be rejected with a [NEW] `TypeEnvError::AmbiguousAssociatedType` error. The programmer must instead use the fully explicit form `Interface::Assoc<T>` (or similar explicit syntax) to disambiguate. If exactly one bound in scope defines the name, `T::Ok` resolves to that interface's associated type.

## 4. IR Changes

### 4.1 AST Updates

**`crates/ash-core/src/ast.rs`**

```rust
pub struct InterfaceDef {
    pub name: Name,
    pub type_params: Vec<TypeVar>,
    pub associated_types: Vec<Name>,        -- NEW
    pub methods: Vec<InterfaceMethodSig>,
    pub visibility: Visibility,
}

pub struct ImplDef {
    pub visibility: Visibility,
    pub interface: Name,
    pub type_params: Vec<TypeVar>,
    pub type_args: Vec<TypeExpr>,
    pub where_bounds: Vec<WhereBound>,
    pub associated_type_bindings: Vec<(Name, TypeExpr)>,  -- NEW
    pub methods: Vec<ImplMethodDef>,
}
```

### 4.2 Type Representation

**`crates/ash-typeck/src/types.rs`**

```rust
pub enum Type {
    // ... existing variants (Int, String, List, Constructor, Fun, Fn, Var, etc.)

    /// Associated type projection: e.g., Serializer<JsonWriter>::Ok
    ///
    /// The `interface` field is required because `base` may have multiple
    /// interface bounds that each define an associated type with the same name.
    Associated {
        interface: String,         -- e.g., "Serializer"
        base: Box<Type>,           -- e.g., JsonWriter
        name: String,              -- e.g., "Ok"
    },
}
```

During parsing and lowering, `S::Ok` becomes `Type::Associated { interface: "Serializer", base: Type::Var(S), name: "Ok" }` when `S` is known to be bound by `Serializer`.

### 4.3 Parser Updates

**`crates/ash-parser/src/parse_module.rs`**

This is a **structural parser-body redesign**, not just an additive grammar tweak. The current body loops only parse methods:

```rust
while !input.input.starts_with("}") {
    methods.push(parse_interface_method_signature(input)?);
    ...
}
```

They must be replaced with dispatch loops that can parse either a method or an associated-type declaration:

- `parse_interface_definition` reads `type Name` declarations inside the interface body.
- `parse_impl_definition` reads `type Name = TypeExpr` bindings inside the impl body.
- `parse_surface_type` (in `parse_module.rs`) is extended to parse `Type::Associated` when encountering `identifier "::" identifier` in a type context.

Example grammar:

```
associated-type = identifier "::" identifier

interface-body = "{" ( interface-method | associated-type-decl )* "}"
associated-type-decl = "type" identifier

impl-body = "{" ( impl-method | associated-type-binding )* "}"
associated-type-binding = "type" identifier "=" type-expr
```

## 5. Type System Changes

### 5.1 Interface Registration

`register_interface` stores the list of associated type names in `InterfaceInfo`:

```rust
pub struct InterfaceInfo {
    pub name: String,
    pub type_params: Vec<String>,
    pub associated_types: Vec<String>,
    pub methods: HashMap<String, InterfaceMethodInfo>,
}
```

### 5.2 Impl Registration

`register_impl` validates that every associated type declared by the interface has exactly one binding in the `impl` block, and that no extra bindings are present.

### 5.3 Type Normalization

The core new operation is **associated type normalization**: replacing `Type::Associated` with its concrete definition.

```rust
/// Normalize all associated type projections in `ty` using the selected impl scheme.
pub fn normalize_associated_types(&self, ty: &Type, scheme: &ImplScheme) -> Result<Type, TypeEnvError> {
    match ty {
        Type::Associated { interface, base, name } => {
            // 1. Verify the scheme matches the projected interface
            if scheme.interface != *interface {
                return Err(TypeEnvError::[NEW] MismatchedProjectionInterface { ... });
            }
            // 2. Look up `name` in the scheme's associated_type_bindings
            let binding = scheme.associated_type_bindings.get(name)
                .ok_or_else(|| TypeEnvError::[NEW] MissingAssociatedType { ... })?;
            // 3. Apply the scheme substitution (from head unification) to the binding
            let normalized = scheme.substitution.apply(binding);
            // 4. Recursively normalize
            self.normalize_associated_types(&normalized, scheme)
        }
        Type::Constructor { name, args, kind } => {
            let normalized_args = args.iter()
                .map(|a| self.normalize_associated_types(a, scheme))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Type::Constructor { name: name.clone(), args: normalized_args, kind: kind.clone() })
        }
        Type::Fun(params, return_type, effect) => {
            let normalized_params = params.iter()
                .map(|p| self.normalize_associated_types(p, scheme))
                .collect::<Result<Vec<_>, _>>()?;
            let normalized_return = self.normalize_associated_types(return_type, scheme)?;
            Ok(Type::Fun(normalized_params, Box::new(normalized_return), *effect))
        }
        Type::Fn(params, return_type) => {
            let normalized_params = params.iter()
                .map(|p| self.normalize_associated_types(p, scheme))
                .collect::<Result<Vec<_>, _>>()?;
            let normalized_return = self.normalize_associated_types(return_type, scheme)?;
            Ok(Type::Fn(normalized_params, Box::new(normalized_return)))
        }
        // ... other variants recurse similarly
        other => Ok(other.clone()),
    }
}
```

Normalization happens **after** an impl scheme is selected and a substitution is known.

### 5.4 Integration with Method Resolution

`resolve_interface_method_call` is extended:

1. Select an impl scheme (as in SPEC-034).
2. Normalize the method's return type using the selected scheme.
3. Return the fully normalized type.

Example:

```ash
interface Serializer<S> {
    type Ok
    serialize_bool(S, Bool) -> S::Ok
}

impl Serializer<JsonWriter> {
    type Ok = String
    serialize_bool(writer, value) = ...
}
```

Call: `Serializer::serialize_bool(my_writer, true)`.

1. Resolve method signature: `(S, Bool) -> S::Ok`.
2. Unify `S` with `JsonWriter`.
3. Select `impl Serializer<JsonWriter>`.
4. Normalize `S::Ok` → `String`.
5. Return `String`.

### 5.5 Unification with Associated Types

Before unification in concrete code, both sides of the equality must be **fully normalized** using the selected impl scheme:

```rust
let normalized_expected = self.normalize_associated_types(expected, scheme)?;
let normalized_actual = self.normalize_associated_types(actual, scheme)?;
unify(&normalized_expected, &normalized_actual)?
```

If an associated type appears in a context where no impl scheme has been selected yet (e.g., inside a generic function body with an abstract bound `T: Serializer`), it remains as `Type::Associated`.

**Rigid projection rule for generic code:**

- Inside `fn<T: Serializer>(s: T) -> T::Ok`, the projection `T::Ok` is treated as a **rigid type variable** scoped to the bound `Serializer`.
- Two identical rigid projections (`T::Ok` and `T::Ok`) unify with each other.
- A rigid projection does **not** unify with an arbitrary concrete type, even if that type happens to be the associated type for some specific impl.
- Rigid projections are resolved to concrete types only during monomorphization, when `T` is replaced by a concrete type and the concrete impl scheme is selected.

This rule is conservative, sound, and sufficient for serde-like and besedarium-like use cases.

## 6. Lowering and Interpreter

### 6.1 Monomorphization

When lowering a concrete interface call, the selected impl scheme provides:

- The instantiated method body (with type arguments substituted).
- The associated type bindings (also substituted).

The **ash-engine post-typecheck lowering pass** (owner defined in SPEC-034) replaces all occurrences of associated types in the method body with their normalized concrete types. Since SPEC-034 already requires monomorphization of generic impls, adding associated type substitution is a straightforward extension of the same pass.

### 6.2 Runtime Representation

`Type::Associated` does not appear at runtime. It is fully resolved during lowering. The interpreter sees only concrete `Type::Constructor`, `Type::Fun`, `Type::Fn`, and primitive types.

## 7. Migration Path

1. Add `associated_types` to `InterfaceDef` and `associated_type_bindings` to `ImplDef`.
2. Update parser (`parse_module.rs`) to read `type Name` in interfaces and `type Name = TypeExpr` in impls, and to parse `Type::Associated` in type contexts.
3. Add `Type::Associated` to the internal type representation.
4. Update interface and impl registration to store associated type metadata.
5. Implement `normalize_associated_types`.
6. Integrate normalization into `resolve_interface_method_call` and unification.
7. Extend the ash-engine monomorphization pass to substitute associated types in instantiated method bodies.
8. Test:
   - `Serializer<JsonWriter>::Ok` → `String`
   - `Iterator<ListIter<Int>>::Item` → `Int`
   - Generic function with rigid projection: `fn<T: Serializer>(s: T) -> T::Ok`

## 8. Conformance

An implementation conforming to SPEC-035 must:

- Parse `type Name` declarations inside `interface` blocks.
- Parse `type Name = TypeExpr` bindings inside `impl` blocks.
- Represent associated type projections (`S::Ok`) as `Type::Associated { interface, base, name }`.
- Normalize associated types to their concrete definitions after impl scheme selection.
- Ensure that every concrete interface call returns a fully normalized type with no remaining projections.
- Apply the rigid-projection rule inside generic function bodies where the impl scheme is not yet known.
- Support generic `impl` blocks where associated type bindings may reference the impl's own type parameters.
- Reject `impl` blocks that are missing required associated type bindings or provide extraneous ones.

## 9. Files Affected

| File | Change |
|------|--------|
| `crates/ash-core/src/ast.rs` | Add `associated_types` to `InterfaceDef`; add `associated_type_bindings` to `ImplDef` |
| `crates/ash-parser/src/parse_module.rs` | Restructure interface/impl body loops to dispatch between methods and associated-type declarations; parse `Type::Associated` in type contexts |
| `crates/ash-parser/src/parse_expr.rs` | Parse `Type::Associated` in expression type annotations if applicable |
| `crates/ash-typeck/src/types.rs` | Add `Type::Associated` variant; normalize before unification; apply rigid-projection rule for unresolved associated types |
| `crates/ash-typeck/src/type_env.rs` | Store associated types in `InterfaceInfo` and `ImplScheme`; implement normalization; add [NEW] `MissingAssociatedType`, `MismatchedProjectionInterface`, and `AmbiguousAssociatedType` errors |
| `crates/ash-engine/src/lib.rs` | Extend monomorphization pass to substitute associated types |
