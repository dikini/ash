---
id: ref.language.functions.local
title: Local and Anonymous Functions
kind: reference
audience: [human, agent]
authority: canonical-adjacent
status: current
stability: alpha
owner: language
last_verified: 2026-05-26
verified_against:
  git_commit: 0874763
  specs:
    - docs/spec/SPEC-027-PURE-FUNCTIONS.md
    - docs/spec/SPEC-031-FIRST-CLASS-FUNCTIONS.md
    - docs/spec/SPEC-072-TOWER-CALLABLE-TYPE-AND-CLOSURE-SYNTAX.md
    - docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-954-functions-reference-chapter.md
    - docs/plan/tasks/TASK-961-callable-syntax-reference-docs.md
  code:
    - crates/ash-parser/src/parse_expr.rs
    - crates/ash-parser/src/lower.rs
    - crates/ash-core/src/ast.rs
  tests:
    []
  examples:
    []
related:
  depends_on:
    - ref.language.functions
  explains:
    []
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/spec/SPEC-031-FIRST-CLASS-FUNCTIONS.md
refresh_trigger:
  - SPEC-027 changes
  - SPEC-031 changes
  - function parser or typechecker changes
---
# Local and Anonymous Functions

## Summary

Ash supports local function values in expression contexts. They are useful for small transformations and higher-order helpers, but they are not the same thing as module-level function exports.

## Anonymous `fn` expressions

An anonymous function expression starts with `fn`, has a parameter list, may have a return type, and has a body block.

```ash
pub fn use_local(n: Int) -> Int {
    let double = fn(x: Int) -> Int { x * 2 };
    double(n)
}
```

The parser represents this as a function-definition expression (`FnDef`) rather than as a module item.

## Named local functions

Inside a function body or block, a named local function desugars to a `let` binding of an anonymous function.

```ash
pub fn use_named_local(n: Int) -> Int {
    fn double(x: Int) -> Int { x * 2 }
    double(n)
}
```

This is equivalent in shape to:

```ash
pub fn use_named_local(n: Int) -> Int {
    let double = fn(x: Int) -> Int { x * 2 };
    double(n)
}
```

## Closure shorthand

For a short expression body, use closure shorthand:

```ash
pub fn use_shorthand(n: Int) -> Int {
    let double = |x| -> x * 2;
    double(n)
}
```

The shorthand desugars immediately to an anonymous `fn` expression and remains pure even when written inside higher tower contexts. It does not carry a return-type annotation; use full `fn(...) -> ... { ... }` syntax when the type needs to be explicit. The higher-stratum closure arrows `|args| -*>`, `|args| =>`, and `|args| =*>` are reserved and rejected until those callable semantics exist.

## Capture

Local function values capture lexical variables from their surrounding scope.

```ash
pub fn add_n(n: Int, x: Int) -> Int {
    let add = fn(y: Int) -> Int { n + y };
    add(x)
}
```

## Current boundaries

- Module-level functions are collected as definitions and imported by module machinery; they are not reified as serializable closure values.
- Local closures are not safe to send across process boundaries.
- Partial application is not supported. If a function expects two arguments, call it with two arguments.
