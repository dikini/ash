# TASK-1005: Match exhaustiveness and missing-witness diagnostics

## Status: 📝 Planned

## Description

Harden ordinary `match` exhaustiveness diagnostics and close audit-discovered gaps in nested/product coverage without expanding beyond ordinary ADTs.

## Specification Reference

- [SPEC-076](../../spec/SPEC-076-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md)
- [PLAN-126](../PLAN-126-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md)

## Dependencies

- ✅ TASK-1000 packet exists
- ✅ TASK-1001 audit gate completed and patched this task with focused commands

## Requirements

1. Preserve SPEC-076 non-goals and decision gates.
2. Add RED tests for missing constructors, wildcard/default coverage, wildcard-only matches over open or non-ADT scrutinees, blocked constructor-specific canonicalization, wrong constructor identity, nested/product coverage gaps, duplicate or unreachable arms where currently supported, and nested field witness rendering identified by the audit.
3. Preserve SPEC-068 alias/projection behavior and universal wildcard/default behavior.
4. Do not infer constructors under neutral or rigid type heads or treat constructor-specific coverage over blocked universes as exhaustive by guesswork.
5. Ensure diagnostics include scrutinee type, missing witness or blocked reason, span, and likely fix.

## File Targets

- Modify candidates: `crates/ash-typeck/src/exhaustiveness.rs`, `crates/ash-typeck/src/check_expr.rs`, `crates/ash-typeck/src/type_env.rs`, `crates/ash-typeck/src/error.rs`
- Test: `crates/ash-typeck/tests/task_1005_match_exhaustiveness.rs`

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
  - bash -lc "cargo test -p ash-typeck --test task_1005_match_exhaustiveness -- --list | rg 'match_wildcard_default_accepts_open_non_adt_scrutinee|match_missing_adt_constructor_reports_NonExhaustiveMatch_diagnostic|match_blocked_constructor_coverage_reports_blocked_reason|match_impossible_pattern_reports_type_error|match_nested_product_coverage_does_not_overgeneralize|match_list_patterns_have_conservative_diagnostics' && cargo test -p ash-typeck --test task_1005_match_exhaustiveness"
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

Match/exhaustiveness semantics
