# TASK-760: CLI Corpus Baseline Harness

## Status: 📝 Planned

## Description

Add a reproducible corpus harness for `ash check` over std and example `.ash` files. The harness must expose the current broken baseline honestly before fixes, distinguish expected-pass from expected-fail/reference files, and prevent future regressions in the modern conformance corpus.

## Specification Reference

- [PLAN-103](../PLAN-103-STDLIB-EXAMPLE-CORPUS-REPAIR.md)
- [SPEC-005](../../spec/SPEC-005-CLI.md)
- [SPEC-012](../../spec/SPEC-012-IMPORTS.md)

## Dependencies

- ✅ Phase 106 complete

## Requirements

1. Add a CLI-level test harness that exercises the same `ash check` path users run.
2. Record the baseline std and example pass/fail counts in test output or an auditable fixture.
3. Avoid masking CLI failures by using only `Engine::check_module_file`.
4. Provide an explicit way to classify files as expected-pass, expected-fail-with-reason, or reference-only.
5. Include Phase 105/106 examples in the expected-pass corpus.

## Files

- Create: `crates/ash-cli/tests/stdlib_corpus_check.rs`
- Create: `crates/ash-cli/tests/example_corpus_check.rs`
- Optional Create: `tests/fixtures/corpus/*.toml` or inline allowlist structures in tests

## TDD Steps

1. Write failing tests that assert all currently expected-pass files check through `ash-cli` command logic.
2. Include expected-fail assertions for the known broken files with reason strings.
3. Run the tests and confirm the broken baseline is visible.
4. Implement/adjust the harness only; do not fix parser/module behavior in this task.
5. Verify the harness reports 31/39 std passing and 19/36 examples passing, or update counts if upstream changed.

## Verification Checklist

- [ ] `cargo test -p ash-cli --test stdlib_corpus_check -- --nocapture` passes.
- [ ] `cargo test -p ash-cli --test example_corpus_check -- --nocapture` passes.
- [ ] Harness uses the CLI check path or equivalent command implementation, not only module-file registration.
- [ ] Known Phase 105/106 examples are expected-pass.
- [ ] `cargo fmt --check` passes.
- [ ] Independent review confirms the baseline is honest and not overfit.
