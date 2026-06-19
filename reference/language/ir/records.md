# Records in CPS IR

## Overview

Records in the CPS IR are ordered collections of named fields, where each field holds a `Value`. They are the runtime representation of Ash record types.

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

- [SPEC-098b: Target IR](../../spec/SPEC-098b-TARGET-IR.md) — IR grammar
- [SPEC-099c: Expanded Operational Semantics](../../spec/SPEC-099c-CPS-IR-EXPANDED-OPERATIONAL-SEMANTICS.md) — §2.1, §2.3
