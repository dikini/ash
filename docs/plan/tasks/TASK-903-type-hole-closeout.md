# TASK-903: Reconcile SPEC-066/PLAN-115 docs, diagnostics, acceptance matrix, broad gates, and review remediation

## Status: ✅ Complete

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
  - scripts/check-rust-tests.sh --workspace
  - cargo doc --workspace --no-deps
checklist:
  - [x] Acceptance matrix evidence is recorded
  - [x] Broad closeout gates pass
  - [x] cargo fmt --check passes
  - [x] git diff --check passes
  - [x] Status surfaces and CHANGELOG are reconciled if this task completes
```

## Completion Notes

- Reconciled SPEC-066 as Implemented MVP in `docs/spec/SPEC-066-TYPE-HOLES-PARTIAL-CONSTRUCTOR-APPLICATION.md` and `docs/spec/README.md`, preserving explicit deferrals for HKT binders, arbitrary type lambdas, Monad evidence, do-target inference, and output-driven inversion.
- Added `docs/plan/audits/TASK-903-type-hole-acceptance-matrix.md`, mapping every SPEC-066 §8 row H-1 through H-6 to focused non-zero TASK-899 through TASK-902 evidence.
- Remediated independent TASK-902 review blockers by updating stale in-module do-target tests and adding direct no-inversion do-target evidence for associated-family hole contexts.
- Verification evidence: focused TASK-899 through TASK-902 count commands are recorded in the acceptance matrix. Broad closeout gates after the final code/doc change are `cargo fmt --check`, `git diff --check`, `cargo check --workspace`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `scripts/check-rust-tests.sh --workspace`, `cargo doc --workspace --no-deps 2>&1 | tee /tmp/ash-phase119-doc.log`, and `! grep -i '^warning:' /tmp/ash-phase119-doc.log`.

## Dependencies for Next Task

This task produces its verified slice for later tasks in [PLAN-115](../PLAN-115-TYPE-HOLES-PARTIAL-CONSTRUCTOR-APPLICATION.md). Later tasks may cite this task's evidence but must not silently expand its scope.

## Notes

Explicit `_` holes are not implicit currying and do not solve by inversion.
