---
id: ref.language.ir.constructors
title: Sum Type Constructors in CPS IR
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
    - ref.language.ir.tuples
  explains:
    - ref.language.ir.pattern_matching
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

# Sum Type Constructors in CPS IR

## Overview

Sum type constructors in the CPS IR are represented using `Atom::ConstructorName` tags inside tuples. The tag is the first element; subsequent elements are the constructor's fields.

This describes internal CPS representation and semantics. Applications submit admitted requests to
`Engine`; they do not invoke a CPS evaluator directly.

## Lowering Rule

An Ash sum type and construction:

```ash
type Shape = Circle { radius: Float } | Rect { width: Float, height: Float };
let s = Circle { radius: 5.0 };
```

Lowers to CPS IR as:

```lisp
(letval s (tuple ((atom (constructor "Circle")) (atom (float 5.0))))
  ...)
```

## Constructor Tag

The tag `ConstructorName("Circle")` is an inert atom used for discrimination in pattern matching. It is not a function — it exists only to identify which constructor was used.

## Pattern Matching

Pattern matching on sum types uses `Term::Match`:

```ash
match s with
  Circle(r) -> ...
  Rect(w, h) -> ...
```

Lowers to:

```lisp
(match s
  (("Circle" ...body1...))
  (("Rect" ...body2...))
  (default (trap MatchFailure)))
```

The `match` term evaluates the scrutinee, extracts the constructor tag from the first tuple element, and dispatches to the matching arm.

## CPS IR Data Model

```rust
Atom::ConstructorName(Name)
```

Constructor names are strings that identify the variant. They are serialized as `"ConstructorName"` in S-expression format.

## Cross-References

- [Tuples in CPS IR](tuples.md) — Tuple construction and element access
- [Pattern Matching in CPS IR](pattern-matching.md) — Match dispatch
- [SPEC-098b: Target IR](../../../docs/spec/SPEC-098b-TARGET-IR.md) — IR grammar
- [SPEC-099c: Expanded Operational Semantics](../../../docs/spec/SPEC-099c-CPS-IR-EXPANDED-OPERATIONAL-SEMANTICS.md) — §3
