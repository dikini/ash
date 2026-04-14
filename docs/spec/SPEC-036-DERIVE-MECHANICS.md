# SPEC-036: Derive Mechanics

**Status:** Draft  
**Date:** 2026-04-14  
**Version:** 0.2  

## 1. Overview

Introduce a compile-time derive mechanism that reduces boilerplate for mechanical interface implementations (`Eq`, `Clone`, `Serialize`). A type definition may declare a `derive` clause listing one or more derive handler names. The compiler expands each clause by looking up a registered built-in handler and splicing the generated `impl` blocks into the module.

For the MVP, derive handlers are **built into the engine in Rust**. User-defined `derive fn` and compile-time AST evaluation are explicitly deferred to a future phase.

## 2. Motivation

Without derives, every concrete record requires hand-written `impl` blocks for common interfaces:

```ash
type Point = { x: Int, y: Int }

impl Eq<Point> {
    eq(a, b) = a.x == b.x && a.y == b.y
}

impl Clone<Point> {
    clone(p) = Point { x: p.x, y: p.y }
}
```

A derive mechanism eliminates this boilerplate:

```ash
type Point = { x: Int, y: Int } derive Eq, Clone
```

The generated definitions are ordinary surface AST nodes evaluated by the existing type checker and interpreter.

## 3. Prerequisite: Type Definitions Enter the Module Pipeline

**This is a blocking prerequisite.**

Currently, `type` definitions live in `crates/ash-parser/src/parse_type_def.rs` as a parser-local `TypeDef` struct (`String` names, local `TypeExpr`, local `Visibility`) and are **not** part of the module-definition parser in `parse_module.rs`. The `Definition` enum in `surface.rs` has no `Type` variant, and `parse_definitions` does not dispatch on the `type` keyword.

Before derive expansion can run, `type` definitions must be first-class module citizens:

1. Add `derives: Vec<String>` to `ash_core::ast::TypeDef`.
2. Make `surface.rs` re-export `ash_core::ast::TypeDef` as `surface::TypeDef` and add `Definition::Type(TypeDef)` to `surface.rs`.
3. Rewrite `parse_type_def.rs` to parse into `ash_core::ast::TypeDef` (converting `String` names to `Name`, local `TypeExpr` to core `TypeExpr`, etc.) and return it.
4. Update `parse_definitions` in `parse_module.rs` to dispatch `starts_with_keyword(input, "type")` to `parse_type_def`.
5. Update lowering to handle `Definition::Type`. Lowering is a trivial pass-through because core `Definition` already includes `TypeDef` as metadata consumed by the type checker.

## 4. Semantics

### 4.1 Derive Clause Syntax

A `derive` clause may follow a `type` definition:

```ash
type Point = { x: Int, y: Int } derive Eq, Clone, Serialize
```

- The clause consists of the keyword `derive` followed by one or more comma-separated handler names.
- Handler names are simple identifiers (no paths or generics in the MVP).
- The parser parses the clause into `TypeDef.derives: Vec<String>`.

### 4.2 Built-In Derive Handlers

Handlers are Rust types implementing `DeriveHandler`, registered in `DeriveRegistry` at engine startup:

```rust
// crates/ash-engine/src/derive.rs
pub trait DeriveHandler: Send + Sync {
    fn name(&self) -> &'static str;
    fn expand(&self, target: &surface::TypeDef) -> Result<Vec<surface::Definition>, DeriveError>;
}

pub struct DeriveRegistry {
    handlers: HashMap<&'static str, Box<dyn DeriveHandler>>,
}

impl DeriveRegistry {
    pub fn with_builtins() -> Self {
        let mut r = Self { handlers: HashMap::new() };
        r.register(Box::new(EqDerive));
        r.register(Box::new(CloneDerive));
        r.register(Box::new(SerializeDerive));
        r
    }
}
```

Built-in handlers construct surface AST nodes using ordinary Rust struct constructors (not via an interpreter or `ast::` builder API). Example:

```rust
pub struct EqDerive;

impl DeriveHandler for EqDerive {
    fn name(&self) -> &'static str { "Eq" }

    fn expand(&self, target: &surface::TypeDef) -> Result<Vec<surface::Definition>, DeriveError> {
        if !target.params.is_empty() {
            return Err(DeriveError::GenericNotSupported(self.name()));
        }
        let struct_body = match &target.body {
            surface::TypeBody::Struct(fields) => fields,
            _ => return Err(DeriveError::NonStructType(self.name())),
        };
        // ... build eq method body using surface::Expr::... constructors ...
        let impl_def = surface::Definition::Impl(surface::ImplDef { ... });
        Ok(vec![impl_def])
    }
}
```

### 4.3 Derive Expansion Pass

Expansion runs after full module parsing and before lowering:

```
Parse Module (including type definitions)
  -> Expand Derives
  -> Lower
  -> Typecheck
```

The expansion pass iterates over the module's `definitions` vector. For each `Definition::Type` with a non-empty `derives` field:

1. Look up each handler name in `DeriveRegistry`.
2. If missing, emit `DeriveError::UnknownDerive`.
3. If found, call `handler.expand(type_def)`.
4. If expansion fails, emit `DeriveError::ExpansionFailed`.
5. Insert the returned `Vec<Definition>` into the module's definition list **immediately after** the type definition.

Generated definitions are ordinary surface AST nodes. The type checker and interpreter see them exactly as if the user had written them by hand.

### 4.4 MVP Built-In Handlers

| Handler | Generated For | Requirements |
|---------|--------------|--------------|
| `Eq` | `impl Eq<T>` | Struct type only; all field types must implement `Eq` (checked by type checker) |
| `Clone` | `impl Clone<T>` | Struct type only; all field types must implement `Clone` |
| `Serialize` | `impl Serialize<T>` | Struct type only; requires SPEC-032–035; all field types must implement `Serialize` |

All three handlers reject generic `type` definitions with `DeriveError::GenericNotSupported`.

### 4.5 Deferred Features

The following are explicitly **not** in the MVP:

- User-defined `derive fn` functions.
- `ast::` builder API exposed to user code.
- Compile-time evaluation of Ash code during derive expansion.
- Derive clauses on `enum` or `alias` type bodies.

## 5. IR Changes

### 5.1 Surface AST

**`crates/ash-parser/src/surface.rs`**

Add `Type` to `Definition`:

```rust
pub enum Definition {
    // ... existing variants
    /// Type definition
    Type(TypeDef),
}
```

`ash_core::ast::TypeDef` gains a `derives` field and is re-exported as `surface::TypeDef`:

```rust
// crates/ash-core/src/ast.rs
pub struct TypeDef {
    pub name: Name,
    pub params: Vec<TypeVar>,
    pub body: TypeBody,
    pub visibility: Visibility,
    pub derives: Vec<String>,   -- NEW
}
```

### 5.2 Type Definition Parser

**`crates/ash-parser/src/parse_type_def.rs`**

- Parse optional `derive Name, Name` after the type body and before the terminating `;`.
- Store the names in `TypeDef.derives`.

### 5.3 Module Definition Parser

**`crates/ash-parser/src/parse_module.rs`**

Add a `type` branch in `parse_definitions`:

```rust
if starts_with_keyword(input, "type") {
    definitions.push(Definition::Type(parse_type_def(input)?));
    continue;
}
```

### 5.4 Lowering

**`crates/ash-parser/src/lower.rs`**

Add a `Definition::Type` lowering branch that maps directly to `core::Definition::Type(TypeDef)`. Type definitions are **metadata** consumed by the type checker; they do not produce runtime instructions. The lowered module preserves `Definition::Type` so the type checker can resolve type names and derive-expanded impls can reference them.

### 5.5 Engine Derive Expansion

**`crates/ash-engine/src/derive.rs`** (new file)

Owns:
- `DeriveHandler` trait
- `DeriveRegistry`
- `DeriveError` enum
- `expand_derives(module: &mut surface::Module, registry: &DeriveRegistry) -> Result<(), DeriveError>`

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum DeriveError {
    UnknownDerive(String),
    ExpansionFailed { handler: String, message: String },
    GenericNotSupported(&'static str),
    NonStructType(&'static str),
}
```

## 6. Migration Path

1. Add `derives: Vec<String>` to `ash_core::ast::TypeDef` and re-export it in `surface.rs`.
2. Rewrite `parse_type_def.rs` to parse into `ash_core::ast::TypeDef`.
3. Add `Definition::Type(TypeDef)` to `surface.rs` and integrate the rewritten `parse_type_def` into `parse_definitions`.
4. Update lowering to pass through `Definition::Type`.
5. Create `crates/ash-engine/src/derive.rs` with registry, built-in handlers, and `DeriveError`.
6. Implement `EqDerive`, `CloneDerive`, and `SerializeDerive` as pure Rust AST constructors.
7. Insert `expand_derives` into the engine pipeline between parsing and lowering.
8. Add tests for `derive Eq`, `derive Clone`, and `derive Serialize` on concrete struct types.

## 7. Conformance

An implementation conforming to SPEC-036 must:

- Parse `type Name = Body derive Handler, Handler`.
- Reject unknown derive handler names with a clear `DeriveError`.
- Reject derive clauses on generic or non-struct type definitions in the MVP.
- Expand built-in `Eq`, `Clone`, and `Serialize` handlers for concrete struct types.
- Inject generated definitions as ordinary surface AST nodes immediately after the source type definition.
- Ensure generated code passes the same type checking and runtime semantics as hand-written impls.

## 8. Files Affected

| File | Change |
|------|--------|
| `crates/ash-parser/src/surface.rs` | Add `Definition::Type(TypeDef)`; add `derives: Vec<String>` to `TypeDef` |
| `crates/ash-parser/src/parse_type_def.rs` | Parse optional `derive` clause after type body |
| `crates/ash-parser/src/parse_module.rs` | Add `type` dispatch branch in `parse_definitions` |
| `crates/ash-parser/src/lower.rs` | Add `Definition::Type` lowering branch |
| `crates/ash-engine/src/derive.rs` | [NEW] Registry, built-in handlers, expansion pass, `DeriveError` |
| `crates/ash-engine/src/lib.rs` | Call `expand_derives` after parsing and before lowering |
