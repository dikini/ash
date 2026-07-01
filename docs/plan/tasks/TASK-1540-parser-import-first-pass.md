# TASK-1540: Parser Import First Pass

## Status: ✅ Complete

## Description

Modify the parser to collect and resolve `use` statements before processing `type` definitions. This enables imported types to be referenced in local type definitions.

## Specification Reference

- [SPEC-090: Type Annotation Quirks](../../spec/SPEC-090-TYPE-ANNOTATION-QUIRKS.md)
- [PLAN-154: Type Annotation Quirks](../PLAN-154-TYPE-ANNOTATION-QUIRKS.md)

## Acceptance Criteria

- [x] Parser collects all `use` statements in a module first
- [x] Imported types are registered in a preliminary scope
- [x] Local `type` definitions can reference imported types
- [x] No regressions in existing parsing tests

## Verification

- `cargo test -p ash-parser` passes
- New parser tests for import-first pass pass


## Completion Evidence

- Engine/module-loader import scanning now parses `use` statements before local module type validation and seeds imported type visibility from resolved module metadata.
- Primary regression coverage: `cargo test -p ash-engine --test task_1540_type_annotation_quirks -- --nocapture`.
