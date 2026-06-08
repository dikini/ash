# TASK-1374: Integration — module-scoped law in `std::io::path`

## Status: ✅ Complete

## Description

Add module-scoped `law` to `std::io::path` and verify.

## Requirements

1. Add `join_preserves_absolute` law to `std/src/io/path.ash`
2. Verify parser accepts
3. Verify typechecker passes
4. Verify synthetic tests generate

## Acceptance Criteria

- [x] Module law added to `std::io::path`
- [x] Full pipeline works
- [x] Integration test passes
- [x] No regressions

## Implementation Notes

- Added pure helper `preserves_absolute_after_join(base, child) -> Bool` so the law proposition remains a single helper call and the synthetic runner preserves the complete proposition string.
- Added module-scoped law `join_preserves_absolute(base: PathBuf, child: String)` to `std/src/io/path.ash`.
- The law is intentionally one-way: if `base` is absolute, then `join(base, child)` is absolute. It does not claim equivalence between `is_absolute(base)` and `is_absolute(join(base, child))`.
- Fixed law-deferred repro metadata so generated law rows replay with `--only-synthesized laws` rather than the generic contract/policy/obligation source set.

## Verification

- RED: `cargo test -p ash-parser --test stdlib_parsing test_io_path_join_preserves_absolute_law_parses -- --nocapture` failed before the law was added.
- RED: `cargo test -p ash-engine --test task_1374_stdlib_path_law -- --nocapture` failed before the law was added.
- RED: `cargo test -p ash-cli --test test_command only_synthesized_laws_generates_std_io_path_join_law_row -- --nocapture` failed before the law was added, then failed again until proposition metadata preserved the helper-call property.
- RED: the same CLI test failed on the reviewed replay-command issue until law rows replayed with `--only-synthesized laws`.
- `cargo test -p ash-parser --test stdlib_parsing test_io_path_join_preserves_absolute_law_parses -- --nocapture` — 1 passed.
- `cargo test -p ash-engine --test task_1374_stdlib_path_law -- --nocapture` — 2 passed.
- `cargo test -p ash-cli --test test_command only_synthesized_laws_generates_std_io_path_join_law_row -- --nocapture` — 1 passed.
- `cargo run -p ash-cli -- check std/src/io/path.ash` — `[OK] std/src/io/path.ash: OK (module file: 1 type(s), 7 fn(s))`.
- `cargo run -p ash-cli -- test std/src/io/path.ash --only-synthesized laws --format json` — emitted one deferred `synthesized:law` row for `join_preserves_absolute` with params `base: PathBuf`, `child: String`, proposition `preserves_absolute_after_join(base, child)`, and replay command `ash test std/src/io/path.ash --only-synthesized laws --seed 0`.
- `cargo fmt --check` — passed.
- `cargo check --workspace` — passed.
- `cargo clippy -p ash-parser -p ash-engine -p ash-cli --all-targets --all-features -- -D warnings` — passed.
- `git diff --check` — passed.

## Related

- [PLAN-136](../PLAN-136-INTERFACE-LAW-SYNTAX.md)
- [TASK-1361](TASK-1361-parser-law-module-scope.md)
- [TASK-1364](TASK-1364-typeck-law-name-checking.md)
- [TASK-1369](TASK-1369-runner-synthetic-test-generation.md)
