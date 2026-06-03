# TASK-1015: Typed Obligation Lifecycle Transition Execution

## Status: Complete

## Description

Move obligation lifecycle synthesized cases from finite metadata control-state equality to a narrow typed transition executor over runner metadata. This task does not claim full lowered/runtime obligation execution; it supports the explicit introduction, discharge, check, and rejection transition slice exposed in the runner snapshot.

## Specification Reference

- [SPEC-077](../../spec/SPEC-077-ASH-TEST-RUNNER-SYNTHESIZED-AND-SMALLWORLD-COMPLETION.md)
- [PLAN-127](../PLAN-127-DESIGN-022-023-SYNTHESIZED-SMALLWORLD-COMPLETION.md)
- [DESIGN-022](../../design/DESIGN-022-SYNTHESIZED-CONTRACT-POLICY-OBLIGATION-CASES.md)
- [DESIGN-023](../../design/DESIGN-023-SMALL-WORLD-EXPLORATION-SUBSTRATE.md)

## Requirements

1. Execute introduced, discharged, missing-discharge rejected, and double-discharge rejected lifecycle slices through explicit typed transition plan/trace metadata.
2. Preserve finite lifecycle world snapshots in repro artifacts.
3. Defer unsupported lifecycle models and missing transition metadata.
4. Ensure pass rows require evaluated lifecycle execution.

## TDD Steps

- RED: Add failing tests for typed lifecycle transitions that currently only evaluate metadata control states.
- GREEN: Wire supported lifecycle transition execution and oracle evaluation.

## Dispatch

Use direct implementation or sub-agents according to the active controller instruction for that session.

## Verification

- Focused obligation lifecycle runner tests.
- `cargo fmt --check`
- `CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli test_runner -- --nocapture`
- `git diff --check`

## Completion Checklist

- [x] Supported lifecycle transitions execute through the narrow typed transition substrate.
- [x] Missing/unsupported lifecycle metadata defers.
- [x] Repro artifacts include lifecycle worlds and transition traces.
- [x] RED/GREEN evidence recorded.

## Evidence

- RED: `CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli obligation_lifecycle -- --nocapture` failed before implementation because `RunnerObligationMetadata` lacked typed lifecycle transition plan/trace metadata.
- GREEN: `CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli obligation_lifecycle -- --nocapture` passed with 11 focused obligation lifecycle tests after adding typed transition execution and review-remediation coverage for unsupported models and non-lifecycle worlds.
- Implementation: `crates/ash-cli/src/test_runner/synthesized.rs` now requires the supported `finite:introduced-discharged` lifecycle model, `lifecycle_transition_plan`, typed lifecycle traces, required closeout behavior, and finite obligation-lifecycle worlds for supported obligation pass rows.
- Verification: `cargo fmt --check`; `CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli obligation -- --nocapture`; `CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli test_runner -- --nocapture`; `CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli --test test_command -- --nocapture`; `CARGO_BUILD_RUSTC_WRAPPER= cargo check --workspace`; `CARGO_BUILD_RUSTC_WRAPPER= cargo clippy -p ash-cli --all-targets -- -D warnings`; and `git diff --check` all exited 0.
