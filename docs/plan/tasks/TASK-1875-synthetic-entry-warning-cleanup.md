# TASK-1875: Synthetic Entry Warning Cleanup

**Status:** Complete
**Plan:** [PLAN-186](../PLAN-186-SURFACE-FUNCTION-CLI-ENTRY.md)

## Description

Prevent function-first target entry files from emitting legacy workflow deprecation warnings that originate from the internal `fn main` runtime adapter.

## Requirements

- Add RED CLI coverage proving `ash check` on a `fn main` source with no `workflow` block does not emit `DeprecatedLegacyWorkflowDeclaration`.
- Preserve existing warning behavior for user-authored legacy workflow syntax.
- Fix the warning source in the engine rather than filtering warnings only in CLI output.
- Do not introduce a second Core semantic path.

## TDD Steps

1. RED: Add failing CLI or engine coverage for a function-first source that currently emits the legacy workflow warning.
2. GREEN: Track whether `parse_program_with_functions` returned a user-authored workflow or a synthetic `fn main` adapter and only emit workflow deprecation warnings for user-authored workflow definitions.
3. REGRESSION: Keep legacy workflow warning tests passing.

## Completion Checklist

- [x] RED captured and recorded.
- [x] GREEN captured and recorded.
- [x] Legacy workflow warning regressions pass.
- [x] Focused CLI/engine tests pass.
- [x] CHANGELOG.md updated if wording needs expansion.

## Evidence

- RED: `cargo test -p ash-cli ash_check_function_first_entry_does_not_emit_legacy_workflow_warning` failed because `ash check` printed `DeprecatedLegacyWorkflowDeclaration` for a function-first source.
- GREEN: `cargo test -p ash-cli --test task_778_legacy_workflow_warning` passed with 4/4 tests after `parse_program_with_functions` returned entry provenance and `Engine::parse` emitted legacy workflow warnings only for user-authored workflow entries.
