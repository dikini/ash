# TASK-1552: Typecheck Destructors

## Status: 📝 Planned

## Description

Typecheck `let` destructuring. Verify that fields exist on the record type, types match, and there are no duplicates.

## Specification Reference

- [SPEC-091: Let Destructors](../../spec/SPEC-091-LET-DESTRUCTORS.md)
- [PLAN-155: Let Destructors](../PLAN-155-LET-DESTRUCTORS.md)
- [TASK-1551](TASK-1551-ast-destructure-representation.md) — AST dependency

## Acceptance Criteria

- [ ] Record destructor: all fields must exist on the type
- [ ] Record destructor: no duplicate fields
- [ ] Tuple destructor: length must match tuple arity
- [ ] Type of each bound variable matches the field/element type
- [ ] Error on sum type (variant) destructuring
- [ ] Error on wrong pattern type (record pattern on non-record, etc.)

## Error Messages

| Error | Message |
|-------|---------|
| Field not found | `Record type {type} has no field '{field}'. Did you mean '{suggestion}'?` |
| Duplicate field | `Duplicate field '{field}' in let destructor` |
| Tuple length mismatch | `Tuple of length {expected} cannot be destructured into {actual} variables` |
| Sum type | `{type} is a sum type (variant). Use 'match' for variant destructuring.` |
| Non-record | `Type {type} is not a record. Cannot use {{ ... }} pattern.` |
| Non-tuple | `Type {type} is not a tuple. Cannot use ( ... ) pattern.` |

## Verification

- `cargo test -p ash-typeck` passes
- New typechecker tests for all error conditions pass
- Property tests: random destructuring patterns typecheck correctly
