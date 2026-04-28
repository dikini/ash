# TASK-764: Parser Comments and Diagnostics

## Status: 📝 Planned

## Description

Align parser behavior with documented comment syntax by supporting `//` line comments, and improve diagnostics for common stale example syntax so users do not only see opaque `ContextError` failures.

## Specification Reference

- [PLAN-103](../PLAN-103-STDLIB-EXAMPLE-CORPUS-REPAIR.md)
- [SPEC-002](../../spec/SPEC-002-SURFACE.md)
- [SPEC-005](../../spec/SPEC-005-CLI.md)

## Dependencies

- 📝 TASK-760: CLI Corpus Baseline Harness

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

1. Add failing parser tests for `//` comments at file start, inside workflow, and trailing after statements.
2. Add failing CLI diagnostic tests asserting targeted invalid syntax does not render only `ContextError`.
3. Implement comment support.
4. Improve targeted parse error rendering without broad grammar relaxation.
5. Re-run example failures to see which move from comment failure to true syntax drift.

## Verification Checklist

- [ ] `cargo test -p ash-parser --test comment_syntax -- --nocapture` passes.
- [ ] `cargo test -p ash-cli --test check_parse_diagnostics -- --nocapture` passes.
- [ ] Existing parser tests pass.
- [ ] `cargo clippy -p ash-parser -p ash-cli --all-targets --all-features -- -D warnings` passes.
- [ ] Independent review confirms diagnostics do not claim unsupported syntax is valid.
