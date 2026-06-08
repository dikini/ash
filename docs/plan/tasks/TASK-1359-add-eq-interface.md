# TASK-1359: Add `Eq` Interface to `std::algebra`

## Status: 📝 Planned

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

- [ ] `Eq` interface file exists
- [ ] `Eq` is exported from `std::algebra`
- [ ] Parser test passes
- [ ] No regressions

## Related

- [PLAN-136](../PLAN-136-INTERFACE-LAW-SYNTAX.md)
- [DESIGN-NOTE-INTERFACE-LAWS](../../design/DESIGN-NOTE-INTERFACE-LAWS.md)
