# TASK-1543: Type Inference Leakage Diagnostics

## Status: 📝 Planned

## Description

Add diagnostics for when type inference produces a type not in the current scope. This prevents accidental type leakage and guides users to add the correct imports.

## Specification Reference

- [SPEC-090: Type Annotation Quirks](../../spec/SPEC-090-TYPE-ANNOTATION-QUIRKS.md)
- [PLAN-154: Type Annotation Quirks](../PLAN-154-TYPE-ANNOTATION-QUIRKS.md)
- [TASK-1542](TASK-1542-type-name-resolution-imported.md) — Resolution dependency

## Acceptance Criteria

- [ ] Clear error message when inferred type is not imported
- [ ] Error includes module path of the type
- [ ] Error suggests the import statement needed
- [ ] No false positives for local types

## Verification

- `cargo test -p ash-typeck` passes
- New diagnostic tests for type leakage pass
