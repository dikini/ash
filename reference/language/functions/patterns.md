---
id: ref.language.functions.patterns
title: Functions with Pattern Matching
kind: reference
audience: [human, agent]
authority: canonical-adjacent
status: current
stability: alpha
owner: language
last_verified: 2026-06-02
verified_against:
  git_commit: 2b35ab6
  specs:
    - docs/spec/SPEC-027-PURE-FUNCTIONS.md
    - docs/spec/SPEC-031-FIRST-CLASS-FUNCTIONS.md
    - docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md
    - docs/spec/SPEC-076-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md
  tasks:
    - docs/plan/tasks/TASK-954-functions-reference-chapter.md
    - docs/plan/tasks/TASK-1008-runtime-defensive-pattern-error-cleanup-closeout.md
  code:
    - crates/ash-parser/src/parse_pattern.rs
    - crates/ash-typeck/src/check_pattern.rs
    - crates/ash-typeck/src/check_expr/mod.rs
  tests:
    - crates/ash-parser/tests/task_1007_if_let_parser_entrypoints.rs
    - crates/ash-typeck/tests/task_1003_let_irrefutability.rs
    - crates/ash-typeck/tests/task_1005_match_exhaustiveness.rs
    - crates/ash-typeck/tests/task_1007_if_let_receive_contract.rs
  examples:
    []
related:
  depends_on:
    - ref.language.functions
    - ref.language.functions.expressions
  explains:
    []
  supersedes: []
  superseded_by: null
  historical_rationale:
    []
refresh_trigger:
  - SPEC-027 changes
  - SPEC-031 changes
  - SPEC-076 changes
  - function parser or typechecker changes
---
# Functions with Pattern Matching

## Summary

Pure functions can use patterns to inspect values. Use `match` when all possible constructors need handling. Use `let` patterns only for irrefutable shapes that the type checker can prove always match.

## Matching constructors

```ash
pub fn is_ok<T, E>(res: Result<T, E>) -> Bool {
    match res {
        Ok { value: _ } => true,
        Err { error: _ } => false
    }
}
```

Each arm produces a value. The type checker expects the arms to agree on result type.

## Extracting payloads

```ash
pub fn unwrap_or<T, E>(res: Result<T, E>, default: T) -> T {
    match res {
        Ok { value: v } => v,
        Err { error: _ } => default
    }
}
```

Pattern variables are scoped to the arm body.

## Wildcards

Use `_` when the value is intentionally ignored.

```ash
pub fn has_error<T, E>(res: Result<T, E>) -> Bool {
    match res {
        Ok { value: _ } => false,
        Err { error: _ } => true
    }
}
```

## Exhaustiveness

Matches in value-producing positions must be exhaustive. A match over a closed sum type, such as `Result<T, E>`, must cover every constructor unless it has a wildcard/default arm. Non-exhaustive matches are type-checking errors rather than runtime fallbacks for checked source.

## Patterns in `let`

A `let` pattern binds names from a value shape only when the pattern is irrefutable for the scrutinee type. Variables and `_` are always irrefutable. Tuple and record patterns are allowed when their nested fields are also irrefutable. Variant patterns over ordinary multi-constructor sum types are refutable and must not be used as plain `let` binders.

```ash
pub fn second(pair: (String, Int)) -> Int {
    let (_, n) = pair;
    n
}
```

For values that may have several constructors, use `match` or explicit `if let ... else` instead of a refutable `let` binder.

```ash
pub fn ok_or_zero(res: Result<Int, String>) -> Int {
    if let Ok { value: n } = res then { n } else { 0 }
}
```

The `else` branch is mandatory. Pattern variables introduced by `if let` are visible only in the then branch; the else branch checks under the original environment.
