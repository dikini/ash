# TASK-1368: Synthetic tests — extract law nodes from AST

## Status: 📝 Planned

## Description

Test runner can iterate over `law` declarations in parsed modules.

## Requirements

1. Add `extract_laws` function to test runner
2. Return structured law metadata (name, params, proposition)
3. Handle both interface laws and module laws

## Acceptance Criteria

- [ ] Laws extracted from interface definitions
- [ ] Laws extracted from module files
- [ ] Test passes
- [ ] No regressions

## Related

- [PLAN-136](../PLAN-136-INTERFACE-LAW-SYNTAX.md)
- [TASK-1360](TASK-1360-parser-law-in-interfaces.md)
- [TASK-1361](TASK-1361-parser-law-module-scope.md)
