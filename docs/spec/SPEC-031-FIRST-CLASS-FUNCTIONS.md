# SPEC-031: First-Class Functions and Closure Values

**Status:** Draft
**Date:** 2026-04-14
**Version:** 0.1

## 1. Overview

Add first-class function values to Ash. Function definitions become expressions that produce closure values. Named functions desugar to let-bindings. The interpreter (`ash-interp`) handles function definition and application natively, eliminating the `pure_runtime` workaround.

## 2. Motivation

The current architecture has a split: `ash-interp` operates on lowered IR that has no function concept, while `pure_runtime.rs` is a duplicate interpreter that exists solely to handle functions. This is tech debt:

- Two expression evaluation implementations that must stay in sync
- Inlining hack only works for non-recursive, non-higher-order cases
- `should_execute_via_pure_runtime` dispatch is a code smell
- Functions cannot be passed as arguments, returned from calls, or stored in data structures

The fix: treat functions as first-class values in the core IR and interpreter.

## 3. Semantics

### 3.1 Anonymous Function Expression

```
fn(a, b) { a + b }
```

This is an expression that evaluates to a closure value capturing its lexical environment.

### 3.2 Named Function Desugaring

```
fn add(a: Int, b: Int) -> Int { a + b }
```

Desugars to:

```
let add = fn(a: Int, b: Int) -> Int { a + b }
```

The name is in lexical scope for all subsequent expressions in the same block (supports recursion).

### 3.3 Closure Syntax (Sugar)

```
|x, y| => x + y
```

Desugars to:

```
fn(x, y) { x + y }
```

### 3.4 Function Application

```
add(1, 2)
```

When `add` resolves to a closure value, the application extends the closure's captured environment with parameter bindings and evaluates the body.

### 3.5 Higher-Order Functions

Functions can be passed as arguments and returned from calls:

```
fn apply(f, x) { f(x) }
fn double(n) { n * 2 }
apply(double, 5)   -- evaluates to 10
```

### 3.6 Recursion

Named functions can call themselves because the let-binding is visible within the function body:

```
fn factorial(n) {
    if n <= 1 then 1 else n * factorial(n - 1)
}
```

This desugars to `let factorial = fn(n) { ... }`. The closure captures `factorial` from its own let-binding, enabling recursion through environment lookup.

### 3.7 Scope and Capture

Closures capture their lexical environment at definition time:

```
fn make_adder(n) {
    fn(x) { n + x }
}
let add5 = make_adder(5);
add5(3)   -- evaluates to 8
```

The inner closure captures `n` from the outer function's scope.

## 4. IR Changes

### 4.1 New Expr Variants (ash-core)

```rust
/// ash-core/src/ast.rs -- Expr enum additions

/// Anonymous function definition (closure creation).
/// Evaluating this expression produces a Value::Closure capturing the current environment.
FnDef {
    params: Vec<(Name, Option<Type>)>,   -- parameter names and optional types
    return_type: Option<Type>,            -- optional return type annotation
    body: Box<Expr>,                      -- function body expression
},

/// Function application with dynamic dispatch.
/// Evaluates `func` to obtain a closure, binds parameters, evaluates body.
FnApply {
    func: Box<Expr>,                      -- expression producing a closure value
    args: Vec<Expr>,                      -- argument expressions
},
```

### 4.2 New Value Variant (ash-core)

```rust
/// ash-core/src/value.rs -- Value enum addition

/// Runtime closure value: captures lexical environment at definition time.
/// When applied, extends the captured environment with parameter bindings.
Closure {
    /// Parameter names (and optional type annotations, for display/debugging)
    params: Vec<(String, Option<String>)>,
    /// The function body expression (boxed to reduce enum size)
    body: Box<ash_core::ast::Expr>,
    /// The captured lexical environment
    env: HashMap<String, Value>,
},
```

### 4.3 Existing `Expr::Call` Transition

`Expr::Call { func: Name, arguments: Vec<Expr> }` currently only handles built-in functions (`len`, `append`, etc.) via string dispatch in `eval_function_call`. After this spec:

- `Expr::Call` remains for known built-in functions (string-name dispatch)
- `Expr::FnApply` handles closure application (dynamic, first-class)
- The lowering step converts named function calls to `FnApply(Variable(name), args)` when `name` resolves to a closure in scope, or keeps them as `Call` for built-ins

## 5. Parser Changes

### 5.1 Anonymous Function Expression

Parse `fn(params) [-> type] { body }` as an expression in `parse_expr`:

```
fn_expr = "fn" "(" params ")" ["->" type] "{" expr "}"
```

Priority: after record constructor detection, before variable fallback.

### 5.2 Closure Syntax

Parse `|params| => expr` as an expression:

```
closure_expr = "|" params "=>" expr
```

### 5.3 Named Function Desugaring

In the parser or lowering step, `fn name(params) { body }` becomes:

```
let name = fn(params) { body }
```

No new IR node needed -- it's syntactic sugar.

## 6. Lowering Changes

### 6.1 Surface `FnDef` to Core

The lowering step (`crates/ash-parser/src/lower.rs`) translates:

- Surface anonymous function expression -> `CoreExpr::FnDef { params, return_type, body }`
- Surface named function definition -> `CoreWorkflow::Let { pattern: name, expr: FnDef { ... }, continuation }`
- Surface function call -> `CoreExpr::FnApply { func: Variable(name), args }` when the callee is a user function (not a built-in)

### 6.2 Built-in Detection

The lowering step does NOT need to know all built-in names. Instead:

- Calls like `f(args)` lower to `FnApply { func: Variable("f"), args }`
- The interpreter first checks if the variable resolves to a `Value::Closure` -> apply it
- If not found, fall through to the built-in `eval_function_call` dispatch

This avoids hardcoding built-in names in the lowering step.

## 7. Interpreter Changes

### 7.1 eval_expr Additions (ash-interp/src/eval.rs)

```rust
Expr::FnDef { params, body, .. } => {
    // Capture current environment
    let env = ctx.snapshot_bindings();  // new method on Context
    Ok(Value::Closure {
        params: params.iter().map(|(n, t)| (n.clone(), t.as_ref().map(|s| s.to_string()))).collect(),
        body: Box::new(body.as_ref().clone()),
        env,
    })
}

Expr::FnApply { func, args } => {
    // Evaluate the function expression to get a closure
    let callee = eval_expr(func, ctx)?;
    match callee {
        Value::Closure { params, body, env } => {
            // Evaluate arguments
            let arg_values: Vec<Value> = args.iter()
                .map(|a| eval_expr(a, ctx))
                .collect::<Result<Vec<_>, _>>()?;
            // Build new environment: captured env + parameter bindings
            let mut call_env = Context::from_bindings(env);
            for ((name, _), value) in params.iter().zip(arg_values.iter()) {
                call_env.set(name.clone(), value.clone());
            }
            // Evaluate body in the new environment
            eval_expr(&body, &call_env)
        }
        _ => Err(EvalError::NotCallable { value: callee }),
    }
}
```

### 7.2 Expr::Call Fallback

After the FnApply path is established, `Expr::Call { func, arguments }` transitions to:

1. Check if `func` is a variable bound to a `Value::Closure` in context -> apply
2. Fall through to built-in `eval_function_call` dispatch

This ensures backward compatibility with existing code that produces `Expr::Call`.

### 7.3 Context Extension

Add `snapshot_bindings` and `from_bindings` to `Context`:

```rust
impl Context {
    /// Snapshot all bindings in this scope and parents into a flat HashMap
    pub fn snapshot_bindings(&self) -> HashMap<Name, Value> { ... }

    /// Create a context from a flat HashMap of bindings
    pub fn from_bindings(bindings: HashMap<Name, Value>) -> Self { ... }
}
```

## 8. Removal of pure_runtime.rs

After the interpreter supports FnDef/FnApply natively:

1. Delete `crates/ash-engine/src/pure_runtime.rs` (476 lines)
2. Remove `should_execute_via_pure_runtime` and related dispatch logic from `crates/ash-engine/src/lib.rs`
3. Remove `inline_imported_calls_in_workflow_def` (inlining hack)
4. All execution goes through `interpret_in_state`
5. `parse_program_with_functions` is no longer needed -- `parse_file` produces core IR with FnDef/FnApply directly

## 9. Migration Path

### Phase A: Add FnDef/FnApply to core IR and interpreter (non-breaking)
1. Add `Expr::FnDef`, `Expr::FnApply` to `ash-core/src/ast.rs`
2. Add `Value::Closure` to `ash-core/src/value.rs`
3. Add eval cases to `ash-interp/src/eval.rs`
4. Add `Context::snapshot_bindings` and `Context::from_bindings`
5. All existing tests pass -- new variants are just not produced yet

### Phase B: Lower surface functions to core IR (non-breaking)
1. Add `lower_fn_def` to `crates/ash-parser/src/lower.rs`
2. Named `fn` in workflow context -> `Let { name, FnDef { ... } }`
3. Function calls -> `FnApply { Variable(name), args }` (with built-in fallback)
4. Keep pure_runtime active during transition

### Phase C: Delete pure_runtime (breaking cleanup)
1. Delete `pure_runtime.rs`
2. Remove dispatch logic from `lib.rs`
3. Remove inlining code
4. Verify all tests pass through the single interpreter path

### Phase D: Parser additions (enhancement)
1. Parse anonymous `fn(params) { body }` expressions
2. Parse `|params| => body` closure syntax
3. These enable inline closures in workflow and function bodies

## 10. Conformance

### 10.1 Minimal Conformance

An implementation conforming to SPEC-031 must:
- Evaluate `Value::Closure` from `Expr::FnDef`
- Apply closures via `Expr::FnApply` with correct environment capture
- Support recursion in named functions
- Support higher-order functions (passing/returning closures)
- Execute all programs previously handled by `pure_runtime`
- Pass all existing tests

### 10.2 Full Conformance

Additionally supports:
- Anonymous function expressions in the parser
- Closure syntax `|x| => ...`
- Proper tail-call optimization for recursive closures (future)

## 11. Files Affected

| File | Change |
|------|--------|
| `crates/ash-core/src/ast.rs` | Add `Expr::FnDef`, `Expr::FnApply` |
| `crates/ash-core/src/value.rs` | Add `Value::Closure` |
| `crates/ash-interp/src/context.rs` | Add `snapshot_bindings`, `from_bindings` |
| `crates/ash-interp/src/eval.rs` | Add eval cases for FnDef, FnApply, closure Call |
| `crates/ash-interp/src/error.rs` | Add `NotCallable` error variant |
| `crates/ash-parser/src/lower.rs` | Add `lower_fn_def`, update `lower_expr` |
| `crates/ash-parser/src/parse_expr.rs` | Add anonymous fn expression, closure syntax |
| `crates/ash-engine/src/lib.rs` | Remove pure_runtime dispatch, inlining |
| `crates/ash-engine/src/pure_runtime.rs` | DELETE |
