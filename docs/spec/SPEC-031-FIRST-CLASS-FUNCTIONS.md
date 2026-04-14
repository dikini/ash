# SPEC-031: First-Class Functions and Closure Values

**Status:** Draft
**Date:** 2026-04-14
**Version:** 0.3

## 1. Overview

Add first-class function values to Ash. Local function definitions become expressions that produce closure values. Named local functions desugar to let-bindings. The interpreter (`ash-interp`) handles function definition and application natively, eliminating the `pure_runtime` workaround.

## 2. Motivation

The current architecture has a split: `ash-interp` operates on lowered IR that has no function concept, while `pure_runtime.rs` is a duplicate interpreter that exists solely to handle functions. This is tech debt:

- Two expression evaluation implementations that must stay in sync
- Inlining hack only works for non-recursive, non-higher-order cases
- `should_execute_via_pure_runtime` dispatch is a code smell
- Functions cannot be passed as arguments, returned from calls, or stored in data structures

The fix: treat functions as first-class values in the core IR and interpreter.

## 3. Definitions and Scope

### 3.1 Two Kinds of Function Definitions

Ash has two distinct contexts for function definitions with different semantics:

**Module-level functions** (`pub fn`): file top-level in stdlib modules. These are module items (`Definition::Function`). They are collected by `collect_module_exports` for cross-module import. They are **never** reified as `Value::Closure` at runtime. When imported and called, they are inlined into the caller's IR during loading (current behavior). This spec does not change their representation or execution model.

**Local functions** (inside workflows and blocks): appear inside `workflow { ... }` or other function bodies. These are expressions. They produce `Value::Closure` at runtime. This spec defines their semantics.

**Reification rule:** Only local functions produce closure values. Module-level functions remain as module exports. There is no mechanism to convert a module-level function into a `Value::Closure`. This keeps the serialization story simple: `Value::Closure` is always local and non-serializable.

### 3.2 Partial Application: Excluded

An implementation conforming to SPEC-031 does **not** support partial application. `f(1)` where `f` takes two arguments is an arity error. Partial application may be addressed in a future spec.

## 4. Semantics

### 4.1 Anonymous Function Expression

```
fn(a, b) { a + b }
```

An expression that evaluates to a closure value capturing its lexical environment. Only valid inside workflow bodies and other function bodies (local context), not at module top-level.

### 4.2 Named Local Function Desugaring

```
fn add(a: Int, b: Int) -> Int { a + b }
```

Inside a workflow or block, desugars to:

```
let add = fn(a: Int, b: Int) -> Int { a + b }
```

The name is in lexical scope for subsequent expressions in the same block.

### 4.3 Closure Syntax (Sugar)

```
|x, y| => x + y
```

Desugars to `fn(x, y) { x + y }`.

### 4.4 Function Application

```
add(1, 2)
```

When `add` resolves to a closure value, the application extends the closure's captured environment with parameter bindings and evaluates the body.

### 4.5 Higher-Order Functions

```
fn apply(f, x) { f(x) }
fn double(n) { n * 2 }
apply(double, 5)   -- evaluates to 10
```

### 4.6 Recursion via Late Binding

```
fn factorial(n) {
    if n <= 1 then 1 else n * factorial(n - 1)
}
```

Desugars to `let factorial = fn(n) { ... }`. Recursion works through late binding: the closure does not statically capture its own value. Instead, the environment uses a `BindingSlot` that is filled after construction (see §5.3). At call time, the closure resolves `factorial` from the shared environment, finding its own binding.

### 4.7 Scope and Capture

```
fn make_adder(n) {
    fn(x) { n + x }
}
let add5 = make_adder(5);
add5(3)   -- evaluates to 8
```

The inner closure captures `n` from the outer function's scope via shared environment frame.

### 4.8 Three-Vertex Boundary

Closures respect the three-vertex model:

- **Closures defined inside `fn` context** have type `Type::Fn(params, ret)` (pure). They can only capture pure values.
- **Closures defined inside `workflow` context** have type `Type::Fun(params, ret, effect)` where `effect >= Epistemic`. The type checker prevents passing these into pure `fn` parameters that expect `Type::Fn`.
- This uses the **existing** `Type::Fn` / `Type::Fun` split in `ash_typeck` (see §6).

## 5. IR Changes

### 5.1 New Expr Variants (ash-core)

```rust
/// ash-core/src/ast.rs -- Expr enum additions

/// Anonymous function definition (closure creation).
/// Only valid in local context, not module top-level.
FnDef {
    params: Vec<(Name, Option<Type>)>,
    return_type: Option<Type>,
    body: Box<Expr>,
},

/// Function application.
/// `func` evaluates to a closure; args are bound; body is evaluated.
/// Distinct from Expr::Call which handles built-in functions by name.
FnApply {
    func: Box<Expr>,
    args: Vec<Expr>,
},
```

### 5.2 New Value Variant (ash-core)

```rust
/// ash-core/src/value.rs -- Value enum addition
///
/// Runtime closure value. Captures a shared reference to the lexical
/// environment at definition time.
///
/// ## Serialization
///
/// NOT serializable. Closures are local-only values. Module-level functions
/// are never reified as Value::Closure (see §3.1 reification rule).
/// Serde implementation: use #[serde(skip)] or manual impl that panics.
///
/// ## Send + Sync
///
/// Naturally Send + Sync because all fields are Send + Sync:
/// - Vec<(String, Option<String>)>: Send + Sync
/// - Box<Expr>: Send + Sync (all Expr variants contain only Send+Sync data)
/// - Arc<EnvFrame>: Send + Sync (EnvFrame contains only Arc, String, Value)
/// No unsafe impl needed.
Closure {
    params: Vec<(String, Option<String>)>,
    body: Box<ash_core::ast::Expr>,
    env: std::sync::Arc<EnvFrame>,
},
```

### 5.3 Environment Frame and Binding Slot (ash-core)

```rust
/// ash-core/src/env_frame.rs -- new file
///
/// Shared environment frame for closure capture.
/// Forms a parent chain via Arc -- O(1) capture, no flattening.
/// All fields are Send + Sync by construction. No unsafe needed.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// A slot in the environment. Supports late binding for recursive closures.
///
/// Normal bindings: `BindingSlot::Bound(value)` -- immutable after creation.
/// Recursive bindings: `BindingSlot::Late(cell)` -- filled after closure construction.
///
/// This is the single representation for all bindings. No separate `Value` vs
/// `LateBinding` ambiguity -- every binding is a `BindingSlot`.
#[derive(Debug, Clone)]
pub enum BindingSlot {
    /// Normal immutable binding.
    Bound(Value),
    /// Late binding for recursive let. The Mutex<Option<Value>> is filled
    /// exactly once after the closure is constructed. Naturally Send+Sync
    /// because Mutex<T> is Send+Sync when T: Send.
    Late(Arc<Mutex<Option<Value>>>),
}

impl BindingSlot {
    pub fn new_late() -> Self {
        Self::Late(Arc::new(Mutex::new(None)))
    }

    pub fn resolve(&self) -> Option<Value> {
        match self {
            BindingSlot::Bound(v) => Some(v.clone()),
            BindingSlot::Late(cell) => cell.lock().unwrap().clone(),
        }
    }

    pub fn set_late(&self, value: Value) {
        if let BindingSlot::Late(cell) = self {
            *cell.lock().unwrap() = Some(value);
        }
    }
}

#[derive(Debug, Clone)]
pub struct EnvFrame {
    /// Bindings in this scope level. Every binding is a BindingSlot.
    bindings: HashMap<String, BindingSlot>,
    /// Parent scope (shared via Arc). None for the root frame.
    parent: Option<Arc<EnvFrame>>,
}
```

**Why this fixes blocker 1:** There is one representation (`BindingSlot`) that handles both normal and recursive bindings. No sum-type mismatch, no `Value::LateBinding`, no inconsistency.

**Why this fixes blocker 5:** All types are naturally `Send + Sync`:
- `BindingSlot::Bound(Value)` -- `Value: Send + Sync`
- `BindingSlot::Late(Arc<Mutex<Option<Value>>>)` -- `Mutex<T>: Send+Sync` when `T: Send`
- `EnvFrame` contains only `HashMap<String, BindingSlot>` and `Option<Arc<EnvFrame>>` -- all `Send + Sync`
- No `unsafe impl` needed.

### 5.4 Call vs FnApply: Unambiguous Lowering

**Current state:** `Expr::Call { func: Name, module, args }` is used for all calls. The typechecker's `lookup_call_target` resolves it. The interpreter's `eval_function_call` handles built-ins by string dispatch.

**After this spec:**

The lowering step (`crates/ash-parser/src/lower.rs`) produces:
- `CoreExpr::Call { func, arguments }` for built-in functions (string dispatch in interpreter)
- `CoreExpr::FnApply { func, args }` for user-defined functions and closures

**Blast radius for this change:**

| Component | Current contract | Required change |
|-----------|-----------------|-----------------|
| `ash-parser/src/lower.rs` | All calls -> `Expr::Call` | Add `lower_fn_apply`, built-in registry |
| `ash-core/src/ast.rs` | `Expr::Call` only | Add `Expr::FnApply` variant |
| `ash-interp/src/eval.rs` | `Expr::Call` handles everything | Add `Expr::FnApply` handler; `Expr::Call` stays for built-ins only |
| `ash-interp/src/error.rs` | `UnknownFunction` for missing built-ins | Add `NotCallable` for FnApply on non-closure |
| `ash-typeck/src/check_expr.rs` | `Expr::Call` typed via `lookup_call_target` | Add `Expr::FnApply` typing; existing `lookup_call_target` handles `Type::Fn`/`Type::Fun` already |
| `ash-engine/src/lib.rs` | Inlines imported callables into Call nodes | Inlines into FnApply nodes instead |

The typechecker already handles `Type::Fn` and `Type::Fun` in `check_expr.rs:193`. Adding `Expr::FnApply` typing reuses this path: evaluate `func` type, check it's `Type::Fn` or `Type::Fun`, unify arg types.

## 6. Type System Integration

### 6.1 Use Existing Types

The typechecker already has the split we need:

```rust
// crates/ash-typeck/src/types.rs (existing)
pub enum Type {
    /// Pure function type: arguments, return type, no effect
    Fn(Vec<Type>, Box<Type>),
    /// Effectful function type: arguments, return type, effect
    Fun(Vec<Type>, Box<Type>, Effect),
    ...
}
```

The surface parser already parses `Fn(T, U) -> V` syntax:

```rust
// crates/ash-parser/src/surface.rs (existing)
pub enum Type {
    Fn(Vec<Type>, Box<Type>),  // Fn(Int, Int) -> Int
    ...
}
```

**SPEC-031 uses these existing types. No new type variants.**

### 6.2 Typing Rules

| Expression | Type |
|-----------|------|
| `fn(x: Int) -> Int { x + 1 }` (in fn context) | `Type::Fn([Int], Int)` |
| `fn(x: Int) -> Int { act ... }` (in workflow context) | `Type::Fun([Int], Int, Operational)` |
| `f(args)` where `f: Type::Fn` or `Type::Fun` | Return type of `f` |
| `Expr::FnApply { func, args }` | Type the func, unify args, return ret type |

### 6.3 Three-Vertex Enforcement

Existing `Type::Fn` (pure) vs `Type::Fun` (effectful) split enforces the boundary:

- A pure function parameter typed `Type::Fn(...)` rejects a `Type::Fun(...)` closure
- This is already how the type system distinguishes pure vs effectful callables
- No new effect marker or third type shape needed

### 6.4 Type Checker Changes

`crates/ash-typeck/src/check_expr.rs` needs a new arm:

```rust
Expr::FnDef { params, return_type, body } => {
    // Type-check body in extended environment
    // Result type: Type::Fn(param_types, ret_type) or Type::Fun(...) if in workflow context
}

Expr::FnApply { func, args } => {
    // Type func, check it's Type::Fn or Type::Fun
    // Unify arg types with param types
    // Return ret type
}
```

The existing `Expr::Call` handler (`check_expr.rs:157-230`) is unchanged for built-ins.

## 7. Serialization Policy

**Rule: `Value::Closure` is never serialized.**

Implementation:
- `Value` currently derives `Serialize, Deserialize`. After adding `Closure`, implement custom serde that:
  - Serializes `Closure` variant as `{ "_type": "closure", "_note": "non-serializable" }` (stub for diagnostics)
  - Deserialization of the stub produces an error at runtime
- Module-level functions are never `Value::Closure` (reification rule §3.1), so they are unaffected.

## 8. Parser Design for Local-Only fn Expressions

### 8.1 Current State

- `parse_fn_definition` in `parse_module.rs:1073-1129` parses top-level `fn` / `pub fn`
- Nested `fn` is rejected: `parse_fn_rejects_nested_fn` test at `fn_parser_tests.rs:253-262`
- The expression parser (`parse_expr.rs:18`) has no context parameter -- `ParseInput` is `Stateful<LocatingSlice<&str>, Position>` with no parse-state field

### 8.2 Strategy: Post-Parse Validation (Not Context-Threading)

Rather than threading context through the parser (which would require changing `ParseInput` and every parser function), use a two-phase approach:

**Phase 1: Parse permissively**
- Extend `parse_expr` to recognize `fn(params) [-> type] { body }` as an expression
- This produces `Expr::FnDef { ... }` in the surface AST regardless of context
- The parser does NOT reject nested fn during parsing

**Phase 2: Validate during lowering/type-checking**
- During `lower_expr`, when encountering `Expr::FnDef`, check that it appears in a valid context (inside a workflow body or another FnDef body)
- If at module top-level, emit a lowering error: "fn expressions are not valid at module scope; use `pub fn` instead"
- The type checker performs the same validation as a second pass

**Why this approach:**
- Does not require changing `ParseInput` or adding context to `expr()`
- Reuses the existing parser infrastructure
- The test `parse_fn_rejects_nested_fn` changes to expect success at parse time, with rejection happening at lower/typeck time
- Matches the existing pattern where structural validation happens in lowering (e.g., `InterfaceMethodCall` is parsed but rejected during lowering at `lower.rs:1181`)

### 8.3 Closure Syntax

`|params| => expr` is parsed as a new expression form in `parse_expr.rs`. It immediately desugars to `Expr::FnDef { params, body }` during parsing (no new surface AST node needed).

## 9. Lowering Changes

### 9.1 Surface to Core

The lowering step (`crates/ash-parser/src/lower.rs`) translates:

- Surface `Expr::FnDef` -> `CoreExpr::FnDef { params, return_type, body: lower_expr(body) }`
- Surface `Expr::Call` to user-defined name -> `CoreExpr::FnApply { func: Variable(name), args }`
- Surface `Expr::Call` to known built-in -> `CoreExpr::Call { func, arguments }` (unchanged)

### 9.2 Built-in Registry

```rust
/// crates/ash-parser/src/lower.rs
///
/// Built-in function names that the interpreter handles via string dispatch.
/// Calls to these names emit Expr::Call; all other calls emit Expr::FnApply.
pub const BUILTIN_FUNCTIONS: &[&str] = &[
    "len", "append", "concat", "prepend",
    "string::length", "string::trim", "string::to_uppercase",
    "string::contains", "string::starts_with", "string::ends_with",
    "string::find", "string::slice", "string::replace",
    // ... complete list from eval_function_call in eval.rs
];
```

### 9.3 Cross-Crate Contract Changes

| Crate | Change |
|-------|--------|
| `ash-parser` | `lower_expr` produces `FnApply` for user calls; `lower_fn_def` for FnDef bodies |
| `ash-core` | `Expr::FnDef`, `Expr::FnApply` added to `ast.rs`; `EnvFrame`, `BindingSlot` in new `env_frame.rs` |
| `ash-typeck` | `check_expr` gains `FnDef` and `FnApply` arms using existing `Type::Fn`/`Type::Fun` |
| `ash-interp` | `eval_expr` gains `FnDef` and `FnApply` eval cases |
| `ash-engine` | `inline_imported_calls_in_workflow_def` produces `FnApply` nodes; `pure_runtime` deleted |

## 10. Interpreter Changes

### 10.1 eval_expr Additions

```rust
// ash-interp/src/eval.rs

Expr::FnDef { params, body, .. } => {
    let env_frame = ctx.to_env_frame();
    Ok(Value::Closure {
        params: params.iter()
            .map(|(n, t)| (n.clone(), t.as_ref().map(|s| s.to_string())))
            .collect(),
        body: Box::new(body.as_ref().clone()),
        env: env_frame,
    })
}

Expr::FnApply { func, args } => {
    let callee = eval_expr(func, ctx)?;
    match callee {
        Value::Closure { params, body, env } => {
            let arg_values: Vec<Value> = args.iter()
                .map(|a| eval_expr(a, ctx))
                .collect::<Result<Vec<_>, _>>()?;
            if arg_values.len() != params.len() {
                return Err(EvalError::WrongArity {
                    expected: params.len(),
                    actual: arg_values.len(),
                });
            }
            let call_env = Context::from_env_frame(&env);
            for ((name, _), value) in params.iter().zip(arg_values.into_iter()) {
                call_env.set(name.clone(), value);
            }
            eval_expr(&body, &call_env)
        }
        _ => Err(EvalError::NotCallable { value: callee }),
    }
}
```

### 10.2 Context Extensions

```rust
impl Context {
    /// Snapshot current scope chain as an EnvFrame (shared via Arc).
    pub fn to_env_frame(&self) -> Arc<EnvFrame> { ... }

    /// Create a Context from a captured EnvFrame.
    pub fn from_env_frame(frame: &Arc<EnvFrame>) -> Self { ... }
}
```

### 10.3 Recursive Let Evaluation

When the interpreter evaluates `Let { pattern: "factorial", expr: FnDef { ... } }`:

1. Create a `BindingSlot::Late(cell)` for "factorial" in the current scope
2. Evaluate the `FnDef` expression, producing `Value::Closure { env: frame_with_late_binding, ... }`
3. Fill the late binding: `cell.set(closure.clone())`
4. The closure now sees "factorial" in its captured environment

### 10.4 Expr::Call: No Closure Fallback

After migration, `Expr::Call` handles built-in functions only. No closure lookup fallback. If a user shadow a built-in name, the lowering step produces `FnApply` (user-defined name wins over built-in registry).

During migration (Phase B), a temporary fallback in `Expr::Call` handles the transition.

## 11. Removal of pure_runtime.rs

1. Delete `crates/ash-engine/src/pure_runtime.rs` (476 lines)
2. Remove `should_execute_via_pure_runtime` from `crates/ash-engine/src/lib.rs`
3. Remove `inline_imported_calls_in_workflow_def`
4. Remove `collect_local_inline_callables`
5. All execution goes through `interpret_in_state`
6. `parse_program_with_functions` removed

## 12. Migration Path

### Phase A: Core IR + Interpreter (non-breaking)
1. Add `Expr::FnDef`, `Expr::FnApply` to `ash-core/src/ast.rs`
2. Add `Value::Closure` to `ash-core/src/value.rs` (custom serde: skip)
3. Add `EnvFrame`, `BindingSlot` to `ash-core/src/env_frame.rs`
4. Add `Send + Sync` compile-time assertions
5. Add eval cases to `ash-interp/src/eval.rs`
6. Add `Context::to_env_frame`, `Context::from_env_frame`
7. Existing tests pass -- new variants not produced yet

### Phase B: Lowering + Type Checker (non-breaking)
1. Add built-in registry to `crates/ash-parser/src/lower.rs`
2. Add `lower_fn_def`, update `lower_expr` for `FnApply`
3. Add `FnDef`/`FnApply` arms to `ash-typeck/src/check_expr.rs` using existing `Type::Fn`/`Type::Fun`
4. Temporary `Expr::Call` closure fallback in interpreter
5. Keep `pure_runtime` active during transition

### Phase C: Delete pure_runtime (breaking cleanup)
1. Delete `pure_runtime.rs`
2. Remove dispatch/inlining from `lib.rs`
3. Remove `Expr::Call` closure fallback
4. Verify all tests through single interpreter path

### Phase D: Parser Expression Forms
1. Parse `fn(params) { body }` as expression in `parse_expr`
2. Parse `|params| => body` closure syntax
3. Post-parse validation: reject `FnDef` at module scope during lowering
4. Update `parse_fn_rejects_nested_fn` test to expect parse-success + lower-reject

### Phase E: Effect Typing for Workflow Closures
1. Type-check FnDef in workflow context as `Type::Fun(..., effect)` 
2. Reject `Type::Fun` where `Type::Fn` expected (three-vertex enforcement)
3. This uses existing `Type::Fun` and `Effect` -- no new types

## 13. Conformance

### 13.1 Minimal Conformance

An implementation must:
- Evaluate `Value::Closure` from `Expr::FnDef` with shared `Arc<EnvFrame>` capture
- Apply closures via `Expr::FnApply` with correct parameter binding
- Support recursion via `BindingSlot::Late`
- Support higher-order functions
- Use existing `Type::Fn` / `Type::Fun` for closure typing
- Distinguish `Expr::Call` (built-ins) from `Expr::FnApply` (user functions)
- Ensure `Value::Closure` is `Send + Sync` without `unsafe`
- NOT serialize `Value::Closure`
- Module-level functions are never `Value::Closure`
- Pass all existing tests

### 13.2 Full Conformance

Additionally supports:
- Anonymous function expressions in parser
- Closure syntax `|x| => ...`
- Post-parse validation of fn-in-local-context
- Three-vertex enforcement via `Type::Fn` vs `Type::Fun`
- Tail-call optimization (future)

## 14. Files Affected

| File | Change |
|------|--------|
| `crates/ash-core/src/ast.rs` | Add `Expr::FnDef`, `Expr::FnApply` |
| `crates/ash-core/src/value.rs` | Add `Value::Closure` (custom serde) |
| `crates/ash-core/src/env_frame.rs` | NEW: `EnvFrame`, `BindingSlot` |
| `crates/ash-core/src/lib.rs` | Export `env_frame` module |
| `crates/ash-interp/src/context.rs` | Add `to_env_frame`, `from_env_frame` |
| `crates/ash-interp/src/eval.rs` | Add `FnDef`/`FnApply` eval cases |
| `crates/ash-interp/src/error.rs` | Add `NotCallable` variant |
| `crates/ash-typeck/src/check_expr.rs` | Add `FnDef`/`FnApply` arms using existing types |
| `crates/ash-typeck/src/types.rs` | No changes (use existing `Fn`/`Fun`) |
| `crates/ash-parser/src/lower.rs` | Add built-in registry, `lower_fn_def`, `FnApply` lowering |
| `crates/ash-parser/src/parse_expr.rs` | Add fn expression, closure syntax (Phase D) |
| `crates/ash-engine/src/lib.rs` | Remove pure_runtime dispatch (Phase C) |
| `crates/ash-engine/src/pure_runtime.rs` | DELETE (Phase C) |
