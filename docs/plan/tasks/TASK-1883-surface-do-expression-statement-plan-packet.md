# TASK-1883: Surface Do Expression Statement Plan Packet

**Status:** Complete
**Plan:** [PLAN-190](../PLAN-190-SURFACE-DO-EXPRESSION-STATEMENTS.md)

## Description

Create the Phase 190 plan packet for accepting ordinary expression statements in target
`do { ... }` sequencing.

## Requirements

1. Add a focused plan that depends on Phase 189 and stays on the unified direct-style `do` path.
2. Add implementation task coverage for parser, engine, and CLI behavior.
3. Update plan/spec/notes orientation indexes.
4. Avoid introducing workflow syntax, tower profiles, or a second semantic path.

## TDD Steps

1. Capture current parse failures for `do { call(); return value; }` and
   `do { expr; return value; }`.
2. Add focused parser/engine regressions.
3. Implement expression-statement parsing/lowering/evaluation in the existing `do` path.
4. Re-run existing Phase 185/188/189 non-interference checks.

## Completion Checklist

- [x] PLAN-190 created.
- [x] TASK-1883 created.
- [x] TASK-1884 created.
- [x] PLAN-INDEX updated.
- [x] SPEC-INDEX and NOTE-INDEX updated.
