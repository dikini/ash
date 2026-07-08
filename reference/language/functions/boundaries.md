---
id: ref.language.functions.boundaries
title: Function Boundaries and Common Mistakes
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
    - crates/ash-typeck/src/lib.rs
    - crates/ash-typeck/src/runtime_verification.rs
  tests:
    []
  examples:
    []
related:
  depends_on:
    - ref.language.functions
    - ref.language.act
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
# Function Boundaries and Common Mistakes

## Summary

Pure functions are for value computation. Runtime effects, provider authority, process control, and
contract evidence belong to target effect rows, provider profiles, process/channel helpers, and
application runtime boundaries rather than pure code.

## Pure functions vs effects

Pure functions must not execute provider-backed effects, process control, or runtime admission
work. Historical examples that returned removed tower carriers are not current source guidance.

## Effectful APIs are not pure functions

Do not document effectful APIs as ordinary pure computation. Use current target examples and
checked standard-library helpers when describing effectful behavior.

## `builtin fn`

A `builtin fn` exposes a runtime/compiler-provided pure operation through a signature.

```ash
pub builtin fn len<T>(items: List<T>) -> Int;
```

It has no Ash-level body. If the operation requires external authority, it belongs in capability/provider machinery instead of as an ordinary pure builtin.

## No implicit tower lifts

Pure code does not implicitly become `Act`, `Proc`, or `Workflow`.

```ash
pub fn value() -> Int { 1 }
```

If application or process code needs that value, pass it through the current target boundary that
owns the effect: a provider profile, process/channel helper, contract/evidence helper, or
application runtime entry.

## Common mistakes

| Mistake | Why it is wrong | Use instead |
| --- | --- | --- |
| Calling `invoke(...)` inside `fn` | `invoke` dispatches through runtime capability machinery. | Put the call in an `Act`/runtime context. |
| Returning a workflow from a pure helper and calling it pure | The returned value belongs to the Workflow layer. | Say the function constructs workflow data. |
| Using `ret` in `fn` | Pure functions use tail-expression return. | Put the return value as the final expression. |
| Treating module functions as serializable closures | Module functions are definitions, not closure payloads. | Use local closures only inside supported local scopes. |
| Assuming partial application | Current function application requires full arity. | Wrap with an explicit local closure. |
