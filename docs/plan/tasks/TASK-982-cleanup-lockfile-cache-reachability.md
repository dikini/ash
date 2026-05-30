# TASK-982: Cleanup lockfile/cache reachability

## Status: ✅ Complete

## Description

Implement broader cleanup reachability across lockfiles, fetched cache, vendor metadata, and installed toolchains.

## Specification Reference

- [SPEC-073](../../spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md): A73-7
- [PLAN-123](../PLAN-123-SPEC-073-ASHGROVE-RELEASE-TRUST-REGISTRY-RUNTIME-SUPPORT-COMPLETION.md)

## Dependencies

- ✅ Depends on TASK-981 completion.

## Requirements

### Functional Requirements

1. Dry-run reports reachable and unreachable cache entries.
2. Known project locks preserve referenced checkouts and toolchains.
3. Cleanup does not crawl unregistered project roots or delete project manifests.

### Property Requirements

1. Dry-run reports lock-reachable and unreachable cache entries without deleting.
2. Known project lockfiles preserve referenced checkouts and installed toolchains.
3. Cleanup never deletes project-local `ash.toml` or `ash.lock`.

## TDD Steps

### Step 1: Use TASK-976 verification binding

Use `docs/plan/audits/TASK-976-ashgrove-completion-acceptance-delta.md` row `cleanup-lockfile-cache-reachability` for exact files, tests, and expected RED failures.

### Step 2: Write focused RED tests

**Likely files:**
- `crates/ashgrove/src/lib.rs`
- `crates/ashgrove/tests/task_982_cleanup_reachability.rs`

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
  - RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --test task_982_cleanup_reachability -- --nocapture
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

- RED: `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --test task_982_cleanup_reachability -- --nocapture` failed before production edits because cleanup reported `would remove cache .../ash/git` instead of the expected reachable checkout and unreachable stale checkout entries; 2 failed, 1 passed.
- GREEN: `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --test task_982_cleanup_reachability -- --nocapture` passed; 3 passed, 0 failed.
- Adjacent cleanup regression: `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --test task_971_remove_cleanup -- --nocapture` passed; 21 passed, 0 failed.
- Packaged dispatcher cleanup regression: `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --test task_980_packaged_dispatcher_lifecycle -- --nocapture` passed; 4 passed, 0 failed.
- Code gates: `cargo fmt --check`, `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo check -p ashgrove`, and `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo clippy -p ashgrove --all-targets --all-features -- -D warnings` passed.
- Review: independent review completed for lock/cache reachability, project-local manifest/lock preservation, no unregistered root crawling, dry-run truthfulness, TASK-980 running-manager owner protection, TASK-981 source/git fail-closed boundaries, and status-surface claims.

## Dependencies for Next Task

This task feeds TASK-986 final closeout evidence.

## Notes

This task used the TASK-976 acceptance-delta row as the implementation binding.
