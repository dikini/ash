# TASK-1366: Typechecker — restrict law propositions to Pure functions

## Status: 📝 Planned

## Description

Law propositions must reference only `Pure` functions. `Act`/`Proc`/`Workflow` in law body = compile-time error.

## Requirements

1. Track effect level of each function referenced in law proposition
2. Reject if any referenced function has effect > Pure
3. Error message indicates which function violates purity

## Acceptance Criteria

- [ ] Law referencing `Act` function produces error
- [ ] Law referencing only `Pure` functions passes
- [ ] Typechecker test passes
- [ ] No regressions

## Related

- [PLAN-136](../PLAN-136-INTERFACE-LAW-SYNTAX.md)
- [TASK-1364](TASK-1364-typeck-law-name-checking.md)
