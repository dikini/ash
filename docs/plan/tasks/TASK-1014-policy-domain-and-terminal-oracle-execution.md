# TASK-1014: Policy Domain and Terminal Oracle Execution

## Status: Planned

## Description

Execute synthesized policy cases over checked bounded domains and stable terminal oracles beyond the Phase 76B exact metadata equality slice.

## Specification Reference

- [SPEC-077](../../spec/SPEC-077-ASH-TEST-RUNNER-SYNTHESIZED-AND-SMALLWORLD-COMPLETION.md)
- [PLAN-127](../PLAN-127-DESIGN-022-023-SYNTHESIZED-SMALLWORLD-COMPLETION.md)
- [DESIGN-022](../../design/DESIGN-022-SYNTHESIZED-CONTRACT-POLICY-OBLIGATION-CASES.md)

## Requirements

1. Materialize finite policy input domains from checked policy metadata.
2. Execute policy targets to produce terminal outcomes.
3. Support allow/deny first, then approval/transform when metadata exposes stable oracles.
4. Preserve required authority metadata and defer missing authority setup.
5. Emit repro artifacts for each executed policy case.

## TDD Steps

- RED: Add failing policy execution tests that currently rely on terminal metadata instead of evaluated policy targets.
- GREEN: Execute supported policy domains and terminal oracles.

## Dispatch

Use direct implementation or sub-agents according to the active controller instruction for that session.

## Verification

- Focused policy synthesized runner tests.
- `cargo fmt --check`
- `CARGO_BUILD_RUSTC_WRAPPER= cargo test -p ash-cli test_runner -- --nocapture`
- `git diff --check`

## Completion Checklist

- [ ] Policy domains materialize from checked metadata.
- [ ] Policy terminal outcomes are evaluated.
- [ ] Authority gaps defer.
- [ ] RED/GREEN evidence recorded.
