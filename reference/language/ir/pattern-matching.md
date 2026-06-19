# Pattern Matching in CPS IR

## Overview

Pattern matching in the CPS IR is implemented via `Term::Match`, which dispatches on constructor tags in the first element of a tuple. It is the runtime mechanism for sum type elimination.

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
- [SPEC-098b: Target IR](../../spec/SPEC-098b-TARGET-IR.md) — IR grammar
- [SPEC-099c: Expanded Operational Semantics](../../spec/SPEC-099c-CPS-IR-EXPANDED-OPERATIONAL-SEMANTICS.md) — §3
