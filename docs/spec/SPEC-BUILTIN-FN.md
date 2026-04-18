# SPEC-BUILTIN-FN: Builtin Function Declaration Form

**Status:** Draft
**Date:** 2026-04-18
**Related:** DESIGN-NOTE-BUILTIN-FN-AND-EXTERN-FN, SPEC-002, SPEC-027, SPEC-031, DESIGN-020

## 1. Overview

This spec introduces `builtin fn` as a new declaration form in Ash. A builtin
function is a **pure** function whose implementation is provided by the Ash
runtime at compile time of the Ash binary. It has no Ash-level body. The
declaration serves as a type signature for the typechecker and a dispatch key
for the runtime.

The `builtin fn` form closes the gap between `pub fn` (user-written Ash bodies)
and capability providers (effectful operations dispatched via `act`). Pure
runtime-provided functions such as `string::concat`, `regex::find`, `len`, and
`head` currently have no proper type-system visibility -- they are hardcoded in
the evaluator with no `.ash` file declaration. This spec gives them first-class
declarations.

**Key properties:**

- Pure: no capability dispatch, no effect escalation.
- No body: the implementation lives in the Rust runtime, not in Ash source.
- Typechecked: the declared signature is authoritative at call sites.
- Always available: compiled into the Ash binary, no runtime loading.

This spec does **not** cover `extern fn` (reserved for future FFI). When
implemented, a separate design note will specify the link-time resolution
protocol, ABI boundary constraints, and effect classification rules for foreign
code. The keyword is reserved now to avoid grammar conflicts.

## 2. Surface Syntax

### 2.1 Grammar

```
builtin_decl ::= visibility? "builtin" "fn" IDENTIFIER type_params? "(" param_list ")" "->" surface_type ";"
visibility   ::= "pub"
type_params  ::= "<" IDENTIFIER ("," IDENTIFIER)* "">"
param_list   ::= /* same as pub fn parameters */
surface_type ::= /* same as existing type expressions */
```

### 2.2 Rules

- **Semicolon-terminated.** No braces, no body. A body is a parse error.
- **`pub` is optional.** Module-private builtins are allowed (visibility is a
  type-checking concern; runtime dispatch is global).
- **Type parameters are optional.** Included for forward compatibility with
  polymorphic builtins (e.g., `List<a>`), but the type system implications of
  generic builtins are deferred.
- **Return type annotation is required.** There is no body to infer from; the
  runtime needs the declared type. A missing return type is a parse error.
- **`builtin` keyword** is placed before `fn`, after any visibility modifier.

### 2.3 Examples

```ash
-- std/src/string.ash
pub builtin fn concat(a: String, b: String) -> String;
pub builtin fn starts_with(s: String, prefix: String) -> Bool;
pub builtin fn ends_with(s: String, suffix: String) -> Bool;
pub builtin fn is_empty(s: String) -> Bool;

-- std/src/regex.ash
pub builtin fn find(pattern: String, text: String) -> Option<String>;
pub builtin fn matches(pattern: String, text: String) -> Bool;

-- module-private builtin
builtin fn internal_helper(x: Int) -> Int;
```

**Not yet declarable (type predicates):**

```ash
-- Requires ad-hoc polymorphism: accepts any value, returns Bool
-- These use a type parameter <a> and cannot be declared until the type
-- system supports universally-quantified builtin parameters.
pub builtin fn is_int<a>(value: a) -> Bool;
pub builtin fn is_string<a>(value: a) -> Bool;
pub builtin fn is_bool<a>(value: a) -> Bool;
pub builtin fn is_list<a>(value: a) -> Bool;
pub builtin fn is_record<a>(value: a) -> Bool;
pub builtin fn is_null<a>(value: a) -> Bool;
```

## 3. AST Changes (ash-parser)

A new variant is added to the surface AST `Definition` enum in
`crates/ash-parser/src/surface.rs`:

```rust
Definition::BuiltinFn(BuiltinFnDef {
    visibility: Option<Visibility>,
    name: Name,
    type_params: Vec<Name>,
    params: Vec<Param>,
    return_type: SurfaceType,  // required, not Optional
    span: Span,
})
```

- **No body field.** Unlike `Definition::Function`, there is no expression body.
- `visibility`, `name`, `params`, `return_type`, and `span` parallel the
  existing `Function` definition fields.
- `return_type` is required (not `Option<SurfaceType>`), matching the grammar.
- `type_params` supports optional generic parameters (reserved for future use).

## 4. Parser Changes (ash-parser)

### 4.1 Recognition

In `parse_module.rs`, after consuming an optional `pub` visibility modifier, the
parser checks for the `builtin` keyword before `fn`:

1. `pub builtin fn ...` -- public builtin function.
2. `builtin fn ...` -- module-private builtin function.
3. `pub fn ...` -- existing public function (unchanged).
4. `fn ...` -- existing module-private function (unchanged).

### 4.2 Parsing Rules

- Consume `builtin`, then `fn`, then identifier, then optional type parameters,
  then parenthesized parameter list, then **required** `->` return type.
- **Expect semicolon terminator.** If the next token after the return type is
  `{`, emit a parse error: "builtin fn must not have a body".
- **Require return type.** If the next token after the closing paren is `;`
  (no `->` return type), emit a parse error: "builtin fn requires a return type
  annotation".
- Produce `Definition::BuiltinFn(BuiltinFnDef { ... })` with no body.

### 4.3 Lowering

`BuiltinFnDef` lowers to a callable registration with a `CallableKind::Builtin`
marker in the IR (see Section 5). No IR expressions are generated for the
function body -- the callable carries no body expression.

## 5. Module Loader and IR Changes (ash-engine)

### 5.1 CallableKind Discriminant

A new discriminant is added to `InlineCallable` in
`crates/ash-engine/src/module_loader.rs` to distinguish builtin callables from
Ash-bodied callables:

```rust
pub struct InlineCallable {
    pub exported_name: String,
    pub params: Vec<String>,
    pub kind: CallableKind,
}

pub enum CallableKind {
    Ash { body: Expr },
    Builtin,
}
```

This replaces the current `body: Expr` field. Existing `pub fn` and `workflow`
callables use `CallableKind::Ash { body }`. Builtin fn callables use
`CallableKind::Builtin`.

**InlineCallable consumer sites** that currently access `.body` directly and
must be updated to match on `CallableKind`:

1. **Evaluator closure construction** (`ash-interp/src/eval.rs`): when
   building a `Value::Closure` from an imported `InlineCallable`, the code
   currently reads `callable.body` unconditionally. For `CallableKind::Builtin`,
   the closure must carry no body and the evaluator must dispatch to the
   builtin table instead of evaluating an expression.
2. **Module import resolution** (`ash-engine/src/module_loader.rs:merge_use_exports`,
   `resolve_import`): these clone `InlineCallable` values. The `CallableKind`
   discriminant is cloned as-is and needs no special handling, but any code
   that destructures `.body` must be audited.
3. **Type environment registration** (`ash-typeck/src/type_env.rs`):
   `add_builtin_functions()` currently seeds type signatures in Rust. After
   migration, the typechecker must read signatures from `InlineCallable`'s
   declared parameter/return types, distinguishing by `CallableKind` only if
   the registration path differs.

### 5.2 Snippet Extraction in collect_module_exports

In `collect_module_exports` (`module_loader.rs:418-479`), builtin fn exports
are discovered via snippet extraction, following the same pattern as existing
declarations. Since `builtin fn` is semicolon-terminated (no braces), it uses
`extract_semicolon_snippets`, similar to `pub type`:

```rust
// Existing: pub type (semicolon-terminated)
for snippet in extract_semicolon_snippets(&source, |trimmed| {
    trimmed.starts_with("pub type ")
}) { ... }

// New: pub builtin fn and builtin fn (semicolon-terminated)
for snippet in extract_semicolon_snippets(&source, |trimmed| {
    trimmed.starts_with("pub builtin fn ") || trimmed.starts_with("builtin fn ")
}) {
    match parse_builtin_fn_callable(&snippet) {
        Ok(Some(callable)) => {
            insert_callable_export(&mut exports, &callable.name, callable.callable)?;
        }
        Err(_) => { /* skip, surfaced via check_module_file */ }
    }
}

// Existing: pub fn (braced) -- unchanged
for snippet in extract_braced_snippets(&source, |trimmed| {
    trimmed.starts_with("pub fn ")
}) { ... }
```

The `parse_builtin_fn_callable` helper parses the snippet, extracts the name
and parameter list, and produces an `InlineCallable` with
`kind: CallableKind::Builtin`.

### 5.3 Exports

- `pub builtin fn` declarations are exported from the module, identical to
  `pub fn`.
- Module-private `builtin fn` (no `pub`) is visible only within the declaring
  module.

### 5.4 Import Resolution

- `use module::{name}` resolves builtin fn signatures the same as pub fn
  signatures.
- The `CallableKind::Builtin` marker is preserved through import resolution.
  At import time, the callable is treated identically to any other callable;
  the marker is inspected only at dispatch time.

### 5.5 pub use Re-export

`pub use` re-exports preserve the `CallableKind::Builtin` marker through the
existing merge logic in `merge_use_exports` (`module_loader.rs:529-580`). The
`InlineCallable` is cloned as-is, including its `kind` field, so the builtin
marker survives re-export without special handling.

### 5.6 Module Cache

Builtin fn exports participate in the module cache identically to other
callable exports. `collect_module_exports` caches results (line 422-425); builtin
fn exports are included in the cached `ModuleExports` and served from cache on
subsequent lookups.

## 6. Typechecker Changes (ash-typeck)

### 6.1 Type Assignment

A builtin fn's type is `Type::Fn(params, ret)` -- identical to `pub fn` (pure).

### 6.2 Checking Rules

- The declared signature is **authoritative** for type checking.
- Calls to builtin fns typecheck identically to calls to pub fns.
- No effect escalation: calling a builtin fn from a `fn` body is legal (both
  are pure).
- No capability dispatch: no effect classification is associated with builtin
  fn calls.

### 6.3 Environment

The typechecker records the builtin fn's signature in the module's type
environment. When a call site references a builtin fn, the typechecker looks up
the declared signature and checks argument types against parameter types.

## 7. Runtime Changes (ash-interp)

### 7.1 Dispatch Mechanism

When the evaluator encounters a call, it determines dispatch as follows:

1. Resolve the callable from the imported closures table by name.
2. Check the callable's `CallableKind`:
   - `CallableKind::Ash { body }` -- evaluate the body expression (existing path).
   - `CallableKind::Builtin` -- dispatch to the builtin dispatch table.
3. For builtin dispatch: resolve the fully qualified name (e.g.,
   `std::string::concat`), look up the Rust implementation in the **builtin
   dispatch table** (a static mapping from qualified names to Rust functions),
   evaluate arguments, invoke the Rust function, return the result.

The `CallableKind` marker is preserved from module export through import
resolution to the runtime closure table, so the evaluator can reliably
distinguish builtin calls from Ash-bodied calls without string-based heuristics.

### 7.2 Error Handling

If a `builtin fn` is declared in an `.ash` file but has no corresponding entry
in the runtime dispatch table, the evaluator produces a distinct error:

```rust
EvalError::UnimplementedBuiltin { name: String }
```

This is distinct from "function not found" errors. The error message is:

```
error: builtin function 'module::name' declared but not implemented in runtime
```

Compile-time verification of the dispatch table against declared builtins is
preferred. Runtime error is the fallback.

### 7.3 Lifecycle

```
.ash declaration (type signature)
        |
        v
  Parser emits BuiltinFnDef surface AST node (ash-parser)
        |
        v
  Module loader extracts via extract_semicolon_snippets,
  produces InlineCallable { kind: CallableKind::Builtin }
        |
        v
  Typechecker records signature in module environment
        |
        v
  At call site: typecheck against declared signature
        |
        v
  Evaluator checks CallableKind::Builtin, dispatches to Rust builtin table
```

No separate registration step. No provider lookup. No capability boundary
crossing.

## 8. Relationship to Existing Constructs

| Declaration      | Implementation         | Effect          | Example                     |
|------------------|------------------------|-----------------|-----------------------------|
| `pub fn`         | Ash expression body    | Pure (no effect)| `io::path::join`            |
| `builtin fn`     | Rust runtime, compiled | Pure (no effect)| `string::concat`, `regex::find` |
| capability+`act` | Rust capability provider| Effect from lattice | `fs::write_file`        |
| `extern fn`      | Reserved (future FFI)  | TBD             | Not in this spec            |

**Composition rules (per DESIGN-020 Three-Vertex Model):**

- `fn` -> `fn` (freely composes)
- `fn` -> `builtin fn` (freely composes -- both are pure transforms)
- `workflow` -> `fn` (workflow calls fn for data transforms)
- `workflow` -> `builtin fn` (same -- pure data transforms)
- `workflow` -> `cap` (workflow uses capabilities for effects)
- `fn` -/-> `workflow` (functions never invoke workflows)
- `fn` -/-> `cap` (functions never use capabilities)

`builtin fn` sits alongside `pub fn` at the Transform vertex. Both are pure.
Both can be called from `fn` bodies and `workflow` bodies. The only difference
is where the implementation lives.

## 9. Migration

### 9.1 Phase 1: Strictly Monomorphic Builtins (In Scope)

**RegexProvider Deletion:**

`RegexProvider` is deleted entirely. Regex operations are pure string
computations and become `builtin fn` declarations in `std/src/regex.ash`. There
is no capability provider, no `act` dispatch, and no effect classification for
regex.

**New .ash Declaration Files (strictly monomorphic):**

| Module                         | Functions                                        | Status |
|--------------------------------|--------------------------------------------------|--------|
| `std/src/string.ash` (new)     | `concat`, `starts_with`, `ends_with`, `is_empty` | In scope -- all concrete types |
| `std/src/regex.ash` (new)      | `find`, `matches`, `replace`                     | In scope -- all concrete types |

All Phase 1 builtins are strictly monomorphic: every parameter and return type
is concrete (no type variables).

**NOT in scope for Phase 1:**

| Category             | Functions                                               | Reason                          |
|----------------------|---------------------------------------------------------|---------------------------------|
| Type predicates      | `is_int`, `is_string`, `is_bool`, `is_list`, `is_record`, `is_null` | Require ad-hoc polymorphism (`<a>`) |
| List operations      | `len`, `head`, `tail`, `append`, `concat`, `filter`, `map` | Require parametric polymorphism (`List<a>`) |
| Record operations    | `keys`, `values`, `record`                              | Require record-type generics    |

These remain hardcoded in the evaluator and typechecker until their respective
generic semantics are designed.

### 9.2 Phase 1.5: Type Predicate Builtins (Blocked on Ad-Hoc Polymorphism)

Type predicates accept any value and return `Bool`. They require at least
simple ad-hoc polymorphism in the typechecker (the type parameter `<a>` is
universally quantified but unused in the return type). This is a simpler
generic mechanism than full parametric polymorphism (`List<a> -> a`), but it
does not exist yet.

### 9.3 Phase 2: Polymorphic Builtins (Out of Scope)

List operations (`len`, `head`, `tail`, `append`, `concat`, `filter`, `map`)
and record operations (`keys`, `values`, `record`) require full generic type
parameters. These are deferred pending generic builtin semantics. Phase 2 is
explicitly out of scope for this spec.

### 9.4 Evaluator Refactor

Evaluator hardcoded builtins remain as dispatch targets but gain proper
declarations. The dispatch mechanism shifts from hardcoded string matching to
qualified-name lookup from the module system.

### 9.5 Backward-Compatibility Contract for Current Builtins

The current evaluator supports **dual dispatch** for several builtins: they
respond to both qualified calls (e.g., `string::concat`) and unqualified calls
(e.g., `starts_with`). The migration must specify the fate of each.

**Current dispatch inventory** (from `eval.rs:477-831`):

| Function | Current qualified form | Current unqualified form | Migration target | Compatibility rule |
|----------|----------------------|--------------------------|-----------------|-------------------|
| `string::concat` | `(Some("string"), "concat")` | -- | `std/src/string.ash` | Qualified only |
| `string::starts_with` | `(Some("string"), "starts_with")` | `(_, "starts_with")` | `std/src/string.ash` | **BREAKING**: unqualified form removed. Must use `string::starts_with` |
| `string::ends_with` | `(Some("string"), "ends_with")` | `(_, "ends_with")` | `std/src/string.ash` | **BREAKING**: unqualified form removed. Must use `string::ends_with` |
| `string::is_empty` | `(Some("string"), "is_empty")` | -- | `std/src/string.ash` | Qualified only |
| `len` | -- | `(_, "len")` | Deferred (D2) | Remains unqualified until generics land |
| `head` | -- | `(_, "head")` | Deferred (D2) | Remains unqualified until generics land |
| `tail` | -- | `(_, "tail")` | Deferred (D2) | Remains unqualified until generics land |
| `append` | -- | `(_, "append")` | Deferred (D2) | Remains unqualified until generics land |
| `concat` (list) | -- | `(_, "concat")` | Deferred (D2) | Remains unqualified until generics land |
| `filter` | -- | `(_, "filter")` | Deferred (D2) | Remains unqualified until generics land |
| `map` | -- | `(_, "map")` | Deferred (D2) | Remains unqualified until generics land |
| `keys` | -- | `(_, "keys")` | Deferred (D2) | Remains unqualified until generics land |
| `values` | -- | `(_, "values")` | Deferred (D2) | Remains unqualified until generics land |
| `is_int` | -- | `(_, "is_int")` | Deferred (D1.5) | Remains unqualified until ad-hoc polymorphism lands |
| `is_string` | -- | `(_, "is_string")` | Deferred (D1.5) | Remains unqualified until ad-hoc polymorphism lands |
| `is_bool` | -- | `(_, "is_bool")` | Deferred (D1.5) | Remains unqualified until ad-hoc polymorphism lands |
| `is_list` | -- | `(_, "is_list")` | Deferred (D1.5) | Remains unqualified until ad-hoc polymorphism lands |
| `is_record` | -- | `(_, "is_record")` | Deferred (D1.5) | Remains unqualified until ad-hoc polymorphism lands |
| `is_null` | -- | `(_, "is_null")` | Deferred (D1.5) | Remains unqualified until ad-hoc polymorphism lands |
| `record` | -- | `(_, "record")` | Deferred (D2) | Remains unqualified until generics land |

**Breaking changes (Phase 1 only):**

Two unqualified forms are removed in Phase 1: `starts_with` and `ends_with`.
These currently have dual dispatch (both `string::starts_with` and bare
`starts_with`). After migration to `std/src/string.ash`, only the qualified
form `string::starts_with` is supported. Ash code calling `starts_with(...)`
must be updated to `string::starts_with(...)`.

This is intentional: the unqualified forms were ambiguous aliases, and the
declaration-based system requires explicit module qualification.

**Non-breaking (deferred):**

All other unqualified builtins remain available exactly as today until their
respective migration tracks (D1.5 for type predicates, D2 for list/record ops)
are unblocked.

## 10. Invariants

1. **No body.** A `builtin fn` MUST NOT have a body. A parse error is emitted
   if braces follow the signature.

2. **Return type required.** A `builtin fn` MUST declare a return type. There
   is no body to infer from.

3. **Runtime implementation required.** A `builtin fn` MUST have a
   corresponding implementation in the runtime dispatch table. Compile-time
   verification is preferred; runtime error (`EvalError::UnimplementedBuiltin`)
   is the fallback.

4. **Pure.** A `builtin fn` is pure: no capability dispatch, no effect
   escalation. It occupies the same position as `pub fn` in the three-vertex
   model.

5. **Callable from pure contexts.** A `builtin fn` can be called from `fn`
   bodies or workflow expressions -- same as `pub fn`.

6. **Signature matches runtime.** The declared signature MUST match the runtime
   implementation's actual behavior. The typechecker trusts the declared
   signature; violations are bugs in the runtime, not type errors.

7. **Unique names.** A module MUST NOT declare both `pub fn foo(...)` and
   `builtin fn foo(...)`. The qualified name must be unique within a module.

## 11. Test Requirements

### 11.1 Parser

| Test                                              | Expected                        |
|---------------------------------------------------|---------------------------------|
| `builtin fn name(params) -> Type;`                | Parses successfully             |
| `pub builtin fn name(params) -> Type;`            | Parses successfully             |
| `builtin fn name(params) -> Type { body }`        | Parse error: no body allowed    |
| `builtin fn name(params);` (no return type)       | Parse error: return type required |
| `builtin fn name<T>(x: T) -> T;`                  | Parses with type params         |

### 11.2 Module Loader

| Test                                              | Expected                        |
|---------------------------------------------------|---------------------------------|
| Module exports `pub builtin fn` declarations      | Visible in module export table  |
| Module-private `builtin fn` not exported          | Not in export table             |
| `use module::{name}` resolves builtin fn          | Import resolves successfully    |
| Builtin fn callable has `CallableKind::Builtin`   | Marker preserved through import |
| `pub use` re-export of builtin fn                 | Builtin marker preserved        |
| Builtin fn exports served from module cache       | Cache hit returns same exports  |

### 11.3 Typechecker

| Test                                              | Expected                        |
|---------------------------------------------------|---------------------------------|
| Call to builtin fn with correct argument types    | Typechecks                      |
| Call to builtin fn with wrong argument types      | Type error                      |
| Builtin fn called from `fn` body                  | Typechecks (pure call)          |
| Builtin fn called from `workflow` body            | Typechecks (pure call)          |

### 11.4 Runtime

| Test                                              | Expected                        |
|---------------------------------------------------|---------------------------------|
| Call to declared+implemented builtin fn           | Returns correct result          |
| Call to declared but unimplemented builtin fn     | `EvalError::UnimplementedBuiltin` |
| Unknown builtin fn name                           | Error with qualified name       |

### 11.5 End-to-End (Regex)

```ash
use regex::{find}
let result = regex::find("a+", "abc")
-- result == Some("a")
```

- `use regex::{find}` resolves the builtin fn signature.
- `regex::find("a+", "abc")` dispatches to the Rust regex implementation.
- Returns `Some("a")`.
