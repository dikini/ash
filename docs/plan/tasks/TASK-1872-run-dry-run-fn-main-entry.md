# TASK-1872: Run Dry-Run `fn main` Entry

**Status:** Complete
**Plan:** [PLAN-186](../PLAN-186-SURFACE-FUNCTION-CLI-ENTRY.md)

## Description

Make `ash run --dry-run` accept and typecheck ordinary target source files whose entry is `fn main` and which contain no `workflow` block.

## Requirements

- Add RED CLI test coverage for `ash run --dry-run` on a function-only `fn main` source.
- Add coverage that ordinary `ash run` still executes the same function-only source.
- Route dry-run through the same engine parsing/checking semantics used for ordinary files when the source is not an import-free runtime-entry workflow.
- Do not treat function-first entry sources as module-only just because they contain declarations such as `policy`, `role`, or `capability`.
- Reject declaration-only modules without a runnable `fn main` entry on `ash run --dry-run`; `ash check` remains the module validation command.
- Do not weaken invalid syntax or workflow-entry diagnostics.
- Do not introduce a new runtime mode or privileged workflow syntax.

## TDD Steps

1. RED: Add failing coverage for function-first `ash run --dry-run`.
2. GREEN: Update the CLI dry-run path to parse/check ordinary files through `engine.parse_file` and `engine.check`.
3. REGRESSION: Keep existing invalid syntax and workflow dry-run tests passing.

## Completion Checklist

- [x] RED captured and recorded.
- [x] GREEN captured and recorded.
- [x] Focused CLI tests pass.
- [x] `cargo fmt --check` passes.
- [x] Focused clippy passes for affected crates.
- [x] CHANGELOG.md updated.

## Evidence

- RED: `cargo test -p ash-cli fn_main_entry` failed before implementation; `test_dry_run_valid_fn_main_entry` reported `'main' has wrong return type` because dry-run forced runtime-entry workflow verification.
- GREEN: `cargo test -p ash-cli fn_main_entry` passed with 2/2 selected tests after ordinary dry-run sources used `engine.parse_file(path)` plus `engine.check`.
- Continuation RED: `cargo test -p ash-cli test_dry_run_fn_main_with_module_declaration_is_checked` failed because dry-run printed `Dry run successful` without checking a `fn main` source that also contained a module-level `policy` declaration.
- Continuation GREEN: `cargo test -p ash-cli test_dry_run_fn_main_with_module_declaration_is_checked` passed after module-only detection excluded token streams containing `fn main`.
- Missing-entry RED: `cargo test -p ash-cli test_dry_run_module_without_entry_is_rejected` failed because dry-run printed `Dry run successful` for a declaration-only module with no `fn main`.
- Missing-entry GREEN: `cargo test -p ash-cli test_dry_run_module_without_entry_is_rejected` passed after dry-run reported `entry file has no fn main or workflow`.
- Regression: `cargo test -p ash-cli commands::run::tests` passed with 14/14 tests.
- Verification: `cargo fmt --check` and `cargo clippy -p ash-cli --all-targets --all-features` passed.
