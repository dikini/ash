# TASK-1012: Live Runner Introspection Snapshot Production

## Status: Planned

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

## Completion Checklist

- [ ] CLI source files produce structured snapshots.
- [ ] Unsupported rows defer explicitly.
- [ ] Raw-source pass rows remain impossible.
- [ ] RED/GREEN evidence recorded.
