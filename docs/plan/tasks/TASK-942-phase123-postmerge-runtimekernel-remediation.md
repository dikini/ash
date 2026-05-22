# TASK-942: Phase 123 Post-Merge RuntimeKernel Remediation

**Status:** Complete
**Phase:** 123 follow-up remediation
**Priority:** High
**Type:** Semantic/Substrate/Docs

## Context

A post-merge review of Phase 122 and Phase 123 on `main` found RuntimeKernel/OS-facing execution blockers after Phase 123 had been promoted to Implemented MVP. Per the Phase 123 promotion rule and verification-before-completion protocol, TASK-941 and the Phase 123 status surface are reopened until this remediation is implemented, independently reviewed, and broadly verified.

Historical narrowing: this task is retained as the first Phase 123
post-merge remediation slice, not as the final Phase 123 remediation record.
Later review found additional gaps after TASK-942 closeout. TASK-943 owns the
spawned-child empty-admission authority regression, TASK-944 owns the daemon
second-read source/config remediation, and TASK-945 owns the final
local-control, binding-alias, verifier, one-shot report, and status evidence.
Use `docs/plan/audits/TASK-941-phase123-closeout-evidence.md`,
`TASK-943-phase123-followup-child-admission-and-status-drift.md`,
`TASK-944-phase123-daemon-admitted-source-config-remediation.md`, and
`TASK-945-phase123-daemon-local-control-security-remediation.md` for the final
Implemented MVP remediation chain.

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
- [x] `ash run` RuntimeKernel lifecycle/reporting was remediated for the
      TASK-942 slice, with the later admission lifecycle caveat narrowed again
      by TASK-945: admission-profile rejection happens before user code and
      before verified artifact reporting; verified artifact reports are emitted
      only after parse/check/artifact construction succeeds.
- [x] daemon admitted-artifact drift checks were added for TASK-942, but later
      review found that this did not fully pin daemon execution because
      `start-execute` still performed a second workflow source read and daemon
      non-default `config_id` values were over-accepted. TASK-944 is the
      owning remediation for executing from already-read/hash-checked source
      bytes and rejecting non-default daemon config IDs before instance
      recording.
- [x] workflow admission carries explicit admitted binding IDs where
      RuntimeKernel authority claims require them; TASK-945 later tightened
      host-provider grants so authority remains scoped per admitted binding
      id/name rather than unioned by backing provider.
- [x] empty-admission RuntimeKernel paths were narrowed for TASK-942, with
      TASK-943 adding the spawned-child regression that proves empty inherited
      authority does not repopulate from globally admitted runtime bindings.
- [x] binding alias dispatch boundary was tested and documented for TASK-942;
      TASK-945 is the final authority-projection evidence for alias/binding
      grant non-union.
- [x] SPEC-070 README wording is scoped to the alpha checked workflow-boundary
      carrier.
- [x] TASK-941/PLAN-119/PLAN-INDEX/CHANGELOG status surfaces were reconciled
      for the original TASK-942 slice, then superseded by TASK-943 through
      TASK-945 status/evidence reconciliation.
- [x] Focused gates pass.
- [x] Broad gates were recorded at original TASK-942 closeout, but final
      Phase 123 evidence must use the later TASK-944 broad serial
      workspace/rustdoc evidence plus TASK-945 final focused and broad evidence
      recorded in `docs/plan/audits/TASK-941-phase123-closeout-evidence.md`.
- [x] The original no-blocker Codex review was historical only; later review
      produced TASK-943, TASK-944, and TASK-945. Final no-blocker status is not
      claimed from TASK-942 alone.
