# TASK-980: Packaged dispatcher lifecycle policy

## Status: ✅ Complete

## Description

Finalize packaged dispatcher lifecycle and launcher update/remove policy beyond temp-root shim evidence.

## Specification Reference

- [SPEC-073](../../spec/SPEC-073-ASHGROVE-INSTALL-UPDATE-CLEANUP-GIT-DEPLOYMENT.md): A73-5
- [PLAN-123](../PLAN-123-SPEC-073-ASHGROVE-RELEASE-TRUST-REGISTRY-RUNTIME-SUPPORT-COMPLETION.md)

## Dependencies

- ✅ TASK-979 complete.

## Requirements

### Functional Requirements

1. Dispatcher refresh is atomic and preserves selected tool exit behavior.
2. Remove/cleanup protects running dispatcher state when executed by TASK-980-aware packaged managers after a packaged update.
3. Default switching does not rewrite project files.

### Property Requirements

1. Packaged install/update refreshes the stable dispatcher atomically.
2. Running-manager toolchain protection remains non-overridable for TASK-980-aware packaged managers after packaged updates.
3. Default switching never rewrites project-local `ash.toml`.

## TDD Steps

### Step 1: Use TASK-976 verification binding

Use `docs/plan/audits/TASK-976-ashgrove-completion-acceptance-delta.md` row `packaged-dispatcher-lifecycle` for exact files, tests, and expected RED failures.

### Step 2: Write focused RED tests

**Likely files:**
- `crates/ashgrove/src/lib.rs`
- `crates/ashgrove/tests/task_980_packaged_dispatcher_lifecycle.rs`
- `scripts/package-ash-toolchain.sh`

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
  - RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --test task_980_packaged_dispatcher_lifecycle -- --nocapture
  - git diff --check
checklist:
  - [x] Focused RED test was observed failing for the intended reason: packaged dispatcher lifecycle metadata was missing and a post-update manager toolchain could be removed by TASK-980-aware manager execution; the project-manifest rewrite regression already passed on baseline.
  - [x] Focused cleanup dry-run coverage proves `cleanup --old-toolchains --dry-run` reports the packaged dispatcher owner as `protected running manager` after packaged update and does not list it as removable.
  - [x] Focused GREEN test passes and runs non-zero tests.
  - [x] `cargo fmt --check` passes when Rust code changed.
  - [x] `git diff --check` passes.
  - [x] `cargo check --workspace` or narrower audited check passes if shared carriers/public APIs changed.
  - [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings` or narrower audited clippy gate passes if code changed.
  - [x] `CHANGELOG.md` updated if code/tooling/docs-policy/release-facing status changed.
  - [x] Independent review completed or status represented honestly.
```

## Dependencies for Next Task

TASK-980 is complete and feeds TASK-986 final closeout evidence. TASK-981 remains the next planned Phase 128 task and was not started in this checkpoint.

## Notes

This task is intentionally blocked on TASK-976 so the implementation cannot drift from the acceptance-delta audit.
