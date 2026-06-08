# TASK-1365: Typechecker — verify proof names match declared laws

## Status: 📝 Planned

## Description

Compiler rejects `proof unknown_law(...) { ... }` if no matching law exists.

## Requirements

1. Add `register_module_proofs` to `TypeEnv`
2. Add `register_impl_proofs` to `TypeEnv`
3. Verify proof name matches a declared law in scope
4. Error if no matching law found

## Acceptance Criteria

- [ ] Proof for unknown law produces error
- [ ] Proof for known law passes
- [ ] Typechecker test passes
- [ ] No regressions

## Related

- [PLAN-136](../PLAN-136-INTERFACE-LAW-SYNTAX.md)
- [TASK-1362](TASK-1362-parser-proof-in-impls.md)
- [TASK-1363](TASK-1363-parser-proof-module-scope.md)
- [TASK-1364](TASK-1364-typeck-law-name-checking.md)
