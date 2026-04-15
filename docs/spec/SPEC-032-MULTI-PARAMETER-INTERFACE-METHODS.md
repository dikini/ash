# SPEC-032: Multi-Parameter Interface Methods

**Status:** Draft  
**Date:** 2026-04-14  
**Version:** 0.2  

## 1. Overview

Remove the single-parameter restriction on interface method signatures and their call sites. Interface methods may declare any number of parameters, and call sites may pass any number of arguments.

This is the first incremental step toward a fully generic interface system. It does not introduce generic `impl` blocks or `where` bounds — concrete impls only — but it enables interfaces to express binary relations (e.g., `Eq`, `Ord`) and operations that require context parameters.

## 2. Motivation

The current closed-world interface MVP hardcodes methods to exactly one parameter:

```rust
if method_info.params.len() != 1 {
    return Err(TypeEnvError::InvalidDefinition(format!(
        "canonical interface method '...' must take exactly one argument"
    )));
}
```

This prevents expressing even the simplest multi-argument contracts:

- `eq(a, b)` — equality comparison
- `compare(a, b)` — ordering
- `serialize(value, serializer)` — serialization with a format driver
- `insert(map, key, value)` — collection operations

Without multi-parameter methods, interfaces are limited to unary type-class patterns (`to_string`, `clone`) and cannot model operations that naturally require two or more operands.

## 3. Semantics

### 3.1 Method Declaration Syntax

Multi-parameter interface methods use parenthesized parameter lists:

```ash
interface Eq<T> {
    eq(T, T) -> Bool
}

interface Ord<T> {
    compare(T, T) -> Ordering
}

interface Serialize<T> {
    serialize(T, Serializer) -> Result<String, SerializeError>
}
```

**Grammar:**
```
interface-method-sig = identifier "(" [ type ("," type)* ] ")" "->" type
```

This replaces the MVP grammar `identifier ":" type "->" type`.

An interface method signature consists of:
- A method name
- A parenthesized, comma-separated list of parameter types (zero or more)
- An arrow `->` and a return type

### 3.2 Concrete Implementation Syntax

```ash
impl Eq<Int> {
    eq(a, b) = a == b
}

impl Serialize<String> {
    serialize(s, ser) = serializer::serialize_string(ser, s)
}
```

`impl` blocks remain concrete: the type argument in the impl head must be a concrete type. Generic impls are deferred to SPEC-034.

### 3.3 Call Syntax

Interface method calls use the existing qualified call syntax:

```ash
Eq::eq(x, y)
Ord::compare(left, right)
Serialize::serialize(value, json_writer)
```

The parser already accepts `Name::Name(args...)`. The type checker intercepts these calls when `Name` resolves to a registered interface.

## 4. IR Changes

### 4.1 Surface AST Updates

**`crates/ash-parser/src/surface.rs`**

No structural change to `InterfaceMethodSig` — its `params` field is already `Vec<Type>` (`surface.rs:246`). The implementation simply stops forcing `len() == 1`.

Change `ImplMethodDef` to support multiple parameters:

```rust
pub struct ImplMethodDef {
    pub name: Name,
    pub params: Vec<Name>,   -- was `param: Name` (surface.rs:274)
    pub body: Expr,
    pub span: Span,
}
```

**Deprecation of `Expr::InterfaceMethodCall`**

The legacy `InterfaceMethodCall { interface, method, argument }` AST node (surface.rs:808-818) is removed. Multi-argument interface calls are represented uniformly as:

```rust
Expr::Call {
    func: Name,
    module: Option<Name>,   -- Some("InterfaceName") for interface methods
    args: Vec<Expr>,
    span: Span,
}
```

This eliminates the AST-level special case. The type checker distinguishes interface calls from module-qualified function calls by looking up whether `module` names a registered interface.

### 4.2 Core AST Updates

**`crates/ash-core/src/ast.rs`**

`InterfaceMethodCall` and `ImplMethodDef` mirror the surface AST. The core `ImplMethodDef` is updated from `param: Name` to `params: Vec<Name>`, and `InterfaceMethodCall` is removed.

### 4.3 Parser Updates

**`crates/ash-parser/src/parse_module.rs`**

- `parse_interface_method_signature` parses `(Type1, Type2, ...) -> ReturnType`.
- `parse_impl_method_definition` parses `name(param1, param2, ...) = expr`.

**`crates/ash-parser/src/parse_expr.rs`**

- `Name::Name(args...)` already produces `Expr::Call { module: Some(name), func: second_name, args }` when parentheses are present (`parse_expr.rs:385-402`).
- The remaining `InterfaceMethodCall` branch (for the no-parens legacy single-arg form) is deleted.

## 5. Type System Changes

### 5.1 Relaxing the Single-Parameter Check

In `ash-typeck/src/type_env.rs`, remove the following restrictions:

- `resolve_interface_method_call`: delete `method_info.params.len() != 1` (`type_env.rs:621-626` and `type_env.rs:972-976`)

The `register_impl` single-type-param check stays until SPEC-033.

### 5.2 Method Resolution

`resolve_interface_method_call` generalizes from single-parameter unification to zip-unification over all parameter types:

```rust
if method_info.params.len() != arg_types.len() {
    return Err(TypeEnvError::[NEW] WrongArity(
        format!("expected {} arguments, found {}", method_info.params.len(), arg_types.len())
    ));
}

let mut subst = Substitution::empty();
for (expected, actual) in method_info.params.iter().zip(arg_types) {
    subst = unify(expected, actual)?;
}
```

The remainder of the resolution algorithm (impl lookup, bound checking, return-type application) is unchanged.

### 5.3 Type Checking Call Sites

In `ash-typeck/src/check_expr.rs` and `ash-typeck/src/lib.rs`, interface call validation is folded into `Expr::Call` handling:

```rust
Expr::Call { func, module: Some(interface_name), args, .. } => {
    if env.lookup_interface(interface_name).is_some() {
        let arg_types: Vec<Type> = args.iter()
            .map(|a| infer_surface_expr_type(env, a))
            .collect::<Result<Vec<_>, _>>()?;
        env.resolve_interface_method_call(interface_name, func, &arg_types)
    } else {
        // regular module-qualified function call
        ...
    }
}
```

## 6. Interpreter Changes

Since interface method calls lower to `Expr::Call` with module qualification, and concrete impl methods lower to regular function definitions (or inline during lowering), the interpreter requires **no changes** for SPEC-032 other than removing the unused `InterfaceMethodCall` evaluation branch.

## 7. Migration Path

1. Update `ImplMethodDef` in surface and core AST to `Vec<Name>` params.
2. Update parsers for interface method signatures and impl method definitions.
3. Replace `InterfaceMethodCall` with `Call` in the expression parser.
4. Update type checker to zip-unify over multiple method parameters.
5. Remove all `InterfaceMethodCall` references from type checker (`lib.rs`, `purity.rs`, `names.rs`, `capability_check.rs`) and interpreter (`eval.rs`).
6. Remove `InterfaceMethodCall` lowering rejection (`lower.rs:1219`).
7. Verify all existing single-parameter interface tests still pass.

> **Scope warning:** Step 5 touches 9+ files across parser, type checker, purity checker, capability checker, and interpreter. This is the highest-risk step in SPEC-032.

## 8. Conformance

An implementation conforming to SPEC-032 must:

- Parse interface method signatures with N parameters where N >= 0.
- Parse impl method definitions with matching N parameters.
- Type-check `Interface::method(arg1, arg2, ...)` by unifying each argument type against the corresponding method parameter.
- Continue to reject generic `impl` blocks (concrete type arguments only).
- Continue to restrict interfaces to a single type parameter (relaxed in SPEC-033).

## 9. Files Affected

| File | Change |
|------|--------|
| `crates/ash-parser/src/surface.rs` | Change `ImplMethodDef.param` to `params: Vec<Name>`; deprecate `InterfaceMethodCall` |
| `crates/ash-core/src/ast.rs` | Mirror surface AST changes |
| `crates/ash-parser/src/parse_module.rs` | Parse multi-param method signatures and impl definitions |
| `crates/ash-parser/src/parse_expr.rs` | Remove `InterfaceMethodCall` branch; route to `Expr::Call` only |
| `crates/ash-typeck/src/type_env.rs` | Zip-unify method parameters; remove `len() == 1` check |
| `crates/ash-typeck/src/check_expr.rs` | Interface call validation via `Call` node |
| `crates/ash-typeck/src/lib.rs` | Remove `InterfaceMethodCall` handling |
| `crates/ash-typeck/src/purity.rs` | Remove `InterfaceMethodCall` handling |
| `crates/ash-typeck/src/names.rs` | Remove `InterfaceMethodCall` handling |
| `crates/ash-interp/src/eval.rs` | Remove `InterfaceMethodCall` eval branch |
| `crates/ash-parser/src/lower.rs` | Remove `InterfaceMethodCall` lowering rejection |
