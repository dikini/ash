# TASK-1376c: Runtime Escape Prevention

## Status: 📝 Planned

## Description

Prevent `Prop`-typed values from escaping into runtime code.

## Requirements

1. Reject functions returning `Prop`
2. Reject `Prop` in struct fields
3. Reject `Prop` in enum variants

## Acceptance Criteria

- [ ] `fn foo() -> Prop` rejected
- [ ] `Prop` in struct field rejected
- [ ] Test passes

## Related

- [TASK-1376](TASK-1376-stage3-prop-kind.md)
