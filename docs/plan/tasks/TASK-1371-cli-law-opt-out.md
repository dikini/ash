# TASK-1371: CLI — `--skip-law-tests` and `--skip-law-test=<name>`

## Status: ✅ Complete

## Description

Add opt-out flags for law testing.

## Requirements

1. Add `--skip-law-tests` CLI flag (skips all law tests)
2. Add `--skip-law-test=<name>` CLI flag (skips specific law by name)
3. Skip law test generation when opted out
4. Document in CLI help

## Acceptance Criteria

- [x] `--skip-law-tests` skips all law tests
- [x] `--skip-law-test=<name>` skips specific law
- [x] CLI test passes
- [x] No regressions

## Verification

- `cargo test -p ash-cli test_runner::executor::tests::run_suite_skip_law -- --nocapture` — 2 passed
- `cargo test -p ash-cli --test test_command test_help_output -- --nocapture` — 1 passed
- `cargo fmt --check` — passed
- `cargo clippy -p ash-cli --all-targets --all-features -- -D warnings` — passed
- `cargo check --workspace` — passed
- `git diff --check` — passed

## Completion Notes

- Added `ash test --skip-law-tests` to drop all law-derived synthesized rows after source selection.
- Added repeatable `ash test --skip-law-test=<name>` filtering by declared law name, while also accepting exact generated row names.
- The filter is law-specific: contract/policy/obligation synthesized rows and authored rows are unaffected.

## Related

- [PLAN-136](../PLAN-136-INTERFACE-LAW-SYNTAX.md)
- [TASK-1369](TASK-1369-runner-synthetic-test-generation.md)
