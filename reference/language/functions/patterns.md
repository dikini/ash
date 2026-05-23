---
id: ref.language.functions.patterns
title: Functions with Pattern Matching
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
    - crates/ash-parser/src/parse_pattern.rs
    - crates/ash-typeck/src/check_pattern.rs
  tests:
    []
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
  - function parser or typechecker changes
---
# Functions with Pattern Matching

## Summary

Pure functions can use patterns to inspect values. Use `match` when all possible constructors need handling, and use `let` patterns only when the shape is known or intentionally constrained.

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

Reference examples should prefer exhaustive matches. If a match is intentionally partial or uses `panic`, say so in the surrounding prose.

## Patterns in `let`

A `let` pattern binds names from a value shape.

```ash
pub fn extract_ok(res: Result<Int, String>) -> Int {
    let Ok { value: n } = res;
    n
}
```

Use this form carefully. For values that may have several constructors, `match` gives a clearer daily-use reference shape.
