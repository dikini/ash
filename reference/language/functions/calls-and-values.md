---
id: ref.language.functions.calls
title: Calling Functions and Using Function Values
kind: reference
audience: [human, agent]
authority: canonical-adjacent
status: current
stability: alpha
owner: language
last_verified: 2026-05-23
verified_against:
  git_commit: 414549f
  specs:
    - docs/spec/SPEC-027-PURE-FUNCTIONS.md
    - docs/spec/SPEC-031-FIRST-CLASS-FUNCTIONS.md
    - docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-954-functions-reference-chapter.md
  code:
    - crates/ash-parser/src/parse_expr.rs
    - crates/ash-typeck/src/types.rs
  tests:
    []
  examples:
    []
related:
  depends_on:
    - ref.language.functions
    - ref.language.functions.local
  explains:
    []
  supersedes: []
  superseded_by: null
  historical_rationale:
    []
refresh_trigger:
  - SPEC-027 changes
  - SPEC-031 changes
  - function parser or typechecker changes
---
# Calling Functions and Using Function Values

## Summary

A function call evaluates arguments and applies a named function or function value. Direct calls, module-qualified calls, and function-value application all use ordinary expression syntax.

## Direct calls

Call a visible function by name.

```ash
pub fn double(n: Int) -> Int { n * 2 }

pub fn answer() -> Int {
    double(21)
}
```

## Module-qualified calls

Use `::` for module-qualified function names.

```ash
pub fn use_math(n: Int) -> Int {
    math::double(n)
}
```

This is distinct from capability/provider dispatch, which uses single-colon provider/action naming in effectful contexts.

## Function types

Pure function values use `Fn(<params>) -> <return>`.

```ash
pub fn apply(value: Int, f: Fn(Int) -> Int) -> Int {
    f(value)
}
```

Multiple parameters are comma-separated:

```ash
pub fn combine(a: Int, b: Int, f: Fn(Int, Int) -> Int) -> Int {
    f(a, b)
}
```

## Higher-order examples

Pass an anonymous function:

```ash
pub fn demo(n: Int) -> Int {
    apply(n, fn(x: Int) -> Int { x + 1 })
}
```

Pass closure shorthand:

```ash
pub fn demo(n: Int) -> Int {
    apply(n, |x| => x + 1)
}
```

Return a local function value only within a pure/local scope where the caller can use it immediately. Do not treat returned closures as workflow/process payloads unless a later reference page documents that boundary explicitly.

## Arity

Function application is not partial application. A call must provide the number of arguments required by the function value or definition.
