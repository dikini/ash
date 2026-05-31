# TASK-991: Ashgrove ignored lockfile source install

## Status: ✅ Complete

## Description

Fix a Phase 129 follow-up regression where `ashgrove install --from source --path "$PWD"` fails when the original Ash checkout contains an ignored `Cargo.lock`. SPEC-074 correctly excludes ignored local state from the live source-root payload and isolated build copy, but `build_source_binaries` still decides to pass `--locked` by checking `source.join("Cargo.lock")` in the original checkout instead of checking the isolated build source copy.

## Specification Reference

- [SPEC-074](../../spec/SPEC-074-ASHGROVE-SOURCE-PAYLOAD-LOCAL-STATE-IGNORE.md) §6-§10
- [TASK-989](TASK-989-ashgrove-source-payload-ignore-implementation.md)

## Requirements

### Functional Requirements

1. For live source-root installs, decide whether to pass Cargo `--locked` from the isolated build source copy after source-payload membership filtering has run.
2. If an ignored/untracked `Cargo.lock` exists only in the original checkout and is not part of the source payload, do not pass `--locked` in the isolated copy.
3. If a tracked/unignored `Cargo.lock` is included in the source payload and copied into the isolated build source, keep passing `--locked`.
4. Preserve source archive behavior and existing Phase 129 payload membership invariants.
5. Add focused regression coverage for ignored `Cargo.lock` in a git source root.

### Property Requirements

No proptest is required. Required invariant:

```text
cargo_locked_flag == isolated_build_source_has_Cargo_lock
```

## TDD Steps

1. Add a failing regression to `crates/ashgrove/tests/task_989_source_payload_ignore.rs` or a new TASK-991 test file using fake cargo. The fake cargo must record whether `--locked` was present in argv.
2. Construct a git source fixture whose `.gitignore` ignores `Cargo.lock`; write an ignored original-root `Cargo.lock`; assert git status remains clean.
3. Run source install with fake cargo and assert success plus `--locked` absent.
4. Optionally add a paired positive regression where `Cargo.lock` is tracked and `--locked` is present.
5. Change production code so `--locked` is based on the isolated build source path, not the original source path.
6. Run focused and ashgrove gates.

## Verification

```yaml
strictness: clean
commands:
  - git diff --check
  - RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove --test task_989_source_payload_ignore -- --nocapture
  - RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove task_991 -- --nocapture
  - RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove
  - RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo clippy -p ashgrove --all-targets --all-features -- -D warnings
  - cargo fmt --all --check
checklist:
  - [x] Ignored original-root `Cargo.lock` does not force `--locked` in isolated source-build copy.
  - [x] Tracked/copied `Cargo.lock` still uses `--locked`.
  - [x] User-reported install command is covered by an equivalent deterministic regression.
```

## Implementation Notes

- Added `task_991_ignored_original_root_cargo_lock_does_not_force_locked_build` and `task_991_tracked_copied_cargo_lock_keeps_locked_build` to `crates/ashgrove/tests/task_989_source_payload_ignore.rs`.
- Extended the fake Cargo fixture to record argv so the regression directly asserts `--locked` presence or absence.
- Changed `build_source_binaries` to inspect `build_source.path().join("Cargo.lock")` after `copy_for_build`, preserving the invariant `cargo_locked_flag == isolated_build_source_has_Cargo_lock`.

## TDD Evidence

- RED: `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove task_991 -- --nocapture` failed because ignored original-root `Cargo.lock` still produced fake Cargo argv containing `--locked` while the isolated copy reported `copy_absent=Cargo.lock`.
- GREEN: `RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo test -p ashgrove task_991 -- --nocapture` passed with 2 focused TASK-991 regressions after deciding `--locked` from the isolated build copy.
