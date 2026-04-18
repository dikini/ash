# Design Note: `builtin fn` and `extern fn` Declaration Forms

**Status:** Draft
**Scope:** Language Grammar, Evaluator Dispatch, Type System
**Related:** DESIGN-020 (Pure Functions / Three-Vertex Model), SPEC-002, SPEC-031

---

## 1. Problem Statement

Ash currently provides two ways to deliver functionality:

| Form | Implementation | Effect |
|------|---------------|--------|
| `pub fn` with Ash-expression body | User-written Ash code | Pure |
| Capability providers via `act` | Rust capability provider | Effect from lattice |

This leaves a gap: **pure runtime-provided functions** that have no Ash-level body but are not effectful. Examples include `string::concat`, `string::starts_with`, `len`, `head`, `tail`, and regex operations. These are currently "magic" -- hardcoded in the evaluator with no `.ash` file declaration.

The type situation is uneven. 13 builtins (len, head, tail, append, concat, filter, map, starts_with, ends_with, string::concat, string::starts_with, string::ends_with, string::is_empty) have hardcoded type signatures in `ash-typeck/src/type_env.rs:add_builtin_functions()`. The remaining ~9 (keys, values, is_int, is_string, is_bool, is_list, is_record, is_null, record) have zero type-system visibility at all. The real problem in both cases is that type signatures are hardcoded in Rust rather than declared in `.ash` files where they belong.

The regex case exposed the problem concretely. An attempt to declare `regex.ash` with:

```ash
pub fn find(...) { act execute Regex.find with ... }
```

failed three ways:

1. **Violates `fn` purity** (SPEC-002, SPEC-031, DESIGN-020): `fn` bodies must not contain `act`.
2. **Doesn't parse**: `act` is not a valid expression form inside `fn` bodies.
3. **Overclaims effect level**: Regex operations are pure computations on strings, not Operational effects.

What is needed is a declaration form that says: "this function exists, it is pure, and its implementation is provided by the runtime -- not by an Ash expression body and not by a capability provider."

Additionally, a related but distinct gap will appear when Ash gains FFI support: functions whose implementation comes from a loaded foreign library (C, WASM, etc.). While this is not needed now, reserving the keyword avoids future grammar conflicts.

---

## 2. Proposed Solution

Introduce two new declaration forms:

1. **`builtin fn`** -- Pure functions whose implementation is compiled into the Ash binary. This is the immediate solution for the gap described above.

2. **`extern fn`** -- Reserved keyword for future FFI. Functions whose implementation will be resolved at link time of an Ash library against a foreign artifact (C shared object, WASM module, etc.). The mechanism does not exist yet. When implemented, a separate design note will specify: the link-time resolution protocol, ABI boundary constraints, and effect classification rules for foreign code.

---

## 3. `builtin fn`

### 3.1 Syntax

```
[pub] builtin fn name(params) -> Type;
```

- Semicolon-terminated, no body, no braces.
- `builtin` keyword placed before `fn`.
- Optional `pub` visibility modifier (module-private by default).
- Declared in `.ash` library files within their appropriate namespaces.
- Full type signature required (params and return type).

Grammar addition (informal):

```
fn_decl ::= visibility? 'builtin' 'fn' IDENT '(' param_list ')' '->' type_expr ';'
visibility ::= 'pub'
```

### 3.2 Semantics

- **No body**: The function has no Ash-level implementation. The runtime (evaluator) recognizes the fully qualified name (e.g., `string::concat`) and dispatches to the corresponding Rust implementation.
- **Pure**: No effect classification. `builtin fn` is subject to the same purity constraints as `pub fn` from the caller's perspective -- it can be called from any `fn` or `workflow` context.
- **No capability provider needed**: The function does not participate in the capability system. It requires no registration, no provider, and no policy gating.
- **Typechecked by signature**: The typechecker uses the declared parameter types and return type for type checking call sites. The Rust implementation is trusted to conform to this signature.
- **Always available**: Compiled into the Ash binary at build time. Available whenever the declaring module is imported -- no runtime loading step.

### 3.3 Examples

**Strictly monomorphic builtins (declarable now):**

All parameters and return types are concrete -- no type variables.

```ash
-- std/src/string.ash
pub builtin fn concat(a: String, b: String) -> String;
pub builtin fn starts_with(s: String, prefix: String) -> Bool;
pub builtin fn ends_with(s: String, suffix: String) -> Bool;
pub builtin fn is_empty(s: String) -> Bool;

-- std/src/regex.ash
pub builtin fn find(pattern: String, text: String) -> Option<String>;
pub builtin fn matches(pattern: String, text: String) -> Bool;
pub builtin fn replace(pattern: String, replacement: String, text: String) -> String;
```

**Type predicates (require ad-hoc polymorphism):**

These accept any value and return `Bool`. They quantify over a type variable
(`value: a`) and therefore require at least simple ad-hoc polymorphism in the
typechecker. They are NOT monomorphic and cannot be declared until the type
system handles universally-quantified builtin parameters.

```ash
-- Future: std/src/type.ash or prelude (requires ad-hoc polymorphism)
pub builtin fn is_int<a>(value: a) -> Bool;
pub builtin fn is_string<a>(value: a) -> Bool;
pub builtin fn is_bool<a>(value: a) -> Bool;
pub builtin fn is_list<a>(value: a) -> Bool;
pub builtin fn is_record<a>(value: a) -> Bool;
pub builtin fn is_null<a>(value: a) -> Bool;
```

**Parametric polymorphic builtins (deferred -- need full generic semantics):**

List operations like `len`, `head`, `tail`, `append`, `concat`, `filter`, `map` require type-parameterized signatures (e.g., `List<a> -> Int`). These cannot be declared until the type system supports generic builtin signatures. Until then, they remain hardcoded in Rust as today.

```ash
-- Future: std/src/list.ash (requires full generic support)
pub builtin fn len<a>(lst: List<a>) -> Int;
pub builtin fn head<a>(lst: List<a>) -> a;
pub builtin fn tail<a>(lst: List<a>) -> List<a>;
pub builtin fn append<a>(lst: List<a>, item: a) -> List<a>;
pub builtin fn concat<a>(left: List<a>, right: List<a>) -> List<a>;
```

Similarly, `keys`, `values`, and `record` require record-type generics and are deferred.

### 3.4 AST and Lowering

The surface `Definition` enum is in `crates/ash-parser/src/surface.rs`. A new `BuiltinFn(BuiltinFnDef)` variant must be added there. During parsing, `builtin fn` produces a `BuiltinFnDef` in the surface layer (`ash-parser`). The surface-to-IR lowering pass (`ash-core`) converts this to a bodyless IR node (e.g., `CoreBuiltinFnDef`) carrying the name, parameter list, and return type.

### 3.5 Module-Loader Dispatch

Stdlib module exports are discovered in `collect_module_exports` (`ash-engine/src/module_loader.rs`) via `extract_semicolon_snippets` (for semicolon-terminated declarations like `pub type`) and `extract_braced_snippets` (for braced declarations like `pub fn`, `workflow`). Since `builtin fn` is semicolon-terminated, it would be discovered via `extract_semicolon_snippets` with a predicate matching `pub builtin fn `. The parsed signature is recorded in module exports for typechecking; no body is extracted.

### 3.6 InlineCallable Representation

`InlineCallable` in `ash-engine/src/module_loader.rs` currently has `body: Expr` (required). A builtin has no body. Preferred fix: introduce a `CallableKind` enum with `Inline { body: Expr }` and `Builtin` variants, replacing the `body` field. This avoids sprinkling `unwrap()`/`expect()` throughout the codebase and makes the dispatch path explicit.

### 3.7 Runtime Dispatch

When the evaluator encounters a call to a `builtin fn`:

1. Resolve the fully qualified name (e.g., `std::string::concat`).
2. Check that the callee is declared `builtin fn` (not `pub fn` with a body).
3. Look up the Rust implementation in the builtin dispatch table (a static mapping from qualified names to Rust functions, compiled into the binary).
4. Evaluate arguments, invoke the Rust function, return the result.

The dispatch table is maintained in Rust code and built at compile time. Adding a new `builtin fn` requires:
- Declaring it in an `.ash` file (for type system visibility).
- Registering the implementation in the dispatch table (in Rust).

### 3.8 Lifecycle

```
.ash declaration (type signature)
        |
        v
  Parser (ash-parser) emits BuiltinFnDef in surface::Definition
        |
        v
  Lowering (ash-core) converts to IR node (no body)
        |
        v
  Typechecker records signature in module environment
        |
        v
  At call site: typecheck against declared signature
        |
        v
  Evaluator dispatches to Rust builtin table by qualified name
```

No separate registration step. No provider lookup. No capability boundary crossing.

---

## 4. `extern fn` (Reserved)

### 4.1 Syntax (Future)

```
[pub] extern "abi" fn name(params) -> Type;
```

- Reserved keyword: `extern`.
- String literal `"abi"` specifies the calling convention (e.g., `"c"`, `"wasm"`, `"ash-abi"`).
- Same semicolon-terminated, no-body form as `builtin fn`.
- **NOT implemented in this design note.**

### 4.2 Distinction from `builtin fn`

| Property | `builtin fn` | `extern fn` (future) |
|----------|-------------|---------------------|
| Implementation source | Compiled into Ash binary | Loaded foreign library |
| Availability | Always (when module imported) | Requires library at link time |
| Dispatch | Static table in evaluator | Dynamic FFI resolution |
| Purity | Pure | TBD (depends on FFI semantics) |

Reserving the keyword now avoids grammar conflicts later. When `extern fn` is implemented, a separate design note will specify its semantics.

---

## 5. Classification: Where Things Live

### 5.1 Three-Way (Eventually Four-Way) Classification

| Declaration | Implementation | Effect | Example |
|-------------|---------------|--------|---------|
| `pub fn` | Ash expression body | Pure (no effect) | `io::path::join`, `test::assert_true` |
| `builtin fn` | Rust runtime, compiled in | Pure (no effect) | `string::concat`, `regex::find` |
| capability + `act` | Rust capability provider | Effect from lattice | `fs::write_file`, `stdio::println` |
| `extern fn` (future) | Foreign library | TBD | C/WASM FFI |

### 5.2 What Stays as Capability Providers

Side-effecting operations that require policy gating remain as capabilities:

- `StdioProvider` (print, read_line) -- Operational, host I/O
- `FsProvider` (read_file, write_file) -- Operational, filesystem access
- `LlmProvider` (chat, embed) -- Operational, network access
- `RuntimeArgProvider` (CLI args) -- Epistemic, host observation
- `McpProvider` (tool calls) -- Deliberative, external tool access

> **Note:** This is the intended classification, not an audited inventory of current engine wiring.

### 5.3 What Converts from Capability to Builtin

**RegexProvider is DELETED.** Regex operations are pure string computations and become `builtin fn` declarations in `std/src/regex.ash`. There is no capability provider, no `act` dispatch, and no effect classification for regex.

> **Current-carrier vs intended-semantics note:** `RegexProvider` currently
> reports `Operational` effect level (`providers/regex.rs:94-96`). This is an
> artifact of the capability-provider implementation carrier, not a statement
> about regex's semantic classification. Regex operations are pure computations
> on strings -- they have no side effects, no external dependencies, and no
> policy gating requirement. The `Operational` classification was inherited from
> the provider framework, not derived from the operation's nature. Migration to
> `builtin fn` aligns the implementation with the correct semantic classification.

### 5.4 What Converts from Magic to Builtin

All currently hardcoded evaluator builtins get `.ash` declarations:

|| Module | Functions | Timing |
||--------|-----------|--------|
|| `std/src/string.ash` (new) | `concat`, `starts_with`, `ends_with`, `is_empty` | Strictly monomorphic -- declarable now |
|| `std/src/regex.ash` (new) | `find`, `matches`, `replace` | Strictly monomorphic -- declarable now |
|| type predicates / prelude | `is_int`, `is_string`, `is_bool`, `is_list`, `is_record`, `is_null` | **Requires ad-hoc polymorphism** -- NOT monomorphic |
|| list / prelude | `len`, `head`, `tail`, `append`, `concat`, `filter`, `map` | **Deferred** -- requires full generic support |
|| record operations / prelude | `keys`, `values`, `record` | **Deferred** -- requires record-type generics |

---

## 6. Relationship to DESIGN-020 (Three-Vertex Model)

This design note is consistent with and reinforces the three-vertex model:

```
         Transform (pure)
         fn / builtin fn -- deterministic, effect-free
        / \
       /   \
      /     \
Orchestrate  Effect (capability)
workflow     observe/execute
```

`builtin fn` sits alongside `pub fn` at the Transform vertex. Both are pure. Both can be called from `fn` bodies and `workflow` bodies. Neither can call capabilities or invoke workflows. The only difference is where the implementation lives: `pub fn` has an Ash-expression body; `builtin fn` has a Rust implementation in the binary.

This preserves the composition rules from DESIGN-020 D1:

- `fn` -> `fn` (freely composes)
- `fn` -> `builtin fn` (freely composes -- both are pure transforms)
- `workflow` -> `fn` (workflow calls fn for data transforms)
- `workflow` -> `builtin fn` (same -- pure data transforms)
- `workflow` -> `cap` (workflow uses capabilities for effects)
- `fn` -X-> `workflow` (functions never invoke workflows)
- `fn` -X-> `cap` (functions never use capabilities)

---

## 7. Scope

### In Scope

- `builtin fn` grammar, surface AST node (`ash-parser`), lowering to IR (`ash-core`), module loading, typechecking, and runtime dispatch.
- `InlineCallable` representation change to accommodate bodyless builtins.
- Migration of regex from capability provider to `builtin fn`.
- Migration of monomorphic magic evaluator builtins to declared `builtin fn` in `.ash` files.
- Reservation of `extern` keyword for future FFI.

### Out of Scope

- `extern fn` implementation (future design note).
- New capabilities or capability system changes.
- Changes to `pub fn` or `act` semantics.
- Polymorphic builtin declarations (`List<a>`, record generics) -- genuinely deferred until the type system supports generic builtin signatures.

---

## 8. Open Questions

1. **Polymorphic builtins**: Functions like `len`, `head`, `tail`, `map` are generic over list element type. These are genuinely deferred until generic semantics are in place, not just an open question. The current hardcoded type-variable bindings in `add_builtin_functions()` will serve until then.

2. **Error model**: Some builtins can fail at runtime (e.g., `head` on empty list). Should they return `Option<T>` / `Result<T, E>`, or panic? The declared signature determines this, but the convention needs to be established.

3. **Module-private builtins**: Should `builtin fn` without `pub` be supported? The runtime dispatch is global, so private visibility is purely a type-checking concern. Probably yes for consistency with `pub fn`.

4. **Overloading**: Can a module declare both `pub fn foo(...)` and `builtin fn foo(...)`? No -- the qualified name must be unique within a module.

---

## 9. Decision

Adopt `builtin fn` as a declaration form for pure runtime-provided functions, and reserve `extern` as a keyword for future FFI support. This closes the gap between `pub fn` (user-written bodies) and capability providers (effectful operations), giving pure runtime functions proper type system visibility without violating the three-vertex model's purity constraints.

This unblocks:
- Proper `.ash` declarations for string and regex builtins (strictly monomorphic, immediate).
- Deletion of RegexProvider (pure operations should not be capability-gated).
- Type system coverage for strictly monomorphic functions callable from Ash code.
- A clean extension path for type predicates (ad-hoc polymorphic) once simple generic builtin semantics land.
- A clean extension path for fully polymorphic builtins (list/record ops) once generics land.
- Future FFI via `extern fn`.
