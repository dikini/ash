# SPEC-034: Where-Bounded Generic Interface Implementations

**Status:** Draft  
**Date:** 2026-04-14  
**Version:** 0.2  

## 1. Overview

Allow `impl` blocks to be generic: their type arguments may contain type variables, and those variables may carry interface bounds in a `where` clause. This enables recursive, polymorphic interface implementations such as `Serialize` for `List<T>` whenever `T: Serialize`.

This is the critical extension that makes interfaces useful for libraries (serde-style serialization, besedarium-style query building, generic collections). Without it, every user must hand-write a concrete impl for every concrete type instantiation.

## 2. Motivation

Consider a `Serialize` interface:

```ash
interface Serialize<T> {
    serialize(T, Serializer) -> Result<String, SerializeError>
}
```

Without generic impls, users must write:

```ash
impl Serialize<Int> { ... }
impl Serialize<String> { ... }
impl Serialize<List<Int>> { ... }
impl Serialize<List<String>> { ... }
impl Serialize<List<List<Int>>> { ... }
-- ad infinitum
```

Generic impls solve this:

```ash
impl<T> Serialize<List<T>> where T: Serialize {
    serialize(items, ser) = {
        let parts = list::map(items, fn(item) {
            Serialize::serialize(item, ser)
        });
        string::concat("[", string::join(",", parts), "]")
    }
}
```

The same pattern appears in besedarium: `Queryable<T>` for `Record<A, B>` requires `A: Queryable` and `B: Queryable` to build nested query fragments.

## 3. Semantics

### 3.1 Syntax

Generic impl syntax uses the live parser contract (`impl<TypeParams> Interface<Head> where Bounds`):

```ash
impl<T> Serialize<List<T>> where T: Serialize {
    serialize(items, ser) = ...
}

impl<K, V> MapOps<Map<K, V>>
    where K: Eq,
          V: Clone
{
    get(m, k) = ...
    insert(m, k, v) = ...
}
```

- `impl` introduces zero or more type parameters in angle brackets.
- The head provides the interface name and the type arguments (which may contain those type parameters).
- `where` introduces zero or more bounds: each bound is `TypeVar: InterfaceName`. Multiple bounds are written as separate clauses; `+` composition is deferred.

### 3.2 Closed-World Instance Search

Ash uses a **closed-world** interface model. All `impl` blocks for a given interface must be visible in the same compilation unit (or explicitly imported). There are no orphan instances.

Resolution proceeds by **ordered search** over all registered generic impl schemes:

1. Attempt to unify the requested concrete head with each impl scheme's head, in declaration order.
2. If unification succeeds, apply the resulting substitution to the `where` bounds and recursively check each bound.
3. If all bounds are satisfied, select that impl.
4. If no scheme matches, it is a type error.

**Coherence rule: no overlapping impls.** During registration, if two schemes for the same interface have heads that unify, registration fails. This makes the resolution algorithm deterministic without requiring run-time disambiguation.

### 3.3 Recursive Bound Checking

A `where` bound may itself require a generic impl:

```ash
impl<T> Serialize<List<T>> where T: Serialize
```

When resolving `Serialize::serialize(my_list, ser)` where `my_list: List<List<Int>>`:

1. Match `List<List<Int>>` against `List<T>` → `T = List<Int>`.
2. Check bound `List<Int>: Serialize`.
3. Recursively match `List<Int>` against `List<T>` → `T = Int`.
4. Check bound `Int: Serialize`.
5. Find concrete `impl Serialize<Int>`.
6. Success.

The type checker must detect and report infinite recursion in pathological cases (e.g., a cycle of generic impls). A simple recursion-depth limit (e.g., 32) is sufficient for the initial implementation.

## 4. IR Changes

### 4.1 Surface AST Reuse

**`crates/ash-parser/src/surface.rs`**

The parser already has bounded-generic machinery for workflows/functions:

```rust
pub struct TypeParam {
    pub name: Name,
    pub bounds: Vec<InterfaceBound>,
    pub span: Span,
}

pub struct InterfaceBound {
    pub interface: Name,
    pub span: Span,
}
```

These are **reused directly** for generic impl parsing. The parsed surface `ImplDef` gains an optional `type_params: Vec<TypeParam>` field.

### 4.2 Core AST Updates

**`crates/ash-core/src/ast.rs`**

After lowering from surface, `ImplDef` carries:

```rust
pub struct ImplDef {
    pub visibility: Visibility,
    pub interface: Name,
    pub type_params: Vec<TypeVar>,      -- lowered from surface TypeParam names
    pub type_args: Vec<TypeExpr>,
    pub where_bounds: Vec<WhereBound>,  -- lowered from surface InterfaceBounds
    pub methods: Vec<ImplMethodDef>,
}

pub struct WhereBound {
    pub type_var: TypeVar,
    pub interface: Name,
    pub span: Span,
}
```

### 4.3 Parser Updates

**`crates/ash-parser/src/parse_module.rs`**

- `parse_impl_definition` gains optional type params: `impl<T> Interface<Head> { ... }`
  - Type params are parsed using the existing `TypeParam` / `InterfaceBound` surface structures.
- `parse_impl_definition` gains optional `where` clause parsing.
- `parse_where_bounds` reads comma-separated `T: Interface` entries.

Example grammar:

```
impl-def = "impl" [ "<" type-param ("," type-param)* ">" ]
           identifier "<" type-arg ("," type-arg)* ">"
           [ "where" where-bound ("," where-bound)* ]
           "{" impl-method* "}"

where-bound = identifier ":" identifier
```

## 5. Type System Changes

### 5.1 From Concrete Impl Map to Impl Schemes

`TypeEnv` currently stores:

```rust
impls: HashMap<(String, Type), ImplInfo>
```

This is replaced with a **scheme list** that holds both concrete and generic impls:

```rust
pub struct ImplScheme {
    pub interface: String,
    pub type_params: Vec<TypeVar>,
    pub head: Type,                       -- e.g., List<T>
    pub where_bounds: Vec<WhereBound>,    -- e.g., [T: Serialize]
    pub methods: Vec<ImplMethodInfo>,
}

impls: Vec<ImplScheme>,
```

For uniformity, every `impl` block lowers to an `ImplScheme` whose head is the **full interface application** (consistent with SPEC-033):

- `impl Serialize<Int>` → `type_params: [], head: Serialize<Int>`
- `impl<T> Serialize<List<T>> where T: Serialize` → `type_params: [T], head: Serialize<List<T>>`

### 5.2 Registering Impl Schemes

`register_impl` (or a new `register_impl_scheme`) validates:

1. The interface exists.
2. The number of type arguments in the head matches the interface's type parameter count.
3. The `where` bounds reference only type parameters introduced by the `impl`.
4. The referenced interfaces in `where` bounds exist.
5. **No overlap**: if any existing scheme for the same interface has a head that unifies with the new scheme's head, registration fails with a [NEW] `OverlappingImpls` error.

### 5.3 Resolution Algorithm

```rust
pub fn resolve_interface_method_call(
    &self,
    interface: &str,
    method: &str,
    arg_types: &[Type],
) -> Result<Type, TypeEnvError> {
    let interface_info = self.interfaces.get(interface)
        .ok_or_else(|| TypeEnvError::MissingInterface(interface.to_string()))?;

    let method_info = interface_info.methods.get(method)
        .ok_or_else(|| TypeEnvError::MissingInterfaceMethod { ... })?;

    // 1. Unify method parameters with argument types
    let mut subst = Substitution::empty();
    if method_info.params.len() != arg_types.len() {
        return Err(TypeEnvError::[NEW] WrongArity(
            format!("expected {} arguments, found {}", method_info.params.len(), arg_types.len())
        ));
    }
    for (expected, actual) in method_info.params.iter().zip(arg_types.iter()) {
        subst = unify(expected, actual).map_err(|e| TypeEnvError::InvalidDefinition(format!("{e}")))?;
    }

    // 2. Compute the concrete impl head from all interface type params
    let head_args: Vec<Type> = method_info.type_params.iter()
        .map(|tp| subst.apply(&Type::Var(*tp)))
        .collect();

    let target_head = Type::Constructor {
        name: QualifiedName { path: vec![], name: interface.to_string() },
        args: head_args,
        kind: Kind::Type,
    };

    // 3. Search impl schemes
    let selected = self.find_matching_impl_scheme(interface, &target_head, 0)?;

    // 4. Apply the scheme's substitution to the method return type
    Ok(selected.substitution.apply(&method_info.return_type))
}
```

`find_matching_impl_scheme`:

```rust
fn find_matching_impl_scheme(
    &self,
    interface: &str,
    target_head: &Type,
    depth: usize,
) -> Result<SelectedScheme, TypeEnvError> {
    if depth > 32 {
        return Err(TypeEnvError::[NEW] RecursiveBound(
            "interface resolution exceeded recursion limit".to_string()
        ));
    }

    for scheme in self.impls.iter().filter(|s| s.interface == interface) {
        // scheme.head is itself a full interface application (e.g., Serialize<List<T>>)
        if let Ok(scheme_subst) = try_unify(&scheme.head, target_head) {
            let mut bounds_ok = true;
            for bound in &scheme.where_bounds {
                let bounded_ty = scheme_subst.apply(&Type::Var(bound.type_var));
                let bound_head = Type::Constructor {
                    name: QualifiedName { path: vec![], name: bound.interface.clone() },
                    args: vec![bounded_ty],
                    kind: Kind::Type,
                };
                if self.find_matching_impl_scheme(&bound.interface, &bound_head, depth + 1).is_err() {
                    bounds_ok = false;
                    break;
                }
            }
            if bounds_ok {
                return Ok(SelectedScheme { substitution: scheme_subst });
            }
        }
    }

    Err(TypeEnvError::MissingImpl { interface: interface.to_string(), ty: target_head.to_string() })
}
```

## 6. Lowering / Monomorphization Owner

Generic impls require **monomorphization**: at each call site, the selected impl scheme and its substitution are used to produce a concrete method body.

**Owner**: `crates/ash-engine/src/lib.rs` (or a dedicated `ash-engine/src/monomorphize.rs` module) in a **post-typecheck lowering pass**.

Responsibility:
1. After type-checking resolves an interface call, the engine receives the selected `ImplScheme` and the concrete substitution.
2. The engine applies the substitution to the scheme's method body AST.
3. The instantiated body is emitted as a fresh internal function (or inlined directly into the caller).
4. The resulting core IR contains only concrete, monomorphized code — the interpreter never sees generic impl dispatch at runtime.

This is preferred over parser-time lowering because the parser does not perform interface resolution, and preferred over interpreter-time dispatch because Ash's type system is static and closed-world.

## 7. Migration Path

1. Define `ImplScheme` and `WhereBound` in the type environment.
2. Update surface `ImplDef` to carry `type_params: Vec<TypeParam>` reusing existing surface structures.
3. Update core `ImplDef` lowering to produce `type_params: Vec<TypeVar>` and `where_bounds: Vec<WhereBound>`.
4. Update parser to read `impl<T> Interface<Head> where T: Interface`.
5. Replace `impls: HashMap<...>` with `impls: Vec<ImplScheme>`, storing full interface applications as heads.
6. Update `register_impl` to build schemes and check overlap.
7. Rewrite `resolve_interface_method_call` to search schemes recursively.
8. Add ash-engine post-typecheck monomorphization pass.
9. Test recursive resolution (`List<List<Int>>: Serialize`).

## 8. Conformance

An implementation conforming to SPEC-034 must:

- Parse generic `impl<T, ...> Interface<HeadType> where T: Interface, ...` blocks.
- Store impls as generic schemes in the type environment.
- Reject overlapping impl schemes for the same interface at registration time.
- Resolve interface method calls by unifying the concrete head against registered schemes.
- Recursively check `where` bounds during resolution.
- Enforce a recursion limit on bound checking to prevent infinite loops.
- Monomorphize generic impl bodies in a post-typecheck lowering pass.
- Support multi-parameter interfaces and methods (requires SPEC-032 and SPEC-033).

## 9. Files Affected

| File | Change |
|------|--------|
| `crates/ash-parser/src/surface.rs` | Add `type_params: Vec<TypeParam>` to surface `ImplDef`; reuse `InterfaceBound` for where bounds |
| `crates/ash-core/src/ast.rs` | Add lowered `type_params` and `where_bounds` to core `ImplDef`; add `WhereBound` struct |
| `crates/ash-parser/src/parse_module.rs` | Parse `impl<T>` and `where T: Interface` |
| `crates/ash-typeck/src/type_env.rs` | Replace impl map with scheme list; rewrite resolution algorithm; add [NEW] `OverlappingImpls` and `RecursiveBound` errors |
| `crates/ash-typeck/src/check_expr.rs` | Update interface call type inference for scheme search |
| `crates/ash-engine/src/lib.rs` | Add post-typecheck monomorphization pass for generic impls |
