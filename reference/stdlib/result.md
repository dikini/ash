---
id: ref.stdlib.result
title: Result Standard Library
kind: reference
audience: [human, agent]
authority: canonical-adjacent
status: current
stability: alpha
owner: stdlib
last_verified: 2026-06-11
verified_against:
  git_commit: 61efd59f
  release_tag: null
  ash_version: unreleased-alpha
  specs:
    - docs/spec/SPEC-050-OPERATIONAL-BOTTOM-AND-SCOPED-HANDLING.md
    - docs/spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md
    - docs/spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md
    - docs/spec/SPEC-075-REFERENCE-SLICE-2-RUNTIME-TOOLCHAIN-MAINTENANCE.md
  tasks:
    - docs/plan/tasks/TASK-997-reference-stdlib-tower-pages.md
  code:
    - std/src/result.ash
    - std/src/lib.ash
  tests:
    - crates/ash-cli/tests/stdlib_corpus_check.rs
    - crates/ash-typeck/tests/alpha_generalized_do_full_bind_lowering.rs
  examples:
    - tests/std/result.ash
related:
  depends_on:
    - ref.stdlib.index
    - ref.language.generalized_do
  explains:
    - ref.stdlib.act
    - ref.status.known_limitations
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/spec/SPEC-050-OPERATIONAL-BOTTOM-AND-SCOPED-HANDLING.md
refresh_trigger:
  - std/src/result.ash changes
  - std/src/lib.ash changes
  - tests/std/result.ash changes
  - docs/spec/SPEC-050-OPERATIONAL-BOTTOM-AND-SCOPED-HANDLING.md changes
  - docs/spec/SPEC-054-GENERALIZED-TYPED-DO-NOTATION.md changes
  - docs/spec/SPEC-069-ALPHA-VISIBLE-TOWER-ALGEBRA-AND-DO-LOWERING.md changes
---

# Result Standard Library

`Result<T, E>` is a public domain value type. It represents a normal completed value that is either `Ok { value: T }` or `Err { error: E }`.

`Result` is not `Act`, not operational bottom, and not the runtime's hidden effect channel. Use [Act](act.md) for effectful computations and this page for ordinary success/failure values.

## Public Type and Constructors

`std/src/result.ash` defines:

```ash
pub type Result<T, E> = Ok { value: T } | Err { error: E };
```

`std/src/lib.ash` re-exports `Result`, `Ok`, and `Err`.

## Public Helper Functions

`std/src/result.ash` currently exposes:

| Name | Public shape | Notes |
| --- | --- | --- |
| `is_ok` | `Result<T, E> -> Bool` | True for `Ok`. |
| `is_err` | `Result<T, E> -> Bool` | True for `Err`. |
| `unwrap` | `Result<T, E> -> T` | Returns the value or panics on `Err`. |
| `unwrap_err` | `Result<T, E> -> E` | Returns the error or panics on `Ok`. |
| `unwrap_or` | `Result<T, E>, T -> T` | Returns the value or a default. |
| `map` | `Result<T, E>, (T) -> U -> Result<U, E>` | Maps the `Ok` value. |
| `map_err` | `Result<T, E>, (E) -> F -> Result<T, F>` | Maps the `Err` value. |
| `and_then` | `Result<T, E>, (T) -> Result<U, E> -> Result<U, E>` | Chains `Ok` values. |
| `ok` | `Result<T, E> -> Option<T>` | Converts `Ok` to `Some`, `Err` to `None`. |
| `err` | `Result<T, E> -> Option<E>` | Converts `Err` to `Some`, `Ok` to `None`. |

`std/src/lib.ash` also re-exports several helpers under prelude-style aliases such as `unwrap_res`, `unwrap_or_res`, `map_res`, and `err_opt`.

## Example

`tests/std/result.ash` exercises the current helper surface. The following snippet is illustrative and matches that test style:

```ash
let ok_val = Ok { value: 42 };
let err_val = Err { error: "oops" };

assert is_ok(ok_val);
assert is_err(err_val);
assert unwrap_or(err_val, 0) == 0;
```

This page does not claim that the snippet alone is a standalone runnable workflow; it is reference-only unless embedded in the current test harness shape used by `tests/std/result.ash`.

## Domain Failure Versus Operational Bottom

`Err { error: e }` is a normal domain value inside `Result<T, E>`. A computation returning `Result<T, E>` completes normally when it returns either `Ok` or `Err`.

`fail e` is operational bottom. It means the current computation does not complete normally. It must not be documented as implicitly constructing `Err { error: e }`, including inside `do:Result<_, E>` evidence paths.

Current typechecker evidence can lower selected `do:Result<_, E>` shapes through `result::and_then` in focused tests, but that does not change the operational-bottom rule.

## Limitations

- `unwrap` and `unwrap_err` panic on the wrong variant.
- `Result` helpers are pure domain helpers; they do not grant capabilities, run processes, or admit workflows.
- `Result` is not a substitute for `Act<Result<A, E>>` when effectful work also needs domain-level success/failure.
