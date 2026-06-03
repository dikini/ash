# TASK-1012: Live Runner Introspection Snapshot Production

## Status: Complete

## Description

Produce `RunnerIntrospectionSnapshot` values from ordinary `ash test` CLI source files and suite roots after parse/check/lowering, replacing raw-source scans as the executable synthesized-test input path.

## Specification Reference

- [SPEC-077](../../spec/SPEC-077-ASH-TEST-RUNNER-SYNTHESIZED-AND-SMALLWORLD-COMPLETION.md)
- [PLAN-127](../PLAN-127-DESIGN-022-023-SYNTHESIZED-SMALLWORLD-COMPLETION.md)
- [DESIGN-022](../../design/DESIGN-022-SYNTHESIZED-CONTRACT-POLICY-OBLIGATION-CASES.md)
- [DESIGN-023](../../design/DESIGN-023-SMALL-WORLD-EXPLORATION-SUBSTRATE.md)

## Requirements

1. Build checked/lowered snapshots for ordinary CLI files before synthesized execution.
2. Preserve source artifact identity, check summary identity, schema version, supported metadata, and unsupported rows.
3. Keep raw-source scans as compatibility discovery only; they must emit deferred skip rows and never pass.
4. Add JSON/human evidence that CLI-source synthesized execution uses structured snapshots when available.

## TDD Steps

- RED: Add CLI/runner tests proving ordinary source files cannot yet produce structured snapshots and fall back to deferred raw-source rows.
- GREEN: Wire checked/lowered summary production into `ash test` synthesized execution and preserve deferred rows for unsupported metadata.

## Dispatch

Use direct implementation or sub-agents according to the active controller instruction for that session.

## Verification

- Focused `ash-cli` runner tests for snapshot production.
- `cargo fmt --check`
- `CARGO_BUILD_RUSTC_WRAPPER= cargo check --workspace`
- `git diff --check`

## Evidence

### RED

- `CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli --test test_command live_checked_snapshot -- --nocapture`
  - Failed before implementation on `only_synthesized_function_contract_module_uses_live_checked_snapshot`.
  - The CLI emitted `check_summary_id: "raw-source-fallback:no-lowered-summary"` and `oracle_snapshot.fallback: "raw_source_pattern"` for `fn bounded(n: Int) -> Int requires: n >= 0 { n }`.
- `CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli --test test_command raw_fallback_is_applied_per_file_in_mixed_live_snapshot_suite -- --nocapture`
  - Failed before review-blocker remediation on `raw_fallback_is_applied_per_file_in_mixed_live_snapshot_suite`.
  - The mixed suite emitted only the live checked snapshot row for `checked_fn_contract_target.ash`; the raw-source fallback row for `raw_fallback_only.ash` was missing.

### GREEN

- `CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli --test test_command live_checked_snapshot -- --nocapture`
  - Passed: 2 passed, 0 failed.
- `CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli --test test_command raw_fallback -- --nocapture`
  - Passed: 1 passed, 0 failed.
- `CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli synthesized -- --nocapture`
  - Passed: synthesized-focused `ash-cli` tests passed, including 29 library tests and 10 `test_command` synthesized tests.
- `CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli --test test_command -- --nocapture`
  - Passed: 26 passed, 0 failed.
- `cargo fmt --check`
  - Passed.
- `CARGO_BUILD_RUSTC_WRAPPER= cargo check --workspace`
  - Passed.
- `git diff --check`
  - Passed.

## Completion Checklist

- [x] CLI source files produce structured snapshots.
- [x] Unsupported rows defer explicitly.
- [x] Raw-source pass rows remain impossible.
- [x] RED/GREEN evidence recorded.
