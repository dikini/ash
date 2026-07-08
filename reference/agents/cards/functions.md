---
id: ref.agents.card.functions
title: Functions Card
kind: agent-card
audience: [agent]
authority: derivative
status: current
stability: alpha
owner: reference-corpus
last_verified: 2026-05-26
verified_against:
  git_commit: 0874763
  specs:
    - docs/spec/SPEC-071-REFERENCE-CORPUS-METADATA-AND-MAINTENANCE.md
    - docs/spec/SPEC-027-PURE-FUNCTIONS.md
    - docs/spec/SPEC-031-FIRST-CLASS-FUNCTIONS.md
    - docs/spec/SPEC-072-TOWER-CALLABLE-TYPE-AND-CLOSURE-SYNTAX.md
  tasks:
    - docs/plan/tasks/TASK-954-functions-reference-chapter.md
    - docs/plan/tasks/TASK-961-callable-syntax-reference-docs.md
  code:
    []
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
  - Canonical page changes
  - function syntax or parser changes
  - SPEC-027 or SPEC-031 changes
---
# Functions Card

canonical_page: ref.language.functions
canonical_page_path: ../../language/functions.md
dependency_order: 1
warning: Pure functions compute values; they do not run provider-backed effects, spawn processes,
or implicitly enter application runtime boundaries.

## Use

Retrieve the canonical page first, then use this card as a compact operational checklist for writing or editing Ash functions.

## Function syntax quick reference

Module-level pure function:

```ash
pub fn name(param: Type) -> ReturnType {
    final_expression
}
```

Generic pure function:

```ash
pub fn identity<T>(value: T) -> T {
    value
}
```

Function-typed parameter:

```ash
pub fn apply<T, U>(value: T, f: (T) -> U) -> U {
    f(value)
}
```

Anonymous function value inside a function body:

```ash
pub fn demo(n: Int) -> Int {
    let f = fn(x: Int) -> Int { x + 1 };
    f(n)
}
```

Closure shorthand inside a function body:

```ash
pub fn demo(n: Int) -> Int {
    let f = |x| -> x + 1;
    f(n)
}
```

Builtin declaration:

```ash
pub builtin fn len<T>(items: List<T>) -> Int;
```

## Rules for agents

- Use tail-expression return in `fn`; do not write `ret`.
- Keep parameter types explicit.
- Use `(T) -> U` and `(A, B) -> C` for pure function values; treat historical `Fn(...) -> ...` as removed syntax.
- Use `::` for module-qualified function calls.
- Treat `builtin fn` as a declaration with no body.
- Do not call capabilities, `invoke(...)`, `act`, `observe`, `send`, `receive`, `spawn`, or workflow obligations inside pure functions.
- Do not assume partial application.
- Do not use historical higher-stratum callable arrows `-*>, =>, =*>` as implemented syntax. `=>` remains legal for match arms, not pure closures.
- Do not serialize or send local closures across process/workflow boundaries.
- If a function constructs effectful work, describe it as construction, not as executing the effect.

## Retrieval tags

- ash
- functions
- pure-functions
- fn
- builtin-fn
- closure
- anonymous-function
- Fn-type
- module-functions
- tail-expression-return

## Must check before editing

- ../../language/functions.md
- ../../language/functions/declarations.md
- ../../language/functions/local-and-anonymous.md
- ../../language/functions/boundaries.md
- ../../language/functions/authority-and-traceability.md
- ../../../docs/spec/SPEC-027-PURE-FUNCTIONS.md
- ../../../docs/spec/SPEC-031-FIRST-CLASS-FUNCTIONS.md
- ../../../crates/ash-parser/src/parse_module.rs
- ../../../crates/ash-parser/src/parse_expr.rs

## Forbidden stale claims

- Pure functions may perform provider-backed effects.
- `ret` is the normal way to return from a pure function.
- Module-level functions are runtime closure values.
- Local closures are serializable process/workflow payloads.
- Partial application is supported.
- Agent cards are normative specs.
