# TASK-981: Registry-scale package metadata substrate

## Status: ✅ Complete

## Description

Add registry-ready package metadata while keeping hosted registry and SemVer solving out of scope.

## Specification Reference

- [SPEC-073](../../spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md): Registry-scale metadata and A73-8/A73-9 adjacency
- [PLAN-123](../PLAN-123-SPEC-073-ASHGROVE-RELEASE-TRUST-REGISTRY-RUNTIME-SUPPORT-COMPLETION.md)

## Dependencies

- ✅ Depends on TASK-976 completion.

## Requirements

### Functional Requirements

1. Manifest and lock surfaces preserve registry-ready package metadata.
2. Vendor provenance records metadata and detects drift.
3. No hosted registry or SemVer resolution is attempted.

### Property Requirements

1. Manifest, lock, and vendor surfaces preserve registry-ready package metadata.
2. Vendor provenance records registry metadata and detects drift.
3. Hosted registry or SemVer dependency solving remains fail-closed and out of scope.

## TDD Steps

### Step 1: Use TASK-976 verification binding

Use `docs/plan/audits/TASK-976-ashgrove-completion-acceptance-delta.md` row `registry-scale-package-metadata` for exact files, tests, and expected RED failures.

### Step 2: Write focused RED tests

**Likely files:**
- `crates/ashgrove/src/lib.rs`
- `crates/ashgrove/tests/task_981_registry_metadata_substrate.rs`
- `crates/ash-engine/src/module_loader.rs`
- `crates/ash-engine/tests/task_981_registry_metadata_lock_consumers.rs`

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
  - RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --test task_981_registry_metadata_substrate -- --nocapture
  - RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ash-engine --test task_981_registry_metadata_lock_consumers -- --nocapture
  - git diff --check
checklist:
  - [x] Focused RED test was observed failing for the intended reason: manifest/lock/vendor surfaces dropped registry-ready metadata, registry-shaped dependencies produced only a generic missing-git diagnostic, ash-engine rejected registry-style lock entries with `source`/`resolved`, and ash-engine vendored lock discovery accepted poisoned `source`/legacy `git` combinations before remediation.
  - [x] Focused GREEN tests pass and run non-zero tests: ashgrove ran 5 tests; ash-engine ran 5 tests.
  - [x] `cargo fmt --check` passes when Rust code changed.
  - [x] `git diff --check` passes.
  - [x] Narrow affected checks pass for touched crates.
  - [x] Narrow affected clippy gates pass for touched crates.
  - [x] `CHANGELOG.md` updated if code/tooling/docs-policy/release-facing status changed.
  - [x] Independent review completed.
```

## Dependencies for Next Task

This task feeds TASK-982 cleanup reachability metadata inputs and TASK-986 final closeout evidence. TASK-982 has not been started.

## Notes

This task is intentionally blocked on TASK-976 so the implementation cannot drift from the acceptance-delta audit.
