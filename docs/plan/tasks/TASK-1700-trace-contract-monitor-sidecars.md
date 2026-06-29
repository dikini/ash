# TASK-1700: Trace Contract Monitor Sidecars

**Status:** 📝 Planned
**Phase:** [PLAN-165](../PLAN-165-CONTRACT-SYSTEM-IMPLEMENTATION-HANDOFF.md)
**Owner:** Phase 165

## Description

Add Core/IR carriers for trace contracts, trace facts, workflow ledger facts, temporal formulas, and monitor plans.

## Specification Reference

- [NOTE-035](../../notes/NOTE-035-TEMPORAL-AND-CONCURRENT-CONTRACTS.md)
- [SPEC-096b §6.5](../../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md)
- [SPEC-097b](../../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md)
- [SPEC-098b](../../spec/SPEC-098b-TARGET-IR.md)

## Dependencies

- 📝 TASK-1694: Core contract predicate artifacts

## Requirements

1. Add carriers equivalent to `TraceContract`, `TraceFactKind`, `WorkflowLedgerFact`, `TemporalFormula`, `MonitorPlan`, and `TraceContractDischarge`.
2. Classify operational alphabets as `Proc`-like, normative/evidence alphabets as `Workflow`-like, and mixed alphabets as mixed.
3. Keep trace contracts separate from value-level `LoweredPredicate`.
4. Reject facts outside the monitor scope in type/validation helpers.

## TDD Steps

1. Add failing tests in `crates/ash-core/tests/task_1700_trace_contract_monitor_sidecars.rs`.
2. Implement carriers in `ash-core` near Core/IR sidecar metadata.
3. Add classification tests for operational, normative, and mixed alphabets.

## Verification

```text
strictness: clean
commands:
  - cargo fmt --check
  - cargo test -p ash-core --test task_1700_trace_contract_monitor_sidecars
  - cargo clippy -p ash-core --all-targets -- -D warnings
checklist:
  - [ ] Trace contracts do not lower to LoweredPredicate.
  - [ ] Mixed alphabet classification is tested.
  - [ ] Workflow ledger facts preserve source trace links.
```

## Dependencies for Next Task

Required by TASK-1701.
