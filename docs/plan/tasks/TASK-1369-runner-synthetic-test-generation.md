# TASK-1369: Synthetic tests — generate small-world tests from laws

## Status: ✅ Complete

## Description

For each law without a `proof` block, generate small-world tests using SPEC-077 runner framework.

## Requirements

1. Generate test cases from law parameters
2. Use small-world generators for parameter types
3. Assert law proposition for each generated case
4. Report failures with seed and counterexample

## Acceptance Criteria

- [x] Tests generate for unproven laws
- [x] Tests pass for valid laws
- [x] Tests fail for broken laws with counterexample
- [x] Runner test passes
- [x] No regressions

## Verification

- `cargo test -p ash-cli test_runner::synthesized::tests::extract_laws_ -- --nocapture` — 5 passed
- `cargo test -p ash-cli test_runner::synthesized::tests::law_smallworld_generation -- --nocapture` — 4 passed
- `cargo test -p ash-cli test_runner::executor::tests::run_suite_executes_structured_snapshot -- --nocapture` — 3 passed
- `cargo fmt --check` — passed
- `cargo clippy -p ash-cli --all-targets --all-features -- -D warnings` — passed
- `cargo check --workspace` — passed
- `git diff --check` — passed

## Completion Notes

- Added `TestSource::Law` and law-sourced small-world result generation from `RunnerIntrospectionSnapshot.laws`.
- Supported deterministic finite parameter worlds for `Int`, `Bool`, and `String` law parameters.
- Omitted laws with matching `proof` declarations from fallback synthetic law tests; proof verification remains later-stage work.
- Added explicit `laws` synthesized-source selection so contract-only synthesized runs do not leak law rows.
- Applied a small deterministic default cap for uncapped law parameter products and handled zero-parameter laws as a single empty binding world.
- Generated law cases execute simple boolean propositions over generated bindings and emit `SmallWorld` results.
- Broken laws report failing case, seed, world index, and counterexample/repro metadata.
- This task does not add `by test` delegation, CLI law skip flags, law-result caching, proof verification, or full expression evaluation for arbitrary law propositions.

## Related

- [PLAN-136](../PLAN-136-INTERFACE-LAW-SYNTAX.md)
- [TASK-1368](TASK-1368-runner-law-extraction.md)
- [SPEC-077](../../spec/SPEC-077-ASH-TEST-RUNNER-SYNTHESIZED-AND-SMALLWORLD-COMPLETION.md)
