# SPEC-031: First-Class Functions and Closure Values

**Status:** Draft
**Date:** 2026-04-14
**Version:** 0.2

## 1. Overview

Add first-class function values to Ash. Function definitions become expressions that produce closure values. Named local functions desugar to let-bindings. The interpreter (`ash-interp`) handles function definition and application natively, eliminating the `pure_runtime` workaround.

## 2. Motivation

The current architecture has a split: `ash-interp` operates on lowered IR that has no function concept, while `pure_runtime.rs` is a duplicate interpreter that exists solely to handle functions. This is tech debt:

- Two expression evaluation implementations that must stay in sync
- Inlining hack only works for non-recursive, non-higher-order cases
- `should_execute_via_pure_runtime` dispatch is a code smell
- Functions cannot be passed as arguments, returned from calls, or stored in data structures

The fix: treat functions as first-class values in the core IR and interpreter.

## 3. Definitions and Scope

### 3.1 Two Kinds of Function Definitions

Ash has two distinct contexts for function definitions, with different semantics:

**Module-level functions** (`pub fn`): appear at file top-level in stdlib modules (`std/src/llm/prompt.ash`, etc.). These are module items, not expressions. They remain as `Definition::Function` in the module's symbol table. They are collected by `collect_module_exports` for cross-module import. This spec does NOT change their representation.

**Local functions** (inside workflows and blocks): appear inside `workflow { ... }` or inside other function bodies. These are expressions. This spec defines their semantics: they desugar to `let name = fn(params) { body }` and produce closure values at runtime.

### 3.2 Partial Application: Excluded

An implementation conforming to SPEC-031 does **not** support partial application. `f(1)` where `f` takes two arguments is an arity error, not a closure. This keeps the evaluation model simple. Partial application may be addressed in a future spec.

## 4. Semantics

### 4.1 Anonymous Function Expression

```
fn(a, b) { a + b }
```

This is an expression that evaluates to a closure value capturing its lexical environment. Only valid inside workflow bodies and other function bodies (local context), not at module top-level.

### 4.2 Named Local Function Desugaring

```
fn add(a: Int, b: Int) -> Int { a + b }
```

Inside a workflow or block, desugars to:

```
let add = fn(a: Int, b: Int) -> Int { a + b }
```

The name is in lexical scope for all subsequent expressions in the same block. This is a local binding only -- it does not create a module-level export.

### 4.3 Closure Syntax (Sugar)

```
|x, y| => x + y
```

Desugars to:

```
fn(x, y) { x + y }
```

### 4.4 Function Application

```
add(1, 2)
```

When `add` resolves to a closure value, the application extends the closure's captured environment with parameter bindings and evaluates the body.

### 4.5 Higher-Order Functions

Functions can be passed as arguments and returned from calls:

```
fn apply(f, x) { f(x) }
fn double(n) { n * 2 }
apply(double, 5)   -- evaluates to 10
```

### 4.6 Recursion via Late Binding

Named functions can call themselves. The mechanism is **late binding** through the call environment, not static capture:

```
fn factorial(n) {
    if n <= 1 then 1 else n * factorial(n - 1)
}
```

This desugars to `let factorial = fn(n) { ... }`. When the closure is applied, the call environment includes the `factorial` binding itself. The interpreter resolves `factorial` dynamically from the call environment at each application, not from a static snapshot taken at definition time.

This avoids infinite nesting in the closure value: the environment does not contain a copy of itself. Instead, the `let` binding is a mutable slot that the closure resolves by name at call time.

**Implementation**: The interpreter evaluates `let factorial = fn(n) { ... }` by:
1. Creating a `Context` with a placeholder binding for `factorial`
2. Constructing the closure, which captures a reference to this context (not a flat copy)
3. Updating the `factorial` binding to point to the constructed closure

Because closures share the environment via `Arc` (see §5.2), updating the slot after construction makes the binding visible to all subsequent lookups, including recursive calls.

### 4.7 Scope and Capture

Closures capture their lexical environment at definition time:

```
fn make_adder(n) {
    fn(x) { n + x }
}
let add5 = make_adder(5);
add5(3)   -- evaluates to 8
```

The inner closure captures `n` from the outer function's scope.

### 4.8 Three-Vertex Boundary

Closures respect the three-vertex model (DESIGN-020):

- **Closures defined inside `fn` context** can only capture pure values (ints, strings, records, lists, other closures). They cannot capture or reference capability contexts, workflow bindings, or `act` results.
- **Closures defined inside `workflow` context** may capture workflow-local bindings, but they are **not first-class** -- they cannot be returned from the workflow, stored in instance state, or passed across process boundaries. If a workflow-local closure escapes, it is a runtime error.
- **The type system** (see §6) enforces this: `fn` closures have type `Fn(T) -> U`, while workflow-captured closures carry an effect marker that prevents them from being used in pure context.

This mirrors the existing `fn -X-> workflow` rule. A closure is tagged with its definition context, and the type checker prevents crossing the boundary.

## 5. IR Changes

### 5.1 New Expr Variants (ash-core)

```rust
/// ash-core/src/ast.rs -- Expr enum additions

/// Anonymous function definition (closure creation).
/// Only valid in local (workflow/block) context, not module top-level.
/// Evaluating this expression produces a Value::Closure.
FnDef {
    params: Vec<(Name, Option<Type>)>,   -- parameter names and optional types
    return_type: Option<Type>,            -- optional return type annotation
    body: Box<Expr>,                      -- function body expression
},

/// Function application.
/// Evaluates `func` to obtain a closure value, binds parameters, evaluates body.
/// Distinct from Expr::Call which handles built-in functions by name.
FnApply {
    func: Box<Expr>,                      -- expression producing a closure value
    args: Vec<Expr>,                      -- argument expressions
},
```

### 5.2 New Value Variant (ash-core)

```rust
/// ash-core/src/value.rs -- Value enum addition
///
/// Runtime closure value. Captures a shared reference to the lexical environment
/// at definition time (Arc<EnvFrame>), not a flat copy.
///
/// ## Serialization
///
/// Value::Closure does NOT implement Serialize/Deserialize. Closures cannot
/// cross process boundaries. If a closure must be referenced across processes,
/// it must be a module-level function referenced by name (not a captured closure).
///
/// ## Send + Sync
///
/// EnvFrame is Arc<EnvFrameData> where EnvFrameData: Send + Sync.
/// This ensures Value::Closure can be held across await points in async workflows.
Closure {
    /// Parameter names (and optional type annotations, for display/debugging)
    params: Vec<(String, Option<String>)>,
    /// The function body expression (boxed to reduce enum size)
    body: Box<ash_core::ast::Expr>,
    /// Shared reference to the captured lexical environment frame.
    /// Uses Arc for O(1) capture and shared parent chains.
    env: std::sync::Arc<EnvFrame>,
},
```

### 5.3 Environment Frame (ash-core)

```rust
/// ash-core/src/env_frame.rs -- new file
///
/// Shared, immutable environment frame for closure capture.
/// Forms a chain: each frame has an optional parent, enabling O(1) capture
/// and shared scope chains without flattening.

use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct EnvFrame {
    /// Bindings in this scope level
    bindings: HashMap<String, Value>,
    /// Parent scope (shared via Arc)
    parent: Option<Arc<EnvFrame>>,
}

// EnvFrame is immutable after construction, so Send + Sync is safe.
// Mutation for recursive let-bindings uses the LateBinding mechanism (§5.4).

unsafe impl Send for EnvFrame {}
unsafe impl Sync for EnvFrame {}
```

### 5.4 Recursive Binding via LateBinding Cell

For recursive functions (`let f = fn(...) { ... f ... }`), the closure needs to see its own binding. Rather than flattening or mutating the EnvFrame, use a late-binding cell:

```rust
/// A mutable cell inside an otherwise-immutable EnvFrame.
/// Used for recursive let-bindings: the slot starts as a placeholder,
/// then gets filled with the closure value after construction.
#[derive(Debug, Clone)]
pub struct LateBinding {
    /// OnceCell-like semantics: can be set exactly once.
    value: std::sync::Arc<std::sync::Mutex<Option<Value>>>,
}

impl LateBinding {
    pub fn new() -> Self { ... }
    pub fn set(&self, value: Value) { ... }
    pub fn get(&self) -> Option<Value> { ... }
}
```

EnvFrame bindings can be either `Value` (normal) or `LateBinding` (recursive). Lookup checks both.

### 5.5 Call vs FnApply: Unambiguous Lowering

The lowering step resolves the ambiguity at compile time:

- Surface calls to **known built-in functions** (`len`, `append`, `concat`, `string::*`, etc.) -> `Expr::Call { func, arguments }`
- Surface calls to **user-defined functions** (local `fn`, imported functions) -> `Expr::FnApply { func: Variable(name), args }`
- If a variable shadows a built-in name, the user-defined function wins: lowering uses `FnApply`

The built-in function registry is available during lowering. The interpreter does NOT need a fallback path from `Call` to closure lookup.

### 5.6 Expr::Call Transition (Backward Compatibility)

During migration (Phase B), existing code that produces `Expr::Call` for user functions will coexist with `FnApply`. The interpreter's `Expr::Call` handler gains a closure-lookup fallback:

1. Check built-in dispatch (`eval_function_call`)
2. If not a built-in, check context for `Value::Closure` -> apply
3. If neither, error: unknown function

After Phase C, all lowering produces `FnApply` for user functions, and the fallback is removed.

## 6. Type System Integration

### 6.1 Function Type Syntax

```
Fn(Int, Int) -> Int
```

This is a first-class type constructor. It appears in:

- Parameter types: `fn apply(f: Fn(Int) -> Int, x: Int) -> Int { f(x) }`
- Return types: `fn make_adder(n: Int) -> Fn(Int) -> Int { fn(x) { n + x } }`
- Type annotations: `let f: Fn(String) -> Bool = |s| => s == "yes"`

### 6.2 Type Representation

```rust
/// In ash_typeck or ash-core type system
Type::Fn {
    params: Vec<Type>,
    return_type: Box<Type>,
}
```

### 6.3 Type Checking Rules

- `Expr::FnDef { params, body }` has type `Fn(T1, ..., Tn) -> R` where `Ti` are the parameter types and `R` is the return type (inferred or annotated).
- `Expr::FnApply { func, args }` type-checks: `func` must have type `Fn(A1, ..., An) -> R`, and each `arg` must have type `Ai`. Result type is `R`.
- Higher-order: `Fn(Fn(T) -> U, T) -> U` is well-formed.

### 6.4 Capture Effect Marker

For three-vertex enforcement (§4.8), closures defined in workflow context carry an effect marker:

```rust
Type::Fn {
    params: Vec<Type>,
    return_type: Box<Type>,
    effects: EffectSet,   -- empty for pure fn, non-empty for workflow-defined
}
```

The type checker rejects passing an effect-annotated closure into a pure `fn` parameter.

This is a **minimal** type system extension. Full effect typing is deferred to a future spec.

## 7. Serialization Policy

`Value::Closure` does **not** implement `Serialize`/`Deserialize`.

Rationale:
- The body is an `Expr` AST node -- serialized AST is brittle across compiler versions
- Captured values may include `Cap`, `Stream`, `InstanceAddr`, `ControlLink` -- runtime-local references
- Closures sent across process boundaries would carry invalid references

Implementation: `Value` currently derives `Serialize, Deserialize` uniformly. After adding `Closure`, use `#[serde(skip_deserializing)]` or implement custom serialization that writes closures as a stub (name + module path) for module-level functions, and errors for captured closures.

If closures need to cross process boundaries in the future, the solution is to reference module-level functions by qualified name (`"llm::prompt::system"`) rather than serializing the closure itself.

## 8. Send + Sync Verification

The interpreter executes workflows asynchronously. `Value::Closure` must be `Send` to move across await points.

Requirements:
- `EnvFrame` uses `Arc<EnvFrameData>` where `EnvFrameData: Send + Sync` (all fields are `String`, `Value`, `Arc`)
- `Expr` must be `Send + Sync`. Current `Expr` variants contain only `String`, `Vec`, `Box`, and `f64` -- all `Send + Sync`. The new `FnDef` and `FnApply` variants follow the same pattern.
- `LateBinding` uses `Arc<Mutex<Option<Value>>>` which is `Send + Sync`.

Verify with a compile-time assertion after implementation:
```rust
const _: () = {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Value>();
    assert_send_sync::<Expr>();
};
```

## 9. Parser Changes

### 9.1 Anonymous Function Expression

Parse `fn(params) [-> type] { body }` as an expression in `parse_expr`:

```
fn_expr = "fn" "(" params ")" ["->" type] "{" expr "}"
```

Priority: after record constructor detection, before variable fallback.

Only valid in local context (inside workflow bodies, function bodies, blocks). Parser enforces this by only trying `fn_expr` when inside a workflow or function body.

### 9.2 Closure Syntax

Parse `|params| => expr` as an expression:

```
closure_expr = "|" params "=>" expr
```

### 9.3 Named Local Function Desugaring

In the parser or lowering step, `fn name(params) { body }` inside a workflow/block becomes:

```
let name = fn(params) { body }
```

No new IR node needed -- it's syntactic sugar. This desugaring applies only in local context, not at module top-level where `pub fn` remains a `Definition::Function`.

## 10. Lowering Changes

### 10.1 Surface FnDef to Core

The lowering step (`crates/ash-parser/src/lower.rs`) translates:

- Surface anonymous function expression -> `CoreExpr::FnDef { params, return_type, body }`
- Surface named local function definition -> `CoreWorkflow::Let { pattern: name, expr: FnDef { ... }, continuation }`
- Surface function call to user-defined name -> `CoreExpr::FnApply { func: Variable(name), args }`
- Surface function call to known built-in -> `CoreExpr::Call { func, arguments }` (unchanged)

### 10.2 Built-in Detection

The lowering step consults a built-in function registry to decide `Call` vs `FnApply`:

```rust
/// Registry of built-in function names known to the interpreter.
/// Used during lowering to disambiguate Expr::Call vs Expr::FnApply.
pub const BUILTIN_FUNCTIONS: &[&str] = &[
    "len", "append", "concat", "prepend",
    "string::length", "string::trim", "string::to_uppercase", ...
];
```

If `func_name` is in `BUILTIN_FUNCTIONS` and is NOT shadowed by a local binding, emit `Expr::Call`. Otherwise emit `Expr::FnApply`.

## 11. Interpreter Changes

### 11.1 eval_expr Additions (ash-interp/src/eval.rs)

```rust
Expr::FnDef { params, body, .. } => {
    // Capture current environment as a shared frame
    let env_frame = ctx.to_env_frame();
    Ok(Value::Closure {
        params: params.iter().map(|(n, t)| (n.clone(), t.as_ref().map(|s| s.to_string()))).collect(),
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
            // Build call environment: captured frame + parameter bindings
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

### 11.2 Context Extensions (ash-interp/src/context.rs)

```rust
impl Context {
    /// Create an EnvFrame snapshot of this context for closure capture.
    /// The frame chain is shared via Arc -- O(1) capture, no flattening.
    pub fn to_env_frame(&self) -> Arc<EnvFrame> { ... }

    /// Create a Context from a captured EnvFrame.
    /// The context inherits the frame chain as its scope.
    pub fn from_env_frame(frame: &Arc<EnvFrame>) -> Self { ... }
}
```

### 11.3 Expr::Call Fallback (Migration Only)

During Phase B, `Expr::Call` gains a closure-lookup fallback:

```rust
Expr::Call { func, arguments } => {
    // 1. Try built-in dispatch
    if let Ok(result) = eval_function_call(func, &args) {
        return Ok(result);
    }
    // 2. Try closure lookup in context (migration fallback)
    if let Some(Value::Closure { .. }) = ctx.get(func) {
        // Re-dispatch as FnApply
        ...
    }
    // 3. Error
    Err(EvalError::UnknownFunction { name: func.clone() })
}
```

After Phase C, this fallback is removed.

## 12. Removal of pure_runtime.rs

After the interpreter supports FnDef/FnApply natively:

1. Delete `crates/ash-engine/src/pure_runtime.rs` (476 lines)
2. Remove `should_execute_via_pure_runtime` and related dispatch logic from `crates/ash-engine/src/lib.rs`
3. Remove `inline_imported_calls_in_workflow_def` (inlining hack)
4. Remove `collect_local_inline_callables`
5. All execution goes through `interpret_in_state`
6. `parse_program_with_functions` is no longer needed -- `parse_file` produces core IR with FnDef/FnApply directly

## 13. Migration Path

### Phase A: Add FnDef/FnApply to core IR and interpreter (non-breaking)
1. Add `Expr::FnDef`, `Expr::FnApply` to `ash-core/src/ast.rs`
2. Add `Value::Closure` to `ash-core/src/value.rs`
3. Add `EnvFrame` to `ash-core/src/env_frame.rs`
4. Add `LateBinding` cell for recursive let-bindings
5. Add eval cases to `ash-interp/src/eval.rs`
6. Add `Context::to_env_frame` and `Context::from_env_frame`
7. Add compile-time `Send + Sync` assertions
8. All existing tests pass -- new variants are just not produced yet

### Phase B: Lower surface functions to core IR (non-breaking)
1. Add `lower_fn_def` to `crates/ash-parser/src/lower.rs`
2. Add built-in function registry (`BUILTIN_FUNCTIONS`)
3. Named local `fn` in workflow context -> `Let { name, FnDef { ... } }`
4. Function calls to user-defined names -> `FnApply { Variable(name), args }`
5. Add closure-lookup fallback to `Expr::Call` handler
6. Keep pure_runtime active during transition

### Phase C: Delete pure_runtime (breaking cleanup)
1. Delete `pure_runtime.rs`
2. Remove dispatch logic from `lib.rs`
3. Remove inlining code
4. Remove `Expr::Call` closure fallback (all user calls now use `FnApply`)
5. Verify all tests pass through the single interpreter path

### Phase D: Parser additions (enhancement)
1. Parse anonymous `fn(params) { body }` expressions in local context
2. Parse `|params| => body` closure syntax
3. Add `Fn(T) -> U` type syntax to the parser
4. These enable inline closures in workflow and function bodies

### Phase E: Type checker integration
1. Add `Type::Fn` variant to type system
2. Type-check `FnDef` and `FnApply` expressions
3. Enforce three-vertex boundary via effect markers on closure types

## 14. Conformance

### 14.1 Minimal Conformance

An implementation conforming to SPEC-031 must:
- Evaluate `Value::Closure` from `Expr::FnDef` with shared environment capture (Arc<EnvFrame>)
- Apply closures via `Expr::FnApply` with correct environment extension
- Support recursion in named local functions via late binding
- Support higher-order functions (passing/returning closures)
- Execute all programs previously handled by `pure_runtime`
- Distinguish `Expr::Call` (built-ins) from `Expr::FnApply` (user functions) in lowering
- Ensure `Value::Closure` is `Send + Sync`
- NOT serialize `Value::Closure` across process boundaries
- Enforce three-vertex boundary for closures
- Pass all existing tests

### 14.2 Full Conformance

Additionally supports:
- Anonymous function expressions in the parser
- Closure syntax `|x| => ...`
- `Fn(T1, T2) -> U` type syntax and type checking
- Effect markers on closure types for three-vertex enforcement
- Proper tail-call optimization for recursive closures (future)

## 15. Files Affected

| File | Change |
|------|--------|
| `crates/ash-core/src/ast.rs` | Add `Expr::FnDef`, `Expr::FnApply` |
| `crates/ash-core/src/value.rs` | Add `Value::Closure` (no Serialize/Deserialize) |
| `crates/ash-core/src/env_frame.rs` | NEW: `EnvFrame`, `LateBinding` |
| `crates/ash-core/src/lib.rs` | Export `env_frame` module |
| `crates/ash-interp/src/context.rs` | Add `to_env_frame`, `from_env_frame` |
| `crates/ash-interp/src/eval.rs` | Add eval cases for FnDef, FnApply, closure Call fallback |
| `crates/ash-interp/src/error.rs` | Add `NotCallable`, `UnknownFunction` error variants |
| `crates/ash-parser/src/lower.rs` | Add `lower_fn_def`, built-in registry, update `lower_expr` |
| `crates/ash-parser/src/parse_expr.rs` | Add anonymous fn expression, closure syntax (Phase D) |
| `crates/ash-engine/src/lib.rs` | Remove pure_runtime dispatch, inlining (Phase C) |
| `crates/ash-engine/src/pure_runtime.rs` | DELETE (Phase C) |
| `crates/ash-typeck/src/` | Add `Type::Fn`, effect markers (Phase E) |
