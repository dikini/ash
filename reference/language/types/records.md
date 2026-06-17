---
id: ref.language.types.records
title: Record Types
kind: reference
audience: [human, agent]
authority: canonical-adjacent
status: current
stability: alpha
owner: language
last_verified: 2026-06-17
verified_against:
  git_commit: ce446c96
  specs:
    - docs/spec/SPEC-020-ADT-TYPES.md
  code:
    - crates/ash-parser/src/parse_pattern.rs
    - crates/ash-parser/src/parse_expr.rs
    - crates/ash-parser/src/parse_module/fn_defs.rs
  tests:
    - crates/ash-parser/tests/let_destructor_tests.rs
  examples:
    - std/src/test/quickcheck/strategy.ash
    - std/src/test/quickcheck/combinator.ash
---

# Record Types

A **record type** is a product type with named fields. Each field has a name and a type. Records are the primary way to group related data in Ash.

## Definition

Record types are defined with the `type` keyword:

```ash
type Point = Point {
    x: Int,
    y: Int,
};
```

The type name (`Point`) and the constructor name (`Point`) are the same. This is conventional in Ash.

### Generic Records

Records can have type parameters:

```ash
type Box<T> = Box {
    value: T,
};
```

### Records with Function Fields

Record fields can have function types. This is how `Strategy<T>` is defined:

```ash
pub type Strategy<T> = Strategy {
    gen: (GenContext) -> T,
    shrink: (T) -> List<T>,
};
```

Here, `gen` and `shrink` are fields that hold functions. The `gen` field takes a `GenContext` and returns a `T`. The `shrink` field takes a `T` and returns a list of smaller `T` values.

## Construction

Create a record value by providing all fields:

```ash
let origin = Point { x: 0, y: 0 };
```

For generic records, the type parameter is inferred:

```ash
let int_box = Box { value: 42 };
-- int_box has type Box<Int>
```

### Constructing Records with Function Fields

When a record contains function fields, use `fn` expressions:

```ash
let always_five = Strategy {
    gen: fn(_ctx) { 5 },
    shrink: fn(_n) { [] },
};
```

Note: `fn` expressions in record constructors must use block syntax (`fn(params) { body }`), not arrow syntax (`fn(params) => expr`).

## Field Access

Access record fields with dot notation:

```ash
let p = Point { x: 10, y: 20 };
let x_val = p.x;  -- 10
let y_val = p.y;  -- 20
```

### Accessing Function Fields

When a field contains a function, access it and then call it:

```ash
let s = Strategy {
    gen: fn(_ctx) { 42 },
    shrink: fn(_n) { [] },
};

let ctx = GenContext { size: 10, seed: 0 };
let value = s.gen(ctx);  -- 42
```

The expression `s.gen` accesses the function, and `(ctx)` calls it. This is `FnApply` in the AST — function application on an arbitrary expression.

## Destructuring

Records can be destructured in `let` bindings within function bodies:

### With Explicit Field:Pattern Pairs

```ash
fn sum_point(p: Point) -> Int {
    let { x: x_val, y: y_val } = p;
    x_val + y_val
}
```

### With Rename

```ash
fn sum_point(p: Point) -> Int {
    let { x: a, y: b } = p;
    a + b
}
```

### Shorthand (Not Yet Supported)

The shorthand syntax `let { x, y } = p` (which would desugar to `let { x: x, y: y } = p`) is **not yet supported**. The parser requires explicit `field: pattern` pairs.

### Where Destructuring Works

Record destructuring works in:
- ✅ `fn` body blocks: `fn foo() { let { x: a } = p; ... }`
- ❌ Workflow `observe` blocks: `observe test { let { x: a } = p; ... }` — not supported (only `let name = value;`)
- ❌ `act` blocks: same limitation

## Comparison with Other Types

### Records vs Tuples

| Feature | Record | Tuple |
|---------|--------|-------|
| Fields | Named (`x`, `y`) | Positional (0, 1) |
| Access | `p.x` | `t.0` (not supported) |
| Pattern | `{ x: a, y: b }` | `(a, b)` |
| Readability | Self-documenting | Requires context |

Use records when field names carry meaning. Use tuples for generic pairs or when names don't add value.

### Records vs Variants (Enums)

| Feature | Record | Variant |
|---------|--------|---------|
| Shape | Single shape | Multiple alternatives |
| Example | `Point { x, y }` | `Option<T> = Some { value: T } \| None` |
| Pattern | `{ x: a }` | `Some { value: x }` |

Records are for "this AND that". Variants are for "this OR that".

## Real-World Example: Strategy<T>

The `Strategy<T>` type from `test::quickcheck` demonstrates all record features:

```ash
pub type Strategy<T> = Strategy {
    gen: (GenContext) -> T,
    shrink: (T) -> List<T>,
};
```

### Using Strategy<T>

```ash
pub fn map<A, B>(s: Strategy<A>, f: (A) -> B) -> Strategy<B> {
    Strategy {
        gen: fn(ctx) { f(s.gen(ctx)) },
        shrink: fn(_b) { [] },
    }
}
```

This function:
1. Takes a `Strategy<A>` and a function `f: (A) -> B`
2. Returns a new `Strategy<B>`
3. The new `gen` field calls `s.gen(ctx)` to get an `A`, then applies `f` to get a `B`
4. The new `shrink` field returns an empty list (no shrinking for mapped values)

### Field Access in Combinators

```ash
pub fn with_shrink<T>(s: Strategy<T>, shrink: (T) -> List<T>) -> Strategy<T> {
    Strategy {
        gen: fn(ctx) { s.gen(ctx) },
        shrink: fn(t) { shrink(t) },
    }
}
```

Here, `s.gen(ctx)` accesses the `gen` field of `s` (which is a function) and calls it with `ctx`.

## Known Limitations

1. **Shorthand destructuring**: `let { x, y } = p` is not supported. Use `let { x: x, y: y } = p`.
2. **Workflow block destructuring**: `let` in `observe` and `act` blocks only supports `let name = value;`.
3. **Arrow syntax in fn expressions**: `fn(x) => expr` is not supported. Use `fn(x) { expr }`.

## See Also

- [Functions and Pure Code](../functions.md) — for `fn` expressions
- [SPEC-020: Algebraic Data Types](../../docs/spec/SPEC-020-ADT-TYPES.md) — formal specification
- [Pattern Matching](../functions/patterns.md) — for record patterns in match expressions
