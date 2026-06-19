---
id: spec.ash.operational-semantics.expanded
title: Ash CPS IR Expanded Operational Semantics
description: Big-step operational semantics for the expanded CPS IR (Phase 160)
kind: spec
audience: [human, agent]
authority: design
status: draft
stability: alpha
owner: language
last_verified: 2026-06-19
verified_against:
  specs:
    - docs/spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md
    - docs/spec/SPEC-098b-TARGET-IR.md
    - docs/plan/PLAN-160-CPS-IR-RUNTIME-EXPANSION.md
---

# SPEC-099c: CPS IR Expanded Operational Semantics

**Status:** Draft — extends SPEC-099b with structured data and pattern matching
**Scope:** Operational semantics for the expanded CPS IR implemented in Phase 160
**Depends on:** SPEC-099b (Base Operational Semantics), SPEC-098b (Target IR)

## §1 Syntax Extensions (to SPEC-099b §1)

### §1.2 Values (extended)

```text
v ::= ... (from SPEC-099b §1.2)
    | Record { fields: [(x, v), ...] }
    | Tuple { elems: [v, ...] }
```

**Note:** The spec grammar (SPEC-098b) uses `Atom` for fields/elements because that's the frontend IR. The operational semantics uses `Value` because atoms are resolved to values during evaluation.

### §1.3 Atoms (extended)

```text
a ::= ... (from SPEC-099b §1.1)
    | ConstructorName(n)
```

Constructor names are used as tags in the first element of tuples for pattern matching.

### §1.4 Terms (extended)

```text
t ::= ... (from SPEC-099b §1.3)
    | Match { scrutinee: a, arms: [(n, t), ...], default: t? }
```

### §1.5 Primitive Operations (extended)

```text
⊙ ::= ... (from SPEC-099b §1.3)
    | RecordGet(x)
    | TupleGet(i)
```

## §2 Record and Tuple Rules

### §2.1 Record Construction

```text
eval(vᵢ, η) = vᵢ' for each field (xᵢ, vᵢ)
-----------------------------------
⟨Record { fields: [(x₁, v₁), ...] }, η⟩ ⇓ Record { fields: [(x₁, v₁'), ...] }
```

Fields are evaluated recursively (resolving any nested atoms to values).

### §2.2 Tuple Construction

```text
eval(vᵢ, η) = vᵢ' for each element
-----------------------------------
⟨Tuple { elems: [v₁, ...] }, η⟩ ⇓ Tuple { elems: [v₁', ...] }
```

Elements are evaluated recursively.

### §2.3 RecordGet

```text
eval(a, η) = a'
resolve_value(a', η) = Record { fields: [... (x, v) ...] }
-----------------------------------
⟨RecordGet(x), [a], η, χ⟩ ⇓ v
```

Resolve the atom to a record value, find the field by name, return the value.

### §2.4 TupleGet

```text
eval(a, η) = a'
resolve_value(a', η) = Tuple { elems: [...] }
elems[i] = v
-----------------------------------
⟨TupleGet(i), [a], η, χ⟩ ⇓ v
```

Resolve the atom to a tuple value, extract the element at index `i`, return the value.

### §2.5 RecordGet (missing field)

```text
eval(a, η) = a'
resolve_value(a', η) = Record { fields: [...] }
no field named x
-----------------------------------
⟨RecordGet(x), [a], η, χ⟩ ⇓ Stuck(InvalidPrimArgs)
```

### §2.6 TupleGet (out of bounds)

```text
eval(a, η) = a'
resolve_value(a', η) = Tuple { elems: [...] }
i ≥ length(elems)
-----------------------------------
⟨TupleGet(i), [a], η, χ⟩ ⇓ Stuck(InvalidPrimArgs)
```

## §3 Match Dispatch Rules

### §3.1 Match (matching arm)

```text
eval(a, η) = a'
resolve_value(a', η) = Tuple { elems: [Atom(ConstructorName(n)), ...] }
arms contains (n, t)
-----------------------------------
⟨Match(a, arms, default), η, χ⟩ ⇓ ⟨t, η, χ⟩
```

The scrutinee must be a tuple whose first element is a `ConstructorName`. The constructor name is matched against the arm tags. If a match is found, the corresponding body is executed.

### §3.2 Match (default)

```text
eval(a, η) = a'
resolve_value(a', η) = Tuple { elems: [Atom(ConstructorName(n)), ...] }
no arm matches n
default = Some(t)
-----------------------------------
⟨Match(a, arms, default), η, χ⟩ ⇓ ⟨t, η, χ⟩
```

If no arm matches and a default is provided, the default body is executed.

### §3.3 Match (no match, no default)

```text
eval(a, η) = a'
resolve_value(a', η) = Tuple { elems: [Atom(ConstructorName(n)), ...] }
no arm matches n
default = None
-----------------------------------
⟨Match(a, arms, default), η, χ⟩ ⇓ Stuck(MatchError(n))
```

If no arm matches and no default is provided, the computation is stuck.

### §3.4 Match (non-tuple scrutinee)

```text
eval(a, η) = a'
resolve_value(a', η) ≠ Tuple { ... }
-----------------------------------
⟨Match(a, arms, default), η, χ⟩ ⇓ Stuck(MatchError("scrutinee is not a tuple"))
```

### §3.5 Match (empty tuple)

```text
eval(a, η) = a'
resolve_value(a', η) = Tuple { elems: [] }
-----------------------------------
⟨Match(a, arms, default), η, χ⟩ ⇓ Stuck(MatchError("empty tuple"))
```

## §4 Mutual Recursion via Tuple-of-Lambdas

### §4.1 Pattern

Mutual recursion is desugared to single `LetRec` with a tuple of lambdas:

```text
letrec even = (lam [n] k ... (call odd [n-1] k) ...)
      odd  = (lam [n] k ... (call even [n-1] k) ...)
in ...

-- desugars to:

letrec pair = Tuple {
  elems: [
    Lam { ...; body: ... (call (tuple_get 1 pair) [n-1] k) ... },
    Lam { ...; body: ... (call (tuple_get 0 pair) [n-1] k) ... }
  ]
}
in ...
```

### §4.2 Semantics

The `rec_binding` mechanism on `Value::Lam` enables scoped mutual recursion:

```text
v = Lam { params: [...], cont: k, body: t, captured_env: η, rec_binding: Some(x), ... }
-----------------------------------
⟨Call(Var(x_f), args, κ), η_call, χ⟩ ⇓ ⟨t, η'' , χ⟩

where:
  lookup(Var(x_f), η_call) = v
  args_values = [eval_atom_to_value(a, η_call) for a in args]
  κ_value = lookup(κ, η_call)
  η' = captured_env ∪ {x ↦ lookup(x, η_call)}  // overlay rec_binding
  η'' = η'[params ↦ args_values][k ↦ κ_value]
```

When a lambda with `rec_binding: Some(x)` is called, the call-site environment's binding for `x` is overlaid into the lambda's execution environment. This allows the lambda body to access the recursive tuple through `Var(x)` without polluting the closure's captured environment.

### §4.3 LetRec with rec_binding

```text
η' = η[x ↦ Null]
v' = eval(v, η')
η'' = η'[x ↦ v']
-----------------------------------
⟨LetRec(x, v, t), η, χ⟩ ⇓ ⟨t, η'', χ⟩
```

For `Value::Lam` values inside `v`, `eval` sets `rec_binding: Some(x)` so that when the lambda is called, the recursive binding `x` is available in the execution environment. For `Value::Record` and `Value::Tuple` values, `eval` recursively marks all nested lambdas with `rec_binding: Some(x)`.

### §4.4 Dynamic Scope Prevention

The `rec_binding` overlay is narrowly scoped:

- Only the binding named in `rec_binding` is overlaid
- No other call-site bindings leak into the lambda body
- Non-recursive lambdas (`rec_binding: None`) receive no overlay

This ensures that accidental free variables in a lambda do not resolve from the caller's environment.

## §5 Primitive Argument Resolution (updated)

### §5.1 eval_atom_to_value

```text
eval_atom_to_value(a, η) =
  case a of
    Var(x) => lookup(x, η)        // may return structured Value
    Atom(a') => Value::Atom(a')   // literal atom
```

Primitive arguments are resolved to `Value` (not just `Atom`) so that structured values can be passed to primitives.

### §5.2 eval_prim (updated signature)

```text
eval_prim(⊙, [v₁, ..., vₙ], η) = v'
```

Primitive operations now take `Value` arguments and return `Value`. This allows `RecordGet` and `TupleGet` to operate on structured values.

## §6 Worked Example: Even/Odd Mutual Recursion

```text
letcont exit [v] (trap return) in
letrec pair = Tuple {
  elems: [
    Lam { params: [n], cont: k,
      body: (
        letprim is_zero = eq n 0 in
        if is_zero then
          (jump k true)
        else
          letprim n_minus_1 = sub n 1 in
          letprim odd_fn = tuple_get 1 pair in
          (call odd_fn [n_minus_1] k)
      )
    },
    Lam { params: [n], cont: k,
      body: (
        letprim is_zero = eq n 0 in
        if is_zero then
          (jump k false)
        else
          letprim n_minus_1 = sub n 1 in
          letprim even_fn = tuple_get 0 pair in
          (call even_fn [n_minus_1] k)
      )
    }
  ]
}
in
letprim even_fn = tuple_get 0 pair in
(call even_fn [4] exit)
```

**Execution trace:**

1. `exit` bound as continuation with body `trap return`
2. `pair` bound via `LetRec` (placeholder → backfill with tuple)
3. `LetRec` automatically marks all nested lambdas with `rec_binding: Some("pair")`
4. `even_fn` extracted from tuple (index 0)
5. `Call even_fn [4] exit`: `n=4`, `k=exit`
6. `is_zero = eq 4 0 = false`
7. `If` takes else branch
8. `n_minus_1 = sub 4 1 = 3`
9. `odd_fn = tuple_get 1 pair` → extracts odd lambda
10. `Call odd_fn [3] k`: `n=3`, `k=exit`
11. `rec_binding` overlays `pair` from call-site into lambda env
12. Odd lambda body can access `pair` via `Var("pair")`
13. Recursion continues: even→odd→even→odd→even (base case)
14. Base case (`n=0`): `jump k true` where `k=exit`
15. `exit` receives `true` (4 is even)

## §7 See Also

- [SPEC-099b: Base Operational Semantics](SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md) — Frozen Phase 159 baseline
- [SPEC-098b: Target IR](SPEC-098b-TARGET-IR.md) — CPS IR syntax and types
- [PLAN-160: CPS IR Runtime Expansion](../plan/PLAN-160-CPS-IR-RUNTIME-EXPANSION.md) — Implementation plan

## §8 Changelog

- 2026-06-19: Created as extension to SPEC-099b documenting Phase 160 structured data, pattern matching, and mutual recursion semantics.
