# TASK-1370: Synthetic tests — `by test` delegation syntax

## Status: ✅ Complete

## Description

Support `proof ... { by test "test_name" }` syntax as explicit synthetic law-test delegation.

## Requirements

1. Parse `by test` proof body
2. Extract delegated test identity from matching proof declarations
3. Delegate matching laws to the synthetic test runner instead of treating `by test` as a hand proof
4. Preserve delegation metadata in runner output/repro artifacts
5. Leave `.ash/law-cache.toml` result caching to TASK-1372

## Acceptance Criteria

- [x] `by test` parses correctly
- [x] Delegated test identity extracted and passed to runner
- [x] Results carry delegation metadata for reproducibility
- [x] Test passes
- [x] No regressions

## Verification

- `cargo test -p ash-cli test_runner::synthesized::tests::extract_laws_ -- --nocapture` — 7 passed
- `cargo test -p ash-cli test_runner::synthesized::tests::law_smallworld_generation -- --nocapture` — 5 passed
- `cargo test -p ash-cli test_runner::executor::tests::run_suite_executes_structured_snapshot -- --nocapture` — 3 passed
- `cargo fmt --check` — passed
- `cargo clippy -p ash-cli --all-targets --all-features -- -D warnings` — passed
- `cargo check --workspace` — passed
- `git diff --check` — passed

## Completion Notes

- Added `RunnerLawMetadata::delegated_test` for `proof ... { by test "..." }` declarations.
- Scoped proof handling now treats `by_definition`/expression proofs as hand proofs that suppress fallback law checks, while `by test` proofs keep the law in runner metadata with a delegated test name.
- Propagated delegated test identity into law small-world repro oracle snapshots.
- Parser support for the current `by test "name"` body already existed from TASK-1362/TASK-1363; structured `by test { ... }` configuration and cache storage remain future work owned by TASK-1372 and later runner extensions.

## Related

- [PLAN-136](../PLAN-136-INTERFACE-LAW-SYNTAX.md)
- [TASK-1369](TASK-1369-runner-synthetic-test-generation.md)
