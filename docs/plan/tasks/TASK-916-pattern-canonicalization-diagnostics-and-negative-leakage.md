# TASK-916: Add blocked-neutral, wrong-identity, and unrelated-name leakage diagnostics/tests

## Status: 📝 Planned

## Description

Add blocked-neutral, wrong-identity, and unrelated-name leakage diagnostics/tests

## Specification Reference

- [SPEC-068](../../spec/SPEC-068-PATTERN-EXHAUSTIVENESS-CANONICALIZATION.md)
- [PLAN-117](../PLAN-117-PATTERN-EXHAUSTIVENESS-CANONICALIZATION.md)

## Dependencies

- ✅ SPEC-068: spec packet exists
- ✅ PLAN-117: implementation plan exists
- Depends on TASK-912 audit gate replacing this task's fail-closed verification guard
- Depends on TASK-915 completion

## Requirements

1. Add blocked-neutral, wrong-identity, and unrelated-name leakage diagnostics/tests.
2. Preserve all non-goals and decision gates from the owning SPEC.
3. Add focused non-zero tests or an explicit docs/audit artifact matching the task type.
4. Avoid broadening later-task semantics by convenience or parser-only lowering.

## File Targets

- `crates/ash-typeck/src/error.rs`: add or route stable pattern canonicalization diagnostics if existing variants are insufficient.
- `crates/ash-typeck/src/check_pattern.rs`: report wrong-identity and blocked canonicalization errors from the TASK-913/TASK-914 boundary.
- `crates/ash-typeck/src/check_expr.rs`: report blocked exhaustiveness universe diagnostics from the TASK-915 boundary.
- `crates/ash-typeck/tests/task_916_pattern_canonicalization_diagnostics.rs`: create focused blocked-neutral and wrong-identity diagnostic tests.
- `crates/ash-typeck/tests/task_916_pattern_canonicalization_negative_leakage.rs`: create focused unrelated same-visible-name and direct ADT non-interference tests.
- Audit reference: [TASK-912 pattern canonicalization audit gate](../audits/TASK-912-pattern-canonicalization-audit-gate.md).

## TDD / Execution Steps

1. Re-read the referenced SPEC and this task file.
2. Add the smallest failing parser/typechecker/core/engine/doc test that proves this task's boundary.
3. Implement only this task's boundary; do not import later-task semantics.
4. Run the focused command set recorded in this task after the audit gate replaces any failing placeholder.
5. Update this task status, the owning PLAN row, PLAN-INDEX, and CHANGELOG only after verification evidence is fresh.

## Dispatch

```yaml
agent: hermes
reasoning: medium
max_turns: 16
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - cargo test -p ash-typeck --test task_916_pattern_canonicalization_diagnostics -- --nocapture
  - cargo test -p ash-typeck --test task_916_pattern_canonicalization_negative_leakage -- --nocapture
  - cargo fmt --check
  - git diff --check
  - cargo check --workspace
checklist:
  - [ ] Focused tests are non-zero and pass
  - [ ] cargo fmt --check passes
  - [ ] git diff --check passes
  - [ ] Status surfaces and CHANGELOG are reconciled if this task completes
```

## Dependencies for Next Task

This task produces its verified slice for later tasks in [PLAN-117](../PLAN-117-PATTERN-EXHAUSTIVENESS-CANONICALIZATION.md). Later tasks may cite this task's evidence but must not silently expand its scope.

## Notes

Pattern canonicalization is audit-first and must not solve under neutral computation heads.
