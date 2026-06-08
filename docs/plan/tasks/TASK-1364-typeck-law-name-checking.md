# TASK-1364: Typechecker — verify law proposition names exist

## Status: 📝 Planned

## Description

Typechecker verifies that all names referenced in a law proposition exist and are well-typed.

## Requirements

1. Add `register_interface_laws` to `TypeEnv`
2. Add `register_module_laws` to `TypeEnv`
3. Check that all identifiers in proposition expression resolve
4. Typecheck the proposition expression

## Acceptance Criteria

- [ ] Unknown names in law proposition produce error
- [ ] Well-formed law propositions pass
- [ ] Typechecker test passes
- [ ] No regressions

## Related

- [PLAN-136](../PLAN-136-INTERFACE-LAW-SYNTAX.md)
- [TASK-1360](TASK-1360-parser-law-in-interfaces.md)
- [TASK-1361](TASK-1361-parser-law-module-scope.md)
