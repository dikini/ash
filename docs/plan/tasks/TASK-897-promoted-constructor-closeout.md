# TASK-897: Reconcile SPEC-065/PLAN-114 docs, acceptance matrix, broad gates, and review remediation

## Status: ✅ Complete

## Description

Reconcile SPEC-065/PLAN-114 docs, acceptance matrix, broad gates, and review remediation

## Specification Reference

- [SPEC-065](../../spec/SPEC-065-PROMOTED-DATA-CONSTRUCTORS-NAMED-DATA-KINDS.md)
- [PLAN-114](../PLAN-114-PROMOTED-DATA-CONSTRUCTORS-NAMED-DATA-KINDS.md)

## Dependencies

- ✅ SPEC-065: spec packet exists
- ✅ PLAN-114: implementation plan exists
- Depends on all prior implementation tasks in this phase and final acceptance evidence
- Depends on TASK-896 completion

## Requirements

1. Reconcile SPEC-065/PLAN-114 docs, acceptance matrix, broad gates, and review remediation.
2. Preserve all non-goals and decision gates from the owning SPEC.
3. Add focused non-zero tests or an explicit docs/audit artifact matching the task type.
4. Avoid broadening later-task semantics by convenience or parser-only lowering.
5. Record final acceptance matrix, broad verification, and independent review remediation.

## File Targets

- Modify: docs/spec/README.md
- Modify: docs/plan/PLAN-INDEX.md
- Modify: CHANGELOG.md
- Modify: docs/plan/PLAN-114-PROMOTED-DATA-CONSTRUCTORS-NAMED-DATA-KINDS.md

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
  - [x] Acceptance matrix evidence is recorded
  - [x] Broad closeout gates pass
  - [x] cargo fmt --check passes
  - [x] git diff --check passes
  - [x] Status surfaces and CHANGELOG are reconciled if this task completes
```

## Dependencies for Next Task

This task produces its verified slice for later tasks in [PLAN-114](../PLAN-114-PROMOTED-DATA-CONSTRUCTORS-NAMED-DATA-KINDS.md). Later tasks may cite this task's evidence but must not silently expand its scope.

## Notes

Promoted constructors are opt-in and distinct from sealed-domain markers and runtime constructors.

## Completion Notes

- Reconciled SPEC-065 as Implemented MVP in `docs/spec/SPEC-065-PROMOTED-DATA-CONSTRUCTORS-NAMED-DATA-KINDS.md` and `docs/spec/README.md`, with explicit scope that parser `data kind` declarations exist but source-to-summary lowering/export is not claimed by this MVP.
- Mapped every SPEC-065 acceptance row PDC-1 through PDC-6 to focused TASK-894/TASK-895/TASK-896 evidence plus the broad workspace gate.
- Remediated independent review blockers by retaining transitive promoted field-domain dependencies in selected type-function summaries and by hiding selected proposition promoted data-kind dependency metadata to avoid source-visible alias leakage.
- Verification evidence: focused TASK-894/TASK-895/TASK-896 tests plus `cargo test -p ash-engine --lib task896_selected` pass after review remediation. Broad closeout gates are `cargo fmt --check`, `git diff --check`, `cargo check --workspace`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace`, and `cargo doc --workspace --no-deps` after the final code/doc change.