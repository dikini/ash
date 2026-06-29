# TASK-1701: Temporal Monitor Runtime Diagnostics

**Status:** 📝 Planned
**Phase:** [PLAN-165](../PLAN-165-CONTRACT-SYSTEM-IMPLEMENTATION-HANDOFF.md)
**Owner:** Phase 165

## Description

Implement temporal monitor result handling and separate temporal contract violations from monitor evaluator faults.

## Specification Reference

- [NOTE-035](../../notes/NOTE-035-TEMPORAL-AND-CONCURRENT-CONTRACTS.md)
- [SPEC-098b](../../spec/SPEC-098b-TARGET-IR.md)
- [SPEC-099 §6.6](../../spec/SPEC-099-CORE-LANGUAGE.md)
- [SPEC-100 §9.5](../../spec/SPEC-100-CORE-TYPE-CHECKING.md)

## Dependencies

- 📝 TASK-1700: Trace contract monitor sidecars
- 📝 TASK-1696: Dynamic contract traps and predicate faults

## Requirements

1. Add monitor result states: satisfied, violated, pending, inconclusive, and faulted.
2. Trap temporal violations with `TemporalContractViolation(TemporalContractDiagnostic)` by default.
3. Trap monitor evaluator faults with `TemporalMonitorFault(TemporalMonitorFaultDiagnostic)`.
4. Keep monitors authority-free: they consume recorded trace/evidence/timer facts only.
5. Model recoverable compensation through explicit row-accounted paths, not silent resume.

## TDD Steps

1. Add failing tests in `crates/ash-interp/tests/task_1701_temporal_monitor_runtime_diagnostics.rs` and Core carrier tests if needed.
2. Implement minimal monitor evaluation over hand-authored facts sufficient for tests.
3. Add negative tests for monitor fault versus violated formula.

## Verification

```text
strictness: clean
commands:
  - cargo fmt --check
  - cargo test -p ash-interp --test task_1701_temporal_monitor_runtime_diagnostics
  - cargo clippy -p ash-interp --all-targets -- -D warnings
checklist:
  - [ ] Violation and monitor fault use distinct trap payloads.
  - [ ] Pending liveness behavior is explicit in tests.
  - [ ] Monitor consumes facts without provider/process authority.
```

## Dependencies for Next Task

Required by TASK-1702.
