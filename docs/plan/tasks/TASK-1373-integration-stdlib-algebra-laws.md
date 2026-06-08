# TASK-1373: Integration — end-to-end law syntax in `std::algebra`

## Status: 📝 Planned

## Description

Add `law` declarations to at least one `std::algebra` interface and verify full pipeline works.

## Requirements

1. Add laws to `std/src/algebra/semigroup.ash`
2. Add laws to `std/src/algebra/monoid.ash`
3. Verify parser accepts
4. Verify typechecker passes
5. Verify synthetic tests generate

## Acceptance Criteria

- [ ] `Semigroup` has `associativity` law
- [ ] `Monoid` has `left_identity` and `right_identity` laws
- [ ] Full pipeline: parse → typecheck → test generation
- [ ] Integration test passes
- [ ] No regressions

## Related

- [PLAN-136](../PLAN-136-INTERFACE-LAW-SYNTAX.md)
- [TASK-1360](TASK-1360-parser-law-in-interfaces.md)
- [TASK-1364](TASK-1364-typeck-law-name-checking.md)
- [TASK-1369](TASK-1369-runner-synthetic-test-generation.md)
