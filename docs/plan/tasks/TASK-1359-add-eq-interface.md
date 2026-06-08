# TASK-1359: Add `Eq` Interface to `std::algebra`

## Status: ✅ Complete

## Description

The law syntax design assumes an `Eq<A>` interface exists for explicit equivalence relations. This task creates the interface in `std::algebra` so that law examples are valid and runnable.

## Requirements

1. Create `std/src/algebra/eq.ash`:
```ash
pub interface Eq<A> {
    equiv(A, A) -> Bool
}
```

2. Add `Eq` to `std/src/algebra/mod.ash` exports (if applicable)

3. Verify parser accepts the syntax

## Acceptance Criteria

- [x] `Eq` interface file exists
- [x] `Eq` is exported from `std::algebra`
- [x] Parser test passes
- [x] No regressions

## Completion Notes

- Added `std/src/algebra/eq.ash` with `Eq<A>.equiv(A, A) -> Bool`.
- Exported `Eq` through the `std::algebra` module surface.
- Verified as part of the Phase 136 parser/stdlib checkpoint before TASK-1360.

## Related

- [PLAN-136](../PLAN-136-INTERFACE-LAW-SYNTAX.md)
- [DESIGN-NOTE-INTERFACE-LAWS](../../design/DESIGN-NOTE-INTERFACE-LAWS.md)
