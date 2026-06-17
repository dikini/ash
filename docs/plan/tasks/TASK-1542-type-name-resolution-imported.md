# TASK-1542: Type Name Resolution with Imported Types

## Status: 📝 Planned

## Description

Update type name resolution to check imported types when resolving type names in annotations, definitions, and expressions.

## Specification Reference

- [SPEC-090: Type Annotation Quirks](../../spec/SPEC-090-TYPE-ANNOTATION-QUIRKS.md)
- [PLAN-154: Type Annotation Quirks](../PLAN-154-TYPE-ANNOTATION-QUIRKS.md)
- [TASK-1541](TASK-1541-typeenv-imported-type-registration.md) — TypeEnv dependency

## Acceptance Criteria

- [ ] Type name resolution checks imported types
- [ ] `fn` parameter types can use imported types
- [ ] `fn` return types can use imported types
- [ ] Record field types can use imported types
- [ ] No regressions in existing resolution tests

## Verification

- `cargo test -p ash-typeck` passes
- New resolution tests for imported types pass
