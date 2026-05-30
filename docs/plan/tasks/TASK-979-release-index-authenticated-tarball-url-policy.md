# TASK-979: Release-index authenticated tarball URL policy

## Status: 📝 Planned

## Description

Add authenticated tarball URL recording and release-index trust policy without best-effort network lookup.

## Specification Reference

- [SPEC-073](../../spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md): A73-2 and A73-4
- [PLAN-123](../PLAN-123-SPEC-073-ASHGROVE-RELEASE-TRUST-REGISTRY-RUNTIME-SUPPORT-COMPLETION.md)

## Dependencies

- 📝 Depends on TASK-978 completion.

## Requirements

### Functional Requirements

1. URL installs require digest or signed release-index evidence.
2. Digest mismatch fails before publish.
3. URL provenance is recorded in install/update metadata.

### Property Requirements

1. URL installs require an expected digest or signed release-index evidence.
2. Digest mismatches fail before any toolchain publish.
3. Authenticated URL provenance is recorded in install/update metadata.

## TDD Steps

### Step 1: Use TASK-976 verification binding

Use `docs/plan/audits/TASK-976-ashgrove-completion-acceptance-delta.md` row `authenticated-tarball-url-release-index` for exact files, tests, and expected RED failures.

### Step 2: Write focused RED tests

**Likely files:**
- `crates/ashgrove/src/lib.rs`
- `crates/ashgrove/tests/task_969_tarball_install.rs`
- `crates/ashgrove/tests/task_970_selectors.rs`

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
  - RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --test task_979_release_index_tarball_url_policy -- --nocapture
  - git diff --check
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
