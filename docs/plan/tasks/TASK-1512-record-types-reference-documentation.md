# TASK-1512: Add Reference Documentation for Record Types

## Status: ✅ Complete

## Description

Create comprehensive reference documentation for Ash record types at `reference/language/types/records.md`. The documentation should explain:

1. What a record type is in Ash
2. How to define record types (using `type` keyword)
3. How to construct record values
4. How to access record fields
5. How to destructure records in `let` bindings
6. How records relate to the `Strategy<T>` example

## Background

During Phase 151 implementation, we discovered that the terminology around "record types" is confusing. The `Strategy<T>` type is defined as:

```ash
pub type Strategy<T> = Strategy {
    gen: (GenContext) -> T,
    shrink: (T) -> List<T>,
};
```

This is a **record type** — a product type with named fields. In Ash, record types are defined using the `type` keyword with a record body (`{ field: Type, ... }`).

The generated Rust code uses `TypeBody::Struct` internally, which adds to the confusion. The user-facing term should be "record type" consistently.

## Goals

1. ✅ Create `reference/language/types/records.md` with:
   - Definition of record types
   - Syntax for defining records
   - Syntax for constructing records
   - Syntax for field access
   - Syntax for destructuring
   - Examples including `Strategy<T>`
   - Comparison with tuple types
   - Comparison with variant types (enums)

2. ✅ Update terminology in existing docs to use "record type" consistently
   - `docs/spec/SPEC-020-ADT-TYPES.md` — added terminology note clarifying "struct" vs "record"

3. ✅ Add cross-references from:
   - `reference/INDEX.md` — added "Record types" and "Tuple types" links
   - `docs/spec/SPEC-020-ADT-TYPES.md` — added link to records.md

## Acceptance Criteria

- ✅ `reference/language/types/records.md` exists and is complete
- ✅ The document uses "record type" consistently (not "struct")
- ✅ Examples include `Strategy<T>` as a real-world record type
- ✅ All syntax examples are verified against the parser (10/10 examples pass)
- ✅ Cross-references are added from relevant docs
- ✅ The document is linked from `reference/INDEX.md`

## Verification Evidence

### Syntax Examples Verified

All 10 syntax examples from the documentation were tested against the parser:

1. ✅ Record type definition (`Point { x: Int, y: Int }`)
2. ✅ Generic record (`Box<T> { value: T }`)
3. ✅ Record with function fields (`Strategy<T>`)
4. ✅ Record construction (`Point { x: 0, y: 0 }`)
5. ✅ Generic construction (`Box { value: 42 }`)
6. ✅ Construction with function fields (`Strategy { gen: fn(_ctx) { 5 } }`)
7. ✅ Field access (`p.x`)
8. ✅ Field access with function fields (`s.gen(ctx)`)
9. ✅ Destructuring with explicit field:pattern pairs (`let { x: x_val } = p`)
10. ✅ Destructuring with rename (`let { x: a } = p`)

### Cross-References Added

- `reference/INDEX.md` — Added "Record types" and "Tuple types" under Language pilot
- `docs/spec/SPEC-020-ADT-TYPES.md` — Added terminology note with link to records.md

## Dependencies

- None — this is a documentation-only task

## Notes

The confusion about "struct" vs "record" stems from the Rust implementation using `TypeBody::Struct` for what Ash calls record types. The user-facing documentation should always use "record type" to avoid confusion with Rust structs.

The `Strategy<T>` type is a perfect example because:
- It has named fields (`gen`, `shrink`)
- It is generic (`<T>`)
- It contains function types as fields
- It is used extensively in the QuickCheck library
- It demonstrates both construction and field access patterns

## Future Work

- TASK-1527: Update `reference/language/types/records.md` with closure field examples and capture rules
- TASK-1556: Update `reference/language/types/records.md` with destructor examples
