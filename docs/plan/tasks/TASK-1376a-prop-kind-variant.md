# TASK-1376a: Add `Kind::Prop` Variant

## Status: 📝 Planned

## Description

Add `Prop` as a distinct kind in the type system.

## Requirements

1. Add `Kind::Prop` variant to `ash_core::Kind`
2. Update kind checking to handle `Prop`
3. Ensure `Prop` is not compatible with `Type`

## Acceptance Criteria

- [ ] `Kind::Prop` exists
- [ ] `Prop` incompatible with `Type`
- [ ] Test passes

## Related

- [TASK-1376](TASK-1376-stage3-prop-kind.md)
