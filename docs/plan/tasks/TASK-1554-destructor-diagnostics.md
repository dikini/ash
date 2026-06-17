# TASK-1554: Destructor Diagnostics

## Status: 📝 Planned

## Description

Add comprehensive error messages for all `let` destructor failure modes. Ensure errors are informative, helpful, and guide users to correct code.

## Specification Reference

- [SPEC-091: Let Destructors](../../spec/SPEC-091-LET-DESTRUCTORS.md)
- [PLAN-155: Let Destructors](../PLAN-155-LET-DESTRUCTORS.md)
- [TASK-1552](TASK-1552-typecheck-destructors.md) — Typechecker dependency

## Acceptance Criteria

- [ ] Field not found error with suggestion
- [ ] Duplicate field error
- [ ] Tuple length mismatch error
- [ ] Sum type (variant) destructuring error
- [ ] Wrong pattern type error
- [ ] All errors include source location
- [ ] All errors suggest the fix

## Error Message Examples

```
Error: Record type Strategy<T> has no field 'generator'
  --> std/src/test/quickcheck/combinator.ash:16:9
   |
16 |     let { generator, shrink } = strategy;
   |         ^^^^^^^^^^^
   |
   = help: Did you mean 'gen'? The fields are: gen, shrink, name

Error: Duplicate field 'gen' in let destructor
  --> std/src/test/quickcheck/combinator.ash:16:9
   |
16 |     let { gen, gen } = strategy;
   |         ^^^  ^^^
   |
   = help: Remove the duplicate field or rename one of them

Error: Result<T, E> is a sum type (variant). Use 'match' for variant destructuring.
  --> example.ash:10:5
   |
10 |     let { value } = result;
   |         ^^^^^^^^^
   |
   = help: Try: match result { Ok { value: v } => v, Err { error: e } => panic(e) }
```

## Verification

- `cargo test -p ash-typeck` passes (diagnostic tests)
- New diagnostic tests verify all error messages
- Error messages reviewed for clarity and helpfulness
