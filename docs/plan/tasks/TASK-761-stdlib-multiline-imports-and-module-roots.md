# TASK-761: Stdlib Multiline Imports and Module Roots

## Status: 📝 Planned

## Description

Fix std module-loader behavior for multiline `use` declarations and `pub mod` / `pub use` module roots so files such as `std/src/llm/dispatch.ash` and `std/src/io/mod.ash` can pass the real `ash check` path.

## Specification Reference

- [PLAN-103](../PLAN-103-STDLIB-EXAMPLE-CORPUS-REPAIR.md)
- [SPEC-009](../../spec/SPEC-009-MODULES.md)
- [SPEC-012](../../spec/SPEC-012-IMPORTS.md)

## Dependencies

- 📝 TASK-760: CLI Corpus Baseline Harness

## Requirements

1. Collect multiline imports until their terminating semicolon before parsing ordinary imports.
2. Preserve existing single-line import behavior.
3. Ensure `std/src/io/mod.ash` succeeds through `ash-cli check` or the same command logic.
4. Add regression tests for nested multiline imports and re-export-only module roots.
5. Do not weaken import diagnostics for invalid imports.

## Files

- Modify: `crates/ash-engine/src/module_loader.rs`
- Possibly Modify: `crates/ash-cli/src/commands/check.rs`
- Test: `crates/ash-engine/tests/module_import_resolution_tests.rs` or adjacent existing module-loader tests
- Test: `crates/ash-cli/tests/stdlib_corpus_check.rs`

## TDD Steps

1. Add a failing fixture/test with:
   ```ash
   use types::{
       A,
       B
   };
   workflow main { done }
   ```
2. Add a failing check for `std/src/llm/dispatch.ash` that fails on `use types::{` before implementation.
3. Add a failing check for `std/src/io/mod.ash`.
4. Implement multiline import collection and module-root checking/fallback repair.
5. Re-run targeted tests and affected `ash check` commands.

## Verification Checklist

- [ ] `cargo run -q -p ash-cli -- check std/src/llm/dispatch.ash` no longer fails on `use types::{`.
- [ ] `cargo run -q -p ash-cli -- check std/src/io/mod.ash` passes.
- [ ] Targeted module import tests pass.
- [ ] `cargo check -p ash-engine -p ash-cli` passes.
- [ ] `cargo clippy -p ash-engine -p ash-cli --all-targets --all-features -- -D warnings` passes.
- [ ] `cargo fmt --check` passes.
- [ ] Independent review confirms no import-parser overreach.
