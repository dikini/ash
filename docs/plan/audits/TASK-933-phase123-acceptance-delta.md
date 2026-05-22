# TASK-933 Phase 123 Acceptance Delta

Status: Complete
Date: 2026-05-21
Branch: phase123-implemented-mvp
Phase: Phase 123 / PLAN-119
Specs: SPEC-069, SPEC-070

## Scope

TASK-933 is a pre-implementation audit gate for Phase 123. It preserves Phase
122 as a closed Partial MVP and assigns every deferred SPEC-069/SPEC-070 row to
exactly one Phase 123 follow-on task. Rows below name the planned RED test,
expected RED failure mode, implementation seam, and closeout owner.

## Acceptance Delta Rows

| Row | Phase 122 limitation | Owner | Planned RED test file / test name | Expected RED failure mode | Primary implementation seams |
| --- | --- | --- | --- | --- | --- |
| A69-8 | `fail` inside `do:Result<_, E>` has type/carrier evidence but no execution proof that it remains operational bottom rather than implicit `Err`. | TASK-934 | `crates/ash-interp/tests/alpha_do_result_fail_execution.rs::do_result_fail_executes_as_operational_bottom_not_domain_err`; paired typeck regression `crates/ash-typeck/tests/alpha_generalized_do_full_bind_lowering.rs::do_result_bind_lowers_through_monad_bind_evidence` | New interp test fails because no concrete `do:Result` execution path demonstrates operational bottom distinct from domain `Err`. | `crates/ash-interp/src/eval.rs`; `crates/ash-interp/src/execute.rs`; `crates/ash-typeck/src/do_target.rs` if selected evidence is lost. |
| A69-12 | `ash run` and daemon host modes share identity/provenance carriers, but bytecode-level artifact equivalence is not proven. | TASK-936 | `crates/ash-cli/tests/alpha_run_daemon_artifact_equivalence.rs::run_and_daemon_share_verified_language_artifact_summary` | Test fails because one-shot and daemon command paths do not expose a shared verifier-normalized artifact summary. | TASK-935 builder; `crates/ash-cli/src/commands/run.rs`; `crates/ash-cli/src/commands/daemon.rs`; `crates/ash-engine/src/runtime_artifact.rs`. |
| A70-2 | `ash run` reports local parse/check/body failures and authority fallback failures, but lacks admission-profile rejection before body execution. | TASK-937 | `crates/ash-cli/tests/alpha_admission_profile.rs::ash_run_rejects_admission_profile_before_body_side_effects` | Sentinel side-effect test fails because no alpha admission profile can reject before user code executes. | `crates/ash-core/src/runtime_kernel.rs`; `crates/ash-cli/src/commands/run.rs`; `crates/ash-engine/src/lib.rs`; `crates/ash-interp/src/runtime_state.rs`. |
| A70-4 | Daemon start records an empty-admission MVP instance only; args/config/admission-profile fields remain deferred. | TASK-938 | `crates/ash-cli/tests/alpha_ashd_local_daemon_control_plane.rs::ashd_start_records_args_config_and_admission_profile` | Test fails because daemon start accepts only workflow name and records no args/config/admission-profile payload. | `crates/ash-cli/src/commands/daemon.rs`; `crates/ash-core/src/runtime_kernel.rs`. |
| A70-6 | Provider existence is not authority for fallback invoke, but broader policy-profile grants are not modeled through RuntimeKernel admission. | TASK-939 | `crates/ash-interp/tests/alpha_policy_profile_grants.rs::policy_profile_grants_required_provider_action_before_execution` | Test fails because grants are not evaluated as a RuntimeKernel policy profile before execution. | `crates/ash-core/src/runtime_kernel.rs`; `crates/ash-interp/src/runtime_state.rs`; `crates/ash-interp/src/context.rs`; `crates/ash-interp/src/eval.rs`; `crates/ash-interp/src/execute.rs`. |
| NI-4 | Bytecode/runtime work must not bypass capability/admission semantics; Phase 122 covers invoke fallback only, not broader policy-profile admission. | TASK-939 | `crates/ash-interp/tests/alpha_policy_profile_grants.rs::policy_profile_denial_does_not_fall_back_to_registered_provider` | Test fails if denial can still invoke a registered provider through host/provider fallback. | Same as A70-6 plus existing `crates/ash-interp/tests/invoke_runtime_dispatch.rs` regressions. |
| A70-7 | Child process failure observation is covered by process identity carriers but not daemon-hosted execution trace semantics. | TASK-940 | `crates/ash-cli/tests/alpha_ashd_child_failure_trace.rs::daemon_child_failure_marks_workflow_instance_failed_without_crashing_host` | Test fails because daemon start is currently record-only and exposes no child failure trace/status. | `crates/ash-cli/src/commands/daemon.rs`; `crates/ash-engine/src/lib.rs`; `crates/ash-interp/src/execute.rs`; `crates/ash-interp/src/runtime_state.rs`. |
| A70-8 | Same artifact under `ash run` and `ash daemon` has matching identity carriers, but language-level bytecode/provenance equivalence is not proven. | TASK-936 | `crates/ash-cli/tests/alpha_run_daemon_artifact_equivalence.rs::host_mode_identity_differs_but_language_artifact_matches` | Test fails because host-mode-specific paths do not compare verifier-normalized bytecode/provenance summaries. | TASK-935 builder; `crates/ash-cli/src/commands/run.rs`; `crates/ash-cli/src/commands/daemon.rs`. |

## Owner Invariant

Every deferred row above has exactly one owning follow-on task in the acceptance delta table:

- TASK-934 owns A69-8.
- TASK-936 owns A69-12 and A70-8 because both require the shared TASK-935 artifact builder.
- TASK-937 owns A70-2.
- TASK-938 owns A70-4.
- TASK-939 owns A70-6 and NI-4.
- TASK-940 owns A70-7.

`Specification Reference` sections in prerequisite tasks may cite rows they support without owning them. The exact-owner verifier therefore reads this artifact's owner table as the canonical ownership surface and separately checks that each owning task file cites its row. TASK-935 is a substrate prerequisite for TASK-936 and TASK-937, not the direct acceptance-row owner for A69-12/A70-8. TASK-941 owns the original final status promotion, broad gates, and Codex phase audit after TASK-934 through TASK-940 are complete; current final Phase 123 status also depends on the later TASK-942 through TASK-945 post-merge remediation record.

## Preflight Findings

1. Phase 123 task files exist in `docs/plan/tasks/` and are implementation-ready after this audit is reconciled.
2. The initial TASK-933 verification recipe checked only row presence; it must enforce exactly-one ownership using this artifact.
3. TASK-935 verification requires both core and engine builder tests, so the builder task must create both targets or narrow the task file before completion.
4. Several later verification commands intentionally target missing tests; those are valid RED targets only after the test files are created and fail for the intended behavioral reason.
5. Phase 123 must preserve historical Phase 122 Partial MVP status. Promotion to Implemented MVP belongs only to TASK-941 after broad gates and status-surface reconciliation; the implementation worker records final independent Codex phase audit status separately in the TASK-941 closeout evidence.
