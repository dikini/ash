---
id: ref.language.functions.syntax
title: Function Declaration Syntax
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
    - crates/ash-parser/src/parse_module.rs
    - crates/ash-parser/src/surface.rs
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
# Function Declaration Syntax

## Summary

A module-level pure function is declared with `fn`. Add `pub` or a restricted visibility modifier when code outside the defining module should call it.

## Basic declarations

Private module function:

```ash
fn double(n: Int) -> Int {
    n * 2
}
```

Public module function:

```ash
pub fn double(n: Int) -> Int {
    n * 2
}
```

Restricted visibility follows the same shape used by other visible module items:

```ash
pub(crate) fn crate_visible(n: Int) -> Int {
    n
}

pub(super) fn parent_visible(n: Int) -> Int {
    n
}
```

## Parameters and return types

Parameters use `name: Type`. The return type follows `->` and is recommended for reference-facing code even when inference can recover it.

```ash
pub fn between(n: Int, low: Int, high: Int) -> Bool {
    low <= n && n <= high
}
```

Zero-parameter functions use empty parentheses:

```ash
pub fn default_count() -> Int {
    0
}
```

## Generic functions

Type parameters appear after the function name.

```ash
pub fn identity<T>(value: T) -> T {
    value
}
```

Function-typed parameters use the preferred callable arrow form `(...) -> ...`:

```ash
pub fn apply<T, U>(value: T, f: (T) -> U) -> U {
    f(value)
}
```

## Proposition tails and contracts

Current parser evidence carries an optional `where` proposition tail and optional `requires:` / `ensures:` contract clauses on `fn` declarations. Use them when the function has a type-level or arithmetic pre/postcondition that the current checker understands.

```ash
pub fn positive_successor(n: Int) -> Int
    requires: n >= 0
    ensures: result > 0
{
    n + 1
}
```

This is a reference example for the declaration shape. Check current typechecker support before relying on a particular proposition in production code.

## Builtin functions are declarations, not bodies

A `builtin fn` has a signature and a semicolon. It has no Ash-level body.

```ash
pub builtin fn len<T>(items: List<T>) -> Int;
```

Use `builtin fn` only for functions provided by the compiler/runtime/stdlib boundary. Ordinary user code should use `fn` with a body.

## Common mistakes

- Do not write `ret` in a pure function body; the final expression is the return value.
- Do not put a capability parameter in a pure function signature.
- Do not give `builtin fn` a body.
- Do not document `extern fn` as current user-facing function syntax unless a later spec and parser implementation land it.
