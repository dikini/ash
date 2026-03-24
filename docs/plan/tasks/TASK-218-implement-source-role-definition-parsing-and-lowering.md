# TASK-218: Implement Source Role Definition Parsing and Lowering

## Status: ✅ Complete

## Description

Implement source role-definition parsing in inline-module contexts plus maintained crate-internal
lowering coverage using the simplified role contract of authority plus named obligations.

## Specification Reference

- SPEC-001: Intermediate Representation (IR)
- SPEC-002: Surface Language

## Requirements

### Functional Requirements

1. Parse `role` definitions in module/inline-definition contexts
2. Preserve named `authority` and named `obligations` in the surface AST
3. Lower parsed role definitions into the current core role representation through the maintained crate-internal `lower_role_definitions()` module helper path exercised by parser regression tests
4. Add focused parser/lowering tests for valid and malformed role definitions

## Files

- Modify: `crates/ash-parser/src/parse_module.rs`
- Modify: `crates/ash-parser/src/lib.rs`
- Modify: `crates/ash-parser/src/surface.rs`
- Modify: `crates/ash-parser/src/lower.rs`
- Test: parser/lowering tests for role definitions
- Modify: `CHANGELOG.md`

## TDD Steps

1. ✅ Write failing parser tests for `role` definitions with authority and obligations
2. ✅ Verify RED with focused `ash-parser` tests
3. ✅ Implement minimal parser and lowering support
4. ✅ Verify GREEN with parser/lowering tests
5. ☐ Commit

## Completion Checklist

- [x] inline module parsing recognizes `Definition::Role`
- [x] named role authorities are preserved in `RoleDef`
- [x] named role obligations are preserved in `RoleDef`
- [x] `lower_role_definitions()` lowers the parsed role shape into the current core role metadata through the maintained crate-internal module helper path
- [x] focused and full `ash-parser` verification passed
- [x] `CHANGELOG.md` updated

## Non-goals

- No authority-enforcement redesign
- No role hierarchy support
- No new general production parser-facing role-lowering API beyond the maintained crate-internal module helper surface used by parser tests
