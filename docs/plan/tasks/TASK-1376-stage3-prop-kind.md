# TASK-1376: Stage 3 — `Prop` kind promotion

## Status: 📝 Planned

## Description

Promote `Prop` from convention to distinct kind.

## Requirements

This task is split into sub-tasks:
- [TASK-1376a](TASK-1376a-prop-kind-variant.md): Add `Kind::Prop` variant
- [TASK-1376b](TASK-1376b-proof-irrelevance.md): Proof irrelevance
- [TASK-1376c](TASK-1376c-runtime-escape-prevention.md): Runtime escape prevention

## Acceptance Criteria

- [ ] All sub-tasks complete

## Acceptance Criteria

- [ ] `Prop` is distinct kind
- [ ] Pure/total/termination enforced
- [ ] No runtime escape
- [ ] Typechecker test passes
- [ ] No regressions

## Related

- [PLAN-136](../PLAN-136-INTERFACE-LAW-SYNTAX.md)
- [TASK-1375](TASK-1375-stage3-totality-checking.md)
