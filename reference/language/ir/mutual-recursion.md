---
id: ref.language.ir.mutual_recursion
title: Mutual Recursion in CPS IR
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
    - docs/plan/tasks/TASK-1596-cps-ir-letrec-recursion.md
    - docs/plan/tasks/TASK-1616-cps-ir-speculative-fixtures.md
    - docs/plan/tasks/TASK-1966-docs-reference-historical-quarantine.md
  code:
    - crates/ash-core/src/cps.rs
    - crates/ash-interp/src/cps/mod.rs
  tests:
    - crates/ash-interp/tests/task_1596_cps_ir.rs
    - crates/ash-interp/tests/task_1616_cps_ir_speculative_fixtures.rs
  examples: []
related:
  depends_on:
    - ref.language.cps-ir
    - ref.runtime.cps-interpreter
  explains: []
  supersedes: []
  superseded_by: null
  historical_rationale:
    - docs/spec/SPEC-098b-TARGET-IR.md
refresh_trigger:
  - crates/ash-core/src/cps.rs changes
  - crates/ash-interp/src/cps/mod.rs changes
  - docs/spec/SPEC-098b-TARGET-IR.md changes
  - docs/spec/SPEC-099c-CPS-IR-EXPANDED-OPERATIONAL-SEMANTICS.md changes
---

# Mutual Recursion in CPS IR

## Overview

Mutual recursion in the CPS IR is desugared to a single `LetRec` binding a tuple of lambdas. Each lambda captures the tuple name via `rec_binding` so it can access the other functions through tuple element extraction.

## Lowering Rule

An Ash mutual recursion:

```ash
letrec even(n) = if n == 0 then true else odd(n - 1)
       odd(n)  = if n == 0 then false else even(n - 1)
```

Lowers to CPS IR as:

```lisp
(letrec pair
  (tuple
    (lam [n] k
      (letprim is_zero = eq n 0 in
       if is_zero then
         (jump k true)
       else
         (letprim n_minus_1 = sub n 1 in
          (letprim odd_fn = tuple_get 1 pair in
           (call odd_fn [n_minus_1] k)))))
    (lam [n] k
      (letprim is_zero = eq n 0 in
       if is_zero then
         (jump k false)
       else
         (letprim n_minus_1 = sub n 1 in
          (letprim even_fn = tuple_get 0 pair in
           (call even_fn [n_minus_1] k))))))
  (letprim even_fn = tuple_get 0 pair
    ...))
```

## Why This Works

1. **Placeholder binding:** `pair` is initially bound to `Null` via `LetRec`
2. **Tuple construction:** The tuple is built with lambdas; `LetRec` automatically marks all nested lambdas with `rec_binding: Some("pair")`
3. **Backfill:** `pair` is updated to the actual tuple
4. **Call-time overlay:** When a lambda is called, `eval_call` overlays the call-site binding for `pair` into the lambda's execution environment
5. **Access:** The lambda body uses `tuple_get` on `pair` to extract the other function

## rec_binding Mechanism

The `rec_binding` field on `Value::Lam` is the key to scoped mutual recursion:

```rust
Value::Lam {
    params: vec!["n".to_string()],
    cont: "k".to_string(),
    body: Box::new(...),
    captured_env: Env::new(),
    rec_binding: Some("pair".to_string()),  // ← marks this lambda as recursive
    row: EffectRow::default(),
}
```

When `eval_call` executes a lambda with `rec_binding: Some(name)`, it:

1. Starts from the lambda's captured environment
2. Looks up `name` in the **call-site** environment
3. Overlays that binding into the execution environment

This ensures the recursive tuple is visible inside the lambda body, but **only** the named binding is overlaid — no other call-site variables leak in.

## Dynamic Scope Prevention

The overlay is narrowly scoped:

- Only the binding named in `rec_binding` is overlaid
- Non-recursive lambdas (`rec_binding: None`) receive no overlay
- Caller-local variables are never visible inside the lambda body

This prevents accidental dynamic scoping while enabling mutual recursion.

## CPS IR Data Model

```rust
Value::Lam {
    // ... existing fields ...
    rec_binding: Option<Name>,  // None for ordinary lambdas, Some(name) for recursive
}
```

The `#[serde(default)]` attribute ensures backward compatibility: lambdas serialized without `rec_binding` deserialize with `None`.

## Cross-References

- [Tuples in CPS IR](tuples.md) — Tuple construction and element access
- [SPEC-099b: Base Operational Semantics](../../../docs/spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md) — §5.1 (LetRec)
- [SPEC-099c: Expanded Operational Semantics](../../../docs/spec/SPEC-099c-CPS-IR-EXPANDED-OPERATIONAL-SEMANTICS.md) — §4
