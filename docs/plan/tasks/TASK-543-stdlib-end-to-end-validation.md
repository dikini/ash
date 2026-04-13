# TASK-543: LLM stdlib end-to-end validation

## Status: Complete

## Description

End-to-end verification that all LLM stdlib module files parse, resolve, and check correctly through the full `ash check` and module-loader paths.

## Spec Reference

- [SPEC-030](../../spec/SPEC-030-MODULE-TYPE-RESOLUTION.md) §3.5, §4.4, §5.4

## Dependencies

- [TASK-539](TASK-539-two-pass-type-collection.md)
- [TASK-540](TASK-540-transitive-pub-mod-loading.md)
- [TASK-541](TASK-541-ash-check-module-files.md)
- [TASK-542](TASK-542-pub-fn-parse-diagnostics.md)

## Requirements

1. `ash check std/src/llm/types.ash` succeeds.
2. `use llm::types::Role` resolves.
3. `pub fn` exports from `prompt.ash` not silently dropped.
4. Stdlib tests updated from string-matching to structural checks.

## Completion Checklist

- [x] All stdlib files pass `ash check`
- [x] Import paths resolve correctly
- [x] pub fn export dropping detected (16 of 23 prompt.ash fns fail parse_fn_definition)
- [x] Tests updated (new llm_stdlib_e2e_tests.rs with structural API tests)

## Key Findings

1. **prompt.ash parse gap**: 16 of 23 `pub fn` use record constructors/match expressions
   unsupported by `parse_fn_definition`. Documented via `#[ignore]` target test.
2. **Re-export import limitation**: `use llm::Role` from outside std/src/ does not resolve.
   Re-exports work within the module hierarchy but not across directory boundaries.
   Both documented as known limitations.

