# TASK-885: Remove alternate Rust test runner from local gates

## Status: ✅ Complete

## Description

Remove the alternate Rust test runner from Ash's local verification gates because the all-target pre-push run can create excessive concurrency and memory pressure. The project gate should use plain `cargo test` through the existing wrapper, with conservative default concurrency for now.

## Specification Reference

- AGENTS.md: Policy Enforcement and task-completion changelog requirements
- TOOLS.md: local tool installation and test-runner documentation

## Dependencies

- ✅ TASK-884: Phase 116 review remediation complete

## Requirements

### Functional Requirements

1. `scripts/check-rust-tests.sh` must use plain `cargo test` only.
2. The script must preserve the same cargo argument surfaces used by the hooks.
3. The default gate must reduce local concurrency by setting `CARGO_BUILD_JOBS=1` unless the caller explicitly overrides it.
4. The default gate must pass `-- --test-threads=1` to the Rust test harness.
5. Tooling documentation must stop recommending the alternate test runner and must document the serial wrapper.
6. `CHANGELOG.md` must record the tooling change.

### Non-Goals

- Do not change Rust implementation semantics.
- Do not remove property tests, doctests, fuzz smoke checks, clippy, or format gates.
- Do not alter remote CI workflows unless they directly invoke the removed runner.

## Implementation Steps

1. Search the repository for references to the removed runner.
2. Replace the `check-rust-tests.sh` alternate-runner branch with a single serial `cargo test` invocation.
3. Remove the alternate runner from local tool installation documentation.
4. Add documentation for the serial gate wrapper.
5. Verify no operational references to the removed runner remain.

## Dispatch

```
agent: hermes
reasoning: low
max_turns: 8
toolsets: [terminal, file]
```

## Verification

```
strictness: clean
commands:
  - repository search confirms no removed-runner references remain outside historical git data
  - bash -n scripts/check-rust-tests.sh scripts/check-full-gate.sh scripts/check-pre-commit-gate.sh
  - scripts/check-rust-tests.sh --workspace --lib
  - cargo fmt --check
  - git diff --check
checklist:
  - [x] Removed-runner references eliminated
  - [x] Shell scripts parse
  - [x] Serial cargo-test wrapper passes for workspace lib tests
  - [x] Formatting clean
  - [x] Whitespace clean
```

## Verification Evidence

- Repository content search found no removed-runner references.
- `bash -n scripts/check-rust-tests.sh scripts/check-full-gate.sh scripts/check-pre-commit-gate.sh` passed.
- `scripts/check-rust-tests.sh --workspace --lib` passed using `CARGO_BUILD_JOBS=1 cargo test --workspace --lib -- --test-threads=1`; visible crate summaries included `80 passed`, `201 passed`, `2 passed`, `601 passed`, and `15 passed` with no failures.
- `cargo fmt --check` passed.
- `git diff --check` passed.

## Notes

The immediate trigger was a pre-push full-gate failure where the all-target test runner exposed concurrency/resource-sensitive failures and very high process fan-out. This task intentionally favors slower but more predictable local gates until the all-target suite is made less resource-sensitive.
