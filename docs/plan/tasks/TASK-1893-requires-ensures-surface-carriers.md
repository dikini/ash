# TASK-1893: Requires Ensures Surface Carriers

**Status:** Planned
**Plan:** [PLAN-194](../PLAN-194-CONTRACT-AND-EVIDENCE-SYSTEM.md)

## Description

Parse and preserve target-surface `requires` and `ensures` clauses on ordinary `fn` declarations.

## Requirements

1. Accept `requires { ... }` and `ensures { ... }` on target `fn` declarations.
2. Preserve clause order, source spans, clause kind, predicate expression, and stable local identity.
3. Bind `result` only in `ensures` predicates.
4. Reject contract clauses on unsupported declarations with structured diagnostics.
5. Preserve contract metadata through public summaries and imports.

## TDD Steps

1. RED: add parser tests for `requires`, `ensures`, multiple clauses, and invalid placement.
2. RED: add engine/typechecker tests proving contract metadata is preserved on local and imported
   callables.
3. GREEN: implement surface AST carriers and summary threading.
4. Verify existing function-first parser and engine fixtures still pass.

## Completion Checklist

- [ ] Parser tests cover accepted and rejected contract clause syntax.
- [ ] Surface AST carries contract metadata with spans and identities.
- [ ] Function summaries preserve local and imported contract metadata.
- [ ] Existing row and function-first behavior is unchanged.
