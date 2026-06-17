# TASK-1527: Update Record Documentation with Closure Fields

## Status: 📝 Planned

## Description

Update `reference/language/types/records.md` with closure field examples and capture rules. Records can contain function fields, and closures in those fields must follow capture rules.

## Specification Reference

- [SPEC-088: Closure Refinement and Effect-Safe Capture](../../spec/SPEC-088-CLOSURE-REFINEMENT-AND-EFFECT-SAFE-CAPTURE.md)
- [TASK-1512](TASK-1512-record-types-reference-documentation.md) — Existing record docs
- [PLAN-152: Closure Refinement and Tower Documentation](../PLAN-152-CLOSURE-REFINEMENT-AND-TOWER-DOCUMENTATION.md)

## Acceptance Criteria

- [ ] Add section on records with function fields
- [ ] Document capture rules for closures stored in records
- [ ] Update `Strategy<T>` example to show refined closure rules
- [ ] Verify all examples work with refined closures
- [ ] Cross-reference to functions.md and tower.md

## Verification

- All examples parse and typecheck correctly
- No regressions in existing record documentation
- Cross-references are accurate
