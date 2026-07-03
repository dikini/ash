# TASK-1879: Surface Match Constructor Scrutinee Plan Packet

**Status:** Complete
**Plan:** [PLAN-188](../PLAN-188-SURFACE-MATCH-CONSTRUCTOR-SCRUTINEES.md)

## Description

Create the Phase 188 plan packet for the next Surface Function Language slice: ordinary ADT
constructor expressions as `match` scrutinees in function-first source.

## Requirements

1. Add a focused plan that depends on Phase 187 and stays inside the function-first target-language
   path.
2. Add implementation task coverage for parsing, checking, executing, and CLI probing
   constructor-expression match scrutinees.
3. Update plan/spec/notes orientation indexes so future work routes through Phase 188.
4. Avoid legacy workflow or tower-as-core wording.

## TDD Steps

1. Capture the current parser failure for `match Some { value: 41 } { ... }`.
2. Add a focused engine regression for the desired behavior.
3. Implement the smallest parser change that makes the regression pass.
4. Verify existing Phase 185 variable-scrutinee match coverage remains green.

## Completion Checklist

- [x] PLAN-188 created.
- [x] TASK-1879 created.
- [x] TASK-1880 created.
- [x] PLAN-INDEX updated.
- [x] SPEC-INDEX and NOTE-INDEX updated.
