# TASK-1002: Type-aware irrefutable pattern API

## Status: 📝 Planned

## Description

Add a shared `ash-typeck` API that decides whether a pattern is irrefutable for a canonical scrutinee type and returns typed bindings plus missing/non-match witnesses.

## Specification Reference

- [SPEC-076](../../spec/SPEC-076-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md)
- [PLAN-126](../PLAN-126-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md)

## Dependencies

- ✅ TASK-1000 packet exists
- ✅ TASK-1001 audit gate completed and patched this task with focused commands

## Requirements

1. Preserve SPEC-076 non-goals and decision gates.
2. Add RED tests for variable, wildcard, single-variant ADT, multi-variant ADT, nested refutable fields, tuple/record, list, literal, duplicate binders, impossible patterns, and blocked variant/product-shape cases.
3. Implement a conservative type-aware irrefutability API using SPEC-068 canonical constructor universes where needed, while separating variant constructor-universe checks from tuple/record product-shape checks.
4. Return structured irrefutable/refutable/impossible/blocked outcomes with witnesses or blocked reasons suitable for diagnostics.
5. Preserve universal wildcard/variable behavior for open or non-ADT scrutinees.
6. Keep parser and runtime unchanged.

## File Targets

- Modify candidates: `crates/ash-typeck/src/check_pattern.rs`, `crates/ash-typeck/src/exhaustiveness.rs`, `crates/ash-typeck/src/type_env.rs`, `crates/ash-typeck/src/error.rs`
- Test: `crates/ash-typeck/tests/task_1002_irrefutable_pattern_api.rs`

## TDD / Execution Steps

1. Stop if this file still contains the fail-closed TASK-1001 verification guard.
2. Write RED tests named by TASK-1001.
3. Implement the smallest semantic change for this task only.
4. Run focused tests and required workspace checks.
5. Request independent review before marking complete.

## Dispatch

```yaml
agent: hermes
reasoning: medium
max_turns: 20
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - bash -lc "cargo test -p ash-typeck --test task_1002_irrefutable_pattern_api -- --list | rg 'irrefutable_variable_and_wildcard_accept_open_non_adt_scrutinees|irrefutable_single_variant_adt_accepts_nested_irrefutable_fields|irrefutable_nested_refutable_binder_reports_missing_witness|irrefutable_list_pattern_without_rest_is_refutable|irrefutable_literal_pattern_is_refutable_without_singleton|irrefutable_duplicate_binders_rejected|irrefutable_impossible_pattern_reports_type_mismatch|irrefutable_blocked_constructor_coverage_reports_blocked_reason' && cargo test -p ash-typeck --test task_1002_irrefutable_pattern_api"
  - cargo fmt --check
  - git diff --check
  - cargo check --workspace
checklist:
  - [x] TASK-1001 replaced the fail-closed guard
  - [ ] RED tests fail before implementation and pass after implementation
  - [ ] Scope did not expand beyond SPEC-076
  - [ ] Diagnostics are asserted where required
```

## Dependencies for Next Task

This task produces its verified slice for later tasks in [PLAN-126](../PLAN-126-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md). Later tasks may cite this task's evidence but must not silently expand its scope.

## Notes

TypeEnv/check_pattern/exhaustiveness substrate
