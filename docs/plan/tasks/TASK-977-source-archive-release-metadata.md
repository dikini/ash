# TASK-977: Source archive release metadata

## Status: ✅ Complete

## Description

Implement source-archive release metadata and reproducibility checks for source installs.

## Specification Reference

- [SPEC-073](../../spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md): A73-1 and part of A73-3
- [PLAN-123](../PLAN-123-SPEC-073-ASHGROVE-RELEASE-TRUST-REGISTRY-RUNTIME-SUPPORT-COMPLETION.md)

## Dependencies

- 📝 Depends on TASK-976 completion.

## Requirements

### Functional Requirements

1. Source archives record origin commit and digest.
2. Unidentified archives fail closed unless explicitly overridden.
3. Source archive metadata participates in reproducibility state.

### Property Requirements

1. Source-archive-shaped inputs without release-source metadata fail closed unless `--allow-unidentified-source` is explicit.
2. Release-source metadata records origin commit and archive digest in install records.
3. Reproducibility state is false whenever origin identity is missing or explicitly overridden.

## TDD Steps

### Step 1: Use TASK-976 verification binding

Use `docs/plan/audits/TASK-976-ashgrove-completion-acceptance-delta.md` row `source-archive-release-metadata` for exact files, tests, and expected RED failures.

### Step 2: Write focused RED tests

**Likely files:**
- `crates/ashgrove/src/lib.rs`
- `crates/ashgrove/tests/task_977_source_archive_release_metadata.rs`
- `scripts/package-ash-source-archive.sh`

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
  - RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --test task_977_source_archive_release_metadata -- --nocapture
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
