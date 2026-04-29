# TASK-766: Reference Example Policy and Closeout

## Status: ✅ Complete

## Description

Decide and apply a corpus policy for large historical/reference examples that use unsupported syntax. Either canonicalize them into current checkable examples or mark/move them as reference-only with an explicit exclusion rule, then close Phase 107 with final corpus verification.

## Specification Reference

- [PLAN-103](../PLAN-103-STDLIB-EXAMPLE-CORPUS-REPAIR.md)
- [SPEC-005](../../spec/SPEC-005-CLI.md)

## Dependencies

- ✅ TASK-760: CLI Corpus Baseline Harness
- ✅ TASK-761: Stdlib Multiline Imports and Module Roots
- ✅ TASK-762: Stdlib Workflow Exports and Relative Imports
- ✅ TASK-763: Runtime Args and LLM Loading Imports
- ✅ TASK-764: Parser Comments and Diagnostics
- ✅ TASK-765: Canonicalize Small Examples

## Requirements

1. Classify large legacy files as either conformance examples or reference/design sketches.
2. For conformance examples, rewrite to current syntax and add to corpus expected-pass list.
3. For reference-only sketches, add an explicit marker or move to a documented reference location excluded by the corpus harness.
4. Include root examples and `examples/workflows/40*` in the decision.
5. Produce a final std/example corpus report with exact pass/fail or pass/reference-only counts.
6. Update PLAN-103, PLAN-INDEX, task files, and CHANGELOG for phase closeout.

## Outcome

TASK-766 keeps the large historical examples as reference/design sketches instead of forcing broad parser relaxations or reducing them into misleading small placeholders.

Reference-only examples now carry visible `REFERENCE-ONLY` markers in the first five lines and are classified by `crates/ash-cli/tests/example_corpus_check.rs`:

- `examples/03-policies/01-role-based.ash`
- `examples/03-policies/02-time-based.ash`
- `examples/04-real-world/code-review.ash`
- `examples/04-real-world/customer-support.ash`
- `examples/simple_workflow.ash`
- `examples/support_ticket.ash`
- `examples/multi_agent_research.ash`
- `examples/workflows/40_tdd_workflow.ash`
- `examples/workflows/40a_tdd_concrete_example.ash`

The examples README files now document the executable conformance vs reference-only distinction. The harness requires every `examples/**/*.ash` file to be either expected-pass, expected-fail, or reference-only and verifies reference-only files have a visible marker.

Final Phase 107 corpus state:

| Corpus | Files | Passing | Expected failing | Reference-only |
|--------|-------|---------|------------------|----------------|
| `std/src/**/*.ash` | 39 | 34 | 5 | 0 |
| `examples/**/*.ash` | 36 | 27 | 0 | 9 |

Remaining std expected failures are accepted as explicitly documented follow-up gaps:

- `std/src/llm/conversation.ash`
- `std/src/llm/router.ash`
- `std/src/llm/supervised.ash`
- `std/src/llm/tool_agent.ash`
- `std/src/runtime/supervisor.ash`

## Verification Checklist

- [x] Every `examples/**/*.ash` file is classified by the corpus harness.
- [x] All conformance examples pass `ash check`.
- [x] Reference-only examples are clearly labeled and excluded by documented policy.
- [x] All `std/src/**/*.ash` files pass `ash check`, or any remaining exceptions are explicitly justified and accepted by the plan.
- [x] `cargo test --workspace` passes.
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes.
- [x] `cargo fmt --check` passes.
- [x] `cargo doc --workspace --no-deps` passes.
- [x] Independent phase audit completed and blockers addressed.
