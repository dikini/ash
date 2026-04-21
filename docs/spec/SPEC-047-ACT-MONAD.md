# SPEC-047: The Act Monad — First-Class Effectful Computation

**Status:** Draft
**Date:** 2026-04-22
**Related:** NOTE-005, SPEC-001, SPEC-004, SPEC-025, SPEC-027, SPEC-031, SPEC-BUILTIN-FN, SPEC-020
**Supersedes:** NOTE-005 (design exploration — this spec is its normative counterpart)

## 1. Overview

Introduce `Act<A>` as a first-class type constructor in the expression layer, unifying the currently split evaluation contexts: pure expressions (`eval_expr`) and effectful workflows (`execute_workflow`). An `Act<A>` value is a suspended computation that, given an environment of capability providers, policies, and provenance, may produce a value of type `A` alongside an accumulated effect log, or fail with an error.

The core friction resolved: `act` currently exists only as a `Workflow` node. You cannot call a capability from inside an expression, and you cannot compose effectful operations as first-class values. The `act {}` block lifts effectful computation into the expression layer while preserving all governance properties (sequential ordering, provenance, policy checking, audit trail).

### 1.1 Design Principles

1. **No runtime magic.** `unit`, `bind`, `then`, `guard` are ordinary Ash functions. Only `invoke` requires runtime support.
2. **Type-system purity boundary.** Functions returning `B` are pure. Functions returning `Act<B>` are effectful. The type system prevents calling effectful code from pure contexts.
3. **Governance preservation.** Every `invoke` passes through the policy stack. Effect logs are append-only. Provenance chains are maintained.
4. **Workflow compatibility.** Workflows become structured sugar over `Act`. Existing workflow syntax, types, and execution remain valid.
5. **Incremental delivery.** The spec is designed to be implemented in phases that leave the system working at each step.

### 1.2 Scope

In scope:
- `Act<A>` type constructor in surface syntax, core IR, and type system
- `act { ... }` block expression for monadic composition
- `invoke` as the single runtime primitive for effectful operations
- Effectful function declarations (`fn f(x) -> Act<B>`)
- Library functions: `unit`, `bind`, `then`, `guard`
- Semantic rules for act blocks, bind, invoke, and purity enforcement
- Migration of `act` from workflow-only to dual-context (workflow + expression)

Out of scope (deferred):
- `observe` vs `execute` type-level distinction (see §11)
- Concurrent composition (`par`)
- Interface-based `Functor`/`Applicative`/`Monad` hierarchy (requires HKT)
- `extern fn` with `Act` return types
- Migration of existing stdlib `.ash` files (separate phase)

## 2. Surface Syntax

### 2.1 Act Block Expression

```
act_expr ::= "act" "{" act_stmt* "}"

act_stmt ::= IDENTIFIER "=" expr ";"          -- bind or inline
           | "ret" expr ";"                    -- unit (return)
           | "invoke" "(" expr "," expr "," expr ")" ";"  -- explicit invoke
```

An `act {}` block is an expression that evaluates to a value of type `Act<A>`. It may appear anywhere an expression is expected: in fn bodies, as arguments, in let-bindings, inside other act blocks.

### 2.2 Effectful Function Declaration

```
fn read(path: String) -> Act String {
    act {
        result = invoke("Fs", "read", [path])
        ret result
    }
}
```

The return type `Act String` distinguishes this from a pure function. The body must produce a value of type `Act String` — either an `act {}` block, a call to another effectful function, or `invoke` directly.

### 2.3 Bind Desugaring

```
act {
    x = read(path)         -- bind (RHS : Act String, x : String)
    n = len(x)             -- inline (RHS : Int, pure substitution)
    y = parse(x)           -- bind (RHS : Act Value, y : Value)
    ret (x, n, y)          -- unit
}
```

Desugars to:

```
bind(read(path), |x|
  bind(parse(x), |y|
    unit((x, len(x), y))))
```

Note: `n` does not appear in the desugared form. `len(x)` is inlined directly. Pure bindings are syntactic convenience, not monadic operations.

### 2.4 Invoke Expression

The primitive effectful operation:

```
invoke(provider: String, action: String, args: List) -> Act Value
```

This is a `builtin fn` whose implementation is provided by the runtime. It dispatches to the appropriate `CapabilityProvider`, passes through the full policy stack, and appends to the effect log.

### 2.5 Library Functions

These are ordinary Ash functions in `std/src/act.ash`:

```ash
fn unit(v: a) -> Act a {
    |env| => Ok((v, env))
}

fn bind(ma: Act a, f: (a -> Act b)) -> Act b {
    |env| => match ma(env) {
        Ok((a, env')) => f(a)(env'),
        Err(e) => Err(e)
    }
}

fn then(ma: Act a, mb: Act b) -> Act b {
    bind(ma, |_a| => mb)
}

fn guard(policy: Policy, ma: Act a) -> Act a {
    |env| => match env.policies.check(policy) {
        Deny(reason) => Err(PolicyViolation(reason)),
        Allow => ma(env)
    }
}
```

### 2.6 Keyword Choice

The block keyword is `act` — consistent with existing workflow `act` syntax and NOTE-005. Alternative names (`do`, `perform`) are equivalent in semantics; `act` preserves continuity.

## 3. Core IR Changes

### 3.1 New Expression Variants

Add to `crates/ash-core/src/ast.rs` `Expr` enum:

```rust
/// Act block: monadic composition of effectful operations
ActBlock {
    /// Sequence of statements (binds, invokes, returns)
    stmts: Vec<ActStmt>,
    span: Span,
},

/// Explicit invoke: the primitive effectful operation
Invoke {
    provider: Box<Expr>,
    action: Box<Expr>,
    arguments: Box<Expr>,
    span: Span,
},
```

### 3.2 ActStmt

New type:

```rust
pub enum ActStmt {
    /// Bind: x = <expr>; — either monadic bind or pure inline
    Bind {
        name: String,
        value: Box<Expr>,
        span: Span,
    },
    /// Return: ret <expr>;
    Return {
        value: Box<Expr>,
        span: Span,
    },
}
```

### 3.3 TypeExpr Addition

Add to `crates/ash-core/src/ast.rs` `TypeExpr` enum:

```rust
/// Act<A> — an effectful computation yielding A
Constructor { name, args } already supports this.
```

No new `TypeExpr` variant needed. `Act<A>` parses as `TypeExpr::Constructor { name: "Act", args: [A] }`.

### 3.4 Kind System

`Act` has kind `* -> *`. This is already expressible via `Kind::Arrow(vec![Kind::Type], Box::new(Kind::Type))` from SPEC-020. No kind system changes required.

## 4. Surface AST Changes

### 4.1 New Expression Variants in `surface.rs`

Add to `Expr` enum:

```rust
ActBlock {
    stmts: Vec<ActStmt>,
    span: Span,
},
```

### 4.2 Surface ActStmt

Mirror of core `ActStmt`:

```rust
pub enum ActStmt {
    Bind { name: Name, value: Box<Expr>, span: Span },
    Return { value: Box<Expr>, span: Span },
}
```

### 4.3 Dual-Context `act` Keyword

The keyword `act` currently dispatches to `act_stmt()` in `parse_workflow.rs` producing `Workflow::Act`. After this spec, `act` also dispatches in expression context producing `Expr::ActBlock`.

Parser dispatch rule:
- In workflow context: `act <action_ref> [where ...] [as ...] [then ...]` → `Workflow::Act` (unchanged)
- In expression context: `act { ... }` → `Expr::ActBlock` (new)

The distinguishing token is `{` after `act`. Workflow `act` never uses `{` (it uses `provider:action(args)`).

## 5. Type System Changes

### 5.1 Act Type Constructor

The type system already has `Type::Constructor { name, args, kind }`. `Act<A>` maps to:

```rust
Type::Constructor {
    name: "Act".into(),
    args: vec![A],
    kind: Kind::Arrow(vec![Kind::Type], Box::new(Kind::Type)),
}
```

### 5.2 Purity Enforcement

The type system must enforce:

```
fn f(x: A) -> B        -- body must not contain act {} blocks or invoke
fn f(x: A) -> Act B    -- body may contain act {} blocks and invoke
```

Implementation: during `check_expr`, if the enclosing function has pure return type (`B`, not `Act<B>`), reject `Expr::ActBlock` and `Expr::Invoke` with a type error.

### 5.3 Act Block Typing

```
Γ ⊢ e : Act a     Γ, x : a ⊢ rest : Act b
─────────────────────────────────────────────  (ACT-BIND)
Γ ⊢ act { x = e; rest } : Act b

Γ ⊢ e : a         Γ, x : a ⊢ rest : Act b
─────────────────────────────────────────────  (ACT-PURE-BIND)
Γ ⊢ act { x = e; rest } : Act b
  (e is inlined; no monadic step)

Γ ⊢ e : a
──────────────  (ACT-RETURN)
Γ ⊢ act { ret e } : Act a
```

### 5.4 Invoke Typing

```
Γ ⊢ provider : String   Γ ⊢ action : String   Γ ⊢ args : List
─────────────────────────────────────────────────────────────────  (ACT-INVOKE)
Γ ⊢ invoke(provider, action, args) : Act Value
```

The type `Act Value` is broad. Future refinements can use capability declarations to narrow the return type.

### 5.5 Bind Typing

```
Γ ⊢ ma : Act a    Γ ⊢ f : a -> Act b
──────────────────────────────────────  (ACT-BIND-CHECK)
Γ ⊢ bind(ma, f) : Act b
```

### 5.6 No New Type Variants

The existing `Type::Constructor` handles `Act<A>`. The existing `Type::Fun(args, ret, effect)` handles effectful function types for workflow-level code. For expression-level code, `Act<A>` as a return type is sufficient — the effect information is carried by the type constructor itself, not by a separate `Effect` annotation.

## 6. Lowerer Changes

### 6.1 New Expr Lowering

In `crates/ash-parser/src/lower.rs`, add match arms for:

- `SurfaceExpr::ActBlock { stmts, .. }` → `CoreExpr::ActBlock { stmts: lowered_stmts }`
- `SurfaceExpr::Invoke { .. }` → Not a surface form. `invoke` is a `builtin fn`, so it's lowered as `Expr::Call { func: "invoke", .. }`.

### 6.2 ActBlock Desugaring

The lowerer transforms `ActBlock` into nested `bind`/`unit` calls:

```rust
fn lower_act_block(stmts: Vec<ActStmt>) -> CoreExpr {
    match stmts.as_slice() {
        [] => panic!("empty act block"),
        [ActStmt::Return { value, .. }] => {
            // act { ret e } => unit(e) => call("unit", [e])
            CoreExpr::Call { func: "unit".into(), module: None, arguments: vec![lower_expr(value)] }
        }
        [ActStmt::Bind { name, value, .. }, rest @ ..] => {
            // Determine if bind or inline by checking value type
            // For MVP: always emit bind; type checker will inline pure cases
            let rest_expr = lower_act_block(rest.to_vec());
            CoreExpr::Call {
                func: "bind".into(),
                module: None,
                arguments: vec![
                    lower_expr(value),
                    CoreExpr::FnDef {
                        params: vec![name.clone()],
                        return_type: None,
                        body: Box::new(rest_expr),
                    },
                ],
            }
        }
        _ => panic!("invalid act stmt sequence"),
    }
}
```

**Optimization note:** Pure bindings (`n = len(x)`) will produce `bind(unit(len(x)), |n| rest)` which is correct by the left-identity monad law: `bind(unit(a), f) = f(a)`. A future optimization pass can eliminate the intermediate `unit`/`bind` pair.

## 7. Interpreter Changes

### 7.1 ActBlock Evaluation

Add to `eval_expr` in `crates/ash-interp/src/eval.rs`:

```rust
Expr::ActBlock { stmts, .. } => {
    // An act block in expression context produces a closure
    // representing the suspended computation.
    // The ActEnv is threaded at invoke time.
    Ok(Value::Closure {
        params: vec!["__env".into()],
        body: Box::new(desugared_act_body(stmts)),
        env: ctx.capture(),
    })
}
```

**Key design decision:** An `act {}` block in expression context produces a `Value::Closure` that takes an `ActEnv` argument. This is the concrete realization of `Act<A> ≈ ActEnv → Result<(A, ActEnv), ExecError>`.

### 7.2 Invoke as Builtin

Add `invoke` to the builtin dispatch table. Implementation:

```rust
fn builtin_invoke(args: &[Value], ctx: &mut EvalContext) -> EvalResult<Value> {
    // args[0] = provider name (String)
    // args[1] = action name (String)
    // args[2] = arguments (List)
    // Returns Act Value (a closure that, given ActEnv, invokes the provider)
    let provider = args[0].as_string()?;
    let action = args[1].as_string()?;
    let invoke_args = args[2].as_list()?;

    Ok(Value::Closure {
        params: vec!["__env".into()],
        body: Box::new(/* synthetic: calls cap_ctx.execute */),
        env: /* captures provider, action, invoke_args */,
    })
}
```

### 7.3 ActEnv Runtime Value

The runtime needs a new value type to represent the threaded environment:

```rust
pub struct ActEnv {
    pub capability_ctx: CapabilityContext,
    pub policies: PolicyStack,
    pub provenance: Provenance,
    pub effects: Vec<Effect>,
}
```

This is a Rust-only type, not an Ash value. It's passed implicitly through the monadic threading.

### 7.4 Workflow::Act Bridge

The existing `Workflow::Act` execution path (in `execute.rs`) continues to work unchanged. It operates at the workflow level with direct capability dispatch. The new `Expr::ActBlock` operates at the expression level through closures.

Bridge: When a workflow's `act` node encounters an expression-level `Act` value, the workflow executor applies it with the current `ActEnv`.

## 8. Engine Changes

### 8.1 Type Registration

Register `Act` as a built-in type constructor in the type environment:

```rust
// In type_env initialization:
type_env.register_type_constructor("Act", Kind::Arrow(vec![Kind::Type], Box::new(Kind::Type)));
```

### 8.2 Builtin Registration

Register `invoke`, `unit`, `bind`, `then`, `guard` as builtin functions in the engine.

### 8.3 ActEnv Construction

When executing a workflow that contains expression-level `Act` values, construct the `ActEnv` from the workflow's existing capability context, policy stack, and provenance.

## 9. Desugarer Changes

### 9.1 ActBlock in Workflow Context

The desugarer already handles workflow-level `act`. For expression-level `act {}` blocks inside workflow bodies (e.g., inside `Orient` expressions), no special desugaring is needed — the act block is an expression that produces a closure value.

### 9.2 ActBlock in Fn Context

Inside `fn` bodies, `act {}` blocks are expressions. The desugarer treats them like any other expression.

## 10. Changes by Spec Amendment

### SPEC-001 (IR)
- Add `Expr::ActBlock`, `Expr::Invoke` to core expression forms
- Add `ActStmt` type definition
- Note that `Act<A>` uses existing `TypeExpr::Constructor`

### SPEC-002 (Surface Syntax)
- Document `act { ... }` as expression form
- Document dual-context dispatch for `act` keyword
- Add grammar rules for act blocks in expression position

### SPEC-003 (Type System)
- Document `Act<A>` type constructor and kind
- Document purity enforcement rules
- Document act block typing rules

### SPEC-004 (Operational Semantics)
- Add semantic rules for `ACT-BIND`, `ACT-PURE-BIND`, `ACT-RETURN`, `ACT-INVOKE`
- Define `ActEnv` semantic domain
- Define monad laws as semantic invariants

### SPEC-025 (Small-Step Semantics)
- Add small-step reduction rules for expression-level `act {}` blocks
- Note that act blocks reduce to closure values

### SPEC-027 (Pure Functions)
- Amend purity definition: pure functions must not contain `act {}` blocks or `invoke`
- Add effectful function declaration form

### SPEC-031 (First-Class Functions)
- Note that closures may capture `ActEnv` (for effectful closures)
- Note that `Act<A>` is a closure type under the hood

### SPEC-BUILTIN-FN
- Add `invoke` as a builtin fn returning `Act Value`
- Add `unit`, `bind`, `then`, `guard` as builtin fns (or note them as library functions)

## 11. Deferred Items

1. **observe vs execute at type level.** `Act<A>` doesn't distinguish effect types. If governance needs type-level distinction: `Observe<A>` / `Execute<A>` as separate type constructors, or phantom type parameter `Act<Eff, A>`.

2. **Concurrent composition.** `par : Act<A> → Act<B> → Act<(A, B)>` runs computations concurrently and merges effect logs.

3. **Interface hierarchy.** `Functor`, `Applicative`, `Monad` as interfaces over `* → *` kind requires HKT support in the type system.

4. **Migration of stdlib .ash files.** Files like `std/src/io/fs.ash` contain `act execute` in workflow context. These remain valid. Files that should become `fn ... -> Act<T>` are a separate migration pass.

5. **Typed invoke.** `invoke` currently returns `Act Value`. Capability declarations could provide typed return types: `invoke(Fs, "read", [path]) : Act String`.

## 12. Semantic Correctness

### 12.1 Monad Laws

```
bind(unit(a), f)       = f(a)            -- left identity
bind(m, unit)          = m               -- right identity
bind(bind(m, f), g)    = bind(m, |x| bind(f(x), g))  -- associativity
```

Proof obligations:
- `unit(v)` produces `|env| Ok((v, env))` — identity on environment
- `bind` threads left-to-right, no reordering
- Effect log concatenation is associative; empty log is identity

### 12.2 Preservation Properties

1. **Sequential ordering:** Effects appear in execution order. `bind` threads left-to-right.
2. **Provenance propagation:** Each effect records its provenance chain.
3. **Append-only effect log:** No computation can remove effects.
4. **Provider immutability:** Providers registered at engine build time.
5. **Policy stacking:** Every `invoke` passes through the full policy stack.
6. **Failure short-circuits:** `bind` on `Err` skips the continuation.

## 13. Implementation Phases

### Phase 97 Track A: Foundation (estimated 20-25 hours)

1. Surface AST + Parser: `Expr::ActBlock`, `ActStmt`, expression-context `act {}` parsing
2. Core AST: `Expr::ActBlock`, `Expr::Invoke` (or `Call` with `invoke` func name)
3. Lowerer: ActBlock desugaring to nested `bind`/`unit` calls
4. Builtin registration: `invoke`, `unit`, `bind` in dispatch table

### Phase 97 Track B: Type System (estimated 15-20 hours)

5. Type registration: `Act` as type constructor with kind `* -> *`
6. Act block typing: bind, pure bind, return rules
7. Purity enforcement: reject `act {}` in pure fn bodies
8. Invoke typing: `String → String → List → Act Value`

### Phase 97 Track C: Runtime (estimated 15-20 hours)

9. ActEnv value type construction
10. `invoke` builtin implementation with capability dispatch
11. ActBlock evaluation: closure production and application
12. Workflow bridge: ActEnv construction from workflow context

### Phase 97 Track D: Specs + Testing (estimated 10-15 hours)

13. Spec amendments (SPEC-001/002/003/004/025/027/031/BUILTIN-FN)
14. Property tests: monad laws, purity enforcement, governance preservation
15. Integration tests: effectful fn composition, nested act blocks, workflow + act interop

Total estimated: 60-80 hours across 4 tracks.
