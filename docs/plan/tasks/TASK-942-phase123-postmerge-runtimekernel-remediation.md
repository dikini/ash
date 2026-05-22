# TASK-942: Phase 123 Post-Merge RuntimeKernel Remediation

**Status:** Complete
**Phase:** 123 follow-up remediation
**Priority:** High
**Type:** Semantic/Substrate/Docs

## Context

A post-merge review of Phase 122 and Phase 123 on `main` found RuntimeKernel/OS-facing execution blockers after Phase 123 had been promoted to Implemented MVP. Per the Phase 123 promotion rule and verification-before-completion protocol, TASK-941 and the Phase 123 status surface are reopened until this remediation is implemented, independently reviewed, and broadly verified.

## Review Findings to Fix

1. `ash run` constructs and emits RuntimeKernel admission/report identity only after successful workflow execution. Failed execution bypasses RuntimeKernel reporting.
2. daemon `start_and_execute` admits an instance with an artifact summary, then reparses/rechecks live source during execution instead of using the pinned admitted definition/artifact source.
3. `Engine::admit_workflow` reports admitted capability strings but does not carry admitted `CapabilityBindingId`s into execution facts/context.
4. RuntimeKernel empty-admission semantics must remain fail-closed and must not use ambient full-provider `ActEnv` authority on admitted execution paths.
5. Binding alias projection must have an explicit negative/positive boundary test: either alias-only dispatch or documented provider-name dispatch.
6. SPEC-070 spec-index wording must qualify artifact equivalence at the alpha checked workflow-boundary carrier.

## Requirements

- Use TDD: add focused regressions first and verify they fail before implementation.
- Preserve Phase 122 historical Partial MVP notes; Phase 123 owns promotion.
- Reopen Phase 123/TASK-941 status while remediation is in progress.
- RuntimeKernel/admitted execution paths must be governed by explicit admission/binding projection, not ambient provider existence.
- Daemon execution must not silently run live source that differs from the admitted artifact/source summary.
- Keep existing focused Phase 123 tests passing.

## Suggested Test Targets

- `crates/ash-cli/tests/alpha_admission_profile.rs`
- `crates/ash-cli/tests/alpha_run_daemon_artifact_equivalence.rs`
- `crates/ash-engine/tests/task_715_workflow_admission_red.rs`
- `crates/ash-interp/tests/task_736_capability_binding_admission.rs`
- `crates/ash-interp/tests/invoke_runtime_dispatch.rs`

## Verification

Focused gates:

```bash
RUSTC_WRAPPER= cargo test -p ash-cli --test alpha_admission_profile -- --nocapture
RUSTC_WRAPPER= cargo test -p ash-cli --test alpha_run_daemon_artifact_equivalence -- --nocapture
RUSTC_WRAPPER= cargo test -p ash-interp --test invoke_runtime_dispatch -- --nocapture
RUSTC_WRAPPER= cargo test -p ash-interp --test task_736_capability_binding_admission -- --nocapture
RUSTC_WRAPPER= cargo test -p ash-interp --test task_741_ash_defined_capability_implementation_execution -- --nocapture
RUSTC_WRAPPER= cargo test -p ash-engine --test task_715_workflow_admission_red -- --nocapture
```

Broad gates before completion:

```bash
cargo fmt --check
git diff --check
RUSTC_WRAPPER= cargo check --workspace
RUSTC_WRAPPER= cargo clippy --workspace --all-targets --all-features -- -D warnings
RUSTC_WRAPPER= scripts/check-rust-tests.sh --workspace
RUSTC_WRAPPER= cargo doc --workspace --no-deps 2>&1 | tee /tmp/phase123-remediation-doc.log && ! grep -i '^warning:' /tmp/phase123-remediation-doc.log
```

## Completion Checklist

- [x] RED tests added and verified against pre-fix behavior.
- [x] `ash run` RuntimeKernel lifecycle/reporting is pre-execution or status/docs are narrowed honestly.
- [x] daemon execution is pinned to admitted source/artifact or fails closed on live-source drift.
- [x] workflow admission carries explicit admitted binding IDs where RuntimeKernel authority claims require them.
- [x] empty-admission RuntimeKernel paths fail closed without ambient full-provider authority.
- [x] binding alias dispatch boundary is tested and documented.
- [x] SPEC-070 README wording is scoped to the alpha checked workflow-boundary carrier.
- [x] TASK-941/PLAN-119/PLAN-INDEX/CHANGELOG status surfaces reconciled.
- [x] Focused gates pass.
- [x] Broad gates pass.
- [x] Independent Codex review finds no blockers.
