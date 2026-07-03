# TASK-1866: Function Body Language Fixture

**Status:** Complete
**Plan:** [PLAN-185](../PLAN-185-SURFACE-FUNCTION-LANGUAGE.md)

## Description

Add a cohesive conformance fixture proving target `fn` bodies compose ordinary expressions, records, ADTs, pattern matching, calls, and `do { ... }` without requiring workflow syntax in source.

## Requirements

- Cover a single realistic `fn`-only module or engine source.
- Include explicit row syntax on at least one function.
- Include direct-style `do { ... }`.
- Include record construction/access, ADT construction, and pattern matching using existing syntax.
- Keep assertions focused on parse/check/lowering evidence rather than broad runtime semantics.

## TDD Steps

1. Add RED fixture coverage for the cohesive source.
2. Implement only the missing parser/typechecker/lowering behavior needed by that fixture.
3. Verify the fixture passes.

## Completion Checklist

- [x] Fixture added.
- [x] RED/GREEN evidence recorded.
- [x] Focused verification recorded.

## Verification Evidence

- Added `fn_main_source_composes_records_adts_match_calls_and_do_without_workflow` covering a function-only source with local type declarations, named record construction, nominal record field access, ADT construction, ADT pattern matching, helper calls, and `do { ... }`.
- Intermediate RED failures exposed missing local type-definition registration and missing nominal record field access.
- GREEN: `cargo test -p ash-engine --test task_1865_surface_fn_main_entry` passed with 2/2 tests.
