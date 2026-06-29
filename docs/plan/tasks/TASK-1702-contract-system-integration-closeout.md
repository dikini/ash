# TASK-1702: Contract System Integration Closeout

**Status:** 📝 Planned
**Phase:** [PLAN-165](../PLAN-165-CONTRACT-SYSTEM-IMPLEMENTATION-HANDOFF.md)
**Owner:** Phase 165

## Description

Close Phase 165 by adding integration fixtures, documentation consistency checks, status reconciliation, and broad verification evidence.

## Specification Reference

- [PLAN-165](../PLAN-165-CONTRACT-SYSTEM-IMPLEMENTATION-HANDOFF.md)
- [NOTE-014](../../notes/NOTE-014-CONTRACT-SYSTEMS-UNIFICATION.md)
- [SPEC-096b](../../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md)
- [SPEC-097b](../../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md)
- [SPEC-098b](../../spec/SPEC-098b-TARGET-IR.md)
- [SPEC-099](../../spec/SPEC-099-CORE-LANGUAGE.md)
- [SPEC-100](../../spec/SPEC-100-CORE-TYPE-CHECKING.md)

## Dependencies

- 📝 TASK-1698: Interface/impl contract subsumption and blame
- 📝 TASK-1699: Capability observation evidence boundary
- 📝 TASK-1701: Temporal monitor runtime diagnostics

## Requirements

1. Add end-to-end Core fixtures covering value contracts, dynamic traps, observation evidence, and trace monitors.
2. Update reference or docs pages only if implementation changes public behavior.
3. Reconcile PLAN-165 and PLAN-INDEX status surfaces.
4. Update CHANGELOG with final implementation closeout evidence.
5. Run stale-claim sweeps listed in PLAN-165.
6. Run broad workspace verification.

## Verification

```text
strictness: clean
commands:
  - cargo fmt --check
  - cargo test --all
  - cargo clippy --all-targets --all-features
  - bash scripts/check-docs-gate.sh
  - git diff --check
checklist:
  - [ ] All Phase 165 task files are complete and checked.
  - [ ] PLAN-165 and PLAN-INDEX agree.
  - [ ] CHANGELOG records implementation closeout.
  - [ ] Stale-claim sweep has no live normative stale matches.
  - [ ] Reference corpus audit is complete if reference pages changed.
```

## Closeout Evidence

To be filled when the implementation phase closes.
