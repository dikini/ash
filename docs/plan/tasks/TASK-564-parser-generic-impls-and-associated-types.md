# TASK-564: Parser/AST — Generic Impl Syntax, `where` Bounds, and Associated Types

**Phase:** 83
**Spec:** SPEC-034 §4, SPEC-035 §4
**Related:** TASK-563 (Complete; next sequential step in Phase 83)
**Estimate:** 5 hours
**Status:** 📝 Planned (blocked: requires TASK-563)

## Description

Extend the parser to read generic impl blocks (`impl<T> Interface<Head> where T: Interface`), associated type declarations inside interfaces (`type Name`), associated type bindings inside impls (`type Name = TypeExpr`), and projection syntax in type contexts (`S::Ok`).

## Requirements

### Functional Requirements

1. `impl<T> Serialize<List<T>> where T: Serialize { ... }` parses correctly.
2. `interface Serializer<S> { type Ok; serialize_bool(S, Bool) -> S::Ok }` parses correctly.
3. `impl Serializer<JsonWriter> { type Ok = String; ... }` parses correctly.
4. Type contexts accept `Param::AssocName` projections (e.g., `S::Ok`, `Map<K,V>::Entry`).
5. Core AST carries `type_params`, `where_bounds`, and `associated_type_bindings`.

## TDD Steps

### Step 1: Write Tests (Red)

**File:** `crates/ash-parser/tests/closed_world_interfaces_task_422.rs`

**Tests:**

```rust
#[test]
fn task564_generic_impl_with_where_parses() {
    let input = r#"impl<T> Serialize<List<T>> where T: Serialize { serialize(x, s) = x }"#;
    let mut parse_input = new_input(input);
    let def = parse_module::parse_impl_definition(&mut parse_input).unwrap();
    match def {
        Definition::Impl(impl_def) => {
            assert_eq!(impl_def.type_params.len(), 1);
            assert_eq!(impl_def.where_bounds.len(), 1);
        }
        _ => panic!("expected impl"),
    }
}

#[test]
fn task564_associated_type_decl_in_interface_parses() {
    let input = r#"interface Serializer<S> { type Ok; serialize_bool(S, Bool) -> S::Ok }"#;
    let mut parse_input = new_input(input);
    let def = parse_module::parse_interface_definition(&mut parse_input).unwrap();
    match def {
        Definition::Interface(iface) => {
            assert_eq!(iface.associated_types.len(), 1);
            assert_eq!(iface.associated_types[0].as_ref(), "Ok");
            assert_eq!(iface.methods[0].return_type, /* S::Ok */);
        }
        _ => panic!("expected interface"),
    }
}

#[test]
fn task564_associated_type_binding_in_impl_parses() {
    let input = r#"impl Serializer<JsonWriter> { type Ok = String; serialize_bool(w, v) = v }"#;
    let mut parse_input = new_input(input);
    let def = parse_module::parse_impl_definition(&mut parse_input).unwrap();
    match def {
        Definition::Impl(impl_def) => {
            assert_eq!(impl_def.associated_type_bindings.len(), 1);
            let (name, ty) = &impl_def.associated_type_bindings[0];
            assert_eq!(name.as_ref(), "Ok");
            // ty should be String
        }
        _ => panic!("expected impl"),
    }
}

#[test]
fn task564_associated_type_projection_in_type_context_parses() {
    // Parse a function return type that uses S::Ok
    let input = r#"fn demo<S>(x: S) -> S::Ok { x }"#;
    // Test via type parser or fn parser
}
```

### Step 2: Surface AST Changes

**File:** `crates/ash-parser/src/surface.rs`

Add to `ImplDef`:
```rust
pub struct ImplDef {
    pub visibility: Visibility,
    pub interface: Name,
    pub type_params: Vec<TypeParam>,   // NEW
    pub type_args: Vec<Type>,
    pub where_bounds: Vec<InterfaceBound>, // NEW
    pub associated_type_bindings: Vec<(Name, Type)>, // NEW
    pub methods: Vec<ImplMethodDef>,
    pub span: Span,
}
```

Add to `InterfaceDef`:
```rust
pub struct InterfaceDef {
    pub name: Name,
    pub type_params: Vec<Name>,
    pub associated_types: Vec<Name>,   // NEW
    pub methods: Vec<InterfaceMethodSig>,
    pub visibility: Visibility,
    pub span: Span,
}
```

Add `Associated` to surface `Type`:
```rust
pub enum Type {
    // ... existing variants
    Associated {
        base: Box<Type>,
        name: Name,
    },
}
```

### Step 3: Core AST Changes

**File:** `crates/ash-core/src/ast.rs`

Mirror surface changes:
- `InterfaceDef` gains `associated_types: Vec<Name>`.
- `ImplDef` gains `type_params: Vec<TypeVar>`, `where_bounds: Vec<WhereBound>`, `associated_type_bindings: Vec<(Name, TypeExpr)>`.
- Add `WhereBound` struct.
- Add `Associated` to `TypeExpr` if it doesn't exist there (or reuse existing path syntax).

### Step 4: Parser Implementation

**File:** `crates/ash-parser/src/parse_module.rs`

- Restructure `parse_interface_definition` body loop to dispatch between `type Name;` and method signatures.
- Restructure `parse_impl_definition` body loop to dispatch between `type Name = TypeExpr;` and method definitions.
- Add `parse_where_bounds` for comma-separated `T: Interface`.
- Add `parse_type_params` reusing existing bounded-generic machinery.
- Extend `parse_surface_type` to handle `identifier "::" identifier` as `Type::Associated`.

## Verification Steps

- [ ] `cargo test -p ash-parser task564` passes
- [ ] `cargo check --all` passes
- [ ] `cargo clippy -p ash-parser --all-targets --all-features` clean
