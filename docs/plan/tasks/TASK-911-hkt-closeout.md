# TASK-911: Reconcile SPEC-067/PLAN-116 docs, broad gates, and independent review remediation

## Status: ✅ Complete

## Description

Reconcile SPEC-067/PLAN-116 docs, broad gates, and independent review remediation

## Specification Reference

- [SPEC-067](../../spec/SPEC-067-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md)
- [PLAN-116](../PLAN-116-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md)

## Dependencies

- ✅ SPEC-067: spec packet exists
- ✅ PLAN-116: implementation plan exists
- Depends on all prior implementation tasks in this phase and final acceptance evidence
- Depends on TASK-910 completion

## Requirements

1. Reconcile SPEC-067/PLAN-116 docs, broad gates, and independent review requirements.
2. Preserve all non-goals and decision gates from the owning SPEC.
3. Add focused non-zero tests or an explicit docs/audit artifact matching the task type.
4. Avoid broadening later-task semantics by convenience or parser-only lowering.
5. Record final acceptance matrix, broad verification, and completed final independent review remediation.

## File Targets

- Modify: docs/spec/README.md
- Modify: docs/plan/PLAN-INDEX.md
- Modify: CHANGELOG.md
- Modify: docs/plan/PLAN-116-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md
- Modify: docs/spec/SPEC-067-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md
- Add: docs/plan/audits/TASK-911-hkt-closeout.md

## TDD / Execution Steps

1. Re-read the referenced SPEC, PLAN, implementation tasks, and acceptance matrix.
2. Verify every SPEC acceptance row has focused non-zero evidence or an explicit scoped deferral.
3. Run the broad closeout command set recorded in this task after the final code/doc change.
4. Reconcile this task status, the owning PLAN row, PLAN-INDEX, docs/spec/README.md, and CHANGELOG.
5. Remediate final independent Codex review findings before the orchestrator reruns final gates.

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
  - [x] Broad closeout gate evidence is recorded; later exact broad test reruns exposed only stale TASK-757/TASK-758 diagnostic expectations and one stale TASK-906 fail-closed interface-registration expectation now remediated
  - [x] cargo fmt --check passes
  - [x] git diff --check passes
  - [x] Status surfaces and CHANGELOG are reconciled if this task completes
```

## Completion Evidence

- Reconciled SPEC-067 as Implemented MVP in `docs/spec/SPEC-067-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md` and `docs/spec/README.md`, preserving explicit deferrals for higher-rank polymorphism, unrestricted type lambdas, automatic do-target inference, law proving, associated-type-family inversion, broad multi-parameter constructor classes, and generalized runtime lowering through arbitrary user-defined Monad methods.
- Reconciled PLAN-116 and PLAN-INDEX Phase 120 to complete after the focused blocker rerun cleared the earlier local-port failure, without touching Phase 121.
- Verified TASK-904 through TASK-910 are complete and TASK-910 maps every SPEC-067 HKT-1 through HKT-8 row to focused non-zero evidence or explicit scoped deferral.
- Added [TASK-911 HKT closeout audit](../audits/TASK-911-hkt-closeout.md) for acceptance reconciliation, broad verification commands, and independent review remediation.
- Broad closeout commands after the final code/doc change:
  - `cargo fmt --check`
  - `git diff --check`
  - `cargo check --workspace`
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  - `cargo test --workspace`
  - `cargo doc --workspace --no-deps 2>&1 | tee /tmp/ash-plan-116-doc.log`
  - `! grep -i '^warning:' /tmp/ash-plan-116-doc.log`
- Earlier blocker: `cargo test --workspace` was attempted, then the repo-owned serial fallback was run as `TMPDIR=/home/dikini/Projects/ash/.worktrees/phase-120-hkt/target/task911-tmp CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=target/task911-verify scripts/check-rust-tests.sh --workspace`. The fallback reached test execution and failed only in `ash-engine --test llm_engine_integration` because `wiremock` could not bind a local OS port in this sandbox (`PermissionDenied: Operation not permitted`). That blocker was then cleared by the orchestrator rerun `CODEX_NETWORK_ALLOW_LOCAL_BINDING=1 TMPDIR=/home/dikini/Projects/ash/.worktrees/phase-120-hkt/target/task911-tmp CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=target/task911-verify cargo test -p ash-engine --test llm_engine_integration -- --test-threads=1`, which passed 9 tests with 0 failures.
- Fresh gates recorded for closeout: `cargo fmt --check`, `git diff --check`, `cargo check --workspace`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `TMPDIR=/home/dikini/Projects/ash/.worktrees/phase-120-hkt/target/task911-tmp CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=target/task911-verify cargo doc --workspace --no-deps 2>&1 | tee /tmp/ash-plan-116-doc-rerun.log`, and `test ${PIPESTATUS[0]} -eq 0 && ! grep -i '^warning:' /tmp/ash-plan-116-doc-rerun.log`.
- Independent review: final independent Codex review completed and requested remediation for stale pending-review wording across PLAN-116/TASK-911/audit/CHANGELOG. This remediation updates those docs to record the review as completed with findings remediated.
- Fresh broad-test remediation: the exact broad workspace rerun `TMPDIR=/home/dikini/Projects/ash/.worktrees/phase-120-hkt/target/task911-tmp CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=target/task911-verify cargo test --workspace` initially failed only in `ash-typeck --test task_757_comprehension_elaboration`, where `comprehension_rejects_missing_dictionary_target` still expected the old `no MVP dictionary` wording. The test now asserts the SPEC-067 missing `Monad<K>` evidence diagnostic, including `missing Monad evidence` and `Monad<Option>`.
- Second fresh broad-test remediation: the next exact broad workspace rerun with the same `TMPDIR`, `CARGO_INCREMENTAL`, and `CARGO_TARGET_DIR` failed only in `ash-typeck --test task_758_comprehension_diagnostics`, test `missing_dictionary_does_not_overclaim_future_dictionaries`, for the same stale expectation. The test now asserts the SPEC-067 missing `Monad<K>` evidence diagnostic, including `missing Monad evidence`, `SPEC-067 Monad<K> evidence`, and `Monad<Option>`, while retaining negative coverage for stale or overclaiming target-inference wording.
- Third fresh broad-test remediation: the next exact broad workspace rerun with the same `TMPDIR`, `CARGO_INCREMENTAL`, and `CARGO_TARGET_DIR` failed only in `ash-typeck --test task_906_hkt_fail_closed`, test `type_env_interface_registration_rejects_constructor_kinded_type_params`, because it still expected TypeEnv interface registration to fail closed for constructor-kinded interface binders. TASK-908 now owns higher-kinded interface registration, so the test now asserts that TypeEnv interface registration accepts TASK-908 constructor-kinded interface binders.
- Post-remediation orchestrator verification completed: the final review findings were remediated, the focused stale-test reruns passed, and the final full `cargo test --workspace` passed against the remediated diff.

## Dependencies for Next Task

This task produces its verified slice for later tasks in [PLAN-116](../PLAN-116-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md). Later tasks may cite this task's evidence but must not silently expand its scope.

## Notes

HKT is a cross-cutting type-system feature; do not implement as do-only magic.
