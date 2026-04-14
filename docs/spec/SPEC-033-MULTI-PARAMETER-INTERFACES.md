# SPEC-033: Multi-Parameter Interfaces

**Status:** Draft  
**Date:** 2026-04-14  
**Version:** 0.2  

## 1. Overview

Remove the single type-parameter restriction on interface declarations and their concrete `impl` blocks. Interfaces may declare any number of type parameters, and concrete `impl` blocks must provide a matching number of type arguments.

This spec builds on SPEC-032 (multi-parameter methods). Together they enable interfaces such as `Map<K, V>`, `Result<T, E>`, and `Pair<A, B>`.

## 2. Motivation

The closed-world interface MVP restricts every interface to exactly one type parameter:

```rust
if interface_info.type_params.len() != 1 || def.type_args.len() != 1 {
    return Err(TypeEnvError::InvalidDefinition(
        "closed-world interface MVP only supports single-parameter ..."
    ));
}
```

This makes it impossible to express:

- Binary containers: `Map<K, V>`, `Pair<A, B>`
- Result types with independent error payloads: `Result<T, E>`
- Conversions between types: `From<A, B>`, `Into<A, B>`

While besedarium and serde-like libraries ultimately require generic `impl` blocks (SPEC-034) and associated types (SPEC-035), multi-parameter interfaces are a prerequisite structural feature. Without them, even fully concrete instantiations of generic interfaces are rejected.

## 3. Semantics

### 3.1 Interface Declaration

```ash
interface Map<K, V> {
    get(Map<K, V>, K) -> Option<V>
    insert(Map<K, V>, K, V) -> Map<K, V>
}

interface From<A, B> {
    from(A) -> B
}
```

Type parameters are introduced positionally in angle brackets. They are in scope for all method signatures inside the interface block.

### 3.2 Concrete Implementation

The parser's live contract uses `impl Interface<TypeArgs> { ... }` syntax (no `for` token):

```ash
impl Map<String, Int> {
    get(m, k) = string_map::get(m, k)
    insert(m, k, v) = string_map::insert(m, k, v)
}

impl From<String, List<String>> {
    from(s) = string::split(s, ",")
}
```

The number of type arguments in the `impl` head must match the number of type parameters declared by the interface. All type arguments in this spec must be concrete types — generic `impl` blocks are deferred to SPEC-034.

**Important semantic redesign**: In the current MVP, `impl Explain<PolicyDecision>` registers the impl against the bare concrete type `PolicyDecision`. For multi-parameter interfaces, this model breaks because there is no single "target type" — the interface itself is parameterized over multiple types.

This spec changes the impl registry model: impls are keyed by the **full interface application** `Interface<T1, T2, ...>` rather than by a single bare type. This means `impl Map<String, Int>` registers against the key `Map<String, Int>`, not against `String` or `Int` individually.

### 3.3 Method Call Resolution

Calls remain syntactically unchanged:

```ash
Map::get(my_map, "key")
From::from("a,b,c")
```

The type checker resolves the interface, the method, and then unifies the concrete type arguments against the method signature.

## 4. IR Changes

No AST structural changes are required in the surface or core layers. The relevant nodes already carry vectors:

- `InterfaceDef { type_params: Vec<Name>, ... }` (`surface.rs:227`)
- `ImplDef { type_args: Vec<Type>, ... }` (`surface.rs:261`)
- `InterfaceMethodSig { params: Vec<Type>, ... }` (`surface.rs:246`)

The implementation simply stops forcing `len() == 1` in the type checker.

## 5. Type System Changes

### 5.1 Registering Multi-Parameter Interfaces

In `ash-typeck/src/type_env.rs`, `register_interface` already accepts `Vec<TypeVar>` mapped from `Vec<Name>`. The only change is ensuring that method signatures can reference any of the interface's type parameters, not just the first one. The current param-mapping code does this correctly:

```rust
let param_mapping: HashMap<String, TypeVar> = def
    .type_params
    .iter()
    .map(|param| (param.to_string(), TypeVar::fresh()))
    .collect();
```

No change needed here.

### 5.2 Registering Concrete Multi-Parameter Impls

In `register_impl`, remove the hardcoded `== 1` checks (`type_env.rs:574-577`) and replace with an arity check:

```rust
if interface.type_params.len() != def.type_args.len() {
    return Err(TypeEnvError::InvalidDefinition(format!(
        "interface '{}' expects {} type parameters, but impl provides {}",
        interface_name,
        interface.type_params.len(),
        def.type_args.len()
    )));
}
```

The impl is then registered using the **full interface application** as the lookup key:

```rust
let impl_head = Type::Constructor {
    name: QualifiedName { path: vec![], name: interface_name },
    args: def.type_args.iter().map(lower_surface_type).collect(),
    kind: Kind::Type,
};

self.impls.insert((interface_name, impl_head), impl_info);
```

This replaces the old model where the key was `(interface_name, bare_concrete_type)`.

**Error model update:** `TypeEnvError::DuplicateImpl` and `TypeEnvError::MissingImpl` currently say "Concrete nominal type" in their messages. They must be updated to report the full interface application (e.g., `Map<String, Int>`) rather than a single bare type.

### 5.3 Method Resolution

`resolve_interface_method_call` currently computes the impl head type by unifying method parameters and applying the substitution to the interface's single type parameter. For multi-parameter interfaces, this is generalized:

1. Unify all method parameter types with the argument types (as in SPEC-032).
2. Apply the resulting substitution to **all** interface type parameters. These type variables live in `InterfaceMethodInfo.type_params` (`Vec<TypeVar>`), not in `InterfaceInfo.type_params` (`Vec<String>`):
   ```rust
   let method_info = interface_info.methods.get(method)?;
   let impl_head_args: Vec<Type> = method_info.type_params.iter()
       .map(|tp| subst.apply(&Type::Var(*tp)))
       .collect();
   ```
3. Construct the impl head as `Type::Constructor { name: interface_name, args: impl_head_args, kind: Kind::Type }`.
4. Look up the concrete impl using this constructed head.

**Underdetermined parameters**: This procedure only works when the method signatures provide enough information to fully determine all interface type parameters. For an interface like `From<A, B> { from(A) -> B }`, calling `From::from("hello")` does **not** determine `B` from the argument alone. In such cases, the caller must provide an explicit type annotation (e.g., `let x: List<String> = From::from("hello")`), or the type checker must propagate the expected type from the surrounding context. The spec for explicit type annotations on calls is deferred; for now, underdetermined parameters are a type error unless context resolves them.

### 5.4 Example Walkthrough

```ash
interface Map<K, V> {
    get(Map<K, V>, K) -> Option<V>
}

impl Map<String, Int> {
    get(m, k) = ...
}
```

Call: `Map::get(my_map, "key")` where `my_map: Map<String, Int>`.

1. Resolve `Map::get` → method signature `(Map<K, V>, K) -> Option<V>`.
2. Unify `Map<K, V>` with `Map<String, Int>` → `K = String, V = Int`.
3. Unify `K` with `String` (consistent).
4. Construct impl head: `Map<String, Int>`.
5. Lookup `impls[("Map", Map<String, Int>)]` → found.
6. Return type: `Option<V>` → `Option<Int>`.

## 6. Migration Path

1. Remove single-type-param restrictions from `register_impl`.
2. Update `register_impl` to use `Type::Constructor { interface_name, args }` as the impl lookup key.
3. Update `resolve_interface_method_call` to construct multi-argument `Type::Constructor` impl heads from all interface type params.
4. Add tests for:
   - `interface Pair<A, B> { first(Pair<A, B>) -> A }`
   - `impl Pair<Int, String> { ... }`
   - `Pair::first(my_pair)`

## 7. Conformance

An implementation conforming to SPEC-033 must:

- Accept interface declarations with N type parameters where N >= 1.
- Accept concrete `impl` blocks providing exactly N type arguments using `impl Interface<T1, T2>` syntax.
- Reject `impl` blocks with the wrong number of type arguments.
- Register impls against the full interface application `Interface<T1, T2>`, not a single bare type.
- Resolve method calls by unifying all method parameters and constructing the full impl head from all type parameters.
- Report a type error when interface type parameters are underdetermined by arguments and context.
- Continue to reject generic `impl` blocks with type variables in the impl head (deferred to SPEC-034).

## 8. Files Affected

| File | Change |
|------|--------|
| `crates/ash-typeck/src/type_env.rs` | Remove `== 1` checks; change impl key to full interface application |
