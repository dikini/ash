# TASK-1696: Dynamic Contract Traps and Predicate Faults

**Status:** 📝 Planned
**Phase:** [PLAN-165](../PLAN-165-CONTRACT-SYSTEM-IMPLEMENTATION-HANDOFF.md)
**Owner:** Phase 165

## Description

Implement structured dynamic contract diagnostics and separate false-predicate violations from predicate evaluator faults.

## Specification Reference

- [NOTE-027](../../notes/NOTE-027-CONTRACT-BLAME-AND-SUBSUMPTION.md)
- [NOTE-029](../../notes/NOTE-029-STRUCTURED-BOTTOM-AND-CONTRACT-DIAGNOSTICS.md)
- [NOTE-031](../../notes/NOTE-031-CONTRACT-PREDICATE-WELL-FORMEDNESS-AND-SNAPSHOTS.md)
- [SPEC-098b](../../spec/SPEC-098b-TARGET-IR.md)
- [SPEC-099 §6](../../spec/SPEC-099-CORE-LANGUAGE.md)

## Dependencies

- 📝 TASK-1695: Contract predicate validation and lowering

## Requirements

1. Add or wire `ContractDiagnostic` and `PredicateFaultDiagnostic` runtime payloads.
2. False dynamic predicate traps with `ContractViolation(ContractDiagnostic)`.
3. Predicate evaluator trap/fault traps with `ContractPredicateFault(PredicateFaultDiagnostic)`.
4. Trap typing remains local row `{}` and does not add a `ContractViolation` row item.
5. Explicit recoverability remains an explicit `fail` or other declared recovery path.

## TDD Steps

1. Add failing tests in `crates/ash-core/tests/task_1696_dynamic_contract_traps.rs` and, if runtime execution is touched, `crates/ash-interp/tests/task_1696_dynamic_contract_traps.rs`.
2. Implement Core trap payload carriers in `ash-core`.
3. Wire CPS/runtime diagnostic handling in `ash-interp` only after the Core payload shape is stable.

## Verification

```text
strictness: clean
commands:
  - cargo fmt --check
  - cargo test -p ash-core --test task_1696_dynamic_contract_traps
  - cargo test -p ash-interp --test task_1696_dynamic_contract_traps
  - cargo clippy -p ash-core -p ash-interp --all-targets -- -D warnings
checklist:
  - [ ] False predicate and predicate fault use distinct trap reasons.
  - [ ] No contract trap appears as a row item.
  - [ ] Explicit fail path remains visible when recovery is modeled.
```

## Dependencies for Next Task

Required by TASK-1697, TASK-1698, TASK-1699, TASK-1701.
