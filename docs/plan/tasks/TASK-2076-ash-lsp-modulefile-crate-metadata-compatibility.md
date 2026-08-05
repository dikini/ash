# TASK-2076: Ash LSP ModuleFile Crate-Metadata Compatibility

**Status:** Complete
**Phase:** Maintenance compatibility
**Owned rule:** Keep `ash-lsp-core` test construction aligned with the public `ash_parser::ModuleFile` shape.

## Description

Update the `ash-lsp-core` test helper that constructs an empty `ModuleFile` so it initializes the
parser's crate-root metadata carrier explicitly. The compatibility value is absent metadata; this
does not add LSP crate-root semantics.

## Requirements

1. Initialize `ModuleFile::crate_metadata` with `None` in the affected LSP test helper.
2. Do not change parser behavior, LSP symbol behavior, or crate-metadata semantics.
3. Keep the change isolated to `crates/ash-lsp-core/src/symbols.rs` and this task evidence.

## TDD Steps

1. Reproduce the test-target compile error for the missing `crate_metadata` field.
2. Add the explicit absent-metadata initializer.
3. Run the `ash-lsp-core` library tests and confirm they pass.

## Completion Checklist

- [x] `ModuleFile::crate_metadata` is initialized as `None` in the empty-module test helper.
- [x] `cargo test -p ash-lsp-core --lib` passes (78 tests).
- [x] No parser or LSP production behavior was changed.
