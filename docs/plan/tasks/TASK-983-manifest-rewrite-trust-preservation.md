# TASK-983: Manifest rewrite trust preservation

## Status: ✅ Complete

## Description

Preserve manifest and lockfile trust metadata during read-modify-write operations.

## Specification Reference

- [SPEC-073](../../spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md): A73-11 preservation boundary
- [PLAN-123](../PLAN-123-SPEC-073-ASHGROVE-RELEASE-TRUST-REGISTRY-RUNTIME-SUPPORT-COMPLETION.md)

## Dependencies

- ✅ Depends on TASK-981 completion.

## Requirements

### Functional Requirements

1. Nested trust and signing fields survive lock rewrites.
2. Manifest rewrites preserve unknown trust metadata.
3. Diagnostics distinguish preservation from enforcement.

### Property Requirements

1. Nested trust and signing tables survive lockfile rewrites.
2. Manifest rewrites preserve unknown trust metadata without interpreting it.
3. Diagnostics distinguish trust preservation from trust enforcement.

## TDD Steps

### Step 1: Use TASK-976 verification binding

Use `docs/plan/audits/TASK-976-ashgrove-completion-acceptance-delta.md` row `manifest-rewrite-trust-preservation` for exact files, tests, and expected RED failures.

### Step 2: Write focused RED tests

**Likely files:**
- `crates/ashgrove/src/lib.rs`
- `crates/ashgrove/tests/task_983_manifest_rewrite_trust_preservation.rs`

Observe the focused tests failing for the intended reason before editing production code.

### Step 3: Implement the minimum behavior

Keep the implementation scoped to this task. Do not claim hosted registry, global install roots, or broad release-channel behavior unless this task's post-TASK-976 verification requires it.

### Step 4: Verify and reconcile status

Run focused tests, broad gates required by TASK-976, and update `CHANGELOG.md`, PLAN-123, PLAN-INDEX, SPEC-073, and audit artifacts honestly.

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
  - RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --test task_983_manifest_rewrite_trust_preservation -- --nocapture
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

## Evidence

- RED: `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --test task_983_manifest_rewrite_trust_preservation -- --nocapture` failed with unresolved import for the missing `rewrite_project_manifest_preserving_trust_metadata` helper.
- GREEN: `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --test task_983_manifest_rewrite_trust_preservation -- --nocapture` passed 3 tests.
- Regression/quality gates: TASK-967 and TASK-972 adjacent tests, `cargo fmt --check`, `cargo check -p ashgrove`, `cargo clippy -p ashgrove --all-targets --all-features -- -D warnings`, `git diff --check`, and local pre-commit passed for the TASK-983 slice.
- Review: independent review checked nested lock trust/signing preservation, unknown manifest trust preservation, preservation-only diagnostics, TASK-981 registry/source metadata compatibility, TASK-982 cleanup reachability/status compatibility, and status-surface wording.

## Dependencies for Next Task

This task feeds TASK-986 final closeout evidence.

## Notes

This task is intentionally blocked on TASK-976 so the implementation cannot drift from the acceptance-delta audit. The live Phase 127 code only rewrites `ash.lock`; if no `ash.toml` write path exists when this task starts, TASK-983 must add an explicit manifest rewrite helper or keep manifest rewrite preservation open.
