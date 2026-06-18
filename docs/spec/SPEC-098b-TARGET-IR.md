---
id: spec.ash.ir.target
title: Ash Intermediate Representation — Target State
description: Target IR with unified effect rows, effect item identities, and a shared computation substrate
kind: spec
audience: [human, agent]
authority: design
status: draft
stability: alpha
owner: language
last_verified: 2026-06-18
verified_against:
  specs:
    - docs/spec/SPEC-095a-CURRENT-GRAMMAR.md
    - docs/spec/SPEC-095b-TARGET-GRAMMAR.md
    - docs/spec/SPEC-096a-CURRENT-EFFECT-SYSTEM.md
    - docs/spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md
    - docs/spec/SPEC-097a-CURRENT-TYPE-SYSTEM.md
    - docs/spec/SPEC-097b-TARGET-TYPE-SYSTEM.md
    - docs/spec/SPEC-098a-CURRENT-IR.md
---

# SPEC-098b: Ash Intermediate Representation — Target State

**Status:** Draft — target IR for unified effect rows
**Scope:** This document defines the IR representation we want Ash to have.
It is a goal-state living document that will be refined as implementation progresses.
**Depends on:** SPEC-095b (Target Grammar), SPEC-096b (Target Effect System), SPEC-097b (Target Type System)

## 1. Summary

The target IR unifies Ash's computation representation into a **CPS (Continuation-Passing Style)**
form with effect-row annotations. Every computation is a function that takes a continuation
(represented as a closure or label) and never returns directly.

Key design decisions:

1. **CPS form**: All expressions are in CPS. A function `A -> {r} B` becomes `A -> (B -> {r} C) -> {r} C`.
2. **EffectRow on continuations**: The continuation itself carries the effect row, ensuring
   effect requirements propagate through the call chain.
3. **Unified computation type**: `Act`, `Proc`, and `Workflow` are views over a shared CPS
   substrate, distinguished only by their row profile.
4. **Effect item identities and namespaces**: Every effect item has a canonical identity.
5. **Contract effect nodes**: Static/evidence/dynamic discharge status is tracked in the IR.
6. **Handler stack as CPS frames**: Handlers are continuations installed at explicit boundaries.
7. **Backward compatibility**: Legacy AST variants are lowered to CPS during migration.

## 2. Target AST Types

### 2.1 CPS Expression AST

In CPS form, every expression takes a continuation `k` and produces a result by invoking `k`.
There is no direct return. The continuation is a first-class value (a closure or label) that
carries its own effect row.

```rust
pub enum Expr {
    -- Primitive values (no continuation needed)
    Lit(Literal),
    Var(Name),
    PrimOp { op: PrimOp, args: Vec<Atom> },

    -- CPS application: apply a function to arguments and a continuation
    App {
        func: Atom,
        args: Vec<Atom>,
        cont: Atom,           -- continuation: A -> {r} C
        row: EffectRow,       -- effect row of this application
    },

    -- CPS abstraction: a function that takes a continuation
    Lam {
        params: Vec<Param>,   -- ordinary parameters
        cont_param: Param,    -- continuation parameter k
        body: Box<Expr>,
        row: EffectRow,       -- effect row of the function body
    },

    -- Let-binding (administrative normal form)
    Let {
        name: Name,
        value: Box<Expr>,     -- must be a value or primitive op
        body: Box<Expr>,
    },

    -- Effect raise: invoke the current continuation with an effect request
    Raise {
        effect: EffectItem,
        args: Vec<Atom>,
        cont: Atom,           -- continuation to resume after handling
        row: EffectRow,
    },

    -- Handler boundary: install a handler around a body
    Handle {
        effect: EffectItem,
        handler: HandlerDef,  -- handler implementation
        body: Box<Expr>,      -- body to execute under the handler
        cont: Atom,           -- continuation after the handler scope
        row: EffectRow,
    },

    -- Conditional (CPS branch)
    If {
        cond: Atom,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
        cont: Atom,
        row: EffectRow,
    },

    -- Record/tuple construction (atomic)
    Record { fields: Vec<(Name, Atom)> },
    Tuple { elems: Vec<Atom> },

    -- Field access (atomic)
    Field { record: Atom, field: Name },

    -- Legacy compatibility aliases (lowered to CPS during migration)
    Act { ... },
    Do { ... },
    Proc { ... },
    Workflow { ... },
}

pub enum Atom {
    Var(Name),
    Lit(Literal),
}
```

**Key CPS invariants:**

1. Every non-atomic expression ends with an `App`, `Raise`, `Handle`, or `If` that invokes a continuation.
2. Functions never return; they always invoke their continuation parameter.
3. The continuation parameter is the last parameter of every `Lam`.
4. Effect rows are attached to every `App`, `Lam`, `Raise`, and `Handle` node.

### 2.2 CPS Function Type

In CPS, a function type is:

```rust
pub enum Type {
    -- ... existing variants ...

    -- CPS function type: (params, cont) -> {row} result
    CpsFn {
        params: Vec<Type>,           -- ordinary parameters
        cont: Box<Type>,             -- continuation type: B -> {row} C
        ret: Box<Type>,              -- result type C (the "final answer")
        row: EffectRow,              -- effect row of the function
    },

    -- Continuation type: a function that takes a value and returns a final answer
    Cont {
        arg: Box<Type>,              -- argument type
        ret: Box<Type>,              -- final answer type
        row: EffectRow,              -- effect row of the continuation
    },

    -- Effect row type
    EffectRow {
        items: Vec<EffectItem>,
        tail: Option<RowVar>,
    },

    -- Legacy compatibility
    Fn { params, ret },             -- pure function, lowered to CpsFn with empty row
    Fun { params, ret, effect },    -- effectful callable, lowered to CpsFn
}
```

A surface function `fn f(x: A) -> {r} B { ... }` lowers to a CPS function:

```text
f : (A, (B -> {r} C)) -> {r} C
```

where `C` is the final answer type (often `Unit` or a top-level computation result).

### 2.3 CPS Value Types

In CPS, a "computation" is not a separate value type. It is simply a CPS function that takes
a continuation. The runtime tracks effect rows through the continuation closure environment.

```rust
pub enum Value {
    -- ... existing variants ...

    -- CPS closure: a function with its environment and continuation
    CpsClosure {
        params: Vec<Param>,
        cont_param: Param,
        body: Box<Expr>,
        env: Env,
        row: EffectRow,
    },

    -- Continuation: a special closure that never returns, only invokes its own continuation
    Cont {
        arg: Param,
        body: Box<Expr>,
        env: Env,
        row: EffectRow,
    },

    -- Handler frame: a continuation that intercepts specific effects
    HandlerFrame {
        effect: EffectItem,
        handler: HandlerDef,
        next: Box<Value>,  -- the next continuation in the chain
    },

    -- Legacy compatibility aliases
    ActEnvToken(...),  -- preserved during migration, lowered to CpsClosure
    Proc(...),         -- preserved during migration, lowered to CpsClosure
}
```

**Key CPS value invariants:**

1. `CpsClosure` always has a `cont_param` as its last parameter.
2. `Cont` never returns; it always invokes another continuation.
3. `HandlerFrame` wraps a continuation and intercepts matching `Raise` nodes.
4. Effect rows are part of the closure environment, not separate runtime values.

## 3. Effect Row Representation

### 3.1 Row Carrier

```rust
pub struct EffectRow {
    pub items: Vec<EffectItem>,
    pub tail: Option<RowVar>,
}

pub struct RowVar {
    pub name: Name,
    pub constraints: Vec<RowConstraint>,
}
```

### 3.2 Effect Item Identity

```rust
pub enum EffectItem {
    Capability(CapabilityEffect),
    Resource(ResourceEffect),
    Role(RoleEffect),
    Policy(PolicyEffect),
    Contract(ContractEffect),
    Channel(ChannelEffect),
    Process(ProcessEffect),
    Failure(FailureEffect),
    Evidence(EvidenceEffect),
    Group(EffectGroupRef),
}
```

See SPEC-097b for the full type definitions.

## 5. Lowering Pipeline

### 5.1 Target Lowering

```text
surface AST (with effect rows)
    |
    v
lower.rs -- lowers to unified IR
    |
    v
core AST (with EffectRow)
    |
    v
type checker (with row discharge)
    |
    v
interpreter (with handler stack)
```

### 5.2 CPS Lowering Rules

| Surface | CPS Target IR |
|---------|---------------|
| `do { ... }` | `Lam { params: [], cont_param: k, body: [lowered body], row: inferred }` |
| `do:Act { ... }` | `Lam { params: [], cont_param: k, body: [lowered body], row: Act_profile }` |
| `do:Proc { ... }` | `Lam { params: [], cont_param: k, body: [lowered body], row: Proc_profile }` |
| `do:Workflow { ... }` | `Lam { params: [], cont_param: k, body: [lowered body], row: Workflow_profile }` |
| `act { ... }` | `Lam { params: [], cont_param: k, body: [lowered body], row: Act_profile }` |
| `workflow { ... }` | `Lam { params: [], cont_param: k, body: [lowered body], row: Workflow_profile }` |
| `fn f(x: A) -> {r} B { body }` | `Lam { params: [x], cont_param: k, body: [lowered body], row: r }` |
| `handle E with { ... }` | `Handle { effect: E, handler: H, body: [lowered body], cont: k, row: r }` |
| `raise E(args)` | `Raise { effect: E, args: [lowered args], cont: k, row: r }` |
| `f(x)` | `App { func: f, args: [x], cont: k, row: r }` |
| `if c then t else e` | `If { cond: c, then_branch: [t], else_branch: [e], cont: k, row: r }` |
| `let x = v in e` | `Let { name: x, value: [v], body: [e] }` |

**CPS lowering examples:**

A pure function:
```ash
fn add(a: Int, b: Int) -> Int { a + b }
```
lowers to:
```text
add = Lam { params: [a, b], cont_param: k,
            body: App { func: PrimOp(Add), args: [a, b], cont: k, row: {} } }
```

An effectful function (using `do` notation):
```ash
fn read_config(path: String) -> {cap fs.read} String {
    do { contents <- fs.read(path); return contents }
}
```
lowers to:
```text
read_config = Lam { params: [path], cont_param: k, row: {cap fs.read},
    body: App { func: fs.read, args: [path],
                cont: Lam { params: [contents], cont_param: k2,
                            body: App { func: k2, args: [contents], cont: k, row: {cap fs.read} } } } }
```

A simple effectful function (no `do` notation, just a direct capability call):
```ash
fn get_user_name(id: Int) -> {cap db.read} String {
    db.read("users", id)
}
```
lowers to:
```text
get_user_name = Lam { params: [id], cont_param: k, row: {cap db.read},
    body: App { func: db.read, args: ["users", id], cont: k, row: {cap db.read} } }
```

This is the minimal case: a function with an effect row that simply invokes a capability
and passes the result directly to the continuation. No `do` block, no bind, no local variables.
The only difference from a pure function is the non-empty effect row on the `Lam` and the `App`.

A handler boundary:
```ash
handle requires {b != 0} with {
    requires -> if b != 0 then () else return 0
} in {
    a / b
}
```
lowers to:
```text
Handle { effect: Contract(requires {b != 0}),
         handler: Lam { params: [], cont_param: resume,
                        body: If { cond: b != 0,
                                   then_branch: App { func: resume, args: [()], cont: k, row: {} },
                                   else_branch: App { func: k, args: [0], cont: k, row: {} } } },
         body: App { func: PrimOp(Div), args: [a, b], cont: k, row: {} },
         cont: k, row: {} }
```

### 5.3 Handler Patterns as Lowering Examples

The following patterns show how common control-flow patterns are expressed as CPS handler
combinations. They use only the core IR nodes (`Handle`, `Lam`, `App`, `If`, `Let`) and do
not require special-purpose IR constructs.

#### Retry Pattern

A retry boundary re-executes a computation up to a maximum number of attempts:

```ash
fn fetch_with_retry(url: String) -> {cap http.get} Result<String, NetworkError> {
    retry max_attempts: 3, backoff_ms: 1000 {
        http.get(url)
    } handle {
        NetworkError => retry;           -- retry the body
        _ => fail UnrecoverableError;    -- exhausted attempts
    }
}
```

Lowers to a recursive CPS wrapper that re-invokes the body on failure:

```text
fetch_with_retry = Lam { params: [url], cont_param: k, row: {cap http.get},
    body: Let { name: retry_loop,
                value: Lam { params: [attempts_left], cont_param: k_retry,
                    body: If { cond: attempts_left > 0,
                        then_branch:
                            -- attempt the body
                            App { func: http.get, args: [url],
                                cont: Lam { params: [result], cont_param: k_body,
                                    body: If { cond: success(result),
                                        then_branch: App { func: k, args: [result], cont: k_retry, row: {cap http.get} },
                                        else_branch:
                                            -- wait then retry
                                            App { func: wait, args: [1000],
                                                cont: Lam { params: [_], cont_param: k_wait,
                                                    body: App { func: retry_loop, args: [attempts_left - 1], cont: k_retry, row: {cap http.get} } } } } },
                        else_branch: App { func: k, args: [Err(UnrecoverableError)], cont: k_retry, row: {} } } },
                body: App { func: retry_loop, args: [3], cont: k, row: {cap http.get} } } }
```

Key points:
- `retry_loop` is a recursive CPS function (a `Lam` that calls itself via `App`).
- The body (`http.get`) is re-invoked on each retry by applying `retry_loop` with a decremented counter.
- The wait is a tail call: `wait(backoff, λ_. retry_loop(n-1, k))`.
- No special `Retry` IR node. The pattern is a recursive wrapper around the body.

#### Rollback / Compensation Pattern

A compensation pair executes a primary action and, on failure, runs an undo action:

```ash
fn transfer_funds(from: Account, to: Account, amount: Int)
    -> {cap db.read, cap db.write} Result<Unit, TxError>
{
    compensate {
        do { reserve <- payment.reserve(from, amount); return reserve }
    } undo {
        payment.release(from, amount);
    } in {
        do {
            transfer <- payment.transfer(from, to, amount);
            return transfer
        }
    }
}
```

Lowers to a sequence where the undo action is threaded through the failure path:

```text
transfer_funds = Lam { params: [from, to, amount], cont_param: k, row: {cap db.read, cap db.write},
    body: App { func: payment.reserve, args: [from, amount],
        cont: Lam { params: [reserve], cont_param: k_reserve,
            body: If { cond: success(reserve),
                then_branch:
                    -- primary action
                    App { func: payment.transfer, args: [from, to, amount],
                        cont: Lam { params: [transfer], cont_param: k_transfer,
                            body: If { cond: success(transfer),
                                then_branch: App { func: k, args: [Ok(())], cont: k_transfer, row: {cap db.write} },
                                else_branch:
                                    -- undo reserve, then fail
                                    App { func: payment.release, args: [from, amount],
                                        cont: Lam { params: [_], cont_param: k_release,
                                            body: App { func: k, args: [Err(TxError)], cont: k_release, row: {cap db.write} } } } } },
                else_branch: App { func: k, args: [Err(TxError)], cont: k_reserve, row: {} } } } } }
```

Key points:
- The undo action (`payment.release`) is inlined into the failure path of the primary action.
- The compensation sequence is explicit in the continuation chain: on failure, the continuation
  invokes the undo before returning the error to the outer continuation `k`.
- For multi-step sagas, the undo chain grows backwards: `undo_n(..., undo_{n-1}(..., k(error)))`.
- No special `Compensate` IR node. The pattern is a sequence of `If` branches with undo actions
  in the failure arms.

#### Transactional Memory Pattern

A transaction boundary intercepts read and write effects, logs them in a transaction-local
read/write set, and either commits or aborts:

```ash
fn update_balance(account: Account, delta: Int)
    -> {transaction {isolation: serializable}, cap db.read, cap db.write} Result<Unit, TxError>
{
    transaction {
        let balance = db.read(account);
        db.write(account, balance + delta);
    }
}
```

Lowers to nested `Handle` frames that intercept `db.read` and `db.write`:

```text
update_balance = Lam { params: [account, delta], cont_param: k, row: {transaction {isolation: serializable}, cap db.read, cap db.write},
    body: Let { name: tx_log, value: new_log(),
        body: Handle {
            effect: cap db.read,
            handler: Lam { params: [key], cont_param: resume,
                body: If { cond: tx_log.has_write(key),
                    then_branch: App { func: resume, args: [tx_log.get_write(key)], cont: resume, row: {transaction {isolation: serializable}} },
                    else_branch: Let { name: value, value: read_from_store(key),
                        body: Let { name: _, value: tx_log.add_read(key, value),
                            body: App { func: resume, args: [value], cont: resume, row: {transaction {isolation: serializable}} } } } } },
            body: Handle {
                effect: cap db.write,
                handler: Lam { params: [key, value], cont_param: resume,
                    body: Let { name: _, value: tx_log.add_write(key, value),
                        body: App { func: resume, args: [()], cont: resume, row: {transaction {isolation: serializable}} } } },
                body: App { func: db.read, args: [account],
                    cont: Lam { params: [balance], cont_param: k_read,
                        body: App { func: db.write, args: [account, balance + delta],
                            cont: Lam { params: [_], cont_param: k_write,
                                body: If { cond: validate(tx_log),
                                    then_branch: Let { name: _, value: commit(tx_log),
                                        body: App { func: k, args: [Ok(())], cont: k_write, row: {transaction {isolation: serializable}} } },
                                    else_branch: Let { name: _, value: abort(tx_log),
                                        body: App { func: k, args: [Err(TxConflict)], cont: k_write, row: {transaction {isolation: serializable}} } } } } },
                cont: k, row: {transaction {isolation: serializable}} },
            cont: k, row: {transaction {isolation: serializable}} } } }
```

Key points:
- Two nested `Handle` frames intercept `db.read` and `db.write`.
- The `tx_log` is a mutable cell threaded through the handler closures.
- Reads check the write log first (read-your-own-writes), then the read log, then the store.
- Writes are logged but not applied until commit.
- Validation and commit happen at the end of the transaction body.
- On conflict, the transaction aborts and returns `Err(TxConflict)`. A retry loop would wrap
  this in the pattern from the retry example above.
- No special `Transaction` IR node. The pattern is nested `Handle` frames with a log cell.

#### Summary of Handler Patterns

| Pattern | Core Mechanism | IR Nodes Used |
|---------|---------------|---------------|
| Retry | Recursive CPS wrapper | `Lam`, `App`, `If`, `Let` |
| Rollback | Undo actions in failure continuations | `Lam`, `App`, `If`, `Let` |
| Transaction | Nested `Handle` frames intercepting effects | `Handle`, `Lam`, `App`, `If`, `Let` |

All three patterns are built from the same six core CPS nodes. No special-purpose constructs.

## 6. Laziness and Evaluation Strategy in CPS

This section explores how call-by-name and call-by-need evaluation can be expressed in the
CPS IR without special constructs. The goal is to show that evaluation strategy is a calling
convention — thunks instead of values — and that the existing CPS nodes are sufficient.

This is a design exploration. The surface syntax, effect annotations, and type-system rules
for laziness are left to future specs. The IR treatment below is the target that those specs
must refine.

### 6.1 Thunks in CPS

A thunk is a zero-parameter CPS function that delays evaluation:

```text
Thunk<A> = Lam { params: [], cont_param: k, body: ... }
```

When applied with `App { func: thunk, args: [], cont: k }`, it evaluates its body and
invokes `k` with the result. No special IR node needed.

### 6.2 Call-by-Name

In call-by-name, arguments are wrapped in thunks and passed unevaluated. The function
forces each thunk at each use site.

**Surface sketch (for illustration only):**
```ash
fn lazy_if(cond: Thunk<<Bool>, then: Thunk<A>, else: Thunk<A>) -> A {
    if force(cond) then force(then) else force(else)
}
```

**CPS lowering:**

```text
lazy_if = Lam { params: [cond, then, else], cont_param: k, row: r,
    body: App { func: cond, args: [],           -- force cond
        cont: Lam { params: [cond_val], cont_param: k_cond,
            body: If { cond: cond_val,
                then_branch: App { func: then, args: [], cont: k, row: r },  -- force then
                else_branch: App { func: else, args: [], cont: k, row: r },  -- force else
                cont: k, row: r } } } }
```

**Lazy application:**
```ash
lazy_if(
    { expensive_predicate(x) },      -- thunk 1: not evaluated yet
    { compute_then(y) },             -- thunk 2: not evaluated yet
    { compute_else(z) }             -- thunk 3: not evaluated yet
)
```

Lowers to:
```text
App { func: lazy_if,
      args: [
          Lam { params: [], cont_param: k1,
                body: App { func: expensive_predicate, args: [x], cont: k1, row: r } },
          Lam { params: [], cont_param: k2,
                body: App { func: compute_then, args: [y], cont: k2, row: r } },
          Lam { params: [], cont_param: k3,
                body: App { func: compute_else, args: [z], cont: k3, row: r } }
      ],
      cont: k, row: r }
```

Key points:
- Arguments are `Lam { params: [] }` — thunks.
- The caller does not evaluate arguments before the call.
- The callee decides when to force each thunk by applying it with `App { func: thunk, args: [] }`.
- Effects inside the thunk fire at the force site, not at the call site.

### 6.3 Call-by-Need (Memoized)

In call-by-need, a thunk is forced once, then its result is cached. Subsequent forces return
the cached value without re-evaluation.

**CPS thunk with memo cell:**

```text
memo_thunk = CpsClosure {
    params: [],
    cont_param: k,
    env: { computed: Bool, value: Option<A>, body: Expr },
    body: If { cond: env.computed,
        then_branch: App { func: k, args: [env.value.unwrap()], cont: k, row: {} },
        else_branch: App { func: env.body, args: [],
            cont: Lam { params: [v], cont_param: k_body,
                body: Let { name: _, value: env.computed = true,
                    body: Let { name: _, value: env.value = Some(v),
                        body: App { func: k, args: [v], cont: k_body, row: {} } } } } } },
    row: r
}
```

Key points:
- The thunk is a `CpsClosure` with a mutable environment cell.
- On first force: evaluate `body`, store result, return it.
- On subsequent force: return cached result directly.
- The memo cell is part of the closure environment, not a separate IR construct.

**Lazy list (streams):**
```ash
data Stream<A> = Cons { head: Thunk<A>, tail: Thunk<Stream<A>> }
```

A stream is a pair of memoized thunks. The head is forced when accessed, the tail is forced
when the stream is advanced. Infinite streams are just recursive thunks.

### 6.4 Strictness and Mixed Strategies

A function can mix strict and lazy parameters:

```text
fn mixed(strict: Int, lazy: Thunk<String>) -> A
```

Lowers to:
```text
mixed = Lam { params: [strict, lazy], cont_param: k, row: r,
    body: ... -- strict is a value, lazy is a thunk }
```

The IR does not distinguish them at the parameter level. The distinction is in the calling
convention: the caller evaluates `strict` before the call, but passes `lazy` as a thunk.

### 6.5 Effect Row Implications (Design Space)

The effect row of a lazy function depends on which thunks are forced:

```text
fn f(x: Thunk<{cap db.read} String>) -> {cap db.read} Int
```

If `x` is forced inside `f`, the row includes `cap db.read`. If `x` is never forced, the row
does not. The type checker must approximate this (e.g., assume all thunks may be forced, or
track force sites).

For call-by-need, the memoization itself is a stateful operation. It could be tracked as an
effect:

```text
memo Thunk<A>  -- reads/writes a memo cell
```

Or it could be treated as a pure runtime primitive (invisible to the effect system).

### 6.6 Open Questions for Future Specs

1. **Surface syntax:** How are thunks created and forced? Explicit syntax (`Thunk { expr }`,
   `force x`) or implicit (`lazy expr`, `x`)?
2. **Type system:** Is `Thunk<A>` a distinct type constructor? Can it appear in effect rows?
3. **Effect tracking:** Are `force` and `memo` explicit effect items, or implicit runtime
   operations?
4. **Pattern matching:** Are patterns lazy or strict by default? Can patterns force thunks?
5. **Data constructors:** Can constructor fields be lazy? How is this annotated?
6. **Interaction with `do`:** Is `do` notation strict or lazy? Can lazy thunks appear in `do`
   blocks?
7. **Memoization scope:** Is memoization per-thunk (global) or per-evaluation-context (local)?

### 6.7 Summary

| Strategy | IR Representation | Force Mechanism | Memoization |
|----------|-------------------|-------------------|-------------|
| Call-by-value | Value passed directly | N/A | N/A |
| Call-by-name | `Lam { params: [] }` | `App { func: thunk, args: [] }` | None |
| Call-by-need | `CpsClosure` with memo cell | `App { func: thunk, args: [] }` | Env cell |

Laziness is a calling convention in the CPS IR. No special nodes needed. The design space for
surface syntax, type system integration, and effect tracking is left to future specs.

## 7. Handler Stack as CPS Continuation Chain

In CPS, handlers are not a separate stack data structure. They are continuations that wrap
the "next" continuation. A handler frame is a `Cont` value that intercepts matching `Raise` nodes.

```rust
pub struct HandlerDef {
    pub effect: EffectItem,
    -- handler body: takes effect args and a resume continuation, produces a final answer
    pub body: Box<Expr>,
}

-- HandlerFrame is a Cont that checks if the raised effect matches,
-- and if so, invokes the handler body with the resume continuation.
pub enum Value {
    -- ...
    HandlerFrame {
        effect: EffectItem,
        handler: HandlerDef,
        next: Box<Value>,  -- the next continuation in the chain
    },
}
```

**Handler dispatch in CPS:**

When a `Raise` node is evaluated:

1. Walk the continuation chain from the current continuation.
2. Find the first `HandlerFrame` whose `effect` matches the raised effect.
3. If found: invoke the handler body with the effect arguments and a resume continuation
   that reconstructs the rest of the chain.
4. If not found: the computation is stuck (unhandled effect).

This is equivalent to the operational semantics in SPEC-099b but expressed directly in the
CPS IR rather than as a separate runtime stack.

## 7. Migration and IR Evolution

### 7.1 Two IRs During Migration

During the migration period, the compiler maintains **two IRs**:

1. **Legacy IR**: The current AST with `Act`, `Proc`, `Workflow`, `Do`, and other
   migration-era artifacts. This is what the current parser produces.
2. **Target IR (CPS)**: The new CPS form with `Lam`, `App`, `Raise`, `Handle`, etc.

The lowering pipeline is:

```text
surface AST (legacy syntax)
    |
    v
Legacy IR (Act/Proc/Workflow/Do variants)
    |
    v
lower_to_cps.rs -- single pass lowering
    |
    v
Target IR (CPS: Lam/App/Raise/Handle/If/Let)
    |
    v
type checker, optimizer, code generator
```

All semantic analysis operates on the Target IR only. The Legacy IR is a transient
representation that exists only between parsing and the CPS lowering pass.

### 7.2 Migration Completion Flag

Migration is complete when:

- The parser produces Target IR directly (no Legacy IR variants);
- The `lower_to_cps` pass becomes an identity function on Target IR;
- No code path constructs `Act`, `Proc`, `Workflow`, or `Do` AST nodes;
- The Legacy IR variants are removed from the `Expr` enum.

At that point, the compiler has a single IR: the CPS form. The presence of a non-identity
lowering pass is a clear flag that migration is still in progress.

### 7.3 Legacy IR to CPS Lowering Rules

During migration, legacy AST variants are lowered to CPS:

```text
legacy Expr::Act { ... }      -> CPS Lam { cont_param: k, body: [lowered], row: Act_profile }
legacy Expr::Do { target, ... } -> CPS Lam { cont_param: k, body: [lowered], row: target_profile }
legacy Expr::Proc { ... }     -> CPS Lam { cont_param: k, body: [lowered], row: Proc_profile }
legacy Expr::Workflow { ... } -> CPS Lam { cont_param: k, body: [lowered], row: Workflow_profile }
legacy Type::Fn { ... }       -> CPS CpsFn { params, cont: Cont { arg: ret, ... }, ret: final_answer, row: {} }
legacy Type::Fun { ... }      -> CPS CpsFn { params, cont: Cont { arg: ret, ... }, ret: final_answer, row: effect }
```

### 7.4 Dual Representation (Temporary)

A conforming implementation may maintain both representations during migration:

```rust
pub enum Expr {
    -- CPS representation (target)
    Lam { ... },
    App { ... },
    Raise { ... },
    Handle { ... },
    If { ... },
    Let { ... },

    -- Legacy compatibility (to be removed after migration)
    Act { ... },
    Do { ... },
    Proc { ... },
    Workflow { ... },
}
```

The legacy variants are always lowered to CPS before semantic analysis. They are never
optimized, interpreted, or code-generated directly.

## 8. CPS Optimization Opportunities

The CPS form enables several standard optimizations:

1. **Contification**: Identify functions that are always called with a known continuation and
   inline the continuation into the function body.
2. **Administrative normal form (ANF)**: All intermediate values are named `Let` bindings,
   making dataflow analysis straightforward.
3. **Effect row propagation**: Effect rows flow through continuations, enabling precise
   effect-based dead code elimination and inlining decisions.
4. **Handler frame simplification**: Nested handlers for the same effect can be merged or
   reordered if their rows are compatible.
5. **Tail call optimization**: Every `App` to a continuation is a tail call by construction in CPS.

## 9. Open Decisions

1. Whether to use explicit labels or closures for continuations (labels enable better
   contification and compilation to machine code; closures are simpler for interpretation).
2. Whether the CPS IR is the canonical IR or an intermediate layer between a higher-level IR
   and a lower-level IR.
3. How to represent mutually recursive CPS functions.
4. Whether contract discharge status is stored in the IR or in a separate sidecar.
5. How row variables are represented in the CPS IR (names, indices, or de Bruijn indices).
6. Whether effect aliases are expanded during CPS lowering or preserved for diagnostics.
7. Whether to support direct-style fragments within CPS for performance-critical pure code.

## 10. See Also

- [SPEC-098a: Current IR](SPEC-098a-CURRENT-IR.md) — what the IR looks like today
- [SPEC-095b: Target Grammar](SPEC-095b-TARGET-GRAMMAR.md)
- [SPEC-096b: Target Effect System](SPEC-096b-TARGET-EFFECT-SYSTEM.md)
- [SPEC-097b: Target Type System](SPEC-097b-TARGET-TYPE-SYSTEM.md)
- [SPEC-099b: Target Operational Semantics](SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md)

## 10. Changelog

- 2026-06-18: Rewrote as CPS-based target IR. All computations are CPS functions with continuation parameters. Handlers are continuation chains, not separate stacks. Added CPS lowering examples, contification, ANF, and tail-call optimization notes.
