# TASK-1371: CLI — `--skip-law-tests` and `--skip-law-test=<name>`

## Status: 📝 Planned

## Description

Add opt-out flags for law testing.

## Requirements

1. Add `--skip-law-tests` CLI flag (skips all law tests)
2. Add `--skip-law-test=<name>` CLI flag (skips specific law by name)
3. Skip law test generation when opted out
4. Document in CLI help

## Acceptance Criteria

- [ ] `--skip-law-tests` skips all law tests
- [ ] `--skip-law-test=<name>` skips specific law
- [ ] CLI test passes
- [ ] No regressions

## Related

- [PLAN-136](../PLAN-136-INTERFACE-LAW-SYNTAX.md)
- [TASK-1369](TASK-1369-runner-synthetic-test-generation.md)
