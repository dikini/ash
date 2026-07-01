# TASK-1542: Type Name Resolution with Imported Types

## Status: ✅ Complete

## Description

Update type name resolution to check imported types when resolving type names in annotations, definitions, and expressions.

## Specification Reference

- [SPEC-090: Type Annotation Quirks](../../spec/SPEC-090-TYPE-ANNOTATION-QUIRKS.md)
- [PLAN-154: Type Annotation Quirks](../PLAN-154-TYPE-ANNOTATION-QUIRKS.md)
- [TASK-1541](TASK-1541-typeenv-imported-type-registration.md) — TypeEnv dependency

## Acceptance Criteria

- [x] Type name resolution checks imported types
- [x] `fn` parameter types can use imported types
- [x] `fn` return types can use imported types
- [x] Record field types can use imported types
- [x] No regressions in existing resolution tests

## Verification

- `cargo test -p ash-typeck` passes
- New resolution tests for imported types pass


## Completion Evidence

- Resolution now treats imported public types and imported callable signature types as known in public type definitions, record fields, fn parameters, and fn returns.
- Primary regression coverage: `cargo test -p ash-engine --test task_1540_type_annotation_quirks -- --nocapture`.
