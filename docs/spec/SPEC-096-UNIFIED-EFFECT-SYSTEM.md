---
id: spec.ash.unified-effect-system
title: Unified Effect System with Row Polymorphism and Contract Effects
kind: spec
audience: [human, agent]
authority: design
status: draft
stability: alpha
owner: language
last_verified: 2026-06-17
verified_against:
  specs:
    - docs/spec/SPEC-095-ASH-SURFACE-GRAMMAR.md
    - docs/spec/SPEC-088-CLOSURE-REFINEMENT-AND-EFFECT-SAFE-CAPTURE.md
    - docs/spec/SPEC-091-LET-DESTRUCTORS.md
    - docs/spec/SPEC-072-TOWER-CALLABLE-TYPE-AND-CLOSURE-SYNTAX.md
---

# SPEC-096: Unified Effect System with Row Polymorphism and Contract Effects

## 1. Summary

Replace the current four-stratum tower (Pure, Act, Proc, Workflow) with a unified effect system based on row polymorphism and algebraic effects. Act, Proc, and Workflow are the same monad; their distinction is tracked via effect rows, not separate types. Contracts (requires, ensures, laws) are effects that can be discharged statically or dynamically.

## 2. Motivation

The current system has four strata with separate syntax and types:

```text
Pure < Act < Proc < Workflow
```

This creates:
- **Syntactic duplication**: `do:Act {}`, `do:Proc {}`, `do:Workflow {}`
- **Semantic gaps**: pure code cannot express effects, workflow code cannot be reused in Act contexts
- **Runtime complexity**: three separate monad implementations for the same structure
- **Contract isolation**: `requires`/`ensures` only exist at the workflow level

The insight: Act, Proc, and Workflow are the *same* monad. Their differences are entirely in what effects are available. Row polymorphism expresses this naturally.

## 3. Core Design

### 3.1 Effect Rows

An effect row is a set of capabilities and contracts, written as `{fs, log, Contract}`.

```ebnf
effect_row = "{" [ effect_item { "," effect_item } [","] ] "}" ;

effect_item = capability_ref
            | contract_effect
            | row_variable
            ;

capability_ref = identifier [ "." identifier ] ;

contract_effect = "Contract" "{" predicate "}" ;

row_variable = identifier ;  (* e.g., r, s, t *)
```

### 3.2 Unified Function Types

```ebnf
function_type = [ effect_row ] "(" [ parameter_list ] ")" "->" type ;
```

Examples:

```ash
-- Pure function: empty effect row
fn add(a: Int, b: Int) -> {} Int { a + b }

-- Function with filesystem effect
fn readFile(path: String) -> {fs} String { ... }

-- Function with multiple effects
fn logAndRead(path: String) -> {fs, log} String { ... }

-- Polymorphic in remaining effects
fn processFile(path: String) -> {fs | r} String { ... }
-- Works in any context that provides fs, regardless of other effects

-- Function with contract effect
fn divide(a: Int, b: Int) -> {Contract {b != 0}} Int { a / b }
```

### 3.3 Row Inclusion (The Tower)

The current tower becomes row inclusion:

| Current | Effect Row | Description |
|---------|-----------|-------------|
| `Pure` | `{}` | No effects |
| `Act` | `{fs, log, ...}` | Capability effects |
| `Proc` | `{fs, log, spawn, send, ...}` | Act + process effects |
| `Workflow` | `{fs, log, spawn, Contract, Policy, ...}` | Proc + contract effects |

Row inclusion: `{fs} ⊂ {fs, log} ⊂ {fs, log, spawn} ⊂ {fs, log, spawn, Contract}`

A function with row `{fs | r}` can be called in any context whose row includes `{fs}`.

### 3.4 Unified `do` Notation

Replace `do:Act {}`, `do:Proc {}`, `do:Workflow {}` with a single `do {}`.

```ash
-- Current
pub fn readConfig(path: String) -> Act<String> {
    do:Act { return fs.read(path) }
}

-- Proposed
pub fn readConfig(path: String) -> {fs} String {
    do { x <- fs.read(path); return x }
}
```

The effect row on the function type determines what effects are available inside `do`.

### 3.5 Contracts as Effects

Contracts are first-class effects that can be discharged statically or dynamically.

```ebnf
contract_effect = "requires" predicate
                | "ensures" predicate
                | "invariant" predicate
                | "law" identifier predicate
                ;

predicate = expr ;
```

Examples:

```ash
-- Contract as part of effect row
fn divide(a: Int, b: Int) -> {requires {b != 0}} Int {
    a / b
}

-- Multiple contracts
fn binarySearch(arr: List<T>, target: T) -> {requires {sorted(arr)}, ensures {result >= -1}} Int {
    ...
}

-- Law as contract effect
fn append(a: List<T>, b: List<T>) -> {law associative {append(append(a, b), c) == append(a, append(b, c))}} List<T> {
    ...
}
```

### 3.6 Contract Discharge

Contracts can be discharged in three ways:

| Method | Mechanism | When |
|--------|-----------|------|
| **Static** | Type system proves predicate always holds | Compile time |
| **Dynamic** | Runtime effect handler checks predicate | Runtime |
| **Proof** | Formal proof or test evidence | Link time |

```ash
-- Static discharge: type system knows x != 0
let x = 5;
divide(x, 2)  -- Contract effect discharged statically

-- Dynamic discharge: runtime check
let y = readInt();
divide(y, 2)  -- Contract effect requires runtime handler

-- Proof discharge: evidence provided
proof divide by test quickcheck {
    -- generates random inputs, checks b != 0
}
```

### 3.7 Contract Effect Handlers

Contract effects are handled by effect handlers, enabling recovery and logging:

```ash
-- Default handler: fail on violation
handle Contract with {
    requires(pred) -> if pred() then () else raise ContractViolation
    ensures(pred) -> if pred() then () else raise ContractViolation
}

-- Logging handler: log and continue
handle Contract with {
    requires(pred) -> 
        if pred() then () else log("requires violated")
    ensures(pred) -> 
        if pred() then () else log("ensures violated")
}

-- Retry handler: adjust and retry
handle Contract with {
    requires(pred) -> 
        if pred() then () else retry_with_adjusted_input()
}
```

## 4. Syntax Changes

### 4.1 What to Remove

| Current | Reason |
|---------|--------|
| `do:Act {}`, `do:Proc {}`, `do:Workflow {}` | Unified `do {}` with effect row |
| `Act<T>`, `Proc<T>`, `Workflow<T>` | Effect rows replace separate types |
| `ret` keyword | Use `return` everywhere |
| `fail` expression | Use `raise` effect or `do` |
| `with_error` expression | Error handling becomes an effect |
| `check` expression | Use `do { check(o) }` |
| `yield` statement | Use `do { yield(x) }` |
| `propose` statement | Use `do { propose(x, r) }` |
| `oblige` statement | Use `do { oblige(o) }` |
| `decide` statement | Use `do { decide(p) }` |
| `maybe` statement | Use `do { catch { ... } }` |
| `must` statement | Use `assert` effect |
| `send`/`receive` statements | Use `Send`/`Receive` effects in `do` |
| `set` statement | Use `State` effect |
| `for` in workflows | Use `traverse` or list comprehensions |
| `observe` statement | Use `Observe` effect |
| `orient` statement | Use `coerce` function |
| `with` statement | Use `do` with capability in scope |
| `workflow` keyword | Just `fn` with contract annotations |
| `capabilities`/`observes`/`receives`/`obligations`/`owns`/`uses` clauses | Part of effect row |
| `plays role` | Part of type signature |

### 4.2 What to Keep

| Syntax | Role |
|--------|------|
| `fn` | Function definition |
| `let` | Binding |
| `if`/`then`/`else` | Conditionals |
| `match` | Pattern matching |
| `do` | Effectful computation |
| `return` | Return from `do` |
| `<-` | Bind in `do` |
| `->` | Function arrow |
| `=>` | Reserved for future use |
| `type` | Type definition |
| `use` | Import |
| `mod` | Module |
| `role` | Role declaration |
| `capability` | Capability interface |
| `interface` | Interface |
| `impl` | Implementation |
| `law` | Law declaration |
| `proof` | Proof evidence |
| `pub` | Visibility |
| `pub(crate)` | Restricted visibility |
| `pub(super)` | Restricted visibility |
| `pub(in path)` | Restricted visibility |
| `struct` | Record type |
| `enum` | Sum type |
| `alias` | Type alias |
| `forall` | Universal quantification |
| `exists` | Existential quantification |
| `panic` | Unrecoverable error |
| `true`/`false` | Boolean literals |
| `null` | Null literal |

### 4.3 What to Add

| Syntax | Role |
|--------|------|
| `{}` | Effect row (empty = pure) |
| `{fs}` | Effect row with capability |
| `{fs \| r}` | Effect row with row variable |
| `{requires {p}}` | Contract effect |
| `{ensures {p}}` | Contract effect |
| `{invariant {p}}` | Contract effect |
| `{law name {p}}` | Law contract effect |
| `handle E with { ... }` | Effect handler |
| `raise E` | Raise effect |
| `retry` | Retry after contract failure |
| `coerce` | Type coercion |
| `assert` | Assertion effect |
| `assume` | Assumption (static only) |

## 5. Type System Integration

### 5.1 Effect Row Subtyping

```
{fs} <: {fs, log}        -- row extension
{fs | r} <: {fs, log}    -- row variable instantiation
{} <: {fs}               -- empty row is subtype of any row
```

### 5.2 Function Subtyping

```
({fs} A -> B) <: ({fs, log} A -> B)  -- contravariant in effect row
```

A function requiring fewer effects can be used where more effects are available.

### 5.3 Contract Subtyping

```
{requires {p}} <: {}  -- only if p is statically provable
{ensures {p}} <: {}  -- only if p is statically provable
```

Contracts can be "subsumed" if the type system can prove them.

### 5.4 Row Polymorphism

```ash
-- map works for any effect row
pub fn map<A, B>(xs: List<A>, f: A -> {r} B) -> {r} List<B> {
    do { ... }
}

-- The caller's effect row is preserved
map([1, 2, 3], fn(x) -> {fs} x + 1)  -- result has type {fs} List<Int>
map([1, 2, 3], fn(x) -> {} x + 1)    -- result has type {} List<Int>
```

## 6. Runtime Semantics

### 6.1 Single Monad Implementation

Act, Proc, and Workflow are the same monad at runtime. The effect row is a type-level construct only.

```rust
// Runtime representation: a single Eff monad
enum Eff<A> {
    Pure(A),
    Effect(Box<dyn Effect<A>>),
}

// Effect dispatch is dynamic, based on the capability
trait Effect<A> {
    fn run(&self, env: &Env) -> Eff<A>;
}
```

### 6.2 Effect Handlers

Effects are handled by a stack of handlers, searched dynamically:

```rust
struct HandlerStack {
    handlers: Vec<Box<dyn Handler>>,
}

impl HandlerStack {
    fn handle<E, A>(&self, effect: E) -> Result<A, UnhandledEffect>
    where E: Effect<A> {
        for handler in self.handlers.iter().rev() {
            if handler.can_handle(&effect) {
                return handler.run(effect);
            }
        }
        Err(UnhandledEffect)
    }
}
```

### 6.3 Contract Handlers

Contract effects are just effects with special handlers:

```rust
struct ContractHandler {
    mode: ContractMode,  // Static, Dynamic, Proof
}

enum ContractMode {
    Static,   -- Type system proves it
    Dynamic,  -- Runtime check
    Proof,    -- Evidence provided
}
```

## 7. Interaction with Existing Features

### 7.1 Capabilities

Capabilities are effects. The capability system becomes the effect system.

```ash
-- Current
capability fs {
    read(path: String) -> String;
    write(path: String, content: String) -> ();
}

-- Proposed: capabilities are effect interfaces
effect fs {
    read: String -> {fs} String;
    write: (String, String) -> {fs} ();
}
```

### 7.2 Closures

Closures capture the effect row of their creation context:

```ash
-- Pure closure: empty effect row
fn makeAdder(n: Int) -> {} (Int -> {} Int) {
    fn(x) -> {} { n + x }
}

-- Effectful closure: captures fs
fn makeReader() -> {fs} (String -> {fs} String) {
    fn(path) -> {fs} { fs.read(path) }
}
```

### 7.3 Pattern Matching

Pattern matching is pure, but can be used in effectful contexts:

```ash
fn process(x: Option<Int>) -> {log} Int {
    do {
        match x {
            Some(n) -> { log("got " + n); return n }
            None -> { log("empty"); return 0 }
        }
    }
}
```

### 7.4 Type Definitions

Type definitions are pure, but can reference effect rows:

```ash
type Handler<T> = {
    onRequest: Request -> {fs} T,
    onError: Error -> {log} T,
};
```

## 8. Migration Path

### 8.1 Phase 1: Syntax (Parser Only)

- Add effect row syntax: `{}`, `{fs}`, `{fs | r}`
- Add `do {}` as unified syntax
- Keep `do:Act {}`, `do:Proc {}`, `do:Workflow {}` as deprecated aliases
- Add `handle` and `raise` keywords

### 8.2 Phase 2: Type System

- Add effect rows to type checker
- Implement row subtyping
- Implement row polymorphism
- Add contract effects to type system

### 8.3 Phase 3: Runtime

- Implement single `Eff` monad
- Implement effect handler stack
- Implement contract handlers
- Keep old runtime as compatibility layer

### 8.4 Phase 4: Standard Library

- Migrate `Act<T>` to `{...} T`
- Migrate `Proc<T>` to `{...} T`
- Migrate `Workflow<T>` to `{...} T`
- Add effect interfaces for capabilities

### 8.5 Phase 5: Removal

- Remove `Act<T>`, `Proc<T>`, `Workflow<T>` types
- Remove `do:Act {}`, `do:Proc {}`, `do:Workflow {}` syntax
- Remove `ret`, `fail`, `with_error`, `check`, `yield`, `propose`, `oblige`, `decide`, `maybe`, `must`, `send`, `receive`, `set`, `observe`, `orient`, `with` keywords
- Remove `workflow` keyword
- Remove `capabilities`, `observes`, `receives`, `obligations`, `owns`, `uses` clauses
- Remove `plays role` syntax

## 9. Known Limitations and Open Questions

### 9.1 Effect Row Inference

How much of the effect row can be inferred? Ideally, the programmer writes:

```ash
fn foo(x: Int) -> Int { x + 1 }
```

And the compiler infers `{}` (pure). But for complex functions, explicit annotation may be needed.

### 9.2 Effect Row Aliases

Should effect rows be aliasable?

```ash
type IO = {fs, log, net};
fn foo() -> IO Int { ... }
```

This improves readability but complicates the type system.

### 9.3 Higher-Order Effects

Can effects be parameterized by types?

```ash
effect State<T> {
    get: {} T;
    put: T -> {} ();
}
```

This is useful but adds complexity.

### 9.4 Effect Interaction

How do effects interact? For example, `State` and `Exception`:

```ash
handle State with {
    get() -> (current_state, current_state)
    put(s) -> ((), s)
}

handle Exception with {
    raise(e) -> (error_value, current_state)
}
```

The order of handlers matters.

### 9.5 Performance

Dynamic effect dispatch has overhead. Can we optimize?

- Static dispatch for known effect rows
- Inline handlers for simple cases
- Compile-time effect row specialization

## 10. Examples

### 10.1 Simple Pure Function

```ash
fn add(a: Int, b: Int) -> {} Int {
    a + b
}
```

### 10.2 Effectful Function

```ash
fn readConfig(path: String) -> {fs} String {
    do { x <- fs.read(path); return x }
}
```

### 10.3 Polymorphic Function

```ash
fn map<A, B>(xs: List<A>, f: A -> {r} B) -> {r} List<B> {
    do {
        match xs {
            [] -> return []
            [h, ..t] -> {
                let h2 = f(h);
                let t2 = map(t, f);
                return [h2, ..t2]
            }
        }
    }
}
```

### 10.4 Function with Contracts

```ash
fn divide(a: Int, b: Int) -> {requires {b != 0}} Int {
    a / b
}
```

### 10.5 Function with Handler

```ash
fn safeDivide(a: Int, b: Int) -> {} Int {
    handle Contract with {
        requires(pred) -> if pred() then () else return 0
    };
    divide(a, b)
}
```

### 10.6 Law as Contract

```ash
law associative<A>: (A, A, A) -> {law associative { ... }} Bool {
    requires {true}
    ensures {result == (a * b) * c == a * (b * c)}
}
```

## 11. See Also

- [SPEC-095: Ash Surface Syntax Grammar](SPEC-095-ASH-SURFACE-GRAMMAR.md)
- [SPEC-088: Closure Refinement and Effect-Safe Capture](SPEC-088-CLOSURE-REFINEMENT-AND-EFFECT-SAFE-CAPTURE.md)
- [SPEC-091: Let Destructors](SPEC-091-LET-DESTRUCTORS.md)
- [SPEC-072: Tower Callable Type and Closure Syntax](SPEC-072-TOWER-CALLABLE-TYPE-AND-CLOSURE-SYNTAX.md)
- [SPEC-027: Pure Functions](SPEC-027-PURE-FUNCTIONS.md)
- [SPEC-031: First-Class Functions](SPEC-031-FIRST-CLASS-FUNCTIONS.md)

## 12. Changelog

- 2026-06-17: Initial draft
