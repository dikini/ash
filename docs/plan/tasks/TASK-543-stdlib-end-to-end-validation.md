# TASK-543: LLM stdlib end-to-end validation

## Status: Draft (v2)

## Description

End-to-end verification that all LLM stdlib module files parse, resolve, and check correctly through the full `ash check` and module-loader paths.

## Spec Reference

- [SPEC-030](../../spec/SPEC-030-MODULE-TYPE-RESOLUTION.md) §3.5, §4.4, §5.4

## Dependencies

- [TASK-539](TASK-539-two-pass-type-collection.md)
- [TASK-540](TASK-540-transitive-pub-mod-loading.md)
- [TASK-541](TASK-541-ash-check-module-files.md)
- [TASK-542](TASK-542-llm-stdlib-end-to-end.md)

## Requirements

1. `ash check std/src/llm/types.ash` succeeds.
2. `use llm::types::Role` resolves.
3. `pub fn` exports from `prompt.ash` not silently dropped.
4. Stdlib tests updated from string-matching to structural checks.

## Completion Checklist

- [ ] All stdlib files pass `ash check`
- [ ] Import paths resolve correctly
- [ ] pub fn export dropping detected
- [ ] Tests updated

