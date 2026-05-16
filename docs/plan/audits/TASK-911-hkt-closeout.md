# TASK-911 HKT Closeout Audit

Status: Complete; final independent review completed with findings remediated
Date: 2026-05-16
Branch: phase-120-hkt
Phase: Phase 120 / PLAN-116
Spec: SPEC-067

## Scope

TASK-911 is documentation/status/verification closeout only. It does not implement new HKT, do-notation, evidence, parser, runtime, or summary semantics.

## Prior Task State

TASK-904 through TASK-910 are complete. Their task files record focused non-zero evidence for the audit gate, core carriers, parser surface, TypeEnv constructor-variable kinding/unification, higher-kinded interface/impl evidence, Monad do-target evidence lookup, diagnostics, acceptance, and non-interference.

Latest committed baseline before TASK-911 closeout was `dd6e6d0 test(hkt): add acceptance diagnostics matrix`.

## Acceptance Reconciliation

| ID | Evidence source | Closeout disposition |
|---|---|---|
| HKT-1 | `docs/plan/audits/TASK-910-hkt-acceptance-matrix.md` row HKT-1; `crates/ash-parser/tests/task_910_hkt_diagnostics_surface.rs::hkt1_parses_functor_applicative_and_monad_constructor_binders` | Covered |
| HKT-2 | TASK-910 row HKT-2; `crates/ash-typeck/tests/task_910_hkt_acceptance_matrix.rs::hkt2_interface_method_signature_accepts_constructor_application` | Covered |
| HKT-3 | TASK-910 row HKT-3; `crates/ash-typeck/tests/task_910_hkt_acceptance_matrix.rs::hkt3_impl_monad_option_registers_empty_method_mvp_evidence` | Covered for empty-method MVP evidence |
| HKT-4 | TASK-910 row HKT-4; `crates/ash-parser/tests/task_910_hkt_diagnostics_surface.rs::hkt4_impl_head_preserves_partial_constructor_hole_surface`; `crates/ash-typeck/tests/task_910_hkt_acceptance_matrix.rs::hkt4_result_partial_impl_head_is_registered_only_as_shape_evidence` | Covered as SPEC-066-shaped partial-constructor evidence shape; generalized runtime method lowering remains deferred |
| HKT-5 | TASK-910 row HKT-5; `crates/ash-typeck/tests/task_910_hkt_acceptance_matrix.rs::hkt5_bare_constructor_variable_in_proper_type_position_is_wrong_kind` | Covered |
| HKT-6 | TASK-910 row HKT-6; `crates/ash-typeck/tests/task_910_hkt_acceptance_matrix.rs::hkt6_duplicate_monad_option_impls_are_rejected_as_overlap` | Covered |
| HKT-7 | TASK-910 row HKT-7; `crates/ash-typeck/tests/task_910_hkt_acceptance_matrix.rs::hkt7_do_option_uses_registered_monad_evidence_at_type_boundary` | Covered for target resolution and return-only type boundary; law/runtime method semantics remain deferred |
| HKT-8 | TASK-910 row HKT-8; `crates/ash-typeck/tests/task_910_hkt_acceptance_matrix.rs::hkt8_do_list_without_monad_evidence_reports_missing_evidence` | Covered |

## Preserved Non-Goals and Deferrals

- Higher-rank polymorphism remains deferred.
- Unrestricted source type lambdas remain deferred.
- Automatic do-target inference remains deferred.
- Monad/Functor/Applicative law proving and automatic law assumptions remain deferred.
- Arbitrary associated-type-family inversion during evidence search remains deferred.
- Broad multi-parameter constructor classes remain deferred.
- Generalized runtime lowering through arbitrary user-defined Monad `return`/`bind` method bodies remains deferred; TASK-909 and TASK-910 cover explicit target resolution and return-only type-boundary evidence.

## Status Surfaces Reconciled

- `docs/spec/SPEC-067-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md`: Implemented MVP with closeout evidence and preserved deferrals.
- `docs/spec/README.md`: SPEC-067 row Implemented MVP with honest MVP/deferral summary.
- `docs/plan/PLAN-116-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md`: records TASK-911 and Phase 120 as complete after the focused blocker rerun cleared the earlier local mock-server binding failure.
- `docs/plan/PLAN-INDEX.md`: Phase 120 and TASK-911 are complete; Phase 121 untouched.
- `docs/plan/tasks/TASK-911-hkt-closeout.md`: complete with exact broad verification commands, resolved blocker evidence, and completed independent-review remediation.
- `CHANGELOG.md`: TASK-911 closeout entry under `[Unreleased]`.

## Broad Verification Commands

These are the required final closeout gates after the final code/doc change:

```bash
cargo fmt --check
git diff --check
cargo check --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo doc --workspace --no-deps 2>&1 | tee /tmp/ash-plan-116-doc.log
! grep -i '^warning:' /tmp/ash-plan-116-doc.log
```

## Current Verification Evidence

- `cargo fmt --check`: passed.
- `git diff --check`: passed.
- `cargo check --workspace`: passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: passed.
- Earlier blocker reproduced: `cargo test --workspace` was attempted first as required, but the tool session became impractical in this sandbox. The repo-owned serial fallback `TMPDIR=/home/dikini/Projects/ash/.worktrees/phase-120-hkt/target/task911-tmp CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=target/task911-verify scripts/check-rust-tests.sh --workspace` reached test execution and failed only in `ash-engine --test llm_engine_integration`, where `wiremock` could not bind a local OS port under the current sandbox: `PermissionDenied: Operation not permitted`.
- Blocker cleared by orchestrator rerun: `CODEX_NETWORK_ALLOW_LOCAL_BINDING=1 TMPDIR=/home/dikini/Projects/ash/.worktrees/phase-120-hkt/target/task911-tmp CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=target/task911-verify cargo test -p ash-engine --test llm_engine_integration -- --test-threads=1`: passed 9 tests, 0 failed.
- Fresh doc gate: `TMPDIR=/home/dikini/Projects/ash/.worktrees/phase-120-hkt/target/task911-tmp CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=target/task911-verify cargo doc --workspace --no-deps 2>&1 | tee /tmp/ash-plan-116-doc-rerun.log`: passed.
- Fresh doc warning check: `test ${PIPESTATUS[0]} -eq 0 && ! grep -i '^warning:' /tmp/ash-plan-116-doc-rerun.log`: passed; no rustdoc warning lines remained.

## Independent Review

The final independent Codex review completed and requested changes for stale pending-review wording across PLAN-116, TASK-911, this audit, and CHANGELOG. This remediation records that review as completed with findings remediated instead of pending.

The fresh exact broad workspace test rerun `TMPDIR=/home/dikini/Projects/ash/.worktrees/phase-120-hkt/target/task911-tmp CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=target/task911-verify cargo test --workspace` initially failed only in `ash-typeck --test task_757_comprehension_elaboration`, test `comprehension_rejects_missing_dictionary_target`, because the test still expected the old `no MVP dictionary` text. The test now asserts the SPEC-067 missing `Monad<K>` evidence diagnostic, including `missing Monad evidence` and `Monad<Option>`.

The next exact broad workspace test rerun with the same `TMPDIR`, `CARGO_INCREMENTAL`, and `CARGO_TARGET_DIR` failed only in `ash-typeck --test task_758_comprehension_diagnostics`, test `missing_dictionary_does_not_overclaim_future_dictionaries`, for a second stale expectation of the old dictionary wording. The test now asserts the SPEC-067 missing `Monad<K>` evidence diagnostic, including `missing Monad evidence`, `SPEC-067 Monad<K> evidence`, and `Monad<Option>`, and keeps negative coverage against stale target-inference overclaims.

The next exact broad workspace test rerun with the same `TMPDIR`, `CARGO_INCREMENTAL`, and `CARGO_TARGET_DIR` failed only in `ash-typeck --test task_906_hkt_fail_closed`, test `type_env_interface_registration_rejects_constructor_kinded_type_params`, because it still expected TypeEnv interface registration to fail closed for constructor-kinded interface binders. TASK-908 now owns higher-kinded interface registration, so the test now asserts that TypeEnv interface registration accepts TASK-908 constructor-kinded interface binders.

Post-remediation orchestrator verification completed: the final review findings were remediated, the focused stale-test reruns passed, and the final full `cargo test --workspace` passed against the remediated diff.
