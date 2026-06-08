# TASK-1374: Integration — module-scoped law in `std::io::path`

## Status: 📝 Planned

## Description

Add module-scoped `law` to `std::io::path` and verify.

## Requirements

1. Add `join_preserves_absolute` law to `std/src/io/path.ash`
2. Verify parser accepts
3. Verify typechecker passes
4. Verify synthetic tests generate

## Acceptance Criteria

- [ ] Module law added to `std::io::path`
- [ ] Full pipeline works
- [ ] Integration test passes
- [ ] No regressions

## Related

- [PLAN-136](../PLAN-136-INTERFACE-LAW-SYNTAX.md)
- [TASK-1361](TASK-1361-parser-law-module-scope.md)
- [TASK-1364](TASK-1364-typeck-law-name-checking.md)
- [TASK-1369](TASK-1369-runner-synthetic-test-generation.md)
