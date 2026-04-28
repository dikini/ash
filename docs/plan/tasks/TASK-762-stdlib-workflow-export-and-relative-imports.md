# TASK-762: Stdlib Workflow Exports and Relative Imports

## Status: 📝 Planned

## Description

Fix module export/import resolution for std workflows and relative imports. This should unblock imports of `dispatch::complete`, `dispatch::complete_with_tools`, and `super::...` paths used by runtime std modules.

## Specification Reference

- [PLAN-103](../PLAN-103-STDLIB-EXAMPLE-CORPUS-REPAIR.md)
- [SPEC-009](../../spec/SPEC-009-MODULES.md)
- [SPEC-012](../../spec/SPEC-012-IMPORTS.md)

## Dependencies

- 📝 TASK-760: CLI Corpus Baseline Harness
- 📝 TASK-761: Stdlib Multiline Imports and Module Roots

## Requirements

1. Decide and document whether imported workflows require `pub workflow` or whether plain workflows export from std modules.
2. Update module export collection to include the chosen workflow visibility form.
3. Resolve leading `super::`, and if appropriate `self::` / `crate::`, relative to the importing file/module.
4. Preserve existing absolute import behavior.
5. Add std/fixture tests that cover `std/src/runtime/supervisor.ash` and LLM dispatch consumers.

## Files

- Modify: `crates/ash-engine/src/module_loader.rs`
- Possibly Modify: `std/src/llm/dispatch.ash`
- Possibly Modify: `std/src/runtime/supervisor.ash`
- Test: `crates/ash-engine/tests/module_import_resolution_tests.rs`
- Test: `crates/ash-cli/tests/stdlib_corpus_check.rs`

## TDD Steps

1. Add failing tests for importing a workflow from another module.
2. Add failing tests for `use super::error::RuntimeError` style imports from a nested module.
3. Implement workflow export collection and relative import normalization.
4. Re-run affected std `ash check` commands.

## Verification Checklist

- [ ] `cargo run -q -p ash-cli -- check std/src/runtime/supervisor.ash` passes or reaches only a separately documented semantic issue.
- [ ] `cargo run -q -p ash-cli -- check std/src/llm/conversation.ash` passes or reaches only a separately documented semantic issue.
- [ ] `cargo run -q -p ash-cli -- check std/src/llm/router.ash` passes or reaches only a separately documented semantic issue.
- [ ] `cargo run -q -p ash-cli -- check std/src/llm/supervised.ash` passes or reaches only a separately documented semantic issue.
- [ ] `cargo run -q -p ash-cli -- check std/src/llm/tool_agent.ash` passes or reaches only a separately documented semantic issue.
- [ ] Targeted module import tests pass.
- [ ] `cargo clippy -p ash-engine -p ash-cli --all-targets --all-features -- -D warnings` passes.
- [ ] Independent review confirms relative paths do not break absolute std imports.
