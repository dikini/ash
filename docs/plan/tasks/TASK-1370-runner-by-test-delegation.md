# TASK-1370: Synthetic tests — `by test` delegation syntax

## Status: 📝 Planned

## Description

Support `proof ... by test { ... }` syntax for explicit synthetic test delegation.

## Requirements

1. Parse `by test` proof body
2. Extract test configuration (generator, equivalence, bounds)
3. Delegate to synthetic test runner
4. Cache results in `.ash/law-cache.toml`

## Acceptance Criteria

- [ ] `by test` parses correctly
- [ ] Configuration extracted and passed to runner
- [ ] Results cached
- [ ] Test passes
- [ ] No regressions

## Related

- [PLAN-136](../PLAN-136-INTERFACE-LAW-SYNTAX.md)
- [TASK-1369](TASK-1369-runner-synthetic-test-generation.md)
