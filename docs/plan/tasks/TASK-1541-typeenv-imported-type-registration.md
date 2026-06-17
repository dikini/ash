# TASK-1541: TypeEnv Imported Type Registration

## Status: 📝 Planned

## Description

Modify TypeEnv to register imported types before local types are processed. This ensures imported types are available when typechecking local type definitions.

## Specification Reference

- [SPEC-090: Type Annotation Quirks](../../spec/SPEC-090-TYPE-ANNOTATION-QUIRKS.md)
- [PLAN-154: Type Annotation Quirks](../PLAN-154-TYPE-ANNOTATION-QUIRKS.md)
- [TASK-1540](TASK-1540-parser-import-first-pass.md) — Parser dependency

## Acceptance Criteria

- [ ] TypeEnv has API to register imported types early
- [ ] Imported types are available during local type definition processing
- [ ] Type unification works with imported types
- [ ] No regressions in existing type tests

## Verification

- `cargo test -p ash-typeck` passes
- New TypeEnv tests for imported type registration pass
