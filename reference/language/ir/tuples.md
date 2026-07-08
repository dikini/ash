---
id: ref.language.ir.tuples
title: Tuples in CPS IR
kind: reference
audience: [human, agent]
authority: canonical-adjacent
status: current
stability: alpha
owner: language
last_verified: 2026-07-07
verified_against:
  git_commit: null
  specs:
    - docs/spec/SPEC-098b-TARGET-IR.md
    - docs/spec/SPEC-099c-CPS-IR-EXPANDED-OPERATIONAL-SEMANTICS.md
  tasks:
    - docs/plan/tasks/TASK-1592-cps-ir-conditionals-data.md
    - docs/plan/tasks/TASK-1966-docs-reference-historical-quarantine.md
  code:
    - crates/ash-core/src/cps.rs
  tests:
    - crates/ash-interp/tests/task_1592_cps_ir.rs
  examples: []
related:
  depends_on:
    - ref.language.cps-ir
  explains:
    - ref.language.ir.constructors
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/spec/SPEC-098b-TARGET-IR.md
refresh_trigger:
  - crates/ash-core/src/cps.rs changes
  - docs/spec/SPEC-098b-TARGET-IR.md changes
  - docs/spec/SPEC-099c-CPS-IR-EXPANDED-OPERATIONAL-SEMANTICS.md changes
---

# Tuples in CPS IR

## Overview

Tuples in the CPS IR are ordered sequences of `Value` elements. They are the runtime representation of Ash tuple types and sum type constructors.

## Lowering Rule

An Ash tuple construction:

```ash
let t = (1, 2, 3);
```

Lowers to CPS IR as:

```lisp
(letval t (tuple ((atom (int 1)) (atom (int 2)) (atom (int 3))))
  ...)
```

## Element Access

Accessing a tuple element by index:

```ash
t.1
```

Lowers to:

```lisp
(letprim second (tuple_get 1 t)
  ...)
```

## Runtime Semantics

Tuple construction evaluates each element recursively. Element access resolves the tuple variable, then extracts the element at the given index.

**Success:** Returns the `Value` at the specified index.

**Failure:** If the index is out of bounds, the primitive returns an error (`InvalidPrimArgs`).

## CPS IR Data Model

```rust
Value::Tuple {
    elems: Vec<Value>,
}
```

Elements are stored in order. Indexing is zero-based.

## Sum Type Constructors

Sum type constructors are represented as tuples where the first element is a `ConstructorName` tag:

```ash
type Shape = Circle { radius: Float } | Rect { width: Float, height: Float };
let s = Circle { radius: 5.0 };
```

Lowers to:

```lisp
(letval s (tuple ((atom (constructor "Circle")) (atom (float 5.0))))
  ...)
```

## Cross-References

- [SPEC-098b: Target IR](../../../docs/spec/SPEC-098b-TARGET-IR.md) — IR grammar
- [SPEC-099c: Expanded Operational Semantics](../../../docs/spec/SPEC-099c-CPS-IR-EXPANDED-OPERATIONAL-SEMANTICS.md) — §2.2, §2.4
