# Sum Type Constructors in CPS IR

## Overview

Sum type constructors in the CPS IR are represented using `Atom::ConstructorName` tags inside tuples. The tag is the first element; subsequent elements are the constructor's fields.

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
- [SPEC-098b: Target IR](../../spec/SPEC-098b-TARGET-IR.md) — IR grammar
- [SPEC-099c: Expanded Operational Semantics](../../spec/SPEC-099c-CPS-IR-EXPANDED-OPERATIONAL-SEMANTICS.md) — §3
