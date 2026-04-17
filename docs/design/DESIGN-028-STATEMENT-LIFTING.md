# DESIGN-028: Statement Lifting and Workflow/Expression Integration

## Status: Draft

## Overview

This design addresses the impedance mismatch between Ash's three callable domains:

- **Workflows** — sequential, effectful statements with metadata and process lifecycle.
- **Pure functions (`fn`)** — expression-oriented, side-effect-free computations.
- **Capabilities** — effectful callables that interface with the external world.

The tension arises because capabilities can only participate in workflow *statements* (e.g., `let dir_list = read_dir(path)`), but cannot appear inside *expressions* (e.g., `filter(ends_with(".md"), read_dir(path))`). This forces users to break natural pipelines into verbose statement sequences.

This design specifies **statement lifting** — a syntactic transformation that allows effectful computations to appear inside expression positions within workflows, without compromising the purity boundary of functions.

## Problem Statement

### Concrete example of the pain point

```ash
// Current forced style (verbose)
workflow process_dir(path: String) -> List<String> {
    let dir_list = read_dir(path)           // stmt: capability call
    let md_files = filter(ends_with(".md"), dir_list)  // expr: pure function
    ret md_files
}
```

The user wants to write:

```ash
// Desired style (natural pipeline)
workflow process_dir(path: String) -> List<String> {
    let md_files = read_dir(path) |> filter(ends_with(".md"))
    ret md_files
}
```

Or equivalently:

```ash
workflow process_dir(path: String) -> List<String> {
    let md_files = filter(ends_with(".md"), read_dir(path))
    ret md_files
}
```

### Root cause

The parser and typechecker enforce a strict separation:
- `Expr` can contain pure calls, literals, operators, lambdas.
- `Stmt` (inside workflows) can contain `Expr` *and* capability calls, observations, spawns, etc.
- But `Expr` cannot contain `Stmt`.

This separation is semantically correct (functions must remain pure), but it is **too strict at the syntactic level** for workflows. A workflow is *already* an effectful context; it should be allowed to sequence effects naturally.

## Goals

1. Allow effectful computations (statements) to appear inside expression argument positions **within workflow bodies only**.
2. Preserve the invariant: **functions (`fn`) remain pure** — no capability calls inside function bodies.
3. Support both a **pipe operator (`|>`)** and **implicit argument lifting** for ergonomic effect/expression mixing.
4. Make the transformation a purely syntactic lowering pass (ANF lifting) rather than a deep type-system change.
5. Enable partial application/currying as a prerequisite for ergonomic pipelines.

## Non-Goals

1. Allowing capability calls inside `fn` bodies.
2. Introducing a first-class `comp T` computation type into the core type system (this design uses syntactic lowering).
3. Changing the evaluation semantics of `act`, `observe`, or `Expr` evaluation.
4. Generalizing statement lifting to arbitrary nested expressions outside workflows.

## Chosen Path and Rationale

**This design selects Option C1** (pipe operator `|>` + ANF lifting) as the MVP path.

C1 is chosen because it delivers the immediate ergonomic win — natural pipelines inside workflows — without destabilizing the type system. The cost is confined to the parser and lowering pass (days, not weeks). It preserves the current mental model: workflows are imperative sequences, functions are pure expressions.

Option C3 (a first-class `comp T` computation type) is the principled long-term target. It would make the workflow monad explicit, enable generic combinators (`map`, `sequence`), and give JIT backends a rigorous typed boundary. However, it requires a foundational rebuild of the type representation, unification engine, generic bounds system, and runtime value domain. That effort is **10–20× C1** and gates most other language work until complete.

Therefore, C1 is implemented now, and C3 is recorded as an architectural north star in `docs/notes/NOTE-001-WORKFLOW-COMPUTATION-TYPE.md` to be revisited after the surface syntax and big-step semantics are frozen.

## Design

### 3.1 Syntactic Classification

We classify syntactic forms by their *immediate* evaluation behavior:

| Form | Category | Type intuition | Can appear in `fn`? |
|---|---|---|---|
| Literal, variable, arithmetic, pure call | `Expr` | `T` | Yes |
| `act`, `observe`, `spawn`, `yield`, `read_dir(...)` | `Stmt` (effectful) | `Stmt(T)` | No (in `fn`) |
| `let x = ___ in ...` | `Stmt` binder | sequencing | No (in `fn`) |

Within a **workflow body**, the category `Stmt(T)` behaves *syntactically* like a monadic computation: it must be "run" to obtain a `T`. The `let` binder is the sequencing operator.

### 3.2 Statement Lifting via ANF

The core mechanism is **A-Normal Form (ANF) lifting**. Any `Stmt(T)` that appears inside an expression argument position is extracted into a preceding `let` binding, and a synthetic variable of type `T` is substituted in its place.

#### Rule 1: Direct argument lifting

```ash
let md_files = filter(ends_with(".md"), read_dir(path))
```

**Lowered (desugared):**

```ash
let __lift_0 = read_dir(path)
let md_files = filter(ends_with(".md"), __lift_0)
```

**Algorithm:**
1. Traverse the RHS expression in pre-order.
2. When a sub-expression is classified as `Stmt(T)` (e.g., a capability call),
   - generate a fresh synthetic name `__lift_N`,
   - emit a preceding `let __lift_N = <stmt>` in the current workflow block,
   - replace the sub-expression with the variable `__lift_N`.
3. Evaluation order is left-to-right, depth-first (call-by-value).

#### Rule 2: Pipe operator lifting

```ash
let md_files = read_dir(path) |> filter(ends_with(".md"))
```

**Lowered:**

```ash
let __lift_0 = read_dir(path)
let md_files = filter(ends_with(".md"), __lift_0)
```

The pipe operator `|>` is syntax sugar that desugars *before* ANF lifting:

```ash
a |> f(b, c)   →   f(b, c, a)
expr |> g      →   g(expr)
stmt |> g      →   let __lift = stmt in g(__lift)
```

### 3.3 Pipe Operator Grammar

Add `|>` as a low-precedence binary operator in workflow expression context only.

```ebnf
workflow_expr     ::= pipeline_expr
pipeline_expr     ::= atomic_expr ( "|>" call_expr )*
atomic_expr       ::= literal | variable | "(" workflow_expr ")" | block_expr
call_expr         ::= name "(" arg_list ")" | name
```

**Precedence:** `|>` binds looser than comma (function arguments) and tighter than `let`.

**Examples:**

```ash
x |> f             →  f(x)
x |> f(a)          →  f(a, x)
x |> f(a) |> g(b)  →  g(b, f(a, x))
```

### 3.4 Partial Application

For the pipe operator to be ergonomic, functions (and capability calls that return functions) must support **partial application**.

#### Grammar

A function call with missing trailing arguments is a valid expression that evaluates to a closure:

```ash
filter(ends_with(".md"))   // type: List<String> -> List<String>
```

**Lowered form:**

```ash
\items -> filter(ends_with(".md"), items)
```

#### Type rule

If `f : (A, B, C) -> D` and `f` is applied to arguments of types `A` and `B`, the result has type `C -> D`.

Partial application is only permitted when:
1. The callee is a known function or workflow reference (not an arbitrary expression).
2. The number of provided arguments is less than the arity.
3. All provided arguments are pure expressions.

*Note: Partial application of effectful calls (e.g., `act foo() |> map(...)`) is allowed only if the effectful call is evaluated first and returns a function, which is then partially applied.*

### 3.5 Lifting Algorithm (Formal)

Define a lowering function `lift(expr, ctx)` that returns `(stmts, pure_expr)`:

```
lift : Expr × LiftingCtx -> (Vec<Stmt>, Expr)
```

**Base cases:**

```
lift(literal, ctx)       = ([], literal)
lift(variable, ctx)      = ([], variable)
lift(pure_call(args), ctx) =
    let (stmts, pure_args) = lift_args(args, ctx)
    (stmts, pure_call(pure_args))
```

**Effectful call case:**

```
lift(effectful_call(args), ctx) =
    let (stmts, pure_args) = lift_args(args, ctx)
    let tmp = ctx.fresh_name()
    (stmts ⊢ [let tmp = effectful_call(pure_args)], tmp)
```

**Argument lifting (left-to-right):**

```
lift_args([], ctx) = ([], [])
lift_args([arg | rest], ctx) =
    let (stmts1, pure_arg) = lift(arg, ctx)
    let (stmts2, pure_rest) = lift_args(rest, ctx)
    (stmts1 ++ stmts2, [pure_arg | pure_rest])
```

**Pipe case:**

```
lift(lhs |> call(args), ctx) =
    let (stmts_lhs, pure_lhs) = lift(lhs, ctx)
    let (stmts_args, pure_args) = lift_args(args, ctx)
    let tmp = ctx.fresh_name()
    (stmts_lhs ++ stmts_args ++ [let tmp = call(pure_args ++ [pure_lhs])], tmp)
```

After lifting, the emitted `stmts` are prepended to the current workflow block before the `let` declaration that contained the RHS.

### 3.6 Interaction with Existing Workflow Forms

Statement lifting applies **only inside workflow bodies**. The contexts where lifting is triggered:

1. RHS of `let` bindings in workflows.
2. RHS of `ret` in workflows (if `ret` contains an effectful sub-expression).
3. Arguments to `act` and `observe` (already evaluated atomically; lifting just sequences them before the call).
4. Conditions of `if` in workflows.
5. Collections of `for` in workflows.

**Explicitly excluded contexts (no lifting, compilation error):**

1. Inside `fn` bodies.
2. Inside `match` guards.
3. Inside pattern expressions.
4. Inside policy expressions.

### 3.7 Typechecker Integration

The typechecker needs minimal changes because lifting occurs **before** typechecking, at the AST/lowering layer.

**Phase ordering:**

1. Parse surface syntax.
2. Lower surface → core AST (`Workflow`, `Expr`, `Stmt`).
3. **Apply ANF lifting** to workflow bodies.
4. Typecheck the lifted AST.

Because the lifted AST contains only pure expressions inside effectful calls, the existing typechecker can handle it without modification.

However, for good diagnostics, the typechecker should **preserve span information** through the lifting pass so that errors on `__lift_N` variables can be mapped back to the original source expression.

### 3.8 Examples

#### Example 1: Nested capability calls

```ash
workflow main() -> String {
    let content = read_text(fetch_url(get_env("API_ENDPOINT")))
    ret content
}
```

**Lowered:**

```ash
workflow main() -> String {
    let __lift_0 = get_env("API_ENDPOINT")
    let __lift_1 = fetch_url(__lift_0)
    let content = read_text(__lift_1)
    ret content
}
```

#### Example 2: Pipe chain with partial application

```ash
workflow main(path: String) -> List<String> {
    let lines = read_dir(path) 
        |> filter(ends_with(".md"))
        |> map(read_text)
        |> filter(contains("TODO"))
    ret lines
}
```

**Lowered:**

```ash
workflow main(path: String) -> List<String> {
    let __lift_0 = read_dir(path)
    let __lift_1 = filter(ends_with(".md"), __lift_0)
    let __lift_2 = map(read_text, __lift_1)
    let lines = filter(contains("TODO"), __lift_2)
    ret lines
}
```

#### Example 3: Mixed pure and effectful arguments

```ash
workflow main(base: String) -> List<String> {
    let result = join(base, read_dir(concat(base, "/sub")))
    ret result
}
```

**Lowered:**

```ash
workflow main(base: String) -> List<String> {
    let __lift_0 = concat(base, "/sub")
    let __lift_1 = read_dir(__lift_0)
    let result = join(base, __lift_1)
    ret result
}
```

Note: `concat(base, "/sub")` is a pure expression, so it is lifted as an argument to `read_dir`, not pulled out as a separate `let`.

#### Example 4: Invalid usage (compile-time error)

```ash
fn pure_join(base: String) -> List<String> {
    join(base, read_dir(base))   // ERROR: capability call inside fn
}
```

The parser or lowering pass detects `read_dir` inside a `fn` body and emits a dedicated error:

```
Error: Effectful computation 'read_dir' is not allowed inside a pure function.
Hint: Move this logic into a workflow, or pass the result as an argument.
```

## 4. Function Lifting (Embedding Pure Functions into Statements)

The user explicitly wants to bring/embed **functions into statement context**, not the other way around. This is already supported: a pure function call is a valid `Expr`, and `Expr` is a valid RHS for `let` in a workflow.

The missing piece is **partial application**, which turns a multi-argument pure function into a unary function that can be piped:

```ash
let processor = filter(ends_with(".md"))   // now a value of type List<String> -> List<String>
let result = processor(my_list)
```

This requires:
1. **Parser:** allow omitted trailing arguments.
2. **Typechecker:** infer the partial type `C -> D` from `f : (A, B, C) -> D` applied to `A` and `B`.
3. **Runtime:** represent partial applications as `Value::Closure` (or equivalent).

## 5. Risks and Open Questions

1. **Evaluation order:** ANF lifting enforces strict left-to-right, call-by-value evaluation. This matches Ash's current semantics but must be documented explicitly.

2. **Error span mapping:** Synthetic `__lift_N` variables will appear in type errors unless the compiler maps them back to the original sub-expression spans. The lifting pass must attach the original span to each synthetic `let`.

3. **Interaction with `Workflow::Call`:** If a called workflow returns a function, partial application of the result must be supported. This may require runtime representation of workflow-returned closures.

4. **Over-eager lifting:** If the user writes `let x = f(a, b, c)` where `f` is a pure function, no lifting occurs. The pass must only trigger on syntactically effectful sub-expressions.

5. **Lambda bodies:** Can a lambda inside a workflow contain a capability call? 
   - **Decision:** No. Lambdas are expressions, and expressions remain pure even inside workflows. This preserves the invariant that `fn` and lambda bodies are interchangeable.

6. **Pipeline operator in expressions vs statements:** The `|>` operator is only valid in workflow expression context. It is not a general expression operator.

## 6. Migration Path

1. **Add partial application support** to the typechecker and runtime (prerequisite).
2. **Add the pipe operator** to the parser as workflow-only syntax sugar.
3. **Implement the ANF lifting pass** in `crates/ash-parser/src/lift.rs` (or as part of the lowering pass).
4. **Wire the pass** so it runs after surface → core lowering but before typechecking.
5. **Add parser/typechecker tests** for all examples in §3.8.
6. **Reject capability calls inside `fn` bodies** with a clear, dedicated error message.

## Acceptance Criteria

1. `filter(ends_with(".md"))` parses, typechecks, and evaluates as a partial application.
2. `read_dir(path) |> filter(ends_with(".md"))` parses and evaluates correctly inside a workflow.
3. `filter(ends_with(".md"), read_dir(path))` inside a workflow desugars to the equivalent sequential form without user-visible `__lift` variables.
4. `fn` bodies containing capability calls produce a compile-time error.
5. All existing tests pass; no regression in pure function behavior.
6. `cargo check`, `cargo clippy`, `cargo fmt --check` clean.
