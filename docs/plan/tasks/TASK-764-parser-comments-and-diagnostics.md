# TASK-764: Parser Comments and Diagnostics

## Status: ✅ Complete

## Description

Align parser behavior with documented comment syntax by supporting `//` line comments, and improve diagnostics for common stale example syntax so users do not only see opaque `ContextError` failures.

## Specification Reference

- [PLAN-103](../PLAN-103-STDLIB-EXAMPLE-CORPUS-REPAIR.md)
- [SPEC-002](../../spec/SPEC-002-SURFACE.md)
- [SPEC-005](../../spec/SPEC-005-CLI.md)

## Dependencies

- ✅ TASK-760: CLI Corpus Baseline Harness

## Requirements

1. Support `//` line comments anywhere normal whitespace/comments are skipped.
2. Preserve existing `--` and `/* ... */` comment behavior.
3. Add tests for leading, trailing, and in-body `//` comments.
4. Add or improve diagnostics for common stale forms: `if cond {`, `for x in xs {`, `decide ... else`, `observe ... with`, and role-shaped `with role:`.
5. Ensure CLI parse errors include a useful source location/snippet or expectation instead of raw `ContextError` for targeted cases.

## Files

- Modify: `crates/ash-parser/src/parse_utils.rs`
- Possibly Modify: `crates/ash-parser/src/error.rs`
- Possibly Modify: `crates/ash-cli/src/commands/check.rs`
- Test: `crates/ash-parser/tests/comment_syntax.rs`
- Test: `crates/ash-cli/tests/check_parse_diagnostics.rs`

## TDD Steps

1. ✅ Add failing parser tests for `//` comments at file start, inside workflow, and trailing after statements.
2. ✅ Add failing CLI diagnostic tests asserting targeted invalid syntax does not render only `ContextError`.
3. ✅ Implement comment support.
4. ✅ Improve targeted parse error rendering without broad grammar relaxation.
5. ✅ Re-run example failures to see which move from comment failure to true syntax drift.

## Outcome

- `//` is now accepted as a line-comment prefix by the parser whitespace/comment skippers, including in workflow bodies and trailing after statements.
- The ordinary-file loader also treats leading `//` comment lines as prelude comments rather than the first non-import line.
- `ash check` now replaces raw `ContextError` output with targeted stale-syntax diagnostics for the TASK-764 cases: `if condition { ... }`, `for item in items { ... }`, `decide ... else`, `observe ... with`, and `with role:`. These diagnostics explicitly say the syntax is unsupported rather than accepting it.
- Corpus baselines remain honest after the diagnostic/comment change: std remains `34/39`, examples remain `20/36`.

## Verification Checklist

- [x] `cargo test -p ash-parser --test comment_syntax -- --nocapture` passes.
- [x] `cargo test -p ash-cli --test check_parse_diagnostics -- --nocapture` passes.
- [x] Existing parser tests pass via `cargo test -p ash-parser --lib -- --nocapture`.
- [x] `cargo clippy -p ash-parser -p ash-cli --all-targets --all-features -- -D warnings` passes as part of `cargo clippy -p ash-parser -p ash-engine -p ash-cli --all-targets --all-features -- -D warnings`.
- [x] Independent review confirms diagnostics do not claim unsupported syntax is valid.
