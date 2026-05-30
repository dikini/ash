# TASK-985: Ashgrove release deployment acceptance integration

## Status: ✅ Complete

## Description

Prove release/deployment flows cover completed SPEC-073 rows end-to-end before closeout.

## Specification Reference

- [SPEC-073](../../spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md): A73-1 through A73-12 integration evidence
- [PLAN-123](../PLAN-123-SPEC-073-ASHGROVE-RELEASE-TRUST-REGISTRY-RUNTIME-SUPPORT-COMPLETION.md)

## Dependencies

- 📝 Depends on TASK-977 through TASK-984 completion.

## Requirements

### Functional Requirements

1. Source archive, tarball URL, release-index, runtime metadata, trust, cleanup, and git policy compose.
2. Existing Phase 127 alpha flows remain green.
3. Acceptance matrix has concrete commands per row.

### Property Requirements

1. Source archive, runtime-support, cleanup, and selected-toolchain flows compose end to end.
2. Tarball URL, release-index, trust, and dispatcher flows compose end to end.
3. Phase 127 alpha acceptance flows remain green while Phase 128 evidence is added.

## TDD Steps

### Step 1: Use TASK-976 verification binding

Use `docs/plan/audits/TASK-976-ashgrove-completion-acceptance-delta.md` row `release-deployment-acceptance-integration` for exact files, tests, and expected RED failures.

### Step 2: Write focused RED tests

**Likely files:**
- `crates/ashgrove/tests/task_985_release_deployment_acceptance.rs`
- `crates/ash-cli/tests/phase128_release_deployment_acceptance.rs`
- `docs/plan/audits/TASK-986-spec073-completion-closeout-evidence.md`

Observe the focused tests failing for the intended reason before editing production code.

RED evidence observed on 2026-05-30:

- `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --test task_985_release_deployment_acceptance -- --nocapture` failed before test creation with `error: no test target named task_985_release_deployment_acceptance`.
- `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ash-cli --test phase128_release_deployment_acceptance -- --nocapture` failed before test creation with `error: no test target named phase128_release_deployment_acceptance`.
- After the new targets were added, the Ashgrove target failed usefully because the composed cleanup flow protected the selected source archive install as `protected default` rather than the test's initial `protected project` assertion; the CLI target failed usefully until it proved runtime-support identity via the packaged selected-toolchain dispatch boundary.

### Step 3: Implement the minimum behavior

Keep the implementation scoped to this task. Do not claim hosted registry, global install roots, or broad release-channel behavior unless this task's post-TASK-976 verification requires it.

### Step 4: Verify and reconcile status

Run focused tests, broad gates required by TASK-976, and update `CHANGELOG.md`, PLAN-123, PLAN-INDEX, SPEC-073, and audit artifacts honestly.

GREEN evidence recorded on 2026-05-30:

- `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --test task_985_release_deployment_acceptance -- --nocapture` passed: 2 passed, 0 failed.
- `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ash-cli --test phase128_release_deployment_acceptance -- --nocapture` passed: 1 passed, 0 failed.

## Dispatch

```yaml
agent: codex
reasoning: high
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: task_976_bound
commands:
  - RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --test task_985_release_deployment_acceptance -- --nocapture
  - RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ash-cli --test phase128_release_deployment_acceptance -- --nocapture
  - git diff --check
checklist:
  - [x] Focused RED test was observed failing for the intended reason.
  - [x] Focused GREEN test passes and runs non-zero tests.
  - [x] `cargo fmt --check` passes when Rust code changed.
  - [x] `git diff --check` passes.
  - [x] `cargo check --workspace` or narrower audited check passes if shared carriers/public APIs changed.
  - [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings` or narrower audited clippy gate passes if code changed.
  - [x] `CHANGELOG.md` updated if code/tooling/docs-policy/release-facing status changed.
  - [x] Independent review completed or status represented honestly.
```

## Dependencies for Next Task

This task feeds TASK-986 final closeout evidence.

## Notes

This task is intentionally blocked on TASK-976 so the implementation cannot drift from the acceptance-delta audit.

TASK-985 added integration proof and acceptance-matrix evidence only. At TASK-985 completion, SPEC-073 remained Draft and TASK-986 owned final closeout, broad status reconciliation, and any promotion.
