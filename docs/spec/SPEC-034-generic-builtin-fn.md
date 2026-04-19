# SPEC-034: Generic Builtin fn Declarations

**Status:** Draft
**Date:** 2026-04-19
**Related:** SPEC-BUILTIN-FN, DESIGN-NOTE-BUILTIN-FN-AND-EXTERN-FN, SPEC-002

## 1. Overview

This spec extends `builtin fn` declarations to support generic type parameters,
unblocking two deferred categories of stdlib builtins:

1. **Type predicates** (`is_int`, `is_string`, `is_bool`, `is_list`, `is_record`,
   `is_null`): ad-hoc polymorphic builtins that accept any value and return `Bool`.
2. **List operations** (`len`, `head`, `tail`, `append`, `concat`, `filter`, `map`):
   parametric polymorphic builtins over `List<a>`.

The good news: the type system already has the machinery. `Type::Var(TypeVar)`
provides type variables, `builtin_fn_signature_type` in `ash-typeck/src/lib.rs`
already maps declared type params to fresh type variables, and the unifier
already handles `Type::List(Box<Type>)` and `Type::Fn(Vec<Type>, Box<Type>)`.
The work is wiring, not architecture.

## 2. Syntax

### 2.1 Grammar (unchanged from SPEC-BUILTIN-FN)

```
builtin_decl ::= visibility? "builtin" "fn" IDENTIFIER type_params? "(" param_list ")" "->" surface_type ";"
type_params  ::= "<" IDENTIFIER ("," IDENTIFIER)* ">"
```

The parser already recognizes `<T>` on `builtin fn`. No grammar changes needed.

### 2.2 New Declaration Files

```ash
-- std/src/list.ash
pub builtin fn len<a>(list: List<a>) -> Int;
pub builtin fn head<a>(list: List<a>) -> a;
pub builtin fn tail<a>(list: List<a>) -> List<a>;
pub builtin fn append<a>(list: List<a>, elem: a) -> List<a>;
pub builtin fn concat<a>(left: List<a>, right: List<a>) -> List<a>;
pub builtin fn filter<a>(list: List<a>, predicate: Fn(a) -> Bool) -> List<a>;
pub builtin fn map<a, b>(list: List<a>, transform: Fn(a) -> b) -> List<b>;

-- std/src/predicate.ash
pub builtin fn is_int<a>(value: a) -> Bool;
pub builtin fn is_string<a>(value: a) -> Bool;
pub builtin fn is_bool<a>(value: a) -> Bool;
pub builtin fn is_list<a>(value: a) -> Bool;
pub builtin fn is_record<a>(value: a) -> Bool;
pub builtin fn is_null<a>(value: a) -> Bool;
```

**Note on type parameter usage:** For type predicates, the type parameter `<a>`
is used only in the parameter position -- the return type is always `Bool`. This
is a special case where the type variable is universally quantified at the call
site: any concrete argument type is accepted. No new type-system mechanism is
needed beyond what `Type::Var` + unification already provides.

## 3. Type System

### 3.1 Type Assignment

A generic builtin fn's type is `Type::Fn(params, ret)` where params/ret may
contain `Type::Var(TypeVar)` for declared type parameters. This is identical to
what `add_builtin_functions()` in `type_env.rs` already produces.

**Example:** `builtin fn len<a>(list: List<a>) -> Int;` produces:
```rust
Type::Fn(
    vec![Type::List(Box::new(Type::Var(TypeVar(N))))],
    Box::new(Type::Int),
)
```

### 3.2 Type Checking at Call Sites

When a call to a generic builtin is typechecked:

1. **Fresh instantiation:** For each call site, the builtin's type signature is
   instantiated with fresh type variables. This ensures calls like
   `len([1, 2, 3])` and `len(["a", "b"])` use distinct type variables.
2. **Unification:** Argument types are unified with the instantiated parameter
   types. The unifier resolves type variables as needed.
3. **Return type:** The return type, after applying the substitution from
   unification, becomes the type of the call expression.

This is already how `check_expr` works for function calls when the variable is
bound to a `Type::Fn(...)` containing `Type::Var`. No new checking logic needed.

### 3.3 No Effect Escalation

Generic builtins are pure, identical to monomorphic builtins. No effect
annotation. No capability dispatch.

## 4. Changes by Crate

### 4.1 ash-parser (minimal)

No changes. The parser already:
- Recognizes `builtin fn name<T>(...) -> Ret;`
- Stores `type_params: Vec<Name>` in `BuiltinFnDef`
- Parses `List<T>` as `Type::List(Box<Type>)` with `T` as `Type::Name`

### 4.2 ash-engine (module loader)

The module loader's `parse_builtin_fn_callable` already parses generic builtin
fn snippets. The `InlineCallable` records param names and `CallableKind::Builtin`.
No changes needed for module resolution.

### 4.3 ash-typeck (minimal)

`builtin_fn_signature_type` already maps type params to fresh `TypeVar`. The
`register_function_signatures` function already handles `BuiltinFn` definitions.

**Change needed:** When `check_expr` resolves a call to a variable bound to a
polymorphic type (containing `Type::Var`), it must **freshen** the type variables
at each call site. Currently `add_builtin_functions` binds each builtin once
with shared `TypeVar` instances. If those are reused across call sites without
freshening, two calls with different argument types would incorrectly unify.

**Detection:** Check if `check_expr`'s call resolution already freshens type
variables when looking up a variable's type. If it does, no change needed. If it
does not, add a `instantiate` step that replaces bound type vars with fresh ones.

### 4.4 ash-interp (runtime dispatch)

The dispatch table in `builtin_dispatch_table()` needs entries for list ops and
type predicates. The existing `eval_function_call` match arms for `(_, "len")`,
`(_, "head")`, etc. continue as the runtime implementations.

For **type predicates**, the current match arms in `eval_function_call` already
handle them under `(_, "is_int")`, `(_, "is_string")`, etc. The dispatch table
just needs entries so the qualified form (`predicate::is_int`) routes correctly.

For **list ops**, same pattern: dispatch table entries map qualified names to
the existing implementations.

### 4.5 ash-typeck (cleanup)

After both declaration files are wired, `add_builtin_functions()` can be deleted
(or reduced to empty) since all its registrations are now handled by
`.ash` file declarations. This is TASK-631B from Phase 92.

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

| Function     | .ash Declaration                                    |
|-------------|-----------------------------------------------------|
| `is_int`    | `pub builtin fn is_int<a>(value: a) -> Bool;`       |
| `is_string` | `pub builtin fn is_string<a>(value: a) -> Bool;`    |
| `is_bool`   | `pub builtin fn is_bool<a>(value: a) -> Bool;`      |
| `is_list`   | `pub builtin fn is_list<a>(value: a) -> Bool;`      |
| `is_record` | `pub builtin fn is_record<a>(value: a) -> Bool;`    |
| `is_null`   | `pub builtin fn is_null<a>(value: a) -> Bool;`      |

### 5.3 Backward Compatibility

These builtins currently work as unqualified names (`len(...)`, `is_int(...)`).
After migration, the unqualified forms continue to work because the dispatch
table has entries for both qualified (`list::len`) and unqualified (`len`) names.
The `.ash` declarations provide the type signatures; the runtime match arms
provide the implementations. No breaking change.

## 6. Risks and Open Questions

### 6.1 Type Variable Freshening at Call Sites

**Risk:** If the typechecker does not freshen type variables when resolving a
call to a polymorphic builtin, then two calls with different argument types
(e.g., `len([1,2])` and `len(["a"])`) would share the same `TypeVar`, causing
incorrect unification or a spurious type error.

**Mitigation:** Audit `check_expr`'s call resolution path. If freshening is
missing, add it. This is a prerequisite for any generic builtin, not specific
to list ops or predicates.

### 6.2 `Fn(a) -> Bool` in Type Signatures

`filter` and `map` accept function-typed parameters (`Fn(a) -> Bool`, `Fn(a) -> b`).
The surface type `Fn(T) -> U` parses as `Type::Fn(Vec<Type>, Box<Type>)`. This
must survive the surface-to-typechecker conversion in `workflow_surface_type_to_type`.
Verify this path handles `Fn(...)` types inside generic builtin fn signatures.

### 6.3 Type Predicate Semantics

Type predicates are unusual: they accept any type but always return `Bool`. The
type parameter `<a>` is a phantom -- it never influences the return type. At
the call site, `is_int("hello")` should typecheck as `Bool` even though `a ~ String`.
This works naturally with the unification approach: the parameter type `a`
unifies with the argument type, and the return type is fixed `Bool`.

## 7. Relationship to Other Specs

| Spec | Relationship |
|------|-------------|
| SPEC-BUILTIN-FN | This spec is an extension: generic type params on builtin fn |
| SPEC-002 (type system) | Provides Type::Var, unification, substitution |
| DESIGN-020 (three-vertex) | Generic builtins sit at the Transform vertex (pure) |
