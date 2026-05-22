# TASK-941 Phase 123 Closeout Evidence

Status: Complete; broad gates and independent final phase audit passed
Date: 2026-05-21
Branch: phase123-implemented-mvp
Phase: Phase 123 / PLAN-119
Specs: SPEC-069, SPEC-070

## Scope

TASK-941 reconciles the current SPEC-069/SPEC-070 status surfaces after
TASK-934 through TASK-940. This audit is the Phase 123 successor evidence for
the Phase 122 Partial MVP limitations recorded in
`docs/plan/audits/TASK-931-alpha-acceptance-matrix.md` and the preflight delta
recorded in `docs/plan/audits/TASK-933-phase123-acceptance-delta.md`.

Historical Phase 122 rows remain historical. Current SPEC-069/SPEC-070 status is
promoted to Implemented MVP because each formerly partial/deferred row now has
Phase 123 execution or artifact evidence.

## Closeout Evidence Rows

| Row | Phase 123 owner | Successor evidence | Closeout status |
| --- | --- | --- | --- |
| A69-8 | TASK-934 | `crates/ash-interp/tests/alpha_do_result_fail_execution.rs::do_result_fail_executes_as_operational_bottom_not_domain_err` proves concrete `do:Result<_, E>` execution returns operational bottom rather than implicit `Err`; `crates/ash-interp/tests/alpha_do_result_fail_execution.rs::do_result_bind_return_success_still_returns_ok_value` preserves successful bind/return execution; `crates/ash-typeck/tests/alpha_generalized_do_full_bind_lowering.rs::do_result_bind_lowers_through_monad_bind_evidence` preserves selected evidence. Focused GREEN evidence: interp suite 2 passed, typeck suite 5 passed. | Closed for Implemented MVP. |
| A69-12 | TASK-936, with TASK-935 substrate | `crates/ash-cli/tests/alpha_run_daemon_artifact_equivalence.rs::run_and_daemon_share_language_artifact_summary_but_not_host_mode` compares verifier-normalized `alpha_checked_workflow_boundary` artifact summaries across one-shot and daemon hosts while proving host-mode identity differs; `crates/ash-cli/tests/alpha_run_daemon_artifact_equivalence.rs::failed_daemon_reload_preserves_admitted_artifact_summary` proves failed daemon reload/indexing does not mutate already-admitted artifact summaries; TASK-935 builder tests prove deterministic shared artifact construction in `ash-core` and `ash-engine`. Focused GREEN evidence: artifact equivalence suite 2 passed; run report suite 3 passed; daemon control-plane suite 4 passed during remediation, later 7 passed after TASK-938. | Closed for Implemented MVP at the explicit alpha checked workflow-boundary carrier. Limitation preserved: this does not claim full workflow-body TCIR equivalence until the production lowering pipeline exposes that carrier. |
| A70-2 | TASK-937 | `crates/ash-cli/tests/alpha_admission_profile.rs` proves `ash run --admission-profile reject` rejects before body execution, emits admission-specific report/status, and leaves the side-effect sentinel absent; `crates/ash-interp/tests/invoke_runtime_dispatch.rs` preserves authority fail-closed behavior. Focused GREEN evidence: admission suite 2 passed; invoke dispatch suite 9 passed; run RuntimeKernel suite 3 passed. | Closed for Implemented MVP. |
| A70-4 | TASK-938 | `crates/ash-cli/tests/alpha_ashd_local_daemon_control_plane.rs` proves daemon start records args/config/admission-profile fields, preserves default empty admission, and rejects invalid admission without recording an active instance. Focused GREEN evidence: daemon control-plane suite 7 passed; admission regression suite 2 passed. | Closed for Implemented MVP. |
| A70-6 | TASK-939 | `crates/ash-interp/tests/task_736_capability_binding_admission.rs` proves admitted binding grants are required for provider/action execution and ungranted actions fail closed; `crates/ash-interp/tests/runtime_action_control.rs::spawned_child_without_inherited_grant_cannot_gain_provider_authority` proves child execution cannot gain provider authority from registry existence; `crates/ash-interp/tests/invoke_runtime_dispatch.rs` preserves fallback fail-closed guards. Focused GREEN evidence: capability admission suite 15 passed; runtime action control suite 18 passed; invoke dispatch suite 9 passed; child admission property target 1 passed with `PROPTEST_CASES=2048`. | Closed for Implemented MVP for capability/action grants across current runtime execution. Limitation preserved: TASK-939 records resource grant facts from existing metadata but does not add a full first-class resource operation enforcement substrate; existing process split/join resource policy remains the resource enforcement path. |
| NI-4 | TASK-939, supported by TASK-937/TASK-938 | `crates/ash-interp/tests/task_736_capability_binding_admission.rs` and `crates/ash-interp/tests/runtime_action_control.rs` prove denied or non-inherited grants do not fall back to registered providers; TASK-937/TASK-938 admission-profile paths reject before user body or daemon instance activation. | Closed for Implemented MVP for the visible capability/admission boundary; resource-operation limitation remains as stated for A70-6. |
| A70-7 | TASK-940 | `crates/ash-cli/tests/alpha_ashd_child_failure_trace.rs::daemon_child_proc_failure_is_instance_failure_not_host_failure` starts a daemon, triggers real Proc `par`/`join` child failure, observes instance status/report as workflow child failure with Proc attribution, and then issues follow-up `status` and `list` requests to prove daemon host health. Focused GREEN evidence: child-failure suite 1 passed; daemon control-plane suite 7 passed; Proc runtime suite 6 passed. | Closed for Implemented MVP. Limitation preserved: `execute=true` remains a narrow JSON-lines protocol evidence hook; ordinary public `ash daemon start` CLI remains record-only. |
| A70-8 | TASK-936, with TASK-935 substrate | Same evidence as A69-12 proves the same verified language artifact summary is exposed under `ash run` and `ash daemon` while host-mode identity, lifetime, and control plane remain distinct. | Closed for Implemented MVP at the explicit alpha checked workflow-boundary carrier; full workflow-body TCIR equivalence remains outside this MVP. |

## Current Honest Boundaries

- No remote or multi-user daemon API.
- No distributed scheduling or cluster service discovery.
- No production init-system integration.
- No arbitrary algebraic effects, effect rows, resumable continuations, or user-defined handlers.
- No full Haskell-grade inference: unrestricted type lambdas, higher-rank polymorphism, and fully free do-target inference remain outside SPEC-069 Implemented MVP.
- No JIT or native-code generation requirement.
- No claim of full workflow-body TCIR equivalence for run/daemon artifacts beyond the explicit `alpha_checked_workflow_boundary` carrier.
- No new full first-class resource operation enforcement substrate beyond the current resource grant facts and existing process split/join resource policy.

## Verification Status

TASK-941 implementation closeout reconciles the status surfaces and adds this
successor evidence map. Final closeout gates passed:

- `cargo fmt --check`
- `git diff --check`
- `RUSTC_WRAPPER= cargo check --workspace`
- `RUSTC_WRAPPER= cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `RUSTC_WRAPPER= scripts/check-rust-tests.sh --workspace`
- `RUSTC_WRAPPER= cargo doc --workspace --no-deps 2>&1 | tee /tmp/ash-phase123-doc.log && ! grep -i '^warning:' /tmp/ash-phase123-doc.log`

The final independent Codex-style phase audit found no semantic/code/test
blocker in the fail-closed RuntimeKernel admission paths and no overbroad
Implemented MVP claim beyond the status-surface reconciliation completed here.


## Post-Merge Remediation Addendum

TASK-942 and TASK-943 are part of the final Phase 123 status evidence after post-merge review reopened the closeout. TASK-942 remediated RuntimeKernel admission/report lifecycle, daemon admitted-artifact lifetime, binding-ID admission facts, empty-admission fail-closed authority, binding alias projection, and artifact-equivalence wording. TASK-943 adds the final spawned-child authority regression proving a child with no inherited admitted binding IDs cannot execute a globally admitted host binding, and reconciles SPEC-069/SPEC-070 status provenance through the post-merge remediation tasks.

Final status claims for SPEC-069/SPEC-070 Implemented MVP therefore cite this TASK-941 successor evidence together with TASK-942 and TASK-943 remediation evidence. Historical Phase 122 remains Partial MVP; Phase 123 owns the promotion after remediation.
