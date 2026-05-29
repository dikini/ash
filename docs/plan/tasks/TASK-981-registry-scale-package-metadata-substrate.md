# TASK-981: Registry-scale package metadata substrate

## Status: 📝 Planned

## Description

Add registry-ready package metadata while keeping hosted registry and SemVer solving out of scope.

## Specification Reference

- [SPEC-073](../../spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md): Registry-scale metadata and A73-8/A73-9 adjacency
- [PLAN-123](../PLAN-123-SPEC-073-ASHGROVE-RELEASE-TRUST-REGISTRY-RUNTIME-SUPPORT-COMPLETION.md)

## Dependencies

- 📝 Depends on TASK-976 completion.

## Requirements

### Functional Requirements

1. Manifest and lock surfaces preserve registry-ready package metadata.
2. Vendor provenance records metadata and detects drift.
3. No hosted registry or SemVer resolution is attempted.

### Property Requirements

TASK-976 must replace this section with concrete invariants and focused RED/GREEN tests before implementation starts.

## TDD Steps

### Step 1: Wait for TASK-976 verification binding

Do not implement this task while the verification block is fail-closed. TASK-976 must name exact tests, files, and expected RED failures.

### Step 2: Write focused RED tests

**Likely files:**
- `crates/ashgrove/src/lib.rs`
- `crates/ashgrove/tests/task_972_manifest_lock_git.rs`
- `crates/ashgrove/tests/task_973_vendor.rs`
- `crates/ash-engine/src/module_loader.rs`

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
