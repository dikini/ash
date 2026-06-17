---
id: spec.ash.type-system-changes
title: Type System Changes for Unified Effect System
kind: spec
audience: [human, agent]
authority: design
status: draft
stability: alpha
owner: language
last_verified: 2026-06-17
verified_against:
  specs:
    - docs/spec/SPEC-096-UNIFIED-EFFECT-SYSTEM.md
  code:
    - crates/ash-core/src/ast.rs
    - crates/ash-core/src/effect.rs
    - crates/ash-typeck/src/
---

# SPEC-097: Type System Changes for Unified Effect System

## 1. Summary

This spec documents the changes to the Ash type system required to support the unified effect system (SPEC-096). It covers: (1) new type constructors for effect rows, (2) changes to function types, (3) row subtyping and polymorphism, (4) contract effects as types, and (5) integration with existing type system features.

## 2. Current Type System Baseline

### 2.1 Existing Types (from `crates/ash-core/src/ast.rs`)

```rust
pub enum TypeExpr {
    Named(Name),                    -- Type name: Int, String, etc.
    Constructor { name, args },     -- Type constructor: List<T>, Option<T>
    Tuple(Vec<TypeExpr>),           -- Tuple type: (A, B)
    Record(Vec<(Name, TypeExpr)>),  -- Record type: {x: Int, y: Int}
    Associated { base, name },      -- Associated type: T.Item
}
```

### 2.2 Existing Function Types

```rust
-- Current: function type is just the return type annotation
fn foo(x: Int) -> Int { ... }   -- return type is Int

-- No effect tracking in the type itself
```

### 2.3 Existing Effect System (from `crates/ash-core/src/effect.rs`)

```rust
pub enum Effect {
    Epistemic = 0,      -- Read-only
    Deliberative = 1,   -- Analysis/planning
    Evaluative = 2,     -- Policy evaluation
    Operational = 3,    -- Side effects
}
```

This is a **4-point lattice**, not a row. Effects are tracked on workflow nodes, not in types.

## 3. New Type Constructors

### 3.1 Effect Row Type

```rust
pub enum TypeExpr {
    -- ... existing variants ...
    
    -- NEW: Effect row type
    EffectRow {
        effects: Vec<EffectItem>,
        row_var: Option<Name>,  -- None for closed rows, Some(r) for open rows
    },
}

pub enum EffectItem {
    Capability(Name),           -- fs, log, net
    Contract(ContractEffect),   -- requires {p}, ensures {p}
}

pub enum ContractEffect {
    Requires(Predicate),
    Ensures(Predicate),
    Invariant(Predicate),
    Law(Name, Predicate),
}
```

### 3.2 Function Type Extension

Functions now carry an effect row:

```rust
pub struct FnType {
    params: Vec<TypeExpr>,
    return_type: TypeExpr,
    effect_row: TypeExpr,  -- NEW: EffectRow or TypeExpr::Named("Pure")
}
```

Surface syntax:
```ash
fn foo(x: Int) -> {fs} Int { ... }        -- function with fs effect
fn bar(x: Int) -> {fs | r} Int { ... }    -- polymorphic in remaining effects
fn baz(x: Int) -> {} Int { ... }          -- pure function (empty row)
```

### 3.3 Type Hole Extension

Type holes can appear in effect rows:

```ash
fn foo(x: Int) -> {_} Int { ... }   -- infer the effect row
```

## 4. Row Subtyping

### 4.1 Row Extension (Closed Rows)

```
{fs} <: {fs, log}          -- adding effects is contravariant
{fs, log} <: {fs, log, net} -- row extension preserves subtyping
```

### 4.2 Row Variable (Open Rows)

```
{fs | r} <: {fs, log}      -- instantiate r with {log}
{fs | r} <: {fs | s}       -- row variables are equivalent
```

### 4.3 Empty Row

```
{} <: {fs}                  -- pure functions can be used where effects are available
{} <: {fs | r}              -- pure functions can be used anywhere
```

### 4.4 Function Subtyping

Functions are contravariant in their effect row:

```
({fs} A -> B) <: ({fs, log} A -> B)
```

A function requiring fewer effects can be used where more effects are available.

### 4.5 Contract Subtyping

Contracts can be subsumed if statically provable:

```
{requires {p}} <: {}       -- only if p is statically true
{ensures {p}} <: {}        -- only if p is statically true
```

## 5. Row Polymorphism

### 5.1 Row Variables in Function Types

```ash
-- map is polymorphic in the effect row
pub fn map<A, B>(xs: List<A>, f: A -> {r} B) -> {r} List<B> {
    do { ... }
}

-- The caller's effect row is preserved
map([1, 2, 3], fn(x) -> {fs} x + 1)   -- result: {fs} List<Int>
map([1, 2, 3], fn(x) -> {} x + 1)     -- result: {} List<Int>
```

### 5.2 Row Variable Inference

The type system infers row variables when not explicitly annotated:

```ash
fn foo(x: Int) -> Int { x + 1 }       -- inferred: {} Int (pure)
fn bar(x: Int) -> Int { fs.read("x") } -- inferred: {fs} Int
```

### 5.3 Row Variable Constraints

Row variables can have constraints:

```ash
fn foo<A>(x: A) -> {r where r <: {fs}} A { ... }
-- r must be a subtype of {fs} (i.e., r can only add effects, not remove fs)
```

## 6. Contract Effects as Types

### 6.1 Contract Effect Types

Contracts are first-class types in the effect row:

```ash
fn divide(a: Int, b: Int) -> {requires {b != 0}} Int {
    a / b
}
```

Type representation:
```rust
EffectRow {
    effects: vec![Contract(Requires(Predicate { b != 0 }))],
    row_var: None,
}
```

### 6.2 Contract Discharge in Types

Contracts are discharged by subtyping:

```ash
-- Static discharge: type system knows x != 0
let x = 5;
divide(x, 2)  -- {requires {5 != 0}} <: {} (5 != 0 is true)

-- Dynamic discharge: runtime check needed
let y = readInt();
divide(y, 2)  -- {requires {y != 0}} not <: {} (y is unknown)
              -- requires dynamic handler
```

### 6.3 Contract Composition

Multiple contracts compose in the row:

```ash
fn binarySearch(arr: List<T>, target: T) -> 
    {requires {sorted(arr)}, ensures {result >= -1}} Int {
    ...
}
```

Type representation:
```rust
EffectRow {
    effects: vec![
        Contract(Requires(Predicate { sorted(arr) })),
        Contract(Ensures(Predicate { result >= -1 })),
    ],
    row_var: None,
}
```

## 7. Integration with Existing Features

### 7.1 Generics

Effect rows are part of generic type parameters:

```ash
fn map<A, B, r>(xs: List<A>, f: A -> {r} B) -> {r} List<B> { ... }
```

### 7.2 Interfaces

Interface methods can have effect rows:

```ash
interface Monad<M> {
    pure: A -> {} M<A>;
    bind: (M<A>, A -> {r} M<B>) -> {r} M<B>;
}
```

### 7.3 Associated Types

Associated types can reference effect rows:

```ash
interface Handler<T> {
    type Response = {log} T;
    onRequest: Request -> Response;
}
```

### 7.4 Type Aliases

Effect rows can be aliased:

```ash
type IO = {fs, log, net};
fn foo() -> IO Int { ... }
```

## 8. Type Checking Changes

### 8.1 New Type Checking Rules

| Rule | Description |
|------|-------------|
| T-EffRow-Intro | `{}` is a valid effect row |
| T-EffRow-Extend | If `R` is a row, `R, e` is a row |
| T-EffRow-Var | `r` is a valid row variable |
| T-Fun-Eff | Function type includes effect row |
| T-Sub-Row | Row subtyping rules |
| T-Sub-Contract | Contract subtyping (static discharge) |
| T-App-Eff | Application preserves effect row |
| T-Do-Eff | `do` block has effect row of its body |
| T-Handle-Eff | `handle` can discharge effects |

### 8.2 Type Environment Extension

The type environment tracks available effects:

```rust
pub struct TypeEnv {
    -- ... existing fields ...
    available_effects: EffectRow,  -- NEW: effects available in current scope
    handler_stack: Vec<Handler>,   -- NEW: active effect handlers
}
```

### 8.3 Error Messages

New error types:

```rust
pub enum TypeError {
    -- ... existing errors ...
    MissingEffect { required: EffectItem, available: EffectRow },
    ContractViolation { contract: ContractEffect, reason: String },
    RowMismatch { expected: EffectRow, found: EffectRow },
    UnhandledEffect { effect: EffectItem },
}
```

## 9. Examples

### 9.1 Simple Pure Function

```ash
fn add(a: Int, b: Int) -> {} Int {
    a + b
}
```

Type: `({} Int, {} Int) -> {} Int`

### 9.2 Effectful Function

```ash
fn readFile(path: String) -> {fs} String {
    do { x <- fs.read(path); return x }
}
```

Type: `({} String) -> {fs} String`

### 9.3 Polymorphic Function

```ash
fn map<A, B>(xs: List<A>, f: A -> {r} B) -> {r} List<B> {
    do { ... }
}
```

Type: `forall A, B, r. ({} List<A>, ({} A -> {r} B)) -> {r} List<B>`

### 9.4 Function with Contracts

```ash
fn divide(a: Int, b: Int) -> {requires {b != 0}} Int {
    a / b
}
```

Type: `({} Int, {} Int) -> {requires {b != 0}} Int`

### 9.5 Function with Handler

```ash
fn safeDivide(a: Int, b: Int) -> {} Int {
    handle Contract with {
        requires(pred) -> if pred() then () else return 0
    };
    divide(a, b)
}
```

Type: `({} Int, {} Int) -> {} Int`

## 10. Migration Path

### 10.1 Phase 1: Add EffectRow type constructor

- Add `TypeExpr::EffectRow` variant
- Add `EffectItem` and `ContractEffect` types
- Update parser to parse effect row syntax

### 10.2 Phase 2: Add row subtyping

- Implement row subtyping rules
- Add row variable inference
- Update type checker to track effect rows

### 10.3 Phase 3: Add contract effects

- Add contract effects to type system
- Implement static discharge
- Add contract error types

### 10.4 Phase 4: Update function types

- Add effect row to function types
- Update function subtyping
- Update application rules

### 10.5 Phase 5: Integration

- Update generics to handle effect rows
- Update interfaces to handle effect rows
- Update type aliases to handle effect rows

## 11. See Also

- [SPEC-096: Unified Effect System](SPEC-096-UNIFIED-EFFECT-SYSTEM.md)
- [SPEC-098: IR Changes for Unified Effect System](SPEC-098-IR-CHANGES.md)
- [SPEC-099: Operational Semantics for Unified Effect System](SPEC-099-OPERATIONAL-SEMANTICS.md)

## 12. Changelog

- 2026-06-17: Initial draft
