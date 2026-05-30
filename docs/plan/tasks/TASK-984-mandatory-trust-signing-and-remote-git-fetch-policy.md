# TASK-984: Mandatory trust signing and remote git fetch policy

## Status: ✅ Complete

## Description

Implement mandatory trust/signing enforcement and remote-authenticated git fetch policy.

## Specification Reference

- [SPEC-073](../../spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md): A73-8, A73-9, A73-11, and A73-12 hardening
- [PLAN-123](../PLAN-123-SPEC-073-ASHGROVE-RELEASE-TRUST-REGISTRY-RUNTIME-SUPPORT-COMPLETION.md)

## Dependencies

- ✅ Depends on TASK-983 completion.

## Requirements

### Functional Requirements

1. Untrusted remote protocols fail closed.
2. Signature or attestation evidence failures fail closed before publish or fetch use.
3. Remote-authenticated git policy records no secrets in lockfiles.
4. URL install/update continues to require explicit digest evidence; release-index signing fails closed until a later resolver binds signed entries to toolchain id, tarball URL, and digest.

### Property Requirements

1. Untrusted remote protocols fail before fetch or publish use.
2. Required tarball sidecar signature evidence, source-archive attestation evidence, and lock signature evidence failures fail before publish or lock use.
3. Authenticated remote policy records no credentials or secrets in lockfiles; HTTPS credentials are redacted and credential-bearing `ssh://` URLs are rejected before serialization.

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

- RED: `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --test task_984_trust_signing_remote_git_policy -- --nocapture` failed with 0 passed / 7 failed before production edits. Failures showed signature and attestation metadata were accepted, untrusted git reached `git clone`, credential-bearing remotes could not produce redacted lockfiles, lock signing failed as drift rather than signature policy, and `--release-index` was absent.
- Review remediation RED: focused TASK-984 and ash-cli regressions failed before remediation for credential-bearing `ssh://` lock serialization, missing source-archive attestation evidence, required tarball sidecar signature evidence, and ash-engine lock signature bypass.
- GREEN: `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --test task_984_trust_signing_remote_git_policy -- --nocapture` passed with 10 passed / 0 failed.
- Regression: `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ash-cli --test phase127_vendored_dependency_resolution -- --nocapture` passed with 25 passed / 0 failed.
- Adjacent ashgrove regressions passed for TASK-979, TASK-981, and TASK-983.

## Dependencies for Next Task

This task feeds TASK-985 release/deployment acceptance integration and TASK-986 final closeout evidence.

## Notes

This task is intentionally blocked on TASK-976 so the implementation cannot drift from the acceptance-delta audit. TASK-984 must cover source/tarball install and update publish paths as well as git fetch/lock paths, and must amend SPEC-073's A73-11 wording before claiming mandatory trust/signing enforcement complete.
