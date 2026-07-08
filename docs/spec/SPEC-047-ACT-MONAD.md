# SPEC-047: The Act Monad — First-Class Effectful Computation

**Status:** Draft
**Date:** 2026-04-22
**Related:** NOTE-005, SPEC-001, SPEC-004, SPEC-025, SPEC-027, SPEC-031, SPEC-BUILTIN-FN, SPEC-020
**Supersedes:** NOTE-005 (design exploration — this spec is its normative counterpart)

> **Target reconciliation.** This spec is the current-state Act/effectful
> computation substrate. Its references to capability providers are compatibility
> vocabulary for the legacy Act environment. Target authority planning should
> translate Act effects into computation-row operation/resource requirements
> discharged by provider/handler admission.

## 1. Overview

Introduce `Act<A>` as a first-class type constructor in the expression layer, adding a composable effectful-computation model that interoperates with the existing workflow runtime. An `Act<A>` value is a suspended computation that, given an environment of capability providers, policies, and provenance, may produce a value of type `A` alongside an accumulated effect log, or fail with an error.

The core friction resolved by Phase 97/105: `act` no longer exists only as a `Workflow` node. Expression-level `act { ... }` sugar lifts effectful computation into the expression layer while preserving all governance properties (sequential ordering, provenance, policy checking, audit trail).

### 1.1 Design Principles

1. **Minimal runtime primitive set.** `unit`, `bind`, `then`, and `guard` belong to the `act::` library algebra. `invoke` is the only required runtime primitive; Phase 97 may realize some algebra members through bridge-backed library exports where opacity requires it.
2. **Act is the outer marker of effectfulness.** Effectful APIs surface as `Act<A>` at the outermost type level. Domain-level failure remains an author-chosen convention inside that effectful result, e.g. `Act<Result<A, E>>`.
3. **Type-system purity boundary.** Functions returning `B` are pure. Functions returning `Act<B>` are effectful. The type system prevents calling effectful code from pure contexts.
4. **Governance preservation.** Every `invoke` passes through the policy stack. Effect logs are append-only. Provenance chains are maintained.
5. **Act opacity.** `Act` is composed and eliminated through effectful contexts (`act { ... }`, effectful function bodies, workflow sequencing, and runtime interpretation). Pure code may carry/combine `Act` values abstractly, but should not deconstruct raw `Act` representations directly.
6. **Workflow layering.** Workflows are intended to evolve into richer constructs built on effectful functions and `Act` sequencing, adding metadata/context rather than introducing a second sequencing foundation.
7. **Incremental delivery.** The spec is designed to be implemented in phases that leave the system working at each step.
8. **Runtime-managed state substrate.** `Act` is modeled after state-threading runtimes such as Haskell `IO`: the semantic substrate is a runtime-managed `ActEnv` carrier, not `Result` and not any public surrogate value encoding.

### 1.2 Scope

In scope:
- `Act<A>` type constructor in surface syntax and type system
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
- Replacing or retiring the existing `Type::Fun(...)` workflow-closure model
- Expression-level micro-stepping in SPEC-025

## 2. Surface Syntax

### 2.1 Act Block Expression

SPEC-054 now owns the generalized/new Act block grammar. In current Phase 105 syntax,
expression-level `act { ... }` is sugar for `do:Act { ... }` and uses the typed-do statement forms:

```ash
act {
    x <- act::unit(1);
    return x
}
```

This is equivalent to:

```ash
do:Act {
    x <- act::unit(1);
    return x
}
```

Phase 201 removes the legacy SPEC-047 statement grammar from current Ash. Historically, that
grammar allowed statement-style act blocks with ambiguous assignment-like binds and `ret`
statements. Current implementations reject those forms. Migration guidance should direct users to
target bind/let forms plus final `return expr`. Removed workflow-level act statements are
historical and are not governed by this expression grammar.

### 2.2 Effectful Function Declaration

```ash
fn read(path: String) -> Act<String> {
    act {
        result <- invoke("Fs", "read", [path]);
        return result
    }
}
```

The return type `Act<String>` distinguishes this from a pure function. The body must produce a value of type `Act<String>` — either an `act {}` block, a call to another effectful function, or `invoke` directly.

### 2.3 Bind Desugaring

```ash
act {
    x <- read(path);        -- bind (RHS : Act<String>, x : String)
    let n = len(x);         -- ordinary pure lexical binding
    y <- parse(x);          -- bind (RHS : Act<Value>, y : Value)
    return (x, n, y)        -- unit
}
```

Desugars to:

```
bind(read(path), |x|
  bind(parse(x), |y|
    unit((x, len(x), y))))
```

Note: `let n = len(x);` remains an ordinary lexical binding and does not introduce a monadic step.

### 2.4 Invoke Expression

The primitive effectful operation:

```
invoke(provider: String, action: String, args: List<Value>) -> Act<Value>
```

Normatively, `invoke` is a runtime primitive callable that travels through the existing `Expr::Call`
path. It is not modeled as a dedicated core `Expr::Invoke` variant, and it is not an ordinary pure
`builtin fn` in the current SPEC-BUILTIN-FN sense.

### 2.5 Library Functions

These are library-level `Act` operations exposed at the `act::` boundary.

Normative surface rules:
- `Act` is the exported effect marker and remains opaque as a public abstraction; user/library code should compose it through the `act::` algebra rather than rely on raw representation deconstruction.
- `unit`, `bind`, `then`, `map`, `apply`, `sequence`, and `traverse` belong to the `act::` algebra.
- Convenience helpers specialized for `Act<Result<...>>` belong under `act::result::...` rather than as suffixed global helpers.
- Pure code may transport and combine `Act` values abstractly, but elimination/inspection happens only after sequencing in effectful contexts.

Illustrative equations:

These equations are normative algebraic semantics, not a requirement that every operation be immediately expressible as raw ordinary Ash definitions under the current Phase-97 substrate.

```ash
fn unit(v: a) -> Act<a> { ... }
fn bind(ma: Act<a>, f: Fn(a) -> Act<b>) -> Act<b> { ... }
fn then(ma: Act<a>, mb: Act<b>) -> Act<b> { ... }
fn guard(policy: Policy, ma: Act<a>) -> Act<a> { ... }
```

`Act<Result<A, E>>` is the preferred conventional shape for effectful computations that also return a domain-level success/failure result. `Result<Act<A>, E>` is reserved for the distinct case where computation construction fails before an effectful computation can be obtained.

### 2.5.1 Builtin Substrate Contract

Phase 97 treats the `Act` substrate as runtime/engine managed.

Preferred semantic reading:

```ash
builtin type ActEnv
type Act<A> = ActEnv -> (ActEnv, A)
```

Interpretation rule:
- the explicit RHS is preferred as a definitional semantic equation when implementation pressure permits;
- if the real parser/typechecker/engine/runtime shows that literal definitional equality creates substantial complexity or fragility, the implementation may downgrade this to a checked normative correspondence while preserving the same public laws and observable typing/composition behavior.

Builtin-boundary fallback ladder:
- A (preferred): `builtin type ActEnv`; ordinary `type Act<A> = ActEnv -> (ActEnv, A)`
- B (fallback): `builtin type ActEnv`; `builtin type Act<A> = ActEnv -> (ActEnv, A)`
- C (last resort): `builtin type ActEnv`; fully opaque builtin `Act<A>` without exposing the equation directly

Selection rule:
- choose A unless real implementation pressure in the parser/typechecker/engine/runtime makes A materially riskier or more complex than B;
- choose B unless B itself becomes materially riskier or more complex than C;
- C carries the most debt and is the last resort.

Builtin artifact identity rule:
- builtin artifacts are identified internally by flat builtin IDs, not by surface module paths;
- surface names such as `act::ActEnv`, reexports, or aliases are only bindings onto that internal builtin identity.

### 2.6 Keyword Choice

The block keyword is `act` — consistent with existing workflow `act` syntax and NOTE-005. Alternative names (`do`, `perform`) are equivalent in semantics; `act` preserves continuity.

## 3. Core IR Changes

### 3.1 Core IR Boundary

Phase 97 does not require adding `ActBlock`, `ActStmt`, or `Invoke` to the canonical core IR.
Expression-level `act { ... }` is now either generalized typed-do sugar (`Expr::DoBlock`
targeting `Act`) or a legacy migration carrier (`Expr::ActBlock`). New-form blocks lower only after
typechecker-owned typed elaboration; the legacy carrier still lowers into existing core expression
forms such as `Expr::Call`, `Expr::FnDef`, and `Expr::FnApply`.
This keeps the initial implementation additive and minimizes churn in `ash-core`.

### 3.2 TypeExpr Addition

Add to `crates/ash-core/src/ast.rs` `TypeExpr` enum:

```rust
/// Act<A> — an effectful computation yielding A
Constructor { name, args } already supports this.
```

No new `TypeExpr` variant needed. `Act<A>` parses as `TypeExpr::Constructor { name: "Act", args: [A] }`.

### 3.3 Kind System

`Act` has kind `* -> *`. This is already expressible in the current kind system as
`Kind::Arrow(Box::new(Kind::Type), Box::new(Kind::Type))`. No kind-system changes are required.

## 4. Surface AST Changes

### 4.1 Expression Variants in `surface.rs`

Phase 97 originally added a surface carrier for the now-removed statement-style act block:

```rust
ActBlock {
    stmts: Vec<ActStmt>,
    span: Span,
},
```

### 4.2 Surface ActStmt

Surface-only lowering carrier:

```rust
pub enum ActStmt {
    Bind { name: Name, value: Box<Expr>, span: Span },
    Return { value: Box<Expr>, span: Span },
}
```

`ActStmt` is now a migration carrier only. SPEC-054 owns the generalized `Expr::DoBlock` surface
node for new `act { ... }` and explicit `do:K { ... }` grammar.

### 4.3 Dual-Context `act` Keyword

The keyword `act` dispatches to `act_stmt()` in `parse_workflow.rs` in workflow context, producing `Workflow::Act`. In expression context, `act { ... }` now dispatches through the generalized typed-do grammar when it uses `let`/`<-`/`return`, with the legacy `Expr::ActBlock` path retained only for migration syntax.

Parser dispatch rule:
- In workflow context: `act <action_ref> [where ...] [as ...] [then ...]` → `Workflow::Act` (unchanged)
- In expression context: new-form `act { ... }` → `Expr::DoBlock` with target `Act`; legacy migration form → `Expr::ActBlock`

The distinguishing token is `{` after `act`. Workflow `act` never uses `{` (it uses `provider:action(args)`).

## 5. Type System Changes

### 5.1 Act Type Constructor

The type system already has `Type::Constructor { name, args, kind }`. `Act<A>` maps to:

```rust
Type::Constructor {
    name: "Act".into(),
    args: vec![A],
    kind: Kind::Arrow(Box::new(Kind::Type), Box::new(Kind::Type)),
}
```

### 5.2 Purity Enforcement

The type system must enforce:

```
fn f(x: A) -> B         -- body must not contain act {} blocks or invoke
fn f(x: A) -> Act<B>    -- body may contain act {} blocks and invoke
```

Implementation: during `check_expr`, if the enclosing function has pure return type (`B`, not `Act<B>`), reject effectful `Expr::DoBlock`/legacy `Expr::ActBlock` and expression-level `invoke(...)` calls with a type error.

### 5.3 Act Block Typing

```
Γ ⊢ e : Act a     Γ, x : a ⊢ rest : Act b
─────────────────────────────────────────────  (ACT-BIND)
Γ ⊢ act { x <- e; rest } : Act b

Γ ⊢ e : a         Γ, x : a ⊢ rest : Act b
─────────────────────────────────────────────  (ACT-PURE-BIND)
Γ ⊢ act { let x = e; rest } : Act b
  (ordinary lexical binding; no monadic step)

Γ ⊢ e : a
──────────────  (ACT-RETURN)
Γ ⊢ act { return e } : Act a
```

### 5.4 Invoke Typing

```
Γ ⊢ provider : String   Γ ⊢ action : String   Γ ⊢ args : List<Value>
─────────────────────────────────────────────────────────────────  (ACT-INVOKE)
Γ ⊢ invoke(provider, action, args) : Act<Value>
```

The type `Act<Value>` is broad. Future refinements can use capability declarations to narrow the return type.

### 5.5 Bind Typing

```
Γ ⊢ ma : Act a    Γ ⊢ f : a -> Act b
──────────────────────────────────────  (ACT-BIND-CHECK)
Γ ⊢ bind(ma, f) : Act b
```

### 5.6 Phase-97 Coexistence with `Type::Fun`

The existing `Type::Constructor` handles `Act<A>`. Phase 97 is additive with respect to the
existing `Type::Fn(...)` / `Type::Fun(...)` split:

- `Act<A>` is the new expression-level effectful computation type constructor.
- Existing `Type::Fun(args, ret, effect)` remains in place for current workflow-context closure
  classification and the already-promoted three-vertex boundary.
- Phase 97 does not retire or redefine `Type::Fun(...)`; later phases may revisit that architecture.

## 6. Lowerer Changes

### 6.1 New Expr Lowering

In Phase 105, raw parser lowering for new generalized `Expr::DoBlock` rejects and callers must use
typechecker-owned typed elaboration. Historical statement-style act blocks used the removed
surface carrier:

- `SurfaceExpr::ActBlock { stmts, .. }` → desugared nested core expressions using existing
  `CoreExpr::Call`, `CoreExpr::FnDef`, and `CoreExpr::FnApply`.

`invoke` is not a dedicated surface AST form. It parses as an ordinary call expression and lowers
through the existing `Expr::Call { func: "invoke", .. }` path.

### 6.2 ActBlock Desugaring

The legacy lowerer transforms `ActBlock` into nested `bind`/`unit` calls:

```rust
fn lower_act_block(stmts: Vec<ActStmt>) -> CoreExpr {
    match stmts.as_slice() {
        [] => panic!("empty act block"),
        [ActStmt::Return { value, .. }] => {
            // legacy act { ret e; } => unit(e) => call("unit", [e])
            CoreExpr::Call { func: "unit".into(), module: None, arguments: vec![lower_expr(value)] }
        }
        [ActStmt::Bind { name, value, .. }, rest @ ..] => {
            // Phase 97 desugaring target. Type-directed optimization is deferred.
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

**Migration note:** new syntax uses `let n = len(x);` for pure lexical binding rather than legacy
`n = len(x);` heuristics.

## 7. Interpreter Changes

### 7.1 ActBlock Evaluation

Because typed-do elaboration or legacy lowering removes expression-level Act block syntax before core evaluation, `eval_expr` does not require a dedicated `Expr::DoBlock`/`Expr::ActBlock` arm. The evaluator sees only the elaborated/desugared core expression forms.

Operationally, an expression-level `Act<A>` value is realized as a closure-shaped runtime value that
threads an internal `ActEnv`.

### 7.2 Invoke Runtime Primitive

Add `invoke` to the runtime callable dispatch path. It is treated as a distinguished runtime
primitive callable routed through `Expr::Call`, not as a pure builtin function under the current
SPEC-BUILTIN-FN contract. Implementation:

```rust
fn runtime_invoke(args: &[Value], ctx: &mut EvalContext) -> EvalResult<Value> {
    // args[0] = provider name (String)
    // args[1] = action name (String)
    // args[2] = arguments (List<Value>)
    // Returns Act<Value> (a closure that, given ActEnv, invokes the provider)
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

The existing `Workflow::Act` execution path (in `execute.rs`) continues to work unchanged in Phase 97.

Normative design direction:
- effect sequencing semantics live in `Act` / effectful functions
- workflows are intended to evolve into richer constructs built on top of that substrate
- workflow syntax may add metadata, authority/role context, provenance/policy framing, and orchestration conveniences
- workflows should not introduce a second competing sequencing foundation

Workflow execution still operates at the workflow level with direct capability dispatch for practical implementation reasons. Expression-level `act {}` interoperates with that runtime, but does not replace workflow execution wholesale.

Bridge: when workflow-level execution encounters an expression-level `Act<A>` value, the runtime may apply it with the current `ActEnv`.

## 8. Engine Changes

### 8.1 Type Registration

Register `Act` as a built-in type constructor in the type environment:

```rust
// In type_env initialization:
type_env.register_type_constructor("Act", Kind::Arrow(Box::new(Kind::Type), Box::new(Kind::Type)));
```

### 8.2 Registration Boundary

- Register `Act` as a recognized type constructor in the type environment.
- Register `invoke` as the runtime primitive callable used by expression-level effectful code.
- Do not register `unit`, `bind`, `then`, or `guard` as runtime builtins in Phase 97; they remain
  library functions.

### 8.3 ActEnv Construction

When executing a workflow that contains expression-level `Act` values, construct the `ActEnv` from the workflow's existing capability context, policy stack, and provenance.

## 9. Desugarer Changes

### 9.1 ActBlock in Workflow Context

Historical workflow-level `act` statement handling is no longer a current source path. Target
expression-level `act {}` blocks must pass through typed-do elaboration; removed legacy lowering
boundaries are retained only as migration history.

### 9.2 ActBlock in Fn Context

Inside `fn` bodies, current `act {}` blocks are expressions and typed-do sugar targeting `Act`.
Legacy blocks are removed rather than accepted as migration carriers.

## 10. Changes by Spec Amendment

### SPEC-001 (IR)
- Clarify that Phase-97 expression-level `act {}` is a surface construct lowered into existing core expression forms
- Note that `Act<A>` uses existing `TypeExpr::Constructor`

### SPEC-002 (Surface Syntax)
- Document `act { ... }` as expression form
- Document dual-context dispatch for `act` keyword
- Add grammar rules for act blocks in expression position

### SPEC-003 (Type System)
- Document `Act<A>` type constructor and kind
- Document purity enforcement rules
- Document act block typing rules
- Document Phase-97 coexistence with the existing `Type::Fun(...)` model

### SPEC-004 (Operational Semantics)
- Add semantic rules for `ACT-BIND`, `ACT-PURE-BIND`, `ACT-RETURN`, `ACT-INVOKE`
- Define `ActEnv` semantic domain
- Define monad laws as semantic invariants

### SPEC-025 (Small-Step Semantics)
- No Phase-97 amendment required. Expression-level micro-stepping remains out of scope under the current frozen workflow-first small-step contract.

### SPEC-027 (Pure Functions)
- Amend purity definition: pure functions must not contain `act {}` blocks or `invoke`
- Add effectful function declaration form

### SPEC-031 (First-Class Functions)
- Note that closures may capture `ActEnv` (for effectful closures)
- Clarify that Phase 97 does not retire the existing `Type::Fun(...)` workflow-closure model

### SPEC-BUILTIN-FN
- No direct Phase-97 amendment required if `invoke` is treated as a separate runtime primitive callable rather than a pure builtin fn.
- `unit`, `bind`, `then`, and `guard` remain library functions.

## 11. Deferred Items

1. **observe vs execute at type level.** `Act<A>` doesn't distinguish effect types. If governance needs type-level distinction: `Observe<A>` / `Execute<A>` as separate type constructors, or phantom type parameter `Act<Eff, A>`.

2. **Concurrent composition.** `par : Act<A> → Act<B> → Act<(A, B)>` runs computations concurrently and merges effect logs.

3. **Interface hierarchy.** `Functor`, `Applicative`, `Monad` as interfaces over `* → *` kind requires HKT support in the type system.

4. **Migration of stdlib .ash files.** Files like `std/src/io/fs.ash` contain `act execute` in workflow context. These remain valid. Files that should become `fn ... -> Act<T>` are a separate migration pass.

5. **Typed invoke.** `invoke` currently returns `Act<Value>`. Capability declarations could provide typed return types: `invoke(Fs, "read", [path]) : Act<String>`.

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

### Phase 97 Track A: Surface + Lowering Foundation (estimated 12-16 hours)

1. Surface AST + Parser: `Expr::ActBlock`, `ActStmt`, expression-context `act {}` parsing
2. Lowerer: ActBlock desugaring to existing core expressions using `bind`/`unit`
3. Runtime-call registration: `invoke` as expression-level primitive callable

### Phase 97 Track B: Type System (estimated 12-16 hours)

4. Type registration: `Act` as type constructor with kind `* -> *`
5. Act block typing: bind, pure bind, return rules
6. Purity enforcement: reject `act {}` in pure fn bodies
7. Invoke typing: `String → String → List<Value> → Act<Value>`
8. Phase-97 coexistence note: retain existing `Type::Fun(...)` model unchanged

### Phase 97 Track C: Runtime (estimated 12-16 hours)

9. `ActEnv` runtime construction
10. `invoke` runtime primitive implementation with capability dispatch
11. Desugared `Act<A>` execution path: closure production and application
12. Workflow bridge: `ActEnv` construction from workflow context

### Phase 97 Track D: Specs + Testing (estimated 8-12 hours)

13. Spec amendments (SPEC-002/003/004/027 and targeted clarifications in SPEC-001/031)
14. Property tests: monad laws, purity enforcement, governance preservation
15. Integration tests: effectful fn composition, nested act blocks, workflow + act interop

Total estimated: 44-60 hours across 4 tracks.
