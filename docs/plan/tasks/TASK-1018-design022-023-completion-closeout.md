# TASK-1018: DESIGN-022/023 Completion Closeout

## Status: Planned

## Description

Close the DESIGN-022 and DESIGN-023 completion phase after implementation tasks land, run broad verification, reconcile docs/status surfaces, and promote design acceptance only when full completion criteria are met.

## Specification Reference

- [SPEC-077](../../spec/SPEC-077-ASH-TEST-RUNNER-SYNTHESIZED-AND-SMALLWORLD-COMPLETION.md)
- [PLAN-127](../PLAN-127-DESIGN-022-023-SYNTHESIZED-SMALLWORLD-COMPLETION.md)
- [DESIGN-022](../../design/DESIGN-022-SYNTHESIZED-CONTRACT-POLICY-OBLIGATION-CASES.md)
- [DESIGN-023](../../design/DESIGN-023-SMALL-WORLD-EXPLORATION-SUBSTRATE.md)

## Requirements

1. Reconcile PLAN-127, PLAN-INDEX, DESIGN-022, DESIGN-023, task files, reference docs, and CHANGELOG.
2. Record focused and broad verification evidence.
3. Ensure no stale docs overclaim Phase 76B or completion behavior.
4. Promote DESIGN-022 and DESIGN-023 only if live snapshots, synthesized execution, and small-world execution meet SPEC-077 acceptance.

## TDD Steps

- RED: Identify stale docs/status rows and missing verification evidence after implementation tasks land.
- GREEN: Update docs/status and run broad verification until claims match reality.

## Dispatch

Use direct implementation or sub-agents according to the active controller instruction for that session.

## Verification

- `cargo fmt --check`
- `CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli test_runner -- --nocapture`
- `CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli --test test_command -- --nocapture`
- `CARGO_BUILD_RUSTC_WRAPPER= cargo clippy -p ash-cli --all-targets -- -D warnings`
- `CARGO_BUILD_RUSTC_WRAPPER= cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `CARGO_BUILD_RUSTC_WRAPPER= cargo test --workspace`
- `git diff --check`

## Completion Checklist

- [ ] DESIGN-022 and DESIGN-023 completion criteria verified.
- [ ] Docs/status surfaces reconciled.
- [ ] CHANGELOG updated.
- [ ] Broad verification recorded.
