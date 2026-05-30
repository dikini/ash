# TASK-978: Runtime-support payload metadata

## Status: ✅ Complete

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

1. Source and tarball install paths produce equivalent required runtime-support metadata.
2. A tarball missing required runtime-support metadata fails before publish.
3. Runtime artifact construction records the selected toolchain runtime-support identity.

## TDD Steps

### Step 1: Use TASK-976 verification binding

Use `docs/plan/audits/TASK-976-ashgrove-completion-acceptance-delta.md` row `runtime-support-payload-metadata` for exact files, tests, and expected RED failures.

### Step 2: Write focused RED tests

**Likely files:**
- `crates/ashgrove/src/lib.rs`
- `crates/ashgrove/tests/task_978_runtime_support_payload_metadata.rs`
- `crates/ash-engine/src/runtime_artifact.rs`
- `crates/ash-engine/tests/task_978_runtime_support_payload.rs`
- `crates/ash-cli/src/commands/run.rs`
- `crates/ash-cli/src/commands/daemon.rs`

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
  - RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --test task_978_runtime_support_payload_metadata -- --nocapture
  - RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ash-engine --test task_978_runtime_support_payload -- --nocapture
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

2026-05-30 completion slice: source-root and tarball toolchains now carry required `[runtime_support]` manifest metadata with identity `ash-runtime-support:<version>` and path `lib/ash/std/src/runtime`; source and tarball installs validate the metadata and payload directory before publish; launcher dispatch passes `ASH_RUNTIME_SUPPORT_IDENTITY` to selected `ash`; and runtime artifact construction incorporates the selected runtime-support identity into artifact/check identity.
