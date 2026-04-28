# TASK-766: Reference Example Policy and Closeout

## Status: 📝 Planned

## Description

Decide and apply a corpus policy for large historical/reference examples that use unsupported syntax. Either canonicalize them into current checkable examples or mark/move them as reference-only with an explicit exclusion rule, then close Phase 107 with final corpus verification.

## Specification Reference

- [PLAN-103](../PLAN-103-STDLIB-EXAMPLE-CORPUS-REPAIR.md)
- [SPEC-005](../../spec/SPEC-005-CLI.md)

## Dependencies

- 📝 TASK-760: CLI Corpus Baseline Harness
- 📝 TASK-761: Stdlib Multiline Imports and Module Roots
- 📝 TASK-762: Stdlib Workflow Exports and Relative Imports
- 📝 TASK-763: Runtime Args and LLM Loading Imports
- 📝 TASK-764: Parser Comments and Diagnostics
- 📝 TASK-765: Canonicalize Small Examples

## Requirements

1. Classify large legacy files as either conformance examples or reference/design sketches.
2. For conformance examples, rewrite to current syntax and add to corpus expected-pass list.
3. For reference-only sketches, add an explicit marker or move to a documented reference location excluded by the corpus harness.
4. Include root examples and `examples/workflows/40*` in the decision.
5. Produce a final std/example corpus report with exact pass/fail or pass/reference-only counts.
6. Update PLAN-103, PLAN-INDEX, task files, and CHANGELOG for phase closeout.

## Candidate Files

- `examples/03-policies/01-role-based.ash`
- `examples/03-policies/02-time-based.ash`
- `examples/04-real-world/code-review.ash`
- `examples/04-real-world/customer-support.ash`
- `examples/simple_workflow.ash`
- `examples/support_ticket.ash`
- `examples/multi_agent_research.ash`
- `examples/workflows/40_tdd_workflow.ash`
- `examples/workflows/40a_tdd_concrete_example.ash`

## TDD Steps

1. Extend the corpus harness to require every example file be either expected-pass or explicitly reference-only.
2. Watch it fail for unclassified files.
3. Canonicalize or mark/move each candidate.
4. Re-run corpus harness and all individual affected checks.
5. Complete docs/status/changelog reconciliation.

## Verification Checklist

- [ ] Every `examples/**/*.ash` file is classified by the corpus harness.
- [ ] All conformance examples pass `ash check`.
- [ ] Reference-only examples are clearly labeled and excluded by documented policy.
- [ ] All `std/src/**/*.ash` files pass `ash check`, or any remaining exceptions are explicitly justified and accepted by the plan.
- [ ] `cargo test --workspace` passes.
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes.
- [ ] `cargo fmt --check` passes.
- [ ] `cargo doc --workspace --no-deps` passes.
- [ ] Independent phase audit completed and blockers addressed.
