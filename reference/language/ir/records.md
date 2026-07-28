---
id: ref.language.ir.records
title: Records in CPS IR
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
  explains:
    - ref.language.types.records
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

# Records in CPS IR

## Overview

Records in the CPS IR are ordered collections of named fields, where each field holds a `Value`. They are the runtime representation of Ash record types.

This is an internal representation rule. Programs reach it only through Engine admission and an
admitted request; there is no public CPS evaluator API.

## Lowering Rule

An Ash record type declaration and construction:

```ash
type Point = { x: Int, y: Int };
let p = Point { x: 1, y: 2 };
```

Lowers to CPS IR as:

```lisp
(letval p (record ((x (atom (int 1))) (y (atom (int 2)))))
  ...)
```

## Field Access

Accessing a record field:

```ash
p.x
```

Lowers to:

```lisp
(letprim x_val (record_get x p)
  ...)
```

## Runtime Semantics

Record construction evaluates each field value recursively. Field access resolves the record variable, then searches for the field by name.

**Success:** Returns the `Value` bound to the field name.

**Failure:** If the field name is not found, the primitive returns an error (`InvalidPrimArgs`).

## CPS IR Data Model

```rust
Value::Record {
    fields: Vec<(Name, Value)>,
}
```

Fields are stored as `(name, value)` pairs. The order is preserved from the source.

## Cross-References

- [SPEC-098b: Target IR](../../../docs/spec/SPEC-098b-TARGET-IR.md) — IR grammar
- [SPEC-099c: Expanded Operational Semantics](../../../docs/spec/SPEC-099c-CPS-IR-EXPANDED-OPERATIONAL-SEMANTICS.md) — §2.1, §2.3
