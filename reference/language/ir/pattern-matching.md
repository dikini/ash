---
id: ref.language.ir.pattern_matching
title: Pattern Matching in CPS IR
kind: reference
audience: [human, agent]
authority: canonical-adjacent
status: current
stability: alpha
owner: language
last_verified: 2026-07-28
verified_against:
  git_commit: null
  specs:
    - docs/spec/SPEC-098b-TARGET-IR.md
    - docs/spec/SPEC-099c-CPS-IR-EXPANDED-OPERATIONAL-SEMANTICS.md
  tasks:
    - docs/plan/tasks/TASK-1592-cps-ir-conditionals-data.md
    - docs/plan/tasks/TASK-2037-engine-owned-cps-executor-and-runtime-crate-rename.md
    - docs/plan/tasks/TASK-1966-docs-reference-historical-quarantine.md
  code:
    - crates/ash-core/src/cps.rs
    - crates/ash-engine/src/private_cps/mod.rs
  tests:
    - crates/ash-engine/src/private_cps/tests/task_1592_cps_ir.rs
  examples: []
related:
  depends_on:
    - ref.language.cps-ir
    - ref.language.ir.constructors
    - ref.language.ir.tuples
  explains: []
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/spec/SPEC-098b-TARGET-IR.md
refresh_trigger:
  - crates/ash-core/src/cps.rs changes
  - crates/ash-engine/src/private_cps/mod.rs changes
  - docs/spec/SPEC-098b-TARGET-IR.md changes
  - docs/spec/SPEC-099c-CPS-IR-EXPANDED-OPERATIONAL-SEMANTICS.md changes
---

# Pattern Matching in CPS IR

## Overview

Pattern matching in the CPS IR is implemented via `Term::Match`, which dispatches on constructor tags in the first element of a tuple. It is the runtime mechanism for sum type elimination.

This is an internal CPS rule. External execution goes through an admitted request to `Engine`; the
kernel that realizes the rule is Engine-private.

## Lowering Rule

An Ash pattern match:

```ash
match s with
  Circle(r) -> ...
  Rect(w, h) -> ...
```

Lowers to CPS IR as:

```lisp
(match s
  (("Circle" (letprim r (tuple_get 1 s) ...body1...)))
  (("Rect" (letprim w (tuple_get 1 s)
              (letprim h (tuple_get 2 s) ...body2...))))
  (default (trap MatchFailure)))
```

## Runtime Semantics

The `match` term:

1. Evaluates the scrutinee atom to a `Value`
2. Expects a `Value::Tuple` whose first element is `Value::Atom(ConstructorName(n))`
3. Matches `n` against the arm tags
4. Executes the body of the first matching arm
5. If no arm matches and a default is provided, executes the default
6. If no arm matches and no default is provided, traps with `MatchError`

**Success:** Executes the matching arm's body in the current environment.

**Failure:** Non-tuple scrutinee, empty tuple, or unmatched constructor without default → `Stuck(MatchError)`.

## CPS IR Data Model

```rust
Term::Match {
    scrutinee: Atom,
    arms: Vec<(Name, Box<Term>)>,
    default: Option<Box<Term>>,
}
```

Each arm is a `(constructor_name, body)` pair. The `default` is optional.

## Dynamic Scope Prevention

The `match` term does not introduce new bindings. Any variables used in arm bodies must be bound in the enclosing environment. This is consistent with the CPS IR's explicit binding discipline.

## Cross-References

- [Sum Type Constructors](constructors.md) — Constructor tags
- [Tuples in CPS IR](tuples.md) — Tuple construction
- [SPEC-098b: Target IR](../../../docs/spec/SPEC-098b-TARGET-IR.md) — IR grammar
- [SPEC-099c: Expanded Operational Semantics](../../../docs/spec/SPEC-099c-CPS-IR-EXPANDED-OPERATIONAL-SEMANTICS.md) — §3
