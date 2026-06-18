---
id: spec.ash.type-system.current
title: Ash Type System — Current State
description: Current type system without effect rows, as of main HEAD
code_commit: e61f2792
kind: spec
audience: [human, agent]
authority: derived-from-code
status: active
stability: beta
owner: language
last_verified: 2026-06-18
verified_against:
  git_commit: e61f2792
  code:
    - crates/ash-core/src/ast.rs
    - crates/ash-core/src/effect.rs
    - crates/ash-typeck/src/
---

# SPEC-097a: Ash Type System — Current State

**Status:** Active — records the live type system as of main HEAD
**Scope:** This document is the authority for what the type checker does today.
It does not propose changes.
**Frozen against:** `e61f2792`

## 1. Summary

The current Ash type system supports:

- named types, type constructors, tuples, records, and associated types;
- function types with return type annotations but no effect rows;
- generic type parameters with kind annotations;
- interface definitions with method signatures;
- implementation definitions with method bodies;
- type aliases;
- sealed domains and type functions (recently implemented);
- pattern typing and exhaustiveness checking;
- constraint and proposition layers (recently implemented).

It does **not** support:

- effect rows on function types;
- row polymorphism;
- effect item identity and namespaces;
- row subtyping or discharge;
- capability/role/policy/contract/channel/process/evidence effect typing;
- transparent effect aliases or diagnostic groups.

## 2. Current Type Expressions

### 2.1 AST Representation

From `crates/ash-core/src/ast.rs`:

```rust
pub enum TypeExpr {
    Named(Name),                    -- Type name: Int, String, etc.
    Constructor { name, args },     -- Type constructor: List<T>, Option<T>
    Tuple(Vec<TypeExpr>),           -- Tuple type: (A, B)
    Record(Vec<(Name, TypeExpr)>),  -- Record type: {x: Int, y: Int}
    Associated { base, name },      -- Associated type: T.Item
}
```

### 2.2 Function Types

Functions are typed by their return type annotation:

```ash
fn foo(x: Int) -> Int { ... }   -- return type is Int
```

There is no effect tracking in the type itself. The `Type::Fun(args, ret, effect)` variant
exists but the `effect` field uses the 4-point `Effect` lattice, not a row system.

### 2.3 Type Holes

Type holes are written as `_` and are inferred by the type checker:

```ash
fn foo(x: _) -> _ { x }
```

## 3. Current Effect Integration

### 3.1 Effect Lattice in Types

The 4-point `Effect` lattice from `crates/ash-core/src/effect.rs` is used in:

- `Type::Fun(args, ret, effect)` for effectful/capability-linked callables;
- workflow node classification;
- audit trail effect recording.

It is not used for:

- ordinary `fn` definitions (pure functions use `Type::Fn`);
- generic effect polymorphism;
- row-based discharge checking.

### 3.2 Workflow Typing

Workflow typing is handled separately from ordinary expression typing. The type checker has
a dedicated workflow-checking module that validates:

- capability references in workflow headers;
- role inclusion clauses;
- policy references in `decide` statements;
- obligation references in `check` and `oblige` statements;
- contract clauses (`requires`, `ensures`) at workflow boundaries.

These checks are not expressed as effect rows. They are ad-hoc checks against the workflow
header and runtime context.

## 4. Current Generics and Interfaces

### 4.1 Generic Type Parameters

```ash
fn map<A, B>(xs: List<A>, f: A -> B) -> List<B> { ... }
```

Generic parameters are inferred or explicitly annotated. There are no row variables or
effect-row polymorphism.

### 4.2 Interface Methods

```ash
interface Monad<M> {
    pure: A -> M<A>;
    bind: (M<A>, A -> M<B>) -> M<B>;
}
```

Interface methods use positional parameter types. Named parameters in interface methods are
not currently accepted by the parser.

### 4.3 Associated Types

```ash
interface Handler<T> {
    type Response = T;
    onRequest: Request -> Response;
}
```

Associated types are supported. They do not currently reference effect rows.

## 5. Current Pattern Typing

### 5.1 Pattern Types

Patterns are typed against scrutinee types. The type checker supports:

- wildcard, variable, literal, record, tuple, list, and variant patterns;
- irrefutability checking for `let` binders;
- exhaustiveness checking for `match` and total handler forms;
- refutable `if let ... else` as a total two-branch eliminator.

### 5.2 Pattern Exhaustiveness

Exhaustiveness is checked using canonical type information. The checker does not use effect
rows or effect-classification for pattern coverage.

## 6. Known Limitations

1. No effect rows on function types.
2. No row polymorphism in generics.
3. No effect item identity or namespace system.
4. No row subtyping or discharge rules.
5. No capability/role/policy/contract/channel/process/evidence effect typing.
6. No transparent effect aliases or diagnostic groups.
7. No user-defined algebraic effects.
8. No resumable continuations.
9. Workflow typing is separate from expression typing, not unified through rows.

## 7. See Also

- [SPEC-097b: Target Type System](SPEC-097b-TARGET-TYPE-SYSTEM.md) — unified effect rows
- [SPEC-003: Type System](SPEC-003-TYPE-SYSTEM.md) — older type system spec
- [SPEC-022: Workflow Typing with Constraints](SPEC-022-WORKFLOW-TYPING.md)
- [SPEC-096a: Current Effect System](SPEC-096a-CURRENT-EFFECT-SYSTEM.md)
- [SPEC-096b: Target Effect System](SPEC-096b-TARGET-EFFECT-SYSTEM.md)

## 8. Changelog

- 2026-06-18: Split from combined SPEC-097 into current-state document. Frozen against `e61f2792`. Added explicit description of current type expressions, effect integration, generics, interfaces, pattern typing, and limitations.
- 2026-06-17: Initial draft.
