# TASK-1376: Stage 3 — `Prop` kind promotion

## Status: ✅ Complete

## Description

Promote `Prop` from convention to distinct kind.

## Requirements

This task is split into sub-tasks:
- [TASK-1376a](TASK-1376a-prop-kind-variant.md): Add `Kind::Prop` variant — ✅ Complete
- [TASK-1376b](TASK-1376b-proof-irrelevance.md): Proof irrelevance — ✅ Complete
- [TASK-1376c](TASK-1376c-runtime-escape-prevention.md): Runtime escape prevention — ✅ Complete

## Acceptance Criteria

- [x] All sub-tasks complete

## Acceptance Criteria

- [x] `Prop` is distinct kind
- [x] Pure/total/termination enforced
- [x] No runtime escape
- [x] Typechecker test passes
- [x] No regressions

## Related

- [PLAN-136](../PLAN-136-INTERFACE-LAW-SYNTAX.md)
- [TASK-1375](TASK-1375-stage3-totality-checking.md)
