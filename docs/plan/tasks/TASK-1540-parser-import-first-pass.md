# TASK-1540: Parser Import First Pass

## Status: 📝 Planned

## Description

Modify the parser to collect and resolve `use` statements before processing `type` definitions. This enables imported types to be referenced in local type definitions.

## Specification Reference

- [SPEC-090: Type Annotation Quirks](../../spec/SPEC-090-TYPE-ANNOTATION-QUIRKS.md)
- [PLAN-154: Type Annotation Quirks](../PLAN-154-TYPE-ANNOTATION-QUIRKS.md)

## Acceptance Criteria

- [ ] Parser collects all `use` statements in a module first
- [ ] Imported types are registered in a preliminary scope
- [ ] Local `type` definitions can reference imported types
- [ ] No regressions in existing parsing tests

## Verification

- `cargo test -p ash-parser` passes
- New parser tests for import-first pass pass
