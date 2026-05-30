# TASK-984: Mandatory trust signing and remote git fetch policy

## Status: 📝 Planned

## Description

Implement mandatory trust/signing enforcement and remote-authenticated git fetch policy.

## Specification Reference

- [SPEC-073](../../spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md): A73-8, A73-9, A73-11, and A73-12 hardening
- [PLAN-123](../PLAN-123-SPEC-073-ASHGROVE-RELEASE-TRUST-REGISTRY-RUNTIME-SUPPORT-COMPLETION.md)

## Dependencies

- 📝 Depends on TASK-983 completion.

## Requirements

### Functional Requirements

1. Untrusted remote protocols fail closed.
2. Signature or attestation failures fail closed before publish or fetch use.
3. Remote-authenticated git policy records no secrets in lockfiles.

### Property Requirements

1. Untrusted remote protocols fail before fetch or publish use.
2. Required signature or attestation failures fail before publish or lock use.
3. Authenticated remote policy records no credentials or secrets in lockfiles.

## TDD Steps

### Step 1: Use TASK-976 verification binding

Use `docs/plan/audits/TASK-976-ashgrove-completion-acceptance-delta.md` rows `mandatory-trust-signing-enforcement` and `remote-authenticated-git-fetch` for exact files, tests, and expected RED failures.

### Step 2: Write focused RED tests

**Likely files:**
- `crates/ashgrove/src/lib.rs`
- `crates/ashgrove/tests/task_984_trust_signing_remote_git_policy.rs`
- `scripts/package-ash-toolchain.sh`
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
  - RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --test task_984_trust_signing_remote_git_policy -- --nocapture
  - RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ash-cli --test phase127_vendored_dependency_resolution -- --nocapture
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

This task is intentionally blocked on TASK-976 so the implementation cannot drift from the acceptance-delta audit. TASK-984 must cover source/tarball install and update publish paths as well as git fetch/lock paths, and must amend SPEC-073's A73-11 wording before claiming mandatory trust/signing enforcement complete.
