# TASK-1699: Capability Observation Evidence Boundary

**Status:** ✅ Complete
**Phase:** [PLAN-165](../PLAN-165-CONTRACT-SYSTEM-IMPLEMENTATION-HANDOFF.md)
**Owner:** Phase 165

## Description

Implement operation-produced observation evidence so contracts can diagnose values produced under authority without granting authority to predicate evaluators.

## Specification Reference

- [NOTE-034](../../notes/NOTE-034-CONTRACT-CAPABILITY-BOUNDARY.md)
- [SPEC-096b §6.1](../../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md)
- [SPEC-098b](../../spec/SPEC-098b-TARGET-IR.md)
- [SPEC-099 §6.5](../../spec/SPEC-099-CORE-LANGUAGE.md)

## Dependencies

- ✅ TASK-1695: Contract predicate validation and lowering
- ✅ TASK-1696: Dynamic contract traps and predicate faults

## Requirements

1. Add `ObservationEvidence` or equivalent sidecar metadata for operation-produced values.
2. Allow predicates to inspect operation-produced boundary values as ordinary values.
3. Ensure predicate evaluators receive no provider handle, role admission token, or authority environment.
4. Keep admission failure, operation failure, predicate false, and predicate fault as separate diagnostic classes.
5. Apply redaction/summary/unavailable policies to observed diagnostic values.

## TDD Steps

1. Add failing tests in `crates/ash-core/tests/task_1699_capability_observation_evidence.rs` and runtime tests if evidence is produced by `ash-interp`.
2. Extend Core value/diagnostic sidecars without changing operation-row semantics.
3. Add a negative leakage test proving a predicate cannot call the provider whose result it inspects.

## Verification

```text
strictness: clean
commands:
  - cargo fmt --check
  - cargo test -p ash-core --test task_1699_capability_observation_evidence
  - cargo test -p ash-interp --test task_1699_capability_observation_evidence
  - cargo clippy -p ash-core -p ash-interp --all-targets -- -D warnings
checklist:
  - [x] Operation-produced value inspection works.
  - [x] Provider authority is unavailable inside predicates.
  - [x] Diagnostic redaction does not erase the contract failure.
```

## Dependencies for Next Task

Required by TASK-1702.
