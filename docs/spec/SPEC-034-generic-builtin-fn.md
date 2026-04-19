# SPEC-034: Generic Builtin fn Declarations

**Status:** Draft
**Date:** 2026-04-19
**Related:** SPEC-BUILTIN-FN, DESIGN-NOTE-BUILTIN-FN-AND-EXTERN-FN, SPEC-002

## 1. Overview

This spec extends `builtin fn` declarations to support generic type parameters,
unblocking two deferred categories of stdlib builtins:

1. **List operations** (`len`, `head`, `tail`, `append`, `concat`, `filter`, `map`):
   parametric polymorphic builtins over `List<a>`.
2. **Type predicates** (`is_int`, `is_string`, `is_bool`, `is_list`, `is_record`,
   `is_null`): ad-hoc polymorphic builtins that accept any value and return `Bool`.

The type system already has the internal machinery (`Type::Var`, unification,
`builtin_fn_signature_type`). The parser already recognizes `<T>` on builtin fn.
The critical gap is **type-signature propagation through `ash-engine`'s import
path**: today, imported builtin declarations lose their type signatures and are
registered in the type environment using arity-only synthetic types.

## 2. Current State

### 2.1 What Works

- Parser recognizes `builtin fn name<T>(...) -> Ret;` with `type_params: Vec<Name>`.
- Surface AST `BuiltinFnDef` carries `type_params` and full parameter/return types.
- Core AST `BuiltinFnDef` carries the same.
- Typechecker's `builtin_fn_signature_type()` correctly maps type params to
  fresh `TypeVar` and resolves parameter/return types through the type environment.
- Typechecker's `register_function_signatures()` handles `Definition::BuiltinFn`.
- Unification handles `Type::List(Box<Type>)` and `Type::Fn(Vec<Type>, Box<Type>)`.

### 2.2 The Gap: Engine Import Path Loses Type Signatures

The engine's import pipeline has two breaks:

**Break 1: `InlineCallable` discards type information.**

`InlineCallable` in `module_loader.rs` stores:
```rust
pub struct InlineCallable {
    pub exported_name: String,
    pub params: Vec<String>,  // names only, no types
    pub kind: CallableKind,
}
```

When `parse_builtin_fn_callable()` extracts a builtin fn from a snippet, it
records param names but discards `params` type annotations and `return_type`.
The original `BuiltinFnDef` with full type information is not preserved.

**Break 2: `Engine::check()` binds imported callables with arity-only types.**

In `Engine::check()` (lines 532-537, 568-573):
```rust
for (name, &param_count) in &workflow.imported_param_counts {
    let param_types: Vec<Type> = (0..param_count)
        .map(|_| Type::Var(TypeVar::fresh()))
        .collect();
    let ret_type = Type::Var(TypeVar::fresh());
    type_env.bind_variable(name, Type::Fn(param_types, Box::new(ret_type)));
}
```

This binds each imported callable as `Fn(Var, Var, ..., Var) -> Var` --
completely unconstrained. For `len<a>(list: List<a>) -> Int`, the type
environment gets `Fn(Var(N)) -> Var(M)` instead of `Fn(List<Var(N)>) -> Int`.

**Consequence:** Even if `std/src/list.ash` declares `builtin fn len<a>(list: List<a>) -> Int;`,
and the module loader successfully extracts and imports it, the typechecker will
never see the `List<a>` constraint or `Int` return type through the import path.
Calls to `len` would typecheck with any argument type and produce an unknown
return type.

### 2.3 What Does NOT Need Fixing

**Call-site freshening is probably not needed.** `instantiate_fn_call` starts
with a fresh `Substitution` per call. Function types bound in the type
environment are immutable. Each call site gets its own substitution scope, so
shared `TypeVar` instances in the stored type do not leak across calls. A
regression test should confirm this, but it is not an architectural prerequisite.

## 3. Required Changes

### 3.1 Preserve Type Signatures in `InlineCallable` (ash-engine)

Add an optional field to `InlineCallable` carrying the declared type signature
for builtin callables:

```rust
pub struct InlineCallable {
    pub exported_name: String,
    pub params: Vec<String>,
    pub kind: CallableKind,
    /// For CallableKind::Builtin: the declared type signature, if available.
    /// Stored as a surface BuiltinFnDef so the typechecker can resolve type
    /// params against its own TypeEnv.
    pub signature: Option<ash_parser::surface::BuiltinFnDef>,
}
```

`parse_builtin_fn_callable()` populates `signature` from the parsed definition.
User-defined callables leave this as `None`.

### 3.2 Bind Imported Builtin Signatures in `Engine::check()` (ash-engine)

When `Engine::check()` iterates imported callables, instead of the arity-only
synthetic binding, check if the callable has a signature and use it:

```rust
for (name, callable) in &workflow.imported_callables {
    if let Some(ref sig) = callable.signature {
        // Use the declared signature with proper type param resolution
        let ty = builtin_fn_signature_type(&type_env, sig)?;
        type_env.bind_variable(name, ty);
    } else {
        // Fallback: arity-only synthetic type
        let param_types: Vec<Type> = (0..callable.params.len())
            .map(|_| Type::Var(TypeVar::fresh()))
            .collect();
        let ret_type = Type::Var(TypeVar::fresh());
        type_env.bind_variable(name, Type::Fn(param_types, Box::new(ret_type)));
    }
}
```

This requires the engine to pass `imported_callables` (not just
`imported_param_counts`) to the typechecking path, or at minimum carry the
signature alongside the param counts.

### 3.3 Stdlib Declaration Files (ash-stdlib)

Two new files with generic builtin fn declarations:

**`std/src/list.ash`:**
```ash
pub builtin fn len<a>(list: List<a>) -> Int;
pub builtin fn head<a>(list: List<a>) -> a;
pub builtin fn tail<a>(list: List<a>) -> List<a>;
pub builtin fn append<a>(list: List<a>, elem: a) -> List<a>;
pub builtin fn concat<a>(left: List<a>, right: List<a>) -> List<a>;
pub builtin fn filter<a>(list: List<a>, predicate: Fn(a) -> Bool) -> List<a>;
pub builtin fn map<a, b>(list: List<a>, transform: Fn(a) -> b) -> List<b>;
```

**`std/src/predicate.ash`:**
```ash
pub builtin fn is_int<a>(value: a) -> Bool;
pub builtin fn is_string<a>(value: a) -> Bool;
pub builtin fn is_bool<a>(value: a) -> Bool;
pub builtin fn is_list<a>(value: a) -> Bool;
pub builtin fn is_record<a>(value: a) -> Bool;
pub builtin fn is_null<a>(value: a) -> Bool;
```

### 3.4 Builtin Dispatch Table (ash-interp)

Add qualified entries to the dispatch table for list ops (`list::len`, etc.)
and type predicates (`predicate::is_int`, etc.). The existing `eval_function_call`
match arms remain as the runtime implementations.

### 3.5 Cleanup: Delete `add_builtin_functions()` (ash-typeck)

After list op declarations are wired end-to-end, `add_builtin_functions()`
(which only contains list ops now) can be deleted. This is the unblocked portion
of TASK-631B.

## 4. Type-Variable Freshening Audit

Current code structure strongly suggests freshening is not needed:

- `instantiate_fn_call` creates a fresh `Substitution` per call (line 408).
- Function types bound in `TypeEnv` are immutable after insertion.
- The substitution is local to each call's `CheckResult`.

A regression test should verify that two sequential calls to the same polymorphic
builtin with different concrete types (e.g., `len([1,2])` then `len(["a"])`)
both typecheck correctly. If this passes, no freshening machinery is needed.

## 5. Migration Inventory

### 5.1 List Operations (Track D2)

| Function  | Current Type (hardcoded)                    | .ash Declaration                              |
|-----------|---------------------------------------------|-----------------------------------------------|
| `len`     | `Fn(List<a>, Int)`                          | `pub builtin fn len<a>(list: List<a>) -> Int;`|
| `head`    | `Fn(List<a>, a)`                            | `pub builtin fn head<a>(list: List<a>) -> a;` |
| `tail`    | `Fn(List<a>, List<a>)`                      | `pub builtin fn tail<a>(list: List<a>) -> List<a>;` |
| `append`  | `Fn(List<a>, a, List<a>)`                   | `pub builtin fn append<a>(list: List<a>, elem: a) -> List<a>;` |
| `concat`  | `Fn(List<a>, List<a>, List<a>)`             | `pub builtin fn concat<a>(left: List<a>, right: List<a>) -> List<a>;` |
| `filter`  | `Fn(List<a>, Fn(a)->Bool, List<a>)`         | `pub builtin fn filter<a>(list: List<a>, predicate: Fn(a) -> Bool) -> List<a>;` |
| `map`     | `Fn(List<a>, Fn(a)->b, List<b>)`            | `pub builtin fn map<a, b>(list: List<a>, transform: Fn(a) -> b) -> List<b>;` |

### 5.2 Type Predicates (Track D1.5)

All declared as `pub builtin fn is_X<a>(value: a) -> Bool;`.

### 5.3 Backward Compatibility

Unqualified forms (`len(...)`, `is_int(...)`) continue to work. The dispatch
table has entries for both qualified and unqualified names. No breaking change.

## 6. Current Implementation Split (Honest)

The current state is not a single unified model:

| Layer | List ops | Type predicates | String ops |
|-------|----------|-----------------|------------|
| **Type env** | Hardcoded via `add_builtin_functions()` | Not registered (removed by TASK-631A) | Via `.ash` declarations |
| **Runtime dispatch** | Unqualified match arms in `eval_function_call` | Unqualified match arms | Qualified dispatch table entries |
| **Module import** | Arity-only synthetic types | Arity-only synthetic types | Arity-only synthetic types |

This spec aims to unify all three categories under the `.ash` declaration path
by fixing the import signature propagation gap.
