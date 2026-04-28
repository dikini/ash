# TASK-763: Runtime Args and LLM Loading Imports

## Status: 📝 Planned

## Description

Repair remaining std import/export drift around `runtime::Args` and `std/src/llm/loading.ash`. This task makes `examples/entrypoint_args.ash` and `llm/loading.ash` checkable once the module-loader substrate is fixed.

## Specification Reference

- [PLAN-103](../PLAN-103-STDLIB-EXAMPLE-CORPUS-REPAIR.md)
- [SPEC-012](../../spec/SPEC-012-IMPORTS.md)
- [SPEC-027](../../spec/SPEC-027-PURE-FUNCTIONS.md) where std pure functions are relevant

## Dependencies

- 📝 TASK-761: Stdlib Multiline Imports and Module Roots
- 📝 TASK-762: Stdlib Workflow Exports and Relative Imports

## Requirements

1. Make `runtime::Args` resolvable from `examples/entrypoint_args.ash`.
2. Ensure `RuntimeError` and `Args` re-exports from `std/src/runtime/mod.ash` are handled consistently.
3. Fix `std/src/llm/loading.ash` imports to use supported `io::path` / `io::fs` surfaces or add documented aliases if that is the intended std API.
4. Add tests that pin both std and example behavior.

## Files

- Modify: `std/src/runtime/mod.ash` or module-loader export handling as needed
- Modify: `std/src/llm/loading.ash`
- Test: `crates/ash-cli/tests/stdlib_corpus_check.rs`
- Test: `crates/ash-cli/tests/example_corpus_check.rs`
- Optional Test: `crates/ash-engine/tests/runtime_stdlib_reexports.rs`

## TDD Steps

1. Add failing test for `ash check examples/entrypoint_args.ash`.
2. Add failing test for `ash check std/src/llm/loading.ash`.
3. Fix re-export/import behavior or std source imports.
4. Verify affected std/example checks pass.

## Verification Checklist

- [ ] `cargo run -q -p ash-cli -- check examples/entrypoint_args.ash` passes.
- [ ] `cargo run -q -p ash-cli -- check std/src/llm/loading.ash` passes.
- [ ] Runtime std re-export tests pass.
- [ ] Corpus harness expected-pass list is updated.
- [ ] `cargo fmt --check` passes.
- [ ] Independent review confirms std API claims are honest.
