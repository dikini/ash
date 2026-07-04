# TASK-1901: Contract Evidence Closeout

**Status:** Planned
**Plan:** [PLAN-194](../PLAN-194-CONTRACT-AND-EVIDENCE-SYSTEM.md)

## Description

Close out Phase 194 with cohesive fixtures, documentation reconciliation, verification gates, and
review remediation.

## Requirements

1. Add end-to-end function-first fixtures covering `requires`, `ensures`, evidence rows, dynamic
   checks, blame, and diagnostics.
2. Reconcile target specs, orientation indexes, task statuses, and changelog.
3. Run focused and broad verification gates.
4. Perform a stale-claim sweep for contract/evidence authority wording.
5. Address code review findings before marking the phase complete.

## TDD Steps

1. Add end-to-end fixtures that fail before all Phase 194 slices are integrated.
2. Run focused tests for parser, typechecker, engine/Core/runtime diagnostics, and CLI output.
3. Run broad workspace and docs gates.
4. Update status surfaces only after evidence proves completion.

## Completion Checklist

- [ ] End-to-end contract/evidence fixtures pass.
- [ ] `cargo fmt --check` passes.
- [ ] `cargo test --all` passes.
- [ ] `cargo clippy --all-targets --all-features` passes.
- [ ] Orientation index validation and docs gate pass.
- [ ] CHANGELOG.md updated.
- [ ] PLAN-INDEX and task statuses reconciled.
