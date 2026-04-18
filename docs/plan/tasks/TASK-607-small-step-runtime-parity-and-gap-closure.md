# TASK-607: Small-Step Runtime Parity and Gap Closure

## Status: ✅ Complete

## Description

Close the remaining small-step runtime gaps and add a parity corpus proving observable agreement with the big-step interpreter on the supported workflow surface.

## Specification Reference

- `docs/spec/SPEC-004-SEMANTICS.md`
- `docs/spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md`
- `docs/design/DESIGN-027-SMALL-STEP-IR-COMPRESSION.md`

## Dependencies

- 🟡 TASK-606: Workflow::Call Runtime Completion

## Requirements

1. Resolve any remaining reachable prototype/stub branches in small-step execution for supported workflow forms.
2. Add differential tests comparing big-step and small-step outcomes on the supported runtime surface.
3. Explicitly classify any still-unsupported forms as out-of-scope rather than silently degraded.
4. Verify blocked/error/terminal outcomes match or are intentionally documented when they differ.

## TDD Steps

1. Add parity tests for representative workflows covering sequencing, branching, `Act`, `ForEach`, `Maybe`, `Must`, and `Call`.
2. Reproduce any remaining divergence and classify it by layer.
3. Implement the minimal runtime fixes needed to remove the divergence.
4. Re-run the parity corpus and workspace tests.

## Verification Steps

- [ ] `cargo test -p ash-interp small_step -- --nocapture`
- [ ] `cargo test -p ash-interp parity -- --nocapture`
- [ ] `cargo test --workspace`

## Notes

Do not claim small-step production quality until parity evidence exists.