# TASK-1881: Surface Match Ordinary Scrutinee Plan Packet

**Status:** Complete
**Plan:** [PLAN-189](../PLAN-189-SURFACE-MATCH-ORDINARY-SCRUTINEES.md)

## Description

Create the Phase 189 plan packet for making function-body match scrutinees accept ordinary call,
field-projection, and binary expressions after Phase 188 added ADT constructor-expression
scrutinees.

## Requirements

1. Add a focused plan that depends on Phase 188 and stays on the function-first target-language
   path.
2. Add implementation task coverage for parser, engine, and CLI behavior.
3. Update plan/spec/notes orientation indexes.
4. Avoid introducing workflow syntax, tower profiles, or a second semantic path.

## TDD Steps

1. Capture current parse failures for call, field-projection, and binary match scrutinees.
2. Add focused parser and engine regressions.
3. Implement the smallest parser change that makes those scrutinees ordinary enough for the target
   function language.
4. Re-run Phase 188 and Phase 185 non-interference checks.

## Completion Checklist

- [x] PLAN-189 created.
- [x] TASK-1881 created.
- [x] TASK-1882 created.
- [x] PLAN-INDEX updated.
- [x] SPEC-INDEX and NOTE-INDEX updated.
