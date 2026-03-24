# TASK-217: Remove Legacy Role Supervision Shape

## Status: ✅ Complete

## Description

Remove the legacy `supervises` field and related parser/core scaffolding from canonical-facing role
structures so the implementation shape matches the simplified role contract.

## Specification Reference

- SPEC-001: Intermediate Representation (IR)
- SPEC-002: Surface Language

## Requirements

### Functional Requirements

1. Remove `supervises` from parser surface role data structures
2. Remove `supervises` from core role data structures
3. Remove placeholder lowering that still manufactures empty `supervises` data
4. Update parser keyword/reserved-word handling and tests accordingly

## Files

- Modify: `crates/ash-parser/src/surface.rs`
- Modify: `crates/ash-parser/src/token.rs`
- Modify: `crates/ash-parser/src/lexer.rs`
- Modify: `crates/ash-parser/src/parse_pattern.rs`
- Modify: `crates/ash-parser/src/parse_expr.rs`
- Modify: `crates/ash-parser/src/parse_workflow.rs`
- Modify: `crates/ash-parser/src/lower.rs`
- Modify: `crates/ash-core/src/ast.rs`
- Modify: parser/core tests that still assert `supervises`
- Modify: `CHANGELOG.md`

## TDD Steps

1. Add failing tests or compile-time assertions proving `supervises` is still exposed
2. Verify RED with focused `ash-parser` / `ash-core` test runs
3. Remove the legacy field and adjust callers
4. Verify GREEN with focused tests
5. Commit

## Non-goals

- No source role-definition parsing yet
- No runtime policy behavior changes yet
