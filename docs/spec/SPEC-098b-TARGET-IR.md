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

## 6. Handler Stack as CPS Continuation Chain

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

## 7. Migration Compatibility

### 7.1 Legacy IR to CPS Lowering

During migration, legacy AST variants are lowered to CPS in a single pass:

```text
legacy Expr::Act { ... }      -> CPS Lam { cont_param: k, body: [lowered], row: Act_profile }
legacy Expr::Do { target, ... } -> CPS Lam { cont_param: k, body: [lowered], row: target_profile }
legacy Expr::Proc { ... }     -> CPS Lam { cont_param: k, body: [lowered], row: Proc_profile }
legacy Expr::Workflow { ... } -> CPS Lam { cont_param: k, body: [lowered], row: Workflow_profile }
legacy Type::Fn { ... }       -> CPS CpsFn { params, cont: Cont { arg: ret, ... }, ret: final_answer, row: {} }
legacy Type::Fun { ... }      -> CPS CpsFn { params, cont: Cont { arg: ret, ... }, ret: final_answer, row: effect }
```

### 7.2 Dual Representation

A conforming implementation may maintain both representations during migration:

```rust
pub enum Expr {
    -- CPS representation (new)
    Lam { ... },
    App { ... },
    Raise { ... },
    Handle { ... },
    If { ... },
    Let { ... },

    -- Legacy compatibility (deprecated, lowered to CPS before semantic analysis)
    Act { ... },
    Do { ... },
    Proc { ... },
    Workflow { ... },
}
```

The legacy variants are always lowered to CPS before type checking, optimization, or code
generation. No semantic analysis operates on the legacy forms directly.

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
