# TASK-1889: Surface Tuple ADT Plan Packet

**Status:** Complete
**Plan:** [PLAN-193](../PLAN-193-SURFACE-TUPLE-ADT-EXPRESSIONS.md)

## Description

Create the Phase 193 plan packet for tuple-payload ADTs in the function-first surface language.

## Requirements

1. Add a focused plan that depends on Phase 192 and stays on the ordinary expression path.
2. Add implementation task coverage for parser, typechecker, engine, and CLI behavior.
3. Update plan/spec/notes orientation indexes.
4. Avoid introducing workflow syntax, a new variant runtime representation, or a second semantic
   path.

## TDD Steps

1. Capture the current function-first tuple-ADT failure.
2. Add focused parser and engine regressions.
3. Implement tuple-payload constructor preservation through existing constructor/lowering paths.
4. Re-run Phase 188/189/192 non-interference checks.

## Completion Checklist

- [x] PLAN-193 created.
- [x] TASK-1889 created.
- [x] TASK-1890 created.
- [x] PLAN-INDEX updated.
- [x] SPEC-INDEX and NOTE-INDEX updated.
