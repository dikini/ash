---
id: ref.language.functions
title: Functions and Pure Code
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
    - docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md
    - docs/spec/SPEC-072-TOWER-CALLABLE-TYPE-AND-CLOSURE-SYNTAX.md
  tasks:
    - docs/plan/tasks/TASK-954-functions-reference-chapter.md
    - docs/plan/tasks/TASK-961-callable-syntax-reference-docs.md
  code:
    - crates/ash-parser/src/parse_module.rs
    - crates/ash-parser/src/parse_expr.rs
    - crates/ash-parser/src/surface.rs
    - crates/ash-parser/src/lower.rs
    - crates/ash-core/src/ast.rs
  tests:
    []
  examples:
    - examples/01-basics/03-expressions.ash
related:
  depends_on:
    []
  explains:
    - ref.language.act
  supersedes: []
  superseded_by: null
  historical_rationale:
    []
refresh_trigger:
  - SPEC-027 changes
  - SPEC-031 changes
  - function parser or typechecker changes
---
# Functions and Pure Code

## Summary

A pure function is Ash code that computes a value without entering the runtime-managed effect tower. It can take parameters, bind local values, branch, match on data, call other pure functions, and return a value through the last expression of its body.

Pure code sits below the rest of the Ash tower:

```text
Pure < Act < Proc < Workflow
```

Crossing that boundary is explicit. A pure function may return an `Act<T>` value as data, but performing the effect belongs to the runtime-managed `Act`, `Proc`, or `Workflow` layer.

## Concept: what is a pure function?

A pure function describes a deterministic transformation from input values to an output value. The type checker treats the body as ordinary value-producing expression code and rejects constructs that require capabilities, process control, workflow obligations, or runtime provider dispatch.

Use pure functions for ordinary computation: formatting a value, selecting a branch, mapping a `Result`, building a record, or extracting data from a constructor. When the code must observe external state, call a provider, spawn work, or enforce workflow contracts, move that behavior into the appropriate effectful layer instead of hiding it inside a function.

## Status

This chapter is the first expanded reference chapter after the Phase 124 skeleton. It is current for the cited alpha parser/typechecker evidence, but it is still conservative about local closure runtime maturity and higher-order behavior. Sections call out known boundaries where the working specs describe future or partial behavior.

## Chapter contents

| Page | Use it for |
| --- | --- |
| [Function declaration syntax](functions/declarations.md) | Module-level `fn`, visibility, generics, return types, `where`, contracts. |
| [Function bodies and expressions](functions/bodies-and-expressions.md) | Tail-expression return, `let`, blocks, `if`, `match`, `panic`, expression limits. |
| [Local and anonymous functions](functions/local-and-anonymous.md) | `fn(...) { ... }`, named local functions, closure shorthand `|x| -> ...`, capture rules. |
| [Calling functions and using function values](functions/calls-and-values.md) | Direct calls, module-qualified calls, `(A, B) -> C` callable types, higher-order patterns. |
| [Functions with pattern matching](functions/patterns.md) | Matching constructor values, using patterns in `let`, and exhaustiveness expectations. |
| [Boundaries and common mistakes](functions/boundaries.md) | Pure vs `Act`, `builtin fn`, no implicit lifts, no capability calls in pure bodies. |
| [Implementation notes](functions/implementation-notes.md) | Parser/core/typechecker/lowering details that explain the current alpha surface. |
| [Authority and traceability](functions/authority-and-traceability.md) | Specs, code paths, tasks, examples, and drift notes used by this chapter. |

## Quick examples

A module-level pure function:

```ash
pub fn add_one(n: Int) -> Int {
    n + 1
}
```

A generic pure function that accepts another pure function:

```ash
pub fn apply_twice<T>(x: T, f: (T) -> T) -> T {
    f(f(x))
}
```

A local anonymous function value:

```ash
pub fn demo(n: Int) -> Int {
    let double = fn(x: Int) -> Int { x * 2 };
    double(n)
}
```

Closure shorthand for a small expression:

```ash
pub fn demo(n: Int) -> Int {
    let double = |x| -> x * 2;
    double(n)
}
```

## Known limitations

- Module-level functions are exported definitions, not runtime closure values.
- Local closures are alpha-scoped and should not be treated as serializable process/workflow values.
- Partial application is not part of the current function contract.
- Higher-stratum callable arrows `-*>`, `=>`, and `=*>` are reserved. Use `->` for pure callables and return `Act<T>`, `Proc<T>`, or `Workflow<T>` values from pure smart constructors when you are only building tower values.
- `extern fn` is not documented here as current Ash syntax. Use `builtin fn` for runtime-provided pure functions exposed by stdlib/compiler surfaces.
- Older docs may describe planned function features more broadly than the current reference chapter claims.

## Agent notes

Agents should treat this chapter as the daily-use entry point for pure functions. Use [Authority and traceability](functions/authority-and-traceability.md) when editing implementation code or adjudicating drift, but do not force ordinary readers to reconstruct syntax from the working specs.
