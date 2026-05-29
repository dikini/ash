# TASK-978: Runtime-support payload metadata

## Status: 📝 Planned

## Description

Define and verify concrete runtime-support payload metadata across source and tarball installs.

## Specification Reference

- [SPEC-073](../../spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md): A73-3 and A73-10
- [PLAN-123](../PLAN-123-SPEC-073-ASHGROVE-RELEASE-TRUST-REGISTRY-RUNTIME-SUPPORT-COMPLETION.md)

## Dependencies

- 📝 Depends on TASK-977 completion.

## Requirements

### Functional Requirements

1. Source and tarball toolchains carry equivalent runtime-support metadata.
2. Missing runtime-support payload fails closed if required.
3. Selected toolchain runtime metadata is visible to runtime artifact construction.

### Property Requirements

TASK-976 must replace this section with concrete invariants and focused RED/GREEN tests before implementation starts.

## TDD Steps

### Step 1: Wait for TASK-976 verification binding

Do not implement this task while the verification block is fail-closed. TASK-976 must name exact tests, files, and expected RED failures.

### Step 2: Write focused RED tests

**Likely files:**
- `crates/ashgrove/src/lib.rs`
- `crates/ashgrove/tests/task_969_tarball_install.rs`
- `crates/ash-engine/src/runtime_artifact.rs`

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
strictness: fail_closed_until_task_976
commands:
  - false # TASK-976 must replace this placeholder with focused non-zero verification before implementation starts.
checklist:
  - [ ] Focused RED test was observed failing for the intended reason.
  - [ ] Focused GREEN test passes and runs non-zero tests.
  - [ ] `cargo fmt --check` passes when Rust code changed.
  - [ ] `git diff --check` passes.
  - [ ] `cargo check --workspace` or narrower audited check passes if shared carriers/public APIs changed.
  - [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` or narrower audited clippy gate passes if code changed.
  - [ ] `CHANGELOG.md` updated if code/tooling/docs-policy/release-facing status changed.
  - [ ] Independent review completed or status represented honestly.
```

## Dependencies for Next Task

This task feeds TASK-986 final closeout evidence.

## Notes

This task is intentionally blocked on TASK-976 so the implementation cannot drift from the acceptance-delta audit.
