# TASK-1512: Add Reference Documentation for Record Types

## Status: 📝 Planned

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

1. Create `reference/language/types/records.md` with:
   - Definition of record types
   - Syntax for defining records
   - Syntax for constructing records
   - Syntax for field access
   - Syntax for destructuring
   - Examples including `Strategy<T>`
   - Comparison with tuple types
   - Comparison with variant types (enums)

2. Update terminology in existing docs to use "record type" consistently
   - `docs/spec/SPEC-020-ADT-TYPES.md` — clarify "struct" vs "record"
   - `docs/TUTORIAL.md` — update pattern table
   - `docs/design/ADT_TYPE_SYSTEM.md` — if it exists

3. Add cross-references from:
   - `Strategy<T>` definition in stdlib
   - `docs/plan/tasks/TASK-1511-deferred-combinators-ordinary-ash.md`
   - `docs/reference/type-system-vocabulary-guidance.md`

## Acceptance Criteria

- [ ] `docs/reference/types/records.md` exists and is complete
- [ ] The document uses "record type" consistently (not "struct")
- [ ] Examples include `Strategy<T>` as a real-world record type
- [ ] All syntax examples are verified against the parser
- [ ] Cross-references are added from relevant docs
- [ ] The document is linked from `docs/reference/README.md` or index

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
