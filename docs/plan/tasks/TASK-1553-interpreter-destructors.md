# TASK-1553: Interpreter Destructors

## Status: 📝 Planned

## Description

Evaluate `let` destructuring in the interpreter. Bind variables to fields/elements of the destructured value.

## Specification Reference

- [SPEC-091: Let Destructors](../../spec/SPEC-091-LET-DESTRUCTORS.md)
- [PLAN-155: Let Destructors](../PLAN-155-LET-DESTRUCTORS.md)
- [TASK-1552](TASK-1552-typecheck-destructors.md) — Typechecker dependency

## Acceptance Criteria

- [ ] Record destructor: bind each variable to the corresponding field value
- [ ] Tuple destructor: bind each variable to the corresponding element
- [ ] Partial destructor: only bind the requested fields/elements
- [ ] Order-independent for records: `{a, b}` and `{b, a}` produce same bindings
- [ ] Order-dependent for tuples: `(a, b)` and `(b, a)` produce different bindings
- [ ] Variables are available in subsequent statements

## Verification

- `cargo test -p ash-interp` passes
- New interpreter tests for all destructor forms pass
- End-to-end tests verify correct variable binding
