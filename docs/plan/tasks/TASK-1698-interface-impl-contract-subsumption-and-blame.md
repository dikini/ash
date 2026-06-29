# TASK-1698: Interface/Impl Contract Subsumption and Blame

**Status:** ✅ Complete
**Phase:** [PLAN-165](../PLAN-165-CONTRACT-SYSTEM-IMPLEMENTATION-HANDOFF.md)
**Owner:** Phase 165

## Description

Check interface-to-impl contract inheritance with behavioral subtyping and preserve blame labels through diagnostics.

## Specification Reference

- [NOTE-027](../../notes/NOTE-027-CONTRACT-BLAME-AND-SUBSUMPTION.md)
- [NOTE-032](../../notes/NOTE-032-CONTRACT-SOUNDNESS-OBLIGATIONS.md)
- [SPEC-097b](../../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md)
- [SPEC-100](../../spec/SPEC-100-CORE-TYPE-CHECKING.md)

## Dependencies

- ✅ TASK-1697: Contract discharge and evidence metadata

## Requirements

1. Enforce `{P} C {Q} ⊑ {P'} C {Q'}` iff `P ⇒ P'` and `Q' ⇒ Q`.
2. Check impl contracts eagerly at impl definition or summary-validation time.
3. Preserve blame polarity: `requires` failure is negative/caller-side; `ensures` failure is positive/callee-or-impl-side.
4. Keep handler decisions separate from blame labels.

## TDD Steps

1. Add failing tests in `crates/ash-core/tests/task_1698_interface_impl_contract_subsumption.rs`.
2. Wire contract summaries into the existing interface/impl checking path or create a Core-summary-level checker if surface integration remains deferred.
3. Add negative tests for strengthened preconditions and weakened postconditions.

## Verification

```text
strictness: clean
commands:
  - cargo fmt --check
  - cargo test -p ash-core --test task_1698_interface_impl_contract_subsumption
  - cargo clippy -p ash-core --all-targets -- -D warnings
checklist:
  - [x] Precondition weakening accepted.
  - [x] Precondition strengthening rejected.
  - [x] Postcondition strengthening accepted.
  - [x] Postcondition weakening rejected.
  - [x] Blame labels survive dynamic diagnostics.
```

## Dependencies for Next Task

Required by TASK-1702.
