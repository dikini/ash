---
id: ref.language.functions.expressions
title: Function Bodies and Expressions
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
    - crates/ash-parser/src/parse_module.rs
    - crates/ash-parser/src/parse_expr.rs
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
    []
refresh_trigger:
  - SPEC-027 changes
  - SPEC-031 changes
  - function parser or typechecker changes
---
# Function Bodies and Expressions

## Summary

A function body is a block. It contains zero or more statements and an optional final expression. When a value is required, the final expression is the return value.

## Tail-expression return

Ash functions do not use `ret`.

```ash
pub fn add_one(n: Int) -> Int {
    n + 1
}
```

The expression `n + 1` is the returned value.

## Local `let` bindings

Use `let` to name intermediate values.

```ash
pub fn area(width: Int, height: Int) -> Int {
    let checked_width = width;
    let checked_height = height;
    checked_width * checked_height
}
```

`let` may use patterns when the expression being bound has a matching shape.

```ash
pub fn unwrap_ok(res: Result<Int, String>) -> Int {
    let Ok { value: n } = res;
    n
}
```

Pattern-binding behavior follows the same pattern rules described in the pattern page; use exhaustive `match` when a value can have multiple constructors.

## Blocks

A nested block can introduce local names and produce a value.

```ash
pub fn scaled(n: Int) -> Int {
    let factor = {
        let base = 2;
        base + 1
    };
    n * factor
}
```

## `if` expressions

Use `if ... then ... else ...` when both branches produce a value.

```ash
pub fn clamp_nonnegative(n: Int) -> Int {
    if n < 0 then { 0 } else { n }
}
```

A one-armed `if` has a `null` else path in the working spec. Prefer explicit `else` branches in reference examples unless the result type is intentionally `Null`.

## `match` expressions

A `match` expression selects a value from constructor or wildcard arms.

```ash
pub fn is_some<T>(opt: Option<T>) -> Bool {
    match opt {
        Some { value: _ } => true,
        None => false
    }
}
```

## `panic`

`panic` aborts pure computation. Use it only when the API intentionally treats a case as invalid.

```ash
pub fn unwrap<T, E>(res: Result<T, E>) -> T {
    match res {
        Ok { value: v } => v,
        Err { error: _ } => panic "called unwrap on Err"
    }
}
```

## Expressions not allowed in pure bodies

Pure functions must not perform runtime-managed effects. Do not use `act`, `observe`, `send`, `receive`, `spawn`, workflow obligations, or `invoke(...)` inside a pure function body.
