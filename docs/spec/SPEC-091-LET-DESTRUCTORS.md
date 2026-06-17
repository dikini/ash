# SPEC-091: Let Destructors for Records and Tuples

**Status:** Draft
**Date:** 2026-06-17
**Amends:** [SPEC-072](SPEC-072-TOWER-CALLABLE-TYPE-AND-CLOSURE-SYNTAX.md)
**Builds on:** [ASSESSMENT-002](../assessments/ASSESSMENT-002-TYPE-ANNOTATION-QUIRKS.md)
**Plan:** [PLAN-155](../plan/PLAN-155-LET-DESTRUCTORS.md)

## 1. Summary

Add `let` destructor syntax for record and tuple types. This is **group assignment** — not pattern matching — providing a convenient way to bind multiple variables from a structured value.

## 2. Motivation

Currently, extracting multiple fields from a record requires verbose field access:

```ash
let gen = strategy.gen;
let shrink = strategy.shrink;
```

With `let` destructors:

```ash
let { gen, shrink } = strategy;
```

This is purely syntactic sugar for sequential field access, but it improves readability and reduces boilerplate.

## 3. Core Design

### 3.1 Record Destructors

```ash
let { gen, shrink } = strategy;
```

**Semantics:** Bind each variable to the corresponding field of the record. The variable names must match the field names exactly.

**Equivalent to:**
```ash
let gen = strategy.gen;
let shrink = strategy.shrink;
```

### 3.2 Tuple Destructors

```ash
let (a, b) = pair;
```

**Semantics:** Bind variables by position. The first variable binds to the first element, etc.

**Equivalent to:**
```ash
let a = pair.0;
let b = pair.1;
```

### 3.3 Explicit Renaming

For records, you can rename fields:

```ash
let { gen: g, shrink: s } = strategy;
```

**Equivalent to:**
```ash
let g = strategy.gen;
let s = strategy.shrink;
```

For tuples, renaming is not applicable (position-based).

### 3.4 Partial Matching

**Partial matching is allowed** — omit fields you don't need:

```ash
type Strategy<T> = Strategy {
    gen: (GenContext) -> T,
    shrink: (T) -> List<T>,
    name: String,
};

let { gen, shrink } = strategy;  -- ✅ Valid: name is ignored
```

**No ellision syntax needed** — simply omit the fields you don't need.

### 3.5 Order Semantics

**Records: order does not matter**
```ash
let { shrink, gen } = strategy;  -- ✅ Same as { gen, shrink }
```

**Tuples: order matters**
```ash
let (a, b) = pair;  -- a is first, b is second
let (b, a) = pair;  -- b is first, a is second (different!)
```

This distinction comes from the fundamental semantics of records (unordered, label-based) vs tuples (ordered, position-based).

## 4. Error Conditions

### 4.1 Field Not Found

```ash
let { generator, shrink } = strategy;
-- Error: Record type Strategy<T> has no field 'generator'. Did you mean 'gen'?
```

The error must include:
- The record type name
- The offending field name
- A suggestion if a similar field exists

### 4.2 Duplicate Field

```ash
let { gen, gen } = strategy;
-- Error: Duplicate field 'gen' in let destructor
```

### 4.3 Wrong Pattern for Type

```ash
let { a, b } = 42;
-- Error: Type Int is not a record. Cannot use { ... } pattern.
```

```ash
let (a, b) = strategy;
-- Error: Type Strategy<T> is not a tuple. Cannot use ( ... ) pattern.
```

### 4.4 Sum Type (Variant) Destructuring

```ash
type Result<T, E> = Ok { value: T } | Err { error: E };

let { value } = result;
-- Error: Result<T, E> is a sum type (variant). Use 'match' for variant destructuring.
```

**Rule:** `let` destructuring works only for **product types** (records, tuples). For **sum types** (variants), use `match`.

### 4.5 Tuple Length Mismatch

```ash
let (a, b, c) = pair;  -- pair is a 2-tuple
-- Error: Tuple of length 2 cannot be destructured into 3 variables
```

## 5. No Silent Defaults

If a field is omitted from the pattern, it is simply not bound. There are no implicit defaults, no automatic ignoring, no silent failures.

```ash
let { gen } = strategy;  -- Only binds gen

let x = shrink;  -- ❌ Error: unbound variable 'shrink'
```

## 6. Relationship to Pattern Matching

`let` destructuring is **not** pattern matching. It is group assignment. The key differences:

| Feature | `let` Destructuring | `match` Pattern Matching |
|---------|---------------------|--------------------------|
| Purpose | Group assignment | Case analysis |
| Works on | Product types (records, tuples) | Sum types (variants) |
| Exhaustiveness | Not required (partial OK) | Required |
| Conditions | No guards | Can have guards |
| Deep patterns | No | Yes |

## 7. Acceptance Criteria

### C91-1: Record destructor

```ash
type Point = Point { x: Int, y: Int };
let p = Point { x: 1, y: 2 };
let { x, y } = p;
assert x == 1;
assert y == 2;
```

### C91-2: Tuple destructor

```ash
let pair = (1, 2);
let (a, b) = pair;
assert a == 1;
assert b == 2;
```

### C91-3: Partial record destructor

```ash
type Strategy<T> = Strategy { gen: (GenContext) -> T, shrink: (T) -> List<T>, name: String };
let s = Strategy { gen: fn(ctx) { 42 }, shrink: fn(x) { [] }, name: "constant" };
let { gen, shrink } = s;  -- name is ignored
```

### C91-4: Explicit renaming

```ash
let { gen: g, shrink: s } = strategy;
-- g is strategy.gen, s is strategy.shrink
```

### C91-5: Order independence for records

```ash
let { y, x } = p;  -- Same as { x, y }
assert x == 1;
assert y == 2;
```

### C91-6: Order dependence for tuples

```ash
let (b, a) = (1, 2);
assert b == 1;  -- First element
assert a == 2;  -- Second element
```

### C91-7: Error on variant (sum type)

```ash
type Result<T, E> = Ok { value: T } | Err { error: E };
let r = Ok { value: 42 };
let { value } = r;  -- Must error: sum type, use match
```

### C91-8: Error on field not found

```ash
let { generator } = strategy;  -- Must error: no field 'generator'
```

### C91-9: Error on duplicate field

```ash
let { gen, gen } = strategy;  -- Must error: duplicate field
```

### C91-10: Error on wrong pattern type

```ash
let { a } = 42;  -- Must error: Int is not a record
let (a) = "hello";  -- Must error: String is not a tuple
```

## 8. Implementation Notes

### Files to Modify

| File | Change |
|------|--------|
| `crates/ash-parser/src/parse_expr.rs` | Parse `let { ... } = ...` and `let ( ... ) = ...` |
| `crates/ash-parser/src/ast.rs` | Add `LetDestructure` variant to `Expr` or `Stmt` |
| `crates/ash-typeck/src/check.rs` | Typecheck destructuring: verify fields exist, types match |
| `crates/ash-typeck/src/diagnostics.rs` | Add error messages for destructor failures |
| `crates/ash-interp/src/eval.rs` | Evaluate destructuring: bind variables to fields |

### Lowering

`let { gen, shrink } = strategy` lowers to:

```rust
// In the AST
Stmt::Let {
    bindings: vec![
        ("gen", Expr::FieldAccess { object: "strategy", field: "gen" }),
        ("shrink", Expr::FieldAccess { object: "strategy", field: "shrink" }),
    ]
}
```

Or equivalently, multiple `Stmt::Let` statements.

## 9. Documentation Tasks

| Task | Description |
|------|-------------|
| TASK-1555 | Update `reference/language/functions/local-and-anonymous.md` with `let` destructor syntax |
| TASK-1556 | Update `reference/language/types/records.md` with destructor examples |
| TASK-1557 | Update `reference/language/types/tuples.md` with destructor examples |
| TASK-1558 | Add destructor examples to cookbook |

## 10. Closeout Criteria

- [ ] C91-1 through C91-10 all pass
- [ ] No regressions in existing tests
- [ ] Documentation updated
- [ ] PLAN-155 and PLAN-INDEX updated
- [ ] CHANGELOG.md records the feature
