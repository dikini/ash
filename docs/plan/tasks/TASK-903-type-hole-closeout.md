# TASK-903: Reconcile SPEC-066/PLAN-115 docs, diagnostics, acceptance matrix, broad gates, and review remediation

## Status: 📝 Planned

## Description

Reconcile SPEC-066/PLAN-115 docs, diagnostics, acceptance matrix, broad gates, and review remediation

## Specification Reference

- [SPEC-066](../../spec/SPEC-066-TYPE-HOLES-PARTIAL-CONSTRUCTOR-APPLICATION.md)
- [PLAN-115](../PLAN-115-TYPE-HOLES-PARTIAL-CONSTRUCTOR-APPLICATION.md)

## Dependencies

- ✅ SPEC-066: spec packet exists
- ✅ PLAN-115: implementation plan exists
- Depends on all prior implementation tasks in this phase and final acceptance evidence
- Depends on TASK-902 completion

## Requirements

1. Reconcile SPEC-066/PLAN-115 docs, diagnostics, acceptance matrix, broad gates, and review remediation.
2. Preserve all non-goals and decision gates from the owning SPEC.
3. Add focused non-zero tests or an explicit docs/audit artifact matching the task type.
4. Avoid broadening later-task semantics by convenience or parser-only lowering.
5. Record final acceptance matrix, broad verification, and independent review remediation.

## File Targets

- Modify: docs/spec/README.md
- Modify: docs/plan/PLAN-INDEX.md
- Modify: CHANGELOG.md
- Modify: docs/plan/PLAN-115-TYPE-HOLES-PARTIAL-CONSTRUCTOR-APPLICATION.md

## TDD / Execution Steps

1. Re-read the referenced SPEC, PLAN, implementation tasks, and acceptance matrix.
2. Verify every SPEC acceptance row has focused non-zero evidence or an explicit scoped deferral.
3. Run the broad closeout command set recorded in this task after the final code/doc change.
4. Reconcile this task status, the owning PLAN row, PLAN-INDEX, docs/spec/README.md, and CHANGELOG.
5. Run independent review remediation before marking the phase complete.

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
  - cargo fmt --check
  - git diff --check
  - cargo check --workspace
  - cargo clippy --workspace --all-targets --all-features -- -D warnings
  - cargo test --workspace
  - cargo doc --workspace --no-deps
checklist:
  - [ ] Acceptance matrix evidence is recorded
  - [ ] Broad closeout gates pass
  - [ ] cargo fmt --check passes
  - [ ] git diff --check passes
  - [ ] Status surfaces and CHANGELOG are reconciled if this task completes
```

## Dependencies for Next Task

This task produces its verified slice for later tasks in [PLAN-115](../PLAN-115-TYPE-HOLES-PARTIAL-CONSTRUCTOR-APPLICATION.md). Later tasks may cite this task's evidence but must not silently expand its scope.

## Notes

Explicit `_` holes are not implicit currying and do not solve by inversion.
