# TASK-1015: Runtime-Backed Obligation Lifecycle Execution

## Status: Planned

## Description

Move obligation lifecycle synthesized cases from finite metadata control-state evaluation to runtime-backed lifecycle transition execution where lowered obligation semantics expose stable introduction, discharge, check, and rejection behavior.

## Specification Reference

- [SPEC-077](../../spec/SPEC-077-ASH-TEST-RUNNER-SYNTHESIZED-AND-SMALLWORLD-COMPLETION.md)
- [PLAN-127](../PLAN-127-DESIGN-022-023-SYNTHESIZED-SMALLWORLD-COMPLETION.md)
- [DESIGN-022](../../design/DESIGN-022-SYNTHESIZED-CONTRACT-POLICY-OBLIGATION-CASES.md)
- [DESIGN-023](../../design/DESIGN-023-SMALL-WORLD-EXPLORATION-SUBSTRATE.md)

## Requirements

1. Execute introduced, discharged, missing-discharge rejected, and double-discharge rejected lifecycle slices through stable lowered/runtime semantics.
2. Preserve finite lifecycle world snapshots in repro artifacts.
3. Defer unsupported lifecycle models and missing transition metadata.
4. Ensure pass rows require evaluated lifecycle execution.

## TDD Steps

- RED: Add failing tests for runtime-backed lifecycle transitions that currently only evaluate metadata control states.
- GREEN: Wire supported lifecycle transition execution and oracle evaluation.

## Dispatch

Use direct implementation or sub-agents according to the active controller instruction for that session.

## Verification

- Focused obligation lifecycle runner/runtime tests.
- `cargo fmt --check`
- `CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli test_runner -- --nocapture`
- `git diff --check`

## Completion Checklist

- [ ] Supported lifecycle transitions execute through runtime-backed semantics.
- [ ] Missing/unsupported lifecycle metadata defers.
- [ ] Repro artifacts include lifecycle worlds and transition traces.
- [ ] RED/GREEN evidence recorded.
